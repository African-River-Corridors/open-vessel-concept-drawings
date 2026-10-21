//! Seam: the gas arrangement search reads the Rule Trace of each candidate (spec 0004, "How `fit`
//! uses it", issue #277). A Rule Set loaded at run time constrains `fit`; nothing that fails a
//! blocking Rule is ever returned as feasible.
//!
//! Label: property

use ovcd_format::rule_set::Library;
use ovcd_format::{RunOutcome, run_design_with};
use serde_json::{Value, json};

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

/// A set of one blocking Rule: each tank's clear side gap at least `gap` m.
fn side_gap_set(id: &str, gap: f64) -> String {
    json!({
        "id": id, "family": id, "title": id, "version": "1.0.0", "edition": {"document": "test"},
        "sources": [], "status": "draft", "applies_when": {"cargo_containment": ["type_c_tanks"]},
        "rules": [{
            "id": format!("{id}/x/side-gap"), "title": "Side gap at least the test value",
            "origin": "physical", "source": {},
            "codified": {"by": "agent", "date": "2026-10-21", "review": "#277"}, "verified": false,
            "kind": "limit", "measure": {"id": "tank.side_gap"}, "compare": "ge",
            "limit": {"value": gap, "unit": "m"}, "consequence": "block", "evaluated": true, "since": "1.0.0",
        }],
    })
    .to_string()
}

fn run(design: &ovcd_format::Design, sets: &[String]) -> Value {
    let mut lib = Library::built_in();
    let mut extra = Vec::new();
    for s in sets {
        let key = lib.add_text(s).unwrap();
        let (id, v) = key.split_once('@').unwrap();
        extra.push((id.to_string(), v.to_string()));
    }
    match &run_design_with(design, &lib, &extra)[0].outcome {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("not computed: {other:?}"),
    }
}

fn cargo(r: &Value) -> f64 {
    r["results"]["cargo_t"].as_f64().unwrap()
}

#[test]
fn a_tighter_rule_loaded_at_run_time_makes_fit_return_a_smaller_passing_arrangement() {
    let design = fixture("gas_igc_2g_self_propelled.vdesign");
    let plain = run(&design, &[]);
    let side = plain["geometry"]["side_clearance_m"].as_f64().unwrap();
    assert_eq!(plain["arrangement"]["feasible"], true);
    let tight = run(&design, &[side_gap_set("tight", side + 0.3)]);
    assert_eq!(
        tight["arrangement"]["feasible"], true,
        "{}",
        tight["checks"]
    );
    assert!(tight["geometry"]["side_clearance_m"].as_f64().unwrap() >= side + 0.3);
    assert!(
        cargo(&tight) < cargo(&plain),
        "{} >= {}",
        cargo(&tight),
        cargo(&plain)
    );
    // The loaded Rule passes on every tank of the answer, and no row fails.
    let rows = tight["rule_trace"]["rows"].as_array().unwrap();
    let mine: Vec<&Value> = rows
        .iter()
        .filter(|r| r["rule"] == "tight/x/side-gap")
        .collect();
    assert!(!mine.is_empty() && mine.iter().all(|r| r["outcome"] == "pass"));
    assert!(rows.iter().all(|r| r["outcome"] != "fail"));
}

#[test]
fn when_nothing_passes_fit_is_infeasible_with_the_failing_rows() {
    let design = fixture("gas_igc_2g_self_propelled.vdesign");
    let r = run(&design, &[side_gap_set("impossible", 100.0)]);
    assert_eq!(r["arrangement"]["feasible"], false);
    let feasible = r["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "Feasible arrangement")
        .unwrap();
    assert_eq!(feasible["ok"], false);
    // The failing rows are named, and the Option's trace shows them failing.
    assert!(
        feasible["message"]
            .as_str()
            .unwrap()
            .contains("impossible/x/side-gap"),
        "{}",
        feasible["message"]
    );
    let rows = r["rule_trace"]["rows"].as_array().unwrap();
    assert!(
        rows.iter()
            .any(|row| row["rule"] == "impossible/x/side-gap" && row["outcome"] == "fail")
    );
    assert_eq!(r["rule_trace"]["verdict"]["passes"], false);
}

#[test]
fn property_a_feasible_answer_never_has_a_failing_row_and_more_rules_never_add_cargo() {
    // A fixed list of hulls and a ladder of side-gap limits: each tighter set is the looser one with
    // a stricter Rule, so its answer may not carry more, and an infeasible answer stays infeasible.
    let base = fixture("gas_igc_2g_self_propelled.vdesign");
    for (loa, beam) in [(110.0, 11.4), (90.0, 14.8), (70.0, 10.0)] {
        let mut design = base.clone();
        let o = &mut design.options[0];
        for (name, v) in [("length_overall_m", loa), ("beam_m", beam)] {
            let i = o.inputs.iter_mut().find(|(n, _)| n == name).unwrap();
            i.1.value = json!(v);
        }
        o.results_hash = None;
        let mut last_cargo = f64::INFINITY;
        let mut was_feasible = true;
        for (k, gap) in [0.0, 1.5, 2.5, 3.5].into_iter().enumerate() {
            let sets: Vec<String> = (0..=k)
                .map(|i| side_gap_set(&format!("s{i}"), [0.0, 1.5, 2.5, 3.5][i]))
                .collect();
            let r = run(&design, &sets);
            let feasible = r["arrangement"]["feasible"].as_bool().unwrap();
            let rows = r["rule_trace"]["rows"].as_array().unwrap();
            if feasible {
                assert!(
                    rows.iter().all(|row| row["outcome"] != "fail"),
                    "{loa}x{beam} gap {gap}: feasible with a failing row"
                );
                assert!(
                    cargo(&r) <= last_cargo + 1e-9,
                    "{loa}x{beam} gap {gap}: more cargo under more Rules"
                );
                last_cargo = cargo(&r);
            }
            assert!(
                was_feasible || !feasible,
                "{loa}x{beam} gap {gap}: a stricter set made it feasible"
            );
            was_feasible = feasible;
        }
    }
}
