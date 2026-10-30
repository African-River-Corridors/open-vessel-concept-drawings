//! Seam: `ovd_core::formation::convoy_sightline`, the pusher's view over its convoy (issue #360,
//! spec 0007 "Wheelhouse, sightline and air draft").
//!
//! The hand case is the eye height of the research note (sec. 9.2, derived), with x from the eye and
//! heights above the water: z_e,min = max over obstructions of z_o (x_b + B) / (x_b + B - x_o). A
//! pusher 20 m long with its eye at mid-length (x = 10 m) pushes two barges 76.5 m long in line. Each
//! barge's top is 4.0 m above the water light. The barge bows are 86.5 m and 163 m ahead of the eye;
//! the convoy bow is 163 m ahead; B = 250 m. The front barge governs: z_e = 4 x 413 / 250 = 6.608 m.
//!
//! Label: sourced - ES-TRIN 2025/1 Article 7.02(2) (250 m) and docs/research/stern-and-tugs.md sec. 9.2 (the formula, derived); the hand case is worked here.

use ovd_core::formation::{
    ConvoySight, PUSHER_WHEELHOUSE_AT, convoy_obstructions, convoy_sightline,
};

const PUSHER_L: f64 = 20.0;
const BARGE_L: f64 = 76.5;

fn sight(eye_lowered_m: f64, max_lift_m: f64) -> ConvoySight {
    ConvoySight {
        eye_x_m: PUSHER_WHEELHOUSE_AT * PUSHER_L,
        eye_lowered_m,
        eye_height_m: 1.5,
        max_lift_m,
        bow_x_m: PUSHER_L + 2.0 * BARGE_L,
    }
}

#[test]
fn a_two_long_convoy_gives_the_eye_height_of_the_hand_case() {
    let light = convoy_obstructions(PUSHER_L, BARGE_L, 2, 4.0);
    let laden = convoy_obstructions(PUSHER_L, BARGE_L, 2, 1.3);
    assert_eq!(light.len(), 2);
    assert_eq!(light[1].x_m, PUSHER_L + 2.0 * BARGE_L);
    let (s, laden_m) = convoy_sightline(&sight(2.5, 6.0), &light, &laden, 250.0);
    // The eye the rule needs, to 1 mm.
    assert!((s.eye_raised_m - 6.608).abs() < 1e-3, "{}", s.eye_raised_m);
    assert!(
        (s.lift_needed_m - 4.108).abs() < 1e-3,
        "{}",
        s.lift_needed_m
    );
    assert!(
        (s.floor_raised_m - 5.108).abs() < 1e-3,
        "{}",
        s.floor_raised_m
    );
    assert!(s.enough && s.blind_distance_m <= 250.0);
    assert!((s.blind_distance_m - 250.0).abs() < 1e-6);
    // Laden, from the same eye: 163 x 6.608 / (6.608 - 1.3) - 163 m.
    let hand = 163.0 * 6.608 / (6.608 - 1.3) - 163.0;
    assert!((laden_m - hand).abs() < 1e-2, "{laden_m} {hand}");
}

#[test]
fn a_fixed_wheelhouse_that_is_too_low_is_not_enough() {
    let light = convoy_obstructions(PUSHER_L, BARGE_L, 2, 4.0);
    let (s, _) = convoy_sightline(&sight(2.5, 0.0), &light, &light, 250.0);
    assert!(!s.enough);
    assert_eq!(s.lift_used_m, 0.0);
    // Eye 2.5 m over a top 4.0 m: the barges hide the water.
    assert!(s.blind_distance_m.is_infinite());
}

#[test]
fn a_longer_convoy_needs_more_lift() {
    let lift = |in_line: u64| {
        let obs = convoy_obstructions(PUSHER_L, BARGE_L, in_line, 4.0);
        let mut s = sight(2.5, 20.0);
        s.bow_x_m = PUSHER_L + in_line as f64 * BARGE_L;
        convoy_sightline(&s, &obs, &obs, 250.0).0.lift_needed_m
    };
    assert!(lift(1) < lift(2) && lift(2) < lift(3));
}
