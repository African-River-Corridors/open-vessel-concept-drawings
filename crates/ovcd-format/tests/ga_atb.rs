//! Seam: the GA plan and side elevation of an ATB barge draw its Notch and the tug in it (issue #363,
//! spec 0007 "Drawings"): the Notch cut at the transom in the plan outline and as a hidden line in
//! profile, the tug as an outline in the Notch, the coupler as two bars across the gap. Any other
//! Formation draws what it drew.
//!
//! Label: property. Each test states a rule about the drawn entities; no number is a printed result.

use ovcd_format::rule_set::Library;
use ovcd_format::{GaViews, Profile, RunOutcome, fill_as_new, ga_views_of, run_option_with};
use serde_json::{Value, json};

fn views(kind: &str, tug_included: bool, extra: Value) -> GaViews {
    let mut inputs = json!({
        "cargo": { "value": "bauxite", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": kind }, "source": "typed" },
        "length_overall_m": { "value": 100.0, "source": "typed" },
        "beam_m": { "value": 20.0, "source": "typed" },
        "moulded_depth_m": { "value": 6.0, "source": "typed" },
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
    ga_views_of(&outcome).expect("views")
}

fn entities<'a>(document: &'a Value, layer: &str, kind: &str) -> Vec<&'a Value> {
    document["viewports"][0]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["layer"] == layer && e["type"] == kind)
        .collect()
}

/// The tug's outline: the OBJECT polyline of the views that stands aft of the Notch's end wall, 14 m
/// from the stern.
fn tug_outline(document: &Value) -> Vec<[f64; 2]> {
    entities(document, "OBJECT", "polyline")
        .into_iter()
        .map(points)
        .find(|p| p.iter().all(|q| q[0] < 14.0 + 1e-9) && p.iter().any(|q| q[0] < 0.0))
        .expect("the tug outline")
}

/// Equal to rounding.
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

fn points(e: &Value) -> Vec<[f64; 2]> {
    e["points_m"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| [p[0].as_f64().unwrap(), p[1].as_f64().unwrap()])
        .collect()
}

#[test]
fn the_plan_of_an_atb_barge_has_the_notch_cut_at_the_transom_and_the_tug_in_it() {
    let v = views("atb", true, json!({}));
    let outline = points(entities(&v.plan, "OUTLINE", "polyline")[0]);
    // The Notch: 0.28 x 100 m tug, half its length deep (14 m), the tug beam 10.6 m and 0.3 m wide.
    assert_eq!(outline.len(), 8);
    let w = 10.9 / 2.0;
    let has = |x: f64, y: f64| outline.iter().any(|p| near(p[0], x) && near(p[1], y));
    assert!(
        has(14.0, w) && has(14.0, -w) && has(0.0, w) && has(0.0, -w),
        "{outline:?}"
    );
    // The tug: an outline inside the Notch, its bow short of the Notch's end wall, and the coupler.
    let tug = tug_outline(&v.plan);
    assert!(tug.iter().all(|p| p[1].abs() <= 10.6 / 2.0 + 1e-9));
    let bow = tug.iter().map(|p| p[0]).fold(f64::MIN, f64::max);
    assert!(bow < 14.0 && bow > 13.0, "{bow}");
    let bars: Vec<_> = entities(&v.plan, "OBJECT", "line")
        .into_iter()
        .filter(|e| near(e["from_m"][0].as_f64().unwrap(), bow))
        .collect();
    assert_eq!(bars.len(), 2, "two coupler bars");
}

#[test]
fn the_side_elevation_has_the_notch_end_wall_hidden_and_the_tug_outline() {
    let v = views("atb", true, json!({}));
    let hidden = entities(&v.side_elevation, "HIDDEN", "line");
    assert!(
        hidden
            .iter()
            .any(|e| near(e["from_m"][0].as_f64().unwrap(), 14.0)
                && near(e["to_m"][0].as_f64().unwrap(), 14.0)),
        "the Notch's end wall"
    );
    // The tug stands aft of the end wall, its length 28 m, at its own draught.
    let tug = points(
        entities(&v.side_elevation, "OBJECT", "polyline")
            .last()
            .unwrap(),
    );
    let (lo, hi) = tug
        .iter()
        .fold((f64::MAX, f64::MIN), |(a, b), p| (a.min(p[0]), b.max(p[0])));
    assert!(near(hi - lo, 28.0) && hi < 14.0, "{lo} {hi}");
}

#[test]
fn without_the_tug_the_notch_is_drawn_and_the_tug_is_not() {
    let with = views("atb", true, json!({}));
    let without = views("atb", false, json!({}));
    let count = |v: &GaViews| entities(&v.plan, "OBJECT", "polyline").len();
    assert_eq!(count(&without) + 1, count(&with));
    assert_eq!(
        points(entities(&without.plan, "OUTLINE", "polyline")[0]).len(),
        8
    );
}

#[test]
fn a_pushed_barge_keeps_its_rectangle() {
    let v = views("pushed_barge", true, json!({}));
    assert_eq!(points(entities(&v.plan, "OUTLINE", "polyline")[0]).len(), 4);
}
