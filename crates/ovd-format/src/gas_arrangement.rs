//! JSON adapters for the `journey_derating` and `gas_arrangement` entry points. They copy the
//! input and output shapes of the Python functions (dataclasses as objects), including
//! which numbers are ints.
//!
//! The arguments reach the Python function as they are (no type coercion), so a JSON integer
//! where Python takes a float would reach the result as an int. Every scalar that is echoed into a
//! result is therefore read strictly (`write 18.0, not 18`); the Python package sends floats.
//! The `depths` and `extra_clearance_m` items only feed float arithmetic, so a whole number is
//! accepted there.

use crate::eval::{
    EvalError, capacity_inputs, capacity_json, checks_json, f64_arg, num, ok, opt_num, raised,
    raised_py, str_arg, tagged_f64,
};
use crate::objects::capacity_result_from_json;
use ovd_core::barge_capacity::BargeCapacityInputs;
use ovd_core::gas_arrangement::{self as ga, ArrangementFit, FitOptions};
use ovd_core::journey_derating::{JourneyDeratingInputs, compute as derate};
use ovd_core::pressure_vessel::PyError;
use serde_json::{Map, Value, json};

/// A JSON object built key by key, in the order written.
fn object(fields: Vec<(&str, Value)>) -> Value {
    let mut map = Map::new();
    for (k, v) in fields {
        map.insert(k.to_string(), v);
    }
    Value::Object(map)
}

fn object_arg<'a>(args: &'a Value, allowed: &[&str]) -> Result<&'a Map<String, Value>, EvalError> {
    let o = args
        .as_object()
        .ok_or_else(|| EvalError::BadInput("`args` must be an object".into()))?;
    for key in o.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(EvalError::BadInput(format!("unknown argument `{key}`")));
        }
    }
    Ok(o)
}

/// `args.<name>` as a strict float, `default` when the key is absent.
fn float_or(o: &Map<String, Value>, name: &str, default: f64) -> Result<f64, EvalError> {
    match o.get(name) {
        None => Ok(default),
        Some(Value::Number(n)) if n.is_f64() => Ok(n.as_f64().unwrap_or(f64::NAN)),
        Some(v @ Value::String(_)) if tagged_f64(v).is_some() => {
            Ok(tagged_f64(v).unwrap_or(f64::NAN))
        }
        Some(_) => Err(EvalError::BadInput(format!(
            "`args.{name}` must be a float (write 18.0, not 18)"
        ))),
    }
}

fn float_list(o: &Map<String, Value>, name: &str) -> Result<Option<Vec<f64>>, EvalError> {
    let Some(v) = o.get(name) else {
        return Ok(None);
    };
    v.as_array()
        .and_then(|items| items.iter().map(Value::as_f64).collect::<Option<Vec<_>>>())
        .map(Some)
        .ok_or_else(|| EvalError::BadInput(format!("`args.{name}` must be a list of numbers")))
}

fn int_list(o: &Map<String, Value>, name: &str) -> Result<Option<Vec<i64>>, EvalError> {
    let Some(v) = o.get(name) else {
        return Ok(None);
    };
    v.as_array()
        .and_then(|items| items.iter().map(Value::as_i64).collect::<Option<Vec<_>>>())
        .map(Some)
        .ok_or_else(|| EvalError::BadInput(format!("`args.{name}` must be a list of integers")))
}

const FIT_OPTION_KEYS: &[&str] = &[
    "ruleset",
    "fill_ratio",
    "self_propelled",
    "depths",
    "tanks_wide",
    "extra_clearance_m",
    "block_coefficient",
    "max_tank_od_m",
    "bow_rake_to_depth_ratio",
    "pushed_stern_void_m",
    "tanks_in_line",
    "tank_support_height_m",
    "tank_dome_height_m",
    "tank_dome_diameter_m",
    "wing_tank_gap_m",
    "wing_tank_height_ratio",
    "weather_shield_thickness_mm",
    "weather_shield_clearance_m",
    "tank_forming_allowance_pct",
    "tank_dome_wall_thickness_mm",
    "aft_end_space_m",
    "fwd_end_space_m",
    "tank_wall_thickness_mm",
    "tank_design_pressure_mpa",
    "tank_material",
    "tank_allowable_stress_mpa",
    "tank_insulation_thickness_mm",
    "tank_structure_factor",
    "n_thrusters",
    "thruster_envelope_m",
    "thruster_clear_m",
    "thruster_room_length_m",
    "n_battery_units",
    "n_accom_units",
    "container_height_m",
    "container_side_gap_m",
    "container_gap_m",
    "container_foundation_m",
    "wheelhouse_length_m",
    "wheelhouse_width_m",
    "wheelhouse_height_m",
    "wheelhouse_gap_m",
    "wheelhouse_eye_above_floor_m",
    "sightline_margin_m",
    "wheelhouse_max_lift_m",
];

/// The keyword arguments of `fit`, over `base` (the defaults of the function being called).
fn fit_options(o: &Map<String, Value>, base: FitOptions) -> Result<FitOptions, EvalError> {
    let mut f = base;
    if let Some(v) = o.get("ruleset") {
        f.ruleset = v
            .as_str()
            .ok_or_else(|| EvalError::BadInput("`args.ruleset` must be a string".into()))?
            .to_string();
    }
    f.fill_ratio = float_or(o, "fill_ratio", f.fill_ratio)?;
    if let Some(v) = o.get("self_propelled") {
        f.self_propelled = v.as_bool().ok_or_else(|| {
            EvalError::BadInput("`args.self_propelled` must be true or false".into())
        })?;
    }
    if let Some(d) = float_list(o, "depths")? {
        f.depths = d;
    }
    if let Some(t) = int_list(o, "tanks_wide")? {
        f.tanks_wide = t;
    }
    if let Some(e) = float_list(o, "extra_clearance_m")? {
        f.extra_clearance_m = e;
    }
    f.block_coefficient = float_or(o, "block_coefficient", f.block_coefficient)?;
    f.max_tank_od_m = float_or(o, "max_tank_od_m", f.max_tank_od_m)?;
    f.bow_rake_to_depth_ratio = float_or(o, "bow_rake_to_depth_ratio", f.bow_rake_to_depth_ratio)?;
    f.pushed_stern_void_m = float_or(o, "pushed_stern_void_m", f.pushed_stern_void_m)?;
    if let Some(v) = o.get("tanks_in_line") {
        f.tanks_in_line = v.as_i64().ok_or_else(|| {
            EvalError::BadInput("`args.tanks_in_line` must be a whole number".into())
        })?;
    }
    f.tank_support_height_m = float_or(o, "tank_support_height_m", f.tank_support_height_m)?;
    f.tank_dome_height_m = float_or(o, "tank_dome_height_m", f.tank_dome_height_m)?;
    f.tank_dome_diameter_m = float_or(o, "tank_dome_diameter_m", f.tank_dome_diameter_m)?;
    f.wing_tank_gap_m = float_or(o, "wing_tank_gap_m", f.wing_tank_gap_m)?;
    f.wing_tank_height_ratio = float_or(o, "wing_tank_height_ratio", f.wing_tank_height_ratio)?;
    f.weather_shield_thickness_mm = float_or(
        o,
        "weather_shield_thickness_mm",
        f.weather_shield_thickness_mm,
    )?;
    f.weather_shield_clearance_m = float_or(
        o,
        "weather_shield_clearance_m",
        f.weather_shield_clearance_m,
    )?;
    f.tank_forming_allowance_pct = float_or(
        o,
        "tank_forming_allowance_pct",
        f.tank_forming_allowance_pct,
    )?;
    f.tank_dome_wall_thickness_mm = float_or(
        o,
        "tank_dome_wall_thickness_mm",
        f.tank_dome_wall_thickness_mm,
    )?;
    f.aft_end_space_m = float_or(o, "aft_end_space_m", f.aft_end_space_m)?;
    f.fwd_end_space_m = float_or(o, "fwd_end_space_m", f.fwd_end_space_m)?;
    // A key left out keeps the base; `null` asks for the pressure-vessel formula (issue #292).
    match o.get("tank_wall_thickness_mm") {
        None => {}
        Some(Value::Null) => f.tank_wall_thickness_mm = None,
        Some(_) => {
            f.tank_wall_thickness_mm = Some(float_or(o, "tank_wall_thickness_mm", 0.0)?);
        }
    }
    // `null` asks for the IGC 4.23.1.2 minimum of each tank (issue #577).
    match o.get("tank_design_pressure_mpa") {
        None => {}
        Some(Value::Null) => f.tank_design.design_pressure_igc_minimum = true,
        Some(_) => {
            f.tank_design.design_pressure_igc_minimum = false;
            f.tank_design.design_pressure_mpa = float_or(
                o,
                "tank_design_pressure_mpa",
                f.tank_design.design_pressure_mpa,
            )?;
        }
    }
    if let Some(v) = o.get("tank_material") {
        f.tank_design.material = v
            .as_str()
            .ok_or_else(|| EvalError::BadInput("`args.tank_material` must be a string".into()))?
            .to_string();
    }
    match o.get("tank_allowable_stress_mpa") {
        None => {}
        Some(Value::Null) => f.tank_design.allowable_stress_mpa = None,
        Some(_) => {
            f.tank_design.allowable_stress_mpa =
                Some(float_or(o, "tank_allowable_stress_mpa", 0.0)?);
        }
    }
    f.tank_design.insulation_thickness_mm = float_or(
        o,
        "tank_insulation_thickness_mm",
        f.tank_design.insulation_thickness_mm,
    )?;
    f.tank_design.structure_factor =
        float_or(o, "tank_structure_factor", f.tank_design.structure_factor)?;
    // The stern block's design values (issue #4): used by every candidate of a self-propelled hull.
    let whole = |name: &str| -> Result<Option<i64>, EvalError> {
        o.get(name)
            .map(|v| {
                v.as_i64().ok_or_else(|| {
                    EvalError::BadInput(format!("`args.{name}` must be a whole number"))
                })
            })
            .transpose()
    };
    if let Some(n) = whole("n_thrusters")? {
        f.aft_design.n_thrusters = n;
    }
    if let Some(n) = whole("n_battery_units")? {
        f.aft_design.n_battery_units = Some(n);
    }
    if let Some(n) = whole("n_accom_units")? {
        f.aft_design.n_accom_units = Some(n);
    }
    f.aft_design.thruster_envelope_m =
        float_or(o, "thruster_envelope_m", f.aft_design.thruster_envelope_m)?;
    f.aft_design.thruster_clear_m = float_or(o, "thruster_clear_m", f.aft_design.thruster_clear_m)?;
    f.aft_design.thruster_room_length_m = float_or(
        o,
        "thruster_room_length_m",
        f.aft_design.thruster_room_length_m,
    )?;
    f.aft_design.container_height_m =
        float_or(o, "container_height_m", f.aft_design.container_height_m)?;
    f.aft_design.container_side_gap_m =
        float_or(o, "container_side_gap_m", f.aft_design.container_side_gap_m)?;
    f.aft_design.container_gap_m = float_or(o, "container_gap_m", f.aft_design.container_gap_m)?;
    f.aft_design.container_foundation_m = float_or(
        o,
        "container_foundation_m",
        f.aft_design.container_foundation_m,
    )?;
    f.aft_design.wheelhouse_length_m =
        float_or(o, "wheelhouse_length_m", f.aft_design.wheelhouse_length_m)?;
    f.aft_design.wheelhouse_width_m =
        float_or(o, "wheelhouse_width_m", f.aft_design.wheelhouse_width_m)?;
    f.aft_design.wheelhouse_height_m =
        float_or(o, "wheelhouse_height_m", f.aft_design.wheelhouse_height_m)?;
    f.aft_design.wheelhouse_gap_m = float_or(o, "wheelhouse_gap_m", f.aft_design.wheelhouse_gap_m)?;
    f.aft_design.wheelhouse_eye_above_floor_m = float_or(
        o,
        "wheelhouse_eye_above_floor_m",
        f.aft_design.wheelhouse_eye_above_floor_m,
    )?;
    f.aft_design.sightline_margin_m =
        float_or(o, "sightline_margin_m", f.aft_design.sightline_margin_m)?;
    f.aft_design.wheelhouse_max_lift_m = float_or(
        o,
        "wheelhouse_max_lift_m",
        f.aft_design.wheelhouse_max_lift_m,
    )?;
    Ok(f)
}

/// The engine inputs of a `fit` result, as the Python `BargeCapacityInputs` has them. The Python
/// bridge builds that dataclass from this object, so a field of the core that Python lacks (the bow
/// rake ratio and the wing tank values, issue #4) is not listed: it keeps its default there.
fn engine_inputs_json(i: &BargeCapacityInputs) -> Value {
    // `fit` never sets the nested aft or separation inputs.
    debug_assert!(i.aft.is_none() && i.separation.is_none());
    object(vec![
        ("loa", num(i.loa)),
        ("beam", num(i.beam)),
        ("depth", num(i.depth)),
        ("cargo_type", json!(i.cargo_type)),
        ("block_coefficient", num(i.block_coefficient)),
        ("water_density", num(i.water_density)),
        ("required_freeboard", num(i.required_freeboard)),
        ("air_draft_limit_m", opt_num(i.air_draft_limit_m)),
        ("ballast_capacity_t", num(i.ballast_capacity_t)),
        ("aft", Value::Null),
        ("separation", Value::Null),
        ("self_propelled", json!(i.self_propelled)),
        ("pushed_stern_void_m", num(i.pushed_stern_void_m)),
        ("bow_rake_length_m", opt_num(i.bow_rake_length_m)),
        ("bulk_end_bulkhead_m", num(i.bulk_end_bulkhead_m)),
        ("vessel_type", json!(i.vessel_type)),
        (
            "side_clearance_override_m",
            opt_num(i.side_clearance_override_m),
        ),
        ("n_tanks_wide", json!(i.n_tanks_wide)),
        ("n_tanks_long", json!(i.n_tanks_long)),
        ("tank_wall_thickness", opt_num(i.tank_wall_thickness)),
        ("fill_ratio", num(i.fill_ratio)),
        ("dome_height", num(i.dome_height)),
        ("dome_diameter", num(i.dome_diameter)),
        ("tank_support_height", num(i.tank_support_height)),
        ("hold_type", json!(i.hold_type)),
        ("lightship_k_t_per_m3", opt_num(i.lightship_k_t_per_m3)),
        ("wing_void_width", num(i.wing_void_width)),
        (
            "double_bottom_height_bulk",
            num(i.double_bottom_height_bulk),
        ),
        ("hopper_slope_h_per_v", num(i.hopper_slope_h_per_v)),
        ("coaming_height", num(i.coaming_height)),
        ("n_holds", json!(i.n_holds)),
        ("hold_fill_ratio", num(i.hold_fill_ratio)),
        ("bow_void_fraction", num(i.bow_void_fraction)),
        ("stern_void_fraction", num(i.stern_void_fraction)),
        ("inner_bottom_t_mm", num(i.inner_bottom_t_mm)),
        ("hopper_plate_t_mm", num(i.hopper_plate_t_mm)),
        ("bulkhead_t_mm", num(i.bulkhead_t_mm)),
        ("coaming_t_mm", num(i.coaming_t_mm)),
        ("steel_density", num(i.steel_density)),
        ("bulk_structure_factor", num(i.bulk_structure_factor)),
        ("bulk_steel_rate_per_cbm", num(i.bulk_steel_rate_per_cbm)),
        ("bulk_piping_factor", num(i.bulk_piping_factor)),
        ("bulk_outfitting_factor", num(i.bulk_outfitting_factor)),
        ("max_tank_od_m", num(i.max_tank_od_m)),
    ])
}

fn fit_json(f: &ArrangementFit) -> Value {
    object(vec![
        ("loa_m", num(f.loa_m)),
        ("beam_m", num(f.beam_m)),
        ("depth_m", num(f.depth_m)),
        ("cargo", json!(f.cargo)),
        ("ruleset", json!(f.ruleset)),
        ("fill_ratio", num(f.fill_ratio)),
        ("self_propelled", json!(f.self_propelled)),
        ("t_max_m", num(f.t_max_m)),
        ("payload_t", num(f.payload_t)),
        ("volume_capacity_t", num(f.volume_capacity_t)),
        ("binding", json!(f.binding)),
        ("draft_at_payload_m", num(f.draft_at_payload_m)),
        ("cargo_zone_m", num(f.cargo_zone_m)),
        ("stern_void_m", num(f.stern_void_m)),
        ("bow_void_m", num(f.bow_void_m)),
        ("tank_od_m", num(f.tank_od_m)),
        ("n_tanks", json!(f.n_tanks)),
        ("n_tanks_wide", json!(f.n_tanks_wide)),
        ("n_tanks_long", json!(f.n_tanks_long)),
        ("lightship_t", num(f.lightship_t)),
        ("air_draft_light_m", num(f.air_draft_light_m)),
        ("side_clearance_m", num(f.side_clearance_m)),
        ("tank_inner_diameter_m", num(f.tank_inner_diameter_m)),
        (
            "containment_inner_volume_m3",
            num(f.containment_inner_volume_m3),
        ),
        ("per_tank_volume_m3", num(f.per_tank_volume_m3)),
        ("cargo_volume_m3", num(f.cargo_volume_m3)),
        ("density_t_m3", num(f.density_t_m3)),
        ("failed_checks", json!(f.failed_checks)),
        ("feasible", json!(f.feasible)),
        ("candidates_tried", json!(f.candidates_tried)),
        ("engine_inputs", engine_inputs_json(&f.engine_inputs)),
        ("engine_result", capacity_json(&f.engine_result)),
    ])
}

/// `fit(**args)`: the arguments are `loa`, `beam`, `cargo`, `t_max` and the keyword options.
/// The outer `Err` is bad input; the inner one is the Python exception.
pub(crate) fn run_fit(args: &Value) -> Result<Result<ArrangementFit, PyError>, EvalError> {
    let mut allowed = vec!["loa", "beam", "cargo", "t_max"];
    allowed.extend_from_slice(FIT_OPTION_KEYS);
    let o = object_arg(args, &allowed)?;
    let options = fit_options(o, FitOptions::default())?;
    Ok(ga::fit(
        f64_arg(args, "loa")?,
        f64_arg(args, "beam")?,
        str_arg(args, "cargo")?,
        f64_arg(args, "t_max")?,
        &options,
    ))
}

/// Run one `gas_arrangement` function on case-shaped arguments.
pub fn run(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "fit" => match run_fit(args)? {
            Ok(f) => ok(fit_json(&f)),
            Err(e) => raised_py(&e),
        },
        "smallest_viable_loa" => {
            let mut allowed = vec!["beam", "cargo", "t_max", "lo", "hi"];
            allowed.extend_from_slice(FIT_OPTION_KEYS);
            let o = object_arg(args, &allowed)?;
            let options = fit_options(o, FitOptions::for_smallest_viable_loa())?;
            match ga::smallest_viable_loa(
                f64_arg(args, "beam")?,
                str_arg(args, "cargo")?,
                f64_arg(args, "t_max")?,
                float_or(o, "lo", 20.0)?,
                float_or(o, "hi", 120.0)?,
                &options,
            ) {
                Ok(loa) => ok(num(loa)),
                Err(e) => raised_py(&e),
            }
        }
        "required_side_clearance" => {
            object_arg(
                args,
                &[
                    "ruleset",
                    "beam",
                    "total_m3",
                    "per_tank_m3",
                    "insulation_mm",
                ],
            )?;
            match ga::required_side_clearance(
                str_arg(args, "ruleset")?,
                f64_arg(args, "beam")?,
                f64_arg(args, "total_m3")?,
                f64_arg(args, "per_tank_m3")?,
                f64_arg(args, "insulation_mm")?,
            ) {
                Ok(c) => ok(num(c)),
                Err(e) => raised_py(&e),
            }
        }
        other => Err(EvalError::BadInput(format!(
            "gas_arrangement has no function `{other}`"
        ))),
    }
}

const JOURNEY_INPUT_KEYS: &[&str] = &[
    "loa",
    "beam",
    "depth",
    "block_coefficient",
    "water_density",
    "design_depth_m",
    "ukc_m",
    "utilisation_floor",
];

/// Run one `journey_derating` function on case-shaped arguments.
pub fn run_journey(function: &str, args: &Value) -> Result<Value, EvalError> {
    if function != "compute" {
        return Err(EvalError::BadInput(format!(
            "journey_derating has no function `{function}`"
        )));
    }
    // A capacity result the caller passes is used as it is. Without one (the parity cases),
    // Python builds the capacity result first, so its errors come first.
    let capacity = match args.get("capacity") {
        Some(c) => capacity_result_from_json(c, "capacity")?,
        None => {
            let cap_inputs = capacity_inputs(&json!({ "inputs": args.get("capacity_inputs") }))?;
            match ovd_core::barge_capacity::compute(&cap_inputs) {
                Ok(r) => r,
                Err(e) => return raised_py(&e),
            }
        }
    };
    // `JourneyDeratingInputs(capacity=..., **args["inputs"])`: no coercion, so floats only.
    let inputs = args
        .get("inputs")
        .ok_or_else(|| EvalError::BadInput("`args.inputs` must be an object".into()))?;
    let o = object_arg(inputs, JOURNEY_INPUT_KEYS)?;
    let need = |name: &str| f64_arg(inputs, name);
    let mut i = JourneyDeratingInputs::new(&capacity, need("loa")?, need("beam")?, need("depth")?);
    // Issue #448: read, so a 0.1.0 call does not fail; the hull's sections give the displacement.
    let _ = float_or(o, "block_coefficient", 0.85)?;
    i.water_density = float_or(o, "water_density", i.water_density)?;
    i.design_depth_m = float_or(o, "design_depth_m", i.design_depth_m)?;
    i.ukc_m = float_or(o, "ukc_m", i.ukc_m)?;
    i.utilisation_floor = float_or(o, "utilisation_floor", i.utilisation_floor)?;
    match derate(&i) {
        Ok(r) => ok(object(vec![
            ("cargo_capacity_t", num(r.cargo_capacity_t)),
            ("t_max_m", num(r.t_max_m)),
            ("full_load_draft_m", num(r.full_load_draft_m)),
            ("achievable_tonnage_t", num(r.achievable_tonnage_t)),
            ("binding_limit", json!(r.binding_limit)),
            ("volume_utilisation", num(r.volume_utilisation)),
            ("draft_at_achievable_m", num(r.draft_at_achievable_m)),
            ("shed_tonnage_t", num(r.shed_tonnage_t)),
            ("checks", checks_json(&r.checks)),
            ("warnings", json!(r.warnings)),
        ])),
        Err(message) => raised("ValueError", &message),
    }
}
