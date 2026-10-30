//! Seam: the stern and propulsion inputs of spec 0007 in format version 3 (issue #347). Inputs
//! only: a file saved before gets the values that reproduce today's stern block, and no result
//! moves; a new Option takes the Defaults Profile values, by Cargo phase where spec 0007 says so.
//!
//! Label: property. Each test states a rule: the schema, the legacy values, the new Option's
//! defaults, the conditions between inputs, the status of each default, the version written.

use ovd_format::vdesign::{FORMAT_VERSION, INPUTS};
use ovd_format::{
    DesignSession, HashStatus, Profile, Source, check, eval, fill_as_new, input_schema, parse_case,
};
use serde_json::{Value, json};

/// The inputs #347 adds, in table order.
const NEW: &[&str] = &[
    "stern_form",
    "stern_run_length_m",
    "stern_transom_height_m",
    "tunnel_crown_height_m",
    "propulsor_type",
    "propulsor_count",
    "propeller_diameter_m",
    "power_source",
    "installed_power_kw",
    "design_speed_kn",
    "installed_power_margin",
    "bow_thruster",
    "bow_thruster_power_kw",
    "battery_energy_kwh",
    "battery_unit_energy_kwh",
    "wheelhouse_type",
    "sightline_method",
    "stern_block_mass_included",
];

fn legacy(name: &str) -> String {
    let path = format!("{}/tests/legacy/{name}.vdesign", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn hand(cargo: &str, formation: &str) -> String {
    json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.3", "units": "SI",
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

#[test]
fn the_schema_lists_every_new_input_with_its_group_and_default_status() {
    let schema = input_schema(&Profile::built_in());
    let inputs = schema["inputs"].as_array().unwrap();
    let entry = |n: &str| {
        inputs
            .iter()
            .find(|i| i["name"] == n)
            .unwrap_or_else(|| panic!("{n} is in the schema"))
    };
    for name in NEW {
        let e = entry(name);
        assert_eq!(e["group"], "Stern and propulsion", "{name}");
        assert_eq!(
            e["applies_to_formation"],
            json!(["self_propelled"]),
            "{name}"
        );
        let status = e["default_status"].as_str().unwrap_or_default();
        assert!(
            ["sourced", "estimate"].contains(&status),
            "{name}: {status}"
        );
        if status == "sourced" {
            assert!(!e["default_source"].as_str().unwrap().is_empty(), "{name}");
        }
        assert!(!Profile::built_in().basis(name).is_empty(), "{name}");
        if e["kind"] == "number" {
            assert!(e.get("hard").is_some(), "{name}: a hard range");
        }
    }
    // The conditions between inputs.
    for (name, by) in [
        ("tunnel_crown_height_m", "stern_form"),
        ("propulsor_count", "propulsor_type"),
        ("propeller_diameter_m", "propulsor_type"),
        ("bow_thruster_power_kw", "bow_thruster"),
        ("battery_energy_kwh", "power_source"),
        ("battery_unit_energy_kwh", "power_source"),
        ("thruster_count", "propulsor_type"),
        ("propulsive_efficiency", "propulsor_type"),
    ] {
        assert_eq!(entry(name)["applies_when"]["input"], by, "{name}");
    }
    assert_eq!(
        entry("thruster_count")["applies_when"]["values"],
        json!(["unspecified"])
    );
    // A controlling input comes before the inputs it decides.
    let pos = |n: &str| INPUTS.iter().position(|s| s.name == n).unwrap();
    for spec in INPUTS {
        if let Some((by, _)) = spec.when {
            assert!(pos(by) < pos(spec.name), "{} after {by}", spec.name);
        }
    }
}

#[test]
fn a_version_2_file_gets_the_legacy_values_with_one_warning_and_keeps_its_results() {
    let text = legacy("v2_gas_lpg_before_313");
    let checked = check(&text, &Profile::built_in());
    assert!(
        checked.report.errors.is_empty(),
        "{:?}",
        checked.report.errors
    );
    let design = checked.design.unwrap();
    for (name, want) in [
        ("stern_form", json!("transom")),
        ("stern_run_length_m", json!(0.0)),
        ("stern_transom_height_m", Value::Null),
        ("propulsor_type", json!("unspecified")),
        ("power_source", json!("battery_electric")),
        ("installed_power_kw", Value::Null),
        ("design_speed_kn", Value::Null),
        ("installed_power_margin", Value::Null),
        ("bow_thruster", json!("none")),
        ("battery_energy_kwh", Value::Null),
        ("battery_unit_energy_kwh", json!(5000.0)),
        ("wheelhouse_type", json!("elevating")),
        ("sightline_method", json!("margin")),
        ("stern_block_mass_included", json!(false)),
    ] {
        assert_eq!(
            value(&design, name),
            Some((want, Source::Default)),
            "{name}"
        );
    }
    // An unspecified propulsor keeps today's thruster count and efficiency; the inputs that need a
    // tunnel, a specified propulsor or a bow thruster are not held.
    assert!(value(&design, "thruster_count").is_some());
    assert!(value(&design, "propulsive_efficiency").is_some());
    for name in [
        "tunnel_crown_height_m",
        "propulsor_count",
        "propeller_diameter_m",
        "bow_thruster_power_kw",
    ] {
        assert!(value(&design, name).is_none(), "{name}");
    }
    let notes: Vec<&String> = checked
        .report
        .warnings
        .iter()
        .filter(|w| w.contains("`stern_form`"))
        .collect();
    assert_eq!(notes.len(), 1, "{:?}", checked.report.warnings);
    assert!(notes[0].contains("results are unchanged"), "{}", notes[0]);
    let (_, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
}

#[test]
fn a_new_option_takes_the_profile_values_by_cargo_phase() {
    let p = Profile::built_in();
    for (cargo, stern, propulsor, power) in [
        ("LNG", "transom", "azimuth_ducted", "gas_fuelled"),
        ("bauxite", "tunnel", "shaft_nozzle", "diesel_mechanical"),
    ] {
        let filled = fill_as_new(&hand(cargo, "self_propelled"), &p);
        assert!(
            filled.report.errors.is_empty(),
            "{:?}",
            filled.report.errors
        );
        let d = filled.design.unwrap();
        let v = |n: &str| value(&d, n).map(|(v, _)| v);
        assert_eq!(v("stern_form"), Some(json!(stern)), "{cargo}");
        assert_eq!(v("propulsor_type"), Some(json!(propulsor)), "{cargo}");
        assert_eq!(v("power_source"), Some(json!(power)), "{cargo}");
        assert_eq!(v("propulsor_count"), Some(json!(2.0)), "{cargo}");
        assert_eq!(v("bow_thruster"), Some(json!("channel_jet")), "{cargo}");
        assert_eq!(v("design_speed_kn"), Some(json!(9.7)), "{cargo}");
        assert_eq!(v("wheelhouse_eye_height_m"), Some(json!(1.65)), "{cargo}");
        assert_eq!(
            v("sightline_method"),
            Some(json!("blind_distance")),
            "{cargo}"
        );
        assert_eq!(v("stern_block_mass_included"), Some(json!(true)), "{cargo}");
        // A specified propulsor: the thruster count and the fixed efficiency are not used.
        assert_eq!(v("thruster_count"), None, "{cargo}");
        assert_eq!(v("propulsive_efficiency"), None, "{cargo}");
        // Defaults coherence: the filled Option validates with no warning of its own.
        let text = d.to_text();
        let again = check(&text, &p);
        assert!(again.report.errors.is_empty(), "{:?}", again.report.errors);
        assert_eq!(again.report.warnings, Vec::<String>::new(), "{cargo}");
    }
}

#[test]
fn a_pushed_barge_holds_none_of_them() {
    let filled = fill_as_new(&hand("LNG", "pushed_barge"), &Profile::built_in());
    let d = filled.design.unwrap();
    for name in NEW {
        assert!(value(&d, name).is_none(), "{name}");
    }
    // The fixed efficiency still applies to a pushed barge: no propulsor decides it.
    assert!(value(&d, "propulsive_efficiency").is_some());
}

#[test]
fn the_propulsor_type_decides_which_inputs_are_held() {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    s.set_input("Option 1", "cargo", json!("LNG")).unwrap();
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    let has = |s: &DesignSession, n: &str| s.design().options[0].input(n).is_some();
    assert!(has(&s, "propulsor_count") && !has(&s, "thruster_count"));
    assert!(
        s.set_input("Option 1", "thruster_count", json!(2.0))
            .is_err(),
        "not used with a ducted azimuth"
    );
    s.set_input("Option 1", "propulsor_type", json!("unspecified"))
        .unwrap();
    assert!(has(&s, "thruster_count") && has(&s, "propulsive_efficiency"));
    assert!(!has(&s, "propulsor_count") && !has(&s, "propeller_diameter_m"));
    // A tunnel stern brings its crown height; a fitted bow thruster its power.
    s.set_input("Option 1", "stern_form", json!("tunnel"))
        .unwrap();
    assert!(has(&s, "tunnel_crown_height_m"));
    s.set_input("Option 1", "bow_thruster", json!("none"))
        .unwrap();
    assert!(!has(&s, "bow_thruster_power_kw"));
    s.set_input("Option 1", "power_source", json!("battery_hybrid"))
        .unwrap();
    assert!(has(&s, "battery_energy_kwh"));
}

#[test]
fn saving_writes_version_3_and_the_saved_file_validates() {
    assert_eq!(FORMAT_VERSION, 3);
    let (s, _) =
        DesignSession::open(&legacy("v2_gas_lpg_before_313"), Profile::built_in()).unwrap();
    let text = s.save().unwrap();
    let saved: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(saved["format_version"], 3);
    let again = check(&text, &Profile::built_in());
    assert!(again.report.errors.is_empty());
    // The legacy note is gone (the file keeps only its own S690QL1 stress warning of #313).
    assert!(
        !again
            .report
            .warnings
            .iter()
            .any(|w| w.contains("`stern_form`")),
        "{:?}",
        again.report.warnings
    );
}

#[test]
fn a_profile_default_that_claims_a_source_must_give_it() {
    let path = format!("{}/src/builtin_profile.json", env!("CARGO_MANIFEST_DIR"));
    let mut p: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    p["inputs"]["bow_thruster"]
        .as_object_mut()
        .unwrap()
        .remove("source");
    let e = Profile::from_json(&p.to_string()).unwrap_err();
    assert!(
        e.contains("`bow_thruster` says its default is sourced"),
        "{e}"
    );
    p["inputs"]["bow_thruster"]["status"] = json!("guess");
    let e = Profile::from_json(&p.to_string()).unwrap_err();
    assert!(e.contains("must be one of sourced, estimate"), "{e}");
}

#[test]
fn the_stern_block_api_takes_power_source_as_the_name_of_propulsion() {
    let run = |inputs: Value| {
        let text = json!({"fn": "compute", "args": {"inputs": inputs}}).to_string();
        eval("aft_arrangement", &parse_case(&text).unwrap())
    };
    let base = json!({"beam": 11.4, "depth": 6.0});
    let mut a = base.clone();
    a["propulsion"] = json!("gas_fuelled");
    let mut b = base.clone();
    b["power_source"] = json!("gas_fuelled");
    assert_eq!(run(a.clone()).unwrap(), run(b).unwrap());
    let mut c = base.clone();
    c["power_source"] = json!("diesel_mechanical");
    assert!(run(c).is_err());
    a["power_source"] = json!("battery_electric");
    assert!(run(a).is_err(), "the two names disagree");
}
