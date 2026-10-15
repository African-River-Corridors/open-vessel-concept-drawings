//! Seam: the public `ovcd_core::design_grid` and `ovcd_core::ga_adapter` API. Expected values are
//! the original Python's, not worked out by hand. JSON cannot carry a NaN or an infinity as an
//! input, so these tests, not the JSON seam, reach those paths.

use ovcd_core::barge_capacity::{BargeCapacityInputs, compute};
use ovcd_core::design_grid::sweep;
use ovcd_core::ga_adapter::{is_gas, to_ga};

#[test]
fn every_cell_has_two_tanks_along_the_hull() {
    let g = sweep(&[60.0, 100.0, 140.0], &[10.0, 14.0], 6.0, "LPG_propane").unwrap();
    assert!(g.cells.iter().all(|c| c.n_tanks_long == 2));
}

#[test]
fn a_tank_is_not_capped_at_28_m() {
    // Issue #175: tank length is limited only by length over diameter and the hull.
    let g = sweep(&[100.0], &[14.0], 6.0, "LPG_propane").unwrap();
    let c = &g.cells[0];
    assert!(c.feasible, "{}", c.reason);
    assert!(c.tank_length_m.unwrap() > 28.0, "{:?}", c.tank_length_m);
    assert!(c.slenderness.unwrap() <= 7.0);
}

#[test]
fn a_cell_whose_tank_is_longer_than_seven_diameters_is_infeasible() {
    let g = sweep(&[200.0], &[11.0], 6.0, "LPG_propane").unwrap();
    let c = &g.cells[0];
    assert!(!c.feasible);
    assert!(c.reason.starts_with("tank length "), "{}", c.reason);
    assert!(c.reason.contains("<= 7"), "{}", c.reason);
}

#[test]
fn no_feasible_cell_has_a_slenderness_above_seven() {
    let lengths: Vec<f64> = (4..=20).map(|i| f64::from(i) * 10.0).collect();
    let beams = [6.0, 8.0, 10.0, 12.0, 16.0, 20.0, 28.0];
    let g = sweep(&lengths, &beams, 6.0, "LPG_propane").unwrap();
    let feasible: Vec<_> = g.cells.iter().filter(|c| c.feasible).collect();
    assert!(feasible.len() > 20);
    assert!(
        g.cells
            .iter()
            .any(|c| !c.feasible && c.reason.contains("<= 7"))
    );
    for c in feasible {
        assert!(c.slenderness.unwrap() <= 7.0, "{c:?}");
    }
}

#[test]
fn the_curve_is_the_feasible_cells_of_one_length_in_beam_order() {
    let g = sweep(&[45.0, 60.0], &[8.0, 12.0, 16.0], 6.0, "LPG_propane").unwrap();
    assert_eq!(g.cells.len(), 6);
    assert!(g.curve(45.0).is_empty());
    let beams: Vec<f64> = g.curve(60.0).iter().map(|c| c.beam_m).collect();
    // The 16 m beam was feasible with the old 12 m tank cap (two tanks across). With the 16 m cap
    // it is one 14 m tank across, and two tanks along do not fit in a 60 m hull: "tank heads don't fit".
    assert_eq!(beams, [8.0, 12.0]);
    assert!(g.cells[5].reason.starts_with("Cylinder length is -1.00m"));
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
    // Python capped a tank at 12 m; the cap is now an input (issue #175).
    let mut i = BargeCapacityInputs::new(100.0, 16.0, 6.0);
    i.max_tank_od_m = 12.0;
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
