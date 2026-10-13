//! Seam: the public `ovcd_core::aft_arrangement` API. Expected values are printed by the frozen
//! Python (`core/aft_arrangement.py`), not worked out by hand.

use ovcd_core::aft_arrangement::{AftArrangementInputs, compute};

fn gas(beam: f64, depth: f64) -> AftArrangementInputs {
    AftArrangementInputs::new(beam, depth)
}

#[test]
fn defaults_match_the_python_dataclass() {
    let i = gas(14.8, 5.4);
    assert_eq!(i.n_thrusters, 3);
    assert_eq!(i.n_battery_units, 1);
    assert_eq!(i.n_accom_units, 2);
    assert_eq!(i.sightline_obstruction_m, None);
    assert_eq!(i.container_length_m, 12.192);
    assert_eq!(i.wheelhouse_max_lift_m, 6.0);
}

#[test]
fn golden_block_for_the_wide_gas_vessel() {
    let r = compute(&gas(14.8, 5.4)).unwrap();
    assert_eq!(r.length_m, 19.892000000000003);
    assert_eq!(r.container_per_row, 3);
    assert_eq!(r.container_rows, 1);
    assert_eq!(r.thruster_offsets_m, vec![-3.2, 0.0, 3.2]);
    assert_eq!(r.container_top_above_keel_m, 8.596);
    assert_eq!(r.block_top_lowered_m, 8.8);
    assert_eq!(
        r.container_labels,
        vec!["BATTERY 40 FT", "ACCOM 40 FT", "ACCOM 40 FT"]
    );
    assert_eq!(r.zones.len(), 3);
    assert_eq!(r.zones[1].0, "40 FT UNITS (3 ABREAST x 1)");
    assert_eq!(r.zones[1].1, 3.8);
    assert_eq!(r.zones[1].2, 15.992);
    assert!(r.ok());
    assert!(r.warnings.is_empty());
}

#[test]
fn a_narrow_beam_stacks_the_units_in_two_rows() {
    let r = compute(&gas(8.0, 5.0)).unwrap();
    assert_eq!((r.container_per_row, r.container_rows), (2, 2));
    assert_eq!(r.length_m, 32.884);
    assert_eq!(r.thruster_offsets_m, vec![-1.6, 1.6]);
    let by_name = |n: &str| r.checks.iter().find(|c| c.name == n).unwrap();
    let c = by_name("requested_thruster_count_fits");
    assert!(!c.ok);
    assert_eq!(c.severity, "warning");
    assert_eq!(
        c.message,
        "2 of 3 requested thrusters fit; 3 would need 10.20 m of 8.00 m beam"
    );
    assert_eq!(
        by_name("units_fit_in_one_row").message,
        "3 units in 2 row(s) of 2; a second row adds 12.99 m to the stern block \
         and comes straight off the cargo zone"
    );
    // Warnings do not gate the result.
    assert!(r.ok());
}

#[test]
fn four_units_abreast_and_a_lifted_wheelhouse() {
    let mut i = gas(14.8, 5.4);
    i.n_accom_units = 3;
    i.thruster_room_length_m = 5.0;
    i.sightline_obstruction_m = Some(12.0);
    let r = compute(&i).unwrap();
    assert_eq!(r.container_per_row, 4);
    assert_eq!(r.length_m, 21.392000000000003);
    assert_eq!(r.wheelhouse_lift_m, 5.6);
    assert_eq!(r.wheelhouse_floor_raised_m, 11.0);
    assert_eq!(r.wheelhouse_top_raised_m, 14.4);
    let last = r.checks.last().unwrap();
    assert_eq!(last.name, "wheelhouse_sees_over_cargo");
    assert!(last.ok);
    assert_eq!(
        last.message,
        "raised eye 12.50 m >= obstruction 12.00 m + 0.50 m margin (lift used 5.60 m of 6.00 m)"
    );
}

#[test]
fn a_column_too_short_for_the_sightline_warns() {
    let mut i = gas(14.8, 5.4);
    i.sightline_obstruction_m = Some(20.0);
    let r = compute(&i).unwrap();
    assert_eq!(r.wheelhouse_lift_m, 6.0);
    assert_eq!(r.wheelhouse_floor_raised_m, 11.4);
    assert_eq!(r.wheelhouse_top_raised_m, 14.8);
    assert!(!r.checks.last().unwrap().ok);
    assert_eq!(
        r.warnings,
        vec![
            "Sightline needs 13.60 m of lift; the column gives 6.00 m. \
             Raise the column or lower the cargo top."
        ]
    );
}

#[test]
fn an_obstruction_below_the_eye_needs_no_lift() {
    let mut i = gas(14.8, 5.4);
    i.sightline_obstruction_m = Some(3.0);
    let r = compute(&i).unwrap();
    assert_eq!(r.wheelhouse_lift_m, 0.0);
    assert!(r.wheelhouse_lift_m.is_sign_positive());
}

#[test]
fn a_failed_error_check_gates_the_result() {
    let mut i = gas(14.8, 5.4);
    i.wheelhouse_width_m = 20.0;
    let r = compute(&i).unwrap();
    assert!(!r.ok());
}

#[test]
fn invalid_inputs_raise_the_python_messages() {
    assert_eq!(
        compute(&gas(0.0, 5.4)).unwrap_err(),
        "beam and depth must be positive"
    );
    assert_eq!(
        compute(&gas(14.8, -1.0)).unwrap_err(),
        "beam and depth must be positive"
    );
    let mut i = gas(14.8, 5.4);
    i.n_thrusters = 0;
    assert_eq!(compute(&i).unwrap_err(), "n_thrusters must be >= 1");
}

/// Python `//` on floats is not `floor(a / b)`: `1.0 // 0.1` is 9.0 in Python, but `1.0 / 0.1`
/// rounds to 10.0. A port that floors the quotient puts ten units abreast, not nine.
#[test]
fn float_floor_division_is_not_floor_of_the_quotient() {
    let mut i = gas(1.0, 5.4);
    i.container_width_m = 0.1;
    i.container_side_gap_m = 0.0;
    i.n_battery_units = 20;
    i.n_accom_units = 0;
    let r = compute(&i).unwrap();
    assert_eq!((r.container_per_row, r.container_rows), (9, 3));
}

#[test]
fn a_hull_too_narrow_for_two_units_still_takes_one_per_row() {
    let mut i = gas(1.6, 5.4);
    i.container_width_m = 0.1;
    i.container_side_gap_m = 0.9;
    let r = compute(&i).unwrap();
    assert_eq!((r.container_per_row, r.container_rows), (1, 3));
}

/// `round()` is half-even in Python: 12.3444 / 0.3048 is exactly 40.5 and rounds to 40, while
/// 12.6492 / 0.3048 is 41.5 and rounds to 42.
#[test]
fn the_feet_label_rounds_half_to_even() {
    let mut i = gas(14.8, 5.4);
    i.container_length_m = 12.3444;
    let label = compute(&i).unwrap().zones[1].0.clone();
    assert!(label.starts_with("40 FT UNITS"), "{label}");
    i.container_length_m = 12.6492;
    let label = compute(&i).unwrap().zones[1].0.clone();
    assert!(label.starts_with("42 FT UNITS"), "{label}");
}

/// `max(0.0, x)` and `min(lift, max)` return the first of equal items. The two zeros are equal
/// but differ in sign, so a port that picks the second one shows `-0.0`. Python prints `0.0` for
/// both cases below.
#[test]
fn max_keeps_the_first_zero() {
    let mut i = gas(14.8, 5.4);
    i.wheelhouse_eye_above_floor_m = -5.4; // eye_lo is +0.0
    i.sightline_obstruction_m = Some(-0.0);
    i.sightline_margin_m = -0.0; // needed is -0.0, so needed - eye_lo is -0.0
    let r = compute(&i).unwrap();
    assert_eq!(r.wheelhouse_lift_m, 0.0);
    assert!(r.wheelhouse_lift_m.is_sign_positive());
}

#[test]
fn min_keeps_the_first_zero() {
    let mut i = gas(14.8, 5.4);
    i.wheelhouse_max_lift_m = -0.0; // min(0.0, -0.0)
    let r = compute(&i).unwrap();
    assert!(r.wheelhouse_lift_m.is_sign_positive());
}

/// The "column too short" warning has a 1e-9 slack. All values here are exact in binary, so the
/// lift lands on the threshold with no rounding. Python gives no warning in either case.
#[test]
fn a_lift_exactly_at_the_column_stroke_gives_no_warning() {
    let mut i = gas(14.8, 1.0);
    i.wheelhouse_eye_above_floor_m = 0.0;
    i.sightline_margin_m = 0.0;
    i.sightline_obstruction_m = Some(7.0); // lift is exactly 6.0, the default stroke
    let r = compute(&i).unwrap();
    assert_eq!(r.wheelhouse_lift_m, 6.0);
    assert!(r.warnings.is_empty());
    assert!(r.checks.last().unwrap().ok);
}

#[test]
fn a_lift_exactly_on_the_stroke_plus_the_slack_gives_no_warning() {
    let mut i = gas(14.8, 1e-9);
    i.wheelhouse_eye_above_floor_m = 0.0;
    i.sightline_margin_m = 0.0;
    i.sightline_obstruction_m = Some(2e-9); // lift is exactly 1e-9 = stroke (0.0) + 1e-9
    i.wheelhouse_max_lift_m = 0.0;
    let r = compute(&i).unwrap();
    assert!(r.warnings.is_empty());
}
