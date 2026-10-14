//! JSON adapters for the `journey_derating` and `gas_arrangement` parity entry points. They copy
//! the frozen Python driver's input and output shapes (`parity/driver.py`, `to_json`), including
//! which numbers are ints.
//!
//! The driver passes `gas_arrangement` arguments to the Python function as they are (no type
//! coercion), so a JSON integer where Python takes a float would reach the result as an int.
//! Every scalar that is echoed into a result is therefore read strictly (`write 18.0, not 18`).
//! The `depths` and `extra_clearance_m` items only feed float arithmetic, so a whole number is
//! accepted there.

use crate::eval::{
    EvalError, capacity_inputs, capacity_json, checks_json, f64_arg, num, ok, opt_num, raised,
    raised_py, str_arg,
};
use ovcd_core::barge_capacity::BargeCapacityInputs;
use ovcd_core::gas_arrangement::{self as ga, ArrangementFit, FitOptions};
use ovcd_core::journey_derating::{JourneyDeratingInputs, compute as derate};
use ovcd_core::pressure_vessel::PyError;
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
    "adn_tank_cap_m3",
    "block_coefficient",
    "max_tank_od_m",
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
    f.adn_tank_cap_m3 = float_or(o, "adn_tank_cap_m3", f.adn_tank_cap_m3)?;
    f.block_coefficient = float_or(o, "block_coefficient", f.block_coefficient)?;
    f.max_tank_od_m = float_or(o, "max_tank_od_m", f.max_tank_od_m)?;
    Ok(f)
}

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
            "gas_arrangement has no parity function `{other}`"
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
            "journey_derating has no parity function `{function}`"
        )));
    }
    // The driver builds the capacity result first, so its errors come first.
    let cap_inputs = capacity_inputs(&json!({ "inputs": args.get("capacity_inputs") }))?;
    let capacity = match ovcd_core::barge_capacity::compute(&cap_inputs) {
        Ok(r) => r,
        Err(e) => return raised_py(&e),
    };
    // `JourneyDeratingInputs(capacity=..., **args["inputs"])`: no coercion, so floats only.
    let inputs = args
        .get("inputs")
        .ok_or_else(|| EvalError::BadInput("`args.inputs` must be an object".into()))?;
    let o = object_arg(inputs, JOURNEY_INPUT_KEYS)?;
    let need = |name: &str| f64_arg(inputs, name);
    let mut i = JourneyDeratingInputs::new(&capacity, need("loa")?, need("beam")?, need("depth")?);
    i.block_coefficient = float_or(o, "block_coefficient", i.block_coefficient)?;
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
