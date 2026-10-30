//! Seam: the dome counts in Vc, and the per-tank d(Vc) sets the side clearance (issue #237).
//!
//! IGC Code 2.4.1.1 (MSC.370(93)): Vc is 100 % of the gross design volume of the individual cargo
//! tank at 20 C, including domes and appendages. 2.4.2: d is measured plate to plate, from the
//! moulded line to the moulded line, excluding insulation, so the clear gap from the outer face of
//! the insulation is d less the insulation.
//!
//! Appendages (nozzles, manways and the like) are not modelled: no volume is added for them.
//!
//! Label: property. Each test states a rule (a relation between the engine's outputs and the
//! formulas of the Code), not a printed number. The only fixed number, the volume of the default
//! dome, is worked by hand: pi/4 x 2 x 2 x 2 = 2 pi.

use ovd_core::barge_capacity::{
    BargeCapacityInputs, BargeCapacityResult, DEFAULT_TANK_DOME_DIAMETER_M, compute, dome_volume_m3,
};
use ovd_core::protective_location::d_vc_exact;

fn run(loa: f64, beam: f64, dome_diameter_m: f64) -> BargeCapacityResult {
    let mut i = BargeCapacityInputs::new(loa, beam, 6.5);
    i.cargo_type = "LPG_propane".to_string();
    i.dome_diameter = dome_diameter_m;
    compute(&i).unwrap()
}

/// The Vc of one tank as the Code defines it: the tank's volume plus its dome.
fn vc_of_one_tank(r: &BargeCapacityResult, dome_diameter_m: f64) -> f64 {
    let tanks = &r.gas_basis.as_ref().unwrap().tanks;
    tanks.total_tank_volume / r.n_tanks_total.unwrap() as f64
        + dome_volume_m3(
            dome_diameter_m,
            r.gas_basis.as_ref().unwrap().inputs.dome_height,
        )
}

#[test]
fn the_default_dome_is_2_m_wide_and_2_m_high_and_holds_2_pi_m3() {
    assert_eq!(DEFAULT_TANK_DOME_DIAMETER_M, 2.0);
    assert_eq!(BargeCapacityInputs::new(90.0, 14.8, 6.5).dome_diameter, 2.0);
    // pi/4 x 2^2 x 2 = 2 pi
    assert!((dome_volume_m3(2.0, 2.0) - 2.0 * core::f64::consts::PI).abs() < 1e-12);
    assert_eq!(dome_volume_m3(0.0, 2.0), 0.0);
}

#[test]
fn the_side_clearance_is_d_of_the_tank_with_its_dome_less_the_insulation() {
    // 2.4.1.1 and 2.4.2, over hulls that put one tank in each of the three d(Vc) bands that
    // matter (up to 1,000 m3, 1,000 to 5,000 m3, and above 5,000 m3), and several dome sizes.
    let mut bands = [false; 3];
    for (loa, beam) in [
        (70.0, 10.0),
        (90.0, 14.8),
        (110.0, 14.8),
        (130.0, 20.0),
        (180.0, 24.0),
    ] {
        for dome in [0.0, 2.0, 6.0] {
            let r = run(loa, beam, dome);
            if r.gas_basis.is_none() || r.n_tanks_total.unwrap_or(0) == 0 {
                continue;
            }
            let ins = r.insulation_thickness_mm.unwrap() / 1000.0;
            let vc = vc_of_one_tank(&r, dome);
            let want = (d_vc_exact(vc) - ins).max(0.0);
            let got = r.side_clearance_m.unwrap();
            // The loop stops when d moves by less than 1e-9, so the clearance it used is within
            // that of the clearance of the final tank.
            assert!(
                (got - want).abs() < 1e-6,
                "{loa} x {beam}, dome {dome}: clearance {got}, rule {want}, Vc {vc}"
            );
            bands[if vc <= 1000.0 {
                0
            } else if vc < 5000.0 {
                1
            } else {
                2
            }] = true;
        }
    }
    assert_eq!(bands, [true, true, true], "the sweep must reach every band");
}

#[test]
fn a_wider_dome_gives_a_larger_clearance_through_vc() {
    // d(Vc) rises with Vc in this band (2.4.1.1: 0.75 + Vc x 0.2 / 4,000), and a wider dome adds
    // volume, so the clearance is larger. The tank shrinks a little as the clearance grows, which
    // takes some of the dome's volume back, but not all of it.
    let none = run(90.0, 14.8, 0.0);
    let big = run(90.0, 14.8, 6.0);
    assert!(big.side_clearance_m.unwrap() > none.side_clearance_m.unwrap());
}

#[test]
fn the_dome_is_not_in_the_cargo_volume_or_the_inner_volume() {
    for dome in [0.0, 2.0, 6.0] {
        let r = run(90.0, 14.8, dome);
        let tanks = &r.gas_basis.as_ref().unwrap().tanks;
        // The inner volume reported is the tanks' own (cylinder and heads), with no dome.
        assert_eq!(r.containment_inner_volume_m3, Some(tanks.total_tank_volume));
        // The cargo is the fill ratio of the tanks' volume.
        assert_eq!(
            r.cargo_volume_m3,
            tanks.total_tank_volume * r.gas_basis.as_ref().unwrap().inputs.fill_ratio
        );
    }
}
