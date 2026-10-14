//! `ovcd draw run`: the conformance runner of the Drawing Format Standard (clause 17.3), through the
//! real binary.

use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

fn document(case: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../ovcd-draw/standard/conformance/documents")
        .join(case)
        .join("input.json")
}

fn run(args: &[&str]) -> (bool, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_ovcd"))
        .args(args)
        .output()
        .expect("ovcd runs");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn a_good_document_prints_its_normalised_content() {
    let path = document("sheet-a3-landscape");
    let (ok, out, _) = run(&["draw", "run", path.to_str().unwrap()]);
    assert!(ok);
    let result: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(result["result"], "ok");
    assert_eq!(result["normalised"]["sheet"]["size"], "A3");
}

#[test]
fn a_bad_document_prints_its_error_clauses() {
    let path = document("err-scale-300");
    let (ok, out, _) = run(&["draw", "run", path.to_str().unwrap()]);
    assert!(ok, "the runner reports errors in its result and exits 0");
    let result: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(result["result"], "error");
    assert_eq!(result["errors"][0]["clause"], "7.3.2");
}

#[test]
fn the_same_document_gives_the_same_bytes() {
    let path = document("two-viewports");
    let first = run(&["draw", "run", path.to_str().unwrap()]).1;
    let second = run(&["draw", "run", path.to_str().unwrap()]).1;
    assert_eq!(first, second);
}

#[test]
fn a_missing_file_is_an_error() {
    let (ok, _, err) = run(&["draw", "run", "/nonexistent/doc.json"]);
    assert!(!ok);
    assert!(err.contains("cannot read"));
}
