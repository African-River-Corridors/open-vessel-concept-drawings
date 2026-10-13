//! `gas_stability` against its Reference Results (issue #18).
//!
//! `box_gz`, `segment_fill` and `criteria_set_for` cases run straight from `parity/cases`.
//! `assess` cases describe the arrangement by `fit` or `capacity`, which need `gas_arrangement`
//! and `barge_capacity` (not ported yet). So the numbers `assess` reads come from
//! `tests/fixtures/gas_stability_results.json`, made by `parity/gas_basis_fixtures.py` from the
//! frozen Python. The Reference Results are the same files `parity.rs` uses. When `fit` lands this
//! test is replaced by the entry point in `parity/ported.json`.

use ovcd_format::gas_stability::run;
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
        match run(function, &args) {
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

#[test]
fn unknown_function_is_a_bad_input() {
    assert!(run("nope", &json!({})).is_err());
}
