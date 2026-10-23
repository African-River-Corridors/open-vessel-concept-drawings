//! Issue #421: one air draft verdict per Design.
//!
//! Label: property. With an air draft limit, the hull's air draft is checked once, by the capacity
//! check that takes the ballasted light condition when the hull carries ballast
//! (`air draft within limit`). The run used to add a second check on the empty light condition
//! (`Air draft (empty, governing)`), so a ballasted Design could get two verdicts.

use serde_json::{Value, json};

fn results_with_limit(file: &str, limit: f64) -> Vec<Value> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(file);
    let mut design: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    for option in design["options"].as_array_mut().unwrap() {
        option["inputs"]["air_draft_limit_m"] = json!({"value": limit, "source": "typed"});
        option.as_object_mut().unwrap().remove("results_hash");
    }
    let checked = ovcd_format::check(&design.to_string(), &ovcd_format::Profile::built_in());
    assert_eq!(checked.report.errors, Vec::<String>::new(), "{file}");
    ovcd_format::run_design(&checked.design.unwrap())
        .into_iter()
        .filter_map(|r| match r.outcome {
            ovcd_format::RunOutcome::Results { results, .. } => Some(results),
            _ => None,
        })
        .collect()
}

#[test]
fn a_design_with_an_air_draft_limit_gets_one_hull_air_draft_verdict() {
    for file in [
        "dry_bulk_box_hold.vdesign",
        "dry_bulk_sloped_hopper.vdesign",
        "gas_igc_2g_self_propelled.vdesign",
        "gas_igc_2g_exact.vdesign",
    ] {
        // One limit the hull meets and one it does not: the one verdict follows the limit.
        for limit in [3.0, 30.0] {
            let all = results_with_limit(file, limit);
            assert!(!all.is_empty(), "{file}");
            for results in all {
                let names: Vec<&str> = results["checks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|c| c["name"].as_str().unwrap())
                    .collect();
                let hull: Vec<&&str> = names
                    .iter()
                    .filter(|n| {
                        **n == "air draft within limit" || **n == "Air draft (empty, governing)"
                    })
                    .collect();
                assert_eq!(hull, [&"air draft within limit"], "{file} at {limit} m");
                let check = results["checks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|c| c["name"] == "air draft within limit")
                    .unwrap();
                assert_eq!(check["ok"], json!(limit > 20.0), "{file} at {limit} m");
            }
        }
    }
}

fn condition_checks(results: &Value, name: &str) -> Vec<bool> {
    results["checks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["name"] == name)
        .map(|c| c["ok"].as_bool().unwrap())
        .collect()
}

fn outcome(results: &Value, rule: &str) -> String {
    let rows: Vec<&Value> = results["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["rule"] == rule)
        .collect();
    assert_eq!(rows.len(), 1, "{rule}: {rows:?}");
    rows[0]["outcome"].as_str().unwrap().to_string()
}

#[test]
fn the_per_condition_checks_report_their_ovcd_rules() {
    // Issue #628: the checks "Air draft" and "Ballast plan" of the IS Code loading conditions come
    // only for a condition that fails them, and each reports its Rule: a block when it is there,
    // not applicable when no condition fails.
    for (file, limit, air, ballast) in [
        ("dry_bulk_box_hold.vdesign", 3.0, "fail", "not applicable"),
        (
            "dry_bulk_box_hold.vdesign",
            30.0,
            "not applicable",
            "not applicable",
        ),
        ("gas_igc_2g_self_propelled.vdesign", 3.0, "fail", "fail"),
    ] {
        for results in results_with_limit(file, limit) {
            for (name, rule, want) in [
                ("Air draft", "ovcd/loading/air-draft", air),
                ("Ballast plan", "ovcd/loading/ballast-plan", ballast),
            ] {
                let checks = condition_checks(&results, name);
                assert!(checks.iter().all(|ok| !ok), "{file} at {limit} m: {name}");
                assert_eq!(
                    checks.is_empty(),
                    want == "not applicable",
                    "{file} at {limit} m: {name}"
                );
                assert_eq!(outcome(&results, rule), want, "{file} at {limit} m: {rule}");
            }
        }
    }
}
