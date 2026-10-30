//! Seam: the design liquid pressure of a Type C tank wall (issue #374).
//!
//! IGC 4.23.2.2 takes the design liquid pressure of 4.13.2 into the internal pressure; 4.13.2.4 sets
//! Peq = Po + Pgd; 4.28.1.2 gives Pgd = a x Z x rho / 1.02e5 MPa. By hand, an LNG tank (rho 450
//! kg/m3) of 9.65 m steel outer diameter at a = 1: Pgd = 9.65 x 450 / 102,000 = 0.042574 MPa.
//!
//! IGC 4.28.2.1 vertical acceleration, by hand for L0 = 110 m, CB = 0.85, V = 10 kn, x = 30 m:
//! a0 = 0.2 x 10 / sqrt(110) + (34 - 600/110) / 110 = 0.190693 + 0.259504 = 0.450197;
//! az = a0 x sqrt(1 + (5.3 - 45/110)^2 (30/110 + 0.05)^2 (0.6/0.85)^1.5) = 0.708623.
//!
//! Label: sourced - IGC Code 4.13.2.4, 4.23.2.2, 4.28.1.2 and 4.28.2.1 (MSC.370(93)); the values are hand calculations (above).

use ovd_core::barge_capacity::{
    BargeCapacityInputs, LiquidPressure, compute, igc_vertical_acceleration, liquid_pressure_basis,
};
use ovd_core::tanks::liquid_pressure_mpa;

#[test]
fn the_liquid_pressure_is_a_z_rho_over_1_02e5() {
    let p = liquid_pressure_mpa(Some(1.0), 9.65, 450.0);
    assert!((p - 0.042574).abs() < 5e-7, "{p}");
    assert_eq!(liquid_pressure_mpa(None, 9.65, 450.0), 0.0);
    assert_eq!(liquid_pressure_mpa(Some(2.0), 9.65, 450.0), 2.0 * p);
}

#[test]
fn the_igc_4_28_2_vertical_acceleration_by_hand() {
    let az = igc_vertical_acceleration(110.0, 0.85, 10.0, 30.0);
    assert!((az - 0.708623).abs() < 5e-7, "{az}");
    // At amidships the x term is 0.05 only; further from amidships it grows.
    assert!(igc_vertical_acceleration(110.0, 0.85, 10.0, 0.0) < az);
}

fn lng(pressure: LiquidPressure) -> BargeCapacityInputs {
    let mut i = BargeCapacityInputs::new(110.0, 11.4, 6.0);
    i.cargo_type = "LNG".into();
    i.tank_wall_thickness = None;
    i.tank_design.design_pressure_mpa = 0.4;
    i.tank_design.liquid_pressure = pressure;
    i
}

#[test]
fn the_wall_takes_the_vapour_pressure_plus_the_liquid_pressure() {
    let wall = |p| {
        compute(&lng(p))
            .unwrap()
            .gas_basis
            .unwrap()
            .tanks
            .cylinder_wall_thickness
    };
    let none = wall(LiquidPressure::None);
    let stat = wall(LiquidPressure::Static);
    let sea = wall(LiquidPressure::Igc4282 {
        service_speed_kn: 10.0,
    });
    assert!(none <= stat && stat <= sea, "{none} {stat} {sea}");
    assert!(none < sea, "{none} {sea}");
    // The basis: a = 1 when static, 1 + az of the tank farthest from amidships at sea; rho is the
    // LNG density of the cargo data.
    let i = lng(LiquidPressure::Static);
    assert_eq!(
        liquid_pressure_basis(&i, 10.0, 20.0, 2.0).unwrap(),
        (Some(1.0), 450.0)
    );
    let (a, _) = liquid_pressure_basis(&lng(LiquidPressure::None), 10.0, 20.0, 2.0).unwrap();
    assert_eq!(a, None);
}

#[test]
fn an_lpg_tank_takes_the_density_at_its_design_temperature() {
    let mut i = lng(LiquidPressure::Static);
    i.cargo_type = "LPG_propane".into();
    i.tank_design.design_temperature_c = Some(-48.0);
    let (_, rho) = liquid_pressure_basis(&i, 10.0, 20.0, 2.0).unwrap();
    // NIST: 587.63 kg/m3 at -48 C, above the cargo data's 581.
    assert_eq!(rho, 587.63);
    i.tank_design.design_temperature_c = Some(0.0);
    let (_, rho) = liquid_pressure_basis(&i, 10.0, 20.0, 2.0).unwrap();
    // At 0 C the saturated density (528.59) is below the cargo density in force: the larger holds.
    assert_eq!(rho, 581.0);
}
