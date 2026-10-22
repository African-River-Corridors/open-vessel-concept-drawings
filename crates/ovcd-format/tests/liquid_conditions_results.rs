//! The loading conditions in a crude oil run (issue #332, spec 0006 sections 2 and 4): the four IS
//! Code conditions with the cargo tanks of the layout, the worst slack-tank condition, the weights of
//! each condition, and the `stability` block the Stability view reads.
//!
//! Label: property. Every test states a rule; no number here is a printed result.

use ovcd_format::{Profile, RunOutcome, check, run_design};
use serde_json::Value;

fn raw_fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn results(name: &str) -> Value {
    let checked = check(&raw_fixture(name), &Profile::built_in());
    assert_eq!(checked.report.errors, Vec::<String>::new());
    let design = checked.design.unwrap();
    match run_design(&design).remove(0).outcome {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn conditions(r: &Value) -> &Vec<Value> {
    r["loading_conditions"]["conditions"].as_array().unwrap()
}

fn num(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

#[test]
fn a_self_propelled_crude_hull_has_the_four_conditions_and_the_slack_one() {
    let r = results("crude_integral_tanks_self_propelled");
    let names: Vec<&str> = conditions(&r)
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "Full load, departure",
            "Full load, arrival",
            "Ballast, departure",
            "Ballast, arrival",
            "Slack cargo tanks, worst case"
        ]
    );
    let roles: Vec<&str> = conditions(&r)
        .iter()
        .map(|c| c["role"].as_str().unwrap())
        .collect();
    assert_eq!(roles.last(), Some(&"slack_cargo"));
    // A pushed barge may sail without ballast: its fifth standard condition comes before the slack one.
    let p = results("crude_integral_tanks_pushed");
    let pushed: Vec<&str> = conditions(&p)
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert_eq!(pushed[4], "Lightship, no ballast");
    assert_eq!(pushed[5], "Slack cargo tanks, worst case");
}

#[test]
fn every_condition_lists_its_weights_and_they_add_up_to_the_displacement_and_kg() {
    for file in [
        "crude_integral_tanks_pushed",
        "crude_integral_tanks_self_propelled",
    ] {
        let r = results(file);
        for c in conditions(&r) {
            let w = c["weights"].as_array().unwrap();
            assert_eq!(w[0]["name"], "Lightship", "{file}");
            let mass: f64 = w.iter().map(|i| num(&i["mass_t"])).sum();
            assert!(
                (mass - num(&c["displacement_t"])).abs() < 1e-6,
                "{file} {}",
                c["name"]
            );
            let moment: f64 = w.iter().map(|i| num(&i["mass_t"]) * num(&i["kcg_m"])).sum();
            assert!(
                (moment / mass - num(&c["kg_m"])).abs() < 1e-9,
                "{file} {}",
                c["name"]
            );
            for i in w {
                assert!(i["lcg_m"].is_number(), "{file}");
            }
        }
    }
}

#[test]
fn the_arrival_conditions_carry_a_tenth_of_the_stores_and_fuel() {
    let r = results("crude_integral_tanks_self_propelled");
    let c = conditions(&r);
    let (dep, arr) = (num(&c[0]["consumables_t"]), num(&c[1]["consumables_t"]));
    assert!(dep > 0.0);
    assert!((arr - 0.1 * dep).abs() < 1e-9);
    // The slack condition is a departure one: full stores and fuel.
    assert!((num(&c[4]["consumables_t"]) - dep).abs() < 1e-9);
}

#[test]
fn the_ballast_conditions_fill_the_double_hull_and_carry_no_cargo() {
    let r = results("crude_integral_tanks_self_propelled");
    let tanks = r["loading_conditions"]["ballast_tanks"].as_array().unwrap();
    let kinds: Vec<&str> = tanks.iter().map(|t| t["kind"].as_str().unwrap()).collect();
    assert!(kinds.contains(&"double_bottom_ballast") && kinds.contains(&"wing_ballast"));
    assert!(kinds.contains(&"peak_ballast"));
    let ballast = &conditions(&r)[2];
    assert_eq!(num(&ballast["cargo_t"]), 0.0);
    // The automatic plan carries the least ballast that meets every criterion. On the hull's
    // sections (issue #448) this light crude hull meets them with none, so the plan is found and
    // may be empty; the box hull before needed some.
    assert_eq!(ballast["plan_found"], true);
    assert!(num(&ballast["ballast_t"]) >= 0.0);
    assert!(
        ballast["weights"]
            .as_array()
            .unwrap()
            .iter()
            .all(|w| w["name"] != "Cargo")
    );
}

#[test]
fn the_slack_condition_is_slack_and_not_better_than_the_full_departure() {
    let r = results("crude_integral_tanks_self_propelled");
    let c = conditions(&r);
    let (laden, slack) = (&c[0], &c[4]);
    assert!(num(&slack["free_surface_m"]) > 0.0);
    assert!(num(&slack["cargo_t"]) <= num(&laden["cargo_t"]));
    assert!(num(&slack["gm_fluid_m"]) <= num(&laden["gm_fluid_m"]) + 1e-9);
    assert!(num(&slack["fill"]) < 0.98);
}

#[test]
fn the_stability_block_is_there_for_the_view_and_the_checks_are_warnings() {
    let r = results("crude_integral_tanks_self_propelled");
    assert!(r["stability"]["conditions"].as_array().unwrap().len() >= 5);
    assert!(r["stability"]["lowest_gm_condition"].is_string());
    let stability_checks: Vec<&Value> = r["checks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["name"].as_str().unwrap().starts_with("Stability"))
        .collect();
    assert!(!stability_checks.is_empty());
    assert!(stability_checks.iter().all(|c| c["severity"] == "warning"));
    assert!(
        stability_checks
            .iter()
            .any(|c| c["name"].as_str().unwrap().contains("slack cargo tanks"))
    );
    // The Rule Trace stays out of it until the liquid Rule Sets exist.
    assert!(
        r["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .all(|w| !w.as_str().unwrap().contains("not computed for liquid"))
    );
}

#[test]
fn the_other_cargo_classes_list_no_weights() {
    for file in ["dry_bulk_box_hold"] {
        let r = results(file);
        if let Some(list) = r["loading_conditions"]["conditions"].as_array() {
            for c in list {
                assert!(c.get("weights").is_none(), "{file}");
            }
        }
    }
}
