//! Seam: the tank dome rule (issue #316), an estimate on two published tanks.
//!
//! Dome diameter = the larger of 1.2 m and 0.17 x the steel outer diameter; dome height above the
//! tank = the dome diameter. The points: a dimensioned dome 2,500 mm in diameter whose top is
//! 2,500 mm above the inside of the shell, on a lobe of radius 7,300 mm (Heo, Kang and Park 2017,
//! Fig. 19; 2.5 / 14.6 = 0.171), and the 1,200 mm valve dome of a 4.73 m river LPG tank
//! (Senjanovic et al., DESIGN 2006). By hand: 0.17 x 14.6 = 2.482 m; on 4.73 m, 0.17 x 4.73 =
//! 0.804 m, so 1.2 m holds; the rule leaves 1.2 m at 7.06 m (1.2 / 0.17).
//!
//! Label: sourced - Heo, Kang and Park 2017, Consideration for IMO Type C independent tank rule scantling process, Fig. 19 (2,500 mm dome on R7300); Senjanovic et al., DESIGN 2006 (1,200 mm valve dome, 4,730 mm tank); the rule values are hand calculations (above).

use ovcd_core::barge_capacity::{
    BargeCapacityInputs, DOME_RULE_DIAMETER_RATIO, DOME_RULE_MIN_DIAMETER_M, compute,
    dome_dimensions, dome_volume_m3,
};

#[test]
fn the_rule_gives_the_two_published_points() {
    let (d, h) = dome_dimensions(14.6, None, None);
    assert!((d - 2.482).abs() < 1e-9, "{d}");
    assert_eq!(h, d);
    // Within 1 % of the published 2,500 mm.
    assert!((d - 2.5).abs() / 2.5 < 0.01);
    assert_eq!(dome_dimensions(4.73, None, None), (1.2, 1.2));
    assert_eq!(
        (DOME_RULE_MIN_DIAMETER_M, DOME_RULE_DIAMETER_RATIO),
        (1.2, 0.17)
    );
    let knee = 1.2 / 0.17;
    assert!((dome_dimensions(knee, None, None).0 - 1.2).abs() < 1e-12);
}

#[test]
fn a_typed_size_is_used_as_it_is_and_the_height_follows_the_diameter_in_force() {
    assert_eq!(dome_dimensions(10.0, Some(2.0), Some(2.0)), (2.0, 2.0));
    assert_eq!(dome_dimensions(10.0, Some(3.0), None), (3.0, 3.0));
    let (d, h) = dome_dimensions(10.0, None, Some(0.5));
    assert!((d - 1.7).abs() < 1e-12);
    assert_eq!(h, 0.5);
}

/// A gas hull, LPG, the old fixed dome or the rule.
fn hull(by_rule: bool) -> BargeCapacityInputs {
    let mut i = BargeCapacityInputs::new(110.0, 11.4, 6.0);
    i.cargo_type = "LPG_propane".into();
    i.dome_diameter_by_rule = by_rule;
    i.dome_height_by_rule = by_rule;
    i
}

#[test]
fn the_rule_is_the_same_as_typing_the_dome_it_gives() {
    let rule = compute(&hull(true)).unwrap();
    let basis = rule.gas_basis.as_ref().unwrap();
    let steel = basis.tanks.outer_diameter - 2.0 * basis.inputs.insulation_thickness / 1000.0;
    let (d, h) = dome_dimensions(steel, None, None);
    assert_eq!(basis.inputs.dome_height, h);
    assert!(h < 2.0, "{h}: smaller than the old 2.0 m on this hull");
    let mut typed = hull(false);
    typed.dome_diameter = d;
    typed.dome_height = h;
    // The same design to the settling of the clearance loop (1e-9 on d).
    let same = compute(&typed).unwrap();
    for (a, b) in [
        (same.cargo_capacity_t, rule.cargo_capacity_t),
        (
            same.air_draft_top_above_keel_m,
            rule.air_draft_top_above_keel_m,
        ),
        (same.lightship_t, rule.lightship_t),
    ] {
        assert!((a - b).abs() < 1e-6, "{a} {b}");
    }
    // And it differs from the old fixed 2.0 m dome: the tank top is lower.
    let fixed = compute(&hull(false)).unwrap();
    assert!(rule.air_draft_top_above_keel_m < fixed.air_draft_top_above_keel_m);
    assert!(dome_volume_m3(d, h) < dome_volume_m3(2.0, 2.0));
}
