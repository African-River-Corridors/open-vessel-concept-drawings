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
    results_with(formation, tug_included, changes, &[])
}

/// As [`results`], with the Rule Sets `add` added to the Option's stack.
fn results_with(
    formation: Value,
    tug_included: bool,
    changes: &[(&str, Value)],
    add: &[&str],
) -> Value {
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
    let (mut session, _) = DesignSession::open(&f.to_string(), Profile::built_in()).unwrap();
    let name = session.design().options[0].name.clone();
    for set in add {
        session.add_rule_set(&name, set).unwrap();
    }
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

#[test]
fn the_formation_says_it_is_uncalibrated_outside_the_results_hash() {
    // As the lightship does (issue #434): the label is in `calibration`, which is not hashed, so
    // the calibration of part 2 flips it without moving a hash.
    let r = results(convoy(2, 2), true, &[]);
    assert_eq!(r["calibration"]["formation"]["state"], "uncalibrated");
    assert!(r["formation"].get("calibration").is_none());
    let mut relabelled = r.clone();
    relabelled["calibration"]["formation"]["state"] = json!("calibrated");
    assert_eq!(
        ovcd_format::results_hash(&r),
        ovcd_format::results_hash(&relabelled)
    );
    // A pushed convoy without its pusher has no such label.
    let bare = results(convoy(2, 2), false, &[]);
    assert!(bare["calibration"].get("formation").is_none());
}

fn coupling_rows(r: &Value) -> Vec<Value> {
    r["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["rule"] == "es-trin/ESI-III-6.2.1/coupling-force"
                && row["outcome"] != "not applicable"
        })
        .cloned()
        .collect()
}

#[test]
fn the_coupling_force_rule_is_a_note_with_the_formation_value() {
    // ES-TRIN 2025/1 ESI-III-6 2.1, by hand from the block: 270 x P_B x L_S / B_S x 10^-3 kN. The
    // fixture stores its Rule Sets; ES-TRIN is added.
    let results = |formation: Value, tug: bool, more: &[(&str, Value)]| {
        results_with(formation, tug, more, &["es-trin-2025"])
    };
    for formation in [json!({ "kind": "pushed_barge" }), convoy(2, 2)] {
        let r = results(formation.clone(), true, &[]);
        let f = &r["formation"];
        let force = 270.0 * num(f, &["tug", "installed_power_kw"]) * num(f, &["tug", "length_m"])
            / num(f, &["tug", "beam_m"])
            * 1e-3;
        let rows = coupling_rows(&r);
        assert_eq!(rows.len(), 1, "{formation}: {rows:?}");
        assert_eq!(rows[0]["outcome"], "note", "{formation}");
        assert!(
            (rows[0]["value"].as_f64().unwrap() - force).abs() < 1e-9,
            "{formation}: {} {force}",
            rows[0]["value"]
        );
        assert_eq!(rows[0]["unit"], "kN");
        // Without its pusher the Formation has no force, and the Rule does not apply.
        let alone = results(formation.clone(), false, &[]);
        assert!(coupling_rows(&alone).is_empty(), "{formation}");
    }
    // An ATB's linkage is rigid: no coupling force and no row.
    let atb = results(
        json!({ "kind": "atb" }),
        true,
        &[
            ("tug_stern_form", json!("transom")),
            ("tug_propulsor_type", json!("azimuth_ducted")),
            ("formation_resistance_allowance", json!(1.05)),
            ("tug_design_speed_kn", json!(11.0)),
        ],
    );
    assert!(atb["formation"].get("coupling_force_kn").is_none());
    assert!(coupling_rows(&atb).is_empty());
}

fn speed_rows(r: &Value) -> Vec<Value> {
    r["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["rule"] == "es-trin/5.06(1)/speed-13kmh" && row["outcome"] != "not applicable"
        })
        .cloned()
        .collect()
}

#[test]
fn the_es_trin_speed_rule_reads_the_formation_speed_in_deep_water() {
    // ES-TRIN 2025/1 5.06(1): at least 13 km/h through the water. A pushed Formation with its pusher
    // reads the deep water service speed of the Formation (km/h = kn x 1.852).
    for formation in [json!({ "kind": "pushed_barge" }), convoy(2, 2)] {
        let r = results_with(formation.clone(), true, &[], &["es-trin-2025"]);
        let deep = r["formation"]["environments"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["name"] == "Deep water")
            .unwrap_or_else(|| panic!("{formation}: no deep water"));
        let kmh = deep["service_speed_kn"].as_f64().unwrap() * 1.852;
        let rows = speed_rows(&r);
        assert_eq!(rows.len(), 1, "{formation}: {rows:?}");
        assert!(
            (rows[0]["value"].as_f64().unwrap() - kmh).abs() < 1e-9,
            "{formation}"
        );
        assert_eq!(
            rows[0]["outcome"],
            if kmh >= 13.0 { "pass" } else { "fail" }
        );
        // Without its pusher the barge has no speed, and the Rule does not apply.
        let alone = results_with(formation.clone(), false, &[], &["es-trin-2025"]);
        assert!(speed_rows(&alone).is_empty(), "{formation}");
    }
    // Both sides of 13 km/h (7.019 kn): a pusher sized to 7.1 kn passes, one sized to 6.9 kn fails.
    for (knots, outcome) in [(7.1, "pass"), (6.9, "fail")] {
        let r = results_with(
            json!({ "kind": "pushed_barge" }),
            true,
            &[("tug_design_speed_kn", json!(knots))],
            &["es-trin-2025"],
        );
        let rows = speed_rows(&r);
        assert_eq!(rows.len(), 1, "{knots} kn");
        assert_eq!(rows[0]["outcome"], outcome, "{knots} kn: {}", rows[0]);
    }
}
