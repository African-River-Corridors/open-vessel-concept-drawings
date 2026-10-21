//! Seam 4: defining an Option by a standard Vessel Class (issue #94), through the Design session,
//! `fill`/`check` and the schema. Each test states a rule the maintainer gave on the issue.
//!
//! Label: property

use ovcd_format::vdesign::Source;
use ovcd_format::{DesignSession, Profile, Scope, Severity, check, fill, input_schema};
use serde_json::{Value, json};

fn session() -> DesignSession {
    // The starter's typed 3.5 m depth is for its 76.5 m hull; a 110 m class hull would be "very
    // slender for its depth" (issue #120). A depth of 5 m keeps these tests about the class.
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    s.set_input("Option 1", "moulded_depth_m", json!(5.0))
        .unwrap();
    s
}

fn input(s: &DesignSession, name: &str) -> (Value, Source) {
    let i = s.design().options[0]
        .input(name)
        .unwrap_or_else(|| panic!("no input {name}"))
        .clone();
    (i.value, i.source)
}

fn has(s: &DesignSession, name: &str) -> bool {
    s.design().options[0].input(name).is_some()
}

fn messages(s: &DesignSession) -> Vec<(Severity, Scope, String)> {
    s.evaluate().options[0]
        .messages
        .iter()
        .map(|m| (m.severity, m.scope.clone(), m.text.clone()))
        .collect()
}

fn warned(s: &DesignSession, needle: &str) -> bool {
    messages(s)
        .iter()
        .any(|(sev, _, t)| *sev == Severity::Warning && t.contains(needle))
}

#[test]
fn a_new_design_has_typed_dimensions_and_no_class() {
    let s = session();
    assert!(!has(&s, "vessel_scheme") && !has(&s, "vessel_class"));
    assert!(s.evaluate().options[0].vessel_class.is_none());
    assert!(messages(&s).is_empty());
}

#[test]
fn picking_a_class_sets_length_beam_and_formation_to_the_class_and_marks_them() {
    let mut s = session();
    s.set_input("Option 1", "vessel_class", json!("CEMT-Va"))
        .unwrap();
    assert_eq!(input(&s, "length_overall_m"), (json!(110.0), Source::Class));
    assert_eq!(input(&s, "beam_m"), (json!(11.4), Source::Class));
    assert_eq!(
        input(&s, "formation"),
        (json!({ "kind": "self_propelled" }), Source::Class)
    );
    assert_eq!(
        input(&s, "vessel_scheme"),
        (json!("CEMT-1992"), Source::Typed)
    );
    assert_eq!(input(&s, "vessel_class"), (json!("CEMT-Va"), Source::Class));
    // Depth stays typed.
    assert_eq!(input(&s, "moulded_depth_m").1, Source::Typed);
    assert!(messages(&s).is_empty(), "{:?}", messages(&s));
}

#[test]
fn the_class_fills_the_draught_limit_with_the_top_and_the_air_draft_limit_with_the_bottom() {
    let mut s = session();
    s.set_input("Option 1", "vessel_class", json!("CEMT-Va"))
        .unwrap();
    assert_eq!(input(&s, "draught_limit_m"), (json!(2.8), Source::Class));
    assert_eq!(input(&s, "air_draft_limit_m"), (json!(5.25), Source::Class));
}

#[test]
fn a_class_that_gives_no_value_fills_nothing() {
    let mut s = session();
    s.set_input("Option 1", "vessel_class", json!("CN-II-MOTOR"))
        .unwrap();
    assert_eq!(input(&s, "draught_limit_m"), (json!(2.6), Source::Class)); // a scalar as it is
    assert_eq!(
        input(&s, "air_draft_limit_m"),
        (Value::Null, Source::Default)
    ); // null fills nothing
}

#[test]
fn a_convoy_class_sets_the_formation_and_its_counts() {
    let mut s = session();
    s.set_input("Option 1", "vessel_class", json!("CEMT-VIa"))
        .unwrap();
    assert_eq!(
        input(&s, "formation"),
        (
            json!({ "kind": "pushed_convoy", "abreast": 2, "in_line": 1 }),
            Source::Class
        )
    );
    assert_eq!(input(&s, "beam_m"), (json!(22.8), Source::Class));
    s.set_input("Option 1", "vessel_class", json!("CN-I-CONVOY-2x2"))
        .unwrap();
    assert_eq!(
        input(&s, "formation").0,
        json!({ "kind": "pushed_convoy", "abreast": 2, "in_line": 2 })
    );
}

#[test]
fn the_class_is_decided_by_length_and_beam_and_the_vessel_moves_up_and_down() {
    let mut s = session();
    s.set_input("Option 1", "vessel_class", json!("CEMT-Va"))
        .unwrap();
    // Longer than Va's 110 m: up a class. The edited input is typed; the other stays the class's.
    s.set_input("Option 1", "length_overall_m", json!(120.0))
        .unwrap();
    let up = input(&s, "vessel_class").0;
    assert_ne!(up, json!("CEMT-Va"));
    assert_eq!(input(&s, "length_overall_m"), (json!(120.0), Source::Typed));
    assert_eq!(input(&s, "beam_m").1, Source::Class);
    // Back below Va's maximum: down again.
    s.set_input("Option 1", "length_overall_m", json!(100.0))
        .unwrap();
    assert_eq!(input(&s, "vessel_class").0, json!("CEMT-Va"));
    // Shorter than the next class down's maximum: down.
    s.set_input("Option 1", "length_overall_m", json!(80.0))
        .unwrap();
    s.set_input("Option 1", "beam_m", json!(9.0)).unwrap();
    assert_eq!(input(&s, "vessel_class").0, json!("CEMT-IV"));
    // There is no modified label: the class is always the one the vessel sits in.
    assert!(
        !messages(&s)
            .iter()
            .any(|(_, _, t)| t.to_lowercase().contains("modified"))
    );
}

#[test]
fn the_limits_follow_the_class_the_vessel_moves_to() {
    let mut s = session();
    s.set_input("Option 1", "vessel_class", json!("CEMT-Va"))
        .unwrap();
    s.set_input("Option 1", "length_overall_m", json!(85.0))
        .unwrap();
    s.set_input("Option 1", "beam_m", json!(9.5)).unwrap();
    assert_eq!(input(&s, "vessel_class").0, json!("CEMT-IV"));
    assert_eq!(input(&s, "draught_limit_m"), (json!(2.5), Source::Class));
}

#[test]
fn past_the_largest_class_of_the_scheme_there_is_a_warning_and_the_scheme_stays() {
    let mut s = session();
    s.set_input("Option 1", "vessel_class", json!("CEMT-Va"))
        .unwrap();
    s.set_input("Option 1", "length_overall_m", json!(400.0))
        .unwrap();
    assert!(
        warned(&s, "beyond the largest CEMT-1992 class"),
        "{:?}",
        messages(&s)
    );
    assert_eq!(input(&s, "vessel_scheme").0, json!("CEMT-1992"));
    let summary = s.evaluate().options[0].vessel_class.clone().unwrap();
    assert_eq!(summary["beyond_largest"], json!(true));
}

#[test]
fn going_back_to_typed_dimensions_keeps_the_values_as_typed() {
    let mut s = session();
    s.set_input("Option 1", "vessel_class", json!("CEMT-Va"))
        .unwrap();
    s.set_input("Option 1", "vessel_scheme", Value::Null)
        .unwrap();
    assert!(!has(&s, "vessel_scheme") && !has(&s, "vessel_class"));
    assert_eq!(input(&s, "length_overall_m"), (json!(110.0), Source::Typed));
    assert_eq!(input(&s, "beam_m"), (json!(11.4), Source::Typed));
    assert_eq!(input(&s, "draught_limit_m"), (json!(2.8), Source::Typed));
    assert!(s.evaluate().options[0].vessel_class.is_none());
}

#[test]
fn picking_a_scheme_keeps_the_dimensions_and_places_the_vessel() {
    let mut s = session(); // the starter: a 76.5 x 11.4 m pushed barge
    s.set_input("Option 1", "vessel_scheme", json!("US-inland"))
        .unwrap();
    assert_eq!(input(&s, "length_overall_m").1, Source::Typed);
    let class = input(&s, "vessel_class");
    assert_eq!(class.1, Source::Class);
    let id = class.0.as_str().unwrap().to_string();
    let c = ovcd_core::vessel_classes::find(&id).unwrap();
    assert_eq!(c.family, "US-inland");
    assert_eq!(id, "US-SUPER-JUMBO-HOPPER"); // the smallest US class holding 76.5 x 11.4 m
    assert_eq!(c.kind(), Some("pushed_barge"));
    // Another scheme replaces it; nothing else changes by itself.
    s.set_input("Option 1", "vessel_scheme", json!("Europa-barge"))
        .unwrap();
    assert_eq!(input(&s, "vessel_scheme").0, json!("Europa-barge"));
    assert_eq!(input(&s, "length_overall_m"), (json!(76.5), Source::Typed));
}

#[test]
fn a_scheme_with_no_class_of_the_formation_kind_warns_and_holds_no_class() {
    let mut s = session(); // a pushed barge; CEMT-1992 has only motor vessels and convoys
    s.set_input("Option 1", "vessel_scheme", json!("CEMT-1992"))
        .unwrap();
    assert!(!has(&s, "vessel_class"));
    assert!(warned(&s, "has no class for a pushed_barge Formation"));
}

#[test]
fn a_typed_limit_stays_and_a_warning_says_when_it_is_above_the_class() {
    let mut s = session();
    s.set_input("Option 1", "draught_limit_m", json!(3.5))
        .unwrap();
    s.set_input("Option 1", "air_draft_limit_m", json!(8.0))
        .unwrap();
    s.set_input("Option 1", "vessel_class", json!("CEMT-Va"))
        .unwrap();
    assert_eq!(input(&s, "draught_limit_m"), (json!(3.5), Source::Typed));
    assert_eq!(input(&s, "air_draft_limit_m"), (json!(8.0), Source::Typed));
    assert!(warned(
        &s,
        "`draught_limit_m` = 3.5 is above the draught of class CEMT-Va"
    ));
    assert!(warned(
        &s,
        "`air_draft_limit_m` = 8 is above the lowest bridge clearance of class CEMT-Va"
    ));
    // Typing a limit at or under the class's gives no warning.
    s.set_input("Option 1", "draught_limit_m", json!(2.8))
        .unwrap();
    assert!(!warned(&s, "`draught_limit_m`"));
}

#[test]
fn the_summary_gives_the_class_and_its_reference_deadweight_for_display() {
    let mut s = session();
    s.set_input("Option 1", "vessel_class", json!("CEMT-Va"))
        .unwrap();
    let v = s.evaluate().options[0].vessel_class.clone().unwrap();
    assert_eq!(v["id"], "CEMT-Va");
    assert_eq!(v["scheme"], "CEMT-1992");
    assert_eq!(v["reference_dwt_t"], json!([1500.0, 3000.0]));
    assert_eq!(v["beyond_largest"], json!(false));
    // It reaches the front end's JSON, beside the outcome, and is not part of the results.
    let j = s.evaluate().to_json();
    assert_eq!(j["options"][0]["vessel_class"]["id"], "CEMT-Va");
    assert!(
        j["options"][0]["outcome"]["results"]
            .get("vessel_class")
            .is_none()
    );
}

#[test]
fn a_class_does_not_change_the_results_of_the_same_dimensions() {
    // "No calculation result changes for a given set of dimensions."
    let mut picked = session();
    picked
        .set_input("Option 1", "vessel_class", json!("CEMT-Va"))
        .unwrap();
    let mut typed = session();
    typed
        .set_input("Option 1", "vessel_class", json!("CEMT-Va"))
        .unwrap();
    typed
        .set_input("Option 1", "vessel_scheme", Value::Null)
        .unwrap();
    let hash = |s: &DesignSession| match &s.evaluate().options[0].outcome {
        Some(ovcd_format::RunOutcome::Results { hash, .. }) => hash.clone(),
        other => panic!("{other:?}"),
    };
    assert_eq!(hash(&picked), hash(&typed));
}

#[test]
fn the_class_survives_save_and_open_and_the_markers_are_kept() {
    let mut s = session();
    s.set_input("Option 1", "vessel_class", json!("CEMT-VIa"))
        .unwrap();
    let text = s.save().unwrap();
    let saved: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(saved["format_version"], 3);
    let inputs = &saved["options"][0]["inputs"];
    assert_eq!(
        inputs["vessel_scheme"],
        json!({ "value": "CEMT-1992", "source": "typed" })
    );
    assert_eq!(
        inputs["vessel_class"],
        json!({ "value": "CEMT-VIa", "source": "class" })
    );
    assert_eq!(inputs["beam_m"]["source"], "class");
    let (back, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
    assert!(opened.report.is_ok(), "{:?}", opened.report);
    assert!(
        opened.report.warnings.is_empty(),
        "{:?}",
        opened.report.warnings
    );
    assert_eq!(back.design().to_text(), text);
    assert!(
        check(&text, &Profile::built_in())
            .report
            .warnings
            .is_empty()
    );
}

// ---- files ---------------------------------------------------------------------------------

fn file(inputs: Value) -> String {
    json!({ "format": "vdesign", "format_version": 2, "engine_version": "0.1.0",
            "options": [{ "name": "A", "inputs": inputs }] })
    .to_string()
}

fn base() -> Value {
    json!({ "cargo": "bauxite", "hull_form": "raked_box", "moulded_depth_m": 3.5 })
}

#[test]
fn fill_writes_length_beam_and_formation_from_a_class_a_hand_written_file_names() {
    let mut inputs = base();
    inputs["vessel_class"] = json!("CEMT-VIa");
    let filled = fill(&file(inputs), &Profile::built_in());
    assert!(filled.report.is_ok(), "{:?}", filled.report);
    let o = &filled.design.unwrap().options[0];
    assert_eq!(o.input("length_overall_m").unwrap().value, json!(110.0));
    assert_eq!(o.input("length_overall_m").unwrap().source, Source::Class);
    assert_eq!(o.input("beam_m").unwrap().value, json!(22.8));
    assert_eq!(
        o.input("formation").unwrap().value,
        json!({ "kind": "pushed_convoy", "abreast": 2, "in_line": 1 })
    );
    assert_eq!(o.input("vessel_scheme").unwrap().value, json!("CEMT-1992"));
    assert_eq!(o.input("draught_limit_m").unwrap().value, json!(4.5));
}

#[test]
fn check_does_not_write_them_in_and_says_the_inputs_are_missing() {
    let mut inputs = base();
    inputs["vessel_class"] = json!("CEMT-VIa");
    let r = check(&file(inputs), &Profile::built_in()).report;
    assert!(
        r.errors
            .iter()
            .any(|e| e.contains("missing required input `length_overall_m`")),
        "{r:?}"
    );
}

#[test]
fn a_typed_length_with_a_class_keeps_the_length_and_the_class_follows_it() {
    let mut inputs = base();
    inputs["vessel_class"] = json!("CEMT-Va");
    inputs["length_overall_m"] = json!(120.0);
    let filled = fill(&file(inputs), &Profile::built_in());
    let o = &filled.design.unwrap().options[0];
    assert_eq!(o.input("length_overall_m").unwrap().value, json!(120.0));
    assert_ne!(o.input("vessel_class").unwrap().value, json!("CEMT-Va"));
    assert!(
        filled
            .report
            .warnings
            .iter()
            .any(|w| w.contains("`vessel_class` CEMT-Va does not hold")),
        "{:?}",
        filled.report.warnings
    );
}

#[test]
fn an_old_id_names_its_class() {
    let mut inputs = base();
    inputs["vessel_class"] = json!("CEMT_Va");
    let o = fill(&file(inputs), &Profile::built_in()).design.unwrap();
    assert_eq!(
        o.options[0].input("vessel_class").unwrap().value,
        json!("CEMT-Va")
    );
}

#[test]
fn bad_class_and_scheme_values_are_hard_errors() {
    let run = |class: Value, scheme: Value| {
        let mut inputs = base();
        inputs["vessel_class"] = class;
        inputs["vessel_scheme"] = scheme;
        inputs["length_overall_m"] = json!(100.0);
        inputs["beam_m"] = json!(11.0);
        inputs["formation"] = json!({ "kind": "self_propelled" });
        fill(&file(inputs), &Profile::built_in()).report.errors
    };
    let e = run(json!("NO-SUCH"), Value::Null);
    assert!(
        e.iter().any(|m| m.contains("is not a Vessel Class")),
        "{e:?}"
    );
    let e = run(json!("CN-V"), Value::Null); // no length or beam
    assert!(
        e.iter().any(|m| m.contains("cannot define a vessel")),
        "{e:?}"
    );
    let e = run(Value::Null, json!("NO-SUCH"));
    assert!(e.iter().any(|m| m.contains("not a scheme")), "{e:?}");
    let e = run(json!("CEMT-Va"), json!("RWS-2010"));
    assert!(
        e.iter()
            .any(|m| m.contains("belongs to the scheme CEMT-1992, not RWS-2010")),
        "{e:?}"
    );
    let e = run(json!(5), Value::Null);
    assert!(
        e.iter().any(|m| m.contains("must be a class id or null")),
        "{e:?}"
    );
}

#[test]
fn a_version_1_file_has_no_class_inputs_and_opens_unchanged() {
    let v1 = json!({ "format": "vdesign", "format_version": 1, "engine_version": "0.1.0",
        "options": [{ "name": "A", "inputs": {
            "cargo": "bauxite", "hull_form": "raked_box", "moulded_depth_m": 3.5,
            "length_overall_m": 76.5, "beam_m": 11.4, "formation": { "kind": "pushed_barge" } } }] });
    let r = fill(&v1.to_string(), &Profile::built_in());
    assert!(r.report.is_ok(), "{:?}", r.report);
    assert!(r.design.unwrap().options[0].input("vessel_class").is_none());
}

#[test]
fn a_class_source_without_a_scheme_is_read_as_typed() {
    let mut inputs = base();
    inputs["length_overall_m"] = json!({ "value": 100.0, "source": "class" });
    inputs["beam_m"] = json!(11.0);
    inputs["formation"] = json!({ "kind": "self_propelled" });
    let o = fill(&file(inputs), &Profile::built_in()).design.unwrap();
    assert_eq!(
        o.options[0].input("length_overall_m").unwrap().source,
        Source::Typed
    );
}

// ---- schema --------------------------------------------------------------------------------

#[test]
fn the_schema_lists_the_schemes_and_the_pickable_classes_with_what_picking_sets() {
    let schema = input_schema(&Profile::built_in());
    let get = |name: &str| {
        schema["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["name"] == name)
            .unwrap()
            .clone()
    };
    let scheme = get("vessel_scheme");
    assert_eq!(scheme["kind"], "vessel_scheme");
    assert_eq!(scheme["nullable"], json!(true));
    assert!(scheme.get("default").is_none());
    let schemes: Vec<&str> = scheme["choices"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["value"].as_str().unwrap())
        .collect();
    assert!(schemes.contains(&"CEMT-1992") && schemes.contains(&"RWS-2010"));
    let class = get("vessel_class");
    assert_eq!(class["kind"], "vessel_class");
    let va = class["choices"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["value"] == "CEMT-Va")
        .unwrap();
    assert_eq!(va["scheme"], "CEMT-1992");
    assert_eq!(va["formation"], "self_propelled");
    assert_eq!(va["length_max_m"], json!(110.0));
    assert_eq!(va["beam_max_m"], json!(11.4));
    assert_eq!(va["draught_limit_m"], json!(2.8));
    assert_eq!(va["air_draft_limit_m"], json!(5.25));
    assert_eq!(va["reference_dwt_t"], json!([1500.0, 3000.0]));
    let convoy = class["choices"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["value"] == "CEMT-VIa")
        .unwrap();
    assert_eq!(
        (convoy["abreast"].as_i64(), convoy["in_line"].as_i64()),
        (Some(2), Some(1))
    );
    // A class that cannot define a vessel is not offered. Cargo does not filter the list.
    assert!(
        class["choices"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["value"] != "CN-V")
    );
    assert!(
        class["choices"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c.get("cargo_modes").is_none())
    );
}
