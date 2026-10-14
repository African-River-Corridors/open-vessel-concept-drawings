//! Seam: the public `ovcd_core::models` API. `models` has no entry point, so its behaviour (the
//! dataclass defaults and `__post_init__`) is pinned here against `core/models.py`.

use ovcd_core::models::BargeInputs;

fn base() -> BargeInputs {
    BargeInputs::new(90.0, 14.8, 6.5, 6.5, 65.0)
}

#[test]
fn defaults_match_the_python_dataclass() {
    let i = base();
    assert_eq!(i.vessel_type, "IGC_2G");
    assert_eq!((i.bow_non_tank_pct, i.stern_non_tank_pct), (0.10, 0.15));
    assert_eq!((i.wing_tank_gap, i.wing_tank_height_ratio), (0.6, 0.66));
    assert_eq!(
        (i.weather_shield_thickness, i.weather_shield_clearance),
        (6.0, 1.5)
    );
    assert_eq!((i.n_tanks_wide, i.n_tanks_long), (1, 2));
    assert_eq!(i.tank_wall_thickness, Some(25.0));
    assert_eq!(
        (i.tank_head_type.as_str(), i.tank_material.as_str()),
        ("hemisphere", "9Ni")
    );
    assert_eq!(i.tank_material_density, 7.85);
    assert_eq!(
        (
            i.design_pressure,
            i.weld_efficiency,
            i.tank_structure_factor
        ),
        (0.5, 0.95, 1.30)
    );
    assert_eq!(
        (i.insulation_thickness, i.insulation_density),
        (350.0, 60.0)
    );
    assert_eq!(i.cargo_type, "LNG_ethane");
    assert_eq!(i.fill_ratio, 0.95);
    assert_eq!(i.double_bottom_height, Some(1.0));
    assert_eq!((i.tug_length, i.tug_beam, i.notch_depth), (25.0, 9.1, 3.0));
    assert!(i.aft_arrangement.is_none() && i.separation.is_none());
}

#[test]
fn post_init_fills_the_unset_fields() {
    let mut i = base();
    assert_eq!(i.side_clearance, None);
    i.post_init().unwrap();
    assert_eq!(i.bow_rake_rise, Some(6.5));
    assert_eq!(i.stern_rake_rise, Some(6.5));
    assert_eq!(i.bow_non_tank_length, Some(0.10 * 90.0));
    assert_eq!(i.stern_non_tank_length, Some(0.15 * 90.0));
    assert_eq!(i.side_clearance, Some(1.0));
    assert_eq!(i.wing_tank_width, Some(1.0 - 0.6));
}

#[test]
fn post_init_keeps_what_was_set() {
    let mut i = base();
    i.bow_rake_rise = Some(2.0);
    i.bow_non_tank_length = Some(11.0);
    i.stern_non_tank_length = Some(12.0);
    i.side_clearance = Some(2.0);
    i.wing_tank_width = Some(0.25);
    i.post_init().unwrap();
    assert_eq!(i.bow_rake_rise, Some(2.0));
    assert_eq!(i.stern_rake_rise, Some(6.5));
    assert_eq!(i.bow_non_tank_length, Some(11.0));
    assert_eq!(i.stern_non_tank_length, Some(12.0));
    assert_eq!(i.side_clearance, Some(2.0));
    assert_eq!(i.wing_tank_width, Some(0.25));
}

#[test]
fn the_wing_tank_width_never_goes_below_zero() {
    let mut i = base();
    i.side_clearance = Some(0.5);
    i.post_init().unwrap();
    assert_eq!(i.wing_tank_width, Some(0.0));
    // max(0.0, -0.0) keeps the first of two equal zeros: positive zero.
    let mut i = base();
    i.side_clearance = Some(0.6);
    i.post_init().unwrap();
    assert!(i.wing_tank_width.unwrap().is_sign_positive());
}

#[test]
fn an_automatic_wall_takes_the_material_density() {
    let mut i = base();
    i.tank_wall_thickness = None;
    i.tank_material = "304L".into();
    i.post_init().unwrap();
    assert_eq!(i.tank_material_density, 7.93);
    // A fixed wall leaves the density as given.
    let mut j = base();
    j.tank_material = "304L".into();
    j.post_init().unwrap();
    assert_eq!(j.tank_material_density, 7.85);
}

#[test]
fn an_unknown_material_with_an_automatic_wall_is_a_value_error() {
    let mut i = base();
    i.tank_wall_thickness = None;
    i.tank_material = "A36".into();
    let e = i.post_init().unwrap_err();
    assert_eq!(
        (e.kind, e.message.as_str()),
        (
            "ValueError",
            "Unknown tank material 'A36'. Valid: 304L, 316L, 9Ni"
        )
    );
}

#[test]
fn a_zero_wing_tank_width_is_positive_zero() {
    // Python `max(0.0, -0.0 - 0.0)` keeps the first of two equal zeros: +0.0.
    let mut i = base();
    i.side_clearance = Some(-0.0);
    i.wing_tank_gap = 0.0;
    i.post_init().unwrap();
    let w = i.wing_tank_width.unwrap();
    assert_eq!(w, 0.0);
    assert!(w.is_sign_positive());
}
