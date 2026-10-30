//! Seam: the longest bow rake the bow rake over depth ratio gives (issue #661). A deep hull whose
//! depth is set by its tanks lost cargo length to a rake of 1.6 x depth; the new input caps it.
//!
//! Label: property. Every test states a rule: the default rake is the smaller of the ratio x depth
//! and the cap, a typed rake is not capped, a file saved before keeps its results (its stored hash
//! still matches), and a new Design takes the profile's cap. No number here is a printed result.

use ovd_core::barge_capacity::default_bow_rake_m;
use ovd_format::{DesignSession, HashStatus, Profile, RunOutcome, check};
use serde_json::{Value, json};

const CAP: &str = "bow_rake_max_length_m";

fn example() -> String {
    let path = format!(
        "{}/../../examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn results(edits: &[(&str, Value)]) -> Value {
    let mut f: Value = serde_json::from_str(&example()).unwrap();
    for (name, value) in edits {
        f["options"][0]["inputs"][*name] = json!({ "value": value, "source": "typed" });
    }
    let (session, _) = DesignSession::open(&f.to_string(), Profile::built_in()).unwrap();
    match session.evaluate().options[0]
        .outcome
        .as_ref()
        .expect("it ran")
    {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

fn geo(v: &Value, key: &str) -> f64 {
    v["geometry"][key].as_f64().unwrap()
}

#[test]
fn the_default_rake_is_the_smaller_of_the_ratio_times_depth_and_the_cap() {
    assert_eq!(default_bow_rake_m(1.6, 12.0, Some(10.0)), 10.0);
    assert_eq!(default_bow_rake_m(1.6, 5.0, Some(10.0)), 1.6 * 5.0);
    assert_eq!(default_bow_rake_m(1.6, 12.0, None), 1.6 * 12.0);
}

#[test]
fn a_file_saved_before_keeps_no_cap_and_its_results() {
    let checked = check(&example(), &Profile::built_in());
    let d = checked.design.unwrap();
    for option in &d.options {
        assert_eq!(option.input(CAP).unwrap().value, Value::Null);
    }
    let (_, opened) = DesignSession::open(&example(), Profile::built_in()).unwrap();
    for (name, status) in &opened.hash_status {
        assert_eq!(*status, HashStatus::Matches, "{name}");
    }
}

#[test]
fn a_new_design_takes_the_profile_cap_as_an_estimate() {
    let profile = Profile::built_in();
    assert_eq!(profile.default_status(CAP).map(|s| s.0), Some("estimate"));
    let s = DesignSession::new_design(profile.clone()).unwrap();
    assert_eq!(
        s.design().options[0].input(CAP).unwrap().value,
        json!(profile.default(CAP).unwrap())
    );
}

#[test]
fn the_cap_shortens_the_rake_of_a_deep_hull_and_the_cargo_zone_takes_the_length() {
    let depth = 12.0;
    let free = results(&[]);
    assert_eq!(free["dims"]["depth_m"].as_f64().unwrap(), depth);
    let capped = results(&[(CAP, json!(10.0))]);
    assert_eq!(geo(&capped, "bow_rake_length_m"), 10.0);
    let gained = geo(&free, "bow_rake_length_m") - geo(&capped, "bow_rake_length_m");
    assert!((gained - (1.6 * depth - 10.0)).abs() < 1e-9);
    assert!(
        (geo(&capped, "cargo_zone_length_m") - geo(&free, "cargo_zone_length_m") - gained).abs()
            < 1e-9
    );
    // A cap above the ratio x depth changes nothing.
    let high = results(&[(CAP, json!(1.6 * depth + 1.0))]);
    assert_eq!(
        geo(&high, "bow_rake_length_m"),
        geo(&free, "bow_rake_length_m")
    );
}

#[test]
fn a_typed_bow_rake_is_not_capped() {
    let r = results(&[(CAP, json!(10.0)), ("bow_rake_length_m", json!(15.0))]);
    assert_eq!(geo(&r, "bow_rake_length_m"), 15.0);
}

#[test]
fn the_cap_refuses_zero_and_below() {
    for bad in [json!(0.0), json!(-1.0)] {
        let mut f: Value = serde_json::from_str(&example()).unwrap();
        f["options"][0]["inputs"][CAP] = json!({ "value": bad, "source": "typed" });
        let checked = check(&f.to_string(), &Profile::built_in());
        assert!(
            checked.report.errors.iter().any(|e| e.contains(CAP)),
            "{bad}: {:?}",
            checked.report.errors
        );
    }
}
