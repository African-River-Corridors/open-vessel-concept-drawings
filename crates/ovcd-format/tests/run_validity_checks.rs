//! Issue #420 (part b): the resistance method's validity checks reach the Design result.
//!
//! Label: property. The rules: in a channel the squat, wave and fit checks at each service speed are
//! in the run's `checks`, once per environment, with their severity; an out-of-range one is a
//! failed warning and blocks nothing; a channel with a depth but no bottom width gets a warning
//! that says these checks are skipped; the `ovcd/squat/*` Rules evaluate (no longer `not
//! applicable`).

use serde_json::{Value, json};

/// The design-impact inputs "sweep hopper channel" (`tools/design_impact.py`), as a new Design.
fn hopper_channel(loa: f64, beam: f64, bottom_width: Option<f64>) -> Value {
    let mut inputs = json!({
        "cargo": "bauxite", "hull_form": "raked_box", "formation": { "kind": "self_propelled" },
        "length_overall_m": loa, "beam_m": beam, "moulded_depth_m": 4.5,
        "cargo_containment": "sloped_hopper_hold", "max_speed_kn": 9.0,
        "channel_current_kn": 1.5, "channel_depth_m": 5.0,
    });
    if let Some(w) = bottom_width {
        inputs["channel_bottom_width_m"] = json!(w);
    }
    let text = json!({ "format": "vdesign", "format_version": 3, "engine_version": "0.2.0",
                       "options": [{ "name": "A", "inputs": inputs }] })
    .to_string();
    let design = ovcd_format::fill_as_new(&text, &ovcd_format::Profile::built_in())
        .design
        .expect("design");
    let mut runs = ovcd_format::run_design(&design).into_iter();
    match runs.next().expect("one Option").outcome {
        ovcd_format::RunOutcome::Results { results, .. } => results,
        other => panic!("no results: {other:?}"),
    }
}

fn checks(results: &Value) -> &Vec<Value> {
    results["checks"].as_array().unwrap()
}

fn named<'a>(results: &'a Value, base: &str) -> Vec<&'a Value> {
    checks(results)
        .iter()
        .filter(|c| {
            c["name"]
                .as_str()
                .unwrap()
                .starts_with(&format!("{base}["))
        })
        .collect()
}

fn rows<'a>(results: &'a Value, prefix: &str) -> Vec<&'a Value> {
    results["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["rule"].as_str().unwrap().starts_with(prefix))
        .collect()
}

#[test]
fn an_out_of_range_depth_froude_number_is_a_failed_warning_that_blocks_nothing() {
    let results = hopper_channel(60.0, 10.0, Some(60.0));
    let froude = named(&results, "froude depth subcritical");
    assert!(!froude.is_empty(), "{:#?}", checks(&results));
    for c in &froude {
        assert_eq!(c["severity"], "warning", "{c}");
    }
    assert!(
        froude.iter().any(|c| c["ok"] == false),
        "the case needs an out-of-range depth Froude number: {froude:#?}"
    );
    // The Rule on it fails as a warning; it gates nothing.
    let row = rows(&results, "ovcd/squat/froude-depth");
    assert_eq!(row.len(), 1, "{row:#?}");
    assert_ne!(row[0]["outcome"], "pass", "{}", row[0]);
    let failures: Vec<&Value> = checks(&results)
        .iter()
        .filter(|c| c["ok"] == false && c["severity"] == "error")
        .collect();
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn every_method_check_of_a_channel_comes_once_per_environment_with_its_severity() {
    let results = hopper_channel(60.0, 10.0, Some(60.0));
    for base in [
        "keel clears bed",
        "froude depth subcritical",
        "service speed below limiting",
    ] {
        let found = named(&results, base);
        assert!(!found.is_empty(), "{base}: {:#?}", checks(&results));
        for c in found {
            assert!(
                c["severity"] == "error" || c["severity"] == "warning",
                "{c}"
            );
        }
    }
    // The run gives the resistance model a stand-in installed power, so that check is left out.
    assert!(named(&results, "power within installed").is_empty());
}

#[test]
fn the_squat_rules_evaluate_in_a_channel_with_a_bottom_width() {
    let results = hopper_channel(60.0, 10.0, Some(60.0));
    let squat = rows(&results, "ovcd/squat/");
    assert!(!squat.is_empty());
    for r in squat {
        assert_ne!(r["outcome"], "not applicable", "{r}");
    }
}

#[test]
fn a_channel_with_no_bottom_width_says_its_squat_is_not_checked() {
    let has_note = |results: &Value| {
        results["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w.as_str().unwrap().contains("no bottom width"))
    };
    let without = hopper_channel(60.0, 10.0, None);
    assert!(has_note(&without), "{:#?}", without["warnings"]);
    assert!(named(&without, "keel clears bed").is_empty());
    assert!(
        rows(&without, "ovcd/squat/")
            .iter()
            .all(|r| r["outcome"] == "not applicable")
    );
    assert!(!has_note(&hopper_channel(60.0, 10.0, Some(60.0))));
}
