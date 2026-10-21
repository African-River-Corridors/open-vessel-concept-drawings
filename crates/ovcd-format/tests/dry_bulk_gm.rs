//! Issue #415: the dry bulk hold GM at the cargo the Design reports.
//!
//! Label: property. The rules: every dry bulk Option has one GM check, at its reported cargo and
//! draught, as a warning; the old check at the volume-bound load is gone; the Rule that reports it
//! is in the trace.

use serde_json::Value;

fn results_of_text(text: &str) -> Vec<Value> {
    let design = ovcd_format::check(text, &ovcd_format::Profile::built_in())
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

fn fixture(file: &str) -> String {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(file),
    )
    .unwrap()
}

fn gm_checks(results: &Value) -> Vec<&Value> {
    results["checks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["name"].as_str().unwrap().contains("gm"))
        .collect()
}

#[test]
fn a_dense_dry_bulk_load_that_sinks_the_hull_still_gets_a_gm_at_the_reported_cargo() {
    // Bauxite in this hold is volume-bound above what the hull floats: the capacity result is
    // overloaded, so before #415 no GM figure appeared at all.
    for results in results_of_text(&fixture("dry_bulk_box_hold.vdesign")) {
        let checks = results["checks"].as_array().unwrap();
        let overloaded = checks
            .iter()
            .any(|c| c["name"] == "not overloaded" && c["ok"] == false);
        assert!(overloaded, "the case needs an overloaded hold");
        let gm = gm_checks(&results);
        assert_eq!(gm.len(), 1, "{gm:?}");
        let c = gm[0];
        assert_eq!(c["name"], "intact gm at reported cargo");
        assert_eq!(c["severity"], "warning");
        let cargo = results["results"]["cargo_t"].as_f64().unwrap();
        let draught = results["results"]["draught_m"].as_f64().unwrap();
        let message = c["message"].as_str().unwrap();
        assert!(
            message.contains(&format!(
                "reported cargo {cargo:.0} t, draught {draught:.2} m"
            )),
            "{message}"
        );
        // The Rule that reports the check is in the trace, with a row that agrees with it.
        let row = results["rule_trace"]["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["rule"] == "ovcd/capacity/intact-gm")
            .expect("a row for ovcd/capacity/intact-gm");
        assert_eq!(row["outcome"] == "pass", c["ok"] == true);
    }
}

#[test]
fn every_dry_bulk_fixture_has_one_gm_check_and_no_volume_bound_one() {
    for file in [
        "dry_bulk_box_hold.vdesign",
        "dry_bulk_sloped_hopper.vdesign",
    ] {
        for results in results_of_text(&fixture(file)) {
            let gm = gm_checks(&results);
            assert_eq!(gm.len(), 1, "{file}: {gm:?}");
            assert_eq!(gm[0]["name"], "intact gm at reported cargo", "{file}");
        }
    }
}
