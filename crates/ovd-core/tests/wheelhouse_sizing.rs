//! Seam: `ovd_core::wheelhouse::assess`, the lift sized to the Blind Distance limit (issue #448 on
//! #354). The solve meets the limit exactly, so rounding could leave the blind distance a hair above
//! it; the sized lift must keep it at or below the limit, with no tolerance.
//!
//! Label: property. Over a sweep of draughts, trims, cargo tops and eye positions, a lift that is not
//! capped gives a Blind Distance at most the limit, and the reported value is the true one.

use ovd_core::wheelhouse::{Obstruction, Waterline, assess, blind_distance};

#[test]
fn a_lift_that_is_not_capped_keeps_the_blind_distance_at_or_below_the_limit() {
    let limit = 250.0;
    let mut cases = 0;
    for i in 0..40 {
        let draught = 1.0 + 0.0731 * f64::from(i);
        for j in 0..25 {
            let trim = -0.6 + 0.05 * f64::from(j);
            let w = Waterline {
                draught_aft_m: draught - trim / 2.0,
                draught_fwd_m: draught + trim / 2.0,
                length_m: 76.5 + 1.3 * f64::from(j),
            };
            let bow = w.length_m;
            let top = Obstruction {
                label: "cargo top",
                x_m: bow - 3.0 - 0.17 * f64::from(i),
                top_above_keel_m: 6.0 + 0.113 * f64::from(j),
            };
            let eye_x = 4.0 + 0.07 * f64::from(i);
            let s = assess(eye_x, draught + 2.0, 1.7, 1e3, &[top], bow, &w, limit);
            assert!(s.lift_used_m < 1e3, "not capped");
            assert!(
                s.blind_distance_m <= limit,
                "draught {draught}, trim {trim}: {} > {limit}",
                s.blind_distance_m
            );
            assert_eq!(
                s.blind_distance_m,
                blind_distance(eye_x, s.eye_raised_m, &[top], bow, &w),
                "the reported value is the true one"
            );
            assert!(s.enough);
            cases += 1;
        }
    }
    assert_eq!(cases, 1000);
}
