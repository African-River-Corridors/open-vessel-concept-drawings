//! Seam: the 0.1.0 lookup API over the Vessel Class catalogue (issue #94). The figures are the
//! catalogue's (`catalogue/vessel_classes.yaml`), which is now the one source. Where the catalogue
//! differs from the 12 classes the core used to hold by hand, the difference is stated beside the
//! assertion.
//!
//! Label: property

use ovcd_core::vessel_standards::{built_in_catalogue, get_standard, standards_for};

#[test]
fn nominal_sizes_are_the_midpoints_and_r_min_uses_the_long_end() {
    let iv = get_standard("CEMT-IV").unwrap();
    assert_eq!(iv.loa_nominal_m(), Some(82.5));
    assert_eq!(iv.draught_nominal_m(), Some(2.5));
    assert_eq!(iv.dwt_nominal_t(), Some(1250.0));
    assert_eq!(iv.r_min_m(4.0), Some(340.0));
    let va = get_standard("CEMT-Va").unwrap();
    assert_eq!(va.loa_nominal_m(), Some(102.5));
    // The catalogue gives 2.5 to 2.8 m. The core's own list had 2.5 to 4.5 m.
    assert_eq!(va.draught_m, Some((2.5, 2.8)));
    assert_eq!(va.dwt_nominal_t(), Some(2250.0));
    assert_eq!(va.r_min_m(4.0), Some(440.0));
}

#[test]
fn a_class_with_no_length_has_no_nominal_length_or_radius() {
    let cn_v = get_standard("CN-V").unwrap();
    assert_eq!(cn_v.loa_m, None);
    assert_eq!(cn_v.loa_nominal_m(), None);
    assert_eq!(cn_v.r_min_m(3.0), None);
}

#[test]
fn the_old_ids_are_aliases_of_the_catalogue_classes() {
    for (old, new) in [
        ("CEMT_IV", "CEMT-IV"),
        ("CEMT_Va", "CEMT-Va"),
        ("CEMT_Vb", "CEMT-Vb"),
        ("CEMT_VIa", "CEMT-VIa"),
        ("CEMT_VIa_motor_NL2011", "RWS-M12"),
        ("CN_I_motor", "CN-I-MOTOR"),
        ("CN_II_motor", "CN-II-MOTOR"),
        ("CN_III_motor", "CN-III-MOTOR"),
        ("CN_IV_motor", "CN-IV-MOTOR"),
        ("CN_III_convoy_2ab", "CN-III-CONVOY-2x2"),
        ("CN_II_convoy_2ab", "CN-II-CONVOY-2x2"),
        ("CN_I_convoy_2ab", "CN-I-CONVOY-2x2"),
    ] {
        let s = get_standard(old).unwrap();
        assert_eq!(s.standard_id, new, "{old}");
        assert_eq!(s, get_standard(new).unwrap());
    }
}

#[test]
fn the_nl2011_class_is_the_catalogue_rhine_max_class_without_the_unsourced_figures() {
    let s = get_standard("CEMT_VIa_motor_NL2011").unwrap();
    assert_eq!(s.loa_m, Some((135.0, 135.0)));
    assert_eq!(s.dwt_t, Some((5601.0, 5601.0)));
    // The core's 7.0 m clearance had no source, and the catalogue gives none.
    assert_eq!(s.min_bridge_clearance_m, None);
}

#[test]
fn no_class_has_a_moulded_depth_and_cn_ii_motor_lost_its_6_5_m() {
    assert_eq!(get_standard("CN_II_motor").unwrap().moulded_depth_m, None);
    assert!(
        built_in_catalogue()
            .iter()
            .all(|(_, s)| s.moulded_depth_m.is_none())
    );
}

#[test]
fn unit_beam_is_beam_over_barges_abreast_for_a_convoy_class() {
    for (id, unit) in [
        ("CEMT-Vb", 11.4),
        ("CEMT-VIa", 11.4),
        ("CN-III-CONVOY-2x2", 10.8),
        ("CN-II-CONVOY-2x2", 16.2),
        ("CN-I-CONVOY-2x2", 16.2),
    ] {
        assert_eq!(get_standard(id).unwrap().unit_beam_m, Some(unit), "{id}");
    }
    assert_eq!(get_standard("CEMT-Va").unwrap().unit_beam_m, None);
}

#[test]
fn the_bridge_clearance_is_the_bottom_of_the_class_air_draft_range() {
    let va = get_standard("CEMT-Va").unwrap();
    assert_eq!(va.air_draft_m, Some((5.25, 9.1)));
    assert_eq!(va.min_bridge_clearance_m, Some(5.25));
    assert_eq!(
        get_standard("CEMT-VIa").unwrap().min_bridge_clearance_m,
        Some(7.0)
    );
}

#[test]
fn standards_for_no_longer_filters_by_cargo_but_still_checks_the_mode() {
    let all = built_in_catalogue().len();
    assert_eq!(all, 292);
    assert_eq!(standards_for("dry_bulk").unwrap().len(), all);
    assert_eq!(standards_for("liquid_gas").unwrap().len(), all);
    assert_eq!(
        standards_for("grain").unwrap_err(),
        "cargo_mode must be dry_bulk or liquid_gas, got 'grain'"
    );
}

#[test]
fn an_unknown_id_names_the_known_ones() {
    let e = get_standard("CEMT_Zz").unwrap_err();
    assert!(
        e.starts_with("unknown vessel standard 'CEMT_Zz'; known: "),
        "{e}"
    );
    assert!(e.contains("CEMT-Va"));
}

#[test]
fn the_convoy_beam_check_fails_for_a_supplied_wrong_unit_beam_and_says_it_is_an_identity_for_the_built_in_one()
 {
    use ovcd_core::vessel_standards::validate_catalogue;
    let find = |checks: &[ovcd_core::checks::Check]| {
        checks
            .iter()
            .find(|c| c.name == "convoy_beam_consistent")
            .unwrap()
            .clone()
    };
    // The built-in catalogue derives its unit beam from the convoy beam: the check passes, and its
    // message says it is an identity there (issue #432).
    let built_in = built_in_catalogue();
    let ok = find(&validate_catalogue(&built_in));
    assert!(ok.ok);
    assert!(ok.message.contains("identity"), "{}", ok.message);
    // A supplied unit beam is independent: a wrong one fails the check.
    let mut bad = built_in.clone();
    let (_, convoy) = bad
        .iter_mut()
        .find(|(_, v)| v.unit_beam_m.is_some() && v.n_abreast > 1)
        .expect("a convoy class");
    convoy.unit_beam_m = convoy.unit_beam_m.map(|u| u + 1.0);
    assert!(!find(&validate_catalogue(&bad)).ok);
}
