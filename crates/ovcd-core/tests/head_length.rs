//! Seam: the length of an insulated tank head (issue #411). A 2:1 semi-ellipsoidal steel head is
//! R_s/2 deep, R_s the shell outer radius, and the insulation adds its full thickness t_ins at the
//! crown: the head is R_s/2 + t_ins long. A hemisphere is R_s + t_ins, the outer radius.
//!
//! Label: sourced - hand calculation (below): head depth of a 2:1 semi-ellipsoid (minor axis R/2)
//! plus the insulation thickness at the crown.

use ovcd_core::models::BargeInputs;
use ovcd_core::tanks::calculate_tank_arrangement;

/// The quickstart barge of `tests/tanks.rs`: OD 5.9 m, insulation 350 mm, 25 mm walls, two tanks
/// along a 90 - 10.9 - 21.892 = 57.208 m tank zone with a 2 m gap: each tank 27.604 m overall.
fn insulated(head: &str) -> BargeInputs {
    let mut i = BargeInputs::new(90.0, 14.8, 6.5, 6.5, 65.0);
    i.bow_rake_rise = Some(6.5);
    i.stern_rake_rise = Some(6.5);
    i.bow_non_tank_length = Some(10.9);
    i.stern_non_tank_length = Some(21.892000000000003);
    i.n_tanks_wide = 2;
    i.n_tanks_long = 2;
    i.double_bottom_height = Some(0.5);
    i.tank_head_type = head.into();
    i.post_init().unwrap();
    i
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn an_insulated_2_to_1_head_is_half_the_shell_radius_plus_the_insulation() {
    let i = insulated("ellipsoidal_2to1");
    assert_eq!(i.insulation_thickness, 350.0);
    let t = calculate_tank_arrangement(&i).unwrap();
    // Hand case: OD 5.9 m, t_ins 0.35 m, so R_s = (5.9 - 2 x 0.35) / 2 = 2.6 m.
    // Head = 2.6 / 2 + 0.35 = 1.65 m (the old (R_s + t_ins) / 2 gave 1.475 m).
    // Overall 27.604 m; cylinder = 27.604 - 2 x 1.65 = 24.304 m.
    assert!(
        close(t.overall_tank_length, 27.604),
        "{}",
        t.overall_tank_length
    );
    assert!(close(t.head_length, 1.65), "{}", t.head_length);
    assert!(close(t.cylinder_length, 24.304), "{}", t.cylinder_length);
}

#[test]
fn an_uninsulated_2_to_1_head_is_half_the_radius() {
    let mut i = insulated("ellipsoidal_2to1");
    i.insulation_thickness = 0.0;
    let t = calculate_tank_arrangement(&i).unwrap();
    // R_s = 5.9 / 2 = 2.95 m; head = 1.475 m; cylinder = 27.604 - 2.95 = 24.654 m.
    assert!(close(t.head_length, 1.475), "{}", t.head_length);
    assert!(close(t.cylinder_length, 24.654), "{}", t.cylinder_length);
}

#[test]
fn a_hemisphere_head_is_the_outer_radius() {
    let t = calculate_tank_arrangement(&insulated("hemisphere")).unwrap();
    // R_s + t_ins = 2.6 + 0.35 = 2.95 m = OD / 2.
    assert!(close(t.head_length, 2.95), "{}", t.head_length);
    assert!(
        close(t.cylinder_length, 27.604 - 5.9),
        "{}",
        t.cylinder_length
    );
}
