//! Seam: the calibration label of a result (issue #434). The lightship of a hopper barge and of a
//! gas barge has no calibration case; the box barge's does. The label is an output, kept out of the
//! results hash, so no saved Design moves.
//!
//! Label: property. Each test states a rule (which hulls carry the label, that the hash does not
//! move). No number here is a printed result.

use ovd_format::{DesignSession, Profile, RunOutcome, UNHASHED_KEYS, results_hash};
use serde_json::{Value, json};

fn raw_fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn results(fixture: &str) -> Value {
    let s = DesignSession::open(&raw_fixture(fixture), Profile::built_in())
        .unwrap()
        .0;
    match s.evaluate().options[0].outcome.as_ref() {
        Some(RunOutcome::Results { results, .. }) => results.clone(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_hopper_and_a_gas_design_show_the_uncalibrated_label_and_a_box_design_does_not() {
    for (fixture, want) in [
        ("dry_bulk_sloped_hopper", "uncalibrated"),
        ("gas_igc_2g_exact", "uncalibrated"),
        ("gas_igc_2g_self_propelled", "uncalibrated"),
        ("dry_bulk_box_hold", "calibrated"),
    ] {
        let r = results(fixture);
        assert_eq!(
            r["calibration"]["lightship"],
            json!(want),
            "{fixture}: {}",
            r["calibration"]
        );
        assert!(
            r["calibration"]["basis"]
                .as_str()
                .is_some_and(|b| !b.is_empty()),
            "{fixture}: the label carries its basis"
        );
    }
}

#[test]
fn the_label_is_not_in_the_results_hash() {
    assert!(UNHASHED_KEYS.contains(&"calibration"));
    let mut r = results("gas_igc_2g_exact");
    let with = results_hash(&r);
    r["calibration"]["lightship"] = json!("something else");
    assert_eq!(results_hash(&r), with);
    r.as_object_mut().unwrap().remove("calibration");
    assert_eq!(results_hash(&r), with);
}
