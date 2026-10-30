//! Spec 0010: the rename to OVD has no compatibility. A file saved by 0.2.x stores the Rule Set of
//! the old name and does not open; the one error names the cause. The fixture is the towed barge as
//! 0.2.4 saved it.
//!
//! Label: property. No number here is a printed result.

use ovd_format::{Profile, check, fill};

fn saved_by_0_2_4() -> String {
    let path = format!(
        "{}/tests/legacy/before_ovd_0_2_4_towed_barge.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn a_file_saved_by_0_2_4_does_not_open_and_the_error_names_the_rename() {
    for checked in [
        check(&saved_by_0_2_4(), &Profile::built_in()),
        fill(&saved_by_0_2_4(), &Profile::built_in()),
    ] {
        assert!(checked.design.is_none());
        assert_eq!(
            checked.report.errors.len(),
            1,
            "{:?}",
            checked.report.errors
        );
        let e = &checked.report.errors[0];
        assert!(
            e.contains("before the rename to OVD") && e.contains("0.2.x"),
            "{e}"
        );
    }
}

#[test]
fn the_same_file_with_the_rule_set_ovd_opens() {
    let old = "\"id\": \"ovcd\""; // before-ovd: the old Rule Set id
    let text = saved_by_0_2_4().replace(old, "\"id\": \"ovd\"");
    let checked = check(&text, &Profile::built_in());
    assert!(checked.design.is_some());
    assert!(
        checked.report.errors.iter().all(|e| !e.contains("rename")),
        "{:?}",
        checked.report.errors
    );
}
