//! Label: property - the default Crude Grade is complete, graded, and agrees with its Cargo.

use ovcd_core::cargo::{CARGO_DATABASE, CargoClass, CargoStatus, get_cargo};
use ovcd_core::liquid_bulk::adn_vessel_type::{AdnFlags, AdnVesselType, Branch, derive};
use ovcd_core::liquid_bulk::crude_grade::{
    CRUDE_GRADES, CRUDE_OIL_KEY, DEFAULT_GRADE, DIESEL_GRADE, DIESEL_KEY, Status, grade_of,
};

#[test]
fn crude_oil_is_a_liquid_bulk_cargo_in_development() {
    let c = get_cargo(CRUDE_OIL_KEY).unwrap();
    assert_eq!(c.phase, CargoClass::LiquidBulk);
    assert_eq!(c.status, CargoStatus::Development);
    assert_eq!(c.status.key(), "development");
}

#[test]
fn every_liquid_bulk_cargo_is_in_development_and_every_other_cargo_is_released() {
    for (key, props) in CARGO_DATABASE.iter() {
        let want = if props.phase == CargoClass::LiquidBulk {
            CargoStatus::Development
        } else {
            CargoStatus::Released
        };
        assert_eq!(props.status, want, "{key}");
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

// ---- the second oil: diesel fuel to EN 590 (issue #342, spec 0006 section 8, ticket 15) -------------

#[test]
fn diesel_is_a_liquid_bulk_cargo_with_its_own_grade_and_the_density_of_its_grade() {
    let c = get_cargo(DIESEL_KEY).unwrap();
    assert_eq!(c.phase, CargoClass::LiquidBulk);
    assert_eq!(c.status, CargoStatus::Development);
    assert_eq!(c.density, DIESEL_GRADE.density_kg_m3.value);
    assert_eq!(c.temperature, DIESEL_GRADE.density_reference_c);
    assert_eq!(grade_of(DIESEL_KEY), Some(&DIESEL_GRADE));
    assert_eq!(CRUDE_GRADES.len(), 2);
}

/// Label: sourced - ADN 2025 3.2.3.2 Table C, the row UN 1202 "DIESEL FUEL complying with standard
/// EN 590:2013 + A1:2017": Class 3, F1, III, hazard labels 3+N2+F, type of tank vessel N, cargo tank
/// design 4 (open), cargo tank type 3 (walls distinct from the outer hull), maximum degree of filling
/// 97 %, relative density at 20 deg C 0.82 to 0.85.
#[test]
fn the_derived_adn_type_of_diesel_is_what_table_c_gives_for_its_row() {
    let r = derive(&AdnFlags::of(&DIESEL_GRADE));
    // type N, open tanks (design 4), walls distinct from the hull (tank type 3)
    assert_eq!(r.vessel_type, AdnVesselType::NOpen);
    assert!(r.walls_distinct_from_hull);
    assert_eq!(r.branch, Branch::Open);
    // the relative density of the grade is inside the range of the row, and so is the design density
    for kg_m3 in [
        DIESEL_GRADE.density_kg_m3.value,
        DIESEL_GRADE.design_density_kg_m3.value,
    ] {
        assert!((820.0..=850.0).contains(&kg_m3), "{kg_m3}");
    }
    assert_eq!(DIESEL_GRADE.density_reference_c, 20.0);
    // What Table C states is sourced; what was not read is an estimate with that said.
    assert_eq!(DIESEL_GRADE.aquatic.status, Status::Sourced);
    assert_eq!(DIESEL_GRADE.cmr.status, Status::Sourced);
    assert_eq!(DIESEL_GRADE.flash_point_below_23c.status, Status::Estimate);
}

/// ADR 0016: a new oil is a data row, not a code path. The key of the second oil appears in the two data
/// tables and nowhere else in the Rust code; every other place reads a Cargo through its Cargo Class.
#[test]
fn the_second_oil_has_no_branch_in_the_rust_code() {
    use std::path::{Path, PathBuf};
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    for c in [
        "ovcd-core",
        "ovcd-format",
        "ovcd-cli",
        "ovcd-shell",
        "ovcd-py",
        "ovcd-web",
        "ovcd-draw",
    ] {
        let src = crates.join(c).join("src");
        if src.is_dir() {
            walk(&src, &mut files);
        }
    }
    assert!(files.len() > 50, "{} source files", files.len());
    let mut found: Vec<String> = Vec::new();
    for f in &files {
        let text = std::fs::read_to_string(f).unwrap();
        if text.contains("diesel_en590")
            || text.contains("DIESEL_KEY")
            || text.contains("DIESEL_GRADE")
        {
            found.push(f.file_name().unwrap().to_string_lossy().into_owned());
        }
    }
    found.sort();
    assert_eq!(found, ["cargo.rs", "crude_grade.rs"], "{found:?}");
}
