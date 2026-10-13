//! `eval(entry_point, json) -> json`: the parity seam. One dispatch over every parity entry
//! point. Input and output use the case and Reference Result shapes in `parity/README.md`.

use ovcd_core::barge_cost::{BargeCostInputs, BargeCostRates, BargeCostResult, MappingValue};
use ovcd_core::cargo::CargoProperties;
use ovcd_core::checks::Check;
use ovcd_core::pyint::PyInt;
use ovcd_core::pystr::repr_str;
use ovcd_core::vessel_standards::{Catalogue, VesselStandard};
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
        "barge_cost" => barge_cost(function, args),
        "vessel_standards" => vessel_standards(function, args),
        "protective_location" => protective_location(function, args),
        "pressure_vessel" => pressure_vessel(function, args),
        ep if ENTRY_POINTS.contains(&ep) => Err(EvalError::NotPorted(ep.into())),
        ep => Err(EvalError::UnknownEntryPoint(ep.into())),
    }
}

/// A float result, tagged the way the frozen driver tags it: JSON has no `inf` or `nan`, so they
/// become the strings `"float:inf"`, `"float:-inf"` and `"float:nan"`.
fn num(x: f64) -> Value {
    if x.is_nan() {
        json!("float:nan")
    } else if x.is_infinite() {
        json!(if x > 0.0 { "float:inf" } else { "float:-inf" })
    } else {
        json!(x)
    }
}

fn ok(value: Value) -> Result<Value, EvalError> {
    Ok(json!({ "ok": value }))
}

/// A raised Python exception of any class, as the frozen Python driver records it.
fn raised(type_name: &str, message: &str) -> Result<Value, EvalError> {
    Ok(json!({ "error": { "type": type_name, "message": message } }))
}

/// A JSON integer holding every digit of a Python `int`, whatever its size.
fn int_value(i: &PyInt) -> Value {
    serde_json::from_str(i.as_str()).expect("a PyInt is valid JSON integer text")
}

/// A Python `ValueError`, as the frozen Python driver records it.
fn value_error(message: String) -> Result<Value, EvalError> {
    Ok(json!({ "error": { "type": "ValueError", "message": message } }))
}

/// A Python exception other than `ValueError`, recorded the same way (class name and message).
fn py_error(kind: &str, message: String) -> Result<Value, EvalError> {
    Ok(json!({ "error": { "type": kind, "message": message } }))
}

/// A float argument. A JSON integer is refused: Python would keep an `int` and may echo it back
/// as an `int`, which the Rust port does not model, so a case must write `18.0`, not `18`.
fn f64_arg(args: &Value, name: &str) -> Result<f64, EvalError> {
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
            "barge_cost has no parity function `{other}`"
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
            "vessel_standards has no parity function `{other}`"
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
            "protective_location has no parity function `{other}`"
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
            "pressure_vessel has no parity function `{other}`"
        ))),
    }
}
