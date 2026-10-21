//! Label: property - the default Crude Grade is complete, graded, and agrees with its Cargo.

use ovcd_core::cargo::{CARGO_DATABASE, CargoClass, CargoStatus, get_cargo};
use ovcd_core::liquid_bulk::crude_grade::{
    CRUDE_GRADES, CRUDE_OIL_KEY, DEFAULT_GRADE, Status, grade_of,
};

#[test]
fn crude_oil_is_a_liquid_bulk_cargo_in_development() {
    let c = get_cargo(CRUDE_OIL_KEY).unwrap();
    assert_eq!(c.phase, CargoClass::LiquidBulk);
    assert_eq!(c.status, CargoStatus::Development);
    assert_eq!(c.status.key(), "development");
}

#[test]
fn every_other_cargo_is_released() {
    for (key, props) in CARGO_DATABASE.iter() {
        if *key != CRUDE_OIL_KEY {
            assert_eq!(props.status, CargoStatus::Released, "{key}");
        }
    }
}

#[test]
fn the_cargo_density_is_the_grade_density_at_15_c() {
    let c = get_cargo(CRUDE_OIL_KEY).unwrap();
    assert_eq!(c.density, DEFAULT_GRADE.density_kg_m3.value);
    assert_eq!(c.temperature, DEFAULT_GRADE.density_reference_c);
    assert_eq!(DEFAULT_GRADE.density_reference_c, 15.0);
}

#[test]
fn the_spec_defaults() {
    assert_eq!(DEFAULT_GRADE.density_kg_m3.value, 870.0);
    assert_eq!(DEFAULT_GRADE.design_density_kg_m3.value, 1025.0);
    assert!(!DEFAULT_GRADE.sour.value);
    assert!(DEFAULT_GRADE.pour_point_c.is_none());
    assert!(DEFAULT_GRADE.carriage_temperature_c.is_none());
}

#[test]
fn every_value_has_a_status_and_a_basis_and_a_sourced_one_cites_a_document() {
    let g = &DEFAULT_GRADE;
    let rows = [
        (g.density_kg_m3.status, g.density_kg_m3.basis),
        (g.design_density_kg_m3.status, g.design_density_kg_m3.basis),
        (g.sour.status, g.sour.basis),
        (g.cmr.status, g.cmr.basis),
        (
            g.benzene_over_10_percent.status,
            g.benzene_over_10_percent.basis,
        ),
        (g.aquatic.status, g.aquatic.basis),
        (
            g.flash_point_below_23c.status,
            g.flash_point_below_23c.basis,
        ),
        (
            g.vapour_pressure_50c_at_least_1kpa.status,
            g.vapour_pressure_50c_at_least_1kpa.basis,
        ),
        (
            g.vapour_pressure_class.status,
            g.vapour_pressure_class.basis,
        ),
    ];
    for (status, basis) in rows {
        assert!(basis.len() > 10, "{basis}");
        if status == Status::Sourced {
            assert!(
                basis.contains("ADN") || basis.contains("MARPOL") || basis.contains("ISO"),
                "{basis}"
            );
        }
    }
}

#[test]
fn every_grade_belongs_to_a_liquid_bulk_cargo() {
    for (key, _) in CRUDE_GRADES.iter() {
        assert_eq!(
            get_cargo(key).unwrap().phase,
            CargoClass::LiquidBulk,
            "{key}"
        );
    }
    assert!(grade_of("LNG").is_none());
    assert!(grade_of(CRUDE_OIL_KEY).is_some());
}
