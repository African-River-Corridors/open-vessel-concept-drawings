//! The `.vdesign` file: a Design holding a named list of Options and the engine version, as
//! versioned JSON, in SI units except speeds, which are in knots (ADR 0007).
//!
//! Every input is stored **as used**, with a marker saying where it came from:
//!
//! ```json
//! "block_coefficient": { "value": 0.85, "source": "default" }
//! ```
//!
//! `source` is `typed` (the designer gave it) or `default` (the Defaults Profile gave it). A bare
//! value in a hand-written file counts as typed. A stored input is never replaced by a profile
//! value, so a later profile change never changes a saved file.
//!
//! A file must hold every input that applies to its Cargo Containment. [`check`] refuses a file
//! that leaves one out, because the value would come from whatever Defaults Profile is loaded
//! today, and a profile edit would change what the file means. [`fill`] writes the left-out
//! inputs in, as used and marked `default`, so a designer can still write only the required six.
//!
//! The `units` field holds `"SI"` and means: SI, except `max_speed_kn` and `channel_current_kn`,
//! which are in knots (the inland-waterway convention, ADR 0007).
//!
//! Validation lives here only. The core copies Python exactly and adds no checks. Besides the
//! range table, validation refuses inputs the original Python turns into a physically impossible
//! result (see [`DRY_BULK_WING_VOID_M`]).

use crate::profile::Profile;
use serde::de::{DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value, json};
use std::collections::HashSet;
use std::fmt;

/// The `.vdesign` format version this tool writes. Version 2 changed the gas tank inputs (issue
/// #175: `adn_tank_cap_m3` and the `ADN_G` ruleset went, `max_tank_od_m` came). A later change to
/// the input table that needs the same kind of reading step reuses version 2 only while it is
/// unreleased; see ADR 0007.
pub const FORMAT_VERSION: u64 = 2;

/// The oldest `.vdesign` format version this tool still reads. A version 1 file is changed to
/// the current inputs as it is read, with a warning for each change (see [`migrate_v1_option`]).
pub const OLDEST_FORMAT_VERSION: u64 = 1;

/// The engine version written into every new Design. One version covers every `ovcd-*` crate.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Cargo Containments, and the Cargo phases each can carry (as in the Python `simulate`).
pub const CARGO_CONTAINMENTS: &[(&str, &str)] = &[
    ("box_hold", "dry_bulk"),
    ("sloped_hopper_hold", "dry_bulk"),
    ("type_c_tanks", "liquid_gas"),
];

/// Cargo phases, as the Python cargo database names them.
pub const CARGO_PHASES: &[&str] = &["dry_bulk", "liquid_gas"];

/// Inputs whose default is chosen by another value: `(input, by, keys)`. The Defaults Profile
/// must give `default_by_<by>[key]` for every key, because each is a lookup the schema can make.
pub const KEYED_DEFAULTS: &[(&str, &str, &[&str])] = &[
    ("cargo_containment", "cargo_phase", CARGO_PHASES),
    (
        "lightship_factor_t_per_m3",
        "cargo_containment",
        &["box_hold"],
    ),
    ("fill_ratio", "ruleset", RULESETS),
];

/// Hull Forms that exist today.
pub const HULL_FORMS: &[&str] = &["raked_box"];

/// The deepest hull the gas depth search (`optimise_depth`) tries, in m. It is fixed in the core
/// (`DEFAULT_DEPTHS_M`, 3.5 to 7.0 m).
pub const DEPTH_SEARCH_MAX_M: f64 = 7.0;

/// The dry-bulk hold section, copied from the `barge_capacity` defaults (now inputs; these are
/// the values older files get): each side has a wing void of this width, in m. The hold width is
/// `beam - 2 * wing void`.
///
/// The rule (issue #41): the original Python returns a negative cargo mass and draught when the
/// hold width or hold height is negative, so validation refuses those inputs. It does not set a
/// minimum beam or depth; it refuses the impossible result. A hold of exactly zero gives zero
/// cargo, which the engine reports itself as a failed check, so it passes.
pub const DRY_BULK_WING_VOID_M: f64 = 1.0;

/// The dry-bulk double bottom height, in m (`barge_capacity` default).
pub const DRY_BULK_DOUBLE_BOTTOM_M: f64 = 1.2;

/// The dry-bulk hatch coaming height, in m (`barge_capacity` default). The hold height is
/// `(depth - double bottom) + coaming`, in that order, as the Python computes it.
pub const DRY_BULK_COAMING_M: f64 = 1.0;

/// The dry-bulk hopper slope, horizontal over vertical (`barge_capacity` default).
pub const DRY_BULK_HOPPER_SLOPE: f64 = 1.0;
/// The number of dry-bulk holds (`barge_capacity` default).
pub const DRY_BULK_N_HOLDS: f64 = 3.0;
/// The dry-bulk hold fill ratio (`barge_capacity` default).
pub const DRY_BULK_HOLD_FILL: f64 = 0.95;
/// Plate thicknesses of the sloped hopper hold, in mm (`barge_capacity` defaults).
pub const DRY_BULK_INNER_BOTTOM_PLATE_MM: f64 = 14.0;
pub const DRY_BULK_HOPPER_PLATE_MM: f64 = 14.0;
pub const DRY_BULK_BULKHEAD_PLATE_MM: f64 = 11.0;
pub const DRY_BULK_COAMING_PLATE_MM: f64 = 10.0;

/// The bow rake length as a multiple of the moulded depth (`barge_capacity` default).
pub const BOW_RAKE_TO_DEPTH_RATIO: f64 = 1.6;
/// The stern void of a pushed barge, in m (`barge_capacity` default).
pub const PUSHED_STERN_VOID_M: f64 = 3.0;
/// The dry-bulk end-bulkhead allowance at each end of the hold, in m (`barge_capacity` default).
pub const BULK_END_BULKHEAD_M: f64 = 0.5;

/// The gas tank arrangement values fixed in the engine before they were inputs (`barge_capacity`
/// and `models` defaults).
pub const GAS_TANKS_IN_LINE: f64 = 2.0;
pub const GAS_TANK_SUPPORT_HEIGHT_M: f64 = 0.5;
pub const GAS_TANK_DOME_HEIGHT_M: f64 = 2.0;
/// The dome diameter (issue #237). Unlike the others it is NOT a value the engine used before: the
/// engine had no dome in Vc. A file that lacks it gets this value, and its results may move.
pub const GAS_TANK_DOME_DIAMETER_M: f64 = 2.0;
pub const GAS_WING_TANK_GAP_M: f64 = 0.6;
pub const GAS_WING_TANK_HEIGHT_RATIO: f64 = 0.66;
pub const GAS_TANK_WALL_THICKNESS_MM: f64 = 25.0;

/// The water density of the run, in t/m3: fresh water (`barge_capacity`, `gas_stability` and
/// `journey_derating` default).
pub const WATER_DENSITY_T_PER_M3: f64 = 1.0;
/// The under-keel clearance kept below the squatted keel, in m (`resistance` default).
pub const SQUAT_SAFETY_MARGIN_M: f64 = 0.15;

/// What the cost rates are, shown with the group and in the capital budget.
pub const COST_RATES_BASIS: &str = "Example rates \u{2014} replace with yours.";

/// Both dry-bulk holds.
const DRY_HOLDS: &[&str] = &["box_hold", "sloped_hopper_hold"];

/// An input that was a fixed value in the engine before it became an input.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegacyFixed {
    pub name: &'static str,
    pub applies_to: AppliesTo,
    /// The Formation kinds it applies to; `None` for every Formation.
    pub formations: Option<&'static [&'static str]>,
    /// The value the engine used for every design.
    pub value: f64,
}

/// The fixed values behind the inputs that came after the first `.vdesign` files. A file that
/// lacks one of these inputs (version 1, or a version 2 file written by a test build) is read with
/// this value, marked `default`, so its results cannot change. The value is held here, not read
/// from the Defaults Profile: a later edit of the profile must not move an old file. A test holds
/// each value equal to the profile default and to the core default.
///
/// A profile that has no entry for one of these inputs takes the built-in profile's entry.
pub const LEGACY_FIXED_INPUTS: &[LegacyFixed] = &[
    LegacyFixed {
        name: "hold_wing_void_width_m",
        applies_to: AppliesTo::AnyOf(DRY_HOLDS),
        formations: None,
        value: DRY_BULK_WING_VOID_M,
    },
    LegacyFixed {
        name: "hold_double_bottom_m",
        applies_to: AppliesTo::AnyOf(DRY_HOLDS),
        formations: None,
        value: DRY_BULK_DOUBLE_BOTTOM_M,
    },
    LegacyFixed {
        name: "hopper_slope_h_per_v",
        applies_to: AppliesTo::Only("sloped_hopper_hold"),
        formations: None,
        value: DRY_BULK_HOPPER_SLOPE,
    },
    LegacyFixed {
        name: "hatch_coaming_height_m",
        applies_to: AppliesTo::AnyOf(DRY_HOLDS),
        formations: None,
        value: DRY_BULK_COAMING_M,
    },
    LegacyFixed {
        name: "n_holds",
        applies_to: AppliesTo::Only("sloped_hopper_hold"),
        formations: None,
        value: DRY_BULK_N_HOLDS,
    },
    LegacyFixed {
        name: "hold_fill_ratio",
        applies_to: AppliesTo::AnyOf(DRY_HOLDS),
        formations: None,
        value: DRY_BULK_HOLD_FILL,
    },
    LegacyFixed {
        name: "inner_bottom_plate_mm",
        applies_to: AppliesTo::Only("sloped_hopper_hold"),
        formations: None,
        value: DRY_BULK_INNER_BOTTOM_PLATE_MM,
    },
    LegacyFixed {
        name: "hopper_plate_mm",
        applies_to: AppliesTo::Only("sloped_hopper_hold"),
        formations: None,
        value: DRY_BULK_HOPPER_PLATE_MM,
    },
    LegacyFixed {
        name: "hold_bulkhead_plate_mm",
        applies_to: AppliesTo::Only("sloped_hopper_hold"),
        formations: None,
        value: DRY_BULK_BULKHEAD_PLATE_MM,
    },
    LegacyFixed {
        name: "hatch_coaming_plate_mm",
        applies_to: AppliesTo::Only("sloped_hopper_hold"),
        formations: None,
        value: DRY_BULK_COAMING_PLATE_MM,
    },
    LegacyFixed {
        name: "bow_rake_length_to_depth_ratio",
        applies_to: AppliesTo::All,
        formations: None,
        value: BOW_RAKE_TO_DEPTH_RATIO,
    },
    LegacyFixed {
        name: "pushed_stern_void_m",
        applies_to: AppliesTo::All,
        formations: Some(PUSHED),
        value: PUSHED_STERN_VOID_M,
    },
    LegacyFixed {
        name: "bulk_end_bulkhead_m",
        applies_to: AppliesTo::AnyOf(DRY_HOLDS),
        formations: None,
        value: BULK_END_BULKHEAD_M,
    },
    LegacyFixed {
        name: "tanks_in_line",
        applies_to: AppliesTo::Only("type_c_tanks"),
        formations: None,
        value: GAS_TANKS_IN_LINE,
    },
    LegacyFixed {
        name: "tank_support_height_m",
        applies_to: AppliesTo::Only("type_c_tanks"),
        formations: None,
        value: GAS_TANK_SUPPORT_HEIGHT_M,
    },
    LegacyFixed {
        name: "tank_dome_height_m",
        applies_to: AppliesTo::Only("type_c_tanks"),
        formations: None,
        value: GAS_TANK_DOME_HEIGHT_M,
    },
    LegacyFixed {
        name: "tank_dome_diameter_m",
        applies_to: AppliesTo::Only("type_c_tanks"),
        formations: None,
        value: GAS_TANK_DOME_DIAMETER_M,
    },
    LegacyFixed {
        name: "wing_tank_gap_m",
        applies_to: AppliesTo::Only("type_c_tanks"),
        formations: None,
        value: GAS_WING_TANK_GAP_M,
    },
    LegacyFixed {
        name: "wing_tank_height_ratio",
        applies_to: AppliesTo::Only("type_c_tanks"),
        formations: None,
        value: GAS_WING_TANK_HEIGHT_RATIO,
    },
    LegacyFixed {
        name: "tank_wall_thickness_mm",
        applies_to: AppliesTo::Only("type_c_tanks"),
        formations: None,
        value: GAS_TANK_WALL_THICKNESS_MM,
    },
    LegacyFixed {
        name: "water_density_t_per_m3",
        applies_to: AppliesTo::All,
        formations: None,
        value: WATER_DENSITY_T_PER_M3,
    },
    LegacyFixed {
        name: "squat_safety_margin_m",
        applies_to: AppliesTo::All,
        formations: None,
        value: SQUAT_SAFETY_MARGIN_M,
    },
    LegacyFixed {
        name: "hull_steel_rate",
        applies_to: AppliesTo::All,
        formations: None,
        value: 2500.0,
    },
    LegacyFixed {
        name: "tank_shell_rate",
        applies_to: AppliesTo::All,
        formations: None,
        value: 10000.0,
    },
    LegacyFixed {
        name: "tank_supports_rate",
        applies_to: AppliesTo::All,
        formations: None,
        value: 2000.0,
    },
    LegacyFixed {
        name: "insulation_rate",
        applies_to: AppliesTo::All,
        formations: None,
        value: 4000.0,
    },
    LegacyFixed {
        name: "piping_rate",
        applies_to: AppliesTo::All,
        formations: None,
        value: 4500.0,
    },
    LegacyFixed {
        name: "outfitting_rate",
        applies_to: AppliesTo::All,
        formations: None,
        value: 4000.0,
    },
    LegacyFixed {
        name: "cargo_system_lump",
        applies_to: AppliesTo::All,
        formations: None,
        value: 500000.0,
    },
    LegacyFixed {
        name: "paint_anodes_misc",
        applies_to: AppliesTo::All,
        formations: None,
        value: 100000.0,
    },
    LegacyFixed {
        name: "battery_rate_per_kwh",
        applies_to: AppliesTo::All,
        formations: None,
        value: 250.0,
    },
    LegacyFixed {
        name: "propulsion_rate_per_kw",
        applies_to: AppliesTo::All,
        formations: None,
        value: 1000.0,
    },
    LegacyFixed {
        name: "wheelhouse_accom_lump",
        applies_to: AppliesTo::All,
        formations: None,
        value: 450000.0,
    },
    LegacyFixed {
        name: "tug_outfitting_lump",
        applies_to: AppliesTo::All,
        formations: None,
        value: 200000.0,
    },
    LegacyFixed {
        name: "yard_margin_fraction",
        applies_to: AppliesTo::All,
        formations: None,
        value: 0.1,
    },
    LegacyFixed {
        name: "delivery_lump",
        applies_to: AppliesTo::All,
        formations: None,
        value: 200000.0,
    },
];

/// Gas tank rulesets.
///
/// One rule set: `IGC_2G` is the per-tank d(Vc) of IGC 2.4.1.1, measured plate to plate (2.4.2).
/// A file that names the old `IGC_2G_EXACT` is read as `IGC_2G` (see [`migrate_ruleset`]).
pub const RULESETS: &[&str] = &["IGC_2G"];

/// The Formation kinds of a pushed barge or convoy: the ones with no stern block of their own.
pub const PUSHED: &[&str] = &["pushed_barge", "pushed_convoy"];

/// Whether an input used for `formations` applies to a Formation of `kind`. An unknown kind
/// (none given yet) counts as applying, so nothing is dropped before the Formation is known.
pub fn formation_includes(formations: Option<&[&str]>, kind: Option<&str>) -> bool {
    match (formations, kind) {
        (Some(list), Some(k)) => list.contains(&k),
        _ => true,
    }
}

/// Formation kinds. Only the first two are computed in spec 0001.
pub const FORMATION_KINDS: &[&str] = &["self_propelled", "pushed_barge", "pushed_convoy"];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputKind {
    /// A float. `nullable` inputs take `null` for "none".
    Number {
        nullable: bool,
    },
    Bool,
    /// One of a fixed list of names.
    Choice(&'static [&'static str]),
    /// A key of the built-in cargo database.
    Cargo,
    /// `{ "kind": ..., "abreast": n, "in_line": n }`.
    Formation,
    /// A scheme of the Vessel Class catalogue (a class `family`), or null.
    VesselScheme,
    /// A Vessel Class id (a pickable class; an old id of the core is read as its class), or null.
    VesselClass,
}

/// Which Cargo Containments an input is used for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppliesTo {
    All,
    Only(&'static str),
    /// Used for each of these Cargo Containments.
    AnyOf(&'static [&'static str]),
}

impl AppliesTo {
    /// Whether an input with this scope is used for the Cargo Containment `c`.
    pub fn includes(self, c: &str) -> bool {
        match self {
            AppliesTo::All => true,
            AppliesTo::Only(only) => only == c,
            AppliesTo::AnyOf(list) => list.contains(&c),
        }
    }

    /// The Cargo Containments the input is used for, in the order of `CARGO_CONTAINMENTS`.
    pub fn containments(self) -> Vec<&'static str> {
        CARGO_CONTAINMENTS
            .iter()
            .map(|(c, _)| *c)
            .filter(|c| self.includes(c))
            .collect()
    }
}

/// The group an input belongs to in a form. The group is part of the input table, so a front end
/// takes it from the input schema and holds no list of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Required,
    VesselClass,
    HullAndContainment,
    GasTanks,
    DryBulkHold,
    WaterwayLimits,
    SpeedAndPower,
    CostRates,
}

impl Group {
    /// Every group, in the order a form shows them.
    pub const ALL: [Group; 8] = [
        Group::Required,
        Group::VesselClass,
        Group::HullAndContainment,
        Group::GasTanks,
        Group::DryBulkHold,
        Group::WaterwayLimits,
        Group::SpeedAndPower,
        Group::CostRates,
    ];

    /// The group's display name.
    pub fn label(self) -> &'static str {
        match self {
            Group::Required => "Required",
            Group::VesselClass => "Vessel Class",
            Group::HullAndContainment => "Hull and containment",
            Group::GasTanks => "Gas tanks",
            Group::DryBulkHold => "Dry-bulk hold",
            Group::WaterwayLimits => "Waterway Limits",
            Group::SpeedAndPower => "Speed and power",
            Group::CostRates => "Cost rates",
        }
    }

    /// A note on the group's values that a form shows under its name, if it has one.
    pub fn note(self) -> Option<&'static str> {
        match self {
            Group::CostRates => Some(COST_RATES_BASIS),
            _ => None,
        }
    }
}

/// Display names of the choices of the choice inputs (a value not listed shows as itself).
pub const CHOICE_LABELS: &[(&str, &str)] = &[
    ("box_hold", "Box hold"),
    ("sloped_hopper_hold", "Sloped hopper hold"),
    ("type_c_tanks", "Type C tanks"),
    ("raked_box", "Raked box"),
    ("self_propelled", "Self-propelled"),
    ("pushed_barge", "Pushed barge"),
    ("pushed_convoy", "Pushed convoy"),
];

/// The display name of a choice value.
pub fn choice_label(value: &str) -> &str {
    CHOICE_LABELS
        .iter()
        .find(|(v, _)| *v == value)
        .map_or(value, |(_, l)| l)
}

/// One input of an Option.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InputSpec {
    pub name: &'static str,
    /// The name in plain words, for a form.
    pub label: &'static str,
    pub kind: InputKind,
    pub required: bool,
    pub applies_to: AppliesTo,
    /// The Formation kinds the input is used for; `None` for every Formation.
    pub formations: Option<&'static [&'static str]>,
    pub group: Group,
}

const fn spec(
    name: &'static str,
    label: &'static str,
    kind: InputKind,
    required: bool,
    group: Group,
) -> InputSpec {
    InputSpec {
        name,
        label,
        kind,
        required,
        applies_to: AppliesTo::All,
        formations: None,
        group,
    }
}

const fn only(
    name: &'static str,
    label: &'static str,
    kind: InputKind,
    containment: &'static str,
    group: Group,
) -> InputSpec {
    InputSpec {
        name,
        label,
        kind,
        required: false,
        applies_to: AppliesTo::Only(containment),
        formations: None,
        group,
    }
}

const fn only_in(
    name: &'static str,
    label: &'static str,
    kind: InputKind,
    containments: &'static [&'static str],
    group: Group,
) -> InputSpec {
    InputSpec {
        name,
        label,
        kind,
        required: false,
        applies_to: AppliesTo::AnyOf(containments),
        formations: None,
        group,
    }
}

const fn pushed(
    name: &'static str,
    label: &'static str,
    kind: InputKind,
    group: Group,
) -> InputSpec {
    InputSpec {
        name,
        label,
        kind,
        required: false,
        applies_to: AppliesTo::All,
        formations: Some(PUSHED),
        group,
    }
}

const NUM: InputKind = InputKind::Number { nullable: false };
const NUM_OR_NONE: InputKind = InputKind::Number { nullable: true };

/// Every input of an Option, in the order defaults are resolved (a default may depend on an
/// input above it). Names follow `GLOSSARY.md`.
pub const INPUTS: &[InputSpec] = &[
    spec("cargo", "Cargo", InputKind::Cargo, true, Group::Required),
    spec(
        "hull_form",
        "Hull Form",
        InputKind::Choice(HULL_FORMS),
        true,
        Group::Required,
    ),
    spec(
        "formation",
        "Formation",
        InputKind::Formation,
        true,
        Group::Required,
    ),
    spec(
        "length_overall_m",
        "Length overall",
        NUM,
        true,
        Group::Required,
    ),
    spec("beam_m", "Beam", NUM, true, Group::Required),
    // Moulded depth: keel to main deck at the side. Left out, it comes from a rule in the Defaults
    // Profile: the depth-to-beam ratio of the Cargo times the beam (issue #120).
    spec(
        "moulded_depth_m",
        "Moulded depth",
        NUM,
        false,
        Group::HullAndContainment,
    ),
    // Held only when the Option is defined by a standard class (see `class_input`).
    spec(
        "vessel_scheme",
        "Class Scheme",
        InputKind::VesselScheme,
        false,
        Group::VesselClass,
    ),
    spec(
        "vessel_class",
        "Vessel Class",
        InputKind::VesselClass,
        false,
        Group::VesselClass,
    ),
    spec(
        "cargo_containment",
        "Cargo Containment",
        InputKind::Choice(&["box_hold", "sloped_hopper_hold", "type_c_tanks"]),
        false,
        Group::HullAndContainment,
    ),
    spec(
        "block_coefficient",
        "Block coefficient",
        NUM,
        false,
        Group::HullAndContainment,
    ),
    spec(
        "min_freeboard_m",
        "Minimum freeboard",
        NUM,
        false,
        Group::HullAndContainment,
    ),
    spec(
        "bow_rake_length_to_depth_ratio",
        "Bow rake length over depth",
        NUM,
        false,
        Group::HullAndContainment,
    ),
    pushed(
        "pushed_stern_void_m",
        "Stern void of a pushed barge",
        NUM,
        Group::HullAndContainment,
    ),
    only(
        "lightship_factor_t_per_m3",
        "Lightship factor",
        NUM,
        "box_hold",
        Group::HullAndContainment,
    ),
    only(
        "ruleset",
        "Ruleset",
        InputKind::Choice(RULESETS),
        "type_c_tanks",
        Group::GasTanks,
    ),
    only(
        "fill_ratio",
        "Fill ratio",
        NUM,
        "type_c_tanks",
        Group::GasTanks,
    ),
    only(
        "max_tank_od_m",
        "Maximum tank outer diameter",
        NUM,
        "type_c_tanks",
        Group::GasTanks,
    ),
    only(
        "tanks_in_line",
        "Tanks in line",
        NUM,
        "type_c_tanks",
        Group::GasTanks,
    ),
    only(
        "tank_support_height_m",
        "Tank support height",
        NUM,
        "type_c_tanks",
        Group::GasTanks,
    ),
    only(
        "tank_dome_height_m",
        "Tank dome height",
        NUM,
        "type_c_tanks",
        Group::GasTanks,
    ),
    only(
        "tank_dome_diameter_m",
        "Tank dome diameter",
        NUM,
        "type_c_tanks",
        Group::GasTanks,
    ),
    only(
        "wing_tank_gap_m",
        "Wing tank gap",
        NUM,
        "type_c_tanks",
        Group::GasTanks,
    ),
    only(
        "wing_tank_height_ratio",
        "Wing tank height over tank diameter",
        NUM,
        "type_c_tanks",
        Group::GasTanks,
    ),
    only(
        "tank_wall_thickness_mm",
        "Tank wall thickness",
        NUM,
        "type_c_tanks",
        Group::GasTanks,
    ),
    only(
        "optimise_depth",
        "Optimise depth",
        InputKind::Bool,
        "type_c_tanks",
        Group::GasTanks,
    ),
    only_in(
        "hold_wing_void_width_m",
        "Wing void width",
        NUM,
        DRY_HOLDS,
        Group::DryBulkHold,
    ),
    only_in(
        "hold_double_bottom_m",
        "Hold double bottom",
        NUM,
        DRY_HOLDS,
        Group::DryBulkHold,
    ),
    only(
        "hopper_slope_h_per_v",
        "Hopper slope (horizontal over vertical)",
        NUM,
        "sloped_hopper_hold",
        Group::DryBulkHold,
    ),
    only_in(
        "hatch_coaming_height_m",
        "Hatch coaming height",
        NUM,
        DRY_HOLDS,
        Group::DryBulkHold,
    ),
    only(
        "n_holds",
        "Number of holds",
        NUM,
        "sloped_hopper_hold",
        Group::DryBulkHold,
    ),
    only_in(
        "bulk_end_bulkhead_m",
        "End bulkhead allowance",
        NUM,
        DRY_HOLDS,
        Group::DryBulkHold,
    ),
    only_in(
        "hold_fill_ratio",
        "Hold fill ratio",
        NUM,
        DRY_HOLDS,
        Group::DryBulkHold,
    ),
    only(
        "inner_bottom_plate_mm",
        "Inner bottom plate thickness",
        NUM,
        "sloped_hopper_hold",
        Group::DryBulkHold,
    ),
    only(
        "hopper_plate_mm",
        "Hopper plate thickness",
        NUM,
        "sloped_hopper_hold",
        Group::DryBulkHold,
    ),
    only(
        "hold_bulkhead_plate_mm",
        "Hold bulkhead plate thickness",
        NUM,
        "sloped_hopper_hold",
        Group::DryBulkHold,
    ),
    only(
        "hatch_coaming_plate_mm",
        "Hatch coaming plate thickness",
        NUM,
        "sloped_hopper_hold",
        Group::DryBulkHold,
    ),
    spec(
        "propulsive_efficiency",
        "Propulsive efficiency",
        NUM,
        false,
        Group::SpeedAndPower,
    ),
    spec(
        "max_speed_kn",
        "Maximum speed",
        NUM,
        false,
        Group::SpeedAndPower,
    ),
    spec(
        "air_draft_limit_m",
        "Air-draft limit",
        NUM_OR_NONE,
        false,
        Group::WaterwayLimits,
    ),
    spec(
        "draught_limit_m",
        "Draught limit",
        NUM_OR_NONE,
        false,
        Group::WaterwayLimits,
    ),
    spec(
        "channel_depth_m",
        "Channel depth",
        NUM_OR_NONE,
        false,
        Group::WaterwayLimits,
    ),
    spec(
        "channel_under_keel_clearance_m",
        "Under-keel clearance",
        NUM,
        false,
        Group::WaterwayLimits,
    ),
    spec(
        "channel_bottom_width_m",
        "Channel bottom width",
        NUM_OR_NONE,
        false,
        Group::WaterwayLimits,
    ),
    spec(
        "channel_bank_slope",
        "Bank slope",
        NUM,
        false,
        Group::WaterwayLimits,
    ),
    spec(
        "channel_current_kn",
        "Current",
        NUM,
        false,
        Group::WaterwayLimits,
    ),
    spec(
        "water_density_t_per_m3",
        "Water density",
        NUM,
        false,
        Group::WaterwayLimits,
    ),
    spec(
        "squat_safety_margin_m",
        "Squat safety margin",
        NUM,
        false,
        Group::WaterwayLimits,
    ),
    spec(
        "hull_steel_rate",
        "Hull steel rate",
        NUM,
        false,
        Group::CostRates,
    ),
    spec(
        "tank_shell_rate",
        "Tank shell rate",
        NUM,
        false,
        Group::CostRates,
    ),
    spec(
        "tank_supports_rate",
        "Tank supports rate",
        NUM,
        false,
        Group::CostRates,
    ),
    spec(
        "insulation_rate",
        "Insulation rate",
        NUM,
        false,
        Group::CostRates,
    ),
    spec(
        "piping_rate",
        "Piping and machinery rate",
        NUM,
        false,
        Group::CostRates,
    ),
    spec(
        "outfitting_rate",
        "Outfitting rate",
        NUM,
        false,
        Group::CostRates,
    ),
    spec(
        "cargo_system_lump",
        "Cargo system",
        NUM,
        false,
        Group::CostRates,
    ),
    spec(
        "paint_anodes_misc",
        "Paint, anodes and miscellaneous",
        NUM,
        false,
        Group::CostRates,
    ),
    spec(
        "battery_rate_per_kwh",
        "Battery rate",
        NUM,
        false,
        Group::CostRates,
    ),
    spec(
        "propulsion_rate_per_kw",
        "Propulsion rate",
        NUM,
        false,
        Group::CostRates,
    ),
    spec(
        "wheelhouse_accom_lump",
        "Wheelhouse and accommodation",
        NUM,
        false,
        Group::CostRates,
    ),
    spec(
        "tug_outfitting_lump",
        "Tug outfitting",
        NUM,
        false,
        Group::CostRates,
    ),
    spec(
        "yard_margin_fraction",
        "Yard margin",
        NUM,
        false,
        Group::CostRates,
    ),
    spec("delivery_lump", "Delivery", NUM, false, Group::CostRates),
];

/// Where a stored input came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// The designer gave it.
    Typed,
    /// The Defaults Profile gave it.
    Default,
    /// The Vessel Class gave it (issue #94).
    Class,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Typed => "typed",
            Source::Default => "default",
            Source::Class => "class",
        }
    }
}

/// One input as used, with its marker.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredInput {
    pub value: Value,
    pub source: Source,
}

/// One complete set of inputs within a Design (glossary: Option).
#[derive(Debug, Clone, PartialEq)]
pub struct DesignOption {
    pub name: String,
    /// Inputs in `INPUTS` order. Inputs that do not apply to the Cargo Containment are left out.
    pub inputs: Vec<(String, StoredInput)>,
    /// SHA-256 of the canonical JSON of this Option's results, as the file stored it (ADR 0007).
    pub results_hash: Option<String>,
}

impl DesignOption {
    pub fn input(&self, name: &str) -> Option<&StoredInput> {
        self.inputs.iter().find(|(n, _)| n == name).map(|(_, i)| i)
    }
}

/// One study saved as a single file (glossary: Design).
#[derive(Debug, Clone, PartialEq)]
pub struct Design {
    pub engine_version: String,
    /// The name of the Defaults Profile the defaults came from, for the record.
    pub defaults_profile: Option<String>,
    pub options: Vec<DesignOption>,
}

impl Design {
    /// The file's JSON.
    pub fn to_json(&self) -> Value {
        let options: Vec<Value> = self
            .options
            .iter()
            .map(|o| {
                let inputs: Map<String, Value> = o
                    .inputs
                    .iter()
                    .map(|(n, i)| {
                        (
                            n.clone(),
                            json!({ "value": i.value, "source": i.source.as_str() }),
                        )
                    })
                    .collect();
                let mut option = json!({ "name": o.name, "inputs": inputs });
                if let Some(h) = &o.results_hash {
                    option["results_hash"] = json!(h);
                }
                option
            })
            .collect();
        let mut root = json!({
            "format": "vdesign",
            "format_version": FORMAT_VERSION,
            "engine_version": self.engine_version,
            "units": "SI",
            "options": options,
        });
        if let Some(p) = &self.defaults_profile {
            root["defaults_profile"] = json!(p);
        }
        root
    }

    /// The file's text: pretty JSON with a final newline.
    pub fn to_text(&self) -> String {
        let mut text = serde_json::to_string_pretty(&self.to_json()).expect("JSON");
        text.push('\n');
        text
    }
}

/// Hard errors and soft warnings found in a Design, each a full sentence for a person.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Report {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    /// The inputs each error is about, one list per entry of `errors`. One name is a message
    /// about that input, several a cross-check, none a message about the Option or the file.
    pub error_inputs: Vec<Vec<String>>,
    /// The same for `warnings`.
    pub warning_inputs: Vec<Vec<String>>,
    /// The warnings that say a version 1 file was changed to the current inputs as it was read
    /// (also in `warnings`), so a front end can show them on their own.
    pub migration_notes: Vec<String>,
}

impl Report {
    /// Add a hard error and the inputs it is about.
    pub fn error(&mut self, text: String, inputs: &[&str]) {
        self.errors.push(text);
        self.error_inputs
            .push(inputs.iter().map(|n| n.to_string()).collect());
    }

    /// Add a soft warning and the inputs it is about.
    pub fn warning(&mut self, text: String, inputs: &[&str]) {
        self.warnings.push(text);
        self.warning_inputs
            .push(inputs.iter().map(|n| n.to_string()).collect());
    }

    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// The result of reading a `.vdesign` file: the Design as used (when it could be read) and
/// what validation found.
#[derive(Debug, Clone, PartialEq)]
pub struct Checked {
    pub design: Option<Design>,
    pub report: Report,
}

/// A starter Design: one Option with example required inputs (typed, for the designer to edit)
/// and every other input from the profile, marked as defaults. Fails when the profile's defaults
/// do not make a valid Design with the example inputs (for example a minimum freeboard deeper
/// than the example hull).
pub fn new_design(profile: &Profile) -> Result<Design, String> {
    let typed = json!({
        "cargo": "bauxite",
        "hull_form": "raked_box",
        "formation": { "kind": "pushed_barge" },
        "length_overall_m": 76.5,
        "beam_m": 11.4,
        "moulded_depth_m": 3.5,
    });
    let mut report = Report::default();
    let option = read_option(
        "Option 1",
        typed.as_object().expect("an object"),
        profile,
        Missing::FillProfile,
        &mut report,
    );
    if !report.is_ok() {
        return Err(format!(
            "the Defaults Profile '{}' does not make a valid starter Design: {}",
            profile.name(),
            report.errors.join("; ")
        ));
    }
    let mut design = Design {
        engine_version: ENGINE_VERSION.into(),
        defaults_profile: Some(profile.name().into()),
        options: vec![option],
    };
    crate::run::stamp(&mut design);
    Ok(design)
}

/// The input whose default is a rule (ratio times beam) and not a value.
pub const DEPTH_INPUT: &str = "moulded_depth_m";

/// The Cargo groups of the depth rule, as the Defaults Profile names them
/// (`depth_to_beam_ratio.default_by_cargo_group`). `liquid` has no Cargo yet.
pub const DEPTH_GROUPS: &[&str] = &["dry_bulk", "liquid", "gas"];

/// The depth rule's Cargo group of a Cargo: a dry-bulk Cargo is `dry_bulk` (the group also holds
/// container), and every liquefied-gas Cargo (LNG, ethane, the LPGs) is `gas`. `None` for a Cargo
/// the database does not have.
pub fn depth_group(cargo: &str) -> Option<&'static str> {
    match ovcd_core::cargo::get_cargo(cargo).ok()?.phase {
        "dry_bulk" => Some("dry_bulk"),
        "liquid_gas" => Some("gas"),
        _ => None,
    }
}

/// What to do with an input the file leaves out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Missing {
    /// A hard error: a saved file must hold every input as used.
    Refuse,
    /// Write the Defaults Profile value in, marked `default`, with a warning. An input that is new
    /// since the first files takes the fixed value the engine used (see [`LEGACY_FIXED_INPUTS`]).
    Fill,
    /// As `Fill`, for a Design being made or edited now (a new Option, a reset input), where the
    /// Defaults Profile gives every value, new inputs included.
    FillProfile,
}

/// Read and validate a `.vdesign` file's text, without running it. A file that leaves out an
/// input is refused; the returned Design still shows the defaults it would take, so every other
/// problem is reported too.
pub fn check(text: &str, profile: &Profile) -> Checked {
    read_text(text, profile, Missing::Refuse)
}

/// Read a `.vdesign` file's text and write every left-out input in, as used and marked
/// `default`, from `profile`. The report holds every other problem, as [`check`] gives it. Save
/// the returned Design only when the report has no hard errors.
pub fn fill(text: &str, profile: &Profile) -> Checked {
    fill_as(text, profile, Missing::Fill)
}

/// As [`fill`], for a Design being edited in a session: an input the Design lacks (a reset input,
/// or one a changed Cargo Containment or Formation now uses) takes the Defaults Profile value,
/// never the fixed value an old file gets.
pub(crate) fn fill_from_profile(text: &str, profile: &Profile) -> Checked {
    fill_as(text, profile, Missing::FillProfile)
}

fn fill_as(text: &str, profile: &Profile, missing: Missing) -> Checked {
    let mut checked = read_text(text, profile, missing);
    if let Some(design) = &mut checked.design
        && checked.report.warnings.iter().any(|w| w.contains(FILLED))
    {
        design.defaults_profile = Some(profile.name().into());
    }
    if checked.report.is_ok()
        && let Some(design) = &mut checked.design
    {
        crate::run::stamp(design);
    }
    checked
}

const FILLED: &str = "was not stored in the file; wrote in the Defaults Profile value";

fn read_text(text: &str, profile: &Profile, missing: Missing) -> Checked {
    let mut report = Report::default();
    let root: Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            report.error(format!("the file is not JSON: {e}"), &[]);
            return Checked {
                design: None,
                report,
            };
        }
    };
    for (path, key) in duplicate_keys(text) {
        let place = if path.is_empty() {
            "the file".to_string()
        } else {
            path
        };
        report.error(
            format!("`{key}` appears twice in {place}; a key may appear once"),
            &[],
        );
    }
    let design = read_design(&root, profile, missing, &mut report);
    Checked { design, report }
}

/// Every key that appears twice in one JSON object of `text`, with the path of that object
/// (`options[0].inputs`; empty for the top level). A plain JSON read keeps the last value and says
/// nothing, so a file with an ambiguous key would give a confident result.
fn duplicate_keys(text: &str) -> Vec<(String, String)> {
    struct Walk<'a> {
        path: String,
        found: &'a mut Vec<(String, String)>,
    }
    impl<'de> DeserializeSeed<'de> for Walk<'_> {
        type Value = ();
        fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<(), D::Error> {
            d.deserialize_any(self)
        }
    }
    impl<'de> Visitor<'de> for Walk<'_> {
        type Value = ();
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("any JSON value")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
            let mut seen = HashSet::new();
            while let Some(key) = map.next_key::<String>()? {
                if !seen.insert(key.clone()) {
                    self.found.push((self.path.clone(), key.clone()));
                }
                let path = if self.path.is_empty() {
                    key
                } else {
                    format!("{}.{key}", self.path)
                };
                map.next_value_seed(Walk {
                    path,
                    found: &mut *self.found,
                })?;
            }
            Ok(())
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
            let mut i = 0;
            while seq
                .next_element_seed(Walk {
                    path: format!("{}[{i}]", self.path),
                    found: &mut *self.found,
                })?
                .is_some()
            {
                i += 1;
            }
            Ok(())
        }
        // Scalars carry no keys. (With `arbitrary_precision` a number arrives as a one-key map,
        // which this treats as an object with no duplicate.)
        fn visit_bool<E>(self, _: bool) -> Result<(), E> {
            Ok(())
        }
        fn visit_str<E>(self, _: &str) -> Result<(), E> {
            Ok(())
        }
        fn visit_unit<E>(self) -> Result<(), E> {
            Ok(())
        }
        fn visit_i64<E>(self, _: i64) -> Result<(), E> {
            Ok(())
        }
        fn visit_u64<E>(self, _: u64) -> Result<(), E> {
            Ok(())
        }
        fn visit_f64<E>(self, _: f64) -> Result<(), E> {
            Ok(())
        }
    }
    let mut found = Vec::new();
    let mut de = serde_json::Deserializer::from_str(text);
    // A text that does not read is reported by the caller's own parse.
    let _ = Walk {
        path: String::new(),
        found: &mut found,
    }
    .deserialize(&mut de);
    found
}

fn read_design(
    root: &Value,
    profile: &Profile,
    missing: Missing,
    report: &mut Report,
) -> Option<Design> {
    let Some(root) = root.as_object() else {
        report.error("the file must hold a JSON object".into(), &[]);
        return None;
    };
    if root.get("format").and_then(Value::as_str) != Some("vdesign") {
        report.error("the file needs `\"format\": \"vdesign\"`".into(), &[]);
    }
    let version = root.get("format_version").and_then(Value::as_u64);
    match version {
        Some(OLDEST_FORMAT_VERSION..=FORMAT_VERSION) => {}
        Some(v) => report.error(
            format!(
                "format_version {v} is not supported; this tool reads versions \
                 {OLDEST_FORMAT_VERSION} to {FORMAT_VERSION}"
            ),
            &[],
        ),
        None => report.error("the file needs an integer `format_version`".into(), &[]),
    }
    let engine_version = match root.get("engine_version").and_then(Value::as_str) {
        Some(v) if !v.trim().is_empty() => v.to_string(),
        _ => {
            report.error(
                "the file needs a non-empty string `engine_version`".into(),
                &[],
            );
            String::new()
        }
    };
    if let Some(units) = root.get("units")
        && units != "SI"
    {
        report.error(format!("`units` must be \"SI\", not {units}"), &[]);
    }
    let defaults_profile = root
        .get("defaults_profile")
        .and_then(Value::as_str)
        .map(String::from);
    for key in root.keys() {
        let known = [
            "format",
            "format_version",
            "engine_version",
            "units",
            "defaults_profile",
            "options",
        ];
        if !known.contains(&key.as_str()) {
            report.warning(format!("unknown top-level key `{key}` is ignored"), &[]);
        }
    }
    let Some(raw_options) = root.get("options").and_then(Value::as_array) else {
        report.error("the file needs an `options` list".into(), &[]);
        return None;
    };
    if raw_options.is_empty() {
        report.error("a Design needs at least one Option".into(), &[]);
    }
    let mut options = Vec::new();
    for (i, raw) in raw_options.iter().enumerate() {
        let name = raw
            .get("name")
            .and_then(Value::as_str)
            .filter(|n| !n.trim().is_empty());
        let Some(name) = name else {
            report.error(
                format!("Option {} needs a non-empty string `name`", i + 1),
                &[],
            );
            continue;
        };
        if options.iter().any(|o: &DesignOption| o.name == name) {
            report.error(
                format!("two Options are named '{name}'; names must differ"),
                &[],
            );
        }
        let Some(inputs) = raw.get("inputs").and_then(Value::as_object) else {
            report.error(format!("Option '{name}': needs an `inputs` object"), &[]);
            continue;
        };
        for key in raw.as_object().into_iter().flat_map(|o| o.keys()) {
            if !["name", "inputs", "results_hash"].contains(&key.as_str()) {
                report.warning(
                    format!("Option '{name}': unknown key `{key}` is ignored"),
                    &[],
                );
            }
        }
        let mut m = inputs.clone();
        migrate_ruleset(name, &mut m, version, report);
        if version == Some(1) {
            migrate_v1_option(name, &mut m, profile, report);
        }
        let inputs = &m;
        let mut option = read_option(name, inputs, profile, missing, report);
        match raw.get("results_hash") {
            None | Some(Value::Null) => {}
            Some(Value::String(h)) if crate::run::is_hash_shape(h) => {
                option.results_hash = Some(h.clone());
            }
            Some(Value::String(_)) => report.error(
                format!("Option '{name}': `results_hash` must be 64 hex digits"),
                &[],
            ),
            Some(_) => report.error(format!("Option '{name}': `results_hash` must be text"), &[]),
        }
        options.push(option);
    }
    Some(Design {
        engine_version,
        defaults_profile,
        options,
    })
}

/// The rule set name of an Option (issue #237): there is one, `IGC_2G`, the per-tank d(Vc) of
/// IGC 2.4.1.1.
///
/// - `IGC_2G_EXACT` (any format version) is read as `IGC_2G`, with a warning. It was the same
///   per-tank rule.
/// - `IGC_2G` in a version 1 file is the old stepped reading (total volume, 0.8 / 1.0 / 2.0 m):
///   it is kept as `IGC_2G`, with a warning that it now means the per-tank rule, so results may
///   change. A version 2 file is written by this build, so it needs no note; version 2 is
///   unreleased (ADR 0007).
fn migrate_ruleset(
    name: &str,
    inputs: &mut Map<String, Value>,
    version: Option<u64>,
    report: &mut Report,
) {
    let current = match inputs.get("ruleset") {
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Object(o)) => o.get("value").and_then(Value::as_str).map(String::from),
        _ => None,
    };
    let text = match (current.as_deref(), version) {
        (Some("IGC_2G_EXACT"), _) => {
            match inputs.get_mut("ruleset") {
                Some(Value::Object(o)) => {
                    o.insert("value".into(), json!("IGC_2G"));
                }
                Some(v) => *v = json!("IGC_2G"),
                None => {}
            }
            "IGC_2G_EXACT is now IGC_2G: the same per-tank rule of IGC 2.4.1.1"
        }
        (Some("IGC_2G"), Some(1)) => {
            "IGC_2G now uses the per-tank rule of IGC 2.4.1.1 (d(Vc) from the gross volume of \
             each tank, dome included, measured plate to plate per 2.4.2), so results may change"
        }
        _ => return,
    };
    let text = format!("Option '{name}': `ruleset`: {text}");
    report.migration_notes.push(text.clone());
    report.warning(text, &["ruleset"]);
}

/// Change one Option of a version 1 file to the current inputs, with a warning for each change.
/// Version 1 files hold three things that no longer exist (issue #175):
///
/// - the `ADN_G` ruleset: read as `IGC_2G`, the marker (`typed` or `default`) kept;
/// - the input `adn_tank_cap_m3`: dropped;
/// - no `max_tank_od_m` (a gas Option only): set to the Defaults Profile value, marked `default`.
///   Version 1 ran with a fixed 12 m cap, so the results may change.
///
/// The file on disk is not touched. Saving writes version 2.
fn migrate_v1_option(
    name: &str,
    inputs: &mut Map<String, Value>,
    profile: &Profile,
    report: &mut Report,
) {
    let at = |msg: String| format!("Option '{name}': {msg}");
    let note = |report: &mut Report, text: String, inputs: &[&str]| {
        report.migration_notes.push(text.clone());
        report.warning(text, inputs);
    };
    let ruleset_is_adn = match inputs.get("ruleset") {
        Some(Value::String(s)) => s == "ADN_G",
        Some(Value::Object(o)) => o.get("value").and_then(Value::as_str) == Some("ADN_G"),
        _ => false,
    };
    if ruleset_is_adn {
        match inputs.get_mut("ruleset") {
            Some(Value::Object(o)) => {
                o.insert("value".into(), json!("IGC_2G"));
            }
            Some(v) => *v = json!("IGC_2G"),
            None => {}
        }
        note(
            report,
            at(
                "`ruleset` ADN_G is no longer offered (the engine applies IGC rules only to gas \
                 tanks); this version 1 file was opened as IGC_2G"
                    .into(),
            ),
            &["ruleset"],
        );
    }
    if inputs.remove("adn_tank_cap_m3").is_some() {
        note(
            report,
            at(
                "`adn_tank_cap_m3` is no longer an input (there is no tank volume cap); it was \
                 dropped"
                    .into(),
            ),
            &["adn_tank_cap_m3"],
        );
    }
    let containment = match inputs.get("cargo_containment") {
        Some(Value::String(s)) => Some(s.as_str()),
        Some(Value::Object(o)) => o.get("value").and_then(Value::as_str),
        _ => None,
    };
    // A profile with no default leaves the input out, and the ordinary "not stored" rule applies.
    if containment == Some("type_c_tanks")
        && !inputs.contains_key("max_tank_od_m")
        && let Some(default) = profile.default("max_tank_od_m")
    {
        inputs.insert(
            "max_tank_od_m".into(),
            json!({ "value": default, "source": "default" }),
        );
        note(
            report,
            at(format!(
                "this version 1 file has no `max_tank_od_m`; it now uses the default {} m \
                 (version 1 used a fixed 12 m cap, so results may change)",
                show(default)
            )),
            &["max_tank_od_m"],
        );
    }
}

/// Split a stored input into its value and marker. A bare value counts as typed.
pub(crate) fn unmark(raw: &Value) -> Result<StoredInput, String> {
    if let Some(o) = raw.as_object()
        && o.contains_key("source")
    {
        let source = match o.get("source").and_then(Value::as_str) {
            Some("typed") => Source::Typed,
            Some("default") => Source::Default,
            Some("class") => Source::Class,
            _ => return Err("`source` must be \"typed\", \"default\" or \"class\"".into()),
        };
        let value = o.get("value").ok_or("a marked input needs a `value`")?;
        if o.len() != 2 {
            return Err("a marked input holds only `value` and `source`".into());
        }
        return Ok(StoredInput {
            value: value.clone(),
            source,
        });
    }
    Ok(StoredInput {
        value: raw.clone(),
        source: Source::Typed,
    })
}

fn read_option(
    name: &str,
    raw: &Map<String, Value>,
    profile: &Profile,
    missing: Missing,
    report: &mut Report,
) -> DesignOption {
    let at = |msg: String| format!("Option '{name}': {msg}");
    let mut left_out: Vec<&str> = Vec::new();
    let mut legacy_added: Vec<&'static str> = Vec::new();
    // A class named with no length, beam or Formation gives them; `fill` writes them in.
    let filled_from_class;
    let raw = if matches!(missing, Missing::Fill | Missing::FillProfile) {
        filled_from_class = crate::class_input::prefill(raw);
        &filled_from_class
    } else {
        raw
    };
    for key in raw.keys() {
        if !INPUTS.iter().any(|s| s.name == key) {
            report.error(at(format!("unknown input `{key}`")), &[]);
        }
    }
    let mut inputs: Vec<(String, StoredInput)> = Vec::new();
    let value_of = |inputs: &[(String, StoredInput)], n: &str| -> Option<Value> {
        inputs
            .iter()
            .find(|(k, _)| k == n)
            .map(|(_, i)| i.value.clone())
    };
    for spec in INPUTS {
        let containment = value_of(&inputs, "cargo_containment");
        let formation = value_of(&inputs, "formation");
        let kind = formation
            .as_ref()
            .and_then(|f| f.get("kind"))
            .and_then(Value::as_str);
        let applies = match spec.applies_to {
            AppliesTo::All => true,
            scope => containment
                .as_ref()
                .and_then(Value::as_str)
                .is_some_and(|c| scope.includes(c)),
        } && formation_includes(spec.formations, kind);
        if let Some(raw_input) = raw.get(spec.name) {
            let stored = match unmark(raw_input) {
                Ok(s) => s,
                Err(e) => {
                    report.error(at(format!("`{}`: {e}", spec.name)), &[spec.name]);
                    continue;
                }
            };
            if !applies {
                let c = containment
                    .as_ref()
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                let text = if formation_includes(spec.formations, kind) {
                    format!(
                        "`{}` is not used for a {c} Cargo Containment; ignored",
                        spec.name
                    )
                } else {
                    format!(
                        "`{}` is not used for a {} Formation; ignored",
                        spec.name,
                        kind.unwrap_or("unknown")
                    )
                };
                report.warning(at(text), &[spec.name]);
                continue;
            }
            match check_value(spec, &stored.value, profile) {
                Ok((value, warnings)) => {
                    for w in warnings.into_iter().map(&at) {
                        report.warning(w, &[spec.name]);
                    }
                    inputs.push((
                        spec.name.into(),
                        StoredInput {
                            value,
                            source: stored.source,
                        },
                    ));
                }
                Err(e) => report.error(at(e), &[spec.name]),
            }
            continue;
        }
        if spec.required {
            report.error(
                at(format!("missing required input `{}`", spec.name)),
                &[spec.name],
            );
            continue;
        }
        if !applies || crate::class_input::UNSET_WHEN_ABSENT.contains(&spec.name) {
            continue;
        }
        // An input that is new since the first files, left out of a file: the fixed value the
        // engine used before it was an input (never the profile value), noted once per Option.
        if !matches!(missing, Missing::FillProfile)
            && let Some(fixed) = LEGACY_FIXED_INPUTS.iter().find(|l| l.name == spec.name)
        {
            legacy_added.push(spec.name);
            inputs.push((
                spec.name.into(),
                StoredInput {
                    value: json!(fixed.value),
                    source: Source::Default,
                },
            ));
            continue;
        }
        let default = match default_for(spec.name, &inputs, profile) {
            Ok(Some(d)) => d,
            // The input it depends on is missing or wrong; that error is already reported.
            Ok(None) => continue,
            Err(e) => {
                report.error(at(e), &[spec.name]);
                continue;
            }
        };
        match missing {
            Missing::Refuse => left_out.push(spec.name),
            Missing::Fill | Missing::FillProfile => report.warning(
                at(format!(
                    "`{}` {FILLED} {}, marked default",
                    spec.name,
                    show(&default)
                )),
                &[spec.name],
            ),
        }
        inputs.push((
            spec.name.into(),
            StoredInput {
                value: default,
                source: Source::Default,
            },
        ));
    }
    // The dome diameter is not a value the engine used before: the dome was not in Vc. It gets its
    // own note, which says the results may change.
    if let Some(at_i) = legacy_added
        .iter()
        .position(|n| *n == "tank_dome_diameter_m")
    {
        legacy_added.remove(at_i);
        let text = format!(
            "Option '{name}': this file has no `tank_dome_diameter_m`; it now holds {:.1} m (marked \
             default). The dome now counts in Vc for d(Vc) (IGC 2.4.1.1), so the results may \
             change",
            GAS_TANK_DOME_DIAMETER_M
        );
        report.migration_notes.push(text.clone());
        report.warning(text, &["tank_dome_diameter_m"]);
    }
    if !legacy_added.is_empty() {
        let list = legacy_added
            .iter()
            .map(|n| format!("`{n}`"))
            .collect::<Vec<_>>()
            .join(", ");
        let text = format!(
            "Option '{name}': this file has no {list}; {} the fixed value the engine \
             used before it was an input (marked default), so the results are unchanged",
            if legacy_added.len() == 1 {
                "it now holds"
            } else {
                "each now holds"
            }
        );
        report.migration_notes.push(text.clone());
        report.warning(text, &legacy_added);
    }
    if !left_out.is_empty() {
        report.error(
            at(format!(
                "inputs not stored in the file: {}. A saved Design holds every input as used, so a \
             Defaults Profile edit cannot change it; run `ovcd fill` to write them in from the \
             Defaults Profile, marked default",
                left_out.join(", ")
            )),
            &left_out,
        );
    }
    crate::class_input::resolve(&mut inputs, profile, report, &at);
    cross_checks(&inputs, profile, report, &at);
    impossible_results(&inputs, report, &at);
    DesignOption {
        name: name.into(),
        inputs,
        results_hash: None,
    }
}

/// The profile's default for an input, given the inputs resolved so far. `Ok(None)` when an
/// input it depends on is missing or wrong (that error is reported elsewhere); `Err` when the
/// profile has no default for this case.
fn default_for(
    name: &str,
    inputs: &[(String, StoredInput)],
    profile: &Profile,
) -> Result<Option<Value>, String> {
    let got = |n: &str| {
        inputs
            .iter()
            .find(|(k, _)| k == n)
            .and_then(|(_, i)| i.value.as_str().map(String::from))
    };
    if name == DEPTH_INPUT {
        return depth_default(inputs, profile);
    }
    let (value, case) = match KEYED_DEFAULTS.iter().find(|(n, _, _)| *n == name) {
        Some((_, by, _)) => {
            let key = match *by {
                "cargo_phase" => {
                    let Some(cargo) = got("cargo") else {
                        return Ok(None);
                    };
                    let Ok(props) = ovcd_core::cargo::get_cargo(&cargo) else {
                        return Ok(None);
                    };
                    props.phase.to_string()
                }
                other => match got(other) {
                    Some(k) => k,
                    None => return Ok(None),
                },
            };
            (
                profile.default_by(name, by, &key),
                format!("`default_by_{by}` for {key}"),
            )
        }
        None => (profile.default(name), "a `default`".to_string()),
    };
    match value {
        Some(v) => Ok(Some(normalise(v.clone()))),
        None => Err(format!(
            "the Defaults Profile '{}' gives no default for `{name}` ({case} is missing)",
            profile.name()
        )),
    }
}

/// The default moulded depth: the Cargo group's depth-to-beam ratio times the beam, rounded to the
/// millimetre. `Ok(None)` when the Cargo or the beam is missing or wrong (that error is reported
/// elsewhere).
fn depth_default(
    inputs: &[(String, StoredInput)],
    profile: &Profile,
) -> Result<Option<Value>, String> {
    let get = |n: &str| inputs.iter().find(|(k, _)| k == n).map(|(_, i)| &i.value);
    let Some(group) = get("cargo").and_then(Value::as_str).and_then(depth_group) else {
        return Ok(None);
    };
    let Some(beam) = get("beam_m")
        .and_then(Value::as_f64)
        .filter(|b| b.is_finite())
    else {
        return Ok(None);
    };
    let ratio = profile
        .default_by("depth_to_beam_ratio", "cargo_group", group)
        .and_then(Value::as_f64)
        .ok_or_else(|| {
            format!(
                "the Defaults Profile '{}' gives no depth-to-beam ratio for {group} Cargo \
                 (`depth_to_beam_ratio.default_by_cargo_group.{group}` is missing)",
                profile.name()
            )
        })?;
    Ok(Some(json!(((ratio * beam) * 1000.0).round() / 1000.0)))
}

/// Store every number as a float, so `10` and `10.0` are the same input.
fn normalise(v: Value) -> Value {
    match v.as_f64() {
        Some(x) => json!(x),
        _ => v,
    }
}

/// Check one value against its type and ranges. Returns the value as stored and any soft
/// warnings, or the hard error.
fn check_value(
    spec: &InputSpec,
    v: &Value,
    profile: &Profile,
) -> Result<(Value, Vec<String>), String> {
    let name = spec.name;
    match spec.kind {
        InputKind::Number { nullable } => {
            if v.is_null() {
                return if nullable {
                    Ok((Value::Null, vec![]))
                } else {
                    Err(format!("`{name}` must be a number"))
                };
            }
            let x = v.as_f64().ok_or_else(|| {
                if nullable {
                    format!("`{name}` must be a number or null (none)")
                } else {
                    format!("`{name}` must be a number")
                }
            })?;
            let warnings = number_ranges(name, name, x, profile)?;
            Ok((json!(x), warnings))
        }
        InputKind::Bool => match v {
            Value::Bool(_) => Ok((v.clone(), vec![])),
            _ => Err(format!("`{name}` must be true or false")),
        },
        InputKind::Choice(choices) => {
            let s = v.as_str().unwrap_or("");
            if choices.contains(&s) {
                Ok((v.clone(), vec![]))
            } else if name == "hull_form" {
                Err(format!(
                    "`hull_form` = {v} is not supported; only raked_box exists for now"
                ))
            } else {
                Err(format!(
                    "`{name}` = {v} must be one of: {}",
                    choices.join(", ")
                ))
            }
        }
        InputKind::Cargo => {
            let s = v.as_str().ok_or("`cargo` must be a string")?;
            ovcd_core::cargo::get_cargo(s).map_err(|e| format!("`cargo`: {e}"))?;
            Ok((v.clone(), vec![]))
        }
        InputKind::Formation => check_formation(v, profile),
        InputKind::VesselScheme => match v {
            Value::Null => Ok((Value::Null, vec![])),
            Value::String(s) if ovcd_core::vessel_classes::schemes().contains(&s.as_str()) => {
                Ok((v.clone(), vec![]))
            }
            _ => Err(format!(
                "`{name}` = {v} is not a scheme with a Vessel Class that can define a vessel \
                 (see `ovcd schema`)"
            )),
        },
        InputKind::VesselClass => match v {
            Value::Null => Ok((Value::Null, vec![])),
            Value::String(s) => match ovcd_core::vessel_classes::find(s) {
                Some(c) if c.pickable() => Ok((json!(c.id), vec![])),
                Some(c) => Err(format!(
                    "`{name}` = {v} cannot define a vessel: class {} gives no length or beam, or \
                     its Formation ({}) is not one a Design has",
                    c.id, c.formation
                )),
                None => Err(format!(
                    "`{name}` = {v} is not a Vessel Class (see `ovcd schema`)"
                )),
            },
            _ => Err(format!("`{name}` must be a class id or null")),
        },
    }
}

fn check_formation(v: &Value, profile: &Profile) -> Result<(Value, Vec<String>), String> {
    let o = v
        .as_object()
        .ok_or("`formation` must be an object with a `kind`")?;
    let kind = o.get("kind").and_then(Value::as_str).unwrap_or("");
    if !FORMATION_KINDS.contains(&kind) {
        return Err(format!(
            "`formation.kind` must be one of: {}",
            FORMATION_KINDS.join(", ")
        ));
    }
    let mut warnings = Vec::new();
    if kind != "pushed_convoy" {
        if o.len() != 1 {
            return Err(format!(
                "`formation`: a {kind} holds only `kind`; `abreast` and `in_line` are for a \
                 pushed_convoy"
            ));
        }
        return Ok((json!({ "kind": kind }), warnings));
    }
    let mut out = json!({ "kind": kind });
    for (key, profile_name) in [
        ("abreast", "formation_abreast"),
        ("in_line", "formation_in_line"),
    ] {
        let label = format!("formation.{key}");
        let n = o
            .get(key)
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("`{label}` must be a whole number for a pushed_convoy"))?;
        warnings.extend(number_ranges(&label, profile_name, n as f64, profile)?);
        out[key] = json!(n);
    }
    if o.len() != 3 {
        return Err(
            "`formation`: a pushed_convoy holds only `kind`, `abreast` and `in_line`".into(),
        );
    }
    Ok((out, warnings))
}

/// Hard range (error) and soft range (warning) of a number.
fn number_ranges(
    label: &str,
    name: &str,
    x: f64,
    profile: &Profile,
) -> Result<Vec<String>, String> {
    let unit = profile.unit(name);
    let with_unit = |n: f64| {
        if unit.is_empty() || unit == "-" {
            format!("{n}")
        } else {
            format!("{n} {unit}")
        }
    };
    if let Some(bound) = profile.hard(name)?.breach(x) {
        return Err(format!(
            "`{label}` = {x} is out of range: must be {bound} (hard limit)"
        ));
    }
    let mut warnings = Vec::new();
    if let Some(s) = profile.soft(name)?
        && (x < s.min || x > s.max)
    {
        warnings.push(format!(
            "`{label}` = {} is outside the usual range {} to {} (soft limit); accepted",
            with_unit(x),
            s.min,
            with_unit(s.max)
        ));
    }
    Ok(warnings)
}

/// Checks across inputs.
fn cross_checks(
    inputs: &[(String, StoredInput)],
    profile: &Profile,
    report: &mut Report,
    at: &dyn Fn(String) -> String,
) {
    let get = |n: &str| inputs.iter().find(|(k, _)| k == n).map(|(_, i)| &i.value);
    // A hull very long for its depth: a soft limit of the Defaults Profile (about 25).
    if let (Some(loa), Some(depth), Ok(Some(soft))) = (
        get("length_overall_m").and_then(Value::as_f64),
        get(DEPTH_INPUT).and_then(Value::as_f64),
        profile.soft("length_to_depth_ratio"),
    ) && depth > 0.0
        && loa / depth > soft.max
    {
        report.warning(
            at(format!(
                "`length_overall_m` = {} is {:.1} times `moulded_depth_m` = {}, over about {}: a \
                 very slender hull for its depth (soft limit); accepted",
                show(&json!(loa)),
                loa / depth,
                show(&json!(depth)),
                show(&json!(soft.max))
            )),
            &["length_overall_m", "moulded_depth_m"],
        );
    }
    if let (Some(cargo), Some(containment)) = (
        get("cargo").and_then(Value::as_str),
        get("cargo_containment").and_then(Value::as_str),
    ) && let Ok(props) = ovcd_core::cargo::get_cargo(cargo)
    {
        let carries = CARGO_CONTAINMENTS
            .iter()
            .any(|(c, phase)| *c == containment && *phase == props.phase);
        if !carries {
            let fits: Vec<&str> = CARGO_CONTAINMENTS
                .iter()
                .filter(|(_, phase)| *phase == props.phase)
                .map(|(c, _)| *c)
                .collect();
            report.error(
                at(format!(
                    "`cargo_containment` = {containment} cannot carry {cargo} ({}); use {}",
                    props.phase,
                    fits.join(" or ")
                )),
                &["cargo_containment", "cargo"],
            );
        }
    }
    if let (Some(fb), Some(d)) = (
        get("min_freeboard_m").and_then(Value::as_f64),
        get("moulded_depth_m").and_then(Value::as_f64),
    ) && fb >= d
    {
        report.error(
            at(format!(
                "`min_freeboard_m` = {fb} must be less than `moulded_depth_m` = {d} (hard limit)"
            )),
            &["min_freeboard_m", "moulded_depth_m"],
        );
    }
    // A count is a whole number.
    for name in ["n_holds", "tanks_in_line"] {
        if let Some(n) = get(name).and_then(Value::as_f64)
            && n.fract() != 0.0
        {
            report.error(
                at(format!(
                    "`{name}` = {n} must be a whole number (hard limit)"
                )),
                &[name],
            );
        }
    }
    // The depth search tries hulls up to a fixed depth; above it no searched hull keeps the
    // freeboard, and the core raises when no waterway limit caps the draught.
    if get("optimise_depth").and_then(Value::as_bool) == Some(true)
        && let Some(fb) = get("min_freeboard_m").and_then(Value::as_f64)
        && fb > DEPTH_SEARCH_MAX_M
    {
        report.error(
            at(format!(
                "`min_freeboard_m` = {fb} must be at most {DEPTH_SEARCH_MAX_M}, the deepest hull \
             `optimise_depth` searches (hard limit)"
            )),
            &["min_freeboard_m", "optimise_depth"],
        );
    }
    // A channel shallower than its own clearance leaves no water to float in.
    if let (Some(depth), Some(ukc)) = (
        get("channel_depth_m").and_then(Value::as_f64),
        get("channel_under_keel_clearance_m").and_then(Value::as_f64),
    ) && depth < ukc
    {
        report.error(
            at(format!(
                "`channel_depth_m` = {depth} must be at least `channel_under_keel_clearance_m` = \
             {ukc} (hard limit)"
            )),
            &["channel_depth_m", "channel_under_keel_clearance_m"],
        );
    }
}

/// A derived length as a person reads it: rounded to the micrometre, so float noise stays out of
/// the message.
fn show_m(x: f64) -> String {
    let r = (x * 1e6).round() / 1e6;
    format!("{}", if r == 0.0 { 0.0 } else { r })
}

/// Refuse inputs the original Python turns into a physically impossible result. See
/// [`DRY_BULK_WING_VOID_M`] for the rule.
fn impossible_results(
    inputs: &[(String, StoredInput)],
    report: &mut Report,
    at: &dyn Fn(String) -> String,
) {
    let get = |n: &str| inputs.iter().find(|(k, _)| k == n).map(|(_, i)| &i.value);
    let containment = get("cargo_containment").and_then(Value::as_str);
    if !matches!(containment, Some("box_hold" | "sloped_hopper_hold")) {
        return;
    }
    const NEGATIVE: &str = "a negative hold gives a negative cargo mass and draught (hard limit)";
    if let Some(beam) = get("beam_m").and_then(Value::as_f64) {
        let void = get("hold_wing_void_width_m")
            .and_then(Value::as_f64)
            .unwrap_or(DRY_BULK_WING_VOID_M);
        let width = beam - 2.0 * void;
        if width < 0.0 {
            report.error(
                at(format!(
                    "`beam_m` = {beam} leaves a hold width of {} m after the two \
                 {void} m wing voids; {NEGATIVE}",
                    show_m(width)
                )),
                &["beam_m", "hold_wing_void_width_m"],
            );
        }
    }
    if let Some(depth) = get("moulded_depth_m").and_then(Value::as_f64) {
        let double_bottom = get("hold_double_bottom_m")
            .and_then(Value::as_f64)
            .unwrap_or(DRY_BULK_DOUBLE_BOTTOM_M);
        let coaming = get("hatch_coaming_height_m")
            .and_then(Value::as_f64)
            .unwrap_or(DRY_BULK_COAMING_M);
        let height = (depth - double_bottom) + coaming;
        if height < 0.0 {
            report.error(
                at(format!(
                    "`moulded_depth_m` = {depth} leaves a hold height of {} m (depth less the \
                 {double_bottom} m double bottom, plus the {coaming} m \
                 coaming); {NEGATIVE}",
                    show_m(height)
                )),
                &[
                    "moulded_depth_m",
                    "hold_double_bottom_m",
                    "hatch_coaming_height_m",
                ],
            );
        }
    }
}

/// Check every default in a profile against its own input's type and hard range.
pub(crate) fn check_profile_defaults(profile: &Profile) -> Result<(), String> {
    for group in DEPTH_GROUPS {
        let ratio = profile
            .default_by("depth_to_beam_ratio", "cargo_group", group)
            .and_then(Value::as_f64)
            .ok_or_else(|| {
                format!(
                    "profile gives no depth-to-beam ratio for {group} Cargo \
                     (`depth_to_beam_ratio.default_by_cargo_group.{group}` is missing)"
                )
            })?;
        if let Some(bound) = profile.hard("depth_to_beam_ratio")?.breach(ratio) {
            return Err(format!(
                "profile default is invalid: `depth_to_beam_ratio` for {group} = {ratio} is out of range: must be {bound} (hard limit)"
            ));
        }
    }
    for spec in INPUTS.iter().filter(|s| !s.required) {
        let mut defaults: Vec<Value> = profile.default(spec.name).cloned().into_iter().collect();
        for (_, by, keys) in KEYED_DEFAULTS.iter().filter(|(n, _, _)| *n == spec.name) {
            for key in *keys {
                defaults.extend(profile.default_by(spec.name, by, key).cloned());
            }
        }
        for d in defaults {
            check_value(spec, &d, profile)
                .map_err(|e| format!("profile default is invalid: {e}"))?;
        }
    }
    Ok(())
}

/// An input value as a person reads it.
pub fn show(v: &Value) -> String {
    match v {
        Value::Null => "none".into(),
        Value::String(s) => s.clone(),
        Value::Number(n) => n.as_f64().map(|x| format!("{x}")).unwrap_or(n.to_string()),
        Value::Object(o) if o.contains_key("kind") => {
            let kind = o["kind"].as_str().unwrap_or("?");
            match (o.get("abreast"), o.get("in_line")) {
                (Some(a), Some(l)) => format!("{kind} ({a} abreast, {l} in line)"),
                _ => kind.into(),
            }
        }
        other => other.to_string(),
    }
}
