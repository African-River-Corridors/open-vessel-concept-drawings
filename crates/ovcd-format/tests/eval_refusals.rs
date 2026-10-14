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

fn convoy_input(unit: &str, n: &str, beam: &str) -> Value {
    let text = format!(
        r#"{{"fn":"validate_catalogue","args":{{"extra":{{"X":{{"standard_id":"X","family":"CN","class_name":"I","formation":"pushed_convoy","loa_m":[1.0,2.0],"beam_m":{beam},"draught_m":[1.0,2.0],"dwt_t":[1.0,2.0],"source":"s","unit_beam_m":{unit},"n_abreast":{n}}}}}}}}}"#
    );
    ovcd_format::parity::parse(&text).unwrap()
}

#[test]
fn integer_convoy_product_beyond_two_pow_53_is_refused() {
    // 3 x 3002399751580331 = 2^53 + 1: Python is exact, a double is not.
    let e = eval(
        "vessel_standards",
        &convoy_input("3002399751580331", "3", "9007199254740992"),
    );
    assert!(e.is_err(), "{e:?}");
    let e = eval(
        "vessel_standards",
        &convoy_input("3", "3002399751580331", "9007199254740992"),
    );
    assert!(e.is_err(), "{e:?}");
    let e = eval(
        "vessel_standards",
        &convoy_input("1", "9007199254740993", "1"),
    );
    assert!(e.is_err(), "{e:?}");
}

#[test]
fn convoy_product_within_two_pow_53_and_float_unit_are_accepted() {
    assert!(eval("vessel_standards", &convoy_input("3", "3", "9")).is_ok());
    assert!(
        eval(
            "vessel_standards",
            &convoy_input("3002399751580330", "3", "9007199254740990")
        )
        .is_ok()
    );
    assert!(
        eval(
            "vessel_standards",
            &convoy_input("3002399751580331.0", "3", "9007199254740992")
        )
        .is_ok()
    );
}

fn cost(inputs: &str, rates: &str) -> Value {
    let text = format!(r#"{{"fn":"compute","args":{{"inputs":{inputs},"rates":{rates}}}}}"#);
    ovcd_format::parity::parse(&text).unwrap()
}

const RATES: &str = r#"{"hull_steel_rate":1,"tank_shell_rate":1.0,"tank_supports_rate":1.0,"insulation_rate":1.0,"piping_rate":1.0,"outfitting_rate":1.0,"cargo_system_lump":1.0,"paint_anodes_misc":1.0,"battery_rate_per_kwh":1.0,"propulsion_rate_per_kw":1.0,"wheelhouse_accom_lump":1.0,"tug_outfitting_lump":1.0,"yard_margin_fraction":0.1,"delivery_lump":50000}"#;

#[test]
fn barge_cost_integers_beyond_two_pow_24_are_refused() {
    let e = eval(
        "barge_cost",
        &cost(r#"{"hull_steel_t":9007199254740993}"#, RATES),
    );
    assert!(e.is_err(), "{e:?}");
    let e = eval("barge_cost", &cost(r#"{"hull_steel_t":16777217}"#, RATES));
    assert!(e.is_err(), "{e:?}");
    let big = RATES.replace(
        r#""delivery_lump":50000"#,
        r#""delivery_lump":9007199254740993"#,
    );
    let e = eval("barge_cost", &cost(r#"{"hull_steel_t":1}"#, &big));
    assert!(e.is_err(), "{e:?}");
}

#[test]
fn barge_cost_small_integers_and_big_floats_are_accepted() {
    assert!(eval("barge_cost", &cost(r#"{"hull_steel_t":16777216}"#, RATES)).is_ok());
    assert!(
        eval(
            "barge_cost",
            &cost(r#"{"hull_steel_t":9007199254740993.0}"#, RATES)
        )
        .is_ok()
    );
}

fn ukc_input(v_hi: &str) -> Value {
    let text = format!(
        r#"{{"fn":"max_speed_for_ukc","args":{{"draft_m":2.0,"beam_m":10.0,"lpp_m":50.0,"water_depth_m":10.0,"channel_area_m2":1000.0,"v_hi_kn":{v_hi}}}}}"#
    );
    ovcd_format::parity::parse(&text).unwrap()
}

#[test]
fn max_speed_for_ukc_refuses_a_json_integer() {
    // Python keeps the int and returns `1`, not `1.0`; the port does not model that.
    let err = eval("squat", &ukc_input("1"));
    assert!(err.is_err(), "{err:?}");
    assert!(eval("squat", &ukc_input("1.0")).is_ok());
}

#[test]
fn a_whole_number_where_a_gas_arrangement_float_is_expected_is_refused() {
    // The adapter reads floats strictly: `18` is an integer, `18.0` a float.
    let text = r#"{"fn":"fit","args":{"loa":90.0,"beam":14.8,"cargo":"LNG","t_max":4.4,"max_tank_od_m":8}}"#;
    let err = eval(
        "gas_arrangement",
        &ovcd_format::parity::parse(text).unwrap(),
    );
    let msg = format!("{err:?}");
    assert!(err.is_err() && msg.contains("must be a float"), "{msg}");
    let ok = r#"{"fn":"fit","args":{"loa":90.0,"beam":14.8,"cargo":"LNG","t_max":4.4,"max_tank_od_m":8.0}}"#;
    assert!(eval("gas_arrangement", &ovcd_format::parity::parse(ok).unwrap()).is_ok());
}

fn simulate_refusal(params: &str) -> String {
    let text = format!(r#"{{"fn":"simulate","args":{{"params":{params}}}}}"#);
    let err = eval("simulate", &ovcd_format::parity::parse(&text).unwrap());
    format!("{err:?}")
}

#[test]
fn simulate_refuses_params_where_python_would_raise_key_or_type_error() {
    let good =
        r#""vessel":"hopper_barge","cargo":"bauxite","loa_m":70,"beam_m":11,"depth_m":4,"cb":0.9"#;
    // Control: the same params without a defect are accepted.
    let ok = ovcd_format::parity::parse(&format!(
        r#"{{"fn":"simulate","args":{{"params":{{{good}}}}}}}"#
    ))
    .unwrap();
    assert!(eval("simulate", &ok).is_ok());
    for (what, params) in [
        (
            "unknown vessel",
            r#"{"vessel":"tug","cargo":"bauxite","loa_m":70,"beam_m":11,"depth_m":4,"cb":0.9}"#,
        ),
        (
            "unknown cargo",
            r#"{"vessel":"hopper_barge","cargo":"coal","loa_m":70,"beam_m":11,"depth_m":4,"cb":0.9}"#,
        ),
        (
            "missing dimension",
            r#"{"vessel":"hopper_barge","cargo":"bauxite","beam_m":11,"depth_m":4,"cb":0.9}"#,
        ),
        (
            "text for a number",
            r#"{"vessel":"hopper_barge","cargo":"bauxite","loa_m":"70","beam_m":11,"depth_m":4,"cb":0.9}"#,
        ),
        (
            "null for a number",
            r#"{"vessel":"hopper_barge","cargo":"bauxite","loa_m":70,"beam_m":11,"depth_m":4,"cb":0.9,"freeboard_min_m":null}"#,
        ),
        (
            "box without lightship_k",
            r#"{"vessel":"box_barge","cargo":"bauxite","loa_m":70,"beam_m":11,"depth_m":4,"cb":0.9}"#,
        ),
    ] {
        let msg = simulate_refusal(params);
        assert!(msg.starts_with("Err(BadInput"), "{what}: {msg}");
    }
}

#[test]
fn simulate_needs_a_params_object_and_a_known_function() {
    let err = eval(
        "simulate",
        &ovcd_format::parity::parse(r#"{"fn":"simulate","args":{}}"#).unwrap(),
    );
    assert!(format!("{err:?}").contains("params"), "{err:?}");
    let err = eval(
        "simulate",
        &ovcd_format::parity::parse(r#"{"fn":"nope","args":{}}"#).unwrap(),
    );
    assert!(format!("{err:?}").contains("no parity function"), "{err:?}");
}
