//! Seam 2: `ovcd run`, end to end through the real binary: the summary, `--json`, the results hash
//! and its warning, and the convoy message (issue #25).

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

fn temp_dir(test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ovcd-run-{}-{test}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn p(path: &Path) -> &str {
    path.to_str().unwrap()
}

fn write_json(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
}

fn read_json(path: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// A starter Design written by `ovcd new`.
fn starter(dir: &Path) -> PathBuf {
    let file = dir.join("starter.vdesign");
    let out = ovcd(&["new", p(&file)]);
    assert!(out.status.success(), "{}", stderr(&out));
    file
}

/// Write a Design with one Option, `ovcd fill` the left-out inputs in, and return the filled file.
fn filled(dir: &Path, name: &str, inputs: Value) -> PathBuf {
    let raw = dir.join(format!("{name}-raw.vdesign"));
    write_json(
        &raw,
        &json!({
            "format": "vdesign",
            "format_version": 2,
            "engine_version": "0.1.0",
            "options": [{ "name": "Option 1", "inputs": inputs }]
        }),
    );
    let out = dir.join(format!("{name}.vdesign"));
    let r = ovcd(&["fill", p(&raw), p(&out)]);
    assert!(r.status.success(), "{}", stderr(&r));
    out
}

#[test]
fn the_summary_names_the_option_and_its_main_numbers() {
    let dir = temp_dir("summary");
    let out = ovcd(&["run", p(&starter(&dir))]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    for needle in [
        "Option 'Option 1': Bauxite",
        "Cargo: ",
        "Lightship",
        "Draught",
        "Checks: all",
        "Results hash",
        "(matches the file)",
    ] {
        assert!(text.contains(needle), "missing {needle:?} in:\n{text}");
    }
    assert!(!text.trim_start().starts_with('{'), "summary is not JSON");
}

#[test]
fn the_summary_keeps_the_case_of_the_vessel_label() {
    // Issue #142. The Python label (frozen at ce96e66, vessel_designer.simulate) is
    // "Box-hold dry bulk barge"; the summary used to lower-case it.
    let dir = temp_dir("label-case");
    let out = ovcd(&["run", p(&starter(&dir))]);
    let v: Value = serde_json::from_str(&stdout(&ovcd(&[
        "run",
        "--json",
        p(&dir.join("starter.vdesign")),
    ])))
    .unwrap();
    let label = v["options"][0]["results"]["vessel"].as_str().unwrap();
    assert!(label.chars().next().unwrap().is_uppercase(), "{label}");
    assert!(
        stdout(&out).contains(&format!(" in a {label} (")),
        "{}",
        stdout(&out)
    );
}

#[test]
fn running_the_same_file_twice_gives_the_same_hash() {
    let dir = temp_dir("twice");
    let file = starter(&dir);
    let a = ovcd(&["run", "--json", p(&file)]);
    let b = ovcd(&["run", "--json", p(&file)]);
    assert!(a.status.success() && b.status.success());
    assert_eq!(stdout(&a), stdout(&b));
    let first: Value = serde_json::from_str(&stdout(&a)).unwrap();
    let hash = first["options"][0]["results_hash"].as_str().unwrap();
    assert_eq!(hash.len(), 64);
    assert!(
        hash.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    );
    // The hash `ovcd new` saved is the one run computes.
    assert_eq!(read_json(&file)["options"][0]["results_hash"], hash);
    assert_eq!(first["options"][0]["hash_status"], "matches");
}

#[test]
fn run_does_not_change_the_file() {
    let dir = temp_dir("readonly");
    let file = starter(&dir);
    let before = std::fs::read_to_string(&file).unwrap();
    assert!(ovcd(&["run", p(&file)]).status.success());
    assert_eq!(std::fs::read_to_string(&file).unwrap(), before);
}

#[test]
fn a_tampered_stored_hash_warns_that_results_differ_from_the_saved_hash() {
    let dir = temp_dir("tamper");
    let file = starter(&dir);
    let mut design = read_json(&file);
    design["engine_version"] = json!("0.0.7");
    design["options"][0]["results_hash"] = json!("00".repeat(32));
    write_json(&file, &design);
    let out = ovcd(&["run", p(&file)]);
    assert!(out.status.success(), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(
        err.contains(
            "results differ from the hash saved in the file (saved with engine version 0.0.7"
        ),
        "{err}"
    );
    assert!(
        stdout(&out).contains("DIFFERS from the file"),
        "{}",
        stdout(&out)
    );
    // The results themselves are still printed.
    assert!(stdout(&out).contains("Cargo: "));
    let json_out = ovcd(&["run", "--json", p(&file)]);
    let v: Value = serde_json::from_str(&stdout(&json_out)).unwrap();
    assert_eq!(v["options"][0]["hash_status"], "differs");
    assert!(stderr(&json_out).contains("results differ from the hash saved in the file"));
}

#[test]
fn an_edited_input_with_the_old_hash_warns_too() {
    let dir = temp_dir("edited");
    let file = starter(&dir);
    let mut design = read_json(&file);
    design["options"][0]["inputs"]["beam_m"]["value"] = json!(11.0);
    write_json(&file, &design);
    let out = ovcd(&["run", p(&file)]);
    assert!(
        stderr(&out).contains("results differ from the hash saved in the file"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn a_file_with_no_stored_hash_runs_and_says_so() {
    let dir = temp_dir("nohash");
    let file = starter(&dir);
    let mut design = read_json(&file);
    design["options"][0]
        .as_object_mut()
        .unwrap()
        .remove("results_hash");
    write_json(&file, &design);
    let out = ovcd(&["run", p(&file)]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("none saved in the file"));
    assert!(!stderr(&out).contains("results differ"));
}

/// The starter Design with its one Option turned into a pushed convoy, hash removed.
fn convoy_design(dir: &Path) -> (PathBuf, Value) {
    let file = starter(dir);
    let mut design = read_json(&file);
    design["options"][0]["inputs"]["formation"] = json!({ "value": { "kind": "pushed_convoy", "abreast": 2, "in_line": 2 }, "source": "typed" });
    design["options"][0]
        .as_object_mut()
        .unwrap()
        .remove("results_hash");
    write_json(&file, &design);
    (file, design)
}

#[test]
fn a_convoy_gives_the_not_yet_available_message_and_no_numbers() {
    // Issue #138: a valid file whose only gap is a convoy exits 0; the message is a warning too.
    let dir = temp_dir("convoy");
    let (file, _) = convoy_design(&dir);
    let out = ovcd(&["run", p(&file)]);
    assert!(
        out.status.success(),
        "a convoy alone is not a failure: {}",
        stderr(&out)
    );
    let text = stdout(&out);
    assert!(text.contains("convoy results not yet available"), "{text}");
    for number_line in ["Cargo:", "Lightship", "Draught", "Results hash"] {
        assert!(!text.contains(number_line), "{number_line} in:\n{text}");
    }
    assert!(
        stderr(&out).contains("warning: Option 'Option 1': convoy results not yet available"),
        "{}",
        stderr(&out)
    );
    let j = ovcd(&["run", "--json", p(&file)]);
    assert!(j.status.success());
    let v: Value = serde_json::from_str(&stdout(&j)).unwrap();
    assert_eq!(v["options"][0]["results"], Value::Null);
    assert_eq!(v["options"][0]["status"], "not_yet_supported");
    assert_eq!(
        v["options"][0]["message"],
        "convoy results not yet available"
    );
    assert!(v["options"][0].get("results_hash").is_none());
}

/// Issue #64: a sloped-hopper Design runs as the sloped-hopper barge, in the summary and in
/// `--json`, and its hold is not the box barge's (it has a hold bottom narrower than the top).
#[test]
fn a_sloped_hopper_design_runs_as_the_sloped_hopper_barge() {
    let dir = temp_dir("hopper");
    let file = filled(
        &dir,
        "hopper",
        json!({
            "cargo": "bauxite",
            "hull_form": "raked_box",
            "formation": { "kind": "pushed_barge" },
            "length_overall_m": 76.5,
            "beam_m": 11.4,
            "moulded_depth_m": 3.5,
            "cargo_containment": "sloped_hopper_hold",
            "block_coefficient": 0.85
        }),
    );
    let out = ovcd(&["run", p(&file)]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stdout(&out).contains("Option 'Option 1': Bauxite in a Sloped-hopper dry bulk barge ("),
        "{}",
        stdout(&out)
    );
    let j = ovcd(&["run", "--json", p(&file)]);
    let v: Value = serde_json::from_str(&stdout(&j)).unwrap();
    let g = &v["options"][0]["results"]["geometry"];
    assert!(g["hold_bottom_width_m"].as_f64().unwrap() < g["hold_top_width_m"].as_f64().unwrap());
}

#[test]
fn json_tells_results_and_not_yet_supported_apart() {
    let dir = temp_dir("status");
    let file = starter(&dir);
    let mut design = read_json(&file);
    let mut convoy = design["options"][0].clone();
    convoy["name"] = json!("Convoy");
    convoy["inputs"]["formation"] = json!({ "value": { "kind": "pushed_convoy", "abreast": 2, "in_line": 1 }, "source": "typed" });
    convoy.as_object_mut().unwrap().remove("results_hash");
    design["options"].as_array_mut().unwrap().push(convoy);
    write_json(&file, &design);
    let v: Value = serde_json::from_str(&stdout(&ovcd(&["run", "--json", p(&file)]))).unwrap();
    assert_eq!(v["options"][0]["status"], "results");
    assert_eq!(v["options"][1]["status"], "not_yet_supported");
}

#[test]
fn a_two_option_design_runs_the_barge_and_reports_the_convoy_beside_it() {
    let dir = temp_dir("mixed");
    let file = starter(&dir);
    let mut design = read_json(&file);
    let mut convoy = design["options"][0].clone();
    convoy["name"] = json!("Convoy");
    convoy["inputs"]["formation"] = json!({ "value": { "kind": "pushed_convoy", "abreast": 2, "in_line": 1 }, "source": "typed" });
    convoy.as_object_mut().unwrap().remove("results_hash");
    design["options"].as_array_mut().unwrap().push(convoy);
    write_json(&file, &design);
    let out = ovcd(&["run", p(&file)]);
    let text = stdout(&out);
    assert!(text.contains("Option 'Option 1': Bauxite"), "{text}");
    assert!(
        text.contains("Option 'Convoy': convoy results not yet available"),
        "{text}"
    );
    assert!(
        out.status.success(),
        "convoy beside a computed Option exits 0"
    );
}

#[test]
fn a_hash_stored_on_a_convoy_option_gets_a_note_that_it_is_not_checked() {
    let dir = temp_dir("convoy-hash");
    let (file, mut design) = convoy_design(&dir);
    design["options"][0]["results_hash"] = json!("ab".repeat(32));
    write_json(&file, &design);
    let out = ovcd(&["run", p(&file)]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("Option 'Option 1': the stored results hash is not checked"),
        "{}",
        stderr(&out)
    );
    let v: Value = serde_json::from_str(&stdout(&ovcd(&["run", "--json", p(&file)]))).unwrap();
    assert_eq!(v["options"][0]["hash_status"], "not_checked");
}

#[test]
fn a_file_with_a_hard_error_is_not_run() {
    let dir = temp_dir("invalid");
    let file = starter(&dir);
    let mut design = read_json(&file);
    design["options"][0]["inputs"]["beam_m"]["value"] = json!(-3.0);
    write_json(&file, &design);
    let out = ovcd(&["run", p(&file)]);
    assert!(!out.status.success());
    assert!(stdout(&out).is_empty(), "{}", stdout(&out));
    assert!(stderr(&out).contains("error:"), "{}", stderr(&out));
}

#[test]
fn a_missing_file_fails_without_a_panic() {
    let out = ovcd(&["run", "/no/such/file.vdesign"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("cannot read"), "{}", stderr(&out));
}

#[test]
fn run_needs_exactly_one_file() {
    assert_eq!(ovcd(&["run"]).status.code(), Some(2));
    assert_eq!(ovcd(&["run", "a", "b"]).status.code(), Some(2));
}

/// What `simulate` answers for these params, through the library's own entry point.
fn simulate_answer(params: Value) -> Value {
    let case = json!({ "fn": "simulate", "args": { "params": params } });
    ovcd_format::eval("simulate", &case).expect("simulate runs")["ok"].clone()
}

/// `a` and `b` are equal, numbers within 1e-9 (relative and absolute).
fn assert_close(a: f64, b: f64, what: &str) {
    assert!(
        (a - b).abs() <= 1e-9_f64.max(1e-9 * b.abs()),
        "{what}: {a} vs {b}"
    );
}

#[test]
fn json_results_are_what_simulate_answers_for_a_pushed_barge() {
    let dir = temp_dir("simulate-box");
    // A calibration-style box barge: 76.5 x 11.4 x 3.5 m, Cb 0.928, lightship factor 0.1093.
    let file = filled(
        &dir,
        "box",
        json!({
            "cargo": "bauxite",
            "hull_form": "raked_box",
            "formation": { "kind": "pushed_barge" },
            "length_overall_m": 76.5,
            "beam_m": 11.4,
            "moulded_depth_m": 3.5,
            "cargo_containment": "box_hold",
            "block_coefficient": 0.928,
            "lightship_factor_t_per_m3": 0.1093,
            "min_freeboard_m": 0.3
        }),
    );
    let out = ovcd(&["run", "--json", p(&file)]);
    assert!(out.status.success(), "{}", stderr(&out));
    let v: Value = serde_json::from_str(&stdout(&out)).unwrap();
    // `run` adds the capital budget (`cost`, null for dry bulk) to what `simulate` answers; the rest is the same.
    let mut results = v["options"][0]["results"].clone();
    assert!(results["cost"].is_null());
    results.as_object_mut().unwrap().remove("cost");
    let results = &results;
    let expected = simulate_answer(json!({
        "vessel": "box_barge", "cargo": "bauxite", "loa_m": 76.5, "beam_m": 11.4, "depth_m": 3.5,
        "cb": 0.928, "lightship_k_t_per_m3": 0.1093, "freeboard_min_m": 0.3,
        "self_propelled": false, "eta_d": 0.45, "speed_max_kn": 10.0
    }));
    assert_eq!(results, &expected);
    // Pinned values from the original Python.
    let r = &results["results"];
    assert_close(r["cargo_t"].as_f64().unwrap(), 2256.166305, "cargo_t");
    assert_close(
        r["lightship_t"].as_f64().unwrap(),
        333.62185500000004,
        "lightship_t",
    );
    assert_close(r["draught_m"].as_f64().unwrap(), 3.2, "draught_m");
}

#[test]
fn json_results_are_what_simulate_answers_for_a_self_propelled_gas_hull() {
    let dir = temp_dir("simulate-gas");
    // A 90 x 14.8 x 6.5 m motor gas barge, the CN II benchmark.
    let file = filled(
        &dir,
        "gas",
        json!({
            "cargo": "LPG_propane",
            "hull_form": "raked_box",
            "formation": { "kind": "self_propelled" },
            "length_overall_m": 90.0,
            "beam_m": 14.8,
            "moulded_depth_m": 6.5,
            "cargo_containment": "type_c_tanks",
            "block_coefficient": 0.85,
            "min_freeboard_m": 0.3,
            "ruleset": "IGC_2G",
            "fill_ratio": 0.95,
            "max_tank_od_m": 16.0,
            "optimise_depth": false,
            "propulsive_efficiency": 0.45,
            "max_speed_kn": 10.0,
            "draught_limit_m": 4.4
        }),
    );
    let out = ovcd(&["run", "--json", p(&file)]);
    assert!(out.status.success(), "{}", stderr(&out));
    let v: Value = serde_json::from_str(&stdout(&out)).unwrap();
    // `run` adds the capital budget (`cost`) to what `simulate` answers; the rest is the same.
    let mut results = v["options"][0]["results"].clone();
    assert!(results["cost"]["unit_total"].is_number());
    assert!(results["cost"]["unit_total"].is_number());
    results.as_object_mut().unwrap().remove("cost");
    let results = &results;
    let expected = simulate_answer(json!({
        "vessel": "gas_barge", "cargo": "LPG_propane", "loa_m": 90.0, "beam_m": 14.8,
        "depth_m": 6.5, "cb": 0.85, "freeboard_min_m": 0.3, "draught_limit_m": 4.4,
        "self_propelled": true, "ruleset": "IGC_2G", "fill_ratio": 0.95, "max_tank_od_m": 16.0,
        "optimise_depth": false, "eta_d": 0.45, "speed_max_kn": 10.0
    }));
    assert_eq!(results, &expected);
    // Pinned values: the original Python, re-pinned (issue #237) to what the current code prints;
    // were 2058.228164213275 and 1813.9027562650056 under the stepped IGC_2G.
    let r = &results["results"];
    assert_close(
        r["cargo_t"].as_f64().unwrap(),
        2943.4407266060916,
        "cargo_t",
    );
    assert_close(
        r["lightship_t"].as_f64().unwrap(),
        2038.2392733939087,
        "lightship_t",
    );
    // Re-pinned (issue #237); was "tank volume".
    assert_eq!(r["governs"], "draught — Draught limit");
    // The summary of a gas Option lists the failed stability checks, if any, by name.
    let s = ovcd(&["run", p(&file)]);
    assert!(stdout(&s).contains("Checks: "), "{}", stdout(&s));
}

#[test]
fn fill_saves_a_hash_for_a_computable_option() {
    let dir = temp_dir("fill-hash");
    let file = filled(
        &dir,
        "f",
        json!({
            "cargo": "bauxite",
            "hull_form": "raked_box",
            "formation": { "kind": "pushed_barge" },
            "length_overall_m": 76.5,
            "beam_m": 11.4,
            "moulded_depth_m": 3.5
        }),
    );
    let design = read_json(&file);
    assert!(design["options"][0]["results_hash"].is_string());
    let out = ovcd(&["run", p(&file)]);
    assert!(
        stdout(&out).contains("(matches the file)"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn fill_names_this_engine_in_the_file_it_writes_a_hash_into() {
    // Issue #137: the file said 0.1.0 and kept saying it, beside a hash that 0.1.0 never made.
    let dir = temp_dir("fill-engine");
    let file = filled(
        &dir,
        "g",
        json!({
            "cargo": "bauxite", "hull_form": "raked_box", "formation": {"kind": "pushed_barge"},
            "length_overall_m": 76.5, "beam_m": 11.4, "moulded_depth_m": 3.5
        }),
    );
    let design = read_json(&file);
    assert_eq!(design["engine_version"], env!("CARGO_PKG_VERSION"));
    assert!(design["options"][0]["results_hash"].is_string());
    // An edit of an input, hash kept: the warning does not blame an engine version.
    let mut edited = design.clone();
    edited["options"][0]["inputs"]["beam_m"]["value"] = json!(11.0);
    write_json(&file, &edited);
    let err = stderr(&ovcd(&["run", p(&file)]));
    assert!(
        err.contains("results differ from the hash saved in the file"),
        "{err}"
    );
}
