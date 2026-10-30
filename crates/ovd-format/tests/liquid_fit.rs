//! `fit` for liquid bulk (issue #340, spec 0006 section 5): the double hull and the cofferdam from the
//! governing `sizes` Rules, a centreline bulkhead from the 0.70 B rule, and the smallest count of tanks in
//! line that meets the tank volume and length limits.
//!
//! Label: sourced - ADN 2025 (ECE/TRANS/351) 9.3.2.11.1 (a) and (d), 9.3.2.11.3 (a), 9.3.2.11.7 and
//! MARPOL Annex I as revised by MEPC.117(52), regulation 19.6. Each expected layout below is worked out by
//! hand from the text, not read back from the Rule Sets. The guarantees (never feasible with a failing
//! blocking Rule; a Rule added never raises the cargo) are properties.

use ovd_core::liquid_bulk::{LiquidLayout, PumpArrangement};
use ovd_format::rule_set::Library;
use ovd_format::trace::{
    LiquidFit, LiquidFitCase, LiquidTraceCase, LiquidTyped, liquid_fit, liquid_trace,
};
use serde_json::{Value, json};

const ADN: &[(&str, &str)] = &[("adn-tank", "1.0.0")];
const MARPOL: &[(&str, &str)] = &[("marpol-i", "1.0.0")];

fn case(loa: f64, beam: f64, depth: f64, service: &str, vessel_type: &str) -> LiquidFitCase {
    LiquidFitCase {
        case: LiquidTraceCase {
            loa_m: loa,
            beam_m: beam,
            depth_m: depth,
            zone_aft_m: 3.0,
            zone_fwd_m: loa - 6.5,
            // the layout `fit` starts from: every input it may choose is at a value no Rule accepts
            layout: LiquidLayout {
                double_side_m: 0.3,
                double_bottom_m: 0.3,
                tanks_in_line: 1,
                centreline_bulkhead: false,
                cofferdam_m: 0.2,
                pump_arrangement: PumpArrangement::Deepwell,
                slop_share: 0.0,
                fill_ratio: 0.98,
                heating_kw_per_m3: 0.0,
                lightship_k_t_per_m3: 0.1141,
            },
            cargo_density_t_m3: 0.87,
            deadweight_t: 1500.0,
            service_area: service.to_string(),
            adn_vessel_type: vessel_type.to_string(),
        },
        typed: LiquidTyped::default(),
    }
}

fn fit(c: &LiquidFitCase, sets: &[(&str, &str)]) -> LiquidFit {
    liquid_fit(c, &Library::built_in(), sets).expect("a layout fits the zone")
}

/// The Rule Trace rows of a layout, by the harness entry point.
fn rows(c: &LiquidTraceCase, layout: &LiquidLayout, sets: &[(&str, &str)]) -> Vec<Value> {
    let mut c = c.clone();
    c.layout = *layout;
    liquid_trace(&c, &Library::built_in(), sets).unwrap()["rows"]
        .as_array()
        .unwrap()
        .clone()
}

fn not_met(rows: &[Value]) -> Vec<String> {
    rows.iter()
        .filter(|r| r["outcome"] == "fail" || r["outcome"] == "warn")
        .map(|r| r["rule"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn a_river_tank_barge_gets_the_adn_double_hull_cofferdam_and_the_fewest_tanks_that_fit_the_length_limit()
 {
    // ADN 9.3.2.11.7: side wall to the bulkhead at least 1.00 m; double bottom at least 0.70 m (and never
    // under 0.60 m). 9.3.2.11.3 (a): cofferdam at least 0.60 m. 9.3.2.11.1 (d): a tank is at most 0.20 L
    // long on a vessel over 50 m: 0.20 x 76.5 = 15.3 m.
    let c = case(76.5, 11.4, 3.5, "river-only", "C");
    let f = fit(&c, ADN);
    assert!(f.feasible && f.sized, "{f:?}");
    let l = &f.layout;
    assert_eq!(
        (l.double_side_m, l.double_bottom_m, l.cofferdam_m),
        (1.00, 0.70, 0.60)
    );
    // The block is the zone (70.0 - 3.0 = 67.0 m) less two cofferdams: 65.8 m. It needs 65.8 / 15.3 =
    // 4.3 tanks, so 5.
    let block = (c.case.zone_fwd_m - c.case.zone_aft_m) - 2.0 * 0.6;
    assert_eq!(l.tanks_in_line, (block / (0.20 * 76.5)).ceil() as u32);
    assert_eq!(l.tanks_in_line, 5);
    // The inner breadth is 11.4 - 2 x 1.00 = 9.4 m, over 0.70 B = 7.98 m: a centreline bulkhead.
    assert!(l.centreline_bulkhead);
    // The answer meets every rule of the set, as the harness entry point reads it.
    assert_eq!(not_met(&rows(&c.case, l, ADN)), Vec::<String>::new());
}

#[test]
fn one_tank_fewer_than_the_answer_breaks_a_limit() {
    let c = case(76.5, 11.4, 3.5, "river-only", "C");
    let f = fit(&c, ADN);
    let mut fewer = f.layout;
    fewer.tanks_in_line -= 1;
    let broken = not_met(&rows(&c.case, &fewer, ADN));
    assert!(
        broken
            .iter()
            .any(|r| r.contains("cargo-tank-length") || r.contains("cargo-tank-capacity")),
        "{broken:?}"
    );
}

#[test]
fn a_wide_tank_needs_the_volume_limit_too() {
    // 9.3.2.11.1 (a): up to 600 m3 of L x B x H, 0.3 x L x B x H; 600 to 3,750, 180 + (LBH - 600) x 0.0635;
    // above, 380 m3. A 100 m x 16.8 m x 3.6 m barge has L x B x H = 6,048: the limit is 380 m3 and the length
    // limit is 20 m. The answer must keep every tank under both, and be the smallest that does.
    let c = case(100.0, 16.8, 3.6, "river-only", "C");
    let f = fit(&c, ADN);
    assert!(f.feasible && f.sized, "{f:?}");
    let l = &f.layout;
    let block = (c.case.zone_fwd_m - c.case.zone_aft_m) - 2.0 * l.cofferdam_m;
    let n = f64::from(l.tanks_in_line);
    let height = 3.6 - l.double_bottom_m;
    let inner = 16.8 - 2.0 * l.double_side_m;
    let breadth = if l.centreline_bulkhead {
        inner / 2.0
    } else {
        inner
    };
    let volume = block / n * breadth * height;
    assert!(
        volume <= 380.0 + 1e-9 && block / n <= 20.0 + 1e-9,
        "{volume} {}",
        block / n
    );
    // one tank fewer breaks one of the two
    let broken = block / (n - 1.0);
    let volume_fewer = broken * breadth * height;
    assert!(
        volume_fewer > 380.0 || broken > 20.0,
        "{volume_fewer} {broken}"
    );
}

#[test]
fn marpol_gives_the_double_hull_of_a_coastal_tanker_under_5000_t() {
    // Reg. 19.6.1: double bottom h = B / 15, at least 0.76 m. 19.6.2: wing tank w = 0.4 + 2.4 DW / 20,000,
    // at least 0.76 m. B = 14.0 m: h = 0.9333, rounded up to the centimetre, 0.94 m. DW = 1,500 t:
    // w = 0.4 + 0.18 = 0.58, so the 0.76 m floor governs.
    let c = case(80.0, 14.0, 4.5, "coastal-and-river", "C");
    let f = fit(&c, MARPOL);
    assert!(f.feasible && f.sized, "{f:?}");
    assert_eq!(f.layout.double_side_m, 0.76);
    assert_eq!(f.layout.double_bottom_m, 0.94);
    // with the ADN set as well, the greater limit of each Measure governs: 1.00 m and 0.94 m
    let both = fit(&c, &[("adn-tank", "1.0.0"), ("marpol-i", "1.0.0")]);
    assert_eq!(both.layout.double_side_m, 1.00);
    assert_eq!(both.layout.double_bottom_m, 0.94);
}

#[test]
fn a_greater_deadweight_widens_the_wing_tank() {
    // 19.6.2 at DW = 4,900 t: 0.4 + 2.4 x 4900 / 20000 = 0.988 m, rounded up to 0.99 m.
    let mut c = case(100.0, 14.0, 4.5, "coastal-and-river", "C");
    c.case.deadweight_t = 4900.0;
    let f = fit(&c, MARPOL);
    assert_eq!(f.layout.double_side_m, 0.99);
}

#[test]
fn a_centreline_bulkhead_is_fitted_when_a_tank_would_be_wider_than_70_percent_of_the_beam() {
    let mut c = case(76.5, 11.4, 3.5, "river-only", "C");
    // typed double side 3.0 m: the inner breadth is 5.4 m, under 0.70 x 11.4 = 7.98 m
    c.case.layout.double_side_m = 3.0;
    c.typed.double_side_m = true;
    let f = fit(&c, ADN);
    assert_eq!(f.layout.double_side_m, 3.0);
    assert!(!f.layout.centreline_bulkhead);
    // a typed bulkhead is kept even where the rule would not fit one
    c.case.layout.centreline_bulkhead = true;
    c.typed.centreline_bulkhead = true;
    assert!(fit(&c, ADN).layout.centreline_bulkhead);
}

#[test]
fn a_typed_input_is_kept_and_the_rule_it_breaks_is_named() {
    let mut c = case(76.5, 11.4, 3.5, "river-only", "C");
    c.case.layout.double_side_m = 0.6;
    c.typed.double_side_m = true;
    let f = fit(&c, ADN);
    assert_eq!(f.layout.double_side_m, 0.6);
    // A `warn` Rule does not make the layout infeasible, but the fit is not sized and says which Rule.
    assert!(f.feasible && !f.sized);
    assert!(
        f.sizing_warnings
            .iter()
            .any(|r| r.rule == "adn/9.3.2.11.7/wing-width-min"
                && r.measure == "hull.double_side_width"
                && r.limit == Some(1.0)),
        "{:?}",
        f.sizing_warnings
    );
    // a typed count of tanks is kept too
    let mut c = case(76.5, 11.4, 3.5, "river-only", "C");
    c.case.layout.tanks_in_line = 2;
    c.typed.tanks_in_line = true;
    let f = fit(&c, ADN);
    assert_eq!(f.layout.tanks_in_line, 2);
    assert_eq!(f.candidates_tried, 1);
    assert!(!f.sized);
}

#[test]
fn a_zone_too_short_for_its_cofferdams_has_no_answer_and_says_so() {
    let mut c = case(76.5, 11.4, 3.5, "river-only", "C");
    c.case.zone_fwd_m = c.case.zone_aft_m + 1.0;
    let e = liquid_fit(&c, &Library::built_in(), ADN).unwrap_err();
    assert!(e.contains("zone is too short"), "{e}");
}

/// The adn-tank sizing Rules as a set of its own with every Rule blocking, so a layout that breaks one
/// fails. Its id is its own, so it sits beside the built-in sets.
fn blocking_set(id: &str, keep: &[&str], tighten_length_to: Option<f64>) -> String {
    let mut set: Value =
        serde_json::from_str(include_str!("../rules/adn-tank/1.0.0.json")).unwrap();
    set["id"] = json!(id);
    set["family"] = json!(id);
    let rules = set["rules"].as_array().unwrap().clone();
    let mut out = Vec::new();
    for mut r in rules {
        let rid = r["id"].as_str().unwrap().to_string();
        if !keep.iter().any(|k| rid.contains(k)) || !rid.contains("9.3.2") {
            continue;
        }
        r["consequence"] = json!("block");
        if let (Some(cap), true) = (tighten_length_to, rid.contains("cargo-tank-length")) {
            r["kind"] = json!("limit");
            r["limit"] = json!({"value": cap, "unit": "m"});
        }
        // the set's own ids: a Rule id is the set's, so two sets may hold a Rule of one text
        r["id"] = json!(format!("{id}/{}", rid.strip_prefix("adn/").unwrap()));
        out.push(r);
    }
    set["rules"] = Value::Array(out);
    set["known_conflicts"] = json!([]);
    set.to_string()
}

fn library_with(texts: &[String]) -> Library {
    let mut lib = Library::built_in();
    for t in texts {
        lib.add_text(t).expect("the test set loads");
    }
    lib
}

#[test]
fn fit_never_returns_a_feasible_layout_that_a_blocking_rule_fails() {
    // Guarantee 1 of ADR 0014, tried over a grid of hulls and over three blocking sets (the sizing Rules as
    // blocking; with a tightened length cap; with a cap that no count can meet). A feasible answer is
    // checked again by the harness entry point; an answer that is not feasible still names a failing row.
    let sets = [
        blocking_set(
            "blk-a",
            &["cargo-tank", "wing-width", "double-bottom", "cofferdam"],
            None,
        ),
        blocking_set(
            "blk-b",
            &["cargo-tank", "wing-width", "double-bottom", "cofferdam"],
            Some(6.0),
        ),
        blocking_set("blk-c", &["cargo-tank-length"], Some(0.5)),
    ];
    let lib = library_with(&sets);
    let mut feasible = 0;
    let mut infeasible = 0;
    for set in ["blk-a", "blk-b", "blk-c"] {
        for (loa, beam, depth) in [
            (60.0, 10.0, 3.0),
            (76.5, 11.4, 3.5),
            (90.0, 14.0, 4.5),
            (110.0, 16.8, 4.0),
        ] {
            for ty in ["C", "N_open"] {
                let c = case(loa, beam, depth, "river-only", ty);
                let chosen = [(set, "1.0.0")];
                let Ok(f) = liquid_fit(&c, &lib, &chosen) else {
                    continue;
                };
                let mut again = c.case.clone();
                again.layout = f.layout;
                let trace = liquid_trace(&again, &lib, &chosen).unwrap();
                let fails = trace["rows"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|r| r["outcome"] == "fail")
                    .count();
                if f.feasible {
                    feasible += 1;
                    assert_eq!(
                        fails, 0,
                        "{set} {loa}x{beam}: a feasible answer fails a row"
                    );
                    assert!(f.fails.is_empty());
                } else {
                    infeasible += 1;
                    assert!(
                        fails > 0 && !f.fails.is_empty(),
                        "{set} {loa}x{beam}: not feasible, no failing row"
                    );
                    assert_eq!(fails, f.fails.len());
                }
            }
        }
    }
    // The grid has both kinds of answer, so neither branch above is empty.
    assert!(
        feasible >= 4 && infeasible >= 4,
        "{feasible} feasible, {infeasible} not"
    );
}

#[test]
fn adding_a_rule_never_raises_the_cargo_or_turns_infeasible_into_feasible() {
    // Guarantee 2: for the same hull, a selection with one more blocking Rule is never better.
    let loose = blocking_set(
        "blk-loose",
        &["wing-width", "double-bottom", "cofferdam"],
        None,
    );
    let tight = blocking_set(
        "blk-tight",
        &["cargo-tank", "wing-width", "double-bottom", "cofferdam"],
        Some(6.0),
    );
    let lib = library_with(&[loose, tight]);
    for (loa, beam, depth) in [(60.0, 10.0, 3.0), (76.5, 11.4, 3.5), (90.0, 14.0, 4.5)] {
        let c = case(loa, beam, depth, "river-only", "C");
        let a = liquid_fit(&c, &lib, &[("blk-loose", "1.0.0")]).unwrap();
        let b = liquid_fit(&c, &lib, &[("blk-loose", "1.0.0"), ("blk-tight", "1.0.0")]).unwrap();
        if b.feasible {
            assert!(a.feasible, "{loa}x{beam}");
            assert!(
                b.cargo_volume_m3 <= a.cargo_volume_m3 + 1e-9,
                "{loa}x{beam}"
            );
        }
    }
}

#[test]
fn the_answer_is_the_same_through_the_evaluation_entry_point() {
    let args = json!({
        "loa_m": 76.5, "beam_m": 11.4, "depth_m": 3.5, "zone_aft_m": 3.0, "zone_fwd_m": 70.0,
        "double_side_m": 0.3, "double_bottom_m": 0.3, "tanks_in_line": 1, "centreline_bulkhead": false,
        "cofferdam_m": 0.2, "slop_share": 0.0, "deadweight_t": 1500.0,
        "service_area": "river-only", "adn_vessel_type": "C", "select": ["adn-tank@1.0.0"],
    });
    let out =
        ovd_format::eval("gas_stability", &json!({"fn": "liquid_fit", "args": args})).unwrap();
    let ok = &out["ok"];
    assert_eq!(ok["feasible"], true, "{out}");
    assert_eq!(ok["layout"]["tanks_in_line"], 5);
    assert_eq!(ok["layout"]["double_side_m"].as_f64(), Some(1.0));
    // a name that `fit` does not choose is refused
    let mut bad = args.clone();
    bad["typed"] = json!(["slop_share"]);
    assert!(ovd_format::eval("gas_stability", &json!({"fn": "liquid_fit", "args": bad})).is_err());
}
