//! Seam 2: the Design session against the real `ovcd` binary. `ovcd schema` prints the session's
//! input schema; the session's new, open and evaluate give what `ovcd new`, `fill` and `run
//! --json` give, for every example Design.

use ovcd_format::{DesignSession, Profile, RunOutcome};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn ovcd(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ovcd"))
        .args(args)
        .output()
        .expect("ovcd runs")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn fixtures() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../ovcd-format/tests/fixtures");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("fixtures folder")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "vdesign"))
        .collect();
    files.sort();
    assert_eq!(files.len(), 8, "the eight example Designs");
    files
}

fn temp_dir(test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ovcd-session-{}-{test}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn schema_prints_the_session_input_schema() {
    let out = ovcd(&["schema"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let printed: Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(printed, ovcd_format::input_schema(&Profile::built_in()));
}

#[test]
fn schema_takes_a_profile() {
    let dir = temp_dir("schema-profile");
    let mut profile: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    profile["name"] = json!("mine");
    profile["inputs"]["max_speed_kn"]["default"] = json!(12.0);
    let file = dir.join("mine.profile.json");
    std::fs::write(&file, profile.to_string()).unwrap();
    let out = ovcd(&["schema", "--profile", file.to_str().unwrap()]);
    assert!(out.status.success(), "{}", stderr(&out));
    let printed: Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(printed["profile"], json!("mine"));
    let speed = printed["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["name"] == "max_speed_kn")
        .unwrap();
    assert_eq!(speed["default"], json!(12.0));
    assert!(
        ovcd(&["--help"])
            .stdout
            .windows(11)
            .any(|w| w == b"ovcd schema")
    );
}

#[test]
fn new_design_equals_ovcd_new() {
    let out = ovcd(&["new"]);
    let session = DesignSession::new_design(Profile::built_in()).unwrap();
    assert_eq!(session.design().to_text(), stdout(&out));
}

#[test]
fn open_of_a_hand_written_file_equals_ovcd_fill() {
    let dir = temp_dir("open-fill");
    let file = dir.join("hand.vdesign");
    let hand = json!({
        "format": "vdesign", "format_version": 2, "engine_version": "0.1.0",
        "options": [
            { "name": "A", "inputs": {
                "cargo": "LPG_propane", "hull_form": "raked_box",
                "formation": { "kind": "self_propelled" },
                "length_overall_m": 100, "beam_m": 12, "moulded_depth_m": 5 } },
            { "name": "B", "inputs": {
                "cargo": "bauxite", "hull_form": "raked_box",
                "formation": { "kind": "pushed_convoy", "abreast": 2, "in_line": 2 },
                "length_overall_m": 76.5, "beam_m": 11.4, "moulded_depth_m": 3.5,
                "cargo_containment": "sloped_hopper_hold", "block_coefficient": 0.9 } }
        ]
    });
    std::fs::write(&file, hand.to_string()).unwrap();
    let filled = ovcd(&["fill", file.to_str().unwrap()]);
    assert!(filled.status.success(), "{}", stderr(&filled));
    let (session, _) = DesignSession::open(&hand.to_string(), Profile::built_in()).unwrap();
    assert_eq!(session.design().to_text(), stdout(&filled));
}

#[test]
fn every_example_design_validates_and_evaluates_as_run_json_does() {
    for file in fixtures() {
        let path = file.to_str().unwrap();
        let validated = ovcd(&["validate", path]);
        assert!(validated.status.success(), "{path}: {}", stdout(&validated));

        let run = ovcd(&["run", "--json", path]);
        let run_json: Value = serde_json::from_str(&stdout(&run)).unwrap();
        let (session, _) = DesignSession::open(
            &std::fs::read_to_string(&file).unwrap(),
            Profile::built_in(),
        )
        .unwrap();
        let evaluation = session.evaluate();
        let cli_options = run_json["options"].as_array().unwrap();
        assert_eq!(evaluation.options.len(), cli_options.len(), "{path}");
        for (o, cli) in evaluation.options.iter().zip(cli_options) {
            assert_eq!(json!(o.name), cli["name"], "{path}");
            match o.outcome.as_ref().unwrap() {
                RunOutcome::Results { results, hash } => {
                    assert_eq!(results, &cli["results"], "{path} / {}", o.name);
                    assert_eq!(json!(hash), cli["results_hash"], "{path} / {}", o.name);
                }
                RunOutcome::NotAvailable(m) | RunOutcome::Failed(m) => {
                    assert!(cli["results"].is_null(), "{path}");
                    assert_eq!(json!(m), cli["message"], "{path}");
                }
            }
        }
    }
}

#[test]
fn a_saved_design_passes_ovcd_validate() {
    let dir = temp_dir("save");
    let (mut session, _) = DesignSession::open(
        &std::fs::read_to_string(&fixtures()[0]).unwrap(),
        Profile::built_in(),
    )
    .unwrap();
    let first = session.design().options[0].name.clone();
    session.set_input(&first, "beam_m", json!(12.0)).unwrap();
    session.add_option(None).unwrap();
    let file = dir.join("saved.vdesign");
    std::fs::write(&file, session.save().unwrap()).unwrap();
    let out = ovcd(&["validate", file.to_str().unwrap()]);
    assert!(out.status.success(), "{}", stdout(&out));
    let run = ovcd(&["run", file.to_str().unwrap()]);
    assert!(
        stdout(&run).contains("(matches the file)"),
        "{}",
        stdout(&run)
    );
}

#[test]
fn session_messages_equal_ovcd_validate_messages() {
    let dir = temp_dir("messages");
    let (mut session, _) = DesignSession::open(
        &std::fs::read_to_string(&fixtures()[0]).unwrap(),
        Profile::built_in(),
    )
    .unwrap();
    let first = session.design().options[0].name.clone();
    session
        .set_input(&first, "length_overall_m", json!(300.0))
        .unwrap();
    session
        .set_input(&first, "min_freeboard_m", json!(4.0))
        .unwrap();
    session.set_input(&first, "beam_m", json!(0.5)).unwrap();
    let file = dir.join("bad.vdesign");
    std::fs::write(&file, session.design().to_text()).unwrap();
    let out = ovcd(&["validate", file.to_str().unwrap()]);
    assert!(!out.status.success());
    let printed: Vec<String> = stdout(&out)
        .lines()
        .filter(|l| l.starts_with("error: ") || l.starts_with("warning: "))
        .map(String::from)
        .collect();
    let mut from_session = Vec::new();
    for sev in ["error", "warning"] {
        for o in &session.evaluate().options {
            for m in &o.messages {
                let is_error = m.severity == ovcd_format::Severity::Error;
                if is_error == (sev == "error") {
                    from_session.push(format!("{sev}: Option '{}': {}", o.name, m.text));
                }
            }
        }
    }
    assert_eq!(from_session, printed);
}
