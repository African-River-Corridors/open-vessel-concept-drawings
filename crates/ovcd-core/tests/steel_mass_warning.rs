//! Issue #404 part B: a warning when the top-down hull steel (lightship) and the bottom-up hull steel
//! (cost) differ by more than 20 % of the bottom-up steel. The 20 % is an Estimate.
//! Label: property

use ovcd_core::barge_capacity::{STEEL_MASS_DISAGREEMENT_WARN_PCT, steel_mass_disagreement};
use ovcd_core::hull::HullGeometry;
use ovcd_core::models::{BargeInputs, WeightBreakdown};
use ovcd_core::tanks::calculate_tank_arrangement_wide;
use ovcd_core::weights::calculate_weights;

fn weights() -> WeightBreakdown {
    let mut i = BargeInputs::new(90.0, 14.8, 6.5, 6.5, 65.0);
    i.bow_rake_rise = Some(6.5);
    i.stern_rake_rise = Some(6.5);
    i.bow_non_tank_length = Some(10.9);
    i.stern_non_tank_length = Some(21.892000000000003);
    i.n_tanks_wide = 2;
    i.n_tanks_long = 2;
    i.double_bottom_height = Some(0.5);
    i.post_init().unwrap();
    let hull = HullGeometry::new(i.loa, i.beam, i.depth, i.bow_rake_length, 6.5, i.stern_rake_length, 6.5);
    let (tanks, _) = calculate_tank_arrangement_wide(&i).unwrap();
    calculate_weights(&i, &hull, &tanks).unwrap()
}

/// The weights with the top-down steel `pct` % above the bottom-up steel.
fn with_gap(pct: f64) -> WeightBreakdown {
    let mut w = weights();
    w.steel_bottom_up = 500.0;
    w.steel_top_down = 500.0 * (1.0 + pct / 100.0);
    w.steel_delta = w.steel_top_down - w.steel_bottom_up;
    w
}

#[test]
fn the_limit_is_20_percent() {
    assert_eq!(STEEL_MASS_DISAGREEMENT_WARN_PCT, 20.0);
}

#[test]
fn no_warning_at_or_below_the_limit_on_either_sign() {
    for pct in [0.0, 19.9, 20.0, -19.9, -20.0] {
        assert!(steel_mass_disagreement(&with_gap(pct)).is_none(), "{pct}");
    }
}

#[test]
fn a_warning_above_the_limit_on_either_sign_with_both_figures_and_the_delta() {
    for pct in [20.1, 40.0, -20.1, -30.0] {
        let t = steel_mass_disagreement(&with_gap(pct)).unwrap_or_else(|| panic!("{pct}"));
        assert_eq!(t.msg.as_ref().unwrap().id, "warn-steel-mass-disagreement");
        let en = t.to_string();
        let w = with_gap(pct);
        assert!(en.contains(&format!("{:.1} t of hull steel (top-down)", w.steel_top_down)), "{en}");
        assert!(en.contains(&format!("prices {:.1} t (bottom-up)", w.steel_bottom_up)), "{en}");
        assert!(en.contains(&format!("differ by {:.1} t", w.steel_delta)), "{en}");
        assert!(en.contains("20 % limit (an Estimate)"), "{en}");
    }
}

#[test]
fn no_warning_when_a_mass_is_not_a_finite_number_or_the_bottom_up_steel_is_zero() {
    let mut w = with_gap(40.0);
    w.steel_bottom_up = f64::INFINITY;
    assert!(steel_mass_disagreement(&w).is_none());
    let mut w = with_gap(40.0);
    w.steel_bottom_up = 0.0;
    assert!(steel_mass_disagreement(&w).is_none());
    let mut w = with_gap(40.0);
    w.steel_delta = f64::NAN;
    assert!(steel_mass_disagreement(&w).is_none());
}
