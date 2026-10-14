//! Seam: the public `ovcd_core::barge_capacity` API. Expected values are printed by the frozen
//! Python (`core/barge_capacity.py`), not worked out by hand. JSON cannot carry a NaN or an
//! infinity as an input, so the parity cases cannot reach those paths: these tests do.

use ovcd_core::barge_capacity::{BargeCapacityInputs, compute, tanks_wide_for_beam};

#[test]
fn the_quickstart_propane_barge_matches_python() {
    let mut i = BargeCapacityInputs::new(90.0, 14.8, 6.5);
    i.cargo_type = "LPG_propane".to_string();
    let r = compute(&i).unwrap();
    assert_eq!(r.cargo_capacity_t, 1156.0677139049296);
    assert_eq!(r.full_load_draft_m, 2.3053150177001953);
    assert_eq!(r.n_tanks_wide, Some(2));
    assert!(r.ok());
}

#[test]
fn a_nan_beam_is_python_s_value_error_from_ceil() {
    let e = tanks_wide_for_beam(f64::NAN, 1.0, 0.0, 12.0).unwrap_err();
    assert_eq!(e.kind, "ValueError");
    assert_eq!(e.message, "cannot convert float NaN to integer");
}

#[test]
fn an_infinite_beam_is_python_s_overflow_error_from_ceil() {
    let e = tanks_wide_for_beam(f64::INFINITY, 1.0, 0.0, 12.0).unwrap_err();
    assert_eq!(e.kind, "OverflowError");
    assert_eq!(e.message, "cannot convert float infinity to integer");
}

#[test]
fn a_nan_tank_cap_is_a_value_error() {
    let e = tanks_wide_for_beam(30.0, 1.0, 0.0, f64::NAN).unwrap_err();
    assert_eq!(e.kind, "ValueError");
}

#[test]
fn an_infinite_tank_cap_needs_one_tank() {
    assert_eq!(
        tanks_wide_for_beam(30.0, 1.0, 0.0, f64::INFINITY).unwrap(),
        1
    );
}

#[test]
fn a_count_no_64_bit_field_holds_is_the_fixed_overflow_error() {
    // Python answers 83333333333333329126323388416; the parity format stops at 64 bits.
    let e = tanks_wide_for_beam(1e30, 1.0, 0.0, 12.0).unwrap_err();
    assert_eq!(e.kind, "OverflowError");
    assert_eq!(e.message, "integer does not fit in 64 bits");
}

#[test]
fn a_narrow_hull_needs_one_tank_and_the_cap_sets_the_rest() {
    assert_eq!(tanks_wide_for_beam(2.0, 1.0, 0.0, 12.0).unwrap(), 1);
    assert_eq!(tanks_wide_for_beam(14.8, 1.0, 1.0, 12.0).unwrap(), 2);
    assert_eq!(tanks_wide_for_beam(32.4, 1.0, 0.0, 12.0).unwrap(), 3);
}
