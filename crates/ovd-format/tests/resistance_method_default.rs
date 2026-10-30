//! Seam: the default `resistance_method` (issue #735). New Designs take `barge_form_factor` from the
//! Defaults Profile; a file saved without the input keeps `barge_fit`, and a file that stores a
//! method keeps it, so saved Designs reproduce.
//!
//! Label: property. No number here is a printed result.

use ovd_format::vdesign::new_design;
use ovd_format::{DesignSession, HashStatus, Profile, check};
use serde_json::{Value, json};

fn legacy(name: &str) -> String {
    let path = format!("{}/tests/legacy/{name}.vdesign", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

#[test]
fn a_new_design_takes_the_barge_form_factor() {
    let d = new_design(&Profile::built_in()).unwrap();
    for o in &d.options {
        assert_eq!(
            o.input("resistance_method").unwrap().value,
            json!("barge_form_factor")
        );
    }
}

#[test]
fn a_file_saved_without_the_input_keeps_the_barge_fit_and_its_results() {
    let text = legacy("v1_dry_bulk_box_hold");
    let raw: Value = serde_json::from_str(&text).unwrap();
    assert!(raw["options"][0]["inputs"].get("resistance_method").is_none());
    let d = check(&text, &Profile::built_in()).design.unwrap();
    assert_eq!(
        d.options[0].input("resistance_method").unwrap().value,
        json!("barge_fit")
    );
    let (_, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
}

#[test]
fn a_stored_method_is_kept() {
    let text = legacy("v1_dry_bulk_box_hold");
    let mut raw: Value = serde_json::from_str(&text).unwrap();
    raw["options"][0]["inputs"]["resistance_method"] =
        json!({ "value": "holtrop_mennen", "source": "typed" });
    let d = check(&raw.to_string(), &Profile::built_in()).design.unwrap();
    assert_eq!(
        d.options[0].input("resistance_method").unwrap().value,
        json!("holtrop_mennen")
    );
}
