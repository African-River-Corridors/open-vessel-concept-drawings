//! Seam: the surface area of a tank head (issue #314).
//!
//! A 2:1 semi-ellipsoidal head is half an oblate spheroid with semi-axes a = R (the radius) and
//! c = R/2. MathWorld, "Oblate Spheroid", eq. (11): the whole spheroid has S = 2 pi a^2 +
//! (pi c^2 / e) ln((1 + e) / (1 - e)), with e = sqrt(1 - c^2 / a^2). With c = a/2, e = sqrt(3)/2,
//! and half of it is pi R^2 (1 + ln((2 + sqrt 3) / (2 - sqrt 3)) / (8 e)) = 1.380173 pi R^2 =
//! 4.335941 R^2 = 1.083985 D^2 (by hand). The old engine took 1.084 pi R^2 = 3.405486 R^2: the
//! 1.084 D^2 rule with pi R^2 in place of D^2, a factor of pi/4. The head was 21.5 % low against
//! the true area, which is 27.3 % above the old one: both reviews found the same error, measured
//! from different bases.
//!
//! A hemispherical head is half a sphere: 2 pi R^2 (MathWorld, "Sphere": S = 4 pi R^2).
//!
//! The steel area is taken at the steel outer surface (the radius inside the insulation) and the
//! insulation area at the tank outer diameter (the outside of the insulation), as before.
//!
//! Label: sourced - MathWorld, "Oblate Spheroid" (eq. 11) and "Sphere"; the 1.380173 and 4.335941 are hand calculations from them (above).

use core::f64::consts::PI;

use ovcd_core::models::BargeInputs;
use ovcd_core::tanks::{calculate_tank_arrangement, ellipsoidal_2to1_head_area};

#[test]
fn the_2_to_1_head_is_half_an_oblate_spheroid() {
    for r in [0.5, 2.365, 4.0, 7.3] {
        let a = ellipsoidal_2to1_head_area(r);
        assert!(
            (a / (r * r) - 4.335_941).abs() < 5e-7,
            "{r}: {}",
            a / (r * r)
        );
        assert!((a / (PI * r * r) - 1.380_173).abs() < 5e-7, "{r}");
        // The 1.084 D^2 rule, to its printed figures.
        assert!((a / (4.0 * r * r) - 1.084).abs() < 5e-4, "{r}");
    }
}

#[test]
fn the_old_constant_was_low_by_a_factor_of_pi_over_4() {
    let r = 3.0;
    let old = 1.084 * PI * r * r;
    let new = ellipsoidal_2to1_head_area(r);
    let low = (new - old) / new;
    let high = (new - old) / old;
    // The two reviews: 21 % low against the true area, 27 % high against the old one.
    assert!((low - 0.2146).abs() < 0.0005, "{low}");
    assert!((high - 0.2732).abs() < 0.0005, "{high}");
}

/// One tank, as the calibration runs it: 4.73 m steel outer diameter, 21 m long, no insulation.
fn one_tank(head: &str, wall_mm: f64, insulation_mm: f64) -> BargeInputs {
    let od = 4.73 + 2.0 * insulation_mm / 1000.0;
    let mut i = BargeInputs::new(21.0, od, 1.0, 1.0, 10.0);
    i.n_tanks_wide = 1;
    i.n_tanks_long = 1;
    i.bow_non_tank_length = Some(0.0);
    i.stern_non_tank_length = Some(0.0);
    i.side_clearance = Some(0.0);
    i.double_bottom_height = Some(0.0);
    i.tank_wall_thickness = Some(wall_mm);
    i.tank_head_type = head.into();
    i.insulation_thickness = insulation_mm;
    i.post_init().unwrap();
    i
}

#[test]
fn the_head_steel_is_the_head_area_at_the_steel_outer_surface_times_the_wall() {
    for head in ["hemisphere", "ellipsoidal_2to1"] {
        let i = one_tank(head, 10.0, 0.0);
        let t = calculate_tank_arrangement(&i).unwrap();
        let (ro, ri, wall) = (2.365, 2.365 - 0.010, 0.010);
        let cylinder = (2.0 * PI * ro + 2.0 * PI * ri) / 2.0 * t.cylinder_length * wall;
        let area = if head == "hemisphere" {
            2.0 * PI * ro * ro
        } else {
            4.335_941_351_672_526 * ro * ro
        };
        let expected = (cylinder + 2.0 * area * wall) * i.tank_material_density;
        assert!(
            (t.tank_shell_weight - expected).abs() < 1e-6,
            "{head}: {} {expected}",
            t.tank_shell_weight
        );
    }
}

#[test]
fn the_head_insulation_is_the_head_area_at_the_tank_outer_diameter() {
    let i = one_tank("ellipsoidal_2to1", 10.0, 200.0);
    let t = calculate_tank_arrangement(&i).unwrap();
    let r = i.beam / 2.0;
    let area = 2.0 * PI * r * t.cylinder_length + 2.0 * 4.335_941_351_672_526 * r * r;
    let expected = area * 0.2 * i.insulation_density / 1000.0;
    assert!((t.insulation_weight - expected).abs() < 1e-9);
}
