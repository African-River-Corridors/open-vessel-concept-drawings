//! `gas_stability` against its Reference Results (issue #18), through `ovcd_format::eval`.
//!
//! `parity.rs` runs `box_gz`, `segment_fill` and `criteria_set_for` on the seam. `assess` cases
//! describe the arrangement by `fit` or `capacity`, which need `gas_arrangement` and
//! `barge_capacity` (not ported yet), so `parity.rs` skips them (`parity/pending.json`). Here the
//! numbers `assess` reads come from `tests/fixtures/gas_stability_results.json`, made by
//! `parity/gas_basis_fixtures.py` from the frozen Python, and go to `eval` as `args.result`. The
//! Reference Results are the same files `parity.rs` uses. The fixture also holds int-`payload_t`
//! rows with the frozen Python's own output. When `fit` lands, `assess` joins the seam proper.

use ovcd_format::parity::{Tolerance, compare};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn dir(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn read_json(path: &Path) -> Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn tolerance() -> Tolerance {
    let text = std::fs::read_to_string(dir("../../parity/tolerance.json")).expect("tolerance.json");
    Tolerance::from_json(&text).expect("tolerance parses")
}

/// Run every `gas_stability` case; `assess` cases get their derived `result` from the fixture.
fn failures(only: &str) -> (usize, Vec<String>) {
    let cases = read_json(&dir("../../parity/cases/gas_stability.json"));
    let reference = read_json(&dir("../../parity/reference/gas_stability.json"));
    let fixtures = read_json(&dir("tests/fixtures/gas_stability_results.json"));
    let fixtures = fixtures["rows"].as_array().expect("fixture rows");
    let tol = tolerance();
    let mut next_assess = 0;
    let (mut count, mut bad) = (0, Vec::new());
    for (case, result) in cases["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .zip(reference["results"].as_array().expect("results"))
    {
        let id = case["id"].as_str().expect("id");
        assert_eq!(case["id"], result["id"], "run `cargo xtask regen`");
        let function = case["input"]["fn"].as_str().expect("fn");
        let mut args = case["input"]["args"].clone();
        if function == "assess" {
            let row = &fixtures[next_assess];
            next_assess += 1;
            assert_eq!(
                row["id"], case["id"],
                "fixture is stale: rerun parity/gas_basis_fixtures.py"
            );
            let derived = &row["result"];
            let obj = args.as_object_mut().expect("args object");
            obj.remove("fit");
            obj.remove("capacity");
            obj.insert("payload_t".into(), derived["payload_t"].clone());
            obj.insert("result".into(), derived.clone());
        }
        if only != function {
            continue;
        }
        count += 1;
        match ovcd_format::eval("gas_stability", &json!({ "fn": function, "args": args })) {
            Ok(actual) => {
                if let Err(diffs) = compare(&result["output"], &actual, tol) {
                    bad.push(format!("{id}:\n    {}", diffs.join("\n    ")));
                }
            }
            Err(e) => bad.push(format!("{id}: {e}")),
        }
    }
    assert_eq!(next_assess, fixtures.len(), "fixture has rows for no case");
    (count, bad)
}

fn green(function: &str, expected_cases: usize) {
    let (count, bad) = failures(function);
    assert_eq!(count, expected_cases, "{function}: case count changed");
    assert!(bad.is_empty(), "{} failed:\n{}", bad.len(), bad.join("\n"));
}

#[test]
fn box_gz_matches_every_reference_result() {
    green("box_gz", 38);
}

#[test]
fn segment_fill_matches_every_reference_result() {
    green("segment_fill", 21);
}

#[test]
fn criteria_set_for_matches_every_reference_result() {
    green("criteria_set_for", 2);
}

#[test]
fn assess_matches_every_reference_result() {
    green("assess", 13);
}

fn eval(function: &str, args: Value) -> Result<Value, ovcd_format::EvalError> {
    ovcd_format::eval("gas_stability", &json!({ "fn": function, "args": args }))
}

#[test]
fn unknown_function_is_a_bad_input() {
    assert!(eval("nope", json!({})).is_err());
}

#[test]
fn assess_by_fit_or_capacity_is_not_ported() {
    for key in ["fit", "capacity"] {
        assert_eq!(
            eval("assess", json!({ key: {}, "payload_t": 1.0 })),
            Err(ovcd_format::EvalError::NotPorted("gas_stability".into()))
        );
    }
}

/// Python returns `(2 * radius, radius)` for a full tank, in the type of `radius`.
#[test]
fn segment_fill_of_a_full_tank_keeps_an_int_radius_an_int() {
    // Values are the frozen Python driver's output.
    for (args, expected) in [
        (
            json!({"radius": 2, "volume_fraction": 1}),
            json!({"ok": [4, 2]}),
        ),
        (
            json!({"radius": 2, "volume_fraction": 1.5}),
            json!({"ok": [4, 2]}),
        ),
        (
            json!({"radius": 2.0, "volume_fraction": 1}),
            json!({"ok": [4.0, 2.0]}),
        ),
        (
            json!({"radius": 2, "volume_fraction": 0}),
            json!({"ok": [0.0, 0.0]}),
        ),
    ] {
        let actual = eval("segment_fill", args.clone()).unwrap();
        assert_eq!(actual, expected, "{args}");
        // Type-strict: compare would reject int against float.
        assert_eq!(compare(&expected, &actual, tolerance()), Ok(()), "{args}");
    }
}

/// Python keeps an int `payload_t` as an int in the Laden `cargo_t`, and its `sum()` (3.12) adds
/// the int without compensation. The rows hold the frozen Python's `cargo_t`, `displacement_t`,
/// `draught_m` and `kg_m` per condition. They must match exactly, to the last bit and in type. The
/// rest of the result is covered by the Reference Results.
#[test]
fn assess_with_an_int_payload_matches_python_exactly() {
    let fixtures = read_json(&dir("tests/fixtures/gas_stability_results.json"));
    let rows = fixtures["int_payload_rows"].as_array().expect("int rows");
    assert!(
        rows.len() >= 12,
        "fixture is stale: rerun parity/gas_basis_fixtures.py"
    );
    let mut bad = Vec::new();
    for row in rows {
        let mut args = row["options"].as_object().expect("options").clone();
        args.insert("result".into(), row["result"].clone());
        args.insert("payload_t".into(), row["payload_t"].clone());
        let actual = eval("assess", Value::Object(args)).unwrap();
        let id = format!("{} payload {}", row["id"], row["payload_t"]);
        let Some(want) = row["output"]["ok"].as_array() else {
            assert_eq!(actual, row["output"], "{id}");
            continue;
        };
        let got = actual["ok"]["conditions"].as_array().expect("conditions");
        assert_eq!(want.len(), got.len(), "{id}");
        for (w, g) in want.iter().zip(got) {
            for key in ["name", "cargo_t", "displacement_t", "draught_m", "kg_m"] {
                if w[key] != g[key] {
                    bad.push(format!(
                        "{id} {}.{key}: python {} rust {}",
                        w["name"], w[key], g[key]
                    ));
                }
            }
        }
    }
    assert!(bad.is_empty(), "{} differ:\n{}", bad.len(), bad.join("\n"));
}
