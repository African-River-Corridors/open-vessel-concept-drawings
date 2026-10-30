//! The `abs-barges` notes for an ATB (issue #363, spec 0007 "Rules"): ABS Steel Barges 2024, Part 5
//! Chapter 3 Section 1, paragraphs 1.3 (the tug and the barge are two vessels) and 11.3 (the wing
//! walls at a notch). Approval items, so each is a note row, `not evaluated`, in the Rule Trace of an
//! ATB Option of a coastal Service Area and in no other.
//!
//! Label: property

use ovd_format::{Profile, RunOutcome, check, fill_as_new, run_design};
use serde_json::{Value, json};

const TWO_VESSELS: &str = "abs-barges/5-3-1.1.3/two-vessels";
const WING_WALLS: &str = "abs-barges/5-3-1.11.3/notch-wing-walls";

/// The Rule Trace rows of a new Option of `formation` in `service_area`.
fn rows(formation: Value, service_area: &str) -> Vec<Value> {
    let raw = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": "bauxite", "source": "typed" },
            "hull_form": { "value": "raked_box", "source": "typed" },
            "formation": { "value": formation, "source": "typed" },
            "service_area": { "value": service_area, "source": "typed" },
            "length_overall_m": { "value": 100.0, "source": "typed" },
            "beam_m": { "value": 20.0, "source": "typed" },
            "moulded_depth_m": { "value": 6.0, "source": "typed" }
        }}]
    });
    let filled = fill_as_new(&raw.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    let text = filled.design.unwrap().to_text();
    let design = check(&text, &Profile::built_in()).design.unwrap();
    match run_design(&design).remove(0).outcome {
        RunOutcome::Results { results, .. } => {
            results["rule_trace"]["rows"].as_array().unwrap().clone()
        }
        other => panic!("{other:?}"),
    }
}

fn outcomes(rows: &[Value], rule: &str) -> Vec<String> {
    rows.iter()
        .filter(|r| r["rule"] == rule)
        .map(|r| r["outcome"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn an_atb_of_a_coastal_service_area_has_both_notes_not_evaluated() {
    let r = rows(json!({ "kind": "atb" }), "coastal-and-river");
    for rule in [TWO_VESSELS, WING_WALLS] {
        assert_eq!(outcomes(&r, rule), ["not evaluated"], "{rule}");
    }
    // The note row carries its source and its gap.
    let row = r.iter().find(|x| x["rule"] == WING_WALLS).unwrap();
    assert!(
        row["gap"].as_str().unwrap().contains("approval item"),
        "{row}"
    );
}

#[test]
fn another_formation_has_no_such_row_or_a_not_applicable_one() {
    for formation in [
        json!({ "kind": "pushed_barge" }),
        json!({ "kind": "self_propelled" }),
    ] {
        let r = rows(formation, "coastal-and-river");
        for rule in [TWO_VESSELS, WING_WALLS] {
            assert!(
                outcomes(&r, rule).iter().all(|o| o == "not applicable"),
                "{rule}: {:?}",
                outcomes(&r, rule)
            );
        }
    }
}

#[test]
fn a_river_only_atb_does_not_have_the_set() {
    // The set is in the coastal stack: a river-only Option names no ABS Rule.
    let r = rows(json!({ "kind": "atb" }), "river-only");
    assert!(
        r.iter()
            .all(|x| !x["rule"].as_str().unwrap().starts_with("abs-barges/"))
    );
}
