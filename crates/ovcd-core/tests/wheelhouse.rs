//! Seam: the wheelhouse sightline as a distance (issue #354, spec 0007 "Wheelhouse, sightline and
//! air draft").
//!
//! The hand case is the similar-triangles formula of the research note (sec. 9.2, derived): with x
//! from the eye and heights above the water, Blind Distance = x_o z_e / (z_e - z_o) - x_b. An eye
//! 8.0 m above the water, a cargo top 4.0 m above it 80.0 m ahead and the bow 100.0 m ahead: 80 x 8 /
//! 4 - 100 = 60.0 m. The limit is ES-TRIN 7.02(2), 250 m.
//!
//! Label: sourced - ES-TRIN 2025/1 Article 7.02(2) (250 m) and docs/research/stern-and-tugs.md sec. 9.2 (the formula, derived); the hand case is worked here.

use ovcd_core::wheelhouse::{Obstruction, Waterline, assess, blind_distance, eye_needed};

const DRAUGHT: f64 = 2.0;

fn cargo(x: f64, top_above_water: f64) -> Obstruction {
    Obstruction {
        label: "cargo top",
        x_m: x,
        top_above_keel_m: DRAUGHT + top_above_water,
    }
}

#[test]
fn the_blind_distance_is_the_similar_triangles_hand_case_to_1_mm() {
    // Eye at x = 10 m, 8.0 m above the water; cargo 80 m ahead of it, bow 100 m ahead.
    let w = Waterline::even(DRAUGHT, 110.0);
    let d = blind_distance(10.0, DRAUGHT + 8.0, &[cargo(90.0, 4.0)], 110.0, &w);
    assert!((d - 60.0).abs() < 1e-3, "{d}");
    // The worst obstruction governs; one below the sightline of another adds nothing.
    let both = [cargo(90.0, 4.0), cargo(50.0, 1.0)];
    assert!((blind_distance(10.0, DRAUGHT + 8.0, &both, 110.0, &w) - 60.0).abs() < 1e-3);
    // An obstruction as high as the eye hides the water.
    assert!(blind_distance(10.0, DRAUGHT + 4.0, &[cargo(90.0, 4.0)], 110.0, &w).is_infinite());
}

#[test]
fn a_trim_by_the_bow_shortens_the_blind_distance_and_by_the_stern_lengthens_it() {
    let even = Waterline::even(DRAUGHT, 110.0);
    let by_bow = Waterline {
        draught_aft_m: DRAUGHT - 0.5,
        draught_fwd_m: DRAUGHT + 0.5,
        length_m: 110.0,
    };
    let by_stern = Waterline {
        draught_aft_m: DRAUGHT + 0.5,
        draught_fwd_m: DRAUGHT - 0.5,
        length_m: 110.0,
    };
    // Obstruction heights fixed above the keel.
    let o = [cargo(90.0, 4.0)];
    let e = DRAUGHT + 8.0;
    let d = |w: &Waterline| blind_distance(10.0, e, &o, 110.0, w);
    assert!(d(&by_bow) < d(&even) && d(&even) < d(&by_stern));
}

#[test]
fn the_lift_needed_is_the_smallest_that_meets_250_m_and_too_high_an_obstruction_fails() {
    let w = Waterline::even(DRAUGHT, 110.0);
    let o = [cargo(90.0, 6.0)];
    // Lowered eye 7.0 m above the water: a long blind distance.
    let lowered = DRAUGHT + 7.0;
    assert!(blind_distance(10.0, lowered, &o, 110.0, &w) > 250.0);
    let s = assess(10.0, lowered, 1.65, 6.0, &o, 110.0, &w, 250.0);
    assert!(s.enough);
    assert!((s.blind_distance_m - 250.0).abs() < 1e-6, "{s:?}");
    // The smallest: 1 mm less lift misses the limit.
    assert!(blind_distance(10.0, s.eye_raised_m - 0.001, &o, 110.0, &w) > 250.0);
    assert!((s.floor_raised_m - (s.eye_raised_m - 1.65)).abs() < 1e-12);
    // No lift when the lowered eye already meets it.
    let low = [cargo(90.0, 1.0)];
    assert_eq!(
        assess(10.0, lowered, 1.65, 6.0, &low, 110.0, &w, 250.0).lift_used_m,
        0.0
    );
    // A cargo top too high for the column: the maximum lift is used, and it is not enough.
    let tall = [cargo(90.0, 14.0)];
    let s = assess(10.0, lowered, 1.65, 6.0, &tall, 110.0, &w, 250.0);
    assert!(!s.enough && s.lift_used_m == 6.0 && s.lift_needed_m > 6.0);
    // An obstruction beyond the point the eye must see can never be cleared.
    let far = [cargo(400.0, 1.0)];
    assert!(eye_needed(10.0, &far, 110.0, &w, 250.0).is_none());
}
