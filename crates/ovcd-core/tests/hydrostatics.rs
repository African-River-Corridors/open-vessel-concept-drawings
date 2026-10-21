//! Seam: the public `ovcd_core::hydrostatics` API. Expected values are printed by the original
//! Python (`core/hydrostatics.py`), not worked out by hand.

use ovcd_core::hull::HullGeometry;
use ovcd_core::hydrostatics::{calculate_stability, calculate_trim, find_equilibrium_draft};
use ovcd_core::models::{BargeInputs, TankResult, WeightBreakdown};
use ovcd_core::tanks::calculate_tank_arrangement;
use ovcd_core::weights::calculate_weights;

fn quickstart() -> BargeInputs {
    let mut i = BargeInputs::new(90.0, 14.8, 6.5, 6.5, 65.0);
    i.bow_rake_rise = Some(6.5);
    i.stern_rake_rise = Some(6.5);
    i.bow_non_tank_length = Some(10.9);
    i.stern_non_tank_length = Some(21.892000000000003);
    i.n_tanks_wide = 2;
    i.n_tanks_long = 2;
    i.double_bottom_height = Some(0.5);
    i.post_init().unwrap();
    i
}

/// The chain: the hull carries the inputs' own rakes, then tanks, then weights.
fn chain(i: &BargeInputs) -> (HullGeometry, TankResult, WeightBreakdown) {
    let hull = HullGeometry::new(
        i.loa,
        i.beam,
        i.depth,
        i.bow_rake_length,
        i.bow_rake_rise.unwrap(),
        i.stern_rake_length,
        i.stern_rake_rise.unwrap(),
    );
    let tanks = calculate_tank_arrangement(i).unwrap();
    let weights = calculate_weights(i, &hull, &tanks).unwrap();
    (hull, tanks, weights)
}

fn box_hull(loa: f64, beam: f64, depth: f64) -> HullGeometry {
    HullGeometry::new(loa, beam, depth, 0.0, depth, 0.0, depth)
}

const CARGO_MASS: f64 = 1156.0677139049296;
const CARGO_DENSITY: f64 = 0.581;
const DRAFT: f64 = 2.3053150177001953;

#[test]
fn the_quickstart_laden_draft_matches_python() {
    let d = find_equilibrium_draft(
        &box_hull(90.0, 14.8, 6.5),
        2610.077663040161,
        1.0,
        6.5,
        0.85,
        0.001,
    );
    assert_eq!(d.unwrap(), DRAFT);
}

#[test]
fn the_calibration_draft_matches_python() {
    let d = find_equilibrium_draft(
        &box_hull(60.96, 10.67, 4.27),
        2463.85,
        1.0,
        4.27,
        1.0,
        0.001,
    );
    assert_eq!(d.unwrap(), 3.7879668045043937);
}

#[test]
fn an_overload_runs_to_the_upper_bound() {
    let d = find_equilibrium_draft(&box_hull(90.0, 14.8, 6.5), 20000.0, 1.0, 6.5, 1.0, 0.001);
    assert_eq!(d.unwrap(), 9.75);
}

#[test]
fn zero_weight_stops_at_the_first_close_enough_midpoint() {
    let d = find_equilibrium_draft(&box_hull(90.0, 14.8, 6.5), 0.0, 1.0, 6.5, 1.0, 0.001);
    assert_eq!(d.unwrap(), 9.298324584960938e-06);
}

#[test]
fn a_zero_density_or_block_coefficient_raises_zero_division() {
    for (rho, cb) in [(0.0, 1.0), (1.0, 0.0)] {
        let e = find_equilibrium_draft(&box_hull(90.0, 14.8, 6.5), 100.0, rho, 6.5, cb, 0.001)
            .unwrap_err();
        assert_eq!(
            (e.kind, e.message.as_str()),
            ("ZeroDivisionError", "float division by zero")
        );
    }
}

#[test]
fn a_nan_volume_never_converges_and_returns_the_last_midpoint() {
    let d = find_equilibrium_draft(&box_hull(90.0, f64::NAN, 6.5), 100.0, 1.0, 6.5, 1.0, 0.001);
    assert_eq!(d.unwrap(), 3.8456969129524326e-30);
}

#[test]
fn the_quickstart_stability_matches_python() {
    let i = quickstart();
    let (hull, tanks, weights) = chain(&i);
    let s = calculate_stability(
        &hull,
        &i,
        &tanks,
        &weights,
        CARGO_MASS,
        CARGO_DENSITY,
        DRAFT,
    )
    .unwrap();
    assert_eq!(s.KB, 1.1526575088500974);
    assert_eq!(s.BM_transverse, 7.917934509246827);
    assert_eq!(s.KG, 3.8835187629638415);
    assert_eq!(s.GM_solid, 5.1870732551330825);
    // Re-pinned (issue #413: the fill is a volume ratio, so the liquid height is the segment's);
    // were 0.01821769039107572 and 5.168855564742007, printed by the original Python.
    assert_eq!(s.free_surface_correction, 0.2199697268681742);
    assert_eq!(s.GM_fluid, 4.967103528264908);
    assert_eq!(s.BM_longitudinal, 292.8016322356615);
}

#[test]
fn a_zero_double_bottom_puts_the_stores_at_a_quarter_of_the_depth() {
    let mut i = quickstart();
    i.double_bottom_height = Some(0.0);
    let (hull, tanks, weights) = chain(&i);
    let s = calculate_stability(
        &hull,
        &i,
        &tanks,
        &weights,
        CARGO_MASS,
        CARGO_DENSITY,
        DRAFT,
    )
    .unwrap();
    assert_eq!(s.KG, 3.514763862574179);
    assert_eq!(s.GM_solid, 5.555828155522745);
    // Re-pinned (issue #413); was 5.537610465131669.
    assert_eq!(s.GM_fluid, 5.3358584286545705);
}

#[test]
fn an_unset_double_bottom_raises_the_python_type_error() {
    let i = quickstart();
    let (hull, tanks, weights) = chain(&i);
    let mut unset = i.clone();
    unset.double_bottom_height = None;
    let e = calculate_stability(
        &hull,
        &unset,
        &tanks,
        &weights,
        CARGO_MASS,
        CARGO_DENSITY,
        DRAFT,
    )
    .unwrap_err();
    assert_eq!(e.kind, "TypeError");
    assert_eq!(
        e.message,
        "unsupported operand type(s) for +: 'NoneType' and 'float'"
    );
}

#[test]
fn a_dry_hull_has_no_buoyancy_and_no_free_surface_correction() {
    let i = quickstart();
    let (hull, tanks, weights) = chain(&i);
    for draft in [0.0, -1.0] {
        let s = calculate_stability(
            &hull,
            &i,
            &tanks,
            &weights,
            CARGO_MASS,
            CARGO_DENSITY,
            draft,
        )
        .unwrap();
        assert_eq!(s.KB, 0.0);
        assert_eq!(s.BM_transverse, 0.0);
        assert_eq!(s.KG, 3.8835187629638415);
        assert_eq!(s.GM_solid, -3.8835187629638415);
        assert_eq!(s.free_surface_correction, 0.0);
        assert_eq!(s.GM_fluid, -3.8835187629638415);
        assert_eq!(s.BM_longitudinal, 0.0);
    }
}

#[test]
fn a_nan_draft_gives_nan_hydrostatics_and_no_free_surface_correction() {
    let i = quickstart();
    let (hull, tanks, weights) = chain(&i);
    let s = calculate_stability(
        &hull,
        &i,
        &tanks,
        &weights,
        CARGO_MASS,
        CARGO_DENSITY,
        f64::NAN,
    )
    .unwrap();
    assert!(s.KB.is_nan() && s.BM_transverse.is_nan() && s.GM_solid.is_nan());
    assert_eq!(s.KG, 3.8835187629638415);
    assert_eq!(s.free_surface_correction, 0.0);
}

#[test]
fn the_quickstart_trim_matches_python() {
    let i = quickstart();
    let (hull, tanks, weights) = chain(&i);
    let t = calculate_trim(&hull, &i, &tanks, &weights, CARGO_MASS, DRAFT).unwrap();
    assert_eq!(t.LCG, 48.30545666709862);
    assert_eq!(t.LCB, 45.0);
    assert_eq!(t.trim, 1.0160158526692904);
    assert_eq!(t.draft_aft, 1.7973070913655502);
    assert_eq!(t.draft_fwd, 2.8133229440348404);
}

#[test]
fn an_unset_non_tank_length_raises_the_python_type_error() {
    let i = quickstart();
    let (hull, tanks, weights) = chain(&i);
    let mut no_bow = i.clone();
    no_bow.bow_non_tank_length = None;
    let e = calculate_trim(&hull, &no_bow, &tanks, &weights, 1.0, 2.0).unwrap_err();
    assert_eq!(e.kind, "TypeError");
    assert_eq!(
        e.message,
        "unsupported operand type(s) for -: 'float' and 'NoneType'"
    );

    let mut no_stern = i.clone();
    no_stern.stern_non_tank_length = None;
    let e = calculate_trim(&hull, &no_stern, &tanks, &weights, 1.0, 2.0).unwrap_err();
    assert_eq!(e.kind, "TypeError");
    assert_eq!(
        e.message,
        "unsupported operand type(s) for +: 'NoneType' and 'float'"
    );
}

#[test]
fn a_dry_hull_has_no_trim() {
    let i = quickstart();
    let (hull, tanks, weights) = chain(&i);
    let t = calculate_trim(&hull, &i, &tanks, &weights, CARGO_MASS, 0.0).unwrap();
    assert_eq!(t.LCG, 48.30545666709862);
    assert_eq!(t.LCB, 45.0);
    assert_eq!(t.trim, 0.0);
    assert_eq!((t.draft_aft, t.draft_fwd), (0.0, 0.0));
}
