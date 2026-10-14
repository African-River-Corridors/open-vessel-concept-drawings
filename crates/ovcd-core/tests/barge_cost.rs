//! Seam: the public `ovcd_core::barge_cost` API. Figures from `core/barge_cost.py` run on the
//! worked case in the audited test (rates 1000 per tonne, margin 10%).

use ovcd_core::barge_cost::*;

fn text_rates(text: &str) -> Result<BargeCostRates, String> {
    let pairs = parse_flat_yaml(text)?;
    let mapping: Vec<_> = pairs
        .into_iter()
        .map(|(k, v)| (k, MappingValue::Text(v)))
        .collect();
    BargeCostRates::from_mapping(&mapping).map_err(|e| e.message)
}

#[test]
fn the_example_rates_price_the_default_barge() {
    let result = compute(&BargeCostInputs::default(), &example_rates().unwrap()).unwrap();
    assert_eq!(result.unit_total, 6_177_125);
    assert!(result.ok());
}

#[test]
fn a_rates_file_is_read_from_text() {
    let r = text_rates(EXAMPLE_RATES_TEXT).unwrap();
    assert_eq!(r.yard_margin_fraction, 0.10);
    assert_eq!(r.currency, "USD");
}

#[test]
fn nested_yaml_is_rejected_loudly() {
    assert_eq!(
        parse_flat_yaml("rates:\n  hull_steel_rate: 1\n"),
        Err("rates file line 1: expected flat 'key: value', got 'rates:'".to_string())
    );
}

#[test]
fn a_duplicate_key_is_rejected() {
    assert_eq!(
        parse_flat_yaml("a: 1\na: 2\n"),
        Err("rates file line 2: duplicate key 'a'".to_string())
    );
}

#[test]
fn a_bad_number_names_the_text() {
    let err = text_rates(&EXAMPLE_RATES_TEXT.replace("2500.0", "lots")).unwrap_err();
    assert_eq!(err, "could not convert string to float: 'lots'");
}

fn flat_error(text: &str) -> String {
    parse_flat_yaml(text).unwrap_err()
}

#[test]
fn each_way_a_line_can_be_malformed_is_rejected() {
    // No colon; empty key; empty value; indented line; key that is not a name.
    assert_eq!(
        flat_error("abc\n"),
        "rates file line 1: expected flat 'key: value', got 'abc'"
    );
    assert_eq!(
        flat_error(": 5\n"),
        "rates file line 1: expected flat 'key: value', got ': 5'"
    );
    assert_eq!(
        flat_error("a:\n"),
        "rates file line 1: expected flat 'key: value', got 'a:'"
    );
    assert_eq!(
        flat_error(" a: 1\n"),
        "rates file line 1: expected flat 'key: value', got ' a: 1'"
    );
    assert_eq!(
        flat_error("1a: 1\n"),
        "rates file line 1: expected flat 'key: value', got '1a: 1'"
    );
}

#[test]
fn comments_blank_lines_and_quotes_are_handled_as_python_does() {
    let pairs = parse_flat_yaml("# note\n\n  # indented comment only\nk: 'v'  # tail\n").unwrap();
    assert_eq!(pairs, vec![("k".to_string(), "v".to_string())]);
}

#[test]
fn line_numbers_count_every_line() {
    assert_eq!(
        flat_error("# one\n\nbad\n"),
        "rates file line 3: expected flat 'key: value', got 'bad'"
    );
}

#[test]
fn a_zero_mass_fails_the_error_check_and_the_result_is_not_ok() {
    let rates = example_rates().unwrap();
    let ok = compute(&BargeCostInputs::default(), &rates).unwrap();
    assert!(ok.ok());
    let bad = compute(
        &BargeCostInputs {
            tank_shell_t: 0.0,
            ..BargeCostInputs::default()
        },
        &rates,
    )
    .unwrap();
    assert!(!bad.ok());
}

/// Masses chosen so three items are exactly 1.0 each; the other two lumps are set per case.
/// Values are what Python 3.12 prints for the same rates: only a compensated `sum()` gets them.
fn subtotal_with_lumps(cargo: f64, paint: f64) -> ovcd_core::pyint::PyInt {
    let inputs = BargeCostInputs {
        hull_steel_t: 0.0,
        wing_plate_t: 0.0,
        weather_shield_t: 0.0,
        tank_shell_t: 0.0,
        tank_supports_t: 0.0,
        insulation_t: 0.0,
        piping_machinery_t: 1.0,
        outfitting_t: 1.0,
        tug_hull_steel_t: 0.0,
        battery_kwh: 0.0,
        propulsion_kw: 0.0,
    };
    let mut rates = example_rates().unwrap();
    rates.hull_steel_rate = 1.0;
    rates.tank_shell_rate = 1.0;
    rates.tank_supports_rate = 1.0;
    rates.insulation_rate = 1.0;
    rates.piping_rate = 1.0;
    rates.outfitting_rate = 1.0;
    rates.cargo_system_lump = cargo;
    rates.paint_anodes_misc = paint;
    rates.battery_rate_per_kwh = 0.0;
    rates.propulsion_rate_per_kw = 0.0;
    rates.wheelhouse_accom_lump = 0.0;
    rates.tug_outfitting_lump = 0.0;
    rates.yard_margin_fraction = 0.0;
    rates.delivery_lump = 0.0;
    compute(&inputs, &rates).unwrap().barge_subtotal
}

#[test]
fn the_subtotal_is_a_compensated_sum_as_in_python() {
    assert_eq!(subtotal_with_lumps(1e16, 1.0), 10_000_000_000_000_004);
    assert_eq!(subtotal_with_lumps(1.0, 1e16), 10_000_000_000_000_004);
    assert_eq!(subtotal_with_lumps(1e16, -1e16), 2);
    assert_eq!(subtotal_with_lumps(-1e16, 1.0), -9_999_999_999_999_996);
}

#[test]
fn the_compensation_from_a_late_big_item_is_kept() {
    // Items in order: 0, 0, 0, 0.5, 0.5, 1e16, 0.5, 1.5. Python 3.12 gives 10000000000000004.
    let inputs = BargeCostInputs {
        hull_steel_t: 0.0,
        wing_plate_t: 0.0,
        weather_shield_t: 0.0,
        tank_shell_t: 0.0,
        tank_supports_t: 0.0,
        insulation_t: 0.5,
        piping_machinery_t: 1e16,
        outfitting_t: 0.5,
        tug_hull_steel_t: 0.0,
        battery_kwh: 0.0,
        propulsion_kw: 0.0,
    };
    let mut rates = example_rates().unwrap();
    rates.hull_steel_rate = 1.0;
    rates.tank_shell_rate = 1.0;
    rates.tank_supports_rate = 1.0;
    rates.insulation_rate = 1.0;
    rates.piping_rate = 1.0;
    rates.outfitting_rate = 1.0;
    rates.cargo_system_lump = 0.5;
    rates.paint_anodes_misc = 1.5;
    assert_eq!(
        compute(&inputs, &rates).unwrap().barge_subtotal,
        10_000_000_000_000_004
    );
}

fn rates_with(name: &str, value: &str) -> BargeCostRates {
    let text = EXAMPLE_RATES_TEXT
        .lines()
        .map(|l| {
            if l.starts_with(name) {
                format!("{name}: {value}")
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    text_rates(&text).unwrap()
}

#[test]
fn an_infinite_or_nan_figure_fails_as_python_round_does() {
    let inp = BargeCostInputs::default();
    let inf = compute(&inp, &rates_with("hull_steel_rate", "inf")).unwrap_err();
    assert_eq!(inf.type_name(), "OverflowError");
    assert_eq!(inf.message(), "cannot convert float infinity to integer");
    let nan = compute(&inp, &rates_with("hull_steel_rate", "nan")).unwrap_err();
    assert_eq!(nan.type_name(), "ValueError");
    assert_eq!(nan.message(), "cannot convert float NaN to integer");
    // A mass so large the product overflows to infinity.
    let huge = BargeCostInputs {
        hull_steel_t: 1e308,
        ..BargeCostInputs::default()
    };
    assert_eq!(
        compute(&huge, &example_rates().unwrap())
            .unwrap_err()
            .type_name(),
        "OverflowError"
    );
}

#[test]
fn a_result_past_the_i64_range_is_the_exact_python_int() {
    // Digits printed by the original Python 3.12 for the same inputs.
    let big = rates_with("hull_steel_rate", "1e30");
    let r = compute(&BargeCostInputs::default(), &big).unwrap();
    assert_eq!(r.unit_total.as_str(), "504460000000000010464469310242816");
    let big_mass = BargeCostInputs {
        hull_steel_t: 1e16,
        ..BargeCostInputs::default()
    };
    let r = compute(&big_mass, &example_rates().unwrap()).unwrap();
    assert_eq!(r.unit_total.as_str(), "27500000000005480448");
}

#[test]
fn round_is_half_to_even_and_exact_at_any_size() {
    use ovcd_core::pyint::{RoundError, round};
    assert_eq!(round(0.5).unwrap(), 0);
    assert_eq!(round(1.5).unwrap(), 2);
    assert_eq!(round(2.5).unwrap(), 2);
    assert_eq!(round(-2.5).unwrap(), -2);
    assert_eq!(round(9.0e18).unwrap().as_str(), "9000000000000000000");
    assert_eq!(round(9.3e18).unwrap().as_str(), "9300000000000000000");
    assert_eq!(round(-1e19).unwrap().as_str(), "-10000000000000000000");
    assert_eq!(
        round(1180591620717411303424.0).unwrap().as_str(),
        "1180591620717411303424"
    );
    assert_eq!(
        round(f64::MAX).unwrap().as_str(),
        "179769313486231570814527423731704356798070567525844996598917476803157260780028538760589558632766878171540458953514382464234321326889464182768467546703537516986049910576551282076245490090389328944075868508455133942304583236903222948165808559332123348274797826204144723168738177180919299881250404026184124858368"
    );
    assert_eq!(round(f64::NAN), Err(RoundError::Nan));
    assert_eq!(round(f64::INFINITY), Err(RoundError::Infinity));
    assert_eq!(round(f64::NEG_INFINITY), Err(RoundError::Infinity));
}
