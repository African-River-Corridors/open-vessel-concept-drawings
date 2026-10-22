//! Seam 5: the general arrangement of a liquid bulk (crude oil) hull (spec 0006 section 4, issue #333).
//! Every drawn tank, cofferdam and pump room is checked against its Space, and every view against the
//! Drawing Format Standard.

use ovcd_draw::{NoFiles, RunResult, run_value};
use ovcd_format::{GaViews, Profile, RunOutcome, check, ga_views_of, run_design};
use serde_json::Value;
use std::path::Path;

/// The drawing tolerance, in metres: a drawn corner is the Space's corner to this.
const TOLERANCE_M: f64 = 1e-6;

const FIXTURES: &[&str] = &[
    "crude_integral_tanks_pushed.vdesign",
    "crude_integral_tanks_self_propelled.vdesign",
    // coastal: slop tanks and a pump room
    "crude_integral_tanks_coastal.vdesign",
];

fn computed(name: &str) -> Vec<(String, Value)> {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name),
    )
    .expect("fixture");
    let design = check(&text, &Profile::built_in())
        .design
        .expect("a fixture is a valid Design");
    run_design(&design)
        .into_iter()
        .filter_map(|r| match r.outcome {
            RunOutcome::Results { results, .. } => Some((r.name, results)),
            _ => None,
        })
        .collect()
}

fn views(results: &Value) -> GaViews {
    ga_views_of(&RunOutcome::Results {
        results: results.clone(),
        hash: String::new(),
    })
    .expect("a liquid bulk hull has views")
}

fn entities(document: &Value) -> &Vec<Value> {
    document["viewports"][0]["entities"]
        .as_array()
        .expect("entities")
}

/// The corners of every closed polyline on a layer.
fn polylines(document: &Value, layer: &str) -> Vec<Vec<[f64; 2]>> {
    entities(document)
        .iter()
        .filter(|e| e["type"] == "polyline" && e["layer"] == layer)
        .map(|e| {
            e["points_m"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| [p[0].as_f64().unwrap(), p[1].as_f64().unwrap()])
                .collect()
        })
        .collect()
}

fn texts(document: &Value) -> Vec<String> {
    entities(document)
        .iter()
        .filter(|e| e["type"] == "leader")
        .map(|e| e["text"].as_str().unwrap().to_string())
        .collect()
}

/// The least and greatest of each coordinate over a list of corners.
fn extent(points: &[[f64; 2]]) -> ([f64; 2], [f64; 2]) {
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    for p in points {
        for i in 0..2 {
            lo[i] = lo[i].min(p[i]);
            hi[i] = hi[i].max(p[i]);
        }
    }
    (lo, hi)
}

fn spaces(results: &Value) -> Vec<Value> {
    results["drawing"]["liquid_tanks"]["spaces"]
        .as_array()
        .expect("the drawing has the Spaces")
        .clone()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOLERANCE_M
}

fn num(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

#[test]
fn every_view_of_every_crude_fixture_validates() {
    for file in FIXTURES {
        let all = computed(file);
        assert!(!all.is_empty(), "{file}: an Option is computed");
        for (option, results) in &all {
            for (view, document) in views(results).all() {
                match run_value(document, &NoFiles) {
                    RunResult::Ok(_) => {}
                    RunResult::Errors(e) => {
                        panic!("{file} {option} {view}: the standard refuses it: {e:?}")
                    }
                }
            }
        }
    }
}

/// Label: sourced - spec 0006 section 4: every drawn element comes from the Spaces. A drawn tank in
/// plan and in elevation has the extent of a Space, and every tank, cofferdam and pump room Space is drawn.
#[test]
fn every_drawn_space_matches_its_space_in_plan_and_elevation() {
    for file in FIXTURES {
        for (option, results) in computed(file) {
            let v = views(&results);
            let plan = polylines(&v.plan, "HIDDEN");
            let side = polylines(&v.side_elevation, "HIDDEN");
            let mut drawn_kinds = 0;
            for sp in spaces(&results) {
                let kind = sp["kind"].as_str().unwrap();
                if !["cargo_tank", "slop", "cofferdam", "pump_room"].contains(&kind) {
                    continue;
                }
                drawn_kinds += 1;
                let (x0, x1) = (num(&sp["x_aft_m"]), num(&sp["x_fwd_m"]));
                let corners: Vec<[f64; 2]> = sp["section_m"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|p| [num(&p[0]), num(&p[1])])
                    .collect();
                let (lo, hi) = extent(&corners);
                let matches = |polys: &[Vec<[f64; 2]>], a: usize| {
                    polys.iter().any(|p| {
                        let (plo, phi) = extent(p);
                        close(plo[0], x0)
                            && close(phi[0], x1)
                            && close(plo[1], lo[a])
                            && close(phi[1], hi[a])
                    })
                };
                let name = sp["name"].as_str().unwrap();
                // A starboard tank of a pair is drawn in plan only: in elevation it stands behind its port twin.
                let in_elevation = !(sp["side"] == "starboard" && kind != "cofferdam");
                assert!(matches(&plan, 0), "{file} {option}: {name} in plan");
                if in_elevation {
                    assert!(matches(&side, 1), "{file} {option}: {name} in elevation");
                }
            }
            assert!(
                drawn_kinds >= 3,
                "{file} {option}: tanks and cofferdams are drawn"
            );
        }
    }
}

/// Label: sourced - spec 0006 section 4: the midship section draws the inner hull from the Spaces.
/// The section's tank and ballast outlines are the corners of the first cargo tank's Spaces.
#[test]
fn the_midship_section_draws_the_corners_of_the_spaces() {
    for file in FIXTURES {
        for (option, results) in computed(file) {
            let v = views(&results);
            let object = polylines(&v.midship_section, "OBJECT");
            let spaces = spaces(&results);
            let first = spaces
                .iter()
                .filter(|s| s["kind"] == "cargo_tank")
                .map(|s| num(&s["x_aft_m"]))
                .fold(f64::INFINITY, f64::min);
            let mut tanks = 0;
            for sp in spaces
                .iter()
                .filter(|s| s["kind"] == "cargo_tank" && close(num(&s["x_aft_m"]), first))
            {
                tanks += 1;
                let corners: Vec<[f64; 2]> = sp["section_m"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|p| [num(&p[0]), num(&p[1])])
                    .collect();
                let (lo, hi) = extent(&corners);
                assert!(
                    object.iter().any(|p| {
                        let (plo, phi) = extent(p);
                        (0..2).all(|i| close(plo[i], lo[i]) && close(phi[i], hi[i]))
                    }),
                    "{file} {option}: tank {} in the section",
                    sp["name"]
                );
            }
            assert!(
                tanks >= 1,
                "{file} {option}: a cargo tank is drawn in the section"
            );
        }
    }
}

/// Label: snapshot - the names a reader sees on the views of a pushed crude barge.
#[test]
fn the_views_name_the_tanks_the_cofferdams_and_the_ballast() {
    let (_, results) = computed("crude_integral_tanks_pushed.vdesign")
        .into_iter()
        .next()
        .expect("an Option");
    let v = views(&results);
    let plan = texts(&v.plan);
    assert!(plan.iter().any(|t| t.starts_with("TANK 1")), "{plan:?}");
    assert!(plan.iter().any(|t| t == "COFFERDAM"), "{plan:?}");
    assert!(plan.iter().any(|t| t == "WBT (BOTH SIDES)"), "{plan:?}");
    let section = texts(&v.midship_section);
    assert!(
        section.iter().any(|t| t.starts_with("CARGO TANK")),
        "{section:?}"
    );
    assert!(section.iter().any(|t| t == "DOUBLE BOTTOM"), "{section:?}");
    // ADR 0011: no pipe routing is drawn
    for (_, doc) in v.all() {
        assert!(!doc.to_string().to_lowercase().contains("pipe"));
    }
}
