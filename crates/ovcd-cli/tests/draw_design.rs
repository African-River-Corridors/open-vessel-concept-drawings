//! `ovcd draw <file.vdesign>`: the three hull views of each Option, through the real binary.

use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../ovcd-format/tests/fixtures")
        .join(name)
        .to_str()
        .unwrap()
        .to_string()
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

fn temp(test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ovcd-draw-{}-{test}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn out_writes_document_normalised_content_dxf_svg_and_pdf_for_each_view() {
    let dir = temp("out");
    let (ok, stdout, stderr) = run(&[
        "draw",
        "--out",
        dir.to_str().unwrap(),
        &fixture("dry_bulk_box_hold.vdesign"),
    ]);
    assert!(ok, "{stderr}");
    assert_eq!(stdout.lines().count(), 15);
    for view in ["side-elevation", "plan", "midship-section"] {
        for ext in ["drawing.json", "normalised.json", "dxf", "svg", "pdf"] {
            assert!(
                dir.join(format!("option-1-{view}.{ext}")).is_file(),
                "{view} {ext}"
            );
        }
    }
    let doc: Value = serde_json::from_str(
        &std::fs::read_to_string(dir.join("option-1-plan.drawing.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(doc["standard_version"], "1.0");
    let norm: Value = serde_json::from_str(
        &std::fs::read_to_string(dir.join("option-1-plan.normalised.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(norm["sheet"]["size"], "A3");
    assert!(
        std::fs::read(dir.join("option-1-plan.pdf"))
            .unwrap()
            .starts_with(b"%PDF-")
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn document_prints_the_drawing_documents_only() {
    let (ok, stdout, stderr) = run(&["draw", "--document", &fixture("two_options.vdesign")]);
    assert!(ok, "{stderr}");
    let all: Value = serde_json::from_str(&stdout).expect("JSON on stdout");
    let options = all.as_object().unwrap();
    assert_eq!(options.len(), 2, "one entry for each Option");
    for views in options.values() {
        for view in ["side-elevation", "plan", "midship-section"] {
            assert_eq!(views[view]["standard_version"], "1.0", "{view}");
        }
    }
}

#[test]
fn option_picks_one_option_and_an_unknown_name_is_refused() {
    let file = fixture("two_options.vdesign");
    let (_, stdout, _) = run(&["draw", "--document", &file]);
    let names: Vec<String> = serde_json::from_str::<Value>(&stdout)
        .unwrap()
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    let (ok, stdout, _) = run(&["draw", "--document", "--option", &names[1], &file]);
    assert!(ok);
    assert_eq!(
        serde_json::from_str::<Value>(&stdout)
            .unwrap()
            .as_object()
            .unwrap()
            .len(),
        1
    );
    let (ok, _, stderr) = run(&["draw", "--document", "--option", "No such Option", &file]);
    assert!(!ok);
    assert!(stderr.contains("no Option named"), "{stderr}");
    assert!(
        stderr.contains(&names[0]),
        "the message lists the Options: {stderr}"
    );
}

#[test]
fn a_convoy_gets_no_drawings_and_the_outputs_panel_message() {
    let dir = temp("convoy");
    let (ok, _, stderr) = run(&[
        "draw",
        "--out",
        dir.to_str().unwrap(),
        &fixture("pushed_convoy.vdesign"),
    ]);
    assert!(!ok);
    assert!(
        stderr.contains("convoy results not yet available"),
        "{stderr}"
    );
    assert!(!dir.exists(), "nothing is written");
}

#[test]
fn it_needs_exactly_one_of_out_and_document() {
    let file = fixture("dry_bulk_box_hold.vdesign");
    let (ok, _, stderr) = run(&["draw", &file]);
    assert!(!ok);
    assert!(stderr.contains("--out"), "{stderr}");
    let (ok, _, _) = run(&["draw", "--out", "x", "--document", &file]);
    assert!(!ok);
}
