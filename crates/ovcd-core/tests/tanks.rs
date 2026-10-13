//! Seam: the public `ovcd_core::tanks` API. Expected values are printed by the frozen Python
//! (`core/tanks.py`), not worked out by hand.

use ovcd_core::models::BargeInputs;
use ovcd_core::tanks::{
    INT_OVERFLOW_MESSAGE, calculate_tank_arrangement, free_surface_moment_cylindrical,
};

/// The quickstart barge: 90 x 14.8 m, two tanks abreast, two rows, fixed 25 mm walls.
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

#[test]
fn the_quickstart_arrangement_matches_python() {
    let t = calculate_tank_arrangement(&quickstart()).unwrap();
    assert_eq!((t.n_tanks_wide, t.n_tanks_long, t.n_tanks_total), (2, 2, 4));
    assert_eq!(t.outer_diameter, 5.9);
    assert_eq!(t.inner_diameter, 5.15);
    assert_eq!(t.overall_tank_length, 27.603999999999996);
    assert_eq!(t.cylinder_length, 21.703999999999994);
    assert_eq!(t.head_length, 2.95);
    assert_eq!(t.volume_per_tank, 523.6288223140365);
    assert_eq!(t.volume_cylinder, 452.1100074061362);
    assert_eq!(t.volume_heads, 71.5188149079003);
    assert_eq!(t.total_tank_volume, 2094.515289256146);
    assert_eq!(t.cargo_volume, 1989.7895247933386);
    assert_eq!(t.tank_shell_weight, 343.67825740993516);
    assert_eq!(t.insulation_weight, 42.97869150096367);
    assert_eq!(t.tank_top_above_keel, 6.9);
    assert_eq!(
        (t.cylinder_wall_thickness, t.head_wall_thickness),
        (25.0, 25.0)
    );
    assert_eq!(t.design_pressure, 0.5);
    assert_eq!(t.tank_material, "");
}

#[test]
fn an_automatic_wall_names_the_material_and_uses_the_pressure_vessel_formula() {
    let mut i = quickstart();
    i.tank_wall_thickness = None;
    i.post_init().unwrap();
    let t = calculate_tank_arrangement(&i).unwrap();
    assert_eq!(
        (t.cylinder_wall_thickness, t.head_wall_thickness),
        (7.5, 5.0)
    );
    assert_eq!(t.tank_material, "9Ni");
    assert_eq!(t.inner_diameter, 5.1850000000000005);
    assert_eq!(t.tank_shell_weight, 96.71601494722185);
}

#[test]
fn an_ellipsoidal_head_is_half_as_long_and_uses_its_own_area_constant() {
    let mut i = quickstart();
    i.tank_head_type = "ellipsoidal_2to1".into();
    i.tank_wall_thickness = None;
    i.post_init().unwrap();
    let t = calculate_tank_arrangement(&i).unwrap();
    assert_eq!(t.head_length, 1.475);
    assert_eq!(t.cylinder_length, 24.653999999999996);
    assert_eq!(t.volume_heads, 36.49344885686704);
    assert_eq!(t.tank_shell_weight, 105.55486138152804);
    assert_eq!(t.insulation_weight, 43.36450948944139);
}

#[test]
fn the_error_messages_print_floats_the_python_way() {
    let mut i = quickstart();
    i.side_clearance = Some(7.5);
    let e = calculate_tank_arrangement(&i).unwrap_err();
    assert_eq!(e.kind, "ValueError");
    assert_eq!(
        e.message,
        "Tank outer diameter is -0.60m - not enough beam. Beam=14.8m, side_clearance=7.5m, \
         n_wide=2, gap_transverse=1.0m"
    );

    let mut i = quickstart();
    i.tank_head_type = "flat".into();
    assert_eq!(
        calculate_tank_arrangement(&i).unwrap_err().message,
        "Unknown tank head type: flat"
    );

    // With an automatic wall the pressure vessel formula meets the head type first.
    i.tank_wall_thickness = None;
    assert_eq!(
        calculate_tank_arrangement(&i).unwrap_err().message,
        "Unknown head type: flat"
    );

    let mut i = quickstart();
    i.tank_wall_thickness = Some(2700.0);
    assert_eq!(
        calculate_tank_arrangement(&i).unwrap_err().message,
        "Tank inner radius is non-positive. Wall too thick for diameter."
    );
}

#[test]
fn a_tank_row_too_short_for_its_heads_is_an_error() {
    let mut i = quickstart();
    i.stern_non_tank_length = Some(70.0);
    let e = calculate_tank_arrangement(&i).unwrap_err();
    assert_eq!(e.kind, "ValueError");
    assert!(
        e.message
            .starts_with("Cylinder length is -2.35m - tank heads don't fit. Available length per tank=3.55m, head length=2.95m each"),
        "{}",
        e.message
    );
}

#[test]
fn a_zero_tank_count_divides_by_zero() {
    let mut i = quickstart();
    i.n_tanks_wide = 0;
    let e = calculate_tank_arrangement(&i).unwrap_err();
    assert_eq!(
        (e.kind, e.message.as_str()),
        ("ZeroDivisionError", "float division by zero")
    );
    let mut i = quickstart();
    i.n_tanks_long = 0;
    assert_eq!(
        calculate_tank_arrangement(&i).unwrap_err().kind,
        "ZeroDivisionError"
    );
}

#[test]
fn a_negative_tank_count_follows_python() {
    let mut i = quickstart();
    i.n_tanks_wide = -1;
    assert_eq!(
        calculate_tank_arrangement(&i).unwrap_err().message,
        "Tank outer diameter is -14.80m - not enough beam. Beam=14.8m, side_clearance=1.0m, \
         n_wide=-1, gap_transverse=1.0m"
    );
}

#[test]
fn a_count_product_beyond_64_bits_fails_loudly() {
    let mut i = quickstart();
    i.n_tanks_wide = i64::MAX;
    i.n_tanks_long = 2;
    let e = calculate_tank_arrangement(&i).unwrap_err();
    assert_eq!(
        (e.kind, e.message.as_str()),
        ("OverflowError", INT_OVERFLOW_MESSAGE)
    );
    let mut i = quickstart();
    i.n_tanks_wide = i64::MIN;
    i.n_tanks_long = 1;
    assert_eq!(
        calculate_tank_arrangement(&i).unwrap_err().kind,
        "OverflowError"
    );
    let mut i = quickstart();
    i.n_tanks_wide = 1;
    i.n_tanks_long = i64::MIN;
    assert_eq!(
        calculate_tank_arrangement(&i).unwrap_err().kind,
        "OverflowError"
    );
}

#[test]
fn a_field_post_init_would_fill_but_is_none_raises_the_python_type_error() {
    let mut i = quickstart();
    i.bow_non_tank_length = None;
    let e = calculate_tank_arrangement(&i).unwrap_err();
    assert_eq!(
        (e.kind, e.message.as_str()),
        (
            "TypeError",
            "unsupported operand type(s) for -: 'float' and 'NoneType'"
        )
    );
    let mut i = quickstart();
    i.stern_non_tank_length = None;
    assert_eq!(
        calculate_tank_arrangement(&i).unwrap_err().kind,
        "TypeError"
    );
}

#[test]
fn a_none_side_clearance_and_double_bottom_count_as_zero() {
    let mut i = quickstart();
    i.side_clearance = None;
    i.double_bottom_height = None;
    let t = calculate_tank_arrangement(&i).unwrap();
    // Beam 14.8 with no clearance: (14.8 - 1.0) / 2 = 6.9 m tanks; top = 0 + 0.5 + 6.9.
    assert_eq!(t.outer_diameter, 6.9);
    assert_eq!(t.tank_top_above_keel, 0.5 + 6.9);
}

#[test]
fn a_none_side_clearance_prints_none_in_the_beam_message() {
    let mut i = quickstart();
    i.side_clearance = None;
    i.beam = 1.0;
    let e = calculate_tank_arrangement(&i).unwrap_err();
    assert!(e.message.contains("side_clearance=Nonem"), "{}", e.message);
}

#[test]
fn free_surface_moment_matches_python() {
    assert_eq!(
        free_surface_moment_cylindrical(2.575, 21.703999999999994, 0.95, 0.581),
        11.887396690482028
    );
    assert_eq!(
        free_surface_moment_cylindrical(2.5, 10.0, 0.5, 0.5),
        52.08333333333333
    );
    assert_eq!(
        free_surface_moment_cylindrical(2.5, 10.0, 0.2, 0.5),
        26.666666666666664
    );
}

#[test]
fn an_empty_or_full_tank_has_no_free_surface() {
    assert_eq!(free_surface_moment_cylindrical(2.5, 10.0, 0.0, 0.5), 0.0);
    assert_eq!(free_surface_moment_cylindrical(2.5, 10.0, 1.0, 0.5), 0.0);
    assert_eq!(free_surface_moment_cylindrical(2.5, 10.0, 1.5, 0.5), 0.0);
}

#[test]
fn three_by_three_tanks_with_gaps_other_than_one_metre_match_python() {
    // A gap of 1.0 m hides a product that became a quotient; 0.7 and 1.3 do not.
    let mut i = quickstart();
    i.n_tanks_wide = 3;
    i.n_tanks_long = 3;
    i.tank_gap_transverse = 0.7;
    i.tank_gap_longitudinal = 1.3;
    i.insulation_thickness = 100.0;
    let t = calculate_tank_arrangement(&i).unwrap();
    assert_eq!(t.n_tanks_total, 9);
    assert_eq!(t.outer_diameter, 3.8000000000000003);
    assert_eq!(t.inner_diameter, 3.5500000000000003);
    assert_eq!(t.overall_tank_length, 18.202666666666662);
    assert_eq!(t.cylinder_length, 14.402666666666661);
    assert_eq!(t.head_length, 1.9000000000000001);
    assert_eq!(t.volume_per_tank, 165.98253188663855);
    assert_eq!(t.total_tank_volume, 1493.842786979747);
    assert_eq!(t.cargo_volume, 1419.1506476307595);
    assert_eq!(t.tank_shell_weight, 357.6200212150267);
    assert_eq!(t.insulation_weight, 11.734436667302626);
    assert_eq!(t.tank_top_above_keel, 4.800000000000001);
}
