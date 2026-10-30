//! Seam: the diameter datum of a gas tank is one face, the outside of the insulation (issue #292).
//!
//! The tank outer diameter of the engine is measured to the outside of the insulation. This file
//! checks that the volume, the wall thickness formula, the clearances and the tank top all start
//! from that one datum, and that the steel diameter is the outer diameter less twice the
//! insulation. It also checks the tank design values the capacity calculation now carries.
//!
//! Label: property. Each test states a relation between the engine's outputs. No printed number.

use ovd_core::barge_capacity::{
    BargeCapacityInputs, DEFAULT_TANK_DESIGN_PRESSURE_MPA, DEFAULT_TANK_INSULATION_MM,
    DEFAULT_TANK_MATERIAL, DEFAULT_TANK_STRUCTURE_FACTOR, TankDesign, compute, dome_volume_m3,
};
use ovd_core::models::BargeInputs;
use ovd_core::pressure_vessel::calculate_wall_thicknesses;
use ovd_core::protective_location::d_vc_exact;
use ovd_core::tanks::calculate_tank_arrangement;

fn gas(design: TankDesign, wall: Option<f64>) -> BargeCapacityInputs {
    let mut i = BargeCapacityInputs::new(110.0, 11.4, 6.0);
    i.cargo_type = "LPG_propane".to_string();
    i.tank_design = design;
    i.tank_wall_thickness = wall;
    i
}

#[test]
fn the_tank_design_defaults_are_what_the_engine_used_before() {
    let d = TankDesign::default();
    let b = BargeInputs::new(90.0, 14.8, 6.5, 6.5, 65.0);
    assert_eq!(d.design_pressure_mpa, b.design_pressure);
    assert_eq!(d.material, b.tank_material);
    assert_eq!(d.insulation_thickness_mm, b.insulation_thickness);
    assert_eq!(d.structure_factor, b.tank_structure_factor);
    assert_eq!(d.allowable_stress_mpa, b.tank_allowable_stress);
    assert_eq!(
        (
            DEFAULT_TANK_DESIGN_PRESSURE_MPA,
            DEFAULT_TANK_MATERIAL,
            DEFAULT_TANK_INSULATION_MM,
            DEFAULT_TANK_STRUCTURE_FACTOR
        ),
        (0.5, "9Ni", 350.0, 1.30)
    );
}

#[test]
fn the_wall_formula_takes_the_steel_diameter_not_the_insulated_one() {
    // A high pressure makes the wall depend on the diameter beyond the 0.5 mm rounding.
    let mut b = BargeInputs::new(90.0, 14.8, 6.5, 6.5, 65.0);
    b.bow_non_tank_length = Some(10.9);
    b.stern_non_tank_length = Some(21.9);
    b.n_tanks_wide = 2;
    b.n_tanks_long = 2;
    b.double_bottom_height = Some(0.5);
    b.tank_wall_thickness = None;
    b.design_pressure = 1.8;
    b.tank_material = "S690QL1".into();
    b.insulation_thickness = 400.0;
    b.wall_formula_on_steel_diameter = true;
    b.post_init().unwrap();
    let t = calculate_tank_arrangement(&b).unwrap();
    let steel_od = t.outer_diameter - 2.0 * 0.4;
    let want =
        calculate_wall_thicknesses(steel_od, 1.8, "S690QL1", "hemisphere", b.weld_efficiency)
            .unwrap();
    assert_eq!((t.cylinder_wall_thickness, t.head_wall_thickness), want);
    // Not the answer for the insulated diameter, which asks for a thicker wall.
    let outer = calculate_wall_thicknesses(
        t.outer_diameter,
        1.8,
        "S690QL1",
        "hemisphere",
        b.weld_efficiency,
    )
    .unwrap();
    assert!(outer.0 > want.0, "{outer:?} vs {want:?}");
    // The cargo space is inside the steel: inner = outer - 2 insulation - 2 wall.
    let inner = t.outer_diameter - 0.8 - 2.0 * t.cylinder_wall_thickness / 1000.0;
    assert!((t.inner_diameter - inner).abs() < 1e-12);
}

#[test]
fn the_python_api_default_keeps_the_original_formula_diameter() {
    // The original engine gave the formula the insulated diameter, and the Python API of the 0.1.0
    // answers still does: `wall_formula_on_steel_diameter` is false unless the gas capacity
    // calculation sets it. Changing that is a change to the public API.
    let mut b = BargeInputs::new(90.0, 14.8, 6.5, 6.5, 65.0);
    b.bow_non_tank_length = Some(10.9);
    b.stern_non_tank_length = Some(21.9);
    b.n_tanks_wide = 2;
    b.n_tanks_long = 2;
    b.double_bottom_height = Some(0.5);
    b.tank_wall_thickness = None;
    b.design_pressure = 1.8;
    b.tank_material = "S690QL1".into();
    b.insulation_thickness = 400.0;
    assert!(!b.wall_formula_on_steel_diameter);
    b.post_init().unwrap();
    let t = calculate_tank_arrangement(&b).unwrap();
    let on_insulated = calculate_wall_thicknesses(
        t.outer_diameter,
        1.8,
        "S690QL1",
        "hemisphere",
        b.weld_efficiency,
    )
    .unwrap();
    assert_eq!(
        (t.cylinder_wall_thickness, t.head_wall_thickness),
        on_insulated
    );
}

#[test]
fn the_capacity_calculation_puts_the_formula_on_the_steel_diameter() {
    // Through `compute`: the wall of a gas design is the formula at the steel diameter.
    let design = TankDesign {
        design_pressure_mpa: 1.77,
        material: "S690QL1".into(),
        insulation_thickness_mm: 200.0,
        ..TankDesign::default()
    };
    let r = compute(&gas(design, None)).unwrap();
    let basis = r.gas_basis.as_ref().unwrap();
    let steel = r.tank_outer_diameter_m.unwrap() - 0.4;
    let want = calculate_wall_thicknesses(
        steel,
        1.77,
        "S690QL1",
        "hemisphere",
        basis.inputs.weld_efficiency,
    )
    .unwrap();
    assert_eq!(
        (
            basis.tanks.cylinder_wall_thickness,
            basis.tanks.head_wall_thickness
        ),
        want
    );
}

#[test]
fn the_clearances_are_measured_to_the_steel_whatever_the_insulation() {
    // IGC 2.4.2: d is plate to plate. For each insulation thickness the side clearance from the
    // insulated face is d less that thickness (not below zero), and the bottom clearance is the
    // larger of B/15 (at most 2 m) and d, less the thickness.
    for ins in [0.0, 200.0, 330.0, 350.0] {
        let design = TankDesign {
            insulation_thickness_mm: ins,
            ..TankDesign::default()
        };
        let i = gas(design, Some(25.0));
        let r = compute(&i).unwrap();
        let basis = r.gas_basis.as_ref().unwrap();
        let n = r.n_tanks_total.unwrap() as f64;
        let vc = basis.tanks.total_tank_volume / n + dome_volume_m3(i.dome_diameter, i.dome_height);
        let d = d_vc_exact(vc);
        let ins_m = ins / 1000.0;
        assert!(
            (r.side_clearance_m.unwrap() - (d - ins_m).max(0.0)).abs() < 1e-6,
            "ins {ins}"
        );
        let bottom = (i.beam / 15.0).min(2.0).max(d) - ins_m;
        assert!(
            (r.bottom_clearance_m.unwrap() - bottom).abs() < 1e-6,
            "ins {ins}"
        );
        // The tank top is the insulated face: the reported diameter is the insulated one.
        assert_eq!(r.insulation_thickness_mm, Some(ins));
        assert_eq!(r.tank_outer_diameter_m, Some(basis.tanks.outer_diameter));
    }
}

#[test]
fn the_steel_mass_follows_the_material_whether_the_wall_is_fixed_or_not() {
    // With a fixed wall the engine used 9Ni's density whatever the material; now the material's own.
    let nine = compute(&gas(TankDesign::default(), Some(25.0))).unwrap();
    let stainless = compute(&gas(
        TankDesign {
            material: "304L".into(),
            ..TankDesign::default()
        },
        Some(25.0),
    ))
    .unwrap();
    let w = |r: &ovd_core::barge_capacity::BargeCapacityResult| {
        r.gas_basis.as_ref().unwrap().tanks.tank_shell_weight
    };
    assert!((w(&stainless) / w(&nine) - 7.93 / 7.85).abs() < 1e-12);
}

#[test]
fn the_design_values_reach_the_tank_and_its_weight() {
    let base = compute(&gas(TankDesign::default(), None)).unwrap();
    let basis = |r: &ovd_core::barge_capacity::BargeCapacityResult| r.gas_basis.clone().unwrap();
    // A higher pressure and a stronger steel move the wall thickness.
    let high = compute(&gas(
        TankDesign {
            design_pressure_mpa: 1.77,
            material: "S690QL1".into(),
            insulation_thickness_mm: 0.0,
            structure_factor: 1.12,
            ..TankDesign::default()
        },
        None,
    ))
    .unwrap();
    let (b0, b1) = (basis(&base), basis(&high));
    assert!(b1.tanks.cylinder_wall_thickness > b0.tanks.cylinder_wall_thickness);
    assert_eq!(b1.tanks.design_pressure, 1.77);
    assert_eq!(b1.tanks.tank_material, "S690QL1");
    assert_eq!(b1.inputs.insulation_thickness, 0.0);
    assert_eq!(b1.tanks.insulation_weight, 0.0);
    assert_eq!(b1.inputs.tank_structure_factor, 1.12);
    // A typed stress is used in place of the material's.
    let typed = compute(&gas(
        TankDesign {
            allowable_stress_mpa: Some(500.0),
            ..TankDesign::default()
        },
        None,
    ))
    .unwrap();
    assert!(basis(&typed).tanks.cylinder_wall_thickness <= b0.tanks.cylinder_wall_thickness);
    // A fixed wall wins over the formula.
    let fixed = compute(&gas(TankDesign::default(), Some(40.0))).unwrap();
    assert_eq!(basis(&fixed).tanks.cylinder_wall_thickness, 40.0);
    assert_eq!(basis(&fixed).tanks.tank_material, "");
}
