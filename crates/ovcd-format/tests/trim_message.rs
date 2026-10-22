//! Seam: the message of `trim_in_range` when no trim balances the load (issue #549).
//!
//! Label: property - the message names the cause, the LCG and the furthest LCB the hull reaches;
//! the numbers are compared with the Laden condition's trim analysis, not fixed.

use ovcd_format::{Profile, RunOutcome, fill_as_new, run_design};
use serde_json::{Value, json};

fn results(loa: f64, beam: f64, depth: f64) -> Value {
    let raw = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": "bauxite", "source": "typed" },
            "hull_form": { "value": "raked_box", "source": "typed" },
            "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
            "length_overall_m": { "value": loa, "source": "typed" },
            "beam_m": { "value": beam, "source": "typed" },
            "moulded_depth_m": { "value": depth, "source": "typed" }
        }}]
    });
    let filled = fill_as_new(&raw.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    match run_design(&filled.design.unwrap()).remove(0).outcome {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn trim_check(r: &Value) -> Value {
    r["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "trim in range")
        .cloned()
        .expect("a trim check")
}

#[test]
fn a_laden_hull_that_no_trim_balances_says_where_its_lcg_is_and_how_far_its_lcb_reaches() {
    let c = trim_check(&results(90.0, 14.0, 4.5));
    assert_eq!(c["ok"], false);
    let m = c["message"].as_str().unwrap();
    assert!(
        m.starts_with("no trim balances the load: Laden: LCG "),
        "{m}"
    );
    assert!(
        m.contains(" m is forward of the furthest LCB the hull reaches ("),
        "{m}"
    );
    assert!(!m.contains("10 degrees"), "{m}");
    // the LCG is further forward than the LCB the hull reaches
    let nums: Vec<f64> = m
        .split(|c: char| !(c.is_ascii_digit() || c == '.'))
        .filter_map(|t| t.parse::<f64>().ok())
        .collect();
    assert!(nums.len() >= 2 && nums[0] > nums[1], "{m}");
}

#[test]
fn a_hull_that_balances_says_it_floats_within_the_limit() {
    let c = trim_check(&results(60.0, 11.0, 4.0));
    assert_eq!(c["ok"], true);
    assert_eq!(
        c["message"],
        "the hull floats within 10 degrees of trim (every condition)"
    );
}
