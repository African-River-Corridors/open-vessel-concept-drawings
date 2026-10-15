//! JSON adapter for the `gas_stability` entry point: `box_gz`, `segment_fill`,
//! `criteria_set_for` and `assess`. It copies the output shapes of the Python functions,
//! including which numbers are ints.
//!
//! `eval("gas_stability", ..)` routes here.
//!
//! `assess` takes the arrangement it assesses as `result` (a capacity result object, used as it is),
//! or as `fit` (runs `gas_arrangement.fit` and
//! uses its engine result and, unless `payload_t` is given, its payload) or as `capacity` (runs
//! `barge_capacity.compute`).

use crate::EvalError;
use crate::eval::{capacity_inputs, raised_py};
use crate::objects::capacity_result_from_json;
use ovcd_core::gas_stability::{self as gs, Condition, Criterion, PyError, PyNum};
use serde_json::{Map, Value, json};

/// A float as the answers write it: NaN and infinity are tagged text.
fn float(x: f64) -> Value {
    if x.is_nan() {
        json!("float:nan")
    } else if x.is_infinite() {
        json!(if x > 0.0 { "float:inf" } else { "float:-inf" })
    } else {
        json!(x)
    }
}

fn num_arg(args: &Value, name: &str) -> Result<f64, EvalError> {
    args.get(name)
        .and_then(Value::as_f64)
        .ok_or_else(|| EvalError::BadInput(format!("`args.{name}` must be a number")))
}

/// A number that Python may hold as an int: a JSON integer stays an int.
fn py_num_arg(args: &Value, name: &str) -> Result<PyNum, EvalError> {
    match args.get(name).and_then(Value::as_i64) {
        Some(i) => Ok(PyNum::Int(i)),
        None => num_arg(args, name).map(PyNum::Float),
    }
}

fn opt_num_arg(args: &Value, name: &str, default: f64) -> Result<f64, EvalError> {
    match args.get(name) {
        None => Ok(default),
        Some(_) => num_arg(args, name),
    }
}

/// The sign and decimal digits of a JSON integer literal of any size. Python keeps such a number
/// as an exact `int`; a float literal (`2.0`, `1e3`) gives `None`.
fn int_literal(value: &Value) -> Option<(bool, String)> {
    let Value::Number(n) = value else { return None };
    let text = n.to_string();
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.as_str()),
    };
    let all_digits = !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit());
    let zero = digits.bytes().all(|b| b == b'0');
    all_digits.then(|| (negative && !zero, digits.to_string()))
}

/// Decimal digits of `2 * magnitude`.
fn double_digits(magnitude: &str) -> String {
    let mut out = Vec::with_capacity(magnitude.len() + 1);
    let mut carry = 0u8;
    for b in magnitude.bytes().rev() {
        let d = (b - b'0') * 2 + carry;
        out.push(b'0' + d % 10);
        carry = d / 10;
    }
    if carry > 0 {
        out.push(b'0' + carry);
    }
    out.reverse();
    String::from_utf8(out).expect("ascii digits")
}

/// Decimal digits of `magnitude * magnitude` (schoolbook, base 10).
fn square_digits(magnitude: &str) -> String {
    let d: Vec<u32> = magnitude
        .bytes()
        .rev()
        .map(|b| u32::from(b - b'0'))
        .collect();
    let mut acc = vec![0u32; d.len() * 2 + 1];
    for (i, &x) in d.iter().enumerate() {
        for (j, &y) in d.iter().enumerate() {
            acc[i + j] += x * y;
        }
    }
    let mut carry = 0u32;
    for cell in &mut acc {
        let v = *cell + carry;
        *cell = v % 10;
        carry = v / 10;
    }
    while acc.len() > 1 && acc.last() == Some(&0) {
        acc.pop();
    }
    acc.iter()
        .rev()
        .map(|&c| char::from(b'0' + c as u8))
        .collect()
}

/// An exact integer's decimal digits as a float, rounded to nearest even as Python's
/// `float(int)` does. `None` where Python raises `OverflowError`.
fn digits_to_float(digits: &str) -> Option<f64> {
    digits.parse::<f64>().ok().filter(|x| x.is_finite())
}

/// `segment_fill` with an int `radius`, as Python runs it. A full tank returns the exact ints
/// `(2 * radius, radius)`. Otherwise Python turns `radius` and `radius ** 2` (an exact int) into
/// floats, and raises `OverflowError("int too large to convert to float")` when either is out of
/// range.
fn segment_fill_int_radius(
    negative: bool,
    magnitude: &str,
    fraction: f64,
) -> Result<Value, EvalError> {
    let sign = if negative { "-" } else { "" };
    if fraction >= 1.0 {
        let int = |digits: &str| -> Value {
            serde_json::from_str(&format!("{sign}{digits}")).expect("integer literal")
        };
        return Ok(json!({ "ok": [int(&double_digits(magnitude)), int(magnitude)] }));
    }
    // A fraction of zero or less returns before Python touches the radius.
    let (radius, radius_sq) = if fraction <= 0.0 {
        (0.0, 0.0)
    } else {
        match (
            digits_to_float(magnitude),
            digits_to_float(&square_digits(magnitude)),
        ) {
            (Some(r), Some(r2)) => (if negative { -r } else { r }, r2),
            _ => return py_result(Err(PyError::IntToFloat)),
        }
    };
    py_result(
        gs::segment_fill_int(radius, radius_sq, fraction).map(|(h, y)| json!([float(h), float(y)])),
    )
}

fn py_result(r: Result<Value, PyError>) -> Result<Value, EvalError> {
    Ok(match r {
        Ok(v) => json!({ "ok": v }),
        Err(e) => json!({ "error": { "type": e.type_name(), "message": e.message() } }),
    })
}

fn criterion_json(c: &Criterion) -> Value {
    let value = match c.value {
        PyNum::Int(i) => json!(i),
        PyNum::Float(x) => float(x),
    };
    json!({
        "name": c.name,
        "value": value,
        "limit": float(c.limit),
        "ok": c.ok,
        "source": c.source,
    })
}

/// A `Condition` as `to_json` writes it. When the barge does not float (`gz` is empty), Python
/// stores the int `-1` or `0` in the fields it does not compute.
pub(crate) fn condition_json(c: &Condition) -> Value {
    let floats = !c.gz.is_empty();
    let fl_or_int = |x: f64| if floats { float(x) } else { json!(x as i64) };
    let mut m = Map::new();
    m.insert("name".into(), json!(c.name));
    m.insert(
        "cargo_t".into(),
        if c.cargo_is_int {
            json!(c.cargo_t as i64)
        } else {
            float(c.cargo_t)
        },
    );
    m.insert("fill_fraction".into(), float(c.fill_fraction));
    m.insert("displacement_t".into(), float(c.displacement_t));
    m.insert("draught_m".into(), float(c.draught_m));
    m.insert("kg_m".into(), float(c.kg_m));
    m.insert("gm_solid_m".into(), fl_or_int(c.gm_solid_m));
    m.insert("free_surface_m".into(), fl_or_int(c.free_surface_m));
    m.insert("gm_fluid_m".into(), fl_or_int(c.gm_fluid_m));
    m.insert(
        "gz".into(),
        Value::Array(c.gz.iter().map(|&(d, g)| json!([d, float(g)])).collect()),
    );
    m.insert("max_gz_m".into(), fl_or_int(c.max_gz_m));
    m.insert("max_gz_deg".into(), json!(c.max_gz_deg));
    m.insert("area_0_27".into(), fl_or_int(c.area_0_27));
    m.insert("area_0_30".into(), fl_or_int(c.area_0_30));
    m.insert("area_0_40".into(), fl_or_int(c.area_0_40));
    m.insert("area_30_40".into(), fl_or_int(c.area_30_40));
    m.insert(
        "deck_edge_deg".into(),
        c.deck_edge_deg.map_or(Value::Null, float),
    );
    m.insert(
        "criteria".into(),
        Value::Array(c.criteria.iter().map(criterion_json).collect()),
    );
    Value::Object(m)
}

/// Run one `gas_stability` function on case-shaped arguments and return `{"ok": ..}` or
/// `{"error": ..}`.
pub fn run(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "box_gz" => py_result(
            gs::box_gz(
                num_arg(args, "beam")?,
                num_arg(args, "depth")?,
                num_arg(args, "draught")?,
                num_arg(args, "kg")?,
                num_arg(args, "phi_deg")?,
            )
            .map(float),
        ),
        "segment_fill" => {
            let fraction = num_arg(args, "volume_fraction")?;
            if let Some((negative, magnitude)) = args.get("radius").and_then(int_literal) {
                return segment_fill_int_radius(negative, &magnitude, fraction);
            }
            let radius = num_arg(args, "radius")?;
            py_result(gs::segment_fill(radius, fraction).map(|(h, y)| json!([float(h), float(y)])))
        }
        "criteria_set_for" => {
            let ruleset = args
                .get("ruleset")
                .and_then(Value::as_str)
                .ok_or_else(|| EvalError::BadInput("`args.ruleset` must be a string".into()))?;
            py_result(Ok(json!(gs::criteria_set_for(ruleset))))
        }
        "assess" => {
            // The result to assess comes from a gas fit (`fit`) or a capacity run (`capacity`).
            // A capacity result the caller passes (`result`) is used as it is.
            let (result, default_payload) = if let Some(passed) = args.get("result") {
                (capacity_result_from_json(passed, "result")?, None)
            } else if let Some(fit_args) = args.get("fit") {
                match crate::gas_arrangement::run_fit(fit_args)? {
                    Ok(f) => (f.engine_result, Some(f.payload_t)),
                    Err(e) => return raised_py(&e),
                }
            } else {
                let cap = args.get("capacity").ok_or_else(|| {
                    EvalError::BadInput("`args.fit` or `args.capacity` is required".into())
                })?;
                let inputs = capacity_inputs(&json!({ "inputs": cap }))?;
                match ovcd_core::barge_capacity::compute(&inputs) {
                    Ok(r) => (r, None),
                    Err(e) => return raised_py(&e),
                }
            };
            let result = gs::GasCapacityResult::try_from(&result)
                .map_err(|m| EvalError::BadInput(m.into()))?;
            let payload = match (args.get("payload_t"), default_payload) {
                (None, Some(p)) => PyNum::Float(p),
                _ => py_num_arg(args, "payload_t")?,
            };
            let cb = opt_num_arg(args, "block_coefficient", gs::DEFAULT_BLOCK_COEFFICIENT)?;
            let rho = opt_num_arg(args, "water_density", gs::DEFAULT_WATER_DENSITY)?;
            let cset = match args.get("criteria_set") {
                None => gs::DEFAULT_CRITERIA_SET,
                Some(v) => v.as_str().ok_or_else(|| {
                    EvalError::BadInput("`args.criteria_set` must be a string".into())
                })?,
            };
            py_result(gs::assess_num(&result, payload, cb, rho, cset).map(|a| {
                json!({
                    "criteria_set": a.criteria_set,
                    "conditions": a.conditions.iter().map(condition_json).collect::<Vec<_>>(),
                    "limitations": a.limitations,
                })
            }))
        }
        other => Err(EvalError::BadInput(format!(
            "gas_stability has no function `{other}`"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digit_helpers_match_python_ints() {
        // Values from Python: 2*n and n*n.
        for (n, double, square) in [
            ("0", "0", "0"),
            ("5", "10", "25"),
            ("9", "18", "81"),
            ("99", "198", "9801"),
            (
                "12345678901234567890",
                "24691357802469135780",
                "152415787532388367501905199875019052100",
            ),
            (
                "999999999999999999999",
                "1999999999999999999998",
                "999999999999999999998000000000000000000001",
            ),
        ] {
            assert_eq!(double_digits(n), double, "2*{n}");
            assert_eq!(square_digits(n), square, "{n}**2");
        }
    }

    #[test]
    fn int_literal_reads_integers_only() {
        let lit = |s: &str| int_literal(&serde_json::from_str::<Value>(s).unwrap());
        assert_eq!(lit("7"), Some((false, "7".into())));
        assert_eq!(lit("-7"), Some((true, "7".into())));
        assert_eq!(lit("-0"), Some((false, "0".into())));
        assert_eq!(
            lit("100000000000000000000"),
            Some((false, "100000000000000000000".into()))
        );
        assert_eq!(lit("2.0"), None);
        assert_eq!(lit("1e3"), None);
        assert_eq!(lit("\"7\""), None);
        assert_eq!(digits_to_float("123"), Some(123.0));
        assert_eq!(digits_to_float(&"9".repeat(400)), None);
    }
}
