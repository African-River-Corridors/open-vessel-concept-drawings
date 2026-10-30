//! Seam: `ovd_core::gas_stability::box_condition`, the stability criteria of a box section for a
//! known KG (issue #184).
//!
//! Label: property. The GM is the closed form KB + BM - KG with KB = T/2 and BM = B^2/(12 T); the
//! rest are relations. The oracle that checks the numbers is `evaluation/oracle/box_oracle.py`.

use ovd_core::gas_stability::{PyError, box_condition, box_gz};

#[test]
fn the_gm_is_the_closed_form_and_the_free_surface_is_taken_off_it() {
    // Box B of the evaluation corpus: B 14, T 5, KG 5.5 gives KB 2.5, BM 196/60.
    let c = box_condition(14.0, 8.0, 5.0, 5.5, 0.0, "IS_CODE_2008").unwrap();
    let expected = 2.5 + 196.0 / 60.0 - 5.5;
    assert!((c.gm_solid_m - expected).abs() < 1e-12);
    assert_eq!(c.gm_fluid_m, c.gm_solid_m);
    let fs = box_condition(14.0, 8.0, 5.0, 5.5, 0.1, "IS_CODE_2008").unwrap();
    assert!((fs.gm_fluid_m - (expected - 0.1)).abs() < 1e-12);
    assert_eq!(fs.free_surface_m, 0.1);
}

#[test]
fn the_curve_is_the_box_gz_on_the_one_degree_grid_with_the_free_surface_taken_off() {
    let c = box_condition(14.0, 8.0, 5.0, 5.5, 0.05, "IS_CODE_2008").unwrap();
    assert_eq!(c.gz.len(), 61);
    for &(deg, gz) in &c.gz {
        let heel = box_gz(14.0, 8.0, 5.0, 5.5, f64::from(deg)).unwrap();
        let want = heel - 0.05 * libm::sin(f64::from(deg).to_radians());
        assert!((gz - want).abs() < 1e-12, "{deg}");
    }
}

#[test]
fn it_reports_the_six_criteria_of_the_is_code() {
    let c = box_condition(14.0, 8.0, 5.0, 5.5, 0.0, "IS_CODE_2008").unwrap();
    let names: Vec<&str> = c.criteria.iter().map(|x| x.name.as_str()).collect();
    assert_eq!(names.len(), 6);
    assert!(c.ok());
}

#[test]
fn a_draught_at_or_above_the_depth_and_an_unknown_criteria_set_are_refused() {
    assert!(matches!(
        box_condition(14.0, 5.0, 5.0, 3.0, 0.0, "IS_CODE_2008"),
        Err(PyError::Value(_))
    ));
    assert!(matches!(
        box_condition(14.0, 8.0, 5.0, 5.5, 0.0, "NOT_A_SET"),
        Err(PyError::Value(_))
    ));
}
