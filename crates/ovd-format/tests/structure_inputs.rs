//! Seam: the `structure` block of the Hull in format version 4 (issue #739, spec 0013, "Format").
//! Inputs only: a file saved before reads the spec defaults with one note and keeps its results
//! hash; a new Option takes the Defaults Profile values; the Vessel Model sets its frames from the
//! frame spacing.
//!
//! Label: property. Each test states a rule: the schema and the status of each default, the legacy
//! values and their note, the version written, the frames of the Vessel Model.

use ovd_core::vessel_model::{SteelGrade, StructureBlock};
use ovd_format::vdesign::{
    FORMAT_VERSION, LEGACY_FIXED_INPUTS, LEGACY_FIXED_OTHER, STRUCTURE_INPUTS, legacy_value,
};
use ovd_format::{
    DesignSession, HashStatus, Profile, Source, check, fill_as_new, input_schema,
    option_vessel_model,
};
use serde_json::{Value, json};

/// The spec's default of each input, and whether it is a rule value (sourced) or an estimate.
const DEFAULTS: &[(&str, &str, &str)] = &[
    ("frame_spacing_m", "3.66", "sourced"),
    ("longitudinal_spacing_mm", "700.0", "estimate"),
    ("stock_plate_width_m", "3.0", "estimate"),
    ("stock_plate_length_m", "12.0", "estimate"),
    ("thickness_step_mm", "0.5", "estimate"),
    ("steel_grade", "\"ordinary\"", "estimate"),
    ("structural_wing_deck", "false", "estimate"),
];

fn legacy(name: &str) -> String {
    let path = format!("{}/tests/legacy/{name}.vdesign", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn hand(cargo: &str, formation: &str) -> String {
    json!({
        "format": "vdesign", "format_version": 4, "engine_version": "0.3.0", "units": "SI",
        "defaults_profile": "built-in",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": cargo, "source": "typed" },
            "hull_form": { "value": "raked_box", "source": "typed" },
            "formation": { "value": { "kind": formation }, "source": "typed" },
            "length_overall_m": { "value": 110.0, "source": "typed" },
            "beam_m": { "value": 11.4, "source": "typed" },
            "moulded_depth_m": { "value": 6.0, "source": "typed" }
        }}]
    })
    .to_string()
}

fn value(design: &ovd_format::Design, name: &str) -> Option<(Value, Source)> {
    design.options[0]
        .input(name)
        .map(|i| (i.value.clone(), i.source))
}

fn num(v: &Value) -> Value {
    // 700 and 700.0 are the same input value.
    v.as_f64().map_or(v.clone(), |x| json!(x))
}

#[test]
fn the_schema_lists_the_structure_block_with_its_defaults_and_status() {
    let p = Profile::built_in();
    let schema = input_schema(&p);
    let inputs = schema["inputs"].as_array().unwrap();
    let names: Vec<&str> = DEFAULTS.iter().map(|d| d.0).collect();
    assert_eq!(names, STRUCTURE_INPUTS);
    for (name, default, status) in DEFAULTS {
        let e = inputs
            .iter()
            .find(|i| i["name"] == *name)
            .unwrap_or_else(|| panic!("{name} is in the schema"));
        assert_eq!(e["group"], "Structure", "{name}");
        assert_eq!(e["default_status"], *status, "{name}");
        if *status == "sourced" {
            let source = e["default_source"].as_str().unwrap();
            assert!(source.contains("5-1-2/5.7.1"), "{name}: {source}");
        }
        assert!(!p.basis(name).is_empty(), "{name}");
        if e["kind"] == "number" {
            assert!(e.get("hard").is_some(), "{name}: a hard range");
        }
        let want: Value = serde_json::from_str(default).unwrap();
        assert_eq!(num(&e["default"]), want, "{name}");
        // The value an old file reads is the same default.
        assert_eq!(
            num(&legacy_value(name, Some("type_c_tanks")).unwrap()),
            want
        );
        assert_eq!(num(&legacy_value(name, Some("box_hold")).unwrap()), want);
    }
    let group = schema["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["name"] == "Structure" || g == &&json!("Structure"));
    assert!(group.is_some(), "{:?}", schema["groups"]);
}

/// The defaults are the same three ways: the legacy tables, the profile (above) and the core.
#[test]
fn the_legacy_values_are_the_core_defaults() {
    let d = StructureBlock::default();
    let fixed = |n: &str| {
        LEGACY_FIXED_INPUTS
            .iter()
            .find(|l| l.name == n)
            .unwrap()
            .value
    };
    assert_eq!(fixed("frame_spacing_m"), d.frame_spacing_m);
    assert_eq!(fixed("longitudinal_spacing_mm"), d.longitudinal_spacing_mm);
    assert_eq!(fixed("stock_plate_width_m"), d.stock_plate_width_m);
    assert_eq!(fixed("stock_plate_length_m"), d.stock_plate_length_m);
    assert_eq!(fixed("thickness_step_mm"), d.thickness_step_mm);
    let other = |n: &str| {
        LEGACY_FIXED_OTHER
            .iter()
            .find(|l| l.name == n)
            .unwrap()
            .value
    };
    assert_eq!(other("steel_grade"), format!("\"{}\"", d.steel_grade.key()));
    assert_eq!(
        other("structural_wing_deck"),
        d.structural_wing_deck.to_string()
    );
    assert_eq!(SteelGrade::from_key("ordinary"), Some(SteelGrade::Ordinary));
    assert_eq!(SteelGrade::from_key("AH36"), None);
}

#[test]
fn a_version_3_file_reads_the_defaults_with_one_note_and_keeps_its_results() {
    let text = legacy("v3_gas_before_739");
    assert!(text.contains("\"format_version\": 3"));
    let checked = check(&text, &Profile::built_in());
    assert!(
        checked.report.errors.is_empty(),
        "{:?}",
        checked.report.errors
    );
    let design = checked.design.unwrap();
    for (name, default, _) in DEFAULTS {
        let want: Value = serde_json::from_str(default).unwrap();
        let (got, source) = value(&design, name).unwrap_or_else(|| panic!("{name}"));
        assert_eq!((num(&got), source), (want, Source::Default), "{name}");
    }
    let notes: Vec<&String> = checked
        .report
        .migration_notes
        .iter()
        .filter(|w| w.contains("structure block"))
        .collect();
    assert_eq!(notes.len(), 1, "{:?}", checked.report.migration_notes);
    assert!(notes[0].contains("results are unchanged"), "{}", notes[0]);
    // No structure input is named in the generic "fixed value" note.
    for w in &checked.report.warnings {
        if w.contains("fixed value the engine used") {
            for name in STRUCTURE_INPUTS {
                assert!(!w.contains(&format!("`{name}`")), "{w}");
            }
        }
    }
    let (_, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
}

#[test]
fn saving_writes_version_4_and_the_saved_file_needs_no_note() {
    assert_eq!(FORMAT_VERSION, 4);
    let (s, _) = DesignSession::open(&legacy("v3_gas_before_739"), Profile::built_in()).unwrap();
    let text = s.save().unwrap();
    let saved: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(saved["format_version"], 4);
    let again = check(&text, &Profile::built_in());
    assert!(again.report.errors.is_empty(), "{:?}", again.report.errors);
    assert!(
        !again
            .report
            .warnings
            .iter()
            .any(|w| w.contains("structure block")),
        "{:?}",
        again.report.warnings
    );
}

#[test]
fn a_new_option_takes_the_profile_values_for_every_formation() {
    let p = Profile::built_in();
    for (cargo, formation) in [
        ("LNG", "self_propelled"),
        ("bauxite", "pushed_barge"),
        ("crude_oil", "self_propelled"),
    ] {
        let filled = fill_as_new(&hand(cargo, formation), &p);
        assert!(
            filled.report.errors.is_empty(),
            "{cargo}: {:?}",
            filled.report.errors
        );
        let d = filled.design.unwrap();
        for (name, default, _) in DEFAULTS {
            let want: Value = serde_json::from_str(default).unwrap();
            let (got, source) = value(&d, name).unwrap_or_else(|| panic!("{cargo} {name}"));
            assert_eq!(
                (num(&got), source),
                (want, Source::Default),
                "{cargo} {name}"
            );
        }
    }
}

#[test]
fn a_typed_value_out_of_its_hard_range_is_refused() {
    let p = Profile::built_in();
    let filled = fill_as_new(&hand("LNG", "self_propelled"), &p);
    let good = filled.design.unwrap().to_json();
    let with = |name: &str, v: Value| {
        let mut f = good.clone();
        f["options"][0]["inputs"][name] = json!({ "value": v, "source": "typed" });
        check(&f.to_string(), &p).report
    };
    for (name, v) in [
        ("frame_spacing_m", json!(0.0)),
        ("longitudinal_spacing_mm", json!(-700.0)),
        ("stock_plate_width_m", json!(0.0)),
        ("thickness_step_mm", json!(-0.5)),
        ("steel_grade", json!("AH36")),
        ("structural_wing_deck", json!("yes")),
    ] {
        let r = with(name, v.clone());
        assert!(
            r.error_inputs.iter().any(|e| e.iter().any(|n| n == name)),
            "{name} = {v}: {:?}",
            r.errors
        );
    }
    for (name, v) in [
        ("frame_spacing_m", json!(2.4)),
        ("structural_wing_deck", json!(true)),
    ] {
        let r = with(name, v);
        assert!(r.errors.is_empty(), "{name}: {:?}", r.errors);
    }
}

/// The Vessel Model's frames follow the Option's frame spacing (spec 0013, "Model").
#[test]
fn the_vessel_model_sets_its_frames_from_the_frame_spacing() {
    let p = Profile::built_in();
    let filled = fill_as_new(&hand("LNG", "self_propelled"), &p);
    let mut design = filled.design.unwrap();
    let model = option_vessel_model(&design.options[0]).unwrap();
    assert_eq!(model.structure, StructureBlock::default());
    assert_eq!(model.frames[0].name, "FR0");
    assert!((model.frames[1].x_m - model.frames[0].x_m - 3.66).abs() < 1e-9);
    let at_366 = model.frames.len();
    // Typed: 2.0 m and a structural wing deck.
    let o = &mut design.options[0];
    for (name, v) in [
        ("frame_spacing_m", json!(2.0)),
        ("structural_wing_deck", json!(true)),
    ] {
        let at = o.inputs.iter().position(|(n, _)| n == name).unwrap();
        o.inputs[at].1.value = v;
        o.inputs[at].1.source = Source::Typed;
    }
    let model = option_vessel_model(&design.options[0]).unwrap();
    assert_eq!(model.structure.frame_spacing_m, 2.0);
    assert!(model.structure.structural_wing_deck);
    assert!(model.frames.len() > at_366);
    for (i, f) in model.frames.iter().enumerate() {
        assert_eq!(f.name, format!("FR{i}"));
        assert!((f.x_m - model.frames[0].x_m - 2.0 * i as f64).abs() < 1e-9);
    }
}
