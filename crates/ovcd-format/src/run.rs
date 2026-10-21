//! Running a saved Design: the Option-to-`simulate` mapping, the results hash, and the text of
//! `ovcd run` (ADR 0007).
//!
//! The results hash is the SHA-256 of the canonical JSON of one Option's results: the `simulate`
//! result object, with object keys in sorted order, no white space, and numbers as `serde_json`
//! writes them (the shortest text that reads back exactly). The core uses the pure-Rust `libm`, so
//! the same inputs give the same bytes on every platform.
//!
//! A pushed convoy is valid in a file but has no results yet: it gets
//! [`CONVOY_NOT_AVAILABLE`] and no numbers. A single self-propelled Hull and a single pushed
//! barge (without its pusher) are computed.

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
    if kind == "pushed_convoy" {
        return Err(CONVOY_NOT_AVAILABLE.to_string());
    }
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
    put("cb", "block_coefficient");
    put("freeboard_min_m", "min_freeboard_m");
    put("lightship_k_t_per_m3", "lightship_factor_t_per_m3");
    put("ruleset", "ruleset");
    put("fill_ratio", "fill_ratio");
    put("max_tank_od_m", "max_tank_od_m");
    put("optimise_depth", "optimise_depth");
    for name in [
        "water_density_t_per_m3",
        "squat_safety_margin_m",
        "tanks_in_line",
        "tank_support_height_m",
        "tank_dome_height_m",
        "tank_dome_diameter_m",
        "wing_tank_gap_m",
        "wing_tank_height_ratio",
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
        "tank_head_type",
        "tank_liquid_pressure",
        "tank_allowable_stress_basis",
        "tank_allowable_stress_mpa",
        "tank_insulation_type",
        "tank_insulation_thickness_mm",
        "tank_structure_factor",
        "bow_rake_length_to_depth_ratio",
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
    ] {
        put(name, name);
    }
    put("eta_d", "propulsive_efficiency");
    put("speed_max_kn", "max_speed_kn");
    put("air_draft_limit_m", "air_draft_limit_m");
    put("draught_limit_m", "draught_limit_m");
    // A null wall thickness asks for the pressure-vessel formula (issue #292); `put` leaves nulls
    // out, and a key left out means the engine's old fixed 25 mm.
    // In the same way a null dome size asks for the dome rule (issue #316).
    for name in [
        "tank_wall_thickness_mm",
        "tank_dome_height_m",
        "tank_dome_diameter_m",
    ] {
        if get(name).is_some_and(|v| v.is_null()) {
            p.insert(name.to_string(), Value::Null);
        }
    }
    // Issue #350: a null block coefficient asks for the rule (the raked box with the Stern Form).
    if get("block_coefficient").is_some_and(|v| v.is_null()) {
        p.insert("cb".to_string(), Value::Null);
    }
    let vessel = match get("cargo_containment").as_ref().and_then(Value::as_str) {
        Some("box_hold") => "box_barge",
        Some("sloped_hopper_hold") => "hopper_barge",
        Some("type_c_tanks") => "gas_barge",
        _ => return Err("the Option has no Cargo Containment".to_string()),
    };
    p.insert("vessel".into(), json!(vessel));
    p.insert("self_propelled".into(), json!(kind == "self_propelled"));
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
    Ok(p)
}

/// The canonical JSON text: sorted keys, no white space.
pub fn canonical_json(v: &Value) -> String {
    let mut out = String::new();
    write_canonical(v, &mut out);
    out
}

fn write_canonical(v: &Value, out: &mut String) {
    match v {
        Value::Object(o) => {
            let mut keys: Vec<&String> = o.keys().collect();
            keys.sort();
            out.push('{');
            for (i, k) in keys.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&Value::String(k.clone()).to_string());
                out.push(':');
                write_canonical(&o[k], out);
            }
            out.push('}');
        }
        Value::Array(a) => {
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(x, out);
            }
            out.push(']');
        }
        other => out.push_str(&other.to_string()),
    }
}

/// Top-level results keys that are not results: drawing geometry, what a view marks, the capital
/// budget with the masses it was worked from, the Message Ids of the result's English (ADR 0015), and
/// the Rule Trace (spec 0004: the stored Rule Set hashes cover the rules). They are left
/// out of the results hash, so adding to one never changes a saved hash, and `tools/design_impact.py`
/// leaves them out of its comparison (it holds the same list; `tests/test_design_impact_unhashed_keys.py` checks
/// that the two lists agree).
pub const UNHASHED_KEYS: [&str; 6] = [
    "drawing",
    "stability_marks",
    "cost_masses",
    "cost",
    "text_ids",
    "rule_trace",
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

/// Compute one Option and check it against its default stack of Rule Sets from `library`, plus the
/// `extra` sets (`id`, `version`) loaded at run time (spec 0004, "Loading at run time").
pub fn run_option_with(
    option: &DesignOption,
    library: &Library,
    extra: &[(String, String)],
) -> RunOutcome {
    let params = match simulate_params_of(option) {
        Ok(p) => p,
        Err(m) if m == CONVOY_NOT_AVAILABLE => return RunOutcome::NotAvailable(m),
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
                let cost = crate::cost::cost_of(option, &v);
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
                    .unwrap_or_default();
                let rule_trace = trace_json(
                    &trace_of(&selection, option, &measured),
                    &selection.warnings,
                );
                if let Some(o) = v.as_object_mut() {
                    o.insert("cost".to_string(), cost);
                    o.insert("rule_trace".to_string(), rule_trace);
                }
                let hash = results_hash(&v);
                RunOutcome::Results { results: v, hash }
            }
        },
        Outcome::Raised(kind, message) => RunOutcome::Failed(format!("{kind}: {message}")),
        Outcome::Refused(message) => RunOutcome::Failed(message),
    }
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
