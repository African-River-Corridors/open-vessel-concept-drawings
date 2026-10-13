//! Seam 2: the `ovcd` binary's design commands, end to end: `new`, `validate` and `profile`.
//! Each test runs the real binary on real files in a temporary folder.

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

/// A fresh, empty folder for one test.
fn temp_dir(test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ovcd-test-{}-{test}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn path_str(p: &Path) -> &str {
    p.to_str().unwrap()
}

fn write_json(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
}

fn read_json(path: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// `ovcd new` into a file, then return the parsed Design.
fn new_design(dir: &Path) -> (PathBuf, Value) {
    let file = dir.join("starter.vdesign");
    let out = ovcd(&["new", path_str(&file)]);
    assert!(out.status.success(), "ovcd new: {}", stderr(&out));
    let design = read_json(&file);
    (file, design)
}

/// A minimal Design: one Option holding only the given typed inputs.
fn design_with(inputs: Value) -> Value {
    json!({
        "format": "vdesign",
        "format_version": 1,
        "engine_version": "0.1.0",
        "options": [{ "name": "A", "inputs": inputs }]
    })
}

/// The six required inputs of a box-hold dry-bulk pushed barge, typed.
fn required_inputs() -> Value {
    json!({
        "cargo": "bauxite",
        "hull_form": "raked_box",
        "length_overall_m": 76.5,
        "beam_m": 11.4,
        "moulded_depth_m": 3.5,
        "formation": { "kind": "pushed_barge" }
    })
}

fn validate_inputs(test: &str, inputs: Value) -> Output {
    let dir = temp_dir(test);
    let file = dir.join("d.vdesign");
    write_json(&file, &design_with(inputs));
    ovcd(&["validate", path_str(&file)])
}

#[test]
fn new_output_passes_validate_with_no_errors() {
    let dir = temp_dir("new-validates");
    let (file, _) = new_design(&dir);
    let out = ovcd(&["validate", path_str(&file)]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("0 error(s)"), "{text}");
    assert!(!text.contains("error:"), "{text}");
}

#[test]
fn new_writes_one_option_with_engine_version_and_si_inputs() {
    let dir = temp_dir("new-shape");
    let (_, design) = new_design(&dir);
    assert_eq!(design["format"], "vdesign");
    assert_eq!(design["format_version"], 1);
    assert_eq!(design["engine_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(design["units"], "SI");
    let options = design["options"].as_array().unwrap();
    assert_eq!(options.len(), 1);
    let inputs = options[0]["inputs"].as_object().unwrap();
    for name in [
        "cargo",
        "hull_form",
        "length_overall_m",
        "beam_m",
        "moulded_depth_m",
        "formation",
        "cargo_containment",
        "block_coefficient",
        "min_freeboard_m",
        "propulsive_efficiency",
        "max_speed_kn",
    ] {
        assert!(inputs.contains_key(name), "missing {name}: {inputs:?}");
    }
}

#[test]
fn new_does_not_overwrite_an_existing_file() {
    let dir = temp_dir("new-no-overwrite");
    let file = dir.join("keep.vdesign");
    std::fs::write(&file, "mine").unwrap();
    let out = ovcd(&["new", path_str(&file)]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("already exists"), "{}", stderr(&out));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "mine");
}

#[test]
fn defaulted_inputs_are_marked_and_distinguishable_from_typed_ones() {
    let dir = temp_dir("markers");
    let (_, design) = new_design(&dir);
    let inputs = &design["options"][0]["inputs"];
    // The required inputs are the designer's own numbers: typed.
    for name in ["cargo", "length_overall_m", "beam_m", "moulded_depth_m"] {
        assert_eq!(inputs[name]["source"], "typed", "{name}: {}", inputs[name]);
    }
    // Everything else came from the built-in Defaults Profile, at its stated values.
    assert_eq!(
        inputs["block_coefficient"],
        json!({ "value": 0.85, "source": "default" })
    );
    assert_eq!(
        inputs["min_freeboard_m"],
        json!({ "value": 0.3, "source": "default" })
    );
    assert_eq!(
        inputs["propulsive_efficiency"],
        json!({ "value": 0.45, "source": "default" })
    );
    assert_eq!(
        inputs["max_speed_kn"],
        json!({ "value": 10.0, "source": "default" })
    );
    assert_eq!(
        inputs["channel_depth_m"],
        json!({ "value": null, "source": "default" })
    );

    // `validate --inputs` shows the same split to a person.
    let file = dir.join("starter.vdesign");
    let out = ovcd(&["validate", "--inputs", path_str(&file)]);
    let text = stdout(&out);
    assert!(text.contains("beam_m = 11.4 (typed)"), "{text}");
    assert!(
        text.contains("block_coefficient = 0.85 (default)"),
        "{text}"
    );
}

#[test]
fn missing_required_input_gives_the_stated_message() {
    let mut inputs = required_inputs();
    inputs.as_object_mut().unwrap().remove("beam_m");
    let out = validate_inputs("missing-required", inputs);
    assert!(!out.status.success());
    let text = stdout(&out);
    assert!(
        text.contains("error: Option 'A': missing required input `beam_m`"),
        "{text}"
    );
}

#[test]
fn missing_depth_says_a_default_rule_is_planned() {
    let mut inputs = required_inputs();
    inputs.as_object_mut().unwrap().remove("moulded_depth_m");
    let out = validate_inputs("missing-depth", inputs);
    assert!(!out.status.success());
    let text = stdout(&out);
    assert!(
        text.contains(
            "error: Option 'A': missing required input `moulded_depth_m`: moulded depth is \
             required for now; a default rule by Cargo is planned"
        ),
        "{text}"
    );
}

#[test]
fn hard_range_breach_is_refused_with_the_stated_message() {
    let mut inputs = required_inputs();
    inputs["beam_m"] = json!(-2.0);
    let out = validate_inputs("hard-breach", inputs);
    assert!(!out.status.success());
    let text = stdout(&out);
    assert!(
        text.contains(
            "error: Option 'A': `beam_m` = -2 is out of range: must be >= 1 (hard limit)"
        ),
        "{text}"
    );
}

#[test]
fn block_coefficient_above_one_is_a_hard_error() {
    let mut inputs = required_inputs();
    inputs["block_coefficient"] = json!(1.2);
    let out = validate_inputs("cb-hard", inputs);
    assert!(!out.status.success());
    let text = stdout(&out);
    assert!(
        text.contains("`block_coefficient` = 1.2 is out of range: must be <= 1 (hard limit)"),
        "{text}"
    );
}

#[test]
fn soft_range_breach_is_accepted_with_the_stated_warning() {
    let mut inputs = required_inputs();
    inputs["length_overall_m"] = json!(300.0);
    let out = validate_inputs("soft-breach", inputs);
    assert!(out.status.success(), "{}", stdout(&out));
    let text = stdout(&out);
    assert!(
        text.contains(
            "warning: Option 'A': `length_overall_m` = 300 m is outside the usual range 20 to \
             250 m (soft limit); accepted"
        ),
        "{text}"
    );
    assert!(text.contains("0 error(s)"), "{text}");
}

#[test]
fn a_pushed_convoy_is_valid_in_the_file() {
    let mut inputs = required_inputs();
    inputs["formation"] = json!({ "kind": "pushed_convoy", "abreast": 3, "in_line": 5 });
    let out = validate_inputs("convoy", inputs);
    assert!(out.status.success(), "{}", stdout(&out));
    assert!(stdout(&out).contains("0 error(s)"), "{}", stdout(&out));
}

#[test]
fn editing_the_defaults_profile_does_not_change_a_saved_files_inputs() {
    let dir = temp_dir("profile-edit");
    let (file, saved) = new_design(&dir);
    let before = ovcd(&["validate", "--inputs", path_str(&file)]);
    assert!(before.status.success());

    // A user copies the built-in profile and changes the block coefficient default.
    let out = ovcd(&["profile"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let mut profile: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(profile["inputs"]["block_coefficient"]["default"], 0.85);
    profile["inputs"]["block_coefficient"]["default"] = json!(0.9);
    let edited = dir.join("edited.profile.json");
    write_json(&edited, &profile);

    // The saved file still holds, and validates with, the inputs it was saved with.
    let after = ovcd(&[
        "validate",
        "--inputs",
        "--profile",
        path_str(&edited),
        path_str(&file),
    ]);
    assert!(after.status.success(), "{}", stdout(&after));
    assert_eq!(stdout(&before), stdout(&after));
    assert!(stdout(&after).contains("block_coefficient = 0.85 (default)"));
    assert_eq!(read_json(&file), saved);

    // A new Design made with the edited profile does take the new default.
    let fresh = dir.join("fresh.vdesign");
    let out = ovcd(&["new", "--profile", path_str(&edited), path_str(&fresh)]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        read_json(&fresh)["options"][0]["inputs"]["block_coefficient"]["value"],
        0.9
    );
}

#[test]
fn validate_fails_loudly_on_a_file_that_is_not_json() {
    let dir = temp_dir("not-json");
    let file = dir.join("bad.vdesign");
    std::fs::write(&file, "not json").unwrap();
    let out = ovcd(&["validate", path_str(&file)]);
    assert!(!out.status.success());
    assert!(stdout(&out).contains("error:"), "{}", stdout(&out));
}

#[test]
fn help_lists_new_and_validate() {
    let out = ovcd(&["--help"]);
    let help = stdout(&out);
    assert!(help.contains("ovcd new"), "{help}");
    assert!(help.contains("ovcd validate"), "{help}");
}

#[test]
fn a_profile_with_a_gap_in_a_keyed_default_is_refused_by_validate_and_new() {
    let dir = temp_dir("profile-gap");
    let out = ovcd(&["profile"]);
    let mut profile: Value = serde_json::from_slice(&out.stdout).unwrap();
    profile["inputs"]["cargo_containment"]["default_by_cargo_phase"]
        .as_object_mut()
        .unwrap()
        .remove("liquid_gas");
    let gap = dir.join("gap.profile.json");
    write_json(&gap, &profile);
    let gas = dir.join("gas.vdesign");
    write_json(
        &gas,
        &design_with(json!({
            "cargo": "LNG",
            "hull_form": "raked_box",
            "length_overall_m": 76.5,
            "beam_m": 11.4,
            "moulded_depth_m": 3.5,
            "formation": { "kind": "self_propelled" }
        })),
    );
    let message = "no default for `cargo_containment` when cargo_phase is liquid_gas";
    let out = ovcd(&["validate", "--profile", path_str(&gap), path_str(&gas)]);
    assert!(!out.status.success(), "{}", stdout(&out));
    assert!(stderr(&out).contains(message), "{}", stderr(&out));
    let out = ovcd(&["new", "--profile", path_str(&gap)]);
    assert!(!out.status.success(), "{}", stdout(&out));
    assert!(stderr(&out).contains(message), "{}", stderr(&out));
}

#[test]
fn new_records_the_built_in_profile_name() {
    let dir = temp_dir("new-profile-name");
    let (_, design) = new_design(&dir);
    assert_eq!(design["defaults_profile"], "built-in");
}

#[test]
fn validate_inputs_shows_a_convoy_and_a_self_propelled_formation() {
    let dir = temp_dir("inputs-formation");
    let file = dir.join("d.vdesign");
    let mut convoy = required_inputs();
    convoy["formation"] = json!({ "kind": "pushed_convoy", "abreast": 2, "in_line": 3 });
    let mut solo = required_inputs();
    solo["formation"] = json!({ "kind": "self_propelled" });
    write_json(
        &file,
        &json!({
            "format": "vdesign",
            "format_version": 1,
            "engine_version": "0.1.0",
            "options": [
                { "name": "Convoy", "inputs": convoy },
                { "name": "Solo", "inputs": solo }
            ]
        }),
    );
    let out = ovcd(&["validate", "--inputs", path_str(&file)]);
    let text = stdout(&out);
    assert!(out.status.success(), "{text}");
    assert!(
        text.contains("formation = pushed_convoy (2 abreast, 3 in line) (typed)"),
        "{text}"
    );
    assert!(
        text.contains("formation = self_propelled (typed)"),
        "{text}"
    );
}

// Validation is the only guard: each hard cross-check refuses the breach and passes its boundary.

fn assert_zero_errors(test: &str, inputs: Value) {
    let out = validate_inputs(test, inputs);
    let text = stdout(&out);
    assert!(out.status.success(), "{text}{}", stderr(&out));
    assert!(text.contains("0 error(s)"), "{text}");
}

#[test]
fn freeboard_at_depth_is_a_hard_error_and_just_below_passes() {
    let mut inputs = required_inputs();
    inputs["min_freeboard_m"] = json!(3.5);
    let out = validate_inputs("fb-breach", inputs);
    assert!(!out.status.success());
    let text = stdout(&out);
    assert!(
        text.contains(
            "error: Option 'A': `min_freeboard_m` = 3.5 must be less than `moulded_depth_m` = 3.5 (hard limit)"
        ),
        "{text}"
    );
    let mut inputs = required_inputs();
    inputs["min_freeboard_m"] = json!(3.49);
    assert_zero_errors("fb-boundary", inputs);
}

#[test]
fn channel_shallower_than_its_clearance_is_a_hard_error_for_every_cargo() {
    for cargo in ["bauxite", "LNG"] {
        let mut inputs = required_inputs();
        inputs["cargo"] = json!(cargo);
        inputs["channel_depth_m"] = json!(0.4);
        inputs["channel_under_keel_clearance_m"] = json!(0.5);
        let out = validate_inputs(&format!("chan-breach-{cargo}"), inputs);
        assert!(!out.status.success(), "{cargo}");
        let text = stdout(&out);
        assert!(
            text.contains(
                "error: Option 'A': `channel_depth_m` = 0.4 must be at least \
                 `channel_under_keel_clearance_m` = 0.5 (hard limit)"
            ),
            "{text}"
        );
    }
}

#[test]
fn channel_depth_equal_to_its_clearance_passes() {
    for cargo in ["bauxite", "LNG"] {
        let mut inputs = required_inputs();
        inputs["cargo"] = json!(cargo);
        inputs["channel_depth_m"] = json!(0.5);
        inputs["channel_under_keel_clearance_m"] = json!(0.5);
        assert_zero_errors(&format!("chan-boundary-{cargo}"), inputs);
    }
}

#[test]
fn channel_clearance_is_checked_against_its_default_when_not_typed() {
    let mut inputs = required_inputs();
    inputs["channel_depth_m"] = json!(0.3);
    let out = validate_inputs("chan-default-ukc", inputs);
    assert!(!out.status.success());
    assert!(
        stdout(&out).contains("must be at least `channel_under_keel_clearance_m` = 0.5"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn block_coefficient_below_point_three_is_a_hard_error_and_point_three_passes() {
    let mut inputs = required_inputs();
    inputs["block_coefficient"] = json!(0.29);
    let out = validate_inputs("cb-floor", inputs);
    assert!(!out.status.success());
    let text = stdout(&out);
    assert!(
        text.contains("`block_coefficient` = 0.29 is out of range: must be >= 0.3 (hard limit)"),
        "{text}"
    );
    let mut inputs = required_inputs();
    inputs["block_coefficient"] = json!(0.3);
    inputs["channel_bottom_width_m"] = json!(60.0);
    let out = validate_inputs("cb-floor-boundary", inputs);
    let text = stdout(&out);
    assert!(out.status.success(), "{text}");
    assert!(text.contains("0 error(s)"), "{text}");
}

#[test]
fn a_channel_one_ulp_shallower_than_its_clearance_is_refused() {
    // The file's number must be read exactly: 1.9999999999999998 is not 2.
    let mut inputs = required_inputs();
    inputs["channel_depth_m"] = json!(1.999_999_999_999_999_8_f64);
    inputs["channel_under_keel_clearance_m"] = json!(2.0);
    let out = validate_inputs("chan-ulp", inputs);
    assert!(!out.status.success(), "{}", stdout(&out));
    assert!(
        stdout(&out).contains("`channel_depth_m` = 1.9999999999999998 must be at least"),
        "{}",
        stdout(&out)
    );
}

/// A gas Option that searches its depth, with a deep hull so freeboard alone is the question.
fn depth_search_inputs(min_freeboard_m: f64) -> Value {
    let mut inputs = required_inputs();
    inputs["cargo"] = json!("LNG");
    inputs["moulded_depth_m"] = json!(12.0);
    inputs["optimise_depth"] = json!(true);
    inputs["min_freeboard_m"] = json!(min_freeboard_m);
    inputs
}

#[test]
fn a_depth_search_with_freeboard_above_the_deepest_hull_is_a_hard_error() {
    let out = validate_inputs("depth-search-breach", depth_search_inputs(7.5));
    assert!(!out.status.success());
    let text = stdout(&out);
    assert!(
        text.contains(
            "error: Option 'A': `min_freeboard_m` = 7.5 must be at most 7, the deepest hull \
             `optimise_depth` searches (hard limit)"
        ),
        "{text}"
    );
}

#[test]
fn a_depth_search_with_freeboard_equal_to_the_deepest_hull_passes() {
    assert_zero_errors("depth-search-boundary", depth_search_inputs(7.0));
}

#[test]
fn a_huge_length_is_a_hard_error() {
    let mut inputs = required_inputs();
    inputs["length_overall_m"] = json!(1e308);
    let out = validate_inputs("huge-length", inputs);
    assert!(!out.status.success());
    assert!(
        stdout(&out).contains("is out of range: must be <= 1000 (hard limit)"),
        "{}",
        stdout(&out)
    );
}

/// The built-in profile with one input's hard range replaced (removed for `null`), in a file.
fn profile_with_hard(dir: &Path, name: &str, hard: Value) -> PathBuf {
    let mut profile: Value = serde_json::from_slice(&ovcd(&["profile"]).stdout).unwrap();
    if hard.is_null() {
        profile["inputs"][name]
            .as_object_mut()
            .unwrap()
            .remove("hard");
    } else {
        profile["inputs"][name]["hard"] = hard;
    }
    let path = dir.join("edited.profile.json");
    write_json(&path, &profile);
    path
}

#[test]
fn a_profile_may_not_loosen_a_hard_range() {
    let dir = temp_dir("profile-loosen");
    let design = dir.join("d.vdesign");
    write_json(&design, &design_with(required_inputs()));
    for hard in [
        json!({ "gt": 0, "le": 1 }),
        json!({ "ge": 0.3 }),
        json!(null),
    ] {
        let profile = profile_with_hard(&dir, "block_coefficient", hard.clone());
        let out = ovcd(&[
            "validate",
            "--profile",
            path_str(&profile),
            path_str(&design),
        ]);
        assert!(!out.status.success(), "{hard}: {}", stdout(&out));
        assert!(
            stderr(&out).contains(
                "profile loosens the hard range of `block_coefficient`; it must be >= 0.3 \
                 and <= 1 at least (a profile may tighten a hard range, never loosen it)"
            ),
            "{hard}: {}",
            stderr(&out)
        );
    }
}

#[test]
fn a_profile_may_tighten_a_hard_range() {
    let dir = temp_dir("profile-tighten");
    let design = dir.join("d.vdesign");
    write_json(&design, &design_with(required_inputs()));
    for hard in [json!({ "gt": 0.3, "le": 1 }), json!({ "ge": 0.5, "lt": 1 })] {
        let profile = profile_with_hard(&dir, "block_coefficient", hard.clone());
        let out = ovcd(&[
            "validate",
            "--profile",
            path_str(&profile),
            path_str(&design),
        ]);
        assert!(
            out.status.success(),
            "{hard}: {}{}",
            stdout(&out),
            stderr(&out)
        );
    }
}
