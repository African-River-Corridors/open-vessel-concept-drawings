//! Running a saved Design: the Option-to-`simulate` mapping, the results hash, and the text of
//! `ovcd run` (ADR 0007).
//!
//! The results hash is the SHA-256 of the canonical JSON of one Option's results: the `simulate`
//! result object, with object keys in sorted order, no white space, and numbers as `serde_json`
//! writes them (the shortest text that reads back exactly). The core uses the pure-Rust `libm`, so
//! the same inputs give the same bytes on every platform.
//!
//! A single self-propelled Hull, a single pushed barge (without its pusher) and a pushed convoy
//! (one barge and the totals of every barge, without the pusher) are computed.

use crate::rule_set::Library;
use crate::simulate::{Outcome, RunRules, simulate_params_measured};
use crate::trace::{rule_inputs, stack_for, trace_json, trace_of};
use crate::vdesign::{Design, DesignOption};
use ovcd_core::pyfmt::fixed;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

/// What a convoy Formation gets in place of results.
pub const CONVOY_NOT_AVAILABLE: &str = "convoy results not yet available";

/// The Message Id of an outcome's fixed text, where it has one (ADR 0015). A refusal's text
/// (`Failed`) is an error and stays English.
pub fn outcome_text(message: &str) -> Option<ovcd_core::text::Text> {
    (message == CONVOY_NOT_AVAILABLE)
        .then(|| ovcd_core::text::Text::fixed("msg-convoy-not-available", CONVOY_NOT_AVAILABLE))
}

/// The outcome of running one Option.
#[derive(Debug, Clone, PartialEq)]
pub enum RunOutcome {
    /// The `simulate` result and its hash.
    Results { results: Value, hash: String },
    /// The Option cannot be computed yet: a convoy.
    NotAvailable(String),
    /// The engine refused or raised; the text says why, for a person.
    Failed(String),
}

impl RunOutcome {
    /// The word `ovcd run --json` gives the Option: `results`, `not_yet_supported` (a convoy: the
    /// file is valid, the engine cannot compute it yet) or `failed`.
    pub fn status(&self) -> &'static str {
        match self {
            RunOutcome::Results { .. } => "results",
            RunOutcome::NotAvailable(_) => "not_yet_supported",
            RunOutcome::Failed(_) => "failed",
        }
    }

    /// Does this outcome make `ovcd run` exit 1? An Option that is not yet supported does not.
    pub fn is_failure(&self) -> bool {
        matches!(self, RunOutcome::Failed(_))
    }
}

/// Is the stored hash of an Option the same as the one this engine computes?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashStatus {
    /// No hash in the file, or nothing to compare it with.
    NotStored,
    /// A hash is in the file, but the Option has no results yet (a convoy), so it is not checked.
    NotChecked,
    Matches,
    /// The file's hash differs from this engine's.
    Differs {
        stored: String,
        computed: String,
    },
}

impl HashStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            HashStatus::NotStored => "not_stored",
            HashStatus::NotChecked => "not_checked",
            HashStatus::Matches => "matches",
            HashStatus::Differs { .. } => "differs",
        }
    }
}

/// One Option, run, with its hash check against the file.
#[derive(Debug, Clone, PartialEq)]
pub struct RunOption {
    pub name: String,
    pub outcome: RunOutcome,
    pub hash_status: HashStatus,
}

/// The `simulate` params of an Option. The glossary names map to the engine's names; the Formation
/// gives `self_propelled`. `Err` for a pushed convoy.
pub fn simulate_params_of(option: &DesignOption) -> Result<Map<String, Value>, String> {
    let get = |name: &str| option.input(name).map(|i| i.value.clone());
    let formation = get("formation").unwrap_or(Value::Null);
    let kind = formation.get("kind").and_then(Value::as_str).unwrap_or("");
    // A pushed convoy is run as one pushed barge; `run_option_with` adds the totals (issue #358).
    let mut p = Map::new();
    let mut put = |key: &str, name: &str| {
        if let Some(v) = get(name)
            && !v.is_null()
        {
            p.insert(key.to_string(), v);
        }
    };
    put("cargo", "cargo");
    put("loa_m", "length_overall_m");
    put("beam_m", "beam_m");
    put("depth_m", "moulded_depth_m");
    // Issue #362: the tug and the Notch of an ATB, each typed or none.
    if kind == "atb" {
        for name in [
            "tug_length_m",
            "tug_beam_m",
            "tug_draught_m",
            "notch_depth_m",
            "notch_width_m",
        ] {
            put(name, name);
        }
    }
    // Issue #448: the bow of the Hull Form; Cb is a result, not an input.
    put("bow_rake_length_m", "bow_rake_length_m");
    put("bow_rake_rise_m", "bow_rake_rise_m");
    // Issue #449: the bilge, sheer and camber of the Hull Form. Issue #450: the Hull Form and its
    // bottom.
    for name in [
        "hull_form",
        "bilge_radius_m",
        "sheer_aft_m",
        "sheer_fwd_m",
        "camber_m",
        "deadrise_deg",
        "keel_half_width_m",
        "chine_half_breadth_m",
        "chine_height_m",
    ] {
        put(name, name);
    }
    // Issue #451: an offsets hull is its typed table, and its L, B and D are the table's.
    put("hull_form", "hull_form");
    put("hull_offsets", "hull_offsets");
    put("shipd_parameters", "shipd_parameters");
    put("freeboard_min_m", "min_freeboard_m");
    put("lightship_k_t_per_m3", "lightship_factor_t_per_m3");
    put("fill_ratio", "fill_ratio");
    put("max_tank_od_m", "max_tank_od_m");
    put("optimise_depth", "optimise_depth");
    for name in [
        "water_density_t_per_m3",
        "squat_safety_margin_m",
        "tanks_across",
        "tanks_in_line",
        "tank_support_height_m",
        "tank_dome_height_m",
        "tank_dome_diameter_m",
        "wing_tank_gap_m",
        "wing_tank_height_ratio",
        "min_side_space_m",
        "weather_shield_thickness_mm",
        "weather_shield_clearance_m",
        "tank_forming_allowance_pct",
        "tank_dome_wall_thickness_mm",
        "aft_end_space_m",
        "fwd_end_space_m",
        "opening_setback_basis",
        "tank_wall_thickness_mm",
        "lpg_carriage_condition",
        "lpg_loading_limit_basis",
        "lpg_loading_temperature_c",
        "tank_design_pressure_mpa",
        "tank_design_temperature_c",
        "tank_material",
        "stern_form",
        "stern_run_length_m",
        "stern_transom_height_m",
        "tunnel_crown_height_m",
        "propulsor_type",
        "propulsor_count",
        "propeller_diameter_m",
        "propeller_diameter_basis",
        "stern_block_mass_included",
        "battery_unit_energy_kwh",
        "installed_power_kw",
        "power_source",
        "battery_energy_kwh",
        "trip_length_km",
        "bow_thruster",
        "bow_thruster_power_kw",
        "design_speed_kn",
        "installed_power_margin",
        "propulsor_efficiency_factor",
        "bollard_pull_factor",
        "sightline_method",
        "wheelhouse_type",
        "tank_head_type",
        "tank_liquid_pressure",
        "tank_allowable_stress_basis",
        "tank_allowable_stress_mpa",
        "tank_insulation_type",
        "tank_insulation_thickness_mm",
        "tank_structure_factor",
        "bow_rake_length_to_depth_ratio",
        "bow_rake_max_length_m",
        "pushed_stern_void_m",
        "bulk_end_bulkhead_m",
        "hold_wing_void_width_m",
        "hold_double_bottom_m",
        "hopper_slope_h_per_v",
        "hatch_coaming_height_m",
        "n_holds",
        "hold_fill_ratio",
        "inner_bottom_plate_mm",
        "hopper_plate_mm",
        "hold_bulkhead_plate_mm",
        "hatch_coaming_plate_mm",
        "thruster_count",
        "thruster_envelope_m",
        "thruster_clearance_m",
        "thruster_room_length_m",
        "battery_unit_count",
        "accommodation_unit_count",
        "unit_height_m",
        "unit_side_gap_m",
        "unit_row_gap_m",
        "unit_foundation_m",
        "wheelhouse_length_m",
        "wheelhouse_width_m",
        "wheelhouse_height_m",
        "wheelhouse_gap_m",
        "wheelhouse_eye_height_m",
        "sightline_margin_m",
        "wheelhouse_max_lift_m",
        "loading_conditions",
        "may_sail_without_ballast",
        "endurance_h",
        "fixed_stores_t",
        "design_speed_kn",
        "power_source",
        "ballast_plan",
        "ballast_departure_double_bottom_fill",
        "ballast_departure_peak_fill",
        "ballast_departure_wing_fill",
        "ballast_arrival_double_bottom_fill",
        "ballast_arrival_peak_fill",
        "ballast_arrival_wing_fill",
        // Issue #331: the cargo tanks of a liquid bulk Cargo.
        "double_side_m",
        "double_bottom_m",
        "liquid_tanks_in_line",
        "centreline_bulkhead",
        "cofferdam_m",
        "pump_arrangement",
        "pump_room_length_m",
        "slop_share",
        "heating_kw_per_m3",
        "liquid_fill_ratio",
    ] {
        put(name, name);
    }
    put("eta_d", "propulsive_efficiency");
    put("speed_max_kn", "max_speed_kn");
    put("air_draft_limit_m", "air_draft_limit_m");
    put("draught_limit_m", "draught_limit_m");
    put("downflooding_angle_deg", "downflooding_angle_deg");
    put("downflooding_openings", "downflooding_openings");
    // A null wall thickness asks for the pressure-vessel formula (issue #292); `put` leaves nulls
    // out, and a key left out means the engine's old fixed 25 mm.
    // In the same way a null dome size asks for the dome rule (issue #316), and a null design
    // pressure for the IGC 4.23.1.2 minimum of each tank (issue #577).
    for name in [
        "tank_wall_thickness_mm",
        "tank_dome_height_m",
        "tank_dome_diameter_m",
        "tank_design_pressure_mpa",
    ] {
        if get(name).is_some_and(|v| v.is_null()) {
            p.insert(name.to_string(), Value::Null);
        }
    }
    let vessel = match get("cargo_containment").as_ref().and_then(Value::as_str) {
        Some("box_hold") => "box_barge",
        Some("sloped_hopper_hold") => "hopper_barge",
        Some("type_c_tanks") => "gas_barge",
        Some("integral_tanks") => "tank_barge",
        _ => return Err("the Option has no Cargo Containment".to_string()),
    };
    p.insert("vessel".into(), json!(vessel));
    p.insert("self_propelled".into(), json!(kind == "self_propelled"));
    // Issue #362: the barge of an ATB has a Notch (see the tug and Notch inputs above).
    if kind == "atb" {
        p.insert("atb".into(), json!(true));
    }
    // The channel is described only when it has a depth: without one the engine ignores it.
    if let Some(depth) = get("channel_depth_m")
        && !depth.is_null()
    {
        let mut ch = Map::new();
        ch.insert("depth_m".into(), depth);
        let mut put_ch = |key: &str, name: &str| {
            if let Some(v) = get(name)
                && !v.is_null()
            {
                ch.insert(key.to_string(), v);
            }
        };
        put_ch("ukc_m", "channel_under_keel_clearance_m");
        put_ch("bottom_width_m", "channel_bottom_width_m");
        put_ch("bank_slope", "channel_bank_slope");
        put_ch("current_kn", "channel_current_kn");
        p.insert("channel".into(), Value::Object(ch));
    }
    if p.get("hull_form").and_then(Value::as_str) == Some("offsets")
        && let Some(d) =
            crate::offsets::dimensions_of(p.get("hull_offsets").unwrap_or(&Value::Null))
    {
        p.insert("loa_m".to_string(), json!(d.length_m));
        p.insert("beam_m".to_string(), json!(d.beam_m));
        p.insert("depth_m".to_string(), json!(d.depth_m));
    }
    Ok(p)
}

/// The canonical JSON text: sorted keys, no white space. The rules kernel hashes Rule Sets with the
/// same function.
pub use arc_rules::load::canonical_json;

/// Top-level results keys that are not results: drawing geometry, what a view marks, the capital
/// budget with the masses it was worked from, the Message Ids of the result's English (ADR 0015), and
/// the Rule Trace (spec 0004: the stored Rule Set hashes cover the rules), and the calibration label
/// of the lightship (issue #434), and the gas tank arrangements the search tried (issue #675). They are left
/// out of the results hash, so adding to one never changes a saved hash, and `tools/design_impact.py`
/// leaves them out of its comparison (it holds the same list; `tests/test_design_impact_unhashed_keys.py` checks
/// that the two lists agree).
pub const UNHASHED_KEYS: [&str; 11] = [
    "drawing",
    "stability_marks",
    "cost_masses",
    "cost",
    "text_ids",
    "rule_trace",
    "calibration",
    "damage",
    "marpol_damage",
    "oil_outflow",
    "fit_tried",
];

/// SHA-256 of the canonical JSON of `results`, without the [`UNHASHED_KEYS`].
pub fn results_hash(results: &Value) -> String {
    let mut hashed = results.clone();
    if let Some(o) = hashed.as_object_mut() {
        for key in UNHASHED_KEYS {
            o.remove(key);
        }
    }
    let digest = Sha256::digest(canonical_json(&hashed).as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Compute one Option.
pub fn run_option(option: &DesignOption) -> RunOutcome {
    run_option_with(option, &Library::built_in(), &[])
}

/// The hull of an Option as its Sections Table, as the run builds it (issue #451; the offsets
/// export of #461 reads it too).
pub fn option_hull_table(
    option: &DesignOption,
) -> Result<ovcd_core::sections::SectionsTable, String> {
    let params = simulate_params_of(option)?;
    crate::simulate::hull_table_of(&params).map_err(|e| match e {
        crate::simulate::Stop::Raised(_, message) => message,
        crate::simulate::Stop::Eval(e) => e.to_string(),
    })
}

/// The Vessel Model of an Option (issue #454): the hull, decks, bulkheads and Spaces of the design
/// the run computes, with the built-in Rule Sets. A convoy, or an Option that cannot be computed,
/// gives the message of the outputs panel.
pub fn option_vessel_model(
    option: &DesignOption,
) -> Result<ovcd_core::vessel_model::VesselModel, String> {
    option_vessel_run(option).map(|run| run.model)
}

/// One floating position of a run, for its waterline: the draughts (m) and the trim (m, by the bow
/// positive).
#[derive(Debug, Clone, PartialEq)]
pub struct Waterline {
    pub name: String,
    pub draught_m: f64,
    pub draught_aft_m: f64,
    pub draught_fwd_m: f64,
    pub trim_m: f64,
}

/// The Vessel Model of an Option and the waterlines of its run (issues #454, #457).
#[derive(Debug, Clone, PartialEq)]
pub struct VesselRun {
    pub model: ovcd_core::vessel_model::VesselModel,
    pub waterlines: Vec<Waterline>,
}

/// Run an Option for its Vessel Model and waterlines.
pub fn option_vessel_run(option: &DesignOption) -> Result<VesselRun, String> {
    let params = simulate_params_of(option)?;
    let library = Library::built_in();
    let selection = stack_for(&library, option, &[])?;
    let rules = RunRules {
        sets: selection.sets.iter().map(|s| (*s).clone()).collect(),
        inputs: rule_inputs(option),
    };
    let (outcome, measured) = simulate_params_measured(&params, &rules);
    match outcome {
        Outcome::Result(v) => match v.get("error").and_then(Value::as_str) {
            Some(message) => Err(message.to_string()),
            None => measured
                .vessel
                .as_ref()
                .map(|inputs| VesselRun {
                    model: ovcd_core::vessel_model::assemble(inputs),
                    waterlines: waterlines(&v),
                })
                .ok_or_else(|| "the run gave no Vessel Model".to_string()),
        },
        Outcome::Raised(kind, message) => Err(format!("{kind}: {message}")),
        Outcome::Refused(message) => Err(message),
    }
}

/// The floating positions of a run: each loading condition, or each stability condition, or with
/// neither the loaded and light draughts, level.
fn waterlines(results: &Value) -> Vec<Waterline> {
    for key in ["loading_conditions", "stability"] {
        let list: Vec<Waterline> = results[key]["conditions"]
            .as_array()
            .map(|cs| {
                cs.iter()
                    .filter_map(|c| {
                        let mid = c["draught_m"].as_f64()?;
                        let aft = c["draught_aft_m"].as_f64().unwrap_or(mid);
                        let fwd = c["draught_fwd_m"].as_f64().unwrap_or(mid);
                        Some(Waterline {
                            name: c["name"].as_str()?.to_string(),
                            draught_m: mid,
                            draught_aft_m: aft,
                            draught_fwd_m: fwd,
                            trim_m: c["trim_m"].as_f64().unwrap_or(fwd - aft),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        if !list.is_empty() {
            return list;
        }
    }
    [("Loaded", "draught_m"), ("Light", "light_draught_m")]
        .into_iter()
        .filter_map(|(name, key)| {
            let t = results["results"][key].as_f64()?;
            Some(Waterline {
                name: name.to_string(),
                draught_m: t,
                draught_aft_m: t,
                draught_fwd_m: t,
                trim_m: 0.0,
            })
        })
        .collect()
}

impl VesselRun {
    /// The meshes of the model (issue #456): the hull, decks, bulkheads, Spaces and the waterline of
    /// each floating position, one group each.
    pub fn meshes(&self) -> ovcd_core::mesh::Mesh {
        ovcd_core::mesh::vessel_meshes(&self.model, &self.waterline_draughts())
    }

    /// The floating positions as `(name, aft draught, forward draught)`.
    fn waterline_draughts(&self) -> Vec<(String, f64, f64)> {
        self.waterlines
            .iter()
            .map(|w| (w.name.clone(), w.draught_aft_m, w.draught_fwd_m))
            .collect()
    }
}

/// A Vessel Model and its floating positions `(name, aft draught, forward draught)`.
pub type ModelWaterlines = (
    ovcd_core::vessel_model::VesselModel,
    Vec<(String, f64, f64)>,
);

/// The Vessel Model of an Option and its floating positions `(name, aft draught, forward draught)`
/// (issue #461): what the panel mesh export clips the hull at.
pub fn option_model_waterlines(option: &DesignOption) -> Result<ModelWaterlines, String> {
    let run = option_vessel_run(option)?;
    let lines = run.waterline_draughts();
    Ok((run.model, lines))
}

/// The meshes of an Option's Vessel Model (issue #456).
pub fn option_meshes(option: &DesignOption) -> Result<ovcd_core::mesh::Mesh, String> {
    option_vessel_run(option).map(|run| run.meshes())
}

/// The mesh hash of the parity check (spec 0008, "Mesh hash"): SHA-256 of the f32 position bytes,
/// the u32 index bytes (little-endian) and the group table (layer, id, start, count). Not in the
/// results hash (ADR 0018).
pub fn mesh_hash(mesh: &ovcd_core::mesh::Mesh) -> String {
    let mut h = Sha256::new();
    for p in &mesh.positions {
        h.update(p.to_le_bytes());
    }
    for i in &mesh.indices {
        h.update(i.to_le_bytes());
    }
    for g in &mesh.groups {
        for text in [&g.layer, &g.id] {
            h.update((text.len() as u32).to_le_bytes());
            h.update(text.as_bytes());
        }
        h.update(g.start.to_le_bytes());
        h.update(g.count.to_le_bytes());
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// The propeller ratios the `ovcd/propulsion` Rules read (issue #355), from the results: the diameter
/// over the Stern Form's largest, the shaft immersion over its limit at the lightest aft draught of the
/// loading conditions, and the blade load over its limit. Each is `None` where the results lack what
/// it needs.
fn propulsion_ratios(v: &Value) -> (Option<f64>, Option<f64>, Option<f64>) {
    let over_max = match (
        v["geometry"]["propeller_diameter_m"].as_f64(),
        v["geometry"]["propeller_d_max_m"].as_f64(),
    ) {
        (Some(d), Some(m)) if m > 0.0 => Some(d / m),
        _ => None,
    };
    let pw = &v["powering"];
    let diameter = pw["propeller_diameter_m"].as_f64().filter(|d| *d > 0.0);
    let immersion = diameter.and_then(|d| {
        let axis = pw["shaft_axis_height_m"].as_f64()?;
        let aft = v["loading_conditions"]["conditions"]
            .as_array()?
            .iter()
            .filter_map(|c| c["draught_aft_m"].as_f64())
            .fold(f64::INFINITY, f64::min);
        if !aft.is_finite() {
            return None;
        }
        let limit = if pw["propeller_ducted"].as_bool()? {
            ovcd_core::powering::IMMERSION_DUCTED
        } else {
            ovcd_core::powering::IMMERSION_OPEN
        };
        Some((aft - axis) / d / limit)
    });
    let blade = match (
        pw["blade_load_kw_m2"].as_f64(),
        pw["blade_load_limit_kw_m2"].as_f64(),
    ) {
        (Some(l), Some(m)) if m > 0.0 => Some(l / m),
        _ => None,
    };
    (over_max, immersion, blade)
}

/// Compute one Option and check it against its default stack of Rule Sets from `library`, plus the
/// `extra` sets (`id`, `version`) loaded at run time (spec 0004, "Loading at run time").
pub fn run_option_with(
    option: &DesignOption,
    library: &Library,
    extra: &[(String, String)],
) -> RunOutcome {
    let params = match simulate_params_of(option) {
        Ok(p) => p,
        Err(m) => return RunOutcome::Failed(m),
    };
    // The Rule Sets of the run: the gas arrangement search keeps only candidates that meet them.
    let selection = match stack_for(library, option, extra) {
        Ok(s) => s,
        Err(m) => return RunOutcome::Failed(m),
    };
    let rules = RunRules {
        sets: selection.sets.iter().map(|s| (*s).clone()).collect(),
        inputs: rule_inputs(option),
    };
    let (outcome, mut measured) = simulate_params_measured(&params, &rules);
    match outcome {
        Outcome::Result(v) => match v.get("error").and_then(Value::as_str) {
            Some(message) => RunOutcome::Failed(message.to_string()),
            None => {
                // The capital budget: an output, not hashed, never fed back into the run.
                let mut v = v;
                if let Some((abreast, in_line)) = crate::convoy::counts(option) {
                    crate::convoy::add_totals(abreast, in_line, &mut v);
                }
                crate::convoy::add_lock_checks(option, &mut v);
                crate::formation::add_atb_drawing(option, &mut v);
                // Issue #359: the pusher of a pushed barge or convoy, when the Option includes it.
                crate::formation::add_formation(
                    option,
                    &params,
                    crate::convoy::counts(option).unwrap_or((1, 1)),
                    &mut v,
                );
                // Issue #464: the gas a gas-fuelled Option burns, and its boil-off check.
                crate::gas_fuel::add_gas_fuel(option, &mut v);
                let mut cost = crate::cost::cost_of(option, &v);
                // Issue #552: the budget of a pushed convoy is that of every barge.
                if let Some((abreast, in_line)) = crate::convoy::counts(option) {
                    crate::convoy::scale_cost(abreast * in_line, &mut cost);
                }
                // The Rule Trace: not hashed either; the stored Rule Set hashes cover the rules.
                measured.checks = v
                    .get("checks")
                    .and_then(Value::as_array)
                    .map(|list| {
                        list.iter()
                            .filter_map(|c| {
                                Some((
                                    c.get("name")?.as_str()?.to_string(),
                                    c.get("ok")?.as_bool()?,
                                ))
                            })
                            .collect()
                    })
                    .map(crate::trace::with_check_verdicts)
                    .unwrap_or_default();
                // Issue #355: the Blind Distance and the speed the Rules of the stern read.
                measured.downflooding_deg = option
                    .input("downflooding_angle_deg")
                    .and_then(|i| i.value.as_f64());
                measured.downflooding_openings = option
                    .input("downflooding_openings")
                    .and_then(|i| i.value.as_array())
                    .is_some_and(|rows| !rows.is_empty());
                let sightline = &v["sightline"];
                measured.blind_distance_light_m = sightline["blind_distance_light_m"].as_f64();
                measured.blind_distance_laden_m = sightline["blind_distance_laden_m"].as_f64();
                let speeds_of = if v["powering"].is_null() {
                    &v["formation"]
                } else {
                    &v["powering"]
                };
                measured.speeds_kn = speeds_of["environments"]
                    .as_array()
                    .map(|list| {
                        list.iter()
                            .filter_map(|e| {
                                Some((
                                    e["name"].as_str()?.to_string(),
                                    e["service_speed_kn"].as_f64()?,
                                ))
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                measured.design_speed_kn = v["powering"]["design_speed_kn"].as_f64();
                // Issue #359: a pushed Formation with its pusher has its speeds in `formation`.
                if v["formation"]["environments"].is_array() && v["powering"].is_null() {
                    measured.design_speed_kn = v["formation"]["design_speed_kn"].as_f64();
                }
                measured.lift_enough = v["sightline"]["lift_enough"].as_bool();
                // A pushed Formation with its pusher: the coupling force of ES-TRIN ESI-III-6 2.1.
                measured.coupling_force_kn = v["formation"]["coupling_force_kn"].as_f64();
                measured.depth_over_draught = match (
                    option
                        .input("channel_depth_m")
                        .and_then(|i| i.value.as_f64()),
                    v["results"]["draught_m"].as_f64(),
                ) {
                    (Some(h), Some(t)) if t > 0.0 => Some(h / t),
                    _ => None,
                };
                let (over_max, immersion, blade) = propulsion_ratios(&v);
                measured.diameter_over_max = over_max;
                measured.immersion_over_limit = immersion;
                measured.blade_load_over_limit = blade;
                // Issue #659: the ICLL freeboard Rules read the Freeboard the result has.
                measured.freeboard_laden_m = v["results"]["freeboard_m"].as_f64();
                let rule_trace = trace_json(
                    &trace_of(&selection, option, &measured),
                    crate::trace::service_area_of(option),
                    &selection.warnings,
                );
                let v_has_formation = v.get("formation").is_some_and(Value::is_object);
                if let Some(o) = v.as_object_mut() {
                    o.insert("cost".to_string(), cost);
                    let mut calibration = calibration_of(option);
                    // Issue #359: the pusher ratios and the allowance are not fitted yet (part 2 of
                    // #359 does it). The label is outside the hash, so its change moves no hash.
                    if v_has_formation {
                        calibration["formation"] = json!({
                            "state": "uncalibrated",
                            "basis": "The pusher ratios and the Formation resistance allowance are estimates not yet fitted to the reference pushers; a small pusher can be far from a real one.",
                        });
                    }
                    o.insert("calibration".to_string(), calibration);
                    o.insert("rule_trace".to_string(), rule_trace);
                    // Spec 0009, D1 (b): the Type C tank Spaces, one row each (ADR 0021). Hashed:
                    // the rows are new, and no other value moves.
                    if let Some(rows) = measured.vessel.as_ref().map(tank_space_rows)
                        && !rows.is_empty()
                    {
                        o.insert("spaces".to_string(), Value::Array(rows));
                    }
                }
                let hash = results_hash(&v);
                RunOutcome::Results { results: v, hash }
            }
        },
        Outcome::Raised(kind, message) => RunOutcome::Failed(format!("{kind}: {message}")),
        Outcome::Refused(message) => RunOutcome::Failed(message),
    }
}

/// The rows of the results Spaces table (spec 0009, D1): each Type C tank Space of a gas Design, with
/// its name, kind, side, volume (m³) and centroid (m), as the 3D view's pick gives a Space.
fn tank_space_rows(inputs: &ovcd_core::vessel_model::VesselModelInputs) -> Vec<Value> {
    ovcd_core::vessel_model::tank_spaces_of(&inputs.capacity, &inputs.result)
        .iter()
        .map(|s| {
            let (x, y, z) = s.centroid_m;
            json!({
                "name": s.space.name,
                "kind": s.space.kind.key(),
                "side": s.space.side.key(),
                "volume_m3": s.volume_m3,
                "lcg_m": x,
                "tcg_m": y,
                "vcg_m": z,
            })
        })
        .collect()
}

/// Whether the lightship of the Option's hull has a calibration case (issue #434). The box-hold
/// factor and block coefficient are fitted to Reference Vessels (`calibration/cases.yaml`); no case
/// covers a sloped-hopper hull or a gas barge, so their lightship is the build-up with fixed factors.
/// An output, not hashed.
fn calibration_of(option: &DesignOption) -> Value {
    let containment = option
        .input("cargo_containment")
        .and_then(|i| i.value.as_str());
    let (state, basis) = match containment {
        Some("box_hold") => (
            "calibrated",
            "The box-hold lightship factor is fitted to Reference Vessels.",
        ),
        Some("sloped_hopper_hold") => (
            "uncalibrated",
            "No calibration case covers the lightship of a sloped-hopper barge: the build-up uses fixed factors.",
        ),
        _ => (
            "uncalibrated",
            "No calibration case covers the lightship of a gas barge: the build-up uses fixed factors.",
        ),
    };
    json!({ "lightship": state, "basis": basis })
}

/// Run every Option of a Design and compare each hash with the one stored in the file.
pub fn run_design(design: &Design) -> Vec<RunOption> {
    run_design_with(design, &Library::built_in(), &[])
}

/// [`run_design`], with Rule Sets loaded at run time (see [`run_option_with`]).
pub fn run_design_with(
    design: &Design,
    library: &Library,
    extra: &[(String, String)],
) -> Vec<RunOption> {
    design
        .options
        .iter()
        .map(|o| {
            let outcome = run_option_with(o, library, extra);
            let hash_status = match (&outcome, &o.results_hash) {
                (RunOutcome::Results { hash, .. }, Some(stored))
                    if hash.eq_ignore_ascii_case(stored) =>
                {
                    HashStatus::Matches
                }
                (RunOutcome::Results { hash, .. }, Some(stored)) => HashStatus::Differs {
                    stored: stored.clone(),
                    computed: hash.clone(),
                },
                (RunOutcome::NotAvailable(_), Some(_)) => HashStatus::NotChecked,
                _ => HashStatus::NotStored,
            };
            RunOption {
                name: o.name.clone(),
                outcome,
                hash_status,
            }
        })
        .collect()
}

/// Is this text a results hash: 64 hexadecimal digits, in either case?
pub fn is_hash_shape(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Write a results hash into every Option that can be computed and has none stored. A stored hash
/// is never replaced: that would hide a change.
///
/// A file names one engine version, so every hash in it must come from that engine. When a hash is
/// written, `engine_version` becomes this engine's. If another Option already holds a hash saved
/// under a different version, no hash is written: the file would claim hashes from an engine that
/// never computed them.
pub fn stamp(design: &mut Design) {
    let this = crate::vdesign::ENGINE_VERSION;
    if design.engine_version != this && design.options.iter().any(|o| o.results_hash.is_some()) {
        return;
    }
    let mut wrote = false;
    for o in &mut design.options {
        if o.results_hash.is_none()
            && let RunOutcome::Results { hash, .. } = run_option(o)
        {
            o.results_hash = Some(hash);
            wrote = true;
        }
    }
    if wrote {
        design.engine_version = this.to_string();
    }
}

/// The warning for an Option whose stored hash differs from this engine's. It does not say why:
/// an engine change and a hand edit of an input look the same to the file.
pub fn hash_warning(name: &str, engine_version: &str, stored: &str, computed: &str) -> String {
    format!(
        "Option '{name}': results differ from the hash saved in the file (saved with engine \
         version {engine_version}; stored hash {}, now {})",
        short(stored),
        short(computed)
    )
}

fn short(hash: &str) -> &str {
    hash.get(..12).unwrap_or(hash)
}

fn f(v: &Value, path: &[&str], digits: usize) -> String {
    let mut cur = v;
    for key in path {
        cur = &cur[*key];
    }
    match cur.as_f64() {
        Some(x) => fixed(x, digits),
        None => "n/a".to_string(),
    }
}

/// The short summary of one Option's results: the main numbers a designer reads first.
pub fn summary_lines(name: &str, results: &Value) -> Vec<String> {
    let r = &results["results"];
    let dims = &results["dims"];
    let mut lines = vec![format!(
        "Option '{name}': {} in a {} ({} x {} x {} m, Cb {})",
        results["cargo"].as_str().unwrap_or("?"),
        results["vessel"].as_str().unwrap_or("?"),
        f(dims, &["loa_m"], 1),
        f(dims, &["beam_m"], 1),
        f(dims, &["depth_m"], 1),
        f(dims, &["cb"], 3),
    )];
    // Issue #358: a pushed convoy, and its totals. The numbers that follow are one barge's.
    let convoy = &results["convoy"];
    if convoy.is_object() {
        let t = &convoy["totals"];
        lines.push(format!(
            "  Convoy: {} abreast x {} in line = {} barges, overall {} x {} m; cargo {} t, \
             lightship {} t, deadweight {} t (the pusher is not in the totals). \
             The numbers below are of one barge.",
            convoy["abreast"],
            convoy["in_line"],
            convoy["barges"],
            f(t, &["overall_length_m"], 1),
            f(t, &["overall_beam_m"], 1),
            f(t, &["cargo_t"], 1),
            f(t, &["lightship_t"], 1),
            f(t, &["deadweight_t"], 1),
        ));
    }
    lines.push(format!(
        "  Cargo: {} t ({})",
        f(r, &["cargo_t"], 1),
        r["governs"].as_str().unwrap_or("?")
    ));
    lines.push(format!(
        "  Lightship {} t, displacement {} t",
        f(r, &["lightship_t"], 1),
        f(r, &["displacement_t"], 1)
    ));
    lines.push(format!(
        "  Draught {} m (cap {} m), freeboard {} m",
        f(r, &["draught_m"], 2),
        f(r, &["draught_max_m"], 2),
        f(r, &["freeboard_m"], 2)
    ));
    lines.push(format!(
        "  Air draft {} m loaded, {} m light",
        f(r, &["air_draft_loaded_m"], 2),
        f(r, &["air_draft_light_m"], 2)
    ));
    // A gas Design: the tank, with its diameter datum said (issue #292).
    let tank = &results["drawing"]["tank"];
    if tank.is_object() {
        lines.push(format!(
            "  Tank: outer diameter {} m to the outside of the insulation ({} mm), steel {} m; \
             wall {} mm ({}), {}, {} MPa",
            f(tank, &["tank_outer_diameter_m"], 2),
            f(tank, &["insulation_thickness_mm"], 0),
            f(tank, &["tank_steel_outer_diameter_m"], 2),
            f(tank, &["cylinder_wall_thickness_mm"], 1),
            match tank["wall_thickness_source"].as_str() {
                Some("override") => "typed",
                _ => "from the formula",
            },
            tank["material"].as_str().unwrap_or("?"),
            f(tank, &["design_pressure_mpa"], 2),
        ));
    }
    // Issue #275: the scope of the check, from the Rule Trace.
    if let Some(scope) = results["rule_trace"]["scope"].as_str() {
        lines.push(format!("  Scope: {scope}"));
    }
    // A failed warning is not a failure: it is listed apart (issue #418). A check with no severity
    // is an error.
    let checks = results["checks"].as_array().cloned().unwrap_or_default();
    let failed_with = |warning: bool| -> Vec<String> {
        checks
            .iter()
            .filter(|c| c["ok"] == json!(false) && (c["severity"] == json!("warning")) == warning)
            .map(|c| c["name"].as_str().unwrap_or("?").to_string())
            .collect()
    };
    let (failed, warned) = (failed_with(false), failed_with(true));
    if failed.is_empty() {
        lines.push(format!(
            "  Checks: all {} pass",
            checks.len() - warned.len()
        ));
    } else {
        lines.push(format!(
            "  Checks: {} of {} fail: {}",
            failed.len(),
            checks.len() - warned.len(),
            failed.join("; ")
        ));
    }
    if !warned.is_empty() {
        lines.push(format!(
            "  Warning checks: {} not met: {}",
            warned.len(),
            warned.join("; ")
        ));
    }
    if let Some(w) = results["warnings"].as_array() {
        for m in w {
            lines.push(format!("  Warning: {}", m.as_str().unwrap_or("?")));
        }
    }
    lines
}
