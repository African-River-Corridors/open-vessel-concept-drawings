//! Seam: the public `ovcd_core::hull` API. Behaviour copied from `core/hull.py`, values taken
//! from the Reference Results of the frozen Python (`parity/reference/hull.json`).

use ovcd_core::hull::HullGeometry;
use ovcd_core::hull::hull_form::RakedBox;

fn quickstart() -> RakedBox {
    HullGeometry::new(90.0, 14.8, 6.5, 0.0, 6.5, 0.0, 6.5)
}

fn raked() -> RakedBox {
    HullGeometry::new(60.0, 10.0, 4.0, 6.0, 2.0, 4.0, 1.5)
}

#[test]
fn the_raked_box_is_the_hull_geometry() {
    // HullGeometry is the name the Python core uses; the raked box is the only Hull Form.
    let a: RakedBox = quickstart();
    let b: HullGeometry = a;
    assert_eq!(a, b);
}

#[test]
fn a_flat_box_matches_the_python_quickstart_result() {
    let h = quickstart();
    let d = 2.3053150177001953;
    assert_eq!(h.parallel_length(), 90.0);
    assert_eq!(h.displaced_volume(d), 3070.67960357666);
    assert_eq!(h.waterplane_area(d), 1332.0);
    assert_eq!(h.waterplane_inertia_transverse(d), 24313.440000000002);
    assert_eq!(h.waterplane_inertia_longitudinal(d), 899100.0);
    assert_eq!(h.KB(d), 1.1526575088500974);
    assert_eq!(h.LCB(d), 45.0);
    assert_eq!(h.BM_transverse(d), 7.917934509246827);
    assert_eq!(h.BM_longitudinal(d), 292.8016322356615);
    assert_eq!(h.total_hull_surface_area(), 3834.0);
}

#[test]
fn zero_and_negative_draft_give_the_python_fallbacks() {
    let h = raked();
    assert_eq!(h.displaced_volume(0.0), 0.0);
    assert_eq!(h.waterplane_area(-1.0), 0.0);
    assert_eq!(h.KB(0.0), 0.0);
    assert_eq!(h.LCB(0.0), 30.0);
    assert_eq!(h.BM_transverse(-1.0), 0.0);
    assert_eq!(h.BM_longitudinal(0.0), 0.0);
}

#[test]
fn a_raked_hull_has_a_partly_submerged_rake_below_the_rise() {
    let h = raked();
    // Draft 1.0 is below both rises: the rake volume is a triangle in profile.
    let v = h.displaced_volume(1.0);
    let expected =
        10.0 * 1.0 * 50.0 + 10.0 * 1.0f64 * 6.0 / (2.0 * 2.0) + 10.0 * 1.0f64 * 4.0 / (2.0 * 1.5);
    assert!((v - expected).abs() < 1e-9, "{v} vs {expected}");
    assert!(h.LCB(1.0) > 0.0 && h.LCB(1.0) < 60.0);
}

#[test]
fn plating_areas_use_the_slope_length() {
    let h = raked();
    assert_eq!(h.bow_rake_area(), (6.0f64 * 6.0 + 2.0 * 2.0).sqrt() * 10.0);
    assert_eq!(
        h.stern_rake_area(),
        0.0 + (4.0f64 * 4.0 + 1.5 * 1.5).sqrt() * 10.0
    );
    assert_eq!(
        HullGeometry::new(10.0, 2.0, 1.0, 0.0, 1.0, 0.0, 1.0).bow_rake_area(),
        0.0
    );
}
