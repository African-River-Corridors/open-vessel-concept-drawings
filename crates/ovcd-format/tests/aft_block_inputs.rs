//! Seam: the aft block design values are inputs (issue #4, slice 4): the thrusters, the 40 ft
//! units, and the wheelhouse of a self-propelled hull. They were fixed in the engine; the defaults
//! are those values, so no design moves.
//!
//! Label: property. Every test states a rule: a default equals the old constant, an old file keeps
//! its results (its stored hash still matches), an input reaches the core and the drawing object,
//! or the file refuses what it should. No number here is a printed result.

use ovcd_core::aft_arrangement::{self as aft, AftArrangementInputs, AftDesign};
use ovcd_core::gas_arrangement::{FitOptions, fit};
use ovcd_format::vdesign::{INPUTS, LEGACY_FIXED_INPUTS};
use ovcd_format::{DesignSession, HashStatus, Profile, RunOutcome, Source, check};
use serde_json::{Value, json};

const NEW: [&str; 17] = [
    "thruster_count",
    "thruster_envelope_m",
    "thruster_clearance_m",
    "thruster_room_length_m",
    "battery_unit_count",
    "accommodation_unit_count",
    "unit_height_m",
    "unit_side_gap_m",
    "unit_row_gap_m",
    "unit_foundation_m",
    "wheelhouse_length_m",
    "wheelhouse_width_m",
    "wheelhouse_height_m",
    "wheelhouse_gap_m",
    "wheelhouse_eye_height_m",
    "sightline_margin_m",
    "wheelhouse_max_lift_m",
];

const GAS: &str = "gas_igc_2g_self_propelled";
const GAS_PUSHED: &str = "gas_igc_2g_exact";
const DRY: &str = "dry_bulk_sloped_hopper";

fn raw_fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// A fixture as a file written before these inputs existed: the keys taken out. Its stored results
/// hash is the one the old engine gave, so a match proves the results did not move.
fn without_keys(text: &str) -> String {
    let mut file: Value = serde_json::from_str(text).unwrap();
    for option in file["options"].as_array_mut().unwrap() {
        let inputs = option["inputs"].as_object_mut().unwrap();
        for key in NEW {
            inputs.remove(key);
        }
    }
    file.to_string()
}

/// The dry-bulk hopper fixture as a self-propelled hull with no stored hash and no aft keys: the
/// shape of an old file that the old engine ran.
fn dry_self_propelled(keys: bool) -> String {
    let mut f: Value = serde_json::from_str(&raw_fixture(DRY)).unwrap();
    let o = &mut f["options"][0];
    o["inputs"]["formation"] = json!({ "value": { "kind": "self_propelled" }, "source": "typed" });
    o["results_hash"] = Value::Null;
    let text = f.to_string();
    if keys {
        // `fill` writes the profile values in.
        let filled = ovcd_format::fill(&text, &Profile::built_in());
        assert!(filled.report.is_ok(), "{:?}", filled.report);
        filled.design.unwrap().to_text()
    } else {
        text
    }
}

/// The core default of an input, by the phase where it differs.
fn core_default(name: &str, gas: bool) -> f64 {
    let a = AftArrangementInputs::new(11.4, 4.5);
    let d = AftDesign::default();
    let same = |x: f64, y: f64| {
        assert_eq!(x, y, "{name}: block default vs design default");
        x
    };
    match name {
        "thruster_count" => same(a.n_thrusters as f64, d.n_thrusters as f64),
        "thruster_envelope_m" => same(a.thruster_envelope_m, d.thruster_envelope_m),
        "thruster_clearance_m" => same(a.thruster_clear_m, d.thruster_clear_m),
        "thruster_room_length_m" => same(a.thruster_room_length_m, d.thruster_room_length_m),
        // `AftArrangementInputs::new` carries the gas mix; the dry mix is the other constants.
        "battery_unit_count" if gas => same(
            a.n_battery_units as f64,
            aft::DEFAULT_BATTERY_UNITS_GAS as f64,
        ),
        "battery_unit_count" => aft::DEFAULT_BATTERY_UNITS_DRY as f64,
        "accommodation_unit_count" if gas => {
            same(a.n_accom_units as f64, aft::DEFAULT_ACCOM_UNITS_GAS as f64)
        }
        "accommodation_unit_count" => aft::DEFAULT_ACCOM_UNITS_DRY as f64,
        "unit_height_m" => same(a.container_height_m, d.container_height_m),
        "unit_side_gap_m" => same(a.container_side_gap_m, d.container_side_gap_m),
        "unit_row_gap_m" => same(a.container_gap_m, d.container_gap_m),
        "unit_foundation_m" => same(a.container_foundation_m, d.container_foundation_m),
        "wheelhouse_length_m" => same(a.wheelhouse_length_m, d.wheelhouse_length_m),
        "wheelhouse_width_m" => same(a.wheelhouse_width_m, d.wheelhouse_width_m),
        "wheelhouse_height_m" => same(a.wheelhouse_height_m, d.wheelhouse_height_m),
        "wheelhouse_gap_m" => same(a.wheelhouse_gap_m, d.wheelhouse_gap_m),
        "wheelhouse_eye_height_m" => same(
            a.wheelhouse_eye_above_floor_m,
            d.wheelhouse_eye_above_floor_m,
        ),
        "sightline_margin_m" => same(a.sightline_margin_m, d.sightline_margin_m),
        "wheelhouse_max_lift_m" => same(a.wheelhouse_max_lift_m, d.wheelhouse_max_lift_m),
        other => panic!("{other}"),
    }
}

const GAS_CONTAINMENT: &str = "type_c_tanks";
const DRY_CONTAINMENTS: [&str; 2] = ["box_hold", "sloped_hopper_hold"];

/// The profile's default of an input for a Cargo phase (a keyed default for the two unit counts).
fn profile_default(profile: &Profile, name: &str, gas: bool) -> f64 {
    let phase = if gas { "liquid_gas" } else { "dry_bulk" };
    profile
        .default(name)
        .or_else(|| profile.default_by(name, "cargo_phase", phase))
        .and_then(Value::as_f64)
        .unwrap_or_else(|| panic!("{name}: no profile default"))
}

/// The legacy table value for the Cargo Containment, and the number of rows that apply to it.
fn legacy_value(name: &str, containment: &str) -> f64 {
    let rows: Vec<_> = LEGACY_FIXED_INPUTS
        .iter()
        .filter(|l| l.name == name && l.applies_to.includes(containment))
        .collect();
    assert_eq!(rows.len(), 1, "{name} / {containment}: one row applies");
    assert_eq!(rows[0].formations, Some(&["self_propelled"][..]), "{name}");
    rows[0].value
}

#[test]
fn each_default_equals_the_legacy_table_value_and_the_core_default() {
    let profile = Profile::built_in();
    for name in NEW {
        // Issue #347: a new Option takes the ES-TRIN 7.02 eye height, 1.65 m; a file saved before
        // keeps the 1.5 m of the table, which is still the core's.
        if name == "wheelhouse_eye_height_m" {
            assert_eq!(profile_default(&profile, name, true), 1.65);
            assert_eq!(
                legacy_value(name, GAS_CONTAINMENT),
                core_default(name, true)
            );
            continue;
        }
        let gas = (
            profile_default(&profile, name, true),
            legacy_value(name, GAS_CONTAINMENT),
            core_default(name, true),
        );
        assert_eq!(gas.0, gas.1, "{name} (gas): profile vs table");
        assert_eq!(gas.1, gas.2, "{name} (gas): table vs core");
        for c in DRY_CONTAINMENTS {
            let dry = (
                profile_default(&profile, name, false),
                legacy_value(name, c),
                core_default(name, false),
            );
            assert_eq!(dry.0, dry.1, "{name} ({c}): profile vs table");
            assert_eq!(dry.1, dry.2, "{name} ({c}): table vs core");
        }
    }
}

#[test]
fn the_unit_counts_are_keyed_by_cargo_phase_in_the_profile() {
    let profile = Profile::built_in();
    for (name, dry, gas) in [
        ("battery_unit_count", 2.0, 1.0),
        ("accommodation_unit_count", 1.0, 2.0),
    ] {
        assert!(
            profile.default(name).is_none(),
            "{name} has a keyed default"
        );
        assert_eq!(
            profile
                .default_by(name, "cargo_phase", "dry_bulk")
                .and_then(Value::as_f64),
            Some(dry)
        );
        assert_eq!(
            profile
                .default_by(name, "cargo_phase", "liquid_gas")
                .and_then(Value::as_f64),
            Some(gas)
        );
    }
    // The other fifteen carry one default for every phase.
    for name in NEW.iter().filter(|n| !n.ends_with("unit_count")) {
        assert!(profile.default(name).is_some(), "{name}");
    }
}

#[test]
fn the_inputs_apply_to_every_containment_and_to_self_propelled_only_in_their_group() {
    for name in NEW {
        let spec = INPUTS.iter().find(|s| s.name == name).unwrap();
        assert_eq!(spec.group.label(), "Aft block", "{name}");
        assert_eq!(
            spec.applies_to.containments(),
            ["box_hold", "sloped_hopper_hold", "type_c_tanks"],
            "{name}"
        );
        assert_eq!(spec.formations, Some(&["self_propelled"][..]), "{name}");
    }
    let schema = ovcd_format::session::input_schema(&Profile::built_in());
    let entry = |n: &str| {
        schema["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["name"] == n)
            .unwrap()
            .clone()
    };
    for name in NEW {
        assert_eq!(
            entry(name)["applies_to_formation"],
            json!(["self_propelled"])
        );
        assert_eq!(entry(name)["group"], "Aft block");
    }
    // The two unit counts give the left panel a default for each Cargo phase.
    assert_eq!(
        entry("battery_unit_count")["default_by"]["values"],
        json!({ "dry_bulk": 2, "liquid_gas": 1 })
    );
}

#[test]
fn a_self_propelled_gas_file_without_the_inputs_opens_with_one_note_and_the_same_results() {
    let text = without_keys(&raw_fixture(GAS));
    let checked = check(&text, &Profile::built_in());
    assert_eq!(checked.report.errors, Vec::<String>::new());
    let design = checked.design.unwrap();
    let o = &design.options[0];
    for name in NEW {
        let stored = o.input(name).unwrap_or_else(|| panic!("{name}"));
        assert_eq!(stored.source, Source::Default, "{name}");
        assert_eq!(
            stored.value.as_f64(),
            Some(core_default(name, true)),
            "{name}"
        );
    }
    let (_, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
    let notes = opened
        .report
        .warnings
        .iter()
        .filter(|w| w.contains("fixed value the engine used before it was an input"))
        .count();
    assert_eq!(notes, 1, "{:?}", opened.report.warnings);
}

#[test]
fn a_version_1_self_propelled_file_gets_the_fixed_values_in_one_grouped_note() {
    // The version 1 file in `tests/legacy` is self-propelled gas; the inputs it lacks are filled
    // from the table.
    let path = format!(
        "{}/tests/legacy/v1_adn_g_self_propelled.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).unwrap();
    let (_, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
    assert_eq!(opened.report.errors, Vec::<String>::new());
    let grouped = opened
        .report
        .warnings
        .iter()
        .find(|w| w.contains("`thruster_count`") && w.contains("`wheelhouse_max_lift_m`"))
        .unwrap_or_else(|| panic!("{:?}", opened.report.warnings));
    assert!(grouped.contains("fixed value the engine used before it was an input"));
    // One note for the Option, not one per input.
    let notes = opened
        .report
        .warnings
        .iter()
        .filter(|w| w.contains("`thruster_count`"))
        .count();
    assert_eq!(notes, 1);
    // That file's ruleset changed in version 2, so its stored hash is expected to differ; the
    // hash proof of an old self-propelled file is the keys-removed fixture above.
    let design = opened_design(&text);
    for name in NEW {
        let stored = design.options[0].input(name).unwrap();
        assert_eq!(stored.source, Source::Default, "{name}");
        assert_eq!(
            stored.value.as_f64(),
            Some(core_default(name, true)),
            "{name}"
        );
    }
}

#[test]
fn a_dry_self_propelled_file_without_the_inputs_gives_the_same_results_as_one_with_the_defaults() {
    // The old fixed values (the table) and the profile defaults give one result, per Cargo phase.
    let (old_style, _) =
        DesignSession::open(&dry_self_propelled(false), Profile::built_in()).unwrap();
    let (filled, _) = DesignSession::open(&dry_self_propelled(true), Profile::built_in()).unwrap();
    let results_of = |s: &DesignSession| match s.evaluate().options[0].outcome.as_ref().unwrap() {
        RunOutcome::Results { results, hash, .. } => (results.clone(), hash.clone()),
        other => panic!("{other:?}"),
    };
    let (a, ha) = results_of(&old_style);
    let (b, hb) = results_of(&filled);
    assert_eq!(a, b);
    assert_eq!(ha, hb);
    // And the table gave the dry mix: two batteries and one accommodation unit.
    let labels = &a["drawing"]["aft_arrangement"]["container_labels"];
    let count = |s: &str| {
        labels
            .as_array()
            .unwrap()
            .iter()
            .filter(|l| l.as_str().unwrap().contains(s))
            .count()
    };
    assert_eq!((count("BATTERY"), count("ACCOM")), (2, 1), "{labels}");
}

#[test]
fn the_gas_unit_mix_is_one_battery_and_two_accommodation_units() {
    let r = results(GAS, &[]);
    let labels = r["drawing"]["aft_arrangement"]["container_labels"]
        .as_array()
        .unwrap()
        .clone();
    let count = |s: &str| {
        labels
            .iter()
            .filter(|l| l.as_str().unwrap().contains(s))
            .count()
    };
    assert_eq!((count("BATTERY"), count("ACCOM")), (1, 2));
}

#[test]
fn a_pushed_file_holds_none_of_them_and_a_self_propelled_file_holds_all() {
    for file in [GAS_PUSHED, DRY, "dry_bulk_box_hold"] {
        let raw: Value = serde_json::from_str(&raw_fixture(file)).unwrap();
        for name in NEW {
            assert!(
                raw["options"][0]["inputs"].get(name).is_none(),
                "{file}: {name}"
            );
        }
        let checked = check(&raw_fixture(file), &Profile::built_in());
        assert_eq!(checked.report.warnings, Vec::<String>::new(), "{file}");
    }
    let raw: Value = serde_json::from_str(&raw_fixture(GAS)).unwrap();
    for name in NEW {
        assert_eq!(
            raw["options"][0]["inputs"][name]["source"], "default",
            "{name}"
        );
    }
}

#[test]
fn a_pushed_file_that_holds_one_ignores_it_with_a_warning() {
    let mut f: Value = serde_json::from_str(&raw_fixture(GAS_PUSHED)).unwrap();
    f["options"][0]["inputs"]["wheelhouse_width_m"] = json!({ "value": 7.0, "source": "typed" });
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
            .any(|w| w.contains("`wheelhouse_width_m` is not used for a pushed_barge Formation")),
        "{:?}",
        checked.report.warnings
    );
}

#[test]
fn the_ranges_refuse_what_the_engine_cannot_compute() {
    for (name, bad, why) in [
        ("thruster_count", json!(0), "thruster_count"),
        ("thruster_count", json!(2.5), "whole number"),
        ("battery_unit_count", json!(-1), "battery_unit_count"),
        ("battery_unit_count", json!(1.5), "whole number"),
        (
            "accommodation_unit_count",
            json!(-1),
            "accommodation_unit_count",
        ),
        ("thruster_envelope_m", json!(0), "thruster_envelope_m"),
        ("thruster_clearance_m", json!(-0.1), "thruster_clearance_m"),
        ("thruster_room_length_m", json!(0), "thruster_room_length_m"),
        ("unit_height_m", json!(0), "unit_height_m"),
        ("unit_side_gap_m", json!(-0.1), "unit_side_gap_m"),
        ("unit_row_gap_m", json!(-0.1), "unit_row_gap_m"),
        ("unit_foundation_m", json!(-0.1), "unit_foundation_m"),
        ("wheelhouse_length_m", json!(0), "wheelhouse_length_m"),
        ("wheelhouse_width_m", json!(0), "wheelhouse_width_m"),
        ("wheelhouse_height_m", json!(0), "wheelhouse_height_m"),
        ("wheelhouse_gap_m", json!(-0.1), "wheelhouse_gap_m"),
        (
            "wheelhouse_eye_height_m",
            json!(0),
            "wheelhouse_eye_height_m",
        ),
        ("sightline_margin_m", json!(-0.1), "sightline_margin_m"),
        ("wheelhouse_max_lift_m", json!(-1), "wheelhouse_max_lift_m"),
    ] {
        let mut f: Value = serde_json::from_str(&raw_fixture(GAS)).unwrap();
        f["options"][0]["inputs"][name] = json!({ "value": bad, "source": "typed" });
        let checked = check(&f.to_string(), &Profile::built_in());
        assert!(
            checked.report.errors.iter().any(|e| e.contains(why)),
            "{name} = {bad}: {:?}",
            checked.report.errors
        );
    }
}

fn opened_design(text: &str) -> ovcd_format::vdesign::Design {
    check(text, &Profile::built_in()).design.unwrap()
}

/// Run a fixture with some inputs changed (typed) and give the results.
fn results(file: &str, changes: &[(&str, Value)]) -> Value {
    let mut f: Value = serde_json::from_str(&raw_fixture(file)).unwrap();
    for (name, value) in changes {
        f["options"][0]["inputs"][*name] = json!({ "value": value, "source": "typed" });
    }
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

fn aft_of(v: &Value) -> Value {
    v["drawing"]["aft_arrangement"].clone()
}

#[test]
fn each_input_moves_the_aft_block_in_the_drawing_object() {
    let base = aft_of(&results(GAS, &[]));
    // (input, a value that is not the default, a field of the aft block that must move)
    let cases: [(&str, Value, &str); 17] = [
        ("thruster_count", json!(2), "thruster_offsets_m"),
        ("thruster_envelope_m", json!(3.0), "thruster_offsets_m"),
        ("thruster_clearance_m", json!(0.8), "thruster_offsets_m"),
        (
            "thruster_room_length_m",
            json!(4.0),
            "thruster_room_length_m",
        ),
        ("battery_unit_count", json!(3), "container_labels"),
        ("accommodation_unit_count", json!(3), "container_labels"),
        ("unit_height_m", json!(2.591), "container_top_above_keel_m"),
        ("unit_side_gap_m", json!(1.1), "container_side_gap_m"),
        ("unit_row_gap_m", json!(1.0), "container_row_gap_m"),
        ("unit_foundation_m", json!(0.5), "container_x0_m"),
        ("wheelhouse_length_m", json!(4.0), "wheelhouse_x1_m"),
        ("wheelhouse_width_m", json!(6.0), "wheelhouse_width_m"),
        ("wheelhouse_height_m", json!(3.0), "wheelhouse_height_m"),
        ("wheelhouse_gap_m", json!(0.5), "wheelhouse_x0_m"),
        ("wheelhouse_eye_height_m", json!(1.7), "wheelhouse_lift_m"),
        ("sightline_margin_m", json!(2.5), "wheelhouse_lift_m"),
        ("wheelhouse_max_lift_m", json!(2.0), "wheelhouse_lift_m"),
    ];
    for (name, value, field) in cases {
        let moved = aft_of(&results(GAS, &[(name, value.clone())]));
        assert_ne!(
            base[field], moved[field],
            "{name} = {value} did not move {field}"
        );
    }
    // A value typed to the default changes nothing at all.
    for name in NEW {
        let default = core_default(name, true);
        let same = aft_of(&results(GAS, &[(name, json!(default))]));
        assert_eq!(base, same, "{name} at its default");
    }
}

#[test]
fn the_values_reach_the_aft_block_in_the_exact_way_the_core_reads_them() {
    let r = |n: &str, v: Value| aft_of(&results(GAS, &[(n, v)]));
    // The room length is the first zone; the units start a foundation after it.
    let a = r("thruster_room_length_m", json!(4.0));
    assert_eq!(a["thruster_room_length_m"], 4.0);
    let a = r("unit_foundation_m", json!(0.5));
    let room = a["thruster_room_length_m"].as_f64().unwrap();
    assert_eq!(a["container_x0_m"].as_f64().unwrap(), room + 0.5);
    // The wheelhouse is as long, wide and high as typed; the units are as high as typed.
    let a = r("wheelhouse_length_m", json!(4.0));
    assert_eq!(
        a["wheelhouse_x1_m"].as_f64().unwrap() - a["wheelhouse_x0_m"].as_f64().unwrap(),
        4.0
    );
    assert_eq!(
        r("wheelhouse_width_m", json!(6.0))["wheelhouse_width_m"],
        6.0
    );
    assert_eq!(
        r("wheelhouse_height_m", json!(3.0))["wheelhouse_height_m"],
        3.0
    );
    assert_eq!(
        r("unit_side_gap_m", json!(1.1))["container_side_gap_m"],
        1.1
    );
    assert_eq!(r("unit_row_gap_m", json!(1.0))["container_row_gap_m"], 1.0);
    // The thruster count is a maximum: two thrusters at most gives two offsets; a request for the
    // most the hull takes does not give more than the beam takes.
    let two = r("thruster_count", json!(2));
    assert_eq!(two["thruster_offsets_m"].as_array().unwrap().len(), 2);
    let many = r("thruster_count", json!(6));
    assert!(many["thruster_offsets_m"].as_array().unwrap().len() <= 6);
    // The unit counts: three batteries and three accommodation units in the labels.
    let labels = r("battery_unit_count", json!(3))["container_labels"].clone();
    let batteries = labels
        .as_array()
        .unwrap()
        .iter()
        .filter(|l| l.as_str().unwrap().contains("BATTERY"))
        .count();
    assert_eq!(batteries, 3);
    let labels = r("accommodation_unit_count", json!(3))["container_labels"].clone();
    let accom = labels
        .as_array()
        .unwrap()
        .iter()
        .filter(|l| l.as_str().unwrap().contains("ACCOM"))
        .count();
    assert_eq!(accom, 3);
    // A lift limit of 2 m shows in the lift: the cab cannot rise more than that.
    let lift = r("wheelhouse_max_lift_m", json!(2.0))["wheelhouse_lift_m"]
        .as_f64()
        .unwrap();
    assert!(lift <= 2.0 + 1e-9, "{lift}");
}

#[test]
fn a_dry_self_propelled_hull_follows_the_inputs_too() {
    // A dry file in a self-propelled Formation: the units default to two batteries and one
    // accommodation unit, and each input is read.
    let file = dry_self_propelled(true);
    let run = |name: &str, value: Value| {
        let mut f: Value = serde_json::from_str(&file).unwrap();
        f["options"][0]["inputs"][name] = json!({ "value": value, "source": "typed" });
        let (s, _) = DesignSession::open(&f.to_string(), Profile::built_in()).unwrap();
        match s.evaluate().options[0].outcome.as_ref().unwrap() {
            RunOutcome::Results { results, .. } => aft_of(results),
            other => panic!("{other:?}"),
        }
    };
    let base = {
        let (s, _) = DesignSession::open(&file, Profile::built_in()).unwrap();
        match s.evaluate().options[0].outcome.as_ref().unwrap() {
            RunOutcome::Results { results, .. } => aft_of(results),
            other => panic!("{other:?}"),
        }
    };
    for (name, value, field) in [
        ("battery_unit_count", json!(3), "container_labels"),
        ("accommodation_unit_count", json!(2), "container_labels"),
        ("wheelhouse_length_m", json!(4.5), "wheelhouse_x1_m"),
        ("thruster_room_length_m", json!(5.0), "length_m"),
    ] {
        assert_ne!(base[field], run(name, value.clone())[field], "{name}");
    }
}

#[test]
fn the_gas_search_carries_the_values_through_to_every_candidate() {
    // `fit` builds each candidate from `FitOptions`; the aft block of the winning candidate follows
    // the values, and a search at the defaults is the search it was.
    let opts = FitOptions::default();
    let base = fit(110.0, 11.4, "LPG_propane", 4.2, &opts).unwrap();
    let again = fit(
        110.0,
        11.4,
        "LPG_propane",
        4.2,
        &FitOptions {
            aft_design: AftDesign::default(),
            ..opts.clone()
        },
    )
    .unwrap();
    assert_eq!(base, again);
    let long_room = AftDesign {
        thruster_room_length_m: 5.5,
        wheelhouse_length_m: 4.6,
        n_accom_units: Some(3),
        ..AftDesign::default()
    };
    let moved = fit(
        110.0,
        11.4,
        "LPG_propane",
        4.2,
        &FitOptions {
            aft_design: long_room,
            ..opts
        },
    )
    .unwrap();
    let a0 = base.engine_result.aft_arrangement.as_ref().unwrap();
    let a1 = moved.engine_result.aft_arrangement.as_ref().unwrap();
    assert_ne!(a0.length_m, a1.length_m);
    assert_eq!(a1.thruster_room_length_m, 5.5);
    // A difference of two positions: equal to the length to within the rounding of the subtraction.
    assert!((a1.wheelhouse_x1_m - a1.wheelhouse_x0_m - 4.6).abs() < 1e-9);
    // The stern void of the candidate grew with the block.
    assert!(moved.stern_void_m > base.stern_void_m);
    // The fit result of a pushed barge does not carry the block.
    let pushed = fit(
        110.0,
        11.4,
        "LPG_propane",
        4.2,
        &FitOptions {
            self_propelled: false,
            aft_design: AftDesign {
                thruster_room_length_m: 9.0,
                ..AftDesign::default()
            },
            ..FitOptions::default()
        },
    )
    .unwrap();
    assert!(pushed.engine_result.aft_arrangement.is_none());
}

#[test]
fn a_session_gives_the_profile_value_when_the_formation_or_cargo_changes() {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    // A starter Design is a pushed barge: no aft inputs.
    assert!(s.design().options[0].input("thruster_count").is_none());
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    for name in NEW {
        // Issue #347: a new dry-bulk Option has shafts in nozzles, so the thruster count of an
        // unspecified propulsor is not used.
        if name == "thruster_count" {
            assert!(s.design().options[0].input(name).is_none());
            continue;
        }
        let got = s.design().options[0]
            .input(name)
            .unwrap_or_else(|| panic!("{name}"));
        assert_eq!(got.source, Source::Default, "{name}");
    }
    // The starter Cargo is dry bulk: two batteries, one accommodation unit.
    let dry = |s: &DesignSession, n: &str| s.design().options[0].input(n).unwrap().value.as_f64();
    assert_eq!(dry(&s, "battery_unit_count"), Some(2.0));
    assert_eq!(dry(&s, "accommodation_unit_count"), Some(1.0));
    // A typed value comes back when the Formation changes back and forth.
    s.set_input("Option 1", "wheelhouse_width_m", json!(7.5))
        .unwrap();
    s.set_formation("Option 1", "pushed_barge", None, None)
        .unwrap();
    assert!(s.design().options[0].input("wheelhouse_width_m").is_none());
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    let back = s.design().options[0].input("wheelhouse_width_m").unwrap();
    assert_eq!(
        (back.value.clone(), back.source),
        (json!(7.5), Source::Typed)
    );
}

#[test]
fn an_edited_profile_changes_a_session_but_never_an_old_file() {
    let mut profile: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    profile["inputs"]["wheelhouse_gap_m"]["default"] = json!(0.45);
    profile["inputs"]["battery_unit_count"]["default_by_cargo_phase"]["dry_bulk"] = json!(3);
    let edited = Profile::from_json(&profile.to_string()).unwrap();
    // An old file lacks the keys: the fixed values, whatever the profile says.
    let checked = check(&without_keys(&raw_fixture(GAS)), &edited);
    let d = checked.design.unwrap();
    assert_eq!(
        d.options[0].input("wheelhouse_gap_m").unwrap().value,
        json!(0.3)
    );
    assert_eq!(
        d.options[0].input("battery_unit_count").unwrap().value,
        json!(1.0)
    );
    // A session under the edited profile: the profile values.
    let mut s = DesignSession::new_design(edited).unwrap();
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    let get = |n: &str| s.design().options[0].input(n).unwrap().value.as_f64();
    assert_eq!(get("wheelhouse_gap_m"), Some(0.45));
    assert_eq!(get("battery_unit_count"), Some(3.0));
}

#[test]
fn a_profile_copy_without_the_new_entries_takes_the_built_in_ones() {
    let mut profile: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    for name in NEW {
        profile["inputs"].as_object_mut().unwrap().remove(name);
    }
    let old = Profile::from_json(&profile.to_string()).unwrap();
    for name in NEW {
        assert_eq!(
            profile_default(&old, name, true),
            profile_default(&Profile::built_in(), name, true),
            "{name}"
        );
    }
}
