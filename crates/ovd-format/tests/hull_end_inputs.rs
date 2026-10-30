//! Seam: the hull end shapes are inputs (issue #4, slice 2): the bow rake over depth, the stern
//! void of a pushed barge and the end bulkhead allowance of a dry-bulk hold. They were fixed in the
//! engine; the defaults are those fixed values, so no design moves.
//!
//! Label: property. Every test states a rule: a default equals the old constant, an old file keeps
//! its results (its stored hash still matches), an input reaches the core, or a session keeps the
//! rule for which Formation an input is used for. No number here is a printed result.

use ovd_core::barge_capacity::BargeCapacityInputs;
use ovd_core::gas_arrangement::FitOptions;
use ovd_format::vdesign::{INPUTS, LEGACY_FIXED_INPUTS, PUSHED, formation_includes};
use ovd_format::{DesignSession, HashStatus, Profile, RunOutcome, Source, check, fill};
use serde_json::{Value, json};

const NEW: [&str; 3] = [
    "bow_rake_length_to_depth_ratio",
    "pushed_stern_void_m",
    "bulk_end_bulkhead_m",
];

fn raw_fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// A fixture as a file written before these inputs existed: the three keys taken out. Its stored
/// results hash is the one the old engine gave, so a match proves the results did not move.
fn fixture(name: &str) -> String {
    let mut file: Value = serde_json::from_str(&raw_fixture(name)).unwrap();
    for option in file["options"].as_array_mut().unwrap() {
        let inputs = option["inputs"].as_object_mut().unwrap();
        for key in NEW {
            inputs.remove(key);
        }
    }
    file.to_string()
}

fn edited_profile(name: &str, value: f64) -> Profile {
    let mut profile: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    profile["inputs"][name]["default"] = json!(value);
    Profile::from_json(&profile.to_string()).unwrap()
}

fn core_default(name: &str) -> f64 {
    let c = BargeCapacityInputs::new(70.0, 11.0, 4.0);
    match name {
        "bow_rake_length_to_depth_ratio" => c.bow_rake_to_depth_ratio,
        "pushed_stern_void_m" => c.pushed_stern_void_m,
        "bulk_end_bulkhead_m" => c.bulk_end_bulkhead_m,
        other => panic!("{other}"),
    }
}

#[test]
fn each_default_equals_the_legacy_table_value_and_the_core_default() {
    let profile = Profile::built_in();
    let fit = FitOptions::default();
    for name in NEW {
        let legacy = LEGACY_FIXED_INPUTS
            .iter()
            .find(|l| l.name == name)
            .unwrap_or_else(|| panic!("{name} is not in the legacy table"));
        let from_profile = profile.default(name).and_then(Value::as_f64).unwrap();
        assert_eq!(from_profile, legacy.value, "{name}: profile vs table");
        assert_eq!(legacy.value, core_default(name), "{name}: table vs core");
    }
    // The gas search starts from the same values.
    assert_eq!(fit.bow_rake_to_depth_ratio, core_default(NEW[0]));
    assert_eq!(fit.pushed_stern_void_m, core_default(NEW[1]));
}

#[test]
fn the_inputs_apply_to_the_right_containments_and_formations() {
    let spec = |n: &str| INPUTS.iter().find(|s| s.name == n).unwrap();
    let rake = spec(NEW[0]);
    assert_eq!(rake.applies_to.containments().len(), 4);
    assert!(rake.formations.is_none());
    let void = spec(NEW[1]);
    assert_eq!(void.applies_to.containments().len(), 4);
    assert_eq!(void.formations, Some(PUSHED));
    assert!(formation_includes(void.formations, Some("pushed_barge")));
    assert!(formation_includes(void.formations, Some("pushed_convoy")));
    assert!(!formation_includes(void.formations, Some("self_propelled")));
    assert!(formation_includes(void.formations, None));
    let bulk = spec(NEW[2]);
    assert_eq!(
        bulk.applies_to.containments(),
        ["box_hold", "sloped_hopper_hold"]
    );
    assert!(bulk.formations.is_none());
    for (n, l) in NEW
        .iter()
        .zip(NEW.map(|n| LEGACY_FIXED_INPUTS.iter().find(|l| l.name == n).unwrap()))
    {
        assert_eq!(spec(n).applies_to, l.applies_to, "{n}");
        assert_eq!(spec(n).formations, l.formations, "{n}");
    }
}

#[test]
fn the_schema_says_which_formations_each_input_is_used_for_and_keeps_applies_to() {
    let schema = ovd_format::session::input_schema(&Profile::built_in());
    let entry = |n: &str| {
        schema["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["name"] == n)
            .unwrap()
            .clone()
    };
    assert_eq!(
        entry("pushed_stern_void_m")["applies_to_formation"],
        json!(["pushed_barge", "pushed_convoy", "atb", "towed_barge"])
    );
    assert_eq!(
        entry("pushed_stern_void_m")["applies_to"],
        json!([
            "box_hold",
            "sloped_hopper_hold",
            "type_c_tanks",
            "integral_tanks"
        ])
    );
    assert_eq!(
        entry("bow_rake_length_to_depth_ratio")["applies_to_formation"],
        json!([
            "self_propelled",
            "pushed_barge",
            "pushed_convoy",
            "atb",
            "towed_barge"
        ])
    );
    assert_eq!(entry("bulk_end_bulkhead_m")["group"], "Dry-bulk hold");
    assert_eq!(
        entry("pushed_stern_void_m")["group"],
        "Hull and containment"
    );
    // Every input has the new key, so a consumer may rely on it.
    for i in schema["inputs"].as_array().unwrap() {
        assert!(i["applies_to_formation"].is_array(), "{}", i["name"]);
    }
}

#[test]
fn files_without_the_three_inputs_open_with_one_note_and_the_same_results() {
    // (file, the inputs it holds out of the three)
    let cases: [(&str, &[&str]); 4] = [
        ("dry_bulk_box_hold", &NEW),
        ("dry_bulk_sloped_hopper", &NEW),
        ("gas_igc_2g_exact", &NEW[..2]),
        ("gas_igc_2g_self_propelled", &NEW[..1]),
    ];
    for (file, held) in cases {
        let text = fixture(file);
        let checked = check(&text, &Profile::built_in());
        assert_eq!(checked.report.errors, Vec::<String>::new(), "{file}");
        let design = checked.design.unwrap();
        let o = &design.options[0];
        for name in NEW {
            let got = o.input(name);
            assert_eq!(got.is_some(), held.contains(&name), "{file}: {name}");
            if let Some(stored) = got {
                assert_eq!(stored.source, Source::Default);
                assert_eq!(stored.value.as_f64(), Some(core_default(name)));
            }
        }
        let (_, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
        assert_eq!(opened.hash_status[0].1, HashStatus::Matches, "{file}");
        let notes = opened
            .report
            .warnings
            .iter()
            .filter(|w| w.contains("fixed value the engine used before it was an input"))
            .count();
        assert_eq!(notes, 1, "{file}: {:?}", opened.report.warnings);
    }
}

#[test]
fn the_stored_fixtures_hold_the_inputs_as_defaults() {
    for (file, want) in [
        ("dry_bulk_box_hold", 3usize),
        ("gas_igc_2g_exact", 2),
        ("gas_igc_2g_self_propelled", 1),
    ] {
        let raw: Value = serde_json::from_str(&raw_fixture(file)).unwrap();
        let inputs = raw["options"][0]["inputs"].as_object().unwrap();
        let held = NEW.iter().filter(|n| inputs.contains_key(**n)).count();
        assert_eq!(held, want, "{file}");
    }
}

#[test]
fn an_old_file_gets_the_fixed_value_but_a_session_edit_gets_the_profile_value() {
    let edited = edited_profile("pushed_stern_void_m", 4.5);
    // An old file: the fixed value, whatever the profile says.
    let checked = check(&fixture("dry_bulk_box_hold"), &edited);
    let d = checked.design.unwrap();
    assert_eq!(
        d.options[0].input("pushed_stern_void_m").unwrap().value,
        json!(3.0)
    );
    // A new Option made in a session under that profile: the profile value.
    let mut s = DesignSession::new_design(edited.clone()).unwrap();
    assert_eq!(
        s.design().options[0]
            .input("pushed_stern_void_m")
            .unwrap()
            .value,
        json!(4.5)
    );
    // A reset input: the profile value.
    s.set_input("Option 1", "pushed_stern_void_m", json!(2.0))
        .unwrap();
    s.reset_input("Option 1", "pushed_stern_void_m").unwrap();
    assert_eq!(
        s.design().options[0]
            .input("pushed_stern_void_m")
            .unwrap()
            .value,
        json!(4.5)
    );
}

#[test]
fn a_changed_containment_gives_the_profile_value_of_the_inputs_it_now_uses() {
    // The same rule for the dry-bulk hold inputs of slice 1.
    let edited = edited_profile("hopper_slope_h_per_v", 1.5);
    let mut s = DesignSession::new_design(edited).unwrap();
    s.set_input("Option 1", "cargo_containment", json!("sloped_hopper_hold"))
        .unwrap();
    assert_eq!(
        s.design().options[0]
            .input("hopper_slope_h_per_v")
            .unwrap()
            .value,
        json!(1.5)
    );
}

#[test]
fn the_pushed_void_follows_the_formation_in_a_session() {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    assert!(s.design().options[0].input("pushed_stern_void_m").is_some());
    s.set_input("Option 1", "pushed_stern_void_m", json!(5.0))
        .unwrap();
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    assert!(s.design().options[0].input("pushed_stern_void_m").is_none());
    let err = s
        .set_input("Option 1", "pushed_stern_void_m", json!(2.0))
        .unwrap_err();
    assert!(
        err.contains("not used for a self_propelled Formation"),
        "{err}"
    );
    // Back to a pushed barge: the typed value comes back.
    s.set_formation("Option 1", "pushed_barge", None, None)
        .unwrap();
    let back = s.design().options[0].input("pushed_stern_void_m").unwrap();
    assert_eq!(
        (back.value.clone(), back.source),
        (json!(5.0), Source::Typed)
    );
}

#[test]
fn a_pushed_only_input_in_a_self_propelled_file_is_ignored_with_a_warning() {
    let mut f: Value = serde_json::from_str(&raw_fixture("gas_igc_2g_self_propelled")).unwrap();
    f["options"][0]["inputs"]["pushed_stern_void_m"] = json!({ "value": 4.0, "source": "typed" });
    let checked = check(&f.to_string(), &Profile::built_in());
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
            .any(|w| w.contains("`pushed_stern_void_m` is not used for a self_propelled Formation")),
        "{:?}",
        checked.report.warnings
    );
}

#[test]
fn a_gas_or_box_file_without_a_bulk_input_is_unchanged_and_fill_writes_the_rest() {
    let filled = fill(&fixture("gas_igc_2g_exact"), &Profile::built_in());
    assert!(filled.report.is_ok(), "{:?}", filled.report);
    let saved: Value = serde_json::from_str(&filled.design.unwrap().to_text()).unwrap();
    let inputs = &saved["options"][0]["inputs"];
    assert_eq!(
        inputs["bow_rake_length_to_depth_ratio"]["source"],
        "default"
    );
    assert_eq!(inputs["pushed_stern_void_m"]["source"], "default");
    assert!(inputs.get("bulk_end_bulkhead_m").is_none());
}

fn results(file: &str, name: &str, value: Value) -> Value {
    let mut f: Value = serde_json::from_str(&raw_fixture(file)).unwrap();
    f["options"][0]["inputs"][name] = json!({ "value": value, "source": "typed" });
    let (session, _) = DesignSession::open(&f.to_string(), Profile::built_in()).unwrap();
    match session.evaluate().options[0]
        .outcome
        .as_ref()
        .expect("it ran")
    {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

fn geo(v: &Value, key: &str) -> f64 {
    v["geometry"][key].as_f64().unwrap()
}

#[test]
fn every_input_reaches_the_core() {
    // Rake: the bow rake is the ratio times the moulded depth, on a dry and a gas hull.
    for (file, depth_key) in [
        ("dry_bulk_sloped_hopper", "depth_m"),
        ("gas_igc_2g_self_propelled", "depth_m"),
    ] {
        let r = results(file, "bow_rake_length_to_depth_ratio", json!(2.0));
        let depth = r["dims"][depth_key].as_f64().unwrap();
        assert_eq!(geo(&r, "bow_rake_length_m"), 2.0 * depth, "{file}");
    }
    // The pushed stern void is the void behind a pushed barge, less the separation it adds: the
    // stern void length is the void plus the aft end allowance (dry: the end bulkhead, 0.5 m).
    let base = results("dry_bulk_sloped_hopper", "pushed_stern_void_m", json!(3.0));
    let more = results("dry_bulk_sloped_hopper", "pushed_stern_void_m", json!(5.0));
    assert_eq!(
        geo(&more, "stern_void_length_m") - geo(&base, "stern_void_length_m"),
        2.0
    );
    assert_eq!(geo(&base, "stern_void_length_m"), 3.0 + 0.5);
    // A gas pushed barge too.
    let g3 = results("gas_igc_2g_exact", "pushed_stern_void_m", json!(3.0));
    let g5 = results("gas_igc_2g_exact", "pushed_stern_void_m", json!(5.0));
    assert!(
        (geo(&g5, "stern_void_length_m") - geo(&g3, "stern_void_length_m") - 2.0).abs() < 1e-12
    );
    // The end bulkhead allowance is taken at both ends of a dry hold.
    let b1 = results("dry_bulk_box_hold", "bulk_end_bulkhead_m", json!(1.0));
    let rake = geo(&b1, "bow_rake_length_m");
    assert_eq!(geo(&b1, "bow_void_length_m"), rake + 1.0);
    assert_eq!(geo(&b1, "stern_void_length_m"), 3.0 + 1.0);
}

#[test]
fn the_ranges_refuse_what_the_engine_cannot_compute() {
    for (name, bad) in [
        ("bow_rake_length_to_depth_ratio", json!(-0.1)),
        ("pushed_stern_void_m", json!(-1.0)),
        ("bulk_end_bulkhead_m", json!(-0.1)),
    ] {
        let mut f: Value = serde_json::from_str(&raw_fixture("dry_bulk_box_hold")).unwrap();
        f["options"][0]["inputs"][name] = json!({ "value": bad, "source": "typed" });
        let checked = check(&f.to_string(), &Profile::built_in());
        assert!(
            checked.report.errors.iter().any(|e| e.contains(name)),
            "{name}: {:?}",
            checked.report.errors
        );
    }
}

#[test]
fn the_bow_rake_ratio_is_an_estimate_and_its_rake_comes_out_of_the_cargo_zone() {
    // Issue #661: no published source sets the ratio, so the profile marks it an estimate.
    let profile = Profile::built_in();
    assert_eq!(
        profile
            .default_status("bow_rake_length_to_depth_ratio")
            .map(|s| s.0),
        Some("estimate")
    );
    // The rake is part of the bow void, so every metre of rake the ratio adds is a metre less of
    // cargo zone, and the cargo volume does not grow.
    let file = "gas_igc_2g_self_propelled";
    let long = results(file, "bow_rake_length_to_depth_ratio", json!(1.6));
    let short = results(file, "bow_rake_length_to_depth_ratio", json!(1.0));
    let depth = long["dims"]["depth_m"].as_f64().unwrap();
    let less_rake = geo(&long, "bow_rake_length_m") - geo(&short, "bow_rake_length_m");
    assert!((less_rake - 0.6 * depth).abs() < 1e-9);
    assert!(
        (geo(&short, "cargo_zone_length_m") - geo(&long, "cargo_zone_length_m") - less_rake).abs()
            < 1e-9
    );
    let volume = |r: &Value| r["results"]["cargo_volume_m3"].as_f64().unwrap();
    assert!(volume(&short) >= volume(&long));
}
