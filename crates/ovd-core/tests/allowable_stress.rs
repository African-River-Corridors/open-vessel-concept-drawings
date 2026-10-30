//! Seam: the allowable stress of each tank steel against the IGC limit (issue #313).
//!
//! IGC 4.23.3.1 (MSC.370(93)): for Type C tanks, f = the lesser of Rm/A and Re/B, with Rm and Re
//! as in 4.18.1.3 (specified minimum tensile strength and yield stress at room temperature; the
//! 0.2 % proof stress where there is no defined yield), and A and B at least 3 and 1.5 for nickel
//! and carbon-manganese steels, 3.5 and 1.5 for austenitic steels. By hand, with the cited
//! strengths:
//!
//! - 304L, 316L (ASTM S30403, S31603: Rp0.2 170, Rm 485 MPa): min(485/3.5, 170/1.5) = min(138.6,
//!   113.3) = 113.3 MPa; the engine holds 113, rounded down.
//! - 9Ni (ASTM A553 Type I: Re 585, Rm 690 MPa): min(690/3, 585/1.5) = min(230, 390) = 230 MPa.
//! - S690QL1 (Re 690, Rm 770 MPa): min(770/3, 690/1.5) = min(256.7, 460) = 256.7 MPa.
//!
//! Label: sourced - IGC Code 4.23.3.1 and 4.18.1.3 (MSC.370(93)); strengths: Outokumpu, Steel grades, properties and global standards (ASTM S30403, S31603), ThyssenKrupp 5 % and 9 % Ni plates (ASTM A553 Type I), Senjanovic et al. DESIGN 2006 and a Macsteel S690QL data sheet (EN 10025-6).

use ovd_core::pressure_vessel::{TANK_MATERIAL_DATABASE, get_tank_material, igc_allowable_stress};

#[test]
fn every_steel_has_the_cited_strengths_and_factors() {
    let expected = [
        ("304L", 485.0, 170.0, 3.5, 1.5),
        ("316L", 485.0, 170.0, 3.5, 1.5),
        ("9Ni", 690.0, 585.0, 3.0, 1.5),
        ("S690QL1", 770.0, 690.0, 3.0, 1.5),
    ];
    assert_eq!(TANK_MATERIAL_DATABASE.len(), expected.len());
    for (key, rm, re, a, b) in expected {
        let m = get_tank_material(key).unwrap();
        assert_eq!(
            (m.tensile_strength, m.yield_strength, m.factor_a, m.factor_b),
            (rm, re, a, b),
            "{key}"
        );
        assert!(!m.strength_source.is_empty(), "{key} cites its strengths");
    }
}

#[test]
fn the_igc_limit_is_the_lesser_of_rm_over_a_and_re_over_b() {
    let f = |key| igc_allowable_stress(get_tank_material(key).unwrap());
    assert!((f("304L") - 170.0 / 1.5).abs() < 1e-12);
    assert!((f("316L") - 170.0 / 1.5).abs() < 1e-12);
    assert_eq!(f("9Ni"), 230.0);
    assert_eq!(f("S690QL1"), 770.0 / 3.0);
}

#[test]
fn every_allowable_stress_is_the_igc_limit_or_that_rounded_down_to_a_whole_mpa() {
    for (key, m) in TANK_MATERIAL_DATABASE.iter() {
        let limit = igc_allowable_stress(m);
        assert!(m.allowable_stress <= limit, "{key}: above the IGC limit");
        assert!(
            m.allowable_stress == limit || m.allowable_stress == libm::floor(limit),
            "{key}: {} is neither {limit} nor that rounded down",
            m.allowable_stress
        );
    }
}

#[test]
fn s690ql1_is_256_7_mpa_and_keeps_its_old_stress_for_saved_files() {
    let m = get_tank_material("S690QL1").unwrap();
    assert_eq!(format!("{:.1}", m.allowable_stress), "256.7");
    // The old basis of the published river LPG tank: Rm 770 over 2.7, printed as 285.
    assert_eq!(m.published_design_stress, 285.0);
    // Only S690QL1 had a stress above the IGC limit.
    for (key, m) in TANK_MATERIAL_DATABASE.iter() {
        if *key != "S690QL1" {
            assert_eq!(m.published_design_stress, m.allowable_stress, "{key}");
        }
    }
}
