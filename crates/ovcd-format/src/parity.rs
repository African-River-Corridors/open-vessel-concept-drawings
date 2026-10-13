//! Compare an `eval` output with its Reference Result (spec 0001, "What identical results means").
//!
//! - Floats match within the relative and absolute tolerance in `parity/tolerance.json`.
//! - Everything discrete matches exactly: integers, booleans, strings (including messages), null,
//!   list length and order, object keys.
//! - An integer never matches a float, so a Python `int` result stays an `int`.
//! - An integer outside the 64-bit range is refused by [`parse`]: it would read as a float.
//! - NaN and infinity are the tagged strings `"float:nan"`, `"float:inf"`, `"float:-inf"`, and so
//!   match exactly.
//!
//! Deleted with the Reference Results when the port is complete.

use serde_json::Value;

/// Parity tolerance, read from `parity/tolerance.json`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tolerance {
    pub rel: f64,
    pub abs: f64,
}

impl Tolerance {
    /// Parse `parity/tolerance.json`.
    pub fn from_json(text: &str) -> Result<Self, String> {
        let v: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        let get = |k: &str| {
            v.get(k)
                .and_then(Value::as_f64)
                .ok_or_else(|| format!("tolerance.json: `{k}` must be a number"))
        };
        Ok(Self {
            rel: get("rel")?,
            abs: get("abs")?,
        })
    }

    /// Same rule as Python's `math.isclose(a, b, rel_tol=rel, abs_tol=abs)`.
    pub fn close(&self, a: f64, b: f64) -> bool {
        a == b || (a - b).abs() <= (self.rel * a.abs().max(b.abs())).max(self.abs)
    }
}

/// Parse a parity file or an `eval` output. Refuses an integer outside the 64-bit range
/// (`i64::MIN..=u64::MAX`): `serde_json` would read it as a float, so two different big integers
/// would compare equal.
pub fn parse(text: &str) -> Result<Value, String> {
    let value: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    // The text is valid JSON now, so outside a string a `-` or a digit starts a number.
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                i += 1;
                while bytes[i] != b'"' {
                    i += if bytes[i] == b'\\' { 2 } else { 1 };
                }
                i += 1;
            }
            b'-' | b'0'..=b'9' => {
                let start = i;
                while i < bytes.len()
                    && matches!(bytes[i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
                {
                    i += 1;
                }
                let token = &text[start..i];
                let integer = !token.contains(['.', 'e', 'E']);
                let fits = token.parse::<i64>().is_ok() || token.parse::<u64>().is_ok();
                if integer && !fits {
                    return Err(format!(
                        "integer {token} is outside the 64-bit range; it would compare as a float, so it is refused"
                    ));
                }
            }
            _ => i += 1,
        }
    }
    Ok(value)
}

/// Compare `actual` (from `eval`) with `expected` (the Reference Result). On a mismatch, returns
/// every differing path, such as `$.ok.density: expected 450.0, got 451.0`.
pub fn compare(expected: &Value, actual: &Value, tol: Tolerance) -> Result<(), Vec<String>> {
    let mut diffs = Vec::new();
    walk("$", expected, actual, tol, &mut diffs);
    if diffs.is_empty() { Ok(()) } else { Err(diffs) }
}

fn walk(path: &str, expected: &Value, actual: &Value, tol: Tolerance, diffs: &mut Vec<String>) {
    match (expected, actual) {
        (Value::Number(e), Value::Number(a)) => {
            let ok = match (e.is_f64(), a.is_f64()) {
                (true, true) => tol.close(
                    e.as_f64().unwrap_or(f64::NAN),
                    a.as_f64().unwrap_or(f64::NAN),
                ),
                (false, false) => e == a,
                _ => false,
            };
            if !ok {
                diffs.push(format!("{path}: expected {e}, got {a}"));
            }
        }
        (Value::Array(e), Value::Array(a)) => {
            if e.len() != a.len() {
                diffs.push(format!(
                    "{path}: expected {} items, got {}",
                    e.len(),
                    a.len()
                ));
            }
            for (i, (ev, av)) in e.iter().zip(a).enumerate() {
                walk(&format!("{path}[{i}]"), ev, av, tol, diffs);
            }
        }
        (Value::Object(e), Value::Object(a)) => {
            for (k, ev) in e {
                match a.get(k) {
                    Some(av) => walk(&format!("{path}.{k}"), ev, av, tol, diffs),
                    None => diffs.push(format!("{path}.{k}: missing")),
                }
            }
            for k in a.keys().filter(|k| !e.contains_key(*k)) {
                diffs.push(format!("{path}.{k}: unexpected"));
            }
        }
        _ => {
            if expected != actual {
                diffs.push(format!("{path}: expected {expected}, got {actual}"));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const TOL: Tolerance = Tolerance {
        rel: 1e-9,
        abs: 1e-9,
    };

    fn ok(e: Value, a: Value) -> bool {
        compare(&e, &a, TOL).is_ok()
    }

    #[test]
    fn floats_match_within_relative_tolerance() {
        assert!(ok(json!(450.0), json!(450.0 * (1.0 + 0.9e-9))));
        assert!(!ok(json!(450.0), json!(450.0 * (1.0 + 1.1e-9))));
    }

    #[test]
    fn floats_near_zero_match_within_absolute_tolerance() {
        assert!(ok(json!(0.0), json!(0.9e-9)));
        assert!(!ok(json!(0.0), json!(1.1e-9)));
    }

    #[test]
    fn an_integer_never_matches_a_float() {
        assert!(!ok(json!(3), json!(3.0)));
        assert!(!ok(json!(3.0), json!(3)));
        assert!(!ok(json!(3), json!(4)));
        assert!(ok(json!(3), json!(3)));
    }

    #[test]
    fn text_and_booleans_match_exactly() {
        assert!(!ok(
            json!("Unknown cargo type 'coal'"),
            json!("Unknown cargo type 'coal' ")
        ));
        assert!(!ok(json!(true), json!(false)));
        assert!(!ok(json!(null), json!(0)));
    }

    #[test]
    fn tagged_non_finite_floats_match_only_themselves() {
        assert!(ok(json!("float:nan"), json!("float:nan")));
        assert!(!ok(json!("float:inf"), json!("float:-inf")));
        assert!(!ok(json!("float:nan"), json!(0.0)));
    }

    #[test]
    fn lists_match_in_order_and_length() {
        assert!(!ok(json!([1, 2]), json!([2, 1])));
        assert!(!ok(json!([1, 2]), json!([1, 2, 3])));
    }

    #[test]
    fn objects_need_the_same_keys_and_report_the_path() {
        let e = json!({"ok": {"density": 450.0, "phase": "liquid_gas"}});
        let diffs = compare(&e, &json!({"ok": {"density": 451.0, "extra": 1}}), TOL).unwrap_err();
        assert!(
            diffs.contains(&"$.ok.density: expected 450.0, got 451.0".to_string()),
            "{diffs:?}"
        );
        assert!(
            diffs.contains(&"$.ok.phase: missing".to_string()),
            "{diffs:?}"
        );
        assert!(
            diffs.contains(&"$.ok.extra: unexpected".to_string()),
            "{diffs:?}"
        );
    }

    #[test]
    fn integers_above_the_64_bit_range_are_refused() {
        // Without the refusal both parse to the same f64 and compare equal.
        for text in [
            "100000000000000000000",
            "100000000000000000001",
            r#"{"ok": [1, -9223372036854775809]}"#,
            r#"{"ok": {"n": 18446744073709551616}}"#,
        ] {
            let err = parse(text).unwrap_err();
            assert!(err.contains("64-bit"), "{text}: {err}");
        }
    }

    #[test]
    fn numbers_inside_the_64_bit_range_and_big_floats_parse() {
        let v = parse(
            r#"{"ok": [18446744073709551615, -9223372036854775808, 1e+20, 1.5E300, -0.0,
                "100000000000000000000", "a \" 100000000000000000000"]}"#,
        )
        .unwrap();
        assert_eq!(v["ok"][0], json!(18446744073709551615u64));
        assert_eq!(v["ok"][1], json!(i64::MIN));
        assert_eq!(v["ok"][2], json!(1e20));
        assert_eq!(v["ok"][5], json!("100000000000000000000"));
        assert!(parse("{not json").is_err());
    }

    #[test]
    fn tolerance_parses_from_its_file() {
        let t = Tolerance::from_json(r#"{"rel": 1e-09, "abs": 2e-09, "reason": "x"}"#).unwrap();
        assert_eq!(
            t,
            Tolerance {
                rel: 1e-9,
                abs: 2e-9
            }
        );
        assert!(Tolerance::from_json(r#"{"rel": 1e-09}"#).is_err());
    }
}
