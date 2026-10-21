//! Seam: the built-in Rule Set `igc-2g` 1.0.0 (spec 0004, issue #271).
//!
//! Label: sourced - IGC Code, resolution MSC.370(93) as amended, 2.3.1.2.3, 2.4.1.1, 2.4.1.2, 2.4.2,
//! 4.23.2.1.2, 4.23.2.1.3 and 4.23.3.1 (MSC.370-93.pdf, annex pp. 20-21, 64 and 66)

use ovcd_core::pressure_vessel::{DEFAULT_WELD_EFFICIENCY, TANK_MATERIAL_DATABASE};
use ovcd_core::protective_location::d_vc_exact;
use ovcd_core::rule_trace::{MeasureValue, formula_value, holds, table_value};
use ovcd_core::rules::{Compare, Limit, RuleSet};
use ovcd_format::rule_set::Library;
use serde_json::Value;

fn set() -> RuleSet {
    Library::built_in()
        .find("igc-2g", "1.0.0")
        .expect("built in")
        .clone()
}

/// d(Vc) from the set's table. The set loads once per test binary.
fn d(vc: f64) -> f64 {
    static SET: std::sync::OnceLock<RuleSet> = std::sync::OnceLock::new();
    let s = SET.get_or_init(set);
    table_value(s.table("igc-d-vc").unwrap(), &MeasureValue::Number(vc)).unwrap()
}

#[test]
fn d_of_vc_at_both_sides_of_each_boundary() {
    // IGC 2.4.1.1, worked by hand: .1 Vc <= 1,000: 0.8 m; .2 1,000 < Vc < 5,000: 0.75 + Vc x 0.2 /
    // 4,000 m; .3 5,000 <= Vc < 30,000: 0.8 + Vc / 25,000 m; .4 Vc >= 30,000: 2 m.
    assert_eq!(d(1000.0), 0.8);
    assert_eq!(d(1000.001), 0.75 + 1000.001 * 0.2 / 4000.0); // 0.80000005
    assert_eq!(d(4999.0), 0.75 + 4999.0 * 0.2 / 4000.0); // 0.99995
    assert_eq!(d(5000.0), 1.0); // 0.8 + 5,000 / 25,000
    assert_eq!(d(29999.0), 0.8 + 29999.0 / 25000.0); // 1.99996
    assert_eq!(d(30000.0), 2.0);
    assert_eq!(d(0.0), 0.8);
    assert_eq!(d(1e6), 2.0);
}

#[test]
fn d_of_vc_is_continuous_and_never_falls() {
    // A typo in a coefficient breaks one or the other.
    for (b, step) in [(1000.0, 1e-9), (5000.0, 1e-9), (30000.0, 1e-9)] {
        assert!((d(b) - d(b + step)).abs() < 1e-6, "jump at {b}");
        assert!((d(b - step) - d(b)).abs() < 1e-6, "jump at {b}");
    }
    let mut last = 0.0;
    for i in 0..4000 {
        let v = d(f64::from(i) * 10.0);
        assert!(v >= last, "d falls at {} m3", i * 10);
        last = v;
    }
}

#[test]
fn the_table_is_the_engines_d_of_vc_bit_for_bit() {
    // The arrangement still uses `d_vc_exact` until `fit` reads the trace (#277); the two must agree.
    for i in 0..40000 {
        let vc = f64::from(i) * 0.8317;
        assert_eq!(d(vc).to_bits(), d_vc_exact(vc).to_bits(), "Vc {vc}");
    }
    for vc in [
        1000.0,
        1000.0f64.next_up(),
        5000.0f64.next_down(),
        5000.0,
        30000.0f64.next_down(),
        30000.0,
    ] {
        assert_eq!(d(vc).to_bits(), d_vc_exact(vc).to_bits(), "Vc {vc}");
    }
}

#[test]
fn the_bottom_distance_is_the_engines_bottom_clearance_bit_for_bit() {
    // The gas arrangement: py_max(py_min(beam / 15.0, 2.0), d_vc) - ins_m.
    let s = set();
    let Some(Limit::Formula {
        shape,
        coefficients,
        ..
    }) = &s.rule("igc/2.4.1.2/bottom-clearance").unwrap().limit
    else {
        panic!("a formula");
    };
    for beam in [6.0, 11.4, 14.8, 22.5, 30.0, 45.0] {
        for vc in [500.0, 1200.0, 4999.0, 12000.0, 40000.0] {
            for ins in [0.0, 0.1, 0.35] {
                let got = formula_value(shape, coefficients, |name| match name {
                    "x" => Ok(beam),
                    "of" => Ok(d(vc)),
                    "less" => Ok(ins),
                    _ => Err(name.to_string()),
                })
                .unwrap();
                let capped = if 2.0 < beam / 15.0 { 2.0 } else { beam / 15.0 };
                let engine = (if d_vc_exact(vc) > capped {
                    d_vc_exact(vc)
                } else {
                    capped
                }) - ins;
                assert_eq!(
                    got.to_bits(),
                    engine.to_bits(),
                    "B {beam} Vc {vc} ins {ins}"
                );
            }
        }
    }
    // By hand: B = 11.4 m, Vc = 1,275.6 m3, 350 mm: B/15 = 0.76 m; d = 0.81378 m; the greater, less
    // 0.35, is 0.46378 m.
    let h = formula_value(shape, coefficients, |n| match n {
        "x" => Ok(11.4),
        "of" => Ok(d(1275.6)),
        _ => Ok(0.35),
    })
    .unwrap();
    assert!((h - 0.46378).abs() < 1e-9, "{h}");
}

#[test]
fn the_wall_tables_are_the_engines_material_data() {
    let s = set();
    let at = |table: &str, material: &str| {
        table_value(
            s.table(table).unwrap(),
            &MeasureValue::Text(material.to_string()),
        )
        .unwrap()
    };
    for (key, m) in TANK_MATERIAL_DATABASE.iter() {
        // 4.23.2.1.2: 5 mm carbon-manganese and nickel steels, 3 mm austenitic steels.
        assert_eq!(at("igc-min-thickness", key), m.min_thickness, "{key}");
        // 4.23.3.1: A 3 (nickel, carbon-manganese) or 3.5 (austenitic); B 1.5.
        assert_eq!(at("igc-factor-a", key), m.factor_a, "{key}");
        assert_eq!(at("igc-factor-b", key), m.factor_b, "{key}");
    }
    assert_eq!(at("igc-min-thickness", "9Ni"), 5.0);
    assert_eq!(at("igc-min-thickness", "304L"), 3.0);
    // 4.23.2.1.3: the factor is 0.95.
    let Some(Limit::Constant { value, .. }) = &s.rule("igc/4.23.2.1.3/weld-factor").unwrap().limit
    else {
        panic!("a constant");
    };
    assert_eq!(*value, DEFAULT_WELD_EFFICIENCY);
}

#[test]
fn the_allowable_stress_rule_at_both_sides_of_the_limit() {
    // 4.23.3.1, by hand for S690QL1 (Rm 770, Re 690 MPa; A 3, B 1.5): Rm/A = 256.67, Re/B = 460,
    // so f is at most 256.67 MPa. The old inland basis, 285 MPa, breaks it (#313).
    let s = set();
    let rule = s.rule("igc/4.23.3.1/allowable-stress").unwrap();
    let Some(Limit::Formula {
        shape,
        coefficients,
        ..
    }) = &rule.limit
    else {
        panic!("a formula");
    };
    let f = formula_value(shape, coefficients, |n| match n {
        "rm" => Ok(770.0),
        "re" => Ok(690.0),
        "a" => Ok(3.0),
        _ => Ok(1.5),
    })
    .unwrap();
    assert_eq!(f, 770.0 / 3.0);
    assert!(holds(Compare::Le, 770.0 / 3.0, f, 0.0));
    assert!(!holds(Compare::Le, 285.0, f, 0.0));
    assert_eq!(rule.finding.as_deref(), Some("#313"));
    assert_eq!(rule.consequence, ovcd_core::rules::Consequence::Warn);
}

#[test]
fn the_set_includes_is2008_a_and_applies_to_type_c_tanks_only() {
    let s = set();
    assert_eq!(s.includes.len(), 1);
    assert_eq!(
        (s.includes[0].id.as_str(), s.includes[0].version.as_str()),
        ("is2008-a", "1.0.0")
    );
    assert_eq!(
        s.applies_when.conditions,
        vec![(
            "cargo_containment".to_string(),
            vec!["type_c_tanks".to_string()]
        )]
    );
    let lib = Library::built_in();
    let sel = lib.select(&[("igc-2g", "1.0.0")]).unwrap();
    let keys: Vec<String> = sel.sets.iter().map(|x| x.key()).collect();
    assert_eq!(keys, ["igc-2g@1.0.0", "is2008-a@1.0.0"]);
}

// --- through real runs --------------------------------------------------------------------------

fn results_of(file: &str) -> Vec<Value> {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(file),
    )
    .unwrap();
    let design = ovcd_format::check(&text, &ovcd_format::Profile::built_in())
        .design
        .unwrap();
    ovcd_format::run_design(&design)
        .into_iter()
        .filter_map(|r| match r.outcome {
            ovcd_format::RunOutcome::Results { results, .. } => Some(results),
            _ => None,
        })
        .collect()
}

#[test]
fn every_gas_option_has_the_igc_rows_of_each_tank_and_the_is_code_rows() {
    for file in [
        "gas_igc_2g_exact.vdesign",
        "gas_igc_2g_self_propelled.vdesign",
    ] {
        for results in results_of(file) {
            let trace = &results["rule_trace"];
            assert_eq!(
                trace["rule_sets"],
                serde_json::json!(["igc-2g@1.0.0", "is2008-a@1.0.0"]),
                "{file}"
            );
            let rows = trace["rows"].as_array().unwrap();
            let n_tanks = results["geometry"]["n_tanks_total"].as_u64().unwrap() as usize;
            for id in [
                "igc/2.4.1.2/side-clearance",
                "igc/2.4.1.2/bottom-clearance",
                "igc/4.23.2.1.2/min-thickness",
                "igc/4.23.2.1.3/weld-factor",
                "igc/4.23.2.1.3/weld-factor-max",
                "igc/4.23.3.1/allowable-stress",
            ] {
                let per_tank: Vec<&Value> = rows.iter().filter(|r| r["rule"] == id).collect();
                assert_eq!(per_tank.len(), n_tanks, "{file} {id}");
                assert!(
                    per_tank.iter().all(|r| r["outcome"] == "pass"),
                    "{file} {id}"
                );
            }
            // The side gap the trace reads is the clearance the arrangement used.
            let side = rows
                .iter()
                .find(|r| r["rule"] == "igc/2.4.1.2/side-clearance")
                .unwrap();
            assert_eq!(
                side["value"], results["geometry"]["side_clearance_m"],
                "{file}"
            );
            assert!(
                rows.iter()
                    .any(|r| r["rule"] == "igc/2.7/damage-survival"
                        && r["outcome"] == "not evaluated")
            );
            assert!(rows.iter().any(|r| r["rule_sets"][0] == "is2008-a@1.0.0"));
        }
    }
}

#[test]
fn a_dry_bulk_option_does_not_get_the_igc_rows() {
    for results in results_of("dry_bulk_box_hold.vdesign") {
        let rows = results["rule_trace"]["rows"].as_array().unwrap();
        assert!(
            rows.iter()
                .all(|r| !r["rule_sets"][0].as_str().unwrap().starts_with("igc-2g"))
        );
    }
}
