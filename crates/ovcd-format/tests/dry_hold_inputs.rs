//! Seam: the dry-bulk hold design values are inputs (issue #4, slice 1). The ten values were fixed
//! in the engine; they are now inputs whose defaults are those fixed values, so no design moves.
//!
//! Label: property. Every test states a rule: a default equals the old constant, an old file keeps
//! its results (its stored hash still matches), an input reaches the core, or the file refuses what
//! it should. No number here is a printed result.

use ovcd_core::barge_capacity::BargeCapacityInputs;
use ovcd_format::vdesign::{AppliesTo, INPUTS, LEGACY_FIXED_INPUTS};
use ovcd_format::{DesignSession, HashStatus, Profile, Source, check, fill};
use serde_json::{Value, json};

fn raw_fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// A fixture as a file written before these inputs existed: the ten keys taken out. Its stored
/// results hash is the one the old engine gave, so a match proves the results did not move.
fn fixture(name: &str) -> String {
    let mut file: Value = serde_json::from_str(&raw_fixture(name)).unwrap();
    for option in file["options"].as_array_mut().unwrap() {
        let inputs = option["inputs"].as_object_mut().unwrap();
        for key in NEW_INPUTS {
            inputs.remove(key);
        }
    }
    file.to_string()
}

const NEW_INPUTS: [&str; 10] = [
    "hold_wing_void_width_m",
    "hold_double_bottom_m",
    "hopper_slope_h_per_v",
    "hatch_coaming_height_m",
    "n_holds",
    "hold_fill_ratio",
    "inner_bottom_plate_mm",
    "hopper_plate_mm",
    "hold_bulkhead_plate_mm",
    "hatch_coaming_plate_mm",
];
/// The old fixed value of an input, read from the core's own defaults.
fn core_default(name: &str) -> f64 {
    let c = BargeCapacityInputs::new(70.0, 11.0, 4.0);
    match name {
        "hold_wing_void_width_m" => c.wing_void_width,
        "hold_double_bottom_m" => c.double_bottom_height_bulk,
        "hopper_slope_h_per_v" => c.hopper_slope_h_per_v,
        "hatch_coaming_height_m" => c.coaming_height,
        "n_holds" => c.n_holds as f64,
        "hold_fill_ratio" => c.hold_fill_ratio,
        "inner_bottom_plate_mm" => c.inner_bottom_t_mm,
        "hopper_plate_mm" => c.hopper_plate_t_mm,
        "hold_bulkhead_plate_mm" => c.bulkhead_t_mm,
        "hatch_coaming_plate_mm" => c.coaming_t_mm,
        other => panic!("no core default for {other}"),
    }
}

#[test]
fn each_default_equals_the_legacy_table_value_and_the_core_default() {
    let profile = Profile::built_in();
    assert!(LEGACY_FIXED_INPUTS.len() >= NEW_INPUTS.len());
    for name in NEW_INPUTS {
        let legacy = LEGACY_FIXED_INPUTS
            .iter()
            .find(|l| l.name == name)
            .unwrap_or_else(|| panic!("{name} is not in the legacy table"));
        let from_profile = profile.default(name).and_then(Value::as_f64).unwrap();
        // Issue #597: a new Option's inner bottom is 0.8 m, from built barges; the table keeps the
        // 1.2 m of the original engine for a file that stored none.
        if name == "hold_double_bottom_m" {
            assert_eq!(from_profile, 0.8, "{name}: the profile default");
            assert_ne!(
                from_profile, legacy.value,
                "{name}: a file saved before keeps 1.2 m"
            );
        } else {
            assert_eq!(from_profile, legacy.value, "{name}: profile vs table");
        }
        assert_eq!(legacy.value, core_default(name), "{name}: table vs core");
    }
}

#[test]
fn the_stored_dry_bulk_fixtures_hold_the_inputs_as_defaults() {
    for (file, want) in [
        ("dry_bulk_box_hold", 4usize),
        ("dry_bulk_sloped_hopper", 10),
    ] {
        let raw: Value = serde_json::from_str(&raw_fixture(file)).unwrap();
        let inputs = raw["options"][0]["inputs"].as_object().unwrap();
        let held = NEW_INPUTS
            .iter()
            .filter(|n| inputs.contains_key(**n))
            .count();
        assert_eq!(held, want, "{file}");
        for name in NEW_INPUTS.iter().filter(|n| inputs.contains_key(**n)) {
            assert_eq!(inputs[*name]["source"], "default", "{name}");
            assert_eq!(inputs[*name]["value"].as_f64(), Some(core_default(name)));
        }
    }
}

#[test]
fn the_inputs_are_in_the_table_in_the_dry_bulk_hold_group_and_apply_to_the_right_holds() {
    let hopper_only = [
        "hopper_slope_h_per_v",
        "n_holds",
        "inner_bottom_plate_mm",
        "hopper_plate_mm",
        "hold_bulkhead_plate_mm",
        "hatch_coaming_plate_mm",
    ];
    for name in NEW_INPUTS {
        let spec = INPUTS.iter().find(|s| s.name == name).unwrap();
        assert_eq!(spec.group.label(), "Dry-bulk hold", "{name}");
        assert!(!spec.required, "{name}");
        let used = spec.applies_to.containments();
        if hopper_only.contains(&name) {
            assert_eq!(used, ["sloped_hopper_hold"], "{name}");
        } else {
            assert_eq!(used, ["box_hold", "sloped_hopper_hold"], "{name}");
        }
        assert_eq!(
            spec.applies_to,
            LEGACY_FIXED_INPUTS
                .iter()
                .find(|l| l.name == name)
                .unwrap()
                .applies_to,
            "{name}: table scope"
        );
    }
    assert_eq!(AppliesTo::All.containments().len(), 4);
}

#[test]
fn a_dry_bulk_fixture_without_the_new_inputs_opens_with_one_note_and_the_same_results() {
    for (file, held) in [
        ("dry_bulk_box_hold", 4usize),
        ("dry_bulk_sloped_hopper", 10usize),
    ] {
        let text = fixture(file);
        let checked = check(&text, &Profile::built_in());
        assert_eq!(checked.report.errors, Vec::<String>::new(), "{file}");
        let notes: Vec<&String> = checked
            .report
            .warnings
            .iter()
            .filter(|w| w.contains("fixed value the engine used before it was an input"))
            .collect();
        assert_eq!(notes.len(), 1, "{file}: {:?}", checked.report.warnings);
        let design = checked.design.unwrap();
        let o = &design.options[0];
        let present: Vec<&str> = NEW_INPUTS
            .iter()
            .copied()
            .filter(|n| o.input(n).is_some())
            .collect();
        assert_eq!(present.len(), held, "{file}: {present:?}");
        for name in present {
            let stored = o.input(name).unwrap();
            assert_eq!(stored.source, Source::Default, "{name}");
            assert_eq!(stored.value.as_f64(), Some(core_default(name)), "{name}");
        }
        // The results hash the file stored still matches: nothing moved.
        let (_, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
        assert_eq!(opened.hash_status[0].1, HashStatus::Matches, "{file}");
    }
}

#[test]
fn a_version_1_dry_bulk_file_is_changed_the_same_way() {
    let path = format!(
        "{}/tests/legacy/v1_dry_bulk_box_hold.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).unwrap();
    let raw: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(raw["format_version"], 1);
    let checked = check(&text, &Profile::built_in());
    assert_eq!(checked.report.errors, Vec::<String>::new());
    assert!(
        checked
            .report
            .migration_notes
            .iter()
            .any(|n| n.contains("`hold_double_bottom_m`")),
        "{:?}",
        checked.report.migration_notes
    );
    let (_, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
}

#[test]
fn the_fixed_value_not_the_profile_value_is_what_an_old_file_gets() {
    // A profile that changes the default does not move a file that lacks the input.
    let mut profile: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    profile["inputs"]["hold_double_bottom_m"]["default"] = json!(1.5);
    let edited = Profile::from_json(&profile.to_string()).unwrap();
    let checked = check(&fixture("dry_bulk_box_hold"), &edited);
    let d = checked.design.unwrap();
    assert_eq!(
        d.options[0].input("hold_double_bottom_m").unwrap().value,
        json!(1.2)
    );
}

#[test]
fn a_value_in_the_file_is_kept_and_any_other_missing_input_is_still_refused() {
    let mut file: Value = serde_json::from_str(&fixture("dry_bulk_box_hold")).unwrap();
    file["options"][0]["inputs"]["hold_double_bottom_m"] =
        json!({ "value": 1.5, "source": "typed" });
    let checked = check(&file.to_string(), &Profile::built_in());
    let o = &checked.design.as_ref().unwrap().options[0];
    let kept = o.input("hold_double_bottom_m").unwrap();
    assert_eq!(
        (kept.value.clone(), kept.source),
        (json!(1.5), Source::Typed)
    );
    assert!(
        !checked
            .report
            .warnings
            .iter()
            .any(|w| w.contains("`hold_double_bottom_m`") && w.contains("fixed value"))
    );
    // Leave out an input that is not a new one: the old rule applies.
    file["options"][0]["inputs"]
        .as_object_mut()
        .unwrap()
        .remove("min_freeboard_m");
    let refused = check(&file.to_string(), &Profile::built_in());
    assert!(
        refused
            .report
            .errors
            .iter()
            .any(|e| e.contains("inputs not stored in the file") && e.contains("min_freeboard_m")),
        "{:?}",
        refused.report.errors
    );
}

#[test]
fn a_hopper_only_input_in_a_box_hold_file_is_ignored_with_a_warning() {
    let mut file: Value = serde_json::from_str(&fixture("dry_bulk_box_hold")).unwrap();
    file["options"][0]["inputs"]["n_holds"] = json!({ "value": 5, "source": "typed" });
    let checked = check(&file.to_string(), &Profile::built_in());
    assert!(
        checked.report.errors.is_empty(),
        "{:?}",
        checked.report.errors
    );
    assert!(
        checked
            .report
            .warnings
            .iter()
            .any(|w| w.contains("`n_holds` is not used for a box_hold")),
        "{:?}",
        checked.report.warnings
    );
}

#[test]
fn a_gas_file_gets_none_of_them() {
    let checked = check(&fixture("gas_igc_2g_exact"), &Profile::built_in());
    assert!(checked.report.errors.is_empty());
    let o = &checked.design.unwrap().options[0];
    for name in NEW_INPUTS {
        assert!(o.input(name).is_none(), "{name}");
    }
}

#[test]
fn a_profile_copy_without_the_new_entries_still_loads_with_the_built_in_ones() {
    let mut profile: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    for name in NEW_INPUTS {
        profile["inputs"].as_object_mut().unwrap().remove(name);
    }
    let old = Profile::from_json(&profile.to_string()).expect("an older copy loads");
    let built_in = Profile::built_in();
    for name in NEW_INPUTS {
        assert_eq!(old.default(name), built_in.default(name), "{name}");
        assert_eq!(old.hard(name), built_in.hard(name), "{name}");
    }
}

#[test]
fn fill_writes_the_new_inputs_into_a_file_that_lacks_them() {
    let filled = fill(&fixture("dry_bulk_sloped_hopper"), &Profile::built_in());
    assert!(filled.report.is_ok(), "{:?}", filled.report);
    let saved: Value = serde_json::from_str(&filled.design.unwrap().to_text()).unwrap();
    let inputs = &saved["options"][0]["inputs"];
    for name in NEW_INPUTS {
        assert_eq!(inputs[name]["source"], "default", "{name}");
    }
    let again = check(&saved.to_string(), &Profile::built_in());
    assert_eq!(again.report.warnings, Vec::<String>::new());
}

/// The cargo volume and lightship of the hopper fixture with one input set.
fn run_with(file: &str, name: &str, value: Value) -> Value {
    let mut f: Value = serde_json::from_str(&fixture(file)).unwrap();
    f["options"][0]["inputs"][name] = json!({ "value": value, "source": "typed" });
    let (session, _) = DesignSession::open(&f.to_string(), Profile::built_in()).unwrap();
    let e = session.evaluate();
    let outcome = e.options[0].outcome.as_ref().expect("it ran");
    match outcome {
        ovcd_format::RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

fn base(file: &str) -> Value {
    let (session, _) = DesignSession::open(&fixture(file), Profile::built_in()).unwrap();
    match session.evaluate().options[0].outcome.as_ref().unwrap() {
        ovcd_format::RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

fn at(v: &Value, a: &str, b: &str) -> f64 {
    v[a][b].as_f64().unwrap()
}

#[test]
fn every_input_reaches_the_core() {
    let base_h = base("dry_bulk_sloped_hopper");
    let vol = |v: &Value| at(v, "results", "cargo_volume_m3");
    let ls = |v: &Value| at(v, "results", "lightship_t");
    // Hold geometry: a bigger wing void or a deeper double bottom gives a smaller hold.
    for (name, bigger) in [
        ("hold_wing_void_width_m", 1.5),
        ("hold_double_bottom_m", 1.5),
    ] {
        let r = run_with("dry_bulk_sloped_hopper", name, json!(bigger));
        assert!(vol(&r) < vol(&base_h), "{name}");
    }
    // A taller coaming gives a larger hold; a flatter slope (larger H:V) a smaller bottom.
    let r = run_with(
        "dry_bulk_sloped_hopper",
        "hatch_coaming_height_m",
        json!(1.5),
    );
    assert!(vol(&r) > vol(&base_h));
    // Fill ratio scales cargo volume exactly: 0.5 / 0.95 of the default.
    let r = run_with("dry_bulk_sloped_hopper", "hold_fill_ratio", json!(0.5));
    let want = vol(&base_h) * 0.5 / 0.95;
    assert!(
        (vol(&r) - want).abs() <= 1e-9 * want,
        "{} vs {want}",
        vol(&r)
    );
    // Hopper steel: each of these moves the sloped-hopper lightship up.
    for (name, more) in [
        ("hopper_slope_h_per_v", 2.0),
        ("n_holds", 6.0),
        ("inner_bottom_plate_mm", 28.0),
        ("hopper_plate_mm", 28.0),
        ("hold_bulkhead_plate_mm", 22.0),
        ("hatch_coaming_plate_mm", 20.0),
    ] {
        let r = run_with("dry_bulk_sloped_hopper", name, json!(more));
        assert!(
            ls(&r) > ls(&base_h),
            "{name}: {} vs {}",
            ls(&r),
            ls(&base_h)
        );
    }
    // A box hold: geometry reaches the volume; the coaming also raises the air draft.
    let box_base = base("dry_bulk_box_hold");
    let r = run_with("dry_bulk_box_hold", "hold_double_bottom_m", json!(1.5));
    assert!(vol(&r) < vol(&box_base));
    let r = run_with("dry_bulk_box_hold", "hatch_coaming_height_m", json!(1.5));
    assert!(
        at(&r, "results", "air_draft_loaded_m") > at(&box_base, "results", "air_draft_loaded_m")
    );
}

#[test]
fn the_wing_void_and_double_bottom_decide_the_impossible_hold_check() {
    let mut f: Value = serde_json::from_str(&fixture("dry_bulk_box_hold")).unwrap();
    let inputs = f["options"][0]["inputs"].as_object_mut().unwrap();
    inputs.insert("beam_m".into(), json!({ "value": 3.0, "source": "typed" }));
    let narrow = check(&f.to_string(), &Profile::built_in());
    // 3 m beam, 1 m wing voids each side: a hold 1 m wide. Fine.
    assert!(
        narrow.report.errors.is_empty(),
        "{:?}",
        narrow.report.errors
    );
    f["options"][0]["inputs"]["hold_wing_void_width_m"] =
        json!({ "value": 2.0, "source": "typed" });
    let bad = check(&f.to_string(), &Profile::built_in());
    assert!(
        bad.report
            .errors
            .iter()
            .any(|e| e.contains("leaves a hold width of")),
        "{:?}",
        bad.report.errors
    );
}

#[test]
fn n_holds_must_be_a_whole_number_of_at_least_one() {
    for (v, why) in [(json!(2.5), "whole number"), (json!(0), "must be >= 1")] {
        let mut f: Value = serde_json::from_str(&fixture("dry_bulk_sloped_hopper")).unwrap();
        f["options"][0]["inputs"]["n_holds"] = json!({ "value": v, "source": "typed" });
        let checked = check(&f.to_string(), &Profile::built_in());
        assert!(
            checked.report.errors.iter().any(|e| e.contains(why)),
            "{why}: {:?}",
            checked.report.errors
        );
    }
}

#[test]
fn the_schema_lists_them_for_the_left_panel() {
    let schema = ovcd_format::session::input_schema(&Profile::built_in());
    assert!(
        schema["groups"]
            .as_array()
            .unwrap()
            .iter()
            .any(|g| g == "Dry-bulk hold")
    );
    for name in NEW_INPUTS {
        let entry = schema["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["name"] == name)
            .unwrap_or_else(|| panic!("{name} not in the schema"));
        assert_eq!(entry["group"], "Dry-bulk hold");
        assert_eq!(entry["kind"], "number");
        assert!(
            entry["unit"].is_string() && entry["basis"].is_string(),
            "{name}"
        );
        assert!(
            entry["hard"].is_object() && entry["soft"].is_object(),
            "{name}"
        );
        assert!(entry["default"].is_number(), "{name}");
    }
}
