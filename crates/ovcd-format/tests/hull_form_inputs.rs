//! Seam: the Hull Form inputs of spec 0008 in format version 3 (issue #446). Inputs only: a file
//! saved before gets the values that keep today's hull, and no result moves; a Hull Form with no
//! generator yet is refused.
//!
//! Label: property. Each test states a rule: the schema, the legacy values, the new Option's
//! defaults, the conditions on the Hull Form, the refusal, the shapes of the object inputs.

use ovcd_format::vdesign::{HULL_FORMS, INPUTS};
use ovcd_format::{DesignSession, HashStatus, Profile, Source, check, fill_as_new, input_schema};
use serde_json::{Value, json};

/// The inputs #446 adds, in table order, and the Hull Forms each applies to.
const NEW: &[(&str, &[&str])] = &[
    (
        "bow_rake_length_m",
        &["box", "raked_box", "v_bottom", "chine"],
    ),
    (
        "bow_rake_rise_m",
        &["box", "raked_box", "v_bottom", "chine"],
    ),
    ("bilge_radius_m", &["box", "raked_box", "chine"]),
    (
        "sheer_aft_m",
        &["box", "raked_box", "v_bottom", "chine", "shipd"],
    ),
    (
        "sheer_fwd_m",
        &["box", "raked_box", "v_bottom", "chine", "shipd"],
    ),
    (
        "camber_m",
        &["box", "raked_box", "v_bottom", "chine", "shipd"],
    ),
    ("deadrise_deg", &["v_bottom", "chine"]),
    ("keel_half_width_m", &["v_bottom", "chine"]),
    ("chine_half_breadth_m", &["chine"]),
    ("chine_height_m", &["chine"]),
    ("shipd_parameters", &["shipd"]),
    ("hull_offsets", &["offsets"]),
];

/// The inputs a raked box holds.
const RAKED: &[&str] = &[
    "bow_rake_length_m",
    "bow_rake_rise_m",
    "bilge_radius_m",
    "sheer_aft_m",
    "sheer_fwd_m",
    "camber_m",
];

fn legacy(name: &str) -> String {
    let path = format!("{}/tests/legacy/{name}.vdesign", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn hand(hull_form: &str) -> Value {
    json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.3", "units": "SI",
        "defaults_profile": "built-in",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": "bauxite", "source": "typed" },
            "hull_form": { "value": hull_form, "source": "typed" },
            "formation": { "value": { "kind": "pushed_barge" }, "source": "typed" },
            "length_overall_m": { "value": 76.5, "source": "typed" },
            "beam_m": { "value": 11.4, "source": "typed" },
            "moulded_depth_m": { "value": 3.5, "source": "typed" }
        }}]
    })
}

fn value(design: &ovcd_format::Design, name: &str) -> Option<(Value, Source)> {
    design.options[0]
        .input(name)
        .map(|i| (i.value.clone(), i.source))
}

#[test]
fn the_schema_lists_every_new_input_with_its_group_condition_and_default_status() {
    let schema = input_schema(&Profile::built_in());
    let inputs = schema["inputs"].as_array().unwrap();
    let entry = |n: &str| {
        inputs
            .iter()
            .find(|i| i["name"] == n)
            .unwrap_or_else(|| panic!("{n} is in the schema"))
    };
    assert_eq!(entry("hull_form")["group"], "Hull Form");
    let choices: Vec<&str> = entry("hull_form")["choices"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["value"].as_str().unwrap())
        .collect();
    assert_eq!(choices, HULL_FORMS);
    for (name, forms) in NEW {
        let e = entry(name);
        assert_eq!(e["group"], "Hull Form", "{name}");
        assert_eq!(e["applies_when"]["input"], "hull_form", "{name}");
        assert_eq!(e["applies_when"]["values"], json!(forms), "{name}");
        assert_eq!(e["default_status"], "estimate", "{name}");
        assert!(!Profile::built_in().basis(name).is_empty(), "{name}");
        if e["kind"] == "number" {
            assert!(e.get("hard").is_some(), "{name}: a hard range");
            assert!(e.get("soft").is_some(), "{name}: a soft range");
            assert!(e.get("unit").is_some(), "{name}: a unit");
        }
    }
    // The two object inputs give their shape.
    let p = entry("shipd_parameters");
    assert_eq!(p["kind"], "parameter_set");
    assert_eq!(p["shape"], json!({ "type": "object", "values": "number" }));
    let t = entry("hull_offsets");
    assert_eq!(t["kind"], "offsets_table");
    assert_eq!(t["shape"]["columns"], json!(["station_x_m", "y_m", "z_m"]));
    // The Hull Form comes before the inputs it decides.
    let pos = |n: &str| INPUTS.iter().position(|s| s.name == n).unwrap();
    for (name, _) in NEW {
        assert!(pos("hull_form") < pos(name), "{name}");
    }
}

#[test]
fn a_version_2_file_keeps_its_hull_and_its_results_with_one_warning() {
    for file in ["v2_gas_lpg_before_313", "v2_gas_lng_before_292"] {
        let text = legacy(file);
        let checked = check(&text, &Profile::built_in());
        assert!(
            checked.report.errors.is_empty(),
            "{file}: {:?}",
            checked.report.errors
        );
        let design = checked.design.unwrap();
        for (name, want) in [
            ("bow_rake_length_m", Value::Null),
            ("bow_rake_rise_m", Value::Null),
            ("bilge_radius_m", json!(0.0)),
            ("sheer_aft_m", json!(0.0)),
            ("sheer_fwd_m", json!(0.0)),
            ("camber_m", json!(0.0)),
        ] {
            assert_eq!(
                value(&design, name),
                Some((want, Source::Default)),
                "{file}: {name}"
            );
        }
        for (name, _) in &NEW[6..] {
            assert!(value(&design, name).is_none(), "{file}: {name}");
        }
        let notes: Vec<&String> = checked
            .report
            .warnings
            .iter()
            .filter(|w| w.contains("`bow_rake_length_m`"))
            .collect();
        assert_eq!(notes.len(), 1, "{file}: {:?}", checked.report.warnings);
        assert!(notes[0].contains("results are unchanged"), "{}", notes[0]);
        let (_, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
        assert_eq!(opened.hash_status[0].1, HashStatus::Matches, "{file}");
    }
}

#[test]
fn a_new_raked_box_option_takes_the_profile_values_and_validates_with_no_warning() {
    let p = Profile::built_in();
    let filled = fill_as_new(&hand("raked_box").to_string(), &p);
    assert!(
        filled.report.errors.is_empty(),
        "{:?}",
        filled.report.errors
    );
    let d = filled.design.unwrap();
    for name in RAKED {
        let (v, source) = value(&d, name).unwrap_or_else(|| panic!("{name} is held"));
        assert_eq!(source, Source::Default, "{name}");
        assert_eq!(Some(&v), p.default(name), "{name}");
    }
    for (name, _) in &NEW[6..] {
        assert!(value(&d, name).is_none(), "{name}");
    }
    // Defaults coherence: the filled Option validates with no warning of its own.
    let again = check(&d.to_text(), &p);
    assert!(again.report.errors.is_empty(), "{:?}", again.report.errors);
    assert_eq!(again.report.warnings, Vec::<String>::new());
}

#[test]
fn every_profile_default_of_a_hull_form_parameter_is_inside_its_ranges() {
    let p = Profile::built_in();
    for (name, _) in NEW {
        let Some(Value::Number(n)) = p.default(name) else {
            continue;
        };
        let v = n.as_f64().unwrap();
        let (hard, soft) = (p.hard(name).unwrap(), p.soft(name).unwrap().unwrap());
        assert!(
            hard.breach(v).is_none(),
            "{name} = {v} is outside its hard range"
        );
        assert!(
            soft.min <= v && v <= soft.max,
            "{name} = {v} is outside its soft range"
        );
    }
}

#[test]
fn a_hull_form_with_no_generator_yet_gets_the_not_built_message() {
    for form in ["shipd", "box"] {
        let checked = check(&hand(form).to_string(), &Profile::built_in());
        assert!(
            checked
                .report
                .errors
                .iter()
                .any(|e| e.contains("`hull_form`") && e.contains("is not built yet")),
            "{form}: {:?}",
            checked.report.errors
        );
    }
}

#[test]
fn the_object_inputs_take_their_shape_or_null() {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    s.set_input("Option 1", "cargo", json!("bauxite")).unwrap();
    // Not held by a raked box: setting one is refused.
    assert!(
        s.set_input("Option 1", "hull_offsets", json!(null))
            .is_err()
    );
    let spec = |n: &str| INPUTS.iter().find(|s| s.name == n).unwrap();
    let p = Profile::built_in();
    let ok = |n: &str, v: Value| ovcd_format::vdesign::check_input_value(spec(n), &v, &p).is_ok();
    assert!(ok("shipd_parameters", json!(null)));
    assert!(ok("shipd_parameters", json!({ "LOA": 10.0, "Bd": 0.2 })));
    assert!(!ok("shipd_parameters", json!({ "LOA": "ten" })));
    assert!(!ok("shipd_parameters", json!([1.0, 2.0])));
    assert!(ok("hull_offsets", json!(null)));
    assert!(ok(
        "hull_offsets",
        json!([
            { "station_x_m": 0.0, "y_m": 0.0, "z_m": 0.0 },
            { "station_x_m": 0.0, "y_m": 5.7, "z_m": 0.0 }
        ])
    ));
    assert!(!ok(
        "hull_offsets",
        json!([{ "station_x_m": 0.0, "y_m": 0.0 }])
    ));
    assert!(!ok(
        "hull_offsets",
        json!([{ "station_x_m": 0.0, "y_m": 0.0, "z_m": 0.0, "w": 1.0 }])
    ));
    assert!(!ok("hull_offsets", json!({ "station_x_m": 0.0 })));
}
