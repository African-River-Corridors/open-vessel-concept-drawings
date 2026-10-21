//! GA drawing recovery, ticket 1 (#192): the `simulate` `drawing` object carries the structural values the
//! core already computes, read from the core's own result and inputs.
//!
//! Label: property - each key is compared with the core value it comes from; no number is fixed.

use ovcd_core::barge_capacity::{self, BargeCapacityInputs};
use ovcd_core::gas_arrangement::{self as ga, FitOptions};
use ovcd_core::gas_separation::GasSeparationInputs;
use ovcd_format::{GaViews, Profile, RunOutcome, check, ga_views_of, run_design};
use serde_json::Value;
use std::path::Path;

fn results_of(fixture: &str, self_propelled: bool) -> Value {
    results_at_depth(fixture, self_propelled, "4.5")
}

/// As `results_of`, with the moulded depth (the fixtures say 4.5 m) replaced by `depth`.
fn results_at_depth(fixture: &str, self_propelled: bool, depth: &str) -> Value {
    let mut text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(fixture),
    )
    .expect("fixture")
    .replace("\"value\": 4.5,", &format!("\"value\": {depth},"));
    if self_propelled {
        text = text.replace("pushed_barge", "self_propelled");
    } else {
        text = text.replace("self_propelled", "pushed_barge");
    }
    let design = check(&text, &Profile::built_in())
        .design
        .expect("a valid Design");
    match run_design(&design).remove(0).outcome {
        RunOutcome::Results { results, .. } => results,
        other => panic!("no results: {other:?}"),
    }
}

/// As `results_of`, with a 4 m air draft limit: no arrangement meets it, so the closest one comes
/// back with its failed checks.
fn results_of_low_air_draft(fixture: &str, self_propelled: bool) -> Value {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(fixture),
    )
    .expect("fixture");
    let mut v: Value = serde_json::from_str(&text).unwrap();
    for o in v["options"].as_array_mut().unwrap() {
        o["inputs"]["air_draft_limit_m"] = serde_json::json!({"value": 4.0, "source": "typed"});
        o.as_object_mut().unwrap().remove("results_hash");
    }
    let text = v.to_string();
    let text = if self_propelled {
        text.replace("pushed_barge", "self_propelled")
    } else {
        text.replace("self_propelled", "pushed_barge")
    };
    let design = check(&text, &Profile::built_in())
        .design
        .expect("a valid Design");
    match run_design(&design).remove(0).outcome {
        RunOutcome::Results { results, .. } => results,
        other => panic!("no results: {other:?}"),
    }
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("not a number: {v}"))
}

fn dry_inputs(self_propelled: bool) -> BargeCapacityInputs {
    let mut inp = BargeCapacityInputs::new(76.5, 11.4, 3.5);
    inp.cargo_type = "bauxite".to_string();
    inp.self_propelled = self_propelled;
    inp.block_coefficient = 0.85;
    inp.required_freeboard = 0.3;
    inp.hold_type = "box".to_string();
    inp.lightship_k_t_per_m3 = Some(0.1093);
    inp
}

const GAS_ONLY: &[&str] = &[
    "tank_support_height_m",
    "dome_height_m",
    "wing_tank_width_m",
    "wing_tank_gap_m",
    "opening_setback_m",
    "fwd_end_bulkhead_m",
    "separation",
];

#[test]
fn dry_bulk_geometry_has_the_inner_bottom_and_coaming_from_the_core_inputs() {
    let r = results_of("dry_bulk_box_hold.vdesign", false);
    let g = &r["drawing"];
    let inp = dry_inputs(false);
    assert_eq!(f(&g["double_bottom_m"]), inp.double_bottom_height_bulk);
    assert_eq!(f(&g["hatch_coaming_height_m"]), inp.coaming_height);
    for key in GAS_ONLY {
        assert!(g.get(*key).is_some_and(Value::is_null), "{key} is null");
    }
    // A pushed barge has no aft block.
    assert!(g["aft_arrangement"].is_null());
}

#[test]
fn dry_bulk_self_propelled_geometry_has_the_core_aft_arrangement() {
    let r = results_of("dry_bulk_box_hold.vdesign", true);
    let core = barge_capacity::compute(&dry_inputs(true)).unwrap();
    let a = core.aft_arrangement.expect("the core builds an aft block");
    let g = &r["drawing"]["aft_arrangement"];
    assert_eq!(f(&g["length_m"]), a.length_m);
    assert_eq!(f(&g["thruster_room_length_m"]), a.thruster_room_length_m);
    assert_eq!(f(&g["wheelhouse_x0_m"]), a.wheelhouse_x0_m);
    assert_eq!(f(&g["wheelhouse_x1_m"]), a.wheelhouse_x1_m);
    assert_eq!(
        f(&g["wheelhouse_floor_lowered_m"]),
        a.wheelhouse_floor_lowered_m
    );
    assert_eq!(
        f(&g["wheelhouse_top_lowered_m"]),
        a.wheelhouse_top_lowered_m
    );
    assert_eq!(
        f(&g["wheelhouse_floor_raised_m"]),
        a.wheelhouse_floor_raised_m
    );
    assert_eq!(f(&g["wheelhouse_top_raised_m"]), a.wheelhouse_top_raised_m);
    assert_eq!(f(&g["wheelhouse_width_m"]), a.wheelhouse_width_m);
    assert_eq!(f(&g["container_x0_m"]), a.container_x0_m);
    assert_eq!(f(&g["container_x1_m"]), a.container_x1_m);
    assert_eq!(f(&g["container_total_width_m"]), a.container_total_width_m);
    assert_eq!(
        f(&g["container_top_above_keel_m"]),
        a.container_top_above_keel_m
    );
    assert_eq!(g["container_rows"].as_i64().unwrap(), a.container_rows);
    assert_eq!(
        g["container_per_row"].as_i64().unwrap(),
        a.container_per_row
    );
    assert_eq!(f(&g["block_top_lowered_m"]), a.block_top_lowered_m);
    let offsets: Vec<f64> = g["thruster_offsets_m"]
        .as_array()
        .unwrap()
        .iter()
        .map(f)
        .collect();
    assert_eq!(offsets, a.thruster_offsets_m);
    let zones = g["zones"].as_array().unwrap();
    assert_eq!(zones.len(), a.zones.len());
    for (z, (label, x_aft, x_fwd)) in zones.iter().zip(&a.zones) {
        assert_eq!(z[0].as_str().unwrap(), label);
        assert_eq!(f(&z[1]), *x_aft);
        assert_eq!(f(&z[2]), *x_fwd);
    }
}

fn gas_fit(self_propelled: bool) -> ga::ArrangementFit {
    let options = FitOptions {
        ruleset: "IGC_2G".to_string(),
        fill_ratio: 0.98,
        self_propelled,
        depths: vec![4.5],
        block_coefficient: 0.85,
        max_tank_od_m: 16.0,
        ..FitOptions::default()
    };
    // The hull limit of the fixture: depth less the 0.3 m freeboard.
    ga::fit(110.0, 11.4, "LPG_propane", 4.5 - 0.3, &options).unwrap()
}

#[test]
fn gas_geometry_has_the_core_values() {
    let r = results_of("gas_igc_2g_self_propelled.vdesign", true);
    let g = &r["drawing"];
    let fit = gas_fit(true);
    let core = &fit.engine_result;
    let basis = core.gas_basis.as_ref().expect("a gas basis");
    assert_eq!(
        f(&g["double_bottom_m"]),
        basis.inputs.double_bottom_height.unwrap()
    );
    assert_eq!(
        f(&g["double_bottom_m"]),
        (core.bottom_clearance_m.unwrap() - basis.inputs.tank_support_height).max(0.0)
    );
    assert_eq!(
        f(&g["tank_support_height_m"]),
        basis.inputs.tank_support_height
    );
    assert_eq!(f(&g["dome_height_m"]), basis.inputs.dome_height);
    assert_eq!(
        f(&g["wing_tank_width_m"]),
        basis.inputs.wing_tank_width.unwrap()
    );
    assert_eq!(f(&g["wing_tank_gap_m"]), basis.inputs.wing_tank_gap);
    let sep = core.separation.as_ref().expect("a separation result");
    let want = GasSeparationInputs::default();
    assert_eq!(f(&g["opening_setback_m"]), want.opening_setback_m);
    assert_eq!(f(&g["fwd_end_bulkhead_m"]), want.fwd_end_bulkhead_m);
    assert_eq!(f(&g["separation"]["cofferdam_m"]), sep.cofferdam_m);
    assert_eq!(
        f(&g["separation"]["aft_separation_m"]),
        sep.aft_separation_m
    );
    assert_eq!(
        f(&g["separation"]["fwd_separation_m"]),
        sep.fwd_separation_m
    );
    assert_eq!(f(&g["separation"]["service_space_m"]), sep.service_space_m);
    assert_eq!(
        f(&g["separation"]["total_length_taken_m"]),
        sep.total_length_taken_m
    );
    let a = core.aft_arrangement.as_ref().expect("an aft block");
    assert_eq!(f(&g["aft_arrangement"]["length_m"]), a.length_m);
    assert_eq!(
        f(&g["aft_arrangement"]["block_top_lowered_m"]),
        a.block_top_lowered_m
    );
    // dry-only keys are null on the gas path
    assert!(g["hatch_coaming_height_m"].is_null());
}

#[test]
fn a_pushed_gas_barge_has_a_separation_but_no_aft_block() {
    let r = results_of("gas_igc_2g_self_propelled.vdesign", false);
    let g = &r["drawing"];
    assert!(g["aft_arrangement"].is_null());
    assert!(g["separation"].is_object());
    assert!(g["double_bottom_m"].is_number());
}

#[test]
fn the_drawing_object_is_not_in_the_results_hash() {
    use serde_json::json;
    let without = json!({"cargo_t": 2038.5, "geometry": {"a": 1.0}});
    let with =
        json!({"cargo_t": 2038.5, "geometry": {"a": 1.0}, "drawing": {"double_bottom_m": 1.2}});
    assert_eq!(
        ovcd_format::results_hash(&without),
        ovcd_format::results_hash(&with)
    );
}

#[test]
fn existing_stored_hashes_still_match_with_the_drawing_object_present() {
    for fixture in [
        "dry_bulk_box_hold.vdesign",
        "gas_igc_2g_self_propelled.vdesign",
        "gas_igc_2g_exact.vdesign",
    ] {
        let text = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(fixture),
        )
        .unwrap();
        let design = check(&text, &Profile::built_in()).design.expect("valid");
        let run = run_design(&design).remove(0);
        match &run.outcome {
            RunOutcome::Results { results, .. } => {
                assert!(results["drawing"].is_object(), "{fixture}: drawing present")
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(
            run.hash_status,
            ovcd_format::HashStatus::Matches,
            "{fixture}"
        );
    }
}

// ---- #193: the dry-bulk hold floor (inner bottom) and the hatch coaming -------------------------

fn views(results: &Value) -> GaViews {
    ga_views_of(&RunOutcome::Results {
        results: results.clone(),
        hash: String::new(),
    })
    .expect("views")
}

fn entities(document: &Value) -> &Vec<Value> {
    document["viewports"][0]["entities"].as_array().unwrap()
}

fn of_layer<'a>(document: &'a Value, layer: &str, kind: &str) -> Vec<&'a Value> {
    entities(document)
        .iter()
        .filter(|e| e["layer"] == layer && e["type"] == kind)
        .collect()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// Both ends of a line: ((x, y), (x, y)).
fn ends(e: &Value) -> ((f64, f64), (f64, f64)) {
    let p = |k: &str| (f(&e[k][0]), f(&e[k][1]));
    (p("from_m"), p("to_m"))
}

fn points(e: &Value) -> Vec<(f64, f64)> {
    e["points_m"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (f(&p[0]), f(&p[1])))
        .collect()
}

const HOLDS: &[&str] = &[
    "dry_bulk_box_hold.vdesign",
    "dry_bulk_sloped_hopper.vdesign",
];

#[test]
fn the_hold_floor_is_the_inner_bottom_not_depth_less_hold_height() {
    for file in HOLDS {
        let r = results_of(file, false);
        let depth = f(&r["dims"]["depth_m"]);
        let db = f(&r["drawing"]["double_bottom_m"]);
        let coaming = f(&r["drawing"]["hatch_coaming_height_m"]);
        let hold_h = f(&r["geometry"]["hold_height_m"]);
        // The core's hold height runs from the inner bottom to the coaming top, so
        // `depth - hold_height` is the inner bottom less the coaming: the two do not agree.
        assert!(close(depth - hold_h, db - coaming), "{file}");
        assert!(!close(depth - hold_h, db), "{file}");

        let v = views(&r);
        // side elevation: the inner bottom is a hidden line across the hull, at the inner bottom
        let hidden = of_layer(&v.side_elevation, "HIDDEN", "line");
        assert!(
            hidden.iter().any(|e| {
                let (a, b) = ends(e);
                close(a.1, db) && close(b.1, db) && b.0 > a.0
            }),
            "{file}: a hidden line at the inner bottom"
        );
        let hold = of_layer(&v.side_elevation, "HIDDEN", "polyline");
        let low = points(hold[0])
            .iter()
            .map(|p| p.1)
            .fold(f64::INFINITY, f64::min);
        assert!(close(low, db), "{file}: the hold floor in elevation");
        // the coaming stands above the deck, at its height
        let top = of_layer(&v.side_elevation, "OUTLINE", "polyline")
            .iter()
            .flat_map(|e| points(e))
            .map(|p| p.1)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            close(top, depth + coaming),
            "{file}: coaming top in elevation"
        );

        // midship section: a line across the hull at the inner bottom, and the coaming above deck
        let half = f(&r["dims"]["beam_m"]) / 2.0;
        let section_lines = of_layer(&v.midship_section, "OUTLINE", "line");
        assert!(
            section_lines.iter().any(|e| {
                let (a, b) = ends(e);
                close(a.1, db) && close(b.1, db) && close(a.0.abs(), half) && close(b.0.abs(), half)
            }),
            "{file}: the inner bottom across the hull"
        );
        let top_w = f(&r["geometry"]["hold_top_width_m"]);
        let coamings: Vec<_> = section_lines
            .iter()
            .filter(|e| {
                let (a, b) = ends(e);
                close(a.0, b.0)
                    && close(a.1.min(b.1), depth)
                    && close(a.1.max(b.1), depth + coaming)
            })
            .collect();
        assert_eq!(
            coamings.len(),
            2,
            "{file}: two coaming sides in the section"
        );
        assert!(
            coamings
                .iter()
                .all(|e| close(ends(e).0.0.abs(), top_w / 2.0))
        );
        // the hold polyline in section ends on the inner bottom
        let hold_section = of_layer(&v.midship_section, "OUTLINE", "polyline");
        assert!(
            hold_section
                .iter()
                .any(|p| points(p).iter().any(|q| close(q.1, db)) && points(p).len() == 4),
            "{file}: hold section floor at the inner bottom"
        );

        // plan: the hold opening, the width and length of the coaming
        let opening = of_layer(&v.plan, "OBJECT", "polyline");
        let pts = points(opening[0]);
        let x0 = f(&r["geometry"]["stern_void_length_m"]);
        let x1 = f(&r["dims"]["loa_m"]) - f(&r["geometry"]["bow_void_length_m"]);
        assert!(pts.iter().any(|p| close(p.0, x0)) && pts.iter().any(|p| close(p.0, x1)));
        assert!(pts.iter().all(|p| close(p.1.abs(), top_w / 2.0)));
    }
}

// ---- #194: the gas tank in the midship section and the elevation ---------------------------------

const GAS_FIXTURES: &[&str] = &[
    "gas_igc_2g_self_propelled.vdesign",
    "gas_igc_2g_exact.vdesign",
];

/// The tank centres across the beam, read off the circles of the section.
fn circles(v: &GaViews) -> Vec<((f64, f64), f64)> {
    of_layer(&v.midship_section, "OUTLINE", "circle")
        .iter()
        .map(|e| {
            (
                (f(&e["centre_m"][0]), f(&e["centre_m"][1])),
                f(&e["radius_m"]),
            )
        })
        .collect()
}

fn leader<'a>(document: &'a Value, starts: &str) -> &'a Value {
    entities(document)
        .iter()
        .find(|e| {
            e["type"] == "leader"
                && e["text"]
                    .as_str()
                    .is_some_and(|t| starts.split('|').any(|s| t.starts_with(s)))
        })
        .unwrap_or_else(|| panic!("no leader `{starts}`"))
}

#[test]
fn the_tank_stands_on_the_double_bottom_plus_support_in_both_views() {
    for file in GAS_FIXTURES {
        for self_propelled in [false, true] {
            let r = results_of(file, self_propelled);
            let d = &r["drawing"];
            let od = f(&r["geometry"]["tank_outer_diameter_m"]);
            let want = f(&d["double_bottom_m"]) + f(&d["tank_support_height_m"]) + od / 2.0;
            let v = views(&r);
            let section = circles(&v);
            assert!(!section.is_empty());
            for ((_, y), radius) in &section {
                assert!(
                    close(*y, want) && close(*radius, od / 2.0),
                    "{file}: section"
                );
            }
            // the elevation's tank arcs have the same centre height
            let arcs = of_layer(&v.side_elevation, "HIDDEN", "arc");
            assert!(!arcs.is_empty());
            assert!(
                arcs.iter().all(|a| close(f(&a["centre_m"][1]), want)),
                "{file}: elevation"
            );
            if !self_propelled {
                // With no aft block the obstruction height is the tank top: the tank top above the
                // keel, less the dome, is the drawn top of the tank.
                let top = f(&r["results"]["air_draft_loaded_m"]) + f(&r["results"]["draught_m"]);
                assert!(
                    close(top - f(&d["dome_height_m"]), want + od / 2.0),
                    "{file}"
                );
            }
        }
    }
}

#[test]
fn the_side_clearance_leader_points_into_the_gap_between_hull_side_and_tank() {
    for file in GAS_FIXTURES {
        let r = results_of(file, false);
        let v = views(&r);
        let half = f(&r["dims"]["beam_m"]) / 2.0;
        let depth = f(&r["dims"]["depth_m"]);
        let target = &leader(&v.midship_section, "SIDE CLEARANCE")["from"]["point_m"];
        let (x, y) = (f(&target[0]), f(&target[1]));
        let tanks = circles(&v);
        let ((xc, yc), radius) = tanks
            .iter()
            .min_by(|a, b| (a.0.0).partial_cmp(&b.0.0).unwrap())
            .copied()
            .unwrap();
        assert!(y > 0.0 && y < depth, "{file}: below the deck");
        assert!(
            x > -half && x < xc,
            "{file}: between the hull side and the tank centre"
        );
        let dist = ((x - xc) * (x - xc) + (y - yc) * (y - yc)).sqrt();
        assert!(dist > radius, "{file}: outside the tank");
        // and it is at the tank's centre height, or just below the deck if the centre is above it
        assert!(
            y <= yc.min(depth) && y > yc.min(depth) - 0.1 * depth,
            "{file}: height"
        );
    }
}

#[test]
fn the_bottom_clearance_leader_points_at_the_gap_under_the_tank() {
    for file in GAS_FIXTURES {
        let r = results_of(file, false);
        let d = &r["drawing"];
        let bottom = f(&d["double_bottom_m"]) + f(&d["tank_support_height_m"]);
        let v = views(&r);
        let l = leader(&v.midship_section, "TANK BOTTOM|BOTTOM CLEARANCE");
        let (x, y) = (f(&l["from"]["point_m"][0]), f(&l["from"]["point_m"][1]));
        assert!(
            y > 0.0 && y < bottom,
            "{file}: in the gap between the keel and the tank"
        );
        let ((xc, _), _) = circles(&v)[0];
        assert!(close(x, xc), "{file}: under the nearest tank");
    }
}

#[test]
fn the_section_deck_line_stops_at_a_tank_and_never_crosses_one() {
    let mut crossed = 0;
    for file in GAS_FIXTURES {
        for self_propelled in [false, true] {
            let r = results_of(file, self_propelled);
            let v = views(&r);
            let depth = f(&r["dims"]["depth_m"]);
            let half = f(&r["dims"]["beam_m"]) / 2.0;
            let deck: Vec<(f64, f64)> = of_layer(&v.midship_section, "OUTLINE", "line")
                .iter()
                .filter_map(|e| {
                    let (a, b) = ends(e);
                    (close(a.1, depth) && close(b.1, depth)).then_some((a.0.min(b.0), a.0.max(b.0)))
                })
                .collect();
            for ((xc, yc), radius) in circles(&v) {
                let dy = depth - yc;
                if dy.abs() >= radius {
                    continue;
                }
                crossed += 1;
                let w = (radius * radius - dy * dy).sqrt();
                for (a, b) in &deck {
                    assert!(
                        *b <= xc - w + 1e-9 || *a >= xc + w - 1e-9,
                        "{file}: a deck segment crosses a tank"
                    );
                }
                // the line starts again past the tank
                assert!(deck.iter().any(|(a, _)| close(*a, xc + w)) || close(xc + w, half));
                assert!(deck.iter().any(|(_, b)| close(*b, xc - w)) || close(xc - w, -half));
            }
        }
    }
    assert!(crossed > 0, "no fixture has a tank through the deck");
}

// ---- #195: the part of a tank above the deck is in view; the part below is hidden ---------------

/// The point of an arc entity at `deg` degrees.
fn arc_point(e: &Value, deg: f64) -> (f64, f64) {
    let (cx, cy, r) = (
        f(&e["centre_m"][0]),
        f(&e["centre_m"][1]),
        f(&e["radius_m"]),
    );
    (
        cx + r * libm::cos(deg.to_radians()),
        cy + r * libm::sin(deg.to_radians()),
    )
}

/// Sample points of an arc, from start to end counter-clockwise.
fn arc_samples(e: &Value) -> Vec<(f64, f64)> {
    let (s, t) = (f(&e["start_deg"]), f(&e["end_deg"]));
    let sweep = (t - s).rem_euclid(360.0);
    (0..=20)
        .map(|i| arc_point(e, s + sweep * f64::from(i) / 20.0))
        .collect()
}

fn line_ys(e: &Value) -> Vec<f64> {
    let (a, b) = ends(e);
    vec![a.1, b.1]
}

#[test]
fn in_elevation_the_tank_above_the_deck_is_visible_and_below_it_hidden() {
    let mut split = 0;
    for file in GAS_FIXTURES {
        for self_propelled in [false, true] {
            let r = results_of(file, self_propelled);
            let v = views(&r);
            let depth = f(&r["dims"]["depth_m"]);
            let eps = 1e-9;
            let ent = &v.side_elevation;
            // visible tank pieces (OBJECT in elevation is only the tank) lie at or above the deck
            for e in of_layer(ent, "OBJECT", "line") {
                assert!(
                    line_ys(e).iter().all(|y| *y >= depth - eps),
                    "{file}: visible line below deck"
                );
            }
            for e in of_layer(ent, "OBJECT", "arc") {
                assert!(
                    arc_samples(e).iter().all(|p| p.1 >= depth - eps),
                    "{file}: visible arc below deck"
                );
            }
            // hidden tank pieces (the polylines are not tank pieces) lie at or below it
            for e in of_layer(ent, "HIDDEN", "line") {
                assert!(
                    line_ys(e).iter().all(|y| *y <= depth + eps),
                    "{file}: hidden line above deck"
                );
            }
            for e in of_layer(ent, "HIDDEN", "arc") {
                assert!(
                    arc_samples(e).iter().all(|p| p.1 <= depth + eps),
                    "{file}: hidden arc above deck"
                );
            }
            // the pieces join at the deck: a visible and a hidden arc end meet at deck height
            let visible = of_layer(ent, "OBJECT", "arc");
            let hidden = of_layer(ent, "HIDDEN", "arc");
            for a in &visible {
                let ends_a = [f(&a["start_deg"]), f(&a["end_deg"])].map(|d| arc_point(a, d));
                for pa in ends_a.iter().filter(|p| close(p.1, depth)) {
                    let joined = hidden.iter().any(|b| {
                        [f(&b["start_deg"]), f(&b["end_deg"])]
                            .map(|d| arc_point(b, d))
                            .iter()
                            .any(|pb| close(pb.0, pa.0) && close(pb.1, pa.1))
                    });
                    assert!(
                        joined,
                        "{file}: a visible arc meets no hidden arc at the deck"
                    );
                    split += 1;
                }
            }
            // the tank is drawn at the section's centre height
            let want = f(&r["drawing"]["double_bottom_m"])
                + f(&r["drawing"]["tank_support_height_m"])
                + f(&r["geometry"]["tank_outer_diameter_m"]) / 2.0;
            for a in visible.iter().chain(hidden.iter()) {
                assert!(close(f(&a["centre_m"][1]), want));
            }
        }
    }
    assert!(split > 0, "no fixture has a tank through the deck");
}

#[test]
fn in_plan_the_tank_part_above_the_deck_is_visible() {
    let mut cases = [0; 3];
    for file in GAS_FIXTURES {
        let r = results_of(file, false);
        let v = views(&r);
        let depth = f(&r["dims"]["depth_m"]);
        let radius = f(&r["geometry"]["tank_outer_diameter_m"]) / 2.0;
        let yc = f(&r["drawing"]["double_bottom_m"])
            + f(&r["drawing"]["tank_support_height_m"])
            + radius;
        // the athwartships centres of the tanks, from the section circles
        let centres: Vec<f64> = circles(&v).iter().map(|c| (c.0).0).collect();
        let has = |layer: &str, half: f64| {
            let lines = of_layer(&v.plan, layer, "line");
            centres.iter().all(|c| {
                [c - half, c + half].iter().all(|y| {
                    lines
                        .iter()
                        .any(|e| close(ends(e).0.1, *y) && close(ends(e).1.1, *y))
                })
            })
        };
        // (a tank line runs along the hull; the cofferdam bulkheads, also hidden, run across it)
        let any_hidden = of_layer(&v.plan, "HIDDEN", "line")
            .iter()
            .any(|e| close(ends(e).0.1, ends(e).1.1));
        if yc >= depth {
            // the widest part is above the deck: the whole outline is in view, nothing hidden
            assert!(has("OBJECT", radius) && !any_hidden, "{file}");
            cases[0] += 1;
        } else if yc + radius > depth {
            // the widest part is under the deck (hidden); above it the tank is narrower, in view
            let dy = depth - yc;
            let w = (radius * radius - dy * dy).sqrt();
            assert!(has("HIDDEN", radius) && has("OBJECT", w), "{file}");
            cases[1] += 1;
        } else {
            assert!(has("HIDDEN", radius), "{file}");
            cases[2] += 1;
        }
    }
    assert!(cases.iter().sum::<i32>() > 0);
}

// ---- #196: the section detail of the earlier Python GA drawer -------------------------------------

fn texts(document: &Value) -> Vec<String> {
    entities(document)
        .iter()
        .filter(|e| e["type"] == "leader")
        .map(|e| e["text"].as_str().unwrap().to_string())
        .collect()
}

fn has_leader(document: &Value, text: &str) -> bool {
    entities(document)
        .iter()
        .any(|e| e["type"] == "leader" && e["layer"] == "ANNO" && e["text"] == text)
}

/// Is there a line (any direction) of this layer from `a` to `b`, in either order?
fn has_line(document: &Value, layer: &str, a: (f64, f64), b: (f64, f64)) -> bool {
    let same = |p: (f64, f64), q: (f64, f64)| close(p.0, q.0) && close(p.1, q.1);
    of_layer(document, layer, "line").iter().any(|e| {
        let (p, q) = ends(e);
        (same(p, a) && same(q, b)) || (same(p, b) && same(q, a))
    })
}

fn polylines_with(document: &Value, layer: &str, x: f64) -> Vec<Vec<(f64, f64)>> {
    of_layer(document, layer, "polyline")
        .iter()
        .map(|e| points(e))
        .filter(|p| p.iter().any(|q| close(q.0, x)))
        .collect()
}

#[test]
fn the_section_draws_the_double_bottom_wing_tanks_voids_cradles_and_domes_from_the_results() {
    let mut with_double_bottom = 0;
    let mut with_wing_tank = 0;
    // Since issue #264 the search keeps only arrangements that pass stability, and the fixtures at
    // their own 4.5 m depth no longer leave a double bottom, so a deeper hull is drawn too.
    for (file, depth) in GAS_FIXTURES.iter().flat_map(|f| [(f, "4.5"), (f, "6.0")]) {
        for self_propelled in [false, true] {
            let r = results_at_depth(file, self_propelled, depth);
            let v = views(&r);
            let d = &r["drawing"];
            let half = f(&r["dims"]["beam_m"]) / 2.0;
            let depth = f(&r["dims"]["depth_m"]);
            let od = f(&r["geometry"]["tank_outer_diameter_m"]);
            let cs = f(&r["geometry"]["side_clearance_m"]);
            let (db, support) = (f(&d["double_bottom_m"]), f(&d["tank_support_height_m"]));
            let ent = &v.midship_section;

            // the tank stands on double bottom + support height once, not twice
            let bottom = circles(&v)
                .iter()
                .map(|((_, y), r)| y - r)
                .fold(f64::INFINITY, f64::min);
            assert!(close(bottom, db + support), "{file}: tank bottom");
            assert!(
                !close(bottom, db + support + support),
                "{file}: not on the support twice"
            );

            // double bottom: a line across the hull and a line hatch in the band under it
            let hatches = of_layer(ent, "HATCH", "hatch");
            if db > 0.0 {
                with_double_bottom += 1;
                assert!(has_line(ent, "OBJECT", (-half, db), (half, db)), "{file}");
                assert_eq!(hatches.len(), 1, "{file}");
                assert_eq!(hatches[0]["pattern"], "lines");
                let ys: Vec<f64> = hatches[0]["boundary_m"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|p| f(&p[1]))
                    .collect();
                assert!(
                    ys.iter().any(|y| close(*y, 0.0)) && ys.iter().any(|y| close(*y, db)),
                    "{file}"
                );
                assert!(has_leader(ent, "DOUBLE BOTTOM"));
            } else {
                assert!(hatches.is_empty(), "{file}: no band, no hatch");
                assert!(!has_leader(ent, "DOUBLE BOTTOM"));
            }

            // wing tanks: the inboard wall from the double bottom to the top, on both sides
            let w = f(&d["wing_tank_width_m"]);
            let gap = f(&d["wing_tank_gap_m"]);
            let top = (db + support + f(&d["wing_tank_height_ratio"]) * od).min(depth);
            // The wing width is the side clearance less the gap (issue #283, built in #277): a
            // clearance no wider than the gap leaves no wing tank, and none is drawn.
            if w > 0.0 {
                with_wing_tank += 1;
                assert!(w < cs, "{file}");
                for side in [-1.0, 1.0] {
                    let wall = polylines_with(ent, "OBJECT", side * (half - w));
                    assert_eq!(wall.len(), 1, "{file}");
                    assert!(
                        wall[0].iter().any(|p| close(p.1, db))
                            && wall[0].iter().any(|p| close(p.1, top))
                    );
                    // the void, to the cargo tank's side clearance at most
                    let edge = side * (half - w - gap.min(cs - w));
                    let void = polylines_with(ent, "ZONE", edge);
                    assert_eq!(void.len(), 1, "{file}: void on the ZONE layer");
                    // the void lies between the wing tank wall and the cargo tank's side clearance
                    assert!(void[0].iter().all(|p| {
                        p.0.abs() >= half - cs - 1e-9 && p.0.abs() <= half - w + 1e-9
                    }));
                }
                assert!(has_leader(ent, "WBT (BOTH SIDES)"));
                assert!(has_leader(ent, "VOID (BOTH SIDES)"));
            } else {
                assert!(
                    cs <= gap,
                    "{file}: no wing tank only when the gap takes the clearance"
                );
                assert!(!has_leader(ent, "WBT (BOTH SIDES)"));
                assert!(!has_leader(ent, "VOID (BOTH SIDES)"));
            }

            // a cradle under each tank, a little outside it, with its legs to the double bottom
            let arcs = of_layer(ent, "OBJECT", "arc");
            assert_eq!(arcs.len(), circles(&v).len(), "{file}");
            for (a, ((cx, cy), radius)) in arcs.iter().zip(circles(&v)) {
                assert!(close(f(&a["centre_m"][0]), cx) && close(f(&a["centre_m"][1]), cy));
                assert!(close(f(&a["radius_m"]), radius + 0.05));
                let (s, e) = (
                    arc_point(a, f(&a["start_deg"])),
                    arc_point(a, f(&a["end_deg"])),
                );
                assert!(
                    has_line(ent, "OBJECT", (s.0, s.1), (s.0, db)),
                    "{file}: left leg"
                );
                assert!(
                    has_line(ent, "OBJECT", (e.0, e.1), (e.0, db)),
                    "{file}: right leg"
                );
            }
            assert!(has_leader(ent, "CRADLE"));

            // a dome on each tank, to the dome height above the tank top
            let dome = f(&d["dome_height_m"]);
            let top_of_tank = db + support + od;
            let apexes: Vec<f64> = of_layer(ent, "OBJECT", "polyline")
                .iter()
                .map(|e| points(e))
                .filter(|p| p.len() == 17)
                .map(|p| p.iter().map(|q| q.1).fold(f64::NEG_INFINITY, f64::max))
                .collect();
            assert_eq!(apexes.len(), circles(&v).len(), "{file}");
            assert!(
                apexes.iter().all(|y| close(*y, top_of_tank + dome)),
                "{file}: dome height"
            );
            assert!(has_leader(ent, "DOME"));
            // as wide as the dome diameter in force (issue #316)
            let width = f(&d["tank"]["dome_diameter_m"]).min(0.95 * od);
            let spans: Vec<f64> = of_layer(ent, "OBJECT", "polyline")
                .iter()
                .map(|e| points(e))
                .filter(|p| p.len() == 17)
                .map(|p| {
                    let xs = p.iter().map(|q| q.0);
                    xs.clone().fold(f64::NEG_INFINITY, f64::max) - xs.fold(f64::INFINITY, f64::min)
                })
                .collect();
            assert!(
                spans.iter().all(|w| close(*w, width)),
                "{file}: dome width {spans:?} against {width}"
            );
        }
    }
    assert!(with_double_bottom > 0);
    assert!(with_wing_tank > 0);
}

#[test]
fn the_elevation_and_plan_draw_the_cofferdam_zone_limits_and_forward_equipment() {
    for file in GAS_FIXTURES {
        let r = results_of(file, true);
        let v = views(&r);
        let loa = f(&r["dims"]["loa_m"]);
        let half = f(&r["dims"]["beam_m"]) / 2.0;
        let depth = f(&r["dims"]["depth_m"]);
        let stern = f(&r["geometry"]["stern_void_length_m"]);
        let bow = f(&r["geometry"]["bow_void_length_m"]);
        let sep = &r["drawing"]["separation"];
        let start = stern - f(&sep["aft_separation_m"]);
        let end = start + f(&sep["cofferdam_m"]);
        for (view, lo, hi) in [(&v.side_elevation, 0.0, depth), (&v.plan, -half, half)] {
            // the cargo zone limits: dashed ZONE lines
            for x in [stern, loa - bow] {
                assert!(
                    has_line(view, "ZONE", (x, lo), (x, hi)),
                    "{file}: zone limit at {x}"
                );
            }
            // the cofferdam's two bulkheads, hidden under the deck
            for x in [start, end] {
                assert!(
                    has_line(view, "HIDDEN", (x, lo), (x, hi)),
                    "{file}: cofferdam bulkhead at {x}"
                );
            }
            assert!(has_leader(view, "COFFERDAM") && has_leader(view, "FWD EQUIP."));
        }
        // elevation: the double bottom is a hidden line along the hull, the wing tank a hidden band
        let db = f(&r["drawing"]["double_bottom_m"]);
        if db > 0.0 {
            let hidden = of_layer(&v.side_elevation, "HIDDEN", "line");
            assert!(hidden.iter().any(|e| close(ends(e).0.1, db)
                && close(ends(e).1.1, db)
                && ends(e).1.0 - ends(e).0.0 > loa / 2.0));
        }
        assert!(has_leader(&v.side_elevation, "WBT (BOTH SIDES)"));
        // elevation domes: one on each tank; the tank top is above the deck here, so they are in view
        let top = f(&r["drawing"]["double_bottom_m"])
            + f(&r["drawing"]["tank_support_height_m"])
            + f(&r["geometry"]["tank_outer_diameter_m"]);
        let domes: Vec<_> = of_layer(&v.side_elevation, "OBJECT", "polyline")
            .iter()
            .map(|e| points(e))
            .filter(|p| p.len() == 17)
            .collect();
        assert!(!domes.is_empty());
        for p in domes {
            let apex = p.iter().map(|q| q.1).fold(f64::NEG_INFINITY, f64::max);
            assert!(
                close(apex, top + f(&r["drawing"]["dome_height_m"])),
                "{file}"
            );
        }
    }
}

#[test]
fn a_dry_bulk_design_draws_its_zone_limits_and_forward_equipment_and_no_gas_items() {
    for file in HOLDS {
        let r = results_of(file, false);
        let v = views(&r);
        let loa = f(&r["dims"]["loa_m"]);
        let depth = f(&r["dims"]["depth_m"]);
        let half = f(&r["dims"]["beam_m"]) / 2.0;
        let (stern, bow) = (
            f(&r["geometry"]["stern_void_length_m"]),
            f(&r["geometry"]["bow_void_length_m"]),
        );
        for (view, lo, hi) in [(&v.side_elevation, 0.0, depth), (&v.plan, -half, half)] {
            for x in [stern, loa - bow] {
                assert!(has_line(view, "ZONE", (x, lo), (x, hi)), "{file}");
            }
            assert!(has_leader(view, "FWD EQUIP."), "{file}");
        }
        for (name, view) in [
            ("elevation", &v.side_elevation),
            ("plan", &v.plan),
            ("section", &v.midship_section),
        ] {
            let t = texts(view);
            for gas in [
                "COFFERDAM",
                "DOME",
                "CRADLE",
                "WBT (BOTH SIDES)",
                "VOID (BOTH SIDES)",
                "DOUBLE BOTTOM",
            ] {
                assert!(!t.iter().any(|x| x == gas), "{file} {name}: {gas}");
            }
            assert!(of_layer(view, "HATCH", "hatch").is_empty(), "{file} {name}");
        }
    }
}

#[test]
fn an_item_whose_value_is_null_is_left_out_without_an_error() {
    let mut r = results_of("gas_igc_2g_self_propelled.vdesign", false);
    for key in [
        "wing_tank_width_m",
        "wing_tank_gap_m",
        "wing_tank_height_ratio",
        "dome_height_m",
        "separation",
    ] {
        r["drawing"][key] = Value::Null;
    }
    let v = views(&r);
    for (name, view) in [
        ("elevation", &v.side_elevation),
        ("plan", &v.plan),
        ("section", &v.midship_section),
    ] {
        let t = texts(view);
        for gone in ["COFFERDAM", "DOME", "WBT (BOTH SIDES)", "VOID (BOTH SIDES)"] {
            assert!(!t.iter().any(|x| x == gone), "{name}: {gone}");
        }
    }
    // what is still given is still drawn
    assert!(has_leader(&v.midship_section, "CRADLE"));
    assert!(has_leader(&v.side_elevation, "FWD EQUIP."));
}

// ---- #197: the aft block of a self-propelled hull ---------------------------------------------------

/// A self-propelled gas Design and a self-propelled dry-bulk Design.
fn self_propelled() -> Vec<(&'static str, Value)> {
    [
        "gas_igc_2g_self_propelled.vdesign",
        "dry_bulk_box_hold.vdesign",
    ]
    .into_iter()
    .map(|file| (file, results_of(file, true)))
    .collect()
}

fn leader_target(document: &Value, text: &str) -> (f64, f64) {
    let l = leader(document, text);
    (f(&l["from"]["point_m"][0]), f(&l["from"]["point_m"][1]))
}

#[test]
fn the_aft_block_x_is_measured_from_the_stern_and_ends_where_the_stern_void_does_less_the_separation()
 {
    for (file, r) in self_propelled() {
        let a = &r["drawing"]["aft_arrangement"];
        let stern_void = f(&r["geometry"]["stern_void_length_m"]);
        // the block starts at the stern (x = 0) and its last zone ends at the block length
        let zones = a["zones"].as_array().unwrap();
        assert!(
            close(f(&zones[0][1]), 0.0),
            "{file}: the first zone starts at the stern"
        );
        assert!(
            close(f(&zones.last().unwrap()[2]), f(&a["length_m"])),
            "{file}"
        );
        // the stern void is the block plus the separation to the cargo zone: the gas separation,
        // or the dry-bulk end bulkhead space (0.5 m, the core's default)
        let gap = if r["drawing"]["separation"].is_object() {
            f(&r["drawing"]["separation"]["aft_separation_m"])
        } else {
            0.5
        };
        assert!(close(stern_void, f(&a["length_m"]) + gap), "{file}");
    }
}

#[test]
fn the_elevation_draws_the_zone_bulkheads_containers_wheelhouse_and_thrusters_where_the_results_put_them()
 {
    for (file, r) in self_propelled() {
        let a = &r["drawing"]["aft_arrangement"];
        let v = views(&r);
        let ent = &v.side_elevation;
        let depth = f(&r["dims"]["depth_m"]);
        // zone bulkheads: the forward end of each zone, under the deck
        // (for a gas hull the last bulkhead is also the cofferdam's first, drawn once)
        for z in a["zones"].as_array().unwrap() {
            let x = f(&z[2]);
            assert!(
                has_line(ent, "HIDDEN", (x, 0.0), (x, depth)),
                "{file}: bulkhead at {x}"
            );
        }
        // containers: one box a row, from the deck to the container top, on EQUIP
        let top = f(&a["container_top_above_keel_m"]);
        let boxes: Vec<Vec<(f64, f64)>> = of_layer(ent, "EQUIP", "polyline")
            .iter()
            .map(|e| points(e))
            .collect();
        assert_eq!(
            boxes.len(),
            a["container_rows"].as_u64().unwrap() as usize,
            "{file}"
        );
        for (row, b) in boxes.iter().enumerate() {
            let x0 = f(&a["container_x0_m"])
                + row as f64 * (f(&a["container_row_length_m"]) + f(&a["container_row_gap_m"]));
            assert!(
                b.iter().any(|p| close(p.0, x0) && close(p.1, depth)),
                "{file}"
            );
            assert!(
                b.iter()
                    .any(|p| close(p.0, x0 + f(&a["container_row_length_m"])) && close(p.1, top)),
                "{file}"
            );
        }
        // the wheelhouse at its lowered height, on OBJECT
        let (wx0, wx1) = (f(&a["wheelhouse_x0_m"]), f(&a["wheelhouse_x1_m"]));
        let (floor, wtop) = (
            f(&a["wheelhouse_floor_lowered_m"]),
            f(&a["wheelhouse_top_lowered_m"]),
        );
        let cab = of_layer(ent, "OBJECT", "polyline")
            .iter()
            .map(|e| points(e))
            .find(|p| {
                p.iter().any(|q| close(q.0, wx0))
                    && p.iter().any(|q| close(q.0, wx1))
                    && p.iter().any(|q| close(q.1, wtop))
            })
            .unwrap_or_else(|| panic!("{file}: no wheelhouse"));
        assert!(cab.iter().any(|p| close(p.1, floor)), "{file}");
        // a thruster circle in the steering flat
        let room = f(&a["thruster_room_length_m"]);
        let circles_e: Vec<_> = of_layer(ent, "EQUIP", "circle");
        assert_eq!(circles_e.len(), 1, "{file}");
        assert!(close(f(&circles_e[0]["centre_m"][0]), room / 2.0), "{file}");
        // the block top is named, at its height
        let block_top = f(&a["block_top_lowered_m"]);
        let cab_label = format!(
            "WHEELHOUSE, BLOCK TOP {} m",
            ovcd_core::pyfmt::fixed(block_top, 2)
        );
        assert!(close(leader_target(ent, &cab_label).1, block_top), "{file}");
        assert!(has_leader(
            ent,
            &format!(
                "CONTAINERS ({} x {})",
                a["container_per_row"], a["container_rows"]
            )
        ));
        assert!(has_leader(
            ent,
            &format!(
                "THRUSTERS ({})",
                a["thruster_offsets_m"].as_array().unwrap().len()
            )
        ));
    }
}

#[test]
fn the_plan_draws_the_wheelhouse_footprint_container_rows_and_thrusters_where_the_results_put_them()
{
    for (file, r) in self_propelled() {
        let a = &r["drawing"]["aft_arrangement"];
        let v = views(&r);
        let ent = &v.plan;
        // wheelhouse footprint on OBJECT
        let hw = f(&a["wheelhouse_width_m"]) / 2.0;
        let (wx0, wx1) = (f(&a["wheelhouse_x0_m"]), f(&a["wheelhouse_x1_m"]));
        let cab = of_layer(ent, "OBJECT", "polyline")
            .iter()
            .map(|e| points(e))
            .find(|p| p.len() == 4 && p.iter().all(|q| close(q.1.abs(), hw)))
            .unwrap_or_else(|| panic!("{file}: no wheelhouse footprint"));
        assert!(cab.iter().any(|q| close(q.0, wx0)) && cab.iter().any(|q| close(q.0, wx1)));
        // container units, `per_row` abreast across the beam, on EQUIP
        let units: Vec<Vec<(f64, f64)>> = of_layer(ent, "EQUIP", "polyline")
            .iter()
            .map(|e| points(e))
            .collect();
        let labels = a["container_labels"].as_array().unwrap().len();
        assert_eq!(units.len(), labels, "{file}");
        let (unit_w, gap) = (
            f(&a["container_unit_width_m"]),
            f(&a["container_side_gap_m"]),
        );
        let y0 = f(&a["container_total_width_m"]) / 2.0 - gap;
        let per_row = a["container_per_row"].as_u64().unwrap() as usize;
        for (i, u) in units.iter().enumerate() {
            let (row, col) = (i / per_row, i % per_row);
            let x0 = f(&a["container_x0_m"])
                + row as f64 * (f(&a["container_row_length_m"]) + f(&a["container_row_gap_m"]));
            let yc = y0 - col as f64 * (unit_w + gap) - unit_w / 2.0;
            assert!(
                u.iter()
                    .any(|p| close(p.0, x0) && close(p.1, yc - unit_w / 2.0)),
                "{file}: unit {i}"
            );
            assert!(
                u.iter()
                    .any(|p| close(p.0, x0 + f(&a["container_row_length_m"]))
                        && close(p.1, yc + unit_w / 2.0)),
                "{file}: unit {i}"
            );
        }
        // thrusters: one circle at each offset, in the steering flat
        let offsets: Vec<f64> = a["thruster_offsets_m"]
            .as_array()
            .unwrap()
            .iter()
            .map(f)
            .collect();
        let room = f(&a["thruster_room_length_m"]);
        let drawn: Vec<(f64, f64)> = of_layer(ent, "EQUIP", "circle")
            .iter()
            .map(|e| (f(&e["centre_m"][0]), f(&e["centre_m"][1])))
            .collect();
        assert_eq!(drawn.len(), offsets.len(), "{file}");
        for off in offsets {
            assert!(
                drawn
                    .iter()
                    .any(|p| close(p.0, room / 2.0) && close(p.1, off)),
                "{file}"
            );
        }
        assert!(has_leader(ent, "WHEELHOUSE"));
    }
}

#[test]
fn a_pushed_barge_has_no_aft_block_and_a_null_arrangement_draws_nothing() {
    for file in [
        "gas_igc_2g_self_propelled.vdesign",
        "dry_bulk_box_hold.vdesign",
    ] {
        let pushed = results_of(file, false);
        assert!(pushed["drawing"]["aft_arrangement"].is_null());
        let mut nulled = results_of(file, true);
        nulled["drawing"]["aft_arrangement"] = Value::Null;
        for r in [&pushed, &nulled] {
            let v = views(r);
            for (name, view) in [("elevation", &v.side_elevation), ("plan", &v.plan)] {
                assert!(
                    of_layer(view, "EQUIP", "polyline").is_empty(),
                    "{file} {name}"
                );
                assert!(
                    of_layer(view, "EQUIP", "circle").is_empty(),
                    "{file} {name}"
                );
                let t = texts(view);
                assert!(
                    !t.iter().any(|x| x == "WHEELHOUSE"
                        || x.starts_with("THRUSTERS")
                        || x.starts_with("CONTAINERS")),
                    "{file} {name}"
                );
            }
        }
    }
}

// ---- #198: a design that fails its checks says so ------------------------------------------------

fn failed_names(r: &Value) -> Vec<String> {
    r["checks"]
        .as_array()
        .unwrap()
        .iter()
        // A failed warning is not a failure (issue #418).
        .filter(|c| c["ok"] == false && c["severity"] != "warning")
        .map(|c| c["name"].as_str().unwrap().to_string())
        .collect()
}

fn notes_of(document: &Value) -> Vec<String> {
    document["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap().to_string())
        .collect()
}

fn accepted(document: &Value) {
    match ovcd_draw::run_value(document, &ovcd_draw::NoFiles) {
        ovcd_draw::RunResult::Ok(_) => {}
        ovcd_draw::RunResult::Errors(e) => panic!("the standard refuses the document {e:?}"),
    }
}

#[test]
fn a_failing_design_carries_a_note_in_each_view_and_still_passes_the_standard() {
    let mut seen = 0;
    // The fixtures, and each one under an air draft limit no arrangement meets: since the search
    // keeps only arrangements that pass every blocking Rule (issue #277), the fixtures pass, so the
    // low limit is the case that fails.
    for (file, low) in GAS_FIXTURES.iter().flat_map(|f| [(f, false), (f, true)]) {
        for self_propelled in [false, true] {
            let r = if low {
                results_of_low_air_draft(file, self_propelled)
            } else {
                results_of(file, self_propelled)
            };
            let failed = failed_names(&r);
            if failed.is_empty() {
                continue;
            }
            seen += 1;
            let v = views(&r);
            for (name, doc) in v.all() {
                let notes = notes_of(doc);
                let fail_notes: Vec<&String> = notes.iter().skip(2).collect();
                assert!(
                    !fail_notes.is_empty(),
                    "{file} {name}: a note for the failed checks"
                );
                assert!(
                    fail_notes[0].starts_with(&format!("DESIGN FAILS {} CHECK", failed.len())),
                    "{file} {name}: the count"
                );
                // one note on one line, in plain capitals
                assert_eq!(fail_notes.len(), 1, "{file} {name}: one failed-checks note");
                let text = fail_notes
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
                assert!(text.is_ascii() && text.chars().all(|c| !c.is_ascii_lowercase()));
                // every name is there, or the note counts the rest ("AND k MORE")
                if text.contains(':') {
                    for n in failed.iter().take(if text.contains(" MORE") {
                        1
                    } else {
                        failed.len()
                    }) {
                        let plain: String = n
                            .chars()
                            .filter_map(|c| match c {
                                '\u{2014}' => Some('-'),
                                '_' => Some(' '),
                                c if c.is_ascii() => Some(c.to_ascii_uppercase()),
                                _ => None,
                            })
                            .collect();
                        assert!(text.contains(plain.trim()), "{file} {name}: {plain}");
                    }
                }
                // the status is still one the standard defines, and the strip holds every note
                assert_eq!(doc["status"], "concept");
                let strip = f(&doc["strips"][0]["size_mm"]);
                assert!(
                    close(strip, (12.0 + 4.5 * notes.len() as f64).max(21.0)),
                    "{file} {name}"
                );
                assert!(notes.iter().all(|n| n.chars().count() <= 120));
                accepted(doc);
            }
        }
    }
    assert!(seen > 0, "no fixture fails a check");
}

#[test]
fn a_passing_design_has_the_usual_notes_and_strip() {
    for file in HOLDS {
        let r = results_of(file, false);
        assert!(failed_names(&r).is_empty(), "{file}");
        for (name, doc) in views(&r).all() {
            assert_eq!(notes_of(doc).len(), 2, "{file} {name}");
            assert!(!notes_of(doc).iter().any(|n| n.starts_with("DESIGN FAILS")));
            assert!(
                close(f(&doc["strips"][0]["size_mm"]), 21.0),
                "{file} {name}"
            );
            accepted(doc);
        }
    }
}

#[test]
fn a_failing_design_with_many_names_names_some_and_counts_the_rest() {
    let mut r = results_of("dry_bulk_box_hold.vdesign", false);
    let checks: Vec<Value> = (0..30)
        .map(|i| serde_json::json!({"name": format!("A VERY LONG CHECK NAME NUMBER {i}"), "ok": false, "message": ""}))
        .collect();
    r["checks"] = Value::Array(checks);
    for (_, doc) in views(&r).all() {
        let notes = notes_of(doc);
        assert_eq!(notes.len(), 3);
        assert!(
            notes[2].starts_with("DESIGN FAILS 30 CHECKS: A VERY LONG CHECK NAME NUMBER 0, "),
            "{}",
            notes[2]
        );
        assert!(notes[2].ends_with(" MORE") && notes[2].chars().count() <= 120);
        accepted(doc);
    }
}
