//! Seam: the GA of a Formation's tug (issue #360, spec 0007 "Drawings": "The tug gets its own GA").
//! One sheet: the side elevation and the plan, from `drawing.tug`. The documents must pass the
//! conformance run of the Drawing Format Standard (spec 0003).
//!
//! Label: property. Each test states a rule about the drawn entities; no number is a printed result.

use ovcd_format::rule_set::Library;
use ovcd_format::{NO_TUG_GA, Profile, RunOutcome, fill_as_new, run_option_with, tug_ga_named};
use serde_json::{Value, json};

fn outcome(formation: Value, tug_included: bool, extra: Value) -> RunOutcome {
    let mut inputs = json!({
        "cargo": { "value": "bauxite", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": formation, "source": "typed" },
        "length_overall_m": { "value": 76.5, "source": "typed" },
        "beam_m": { "value": 11.4, "source": "typed" },
        "moulded_depth_m": { "value": 3.5, "source": "typed" },
        "tug_included": { "value": tug_included, "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    let design = fill_as_new(&file.to_string(), &Profile::built_in())
        .design
        .unwrap();
    let outcome = run_option_with(&design.options[0], &Library::built_in(), &[]);
    assert!(matches!(outcome, RunOutcome::Results { .. }), "{outcome:?}");
    outcome
}

fn entities<'a>(document: &'a Value, layer: &str, kind: &str) -> Vec<&'a Value> {
    document["viewports"][0]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["layer"] == layer && e["type"] == kind)
        .collect()
}

fn conforms(document: &Value) {
    match ovcd_draw::run_value(document, &ovcd_draw::NoFiles) {
        ovcd_draw::RunResult::Ok(_) => {}
        ovcd_draw::RunResult::Errors(set) => panic!("breaks the standard: {set:?}"),
    }
}

fn convoy() -> serde_json::Value {
    json!({ "kind": "pushed_convoy", "abreast": 2, "in_line": 3 })
}

fn leaders(document: &Value) -> Vec<String> {
    entities(document, "ANNO", "leader")
        .iter()
        .map(|l| l["text"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn a_pusher_with_two_shafts_draws_its_propellers_rudders_engines_and_wheelhouse() {
    let o = outcome(
        convoy(),
        true,
        json!({ "tug_propulsor_type": "shaft_nozzle", "tug_propulsor_count": 2.0,
                "tug_sightline_method": "blind_distance" }),
    );
    let doc = tug_ga_named(&o, Some("Option 1")).expect("a GA");
    conforms(&doc);
    let names = leaders(&doc);
    for want in [
        "PROPELLER",
        "RUDDER",
        "FLANKING RUDDERS",
        "PUSH KNEES",
        "WHEELHOUSE",
        "ENGINE ROOM",
    ] {
        assert!(names.iter().any(|n| n == want), "{want}: {names:?}");
    }
    // The wheelhouse is lifted for the sightline: its raised outline is drawn.
    assert!(names.iter().any(|n| n == "RAISED"), "{names:?}");
    assert!(names.iter().any(|n| n == "ENGINES (2)"), "{names:?}");
    // In plan, each shaft has its centre line from the propeller to its engine.
    let tug = &match &o {
        RunOutcome::Results { results, .. } => results,
        _ => unreachable!(),
    }["drawing"]["tug"];
    assert_eq!(tug["propulsor_count"], 2);
    assert!(entities(&doc, "CENTRE", "centre_line").len() >= 1 + 1 + 2);
    assert!(
        doc["title_block"]["fields"]["title"]
            .as_str()
            .unwrap()
            .starts_with("PUSHER")
    );
}

#[test]
fn an_azimuth_tug_and_a_towing_tug_draw_and_conform() {
    let az = outcome(
        convoy(),
        true,
        json!({ "tug_propulsor_type": "azimuth_ducted" }),
    );
    let doc = tug_ga_named(&az, None).expect("a GA");
    conforms(&doc);
    // No shaft line and no rudders: the azimuth unit steers.
    assert!(!leaders(&doc).iter().any(|n| n == "RUDDER"));
    let tow = outcome(json!({ "kind": "towed_barge" }), true, json!({}));
    let doc = tug_ga_named(&tow, None).expect("a GA");
    conforms(&doc);
    // A towing tug has no push knees.
    assert!(!leaders(&doc).iter().any(|n| n == "PUSH KNEES"));
    assert!(
        doc["title_block"]["fields"]["title"]
            .as_str()
            .unwrap()
            .starts_with("TOWING TUG")
    );
    let atb = outcome(json!({ "kind": "atb" }), true, json!({}));
    conforms(&tug_ga_named(&atb, None).expect("a GA"));
}

#[test]
fn an_option_without_its_tug_has_no_tug_ga() {
    let o = outcome(convoy(), false, json!({}));
    assert_eq!(tug_ga_named(&o, None), Err(NO_TUG_GA.to_string()));
    let own = outcome(json!({ "kind": "self_propelled" }), false, json!({}));
    assert_eq!(tug_ga_named(&own, None), Err(NO_TUG_GA.to_string()));
}
