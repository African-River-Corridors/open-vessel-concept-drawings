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
    let mut text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(fixture),
    )
    .expect("fixture");
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
