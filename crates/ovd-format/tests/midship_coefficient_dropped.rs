//! Seam: `midship_coefficient` leaves format version 3 (issue #535). The hull gives its own midship
//! section from its Sections Table (#448), so Cm is a result (`hydrostatics.cm`), not an input.
//!
//! Label: property. A file that stores the input opens with it dropped, one warning per Option, and
//! the same results; no input of that name is left in the schema.

use ovd_format::vdesign::INPUTS;
use ovd_format::{DesignSession, HashStatus, Profile, check};
use serde_json::{Value, json};

fn with_cm(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut file: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    for option in file["options"].as_array_mut().unwrap() {
        option["inputs"]["midship_coefficient"] = json!({ "value": 0.9, "source": "typed" });
    }
    file.to_string()
}

#[test]
fn no_input_is_named_midship_coefficient() {
    assert!(INPUTS.iter().all(|s| s.name != "midship_coefficient"));
}

#[test]
fn a_file_that_stores_it_opens_with_one_warning_per_option_and_the_same_results() {
    for (file, options) in [("dry_bulk_box_hold", 1), ("two_options", 2)] {
        let text = with_cm(file);
        let checked = check(&text, &Profile::built_in());
        assert!(
            checked.report.errors.is_empty(),
            "{:?}",
            checked.report.errors
        );
        let notes = checked
            .report
            .warnings
            .iter()
            .filter(|w| w.contains("the midship coefficient now comes from the hull shape"))
            .count();
        assert_eq!(notes, options, "{file}: {:?}", checked.report.warnings);
        let design = checked.design.unwrap();
        for o in &design.options {
            assert!(o.input("midship_coefficient").is_none(), "{file}");
        }
        let (_, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
        for (name, status) in &opened.hash_status {
            assert_eq!(*status, HashStatus::Matches, "{file}: {name}");
        }
    }
}
