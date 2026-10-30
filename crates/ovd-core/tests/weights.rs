//! Seam: the public `ovd_core::weights` API. Expected values are printed by the original Python
//! (`core/weights.py`), not worked out by hand.
//! Label: snapshot
//! A value re-pinned after an agreed change is what the code printed then; the reason is named next to it.

use ovd_core::hull::HullGeometry;
use ovd_core::models::{BargeInputs, WeightBreakdown};
use ovd_core::tanks::{calculate_tank_arrangement, calculate_tank_arrangement_wide};
use ovd_core::weights::calculate_weights;

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
fn weigh(i: &BargeInputs) -> WeightBreakdown {
    let hull = HullGeometry::new(
        i.loa,
        i.beam,
        i.depth,
        i.bow_rake_length,
        i.bow_rake_rise.unwrap(),
        i.stern_rake_length,
        i.stern_rake_rise.unwrap(),
    );
    let (tanks, _) = calculate_tank_arrangement_wide(i).unwrap();
    calculate_weights(i, &hull, &tanks).unwrap()
}

#[test]
fn the_quickstart_weights_match_python() {
    let w = weigh(&quickstart());
    assert_eq!(w.hull_bottom_plating, 125.47439999999999);
    assert_eq!(w.hull_side_plating, 91.845);
    assert_eq!(w.hull_deck_plating, 104.562);
    assert_eq!(w.hull_rake_plating, 0.0);
    assert_eq!(w.hull_internal_structure, 128.75255999999996);
    assert_eq!(w.hull_steel_total, 450.63395999999995);
    assert_eq!(w.tank_shells, 343.67825740993516);
    assert_eq!(w.tank_internals_supports, 103.10347722298056);
    assert_eq!(w.tank_total, 446.7817346329157);
    assert_eq!(w.insulation, 42.97869150096367);
    assert_eq!(w.wing_plate, 54.356667110341945);
    assert_eq!(w.weather_shield, 63.660289161599984);
    assert_eq!(w.piping_machinery, 52.503269385316635);
    assert_eq!(w.outfitting, 45.94036071215206);
    assert_eq!(w.stores, 0.0);
    assert_eq!(w.moulded_volume, 8658.0);
    assert_eq!(w.moulded_depth, 6.5);
    assert_eq!(w.steel_top_down, 865.8000000000001);
    assert_eq!(w.steel_bottom_up, 568.6509162719419);
    assert_eq!(w.steel_delta, 297.14908372805814);
    // Re-pinned (issue #410): the weather shield above D is in the lightship; was 1454.0040562313482.
    assert_eq!(w.lightship, 1506.7706007113482);
}

#[test]
fn raked_ends_add_rake_plating() {
    let mut i = quickstart();
    i.bow_rake_length = 4.0;
    i.bow_rake_rise = Some(2.0);
    i.stern_rake_length = 3.0;
    i.stern_rake_rise = Some(1.5);
    let w = weigh(&i);
    assert!(w.hull_rake_plating > 0.0);
    assert!(w.hull_bottom_plating < 125.47439999999999);
}

#[test]
fn no_weather_shield_weighs_nothing() {
    let mut i = quickstart();
    i.weather_shield_thickness = 0.0;
    let w = weigh(&i);
    assert_eq!(w.weather_shield, 0.0);
    assert_eq!(w.wing_plate, 54.356667110341945);
    assert_eq!(w.steel_bottom_up, 504.9906271103419);
}

#[test]
fn no_wing_tank_means_no_wing_plate_or_shield_and_the_hull_depth_rules() {
    let mut i = quickstart();
    i.wing_tank_width = Some(0.0);
    i.depth = 3.0;
    let w = weigh(&i);
    assert_eq!((w.wing_plate, w.weather_shield), (0.0, 0.0));
    assert_eq!(w.moulded_depth, 3.0);
    assert_eq!(w.moulded_volume, 3996.0);
    assert_eq!(w.lightship, 952.8390562313481);
}

#[test]
fn a_full_height_wing_tank_has_no_plate_and_the_hull_is_weighed_to_d() {
    let mut i = quickstart();
    i.wing_tank_height_ratio = 1.0;
    let w = weigh(&i);
    assert_eq!((w.wing_plate, w.weather_shield), (0.0, 0.0));
    // Re-pinned (issue #410): the hull steel is weighed to D, not to the wing tank top; was 6.9,
    // 9190.800000000001 and 1511.280056231348.
    assert_eq!(w.moulded_depth, 6.5);
    assert_eq!(w.moulded_volume, 8658.0);
    assert_eq!(w.lightship, 1454.0040562313482);
}

#[test]
fn a_wing_plate_above_the_deck_adds_a_side_extension() {
    let mut i = quickstart();
    i.wing_tank_height_ratio = 0.9;
    i.depth = 3.0;
    let w = weigh(&i);
    assert_eq!(w.wing_plate, 119.53723776000002);
    assert_eq!(w.weather_shield, 54.055343664);
    // Re-pinned (issue #410): the hull is weighed to D; was 6.3100000000000005.
    assert_eq!(w.moulded_depth, 3.0);
    assert_eq!(w.steel_bottom_up, 554.989541424);
}

#[test]
fn a_wing_plate_below_the_deck_has_no_side_extension() {
    let mut i = quickstart();
    i.n_tanks_wide = 1;
    i.side_clearance = Some(3.0);
    i.wing_tank_width = None;
    i.post_init().unwrap();
    let w = weigh(&i);
    assert_eq!(w.wing_plate, 71.47277479415334);
    // Re-pinned (issue #410): the hull is weighed to D; was 6.808000000000001.
    assert_eq!(w.moulded_depth, 6.5);
}

#[test]
fn a_none_double_bottom_counts_as_zero() {
    let mut i = quickstart();
    i.double_bottom_height = None;
    let w = weigh(&i);
    assert_eq!(w.wing_plate, 54.356667110341945);
    assert_eq!(w.moulded_depth, 6.5);
}

#[test]
fn an_unset_wing_tank_width_raises_the_python_type_error() {
    let i = quickstart();
    let hull = HullGeometry::new(90.0, 14.8, 6.5, 0.0, 6.5, 0.0, 6.5);
    let tanks = calculate_tank_arrangement(&i).unwrap();
    let mut bad = i.clone();
    bad.wing_tank_width = None;
    let e = calculate_weights(&bad, &hull, &tanks).unwrap_err();
    assert_eq!(
        (e.kind, e.message.as_str()),
        (
            "TypeError",
            "'>' not supported between instances of 'NoneType' and 'int'"
        )
    );
    // The same field is read again behind a zero shield thickness: still the first read.
    bad.weather_shield_thickness = 0.0;
    assert_eq!(
        calculate_weights(&bad, &hull, &tanks).unwrap_err().kind,
        "TypeError"
    );
}

#[test]
fn an_unset_non_tank_length_raises_the_python_type_error() {
    let i = quickstart();
    let hull = HullGeometry::new(90.0, 14.8, 6.5, 0.0, 6.5, 0.0, 6.5);
    let tanks = calculate_tank_arrangement(&i).unwrap();
    let mut bad = i.clone();
    bad.bow_non_tank_length = None;
    assert_eq!(
        calculate_weights(&bad, &hull, &tanks).unwrap_err().kind,
        "TypeError"
    );
    let mut bad = i;
    bad.stern_non_tank_length = None;
    assert_eq!(
        calculate_weights(&bad, &hull, &tanks).unwrap_err().kind,
        "TypeError"
    );
}

#[test]
fn the_shield_height_adds_the_double_bottom_to_the_support_height() {
    // Python (quickstart with a double bottom of 2**52): 67.01116704.
    // A difference in place of the sum gives 60.228013439999984, a product 6e-9 off at 1e9.
    let mut i = quickstart();
    i.double_bottom_height = Some(4503599627370496.0);
    assert_eq!(weigh(&i).weather_shield, 67.01116704);
    // Python at 1e9: 63.66028863761752.
    i.double_bottom_height = Some(1e9);
    assert_eq!(weigh(&i).weather_shield, 63.66028863761752);
}

#[test]
fn a_zero_shield_thickness_skips_the_shield_even_when_its_area_is_not_finite() {
    // Python: weather_shield 0.0 and steel_bottom_up inf. With `>=` the shield runs, inf * 0.0 is
    // NaN, and the shield and both steel totals turn NaN.
    let mut i = quickstart();
    i.beam = 1e120;
    i.n_tanks_wide = 1_000_000_000_000_000_000;
    i.tank_gap_transverse = 0.0;
    i.loa = 1e200;
    i.bow_non_tank_length = None;
    i.stern_non_tank_length = None;
    i.post_init().unwrap();
    i.weather_shield_thickness = 0.0;
    let w = weigh(&i);
    assert_eq!(w.weather_shield, 0.0);
    assert_eq!(w.steel_bottom_up, f64::INFINITY);
    assert!(w.steel_delta.is_nan());
}

#[test]
fn a_count_product_beyond_64_bits_does_not_stop_the_weights() {
    // Python answers: the weights carry no integer. 3037000500 ** 2 is just above i64::MAX.
    let mut i = quickstart();
    i.n_tanks_wide = 3_037_000_500;
    i.n_tanks_long = 3_037_000_500;
    i.beam = 1e12;
    i.loa = 1e12;
    i.bow_non_tank_length = Some(0.0);
    i.stern_non_tank_length = Some(0.0);
    i.tank_gap_transverse = 0.0;
    i.tank_gap_longitudinal = 0.0;
    let w = weigh(&i);
    assert!(w.lightship.is_finite());
}

#[test]
fn a_wing_plate_at_an_infinite_tank_radius_keeps_the_cutout_chord_at_zero() {
    // Python (quickstart with tank_gap_transverse = tank_gap_longitudinal = -1e308,
    // three tanks each way, wing_tank_height_ratio = -1.0): wing_plate 104.562. The tank diameter
    // is inf, so |dy| == tank_r == inf; with `<=` the chord is sqrt(inf - inf) = NaN.
    // Kills mutant crates/ovd-core/src/weights.rs:94 (`<` to `<=`).
    let mut i = quickstart();
    i.tank_gap_transverse = -1e308;
    i.n_tanks_wide = 3;
    i.tank_gap_longitudinal = -1e308;
    i.n_tanks_long = 3;
    i.wing_tank_height_ratio = -1.0;
    assert_eq!(weigh(&i).wing_plate, 104.562);
}
