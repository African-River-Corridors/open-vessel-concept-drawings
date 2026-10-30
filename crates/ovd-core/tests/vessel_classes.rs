//! Seam: the Vessel Class ladder of `ovd_core::vessel_classes` (issue #94). Each test states a
//! rule the maintainer gave: length and beam decide the class; a class's figures are maxima; motor
//! and convoy classes are separate ladders; past the largest class there is a flag; the scheme never
//! changes by itself.
//!
//! Label: property

use ovd_core::vessel_classes::{CLASSES, find, formation_kind, ladder, place, schemes};

#[test]
fn the_formation_words_map_to_the_kinds_a_design_has() {
    assert_eq!(formation_kind("motor_vessel"), Some("self_propelled"));
    assert_eq!(formation_kind("pushed_barge"), Some("pushed_barge"));
    assert_eq!(formation_kind("pushed_convoy"), Some("pushed_convoy"));
    assert_eq!(formation_kind("coupled_unit"), Some("pushed_convoy"));
    // Issue #357: the ATB and the towed barge are Formation kinds.
    assert_eq!(formation_kind("atb"), Some("atb"));
    assert_eq!(formation_kind("towed_barge"), Some("towed_barge"));
    for word in ["pusher", "towboat", "tug"] {
        assert_eq!(formation_kind(word), None, "{word}");
    }
}

#[test]
fn a_class_with_no_length_or_beam_or_kind_cannot_be_picked() {
    assert!(!find("CN-V").unwrap().pickable()); // no length or beam
    let tug = CLASSES.iter().find(|c| c.formation == "tug").unwrap();
    assert!(!tug.pickable());
    assert!(find("CEMT-Va").unwrap().pickable());
    // The 35 classes with barge counts: all of them pushed convoys or coupled units.
    let counted = CLASSES
        .iter()
        .filter(|c| c.n_abreast.unwrap_or(1) > 1 || c.n_in_line.unwrap_or(1) > 1)
        .count();
    assert_eq!(counted, 35);
}

#[test]
fn a_class_gives_its_maxima_and_the_two_limits() {
    let va = find("CEMT-Va").unwrap();
    assert_eq!((va.loa_max_m(), va.beam_max_m()), (Some(110.0), Some(11.4)));
    assert_eq!(va.draught_limit_m(), Some(2.8)); // the top of the range
    assert_eq!(va.air_draft_limit_m(), Some(5.25)); // the bottom of the range
    let cn = find("CN-II-MOTOR").unwrap();
    assert_eq!(cn.draught_limit_m(), Some(2.6)); // a scalar is used as it is
    assert_eq!(cn.air_draft_limit_m(), None); // null fills nothing
}

#[test]
fn motor_and_convoy_classes_are_separate_ladders_of_one_scheme() {
    let motor = ladder("CEMT-1992", "self_propelled");
    let convoy = ladder("CEMT-1992", "pushed_convoy");
    assert!(motor.iter().all(|c| c.formation == "motor_vessel"));
    assert!(convoy.iter().all(|c| c.kind() == Some("pushed_convoy")));
    assert!(motor.iter().any(|c| c.id == "CEMT-Va"));
    assert!(convoy.iter().any(|c| c.id == "CEMT-VIa"));
    assert!(!convoy.iter().any(|c| c.id == "CEMT-Va"));
}

#[test]
fn the_ladder_runs_from_the_smallest_class_up_and_cemt_regional_variants_share_it() {
    let motor = ladder("CEMT-1992", "self_propelled");
    let lengths: Vec<f64> = motor.iter().map(|c| c.loa_max_m().unwrap()).collect();
    assert!(lengths.windows(2).all(|w| w[0] <= w[1]), "{lengths:?}");
    // The -W and -E classes are on the one ladder with the unmarked classes.
    for id in ["CEMT-I-W", "CEMT-II-E", "CEMT-IV", "CEMT-Va"] {
        assert!(motor.iter().any(|c| c.id == id), "{id}");
    }
}

#[test]
fn a_vessel_sits_in_the_smallest_class_whose_length_and_beam_both_hold_it() {
    let at = |loa: f64, beam: f64| place("CEMT-1992", "self_propelled", loa, beam, None).unwrap();
    let va = at(110.0, 11.4);
    assert_eq!(va.class.id, "CEMT-Va");
    assert!(!va.beyond_largest);
    // Past a maximum the vessel moves up; below the next class down's maximum it moves down.
    assert_ne!(at(110.5, 11.4).class.id, "CEMT-Va");
    assert!(at(110.5, 11.4).class.loa_max_m().unwrap() > 110.0);
    assert_ne!(at(100.0, 11.5).class.id, "CEMT-Va");
    let smaller = at(84.0, 9.0);
    assert_eq!(smaller.class.id, "CEMT-IV");
    assert!(at(60.0, 8.0).class.loa_max_m().unwrap() < 85.0);
}

#[test]
fn past_the_largest_class_the_vessel_stays_in_it_and_is_flagged() {
    let big = place("CEMT-1992", "self_propelled", 400.0, 30.0, None).unwrap();
    assert!(big.beyond_largest);
    let largest = ladder("CEMT-1992", "self_propelled").pop().unwrap();
    assert_eq!(big.class.id, largest.id);
    // Wide but short is beyond too: both maxima have to hold.
    assert!(
        place("CEMT-1992", "self_propelled", 90.0, 40.0, None)
            .unwrap()
            .beyond_largest
    );
}

#[test]
fn the_scheme_has_to_have_a_class_of_the_kind() {
    assert!(place("CEMT-1992", "pushed_barge", 60.0, 10.0, None).is_none());
    assert!(place("no-such-scheme", "self_propelled", 60.0, 10.0, None).is_none());
}

#[test]
fn picking_a_class_and_placing_its_maxima_gives_the_class_back_for_every_class() {
    let mut n = 0;
    for c in CLASSES.iter().filter(|c| c.pickable()) {
        let p = place(
            c.family,
            c.kind().unwrap(),
            c.loa_max_m().unwrap(),
            c.beam_max_m().unwrap(),
            Some(c.id),
        )
        .unwrap();
        assert_eq!(p.class.id, c.id, "{}", c.id);
        assert!(!p.beyond_largest, "{}", c.id);
        n += 1;
    }
    assert!(n > 200, "{n}");
}

#[test]
fn every_scheme_listed_has_a_pickable_class_and_none_is_left_out() {
    let list = schemes();
    for s in &list {
        assert!(
            CLASSES.iter().any(|c| c.family == *s && c.pickable()),
            "{s}"
        );
    }
    for c in CLASSES.iter().filter(|c| c.pickable()) {
        assert!(list.contains(&c.family), "{}", c.family);
    }
}

#[test]
fn cargo_modes_are_catalogue_data_and_do_not_filter_a_ladder() {
    // CN-GB50139 lists dry_bulk only; the ladder holds its classes whatever the Cargo.
    assert!(find("CN-II-MOTOR").unwrap().cargo_modes == ["dry_bulk"]);
    assert!(
        ladder("CN-GB50139", "self_propelled")
            .iter()
            .any(|c| c.id == "CN-II-MOTOR")
    );
}
