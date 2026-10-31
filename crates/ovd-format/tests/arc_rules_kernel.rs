//! Label: property
//!
//! OVD on the domain-free rules kernel (`arc-rules`, ADR 0019): the schema beside the built-in
//! files is the kernel's, and OVD declares its vocabulary to the kernel's loader.

use arc_rules::domain::{Domain, InputScope};
use ovd_format::rule_set::{Ovd, SCHEMA};

#[test]
fn the_schema_beside_the_rule_files_is_the_kernels() {
    let here = include_str!("../rules/rule-set.schema.json");
    assert_eq!(
        here, SCHEMA,
        "copy crates/arc-rules/rule-set.schema.json here"
    );
}

#[test]
fn ovd_declares_its_vocabulary() {
    assert_eq!(Ovd.design_object(), "option");
    assert_eq!(Ovd.measure("gm0").unwrap().object, "loading-condition");
    assert_eq!(Ovd.measure("input.beam_m").unwrap().object, "option");
    let shapes: Vec<&str> = Ovd.shapes().iter().map(|s| s.id).collect();
    assert_eq!(
        shapes,
        [
            "less-input",
            "clamp",
            "at-least",
            "max-capped-less",
            "lesser-ratio",
            // Issue #740: the scantling shapes of `abs-steel-barges`.
            "linear-ls",
            "linear-ls-less",
            "linear-ls-split",
            "linear-ls-split-capped",
            "deck-b",
            "abs-smr",
            "plate-sqrt-head",
            "sm-chsl2",
            "sm-chsl2-hull-k",
            "tank-head",
            "bulkhead-head",
            "web-minimum",
        ]
    );
    // Every OVD input has one value per Option.
    let cargo = Ovd.input("cargo_class").unwrap();
    assert_eq!(cargo.scope, InputScope::Design);
    assert!(Ovd.input("no_such_input").is_none());
}
