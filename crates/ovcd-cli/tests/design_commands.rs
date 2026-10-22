//! Seam 2: the `ovcd` binary's design commands, end to end: `new`, `validate` and `profile`.
//! Each test runs the real binary on real files in a temporary folder.
//! Label: property
//! Each test states a rule of a command (an exit code, a message, a value kept exactly); the one long number is a refusal at one ulp, not a result.

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
        "format_version": 2,
        "engine_version": "0.1.0",
        "options": [{ "name": "A", "inputs": inputs }]
    })
}

/// The inputs of a box-hold dry-bulk pushed barge, typed (depth too, though it has a default).
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

/// Write a Design holding `inputs`, `ovcd fill` in the left-out inputs as a designer would, and
/// validate the filled file. When fill refuses (a hard error), validate the file as written, so
/// the test still sees every error.
fn validate_inputs(test: &str, inputs: Value) -> Output {
    let dir = temp_dir(test);
    let file = dir.join("d.vdesign");
    write_json(&file, &design_with(inputs));
    let filled = dir.join("filled.vdesign");
    if ovcd(&["fill", path_str(&file), path_str(&filled)])
        .status
        .success()
    {
        ovcd(&["validate", path_str(&filled)])
    } else {
        ovcd(&["validate", path_str(&file)])
    }
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
    assert_eq!(design["format_version"], 3);
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
    // Issue #446: the bow rake length's default is none (the bow rake over depth rule).
    assert_eq!(
        inputs["bow_rake_length_m"],
        json!({ "value": null, "source": "default" })
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
    assert!(text.contains("max_speed_kn = 10 (default)"), "{text}");
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
fn a_left_out_depth_is_the_cargo_ratio_times_the_beam_marked_default() {
    // Issue #120: depth is no longer required; `fill` writes the rule's value, marked default.
    let dir = temp_dir("missing-depth");
    let file = dir.join("d.vdesign");
    let mut inputs = required_inputs(); // bauxite, beam 11.4
    inputs.as_object_mut().unwrap().remove("moulded_depth_m");
    write_json(&file, &design_with(inputs));
    let filled = dir.join("f.vdesign");
    let out = ovcd(&["fill", path_str(&file), path_str(&filled)]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("`moulded_depth_m` was not stored in the file; wrote in the Defaults Profile value 3.762, marked default"),
        "{}",
        stderr(&out)
    );
    let design = read_json(&filled);
    assert_eq!(
        design["options"][0]["inputs"]["moulded_depth_m"],
        json!({ "value": 3.762, "source": "default" })
    );
    // `validate` of the unfilled file says it is not stored, like any other default.
    let v = ovcd(&["validate", path_str(&file)]);
    assert!(!v.status.success());
    assert!(
        stdout(&v).contains("inputs not stored in the file: "),
        "{}",
        stdout(&v)
    );
    assert!(stdout(&v).contains("moulded_depth_m"));
}

#[test]
fn a_null_depth_is_refused_a_left_out_depth_is_not() {
    // Null is not "left out": the designer wrote it. (Left out, the rule gives the value.)
    let mut inputs = required_inputs();
    inputs["moulded_depth_m"] = Value::Null;
    let out = validate_inputs("null-depth", inputs);
    assert!(!out.status.success());
    let text = stdout(&out);
    assert!(
        text.contains("`moulded_depth_m` must be a number"),
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
fn a_water_density_above_its_hard_maximum_is_a_hard_error() {
    let mut inputs = required_inputs();
    inputs["water_density_t_per_m3"] = json!(1.2);
    let out = validate_inputs("rho-hard", inputs);
    assert!(!out.status.success());
    let text = stdout(&out);
    assert!(
        text.contains(
            "`water_density_t_per_m3` = 1.2 is out of range: must be <= 1.1 (hard limit)"
        ),
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

    // A user copies the built-in profile and changes the maximum speed default.
    let out = ovcd(&["profile"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let mut profile: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(profile["inputs"]["max_speed_kn"]["default"], 10.0);
    profile["inputs"]["max_speed_kn"]["default"] = json!(12.0);
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
    assert!(
        stdout(&after).contains("max_speed_kn = 10 (default)"),
        "{}",
        stdout(&after)
    );
    assert_eq!(read_json(&file), saved);

    // A new Design made with the edited profile does take the new default.
    let fresh = dir.join("fresh.vdesign");
    let out = ovcd(&["new", "--profile", path_str(&edited), path_str(&fresh)]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        read_json(&fresh)["options"][0]["inputs"]["max_speed_kn"]["value"],
        12.0
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
fn help_lists_new_fill_and_validate() {
    let out = ovcd(&["--help"]);
    let help = stdout(&out);
    assert!(help.contains("ovcd fill"), "{help}");
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
fn the_help_and_profile_say_speeds_are_in_knots() {
    // Issue #134: the file is SI except speeds. `ovcd new` writes `max_speed_kn`.
    let help = stdout(&ovcd(&["--help"]));
    assert!(help.contains("except speeds, which are in knots"), "{help}");
    let profile = stdout(&ovcd(&["profile"]));
    assert!(profile.contains("except speeds, which are in knots"));
    let dir = temp_dir("knots");
    let (_, design) = new_design(&dir);
    assert!(design["options"][0]["inputs"]["max_speed_kn"].is_object());
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
            "format_version": 2,
            "engine_version": "0.1.0",
            "options": [
                { "name": "Convoy", "inputs": convoy },
                { "name": "Solo", "inputs": solo }
            ]
        }),
    );
    let filled = dir.join("filled.vdesign");
    let out = ovcd(&["fill", path_str(&file), path_str(&filled)]);
    assert!(out.status.success(), "{}", stderr(&out));
    let out = ovcd(&["validate", "--inputs", path_str(&filled)]);
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
fn a_water_density_below_its_hard_minimum_is_a_hard_error_and_the_minimum_passes() {
    let mut inputs = required_inputs();
    inputs["water_density_t_per_m3"] = json!(0.89);
    let out = validate_inputs("rho-floor", inputs);
    assert!(!out.status.success());
    let text = stdout(&out);
    assert!(
        text.contains(
            "`water_density_t_per_m3` = 0.89 is out of range: must be >= 0.9 (hard limit)"
        ),
        "{text}"
    );
    let mut inputs = required_inputs();
    inputs["water_density_t_per_m3"] = json!(0.9);
    inputs["channel_bottom_width_m"] = json!(60.0);
    let out = validate_inputs("rho-floor-boundary", inputs);
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
        json!({ "gt": 0, "le": 1.1 }),
        json!({ "ge": 0.9 }),
        json!(null),
    ] {
        let profile = profile_with_hard(&dir, "water_density_t_per_m3", hard.clone());
        let out = ovcd(&[
            "validate",
            "--profile",
            path_str(&profile),
            path_str(&design),
        ]);
        assert!(!out.status.success(), "{hard}: {}", stdout(&out));
        assert!(
            stderr(&out).contains(
                "profile loosens the hard range of `water_density_t_per_m3`; it must be >= 0.9 \
                 and <= 1.1 at least (a profile may tighten a hard range, never loosen it)"
            ),
            "{hard}: {}",
            stderr(&out)
        );
    }
}

#[test]
fn a_profile_may_tighten_a_hard_range() {
    let dir = temp_dir("profile-tighten");
    let (design, _) = new_design(&dir);
    for hard in [
        json!({ "gt": 0.9, "le": 1.1 }),
        json!({ "ge": 0.95, "lt": 1.1 }),
    ] {
        let profile = profile_with_hard(&dir, "water_density_t_per_m3", hard.clone());
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

// Inputs stored as used (issue #41): a hand-written file that leaves inputs out is refused until
// `ovcd fill` writes them in; after that, a profile edit never changes the file.

fn edited_profile(dir: &Path) -> PathBuf {
    let mut profile: Value = serde_json::from_slice(&ovcd(&["profile"]).stdout).unwrap();
    profile["inputs"]["water_density_t_per_m3"]["default"] = json!(1.02);
    profile["inputs"]["min_freeboard_m"]["default"] = json!(0.5);
    let path = dir.join("edited.profile.json");
    write_json(&path, &profile);
    path
}

#[test]
fn validate_refuses_a_hand_written_file_that_leaves_out_inputs() {
    let dir = temp_dir("left-out");
    let file = dir.join("hand.vdesign");
    write_json(&file, &design_with(required_inputs()));
    let out = ovcd(&["validate", path_str(&file)]);
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    let text = stdout(&out);
    assert!(
        text.contains("error: Option 'A': inputs not stored in the file: cargo_containment, "),
        "{text}"
    );
    assert!(text.contains("run `ovcd fill`"), "{text}");
}

#[test]
fn fill_writes_left_out_inputs_in_and_a_profile_edit_then_changes_nothing() {
    let dir = temp_dir("fill");
    let hand = dir.join("hand.vdesign");
    write_json(&hand, &design_with(required_inputs()));
    let filled = dir.join("filled.vdesign");
    let out = ovcd(&["fill", path_str(&hand), path_str(&filled)]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stderr(&out).contains(
            "`max_speed_kn` was not stored in the file; wrote in the Defaults Profile value \
             10, marked default"
        ),
        "{}",
        stderr(&out)
    );
    let design = read_json(&filled);
    assert_eq!(
        design["options"][0]["inputs"]["max_speed_kn"],
        json!({ "value": 10.0, "source": "default" })
    );
    assert_eq!(
        design["options"][0]["inputs"]["beam_m"],
        json!({ "value": 11.4, "source": "typed" })
    );
    assert_eq!(design["defaults_profile"], "built-in");

    let before = ovcd(&["validate", "--inputs", path_str(&filled)]);
    assert!(before.status.success(), "{}", stdout(&before));
    let edited = edited_profile(&dir);
    let after = ovcd(&[
        "validate",
        "--inputs",
        "--profile",
        path_str(&edited),
        path_str(&filled),
    ]);
    assert!(after.status.success(), "{}", stdout(&after));
    assert_eq!(stdout(&before), stdout(&after));
    assert!(stdout(&after).contains("min_freeboard_m = 0.3 (default)"));
    assert_eq!(read_json(&filled), design);
}

#[test]
fn fill_prints_to_stdout_without_an_output_file_and_never_overwrites() {
    let dir = temp_dir("fill-stdout");
    let hand = dir.join("hand.vdesign");
    write_json(&hand, &design_with(required_inputs()));
    let out = ovcd(&["fill", path_str(&hand)]);
    assert!(out.status.success(), "{}", stderr(&out));
    let design: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        design["options"][0]["inputs"]["max_speed_kn"]["source"],
        "default"
    );
    let out = ovcd(&["fill", path_str(&hand), path_str(&hand)]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("already exists"), "{}", stderr(&out));
    assert_eq!(read_json(&hand), design_with(required_inputs()));
}

#[test]
fn fill_refuses_a_file_with_a_hard_error_and_writes_nothing() {
    let dir = temp_dir("fill-refuses");
    let hand = dir.join("hand.vdesign");
    let mut inputs = required_inputs();
    inputs["beam_m"] = json!(1.0);
    write_json(&hand, &design_with(inputs));
    let filled = dir.join("filled.vdesign");
    let out = ovcd(&["fill", path_str(&hand), path_str(&filled)]);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("error: Option 'A': `beam_m` = 1 leaves a hold width of -1 m"),
        "{}",
        stderr(&out)
    );
    assert!(!filled.exists());
}

// A bad Defaults Profile is an error message and a non-zero exit, never a panic.

#[test]
fn new_with_a_profile_whose_defaults_break_a_cross_check_fails_without_a_panic() {
    let dir = temp_dir("bad-profile-new");
    let mut profile: Value = serde_json::from_slice(&ovcd(&["profile"]).stdout).unwrap();
    profile["inputs"]["min_freeboard_m"]["default"] = json!(4.0);
    let bad = dir.join("deep.profile.json");
    write_json(&bad, &profile);
    let file = dir.join("new.vdesign");
    let out = ovcd(&["new", "--profile", path_str(&bad), path_str(&file)]);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(!err.contains("panicked"), "{err}");
    assert!(
        err.contains("does not make a valid starter Design"),
        "{err}"
    );
    assert!(
        err.contains("`min_freeboard_m` = 4 must be less than `moulded_depth_m` = 3.5"),
        "{err}"
    );
    assert!(!file.exists());
}

#[test]
fn validate_refuses_a_one_metre_beam_box_barge() {
    let mut inputs = required_inputs();
    inputs["beam_m"] = json!(1.0);
    let out = validate_inputs("one-metre-beam", inputs);
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(
        stdout(&out).contains(
            "`beam_m` = 1 leaves a hold width of -1 m after the two 1 m wing voids; a negative \
             hold gives a negative cargo mass and draught (hard limit)"
        ),
        "{}",
        stdout(&out)
    );
}

#[test]
fn a_duplicate_key_fails_validate_run_and_fill_naming_the_key() {
    // Issue #139: a second "beam_m" used to win without a word (0 errors, a confident result).
    let dir = temp_dir("duplicate-key");
    let (file, _) = new_design(&dir);
    let text = std::fs::read_to_string(&file).unwrap();
    let doubled = text.replacen(
        "\"beam_m\": {",
        "\"beam_m\": {\"value\": 14.8, \"source\": \"typed\"},\n        \"beam_m\": {",
        1,
    );
    assert_ne!(doubled, text);
    let dup = dir.join("dup.vdesign");
    std::fs::write(&dup, doubled).unwrap();
    let out = ovcd(&["validate", path_str(&dup)]);
    assert!(!out.status.success(), "{}", stdout(&out));
    assert!(
        stdout(&out).contains("`beam_m` appears twice in options[0].inputs"),
        "{}",
        stdout(&out)
    );
    let out = ovcd(&["run", path_str(&dup)]);
    assert!(!out.status.success());
    assert!(stdout(&out).is_empty(), "no results from an ambiguous file");
    assert!(
        stderr(&out).contains("`beam_m` appears twice"),
        "{}",
        stderr(&out)
    );
    let out = ovcd(&["fill", path_str(&dup)]);
    assert!(!out.status.success());
    assert!(stdout(&out).is_empty());
}

// ---- version 1 files (issue #175) ---------------------------------------------------------------

/// The version 1 file that names `ADN_G` and sets `adn_tank_cap_m3`, as version 1 wrote it.
fn legacy_adn_file() -> String {
    format!(
        "{}/../ovcd-format/tests/legacy/v1_adn_g_self_propelled.vdesign",
        env!("CARGO_MANIFEST_DIR")
    )
}

#[test]
fn validate_opens_a_version_1_adn_g_file_as_igc_2g_and_warns() {
    let out = ovcd(&["validate", &legacy_adn_file()]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    let err = stdout(&out);
    assert!(
        err.contains("`ruleset` ADN_G is no longer offered"),
        "{err}"
    );
    assert!(err.contains("opened as IGC_2G"), "{err}");
    assert!(
        err.contains("`adn_tank_cap_m3` is no longer an input"),
        "{err}"
    );
    assert!(
        // Three for the version 1 changes, one for the dome diameter a version 1 file lacks (issue
        // #237), one for the hull end input a version 1 file lacks, one for the LPG carriage
        // condition the file lacks (issue #292: read as fully pressurised), one for the block
        // coefficient the file stores, now a result of the hull (issue #448).
        stdout(&out).contains("0 error(s), 7 warning(s)"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn fill_writes_a_version_1_adn_g_file_as_a_version_2_igc_2g_file_and_warns() {
    let dir = temp_dir("legacy-fill");
    let out_file = dir.join("out.vdesign");
    let out = ovcd(&["fill", &legacy_adn_file(), path_str(&out_file)]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    let err = stderr(&out);
    assert!(
        err.contains("`ruleset` ADN_G is no longer offered"),
        "{err}"
    );
    assert!(
        err.contains("`adn_tank_cap_m3` is no longer an input"),
        "{err}"
    );
    let design = read_json(&out_file);
    assert_eq!(design["format_version"], 3);
    let inputs = &design["options"][0]["inputs"];
    assert!(inputs.get("ruleset").is_none());
    assert!(inputs.get("adn_tank_cap_m3").is_none());
    // The written file needs no more help.
    let again = ovcd(&["validate", path_str(&out_file)]);
    assert!(again.status.success(), "{}", stderr(&again));
    assert!(
        stdout(&again).contains("0 error(s), 0 warning(s)"),
        "{}",
        stdout(&again)
    );
}

#[test]
fn run_runs_a_version_1_adn_g_file_as_igc_2g() {
    let out = ovcd(&["run", "--json", &legacy_adn_file()]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    assert!(stderr(&out).contains("`ruleset` ADN_G is no longer offered"));
    let v: Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(
        v["options"][0]["results"]["arrangement"]["ruleset"],
        "IGC_2G"
    );
}

#[test]
fn schema_has_no_adn_g_and_no_tank_cap_but_the_diameter_cap_with_its_basis() {
    let out = ovcd(&["schema"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(!text.contains("ADN_G"), "ADN_G is still in the schema");
    assert!(
        !text.contains("adn_tank_cap"),
        "adn_tank_cap_m3 is still in the schema"
    );
    let v: Value = serde_json::from_str(&text).unwrap();
    let od = v["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["name"] == "max_tank_od_m")
        .expect("max_tank_od_m is an input");
    assert_eq!(od["default"], 16.0);
    assert!(!od["basis"].as_str().unwrap().is_empty());
}

// ---- Vessel Class (issue #94) ---------------------------------------------------------------

#[test]
fn new_class_writes_an_option_defined_by_the_class_and_it_validates() {
    let dir = temp_dir("new-class");
    let file = dir.join("c.vdesign");
    let out = ovcd(&["new", "--class", "CEMT-VIa", path_str(&file)]);
    assert!(out.status.success(), "{}", stderr(&out));
    let design = read_json(&file);
    assert_eq!(design["format_version"], 3);
    let inputs = &design["options"][0]["inputs"];
    assert_eq!(
        inputs["vessel_class"],
        json!({ "value": "CEMT-VIa", "source": "class" })
    );
    assert_eq!(inputs["vessel_scheme"]["value"], "CEMT-1992");
    assert_eq!(
        inputs["beam_m"],
        json!({ "value": 22.8, "source": "class" })
    );
    assert_eq!(
        inputs["formation"],
        json!({ "value": { "kind": "pushed_convoy", "abreast": 2, "in_line": 1 }, "source": "class" })
    );
    assert_eq!(
        inputs["draught_limit_m"],
        json!({ "value": 4.5, "source": "class" })
    );
    // The starter's typed depth is dropped: the depth comes from the rule, 0.33 x 22.8 m.
    assert_eq!(
        inputs["moulded_depth_m"],
        json!({ "value": 7.524, "source": "default" })
    );
    let v = ovcd(&["validate", path_str(&file)]);
    assert!(v.status.success(), "{}{}", stdout(&v), stderr(&v));
    assert!(
        stdout(&v).contains("0 error(s), 0 warning(s)"),
        "{}",
        stdout(&v)
    );
}

#[test]
fn new_class_accepts_an_old_id_and_refuses_a_class_that_cannot_define_a_vessel() {
    let dir = temp_dir("new-class-bad");
    let ok = ovcd(&[
        "new",
        "--class",
        "CEMT_Va",
        path_str(&dir.join("a.vdesign")),
    ]);
    assert!(ok.status.success(), "{}", stderr(&ok));
    assert_eq!(
        read_json(&dir.join("a.vdesign"))["options"][0]["inputs"]["vessel_class"]["value"],
        "CEMT-Va"
    );
    for id in ["NO-SUCH-CLASS", "CN-V"] {
        let bad = ovcd(&["new", "--class", id, path_str(&dir.join("b.vdesign"))]);
        assert!(!bad.status.success(), "{id}");
        assert!(!dir.join("b.vdesign").exists());
    }
    let missing = ovcd(&["new", "--class"]);
    assert!(!missing.status.success());
}

#[test]
fn fill_writes_the_dimensions_of_a_class_a_hand_written_design_names() {
    let dir = temp_dir("fill-class");
    let file = dir.join("h.vdesign");
    let mut inputs = required_inputs();
    for k in ["length_overall_m", "beam_m", "formation"] {
        inputs.as_object_mut().unwrap().remove(k);
    }
    inputs["vessel_class"] = json!("CEMT-Va");
    write_json(&file, &design_with(inputs));
    let filled = dir.join("f.vdesign");
    let out = ovcd(&["fill", path_str(&file), path_str(&filled)]);
    assert!(out.status.success(), "{}", stderr(&out));
    let design = read_json(&filled);
    let i = &design["options"][0]["inputs"];
    assert_eq!(
        i["length_overall_m"],
        json!({ "value": 110.0, "source": "class" })
    );
    assert_eq!(i["vessel_class"]["value"], "CEMT-Va");
    let v = ovcd(&["validate", path_str(&filled)]);
    assert!(v.status.success(), "{}{}", stdout(&v), stderr(&v));
}

#[test]
fn schema_lists_the_vessel_class_inputs() {
    let out = ovcd(&["schema"]);
    let v: Value = serde_json::from_str(&stdout(&out)).unwrap();
    let names: Vec<&str> = v["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"vessel_scheme") && names.contains(&"vessel_class"));
    assert!(
        v["groups"]
            .as_array()
            .unwrap()
            .contains(&json!("Vessel Class"))
    );
}

// ---- the Type C tank defaults (issue #292) ----------------------------------------------------

#[test]
fn fill_keeps_the_old_tank_values_and_fill_as_new_takes_the_defaults_of_the_cargo() {
    let dir = temp_dir("fill-as-new");
    let raw = dir.join("raw.vdesign");
    write_json(
        &raw,
        &json!({
            "format": "vdesign", "format_version": 2, "engine_version": "0.0.0",
            "options": [{ "name": "o", "inputs": {
                "cargo": "LPG_propane", "hull_form": "raked_box",
                "formation": { "kind": "self_propelled" },
                "length_overall_m": 110.0, "beam_m": 11.4, "moulded_depth_m": 6.0,
                "cargo_containment": "type_c_tanks"
            }}]
        }),
    );
    let old = dir.join("old.vdesign");
    let out = ovcd(&["fill", path_str(&raw), path_str(&old)]);
    assert!(out.status.success(), "{}", stderr(&out));
    // An old file: the fixed values, and the note that it is read as fully pressurised.
    assert!(
        stderr(&out).contains("has no `lpg_carriage_condition`"),
        "{}",
        stderr(&out)
    );
    let i = read_json(&old)["options"][0]["inputs"].clone();
    assert_eq!(i["tank_design_pressure_mpa"]["value"], 0.5);
    assert_eq!(i["tank_wall_thickness_mm"]["value"], 25.0);
    let new = dir.join("new.vdesign");
    let out = ovcd(&["fill", "--as-new", path_str(&raw), path_str(&new)]);
    assert!(out.status.success(), "{}", stderr(&out));
    let i = read_json(&new)["options"][0]["inputs"].clone();
    assert_eq!(
        i["lpg_carriage_condition"],
        json!({ "value": "fully_pressurised", "source": "default" })
    );
    assert_eq!(i["tank_design_pressure_mpa"]["value"], 1.77);
    assert_eq!(i["tank_material"]["value"], "S690QL1");
    assert_eq!(i["tank_wall_thickness_mm"]["value"], Value::Null);
    assert_eq!(i["fill_ratio"]["value"], 0.98);
    // The filled file is a valid Design and runs.
    let v = ovcd(&["validate", path_str(&new)]);
    assert!(v.status.success(), "{}{}", stdout(&v), stderr(&v));
    let r = ovcd(&["run", "--json", path_str(&new)]);
    assert!(r.status.success(), "{}", stderr(&r));
    let results: Value = serde_json::from_str(&stdout(&r)).unwrap();
    let tank = &results["options"][0]["results"]["drawing"]["tank"];
    assert_eq!(tank["wall_thickness_source"], "formula");
    assert_eq!(tank["material"], "S690QL1");
}

#[test]
fn the_schema_gives_the_carriage_condition_and_the_default_tables_with_their_status() {
    let out = ovcd(&["schema"]);
    let v: Value = serde_json::from_str(&stdout(&out)).unwrap();
    let get = |name: &str| {
        v["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["name"] == name)
            .unwrap_or_else(|| panic!("{name}"))
            .clone()
    };
    let carriage = get("lpg_carriage_condition");
    assert_eq!(
        carriage["applies_to_cargo"],
        json!(["LPG_propane", "LPG_butane", "LPG_mix"])
    );
    let pressure = get("tank_design_pressure_mpa");
    let rows = pressure["default_table"]["rows"].as_array().unwrap();
    assert!(
        rows.iter()
            .all(|r| r["status"] == "sourced" || r["status"] == "estimate")
    );
    assert!(rows.iter().any(|r| r["status"] == "estimate"));
    assert_eq!(get("tank_wall_thickness_mm")["default"], Value::Null);
    assert_eq!(get("tank_wall_thickness_mm")["nullable"], true);
    assert!(
        v["group_notes"]["Gas tanks"]
            .as_str()
            .unwrap()
            .contains("outside of the insulation")
    );
}
