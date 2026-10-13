//! `eval(entry_point, json) -> json`: the parity seam. One dispatch over every parity entry
//! point. Input and output use the case and Reference Result shapes in `parity/README.md`.

use ovcd_core::cargo::CargoProperties;
use ovcd_core::checks::Check;
use ovcd_core::{squat as core_squat, waves as core_waves};
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
        "squat" => squat(function, args),
        "waves" => waves(function, args),
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

/// A float as the parity files write it: NaN and infinity are tagged strings.
fn float_json(x: f64) -> Value {
    if x.is_nan() {
        json!("float:nan")
    } else if x == f64::INFINITY {
        json!("float:inf")
    } else if x == f64::NEG_INFINITY {
        json!("float:-inf")
    } else {
        json!(x)
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

/// A float field; absent means Python's default.
fn f64_field(obj: &Value, name: &str, default: Option<f64>) -> Result<f64, EvalError> {
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
        f64_field(o, "speed_through_water_kn", None)?,
        f64_field(o, "draft_m", None)?,
        f64_field(o, "beam_m", None)?,
        f64_field(o, "lpp_m", None)?,
    );
    Ok(core_squat::SquatInputs {
        block_coefficient: f64_field(o, "block_coefficient", Some(base.block_coefficient))?,
        water_depth_m: f64_field(o, "water_depth_m", Some(base.water_depth_m))?,
        channel_area_m2: f64_field(o, "channel_area_m2", Some(base.channel_area_m2))?,
        displacement_m3: f64_field(o, "displacement_m3", Some(base.displacement_m3))?,
        safety_margin_m: f64_field(o, "safety_margin_m", Some(base.safety_margin_m))?,
        ..base
    })
}

fn squat(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "compute" => match core_squat::compute(&squat_inputs(args)?) {
            Ok(r) => ok(json!({
                "method": r.method,
                "squat_m": float_json(r.squat_m),
                "squat_barrass_m": float_json(r.squat_barrass_m),
                "squat_icorels_m": r.squat_icorels_m.map_or(Value::Null, float_json),
                "blockage_ratio": float_json(r.blockage_ratio),
                "froude_depth": float_json(r.froude_depth),
                "static_ukc_m": float_json(r.static_ukc_m),
                "dynamic_ukc_m": float_json(r.dynamic_ukc_m),
                "grounds": r.grounds,
                "checks": checks_json(&r.checks),
                "warnings": r.warnings,
            })),
            Err(message) => value_error(message),
        },
        "max_speed_for_ukc" => {
            let result = core_squat::max_speed_for_ukc(
                f64_field(args, "draft_m", None)?,
                f64_field(args, "beam_m", None)?,
                f64_field(args, "lpp_m", None)?,
                f64_field(args, "water_depth_m", None)?,
                f64_field(args, "channel_area_m2", None)?,
                f64_field(args, "block_coefficient", Some(0.85))?,
                f64_field(
                    args,
                    "safety_margin_m",
                    Some(core_squat::DEFAULT_SAFETY_MARGIN_M),
                )?,
                f64_field(args, "v_hi_kn", Some(20.0))?,
                f64_field(args, "tol_kn", Some(0.01))?,
            );
            match result {
                Ok(v) => ok(float_json(v)),
                Err(message) => value_error(message),
            }
        }
        other => Err(EvalError::BadInput(format!(
            "squat has no parity function `{other}`"
        ))),
    }
}

fn wave_inputs(args: &Value) -> Result<core_waves::WaveResistanceInputs, EvalError> {
    let o = args
        .get("inputs")
        .ok_or_else(|| EvalError::BadInput("`args.inputs` is required".into()))?;
    let base = core_waves::WaveResistanceInputs::new(
        f64_field(o, "significant_wave_height_m", None)?,
        f64_field(o, "beam_waterline_m", None)?,
        f64_field(o, "lpp_m", None)?,
    );
    Ok(core_waves::WaveResistanceInputs {
        bow_waterline_length_m: f64_field(
            o,
            "bow_waterline_length_m",
            Some(base.bow_waterline_length_m),
        )?,
        heading_deg_off_bow: f64_field(o, "heading_deg_off_bow", Some(base.heading_deg_off_bow))?,
        peak_wave_period_s: f64_field(o, "peak_wave_period_s", Some(base.peak_wave_period_s))?,
        water_density_t_m3: f64_field(o, "water_density_t_m3", Some(base.water_density_t_m3))?,
        speed_kn: f64_field(o, "speed_kn", Some(base.speed_kn))?,
        ..base
    })
}

fn waves(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "compute" => match core_waves::compute(&wave_inputs(args)?) {
            Ok(r) => ok(json!({
                "added_resistance_kn": float_json(r.added_resistance_kn),
                "added_resistance_head_seas_kn": float_json(r.added_resistance_head_seas_kn),
                "heading_factor": float_json(r.heading_factor),
                "bow_waterline_length_m": float_json(r.bow_waterline_length_m),
                "wavelength_m": float_json(r.wavelength_m),
                "lambda_over_lpp": float_json(r.lambda_over_lpp),
                "added_power_kw": float_json(r.added_power_kw),
                "checks": checks_json(&r.checks),
                "warnings": r.warnings,
            })),
            Err(message) => value_error(message),
        },
        other => Err(EvalError::BadInput(format!(
            "waves has no parity function `{other}`"
        ))),
    }
}
