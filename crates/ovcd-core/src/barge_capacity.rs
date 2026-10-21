//! Unified barge design and volume-bound cargo capacity. Port of `core/barge_capacity.py`.
//!
//! Behaviour is copied exactly, including the errors and their message text. The reasoning
//! behind the design (what is derived, what is an estimate, why the light air draft governs)
//! lives in the Python file. Python's two-argument `max` and `min` keep their first-of-equal
//! rule (`py_max`, `py_min`), `x or 0.0` on a float is `truthy_or_zero`, every `**` is
//! `libm::pow` (inside the modules this one calls), and every expression keeps Python's
//! evaluation order so the rounding is the same.
//!
//! Only a `ValueError` out of the gas branch is caught: it becomes a failed `feasible_geometry`
//! check, as in Python. Any other exception, and every `ValueError` from anywhere else, comes
//! back as a [`PyError`]. A tank count that no 64-bit result field can hold is an
//! `OverflowError` with one fixed message, raised after the rest of the calculation succeeds
//! (the integer range rule: Python holds a count of any size, a 64-bit field cannot).

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use libm::{fabs, sqrt};

use crate::aft_arrangement::{
    AftArrangementInputs, AftArrangementResult, AftDesign, DEFAULT_ACCOM_UNITS_DRY,
    DEFAULT_ACCOM_UNITS_GAS, DEFAULT_BATTERY_UNITS_DRY, DEFAULT_BATTERY_UNITS_GAS, PyException,
    compute as compute_aft_arrangement,
};
use crate::cargo::{CargoClass, get_cargo};
use crate::checks::{Check, all_ok};
use crate::gas_separation::{
    GasSeparationInputs, GasSeparationResult, compute as compute_gas_separation,
};
use crate::hull::HullGeometry;
use crate::hydrostatics::find_equilibrium_draft;
use crate::models::{BargeInputs, TankResult, WeightBreakdown};
use crate::pressure_vessel::{PyError, get_tank_material};
use crate::protective_location::{compute_required_clearances, d_vc_exact};
use crate::pystr::repr_str;
use crate::tanks::{INT_OVERFLOW_MESSAGE, calculate_tank_arrangement_wide};
use crate::text;
use crate::text::{Arg, Text};

use crate::weights::calculate_weights;

/// Default cap on a gas tank's outer diameter (m). It is a design limit of this tool, not a rule of
/// the IGC Code: above it the engine goes to more tanks across the beam. The cap is an input
/// (`BargeCapacityInputs::max_tank_od_m`); issue #175 raised it from 12 m and made it one.
pub const DEFAULT_MAX_TANK_OD_M: f64 = 16.0;

/// Default bow rake length as a multiple of the moulded depth.
pub const DEFAULT_BOW_RAKE_TO_DEPTH_RATIO: f64 = 1.6;
/// Default number of gas tanks along the hull.
pub const DEFAULT_TANKS_IN_LINE: i64 = 2;
/// Default height of the tank saddles above the inner bottom (m).
pub const DEFAULT_TANK_SUPPORT_HEIGHT_M: f64 = 0.5;
/// Default dome height above the tank top (m).
pub const DEFAULT_TANK_DOME_HEIGHT_M: f64 = 2.0;
/// Default dome diameter (m), used only for the dome's volume in Vc. A nominal cylindrical dome as
/// wide as it is high (the default dome height is 2.0 m); there is no published source for it yet.
pub const DEFAULT_TANK_DOME_DIAMETER_M: f64 = 2.0;
/// The dome rule (issue #316, an estimate on two published tanks): dome diameter = the larger of
/// 1.2 m and 0.17 x the steel outer diameter, and dome height above the shell = the dome diameter.
/// Points: a dimensioned dome of 2,500 mm diameter, 2,500 mm from the inside of the shell to the
/// top, on a 14.6 m lobe (R7300; Heo, Kang and Park 2017, Fig. 19; 2.5 / 14.6 = 0.171), and the
/// 1,200 mm valve dome of a 4.73 m river LPG tank (Senjanovic et al., DESIGN 2006).
pub const DOME_RULE_MIN_DIAMETER_M: f64 = 1.2;
pub const DOME_RULE_DIAMETER_RATIO: f64 = 0.17;

/// The dome of a tank of steel outer diameter `steel_od_m` (m): `(diameter, height)`. A dimension
/// given as `Some` is used as it is; `None` takes the dome rule. The height by rule is the diameter
/// in force (typed or by rule).
pub fn dome_dimensions(
    steel_od_m: f64,
    diameter_m: Option<f64>,
    height_m: Option<f64>,
) -> (f64, f64) {
    let by_rule = DOME_RULE_DIAMETER_RATIO * steel_od_m;
    let d = diameter_m.unwrap_or(if by_rule > DOME_RULE_MIN_DIAMETER_M {
        by_rule
    } else {
        DOME_RULE_MIN_DIAMETER_M
    });
    (d, height_m.unwrap_or(d))
}

/// Default fixed tank wall thickness (mm).
pub const DEFAULT_TANK_WALL_THICKNESS_MM: f64 = 25.0;
/// The tank design values the engine used before they were inputs (`models::BargeInputs`
/// defaults): design pressure 0.5 MPa, 9 % nickel steel, 350 mm insulation, structure factor 1.30.
pub const DEFAULT_TANK_DESIGN_PRESSURE_MPA: f64 = 0.5;
pub const DEFAULT_TANK_MATERIAL: &str = "9Ni";
/// The tank head type the engine used before it was an input (issue #375).
pub const DEFAULT_TANK_HEAD_TYPE: &str = "hemisphere";
pub const DEFAULT_TANK_INSULATION_MM: f64 = 350.0;
pub const DEFAULT_TANK_STRUCTURE_FACTOR: f64 = 1.30;
/// Default gap between the wing tank and the cargo tank, per side (m).
pub const DEFAULT_WING_TANK_GAP_M: f64 = 0.6;
/// Default wing tank height as a fraction of the cargo tank outer diameter.
pub const DEFAULT_WING_TANK_HEIGHT_RATIO: f64 = 0.66;
/// Default aft void of a pushed barge (m).
pub const DEFAULT_PUSHED_STERN_VOID_M: f64 = 3.0;

/// How the design liquid pressure of a gas tank is worked out (issue #374).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LiquidPressure {
    /// No liquid pressure: the wall takes the vapour pressure only (the original engine).
    None,
    /// Gravity only, a = 1 (the default; an estimate for inland and sheltered service).
    Static,
    /// IGC 4.28.2: a = 1 + az, the guidance vertical acceleration for the North Atlantic, for a ship
    /// over 50 m at its service speed (knots).
    Igc4282 { service_speed_kn: f64 },
}

/// The design values of a gas tank that depend on the Cargo (issue #292): the pressure, the steel,
/// the insulation and the structure factor. `Default` is what the engine used before they were
/// inputs. The tank wall thickness is not here: it is `BargeCapacityInputs::tank_wall_thickness`
/// (`None`: the pressure-vessel formula gives it).
#[derive(Debug, Clone, PartialEq)]
pub struct TankDesign {
    /// Design pressure, MARVS (MPa).
    pub design_pressure_mpa: f64,
    /// Material key of `pressure_vessel::TANK_MATERIAL_DATABASE`.
    pub material: String,
    /// Allowable stress (MPa); `None`: the material's own.
    pub allowable_stress_mpa: Option<f64>,
    /// Insulation thickness (mm). The tank outer diameter is the outside of it.
    pub insulation_thickness_mm: f64,
    /// Internal stiffeners, supports and saddles as a multiple of the shell weight.
    pub structure_factor: f64,
    /// The liquid pressure added to the design pressure in the wall formula (issue #374).
    pub liquid_pressure: LiquidPressure,
    /// The lowest design temperature (C): the liquid pressure takes the cargo density there.
    pub design_temperature_c: Option<f64>,
    /// The tank head type, `"hemisphere"` or `"ellipsoidal_2to1"` (issue #375).
    pub head_type: String,
}

impl Default for TankDesign {
    fn default() -> Self {
        Self {
            design_pressure_mpa: DEFAULT_TANK_DESIGN_PRESSURE_MPA,
            material: DEFAULT_TANK_MATERIAL.to_string(),
            allowable_stress_mpa: None,
            insulation_thickness_mm: DEFAULT_TANK_INSULATION_MM,
            structure_factor: DEFAULT_TANK_STRUCTURE_FACTOR,
            liquid_pressure: LiquidPressure::None,
            design_temperature_c: None,
            head_type: DEFAULT_TANK_HEAD_TYPE.to_string(),
        }
    }
}

/// Inputs for the unified barge capacity calculation. [`BargeCapacityInputs::new`] gives the
/// Python dataclass defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct BargeCapacityInputs {
    // --- Scenario fundamentals ---
    /// Length overall (m)
    pub loa: f64,
    /// Moulded beam (m)
    pub beam: f64,
    /// Moulded depth (m)
    pub depth: f64,
    /// Key into the cargo database (gas, or "bauxite")
    pub cargo_type: String,

    // --- Common assumptions ---
    pub block_coefficient: f64,
    /// t/m^3
    pub water_density: f64,
    /// Minimum freeboard (m)
    pub required_freeboard: f64,
    /// Bridge clearance limit (m), checked against the LIGHT condition.
    pub air_draft_limit_m: Option<f64>,
    /// Ballast the empty leg can carry (t)
    pub ballast_capacity_t: f64,

    // --- Arrangement ---
    pub aft: Option<AftArrangementInputs>,
    /// The design values of the default stern block (thrusters, units, wheelhouse). Used only
    /// when `aft` is `None`; a given `aft` is used as it is.
    pub aft_design: AftDesign,
    pub separation: Option<GasSeparationInputs>,
    /// A self-propelled vessel carries the stern block; a pushed barge does not.
    pub self_propelled: bool,
    /// Issue #180. Propulsion of the default stern block, one of
    /// [`crate::aft_arrangement::PROPULSION_OPTIONS`]. Used only when `aft` is `None`; a given
    /// `aft` must agree.
    pub propulsion: String,
    /// Issue #180. Electrical load (kWe) the gensets carry with one set in reserve; selects the
    /// genset from the catalogue. Gas-fuelled only.
    pub electrical_load_kwe: Option<f64>,
    /// Aft void of a pushed barge (m)
    pub pushed_stern_void_m: f64,
    /// Bow bottom rake (m). `None`: `bow_rake_to_depth_ratio` x moulded depth.
    pub bow_rake_length_m: Option<f64>,
    /// The bow rake as a multiple of the moulded depth, when `bow_rake_length_m` is `None`.
    pub bow_rake_to_depth_ratio: f64,
    /// Dry-bulk end-bulkhead allowance at each end of the hold (m)
    pub bulk_end_bulkhead_m: f64,

    // --- Gas branch (IGC/IGF) ---
    pub vessel_type: String,
    pub side_clearance_override_m: Option<f64>,
    /// `None`: auto, the fewest tanks across that keeps each tank's OD within the cap.
    pub n_tanks_wide: Option<i64>,
    pub n_tanks_long: i64,
    /// Cap on a tank's outer diameter (m): sets the automatic tank count across the beam.
    pub max_tank_od_m: f64,
    /// mm; `None`: auto pressure-vessel formula
    pub tank_wall_thickness: Option<f64>,
    /// The Cargo-dependent tank design values (issue #292).
    pub tank_design: TankDesign,
    pub fill_ratio: f64,
    /// Cargo density (kg/m^3) in place of the cargo data's (issue #315: an LPG Cargo at its loading
    /// temperature); `None`: the cargo data's.
    pub cargo_density_kg_m3: Option<f64>,
    /// Issue #348: `Some` puts the stern block's mass into the lightship of a self-propelled hull;
    /// `None` leaves it out, as before.
    pub stern_block_mass: Option<crate::aft_arrangement::SternMassInputs>,
    /// Issue #350: the Stern Form, when it is not today's square stern: it corrects the LCB of the
    /// trim. `None`: a square stern.
    pub stern_geometry: Option<crate::stern_form::SternGeometry>,
    /// Issue #354: `Some(limit)` checks the sightline as a Blind Distance (ES-TRIN 7.02(2)) in the
    /// light condition, and lifts the wheelhouse to meet it; `None`: today's margin over the cargo.
    pub sightline_limit_m: Option<f64>,
    /// Issue #352: the machinery of a specified Propulsor; it sets the stern block's run below deck
    /// and its mass items. `None`: today's stern block.
    pub machinery: Option<crate::machinery::MachineryInputs>,
    pub dome_height: f64,
    /// Diameter of the cylindrical dome (m). With `dome_height` it gives the dome's volume, which
    /// counts in Vc for d(Vc) only (IGC 2.4.1.1: Vc includes domes and appendages), not in the
    /// cargo volume, the fill or the weights.
    pub dome_diameter: f64,
    /// Issue #316: `true` takes the dome diameter, or the dome height, from the dome rule
    /// ([`dome_dimensions`]) of each tank instead of the fixed value above.
    pub dome_diameter_by_rule: bool,
    pub dome_height_by_rule: bool,
    pub tank_support_height: f64,
    /// Gap between the wing tank and the cargo tank, per side (m).
    pub wing_tank_gap_m: f64,
    /// Wing tank height as a fraction of the cargo tank outer diameter.
    pub wing_tank_height_ratio: f64,

    // --- Bulk branch ---
    /// "sloped_hopper", "box" or "container"
    pub hold_type: String,
    pub lightship_k_t_per_m3: Option<f64>,
    pub wing_void_width: f64,
    pub double_bottom_height_bulk: f64,
    pub hopper_slope_h_per_v: f64,
    pub coaming_height: f64,
    pub n_holds: i64,
    pub hold_fill_ratio: f64,
    pub bow_void_fraction: f64,
    pub stern_void_fraction: f64,
    pub inner_bottom_t_mm: f64,
    pub hopper_plate_t_mm: f64,
    pub bulkhead_t_mm: f64,
    pub coaming_t_mm: f64,
    pub steel_density: f64,
    pub bulk_structure_factor: f64,
    pub bulk_steel_rate_per_cbm: f64,
    pub bulk_piping_factor: f64,
    pub bulk_outfitting_factor: f64,
}

impl BargeCapacityInputs {
    /// The dataclass defaults, with the three required principal dimensions.
    pub fn new(loa: f64, beam: f64, depth: f64) -> Self {
        Self {
            loa,
            beam,
            depth,
            cargo_type: "LNG_ethane".to_string(),
            block_coefficient: 0.85,
            water_density: 1.0,
            required_freeboard: 0.3,
            air_draft_limit_m: None,
            ballast_capacity_t: 0.0,
            aft: None,
            aft_design: AftDesign::default(),
            separation: None,
            self_propelled: true,
            propulsion: "battery_electric".to_string(),
            electrical_load_kwe: None,
            pushed_stern_void_m: DEFAULT_PUSHED_STERN_VOID_M,
            bow_rake_length_m: None,
            bow_rake_to_depth_ratio: DEFAULT_BOW_RAKE_TO_DEPTH_RATIO,
            bulk_end_bulkhead_m: 0.50,
            vessel_type: "IGC_2G".to_string(),
            side_clearance_override_m: None,
            n_tanks_wide: None,
            n_tanks_long: DEFAULT_TANKS_IN_LINE,
            max_tank_od_m: DEFAULT_MAX_TANK_OD_M,
            tank_wall_thickness: Some(DEFAULT_TANK_WALL_THICKNESS_MM),
            tank_design: TankDesign::default(),
            fill_ratio: 0.95,
            cargo_density_kg_m3: None,
            stern_block_mass: None,
            stern_geometry: None,
            sightline_limit_m: None,
            machinery: None,
            dome_height: DEFAULT_TANK_DOME_HEIGHT_M,
            dome_diameter: DEFAULT_TANK_DOME_DIAMETER_M,
            dome_diameter_by_rule: false,
            dome_height_by_rule: false,
            tank_support_height: DEFAULT_TANK_SUPPORT_HEIGHT_M,
            wing_tank_gap_m: DEFAULT_WING_TANK_GAP_M,
            wing_tank_height_ratio: DEFAULT_WING_TANK_HEIGHT_RATIO,
            hold_type: "sloped_hopper".to_string(),
            lightship_k_t_per_m3: None,
            wing_void_width: 1.0,
            double_bottom_height_bulk: 1.2,
            hopper_slope_h_per_v: 1.0,
            coaming_height: 1.0,
            n_holds: 3,
            hold_fill_ratio: 0.95,
            bow_void_fraction: 0.10,
            stern_void_fraction: 0.15,
            inner_bottom_t_mm: 14.0,
            hopper_plate_t_mm: 14.0,
            bulkhead_t_mm: 11.0,
            coaming_t_mm: 10.0,
            steel_density: 7.85,
            bulk_structure_factor: 1.40,
            bulk_steel_rate_per_cbm: 0.10,
            bulk_piping_factor: 0.04,
            bulk_outfitting_factor: 0.035,
        }
    }
}

/// The values of one Type C tank that the IGC Rules read (spec 0004, `igc-2g`), exactly as the
/// arrangement worked with them. Every tank of an arrangement is alike.
#[derive(Debug, Clone, PartialEq)]
pub struct TankRuleValues {
    /// Tanks in the arrangement.
    pub n_tanks: i64,
    /// Vc of one tank (IGC 2.4.1.1): its gross volume plus the dome, m^3.
    pub vc_m3: f64,
    /// The clear gap from the moulded side to the outer face of the insulation, m: the arrangement
    /// sizes the tank diameter from it.
    pub side_gap_m: f64,
    /// The clear height from the moulded base to the outer face of the insulation, m: the double
    /// bottom plus the support height.
    pub bottom_gap_m: f64,
    /// The insulation thickness, m.
    pub insulation_m: f64,
    pub cylinder_wall_mm: f64,
    pub head_wall_mm: f64,
    pub weld_factor: f64,
    /// The allowable stress f the wall formula used, MPa.
    pub allowable_stress_mpa: f64,
    pub material: String,
    /// Rm and Re of the material, MPa (IGC 4.18.1.3).
    pub tensile_strength_mpa: f64,
    pub yield_strength_mpa: f64,
}

/// The converged gas arrangement and its weight split, for the gas stability check.
#[derive(Debug, Clone, PartialEq)]
pub struct GasBasis {
    pub inputs: BargeInputs,
    pub tanks: TankResult,
    pub weights: WeightBreakdown,
}

/// Volume-bound capacity and the resulting full-load hydrostatics.
#[derive(Debug, Clone, PartialEq)]
pub struct BargeCapacityResult {
    pub cargo_type: String,
    /// The Cargo Class (`liquid_gas` or `dry_bulk` in the results).
    pub phase: CargoClass,
    pub cargo_density_t_per_m3: f64,

    // Headline outputs
    pub cargo_capacity_t: f64,
    pub full_load_draft_m: f64,
    pub air_draft_m: f64,

    // Supporting
    pub cargo_volume_m3: f64,
    pub lightship_t: f64,
    pub displacement_t: f64,
    pub deadweight_t: f64,
    pub freeboard_m: f64,
    pub air_draft_top_above_keel_m: f64,
    pub overloaded: bool,

    // Gas design detail (`None` for bulk)
    pub tank_outer_diameter_m: Option<f64>,
    pub n_tanks_wide: Option<i64>,
    pub max_tank_od_m: Option<f64>,
    pub n_tanks_total: Option<i64>,
    pub side_clearance_m: Option<f64>,
    pub bottom_clearance_m: Option<f64>,

    // Bulk design detail (`None` for gas)
    pub hold_height_m: Option<f64>,
    pub hold_top_width_m: Option<f64>,
    pub hold_bottom_width_m: Option<f64>,
    pub hold_section_area_m2: Option<f64>,

    /// Transverse metacentric height (bulk hopper only)
    pub gm_m: Option<f64>,

    // --- Arrangement ---
    pub bow_void_length_m: Option<f64>,
    pub stern_void_length_m: Option<f64>,
    pub cargo_zone_length_m: Option<f64>,
    pub bow_rake_length_m: Option<f64>,
    pub aft_arrangement: Option<AftArrangementResult>,
    pub separation: Option<GasSeparationResult>,

    // --- Cargo volume build-up ---
    pub containment_inner_volume_m3: Option<f64>,
    pub fill_ratio_applied: Option<f64>,
    pub insulation_thickness_mm: Option<f64>,
    pub tank_wall_thickness_mm: Option<f64>,
    pub tank_inner_diameter_m: Option<f64>,
    /// The tank values the IGC Rules of the Rule Trace read (gas only). Not in the result JSON:
    /// the trace reads them typed.
    pub tank_rules: Option<TankRuleValues>,

    // --- Light / ballast load conditions ---
    pub light_draft_m: Option<f64>,
    pub air_draft_light_m: Option<f64>,
    pub ballast_draft_m: Option<f64>,
    pub air_draft_ballast_m: Option<f64>,
    pub air_draft_navigating_light_m: Option<f64>,
    pub navigating_top_above_keel_m: Option<f64>,

    /// Gas only: the converged arrangement, for gas stability.
    /// Issue #348: the stern block's mass items when they are in the lightship (empty otherwise).
    pub stern_block_items: Vec<crate::weights::MassItem>,
    /// Issue #349: the trim of a dry-bulk hull, laden and light, when the stern block is in the
    /// lightship (empty otherwise; a gas hull has it per stability condition).
    pub trim_conditions: Vec<(&'static str, crate::models::TrimResult)>,
    /// Issue #350: the Stern Form the result was worked with (`None`: a square stern).
    pub stern_geometry: Option<crate::stern_form::SternGeometry>,
    /// Issue #354: the sightline as a Blind Distance in the light condition, and the laden Blind
    /// Distance at the same lift; `None` with the margin method or no wheelhouse.
    pub sightline: Option<crate::wheelhouse::Sightline>,
    pub blind_distance_laden_m: Option<f64>,
    pub gas_basis: Option<GasBasis>,

    pub checks: Vec<Check>,
    pub warnings: Vec<Text>,
}

impl BargeCapacityResult {
    /// The Python `ok` property.
    pub fn ok(&self) -> bool {
        all_ok(&self.checks)
    }
}

/// Python `max(a, b)`: the first of two equal items; `b` only if `b > a`.
fn py_max(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// Python `min(a, b)`: the first of two equal items; `b` only if `b < a`.
fn py_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// Python `x or 0.0` on a float: zero is false, so it gives `0.0`; NaN is true.
fn truthy_or_zero(x: f64) -> f64 {
    if x == 0.0 { 0.0 } else { x }
}

fn value_error(message: String) -> PyError {
    PyError {
        kind: "ValueError",
        message,
    }
}

fn zero_division() -> PyError {
    PyError {
        kind: "ZeroDivisionError",
        message: "float division by zero".to_string(),
    }
}

fn overflow() -> PyError {
    PyError {
        kind: "OverflowError",
        message: INT_OVERFLOW_MESSAGE.to_string(),
    }
}

impl From<PyException> for PyError {
    fn from(e: PyException) -> Self {
        PyError {
            kind: e.type_name(),
            message: e.message().to_string(),
        }
    }
}

/// Fewest tanks across the beam that keeps each tank's OD within `max_od_m`.
///
/// Inverts the arrangement geometry in `tanks::calculate_tank_arrangement`:
/// `OD = (beam - 2*cs - (n-1)*gap) / n <= max_od`, so `n >= (beam - 2*cs + gap) / (max_od + gap)`.
/// Always at least 1, so a narrow hull is unaffected.
///
/// `ZeroDivisionError` when `max_od_m + gap` is exactly zero. `math.ceil` raises `ValueError` on
/// NaN and `OverflowError` on infinity. A count above the 64-bit range is the fixed
/// `OverflowError`; a ceiling below 1 is 1, however negative.
pub fn tanks_wide_for_beam(
    beam_m: f64,
    side_clearance_m: f64,
    tank_gap_transverse_m: f64,
    max_od_m: f64,
) -> Result<i64, PyError> {
    let usable = beam_m - 2.0 * side_clearance_m;
    if usable <= 0.0 {
        return Ok(1);
    }
    let divisor = max_od_m + tank_gap_transverse_m;
    if divisor == 0.0 {
        return Err(zero_division());
    }
    let ratio = (usable + tank_gap_transverse_m) / divisor;
    if ratio.is_nan() {
        return Err(value_error(
            "cannot convert float NaN to integer".to_string(),
        ));
    }
    if ratio.is_infinite() {
        return Err(PyError {
            kind: "OverflowError",
            message: "cannot convert float infinity to integer".to_string(),
        });
    }
    let n = libm::ceil(ratio);
    if n < 1.0 {
        return Ok(1);
    }
    // 2^63: the first float that no `i64` holds.
    if n >= 9_223_372_036_854_775_808.0 {
        return Err(overflow());
    }
    Ok(n as i64)
}

/// What the volume branch reports besides the four numbers it returns.
#[derive(Debug, Clone, Default)]
struct Detail {
    tank_outer_diameter_m: Option<f64>,
    n_tanks_wide: Option<i64>,
    max_tank_od_m: Option<f64>,
    n_tanks_total: Option<i64>,
    side_clearance_m: Option<f64>,
    bottom_clearance_m: Option<f64>,
    hold_height_m: Option<f64>,
    hold_top_width_m: Option<f64>,
    hold_bottom_width_m: Option<f64>,
    hold_section_area_m2: Option<f64>,
    containment_inner_volume_m3: Option<f64>,
    fill_ratio_applied: Option<f64>,
    insulation_thickness_mm: Option<f64>,
    tank_wall_thickness_mm: Option<f64>,
    tank_inner_diameter_m: Option<f64>,
    gas_basis: Option<GasBasis>,
    tank_rules: Option<TankRuleValues>,
}

/// (cargo volume, lightship, top above keel, detail) of a volume branch. `n_total` is the exact
/// tank count product of the gas branch, which no 64-bit field may hold.
struct Volumetric {
    cargo_volume: f64,
    lightship: f64,
    h_top: f64,
    detail: Detail,
    n_total: i128,
}

/// Volume of one cylindrical dome (m^3): pi/4 x diameter^2 x height. It counts in Vc for d(Vc)
/// only (IGC 2.4.1.1: Vc includes domes and appendages).
pub fn dome_volume_m3(diameter_m: f64, height_m: f64) -> f64 {
    core::f64::consts::FRAC_PI_4 * diameter_m * diameter_m * height_m
}

/// The IGC 4.28.2 guidance vertical acceleration az (no static weight), at a tank whose centre of
/// gravity is `x_m` from amidships (forward positive), on the centreline (y = 0, K = 1):
/// az = a0 x sqrt(1 + (5.3 - 45/L0)^2 (x/L0 + 0.05)^2 (0.6/CB)^1.5), a0 = 0.2 V / sqrt(L0) +
/// (34 - 600/L0) / L0, with L0 the length (m), CB the block coefficient and V the speed (knots).
pub fn igc_vertical_acceleration(l0_m: f64, cb: f64, speed_kn: f64, x_m: f64) -> f64 {
    let a0 = 0.2 * speed_kn / libm::sqrt(l0_m) + (34.0 - 600.0 / l0_m) / l0_m;
    let k = 5.3 - 45.0 / l0_m;
    let r = x_m / l0_m + 0.05;
    a0 * libm::sqrt(1.0 + k * k * r * r * libm::pow(0.6 / cb, 1.5))
}

/// The acceleration a and the density rho (kg/m^3) of the design liquid pressure of a gas design
/// (issue #374). rho is the largest cargo density at the design temperature: the saturated
/// liquid density there for an LPG Cargo inside the tables, else the cargo density in force.
/// For IGC 4.28.2 the tank is the one farthest from amidships, `n_long` tanks of equal length
/// between the voids with `gap_m` between them.
pub fn liquid_pressure_basis(
    inp: &BargeCapacityInputs,
    bow_void_m: f64,
    stern_void_m: f64,
    gap_m: f64,
) -> Result<(Option<f64>, f64), PyError> {
    let props = get_cargo(&inp.cargo_type).map_err(value_error)?;
    let in_force = inp.cargo_density_kg_m3.unwrap_or(props.density);
    let rho = inp
        .tank_design
        .design_temperature_c
        .and_then(|t| crate::lpg_saturation::liquid_density_kg_m3(&inp.cargo_type, t))
        .map_or(in_force, |d| if d > in_force { d } else { in_force });
    let accel = match inp.tank_design.liquid_pressure {
        LiquidPressure::None => None,
        LiquidPressure::Static => Some(1.0),
        LiquidPressure::Igc4282 { service_speed_kn } => {
            let n = if inp.n_tanks_long > 0 {
                inp.n_tanks_long
            } else {
                1
            };
            let zone = inp.loa - bow_void_m - stern_void_m;
            let length = (zone - (n - 1) as f64 * gap_m) / n as f64;
            let mut worst = 0.0;
            for i in 0..n {
                let centre = stern_void_m + (i as f64 + 0.5) * length + i as f64 * gap_m;
                let az = igc_vertical_acceleration(
                    inp.loa,
                    inp.block_coefficient,
                    service_speed_kn,
                    centre - inp.loa / 2.0,
                );
                if az > worst {
                    worst = az;
                }
            }
            Some(1.0 + worst)
        }
    };
    Ok((accel, rho))
}

/// IGC/IGF protective-location clearances, then the tank arrangement, then cargo volume.
///
/// Resolves the volume and d(Vc) circular dependency by iterating.
fn gas_volumetric(
    inp: &BargeCapacityInputs,
    bow_void_m: f64,
    stern_void_m: f64,
) -> Result<Volumetric, PyError> {
    let mut base = BargeInputs::new(inp.loa, inp.beam, inp.depth, inp.depth, inp.depth * 10.0);
    base.vessel_type = inp.vessel_type.clone();
    base.cargo_type = inp.cargo_type.clone();
    // The cargo zone is bounded by the derived voids, not by a fraction of LOA.
    base.bow_non_tank_length = Some(bow_void_m);
    base.stern_non_tank_length = Some(stern_void_m);
    // `inp.n_tanks_wide or 1`
    base.n_tanks_wide = match inp.n_tanks_wide {
        Some(n) if n != 0 => n,
        _ => 1,
    };
    base.n_tanks_long = inp.n_tanks_long;
    base.tank_wall_thickness = inp.tank_wall_thickness;
    base.design_pressure = inp.tank_design.design_pressure_mpa;
    base.tank_head_type = inp.tank_design.head_type.clone();
    // Issue #374: the design liquid pressure (IGC 4.23.2.2, 4.28.1.2) joins the wall formula.
    let (accel, rho) =
        liquid_pressure_basis(inp, bow_void_m, stern_void_m, base.tank_gap_longitudinal)?;
    base.liquid_pressure_accel = accel;
    base.liquid_pressure_density = rho;
    base.tank_material = inp.tank_design.material.clone();
    base.tank_allowable_stress = inp.tank_design.allowable_stress_mpa;
    // The wall formula takes the steel diameter, inside the insulation (issue #292).
    base.wall_formula_on_steel_diameter = true;
    base.insulation_thickness = inp.tank_design.insulation_thickness_mm;
    base.tank_structure_factor = inp.tank_design.structure_factor;
    // The mass of the steel is the material's, whether the wall is fixed or from the formula. (The
    // dataclass default is 9Ni's 7.85 t/m3, so a design on the default material is unchanged.)
    base.tank_material_density = get_tank_material(&inp.tank_design.material)?.density;
    base.fill_ratio = inp.fill_ratio;
    base.dome_height = inp.dome_height;
    base.tank_support_height = inp.tank_support_height;
    base.wing_tank_gap = inp.wing_tank_gap_m;
    base.wing_tank_height_ratio = inp.wing_tank_height_ratio;
    base.post_init()?;

    let mut d_vc = 1.0;
    let mut bi = base.clone();
    let mut tanks: Option<(TankResult, i128)> = None;
    let mut cs = 0.0;
    let mut cb = 0.0;
    let mut db = 0.0;
    // IGC 2.4.2: d is measured plate to plate, from the moulded line to the moulded line,
    // excluding insulation. So the insulation lies inside d and the clear gap needed from the
    // engine's datum (the outer face of the insulation, which the tank OD includes) is d less it.
    let ins_m = truthy_or_zero(base.insulation_thickness) / 1000.0;
    // IGC 2.4.1.1: Vc is 100 % of the gross design volume of the individual cargo tank at 20 C,
    // including domes and appendages. The dome is a cylinder of `dome_diameter` and `dome_height`.
    // Appendages (nozzles, manways and the like) are not modelled: no volume is added for them.
    // The dome counts in Vc only, not in the cargo volume, the fill or the weights. By the dome rule
    // (issue #316) its size follows the steel diameter of the tank, so it is set in the loop.
    let dome_of = |outer_diameter: f64| {
        dome_dimensions(
            outer_diameter - 2.0 * ins_m,
            (!inp.dome_diameter_by_rule).then_some(inp.dome_diameter),
            (!inp.dome_height_by_rule).then_some(inp.dome_height),
        )
    };
    let mut dome = (inp.dome_diameter, inp.dome_height);
    let is_2g = inp.vessel_type == "IGC_2G";
    // d(Vc) depends on the tank volume, which depends on the clearance: iterate. d is continuous
    // and piecewise linear in Vc with a slope of at most 5e-5 per m^3, and a larger d gives a
    // smaller tank, so the map is a contraction and settles in a few rounds.
    for _ in 0..40 {
        if is_2g {
            cs = py_max(d_vc - ins_m, 0.0);
            cb = py_max(py_min(inp.beam / 15.0, 2.0), d_vc) - ins_m;
        } else {
            let req = compute_required_clearances(&inp.vessel_type, inp.beam, d_vc)
                .map_err(value_error)?;
            cs = req.cs_required;
            cb = req.cb_required;
        }
        if let Some(over) = inp.side_clearance_override_m {
            cs = over;
        }
        db = py_max(0.0, cb - inp.tank_support_height);
        // AUTO tank arrangement: cs is only known inside this loop.
        let n_wide = match inp.n_tanks_wide {
            Some(n) => n,
            None => tanks_wide_for_beam(
                inp.beam,
                cs,
                truthy_or_zero(base.tank_gap_transverse),
                inp.max_tank_od_m,
            )?,
        };
        // `dataclasses.replace`: copy, set, run `__post_init__` again.
        bi = base.clone();
        bi.side_clearance = Some(cs);
        // The wing ballast tank fills the governing side clearance inboard of the cargo-tank gap
        // (IGC 2.4.1.2 sets the clearance; maintainer decision on issue #283, built in #277). It
        // took the 1.0 m default before, whatever clearance the rule gave.
        bi.wing_tank_width = Some(py_max(0.0, cs - inp.wing_tank_gap_m));
        bi.double_bottom_height = Some(db);
        bi.n_tanks_wide = n_wide;
        bi.post_init()?;
        let (t, n_total) = calculate_tank_arrangement_wide(&bi)?;
        dome = dome_of(t.outer_diameter);
        bi.dome_height = dome.1;
        // Per-tank Vc (IGC 2.4.1.1), tank plus dome.
        let count = if n_total > 1 { n_total } else { 1 };
        let new_d_vc =
            d_vc_exact(t.total_tank_volume / (count as f64) + dome_volume_m3(dome.0, dome.1));
        tanks = Some((t, n_total));
        if fabs(new_d_vc - d_vc) < 1e-9 {
            break;
        }
        d_vc = new_d_vc;
    }
    let (tanks, n_total) = tanks.expect("the loop runs at least once");

    let hull = HullGeometry::new(inp.loa, inp.beam, inp.depth, 0.0, 0.0, 0.0, 0.0);
    let weights = calculate_weights(&bi, &hull, &tanks)?;
    let lightship = weights.lightship;
    let h_top = db + inp.tank_support_height + tanks.outer_diameter + dome.1;
    let detail = Detail {
        tank_outer_diameter_m: Some(tanks.outer_diameter),
        n_tanks_wide: Some(tanks.n_tanks_wide),
        max_tank_od_m: Some(inp.max_tank_od_m),
        n_tanks_total: Some(tanks.n_tanks_total),
        side_clearance_m: Some(cs),
        bottom_clearance_m: Some(cb),
        containment_inner_volume_m3: Some(tanks.total_tank_volume),
        fill_ratio_applied: Some(inp.fill_ratio),
        insulation_thickness_mm: Some(bi.insulation_thickness),
        tank_wall_thickness_mm: Some(tanks.cylinder_wall_thickness),
        tank_inner_diameter_m: Some(tanks.inner_diameter),
        tank_rules: Some({
            let mat = get_tank_material(&bi.tank_material)?;
            let count = if n_total > 1 { n_total } else { 1 };
            TankRuleValues {
                n_tanks: tanks.n_tanks_total,
                // The same expression as the last round of the clearance loop.
                vc_m3: tanks.total_tank_volume / (count as f64) + dome_volume_m3(dome.0, dome.1),
                side_gap_m: cs,
                bottom_gap_m: db + inp.tank_support_height,
                insulation_m: ins_m,
                cylinder_wall_mm: tanks.cylinder_wall_thickness,
                head_wall_mm: tanks.head_wall_thickness,
                weld_factor: bi.weld_efficiency,
                allowable_stress_mpa: bi.tank_allowable_stress.unwrap_or(mat.allowable_stress),
                material: bi.tank_material.clone(),
                tensile_strength_mpa: mat.tensile_strength,
                yield_strength_mpa: mat.yield_strength,
            }
        }),
        gas_basis: Some(GasBasis {
            // The two calculation-only switches of issue #292 go back to their defaults: the stored
            // inputs are the ones the Python API's `BargeInputs` has, and a result written as JSON
            // and read back must be the same result.
            inputs: BargeInputs {
                wall_formula_on_steel_diameter: false,
                tank_allowable_stress: None,
                liquid_pressure_accel: None,
                liquid_pressure_density: 0.0,
                ..bi
            },
            tanks: tanks.clone(),
            weights,
        }),
        ..Detail::default()
    };
    Ok(Volumetric {
        cargo_volume: tanks.cargo_volume,
        lightship,
        h_top,
        detail,
        n_total,
    })
}

/// Trapezoidal hopper hold geometry, then cargo volume and hopper structural weight.
fn bulk_volumetric(
    inp: &BargeCapacityInputs,
    bow_void_m: f64,
    stern_void_m: f64,
) -> Result<Volumetric, PyError> {
    let d_db = inp.double_bottom_height_bulk;
    let s = inp.hopper_slope_h_per_v;
    let h_hold = (inp.depth - d_db) + inp.coaming_height;
    let w_top = inp.beam - 2.0 * inp.wing_void_width;
    let w_bot = if inp.hold_type == "sloped_hopper" {
        py_max(1.0, w_top - 2.0 * s * h_hold)
    } else if inp.hold_type == "box" || inp.hold_type == "container" {
        w_top // vertical inner sides
    } else {
        return Err(value_error(format!(
            "unknown hold_type {}",
            repr_str(&inp.hold_type)
        )));
    };
    let area = 0.5 * (w_top + w_bot) * h_hold;
    let l_cargo = py_max(0.0, inp.loa - bow_void_m - stern_void_m);
    let hold_volume = area * l_cargo;
    let cargo_volume = hold_volume * inp.hold_fill_ratio;

    // --- Hopper structural steel (plating-area method) ---
    let t_ib = inp.inner_bottom_t_mm / 1000.0;
    let t_hop = inp.hopper_plate_t_mm / 1000.0;
    let t_bhd = inp.bulkhead_t_mm / 1000.0;
    let t_coam = inp.coaming_t_mm / 1000.0;
    let inner_bottom = l_cargo * inp.beam * t_ib;
    let hopper_slopes = 2.0 * (h_hold * sqrt(s * s + 1.0)) * l_cargo * t_hop;
    let wing_bulkheads = 2.0 * h_hold * l_cargo * t_bhd;
    // `(n_holds + 1)` is an exact integer that Python rounds once to a float.
    let holds_plus_one = (i128::from(inp.n_holds) + 1) as f64;
    let transverse_bhd = holds_plus_one * area * t_bhd;
    let coaming = 2.0 * inp.coaming_height * l_cargo * t_coam;
    let steel_vol = inner_bottom + hopper_slopes + wing_bulkheads + transverse_bhd + coaming;
    let hopper_steel = steel_vol * inp.steel_density * inp.bulk_structure_factor;

    let hull_steel_td = inp.bulk_steel_rate_per_cbm * inp.loa * inp.beam * inp.depth;
    let base_steel = hull_steel_td + hopper_steel;
    let mut lightship = base_steel * (1.0 + inp.bulk_piping_factor + inp.bulk_outfitting_factor);
    if inp.hold_type != "sloped_hopper" {
        let k = inp.lightship_k_t_per_m3.ok_or_else(|| {
            value_error(format!(
                "hold_type {} needs lightship_k_t_per_m3 (see calibration/factors.yaml)",
                repr_str(&inp.hold_type)
            ))
        })?;
        lightship = k * inp.loa * inp.beam * inp.depth;
    }

    let h_top = inp.depth + inp.coaming_height;
    let detail = Detail {
        hold_height_m: Some(h_hold),
        hold_top_width_m: Some(w_top),
        hold_bottom_width_m: Some(w_bot),
        hold_section_area_m2: Some(area),
        containment_inner_volume_m3: Some(hold_volume),
        fill_ratio_applied: Some(inp.hold_fill_ratio),
        ..Detail::default()
    };
    Ok(Volumetric {
        cargo_volume,
        lightship,
        h_top,
        detail,
        n_total: 0,
    })
}

/// First-order transverse intact GM for the bulk hopper: `GM = KB + BM_t - KG`.
///
/// Solid bulk has no liquid free surface. KG combines lightship (at 0.5 x moulded depth) and
/// cargo (the centroid of the filled trapezoidal hold above the double bottom). `hold` is the
/// hold's top width, bottom width and height.
fn bulk_gm(
    inp: &BargeCapacityInputs,
    (w_top, w_bot, h): (f64, f64, f64),
    draft: f64,
    (lightship, stern): (f64, &[crate::weights::MassItem]),
    cargo_t: f64,
    hull: &HullGeometry,
) -> f64 {
    // Trapezoid centroid above its base: h/3 * (2*top + bot)/(top + bot).
    let zc = if (w_top + w_bot) > 0.0 {
        (h / 3.0) * (2.0 * w_top + w_bot) / (w_top + w_bot)
    } else {
        h / 2.0
    };
    let kg_cargo = inp.double_bottom_height_bulk + zc;
    let kg_light = 0.5 * inp.depth;
    let total = lightship + cargo_t;
    // Issue #348: the stern block items (in `lightship`) at their own heights; none, as before.
    let kg = if stern.is_empty() {
        if total > 0.0 {
            (lightship * kg_light + cargo_t * kg_cargo) / total
        } else {
            kg_light
        }
    } else {
        let block = crate::weights::total_mass(stern);
        ((lightship - block) * kg_light
            + crate::weights::vertical_moment(stern)
            + cargo_t * kg_cargo)
            / total
    };
    hull.KB(draft) + hull.BM_transverse(draft) - kg
}

/// The note that goes with the first-order dry bulk GM.
pub fn bulk_gm_note() -> Text {
    Text::fixed("warn-barge-bulk-gm", BULK_GM_WARNING)
}

/// The first-order GM of a dry bulk hold at `cargo_t` and `draft_m` (issue #415): the cargo and the
/// draught the Design reports, which a draught limit can make less than the volume-bound load. The
/// cargo stays at the centroid of the full hold: a smaller load sits lower, so this KG is on the
/// safe side. `r` is the capacity result of `inp`.
pub fn bulk_gm_at(
    inp: &BargeCapacityInputs,
    r: &BargeCapacityResult,
    cargo_t: f64,
    draft_m: f64,
) -> f64 {
    let hull = HullGeometry::new(inp.loa, inp.beam, inp.depth, 0.0, 0.0, 0.0, 0.0);
    let hold = (
        r.hold_top_width_m.unwrap_or(f64::NAN),
        r.hold_bottom_width_m.unwrap_or(f64::NAN),
        r.hold_height_m.unwrap_or(f64::NAN),
    );
    bulk_gm(
        inp,
        hold,
        draft_m,
        (r.lightship_t, &r.stern_block_items),
        cargo_t,
        &hull,
    )
}

/// `replace(aft_in, sightline_obstruction_m=...)` and friends keep the other fields.
struct Arrangement {
    bow_void_m: f64,
    stern_void_m: f64,
    rake_m: f64,
    aft_inputs: AftArrangementInputs,
    separation: Option<GasSeparationResult>,
}

/// Resolve the two non-cargo end lengths from the equipment and the code, not a percentage.
///
/// The aft block's length does not depend on the cargo, so it is resolved here; only the
/// wheelhouse lift does, and that is a second pass once the cargo top is known.
fn arrangement(inp: &BargeCapacityInputs, is_gas: bool) -> Result<Arrangement, PyError> {
    // The mix of 40 ft units differs by cargo: dry bulk carries a second battery, gas a second
    // accommodation unit.
    // Gas-fuelled (issue #180): no battery; the gensets burn the cargo boil-off.
    let mut aft_in = match &inp.aft {
        Some(a) => {
            if inp.propulsion != "battery_electric" && a.propulsion != inp.propulsion {
                return Err(value_error(format!(
                    "propulsion={:?} disagrees with aft.propulsion={:?}",
                    inp.propulsion, a.propulsion
                )));
            }
            a.clone()
        }
        None => {
            let d = &inp.aft_design;
            let mut a = AftArrangementInputs::new(inp.beam, inp.depth);
            a.n_thrusters = d.n_thrusters;
            a.thruster_envelope_m = d.thruster_envelope_m;
            a.thruster_clear_m = d.thruster_clear_m;
            a.thruster_room_length_m = d.thruster_room_length_m;
            a.container_height_m = d.container_height_m;
            a.container_side_gap_m = d.container_side_gap_m;
            a.container_gap_m = d.container_gap_m;
            a.container_foundation_m = d.container_foundation_m;
            a.wheelhouse_length_m = d.wheelhouse_length_m;
            a.wheelhouse_width_m = d.wheelhouse_width_m;
            a.wheelhouse_height_m = d.wheelhouse_height_m;
            a.wheelhouse_gap_m = d.wheelhouse_gap_m;
            a.wheelhouse_eye_above_floor_m = d.wheelhouse_eye_above_floor_m;
            a.sightline_margin_m = d.sightline_margin_m;
            a.wheelhouse_max_lift_m = d.wheelhouse_max_lift_m;
            a.propulsion = inp.propulsion.clone();
            if a.is_gas_fuelled() {
                a.n_battery_units = 0;
                a.electrical_load_kwe = inp.electrical_load_kwe;
            } else {
                a.n_battery_units = d.n_battery_units.unwrap_or(if is_gas {
                    DEFAULT_BATTERY_UNITS_GAS
                } else {
                    DEFAULT_BATTERY_UNITS_DRY
                });
            }
            a.n_accom_units = d.n_accom_units.unwrap_or(if is_gas {
                DEFAULT_ACCOM_UNITS_GAS
            } else {
                DEFAULT_ACCOM_UNITS_DRY
            });
            a
        }
    };
    // Keep the block's beam and depth honest even if a caller passed an `aft` built elsewhere.
    aft_in.beam = inp.beam;
    aft_in.depth = inp.depth;
    if inp.self_propelled
        && let Some(m) = &inp.machinery
    {
        aft_in.machinery = Some(m.clone());
        // A gas-fuelled Power Source: the gas gensets and their rooms of issue #180.
        if m.power_source == crate::machinery::PowerSource::GasFuelled && !aft_in.is_gas_fuelled() {
            aft_in.propulsion = "gas_fuelled".to_string();
            aft_in.n_battery_units = 0;
            aft_in.electrical_load_kwe = inp.electrical_load_kwe;
        }
    }
    let aft0 = compute_aft_arrangement(&aft_in)?;
    let block_len = if inp.self_propelled {
        aft0.length_m
    } else {
        inp.pushed_stern_void_m
    };

    let rake = match inp.bow_rake_length_m {
        Some(r) => r,
        None => inp.bow_rake_to_depth_ratio * inp.depth,
    };

    let (aft_sep, fwd_sep, sep) = if is_gas {
        let sep_in = inp.separation.clone().unwrap_or_default();
        let sep = compute_gas_separation(&sep_in).map_err(value_error)?;
        (sep.aft_separation_m, sep.fwd_separation_m, Some(sep))
    } else {
        (inp.bulk_end_bulkhead_m, inp.bulk_end_bulkhead_m, None)
    };

    Ok(Arrangement {
        bow_void_m: rake + fwd_sep,
        stern_void_m: block_len + aft_sep,
        rake_m: rake,
        aft_inputs: aft_in,
        separation: sep,
    })
}

const PUSHED_WARNING: &str = "PUSHED barge: crew, battery and propulsion are on the tug and are \
NOT in this hull's lightship, capacity or air draft. The tug is a separate vessel and must be \
costed and air-draft-gated separately.";

const BULK_GM_WARNING: &str = "Bulk GM is first-order (assumed lightship KG); \
cargo-shift/angle-of-repose not modelled \u{2014} confirm with a full intact-stability booklet.";

/// Whether a Cargo Class takes the gas path (Type C tanks) or the dry path (a hold). Liquid bulk
/// has no path yet, so it is an error, never the gas path.
pub fn takes_gas_path(class: CargoClass) -> Result<bool, PyError> {
    match class {
        CargoClass::Gas => Ok(true),
        CargoClass::DryBulk => Ok(false),
        CargoClass::LiquidBulk => Err(value_error(format!(
            "Cargo class '{class}' is not supported yet: there is no liquid bulk capacity path"
        ))),
    }
}

/// Run the unified barge capacity chain (volume-bound).
pub fn compute(inp: &BargeCapacityInputs) -> Result<BargeCapacityResult, PyError> {
    // Python builds the nested inputs, and so runs their `__post_init__` checks, before
    // `compute` starts: aft first, then separation, and before the cargo lookup.
    if let Some(a) = &inp.aft {
        a.validate().map_err(value_error)?;
    }
    if let Some(sep) = &inp.separation {
        sep.validate().map_err(value_error)?;
    }
    let props = get_cargo(&inp.cargo_type).map_err(value_error)?;
    let density_t = inp.cargo_density_kg_m3.unwrap_or(props.density) / 1000.0;
    let is_gas = takes_gas_path(props.phase)?;

    let arr = arrangement(inp, is_gas)?;
    let (bow_void, stern_void, rake) = (arr.bow_void_m, arr.stern_void_m, arr.rake_m);
    let cargo_zone = inp.loa - bow_void - stern_void;

    let mut geometry_error: Option<String> = None;
    let vol = if !is_gas {
        bulk_volumetric(inp, bow_void, stern_void)?
    } else {
        match gas_volumetric(inp, bow_void, stern_void) {
            Ok(v) => v,
            // A vessel too short for its own stern block, or too narrow for its tanks, is
            // infeasible, not an exception: it comes back as a failed check.
            Err(e) if e.kind == "ValueError" => {
                geometry_error = Some(e.message);
                Volumetric {
                    cargo_volume: 0.0,
                    lightship: 0.0,
                    h_top: inp.depth,
                    detail: Detail {
                        max_tank_od_m: Some(inp.max_tank_od_m),
                        containment_inner_volume_m3: Some(0.0),
                        fill_ratio_applied: Some(inp.fill_ratio),
                        ..Detail::default()
                    },
                    n_total: 0,
                }
            }
            Err(e) => return Err(e),
        }
    };
    let Volumetric {
        cargo_volume,
        lightship,
        mut h_top,
        detail,
        n_total,
    } = vol;
    let mut lightship = lightship;

    // Second pass on the aft block: the wheelhouse has to see over the cargo, and the cargo top
    // is only known now. The length is unchanged by the lift.
    let cargo_top_above_keel = h_top;
    let mut aft_second = arr.aft_inputs.clone();
    aft_second.sightline_obstruction_m = Some(cargo_top_above_keel);
    let aft = compute_aft_arrangement(&aft_second)?;

    // The tallest fixed point of the vessel with the wheelhouse lowered for transit. A pushed
    // barge carries none of it: the block is on the tug.
    let (mut aft, mut h_top_navigating) = if inp.self_propelled {
        h_top = py_max(h_top, aft.block_top_lowered_m);
        let nav = py_max(h_top, aft.wheelhouse_top_raised_m);
        (Some(aft), nav)
    } else {
        (None, h_top)
    };
    // Issue #348: the stern block's mass, in the lightship only when asked for.
    let stern_block_items = match (&aft, &inp.stern_block_mass) {
        (Some(a), Some(m)) => {
            let db = if is_gas {
                detail.bottom_clearance_m.unwrap_or(0.0) - inp.tank_support_height
            } else {
                inp.double_bottom_height_bulk
            };
            let db = if db > 0.0 { db } else { 0.0 };
            crate::aft_arrangement::stern_block_mass(&aft_second, a, m, db)
        }
        _ => Vec::new(),
    };
    lightship += crate::weights::total_mass(&stern_block_items);

    let cargo_capacity = cargo_volume * density_t;

    // --- Full-load hydrostatics (volume-bound load) ---
    let hull = HullGeometry::new(inp.loa, inp.beam, inp.depth, 0.0, 0.0, 0.0, 0.0);
    let cb = inp.block_coefficient;
    let rho = inp.water_density;
    let total_weight = lightship + cargo_capacity;

    // Box hull displacement plateaus at moulded depth; a load that needs more is overloaded.
    let max_float_volume = hull.displaced_volume(inp.depth);
    let divisor = cb * rho;
    if divisor == 0.0 {
        return Err(zero_division());
    }
    let required_volume = total_weight / divisor;
    let overloaded = required_volume > max_float_volume;

    let draft = find_equilibrium_draft(&hull, total_weight, rho, inp.depth, cb, 0.001)?;
    let draft = py_min(draft, inp.depth); // clamp to the gunwale
    let displacement = cb * hull.displaced_volume(draft) * rho;
    let freeboard = inp.depth - draft;
    let air_draft = h_top - draft;

    // --- Light / ballast conditions ---
    let light_draft = find_equilibrium_draft(&hull, lightship, rho, inp.depth, cb, 0.001)?;
    let light_draft = py_min(light_draft, inp.depth);
    let air_draft_light = h_top - light_draft;

    let mut ballast_draft = light_draft;
    if inp.ballast_capacity_t > 0.0 {
        ballast_draft = find_equilibrium_draft(
            &hull,
            lightship + inp.ballast_capacity_t,
            rho,
            inp.depth,
            cb,
            0.001,
        )?;
        ballast_draft = py_min(ballast_draft, inp.depth);
    }
    let air_draft_ballast = h_top - ballast_draft;
    let mut air_draft_navigating_light = h_top_navigating - light_draft;

    // Issue #349: the trim of a dry-bulk hull when the stern block is in the lightship: the rest of
    // the lightship at the middle of the hull, the block items at their zones, the cargo at the
    // middle of the hold zone.
    // Issue #350: the Stern Form moves the LCB.
    let stern = inp.stern_geometry.as_ref();
    let trim_conditions = if !is_gas && !stern_block_items.is_empty() {
        let block = crate::weights::total_mass(&stern_block_items);
        let hold_centre = (stern_void + (inp.loa - bow_void)) / 2.0;
        let mut light: Vec<(f64, f64)> = vec![(lightship - block, inp.loa / 2.0)];
        light.extend(stern_block_items.iter().map(|i| (i.mass_t, i.lcg_m)));
        let mut laden = light.clone();
        laden.push((cargo_capacity, hold_centre));
        vec![
            (
                "Laden",
                crate::hydrostatics::trim_with_stern(&hull, draft, &laden, stern),
            ),
            (
                "Light",
                crate::hydrostatics::trim_with_stern(&hull, light_draft, &light, stern),
            ),
        ]
    } else {
        Vec::new()
    };

    // Issue #354: the sightline as a Blind Distance, in the light condition (ES-TRIN 7.02(2):
    // unladen), on the trimmed waterline where there is one. The wheelhouse lifts as far as the
    // limit needs, up to its maximum; the check of the margin method gives way to it.
    let mut sightline = None;
    let mut blind_distance_laden_m = None;
    if let (Some(limit), Some(a)) = (inp.sightline_limit_m, aft.as_mut()) {
        let waterline = |name: &str, t: f64| match trim_conditions.iter().find(|(n, _)| *n == name)
        {
            Some((_, tr)) => crate::wheelhouse::Waterline {
                draught_aft_m: tr.draft_aft,
                draught_fwd_m: tr.draft_fwd,
                length_m: inp.loa,
            },
            None => crate::wheelhouse::Waterline::even(t, inp.loa),
        };
        let units = (!a.container_units.is_empty())
            .then_some((a.container_x1_m, a.container_top_above_keel_m));
        let obstructions = crate::wheelhouse::obstructions(
            units,
            (inp.loa - bow_void, cargo_top_above_keel),
            (inp.loa, inp.depth),
        );
        let eye_x = a.wheelhouse_x1_m;
        let eye_height = aft_second.wheelhouse_eye_above_floor_m;
        let s = crate::wheelhouse::assess(
            eye_x,
            a.wheelhouse_floor_lowered_m + eye_height,
            eye_height,
            aft_second.wheelhouse_max_lift_m,
            &obstructions,
            inp.loa,
            &waterline("Light", light_draft),
            limit,
        );
        blind_distance_laden_m = Some(crate::wheelhouse::blind_distance(
            eye_x,
            s.eye_raised_m,
            &obstructions,
            inp.loa,
            &waterline("Laden", draft),
        ));
        a.wheelhouse_lift_m = s.lift_used_m;
        a.wheelhouse_floor_raised_m = s.floor_raised_m;
        a.wheelhouse_top_raised_m = s.floor_raised_m + a.wheelhouse_height_m;
        a.checks.retain(|c| c.name != "wheelhouse_sees_over_cargo");
        a.checks.push(Check::new(
            "wheelhouse_sees_over_cargo",
            s.enough,
            text!(
                "check-wheelhouse-blind-distance",
                "blind distance ahead of the bow {blind} m <= {limit} m, light (ES-TRIN 7.02(2)); lift used {lift} m of {max_lift} m",
                blind = Arg::fixed(s.blind_distance_m, 1),
                limit = Arg::fixed(limit, 0),
                lift = Arg::fixed(s.lift_used_m, 2),
                max_lift = Arg::fixed(aft_second.wheelhouse_max_lift_m, 2),
            ),
        ));
        a.warnings
            .retain(|w| !alloc::string::ToString::to_string(w).starts_with("Sightline needs "));
        if !s.enough {
            a.warnings.push(text!(
                "warn-aft-blind-distance-lift",
                "The blind distance needs {needed} m of lift; the column gives {max_lift} m. Raise the column or lower the cargo top.",
                needed = Arg::fixed(s.lift_needed_m, 2),
                max_lift = Arg::fixed(aft_second.wheelhouse_max_lift_m, 2),
            ));
        }
        h_top_navigating = py_max(h_top, a.wheelhouse_top_raised_m);
        air_draft_navigating_light = h_top_navigating - light_draft;
        sightline = Some(s);
    }

    // A geometry error is error text: it stays English, with no Message Id.
    let feasible_message = match &geometry_error {
        Some(m) if !m.is_empty() => Text::plain(m.clone()),
        _ => text!(
            "check-feasible-geometry",
            "cargo volume must be positive (tank/hold fits)"
        ),
    };
    let mut checks = vec![
        Check::new("feasible_geometry", cargo_volume > 0.0, feasible_message),
        Check::new(
            "not_overloaded",
            !overloaded,
            text!(
                "check-not-overloaded",
                "volume-bound load must float within the moulded depth"
            ),
        ),
        Check::new(
            "freeboard_ok",
            freeboard >= inp.required_freeboard,
            text!(
                "check-freeboard-ok",
                "freeboard {freeboard} m >= required {required} m",
                freeboard = Arg::fixed(freeboard, 2),
                required = Arg::py(inp.required_freeboard),
            ),
        ),
        Check::new(
            "cargo_zone_positive",
            cargo_zone > 0.0,
            text!(
                "check-cargo-zone-positive",
                "cargo zone {zone} m = LOA {loa} - bow void {bow_void} m - stern void {stern_void} m",
                zone = Arg::fixed(cargo_zone, 2),
                loa = Arg::fixed(inp.loa, 1),
                bow_void = Arg::fixed(bow_void, 2),
                stern_void = Arg::fixed(stern_void, 2),
            ),
        ),
    ];
    // Roll up the arrangement and gas-code checks so one `ok` covers the whole vessel.
    if let Some(a) = &aft {
        checks.extend(a.checks.iter().cloned());
    }
    // The separation was resolved once in `arrangement`; its checks join the roll-up here.
    let sep = arr.separation;
    if let Some(s) = &sep {
        checks.extend(s.checks.iter().cloned());
    }
    // Bridge clearance is gated on the LIGHT (or ballasted-light) condition, never the laden one.
    if let Some(limit) = inp.air_draft_limit_m {
        let ballasted = inp.ballast_capacity_t > 0.0;
        let governing = if ballasted {
            air_draft_ballast
        } else {
            air_draft_light
        };
        let label = if ballasted {
            Text::fixed(
                "check-air-draft-condition-ballasted-light",
                "ballasted light",
            )
        } else {
            Text::fixed("check-air-draft-condition-light", "light")
        };
        checks.push(Check::new(
            "air_draft_within_limit",
            governing <= limit,
            text!(
                "check-air-draft-within-limit",
                "{condition} air draft {governing} m <= limit {limit} m (laden is {laden} m and is NOT the governing case)",
                condition = label,
                governing = Arg::fixed(governing, 2),
                limit = Arg::fixed(limit, 2),
                laden = Arg::fixed(air_draft, 2),
            ),
        ));
    }
    let mut warnings: Vec<Text> = match &aft {
        Some(a) => a.warnings.clone(),
        None => vec![Text::fixed("warn-barge-pushed", PUSHED_WARNING)],
    };
    if let Some(s) = &sep {
        warnings.extend(s.warnings.iter().cloned());
    }
    let mut gm = None;
    if props.phase == CargoClass::DryBulk && draft > 0.0 && !overloaded {
        // The bulk branch always sets these.
        let hold = (
            detail.hold_top_width_m.unwrap_or(f64::NAN),
            detail.hold_bottom_width_m.unwrap_or(f64::NAN),
            detail.hold_height_m.unwrap_or(f64::NAN),
        );
        let g = bulk_gm(
            inp,
            hold,
            draft,
            (lightship, &stern_block_items),
            cargo_capacity,
            &hull,
        );
        checks.push(Check::with_severity(
            "gm_positive",
            g >= 0.15,
            text!(
                "check-gm-positive",
                "hopper intact GM {gm} m >= 0.15 m (first-order; KG_light @0.5D)",
                gm = Arg::fixed(g, 2),
            ),
            "warning",
        ));
        warnings.push(bulk_gm_note());

        gm = Some(g);
    }

    // A tank count that no 64-bit field can hold: after everything else has succeeded.
    if i64::try_from(n_total).is_err() {
        return Err(overflow());
    }

    Ok(BargeCapacityResult {
        cargo_type: props.name.to_string(),
        phase: props.phase,
        cargo_density_t_per_m3: density_t,
        cargo_capacity_t: cargo_capacity,
        full_load_draft_m: draft,
        air_draft_m: air_draft,
        cargo_volume_m3: cargo_volume,
        lightship_t: lightship,
        displacement_t: displacement,
        deadweight_t: displacement - lightship,
        freeboard_m: freeboard,
        air_draft_top_above_keel_m: h_top,
        overloaded,
        tank_outer_diameter_m: detail.tank_outer_diameter_m,
        n_tanks_wide: detail.n_tanks_wide,
        max_tank_od_m: detail.max_tank_od_m,
        n_tanks_total: detail.n_tanks_total,
        side_clearance_m: detail.side_clearance_m,
        bottom_clearance_m: detail.bottom_clearance_m,
        hold_height_m: detail.hold_height_m,
        hold_top_width_m: detail.hold_top_width_m,
        hold_bottom_width_m: detail.hold_bottom_width_m,
        hold_section_area_m2: detail.hold_section_area_m2,
        gm_m: gm,
        bow_void_length_m: Some(bow_void),
        stern_void_length_m: Some(stern_void),
        cargo_zone_length_m: Some(cargo_zone),
        bow_rake_length_m: Some(rake),
        aft_arrangement: aft,
        separation: sep,
        containment_inner_volume_m3: detail.containment_inner_volume_m3,
        fill_ratio_applied: detail.fill_ratio_applied,
        insulation_thickness_mm: detail.insulation_thickness_mm,
        tank_wall_thickness_mm: detail.tank_wall_thickness_mm,
        tank_inner_diameter_m: detail.tank_inner_diameter_m,
        light_draft_m: Some(light_draft),
        air_draft_light_m: Some(air_draft_light),
        ballast_draft_m: Some(ballast_draft),
        air_draft_ballast_m: Some(air_draft_ballast),
        air_draft_navigating_light_m: Some(air_draft_navigating_light),
        navigating_top_above_keel_m: Some(h_top_navigating),
        gas_basis: detail.gas_basis,
        stern_block_items: stern_block_items.clone(),
        trim_conditions,
        sightline,
        blind_distance_laden_m,
        stern_geometry: inp.stern_geometry.clone(),
        tank_rules: detail.tank_rules,
        checks,
        warnings,
    })
}
