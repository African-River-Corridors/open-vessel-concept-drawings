//! Seam: the arrangement plan of a Formation (issue #360, spec 0007 "Drawings"): the barges and the
//! tug in plan with the overall dimensions and the coupling points, and the profile in the light
//! condition with the tug's sightline over the convoy. The documents must pass the conformance run
//! of the Drawing Format Standard (spec 0003).
//!
//! Label: property. Each test states a rule about the drawn entities or the results; no number is a
//! printed result.

use ovd_format::rule_set::Library;
use ovd_format::{
    NO_ARRANGEMENT_PLAN, Profile, RunOutcome, arrangement_plan_named, fill_as_new, run_option_with,
};
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

fn convoy(abreast: u64, in_line: u64) -> Value {
    json!({ "kind": "pushed_convoy", "abreast": abreast, "in_line": in_line })
}

fn results(o: &RunOutcome) -> &Value {
    match o {
        RunOutcome::Results { results, .. } => results,
        _ => unreachable!(),
    }
}

fn entities<'a>(document: &'a Value, layer: &str, kind: &str) -> Vec<&'a Value> {
    document["viewports"][0]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["layer"] == layer && e["type"] == kind)
        .collect()
}

fn points(e: &Value) -> Vec<[f64; 2]> {
    e["points_m"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| [p[0].as_f64().unwrap(), p[1].as_f64().unwrap()])
        .collect()
}

/// The plan's barge outlines: the closed OUTLINE polylines above the profile (y within the beam).
fn barge_outlines(document: &Value, half_beam: f64) -> Vec<Vec<[f64; 2]>> {
    entities(document, "OUTLINE", "polyline")
        .into_iter()
        .map(points)
        .filter(|p| p.iter().all(|q| q[1].abs() <= half_beam + 1e-9))
        .collect()
}

fn conforms(document: &Value) {
    match ovd_draw::run_value(document, &ovd_draw::NoFiles) {
        ovd_draw::RunResult::Ok(_) => {}
        ovd_draw::RunResult::Errors(set) => panic!("breaks the standard: {set:?}"),
    }
}

fn horizontal_dimensions(document: &Value) -> Vec<f64> {
    entities(document, "DIMENSIONS", "dimension")
        .into_iter()
        .filter(|d| d["orientation"] == "horizontal")
        .map(|d| {
            (d["to"]["point_m"][0].as_f64().unwrap() - d["from"]["point_m"][0].as_f64().unwrap())
                .abs()
        })
        .collect()
}

#[test]
fn a_two_by_three_convoy_with_its_pusher_draws_six_barges_the_pusher_and_the_sightline() {
    let o = outcome(
        convoy(2, 3),
        true,
        json!({ "tug_sightline_method": "blind_distance" }),
    );
    let r = results(&o);
    let doc = arrangement_plan_named(&o, Some("Option 1")).expect("a plan");
    conforms(&doc);
    // Six barges in plan, each 76.5 m by 11.4 m.
    let barges = barge_outlines(&doc, 11.4);
    assert_eq!(barges.len(), 6, "{barges:?}");
    // The overall length runs from the pusher's stern to the bow: the totals with the tug.
    let overall = r["convoy"]["totals"]["overall_length_with_tug_m"]
        .as_f64()
        .unwrap();
    let dims = horizontal_dimensions(&doc);
    assert!(dims.iter().any(|d| (d - overall).abs() < 1e-9), "{dims:?}");
    assert!(dims.iter().any(|d| (d - 76.5).abs() < 1e-9), "{dims:?}");
    // The coupling points: three across each of the two joints between rows, and two on the
    // pusher's bow.
    let marks = entities(&doc, "OBJECT", "circle");
    assert_eq!(marks.len(), 2 * 3 + 2);
    // The sightline (a chain line from the raised eye) and the RAISED wheelhouse.
    let s = &r["formation"]["sightline"];
    assert_eq!(s["enough"], true, "{s}");
    let eye_x = s["eye_x_m"].as_f64().unwrap();
    let rays: Vec<_> = entities(&doc, "CENTRE", "line")
        .into_iter()
        .filter(|l| (l["from_m"][0].as_f64().unwrap() - eye_x).abs() < 1e-9)
        .collect();
    assert_eq!(rays.len(), 1);
    let leaders: Vec<&str> = entities(&doc, "ANNO", "leader")
        .iter()
        .map(|l| l["text"].as_str().unwrap())
        .collect();
    assert!(leaders.contains(&"RAISED"), "{leaders:?}");
    assert!(leaders.contains(&"SIGHTLINE, LIGHT"), "{leaders:?}");
    let notes = doc["notes"].to_string();
    assert!(notes.contains("2 ABREAST BY 3 IN LINE"), "{notes}");
    assert!(notes.contains("BLIND DISTANCE AHEAD OF THE BOW"), "{notes}");
}

#[test]
fn a_convoy_without_its_pusher_draws_the_barges_alone() {
    let o = outcome(convoy(2, 2), false, json!({}));
    let doc = arrangement_plan_named(&o, None).expect("a plan");
    conforms(&doc);
    assert_eq!(barge_outlines(&doc, 11.4).len(), 4);
    // One joint between the rows: three coupling points; no pusher, no sightline.
    assert_eq!(entities(&doc, "OBJECT", "circle").len(), 3);
    assert!(
        horizontal_dimensions(&doc)
            .iter()
            .any(|d| (d - 153.0).abs() < 1e-9)
    );
    assert!(!doc["notes"].to_string().contains("BLIND DISTANCE"));
}

#[test]
fn the_margin_method_gives_no_sightline_result() {
    // A file saved before keeps `margin`: no `sightline` in the Formation's result.
    let o = outcome(
        convoy(2, 3),
        true,
        json!({ "tug_sightline_method": "margin" }),
    );
    assert!(results(&o)["formation"].get("sightline").is_none());
}

#[test]
fn an_atb_draws_its_barge_with_the_notch_and_the_tug_in_it() {
    let o = outcome(json!({ "kind": "atb" }), true, json!({}));
    let doc = arrangement_plan_named(&o, None).expect("a plan");
    conforms(&doc);
    let barges = barge_outlines(&doc, 11.4);
    assert_eq!(barges.len(), 1);
    // The Notch: the outline has eight corners.
    assert_eq!(barges[0].len(), 8);
    // The coupler: two coupling points.
    assert_eq!(entities(&doc, "OBJECT", "circle").len(), 2);
}

#[test]
fn a_tow_and_a_self_propelled_hull_have_no_arrangement_plan() {
    let tow = outcome(json!({ "kind": "towed_barge" }), true, json!({}));
    assert_eq!(
        arrangement_plan_named(&tow, None),
        Err(NO_ARRANGEMENT_PLAN.to_string())
    );
    let own = outcome(json!({ "kind": "self_propelled" }), false, json!({}));
    assert_eq!(
        arrangement_plan_named(&own, None),
        Err(NO_ARRANGEMENT_PLAN.to_string())
    );
    // A pushed barge alone has none either: there is nothing to arrange.
    let barge = outcome(json!({ "kind": "pushed_barge" }), false, json!({}));
    assert!(arrangement_plan_named(&barge, None).is_err());
}

#[test]
fn the_same_results_give_the_same_plan() {
    let o = outcome(
        convoy(3, 2),
        true,
        json!({ "tug_sightline_method": "blind_distance" }),
    );
    assert_eq!(
        arrangement_plan_named(&o, None),
        arrangement_plan_named(&o, None)
    );
}

#[test]
fn a_fixed_wheelhouse_hidden_by_the_barges_says_so_in_words() {
    let o = outcome(
        convoy(2, 3),
        true,
        json!({ "tug_sightline_method": "blind_distance", "tug_wheelhouse_type": "fixed" }),
    );
    let f = &results(&o)["formation"];
    assert_eq!(f["sightline"]["enough"], false);
    let notes = f["notes"].to_string();
    assert!(notes.contains("the barges hide the water"), "{notes}");
    assert!(!notes.contains(" inf "), "{notes}");
}
