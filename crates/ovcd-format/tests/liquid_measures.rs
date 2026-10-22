//! The liquid bulk Measures (issue #337, spec 0006 section 5.3), through a real run: a Rule Set loaded at
//! run time that has one Rule on each Measure gives a row per object, with the number the layout has.
//!
//! Label: property. Every test states a rule: a Measure reads the liquid layout and nothing else, a
//! tank Measure has one row per cargo tank, and a Design with no liquid layout gets no row.

use ovcd_format::rule_set::Library;
use ovcd_format::{Profile, RunOutcome, check, run_design_with};
use serde_json::{Value, json};

/// One Rule of the test set on `measure`, `compare` and `limit` of `unit`. The limit is far from any
/// design's value, so the verdict is the side asked for: the tests read the value, not the verdict.
fn rule(id: &str, measure: &str, unit: &str, compare: &str, limit: f64) -> Value {
    json!({
        "id": id,
        "title": format!("{measure} {compare} {limit}"),
        "origin": "tool-limit",
        "source": {},
        "codified": { "by": "agent", "date": "2026-10-22", "review": "#337" },
        "verified": false,
        "kind": "limit",
        "measure": { "id": measure },
        "compare": compare,
        "limit": { "value": limit, "unit": unit },
        "consequence": "warn",
        "evaluated": true,
        "since": "1.0.0"
    })
}

fn liquid_set() -> String {
    let rules = vec![
        rule(
            "liquid-test/hull/double-side",
            "hull.double_side_width",
            "m",
            "ge",
            0.1,
        ),
        rule(
            "liquid-test/hull/double-bottom",
            "hull.double_bottom_height",
            "m",
            "ge",
            0.1,
        ),
        rule("liquid-test/hull/lbh", "hull.lbh", "m3", "ge", 1.0),
        rule(
            "liquid-test/tank/volume",
            "cargo-tank.volume",
            "m3",
            "le",
            1.0e9,
        ),
        rule(
            "liquid-test/tank/length",
            "cargo-tank.length",
            "m",
            "le",
            1.0e9,
        ),
        rule(
            "liquid-test/option/deadweight",
            "option.deadweight",
            "t",
            "ge",
            1.0,
        ),
        rule(
            "liquid-test/option/capacity",
            "option.cargo_capacity_m3",
            "m3",
            "ge",
            1.0,
        ),
        rule(
            "liquid-test/option/slop",
            "option.slop_capacity_share",
            "-",
            "ge",
            0.0,
        ),
    ];
    json!({
        "id": "liquid-test",
        "family": "liquid-test",
        "title": "Test set on the liquid bulk Measures (not a rule book)",
        "version": "1.0.0",
        "edition": { "document": "OVCD test", "edition": "none" },
        "sources": [],
        "status": "draft",
        "applies_when": {},
        "tables": [],
        "rules": rules,
    })
    .to_string()
}

fn run(name: &str, edits: &[(&str, Value)]) -> Value {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut file: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let inputs = file["options"][0]["inputs"].as_object_mut().unwrap();
    for (k, v) in edits {
        inputs.insert((*k).to_string(), json!({ "value": v, "source": "typed" }));
    }
    let design = check(&file.to_string(), &Profile::built_in())
        .design
        .unwrap();
    let mut lib = Library::built_in();
    lib.add_text(&liquid_set()).unwrap();
    let extra = vec![("liquid-test".to_string(), "1.0.0".to_string())];
    match run_design_with(&design, &lib, &extra).remove(0).outcome {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn rows<'a>(r: &'a Value, prefix: &str) -> Vec<&'a Value> {
    r["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["rule"]
                .as_str()
                .is_some_and(|id| id.starts_with(prefix))
        })
        .collect()
}

fn num(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

#[test]
fn the_hull_measures_are_the_layout_widths_and_adn_l_b_h() {
    let r = run("crude_integral_tanks_pushed", &[]);
    let side = rows(&r, "liquid-test/hull/double-side");
    assert_eq!(side.len(), 1);
    assert_eq!(side[0]["object"], "Hull");
    assert_eq!(num(&side[0]["value"]), 1.0);
    assert_eq!(
        num(&rows(&r, "liquid-test/hull/double-bottom")[0]["value"]),
        0.7
    );
    // ADN 9.3.2.11.1: L is the overall length, B the extreme breadth, H the moulded depth.
    assert_eq!(
        num(&rows(&r, "liquid-test/hull/lbh")[0]["value"]),
        76.5 * 11.4 * 3.5
    );
    assert!(rows(&r, "liquid-test/hull/lbh")[0]["outcome"] == "pass");
}

#[test]
fn a_cargo_tank_measure_has_one_row_per_cargo_tank_with_its_volume_and_length() {
    let r = run("crude_integral_tanks_pushed", &[]);
    let tanks: Vec<&Value> = r["liquid"]["spaces"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["kind"] == "cargo_tank")
        .collect();
    assert_eq!(tanks.len(), 8);
    let volume = rows(&r, "liquid-test/tank/volume");
    let length = rows(&r, "liquid-test/tank/length");
    assert_eq!(volume.len(), 8);
    assert_eq!(length.len(), 8);
    for ((t, v), l) in tanks.iter().zip(&volume).zip(&length) {
        assert_eq!(v["object"], t["name"]);
        assert_eq!(num(&v["value"]), num(&t["volume_m3"]));
        assert_eq!(num(&l["value"]), num(&t["x_fwd_m"]) - num(&t["x_aft_m"]));
    }
}

#[test]
fn the_option_measures_are_the_deadweight_the_cargo_capacity_and_the_slop_share() {
    let r = run(
        "crude_integral_tanks_pushed",
        &[
            ("slop_share", json!(0.03)),
            ("service_area", json!("coastal-and-river")),
        ],
    );
    let one = |id: &str| num(&rows(&r, id)[0]["value"]);
    assert_eq!(
        one("liquid-test/option/deadweight"),
        num(&r["results"]["deadweight_t"])
    );
    assert_eq!(
        one("liquid-test/option/capacity"),
        num(&r["liquid"]["cargo_gross_m3"])
    );
    let (cargo, slop) = (
        num(&r["liquid"]["cargo_gross_m3"]),
        num(&r["liquid"]["slop_gross_m3"]),
    );
    assert!(slop > 0.0);
    assert!((one("liquid-test/option/slop") - slop / (cargo + slop)).abs() < 1e-12);
    assert!((one("liquid-test/option/slop") - 0.03).abs() < 1e-9);
}

#[test]
fn a_centreline_bulkhead_doubles_the_cargo_tank_rows() {
    let with = run("crude_integral_tanks_pushed", &[]);
    let without = run(
        "crude_integral_tanks_pushed",
        &[("centreline_bulkhead", json!(false))],
    );
    assert_eq!(rows(&with, "liquid-test/tank/volume").len(), 8);
    assert_eq!(rows(&without, "liquid-test/tank/volume").len(), 4);
}

#[test]
fn a_dry_bulk_design_gets_no_row_for_a_liquid_measure_but_has_a_deadweight() {
    let r = run("dry_bulk_box_hold", &[]);
    for id in [
        "liquid-test/hull/double-side",
        "liquid-test/hull/lbh",
        "liquid-test/tank/volume",
        "liquid-test/option/capacity",
        "liquid-test/option/slop",
    ] {
        for row in rows(&r, id) {
            assert_eq!(row["outcome"], "not applicable", "{id}: {row}");
        }
    }
    let dw = rows(&r, "liquid-test/option/deadweight");
    assert_eq!(dw.len(), 1);
    assert_eq!(num(&dw[0]["value"]), num(&r["results"]["deadweight_t"]));
}

#[test]
fn the_registry_has_each_measure_with_its_unit_and_object_type() {
    let entries = [
        ("hull.double_side_width", "m", "hull"),
        ("hull.double_bottom_height", "m", "hull"),
        ("hull.lbh", "m3", "hull"),
        ("cargo-tank.volume", "m3", "cargo-tank"),
        ("cargo-tank.length", "m", "cargo-tank"),
        ("cargo-tank.vessel_lbh", "m3", "cargo-tank"),
        ("cargo-tank.vessel_length", "m", "cargo-tank"),
        ("option.adn_vessel_type", "-", "option"),
        ("option.deadweight", "t", "option"),
        ("option.cargo_capacity_m3", "m3", "option"),
        ("option.slop_capacity_share", "-", "option"),
    ];
    for (id, unit, object) in entries {
        let info = ovcd_format::trace::measure_info(id).unwrap_or_else(|| panic!("{id}"));
        assert_eq!(info.unit, unit, "{id}");
        assert_eq!(info.object, object, "{id}");
        assert_eq!(
            info.method_status,
            ovcd_core::rule_trace::MethodStatus::Unverified,
            "{id}"
        );
    }
}
