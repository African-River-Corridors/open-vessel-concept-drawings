//! `ovd draw run`: the conformance runner of the Drawing Format Standard (clause 17.3), through the
//! real binary.

use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

fn document(case: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        // Four documents of the standard's conformance suite, copied from the otd crate's vendored copy.
        .join("tests/data/conformance")
        .join(case)
        .join("input.json")
}

fn run(args: &[&str]) -> (bool, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_ovd"))
        .args(args)
        .output()
        .expect("ovd runs");
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

#[test]
fn build_writes_the_normalised_content_the_dxf_the_svg_and_the_pdf() {
    let path = document("tb-register-two");
    let out = std::env::temp_dir().join(format!("ovd-draw-build-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let (ok, stdout, stderr) = run(&[
        "draw",
        "build",
        path.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert!(ok, "{stderr}");
    assert_eq!(stdout.lines().count(), 4);
    for name in [
        "input.normalised.json",
        "input.dxf",
        "input.svg",
        "input.pdf",
    ] {
        assert!(out.join(name).is_file(), "{name}");
    }
    assert!(
        std::fs::read(out.join("input.pdf"))
            .unwrap()
            .starts_with(b"%PDF-")
    );
    assert!(
        std::fs::read_to_string(out.join("input.svg"))
            .unwrap()
            .contains("<svg ")
    );
    // a document with errors writes nothing
    let bad = document("err-scale-300");
    let out2 = out.join("none");
    let (ok, _, stderr) = run(&[
        "draw",
        "build",
        bad.to_str().unwrap(),
        "--out",
        out2.to_str().unwrap(),
    ]);
    assert!(!ok);
    assert!(stderr.contains("7.3.2"));
    assert!(!out2.exists());
    let _ = std::fs::remove_dir_all(&out);
}
