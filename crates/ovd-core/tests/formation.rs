//! The geometry and the sizing loop of a pushed Formation (issue #359).
//!
//! Label: property

use ovd_core::formation::*;

#[test]
fn the_pusher_ratios_give_their_reference_size_at_1400_kw() {
    // The ratios are 19.5 m and 9.5 m at the reference power, 2.0 m draught.
    assert!((pusher_length_m(1400.0) - 19.5).abs() < 1e-12);
    assert!((pusher_beam_m(1400.0) - 9.5).abs() < 1e-12);
    assert_eq!(pusher_draught_m(None), 2.0);
    assert_eq!(pusher_draught_m(Some(1.6)), 1.6);
    assert_eq!(pusher_draught_m(Some(3.0)), 2.0);
    // A bigger pusher is longer and wider.
    assert!(pusher_length_m(2800.0) > pusher_length_m(1400.0));
    assert!(pusher_beam_m(2800.0) > pusher_beam_m(1400.0));
}

#[test]
fn the_equivalent_hull_of_a_two_by_three_convoy() {
    let tug = TugDims {
        length_m: 20.0,
        beam_m: 10.0,
        draught_m: 2.0,
    };
    let h = pushed_equivalent_hull((76.5, 11.4, 3.2), 2500.0, (2, 3), &tug, 400.0);
    assert_eq!(h.length_m, 3.0 * 76.5 + 20.0);
    assert_eq!(h.beam_m, 22.8);
    assert_eq!(h.draught_m, 3.2);
    assert_eq!(h.displaced_volume_m3, 6.0 * 2500.0 + 400.0);
    let cb = h.displaced_volume_m3 / (h.length_m * h.beam_m * h.draught_m);
    assert_eq!(h.block_coefficient, cb);
}

#[test]
fn the_coupling_force_is_the_es_trin_formula() {
    // F = 270 x 1,400 x 20 / 9.5 x 1e-3 kN.
    let f = coupling_force_kn(1400.0, 20.0, 9.5);
    assert!((f - 270.0 * 1400.0 * 20.0 / 9.5 * 1e-3).abs() < 1e-9);
}

#[test]
fn the_loop_stops_when_the_power_moves_less_than_one_percent() {
    // A contraction to 1,000 kW: each pass cuts the distance to a tenth.
    let s = size_loop(2000.0, |p| 1000.0 + (p - 1000.0) / 10.0);
    assert!(s.converged);
    assert!(s.passes <= MAX_PASSES);
    assert!((s.power_kw - 1000.0).abs() / 1000.0 < 0.05);
    // A power that does not move stops at the first pass.
    let s = size_loop(1500.0, |p| p);
    assert_eq!((s.passes, s.converged), (1, true));
}

#[test]
fn the_loop_stops_at_five_passes_when_it_does_not_converge() {
    let mut calls = 0;
    let s = size_loop(100.0, |p| {
        calls += 1;
        p * 2.0
    });
    assert_eq!(calls, MAX_PASSES);
    assert_eq!(s.passes, MAX_PASSES);
    assert!(!s.converged);
}

#[test]
fn the_towing_tug_ratios_give_their_reference_size_at_3000_kw() {
    // 23 m at the reference power; the beam is the length over 2.5; the draught 0.42 of the beam.
    assert!((towing_tug_length_m(3000.0) - 23.0).abs() < 1e-12);
    assert!((towing_tug_beam_m(23.0) - 9.2).abs() < 1e-12);
    assert!((towing_tug_draught_m(9.2) - 0.42 * 9.2).abs() < 1e-12);
    assert!(towing_tug_length_m(4500.0) > towing_tug_length_m(3000.0));
}

#[test]
fn the_resistance_of_a_tow_is_the_sum_of_the_parts_times_the_allowance() {
    // One barge: barge + tug. Three barges: 3 x barge + tug. The allowance multiplies the sum.
    assert_eq!(tow_resistance_kn(40.0, 1, 10.0, 1.0), 50.0);
    assert_eq!(tow_resistance_kn(40.0, 3, 10.0, 1.0), 130.0);
    assert!((tow_resistance_kn(40.0, 1, 10.0, 1.2) - 60.0).abs() < 1e-12);
}
