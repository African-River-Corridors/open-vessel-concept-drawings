//! Seam: `ovcd rules` and `--rule-set`, through the real binary (spec 0004, issue #268).

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

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn example() -> String {
    root()
        .join("crates/ovcd-format/rules/examples/example-1.0.0.json")
        .to_str()
        .unwrap()
        .to_string()
}

fn fixture(name: &str) -> String {
    root()
        .join("crates/ovcd-format/tests/fixtures")
        .join(name)
        .to_str()
        .unwrap()
        .to_string()
}

fn temp_file(test: &str, value: &Value) -> String {
    let path = std::env::temp_dir().join(format!("ovcd-rules-{}-{test}.json", std::process::id()));
    std::fs::write(&path, serde_json::to_string_pretty(value).unwrap()).unwrap();
    path.to_str().unwrap().to_string()
}

#[test]
fn rules_lists_the_example_set_loaded_at_run_time() {
    let out = ovcd(&["rules", "--rule-set", &example()]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("example@1.0.0  draft  6 Rules"), "{text}");
    // Blocking and unverified: a warning, not a refusal.
    assert!(stderr(&out).contains("blocking Rule `example/1.1/area-0-30` is unverified"));
}

#[test]
fn rules_show_prints_every_rule() {
    for spec in ["example", "example@1.0.0"] {
        let out = ovcd(&["rules", "show", spec, "--rule-set", &example()]);
        assert!(out.status.success(), "{}", stderr(&out));
        let text = stdout(&out);
        assert!(text.starts_with("example 1.0.0: "), "{text}");
        assert!(text.contains("example/2.1/gm-table"));
        assert!(text.contains("verified by agent on 2026-10-20"));
    }
    let out = ovcd(&["rules", "show", "example@9.9.9", "--rule-set", &example()]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("no Rule Set `example@9.9.9`"));
}

#[test]
fn a_bad_rule_set_file_is_refused_by_every_command() {
    let mut v: Value = serde_json::from_str(&std::fs::read_to_string(example()).unwrap()).unwrap();
    v["rules"][0]["id"] = json!("not a rule id");
    let bad = temp_file("bad", &v);
    let design = fixture("dry_bulk_box_hold.vdesign");
    for args in [
        vec!["rules", "--rule-set", &bad],
        vec!["run", "--rule-set", &bad, &design],
        vec!["validate", "--rule-set", &bad, &design],
        vec!["fill", "--rule-set", &bad, &design],
    ] {
        let out = ovcd(&args);
        assert!(!out.status.success(), "{args:?}");
        assert!(
            stderr(&out).contains("rule id grammar"),
            "{args:?}: {}",
            stderr(&out)
        );
    }
}

#[test]
fn the_same_id_and_version_with_other_content_is_refused() {
    let mut v: Value = serde_json::from_str(&std::fs::read_to_string(example()).unwrap()).unwrap();
    v["rules"][0]["limit"]["value"] = json!(0.06);
    let changed = temp_file("changed", &v);
    let out = ovcd(&["rules", "--rule-set", &example(), "--rule-set", &changed]);
    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("needs a new version"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn a_rule_set_changes_no_result_and_shows_in_the_trace() {
    let design = fixture("gas_igc_2g_exact.vdesign");
    let plain = ovcd(&["run", "--json", &design]);
    let with = ovcd(&["run", "--json", "--rule-set", &example(), &design]);
    assert!(plain.status.success() && with.status.success());
    let (p, w): (Value, Value) = (
        serde_json::from_str(&stdout(&plain)).unwrap(),
        serde_json::from_str(&stdout(&with)).unwrap(),
    );
    let (po, wo) = (&p["options"][0], &w["options"][0]);
    assert_eq!(po["results_hash"], wo["results_hash"]);
    assert_eq!(wo["hash_status"], "matches");
    let mut pr = po["results"].clone();
    let mut wr = wo["results"].clone();
    pr.as_object_mut().unwrap().remove("rule_trace");
    wr.as_object_mut().unwrap().remove("rule_trace");
    assert_eq!(pr, wr);
    let trace = &wo["results"]["rule_trace"];
    // The default stack of a gas Option, then the loaded set.
    assert_eq!(
        trace["rule_sets"],
        json!(["is2008-a@1.0.0", "example@1.0.0"])
    );
    let not_evaluated = trace["verdict"]["blocking_not_evaluated"]
        .as_array()
        .unwrap();
    assert!(
        not_evaluated
            .iter()
            .any(|r| r["rule"] == "example/1.3/weather")
    );
}

#[test]
fn rules_measures_prints_the_registry() {
    let out = ovcd(&["rules", "--measures"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    for id in [
        "gz.area",
        "gz.max_from",
        "gz.angle_of_max",
        "gm0",
        "input.fill_ratio",
    ] {
        assert!(text.contains(id), "{id}");
    }
}

#[test]
fn rule_set_is_not_an_option_of_new_or_schema() {
    for cmd in ["new", "schema"] {
        let out = ovcd(&[cmd, "--rule-set", &example()]);
        assert!(!out.status.success(), "{cmd}");
        assert!(
            stderr(&out).contains("unknown option `--rule-set`"),
            "{cmd}"
        );
    }
}
