//! Seam 4: the Design session. A Design and edits in; the Design state, messages, results and
//! saved text out. Each test uses only the session's public interface.

use ovcd_format::vdesign::{INPUTS, InputKind, Source};
use ovcd_format::{
    DesignSession, HashStatus, Profile, RunOutcome, Scope, Severity, check, input_schema,
    new_design,
};
use serde_json::{Value, json};

const FIXTURES: &[&str] = &[
    "dry_bulk_box_hold",
    "dry_bulk_sloped_hopper",
    "gas_igc_2g_exact",
    "gas_igc_2g_self_propelled",
    "two_options",
    "pushed_convoy",
];

fn fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn open(name: &str) -> DesignSession {
    DesignSession::open(&fixture(name), Profile::built_in())
        .expect("fixture opens")
        .0
}

fn starter() -> DesignSession {
    DesignSession::new_design(Profile::built_in()).unwrap()
}

fn input<'a>(s: &'a DesignSession, option: &str, name: &str) -> &'a ovcd_format::StoredInput {
    s.design()
        .options
        .iter()
        .find(|o| o.name == option)
        .and_then(|o| o.input(name))
        .unwrap_or_else(|| panic!("no input {name} in {option}"))
}

fn has_input(s: &DesignSession, option: &str, name: &str) -> bool {
    s.design()
        .options
        .iter()
        .find(|o| o.name == option)
        .is_some_and(|o| o.input(name).is_some())
}

// ---- new, open, save --------------------------------------------------------------------

#[test]
fn new_design_is_the_starter_design() {
    let s = starter();
    let expected = new_design(&Profile::built_in()).unwrap();
    assert_eq!(s.design().to_text(), expected.to_text());
    assert_eq!(s.design().options.len(), 1);
}

#[test]
fn every_fixture_opens_without_error_and_with_matching_hashes() {
    for name in FIXTURES {
        let (s, opened) = DesignSession::open(&fixture(name), Profile::built_in()).unwrap();
        assert!(
            opened.report.errors.is_empty(),
            "{name}: {:?}",
            opened.report
        );
        assert!(
            opened.report.warnings.is_empty(),
            "{name}: {:?}",
            opened.report
        );
        for (option, status) in &opened.hash_status {
            assert_eq!(status, &HashStatus::Matches, "{name} / {option}");
        }
        // A stored Design opens unchanged.
        assert_eq!(s.design().to_text(), fixture(name), "{name}");
    }
}

#[test]
fn open_reports_a_changed_hash_per_option() {
    let mut file: Value = serde_json::from_str(&fixture("two_options")).unwrap();
    file["options"][1]["results_hash"] = json!("0".repeat(64));
    file["options"][0]["results_hash"] = Value::Null;
    let (_, opened) = DesignSession::open(&file.to_string(), Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::NotStored);
    assert!(matches!(
        opened.hash_status[1].1,
        HashStatus::Differs { .. }
    ));
}

#[test]
fn open_writes_in_left_out_inputs_marked_default() {
    let file = json!({
        "format": "vdesign", "format_version": 2, "engine_version": "0.2.0",
        "options": [{ "name": "A", "inputs": {
            "cargo": "LNG", "hull_form": "raked_box", "formation": { "kind": "self_propelled" },
            "length_overall_m": 100.0, "beam_m": 12.0, "moulded_depth_m": 5.0,
        }}]
    });
    let (s, opened) = DesignSession::open(&file.to_string(), Profile::built_in()).unwrap();
    assert!(opened.report.errors.is_empty());
    assert_eq!(
        input(&s, "A", "cargo_containment").value,
        json!("type_c_tanks")
    );
    assert_eq!(input(&s, "A", "cargo_containment").source, Source::Default);
    assert_eq!(input(&s, "A", "fill_ratio").value, json!(0.98));
    assert_eq!(input(&s, "A", "beam_m").source, Source::Typed);
    assert!(!has_input(&s, "A", "lightship_factor_t_per_m3"));
}

#[test]
fn a_file_with_a_hard_error_still_opens_so_it_can_be_fixed() {
    let mut file: Value = serde_json::from_str(&fixture("dry_bulk_box_hold")).unwrap();
    file["options"][0]["inputs"]["beam_m"] = json!({ "value": -1.0, "source": "typed" });
    let (mut s, opened) = DesignSession::open(&file.to_string(), Profile::built_in()).unwrap();
    assert!(!opened.report.errors.is_empty());
    assert_eq!(input(&s, "Option 1", "beam_m").value, json!(-1.0));
    assert!(s.evaluate().options[0].outcome.is_none());
    s.set_input("Option 1", "beam_m", json!(11.4)).unwrap();
    assert!(matches!(
        s.evaluate().options[0].outcome,
        Some(RunOutcome::Results { .. })
    ));
}

#[test]
fn open_refuses_text_that_is_not_a_design() {
    assert!(DesignSession::open("not json", Profile::built_in()).is_err());
    assert!(DesignSession::open("{}", Profile::built_in()).is_err());
}

#[test]
fn a_saved_design_passes_validate_and_round_trips_through_open() {
    for name in FIXTURES {
        let mut s = open(name);
        let first = s.design().options[0].name.clone();
        // An edit, so the saved text differs from the fixture.
        s.set_input(&first, "max_speed_kn", json!(12.0)).unwrap();
        let saved = s.save().unwrap();
        let checked = check(&saved, &Profile::built_in());
        assert!(checked.report.is_ok(), "{name}: {:?}", checked.report);
        assert!(checked.report.warnings.is_empty(), "{name}");
        let (reopened, opened) = DesignSession::open(&saved, Profile::built_in()).unwrap();
        assert_eq!(reopened.design().to_text(), saved, "{name}");
        assert_eq!(reopened.save().unwrap(), saved, "{name}");
        for (option, status) in &opened.hash_status {
            assert!(
                matches!(status, HashStatus::Matches | HashStatus::NotStored),
                "{name} / {option}: {status:?}"
            );
        }
    }
}

#[test]
fn save_recomputes_the_hash_after_an_edit() {
    let mut s = open("dry_bulk_box_hold");
    let before = s.design().options[0].results_hash.clone().unwrap();
    s.set_input("Option 1", "beam_m", json!(12.4)).unwrap();
    let saved = s.save().unwrap();
    let after: Value = serde_json::from_str(&saved).unwrap();
    let after = after["options"][0]["results_hash"].as_str().unwrap();
    assert_ne!(after, before);
    let (_, opened) = DesignSession::open(&saved, Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
}

#[test]
fn save_is_refused_while_a_hard_error_stands() {
    let mut s = starter();
    s.set_input("Option 1", "beam_m", json!(0.0)).unwrap();
    assert!(s.save().unwrap_err().contains("hard error"));
}

// ---- edits ------------------------------------------------------------------------------

#[test]
fn set_input_marks_typed_and_reset_returns_to_default() {
    let mut s = starter();
    assert_eq!(
        input(&s, "Option 1", "max_speed_kn").source,
        Source::Default
    );
    s.set_input("Option 1", "max_speed_kn", json!(12.0))
        .unwrap();
    let typed = input(&s, "Option 1", "max_speed_kn");
    assert_eq!(
        (typed.value.clone(), typed.source),
        (json!(12.0), Source::Typed)
    );
    s.reset_input("Option 1", "max_speed_kn").unwrap();
    let back = input(&s, "Option 1", "max_speed_kn");
    assert_eq!(
        (back.value.clone(), back.source),
        (json!(10.0), Source::Default)
    );
}

#[test]
fn a_required_input_cannot_be_reset_and_unknown_names_are_refused() {
    let mut s = starter();
    assert!(s.reset_input("Option 1", "beam_m").is_err());
    assert!(s.set_input("Option 1", "no_such_input", json!(1)).is_err());
    assert!(s.set_input("No such option", "beam_m", json!(1)).is_err());
}

#[test]
fn set_formation_sets_the_kind_and_the_convoy_counts() {
    let mut s = starter();
    s.set_formation("Option 1", "pushed_convoy", Some(2), Some(3))
        .unwrap();
    let f = input(&s, "Option 1", "formation");
    assert_eq!(
        f.value,
        json!({ "kind": "pushed_convoy", "abreast": 2, "in_line": 3 })
    );
    assert_eq!(f.source, Source::Typed);
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    assert_eq!(
        input(&s, "Option 1", "formation").value,
        json!({ "kind": "self_propelled" })
    );
}

fn convoy_round_trip(other: &str) {
    let mut s = starter();
    s.set_formation("Option 1", "pushed_convoy", Some(2), Some(3))
        .unwrap();
    // The front end sends the counts it holds with the new kind; the session keeps only the kind.
    s.set_formation("Option 1", other, Some(2), Some(3))
        .unwrap();
    assert_eq!(
        input(&s, "Option 1", "formation").value,
        json!({ "kind": other })
    );
    let saved: Value = serde_json::from_str(&s.save().unwrap()).unwrap();
    assert_eq!(
        saved["options"][0]["inputs"]["formation"]["value"],
        json!({ "kind": other })
    );
    let ev = s.evaluate();
    assert!(
        ev.options[0].outcome.is_some(),
        "not run: {:?}",
        ev.options[0].messages
    );
    s.set_formation("Option 1", "pushed_convoy", None, None)
        .unwrap();
    assert_eq!(
        input(&s, "Option 1", "formation").value,
        json!({ "kind": "pushed_convoy", "abreast": 2, "in_line": 3 })
    );
}

#[test]
fn convoy_to_pushed_barge_and_back_restores_the_counts() {
    convoy_round_trip("pushed_barge");
}

#[test]
fn convoy_to_self_propelled_and_back_restores_the_counts() {
    convoy_round_trip("self_propelled");
}

#[test]
fn a_default_that_follows_another_input_follows_it_when_that_input_changes() {
    let mut s = open("gas_igc_2g_exact");
    assert_eq!(input(&s, "Option 1", "fill_ratio").value, json!(0.98));
    assert_eq!(input(&s, "Option 1", "fill_ratio").source, Source::Default);
    // A typed value stays.
    s.set_input("Option 1", "fill_ratio", json!(0.9)).unwrap();
    assert_eq!(input(&s, "Option 1", "fill_ratio").value, json!(0.9));
}

#[test]
fn changing_the_cargo_containment_swaps_the_inputs_that_apply() {
    let mut s = starter();
    assert!(has_input(&s, "Option 1", "lightship_factor_t_per_m3"));
    s.set_input("Option 1", "cargo", json!("LNG")).unwrap();
    // The default containment followed the Cargo phase.
    assert_eq!(
        input(&s, "Option 1", "cargo_containment").value,
        json!("type_c_tanks")
    );
    assert!(!has_input(&s, "Option 1", "lightship_factor_t_per_m3"));
    assert!(has_input(&s, "Option 1", "fill_ratio"));
    assert!(
        s.set_input("Option 1", "lightship_factor_t_per_m3", json!(0.1))
            .is_err()
    );
}

#[test]
fn options_are_added_duplicated_renamed_and_deleted() {
    let mut s = starter();
    assert_eq!(s.add_option(None).unwrap(), "Option 2");
    assert_eq!(s.add_option(Some(" Wide ")).unwrap(), "Wide");
    assert!(s.add_option(Some("Wide")).is_err());
    assert!(s.add_option(Some("  ")).is_err());

    s.set_input("Wide", "beam_m", json!(14.0)).unwrap();
    let copy = s.duplicate_option("Wide", None).unwrap();
    assert_eq!(copy, "Wide copy");
    assert_eq!(input(&s, &copy, "beam_m").value, json!(14.0));
    assert_eq!(s.duplicate_option("Wide", None).unwrap(), "Wide copy 2");
    let names: Vec<&str> = s.design().options.iter().map(|o| o.name.as_str()).collect();
    assert_eq!(
        names,
        ["Option 1", "Option 2", "Wide", "Wide copy 2", "Wide copy"]
    );

    s.rename_option("Wide copy", "Narrow").unwrap();
    assert!(s.rename_option("Narrow", "Option 1").is_err());
    assert!(s.rename_option("Narrow", "").is_err());
    assert!(s.rename_option("Missing", "X").is_err());

    s.delete_option("Narrow").unwrap();
    assert!(s.design().options.iter().all(|o| o.name != "Narrow"));
    assert!(s.delete_option("Narrow").is_err());
}

#[test]
fn deleting_the_last_option_is_refused() {
    let mut s = starter();
    let err = s.delete_option("Option 1").unwrap_err();
    assert!(err.contains("at least one Option"), "{err}");
    assert_eq!(s.design().options.len(), 1);
}

// ---- profile ----------------------------------------------------------------------------

fn profile_text(edit: impl Fn(&mut Value)) -> String {
    let mut p: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    edit(&mut p);
    p.to_string()
}

#[test]
fn a_profile_that_tightens_a_range_or_changes_a_default_loads() {
    let mut s = starter();
    let text = profile_text(|p| {
        p["name"] = json!("mine");
        p["inputs"]["max_speed_kn"]["default"] = json!(12.0);
        p["inputs"]["beam_m"]["hard"]["le"] = json!(100);
    });
    s.load_profile(&text).unwrap();
    assert_eq!(s.profile().name(), "mine");
    let schema = s.input_schema();
    assert_eq!(schema["profile"], json!("mine"));
    // A stored input never changes; a new Option takes the new default.
    assert_eq!(input(&s, "Option 1", "max_speed_kn").value, json!(10.0));
    s.add_option(None).unwrap();
    assert_eq!(input(&s, "Option 2", "max_speed_kn").value, json!(12.0));
}

#[test]
fn a_profile_that_loosens_a_hard_range_is_refused() {
    let mut s = starter();
    let text = profile_text(|p| p["inputs"]["beam_m"]["hard"]["le"] = json!(5000));
    let err = s.load_profile(&text).unwrap_err();
    assert!(err.contains("loosens the hard range of `beam_m`"), "{err}");
    assert_eq!(s.profile().name(), "built-in");
    assert!(s.load_profile("not json").is_err());
}

// ---- evaluate ---------------------------------------------------------------------------

#[test]
fn every_fixture_evaluates_to_results() {
    for name in FIXTURES {
        let evaluation = open(name).evaluate();
        for o in &evaluation.options {
            assert!(o.messages.is_empty(), "{name}: {:?}", o.messages);
            match (&o.outcome, *name) {
                (Some(RunOutcome::Results { results, hash }), _) => {
                    assert_eq!(&ovcd_format::results_hash(results), hash)
                }
                (other, _) => panic!("{name}: {other:?}"),
            }
        }
    }
}

/// What `ovcd validate` says of a Design: every error then every warning, with the Option lead-in.
fn validate_lines(s: &DesignSession) -> Vec<String> {
    let r = check(&s.design().to_text(), s.profile()).report;
    r.errors.into_iter().chain(r.warnings).collect()
}

fn session_lines(s: &DesignSession) -> Vec<String> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    for o in &s.evaluate().options {
        for m in &o.messages {
            let line = format!("Option '{}': {}", o.name, m.text);
            match m.severity {
                Severity::Error => errors.push(line),
                Severity::Warning => warnings.push(line),
            }
        }
    }
    errors.into_iter().chain(warnings).collect()
}

#[test]
fn a_hard_range_error_attaches_to_its_input_and_stops_the_run() {
    let mut s = starter();
    s.set_input("Option 1", "beam_m", json!(0.0)).unwrap();
    // The edit is kept, so a form shows what was typed.
    assert_eq!(input(&s, "Option 1", "beam_m").value, json!(0.0));
    let e = s.evaluate();
    let o = &e.options[0];
    assert!(o.outcome.is_none());
    let m = o
        .messages
        .iter()
        .find(|m| m.severity == Severity::Error)
        .unwrap();
    assert_eq!(m.scope, Scope::Input("beam_m".into()));
    assert!(m.text.contains("hard limit"), "{}", m.text);
    assert_eq!(session_lines(&s), validate_lines(&s));
}

#[test]
fn a_hard_error_on_a_wrong_type_or_choice_is_kept_and_reported() {
    let mut s = starter();
    s.set_input("Option 1", "cargo_containment", json!("barn"))
        .unwrap();
    s.set_input("Option 1", "water_density_t_per_m3", json!("wide"))
        .unwrap();
    assert_eq!(
        input(&s, "Option 1", "cargo_containment").value,
        json!("barn")
    );
    let o = &s.evaluate().options[0];
    let scopes: Vec<&Scope> = o.messages.iter().map(|m| &m.scope).collect();
    assert!(scopes.contains(&&Scope::Input("cargo_containment".into())));
    assert!(scopes.contains(&&Scope::Input("water_density_t_per_m3".into())));
    assert_eq!(session_lines(&s), validate_lines(&s));
}

#[test]
fn a_soft_range_warning_attaches_to_its_input_and_the_option_still_runs() {
    let mut s = starter();
    // 300 m is outside the usual length range. The depth goes with it, so that the hull is not
    // also "very slender for its depth" (a separate warning, tested in `default_depth.rs`).
    s.set_input("Option 1", "length_overall_m", json!(300.0))
        .unwrap();
    s.set_input("Option 1", "moulded_depth_m", json!(14.0))
        .unwrap();
    let o = &s.evaluate().options[0];
    let w: Vec<_> = o
        .messages
        .iter()
        .filter(|m| m.severity == Severity::Warning)
        .collect();
    assert_eq!(w.len(), 1);
    assert_eq!(w[0].scope, Scope::Input("length_overall_m".into()));
    assert!(w[0].text.contains("soft limit"));
    assert!(matches!(o.outcome, Some(RunOutcome::Results { .. })));
    assert_eq!(session_lines(&s), validate_lines(&s));
}

#[test]
fn a_cross_check_names_every_input_it_covers() {
    let mut s = starter();
    s.set_input("Option 1", "min_freeboard_m", json!(3.5))
        .unwrap();
    let o = &s.evaluate().options[0];
    let m = o
        .messages
        .iter()
        .find(|m| m.text.contains("must be less than"))
        .expect("the freeboard cross-check");
    assert_eq!(
        m.scope,
        Scope::CrossCheck(vec!["min_freeboard_m".into(), "moulded_depth_m".into()])
    );
    assert!(o.outcome.is_none());
    assert_eq!(session_lines(&s), validate_lines(&s));

    let mut s = starter();
    s.set_input("Option 1", "cargo", json!("LNG")).unwrap();
    s.set_input("Option 1", "cargo_containment", json!("box_hold"))
        .unwrap();
    let m = &s.evaluate().options[0].messages;
    assert!(m.iter().any(|m| m.text.contains("cannot carry LNG")));
    assert_eq!(session_lines(&s), validate_lines(&s));
}

#[test]
fn messages_belong_to_their_own_option() {
    let mut s = starter();
    s.add_option(Some("Second")).unwrap();
    s.set_input("Second", "beam_m", json!(0.0)).unwrap();
    let e = s.evaluate();
    assert!(e.options[0].messages.is_empty());
    assert!(matches!(
        e.options[0].outcome,
        Some(RunOutcome::Results { .. })
    ));
    assert!(!e.options[1].messages.is_empty());
    assert!(e.options[1].outcome.is_none());
}

#[test]
fn an_engine_message_comes_back_as_the_outcome() {
    // A hold with no cargo room is valid input; the engine reports it.
    let mut s = starter();
    s.set_input("Option 1", "beam_m", json!(2.0)).unwrap();
    let o = &s.evaluate().options[0];
    assert!(o.messages.iter().all(|m| m.severity == Severity::Warning));
    match &o.outcome {
        Some(RunOutcome::Results { .. }) | Some(RunOutcome::Failed(_)) => {}
        other => panic!("{other:?}"),
    }
}

#[test]
fn evaluation_json_carries_the_results_object() {
    let s = open("dry_bulk_box_hold");
    let json = s.evaluate().to_json();
    let option = &json["options"][0];
    assert_eq!(option["name"], json!("Option 1"));
    assert_eq!(option["outcome"]["status"], json!("results"));
    assert!(option["outcome"]["results"]["results"]["cargo_t"].is_number());
}

// ---- input schema -----------------------------------------------------------------------

#[test]
fn the_schema_lists_every_input_in_table_order_with_its_group() {
    let schema = input_schema(&Profile::built_in());
    let listed: Vec<&str> = schema["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["name"].as_str().unwrap())
        .collect();
    let table: Vec<&str> = INPUTS.iter().map(|s| s.name).collect();
    assert_eq!(listed, table);
    assert_eq!(
        schema["groups"],
        json!([
            "Required",
            "Vessel Class",
            "Hull and containment",
            "Hull Form",
            "Gas tanks",
            "Dry-bulk hold",
            "Liquid cargo tanks",
            "Aft block",
            "Openings",
            "Stern and propulsion",
            "Tug",
            "Waterway Limits",
            "Rules",
            "Loading conditions",
            "Speed and power",
            "Cost rates"
        ])
    );
    let group_of = |name: &str| {
        schema["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["name"] == name)
            .unwrap()["group"]
            .as_str()
            .unwrap()
            .to_string()
    };
    assert_eq!(group_of("cargo"), "Required");
    assert_eq!(group_of("moulded_depth_m"), "Hull and containment");
    assert_eq!(group_of("vessel_scheme"), "Vessel Class");
    assert_eq!(group_of("vessel_class"), "Vessel Class");
    assert_eq!(group_of("cargo_containment"), "Hull and containment");
    assert_eq!(
        group_of("lightship_factor_t_per_m3"),
        "Hull and containment"
    );
    assert_eq!(group_of("fill_ratio"), "Gas tanks");
    assert_eq!(group_of("optimise_depth"), "Gas tanks");
    assert_eq!(group_of("draught_limit_m"), "Waterway Limits");
    assert_eq!(group_of("channel_current_kn"), "Waterway Limits");
    assert_eq!(group_of("max_speed_kn"), "Speed and power");
    // Every input is in a listed group.
    for i in schema["inputs"].as_array().unwrap() {
        assert!(schema["groups"].as_array().unwrap().contains(&i["group"]));
    }
}

#[test]
fn schema_ranges_equal_the_profiles() {
    let profile = Profile::built_in();
    let schema = input_schema(&profile);
    for spec in INPUTS {
        let entry = schema["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["name"] == spec.name)
            .unwrap();
        if matches!(spec.kind, InputKind::Number { .. }) {
            let hard = profile.hard(spec.name).unwrap();
            for (key, want) in [
                ("gt", hard.gt),
                ("ge", hard.ge),
                ("lt", hard.lt),
                ("le", hard.le),
            ] {
                assert_eq!(
                    entry["hard"][key].as_f64(),
                    want,
                    "{} hard {key}",
                    spec.name
                );
            }
            match profile.soft(spec.name).unwrap() {
                Some(s) => {
                    assert_eq!(entry["soft"]["min"].as_f64(), Some(s.min), "{}", spec.name);
                    assert_eq!(entry["soft"]["max"].as_f64(), Some(s.max), "{}", spec.name);
                }
                None => assert!(entry["soft"].is_null(), "{}", spec.name),
            }
            assert_eq!(entry["unit"], json!(profile.unit(spec.name)));
            assert_eq!(entry["basis"], json!(profile.basis(spec.name)));
        }
        assert_eq!(entry["required"], json!(spec.required), "{}", spec.name);
        // The class inputs are held only by an Option defined by a class: they have no default.
        // The class inputs have no default; the depth has a rule (`default_rule`), not a value.
        if !spec.required
            && !ovcd_format::class_input::UNSET_WHEN_ABSENT.contains(&spec.name)
            && spec.name != "moulded_depth_m"
            && spec.name != "rule_sets"
        {
            assert!(
                entry.get("default").is_some() || entry.get("default_by").is_some(),
                "{} has no default",
                spec.name
            );
        }
    }
    // The convoy counts take the profile's ranges.
    let formation = schema["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["name"] == "formation")
        .unwrap();
    assert_eq!(
        formation["convoy"]["abreast"]["hard"]["ge"].as_f64(),
        profile.hard("formation_abreast").unwrap().ge
    );
    assert_eq!(
        formation["convoy"]["in_line"]["soft"]["max"].as_f64(),
        profile.soft("formation_in_line").unwrap().map(|s| s.max)
    );
}

#[test]
fn the_schema_names_cargo_and_gives_keyed_defaults_and_containments() {
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
    let cargo = get("cargo");
    let lng = cargo["choices"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["value"] == "LNG_ethane")
        .unwrap();
    assert_eq!(lng["label"], json!("LNG / Ethane"));
    assert_eq!(lng["phase"], json!("liquid_gas"));
    assert_eq!(get("fill_ratio")["applies_to"], json!(["type_c_tanks"]));
    // Issue #292: a table by Cargo and carriage condition (98 % for all, IGC 15.3). `default_by` is
    // its short form for a consumer that knows only that; `default_table` is the table itself.
    assert_eq!(
        get("fill_ratio")["default_by"]["by"],
        json!("Cargo, LPG carriage condition")
    );
    assert_eq!(
        get("fill_ratio")["default_by"]["values"],
        json!({ "any": 0.98 })
    );
    assert_eq!(
        get("fill_ratio")["default_table"]["rows"][0]["status"],
        json!("sourced")
    );
    // Issue #175: the ADN ruleset and its tank cap are gone; the diameter cap is an input. Issue
    // #274: the ruleset is gone too (the Rule Sets of the Option are its `rule_sets`).
    assert!(
        schema["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .all(|i| i["name"] != "ruleset")
    );
    assert!(
        schema["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .all(|i| i["name"] != "adn_tank_cap_m3")
    );
    let od = get("max_tank_od_m");
    assert_eq!(od["default"], json!(16.0));
    assert_eq!(od["unit"], json!("m"));
    assert_eq!(od["applies_to"], json!(["type_c_tanks"]));
    assert_eq!(od["group"], json!("Gas tanks"));
    assert!(
        od["basis"]
            .as_str()
            .unwrap()
            .contains("not an IGC Code rule"),
        "{}",
        od["basis"]
    );
    assert_eq!(get("max_speed_kn")["default"], json!(10.0));
    assert!(get("channel_depth_m")["default"].is_null());
    assert_eq!(get("channel_depth_m")["nullable"], json!(true));
    assert_eq!(get("beam_m")["required"], json!(true));
    assert!(get("beam_m").get("default").is_none());
}

// ---- inputs that stop applying are hidden, not lost ----------------------------------------

#[test]
fn switching_the_cargo_containment_back_restores_what_was_typed() {
    let mut s = starter();
    s.set_input("Option 1", "lightship_factor_t_per_m3", json!(0.12))
        .unwrap();
    s.set_input("Option 1", "cargo", json!("LNG")).unwrap();
    assert!(!has_input(&s, "Option 1", "lightship_factor_t_per_m3"));
    // The saved file keeps only what applies, as `ovcd fill` would.
    s.set_input("Option 1", "fill_ratio", json!(0.8)).unwrap();
    let saved: Value = serde_json::from_str(&s.save().unwrap()).unwrap();
    assert!(saved["options"][0]["inputs"]["lightship_factor_t_per_m3"].is_null());

    s.set_input("Option 1", "cargo", json!("bauxite")).unwrap();
    let back = input(&s, "Option 1", "lightship_factor_t_per_m3");
    assert_eq!(back.value, json!(0.12));
    assert_eq!(back.source, Source::Typed);
    assert!(!has_input(&s, "Option 1", "fill_ratio"));

    s.set_input("Option 1", "cargo", json!("LNG")).unwrap();
    assert_eq!(input(&s, "Option 1", "fill_ratio").value, json!(0.8));
}

#[test]
fn a_default_that_stopped_applying_comes_back_as_the_default_not_a_stale_copy() {
    let mut s = starter();
    s.set_input("Option 1", "cargo", json!("LNG")).unwrap();
    s.set_input("Option 1", "cargo", json!("bauxite")).unwrap();
    let back = input(&s, "Option 1", "lightship_factor_t_per_m3");
    assert_eq!(back.source, Source::Default);
}

#[test]
fn hidden_inputs_follow_an_option_that_is_renamed_or_duplicated() {
    let mut s = starter();
    s.set_input("Option 1", "lightship_factor_t_per_m3", json!(0.12))
        .unwrap();
    s.set_input("Option 1", "cargo", json!("LNG")).unwrap();
    s.rename_option("Option 1", "Gas").unwrap();
    let copy = s.duplicate_option("Gas", Some("Gas 2")).unwrap();
    s.set_input("Gas", "cargo", json!("bauxite")).unwrap();
    s.set_input(&copy, "cargo", json!("bauxite")).unwrap();
    assert_eq!(
        input(&s, "Gas", "lightship_factor_t_per_m3").value,
        json!(0.12)
    );
    assert_eq!(
        input(&s, &copy, "lightship_factor_t_per_m3").value,
        json!(0.12)
    );
}

#[test]
fn a_deleted_option_leaves_no_hidden_inputs_for_a_new_option_of_that_name() {
    let mut s = starter();
    s.add_option(Some("Two")).unwrap();
    s.set_input("Two", "lightship_factor_t_per_m3", json!(0.12))
        .unwrap();
    s.set_input("Two", "cargo", json!("LNG")).unwrap();
    s.delete_option("Two").unwrap();
    s.add_option(Some("Two")).unwrap();
    s.set_input("Two", "cargo", json!("LNG")).unwrap();
    s.set_input("Two", "cargo", json!("bauxite")).unwrap();
    assert_eq!(
        input(&s, "Two", "lightship_factor_t_per_m3").source,
        Source::Default
    );
}

#[test]
fn every_input_has_a_plain_label_in_the_schema() {
    let schema = input_schema(&Profile::built_in());
    for entry in schema["inputs"].as_array().unwrap() {
        let label = entry["label"].as_str().unwrap_or("");
        assert!(!label.is_empty(), "{} has no label", entry["name"]);
        assert!(!label.contains('_'), "{label} is a key, not a label");
    }
}

#[test]
fn a_typed_lightship_factor_survives_box_hold_to_hopper_and_back() {
    let mut s = starter();
    s.set_input("Option 1", "lightship_factor_t_per_m3", json!(0.12))
        .unwrap();
    s.set_input("Option 1", "cargo_containment", json!("sloped_hopper_hold"))
        .unwrap();
    assert!(!has_input(&s, "Option 1", "lightship_factor_t_per_m3"));
    s.set_input("Option 1", "cargo_containment", json!("box_hold"))
        .unwrap();
    let back = input(&s, "Option 1", "lightship_factor_t_per_m3");
    assert_eq!(back.value, json!(0.12));
    assert_eq!(back.source, Source::Typed);
}

#[test]
fn choice_inputs_show_plain_labels_from_the_input_table() {
    let schema = input_schema(&Profile::built_in());
    for entry in schema["inputs"].as_array().unwrap() {
        if entry["kind"] != "choice" || entry["name"] == "ruleset" {
            continue;
        }
        for c in entry["choices"].as_array().unwrap() {
            assert_ne!(c["label"], c["value"], "{} has no display name", c["value"]);
        }
    }
}

#[test]
fn the_evaluation_carries_file_level_messages() {
    let j = starter().evaluate().to_json();
    assert_eq!(j["file_messages"], json!([]));
}

// ---- messages in the app use plain labels ------------------------------------------------------

#[test]
fn app_messages_use_input_labels_and_the_raw_text_is_unchanged() {
    let mut s = starter();
    s.set_input("Option 1", "beam_m", json!(-1)).unwrap();
    s.set_formation("Option 1", "pushed_convoy", Some(0), Some(3))
        .unwrap();
    let e = s.evaluate();
    let raw: Vec<&str> = e.options[0]
        .messages
        .iter()
        .map(|m| m.text.as_str())
        .collect();
    assert!(raw.iter().any(|t| t.contains("`beam_m`")), "{raw:?}");
    assert!(
        raw.iter().any(|t| t.contains("`formation.abreast`")),
        "{raw:?}"
    );
    let j = e.to_json();
    let shown: Vec<String> = j["options"][0]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["text"].as_str().unwrap().to_string())
        .collect();
    assert!(shown.iter().all(|t| !t.contains('`')), "{shown:?}");
    assert!(shown.iter().any(|t| t.starts_with("Beam")), "{shown:?}");
    assert!(shown.iter().any(|t| t.starts_with("Abreast")), "{shown:?}");
}
