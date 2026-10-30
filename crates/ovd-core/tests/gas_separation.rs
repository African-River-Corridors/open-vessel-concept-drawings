//! Seam: the public `ovd_core::gas_separation` API. Expected values are printed by the original
//! Python (`core/gas_separation.py`), not worked out by hand, except the aft end-bulkhead space check,
//! which replaced the type C cofferdam check in issue #280 (Label: sourced - LR inland rules Pt 4 Ch 4
//! 3.5.3 and ADN 2025 9.3.1.11.3(a): 0.20 m, or 0.50 m for plane tank ends).

use ovd_core::gas_separation::{
    COFFERDAM_MIN_M, END_BULKHEAD_SPACE_M, END_BULKHEAD_SPACE_PRESSURE_M, GasSeparationInputs,
    OPENING_SETBACK_M, compute, default_end_space,
};

const CLAUSE_WARNING: &str = "Separation clauses are cited from the classification rules for \
inland-waterways ships (Pt 4 Ch 4 Sec 3, Jul 2022). Confirm against the ADN text before release.";

#[test]
fn constants_are_the_cited_values() {
    assert_eq!(COFFERDAM_MIN_M, 0.60);
    assert_eq!(OPENING_SETBACK_M, 2.00);
    assert_eq!(END_BULKHEAD_SPACE_M, 0.50);
    assert_eq!(END_BULKHEAD_SPACE_PRESSURE_M, 0.20);
}

#[test]
fn the_defaults_are_the_type_g_rule_for_domed_ends() {
    // Issue #280 (maintainer decision, 2026-10-22): the end space defaults to the type G rule, 0.20 m
    // for domed tank ends and 0.50 m for plane ends, in place of the 0.60 m type C cofferdam the
    // original Python used. The Python constant `COFFERDAM_MIN_M` keeps its value.
    assert_eq!(default_end_space(false), 0.20);
    assert_eq!(default_end_space(true), 0.50);
    let i = GasSeparationInputs::default();
    assert_eq!(i.cofferdam_m, 0.20);
    assert_eq!(i.opening_setback_m, 2.00);
    assert_eq!(i.fwd_end_bulkhead_m, 0.20);
    // Issue #381: the setback is measured from the cargo area.
    assert!(i.setback_from_cargo_area);
    assert!(i.is_pressure_tank);
    assert_eq!(i.side_clearance_m, 0.0);
    assert_eq!(i.double_bottom_m, 0.0);
}

#[test]
fn golden_defaults() {
    let r = compute(&GasSeparationInputs::default()).unwrap();
    // Issue #381: the aft separation is the end space plus the 2.00 m setback from the cargo area.
    assert_eq!(r.aft_separation_m, 2.2);
    assert_eq!(r.fwd_separation_m, 0.2);
    assert_eq!(r.cofferdam_m, 0.2);
    assert_eq!(r.service_space_m, 2.0);
    assert!((r.total_length_taken_m - 2.4).abs() < 1e-12);
    assert!(r.ok());
    assert_eq!(r.warnings, vec![CLAUSE_WARNING]);
    let names: Vec<&str> = r.checks.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "aft_end_bulkhead_space",
            "opening_setback",
            "fwd_end_bulkhead_space"
        ]
    );
    assert_eq!(
        r.checks[0].message,
        "aft end-bulkhead space 0.20 m >= 0.20 m (domed tank ends, cl. 3.5.3)"
    );
    assert_eq!(
        r.checks[1].message,
        "distance 2.00 m from the cargo area (end space included) to any entrance or openable \
         window >= 2.00 m (cl. 3.3.7 / 3.3.11)"
    );
    assert_eq!(
        r.checks[2].message,
        "forward end-bulkhead space 0.20 m >= 0.20 m (domed tank ends, cl. 3.5.3)"
    );
}

#[test]
fn a_thin_aft_space_fails_and_the_setback_still_governs_aft() {
    let r = compute(&GasSeparationInputs {
        cofferdam_m: 0.1,
        ..Default::default()
    })
    .unwrap();
    assert!(!r.ok());
    assert_eq!(r.aft_separation_m, 2.1);
    assert_eq!(r.service_space_m, 2.0);
    assert_eq!(
        r.checks[0].message,
        "aft end-bulkhead space 0.10 m >= 0.20 m (domed tank ends, cl. 3.5.3)"
    );
}

/// Issue #280: the 0.60 m cofferdam of the type C part (3.6.13) is not a limit on a gas Design. The
/// type G space is 0.20 m for domed ends, at and just below the limit, and 0.50 m for plane ends.
#[test]
fn the_aft_end_space_is_the_type_g_limit_not_the_type_c_cofferdam() {
    let check = |cofferdam_m: f64, plane: bool| {
        compute(&GasSeparationInputs {
            cofferdam_m,
            plane_tank_ends: plane,
            ..Default::default()
        })
        .unwrap()
        .checks[0]
            .ok
    };
    assert!(check(0.4, false), "0.40 m is enough for domed ends");
    assert!(check(0.20, false));
    assert!(!check(0.19, false));
    assert!(!check(0.4, true), "plane ends need 0.50 m");
    assert!(check(0.50, true));
    assert!(!check(0.49, true));
}

#[test]
fn a_short_setback_fails_the_opening_check() {
    let r = compute(&GasSeparationInputs {
        opening_setback_m: 1.0,
        ..Default::default()
    })
    .unwrap();
    assert!(!r.ok());
    assert_eq!(r.aft_separation_m, 1.2);
    assert_eq!(r.service_space_m, 1.0);
    assert_eq!(r.total_length_taken_m, 1.4);
}

#[test]
fn a_wider_cofferdam_adds_to_the_separation_and_the_service_space_stays_the_setback() {
    let r = compute(&GasSeparationInputs {
        cofferdam_m: 3.0,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(r.aft_separation_m, 5.0);
    assert_eq!(r.service_space_m, 2.0);
    assert_eq!(r.total_length_taken_m, 5.2);
}

/// Issue #215: the inland type-G floors are gone. Clearances given in the inputs add no check, however
/// small they are; the three separation checks are all there is.
#[test]
fn as_designed_clearances_add_no_check() {
    let r = compute(&GasSeparationInputs {
        fwd_end_bulkhead_m: 0.3,
        plane_tank_ends: true,
        side_clearance_m: 0.05,
        double_bottom_m: 0.05,
        ..Default::default()
    })
    .unwrap();
    assert!(!r.ok());
    assert_eq!(r.total_length_taken_m, 2.5);
    let names: Vec<&str> = r.checks.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "aft_end_bulkhead_space",
            "opening_setback",
            "fwd_end_bulkhead_space"
        ]
    );
    assert_eq!(
        r.checks[2].message,
        "forward end-bulkhead space 0.30 m >= 0.50 m (plane tank ends, cl. 3.5.3)"
    );
    assert!(r.checks.iter().all(|c| !c.name.contains("type_g")));
}

/// Issue #281: the space is keyed to the tank end shape (3.5.3), not to the tank kind. A tank that
/// is not a pressure tank but has domed ends needs 0.20 m, as a pressure tank does.
#[test]
fn the_end_space_follows_the_tank_ends_not_the_tank_kind() {
    for is_pressure_tank in [true, false] {
        let r = compute(&GasSeparationInputs {
            fwd_end_bulkhead_m: 0.3,
            is_pressure_tank,
            ..Default::default()
        })
        .unwrap();
        assert!(
            r.checks[2].ok,
            "domed ends, pressure tank {is_pressure_tank}"
        );
    }
    let plane = compute(&GasSeparationInputs {
        fwd_end_bulkhead_m: 0.3,
        plane_tank_ends: true,
        ..Default::default()
    })
    .unwrap();
    assert!(!plane.checks[2].ok);
}

/// The checks allow a 1e-9 slack, so a value a hair under the floor still passes.
#[test]
fn the_checks_allow_one_nano_metre_of_slack() {
    let r = compute(&GasSeparationInputs {
        cofferdam_m: 0.2 - 1e-10,
        ..Default::default()
    })
    .unwrap();
    assert!(r.checks[0].ok);
    let r = compute(&GasSeparationInputs {
        cofferdam_m: 0.2 - 1e-8,
        ..Default::default()
    })
    .unwrap();
    assert!(!r.checks[0].ok);
}

#[test]
fn negative_separations_raise_the_python_message() {
    for bad in [
        GasSeparationInputs {
            cofferdam_m: -0.1,
            ..Default::default()
        },
        GasSeparationInputs {
            opening_setback_m: -0.1,
            ..Default::default()
        },
        GasSeparationInputs {
            fwd_end_bulkhead_m: -0.1,
            ..Default::default()
        },
    ] {
        assert_eq!(
            compute(&bad).unwrap_err(),
            "separations must be non-negative"
        );
    }
}

/// `max` returns the first of equal items. `0.0` and `-0.0` are equal but differ in sign;
/// Python gives `0.0` here, so a port that takes the second item would show `-0.0`.
#[test]
fn max_keeps_the_first_zero() {
    let r = compute(&GasSeparationInputs {
        cofferdam_m: 0.0,
        opening_setback_m: -0.0,
        ..Default::default()
    })
    .unwrap();
    assert!(r.aft_separation_m.is_sign_positive());
    assert!(r.service_space_m.is_sign_positive());
}

/// Zero is a valid separation: only a negative one raises.
#[test]
fn zero_separations_are_valid_and_fail_their_checks() {
    let r = compute(&GasSeparationInputs {
        cofferdam_m: 0.0,
        opening_setback_m: 0.0,
        fwd_end_bulkhead_m: 0.0,
        ..Default::default()
    })
    .unwrap();
    let oks: Vec<bool> = r.checks.iter().map(|c| c.ok).collect();
    assert_eq!(oks, [false, false, false]);
}

// --- Issue #381: the setback measured from the cargo area ---

#[test]
fn a_deep_cofferdam_no_longer_stands_in_for_the_setback_to_the_openings() {
    // 3.3.7 and 3.3.11 measure from the cargo area, which holds the cofferdam. A 3.00 m cofferdam
    // with a 1.00 m setback put the first opening 1.00 m from the cargo area: it fails. Measured from
    // the tank end (the earlier reading) the aft separation was 3.00 m and it passed.
    let inputs = |from_cargo_area| GasSeparationInputs {
        cofferdam_m: 3.0,
        opening_setback_m: 1.0,
        setback_from_cargo_area: from_cargo_area,
        ..Default::default()
    };
    let earlier = compute(&inputs(false)).unwrap();
    assert_eq!(earlier.aft_separation_m, 3.0);
    assert!(
        earlier
            .checks
            .iter()
            .find(|c| c.name == "opening_setback")
            .unwrap()
            .ok
    );
    let now = compute(&inputs(true)).unwrap();
    assert_eq!(now.aft_separation_m, 4.0);
    assert_eq!(now.service_space_m, 1.0);
    let check = now
        .checks
        .iter()
        .find(|c| c.name == "opening_setback")
        .unwrap();
    assert!(!check.ok);
    assert_eq!(
        check.message,
        "distance 1.00 m from the cargo area (end space included) to any entrance or openable \
         window >= 2.00 m (cl. 3.3.7 / 3.3.11)"
    );
}
