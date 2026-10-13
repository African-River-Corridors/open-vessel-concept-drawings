//! `eval(entry_point, json) -> json`: the parity seam. One dispatch over every parity entry
//! point. Input and output use the case and Reference Result shapes in `parity/README.md`.

use ovcd_core::aft_arrangement::{AftArrangementInputs, AftArrangementResult};
use ovcd_core::cargo::CargoProperties;
use ovcd_core::checks::Check;
use ovcd_core::gas_separation::{GasSeparationInputs, GasSeparationResult};
use serde_json::{Value, json};

/// Why `eval` could not produce a result. A Python `ValueError` is not an `EvalError`: it is a
/// normal result, `{"error": {"type": "ValueError", "message": ...}}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    /// No such parity entry point.
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

/// Every parity entry point in spec 0001.
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

/// Run one parity case through the Rust code.
pub fn eval(entry_point: &str, input: &Value) -> Result<Value, EvalError> {
    let function = input
        .get("fn")
        .and_then(Value::as_str)
        .ok_or_else(|| EvalError::BadInput("case input needs a string `fn`".into()))?;
    let args = input.get("args").unwrap_or(&Value::Null);
    match entry_point {
        "cargo" => cargo(function, args),
        "aft_arrangement" => aft_arrangement(function, args),
        "gas_separation" => gas_separation(function, args),
        ep if ENTRY_POINTS.contains(&ep) => Err(EvalError::NotPorted(ep.into())),
        ep => Err(EvalError::UnknownEntryPoint(ep.into())),
    }
}

fn ok(value: Value) -> Result<Value, EvalError> {
    Ok(json!({ "ok": value }))
}

/// A Python `ValueError`, as the frozen Python driver records it.
fn value_error(message: String) -> Result<Value, EvalError> {
    Ok(json!({ "error": { "type": "ValueError", "message": message } }))
}

/// A Python exception of any recorded type.
fn py_error(e: &ovcd_core::aft_arrangement::PyException) -> Result<Value, EvalError> {
    Ok(json!({ "error": { "type": e.type_name(), "message": e.message() } }))
}

/// A float as the parity files write it: NaN and infinity are the tagged strings
/// `"float:nan"`, `"float:inf"`, `"float:-inf"`. `json!` alone would write them as `null`.
fn num(x: f64) -> Value {
    if x.is_nan() {
        Value::String("float:nan".into())
    } else if x.is_infinite() {
        Value::String(if x > 0.0 { "float:inf" } else { "float:-inf" }.into())
    } else {
        json!(x)
    }
}

fn nums(xs: &[f64]) -> Value {
    Value::Array(xs.iter().map(|x| num(*x)).collect())
}

fn str_arg<'a>(args: &'a Value, name: &str) -> Result<&'a str, EvalError> {
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
            "cargo has no parity function `{other}`"
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

fn checks_json(checks: &[Check]) -> Value {
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
            Err(e) => py_error(&e),
        },
        other => Err(EvalError::BadInput(format!(
            "aft_arrangement has no parity function `{other}`"
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
            "gas_separation has no parity function `{other}`"
        ))),
    }
}
