//! Issue #722: the hull steel built up from structural members (`ovd_core::hull_panels`).
//! Label: sourced - ABS Rules for Building and Classing Steel Barges 2021 (3-2-1/3.1, 3-2-2/3.5.1,
//! 3-2-3/1.1.2, 3-2-6/5.1, 3-2-7/3.1, 3-2-4/3.11), each value worked out by hand from the printed
//! formula in the test that uses it; the other tests are properties.

use ovd_core::barge_capacity::{BargeCapacityInputs, compute};
use ovd_core::hull_panels::{
    DoubleHullSpec, ScantlingChoices, Section, double_hull_panels, gas_hull_spec,
    required_section_modulus, tank_boundary_plate_mm, watertight_bulkhead_plate_mm,
    web_thickness_mm,
};

fn spec(depth: f64) -> DoubleHullSpec {
    DoubleHullSpec {
        loa_m: 140.0,
        beam_m: 29.0,
        depth_m: depth,
        block_coefficient: 0.9,
        cargo_zone_aft_m: 22.0,
        cargo_zone_fwd_m: 120.0,
        double_bottom_m: 1.5,
        wing_width_m: 2.4,
        wing_top_m: 8.0,
        deck_opening_width_m: 0.0,
        wing_deck_opening_width_m: 0.0,
        holds_along: 2,
        tanks_abreast: 2,
        steel_density: 7.85,
    }
}

#[test]
fn smr_is_the_formula_of_3_2_1_3_1() {
    // C1 = 10.75 - ((300 - 140)/100)^1.5 = 8.726128; SMR = C1 x 0.01 x 140^2 x 29 x (0.9 + 0.7).
    let c1 = 10.75 - libm::pow(1.6, 1.5);
    let hand = c1 * 0.01 * 140.0 * 140.0 * 29.0 * 1.6;
    assert!((required_section_modulus(140.0, 29.0, 0.9) - hand).abs() < 1e-9);
    assert!((hand - 79_359.0).abs() < 0.1, "{hand}");
}

#[test]
fn tank_boundary_plate_is_3_2_7_3_1() {
    // s = 700 mm, a > 2 (k = 1), h = 10 m: 700 x sqrt(10) / 254 + 2.5 = 11.215 mm.
    let t = tank_boundary_plate_mm(700.0, 5.0, 10.0);
    assert!((t - (700.0 * 10.0_f64.sqrt() / 254.0 + 2.5)).abs() < 1e-12);
    assert!((t - 11.215).abs() < 1e-3);
    // The minimum governs at a small head: max(6.5, 700/150 + 2.5 = 7.167).
    assert!((tank_boundary_plate_mm(700.0, 5.0, 0.1) - (700.0 / 150.0 + 2.5)).abs() < 1e-12);
}

#[test]
fn watertight_bulkhead_plate_is_3_2_6_5_1() {
    // 700 x sqrt(9) / 290 + 1.5 = 8.741 mm.
    let t = watertight_bulkhead_plate_mm(700.0, 5.0, 9.0);
    assert!((t - (2100.0 / 290.0 + 1.5)).abs() < 1e-12);
    // At no head the minimum 700/200 + 2.5 = 6.0 holds.
    assert!((watertight_bulkhead_plate_mm(700.0, 5.0, 0.0) - 6.0).abs() < 1e-12);
}

#[test]
fn web_thickness_is_3_2_4_3_11() {
    // L = 140 m in a tank: L/36 + 7.0 = 10.889 mm governs a 300 mm deep web (0.01 x 300 + 3 = 6).
    assert!((web_thickness_mm(140.0, 300.0, true) - (140.0 / 36.0 + 7.0)).abs() < 1e-12);
    // The depth term caps at 11.5 mm.
    assert!((web_thickness_mm(140.0, 2000.0, true) - 11.5).abs() < 1e-12);
}

#[test]
fn bottom_and_deck_plates_follow_3_2_2_and_3_2_3() {
    let m = double_hull_panels(&spec(14.0), &ScantlingChoices::default());
    let get = |n: &str| m.members.iter().find(|p| p.member == n).unwrap();
    let hg = m.hull_girder;
    // Bottom, L > 123 m: 0.08 x 140 + 0.007 x 700 - 2.5 = 13.6 mm, ordered to 14.0 (no girder addition).
    assert_eq!(hg.bottom_added_mm, 0.0);
    assert_eq!(get("bottom shell, midship 0.4L").plate_mm, 14.0);
    // Deck, 1.1.2(b): 700 x (140 + 48.76) / (26 x 140 + 8681) = 10.725 mm; (a) 0.009 x 700 + 2.4 = 8.7.
    let t = 700.0 * (140.0 + 48.76) / (26.0 * 140.0 + 8681.0);
    let ordered = ((t + hg.deck_added_mm) * 2.0).ceil() / 2.0;
    assert_eq!(get("strength deck, midship 0.4L").plate_mm, ordered);
}

#[test]
fn the_weight_is_the_sum_of_the_records_plus_the_named_allowance() {
    let m = double_hull_panels(&spec(12.0), &ScantlingChoices::default());
    let sum: f64 = m.members.iter().map(|p| p.total_t).sum();
    assert!((m.net_t - sum).abs() < 1e-9);
    for p in &m.members {
        assert!(
            (p.total_t - (p.plate_t + p.stiffeners_t)).abs() < 1e-9,
            "{}",
            p.member
        );
        assert!(
            (p.plate_t - p.plate_area_m2 * p.plate_mm / 1000.0 * 7.85).abs() < 1e-9,
            "{}",
            p.member
        );
    }
    assert!((m.allowance_t - m.net_t * m.allowance_pct / 100.0).abs() < 1e-9);
    assert!((m.total_t - (m.net_t + m.allowance_t)).abs() < 1e-9);
}

#[test]
fn the_double_hull_members_are_all_there() {
    let m = double_hull_panels(&spec(12.0), &ScantlingChoices::default());
    for name in [
        "bottom shell, midship 0.4L",
        "inner bottom",
        "side shell, midship 0.4L",
        "inner side (longitudinal bulkhead)",
        "strength deck, midship 0.4L",
        "transverse bulkheads",
        "centre girder",
        "side girders",
        "solid floors",
        "wing web frames",
        "deck transverses",
    ] {
        let p = m.members.iter().find(|p| p.member == name);
        assert!(p.is_some_and(|p| p.total_t > 0.0), "{name}");
    }
}

#[test]
fn every_record_cites_a_rule_and_says_its_basis() {
    let m = double_hull_panels(&spec(12.0), &ScantlingChoices::default());
    for p in &m.members {
        assert!(p.rule.starts_with("ABS "), "{}: {}", p.member, p.rule);
        assert!(p.basis == "rule" || p.basis == "estimate");
        assert!(
            p.x_m.0 <= p.x_m.1 && p.y_m.0 <= p.y_m.1 && p.z_m.0 <= p.z_m.1,
            "{}",
            p.member
        );
    }
}

#[test]
fn a_shallower_hull_needs_a_thicker_deck_or_bottom_for_the_same_smr() {
    // ABS 3-2-1/3.1 has no depth term: the shallower hull has less lever arm, so it adds plate.
    let deep = double_hull_panels(&spec(14.0), &ScantlingChoices::default()).hull_girder;
    let shallow = double_hull_panels(&spec(9.0), &ScantlingChoices::default()).hull_girder;
    assert_eq!(deep.smr_cm2m, shallow.smr_cm2m);
    assert!(
        shallow.deck_added_mm + shallow.bottom_added_mm > deep.deck_added_mm + deep.bottom_added_mm
    );
    assert!(shallow.sm_deck_cm2m >= shallow.smr_cm2m && shallow.sm_bottom_cm2m >= shallow.smr_cm2m);
}

#[test]
fn a_section_modulus_with_plate_is_the_hand_value() {
    // FB 200x14 on 10 mm x 700 mm plate. Areas: plate 7000 mm2 at 5, web 2800 at 110. NA = (35000 +
    // 308000)/9800 = 35.0 mm. I = 700x1000/12 + 7000x30^2 + 14x200^3/12 + 2800x75^2 = 31,441,667 mm4.
    // SM to the web top (175 mm from NA) = 179.67 cm3.
    let s = Section {
        name: "FB 200x14".into(),
        web_mm: (200.0, 14.0),
        flange_mm: (0.0, 0.0),
    };
    let sm = s.sm_with_plate_cm3(10.0, 700.0);
    assert!((sm - 31_441_666.67 / 175.0 / 1000.0).abs() < 1e-3, "{sm}");
}

#[test]
fn a_gas_hull_gets_a_panel_model_from_its_arrangement() {
    let mut i = BargeCapacityInputs::new(110.0, 11.4, 6.0);
    i.cargo_type = "LPG_propane".into();
    let r = compute(&i).unwrap();
    let b = r.gas_basis.as_ref().unwrap();
    let s = gas_hull_spec(&b.inputs, &b.tanks, 0.9);
    assert_eq!(s.loa_m, 110.0);
    assert_eq!(s.double_bottom_m, b.inputs.double_bottom_height.unwrap());
    let m = double_hull_panels(&s, &ScantlingChoices::default());
    assert!(m.total_t > 0.0 && m.total_t.is_finite());
}

/// Issue #722 (maintainer decision): the panel steel drives a gas hull's lightship and cost; the
/// structure factor of 2.0 is retired. Property tests: no fixed number.
mod the_switch {
    use super::*;

    fn gas() -> ovd_core::barge_capacity::BargeCapacityResult {
        let mut i = BargeCapacityInputs::new(110.0, 11.4, 6.0);
        i.cargo_type = "LPG_propane".into();
        compute(&i).unwrap()
    }

    fn panel_of(b: &ovd_core::barge_capacity::GasBasis) -> ovd_core::hull_panels::HullPanelModel {
        let s = gas_hull_spec(&b.inputs, &b.tanks, b.inputs.block_coefficient);
        double_hull_panels(&s, &ScantlingChoices::default())
    }

    #[test]
    fn the_hull_steel_is_the_panel_model() {
        let r = gas();
        let b = r.gas_basis.as_ref().unwrap();
        let m = panel_of(b);
        assert_eq!(b.weights.hull_steel_total, m.total_t);
        let parts = b.weights.hull_bottom_plating
            + b.weights.hull_side_plating
            + b.weights.hull_deck_plating
            + b.weights.hull_rake_plating
            + b.weights.hull_internal_structure;
        assert!((parts - m.total_t).abs() < 1e-9);
    }

    #[test]
    fn the_lightship_takes_the_panel_steel_not_the_top_down_rate() {
        let r = gas();
        let w = &r.gas_basis.as_ref().unwrap().weights;
        let steel = w.hull_steel_total;
        assert!((w.piping_machinery - 0.04 * (steel + w.tank_total)).abs() < 1e-9);
        assert!((w.outfitting - 0.035 * (steel + w.tank_total)).abs() < 1e-9);
        // The top-down figure is still reported, as a comparison only.
        assert_eq!(w.steel_top_down, 0.10 * 110.0 * 11.4 * 6.0);
        let b = r.gas_basis.as_ref().unwrap();
        let cover: f64 = ovd_core::weights::covering_items(&b.inputs, &b.tanks)
            .unwrap()
            .iter()
            .chain(r.dome_items.iter())
            .map(|i| i.mass_t)
            .sum();
        let sum = steel + w.tank_total + w.insulation + w.piping_machinery + w.outfitting + cover;
        assert!((w.lightship - sum).abs() < 1e-9, "{} vs {sum}", w.lightship);
        assert_eq!(r.lightship_t, w.lightship);
    }

    #[test]
    fn the_cost_prices_only_the_covering_above_d_beside_the_members() {
        let r = gas();
        let b = r.gas_basis.as_ref().unwrap();
        let cover: f64 = ovd_core::weights::covering_items(&b.inputs, &b.tanks)
            .unwrap()
            .iter()
            .map(|i| i.mass_t)
            .sum();
        assert!((b.weights.wing_plate + b.weights.weather_shield - cover).abs() < 1e-9);
    }
}
