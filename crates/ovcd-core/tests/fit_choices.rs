//! Seam: the gas arrangement search takes the user's tanks across and least side space, and says
//! what became of every arrangement it tried (issue #675).
//!
//! Label: property. Every test states a rule: the answer keeps to a fixed number of tanks across or
//! says why it cannot, a least side space is kept and makes a wing tank, each arrangement tried has
//! one outcome and a failing one names its checks. No number here is a printed result.

use ovcd_core::barge_capacity::DEFAULT_WING_TANK_GAP_M;
use ovcd_core::gas_arrangement::{FitOptions, TriedOutcome, fit, wing_tank_width};

/// The hull of the issue: 140 x 29 m, LNG, one depth, with the ES-TRIN 7.02(2) Blind Distance
/// limit of 250 m (a Design sets it; the core default has none).
fn options(depth: f64) -> FitOptions {
    FitOptions {
        depths: vec![depth],
        sightline_limit_m: Some(250.0),
        ..FitOptions::default()
    }
}

const LOA: f64 = 140.0;
const BEAM: f64 = 29.0;

#[test]
fn a_fixed_number_of_tanks_across_is_the_only_one_tried() {
    for n in [1, 2, 3] {
        let o = FitOptions {
            tanks_wide: vec![n],
            ..options(9.0)
        };
        let f = fit(LOA, BEAM, "LNG", 5.0, &o).unwrap();
        assert_eq!(f.n_tanks_wide, n, "the answer keeps {n} across");
        assert!(!f.tried.is_empty());
        assert!(
            f.tried.iter().all(|t| t.n_wide == n),
            "only {n} across is tried"
        );
    }
}

#[test]
fn a_fixed_number_that_cannot_pass_is_infeasible_and_says_why() {
    // Two across on the 9 m hull: the tanks are too tall for the wheelhouse to see over with the
    // core's lift, so no candidate passes. The answer says so, and so does the row.
    let o = FitOptions {
        tanks_wide: vec![2],
        ..options(9.0)
    };
    let f = fit(LOA, BEAM, "LNG", 5.0, &o).unwrap();
    assert!(!f.feasible);
    assert!(!f.failed_checks.is_empty());
    let row = &f.tried[0];
    match &row.outcome {
        TriedOutcome::Fails(checks) => {
            assert!(!checks.is_empty());
            assert!(checks.iter().all(|c| !c.ok && c.is_error()));
        }
        other => panic!("an infeasible answer's row fails, not {other:?}"),
    }
}

#[test]
fn a_least_side_space_is_kept_and_makes_a_wing_tank() {
    let plain = fit(LOA, BEAM, "LNG", 5.0, &options(9.0)).unwrap();
    let min = 2.0;
    let o = FitOptions {
        min_side_space_m: min,
        ..options(9.0)
    };
    let f = fit(LOA, BEAM, "LNG", 5.0, &o).unwrap();
    assert!(
        f.side_clearance_m + 1e-9 >= min,
        "side {} m",
        f.side_clearance_m
    );
    let wing = wing_tank_width(&f.engine_result);
    assert!(
        (wing - (f.side_clearance_m - DEFAULT_WING_TANK_GAP_M)).abs() < 1e-9,
        "the wing tank is the side space less the gap"
    );
    assert!(wing > 0.0);
    // Every built candidate keeps the least side space, not only the answer.
    for t in f.tried.iter().filter(|t| t.tank_od_m > 0.0) {
        assert!(t.side_clearance_m + 1e-9 >= min, "{t:?}");
    }
    // With no least side space the answer may sit on the IGC minimum, under the gap.
    assert!(plain.side_clearance_m <= f.side_clearance_m + 1e-9);
}

#[test]
fn a_side_space_with_no_room_for_a_tank_is_not_built_with_a_reason() {
    let o = FitOptions {
        min_side_space_m: 14.0,
        ..options(9.0)
    };
    let f = fit(LOA, BEAM, "LNG", 5.0, &o).unwrap();
    assert!(!f.feasible);
    assert!(f.tried.iter().all(|t| match &t.outcome {
        TriedOutcome::NotBuilt(why) => !why.en.is_empty(),
        TriedOutcome::Fails(checks) => !checks.is_empty(),
        _ => false,
    }));
}

#[test]
fn every_arrangement_tried_has_one_outcome_and_the_answer_is_chosen() {
    for depth in [9.0, 12.0] {
        let f = fit(LOA, BEAM, "LNG", 5.0, &options(depth)).unwrap();
        // One row per number of tanks across, in the search's order.
        let across: Vec<i64> = f.tried.iter().map(|t| t.n_wide).collect();
        assert_eq!(across, vec![1, 2, 3], "depth {depth}");
        let chosen: Vec<_> = f
            .tried
            .iter()
            .filter(|t| t.outcome == TriedOutcome::Chosen)
            .collect();
        if f.feasible {
            assert_eq!(chosen.len(), 1, "depth {depth}");
            assert_eq!(chosen[0].n_wide, f.n_tanks_wide);
            assert_eq!(chosen[0].tank_od_m, f.tank_od_m);
            assert_eq!(chosen[0].side_clearance_m, f.side_clearance_m);
        } else {
            assert!(chosen.is_empty());
        }
        for t in &f.tried {
            assert!(t.variants >= 1, "{t:?}");
            if let TriedOutcome::Fails(checks) = &t.outcome {
                assert!(!checks.is_empty(), "a failing row names its checks: {t:?}");
            }
            if t.outcome == TriedOutcome::LessCargo {
                assert!(t.cargo_t <= f.payload_t + 1e-6, "{t:?}");
            }
        }
    }
}

#[test]
fn a_failing_blind_distance_says_the_lift_needed_beside_the_lift_allowed() {
    let f = fit(LOA, BEAM, "LNG", 5.0, &options(9.0)).unwrap();
    let blind: Vec<_> = f
        .tried
        .iter()
        .filter_map(|t| match &t.outcome {
            TriedOutcome::Fails(c) => c.iter().find(|c| c.name == "wheelhouse_sees_over_cargo"),
            _ => None,
        })
        .collect();
    assert!(
        !blind.is_empty(),
        "the 9 m hull turns an arrangement down on the blind distance"
    );
    for c in blind {
        assert!(c.message.contains("lift needed"), "{}", c.message);
        assert!(c.message.contains(" m of "), "{}", c.message);
    }
}
