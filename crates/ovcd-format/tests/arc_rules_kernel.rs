//! Label: property
//!
//! OVCD on the domain-free rules kernel (`arc-rules`, ADR 0019): the schema beside the built-in
//! files is the kernel's, and OVCD declares its vocabulary to the kernel's loader.

use arc_rules::domain::{Domain, InputScope};
use ovcd_format::rule_set::{Ovcd, SCHEMA};

#[test]
fn the_schema_beside_the_rule_files_is_the_kernels() {
    let here = include_str!("../rules/rule-set.schema.json");
    assert_eq!(
        here, SCHEMA,
        "copy crates/arc-rules/rule-set.schema.json here"
    );
}

#[test]
fn ovcd_declares_its_vocabulary() {
    assert_eq!(Ovcd.design_object(), "option");
    assert_eq!(Ovcd.measure("gm0").unwrap().object, "loading-condition");
    assert_eq!(Ovcd.measure("input.beam_m").unwrap().object, "option");
    let shapes: Vec<&str> = Ovcd.shapes().iter().map(|s| s.id).collect();
    assert_eq!(
        shapes,
        [
            "less-input",
            "clamp",
            "at-least",
            "max-capped-less",
            "lesser-ratio"
        ]
    );
    // Every OVCD input has one value per Option.
    let cargo = Ovcd.input("cargo_class").unwrap();
    assert_eq!(cargo.scope, InputScope::Design);
    assert!(Ovcd.input("no_such_input").is_none());
}
