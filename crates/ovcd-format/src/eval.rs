//! `eval(entry_point, json) -> json`: one dispatch over every entry point of the calculation core.
//! It is the seam the Python package calls (`ovcd-py`, `vessel_designer/_bridge.py`).
//!
//! Input is `{"fn": <function>, "args": {...}}`. The answer is `{"ok": <value>}`, or
//! `{"error": {"type": <Python exception class>, "message": ...}}` where Python would raise.
//! Floats are shortest round-trip text; NaN and infinity are the tagged strings "float:nan",
//! "float:inf" and "float:-inf".

use ovcd_core::aft_arrangement::{AftArrangementInputs, AftArrangementResult};
use ovcd_core::barge_capacity::{BargeCapacityInputs, BargeCapacityResult};
use ovcd_core::barge_cost::{BargeCostInputs, BargeCostRates, BargeCostResult, MappingValue};
use ovcd_core::cargo::CargoProperties;
use ovcd_core::checks::Check;
use ovcd_core::gas_separation::{GasSeparationInputs, GasSeparationResult};
use ovcd_core::hull::HullGeometry;
use ovcd_core::models::{BargeInputs, TankResult, WeightBreakdown};
use ovcd_core::pressure_vessel::PyError;
use ovcd_core::pyint::PyInt;
use ovcd_core::pystr::repr_str;
use ovcd_core::vessel_standards::{Catalogue, VesselStandard};
use ovcd_core::{
    design_grid as core_grid, electric_energy as core_energy, ga_adapter as core_ga,
    resistance as core_resistance, squat as core_squat, waves as core_waves,
};
use serde_json::{Value, json};

/// Why `eval` could not produce a result. A Python `ValueError` is not an `EvalError`: it is a
/// normal result, `{"error": {"type": "ValueError", "message": ...}}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    /// No such entry point.
    UnknownEntryPoint(String),
    /// The entry point exists in the spec but is not ported yet.
    NotPorted(String),
    /// The case input does not have the shape the entry point needs.
    BadInput(String),
}

impl core::fmt::Display for EvalError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EvalError::UnknownEntryPoint(ep) => write!(f, "unknown entry point `{ep}`"),
            EvalError::NotPorted(ep) => write!(f, "entry point `{ep}` is not ported yet"),
            EvalError::BadInput(msg) => write!(f, "bad input: {msg}"),
        }
    }
}

impl std::error::Error for EvalError {}

/// Every entry point.
pub const ENTRY_POINTS: &[&str] = &[
    "cargo",
    "protective_location",
    "pressure_vessel",
    "hull",
    "aft_arrangement",
    "gas_separation",
    "tanks",
    "weights",
    "hydrostatics",
    "barge_capacity",
    "journey_derating",
    "gas_arrangement",
    "gas_stability",
    "squat",
    "waves",
    "resistance",
    "electric_energy",
    "barge_cost",
    "design_grid",
    "ga_adapter",
    "vessel_standards",
    "simulate",
];

/// Parse a case or an answer. `serde_json` keeps every number as written (`arbitrary_precision`),
/// so a Python `int` of any size stays exact.
pub fn parse_case(text: &str) -> Result<Value, String> {
    serde_json::from_str(text).map_err(|e| e.to_string())
}

/// Run one case (`{"fn": ..., "args": ...}`) through the Rust code.
pub fn eval(entry_point: &str, input: &Value) -> Result<Value, EvalError> {
    let function = input
        .get("fn")
        .and_then(Value::as_str)
        .ok_or_else(|| EvalError::BadInput("case input needs a string `fn`".into()))?;
    let args = input.get("args").unwrap_or(&Value::Null);
    match entry_point {
        "cargo" => cargo(function, args),
        "gas_stability" => crate::gas_stability::run(function, args),
        "hull" => hull(function, args),
        "tanks" => tanks(function, args),
        "weights" => weights(function, args),
        "hydrostatics" => hydrostatics(function, args),
        "barge_capacity" => barge_capacity(function, args),
        "journey_derating" => crate::gas_arrangement::run_journey(function, args),
        "gas_arrangement" => crate::gas_arrangement::run(function, args),
        "design_grid" => design_grid(function, args),
        "ga_adapter" => ga_adapter(function, args),
        "simulate" => crate::simulate::run(function, args),
        "squat" => squat(function, args),
        "waves" => waves(function, args),
        "resistance" => resistance(function, args),
        "electric_energy" => electric_energy(function, args),
        "aft_arrangement" => aft_arrangement(function, args),
        "gas_separation" => gas_separation(function, args),
        "barge_cost" => barge_cost(function, args),
        "vessel_standards" => vessel_standards(function, args),
        "protective_location" => protective_location(function, args),
        "pressure_vessel" => pressure_vessel(function, args),
        ep if ENTRY_POINTS.contains(&ep) => Err(EvalError::NotPorted(ep.into())),
        ep => Err(EvalError::UnknownEntryPoint(ep.into())),
    }
}

/// A float result, tagged: JSON has no `inf` or `nan`, so they
/// become the strings `"float:inf"`, `"float:-inf"` and `"float:nan"`.
pub(crate) fn num(x: f64) -> Value {
    if x.is_nan() {
        json!("float:nan")
    } else if x.is_infinite() {
        json!(if x > 0.0 { "float:inf" } else { "float:-inf" })
    } else {
        json!(x)
    }
}

pub(crate) fn ok(value: Value) -> Result<Value, EvalError> {
    Ok(json!({ "ok": value }))
}

/// A raised Python exception of any class, recorded as its class name and message.
pub(crate) fn raised(type_name: &str, message: &str) -> Result<Value, EvalError> {
    Ok(json!({ "error": { "type": type_name, "message": message } }))
}

/// A JSON integer holding every digit of a Python `int`, whatever its size.
fn int_value(i: &PyInt) -> Value {
    serde_json::from_str(i.as_str()).expect("a PyInt is valid JSON integer text")
}

/// A Python `ValueError`, recorded as its class name and message.
fn value_error(message: String) -> Result<Value, EvalError> {
    Ok(json!({ "error": { "type": "ValueError", "message": message } }))
}

fn nums(xs: &[f64]) -> Value {
    Value::Array(xs.iter().map(|x| num(*x)).collect())
}

/// A Python exception of any recorded type, from the aft_arrangement port.
fn py_exception(e: &ovcd_core::aft_arrangement::PyException) -> Result<Value, EvalError> {
    Ok(json!({ "error": { "type": e.type_name(), "message": e.message() } }))
}

/// A Python exception other than `ValueError`, recorded the same way (class name and message).
fn py_error(kind: &str, message: String) -> Result<Value, EvalError> {
    Ok(json!({ "error": { "type": kind, "message": message } }))
}

/// A float argument. A JSON integer is refused: Python would keep an `int` and may echo it back
/// as an `int`, which the Rust port does not model, so a case must write `18.0`, not `18`.
pub(crate) fn f64_arg(args: &Value, name: &str) -> Result<f64, EvalError> {
    match args.get(name) {
        Some(Value::Number(n)) if n.is_f64() => Ok(n.as_f64().unwrap_or(f64::NAN)),
        _ => Err(EvalError::BadInput(format!(
            "`args.{name}` must be a float (write 18.0, not 18)"
        ))),
    }
}

/// A float argument with a default when the key is absent (a Python default argument).
fn f64_arg_or(args: &Value, name: &str, default: f64) -> Result<f64, EvalError> {
    if args.get(name).is_none() {
        Ok(default)
    } else {
        f64_arg(args, name)
    }
}

/// A string argument that may be null or absent (Python `None`).
fn opt_str_arg<'a>(args: &'a Value, name: &str) -> Result<Option<&'a str>, EvalError> {
    match args.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        Some(_) => Err(EvalError::BadInput(format!(
            "`args.{name}` must be a string or null"
        ))),
    }
}

pub(crate) fn str_arg<'a>(args: &'a Value, name: &str) -> Result<&'a str, EvalError> {
    args.get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| EvalError::BadInput(format!("`args.{name}` must be a string")))
}

fn cargo_json(c: &CargoProperties) -> Value {
    json!({
        "name": c.name,
        "density": c.density,
        "temperature": c.temperature,
        "pressure": c.pressure,
        "description": c.description,
        "phase": c.phase,
    })
}

fn cargo(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "get_cargo" => match ovcd_core::cargo::get_cargo(str_arg(args, "cargo_type")?) {
            Ok(c) => ok(cargo_json(c)),
            Err(message) => value_error(message),
        },
        "cargo_database" => ok(Value::Array(
            ovcd_core::cargo::CARGO_DATABASE
                .iter()
                .map(|(key, c)| json!({ "key": key, "cargo": cargo_json(c) }))
                .collect(),
        )),
        other => Err(EvalError::BadInput(format!(
            "cargo has no function `{other}`"
        ))),
    }
}

/// Reject any key of `args.inputs` that is not an input field, as Python's `build` does.
fn inputs_object<'a>(
    args: &'a Value,
    fields: &[&str],
) -> Result<&'a serde_json::Map<String, Value>, EvalError> {
    let inputs = args
        .get("inputs")
        .and_then(Value::as_object)
        .ok_or_else(|| EvalError::BadInput("`args.inputs` must be an object".into()))?;
    for key in inputs.keys() {
        if !fields.contains(&key.as_str()) {
            return Err(EvalError::BadInput(format!("unknown input field `{key}`")));
        }
    }
    Ok(inputs)
}

fn f64_field(
    inputs: &serde_json::Map<String, Value>,
    name: &str,
) -> Result<Option<f64>, EvalError> {
    match inputs.get(name) {
        None => Ok(None),
        Some(v) => v
            .as_f64()
            .map(Some)
            .ok_or_else(|| EvalError::BadInput(format!("`inputs.{name}` must be a number"))),
    }
}

fn i64_field(
    inputs: &serde_json::Map<String, Value>,
    name: &str,
) -> Result<Option<i64>, EvalError> {
    match inputs.get(name) {
        None => Ok(None),
        Some(v) => v
            .as_i64()
            .map(Some)
            .ok_or_else(|| EvalError::BadInput(format!("`inputs.{name}` must be an integer"))),
    }
}

pub(crate) fn checks_json(checks: &[Check]) -> Value {
    Value::Array(
        checks
            .iter()
            .map(|c| {
                json!({
                    "name": c.name,
                    "ok": c.ok,
                    "message": c.message,
                    "severity": c.severity,
                })
            })
            .collect(),
    )
}

const AFT_FIELDS: &[&str] = &[
    "beam",
    "depth",
    "n_thrusters",
    "thruster_envelope_m",
    "thruster_clear_m",
    "thruster_room_length_m",
    "n_battery_units",
    "n_accom_units",
    "container_length_m",
    "container_width_m",
    "container_height_m",
    "container_side_gap_m",
    "container_gap_m",
    "container_foundation_m",
    "wheelhouse_length_m",
    "wheelhouse_width_m",
    "wheelhouse_height_m",
    "wheelhouse_gap_m",
    "wheelhouse_eye_above_floor_m",
    "sightline_obstruction_m",
    "sightline_margin_m",
    "wheelhouse_max_lift_m",
];

fn aft_inputs(args: &Value) -> Result<AftArrangementInputs, EvalError> {
    let o = inputs_object(args, AFT_FIELDS)?;
    let need = |name: &str| {
        f64_field(o, name)?
            .ok_or_else(|| EvalError::BadInput(format!("`inputs.{name}` is required")))
    };
    let mut i = AftArrangementInputs::new(need("beam")?, need("depth")?);
    macro_rules! set_f64 {
        ($($field:ident),*) => {$(
            if let Some(v) = f64_field(o, stringify!($field))? { i.$field = v; }
        )*};
    }
    macro_rules! set_i64 {
        ($($field:ident),*) => {$(
            if let Some(v) = i64_field(o, stringify!($field))? { i.$field = v; }
        )*};
    }
    set_i64!(n_thrusters, n_battery_units, n_accom_units);
    set_f64!(
        thruster_envelope_m,
        thruster_clear_m,
        thruster_room_length_m,
        container_length_m,
        container_width_m,
        container_height_m,
        container_side_gap_m,
        container_gap_m,
        container_foundation_m,
        wheelhouse_length_m,
        wheelhouse_width_m,
        wheelhouse_height_m,
        wheelhouse_gap_m,
        wheelhouse_eye_above_floor_m,
        sightline_margin_m,
        wheelhouse_max_lift_m
    );
    // `sightline_obstruction_m` may be an explicit null: no cargo to see over.
    if let Some(v) = o.get("sightline_obstruction_m") {
        i.sightline_obstruction_m = if v.is_null() {
            None
        } else {
            f64_field(o, "sightline_obstruction_m")?
        };
    }
    Ok(i)
}

fn aft_json(r: &AftArrangementResult) -> Value {
    json!({
        "length_m": num(r.length_m),
        "zones": r.zones.iter().map(|(l, a, f)| json!([l, num(*a), num(*f)])).collect::<Vec<_>>(),
        "thruster_offsets_m": nums(&r.thruster_offsets_m),
        "thruster_room_length_m": num(r.thruster_room_length_m),
        "container_x0_m": num(r.container_x0_m),
        "container_x1_m": num(r.container_x1_m),
        "container_total_width_m": num(r.container_total_width_m),
        "container_top_above_keel_m": num(r.container_top_above_keel_m),
        "container_labels": r.container_labels,
        "container_unit_width_m": num(r.container_unit_width_m),
        "container_side_gap_m": num(r.container_side_gap_m),
        "container_rows": r.container_rows,
        "container_per_row": r.container_per_row,
        "container_row_length_m": num(r.container_row_length_m),
        "container_row_gap_m": num(r.container_row_gap_m),
        "wheelhouse_x0_m": num(r.wheelhouse_x0_m),
        "wheelhouse_x1_m": num(r.wheelhouse_x1_m),
        "wheelhouse_floor_lowered_m": num(r.wheelhouse_floor_lowered_m),
        "wheelhouse_top_lowered_m": num(r.wheelhouse_top_lowered_m),
        "wheelhouse_floor_raised_m": num(r.wheelhouse_floor_raised_m),
        "wheelhouse_top_raised_m": num(r.wheelhouse_top_raised_m),
        "wheelhouse_lift_m": num(r.wheelhouse_lift_m),
        "wheelhouse_width_m": num(r.wheelhouse_width_m),
        "wheelhouse_height_m": num(r.wheelhouse_height_m),
        "block_top_lowered_m": num(r.block_top_lowered_m),
        "checks": checks_json(&r.checks),
        "warnings": r.warnings,
    })
}

fn aft_arrangement(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "compute" => match ovcd_core::aft_arrangement::compute(&aft_inputs(args)?) {
            Ok(r) => ok(aft_json(&r)),
            Err(e) => py_exception(&e),
        },
        other => Err(EvalError::BadInput(format!(
            "aft_arrangement has no function `{other}`"
        ))),
    }
}

const GAS_SEPARATION_FIELDS: &[&str] = &[
    "cofferdam_m",
    "opening_setback_m",
    "fwd_end_bulkhead_m",
    "is_pressure_tank",
    "side_clearance_m",
    "double_bottom_m",
];

fn gas_separation_inputs(args: &Value) -> Result<GasSeparationInputs, EvalError> {
    let o = inputs_object(args, GAS_SEPARATION_FIELDS)?;
    let mut i = GasSeparationInputs::default();
    macro_rules! set_f64 {
        ($($field:ident),*) => {$(
            if let Some(v) = f64_field(o, stringify!($field))? { i.$field = v; }
        )*};
    }
    set_f64!(
        cofferdam_m,
        opening_setback_m,
        fwd_end_bulkhead_m,
        side_clearance_m,
        double_bottom_m
    );
    if let Some(v) = o.get("is_pressure_tank") {
        i.is_pressure_tank = v.as_bool().ok_or_else(|| {
            EvalError::BadInput("`inputs.is_pressure_tank` must be a boolean".into())
        })?;
    }
    Ok(i)
}

fn gas_separation_json(r: &GasSeparationResult) -> Value {
    json!({
        "aft_separation_m": num(r.aft_separation_m),
        "fwd_separation_m": num(r.fwd_separation_m),
        "cofferdam_m": num(r.cofferdam_m),
        "service_space_m": num(r.service_space_m),
        "total_length_taken_m": num(r.total_length_taken_m),
        "checks": checks_json(&r.checks),
        "warnings": r.warnings,
    })
}

fn gas_separation(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "compute" => match ovcd_core::gas_separation::compute(&gas_separation_inputs(args)?) {
            Ok(r) => ok(gas_separation_json(&r)),
            Err(message) => value_error(message),
        },
        other => Err(EvalError::BadInput(format!(
            "gas_separation has no function `{other}`"
        ))),
    }
}

/// A Python `KeyError`: `str(KeyError(msg))` is `repr(msg)`.
fn key_error(message: &str) -> Result<Value, EvalError> {
    Ok(json!({ "error": { "type": "KeyError", "message": repr_str(message) } }))
}

fn number_arg(value: &Value, what: &str) -> Result<f64, EvalError> {
    value
        .as_f64()
        .ok_or_else(|| EvalError::BadInput(format!("`{what}` must be a number")))
}

fn rates_json(r: &BargeCostRates) -> Value {
    json!({
        "hull_steel_rate": r.hull_steel_rate,
        "tank_shell_rate": r.tank_shell_rate,
        "tank_supports_rate": r.tank_supports_rate,
        "insulation_rate": r.insulation_rate,
        "piping_rate": r.piping_rate,
        "outfitting_rate": r.outfitting_rate,
        "cargo_system_lump": r.cargo_system_lump,
        "paint_anodes_misc": r.paint_anodes_misc,
        "battery_rate_per_kwh": r.battery_rate_per_kwh,
        "propulsion_rate_per_kw": r.propulsion_rate_per_kw,
        "wheelhouse_accom_lump": r.wheelhouse_accom_lump,
        "tug_outfitting_lump": r.tug_outfitting_lump,
        "yard_margin_fraction": r.yard_margin_fraction,
        "delivery_lump": r.delivery_lump,
        "currency": r.currency,
    })
}

fn cost_result_json(r: &BargeCostResult) -> Value {
    let breakdown = |rows: &[(&str, PyInt)]| {
        Value::Object(
            rows.iter()
                .map(|(k, v)| (k.to_string(), int_value(v)))
                .collect(),
        )
    };
    json!({
        "barge_breakdown": breakdown(&r.barge_breakdown),
        "tug_breakdown": breakdown(&r.tug_breakdown),
        "barge_subtotal": int_value(&r.barge_subtotal),
        "tug_subtotal": int_value(&r.tug_subtotal),
        "yard_margin": int_value(&r.yard_margin),
        "delivery": r.delivery,
        "unit_total": int_value(&r.unit_total),
        "checks": checks_json(&r.checks),
        "rates": rates_json(&r.rates),
    })
}

fn cost_inputs(args: &Value) -> Result<BargeCostInputs, EvalError> {
    let mut inp = BargeCostInputs::default();
    let Some(map) = args.get("inputs").and_then(Value::as_object) else {
        return Err(EvalError::BadInput(
            "`args.inputs` must be an object".into(),
        ));
    };
    for (key, value) in map {
        let x = small_int_number(value, &format!("args.inputs.{key}"))?;
        match key.as_str() {
            "hull_steel_t" => inp.hull_steel_t = x,
            "wing_plate_t" => inp.wing_plate_t = x,
            "weather_shield_t" => inp.weather_shield_t = x,
            "tank_shell_t" => inp.tank_shell_t = x,
            "tank_supports_t" => inp.tank_supports_t = x,
            "insulation_t" => inp.insulation_t = x,
            "piping_machinery_t" => inp.piping_machinery_t = x,
            "outfitting_t" => inp.outfitting_t = x,
            "tug_hull_steel_t" => inp.tug_hull_steel_t = x,
            "battery_kwh" => inp.battery_kwh = x,
            "propulsion_kw" => inp.propulsion_kw = x,
            other => {
                return Err(EvalError::BadInput(format!(
                    "barge_cost inputs have no field `{other}`"
                )));
            }
        }
    }
    Ok(inp)
}

fn mapping_value(value: &Value, key: &str) -> Result<MappingValue, EvalError> {
    if let Some(i) = value.as_i64() {
        Ok(MappingValue::Int(i))
    } else if let Some(digits) = big_int_digits(value) {
        Ok(MappingValue::BigInt(digits))
    } else if let Some(f) = value.as_f64() {
        Ok(MappingValue::Float(f))
    } else if let Some(s) = value.as_str() {
        Ok(MappingValue::Text(s.to_string()))
    } else {
        Err(EvalError::BadInput(format!(
            "`args.rates_mapping.{key}` must be a number or text"
        )))
    }
}

/// The exact digits of a JSON integer literal that does not fit `i64` (Python keeps such an `int`).
fn big_int_digits(value: &Value) -> Option<String> {
    let Value::Number(n) = value else { return None };
    let text = n.to_string();
    let digits = text.strip_prefix('-').unwrap_or(&text);
    (!digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())).then_some(text)
}

/// A number the core holds as `f64`, refused when it is an integer literal beyond 2^53: Python
/// compares such ints exactly, and rounding them to a double could change a verdict.
fn exact_number(value: &Value, what: &str) -> Result<f64, EvalError> {
    if let Some(digits) = big_int_digits(value) {
        let magnitude = digits.trim_start_matches('-');
        let exact = magnitude.parse::<u64>().is_ok_and(|m| m <= 1 << 53);
        if !exact {
            return Err(EvalError::BadInput(format!(
                "`{what}` is an integer beyond 2^53, which the core cannot hold exactly; write it as a float"
            )));
        }
    }
    number_arg(value, what)
}

/// Integer literals up to this size cannot make a product or a sum that `f64` holds inexactly.
const BARGE_INT_LIMIT: u64 = 1 << 24;

/// A barge-cost number held as `f64`. Python keeps a JSON integer as an exact `int` and does exact
/// integer arithmetic; the core rounds every product and sum to a double. The two agree while
/// every integer product and sum stays below 2^53, which holds when each integer literal is at
/// most 2^24. A larger integer is refused loudly; write it as a float.
fn small_int_number(value: &Value, what: &str) -> Result<f64, EvalError> {
    let too_big = big_int_digits(value).is_some_and(|digits| {
        digits
            .trim_start_matches('-')
            .parse::<u64>()
            .map_or(true, |m| m > BARGE_INT_LIMIT)
    });
    if too_big {
        return Err(EvalError::BadInput(format!(
            "`{what}` is an integer beyond 2^24, which the core cannot hold exactly in a product; write it as a float"
        )));
    }
    number_arg(value, what)
}

fn explicit_rates(map: &serde_json::Map<String, Value>) -> Result<BargeCostRates, EvalError> {
    let get = |name: &str| -> Result<f64, EvalError> {
        small_int_number(
            map.get(name)
                .ok_or_else(|| EvalError::BadInput(format!("`args.rates.{name}` is missing")))?,
            &format!("args.rates.{name}"),
        )
    };
    Ok(BargeCostRates {
        hull_steel_rate: get("hull_steel_rate")?,
        tank_shell_rate: get("tank_shell_rate")?,
        tank_supports_rate: get("tank_supports_rate")?,
        insulation_rate: get("insulation_rate")?,
        piping_rate: get("piping_rate")?,
        outfitting_rate: get("outfitting_rate")?,
        cargo_system_lump: get("cargo_system_lump")?,
        paint_anodes_misc: get("paint_anodes_misc")?,
        battery_rate_per_kwh: get("battery_rate_per_kwh")?,
        propulsion_rate_per_kw: get("propulsion_rate_per_kw")?,
        wheelhouse_accom_lump: get("wheelhouse_accom_lump")?,
        tug_outfitting_lump: get("tug_outfitting_lump")?,
        yard_margin_fraction: get("yard_margin_fraction")?,
        delivery_lump: get("delivery_lump")?,
        currency: match map.get("currency") {
            None => "USD".to_string(),
            Some(v) => v
                .as_str()
                .ok_or_else(|| EvalError::BadInput("`args.rates.currency` must be text".into()))?
                .to_string(),
        },
    })
}

fn barge_cost(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "compute" => {
            let inputs = cost_inputs(args)?;
            let rates = if let Some(mapping) = args.get("rates_mapping") {
                let Some(map) = mapping.as_object() else {
                    return Err(EvalError::BadInput(
                        "`args.rates_mapping` must be an object".into(),
                    ));
                };
                let pairs = map
                    .iter()
                    .map(|(k, v)| Ok((k.clone(), mapping_value(v, k)?)))
                    .collect::<Result<Vec<_>, EvalError>>()?;
                BargeCostRates::from_mapping(&pairs).map_err(|e| (e.type_name, e.message))
            } else {
                match args.get("rates") {
                    None | Some(Value::Null) => {
                        ovcd_core::barge_cost::example_rates().map_err(|m| ("ValueError", m))
                    }
                    Some(Value::Object(map)) => Ok(explicit_rates(map)?),
                    Some(_) => {
                        return Err(EvalError::BadInput(
                            "`args.rates` must be an object or null".into(),
                        ));
                    }
                }
            };
            match rates {
                Ok(rates) => match ovcd_core::barge_cost::compute(&inputs, &rates) {
                    Ok(result) => ok(cost_result_json(&result)),
                    Err(e) => raised(e.type_name(), e.message()),
                },
                Err((type_name, message)) => raised(type_name, &message),
            }
        }
        "parse_flat_yaml" => match ovcd_core::barge_cost::parse_flat_yaml(str_arg(args, "text")?) {
            Ok(pairs) => ok(Value::Object(
                pairs.into_iter().map(|(k, v)| (k, json!(v))).collect(),
            )),
            Err(message) => value_error(message),
        },
        other => Err(EvalError::BadInput(format!(
            "barge_cost has no function `{other}`"
        ))),
    }
}

fn standard_json(s: &VesselStandard) -> Value {
    json!({
        "standard_id": s.standard_id,
        "family": s.family,
        "class_name": s.class_name,
        "formation": s.formation,
        "loa_m": [s.loa_m.0, s.loa_m.1],
        "beam_m": s.beam_m,
        "draught_m": [s.draught_m.0, s.draught_m.1],
        "dwt_t": [s.dwt_t.0, s.dwt_t.1],
        "n_abreast": s.n_abreast,
        "n_in_line": s.n_in_line,
        "unit_beam_m": s.unit_beam_m,
        "min_bridge_clearance_m": s.min_bridge_clearance_m,
        "moulded_depth_m": s.moulded_depth_m,
        "source": s.source,
        "cargo_modes": s.cargo_modes,
        "notes": s.notes,
    })
}

fn pair_field(fields: &Value, name: &str) -> Result<(f64, f64), EvalError> {
    let bad = || EvalError::BadInput(format!("`{name}` must be a list of two numbers"));
    let items = fields.get(name).and_then(Value::as_array).ok_or_else(bad)?;
    match items.as_slice() {
        [a, b] => {
            let _ = (a.as_f64().ok_or_else(bad)?, b.as_f64().ok_or_else(bad)?);
            Ok((exact_number(a, name)?, exact_number(b, name)?))
        }
        _ => Err(bad()),
    }
}

fn text_field(fields: &Value, name: &str, default: Option<&str>) -> Result<String, EvalError> {
    match (fields.get(name), default) {
        (Some(v), _) => v
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| EvalError::BadInput(format!("`{name}` must be text"))),
        (None, Some(d)) => Ok(d.to_string()),
        (None, None) => Err(EvalError::BadInput(format!("`{name}` is required"))),
    }
}

fn optional_number(fields: &Value, name: &str) -> Result<Option<f64>, EvalError> {
    match fields.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => Ok(Some(exact_number(v, name)?)),
    }
}

fn count_field(fields: &Value, name: &str) -> Result<i64, EvalError> {
    match fields.get(name) {
        None => Ok(1),
        Some(v) => {
            let n = v
                .as_i64()
                .ok_or_else(|| EvalError::BadInput(format!("`{name}` must be an integer")))?;
            if n.unsigned_abs() > 1 << 53 {
                return Err(EvalError::BadInput(format!(
                    "`{name}` is an integer beyond 2^53, which the core cannot hold exactly"
                )));
            }
            Ok(n)
        }
    }
}

/// One `VesselStandard(**fields)`, with the Python dataclass defaults.
fn standard_from_fields(fields: &Value) -> Result<VesselStandard, EvalError> {
    let cargo_modes = match fields.get("cargo_modes") {
        None => vec!["dry_bulk".to_string(), "liquid_gas".to_string()],
        Some(v) => v
            .as_array()
            .ok_or_else(|| EvalError::BadInput("`cargo_modes` must be a list".into()))?
            .iter()
            .map(|m| {
                m.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| EvalError::BadInput("`cargo_modes` must hold text".into()))
            })
            .collect::<Result<Vec<_>, _>>()?,
    };
    // Python multiplies `unit_beam_m * n_abreast` as exact ints when both are ints; the core
    // multiplies doubles. Refuse an integer product that a double would round.
    let n_abreast = count_field(fields, "n_abreast")?;
    let unit_int = fields.get("unit_beam_m").and_then(Value::as_i64);
    if unit_int.is_some_and(|unit| i128::from(unit).abs() * i128::from(n_abreast).abs() > 1 << 53) {
        return Err(EvalError::BadInput(
            "`unit_beam_m` x `n_abreast` is an integer beyond 2^53, which the core cannot hold exactly; write the unit beam as a float".into(),
        ));
    }
    Ok(VesselStandard {
        standard_id: text_field(fields, "standard_id", None)?,
        family: text_field(fields, "family", None)?,
        class_name: text_field(fields, "class_name", None)?,
        formation: text_field(fields, "formation", None)?,
        loa_m: pair_field(fields, "loa_m")?,
        beam_m: exact_number(
            fields
                .get("beam_m")
                .ok_or_else(|| EvalError::BadInput("`beam_m` is required".into()))?,
            "beam_m",
        )?,
        draught_m: pair_field(fields, "draught_m")?,
        dwt_t: pair_field(fields, "dwt_t")?,
        n_abreast,
        n_in_line: count_field(fields, "n_in_line")?,
        unit_beam_m: optional_number(fields, "unit_beam_m")?,
        min_bridge_clearance_m: optional_number(fields, "min_bridge_clearance_m")?,
        moulded_depth_m: optional_number(fields, "moulded_depth_m")?,
        source: text_field(fields, "source", Some(""))?,
        cargo_modes,
        notes: text_field(fields, "notes", Some(""))?,
    })
}

fn vessel_standards(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "get_standard" => {
            match ovcd_core::vessel_standards::get_standard(str_arg(args, "standard_id")?) {
                Ok(s) => ok(standard_json(&s)),
                Err(message) => key_error(&message),
            }
        }
        "standards_for" => {
            match ovcd_core::vessel_standards::standards_for(str_arg(args, "cargo_mode")?) {
                Ok(list) => ok(Value::Array(list.iter().map(standard_json).collect())),
                Err(message) => value_error(message),
            }
        }
        "validate_catalogue" => {
            let mut catalogue: Catalogue = ovcd_core::vessel_standards::built_in_catalogue();
            if let Some(extra) = args.get("extra").and_then(Value::as_object) {
                for (key, fields) in extra {
                    let standard = standard_from_fields(fields)?;
                    // A dict assignment keeps an existing key's position.
                    match catalogue.iter_mut().find(|(k, _)| k == key) {
                        Some(slot) => slot.1 = standard,
                        None => catalogue.push((key.clone(), standard)),
                    }
                }
            }
            ok(checks_json(
                &ovcd_core::vessel_standards::validate_catalogue(&catalogue),
            ))
        }
        other => Err(EvalError::BadInput(format!(
            "vessel_standards has no function `{other}`"
        ))),
    }
}

fn protective_location(function: &str, args: &Value) -> Result<Value, EvalError> {
    use ovcd_core::protective_location as pl;
    // A `ValueError` is a normal result; anything else is a bad case.
    fn lift<T>(r: Result<T, String>, f: impl FnOnce(T) -> Value) -> Result<Value, EvalError> {
        match r {
            Ok(v) => ok(f(v)),
            Err(message) => value_error(message),
        }
    }
    let vessel_type = |args: &Value| -> Result<Result<Option<pl::VesselType>, String>, EvalError> {
        Ok(match opt_str_arg(args, "vessel_type")? {
            None => Ok(None),
            Some(text) => pl::VesselType::from_value(text).map(Some),
        })
    };
    match function {
        "distance_d_vc" => ok(num(pl::distance_d_vc(f64_arg(args, "vc")?))),
        "d_vc_exact" => ok(num(pl::d_vc_exact(f64_arg(args, "vc_individual_m3")?))),
        "required_side_protection" => {
            let (beam, d_vc) = (f64_arg(args, "beam")?, f64_arg(args, "d_vc")?);
            match vessel_type(args)? {
                Ok(vt) => lift(pl::required_side_protection(vt, beam, d_vc), num),
                Err(message) => value_error(message),
            }
        }
        "required_bottom_protection" => {
            let (beam, d_vc) = (f64_arg(args, "beam")?, f64_arg(args, "d_vc")?);
            match vessel_type(args)? {
                Ok(vt) => ok(num(pl::required_bottom_protection(beam, d_vc, vt))),
                Err(message) => value_error(message),
            }
        }
        "compute_required_clearances" => lift(
            pl::compute_required_clearances(
                str_arg(args, "vessel_type_str")?,
                f64_arg(args, "beam")?,
                f64_arg(args, "d_vc")?,
            ),
            |r| {
                json!({
                    "cs_required": num(r.cs_required),
                    "cb_required": num(r.cb_required),
                    "d_vc": num(r.d_vc),
                })
            },
        ),
        "check_protective_location" => lift(
            pl::check_protective_location(
                str_arg(args, "vessel_type_str")?,
                f64_arg(args, "beam")?,
                f64_arg(args, "total_tank_volume")?,
                f64_arg(args, "side_clearance")?,
                f64_arg(args, "double_bottom_height")?,
                f64_arg(args, "tank_support_height")?,
            ),
            |r| {
                json!({
                    "vessel_type": r.vessel_type,
                    "d_vc": num(r.d_vc),
                    "total_tank_volume": num(r.total_tank_volume),
                    "cs_required": num(r.cs_required),
                    "cs_actual": num(r.cs_actual),
                    "cs_ok": r.cs_ok,
                    "cb_required": num(r.cb_required),
                    "cb_actual": num(r.cb_actual),
                    "cb_ok": r.cb_ok,
                })
            },
        ),
        other => Err(EvalError::BadInput(format!(
            "protective_location has no function `{other}`"
        ))),
    }
}

fn pressure_vessel(function: &str, args: &Value) -> Result<Value, EvalError> {
    use ovcd_core::pressure_vessel as pv;
    let weld = |args: &Value| f64_arg_or(args, "weld_efficiency", pv::DEFAULT_WELD_EFFICIENCY);
    let thickness = |r: Result<f64, pv::PyError>| match r {
        Ok(t) => ok(num(t)),
        Err(e) => py_error(e.kind, e.message),
    };
    match function {
        "get_tank_material" => match pv::get_tank_material(str_arg(args, "material_key")?) {
            Ok(m) => ok(json!({
                "name": m.name,
                "allowable_stress": num(m.allowable_stress),
                "density": num(m.density),
                "min_thickness": num(m.min_thickness),
                "corrosion_allowance": num(m.corrosion_allowance),
                "description": m.description,
            })),
            Err(e) => py_error(e.kind, e.message),
        },
        "cylinder_wall_thickness" => thickness(pv::cylinder_wall_thickness(
            f64_arg(args, "design_pressure_mpa")?,
            f64_arg(args, "inner_radius_mm")?,
            f64_arg(args, "allowable_stress_mpa")?,
            weld(args)?,
        )),
        "hemisphere_head_thickness" => thickness(pv::hemisphere_head_thickness(
            f64_arg(args, "design_pressure_mpa")?,
            f64_arg(args, "inner_radius_mm")?,
            f64_arg(args, "allowable_stress_mpa")?,
            weld(args)?,
        )),
        "ellipsoidal_head_thickness" => thickness(pv::ellipsoidal_head_thickness(
            f64_arg(args, "design_pressure_mpa")?,
            f64_arg(args, "inner_diameter_mm")?,
            f64_arg(args, "allowable_stress_mpa")?,
            weld(args)?,
        )),
        "calculate_wall_thicknesses" => {
            match pv::calculate_wall_thicknesses(
                f64_arg(args, "outer_diameter_m")?,
                f64_arg(args, "design_pressure_mpa")?,
                str_arg(args, "material_key")?,
                str_arg(args, "head_type")?,
                weld(args)?,
            ) {
                Ok((cyl, head)) => ok(json!([num(cyl), num(head)])),
                Err(e) => py_error(e.kind, e.message),
            }
        }
        other => Err(EvalError::BadInput(format!(
            "pressure_vessel has no function `{other}`"
        ))),
    }
}

/// `args.hull`: the seven floats of one hull.
fn hull_arg(args: &Value) -> Result<HullGeometry, EvalError> {
    let spec = args
        .get("hull")
        .ok_or_else(|| EvalError::BadInput("`args.hull` is missing".into()))?;
    Ok(HullGeometry::new(
        f64_arg(spec, "loa")?,
        f64_arg(spec, "beam")?,
        f64_arg(spec, "depth")?,
        f64_arg(spec, "bow_rake_length")?,
        f64_arg(spec, "bow_rake_rise")?,
        f64_arg(spec, "stern_rake_length")?,
        f64_arg(spec, "stern_rake_rise")?,
    ))
}

fn hull(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        // One hull, every public quantity at `args.draft`, then the plating areas, in the key
        // order of the Python function.
        "quantities" => {
            let h = hull_arg(args)?;
            let d = f64_arg(args, "draft")?;
            ok(json!({
                "parallel_length": num(h.parallel_length()),
                "displaced_volume": num(h.displaced_volume(d)),
                "waterplane_area": num(h.waterplane_area(d)),
                "waterplane_length": num(h.waterplane_length(d)),
                "waterplane_inertia_transverse": num(h.waterplane_inertia_transverse(d)),
                "waterplane_inertia_longitudinal": num(h.waterplane_inertia_longitudinal(d)),
                "KB": num(h.KB(d)),
                "LCB": num(h.LCB(d)),
                "BM_transverse": num(h.BM_transverse(d)),
                "BM_longitudinal": num(h.BM_longitudinal(d)),
                "bottom_area": num(h.bottom_area()),
                "side_area": num(h.side_area()),
                "deck_area": num(h.deck_area()),
                "bow_rake_area": num(h.bow_rake_area()),
                "stern_rake_area": num(h.stern_rake_area()),
                "total_hull_surface_area": num(h.total_hull_surface_area()),
            }))
        }
        other => Err(EvalError::BadInput(format!(
            "hull has no function `{other}`"
        ))),
    }
}

const BARGE_FIELDS: &[&str] = &[
    "loa",
    "beam",
    "depth",
    "max_draft",
    "vertical_clearance",
    "vessel_type",
    "bow_rake_length",
    "bow_rake_rise",
    "stern_rake_length",
    "stern_rake_rise",
    "bow_non_tank_pct",
    "stern_non_tank_pct",
    "bow_non_tank_length",
    "stern_non_tank_length",
    "wing_tank_width",
    "wing_tank_gap",
    "wing_tank_height_ratio",
    "weather_shield_thickness",
    "weather_shield_clearance",
    "dome_height",
    "side_clearance",
    "tank_gap_transverse",
    "tank_gap_longitudinal",
    "n_tanks_wide",
    "n_tanks_long",
    "tank_wall_thickness",
    "tank_head_type",
    "tank_material",
    "tank_material_density",
    "design_pressure",
    "weld_efficiency",
    "tank_structure_factor",
    "insulation_thickness",
    "insulation_density",
    "cargo_type",
    "cargo_density_override",
    "fill_ratio",
    "bottom_plate_thickness",
    "side_plate_thickness",
    "deck_plate_thickness",
    "structure_factor",
    "hull_steel_density",
    "steel_rate_per_cbm",
    "block_coefficient",
    "water_density",
    "underkeel_clearance",
    "outfitting_factor",
    "piping_factor",
    "stores_weight",
    "tank_support_height",
    "double_bottom_height",
    "aft_arrangement",
    "separation",
    "tug_length",
    "tug_beam",
    "notch_depth",
];

fn string_field(
    inputs: &serde_json::Map<String, Value>,
    name: &str,
) -> Result<Option<String>, EvalError> {
    match inputs.get(name) {
        None => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(_) => Err(EvalError::BadInput(format!(
            "`inputs.{name}` must be a string"
        ))),
    }
}

/// An `Optional[float]` field: absent keeps the dataclass default, an explicit null is `None`.
fn opt_f64_field(
    inputs: &serde_json::Map<String, Value>,
    name: &str,
    default: Option<f64>,
) -> Result<Option<f64>, EvalError> {
    match inputs.get(name) {
        None => Ok(default),
        Some(Value::Null) => Ok(None),
        Some(_) => f64_field(inputs, name),
    }
}

/// `BargeInputs` from plain data, as the Python bridge builds it. The caller runs
/// `post_init`, as Python's `__post_init__` runs after the constructor.
fn barge_inputs(args: &Value) -> Result<BargeInputs, EvalError> {
    let o = inputs_object(args, BARGE_FIELDS)?;
    let need = |name: &str| {
        f64_field(o, name)?
            .ok_or_else(|| EvalError::BadInput(format!("`inputs.{name}` is required")))
    };
    let mut i = BargeInputs::new(
        need("loa")?,
        need("beam")?,
        need("depth")?,
        need("max_draft")?,
        need("vertical_clearance")?,
    );
    macro_rules! set_f64 {
        ($($field:ident),*) => {$(
            if let Some(v) = f64_field(o, stringify!($field))? { i.$field = v; }
        )*};
    }
    macro_rules! set_opt_f64 {
        ($($field:ident),*) => {$(
            i.$field = opt_f64_field(o, stringify!($field), None)?;
        )*};
    }
    macro_rules! set_string {
        ($($field:ident),*) => {$(
            if let Some(v) = string_field(o, stringify!($field))? { i.$field = v; }
        )*};
    }
    set_string!(vessel_type, tank_head_type, tank_material, cargo_type);
    set_f64!(
        bow_rake_length,
        stern_rake_length,
        bow_non_tank_pct,
        stern_non_tank_pct,
        wing_tank_gap,
        wing_tank_height_ratio,
        weather_shield_thickness,
        weather_shield_clearance,
        dome_height,
        tank_gap_transverse,
        tank_gap_longitudinal,
        tank_material_density,
        design_pressure,
        weld_efficiency,
        tank_structure_factor,
        insulation_thickness,
        insulation_density,
        fill_ratio,
        bottom_plate_thickness,
        side_plate_thickness,
        deck_plate_thickness,
        structure_factor,
        hull_steel_density,
        steel_rate_per_cbm,
        block_coefficient,
        water_density,
        underkeel_clearance,
        outfitting_factor,
        piping_factor,
        stores_weight,
        tank_support_height,
        tug_length,
        tug_beam,
        notch_depth
    );
    set_opt_f64!(
        bow_rake_rise,
        stern_rake_rise,
        bow_non_tank_length,
        stern_non_tank_length,
        wing_tank_width,
        side_clearance,
        cargo_density_override
    );
    // These two default to a number, so an absent key and an explicit null differ.
    i.tank_wall_thickness = opt_f64_field(o, "tank_wall_thickness", i.tank_wall_thickness)?;
    i.double_bottom_height = opt_f64_field(o, "double_bottom_height", i.double_bottom_height)?;
    if let Some(v) = i64_field(o, "n_tanks_wide")? {
        i.n_tanks_wide = v;
    }
    if let Some(v) = i64_field(o, "n_tanks_long")? {
        i.n_tanks_long = v;
    }
    for pass_through in ["aft_arrangement", "separation"] {
        if !matches!(o.get(pass_through), None | Some(Value::Null)) {
            return Err(EvalError::BadInput(format!(
                "`inputs.{pass_through}` is a drawing-only pass-through; leave it null"
            )));
        }
    }
    Ok(i)
}

pub(crate) fn raised_py(e: &PyError) -> Result<Value, EvalError> {
    raised(e.kind, &e.message)
}

fn tank_json(t: &TankResult) -> Value {
    json!({
        "n_tanks_wide": t.n_tanks_wide,
        "n_tanks_long": t.n_tanks_long,
        "n_tanks_total": t.n_tanks_total,
        "outer_diameter": num(t.outer_diameter),
        "inner_diameter": num(t.inner_diameter),
        "overall_tank_length": num(t.overall_tank_length),
        "cylinder_length": num(t.cylinder_length),
        "head_length": num(t.head_length),
        "volume_per_tank": num(t.volume_per_tank),
        "volume_cylinder": num(t.volume_cylinder),
        "volume_heads": num(t.volume_heads),
        "total_tank_volume": num(t.total_tank_volume),
        "cargo_volume": num(t.cargo_volume),
        "tank_shell_weight": num(t.tank_shell_weight),
        "insulation_weight": num(t.insulation_weight),
        "tank_top_above_keel": num(t.tank_top_above_keel),
        "cylinder_wall_thickness": num(t.cylinder_wall_thickness),
        "head_wall_thickness": num(t.head_wall_thickness),
        "design_pressure": num(t.design_pressure),
        "tank_material": t.tank_material,
    })
}

fn weights_json(w: &WeightBreakdown) -> Value {
    json!({
        "hull_bottom_plating": num(w.hull_bottom_plating),
        "hull_side_plating": num(w.hull_side_plating),
        "hull_deck_plating": num(w.hull_deck_plating),
        "hull_rake_plating": num(w.hull_rake_plating),
        "hull_internal_structure": num(w.hull_internal_structure),
        "hull_steel_total": num(w.hull_steel_total),
        "tank_shells": num(w.tank_shells),
        "tank_internals_supports": num(w.tank_internals_supports),
        "tank_total": num(w.tank_total),
        "insulation": num(w.insulation),
        "wing_plate": num(w.wing_plate),
        "weather_shield": num(w.weather_shield),
        "piping_machinery": num(w.piping_machinery),
        "outfitting": num(w.outfitting),
        "stores": num(w.stores),
        "moulded_volume": num(w.moulded_volume),
        "moulded_depth": num(w.moulded_depth),
        "steel_top_down": num(w.steel_top_down),
        "steel_bottom_up": num(w.steel_bottom_up),
        "steel_delta": num(w.steel_delta),
        "lightship": num(w.lightship),
    })
}

fn tanks(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "calculate_tank_arrangement" => {
            let mut inputs = barge_inputs(args)?;
            if let Err(e) = inputs.post_init() {
                return raised_py(&e);
            }
            match ovcd_core::tanks::calculate_tank_arrangement(&inputs) {
                Ok(t) => ok(tank_json(&t)),
                Err(e) => raised_py(&e),
            }
        }
        "free_surface_moment_cylindrical" => {
            ok(num(ovcd_core::tanks::free_surface_moment_cylindrical(
                f64_arg(args, "inner_radius")?,
                f64_arg(args, "cylinder_length")?,
                f64_arg(args, "fill_ratio")?,
                f64_arg(args, "cargo_density")?,
            )))
        }
        other => Err(EvalError::BadInput(format!(
            "tanks has no function `{other}`"
        ))),
    }
}

/// Why the chain (inputs, hull, tanks, weights) stopped before the function asked for.
enum ChainStop {
    /// The case input is not usable.
    Eval(EvalError),
    /// Python would have raised: the answer is that error.
    Raised(PyError),
}

impl From<EvalError> for ChainStop {
    fn from(e: EvalError) -> Self {
        ChainStop::Eval(e)
    }
}

impl From<PyError> for ChainStop {
    fn from(e: PyError) -> Self {
        ChainStop::Raised(e)
    }
}

/// The chain: inputs, then the hull with the inputs' own rakes, then the tank
/// arrangement, then the weights. The tank count product is not carried (`n_tanks_total` of the
/// returned tanks is clamped), so a count Python answers for never raises here. A caller that
/// reads `n_tanks_total` must check the product itself.
fn barge_chain(
    args: &Value,
) -> Result<(BargeInputs, HullGeometry, TankResult, WeightBreakdown, i128), ChainStop> {
    let mut inputs = barge_inputs(args)?;
    inputs.post_init()?;
    let rake = |v: Option<f64>, name: &str| {
        v.ok_or_else(|| EvalError::BadInput(format!("`inputs.{name}` is unset")))
    };
    let hull = HullGeometry::new(
        inputs.loa,
        inputs.beam,
        inputs.depth,
        inputs.bow_rake_length,
        rake(inputs.bow_rake_rise, "bow_rake_rise")?,
        inputs.stern_rake_length,
        rake(inputs.stern_rake_rise, "stern_rake_rise")?,
    );
    let (tanks, n_total) = ovcd_core::tanks::calculate_tank_arrangement_wide(&inputs)?;
    let weights = ovcd_core::weights::calculate_weights(&inputs, &hull, &tanks)?;
    Ok((inputs, hull, tanks, weights, n_total))
}

fn chain_result(r: Result<Value, ChainStop>) -> Result<Value, EvalError> {
    match r {
        Ok(v) => Ok(v),
        Err(ChainStop::Eval(e)) => Err(e),
        Err(ChainStop::Raised(e)) => raised_py(&e),
    }
}

fn weights(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "calculate_weights" => {
            chain_result(barge_chain(args).map(|(_, _, _, w, _)| json!({ "ok": weights_json(&w) })))
        }
        other => Err(EvalError::BadInput(format!(
            "weights has no function `{other}`"
        ))),
    }
}

fn hydrostatics(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "find_equilibrium_draft" => {
            let hull = hull_arg(args)?;
            match ovcd_core::hydrostatics::find_equilibrium_draft(
                &hull,
                f64_arg(args, "total_weight")?,
                f64_arg(args, "water_density")?,
                f64_arg(args, "max_draft")?,
                f64_arg_or(args, "block_coefficient", 1.0)?,
                f64_arg_or(args, "tolerance", 0.001)?,
            ) {
                Ok(d) => ok(num(d)),
                Err(e) => raised_py(&e),
            }
        }
        "calculate_stability" => chain_result((|| {
            let (inputs, hull, mut tanks, weights, n_total) = barge_chain(args)?;
            // Python's chain holds the exact count; `n_tanks_total` is a 64-bit field here.
            tanks.n_tanks_total = i64::try_from(n_total).map_err(|_| PyError {
                kind: "OverflowError",
                message: ovcd_core::tanks::INT_OVERFLOW_MESSAGE.into(),
            })?;
            let s = ovcd_core::hydrostatics::calculate_stability(
                &hull,
                &inputs,
                &tanks,
                &weights,
                f64_arg(args, "cargo_mass")?,
                f64_arg(args, "cargo_density_t")?,
                f64_arg(args, "draft")?,
            )?;
            Ok(json!({ "ok": {
                "KB": num(s.KB),
                "BM_transverse": num(s.BM_transverse),
                "KG": num(s.KG),
                "GM_solid": num(s.GM_solid),
                "free_surface_correction": num(s.free_surface_correction),
                "GM_fluid": num(s.GM_fluid),
                "BM_longitudinal": num(s.BM_longitudinal),
            }}))
        })()),
        "calculate_trim" => chain_result((|| {
            let (inputs, hull, tanks, weights, _) = barge_chain(args)?;
            let t = ovcd_core::hydrostatics::calculate_trim(
                &hull,
                &inputs,
                &tanks,
                &weights,
                f64_arg(args, "cargo_mass")?,
                f64_arg(args, "draft")?,
            )?;
            Ok(json!({ "ok": {
                "LCG": num(t.LCG),
                "LCB": num(t.LCB),
                "trim": num(t.trim),
                "draft_aft": num(t.draft_aft),
                "draft_fwd": num(t.draft_fwd),
            }}))
        })()),
        other => Err(EvalError::BadInput(format!(
            "hydrostatics has no function `{other}`"
        ))),
    }
}

/// A float field; absent means Python's default.
/// A JSON object built key by key, in the order written. `json!` hits the macro recursion limit
/// on objects this wide.
macro_rules! wide_object {
    ({ $($key:literal : $value:expr),* $(,)? }) => {{
        let mut map = serde_json::Map::new();
        $( map.insert($key.to_string(), json!($value)); )*
        Value::Object(map)
    }};
}

fn barge_inputs_json(i: &BargeInputs) -> Value {
    wide_object!({
        "loa": num(i.loa),
        "beam": num(i.beam),
        "depth": num(i.depth),
        "max_draft": num(i.max_draft),
        "vertical_clearance": num(i.vertical_clearance),
        "vessel_type": i.vessel_type,
        "bow_rake_length": num(i.bow_rake_length),
        "bow_rake_rise": opt_num(i.bow_rake_rise),
        "stern_rake_length": num(i.stern_rake_length),
        "stern_rake_rise": opt_num(i.stern_rake_rise),
        "bow_non_tank_pct": num(i.bow_non_tank_pct),
        "stern_non_tank_pct": num(i.stern_non_tank_pct),
        "bow_non_tank_length": opt_num(i.bow_non_tank_length),
        "stern_non_tank_length": opt_num(i.stern_non_tank_length),
        "wing_tank_width": opt_num(i.wing_tank_width),
        "wing_tank_gap": num(i.wing_tank_gap),
        "wing_tank_height_ratio": num(i.wing_tank_height_ratio),
        "weather_shield_thickness": num(i.weather_shield_thickness),
        "weather_shield_clearance": num(i.weather_shield_clearance),
        "dome_height": num(i.dome_height),
        "side_clearance": opt_num(i.side_clearance),
        "tank_gap_transverse": num(i.tank_gap_transverse),
        "tank_gap_longitudinal": num(i.tank_gap_longitudinal),
        "n_tanks_wide": i.n_tanks_wide,
        "n_tanks_long": i.n_tanks_long,
        "tank_wall_thickness": opt_num(i.tank_wall_thickness),
        "tank_head_type": i.tank_head_type,
        "tank_material": i.tank_material,
        "tank_material_density": num(i.tank_material_density),
        "design_pressure": num(i.design_pressure),
        "weld_efficiency": num(i.weld_efficiency),
        "tank_structure_factor": num(i.tank_structure_factor),
        "insulation_thickness": num(i.insulation_thickness),
        "insulation_density": num(i.insulation_density),
        "cargo_type": i.cargo_type,
        "cargo_density_override": opt_num(i.cargo_density_override),
        "fill_ratio": num(i.fill_ratio),
        "bottom_plate_thickness": num(i.bottom_plate_thickness),
        "side_plate_thickness": num(i.side_plate_thickness),
        "deck_plate_thickness": num(i.deck_plate_thickness),
        "structure_factor": num(i.structure_factor),
        "hull_steel_density": num(i.hull_steel_density),
        "steel_rate_per_cbm": num(i.steel_rate_per_cbm),
        "block_coefficient": num(i.block_coefficient),
        "water_density": num(i.water_density),
        "underkeel_clearance": num(i.underkeel_clearance),
        "outfitting_factor": num(i.outfitting_factor),
        "piping_factor": num(i.piping_factor),
        "stores_weight": num(i.stores_weight),
        "tank_support_height": num(i.tank_support_height),
        "double_bottom_height": opt_num(i.double_bottom_height),
        // The drawing-only pass-throughs: the capacity chain never sets them.
        "aft_arrangement": i.aft_arrangement.as_ref().map(aft_json),
        "separation": i.separation.as_ref().map(gas_separation_json),
        "tug_length": num(i.tug_length),
        "tug_beam": num(i.tug_beam),
        "notch_depth": num(i.notch_depth),
    })
}

/// An `Optional[float]` result: `None` is JSON null.
pub(crate) fn opt_num(x: Option<f64>) -> Value {
    x.map_or(Value::Null, num)
}

const CAPACITY_FIELDS: &[&str] = &[
    "loa",
    "beam",
    "depth",
    "cargo_type",
    "block_coefficient",
    "water_density",
    "required_freeboard",
    "air_draft_limit_m",
    "ballast_capacity_t",
    "aft",
    "separation",
    "self_propelled",
    "pushed_stern_void_m",
    "bow_rake_length_m",
    "bulk_end_bulkhead_m",
    "vessel_type",
    "side_clearance_override_m",
    "n_tanks_wide",
    "n_tanks_long",
    "tank_wall_thickness",
    "fill_ratio",
    "dome_height",
    "tank_support_height",
    "hold_type",
    "lightship_k_t_per_m3",
    "wing_void_width",
    "double_bottom_height_bulk",
    "hopper_slope_h_per_v",
    "coaming_height",
    "n_holds",
    "hold_fill_ratio",
    "bow_void_fraction",
    "stern_void_fraction",
    "inner_bottom_t_mm",
    "hopper_plate_t_mm",
    "bulkhead_t_mm",
    "coaming_t_mm",
    "steel_density",
    "bulk_structure_factor",
    "bulk_steel_rate_per_cbm",
    "bulk_piping_factor",
    "bulk_outfitting_factor",
];

/// An `Optional[int]` field: absent keeps the default, an explicit null is `None`.
fn opt_i64_field(
    inputs: &serde_json::Map<String, Value>,
    name: &str,
    default: Option<i64>,
) -> Result<Option<i64>, EvalError> {
    match inputs.get(name) {
        None => Ok(default),
        Some(Value::Null) => Ok(None),
        Some(_) => i64_field(inputs, name),
    }
}

/// An optional nested input object (`aft`, `separation`): absent or null is `None`.
fn nested_inputs<T>(
    o: &serde_json::Map<String, Value>,
    name: &str,
    build: fn(&Value) -> Result<T, EvalError>,
) -> Result<Option<T>, EvalError> {
    match o.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => build(&json!({ "inputs": v })).map(Some),
    }
}

pub(crate) fn capacity_inputs(args: &Value) -> Result<BargeCapacityInputs, EvalError> {
    let o = inputs_object(args, CAPACITY_FIELDS)?;
    let need = |name: &str| {
        f64_field(o, name)?
            .ok_or_else(|| EvalError::BadInput(format!("`inputs.{name}` is required")))
    };
    let mut i = BargeCapacityInputs::new(need("loa")?, need("beam")?, need("depth")?);
    macro_rules! set_f64 {
        ($($field:ident),*) => {$(
            if let Some(v) = f64_field(o, stringify!($field))? { i.$field = v; }
        )*};
    }
    macro_rules! set_opt_f64 {
        ($($field:ident),*) => {$(
            i.$field = opt_f64_field(o, stringify!($field), None)?;
        )*};
    }
    macro_rules! set_string {
        ($($field:ident),*) => {$(
            if let Some(v) = string_field(o, stringify!($field))? { i.$field = v; }
        )*};
    }
    set_string!(cargo_type, vessel_type, hold_type);
    set_f64!(
        block_coefficient,
        water_density,
        required_freeboard,
        ballast_capacity_t,
        pushed_stern_void_m,
        bulk_end_bulkhead_m,
        fill_ratio,
        dome_height,
        tank_support_height,
        wing_void_width,
        double_bottom_height_bulk,
        hopper_slope_h_per_v,
        coaming_height,
        hold_fill_ratio,
        bow_void_fraction,
        stern_void_fraction,
        inner_bottom_t_mm,
        hopper_plate_t_mm,
        bulkhead_t_mm,
        coaming_t_mm,
        steel_density,
        bulk_structure_factor,
        bulk_steel_rate_per_cbm,
        bulk_piping_factor,
        bulk_outfitting_factor
    );
    set_opt_f64!(
        air_draft_limit_m,
        bow_rake_length_m,
        side_clearance_override_m,
        lightship_k_t_per_m3
    );
    // This one defaults to a number, so an absent key and an explicit null differ.
    i.tank_wall_thickness = opt_f64_field(o, "tank_wall_thickness", i.tank_wall_thickness)?;
    i.n_tanks_wide = opt_i64_field(o, "n_tanks_wide", None)?;
    if let Some(v) = i64_field(o, "n_tanks_long")? {
        i.n_tanks_long = v;
    }
    if let Some(v) = i64_field(o, "n_holds")? {
        i.n_holds = v;
    }
    i.self_propelled = bool_or(o, "self_propelled", i.self_propelled)?;
    i.aft = nested_inputs(o, "aft", aft_inputs)?;
    i.separation = nested_inputs(o, "separation", gas_separation_inputs)?;
    Ok(i)
}

pub(crate) fn capacity_json(r: &BargeCapacityResult) -> Value {
    wide_object!({
        "cargo_type": r.cargo_type,
        "phase": r.phase,
        "cargo_density_t_per_m3": num(r.cargo_density_t_per_m3),
        "cargo_capacity_t": num(r.cargo_capacity_t),
        "full_load_draft_m": num(r.full_load_draft_m),
        "air_draft_m": num(r.air_draft_m),
        "cargo_volume_m3": num(r.cargo_volume_m3),
        "lightship_t": num(r.lightship_t),
        "displacement_t": num(r.displacement_t),
        "deadweight_t": num(r.deadweight_t),
        "freeboard_m": num(r.freeboard_m),
        "air_draft_top_above_keel_m": num(r.air_draft_top_above_keel_m),
        "overloaded": r.overloaded,
        "tank_outer_diameter_m": opt_num(r.tank_outer_diameter_m),
        "n_tanks_wide": r.n_tanks_wide,
        "max_tank_od_m": opt_num(r.max_tank_od_m),
        "n_tanks_total": r.n_tanks_total,
        "side_clearance_m": opt_num(r.side_clearance_m),
        "bottom_clearance_m": opt_num(r.bottom_clearance_m),
        "hold_height_m": opt_num(r.hold_height_m),
        "hold_top_width_m": opt_num(r.hold_top_width_m),
        "hold_bottom_width_m": opt_num(r.hold_bottom_width_m),
        "hold_section_area_m2": opt_num(r.hold_section_area_m2),
        "gm_m": opt_num(r.gm_m),
        "bow_void_length_m": opt_num(r.bow_void_length_m),
        "stern_void_length_m": opt_num(r.stern_void_length_m),
        "cargo_zone_length_m": opt_num(r.cargo_zone_length_m),
        "bow_rake_length_m": opt_num(r.bow_rake_length_m),
        "aft_arrangement": r.aft_arrangement.as_ref().map(aft_json),
        "separation": r.separation.as_ref().map(gas_separation_json),
        "containment_inner_volume_m3": opt_num(r.containment_inner_volume_m3),
        "fill_ratio_applied": opt_num(r.fill_ratio_applied),
        "insulation_thickness_mm": opt_num(r.insulation_thickness_mm),
        "tank_wall_thickness_mm": opt_num(r.tank_wall_thickness_mm),
        "tank_inner_diameter_m": opt_num(r.tank_inner_diameter_m),
        "light_draft_m": opt_num(r.light_draft_m),
        "air_draft_light_m": opt_num(r.air_draft_light_m),
        "ballast_draft_m": opt_num(r.ballast_draft_m),
        "air_draft_ballast_m": opt_num(r.air_draft_ballast_m),
        "air_draft_navigating_light_m": opt_num(r.air_draft_navigating_light_m),
        "navigating_top_above_keel_m": opt_num(r.navigating_top_above_keel_m),
        "gas_basis": r.gas_basis.as_ref().map(|b| json!({
            "inputs": barge_inputs_json(&b.inputs),
            "tanks": tank_json(&b.tanks),
            "weights": weights_json(&b.weights),
        })),
        "checks": checks_json(&r.checks),
        "warnings": r.warnings,
    })
}

fn barge_capacity(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "compute" => {
            let inputs = capacity_inputs(args)?;
            // Python builds nested dataclasses in JSON key order, so with both
            // nested inputs bad the first key's message wins. `compute` checks aft first;
            // when `separation` comes first in the JSON, check it here before `compute` runs.
            if let Some(obj) = args.get("inputs").and_then(Value::as_object) {
                let pos = |name: &str| obj.keys().position(|k| k == name);
                if let (Some(sep), Some(_)) = (&inputs.separation, &inputs.aft)
                    && pos("separation") < pos("aft")
                    && let Err(message) = sep.validate()
                {
                    return raised("ValueError", &message);
                }
            }
            match ovcd_core::barge_capacity::compute(&inputs) {
                Ok(r) => ok(capacity_json(&r)),
                Err(e) => raised_py(&e),
            }
        }
        "tanks_wide_for_beam" => {
            match ovcd_core::barge_capacity::tanks_wide_for_beam(
                f64_arg(args, "beam_m")?,
                f64_arg(args, "side_clearance_m")?,
                f64_arg_or(args, "tank_gap_transverse_m", 0.0)?,
                f64_arg_or(args, "max_od_m", ovcd_core::barge_capacity::MAX_TANK_OD_M)?,
            ) {
                Ok(n) => ok(json!(n)),
                Err(e) => raised_py(&e),
            }
        }
        other => Err(EvalError::BadInput(format!(
            "barge_capacity has no function `{other}`"
        ))),
    }
}

fn field_or(obj: &Value, name: &str, default: Option<f64>) -> Result<f64, EvalError> {
    match obj.get(name) {
        Some(v) => v
            .as_f64()
            .ok_or_else(|| EvalError::BadInput(format!("`{name}` must be a number"))),
        None => default.ok_or_else(|| EvalError::BadInput(format!("`{name}` is required"))),
    }
}

fn squat_inputs(args: &Value) -> Result<core_squat::SquatInputs, EvalError> {
    let o = args
        .get("inputs")
        .ok_or_else(|| EvalError::BadInput("`args.inputs` is required".into()))?;
    let base = core_squat::SquatInputs::new(
        field_or(o, "speed_through_water_kn", None)?,
        field_or(o, "draft_m", None)?,
        field_or(o, "beam_m", None)?,
        field_or(o, "lpp_m", None)?,
    );
    Ok(core_squat::SquatInputs {
        block_coefficient: field_or(o, "block_coefficient", Some(base.block_coefficient))?,
        water_depth_m: field_or(o, "water_depth_m", Some(base.water_depth_m))?,
        channel_area_m2: field_or(o, "channel_area_m2", Some(base.channel_area_m2))?,
        displacement_m3: field_or(o, "displacement_m3", Some(base.displacement_m3))?,
        safety_margin_m: field_or(o, "safety_margin_m", Some(base.safety_margin_m))?,
        ..base
    })
}

fn squat(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "compute" => match core_squat::compute(&squat_inputs(args)?) {
            Ok(r) => ok(json!({
                "method": r.method,
                "squat_m": num(r.squat_m),
                "squat_barrass_m": num(r.squat_barrass_m),
                "squat_icorels_m": r.squat_icorels_m.map_or(Value::Null, num),
                "blockage_ratio": num(r.blockage_ratio),
                "froude_depth": num(r.froude_depth),
                "static_ukc_m": num(r.static_ukc_m),
                "dynamic_ukc_m": num(r.dynamic_ukc_m),
                "grounds": r.grounds,
                "checks": checks_json(&r.checks),
                "warnings": r.warnings,
            })),
            Err(message) => value_error(message),
        },
        "max_speed_for_ukc" => {
            let result = core_squat::max_speed_for_ukc(
                f64_arg(args, "draft_m")?,
                f64_arg(args, "beam_m")?,
                f64_arg(args, "lpp_m")?,
                f64_arg(args, "water_depth_m")?,
                f64_arg(args, "channel_area_m2")?,
                f64_arg_or(args, "block_coefficient", 0.85)?,
                f64_arg_or(args, "safety_margin_m", core_squat::DEFAULT_SAFETY_MARGIN_M)?,
                f64_arg_or(args, "v_hi_kn", 20.0)?,
                f64_arg_or(args, "tol_kn", 0.01)?,
            );
            match result {
                Ok(v) => ok(num(v)),
                Err(message) => value_error(message),
            }
        }
        other => Err(EvalError::BadInput(format!(
            "squat has no function `{other}`"
        ))),
    }
}

fn wave_inputs(args: &Value) -> Result<core_waves::WaveResistanceInputs, EvalError> {
    let o = args
        .get("inputs")
        .ok_or_else(|| EvalError::BadInput("`args.inputs` is required".into()))?;
    let base = core_waves::WaveResistanceInputs::new(
        field_or(o, "significant_wave_height_m", None)?,
        field_or(o, "beam_waterline_m", None)?,
        field_or(o, "lpp_m", None)?,
    );
    Ok(core_waves::WaveResistanceInputs {
        bow_waterline_length_m: field_or(
            o,
            "bow_waterline_length_m",
            Some(base.bow_waterline_length_m),
        )?,
        heading_deg_off_bow: field_or(o, "heading_deg_off_bow", Some(base.heading_deg_off_bow))?,
        peak_wave_period_s: field_or(o, "peak_wave_period_s", Some(base.peak_wave_period_s))?,
        water_density_t_m3: field_or(o, "water_density_t_m3", Some(base.water_density_t_m3))?,
        speed_kn: field_or(o, "speed_kn", Some(base.speed_kn))?,
        ..base
    })
}

fn waves(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "compute" => match core_waves::compute(&wave_inputs(args)?) {
            Ok(r) => ok(json!({
                "added_resistance_kn": num(r.added_resistance_kn),
                "added_resistance_head_seas_kn": num(r.added_resistance_head_seas_kn),
                "heading_factor": num(r.heading_factor),
                "bow_waterline_length_m": num(r.bow_waterline_length_m),
                "wavelength_m": num(r.wavelength_m),
                "lambda_over_lpp": num(r.lambda_over_lpp),
                "added_power_kw": num(r.added_power_kw),
                "checks": checks_json(&r.checks),
                "warnings": r.warnings,
            })),
            Err(message) => value_error(message),
        },
        other => Err(EvalError::BadInput(format!(
            "waves has no function `{other}`"
        ))),
    }
}

/// Reject any key of an input object that is not a field, as Python's `build` does.
fn fields_object<'a>(
    value: &'a Value,
    fields: &[&str],
    what: &str,
) -> Result<&'a serde_json::Map<String, Value>, EvalError> {
    let object = value
        .as_object()
        .ok_or_else(|| EvalError::BadInput(format!("`{what}` must be an object")))?;
    for key in object.keys() {
        if !fields.contains(&key.as_str()) {
            return Err(EvalError::BadInput(format!(
                "unknown field `{key}` in `{what}`"
            )));
        }
    }
    Ok(object)
}

fn f64_or(
    object: &serde_json::Map<String, Value>,
    name: &str,
    default: f64,
) -> Result<f64, EvalError> {
    Ok(f64_field(object, name)?.unwrap_or(default))
}

fn bool_or(
    object: &serde_json::Map<String, Value>,
    name: &str,
    default: bool,
) -> Result<bool, EvalError> {
    match object.get(name) {
        None => Ok(default),
        Some(v) => v
            .as_bool()
            .ok_or_else(|| EvalError::BadInput(format!("`{name}` must be true or false"))),
    }
}

const TUG_FIELDS: &[&str] = &[
    "engine_power_kw",
    "propulsive_efficiency",
    "sfc_g_per_kwh",
    "fuel_density_kg_per_litre",
];

const ENVIRONMENT_FIELDS: &[&str] = &[
    "name",
    "water_depth",
    "channel_width_bottom",
    "channel_width_surface",
    "bank_slope",
    "current_speed_knots",
    "water_density",
    "is_deep_water",
    "significant_wave_height_m",
    "wave_heading_deg_off_bow",
    "peak_wave_period_s",
    "squat_safety_margin_m",
    "apply_squat_limit",
];

const RESISTANCE_FIELDS: &[&str] = &[
    "loa",
    "lwl",
    "beam",
    "draft",
    "block_coefficient",
    "wetted_surface_area",
    "displacement_tonnes",
    "tug",
    "environments",
    "speed_min_knots",
    "speed_max_knots",
    "speed_step_knots",
];

fn tug_inputs(value: &Value) -> Result<core_resistance::TugInputs, EvalError> {
    let o = fields_object(value, TUG_FIELDS, "tug")?;
    let d = core_resistance::TugInputs::default();
    Ok(core_resistance::TugInputs {
        engine_power_kw: f64_or(o, "engine_power_kw", d.engine_power_kw)?,
        propulsive_efficiency: f64_or(o, "propulsive_efficiency", d.propulsive_efficiency)?,
        sfc_g_per_kwh: f64_or(o, "sfc_g_per_kwh", d.sfc_g_per_kwh)?,
        fuel_density_kg_per_litre: f64_or(
            o,
            "fuel_density_kg_per_litre",
            d.fuel_density_kg_per_litre,
        )?,
    })
}

fn environment_inputs(value: &Value) -> Result<core_resistance::EnvironmentInputs, EvalError> {
    let o = fields_object(value, ENVIRONMENT_FIELDS, "environments[]")?;
    let d = core_resistance::EnvironmentInputs::default();
    let name = match o.get("name") {
        None => d.name.clone(),
        Some(v) => v
            .as_str()
            .ok_or_else(|| EvalError::BadInput("`name` must be text".into()))?
            .to_string(),
    };
    Ok(core_resistance::EnvironmentInputs {
        name,
        water_depth: f64_or(o, "water_depth", d.water_depth)?,
        channel_width_bottom: f64_or(o, "channel_width_bottom", d.channel_width_bottom)?,
        channel_width_surface: f64_or(o, "channel_width_surface", d.channel_width_surface)?,
        bank_slope: f64_or(o, "bank_slope", d.bank_slope)?,
        current_speed_knots: f64_or(o, "current_speed_knots", d.current_speed_knots)?,
        water_density: f64_or(o, "water_density", d.water_density)?,
        is_deep_water: bool_or(o, "is_deep_water", d.is_deep_water)?,
        significant_wave_height_m: f64_or(
            o,
            "significant_wave_height_m",
            d.significant_wave_height_m,
        )?,
        wave_heading_deg_off_bow: f64_or(
            o,
            "wave_heading_deg_off_bow",
            d.wave_heading_deg_off_bow,
        )?,
        peak_wave_period_s: f64_or(o, "peak_wave_period_s", d.peak_wave_period_s)?,
        squat_safety_margin_m: f64_or(o, "squat_safety_margin_m", d.squat_safety_margin_m)?,
        apply_squat_limit: bool_or(o, "apply_squat_limit", d.apply_squat_limit)?,
    })
}

fn resistance_inputs(args: &Value) -> Result<core_resistance::ResistanceInputs, EvalError> {
    let inputs = args
        .get("inputs")
        .ok_or_else(|| EvalError::BadInput("`args.inputs` is required".into()))?;
    let o = fields_object(inputs, RESISTANCE_FIELDS, "inputs")?;
    let d = core_resistance::ResistanceInputs::default();
    let tug = match o.get("tug") {
        None => d.tug.clone(),
        Some(v) => tug_inputs(v)?,
    };
    let environments = match o.get("environments") {
        None => d.environments.clone(),
        Some(v) => v
            .as_array()
            .ok_or_else(|| EvalError::BadInput("`environments` must be a list".into()))?
            .iter()
            .map(environment_inputs)
            .collect::<Result<_, _>>()?,
    };
    Ok(core_resistance::ResistanceInputs {
        loa: f64_or(o, "loa", d.loa)?,
        lwl: f64_or(o, "lwl", d.lwl)?,
        beam: f64_or(o, "beam", d.beam)?,
        draft: f64_or(o, "draft", d.draft)?,
        block_coefficient: f64_or(o, "block_coefficient", d.block_coefficient)?,
        wetted_surface_area: f64_or(o, "wetted_surface_area", d.wetted_surface_area)?,
        displacement_tonnes: f64_or(o, "displacement_tonnes", d.displacement_tonnes)?,
        tug,
        environments,
        speed_min_knots: f64_or(o, "speed_min_knots", d.speed_min_knots)?,
        speed_max_knots: f64_or(o, "speed_max_knots", d.speed_max_knots)?,
        speed_step_knots: f64_or(o, "speed_step_knots", d.speed_step_knots)?,
    })
}

fn environment_inputs_json(e: &core_resistance::EnvironmentInputs) -> Value {
    json!({
        "name": e.name,
        "water_depth": num(e.water_depth),
        "channel_width_bottom": num(e.channel_width_bottom),
        "channel_width_surface": num(e.channel_width_surface),
        "bank_slope": num(e.bank_slope),
        "current_speed_knots": num(e.current_speed_knots),
        "water_density": num(e.water_density),
        "is_deep_water": e.is_deep_water,
        "significant_wave_height_m": num(e.significant_wave_height_m),
        "wave_heading_deg_off_bow": num(e.wave_heading_deg_off_bow),
        "peak_wave_period_s": num(e.peak_wave_period_s),
        "squat_safety_margin_m": num(e.squat_safety_margin_m),
        "apply_squat_limit": e.apply_squat_limit,
    })
}

fn resistance_inputs_json(i: &core_resistance::ResistanceInputs) -> Value {
    json!({
        "loa": num(i.loa),
        "lwl": num(i.lwl),
        "beam": num(i.beam),
        "draft": num(i.draft),
        "block_coefficient": num(i.block_coefficient),
        "wetted_surface_area": num(i.wetted_surface_area),
        "displacement_tonnes": num(i.displacement_tonnes),
        "tug": {
            "engine_power_kw": num(i.tug.engine_power_kw),
            "propulsive_efficiency": num(i.tug.propulsive_efficiency),
            "sfc_g_per_kwh": num(i.tug.sfc_g_per_kwh),
            "fuel_density_kg_per_litre": num(i.tug.fuel_density_kg_per_litre),
        },
        "environments": i.environments.iter().map(environment_inputs_json).collect::<Vec<_>>(),
        "speed_min_knots": num(i.speed_min_knots),
        "speed_max_knots": num(i.speed_max_knots),
        "speed_step_knots": num(i.speed_step_knots),
    })
}

fn speed_point_json(p: &core_resistance::SpeedPoint) -> Value {
    json!({
        "speed_knots": num(p.speed_knots),
        "speed_through_water_knots": num(p.speed_through_water_knots),
        "froude_number": num(p.froude_number),
        "froude_depth": num(p.froude_depth),
        "resistance_frictional_kn": num(p.resistance_frictional_kn),
        "resistance_residuary_kn": num(p.resistance_residuary_kn),
        "resistance_shallow_factor": num(p.resistance_shallow_factor),
        "resistance_total_kn": num(p.resistance_total_kn),
        "effective_power_kw": num(p.effective_power_kw),
        "delivered_power_kw": num(p.delivered_power_kw),
        "fuel_litres_per_hour": num(p.fuel_litres_per_hour),
        "fuel_litres_per_nm": num(p.fuel_litres_per_nm),
        "power_available": p.power_available,
        "resistance_waves_kn": num(p.resistance_waves_kn),
        "squat_m": num(p.squat_m),
        "dynamic_ukc_m": num(p.dynamic_ukc_m),
        "squat_limited": p.squat_limited,
    })
}

fn environment_result_json(e: &core_resistance::EnvironmentResult) -> Value {
    json!({
        "name": e.name,
        "limiting_speed_knots": e.limiting_speed_knots.map_or(Value::Null, num),
        "service_speed_knots": num(e.service_speed_knots),
        "blockage_ratio": num(e.blockage_ratio),
        "speed_curve": e.speed_curve.iter().map(speed_point_json).collect::<Vec<_>>(),
        "resistance_at_service_kn": num(e.resistance_at_service_kn),
        "power_at_service_kw": num(e.power_at_service_kw),
        "fuel_at_service_lph": num(e.fuel_at_service_lph),
        "fuel_at_service_lpnm": num(e.fuel_at_service_lpnm),
        "squat_limited_speed_knots": e.squat_limited_speed_knots.map_or(Value::Null, num),
        "service_speed_governed_by": e.service_speed_governed_by,
    })
}

fn resistance(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "compute" => match core_resistance::compute(&resistance_inputs(args)?) {
            Ok(r) => ok(json!({
                "inputs": resistance_inputs_json(&r.inputs),
                "wetted_surface_area": num(r.wetted_surface_area),
                "environments": r.environments.iter().map(environment_result_json).collect::<Vec<_>>(),
                "checks": checks_json(&r.checks),
                "warnings": r.warnings,
            })),
            Err(e) => py_exception(&e),
        },
        other => Err(EvalError::BadInput(format!(
            "resistance has no function `{other}`"
        ))),
    }
}

const ENERGY_FIELDS: &[&str] = &[
    "beam",
    "loa",
    "lwl",
    "draft",
    "block_coefficient",
    "channel_depth_m",
    "channel_bottom_width_m",
    "channel_top_width_m",
    "current_knots",
    "fresh_water",
    "also_open_sea",
    "propulsive_coefficient",
    "drivetrain_efficiency",
    "target_service_speed_knots",
    "service_speed_safety_fraction",
    "power_margin",
    "one_way_km",
    "trips_per_year",
    "speed_min_knots",
    "speed_max_knots",
    "speed_step_knots",
];

fn energy_inputs(args: &Value) -> Result<core_energy::EnergyInputs, EvalError> {
    let inputs = args
        .get("inputs")
        .ok_or_else(|| EvalError::BadInput("`args.inputs` is required".into()))?;
    let o = fields_object(inputs, ENERGY_FIELDS, "inputs")?;
    let d = core_energy::EnergyInputs::default();
    Ok(core_energy::EnergyInputs {
        beam: f64_or(o, "beam", d.beam)?,
        loa: f64_or(o, "loa", d.loa)?,
        lwl: f64_or(o, "lwl", d.lwl)?,
        draft: f64_or(o, "draft", d.draft)?,
        block_coefficient: f64_or(o, "block_coefficient", d.block_coefficient)?,
        channel_depth_m: f64_or(o, "channel_depth_m", d.channel_depth_m)?,
        channel_bottom_width_m: f64_or(o, "channel_bottom_width_m", d.channel_bottom_width_m)?,
        channel_top_width_m: f64_or(o, "channel_top_width_m", d.channel_top_width_m)?,
        current_knots: f64_or(o, "current_knots", d.current_knots)?,
        fresh_water: bool_or(o, "fresh_water", d.fresh_water)?,
        also_open_sea: bool_or(o, "also_open_sea", d.also_open_sea)?,
        propulsive_coefficient: f64_or(o, "propulsive_coefficient", d.propulsive_coefficient)?,
        drivetrain_efficiency: f64_or(o, "drivetrain_efficiency", d.drivetrain_efficiency)?,
        target_service_speed_knots: f64_or(
            o,
            "target_service_speed_knots",
            d.target_service_speed_knots,
        )?,
        service_speed_safety_fraction: f64_or(
            o,
            "service_speed_safety_fraction",
            d.service_speed_safety_fraction,
        )?,
        power_margin: f64_or(o, "power_margin", d.power_margin)?,
        one_way_km: f64_or(o, "one_way_km", d.one_way_km)?,
        trips_per_year: f64_or(o, "trips_per_year", d.trips_per_year)?,
        speed_min_knots: f64_or(o, "speed_min_knots", d.speed_min_knots)?,
        speed_max_knots: f64_or(o, "speed_max_knots", d.speed_max_knots)?,
        speed_step_knots: f64_or(o, "speed_step_knots", d.speed_step_knots)?,
    })
}

fn energy_inputs_json(i: &core_energy::EnergyInputs) -> Value {
    json!({
        "beam": num(i.beam),
        "loa": num(i.loa),
        "lwl": num(i.lwl),
        "draft": num(i.draft),
        "block_coefficient": num(i.block_coefficient),
        "channel_depth_m": num(i.channel_depth_m),
        "channel_bottom_width_m": num(i.channel_bottom_width_m),
        "channel_top_width_m": num(i.channel_top_width_m),
        "current_knots": num(i.current_knots),
        "fresh_water": i.fresh_water,
        "also_open_sea": i.also_open_sea,
        "propulsive_coefficient": num(i.propulsive_coefficient),
        "drivetrain_efficiency": num(i.drivetrain_efficiency),
        "target_service_speed_knots": num(i.target_service_speed_knots),
        "service_speed_safety_fraction": num(i.service_speed_safety_fraction),
        "power_margin": num(i.power_margin),
        "one_way_km": num(i.one_way_km),
        "trips_per_year": num(i.trips_per_year),
        "speed_min_knots": num(i.speed_min_knots),
        "speed_max_knots": num(i.speed_max_knots),
        "speed_step_knots": num(i.speed_step_knots),
    })
}

fn energy_point_json(p: &core_energy::EnergyPoint) -> Value {
    json!({
        "speed_knots": num(p.speed_knots),
        "speed_kmh": num(p.speed_kmh),
        "froude_depth": num(p.froude_depth),
        "resistance_kn": num(p.resistance_kn),
        "shaft_power_kw": num(p.shaft_power_kw),
        "battery_power_kw": num(p.battery_power_kw),
        "energy_kwh_per_km": num(p.energy_kwh_per_km),
    })
}

fn energy_environment_json(e: &core_energy::EnergyEnvironment) -> Value {
    json!({
        "name": e.name,
        "limiting_speed_knots": e.limiting_speed_knots.map_or(Value::Null, num),
        "service_speed_knots": num(e.service_speed_knots),
        "blockage_ratio": num(e.blockage_ratio),
        "curve": e.curve.iter().map(energy_point_json).collect::<Vec<_>>(),
        "resistance_kn": num(e.resistance_kn),
        "shaft_power_kw": num(e.shaft_power_kw),
        "battery_power_kw": num(e.battery_power_kw),
        "energy_kwh_per_km": num(e.energy_kwh_per_km),
        "energy_per_trip_kwh": num(e.energy_per_trip_kwh),
        "energy_per_year_mwh": num(e.energy_per_year_mwh),
    })
}

fn electric_energy(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "compute" => match core_energy::compute(&energy_inputs(args)?) {
            Ok(r) => ok(json!({
                "inputs": energy_inputs_json(&r.inputs),
                "installed_power_kw": num(r.installed_power_kw),
                "environments": r.environments.iter().map(energy_environment_json).collect::<Vec<_>>(),
                "checks": checks_json(&r.checks),
            })),
            Err(e) => py_exception(&e),
        },
        other => Err(EvalError::BadInput(format!(
            "electric_energy has no function `{other}`"
        ))),
    }
}

/// A list of numbers, as given: the sweep echoes them back as Python keeps them (`60` stays an
/// integer, `12.0` stays a float). Each must be exact as a double.
fn number_list(args: &Value, name: &str) -> Result<(Vec<Value>, Vec<f64>), EvalError> {
    let items = args
        .get(name)
        .and_then(Value::as_array)
        .ok_or_else(|| EvalError::BadInput(format!("`args.{name}` must be a list of numbers")))?;
    let mut values = Vec::with_capacity(items.len());
    let mut floats = Vec::with_capacity(items.len());
    for item in items {
        floats.push(exact_number(item, &format!("args.{name}[]"))?);
        values.push(item.clone());
    }
    Ok((values, floats))
}

fn grid_cell_json(c: &core_grid::GridCell) -> Value {
    json!({
        "barge_length_m": num(c.barge_length_m),
        "beam_m": num(c.beam_m),
        "n_tanks_long": c.n_tanks_long,
        "feasible": c.feasible,
        "cargo_t": opt_num(c.cargo_t),
        "tank_outer_diameter_m": opt_num(c.tank_outer_diameter_m),
        "tank_length_m": opt_num(c.tank_length_m),
        "slenderness": opt_num(c.slenderness),
        "full_load_draft_m": opt_num(c.full_load_draft_m),
        "air_draft_m": opt_num(c.air_draft_m),
        "reason": c.reason,
    })
}

fn design_grid(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "n_tanks_for_length" => {
            let length = exact_number(
                args.get("barge_length_m").unwrap_or(&Value::Null),
                "args.barge_length_m",
            )?;
            let max_len = match args.get("max_tank_length_m") {
                None => core_grid::MAX_TANK_LENGTH_M,
                Some(v) => exact_number(v, "args.max_tank_length_m")?,
            };
            match core_grid::n_tanks_for_length(length, max_len) {
                Ok(n) => ok(json!(n)),
                Err(e) => raised_py(&e),
            }
        }
        "sweep" => {
            let (length_values, lengths) = number_list(args, "barge_lengths_m")?;
            let (beam_values, beams) = number_list(args, "beams_m")?;
            let (depth_value, depth) = match args.get("depth_m") {
                None => (json!(6.0), 6.0),
                Some(v) => (v.clone(), exact_number(v, "args.depth_m")?),
            };
            let cargo_type = match args.get("cargo_type") {
                None => "LPG_propane",
                Some(Value::String(s)) => s.as_str(),
                Some(_) => {
                    return Err(EvalError::BadInput(
                        "`args.cargo_type` must be a string".into(),
                    ));
                }
            };
            match core_grid::sweep(&lengths, &beams, depth, cargo_type) {
                Ok(g) => ok(wide_object!({
                    "barge_lengths_m": length_values,
                    "beams_m": beam_values,
                    "depth_m": depth_value,
                    "cargo_type": g.cargo_type,
                    "clearance_basis": g.clearance_basis,
                    "max_tank_length_m": num(g.max_tank_length_m),
                    "cells": g.cells.iter().map(grid_cell_json).collect::<Vec<_>>(),
                })),
                Err(e) => raised_py(&e),
            }
        }
        other => Err(EvalError::BadInput(format!(
            "design_grid has no function `{other}`"
        ))),
    }
}

fn ga_inputs_json(i: &core_ga::GaInputs) -> Value {
    let mut value = barge_inputs_json(&i.inputs);
    // The drawing carries the capacity result's `Optional[int]` as it is.
    value["n_tanks_wide"] = i.n_tanks_wide.map_or(Value::Null, |n| json!(n));
    value
}

fn ga_results_json(r: &core_ga::GaResults) -> Value {
    wide_object!({
        "inputs": ga_inputs_json(&r.inputs),
        "tanks": r.tanks.as_ref().map(tank_json),
        "cargo_type": r.cargo_type,
        "cargo_density": num(r.cargo_density),
        "cargo_mass": num(r.cargo_mass),
        // The drawing never carries these three (Python passes `None`).
        "weights": r.weights.as_ref().map(weights_json),
        "displacement": num(r.displacement),
        "deadweight": num(r.deadweight),
        "draft": num(r.draft),
        "freeboard": num(r.freeboard),
        "min_freeboard_ok": r.min_freeboard_ok,
        "air_draft": num(r.air_draft),
        "air_draft_ok": r.air_draft_ok,
        "stability": Value::Null,
        "stability_ok": r.stability_ok,
        "trim": num(r.trim),
        "total_depth_required": num(r.total_depth_required),
        "air_draft_unladen": num(r.air_draft_unladen),
        "light_draft": num(r.light_draft),
        "air_draft_navigating_unladen": num(r.air_draft_navigating_unladen),
        "protective_location": Value::Null,
        "binding_constraints": r.binding_constraints,
        "warnings": r.warnings,
    })
}

/// `to_ga`'s keyword arguments: `laden_draft_m` (a number or null) and `motor_vessel` (a bool).
fn ga_kwargs(args: &Value) -> Result<(Option<f64>, bool), EvalError> {
    let mut laden = None;
    let mut motor_vessel = true;
    match args.get("kwargs") {
        None => {}
        Some(Value::Object(kwargs)) => {
            for (key, value) in kwargs {
                match (key.as_str(), value) {
                    ("laden_draft_m", Value::Null) => laden = None,
                    ("laden_draft_m", v) => {
                        laden = Some(exact_number(v, "args.kwargs.laden_draft_m")?);
                    }
                    ("motor_vessel", Value::Bool(b)) => motor_vessel = *b,
                    ("motor_vessel", _) => {
                        return Err(EvalError::BadInput(
                            "`args.kwargs.motor_vessel` must be a bool".into(),
                        ));
                    }
                    (other, _) => {
                        return Err(EvalError::BadInput(format!(
                            "to_ga has no keyword argument `{other}`"
                        )));
                    }
                }
            }
        }
        Some(_) => {
            return Err(EvalError::BadInput(
                "`args.kwargs` must be an object".into(),
            ));
        }
    }
    Ok((laden, motor_vessel))
}

fn ga_adapter(function: &str, args: &Value) -> Result<Value, EvalError> {
    if !matches!(function, "to_ga" | "is_gas") {
        return Err(EvalError::BadInput(format!(
            "ga_adapter has no function `{function}`"
        )));
    }
    let inputs = capacity_inputs(args)?;
    // Python runs `compute` first, for both functions.
    let result = match ovcd_core::barge_capacity::compute(&inputs) {
        Ok(r) => r,
        Err(e) => return raised_py(&e),
    };
    if function == "is_gas" {
        return ok(json!(core_ga::is_gas(&result)));
    }
    let (laden, motor_vessel) = ga_kwargs(args)?;
    match core_ga::to_ga(&inputs, &result, laden, motor_vessel) {
        Ok((gi, gr)) => ok(json!([ga_inputs_json(&gi), ga_results_json(&gr)])),
        Err(e) => raised_py(&e),
    }
}
