//! `eval(entry_point, json) -> json`: the parity seam. One dispatch over every parity entry
//! point. Input and output use the case and Reference Result shapes in `parity/README.md`.

use ovcd_core::cargo::CargoProperties;
use ovcd_core::hull::HullGeometry;
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
        "hull" => hull(function, args),
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

fn f64_arg(args: &Value, name: &str) -> Result<f64, EvalError> {
    args.get(name)
        .and_then(Value::as_f64)
        .ok_or_else(|| EvalError::BadInput(format!("`args.{name}` must be a number")))
}

fn hull(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        // One hull, every public quantity at `args.draft`, then the plating areas, in the key
        // order of the frozen driver.
        "quantities" => {
            let spec = args
                .get("hull")
                .ok_or_else(|| EvalError::BadInput("`args.hull` is missing".into()))?;
            let h = HullGeometry::new(
                f64_arg(spec, "loa")?,
                f64_arg(spec, "beam")?,
                f64_arg(spec, "depth")?,
                f64_arg(spec, "bow_rake_length")?,
                f64_arg(spec, "bow_rake_rise")?,
                f64_arg(spec, "stern_rake_length")?,
                f64_arg(spec, "stern_rake_rise")?,
            );
            let d = f64_arg(args, "draft")?;
            ok(json!({
                "parallel_length": h.parallel_length(),
                "displaced_volume": h.displaced_volume(d),
                "waterplane_area": h.waterplane_area(d),
                "waterplane_length": h.waterplane_length(d),
                "waterplane_inertia_transverse": h.waterplane_inertia_transverse(d),
                "waterplane_inertia_longitudinal": h.waterplane_inertia_longitudinal(d),
                "KB": h.KB(d),
                "LCB": h.LCB(d),
                "BM_transverse": h.BM_transverse(d),
                "BM_longitudinal": h.BM_longitudinal(d),
                "bottom_area": h.bottom_area(),
                "side_area": h.side_area(),
                "deck_area": h.deck_area(),
                "bow_rake_area": h.bow_rake_area(),
                "stern_rake_area": h.stern_rake_area(),
                "total_hull_surface_area": h.total_hull_surface_area(),
            }))
        }
        other => Err(EvalError::BadInput(format!(
            "hull has no parity function `{other}`"
        ))),
    }
}
