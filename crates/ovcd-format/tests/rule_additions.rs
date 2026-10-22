//! Seam: the spec 0004 additions for liquid cargo Rules (spec 0006 §5.2, issue #336): the clamp
//! formula shapes, the derived inputs `cargo_class` and `deadweight_band`, one object type per tank
//! role, and Applicability that names only real inputs and values.
//!
//! Label: sourced - MARPOL Annex I as revised by MEPC.117(52), Reg. 19.1, 19.2, 19.3.1, 19.3.2,
//! 19.3.4, 19.6 (MEPC.117-52.pdf, annex pp. 26-29)

use ovcd_core::rule_trace::{
    MeasureInfo, MeasureSource, MeasureValue, ObjectType, Outcome, evaluate, formula_value,
};
use ovcd_core::rules::MeasureRef;
use ovcd_format::rule_set::{Library, load};
use ovcd_format::trace::{deadweight_band, length_band, measure_info};
use serde_json::{Value, json};

fn coeffs(pairs: &[(&str, f64)]) -> Vec<(String, f64)> {
    pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

fn clamp(x: f64, c: &[(&str, f64)]) -> f64 {
    formula_value("clamp", &coeffs(c), |_| Ok(x)).unwrap()
}

#[test]
fn clamp_gives_the_marpol_wing_tank_width_of_19_3_1() {
    // w = 0.5 + DW/20,000 (m), or w = 2.0 m, whichever is the lesser; the minimum value of w is
    // 1.0 m. As data: a = 0.5, b = 1, c = 20,000, lo = 1.0, hi = 2.0.
    let w = |dw: f64| {
        clamp(
            dw,
            &[
                ("a", 0.5),
                ("b", 1.0),
                ("c", 20000.0),
                ("lo", 1.0),
                ("hi", 2.0),
            ],
        )
    };
    assert_eq!(w(5000.0), 1.0); // 0.5 + 0.25 = 0.75, below the minimum
    assert_eq!(w(10000.0), 1.0); // 0.5 + 0.5 = 1.0, at the minimum
    assert_eq!(w(20000.0), 1.5); // 0.5 + 1.0
    assert_eq!(w(30000.0), 2.0); // 0.5 + 1.5 = 2.0, at the cap
    assert_eq!(w(40000.0), 2.0); // 0.5 + 2.0 = 2.5, the lesser is 2.0
}

#[test]
fn clamp_gives_the_marpol_double_bottom_depth_of_19_3_2() {
    // h = B/15 (m) or 2.0 m, whichever is the lesser; the minimum value of h is 1.0 m.
    let h = |b: f64| {
        clamp(
            b,
            &[
                ("a", 0.0),
                ("b", 1.0),
                ("c", 15.0),
                ("lo", 1.0),
                ("hi", 2.0),
            ],
        )
    };
    assert_eq!(h(12.0), 1.0); // 0.8, below the minimum
    assert_eq!(h(24.0), 1.6);
    assert_eq!(h(45.0), 2.0); // 3.0, the lesser is 2.0
}

#[test]
fn at_least_gives_the_marpol_small_tanker_widths_of_19_6() {
    // 19.6.1: h = B/15 (m) with a minimum value of h = 0.76 m.
    let h = |b: f64| {
        formula_value(
            "at-least",
            &coeffs(&[("a", 0.0), ("b", 1.0), ("c", 15.0), ("lo", 0.76)]),
            |_| Ok(b),
        )
        .unwrap()
    };
    assert_eq!(h(9.0), 0.76); // 0.6
    assert_eq!(h(15.0), 1.0);
    // 19.6.2: w = 0.4 + 2.4 DW/20,000 (m) with a minimum value of w = 0.76 m.
    let w = |dw: f64| {
        formula_value(
            "at-least",
            &coeffs(&[("a", 0.4), ("b", 2.4), ("c", 20000.0), ("lo", 0.76)]),
            |_| Ok(dw),
        )
        .unwrap()
    };
    assert_eq!(w(1000.0), 0.76); // 0.4 + 0.12 = 0.52
    assert_eq!(w(4000.0), 0.4 + 4000.0 * 2.4 / 20000.0); // 0.88, in the text's order
}

#[test]
fn a_formula_shape_needs_its_coefficients() {
    let e = formula_value("clamp", &coeffs(&[("a", 0.5)]), |_| Ok(1.0)).unwrap_err();
    assert!(e.contains("needs coefficient"), "{e}");
    assert!(formula_value("nope", &[], |_| Ok(1.0)).is_err());
}

#[test]
fn the_deadweight_bands_split_where_marpol_reg_19_does() {
    // "600 tonnes deadweight and above" (19.1), "5,000 tonnes deadweight and above" (19.2),
    // "less than 5,000 tonnes deadweight" (19.6), "20,000 tonnes deadweight and above" (19.3.4).
    assert_eq!(deadweight_band(0.0), "under-600-t");
    assert_eq!(deadweight_band(599.999), "under-600-t");
    assert_eq!(deadweight_band(600.0), "600-to-5000-t");
    assert_eq!(deadweight_band(4999.999), "600-to-5000-t");
    assert_eq!(deadweight_band(5000.0), "5000-to-20000-t");
    assert_eq!(deadweight_band(19999.999), "5000-to-20000-t");
    assert_eq!(deadweight_band(20000.0), "20000-t-and-above");
}

#[test]
fn the_length_bands_split_where_solas_v_22_and_es_trin_28_do() {
    // SOLAS V/22.1: "ships of not less than 45 m"; ES-TRIN 28.02: "craft whose length L exceeds 110 m".
    assert_eq!(length_band(0.0), "under-45-m");
    assert_eq!(length_band(44.999), "under-45-m");
    assert_eq!(length_band(45.0), "45-to-110-m");
    assert_eq!(length_band(110.0), "45-to-110-m");
    assert_eq!(length_band(110.001), "over-110-m");
}

#[test]
fn every_tank_role_is_its_own_object_type() {
    let keys: Vec<&str> = [
        ObjectType::Tank,
        ObjectType::CargoTank,
        ObjectType::BallastTank,
        ObjectType::SlopTank,
    ]
    .iter()
    .map(|o| o.key())
    .collect();
    assert_eq!(keys, ["tank", "cargo-tank", "ballast-tank", "slop-tank"]);
}

// --- Applicability and the loader ---------------------------------------------------------------

fn rule_with(applies_when: Value, limit: Value) -> Value {
    json!({
        "id": "t/x/r", "title": "t", "origin": "physical", "source": {},
        "codified": {"by": "agent", "date": "2026-10-21", "review": "#336"},
        "verified": false, "applies_when": applies_when, "kind": "formula",
        "measure": {"id": "input.moulded_depth_m"}, "compare": "ge", "limit": limit,
        "consequence": "warn", "evaluated": true, "since": "1.0.0",
    })
}

fn set_with(set_applies: Value, rule: Value) -> String {
    json!({
        "id": "t", "family": "t", "title": "t", "version": "1.0.0", "edition": {"document": "d"},
        "sources": [], "status": "draft", "applies_when": set_applies, "rules": [rule],
    })
    .to_string()
}

fn clamp_limit(x: &str) -> Value {
    json!({"shape": "clamp", "x": x, "a": 0.5, "b": 1, "c": 20000, "lo": 1.0, "hi": 2.0, "unit": "m"})
}

#[test]
fn a_clamp_rule_loads_with_an_operand_in_any_unit() {
    for x in ["input.length_overall_m", "input.fill_ratio"] {
        load(&set_with(json!({}), rule_with(json!({}), clamp_limit(x)))).unwrap();
    }
    let e = load(&set_with(
        json!({}),
        rule_with(json!({}), clamp_limit("input.cargo")),
    ))
    .unwrap_err();
    assert!(e.contains("where `a number` is needed"), "{e}");
    let mut missing = clamp_limit("input.fill_ratio");
    missing.as_object_mut().unwrap().remove("hi");
    let e = load(&set_with(json!({}), rule_with(json!({}), missing))).unwrap_err();
    assert!(
        e.contains("takes the operands [x] and the coefficients [a, b, c, lo, hi]"),
        "{e}"
    );
}

#[test]
fn applicability_names_only_real_inputs_and_values() {
    let ok = |set: Value, rule: Value| {
        load(&set_with(
            set,
            rule_with(rule, clamp_limit("input.fill_ratio")),
        ))
    };
    ok(json!({"cargo_containment": ["type_c_tanks"]}), json!({})).unwrap();
    ok(json!({"cargo_class": ["liquid_bulk"], "deadweight_band": ["under-600-t", "600-to-5000-t"]}), json!({"service_area": ["coastal-and-river"]})).unwrap();
    ok(json!({}), json!({"cargo": ["bauxite"]})).unwrap();
    for (set, rule, expect) in [
        (
            json!({"cargo_containment": ["type-c"]}),
            json!({}),
            "has `type-c`; its values are",
        ),
        (
            json!({"containment": ["type_c_tanks"]}),
            json!({}),
            "names `containment`, which is not an input",
        ),
        (json!({}), json!({"cargo_class": ["oil"]}), "has `oil`"),
        (
            json!({}),
            json!({"deadweight_band": ["5000-t-and-above"]}),
            "has `5000-t-and-above`",
        ),
        // A number input is not an enumerated one: no threshold in an Applicability.
        (
            json!({}),
            json!({"length_overall_m": ["24"]}),
            "is not an input it can name",
        ),
    ] {
        let e = ok(set, rule).unwrap_err();
        assert!(e.contains(expect), "{e}");
    }
}

// --- derived inputs in a real run ---------------------------------------------------------------

fn fixture(name: &str) -> ovcd_format::Design {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name),
    )
    .unwrap();
    ovcd_format::check(&text, &ovcd_format::Profile::built_in())
        .design
        .unwrap()
}

/// A set of one `input.fill_ratio` note that applies when `applies_when` holds.
fn note_set(id: &str, applies_when: Value) -> String {
    json!({
        "id": id, "family": id, "title": id, "version": "1.0.0", "edition": {"document": "d"},
        "sources": [], "status": "draft", "applies_when": applies_when,
        "rules": [{
            "id": format!("{id}/x/fill"), "title": "fill", "origin": "physical", "source": {},
            "codified": {"by": "agent", "date": "2026-10-21", "review": "#336"}, "verified": false,
            "kind": "note", "measure": {"id": "input.fill_ratio"}, "consequence": "note",
            "evaluated": true, "since": "1.0.0",
        }],
    })
    .to_string()
}

#[test]
fn cargo_class_and_deadweight_band_are_worked_out_for_each_option() {
    let mut lib = Library::built_in();
    let sets = [
        ("gas", json!({"cargo_class": ["gas"]})),
        ("dry", json!({"cargo_class": ["dry_bulk"]})),
        ("liquid", json!({"cargo_class": ["liquid_bulk"]})),
        ("small", json!({"deadweight_band": ["600-to-5000-t"]})),
    ];
    let mut extra = Vec::new();
    for (id, a) in &sets {
        lib.add_text(&note_set(id, a.clone())).unwrap();
        extra.push((id.to_string(), "1.0.0".to_string()));
    }
    for (file, applies) in [
        ("gas_igc_2g_exact.vdesign", ["gas@1.0.0", "small@1.0.0"]),
        ("dry_bulk_box_hold.vdesign", ["dry@1.0.0", "small@1.0.0"]),
    ] {
        let run = &ovcd_format::run_design_with(&fixture(file), &lib, &extra)[0];
        let ovcd_format::RunOutcome::Results { results, .. } = &run.outcome else {
            panic!("{file}: not computed");
        };
        let dw = results["results"]["deadweight_t"].as_f64().unwrap();
        assert!((600.0..5000.0).contains(&dw), "{file}: {dw} t");
        for row in results["rule_trace"]["rows"].as_array().unwrap() {
            let set = row["rule_sets"][0].as_str().unwrap();
            if ["is2008-a@", "igc-2g@", "lr-inland-gas@", "ovcd@"]
                .iter()
                .any(|p| set.starts_with(p))
            {
                continue; // the default stack, not a set of this test
            }
            let applies_here = applies.contains(&set);
            assert_eq!(
                row["outcome"] != "not applicable",
                applies_here,
                "{file} {set}"
            );
        }
    }
}

/// A stand-in liquid bulk Option, for the gas-set Applicability (spec 0006 §5.2).
struct Crude;

impl MeasureSource for Crude {
    fn def(&self, id: &str) -> Option<MeasureInfo> {
        measure_info(id)
    }
    fn objects(&self, object: &str) -> Vec<String> {
        match ObjectType::from_key(object).expect("an OVCD object type") {
            ObjectType::Option => vec!["Crude".into()],
            _ => Vec::new(),
        }
    }
    fn value(&self, _: &MeasureRef, _: usize) -> Result<MeasureValue, String> {
        Ok(MeasureValue::Number(0.98))
    }
    fn input(&self, name: &str) -> Option<String> {
        match name {
            "cargo_containment" => Some("integral_tanks".into()),
            "cargo_class" => Some("liquid_bulk".into()),
            _ => None,
        }
    }
}

#[test]
fn a_set_keyed_to_type_c_tanks_gives_a_crude_option_only_a_not_applicable_row() {
    let set = load(&note_set(
        "gasset",
        json!({"cargo_containment": ["type_c_tanks"]}),
    ))
    .unwrap()
    .set;
    let t = evaluate(&[&set], &Crude);
    assert_eq!(t.rows.len(), 1);
    assert_eq!(
        (t.rows[0].rule.clone(), t.rows[0].outcome),
        (None, Outcome::NotApplicable)
    );
}
