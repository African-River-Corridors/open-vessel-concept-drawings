//! Label: sourced - ADN 2025 (ECE/TRANS/351) 3.2.3.3 flowchart for liquids of Classes 3, 6.1, 8 and 9
//! in tanks in inland navigation, and 3.2.3.2 Table C, UN 3494 column (6). One test per branch.

use ovcd_core::liquid_bulk::adn_vessel_type::{
    AdnFlags, AdnVesselType, Aquatic, Branch, Flag, derive, effective,
};
use ovcd_core::liquid_bulk::crude_grade::DEFAULT_GRADE;

/// A crude with every hazard known and absent: flammable, flash point below 23 deg C, nothing else.
fn plain() -> AdnFlags {
    AdnFlags {
        sour: false,
        cmr: Flag::No,
        benzene_over_10_percent: Flag::No,
        aquatic: Aquatic::None,
        flash_point_below_23c: Flag::Yes,
        vapour_pressure_50c_at_least_1kpa: Flag::Yes,
    }
}

#[test]
fn un_3494_sour_crude_is_type_c() {
    let r = derive(&AdnFlags {
        sour: true,
        ..plain()
    });
    assert_eq!((r.vessel_type, r.branch), (AdnVesselType::C, Branch::Sour));
}

#[test]
fn sour_is_type_c_even_when_every_other_flag_is_clear_or_unknown() {
    for cmr in [Flag::No, Flag::Unknown] {
        let r = derive(&AdnFlags {
            sour: true,
            cmr,
            ..plain()
        });
        assert_eq!(r.vessel_type, AdnVesselType::C);
    }
}

#[test]
fn the_default_grade_has_unknown_flags_so_it_is_type_c() {
    let r = derive(&AdnFlags::of(&DEFAULT_GRADE));
    assert_eq!(
        (r.vessel_type, r.branch),
        (AdnVesselType::C, Branch::UnknownDefaultsToC)
    );
    assert!(r.walls_distinct_from_hull);
}

#[test]
fn each_unknown_flag_alone_gives_type_c() {
    let cases = [
        AdnFlags {
            cmr: Flag::Unknown,
            ..plain()
        },
        AdnFlags {
            benzene_over_10_percent: Flag::Unknown,
            ..plain()
        },
        AdnFlags {
            aquatic: Aquatic::Unknown,
            ..plain()
        },
        AdnFlags {
            flash_point_below_23c: Flag::Unknown,
            ..plain()
        },
        AdnFlags {
            aquatic: Aquatic::N1,
            vapour_pressure_50c_at_least_1kpa: Flag::Unknown,
            ..plain()
        },
    ];
    for f in cases {
        assert_eq!(derive(&f).branch, Branch::UnknownDefaultsToC, "{f:?}");
    }
}

#[test]
fn an_unknown_vapour_pressure_does_not_matter_unless_the_crude_is_n1() {
    let f = AdnFlags {
        vapour_pressure_50c_at_least_1kpa: Flag::Unknown,
        ..plain()
    };
    assert_eq!(derive(&f).vessel_type, AdnVesselType::NClosed);
}

#[test]
fn benzene_over_10_percent_is_type_c() {
    let r = derive(&AdnFlags {
        benzene_over_10_percent: Flag::Yes,
        ..plain()
    });
    assert_eq!(
        (r.vessel_type, r.branch),
        (AdnVesselType::C, Branch::Benzene)
    );
}

#[test]
fn aquatic_n1_with_vapour_pressure_of_1_kpa_or_more_is_type_c() {
    let f = AdnFlags {
        aquatic: Aquatic::N1,
        ..plain()
    };
    let r = derive(&f);
    assert_eq!(
        (r.vessel_type, r.branch),
        (AdnVesselType::C, Branch::AquaticN1HighVapourPressure)
    );
}

#[test]
fn aquatic_n1_with_vapour_pressure_below_1_kpa_is_type_n_closed_distinct() {
    let f = AdnFlags {
        aquatic: Aquatic::N1,
        vapour_pressure_50c_at_least_1kpa: Flag::No,
        ..plain()
    };
    let r = derive(&f);
    assert_eq!(
        (r.vessel_type, r.branch),
        (AdnVesselType::NClosed, Branch::ClosedDistinct)
    );
    assert!(r.walls_distinct_from_hull);
}

#[test]
fn cmr_is_type_n_closed_with_walls_distinct_from_the_hull() {
    let r = derive(&AdnFlags {
        cmr: Flag::Yes,
        ..plain()
    });
    assert_eq!(
        (r.vessel_type, r.branch),
        (AdnVesselType::NClosed, Branch::ClosedDistinct)
    );
    assert!(r.walls_distinct_from_hull);
}

#[test]
fn aquatic_n2_with_flash_point_below_23_is_type_n_closed_distinct() {
    let r = derive(&AdnFlags {
        aquatic: Aquatic::N2,
        ..plain()
    });
    assert_eq!(
        (r.vessel_type, r.branch),
        (AdnVesselType::NClosed, Branch::ClosedDistinct)
    );
}

#[test]
fn a_flammable_crude_with_no_other_hazard_is_type_n_closed_and_the_walls_may_form_the_hull() {
    let r = derive(&plain());
    assert_eq!(
        (r.vessel_type, r.branch),
        (AdnVesselType::NClosed, Branch::ClosedHull)
    );
    assert!(!r.walls_distinct_from_hull);
}

#[test]
fn flash_point_of_23_or_more_with_no_hazard_is_type_n_open() {
    let r = derive(&AdnFlags {
        flash_point_below_23c: Flag::No,
        ..plain()
    });
    assert_eq!(
        (r.vessel_type, r.branch),
        (AdnVesselType::NOpen, Branch::Open)
    );
    assert!(!r.walls_distinct_from_hull);
}

#[test]
fn flash_point_of_23_or_more_with_n2_is_type_n_open_with_walls_distinct() {
    let f = AdnFlags {
        flash_point_below_23c: Flag::No,
        aquatic: Aquatic::N2,
        ..plain()
    };
    let r = derive(&f);
    assert_eq!(
        (r.vessel_type, r.branch),
        (AdnVesselType::NOpen, Branch::Open)
    );
    assert!(r.walls_distinct_from_hull);
}

#[test]
fn cmr_beats_a_high_flash_point() {
    let f = AdnFlags {
        flash_point_below_23c: Flag::No,
        cmr: Flag::Yes,
        ..plain()
    };
    assert_eq!(derive(&f).vessel_type, AdnVesselType::NClosed);
}

#[test]
fn benzene_beats_cmr() {
    let f = AdnFlags {
        cmr: Flag::Yes,
        benzene_over_10_percent: Flag::Yes,
        ..plain()
    };
    assert_eq!(derive(&f).vessel_type, AdnVesselType::C);
}

#[test]
fn a_user_override_wins_and_says_so() {
    let r = effective(&AdnFlags::of(&DEFAULT_GRADE), Some(AdnVesselType::NClosed));
    assert_eq!(
        (r.vessel_type, r.branch),
        (AdnVesselType::NClosed, Branch::Override)
    );
    assert_eq!(effective(&plain(), None), derive(&plain()));
}

#[test]
fn the_type_keys_read_back() {
    for t in AdnVesselType::ALL {
        assert_eq!(AdnVesselType::from_key(t.key()), Some(t));
    }
    assert_eq!(AdnVesselType::from_key("N"), None);
}
