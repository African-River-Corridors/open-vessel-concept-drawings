//! The pusher of a pushed barge or convoy (issue #359, part 1): the Equivalent Hull, the sizing loop,
//! the power and speed in each environment, the coupling force. The barge's own results do not move.
//!
//! Label: property

use ovcd_format::{DesignSession, Profile, RunOutcome};
use serde_json::{Value, json};

fn raw(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The results of the convoy fixture as `kind` with `tug_included`, and `changes` typed.
fn results(formation: Value, tug_included: bool, changes: &[(&str, Value)]) -> Value {
    let mut f: Value = serde_json::from_str(&raw("pushed_convoy")).unwrap();
    let inputs = &mut f["options"][0]["inputs"];
    inputs["formation"] = json!({ "value": formation, "source": "typed" });
    inputs["tug_included"] = json!({ "value": tug_included, "source": "typed" });
    for (name, value) in changes {
        inputs[*name] = json!({ "value": value, "source": "typed" });
    }
    f["options"][0]
        .as_object_mut()
        .unwrap()
        .remove("results_hash");
    let (session, _) = DesignSession::open(&f.to_string(), Profile::built_in()).unwrap();
    match session.evaluate().options[0].outcome.as_ref().unwrap() {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

fn convoy(abreast: u64, in_line: u64) -> Value {
    json!({ "kind": "pushed_convoy", "abreast": abreast, "in_line": in_line })
}

fn num(v: &Value, path: &[&str]) -> f64 {
    let mut cur = v;
    for key in path {
        cur = &cur[*key];
    }
    cur.as_f64().unwrap_or_else(|| panic!("{path:?} in {cur}"))
}

#[test]
fn with_the_tug_left_out_the_result_is_the_barges_bit_for_bit() {
    // `tug_included` false: no formation block, and the barge's own results are what they were.
    let without = results(convoy(2, 2), false, &[]);
    let with = results(convoy(2, 2), true, &[]);
    assert!(without.get("formation").is_none());
    assert!(with.get("formation").is_some());
    for key in [
        "results",
        "dims",
        "geometry",
        "checks",
        "arrangement",
        "curves",
    ] {
        assert_eq!(without[key], with[key], "{key}");
    }
    // A pushed barge too.
    let one = results(json!({ "kind": "pushed_barge" }), false, &[]);
    assert!(one.get("formation").is_none());
    let one_with = results(json!({ "kind": "pushed_barge" }), true, &[]);
    assert_eq!(one["results"], one_with["results"]);
    assert!(
        one_with["formation"]["tug"]["installed_power_kw"]
            .as_f64()
            .unwrap()
            > 0.0
    );
}

#[test]
fn the_sizing_loop_converges_in_at_most_five_passes_for_every_convoy_size() {
    for (abreast, in_line) in [(1, 1), (1, 2), (2, 2), (2, 3), (3, 3)] {
        let r = results(convoy(abreast, in_line), true, &[]);
        let s = &r["formation"]["sizing"];
        assert_eq!(s["converged"], true, "{abreast}x{in_line}: {s}");
        assert!(
            s["passes"].as_u64().unwrap() <= 5,
            "{abreast}x{in_line}: {s}"
        );
    }
}

#[test]
fn the_formation_holds_its_equivalent_hull_and_the_pusher_ratios() {
    let r = results(convoy(2, 3), true, &[]);
    let f = &r["formation"];
    let tug_l = num(f, &["tug", "length_m"]);
    // Pushed: L = barges in line x barge length + pusher length; B = barges abreast x barge beam.
    assert_eq!(
        num(f, &["equivalent_hull", "length_m"]),
        3.0 * num(&r, &["dims", "loa_m"]) + tug_l
    );
    assert_eq!(
        num(f, &["equivalent_hull", "beam_m"]),
        2.0 * num(&r, &["dims", "beam_m"])
    );
    assert_eq!(
        num(f, &["equivalent_hull", "draught_m"]),
        num(&r, &["results", "draught_m"])
    );
    // The size of the pusher follows its power by the ratios (within the last pass's 1 % move).
    let p = num(f, &["tug", "installed_power_kw"]);
    let want = 19.5 * libm::pow(p / 1400.0, 0.67);
    assert!((tug_l - want).abs() / want < 0.02, "{tug_l} vs {want}");
    // The coupling force is the ES-TRIN formula.
    let force = 270.0 * p * tug_l / num(f, &["tug", "beam_m"]) * 1e-3;
    assert!((num(f, &["coupling_force_kn"]) - force).abs() < 1e-9);
    assert_eq!(f["method_status"], "unverified");
    assert!(f["notes"].as_array().unwrap().len() >= 2);
}

#[test]
fn a_bigger_convoy_needs_more_power_and_the_service_speed_reaches_the_design_speed() {
    let small = results(convoy(1, 2), true, &[]);
    let big = results(convoy(2, 3), true, &[]);
    let p = |r: &Value| num(r, &["formation", "tug", "installed_power_kw"]);
    assert!(p(&big) > p(&small), "{} vs {}", p(&big), p(&small));
    for r in [&small, &big] {
        let design = num(r, &["formation", "design_speed_kn"]);
        for e in r["formation"]["environments"].as_array().unwrap() {
            let v = e["service_speed_kn"].as_f64().unwrap();
            assert!(v >= design - 1e-6, "{e}");
        }
    }
    // The resistance allowance scales the power.
    let more = results(
        convoy(2, 3),
        true,
        &[("formation_resistance_allowance", json!(1.5))],
    );
    assert!(p(&more) > p(&big));
}

#[test]
fn the_tug_joins_the_convoy_totals() {
    let r = results(convoy(2, 3), true, &[]);
    let t = &r["convoy"]["totals"];
    let tug = num(&r, &["formation", "tug", "lightship_t"]);
    assert_eq!(num(t, &["tug_lightship_t"]), tug);
    assert_eq!(
        num(t, &["lightship_with_tug_t"]),
        num(t, &["lightship_t"]) + tug
    );
    assert_eq!(
        num(t, &["overall_length_with_tug_m"]),
        num(t, &["overall_length_m"]) + num(&r, &["formation", "tug", "length_m"])
    );
    assert_eq!(r["convoy"]["pusher_included"], true);
    // And its lightship is its hull steel and its machinery.
    let f = &r["formation"]["tug"];
    assert!((tug - num(f, &["hull_steel_t"]) - num(f, &["machinery_t"])).abs() < 1e-9);
}

#[test]
fn a_typed_pusher_power_is_taken_as_typed_and_sizes_the_pusher_by_the_ratios() {
    let r = results(
        convoy(2, 2),
        true,
        &[("tug_installed_power_kw", json!(2800.0))],
    );
    let f = &r["formation"];
    assert_eq!(num(f, &["tug", "installed_power_kw"]), 2800.0);
    assert_eq!(f["tug"]["installed_power_sized"], false);
    let want = 19.5 * libm::pow(2.0, 0.67);
    assert!((num(f, &["tug", "length_m"]) - want).abs() < 1e-9);
    // A typed length is used as typed.
    let typed = results(convoy(2, 2), true, &[("tug_length_m", json!(30.0))]);
    assert_eq!(num(&typed, &["formation", "tug", "length_m"]), 30.0);
}

#[test]
fn the_speed_of_the_formation_reaches_the_rules_that_read_it() {
    // The Rules read `powering.environments`; a Formation with its pusher has its own speeds, read
    // as the same Measure. A convoy of es-trin-2025 gets the speed row evaluated, or not applicable.
    let r = results(convoy(2, 2), true, &[]);
    let speeds = r["formation"]["environments"].as_array().unwrap();
    assert!(!speeds.is_empty());
    assert!(
        speeds
            .iter()
            .all(|e| e["service_speed_kn"].as_f64().unwrap() > 0.0)
    );
    assert!(r["rule_trace"]["rows"].is_array());
}
