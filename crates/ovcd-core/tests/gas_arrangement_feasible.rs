//! Seam: the public `ovcd_core::gas_arrangement` API, issue #264: `fit` returns an arrangement that
//! meets every check (stability included), feasible first and then most cargo; when none does it says
//! so (`feasible == false`) and names the failing checks of the closest candidate.
//!
//! Label: property
//!
//! No test here pins a calculated number as engineering evidence. The hulls are the design-impact
//! cases of the repo; the payloads in the comments are what the code printed before and after the
//! change (the 2,808 t of the old `fit` is the `cargo_t` the design-impact base run printed for
//! "sweep gas passes all checks IGC_2G 110x11.4").

use ovcd_core::checks::all_ok;
use ovcd_core::gas_arrangement::{ArrangementFit, FitOptions, fit, smallest_viable_loa};
use ovcd_core::gas_stability::{self as gs, GasCapacityResult};

/// The stability of a returned arrangement, assessed again on its own, not through `fit`.
fn stability_ok(f: &ArrangementFit) -> bool {
    let basis = GasCapacityResult::try_from(&f.engine_result).unwrap();
    gs::assess(
        &basis,
        f.payload_t,
        0.85,
        gs::DEFAULT_WATER_DENSITY,
        gs::criteria_set_for(&f.ruleset),
    )
    .unwrap()
    .ok()
}

fn options(depth: f64) -> FitOptions {
    FitOptions {
        depths: vec![depth],
        fill_ratio: 0.98,
        ..FitOptions::default()
    }
}

#[test]
fn the_most_cargo_arrangement_fails_stability_and_a_smaller_one_that_passes_is_returned() {
    // 110 x 11.4 x 6.0 m LPG propane, draught cap 5.7 m. Only the base arrangement (no extra side
    // clearance) is searched: it is the one the old `fit` returned (2,808 t) and it fails laden stability.
    let only_most_cargo = FitOptions {
        extra_clearance_m: vec![],
        ..options(6.0)
    };
    let worst = fit(110.0, 11.4, "LPG_propane", 5.7, &only_most_cargo).unwrap();
    assert!(!worst.feasible);
    assert!(worst.payload_t > 2808.0);
    assert!(
        worst
            .failed_checks
            .iter()
            .any(|c| c.starts_with("Stability \u{2014} laden")),
        "{:?}",
        worst.failed_checks
    );
    assert!(!stability_ok(&worst));

    // The whole search keeps the arrangements that pass and takes the one with most cargo.
    let f = fit(110.0, 11.4, "LPG_propane", 5.7, &options(6.0)).unwrap();
    assert!(f.feasible, "{:?}", f.failed_checks);
    assert!(f.failed_checks.is_empty());
    assert!(stability_ok(&f));
    assert!(all_ok(&f.engine_result.checks));
    assert!(f.payload_t < worst.payload_t);
    assert!(f.payload_t > 0.0);
    // The wider side clearance (a smaller tank) is what made it stable.
    assert!(f.side_clearance_m > worst.side_clearance_m);
    assert!(f.tank_od_m < worst.tank_od_m);
}

#[test]
fn a_hull_where_nothing_passes_is_infeasible_and_lists_the_failing_checks() {
    // 60 x 10 x 6 m LPG propane: every arrangement fails the light stability criteria.
    let f = fit(60.0, 10.0, "LPG_propane", 5.7, &options(6.0)).unwrap();
    assert!(!f.feasible);
    assert!(f.payload_t > 0.0, "the closest candidate is still shown");
    assert!(!f.failed_checks.is_empty());
    assert!(
        f.failed_checks
            .iter()
            .all(|c| c.starts_with("Stability \u{2014} ")),
        "{:?}",
        f.failed_checks
    );
    assert!(
        f.failed_checks
            .iter()
            .any(|c| c.starts_with("Stability \u{2014} light")),
        "{:?}",
        f.failed_checks
    );
    assert!(!stability_ok(&f));
    // Nothing in the search passes it either: the viable length is beyond this hull's own.
    assert!(
        smallest_viable_loa(10.0, "LPG_propane", 5.7, 20.0, 60.0, &options(6.0))
            .unwrap()
            .is_infinite()
    );
}

#[test]
fn the_closest_candidate_is_the_one_with_the_fewest_failed_checks() {
    // 60 x 20 x 3.5 m LPG propane fails three stability conditions in every arrangement; the
    // closest one has no more failures than any single arrangement searched alone.
    let all = fit(60.0, 20.0, "LPG_propane", 3.2, &options(3.5)).unwrap();
    assert!(!all.feasible);
    for nw in [1, 2, 3] {
        let one = fit(
            60.0,
            20.0,
            "LPG_propane",
            3.2,
            &FitOptions {
                tanks_wide: vec![nw],
                ..options(3.5)
            },
        )
        .unwrap();
        assert!(all.failed_checks.len() <= one.failed_checks.len() || one.payload_t == 0.0);
    }
}

#[test]
fn a_candidate_that_over_runs_the_air_draft_limit_does_not_pass() {
    // The light air draft of this hull is over 1 m, so no arrangement meets a 1 m limit.
    let o = FitOptions {
        air_draft_limit_m: Some(1.0),
        ..options(6.0)
    };
    let f = fit(110.0, 11.4, "LPG_propane", 5.7, &o).unwrap();
    assert!(!f.feasible);
    assert!(
        f.failed_checks
            .iter()
            .any(|c| c.starts_with("air_draft_within_limit")),
        "{:?}",
        f.failed_checks
    );
}

#[test]
fn a_load_that_leaves_less_freeboard_than_required_does_not_pass() {
    // A 3.5 m deep hull held to a 0.8 m draught is draught-bound and keeps 2.7 m of freeboard. The
    // search passes it at the default 0.3 m and refuses it when 2.8 m is required.
    let ok = fit(110.0, 20.0, "LPG_propane", 0.8, &options(3.5)).unwrap();
    assert_eq!(ok.binding, "draft");
    assert!(
        !ok.failed_checks
            .iter()
            .any(|c| c.starts_with("freeboard_at_payload")),
        "{:?}",
        ok.failed_checks
    );
    let o = FitOptions {
        required_freeboard_m: 2.8,
        ..options(3.5)
    };
    let f = fit(110.0, 20.0, "LPG_propane", 0.8, &o).unwrap();
    assert!(!f.feasible);
    assert!(
        f.failed_checks
            .iter()
            .any(|c| c.starts_with("freeboard_at_payload")),
        "{:?}",
        f.failed_checks
    );
}

#[test]
fn no_arrangement_labelled_feasible_has_a_failing_check() {
    let mut feasible = 0;
    let mut infeasible = 0;
    for loa in [40.0, 60.0, 80.0, 110.0, 150.0] {
        for beam in [8.0, 10.0, 14.0, 20.0] {
            for depth in [3.5, 4.5, 6.0] {
                for cargo in ["LPG_propane", "LNG", "LPG_butane"] {
                    for self_propelled in [true, false] {
                        let o = FitOptions {
                            self_propelled,
                            ..options(depth)
                        };
                        let f = fit(loa, beam, cargo, depth - 0.3, &o).unwrap();
                        let what = format!("{loa}x{beam}x{depth} {cargo} sp={self_propelled}");
                        if f.feasible {
                            feasible += 1;
                            assert!(f.failed_checks.is_empty(), "{what}: {:?}", f.failed_checks);
                            assert!(f.payload_t > 0.0, "{what}");
                            assert!(all_ok(&f.engine_result.checks), "{what}");
                            assert!(stability_ok(&f), "{what}");
                        } else {
                            infeasible += 1;
                            assert!(!f.failed_checks.is_empty(), "{what}");
                        }
                    }
                }
            }
        }
    }
    // The sweep must have both kinds, or it proves nothing.
    assert!(
        feasible > 20 && infeasible > 20,
        "{feasible} / {infeasible}"
    );
}

#[test]
fn the_smallest_viable_loa_has_a_passing_arrangement() {
    let o = FitOptions {
        fill_ratio: 0.98,
        ..FitOptions::for_smallest_viable_loa()
    };
    let loa = smallest_viable_loa(12.0, "LNG", 3.0, 20.0, 150.0, &o).unwrap();
    assert!(loa.is_finite());
    let f = fit(loa, 12.0, "LNG", 3.0, &o).unwrap();
    assert!(f.feasible, "{loa}: {:?}", f.failed_checks);
    assert!(stability_ok(&f));
}
