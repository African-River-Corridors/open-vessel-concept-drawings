//! `gas_stability` against its Reference Results (issues #18 and #17), through
//! `ovcd_format::eval`.
//!
//! `parity.rs` runs every case of every ported entry point. This file counts the cases per
//! function, so a function cannot go green by losing its cases, and holds the checks that need an
//! input the parity format cannot write (very large integers).

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

/// Run every `gas_stability` case of one function.
fn failures(only: &str) -> (usize, Vec<String>) {
    let cases = read_json(&dir("../../parity/cases/gas_stability.json"));
    let reference = read_json(&dir("../../parity/reference/gas_stability.json"));
    let tol = tolerance();
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
        if only != function {
            continue;
        }
        count += 1;
        match ovcd_format::eval("gas_stability", &case["input"]) {
            Ok(actual) => {
                if let Err(diffs) = compare(&result["output"], &actual, tol) {
                    bad.push(format!("{id}:\n    {}", diffs.join("\n    ")));
                }
            }
            Err(e) => bad.push(format!("{id}: {e}")),
        }
    }
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
    green("assess", 19);
}

fn eval(function: &str, args: Value) -> Result<Value, ovcd_format::EvalError> {
    ovcd_format::eval("gas_stability", &json!({ "fn": function, "args": args }))
}

#[test]
fn unknown_function_is_a_bad_input() {
    assert!(eval("nope", json!({})).is_err());
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

/// An int radius of any size stays an exact Python int. Expected values: the frozen Python
/// (3.12, Linux glibc) through `driver.run_case`.
#[test]
fn segment_fill_keeps_an_int_radius_exact_at_any_size() {
    let int = |text: &str| serde_json::from_str::<Value>(text).expect("integer literal");
    let overflow = json!({"error": {"type": "OverflowError",
        "message": "int too large to convert to float"}});
    for (radius, fraction, expected) in [
        // The full tank returns exact ints, even where 2 * radius leaves i64 or the radius u64.
        (
            "4611686018427387904",
            "1",
            json!({"ok": [int("9223372036854775808"), int("4611686018427387904")]}),
        ),
        (
            "-9223372036854775808",
            "1",
            json!({"ok": [int("-18446744073709551616"), int("-9223372036854775808")]}),
        ),
        (
            "100000000000000000000",
            "1",
            json!({"ok": [int("200000000000000000000"), int("100000000000000000000")]}),
        ),
        (
            "-100000000000000000000",
            "2",
            json!({"ok": [int("-200000000000000000000"), int("-100000000000000000000")]}),
        ),
        ("0", "1", json!({"ok": [0, 0]})),
        ("-0", "1", json!({"ok": [0, 0]})),
        // Below a full tank Python makes floats from the radius and its exact square.
        // A negative int radius below a full tank keeps its sign (kills the `delete -` mutant).
        (
            "-3",
            "0.5",
            json!({"ok": [-2.9999999999999996, -1.7267604552648372]}),
        ),
        (
            "-2",
            "0.25",
            json!({"ok": [-1.1920544934009658, -0.7004707927710812]}),
        ),
        (
            "13407807929942596727432598158855113001725820008116749133157341386316719597868925090818930617285026583319956944557042494632332259080864313586268726512374783",
            "0.5",
            json!({"ok": [1.3407807929942596e+154, 7.717357508403725e+153]}),
        ),
        (
            "13407807929942596727432598158855113001725820008116749133157341386316719597868925090818930617285026583319956944557042494632332259080864313586268726512374784",
            "0.5",
            overflow.clone(),
        ),
        (
            "-13407807929942596727432598158855113001725820008116749133157341386316719597868925090818930617285026583319956944557042494632332259080864313586268726512374784",
            "0.5",
            overflow.clone(),
        ),
    ] {
        let args: Value = serde_json::from_str(&format!(
            "{{\"radius\": {radius}, \"volume_fraction\": {fraction}}}"
        ))
        .unwrap();
        let actual = eval("segment_fill", args.clone()).unwrap();
        assert_eq!(actual, expected, "{args}");
        assert_eq!(compare(&expected, &actual, tolerance()), Ok(()), "{args}");
    }
    // 2 ** 512 and 10 ** 400: Python 3.12 says "int too large to convert to float" for both.
    let big = format!("1{}", "0".repeat(400));
    let args: Value =
        serde_json::from_str(&format!("{{\"radius\": {big}, \"volume_fraction\": 0.5}}")).unwrap();
    assert_eq!(eval("segment_fill", args).unwrap(), overflow);
    // A fraction of zero or less returns before Python touches the radius, so it cannot overflow.
    for fraction in ["0", "-1"] {
        let args: Value = serde_json::from_str(&format!(
            "{{\"radius\": {big}, \"volume_fraction\": {fraction}}}"
        ))
        .unwrap();
        assert_eq!(
            eval("segment_fill", args).unwrap(),
            json!({"ok": [0.0, 0.0]})
        );
    }
    // A float radius stays a float.
    let args = json!({"radius": 1e20, "volume_fraction": 1});
    assert_eq!(
        eval("segment_fill", args).unwrap(),
        json!({"ok": [2e20, 1e20]})
    );
}
