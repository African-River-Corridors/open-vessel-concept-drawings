//! Seam: the public `ovcd_core::design_grid` and `ovcd_core::ga_adapter` API. Expected values are
//! the original Python's, not worked out by hand. JSON cannot carry a NaN or an infinity as an
//! input, so these tests, not the JSON seam, reach those paths.

use ovcd_core::barge_capacity::{BargeCapacityInputs, compute};
use ovcd_core::design_grid::{MAX_TANK_LENGTH_M, n_tanks_for_length, sweep};
use ovcd_core::ga_adapter::{is_gas, to_ga};

#[test]
fn a_nan_length_is_python_s_value_error_from_ceil() {
    let e = n_tanks_for_length(f64::NAN, MAX_TANK_LENGTH_M).unwrap_err();
    assert_eq!(e.kind, "ValueError");
    assert_eq!(e.message, "cannot convert float NaN to integer");
}

#[test]
fn an_infinite_length_is_python_s_overflow_error_from_ceil() {
    let e = n_tanks_for_length(f64::INFINITY, MAX_TANK_LENGTH_M).unwrap_err();
    assert_eq!(e.kind, "OverflowError");
    assert_eq!(e.message, "cannot convert float infinity to integer");
}

#[test]
fn a_count_no_64_bit_field_holds_is_the_fixed_overflow_error() {
    let e = n_tanks_for_length(1e30, MAX_TANK_LENGTH_M).unwrap_err();
    assert_eq!(e.kind, "OverflowError");
    assert_eq!(e.message, "integer does not fit in 64 bits");
}

#[test]
fn a_zero_maximum_is_a_zero_division_error() {
    let e = n_tanks_for_length(60.0, 0.0).unwrap_err();
    assert_eq!(e.kind, "ZeroDivisionError");
    assert_eq!(e.message, "float division by zero");
}

#[test]
fn an_error_while_counting_tanks_ends_the_sweep() {
    // The count is worked out outside Python's `try`, so it is not an infeasible cell.
    let e = sweep(&[f64::NAN], &[12.0], 6.0, "LPG_propane").unwrap_err();
    assert_eq!(e.kind, "ValueError");
}

#[test]
fn the_curve_is_the_feasible_cells_of_one_length_in_beam_order() {
    let g = sweep(&[45.0, 60.0], &[8.0, 12.0, 16.0], 6.0, "LPG_propane").unwrap();
    assert_eq!(g.cells.len(), 6);
    assert!(g.curve(45.0).is_empty());
    let beams: Vec<f64> = g.curve(60.0).iter().map(|c| c.beam_m).collect();
    assert_eq!(beams, [8.0, 12.0, 16.0]);
    assert_eq!(g.curve(60.0)[1].cargo_t, Some(680.3055097251973));
    assert!(g.curve(75.0).is_empty());
}

#[test]
fn a_reason_keeps_the_first_80_characters() {
    let g = sweep(&[45.0], &[8.0], 6.0, "LPG_propane").unwrap();
    assert_eq!(g.cells[0].reason.chars().count(), 80);
    assert!(!g.cells[0].feasible);
}

#[test]
fn a_dry_bulk_result_has_no_tank_geometry_and_no_tank_count_across() {
    let mut i = BargeCapacityInputs::new(100.0, 16.0, 6.0);
    i.cargo_type = "bauxite".to_string();
    let r = compute(&i).unwrap();
    assert!(!is_gas(&r));
    let (gi, gr) = to_ga(&i, &r, None, true).unwrap();
    assert_eq!(gi.n_tanks_wide, None);
    assert!(gr.tanks.is_none());
    assert_eq!(gr.draft, 4.603855133056641);
    assert_eq!(gi.inputs.tug_length, 0.0);
}

#[test]
fn a_pushed_barge_with_a_channel_limited_draft_keeps_both() {
    let i = BargeCapacityInputs::new(100.0, 16.0, 6.0);
    let r = compute(&i).unwrap();
    let (gi, gr) = to_ga(&i, &r, Some(2.0), false).unwrap();
    assert_eq!(gi.inputs.max_draft, 2.0);
    assert_eq!(gr.draft, 2.0);
    assert_eq!(gr.freeboard, 4.0);
    assert_eq!(gi.inputs.tug_length, 25.0);
    assert_eq!(gr.tanks.unwrap().n_tanks_total, 4);
}

#[test]
fn a_range_rule_overflow_in_capacity_ends_the_sweep_loudly() {
    // Python holds the huge integer and returns a feasible cell; Rust cannot, so it must
    // fail with the fixed error, never turn it into an infeasible cell.
    let e = sweep(&[100.0], &[1e300], 6.0, "LPG_propane").unwrap_err();
    assert_eq!(e.kind, "OverflowError");
    assert_eq!(e.message, "integer does not fit in 64 bits");
}
