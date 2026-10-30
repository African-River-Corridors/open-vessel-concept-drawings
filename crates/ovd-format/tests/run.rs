//! Seam 4: running a Design in memory: the Option-to-`simulate` mapping, the canonical JSON and
//! the results hash (issue #25).

use ovd_format::{
    CONVOY_NOT_AVAILABLE, HashStatus, Profile, RunOutcome, canonical_json, fill, new_design,
    results_hash, run_design, run_option, simulate_params_of, stamp,
};
use serde_json::{Value, json};

#[test]
fn canonical_json_sorts_keys_at_every_depth_and_drops_white_space() {
    let a = json!({"b": {"d": 1, "c": 2}, "a": [true, null, 1.5, "x\u{e9}"]});
    assert_eq!(
        canonical_json(&a),
        "{\"a\":[true,null,1.5,\"x\u{e9}\"],\"b\":{\"c\":2,\"d\":1}}"
    );
}

#[test]
fn key_order_does_not_change_the_hash() {
    let a: Value = serde_json::from_str(r#"{"x":1,"y":{"p":1.0,"q":2}}"#).unwrap();
    let b: Value = serde_json::from_str(r#"{"y":{"q":2,"p":1.0},"x":1}"#).unwrap();
    assert_eq!(results_hash(&a), results_hash(&b));
}

#[test]
fn the_hash_is_sha256_of_the_canonical_text() {
    // Printed by Python: hashlib.sha256(canonical text bytes).hexdigest().
    let v = json!({"a": [true, null, 1.5, "x\u{e9}"], "b": {"d": 1, "c": 2}});
    assert_eq!(
        results_hash(&v),
        "d0dd10764352aa909750d9cf4a1a0eefb5fdbbfa555cf3e06b56c104cc831e7d"
    );
}

#[test]
fn one_changed_digit_changes_the_hash() {
    assert_ne!(
        results_hash(&json!({"cargo_t": 2038.5})),
        results_hash(&json!({"cargo_t": 2038.6}))
    );
}

fn starter() -> ovd_format::Design {
    new_design(&Profile::built_in()).unwrap()
}

#[test]
fn a_starter_design_carries_the_hash_run_computes() {
    let design = starter();
    let stored = design.options[0].results_hash.clone().unwrap();
    let runs = run_design(&design);
    assert_eq!(runs[0].hash_status, HashStatus::Matches);
    match &runs[0].outcome {
        RunOutcome::Results { hash, .. } => assert_eq!(*hash, stored),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_different_stored_hash_is_reported_as_differing() {
    let mut design = starter();
    design.options[0].results_hash = Some("ab".repeat(32));
    match &run_design(&design)[0].hash_status {
        HashStatus::Differs { stored, computed } => {
            assert_eq!(stored, &"ab".repeat(32));
            assert_ne!(stored, computed);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn stamp_never_replaces_a_stored_hash() {
    let mut design = starter();
    design.options[0].results_hash = Some("ab".repeat(32));
    stamp(&mut design);
    assert_eq!(design.options[0].results_hash, Some("ab".repeat(32)));
}

#[test]
fn the_formation_gives_self_propelled() {
    let mut design = starter();
    let formation = |kind: &str| json!({ "kind": kind });
    for (kind, expected) in [("pushed_barge", false), ("self_propelled", true)] {
        let o = &mut design.options[0];
        let slot = o.inputs.iter_mut().find(|(n, _)| n == "formation").unwrap();
        slot.1.value = formation(kind);
        let params = simulate_params_of(o).unwrap();
        assert_eq!(params["self_propelled"], json!(expected), "{kind}");
    }
}

#[test]
fn glossary_names_map_to_engine_names() {
    let design = starter();
    let p = simulate_params_of(&design.options[0]).unwrap();
    assert_eq!(p["vessel"], "box_barge");
    assert_eq!(p["cargo"], "bauxite");
    assert_eq!(p["loa_m"], json!(76.5));
    assert_eq!(p["beam_m"], json!(11.4));
    assert_eq!(p["depth_m"], json!(3.5));
    // Issue #448: Cb is a result of the hull's sections, not a param.
    assert!(!p.contains_key("cb") && p.contains_key("freeboard_min_m"));
    assert!(p.contains_key("lightship_k_t_per_m3"));
    assert!(p.contains_key("eta_d") && p.contains_key("speed_max_kn"));
    // No channel depth: no channel at all.
    assert!(!p.contains_key("channel"));
}

#[test]
fn a_channel_with_a_depth_is_described_to_the_engine() {
    let mut design = starter();
    let o = &mut design.options[0];
    for (name, value) in [
        ("channel_depth_m", json!(6.0)),
        ("channel_bottom_width_m", json!(60.0)),
    ] {
        let slot = o.inputs.iter_mut().find(|(n, _)| n == name).unwrap();
        slot.1.value = value;
    }
    let p = simulate_params_of(o).unwrap();
    assert_eq!(p["channel"]["depth_m"], json!(6.0));
    assert_eq!(p["channel"]["bottom_width_m"], json!(60.0));
    assert!(p["channel"].get("ukc_m").is_some());
    assert!(p["channel"].get("bank_slope").is_some());
    assert!(p["channel"].get("current_kn").is_some());
}

#[test]
fn a_convoy_runs_as_one_pushed_barge_and_gets_totals() {
    let mut design = starter();
    let o = &mut design.options[0];
    let slot = o.inputs.iter_mut().find(|(n, _)| n == "formation").unwrap();
    slot.1.value = json!({ "kind": "pushed_convoy", "abreast": 2, "in_line": 2 });
    let p = simulate_params_of(o).unwrap();
    assert_eq!(p["self_propelled"], json!(false));
    let RunOutcome::Results { results, .. } = run_option(o) else {
        panic!("a convoy has results");
    };
    assert_eq!(results["convoy"]["barges"], json!(4));
}

#[test]
fn fill_does_not_stamp_a_file_that_has_a_hard_error() {
    let text = json!({
        "format": "vdesign", "format_version": 2, "engine_version": "0.1.0",
        "options": [{ "name": "A", "inputs": {
            "cargo": "bauxite", "hull_form": "raked_box", "formation": {"kind": "pushed_barge"},
            "length_overall_m": 76.5, "beam_m": -1.0, "moulded_depth_m": 3.5 } }]
    })
    .to_string();
    let c = fill(&text, &Profile::built_in());
    assert!(!c.report.is_ok());
    assert!(c.design.unwrap().options[0].results_hash.is_none());
}

#[test]
fn a_results_hash_that_is_not_text_is_a_hard_error() {
    let mut design = starter().to_json();
    design["options"][0]["results_hash"] = json!(5);
    let c = ovd_format::check(&design.to_string(), &Profile::built_in());
    assert!(
        c.report
            .errors
            .iter()
            .any(|e| e.contains("`results_hash` must be text")),
        "{:?}",
        c.report.errors
    );
}

// Issue #137: the engine version, the warning wording, and the shape of a stored hash.

fn raw_file(engine_version: &str, options: Value) -> String {
    json!({
        "format": "vdesign", "format_version": 2, "engine_version": engine_version,
        "options": options
    })
    .to_string()
}

fn raw_option(name: &str, hash: Option<&str>) -> Value {
    let mut o = json!({ "name": name, "inputs": {
        "cargo": "bauxite", "hull_form": "raked_box", "formation": {"kind": "pushed_barge"},
        "length_overall_m": 76.5, "beam_m": 11.4, "moulded_depth_m": 3.5 } });
    if let Some(h) = hash {
        o["results_hash"] = json!(h);
    }
    o
}

#[test]
fn fill_that_writes_a_hash_sets_the_engine_version_to_this_engine() {
    let text = raw_file("0.1.0", json!([raw_option("A", None)]));
    let c = fill(&text, &Profile::built_in());
    assert!(c.report.is_ok(), "{:?}", c.report.errors);
    let design = c.design.unwrap();
    assert!(design.options[0].results_hash.is_some());
    assert_eq!(design.engine_version, ovd_format::vdesign::ENGINE_VERSION);
}

#[test]
fn fill_does_not_mix_engines_in_one_file() {
    // Option A holds a hash saved under 0.1.0. A hash for B made by this engine would sit under
    // the same version, so B is left without one.
    let text = raw_file(
        "0.1.0",
        json!([
            raw_option("A", Some(&"ab".repeat(32))),
            raw_option("B", None)
        ]),
    );
    let design = fill(&text, &Profile::built_in()).design.unwrap();
    assert_eq!(design.engine_version, "0.1.0");
    assert_eq!(design.options[0].results_hash, Some("ab".repeat(32)));
    assert!(design.options[1].results_hash.is_none());
}

#[test]
fn the_warning_does_not_blame_the_engine() {
    let w = ovd_format::hash_warning("G", "0.0.7", &"00".repeat(32), &"11".repeat(32));
    assert!(
        w.contains("results differ from the hash saved in the file"),
        "{w}"
    );
    assert!(w.contains("saved with engine version 0.0.7"), "{w}");
    assert!(!w.contains("changed since"), "{w}");
}

#[test]
fn an_upper_case_copy_of_the_right_hash_matches() {
    let mut design = starter();
    let h = design.options[0].results_hash.clone().unwrap();
    design.options[0].results_hash = Some(h.to_uppercase());
    assert_eq!(run_design(&design)[0].hash_status, HashStatus::Matches);
}

#[test]
fn a_stored_hash_that_is_not_64_hex_digits_is_a_hard_error() {
    for bad in ["", "abc", &"g".repeat(64), &"ab".repeat(33)] {
        let text = raw_file("0.1.0", json!([raw_option("A", Some(bad))]));
        let c = ovd_format::check(&text, &Profile::built_in());
        assert!(
            c.report
                .errors
                .iter()
                .any(|e| e.contains("`results_hash` must be 64 hex digits")),
            "{bad:?}: {:?}",
            c.report.errors
        );
    }
}

#[test]
fn an_empty_engine_version_is_a_hard_error() {
    let text = raw_file("", json!([raw_option("A", None)]));
    let c = fill(&text, &Profile::built_in());
    assert!(
        c.report
            .errors
            .iter()
            .any(|e| e.contains("`engine_version`")),
        "{:?}",
        c.report.errors
    );
}

// Issue #138: a not-yet-supported Option is told apart from a failed one.

#[test]
fn each_outcome_has_its_own_status_word() {
    let results = run_option(&starter().options[0]);
    assert_eq!(results.status(), "results");
    assert_eq!(
        RunOutcome::NotAvailable(CONVOY_NOT_AVAILABLE.into()).status(),
        "not_yet_supported"
    );
    assert_eq!(RunOutcome::Failed("no".into()).status(), "failed");
    assert!(!RunOutcome::NotAvailable("x".into()).is_failure());
    assert!(RunOutcome::Failed("x".into()).is_failure());
    assert!(!results.is_failure());
}

#[test]
fn a_stored_hash_on_a_convoy_is_checked_like_any_other() {
    let mut design = starter();
    let o = &mut design.options[0];
    let slot = o.inputs.iter_mut().find(|(n, _)| n == "formation").unwrap();
    slot.1.value = json!({ "kind": "pushed_convoy", "abreast": 2, "in_line": 2 });
    assert!(matches!(
        run_design(&design)[0].hash_status,
        HashStatus::Differs { .. }
    ));
    design.options[0].results_hash = None;
    assert_eq!(run_design(&design)[0].hash_status, HashStatus::NotStored);
}
