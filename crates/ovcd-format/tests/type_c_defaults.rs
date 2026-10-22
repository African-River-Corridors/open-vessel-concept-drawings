//! Seam: the Type C tank defaults depend on the Cargo Containment, the Cargo and, for an LPG Cargo,
//! its carriage condition (issue #292): design pressure, design temperature, tank steel, insulation,
//! fill ratio and tank structure factor come from a table of the Defaults Profile, each value with
//! its source or marked an estimate with its basis. The wall thickness comes from the
//! pressure-vessel formula unless a value is typed.
//!
//! Label: property. Each test states a rule: every combination has a row and a new Design takes it,
//! a value reaches the results, an old file keeps its results, an older profile copy still loads.
//! No number here is a printed result; the values compared are the table's own.

use ovcd_core::pressure_vessel::{TANK_MATERIAL_DATABASE, calculate_wall_thicknesses_with};
use ovcd_format::vdesign::{
    CARRIAGE_CONDITIONS, DEFAULT_CARRIAGE_CONDITION, INPUTS, INSULATION_TYPES, LEGACY_FIXED_OTHER,
    LPG_CARGOS, TABLE_DEFAULTS, TABLE_STATUSES, TANK_MATERIALS, table_combinations,
};
use ovcd_format::{DesignSession, HashStatus, Profile, RunOutcome, Source, check, fill_as_new};
use serde_json::{Value, json};

fn legacy(name: &str) -> String {
    let path = format!("{}/tests/legacy/{name}.vdesign", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// A new Design of this Cargo (and carriage condition, for an LPG Cargo), as a session makes it.
fn new_session(cargo: &str, carriage: &str) -> DesignSession {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    s.set_input("Option 1", "cargo", json!(cargo)).unwrap();
    s.set_input("Option 1", "length_overall_m", json!(110.0))
        .unwrap();
    s.set_input("Option 1", "beam_m", json!(11.4)).unwrap();
    s.set_input("Option 1", "moulded_depth_m", json!(6.0))
        .unwrap();
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    if !carriage.is_empty() {
        s.set_input("Option 1", "lpg_carriage_condition", json!(carriage))
            .unwrap();
    }
    s
}

fn stored(s: &DesignSession, name: &str) -> Option<(Value, Source)> {
    s.design().options[0]
        .input(name)
        .map(|i| (i.value.clone(), i.source))
}

fn results_of(s: &DesignSession) -> Value {
    match s.evaluate().options[0].outcome.as_ref().expect("it ran") {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

fn context<'a>(cargo: &'a str, carriage: &'a str) -> [(&'a str, &'a str); 3] {
    [
        ("cargo_containment", "type_c_tanks"),
        ("cargo", cargo),
        ("lpg_carriage_condition", carriage),
    ]
}

#[test]
fn the_choices_match_the_core() {
    let keys: Vec<&str> = TANK_MATERIAL_DATABASE.iter().map(|(k, _)| *k).collect();
    assert_eq!(keys, TANK_MATERIALS);
    assert_eq!(
        CARRIAGE_CONDITIONS,
        [
            "fully_pressurised",
            "semi_pressurised",
            "fully_refrigerated"
        ]
    );
    assert_eq!(DEFAULT_CARRIAGE_CONDITION, "fully_pressurised");
    // Every LPG key is a Cargo of the database; no other gas Cargo is LPG.
    for cargo in LPG_CARGOS {
        assert!(ovcd_core::cargo::get_cargo(cargo).is_ok(), "{cargo}");
    }
    for (key, props) in ovcd_core::cargo::CARGO_DATABASE.iter() {
        assert_eq!(key.starts_with("LPG_"), LPG_CARGOS.contains(key), "{key}");
        let _ = props;
    }
}

#[test]
fn every_table_row_has_a_status_and_a_basis_and_a_sourced_one_has_a_source() {
    let profile = Profile::built_in();
    for name in TABLE_DEFAULTS {
        let table = profile.table_json(name).unwrap_or_else(|| panic!("{name}"));
        let rows = table["rows"].as_array().unwrap();
        assert!(!rows.is_empty(), "{name}");
        for (i, row) in rows.iter().enumerate() {
            let status = row["status"].as_str().unwrap();
            assert!(TABLE_STATUSES.contains(&status), "{name} row {i}");
            assert!(
                !row["basis"].as_str().unwrap().trim().is_empty(),
                "{name} row {i}"
            );
            if status == "sourced" {
                assert!(
                    !row["source"].as_str().unwrap().trim().is_empty(),
                    "{name} row {i}"
                );
            }
        }
    }
}

#[test]
fn a_profile_row_that_claims_a_source_it_does_not_give_is_refused() {
    let edit = |f: &dyn Fn(&mut Value)| {
        let mut p: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
        f(&mut p);
        Profile::from_json(&p.to_string())
    };
    // Sourced, no source.
    let err = edit(&|p| {
        let row = &mut p["inputs"]["tank_design_pressure_mpa"]["default_by_table"]["rows"][0];
        row.as_object_mut().unwrap().remove("source");
    })
    .unwrap_err();
    assert!(err.contains("`sourced` but gives no `source`"), "{err}");
    // No basis.
    let err = edit(&|p| {
        let row = &mut p["inputs"]["tank_material"]["default_by_table"]["rows"][0];
        row.as_object_mut().unwrap().remove("basis");
    })
    .unwrap_err();
    assert!(err.contains("needs a `basis`"), "{err}");
    // A status that is neither.
    let err = edit(&|p| {
        p["inputs"]["tank_material"]["default_by_table"]["rows"][0]["status"] = json!("measured");
    })
    .unwrap_err();
    assert!(err.contains("needs a `status`"), "{err}");
    // A combination with no row: LPG propane, semi-pressurised, for the insulation thickness.
    let err = edit(&|p| {
        let rows = p["inputs"]["tank_insulation_thickness_mm"]["default_by_table"]["rows"]
            .as_array_mut()
            .unwrap();
        rows.retain(|r| {
            r["when"]["lpg_carriage_condition"] != json!(["semi_pressurised", "fully_refrigerated"])
        });
    })
    .unwrap_err();
    assert!(
        err.contains("no default for `tank_insulation_thickness_mm`"),
        "{err}"
    );
    assert!(err.contains("semi_pressurised"), "{err}");
    // A value outside the hard range of its input.
    let err = edit(&|p| {
        p["inputs"]["tank_structure_factor"]["default_by_table"]["rows"][1]["value"] = json!(0.5);
    })
    .unwrap_err();
    assert!(err.contains("tank_structure_factor"), "{err}");
}

#[test]
fn the_table_values_are_in_the_choices_and_the_ranges() {
    let profile = Profile::built_in();
    for (name, choices) in [
        ("tank_material", TANK_MATERIALS),
        ("tank_insulation_type", INSULATION_TYPES),
    ] {
        for value in profile.table_values(name) {
            assert!(choices.contains(&value.as_str().unwrap()), "{name} {value}");
        }
    }
}

#[test]
fn a_new_design_takes_the_table_row_of_every_combination_and_marks_it_default() {
    let profile = Profile::built_in();
    let mut combinations = 0;
    for combination in table_combinations() {
        let cargo = combination[1].1.clone();
        let carriage = combination[2].1.clone();
        let s = new_session(&cargo, &carriage);
        for name in TABLE_DEFAULTS {
            // The loading temperature is an input of the LPG Cargos only (issue #315).
            if *name == "lpg_loading_temperature_c" && !LPG_CARGOS.contains(&cargo.as_str()) {
                assert!(stored(&s, name).is_none(), "{name}: {cargo}");
                continue;
            }
            let want = profile
                .table_default(name, &context(&cargo, &carriage))
                .unwrap_or_else(|| panic!("{name}: {cargo} {carriage}"))
                .value
                .clone();
            let (got, source) =
                stored(&s, name).unwrap_or_else(|| panic!("{name}: {cargo} {carriage}"));
            assert_eq!(
                got.as_f64().map(|x| json!(x)).unwrap_or(got.clone()),
                want.as_f64().map(|x| json!(x)).unwrap_or(want.clone()),
                "{name}: {cargo} {carriage}"
            );
            assert_eq!(source, Source::Default, "{name}: {cargo} {carriage}");
        }
        // The wall thickness is left to the formula, and the stress to the material.
        for name in ["tank_wall_thickness_mm", "tank_allowable_stress_mpa"] {
            let (got, source) = stored(&s, name).unwrap();
            assert_eq!((got, source), (Value::Null, Source::Default), "{name}");
        }
        combinations += 1;
    }
    // LNG and LNG_ethane once each; each of the three LPG Cargos in each of three conditions.
    assert_eq!(combinations, 2 + 3 * 3);
}

#[test]
fn the_table_values_reach_the_results_and_the_wall_is_the_formula_on_the_steel_diameter() {
    let profile = Profile::built_in();
    for combination in table_combinations() {
        let cargo = combination[1].1.clone();
        let carriage = combination[2].1.clone();
        let s = new_session(&cargo, &carriage);
        let r = results_of(&s);
        let tank = &r["drawing"]["tank"];
        let want = |name: &str| {
            profile
                .table_default(name, &context(&cargo, &carriage))
                .unwrap()
                .value
                .clone()
        };
        let num = |v: &Value| v.as_f64().unwrap();
        assert_eq!(
            num(&tank["design_pressure_mpa"]),
            num(&want("tank_design_pressure_mpa"))
        );
        assert_eq!(
            tank["design_temperature_c"].as_f64(),
            want("tank_design_temperature_c").as_f64()
        );
        assert_eq!(tank["material"], want("tank_material"));
        assert_eq!(tank["insulation_type"], want("tank_insulation_type"));
        assert_eq!(
            num(&tank["insulation_thickness_mm"]),
            num(&want("tank_insulation_thickness_mm"))
        );
        assert_eq!(
            num(&tank["structure_factor"]),
            num(&want("tank_structure_factor"))
        );
        assert_eq!(
            num(&r["arrangement"]["fill_ratio"]),
            num(&want("fill_ratio"))
        );
        assert_eq!(
            tank["lpg_carriage_condition"].as_str().unwrap_or(""),
            carriage
        );
        assert_eq!(tank["wall_thickness_source"], "formula");
        // The datum: the steel diameter is the outer diameter less twice the insulation.
        let (od, steel, inner) = (
            num(&tank["tank_outer_diameter_m"]),
            num(&tank["tank_steel_outer_diameter_m"]),
            num(&tank["tank_inner_diameter_m"]),
        );
        let ins = num(&tank["insulation_thickness_mm"]) / 1000.0;
        assert!(
            (steel - (od - 2.0 * ins)).abs() < 1e-9,
            "{cargo} {carriage}"
        );
        assert_eq!(od, num(&r["geometry"]["tank_outer_diameter_m"]));
        let wall = num(&tank["cylinder_wall_thickness_mm"]);
        assert!(
            (inner - (steel - 2.0 * wall / 1000.0)).abs() < 1e-9,
            "{cargo} {carriage}"
        );
        // The wall is the formula on the steel diameter, with the material's allowance and stress,
        // at the design pressure plus the liquid pressure (issue #374).
        assert!(
            num(&tank["liquid_pressure_mpa"]) > 0.0,
            "{cargo} {carriage}"
        );
        let (cyl, head) = calculate_wall_thicknesses_with(
            steel,
            num(&tank["internal_pressure_mpa"]),
            tank["material"].as_str().unwrap(),
            tank["head_type"].as_str().unwrap(),
            0.95,
            None,
        )
        .unwrap();
        assert_eq!(
            (wall, num(&tank["head_wall_thickness_mm"])),
            (cyl, head),
            "{cargo} {carriage}"
        );
        assert_eq!(num(&r["geometry"]["insulation_thickness_mm"]), ins * 1000.0);
    }
}

#[test]
fn the_carriage_condition_is_for_lpg_cargos_only() {
    let mut s = new_session("LNG", "");
    assert!(stored(&s, "lpg_carriage_condition").is_none());
    let err = s
        .set_input(
            "Option 1",
            "lpg_carriage_condition",
            json!("semi_pressurised"),
        )
        .unwrap_err();
    assert!(err.contains("not used for the Cargo LNG"), "{err}");
    // The schema says so, and the LNG row of the table has no carriage condition.
    let schema = s.input_schema();
    let spec = schema["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["name"] == "lpg_carriage_condition")
        .unwrap();
    assert_eq!(
        spec["applies_to_cargo"],
        json!(["LPG_propane", "LPG_butane", "LPG_mix"])
    );
    assert_eq!(spec["group"], "Gas tanks");
    assert_eq!(spec["kind"], "choice");
    assert_eq!(spec["default"], "fully_pressurised");
    let values: Vec<&str> = spec["choices"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["value"].as_str().unwrap())
        .collect();
    assert_eq!(values, CARRIAGE_CONDITIONS);
    // An LPG Cargo has it, fully pressurised by default; a typed condition comes back after LNG.
    s.set_input("Option 1", "cargo", json!("LPG_propane"))
        .unwrap();
    assert_eq!(
        stored(&s, "lpg_carriage_condition"),
        Some((json!("fully_pressurised"), Source::Default))
    );
    s.set_input(
        "Option 1",
        "lpg_carriage_condition",
        json!("fully_refrigerated"),
    )
    .unwrap();
    s.set_input("Option 1", "cargo", json!("LNG")).unwrap();
    assert!(stored(&s, "lpg_carriage_condition").is_none());
    s.set_input("Option 1", "cargo", json!("LPG_butane"))
        .unwrap();
    assert_eq!(
        stored(&s, "lpg_carriage_condition"),
        Some((json!("fully_refrigerated"), Source::Typed))
    );
}

#[test]
fn a_changed_carriage_condition_or_cargo_moves_the_defaults_and_keeps_what_was_typed() {
    let profile = Profile::built_in();
    let mut s = new_session("LPG_propane", "fully_pressurised");
    let pressure = |s: &DesignSession| stored(s, "tank_design_pressure_mpa").unwrap();
    let pressurised = profile
        .table_default(
            "tank_design_pressure_mpa",
            &context("LPG_propane", "fully_pressurised"),
        )
        .unwrap()
        .value
        .clone();
    assert_eq!(pressure(&s), (pressurised.clone(), Source::Default));
    // The insulation thickness is typed; the pressure follows the carriage condition.
    s.set_input("Option 1", "tank_insulation_thickness_mm", json!(120.0))
        .unwrap();
    s.set_input(
        "Option 1",
        "lpg_carriage_condition",
        json!("semi_pressurised"),
    )
    .unwrap();
    let semi = profile
        .table_default(
            "tank_design_pressure_mpa",
            &context("LPG_propane", "semi_pressurised"),
        )
        .unwrap()
        .value
        .clone();
    assert_eq!(pressure(&s), (semi, Source::Default));
    assert_eq!(
        stored(&s, "tank_insulation_thickness_mm"),
        Some((json!(120.0), Source::Typed))
    );
    assert_eq!(stored(&s, "tank_material").unwrap().0, json!("9Ni"));
    // A new Cargo of another kind moves them again.
    s.set_input("Option 1", "cargo", json!("LNG")).unwrap();
    let lng = profile
        .table_default("tank_design_pressure_mpa", &context("LNG", ""))
        .unwrap()
        .value
        .clone();
    assert_eq!(pressure(&s), (lng, Source::Default));
    // A reset input takes the table value again.
    s.reset_input("Option 1", "tank_insulation_thickness_mm")
        .unwrap();
    assert_eq!(
        stored(&s, "tank_insulation_thickness_mm").unwrap().1,
        Source::Default
    );
}

#[test]
fn a_typed_wall_thickness_is_an_override_and_a_typed_stress_replaces_the_materials() {
    let mut s = new_session("LPG_propane", "fully_pressurised");
    let wall = |s: &DesignSession| {
        results_of(s)["drawing"]["tank"]["cylinder_wall_thickness_mm"]
            .as_f64()
            .unwrap()
    };
    let formula = wall(&s);
    s.set_input("Option 1", "tank_wall_thickness_mm", json!(25.0))
        .unwrap();
    let r = results_of(&s);
    assert_eq!(r["drawing"]["tank"]["wall_thickness_source"], "override");
    assert_eq!(wall(&s), 25.0);
    assert_ne!(formula, 25.0);
    // Back to the formula by resetting; none is the default.
    s.reset_input("Option 1", "tank_wall_thickness_mm").unwrap();
    assert_eq!(wall(&s), formula);
    // A typed stress: a lower stress asks for a thicker wall.
    s.set_input("Option 1", "tank_allowable_stress_mpa", json!(200.0))
        .unwrap();
    let r = results_of(&s);
    assert_eq!(r["drawing"]["tank"]["allowable_stress_mpa"], 200.0);
    assert!(wall(&s) > formula);
}

#[test]
fn an_lpg_file_that_names_no_carriage_condition_is_read_as_fully_pressurised_with_a_warning() {
    {
        let file = "v2_gas_lpg_before_292";
        let text = legacy(file);
        let checked = check(&text, &Profile::built_in());
        assert!(
            checked.report.errors.is_empty(),
            "{:?}",
            checked.report.errors
        );
        let design = checked.design.unwrap();
        let o = &design.options[0];
        assert_eq!(
            o.input("lpg_carriage_condition")
                .map(|i| (i.value.clone(), i.source)),
            Some((json!("fully_pressurised"), Source::Default))
        );
        let note = checked
            .report
            .warnings
            .iter()
            .find(|w| w.contains("has no `lpg_carriage_condition`"))
            .unwrap_or_else(|| panic!("{:?}", checked.report.warnings));
        assert!(note.contains("read as fully_pressurised"), "{note}");
        assert!(note.contains("results are unchanged"), "{note}");
        // The tank values the file lacks are the old ones, so the results do not move.
        let value = |n: &str| o.input(n).unwrap().value.clone();
        assert_eq!(value("tank_design_pressure_mpa"), json!(0.5));
        assert_eq!(value("tank_material"), json!("9Ni"));
        assert_eq!(value("tank_insulation_thickness_mm"), json!(350.0));
        assert_eq!(value("tank_structure_factor"), json!(1.3));
        assert_eq!(value("tank_wall_thickness_mm"), json!(25.0));
        assert_eq!(value("tank_allowable_stress_mpa"), Value::Null);
        let (_, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
        assert_eq!(opened.hash_status[0].1, HashStatus::Matches, "{file}");
    }
}

#[test]
fn an_lng_file_from_before_has_no_carriage_condition_and_keeps_its_results() {
    let text = legacy("v2_gas_lng_before_292");
    let checked = check(&text, &Profile::built_in());
    assert!(
        checked.report.errors.is_empty(),
        "{:?}",
        checked.report.errors
    );
    assert!(
        !checked
            .report
            .warnings
            .iter()
            .any(|w| w.contains("lpg_carriage_condition")),
        "{:?}",
        checked.report.warnings
    );
    assert!(
        checked.design.unwrap().options[0]
            .input("lpg_carriage_condition")
            .is_none()
    );
    let (_, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
}

#[test]
fn a_saved_file_that_stores_25_mm_keeps_it_and_a_new_design_gets_the_formula() {
    // The file stores 25 mm as used (ADR 0007); the formula would give another thickness.
    let (s, _) =
        DesignSession::open(&legacy("v2_gas_lpg_before_292"), Profile::built_in()).unwrap();
    let r = results_of(&s);
    assert_eq!(r["drawing"]["tank"]["wall_thickness_source"], "override");
    assert_eq!(r["drawing"]["tank"]["cylinder_wall_thickness_mm"], 25.0);
    let new = results_of(&new_session("LPG_propane", "fully_pressurised"));
    assert_eq!(new["drawing"]["tank"]["wall_thickness_source"], "formula");
}

#[test]
fn a_file_with_the_carriage_condition_does_not_warn() {
    let s = new_session("LPG_butane", "semi_pressurised");
    let text = s.save().unwrap();
    let checked = check(&text, &Profile::built_in());
    assert_eq!(checked.report.errors, Vec::<String>::new());
    assert_eq!(checked.report.warnings, Vec::<String>::new());
}

#[test]
fn the_legacy_other_table_covers_every_text_or_none_default() {
    // The gas tank inputs only: the stern inputs of issue #347 are in their own group.
    let names: Vec<&str> = LEGACY_FIXED_OTHER
        .iter()
        .filter(|l| l.applies_to.containments() == ["type_c_tanks"])
        // The typed openings (issue #417) are in their own group.
        .filter(|l| l.name != "deckhouse_front_opening")
        .map(|l| l.name)
        .collect();
    for name in [
        "lpg_carriage_condition",
        "tank_material",
        "tank_insulation_type",
        "tank_design_temperature_c",
        "tank_allowable_stress_mpa",
    ] {
        assert!(names.contains(&name), "{name}");
    }
    // Every new input is in the input table, in the Gas tanks group, for the type C tanks.
    for name in names.iter().copied().chain(TABLE_DEFAULTS.iter().copied()) {
        let spec = INPUTS
            .iter()
            .find(|s| s.name == name)
            .unwrap_or_else(|| panic!("{name}"));
        assert_eq!(spec.group.label(), "Gas tanks", "{name}");
        assert_eq!(spec.applies_to.containments(), ["type_c_tanks"], "{name}");
    }
}

#[test]
fn an_older_profile_copy_without_the_entries_loads_and_takes_the_built_in_ones() {
    let mut profile: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    for name in [
        "lpg_carriage_condition",
        "tank_design_pressure_mpa",
        "tank_design_temperature_c",
        "tank_material",
        "tank_allowable_stress_mpa",
        "tank_insulation_type",
        "tank_insulation_thickness_mm",
        "tank_structure_factor",
    ] {
        profile["inputs"].as_object_mut().unwrap().remove(name);
    }
    // An older copy kept the fill ratio by ruleset, and had a fixed wall.
    profile["inputs"]["fill_ratio"] = json!({
        "default_by_ruleset": { "IGC_2G": 0.96 },
        "unit": "-", "hard": { "ge": 0.1, "le": 1 }, "soft": { "min": 0.8, "max": 0.99 },
        "basis": "an older copy"
    });
    profile["inputs"]["tank_wall_thickness_mm"]["default"] = json!(25.0);
    let old = Profile::from_json(&profile.to_string()).unwrap();
    let built_in = Profile::built_in();
    for name in TABLE_DEFAULTS.iter().filter(|n| **n != "fill_ratio") {
        assert_eq!(old.table_json(name), built_in.table_json(name), "{name}");
    }
    assert_eq!(
        old.default("lpg_carriage_condition"),
        built_in.default("lpg_carriage_condition")
    );
    // The copy's own fill ratio is kept, as an estimate that says where it came from.
    let hit = old
        .table_default("fill_ratio", &context("LPG_propane", "semi_pressurised"))
        .unwrap();
    assert_eq!(hit.value, &json!(0.96));
    assert_eq!(hit.status, "estimate");
    assert!(hit.basis.contains("default_by_ruleset"), "{}", hit.basis);
    // The copy's fixed wall stays the copy's default.
    assert_eq!(old.default("tank_wall_thickness_mm"), Some(&json!(25.0)));
    // And it makes a Design.
    let mut s = DesignSession::new_design(old).unwrap();
    s.set_input("Option 1", "cargo", json!("LPG_propane"))
        .unwrap();
    assert_eq!(stored(&s, "fill_ratio").unwrap().0, json!(0.96));
    assert!(stored(&s, "tank_design_pressure_mpa").is_some());
}

#[test]
fn fill_as_new_takes_the_profile_where_fill_takes_the_old_fixed_value() {
    // A hand-written file with the required inputs only. `fill` reads it as an old file (the
    // fixed tank values, results unchanged); `fill_as_new` gives it what a new Design gets.
    let text = json!({
        "format": "vdesign", "format_version": 2, "engine_version": "0.0.0",
        "options": [{ "name": "o", "inputs": {
            "cargo": "LPG_propane", "hull_form": "raked_box",
            "formation": { "kind": "self_propelled" },
            "length_overall_m": 110.0, "beam_m": 11.4, "moulded_depth_m": 6.0,
            "cargo_containment": "type_c_tanks"
        }}]
    })
    .to_string();
    let profile = Profile::built_in();
    let old = ovcd_format::fill(&text, &profile).design.unwrap();
    let new = fill_as_new(&text, &profile).design.unwrap();
    let get = |d: &ovcd_format::Design, n: &str| d.options[0].input(n).unwrap().value.clone();
    assert_eq!(get(&old, "tank_design_pressure_mpa"), json!(0.5));
    assert_eq!(get(&old, "tank_wall_thickness_mm"), json!(25.0));
    assert_eq!(get(&new, "tank_design_pressure_mpa"), json!(1.77));
    assert_eq!(get(&new, "tank_wall_thickness_mm"), Value::Null);
    assert_eq!(
        get(&new, "lpg_carriage_condition"),
        json!("fully_pressurised")
    );
}

#[test]
fn the_fill_ratio_of_a_design_is_the_igc_limit_and_the_old_0_95_is_never_used() {
    // IGC 15.3 default 98 %: the profile, the simulate fallback and a new Design agree.
    let s = new_session("LNG", "");
    assert_eq!(stored(&s, "fill_ratio").unwrap().0, json!(0.98));
    let r = results_of(&s);
    assert_eq!(r["arrangement"]["fill_ratio"], 0.98);
    let case = ovcd_format::parse_case(
        &json!({ "fn": "simulate", "args": { "params": {
            "vessel": "gas_barge", "cargo": "LNG", "loa_m": 110.0, "beam_m": 11.4, "depth_m": 6.0, "cb": 0.85,
            "self_propelled": true
        } } })
        .to_string(),
    )
    .unwrap();
    let direct = ovcd_format::eval("simulate", &case).unwrap()["ok"].clone();
    assert_eq!(direct["arrangement"]["fill_ratio"], 0.98);
}

/// The `structure_factor` of each runnable case of `calibration/tank_cases.yaml`, with its carriage
/// condition, read as plain text (no YAML parser): the two lines of a case.
fn calibration_structure_factors() -> Vec<(String, String, f64)> {
    let path = format!(
        "{}/../../calibration/tank_cases.yaml",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let runnable = text.split("not_yet_runnable_tank_cases:").next().unwrap();
    let (mut id, mut carriage, mut out) = (String::new(), String::new(), Vec::new());
    for line in runnable.lines() {
        let line = line.trim_start_matches(" - ").trim();
        if let Some(v) = line.strip_prefix("- id:") {
            id = v.trim().to_string();
            carriage.clear();
        } else if let Some(v) = line.strip_prefix("id:") {
            id = v.trim().to_string();
            carriage.clear();
        } else if let Some(v) = line.strip_prefix("carriage_condition:") {
            carriage = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("structure_factor:") {
            out.push((id.clone(), carriage.clone(), v.trim().parse().unwrap()));
        }
    }
    out
}

#[test]
fn a_tank_calibration_case_uses_the_structure_factor_of_the_profile() {
    let profile = Profile::built_in();
    let cases = calibration_structure_factors();
    assert!(cases.len() >= 2, "{cases:?}");
    for (id, carriage, factor) in cases {
        // The case's Cargo does not matter to the factor; an LPG carriage condition does.
        let cargo = if carriage.is_empty() {
            "LNG"
        } else {
            "LPG_propane"
        };
        let hit = profile
            .table_default("tank_structure_factor", &context(cargo, &carriage))
            .unwrap_or_else(|| panic!("{id}"));
        assert_eq!(hit.value, &json!(factor), "{id}");
    }
}
