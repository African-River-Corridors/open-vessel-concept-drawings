//! Seam: the tank wall thickness formula, tested against a published tank (issue #292).
//!
//! A published river LPG tank (Senjanovic et al., "Structure design of cargo tanks in river
//! liquefied gas carriers", DESIGN 2006, Design Society): outer diameter 4,730 mm, design pressure
//! 15.8 bar, S690QL1 steel with Re 690 and Rm 770 MPa, allowable stress Rm/2.7 = 285 MPa, cylinder
//! shell 13.1 mm.
//!
//! The pressure-vessel formula (ASME UG-27 shell formula, two passes) is worked by hand with the
//! paper's stress and a weld efficiency of 1.0, on the steel outer radius R = 2,365 mm:
//!
//! - first pass: t = P R / (f E - 0.6 P) = 1.58 x 2365 / (285 - 0.948) = 13.155 mm;
//! - second pass, on the inner radius 2365 - 13.155 = 2351.845 mm: t = 1.58 x 2351.845 / 284.052 = 13.08 mm.
//!
//! The paper prints 13.1 mm (to the tenth of a millimetre), so the formula agrees with it to the
//! rounding of the printed figure.
//!
//! Since issue #313 the material's own stress is the IGC 4.23.3.1 limit, Rm/3 = 256.7 MPa, so the
//! paper's 285 MPa is typed here; the IGC stress gives a thicker shell (worked below).
//!
//! Label: sourced - Senjanovic et al., "Structure design of cargo tanks in river liquefied gas carriers", DESIGN 2006: 4,730 mm outer diameter, 15.8 bar, S690QL1 (Rm 770 MPa, allowable stress 285 MPa), cylinder shell 13.1 mm; the 13.08 mm of the formula and the 14.52 mm at the IGC stress are hand calculations (above and below).

use ovcd_core::pressure_vessel::{
    TANK_MATERIAL_DATABASE, calculate_wall_thicknesses_with, get_tank_material,
    required_cylinder_thickness_mm,
};

/// The published tank: steel outer diameter (m), design pressure (MPa), allowable stress (MPa).
const OUTER_DIAMETER_M: f64 = 4.73;
const DESIGN_PRESSURE_MPA: f64 = 1.58;
const ALLOWABLE_STRESS_MPA: f64 = 285.0;
const PUBLISHED_CYLINDER_MM: f64 = 13.1;

#[test]
fn the_formula_gives_the_published_cylinder_thickness() {
    let t = required_cylinder_thickness_mm(
        OUTER_DIAMETER_M,
        DESIGN_PRESSURE_MPA,
        ALLOWABLE_STRESS_MPA,
        1.0,
    )
    .unwrap();
    // The hand calculation above.
    assert!((t - 13.08).abs() < 0.005, "{t}");
    // The paper's figure is printed to 0.1 mm: half of that is its rounding.
    assert!((t - PUBLISHED_CYLINDER_MM).abs() <= 0.05, "{t}");
}

#[test]
fn the_database_holds_the_papers_steel_and_its_stress_for_saved_files() {
    let (_, steel) = TANK_MATERIAL_DATABASE
        .iter()
        .find(|(key, _)| *key == "S690QL1")
        .expect("S690QL1 is in the database");
    // Rm 770 MPa over 2.7 is 285.19 MPa; the paper prints 285. Since issue #313 it is the stress
    // of a file saved before; the material's own stress is the IGC limit (tests/allowable_stress.rs).
    assert_eq!(steel.tensile_strength, 770.0);
    assert!((steel.published_design_stress - 770.0 / 2.7).abs() < 0.5);
    assert_eq!(steel.published_design_stress, ALLOWABLE_STRESS_MPA);
}

#[test]
fn the_full_chain_adds_the_allowance_and_rounds_up_to_half_a_millimetre() {
    // 13.08 mm required, plus the 0.5 mm corrosion allowance (13.58), rounded up to 0.5 mm: 14.0.
    let (cyl, _) = calculate_wall_thicknesses_with(
        OUTER_DIAMETER_M,
        DESIGN_PRESSURE_MPA,
        "S690QL1",
        "hemisphere",
        1.0,
        Some(ALLOWABLE_STRESS_MPA),
    )
    .unwrap();
    assert_eq!(cyl, 14.0);
}

#[test]
fn the_igc_stress_of_the_material_gives_a_thicker_shell_than_the_paper() {
    // Issue #313, by hand at f = 770/3 = 256.667 MPa, E = 1.0, R = 2,365 mm: first pass
    // 1.58 x 2365 / (256.667 - 0.948) = 14.613 mm; second pass 1.58 x 2350.387 / 255.719 =
    // 14.522 mm; plus 0.5 mm allowance (15.022), rounded up to 0.5 mm: 15.5 mm.
    let t = required_cylinder_thickness_mm(OUTER_DIAMETER_M, DESIGN_PRESSURE_MPA, 770.0 / 3.0, 1.0)
        .unwrap();
    assert!((t - 14.522).abs() < 0.0005, "{t}");
    let (cyl, _) = calculate_wall_thicknesses_with(
        OUTER_DIAMETER_M,
        DESIGN_PRESSURE_MPA,
        "S690QL1",
        "hemisphere",
        1.0,
        None,
    )
    .unwrap();
    assert_eq!(cyl, 15.5);
}

#[test]
fn a_typed_stress_replaces_the_materials_and_keeps_its_allowance_and_minimum() {
    // 9Ni at the paper's stress is the S690QL1 answer: both have a 0.5 mm allowance.
    let nine_ni = calculate_wall_thicknesses_with(
        OUTER_DIAMETER_M,
        DESIGN_PRESSURE_MPA,
        "9Ni",
        "hemisphere",
        1.0,
        Some(ALLOWABLE_STRESS_MPA),
    )
    .unwrap();
    let s690 = calculate_wall_thicknesses_with(
        OUTER_DIAMETER_M,
        DESIGN_PRESSURE_MPA,
        "S690QL1",
        "hemisphere",
        1.0,
        Some(ALLOWABLE_STRESS_MPA),
    )
    .unwrap();
    assert_eq!(nine_ni, s690);
    // A thin result is held up to the material's minimum.
    let (thin, _) =
        calculate_wall_thicknesses_with(2.0, 0.01, "9Ni", "hemisphere", 0.95, Some(1000.0))
            .unwrap();
    assert_eq!(thin, get_tank_material("9Ni").unwrap().min_thickness);
}
