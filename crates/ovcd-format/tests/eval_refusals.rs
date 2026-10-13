//! `ovcd eval` refuses an input the core cannot hold exactly, instead of returning a different
//! answer from Python's.

use ovcd_format::eval;
use serde_json::Value;

fn vs_input(loa: &str) -> Value {
    let text = format!(
        r#"{{"fn":"validate_catalogue","args":{{"extra":{{"X":{{"standard_id":"X","family":"CN","class_name":"I","formation":"motor_vessel","loa_m":{loa},"beam_m":5.0,"draught_m":[1.0,2.0],"dwt_t":[1.0,2.0],"source":"s"}}}}}}}}"#
    );
    ovcd_format::parity::parse(&text).unwrap()
}

#[test]
fn integer_range_beyond_two_pow_53_is_refused() {
    let err = eval(
        "vessel_standards",
        &vs_input("[9007199254740993,9007199254740992]"),
    );
    assert!(err.is_err(), "{err:?}");
    let err = eval("vessel_standards", &vs_input("[-9007199254740993,1]"));
    assert!(err.is_err(), "{err:?}");
}

#[test]
fn integer_range_up_to_two_pow_53_and_floats_are_accepted() {
    assert!(
        eval(
            "vessel_standards",
            &vs_input("[9007199254740992,9007199254740992]")
        )
        .is_ok()
    );
    assert!(
        eval(
            "vessel_standards",
            &vs_input("[9.007199254740993e15,9.1e15]")
        )
        .is_ok()
    );
    assert!(eval("vessel_standards", &vs_input("[1,2]")).is_ok());
}
