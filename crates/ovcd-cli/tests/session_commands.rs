//! The `ovcd schema` command and the Design session against `ovcd run --json`, on the example
//! Designs.

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

fn path_str(p: &Path) -> &str {
    p.to_str().unwrap()
}

/// The example Designs of the `ovcd-format` tests.
fn fixtures() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../ovcd-format/tests/fixtures");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("fixtures folder")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "vdesign"))
        .collect();
    files.sort();
    assert_eq!(files.len(), 6, "{files:?}");
    files
}

fn session_evaluate(path: &Path) -> Value {
    let text = std::fs::read_to_string(path).unwrap();
    let mut s = ovcd_format::session::Session::new();
    s.open(&text)
        .unwrap_or_else(|e| panic!("open {path:?}: {e}"));
    s.evaluate()
        .unwrap_or_else(|e| panic!("evaluate {path:?}: {e}"))
}

/// One Option as `ovcd run --json` prints it: results and hash, or the engine's message.
fn run_json_option(option: &Value) -> Value {
    if option["results"].is_null() {
        json!({
            "name": option["name"],
            "results": Value::Null,
            "message": option["message"],
        })
    } else {
        json!({
            "name": option["name"],
            "results_hash": option["results_hash"],
            "hash_status": option["hash_status"],
            "results": option["results"],
        })
    }
}

#[test]
fn schema_prints_the_same_schema_the_session_gives() {
    let out = ovcd(&["schema"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let printed: Value = serde_json::from_str(&stdout(&out)).expect("schema is JSON");
    let from_session = ovcd_format::session::Session::new().input_schema();
    assert_eq!(printed, from_session);
}

#[test]
fn evaluate_gives_exactly_what_run_json_gives_for_every_example_design() {
    for path in fixtures() {
        let validate = ovcd(&["validate", path_str(&path)]);
        assert!(
            validate.status.success(),
            "{} {:?}{}",
            path.display(),
            stdout(&validate),
            stderr(&validate)
        );
        assert!(
            stdout(&validate).contains("0 error(s)"),
            "{} {}",
            path.display(),
            stdout(&validate)
        );

        let out = ovcd(&["run", "--json", path_str(&path)]);
        // `run` exits non-zero when an Option is not computable (a pushed convoy), and still
        // prints the JSON. The session must give the same JSON either way.
        let run: Value = serde_json::from_str(&stdout(&out))
            .unwrap_or_else(|e| panic!("run --json is JSON: {e}{}{}", stdout(&out), stderr(&out)));

        let eval = session_evaluate(&path);
        assert_eq!(
            eval["engine_version"],
            run["engine_version"],
            "{}",
            path.display()
        );
        assert_eq!(
            eval["file_engine_version"],
            run["file_engine_version"],
            "{}",
            path.display()
        );
        let eval_options: Vec<Value> = eval["options"]
            .as_array()
            .unwrap()
            .iter()
            .map(run_json_option)
            .collect();
        assert_eq!(
            Value::Array(eval_options),
            run["options"],
            "{}",
            path.display()
        );
    }
}
