//! Seam: the public `ovcd_core::checks` API. Behaviour copied from `core/checks.py`:
//! a `Check` is error-severity by default, and `all_ok` ignores failed warnings.

use ovcd_core::checks::{Check, all_ok};

#[test]
fn a_new_check_is_an_error_severity_check() {
    let c = Check::new("draught", true, "within limit");
    assert_eq!(c.name, "draught");
    assert!(c.ok);
    assert_eq!(c.message, "within limit");
    assert_eq!(c.severity, "error");
    assert!(c.is_error());
}

#[test]
fn a_warning_is_not_an_error() {
    let c = Check::with_severity("freeboard", false, "low", "warning");
    assert!(!c.is_error());
}

#[test]
fn any_other_severity_text_is_not_an_error_either() {
    // Python compares with `== "error"`, so "Error" or "fatal" do not gate.
    assert!(!Check::with_severity("a", false, "m", "Error").is_error());
    assert!(!Check::with_severity("a", false, "m", "").is_error());
}

#[test]
fn no_checks_is_ok() {
    assert!(all_ok(&[]));
}

#[test]
fn a_failed_error_gates_the_result() {
    let checks = [Check::new("a", true, ""), Check::new("b", false, "bad")];
    assert!(!all_ok(&checks));
}

#[test]
fn a_failed_warning_does_not_gate_the_result() {
    let checks = [
        Check::new("a", true, ""),
        Check::with_severity("b", false, "advisory", "warning"),
    ];
    assert!(all_ok(&checks));
}

#[test]
fn a_failed_check_of_unknown_severity_does_not_gate_the_result() {
    let checks = [Check::with_severity("b", false, "x", "fatal")];
    assert!(all_ok(&checks));
}
