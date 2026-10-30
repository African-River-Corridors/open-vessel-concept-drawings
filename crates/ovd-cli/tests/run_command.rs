//! Seam 2: `ovd run`, end to end through the real binary: the summary, `--json`, the results hash
//! and its warning, and the convoy message (issue #25).
//! Label: snapshot
//! The pinned numbers are what the code printed: the original Python, then each re-pinning named at its value.

use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn ovd(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ovd"))
        .args(args)
        .output()
        .expect("ovd runs")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn temp_dir(test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ovd-run-{}-{test}", std::process::id()));
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

/// A starter Design written by `ovd new`.
fn starter(dir: &Path) -> PathBuf {
    let file = dir.join("starter.vdesign");
    let out = ovd(&["new", p(&file)]);
    assert!(out.status.success(), "{}", stderr(&out));
    file
}

/// Write a Design with one Option, `ovd fill` the left-out inputs in, and return the filled file.
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
    let r = ovd(&["fill", p(&raw), p(&out)]);
    assert!(r.status.success(), "{}", stderr(&r));
    out
}

#[test]
fn the_summary_names_the_option_and_its_main_numbers() {
    let dir = temp_dir("summary");
    let out = ovd(&["run", p(&starter(&dir))]);
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
    // Issue #142. The Python label (frozen at ce96e66, ovd.simulate) is
    // "Box-hold dry bulk barge"; the summary used to lower-case it.
    let dir = temp_dir("label-case");
    let out = ovd(&["run", p(&starter(&dir))]);
    let v: Value = serde_json::from_str(&stdout(&ovd(&[
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
    let a = ovd(&["run", "--json", p(&file)]);
    let b = ovd(&["run", "--json", p(&file)]);
    assert!(a.status.success() && b.status.success());
    assert_eq!(stdout(&a), stdout(&b));
    let first: Value = serde_json::from_str(&stdout(&a)).unwrap();
    let hash = first["options"][0]["results_hash"].as_str().unwrap();
    assert_eq!(hash.len(), 64);
    assert!(
        hash.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    );
    // The hash `ovd new` saved is the one run computes.
    assert_eq!(read_json(&file)["options"][0]["results_hash"], hash);
    assert_eq!(first["options"][0]["hash_status"], "matches");
}

#[test]
fn run_does_not_change_the_file() {
    let dir = temp_dir("readonly");
    let file = starter(&dir);
    let before = std::fs::read_to_string(&file).unwrap();
    assert!(ovd(&["run", p(&file)]).status.success());
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
    let out = ovd(&["run", p(&file)]);
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
    let json_out = ovd(&["run", "--json", p(&file)]);
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
    let out = ovd(&["run", p(&file)]);
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
    let out = ovd(&["run", p(&file)]);
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
fn a_convoy_prints_its_totals_and_its_barge() {
    // Issue #358: a pushed convoy has results: the totals of every barge, then one barge's numbers.
    let dir = temp_dir("convoy");
    let (file, _) = convoy_design(&dir);
    let out = ovd(&["run", p(&file)]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains("Convoy: 2 abreast x 2 in line = 4 barges"),
        "{text}"
    );
    assert!(!text.contains("not yet available"), "{text}");
    assert!(text.contains("Results hash"), "{text}");
    let j = ovd(&["run", "--json", p(&file)]);
    assert!(j.status.success());
    let v: Value = serde_json::from_str(&stdout(&j)).unwrap();
    assert_eq!(v["options"][0]["status"], "results");
    let c = &v["options"][0]["results"]["convoy"];
    assert_eq!(c["barges"], 4);
    assert_eq!(
        c["totals"]["cargo_t"].as_f64().unwrap(),
        4.0 * c["barge"]["cargo_t"].as_f64().unwrap()
    );
}

#[test]
fn json_tells_a_barge_and_a_convoy_apart() {
    let dir = temp_dir("status");
    let file = starter(&dir);
    let mut design = read_json(&file);
    let mut convoy = design["options"][0].clone();
    convoy["name"] = json!("Convoy");
    convoy["inputs"]["formation"] = json!({ "value": { "kind": "pushed_convoy", "abreast": 2, "in_line": 1 }, "source": "typed" });
    convoy.as_object_mut().unwrap().remove("results_hash");
    design["options"].as_array_mut().unwrap().push(convoy);
    write_json(&file, &design);
    let v: Value = serde_json::from_str(&stdout(&ovd(&["run", "--json", p(&file)]))).unwrap();
    assert_eq!(v["options"][0]["status"], "results");
    assert_eq!(v["options"][1]["status"], "results");
    assert!(v["options"][0]["results"].get("convoy").is_none());
    assert_eq!(v["options"][1]["results"]["convoy"]["barges"], 2);
}

#[test]
fn a_hash_stored_on_a_convoy_option_is_checked() {
    let dir = temp_dir("convoy-hash");
    let (file, mut design) = convoy_design(&dir);
    design["options"][0]["results_hash"] = json!("ab".repeat(32));
    write_json(&file, &design);
    let out = ovd(&["run", p(&file)]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("results differ from the hash saved in the file"),
        "{}",
        stderr(&out)
    );
    let v: Value = serde_json::from_str(&stdout(&ovd(&["run", "--json", p(&file)]))).unwrap();
    assert_eq!(v["options"][0]["hash_status"], "differs");
}

#[test]
fn a_file_with_a_hard_error_is_not_run() {
    let dir = temp_dir("invalid");
    let file = starter(&dir);
    let mut design = read_json(&file);
    design["options"][0]["inputs"]["beam_m"]["value"] = json!(-3.0);
    write_json(&file, &design);
    let out = ovd(&["run", p(&file)]);
    assert!(!out.status.success());
    assert!(stdout(&out).is_empty(), "{}", stdout(&out));
    assert!(stderr(&out).contains("error:"), "{}", stderr(&out));
}

#[test]
fn a_missing_file_fails_without_a_panic() {
    let out = ovd(&["run", "/no/such/file.vdesign"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("cannot read"), "{}", stderr(&out));
}

#[test]
fn run_needs_exactly_one_file() {
    assert_eq!(ovd(&["run"]).status.code(), Some(2));
    assert_eq!(ovd(&["run", "a", "b"]).status.code(), Some(2));
}

/// What `simulate` answers for these params, through the library's own entry point.
fn simulate_answer(params: Value) -> Value {
    let case = json!({ "fn": "simulate", "args": { "params": params } });
    ovd_format::eval("simulate", &case).expect("simulate runs")["ok"].clone()
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
    let out = ovd(&["run", "--json", p(&file)]);
    assert!(out.status.success(), "{}", stderr(&out));
    let v: Value = serde_json::from_str(&stdout(&out)).unwrap();
    // `run` adds the capital budget (`cost`, null for dry bulk) to what `simulate` answers; the rest is the same.
    let mut results = v["options"][0]["results"].clone();
    assert!(results["cost"].is_null());
    results.as_object_mut().unwrap().remove("cost");
    // And the Rule Trace (issue #269), outside the results hash.
    assert!(results["rule_trace"].is_object());
    results.as_object_mut().unwrap().remove("rule_trace");
    // And the calibration label (issue #434), outside the results hash.
    assert!(results["calibration"].is_object());
    results.as_object_mut().unwrap().remove("calibration");
    let results = &results;
    let expected = simulate_answer(json!({
        "vessel": "box_barge", "cargo": "bauxite", "loa_m": 76.5, "beam_m": 11.4, "depth_m": 3.5,
        "cb": 0.928, "lightship_k_t_per_m3": 0.1093, "freeboard_min_m": 0.3,
        "self_propelled": false, "eta_d": 0.45, "speed_max_kn": 10.0
    }));
    assert_eq!(results, &expected);
    // Pinned values from the original Python.
    let r = &results["results"];
    // Re-pinned (#448); was 2256.166305. The hull's sections give the volume, so the stored
    // block coefficient 0.928 no longer reduces it (the file gets one warning for it).
    assert_close(
        r["cargo_t"].as_f64().unwrap(),
        2346.1989450000005,
        "cargo_t",
    );
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
    // The file is a version 2 file, read with the stack of the time, in which every light condition
    // keeps the angle of max GZ. `simulate` has no Rule Sets: the angle of max GZ alone in a light
    // condition with no ballast is a warning (issue #498). The IS Code and IGC Rule Sets of that
    // kind make `run` answer the same.
    let rules = |set: &str| {
        format!(
            "{}/../ovd-format/rules/{set}.json",
            env!("CARGO_MANIFEST_DIR")
        )
    };
    let (is_code, igc) = (rules("is2008-a/2.0.0"), rules("igc-2g/2.0.0"));
    // The stack differs from the one the hash was stored with, so the stored hash is dropped.
    let mut saved: Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
    saved["options"][0]
        .as_object_mut()
        .unwrap()
        .remove("results_hash");
    write_json(&file, &saved);
    let out = ovd(&[
        "run",
        "--json",
        "--rule-set",
        &is_code,
        "--rule-set",
        &igc,
        p(&file),
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    let v: Value = serde_json::from_str(&stdout(&out)).unwrap();
    // `run` adds the capital budget (`cost`) to what `simulate` answers; the rest is the same.
    let mut results = v["options"][0]["results"].clone();
    assert!(results["cost"]["unit_total"].is_number());
    assert!(results["cost"]["unit_total"].is_number());
    results.as_object_mut().unwrap().remove("cost");
    // And the Rule Trace (issue #269), outside the results hash.
    assert!(results["rule_trace"].is_object());
    results.as_object_mut().unwrap().remove("rule_trace");
    // And the calibration label (issue #434), outside the results hash.
    assert!(results["calibration"].is_object());
    results.as_object_mut().unwrap().remove("calibration");
    // And the tank Spaces table (issue #687), inside the results hash: one row per Type C tank.
    let spaces = results.as_object_mut().unwrap().remove("spaces").unwrap();
    assert!(spaces.as_array().is_some_and(|rows| !rows.is_empty()));
    // `drawing.tank` (issue #292) echoes the tank design values the file held; the plain `simulate`
    // call below is given none of them. The numbers are the same, so compare without it.
    let tank = results["drawing"].as_object_mut().unwrap().remove("tank");
    assert_eq!(tank.unwrap()["material"], "9Ni");
    let results = &results;
    let mut expected = simulate_answer(json!({
        "vessel": "gas_barge", "cargo": "LPG_propane", "loa_m": 90.0, "beam_m": 14.8,
        "depth_m": 6.5, "cb": 0.85, "freeboard_min_m": 0.3, "draught_limit_m": 4.4,
        "self_propelled": true, "ruleset": "IGC_2G", "fill_ratio": 0.95, "max_tank_od_m": 16.0,
        "optimise_depth": false, "eta_d": 0.45, "speed_max_kn": 10.0,
        "aft_end_space_m": 0.6, "fwd_end_space_m": 0.5,
        "opening_setback_basis": "tank_end"
    }));
    expected["drawing"].as_object_mut().unwrap().remove("tank");
    assert_eq!(results, &expected);
    // Pinned values: the original Python, re-pinned (issue #237) to what the current code prints;
    // were 2058.228164213275 and 1813.9027562650056 under the stepped IGC_2G. `cargo_t` re-pinned
    // again (issue #264, the search keeps only arrangements that pass stability); was
    // 2943.4407266060916, and lightship_t 2038.2392733939087. Re-pinned again (issue #277, the
    // wheelhouse sightline blocks the search); was 2832.6643735601965 and 1893.6869533634328.
    // Re-pinned again (issue #410, the covering above D is in the lightship; the one-wide
    // arrangement fails the light-condition angle of max GZ); was 2021.2487121368047 and
    // 1604.5823133024817. Re-pinned again (issue #498, the angle of max GZ alone in the light
    // condition with no ballast is a warning; the one-wide arrangement is feasible again): was
    // 1405.8516419707348 and 1509.0556682648712; the lightship is what the code prints now (the
    // stern and covering inputs of the issues since #410 moved it from 1604.5823133024817).
    let r = &results["results"];
    assert_close(
        r["cargo_t"].as_f64().unwrap(),
        2021.2487121368047,
        "cargo_t",
    );
    assert_close(
        r["lightship_t"].as_f64().unwrap(),
        1583.438359298561,
        "lightship_t",
    );
    // Re-pinned (issue #264); was "tank volume" (stepped IGC_2G) and "draught — Draught limit" (issue #237).
    assert_eq!(r["governs"], "tank volume");
    // The summary of a gas Option lists the failed stability checks, if any, by name.
    let s = ovd(&["run", p(&file)]);
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
    let out = ovd(&["run", p(&file)]);
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
    let err = stderr(&ovd(&["run", p(&file)]));
    assert!(
        err.contains("results differ from the hash saved in the file"),
        "{err}"
    );
}

/// Issue #274: the stored Rule Sets are the ones a run checks. A stored version the engine lacks
/// refuses the Option and names the set; a stored hash that differs from the built-in one is a
/// warning, and the run still computes.
#[test]
fn a_run_checks_the_stored_rule_sets_and_names_a_missing_or_changed_one() {
    let dir = temp_dir("stored-rule-sets");
    let file = starter(&dir);
    let design = read_json(&file);
    let sets = |version: &str, hash: &str| {
        json!([
            {"id": "ovd", "version": version, "hash": hash, "source": "typed"}
        ])
    };
    let built_in = design["options"][0]["inputs"]["rule_sets"]["value"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == "ovd")
        .unwrap()
        .clone();
    let version = built_in["version"].as_str().unwrap();
    let hash = built_in["hash"].as_str().unwrap();

    let mut missing = design.clone();
    missing["options"][0]["inputs"]["rule_sets"]["value"] = sets("99.0.0", hash);
    let file_missing = dir.join("missing.vdesign");
    write_json(&file_missing, &missing);
    let out = ovd(&["run", p(&file_missing)]);
    assert!(
        stdout(&out).contains("Rule Set ovd@99.0.0 is not in this engine"),
        "{}",
        stdout(&out)
    );

    let mut changed = design.clone();
    changed["options"][0]["inputs"]["rule_sets"]["value"] = sets(version, &"0".repeat(64));
    let file_changed = dir.join("changed.vdesign");
    write_json(&file_changed, &changed);
    let out = ovd(&["run", p(&file_changed)]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stdout(&out).contains("the rules differ from the file"),
        "{}",
        stdout(&out)
    );
    assert!(stdout(&out).contains("Results hash"), "{}", stdout(&out));
}

/// Issue #275: `fill` adds and removes Rule Sets, and `run` states the scope.
#[test]
fn fill_adds_and_removes_rule_sets_and_run_states_the_scope() {
    let dir = temp_dir("rule-set-flags");
    let file = starter(&dir);
    let stack = |path: &Path| -> Vec<(String, String, String)> {
        read_json(path)["options"][0]["inputs"]["rule_sets"]["value"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| {
                let t = |k: &str| e[k].as_str().unwrap().to_string();
                (t("id"), t("version"), t("source"))
            })
            .collect()
    };
    // Add an older version of a set in the stack: it replaces the default one, marked typed.
    let added = dir.join("added.vdesign");
    let out = ovd(&[
        "fill",
        "--add-rule-set",
        "is2008-a@1.0.0",
        p(&file),
        p(&added),
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stack(&added).contains(&("is2008-a".into(), "1.0.0".into(), "typed".into())));
    // `ovd` cannot be removed; a refusal writes nothing.
    let refused = dir.join("refused.vdesign");
    let out = ovd(&["fill", "--remove-rule-set", "ovd", p(&file), p(&refused)]);
    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("cannot be removed"),
        "{}",
        stderr(&out)
    );
    assert!(!refused.exists());
    // An unknown set is refused too.
    let out = ovd(&[
        "fill",
        "--add-rule-set",
        "nope@1.0.0",
        p(&file),
        p(&refused),
    ]);
    assert!(
        stderr(&out).contains("not in this engine"),
        "{}",
        stderr(&out)
    );
    // The run says its scope, in the summary and in `--json`.
    let out = ovd(&["run", p(&file)]);
    assert!(
        stdout(&out).contains("Scope: Service Area river-only; Rule Sets is2008-a@"),
        "{}",
        stdout(&out)
    );
    assert!(
        stdout(&out).contains("no statutory technical code was checked, only these Rule Sets"),
        "{}",
        stdout(&out)
    );
    let out = ovd(&["run", "--json", p(&file)]);
    let json: Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert!(
        json["options"][0]["results"]["rule_trace"]["scope"]
            .as_str()
            .unwrap()
            .starts_with("Service Area river-only;")
    );
}
