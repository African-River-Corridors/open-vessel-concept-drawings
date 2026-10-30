//! Seam: the public `ovd_core::barge_capacity` API. Expected values are printed by the original
//! Python (`core/barge_capacity.py`), not worked out by hand. JSON cannot carry a NaN or an
//! infinity as an input, so these tests, not the JSON seam, reach those paths.
//! Label: snapshot
//! A value re-pinned after an agreed change is what the code printed then; the old value is named next to it.

use ovd_core::barge_capacity::{BargeCapacityInputs, compute, tanks_wide_for_beam};

#[test]
fn the_quickstart_propane_barge_matches_python() {
    // Python capped a tank at 12 m; the cap is now an input, 16 m by default (issue #175), so the
    // case that Python printed sets the old value.
    let mut i = BargeCapacityInputs::new(90.0, 14.8, 6.5);
    i.cargo_type = "LPG_propane".to_string();
    i.max_tank_od_m = 12.0;
    let r = compute(&i).unwrap();
    // Re-pinned (issue #237); was 1156.0677139049296 under the stepped IGC_2G. Re-pinned again (issue
    // #280, the end space defaults to the type G rule, 0.20 m, not 0.60 m aft and 0.50 m forward);
    // was 1405.8516419707348.
    assert_eq!(r.cargo_capacity_t, 1408.66852832557);
    // Re-pinned (issue #237); was 2.3053150177001953.
    // Re-pinned (#448); was 2.574557304382324.
    // Re-pinned again (issue #280); was 2.4156399922106404.
    assert_eq!(r.full_load_draft_m, 2.4187877904714594);
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
    // Python answers 83333333333333329126323388416; the JSON seam stops at 64 bits.
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

#[test]
fn a_hull_too_short_for_its_stern_block_is_not_ok() {
    // Python's `.ok` is False: the geometry check fails, and compute returns it, not an error.
    let mut i = BargeCapacityInputs::new(30.0, 14.8, 6.5);
    i.cargo_type = "LNG_ethane".to_string();
    let r = compute(&i).unwrap();
    assert!(!r.ok());
    assert!(r.checks.iter().any(|c| !c.ok));
}
