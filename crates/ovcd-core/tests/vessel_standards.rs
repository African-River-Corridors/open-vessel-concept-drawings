//! Seam: the nominal-size helpers of `ovcd_core::vessel_standards::VesselStandard`. No parity entry
//! point reaches them, so these figures (from the frozen Python on the same classes) pin them.

use ovcd_core::vessel_standards::get_standard;

#[test]
fn nominal_sizes_are_the_midpoints_and_r_min_uses_the_long_end() {
    let iv = get_standard("CEMT_IV").unwrap();
    assert_eq!(iv.loa_nominal_m(), 82.5);
    assert_eq!(iv.draught_nominal_m(), 2.5);
    assert_eq!(iv.dwt_nominal_t(), 1250.0);
    assert_eq!(iv.r_min_m(4.0), 340.0);
    let va = get_standard("CEMT_Va").unwrap();
    assert_eq!(va.loa_nominal_m(), 102.5);
    assert_eq!(va.draught_nominal_m(), 3.5);
    assert_eq!(va.dwt_nominal_t(), 2250.0);
    assert_eq!(va.r_min_m(4.0), 440.0);
}
