//! Seam: the public `ovcd_core::pressure_vessel` API. Expected numbers are what CPython 3.12
//! printed for the `core/pressure_vessel.py`.
//! Label: snapshot
//! The formula is ASME UG-27, but the expected numbers are what the code printed, not worked out from the paragraph.

use ovcd_core::pressure_vessel::*;

#[test]
fn material_database_keeps_pythons_values_and_order() {
    let keys: Vec<&str> = TANK_MATERIAL_DATABASE.iter().map(|(k, _)| *k).collect();
    // Issue #292 added S690QL1 after the three of the original engine.
    assert_eq!(keys, ["304L", "316L", "9Ni", "S690QL1"]);
    let m = get_tank_material("9Ni").unwrap();
    assert_eq!(m.name, "9% Nickel Steel");
    assert_eq!(
        (
            m.allowable_stress,
            m.density,
            m.min_thickness,
            m.corrosion_allowance
        ),
        (230.0, 7.85, 5.0, 0.5)
    );
    assert_eq!(
        m.description,
        "ASTM A553 Type I, LNG service to -196\u{b0}C"
    );
    let m = get_tank_material("304L").unwrap();
    assert_eq!(
        (
            m.allowable_stress,
            m.density,
            m.min_thickness,
            m.corrosion_allowance
        ),
        (113.0, 7.93, 3.0, 0.0)
    );
    let m = get_tank_material("316L").unwrap();
    assert_eq!(m.density, 7.98);
}

#[test]
fn unknown_material_fails_with_pythons_message() {
    let e = get_tank_material("A36").unwrap_err();
    assert_eq!(e.kind, "ValueError");
    assert_eq!(
        e.message,
        "Unknown tank material 'A36'. Valid: 304L, 316L, 9Ni, S690QL1"
    );
}

#[test]
fn asme_ug27_formulas() {
    let e = DEFAULT_WELD_EFFICIENCY;
    assert_eq!(e, 0.95);
    assert_eq!(
        cylinder_wall_thickness(0.5, 2500.0, 230.0, e).unwrap(),
        5.728689275893676
    );
    assert_eq!(
        hemisphere_head_thickness(0.5, 2500.0, 230.0, e).unwrap(),
        2.861066605630579
    );
    assert_eq!(
        ellipsoidal_head_thickness(0.5, 2500.0, 230.0, e).unwrap(),
        2.861066605630579
    );
    assert_eq!(
        cylinder_wall_thickness(0.5, 2500.0, 230.0, 1.0).unwrap(),
        5.44188071397475
    );
    assert_eq!(
        hemisphere_head_thickness(0.5, 2500.0, 230.0, 1.0).unwrap(),
        2.7179821700369646
    );
    assert_eq!(cylinder_wall_thickness(0.0, 2500.0, 230.0, e).unwrap(), 0.0);
    assert_eq!(
        hemisphere_head_thickness(0.0, 2500.0, 230.0, e).unwrap(),
        0.0
    );
    assert_eq!(
        ellipsoidal_head_thickness(0.0, 2500.0, 230.0, e).unwrap(),
        0.0
    );
}

#[test]
fn division_by_exactly_zero_raises_like_python() {
    // f * E == 0.6 * P exactly: 0.6 * 1.0 - 0.6 * 1.0 == 0.0.
    let e = cylinder_wall_thickness(1.0, 10.0, 0.6, 1.0).unwrap_err();
    assert_eq!(e.kind, "ZeroDivisionError");
    assert_eq!(e.message, "float division by zero");
    // 2 f E == 0.2 P: 2 * 0.1 * 1.0 - 0.2 * 1.0 == 0.0.
    assert_eq!(
        hemisphere_head_thickness(1.0, 10.0, 0.1, 1.0)
            .unwrap_err()
            .kind,
        "ZeroDivisionError"
    );
    assert_eq!(
        ellipsoidal_head_thickness(1.0, 10.0, 0.1, 1.0)
            .unwrap_err()
            .kind,
        "ZeroDivisionError"
    );
}

fn wall(d: f64, p: f64, m: &str, h: &str) -> (f64, f64) {
    calculate_wall_thicknesses(d, p, m, h, DEFAULT_WELD_EFFICIENCY).unwrap()
}

#[test]
fn wall_thicknesses_round_up_to_half_millimetre_and_clamp_to_the_minimum() {
    assert_eq!(wall(5.9, 0.5, "304L", "hemisphere"), (14.0, 7.0));
    assert_eq!(wall(5.9, 0.5, "304L", "ellipsoidal_2to1"), (14.0, 14.0));
    assert_eq!(wall(3.0, 1.0, "316L", "hemisphere"), (14.0, 7.0));
    assert_eq!(wall(5.9, 0.5, "9Ni", "hemisphere"), (7.5, 5.0));
    assert_eq!(wall(5.9, 0.5, "9Ni", "ellipsoidal_2to1"), (7.5, 7.5));
    assert_eq!(wall(3.0, 1.0, "9Ni", "ellipsoidal_2to1"), (7.5, 7.5));
    // Thin shells clamp to the material's IGC minimum.
    assert_eq!(wall(2.0, 0.1, "304L", "hemisphere"), (3.0, 3.0));
    assert_eq!(wall(0.01, 0.0, "316L", "ellipsoidal_2to1"), (3.0, 3.0));
    assert_eq!(wall(2.0, 0.1, "9Ni", "hemisphere"), (5.0, 5.0));
    // The weld efficiency argument is used.
    assert_eq!(
        calculate_wall_thicknesses(5.9, 0.5, "9Ni", "hemisphere", 0.85).unwrap(),
        (8.5, 5.0)
    );
}

#[test]
fn unknown_head_type_fails_after_the_cylinder_passes() {
    let e = calculate_wall_thicknesses(5.9, 0.5, "9Ni", "torispherical", 0.95).unwrap_err();
    assert_eq!(e.kind, "ValueError");
    assert_eq!(e.message, "Unknown head type: torispherical");
    // The material is looked up first.
    let e = calculate_wall_thicknesses(5.9, 0.5, "duplex", "torispherical", 0.95).unwrap_err();
    assert_eq!(
        e.message,
        "Unknown tank material 'duplex'. Valid: 304L, 316L, 9Ni, S690QL1"
    );
}

#[test]
fn rounding_raises_on_nan_and_infinity_like_math_ceil() {
    let e = calculate_wall_thicknesses(f64::NAN, 0.5, "9Ni", "hemisphere", 0.95).unwrap_err();
    assert_eq!(e.kind, "ValueError");
    assert_eq!(e.message, "cannot convert float NaN to integer");
    // P = inf gives inf / -inf = NaN in the first pass, as in Python.
    let e = calculate_wall_thicknesses(1.0, f64::INFINITY, "9Ni", "hemisphere", 0.95).unwrap_err();
    assert_eq!(e.message, "cannot convert float NaN to integer");
    // A huge shell overflows to infinity in the rounding step (CPython: OverflowError).
    let e = calculate_wall_thicknesses(1e305, 1000.0, "304L", "hemisphere", 0.95).unwrap_err();
    assert_eq!(e.kind, "OverflowError");
    assert_eq!(e.message, "cannot convert float infinity to integer");
    // Very large but finite results still come back, unrounded by the 0.5 mm step.
    assert_eq!(
        calculate_wall_thicknesses(1e300, 1.0, "304L", "hemisphere", 0.95).unwrap(),
        (4.639964021082768e300, 2.309166243592473e300)
    );
}

#[test]
fn inner_radius_correction_shows_at_type_c_pressure() {
    // At 0.5 MPa the 0.5 mm rounding hides the inner-radius correction (outer radius minus the
    // first-pass thickness); at 1.8 MPa it does not. CPython 3.12 values. Without the
    // correction the first pair would be (51.0, 25.5) and the second (17.5, 9.0).
    assert_eq!(wall(5.9, 1.8, "304L", "hemisphere"), (49.5, 24.5));
    assert_eq!(wall(5.9, 1.8, "304L", "ellipsoidal_2to1"), (49.5, 49.0));
    assert_eq!(wall(4.0, 1.8, "9Ni", "hemisphere"), (17.0, 9.0));
    assert_eq!(wall(4.0, 1.8, "9Ni", "ellipsoidal_2to1"), (17.0, 17.0));
}
