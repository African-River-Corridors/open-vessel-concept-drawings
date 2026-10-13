//! Seam: the public `ovcd_core::barge_cost` API. Figures from `core/barge_cost.py` run on the
//! worked case in the audited test (rates 1000 per tonne, margin 10%).

use ovcd_core::barge_cost::*;

fn text_rates(text: &str) -> Result<BargeCostRates, String> {
    let pairs = parse_flat_yaml(text)?;
    let mapping: Vec<_> = pairs
        .into_iter()
        .map(|(k, v)| (k, MappingValue::Text(v)))
        .collect();
    BargeCostRates::from_mapping(&mapping)
}

#[test]
fn the_example_rates_price_the_default_barge() {
    let result = compute(&BargeCostInputs::default(), &example_rates().unwrap());
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
