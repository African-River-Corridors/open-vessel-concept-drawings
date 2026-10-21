//! Seam: the public `ovcd_core::gas_arrangement` API. Expected values are printed by the original
//! Python (`core/gas_arrangement.py`), not worked out by hand, except the ones marked re-pinned
//! (issue #237: the rule set changed, so they are what the current code prints; the old values are
//! in the history and the before/after table is in `CHANGELOG.md`).

use ovcd_core::gas_arrangement::{
    DEFAULT_DEPTHS_M, FitOptions, RULESETS, fit, required_side_clearance, smallest_viable_loa,
};

#[test]
fn the_default_depths_are_python_s_rounded_grid() {
    assert_eq!(DEFAULT_DEPTHS_M.len(), 15);
    assert_eq!(DEFAULT_DEPTHS_M[0], 3.5);
    assert_eq!(DEFAULT_DEPTHS_M[14], 7.0);
}

#[test]
fn the_cn_ii_benchmark_is_reproduced() {
    let o = FitOptions {
        depths: vec![6.5],
        ..FitOptions::default()
    };
    let f = fit(90.0, 14.8, "LPG_propane", 4.4, &o).unwrap();
    // Re-pinned (issue #264, the search keeps only arrangements that pass stability); was
    // 2943.4407266060916 (issue #237) and 2058.228164213275 under the stepped IGC_2G. Re-pinned
    // again (issue #277, the wheelhouse sightline blocks the search); was 2832.6643735601965.
    assert_eq!(f.payload_t, 2021.2487121368047);
    assert_eq!((f.n_tanks, f.n_tanks_wide, f.n_tanks_long), (2, 1, 2));
    // Re-pinned (issue #264); was "volume" under the stepped IGC_2G and "draft" (issue #237): the
    // arrangement that passes stability is smaller, so the volume binds again.
    assert_eq!(f.binding, "volume");
    assert_eq!(f.candidates_tried, 15);
    assert!(f.feasible);
}

#[test]
fn a_hull_too_short_for_a_tank_is_reported_infeasible_not_raised() {
    let o = FitOptions {
        depths: vec![4.0],
        ..FitOptions::default()
    };
    let f = fit(30.0, 14.8, "LNG", 3.0, &o).unwrap();
    assert_eq!(f.binding, "no cargo area");
    assert_eq!(f.payload_t, 0.0);
    assert!(!f.feasible);
    // Re-pinned (issue #264): `failed_checks` lists the failed error checks only; it was 3 and
    // included a warning.
    assert_eq!(f.failed_checks.len(), 2);
}

#[test]
fn an_unknown_ruleset_is_a_value_error() {
    let o = FitOptions {
        ruleset: "IGF".to_string(),
        ..FitOptions::default()
    };
    let e = fit(90.0, 14.8, "LNG", 3.0, &o).unwrap_err();
    assert_eq!(e.kind, "ValueError");
    assert_eq!(e.message, "ruleset must be one of ('IGC_2G',)");
}

#[test]
fn adn_g_is_no_longer_a_ruleset() {
    // Issue #175: only IGC rule sets are left; issue #237: one, IGC_2G (the per-tank rule).
    let o = FitOptions {
        ruleset: "ADN_G".to_string(),
        ..FitOptions::default()
    };
    let e = fit(90.0, 14.8, "LNG", 3.0, &o).unwrap_err();
    assert_eq!(e.kind, "ValueError");
    assert_eq!(e.message, "ruleset must be one of ('IGC_2G',)");
    assert_eq!(RULESETS, ["IGC_2G"]);
}

#[test]
fn an_empty_depth_list_is_an_index_error_when_nothing_fits() {
    let o = FitOptions {
        depths: vec![],
        ..FitOptions::default()
    };
    let e = fit(90.0, 14.8, "LNG", 3.0, &o).unwrap_err();
    assert_eq!(e.kind, "IndexError");
}

#[test]
fn the_smallest_viable_loa_is_found_to_a_tenth_of_a_metre() {
    let o = FitOptions::for_smallest_viable_loa();
    assert_eq!(
        smallest_viable_loa(12.0, "LNG", 3.0, 20.0, 120.0, &o).unwrap(),
        // Re-pinned (issue #264, a viable length needs an arrangement that passes every check,
        // stability included); was 36.1 (issue #237) and 35.6 under the stepped IGC_2G.
        39.9
    );
}

#[test]
fn a_beam_that_never_gets_cargo_area_gives_infinity() {
    let o = FitOptions::for_smallest_viable_loa();
    assert_eq!(
        smallest_viable_loa(12.0, "LNG", 3.0, 20.0, 25.0, &o).unwrap(),
        f64::INFINITY
    );
}

/// sourced - IGC Code 2.4.1.1 and 2.4.2 (MSC.370(93)): d per tank, measured plate to plate,
/// excluding insulation, so the clear gap is d less the insulation. A 1,000 m3 tank: d = 0.8 m,
/// less 0.35 m of insulation, is 0.45 m (the total volume, 2,000 m3 here, is not used).
#[test]
fn the_rule_measures_to_steel_so_insulation_sits_inside() {
    let c = required_side_clearance("IGC_2G", 14.8, 2000.0, 1000.0, 350.0).unwrap();
    assert_eq!(c, 0.45000000000000007);
    // A 3,232 m3 tank: d = 0.75 + 3,232 x 0.2 / 4,000 = 0.9116 m; less 0.35 m is 0.5616 m.
    let c = required_side_clearance("IGC_2G", 14.8, 99999.0, 3232.0, 350.0).unwrap();
    assert!((c - 0.5616).abs() < 1e-12, "{c}");
}

#[test]
fn the_old_exact_name_is_not_a_rule_set_of_the_core() {
    let e = required_side_clearance("IGC_2G_EXACT", 14.8, 2000.0, 1000.0, 350.0).unwrap_err();
    assert_eq!(e.kind, "ValueError");
    assert_eq!(e.message, "'IGC_2G_EXACT' is not a valid VesselType");
}

#[test]
fn an_unknown_ruleset_has_no_side_clearance() {
    let e = required_side_clearance("IGC_9G", 14.8, 2000.0, 1000.0, 350.0).unwrap_err();
    assert_eq!(e.kind, "ValueError");
    assert_eq!(e.message, "'IGC_9G' is not a valid VesselType");
}

// Issue #146: two of the four mutation survivors of PR #56 are killed by the inputs below. The
// other two are equivalent mutants; the reason is in a comment beside each line of
// `src/gas_arrangement.rs`. Expected values are printed by the frozen Python at commit `ce96e66`
// (`core/gas_arrangement.py` `fit`), the same inputs.

#[test]
fn a_cap_clearance_a_hair_under_the_rule_is_accepted_not_refused() {
    // The clearance of the tank on the cap is checked as `cs + 1e-9 < need` (survivor: `+` to
    // `-`, a 2e-9 window). IGC_2G steps d(Vc) from 0.8 to 1.0 m above 1000 m3. Here the hull is
    // cut so that the tank volume at a 1.0 m clearance is just under 1000 m3 (999.9999999999999)
    // and the full-beam attempt does not settle (it flips between the two bands). The cap is set
    // so the clearance is 0.9999999995 m. It is 5e-10 m under the 1.0 m the rule asks for at
    // 1000.000000127 m3, inside the 1e-9 tolerance, so the arrangement is accepted. With `-` it
    // is refused and a different (heavier-payload, wider-clearance) arrangement wins.
    let o = FitOptions {
        depths: vec![4.5],
        tanks_wide: vec![1],
        extra_clearance_m: vec![],
        self_propelled: false,
        max_tank_od_m: 8.000000001,
        ..FitOptions::default()
    };
    let f = fit(45.25667782, 10.0, "LNG", 4.0, &o).unwrap();
    // ce96e66
    assert_eq!(f.payload_t, 427.5000000542083);
    assert_eq!(f.side_clearance_m, 0.9999999995);
    assert_eq!(f.tank_od_m, 8.000000001);
    assert_eq!(f.containment_inner_volume_m3, 1000.000000126803);
    assert_eq!(f.per_tank_volume_m3, 500.0000000634015);
    assert_eq!((f.n_tanks, f.candidates_tried), (2, 1));
    assert_eq!(f.binding, "volume");
    assert!(f.feasible && f.failed_checks.is_empty());
}

// --- Issue #175: tank outer diameter cap and length over diameter, under both IGC rulesets ---
//
// Label: property. These tests state a rule (OD <= cap, L/D <= 7), not a printed number.

const IGC_RULESETS: [&str; 1] = ["IGC_2G"];

/// Overall length of one tank over its outer diameter, from the engine's own tank result.
fn slenderness(f: &ovcd_core::gas_arrangement::ArrangementFit) -> f64 {
    let t = &f.engine_result.gas_basis.as_ref().unwrap().tanks;
    t.overall_tank_length / t.outer_diameter
}

#[test]
fn the_tank_diameter_cap_is_an_option_and_defaults_to_16_m() {
    assert_eq!(FitOptions::default().max_tank_od_m, 16.0);
}

#[test]
fn a_wide_hull_gets_tanks_up_to_the_cap_under_both_rulesets() {
    for ruleset in IGC_RULESETS {
        for cap in [8.0, 12.0, 16.0, 20.0] {
            let o = FitOptions {
                depths: vec![5.0],
                tanks_wide: vec![1],
                ruleset: ruleset.to_string(),
                max_tank_od_m: cap,
                ..FitOptions::default()
            };
            let f = fit(110.0, 26.0, "LNG", 4.0, &o).unwrap();
            assert!(
                f.tank_od_m <= cap + 1e-6,
                "{ruleset} cap {cap}: OD {}",
                f.tank_od_m
            );
            assert_eq!(f.engine_inputs.max_tank_od_m, cap, "{ruleset} cap {cap}");
            assert_eq!(f.engine_result.max_tank_od_m, Some(cap));
        }
    }
}

#[test]
fn the_default_cap_lets_a_tank_pass_the_old_12_m() {
    // IGC_2G: the full-beam tank of a 26 m hull is over every cap tried, so it sits on the cap.
    // The default cap, 16 m, gives a 16 m tank. Since issue #264 the search keeps only arrangements
    // that pass stability, so the 12 m cap no longer gives a 12 m tank on this hull: it gives a
    // smaller one, never over the cap.
    let run = |cap: Option<f64>| {
        let mut o = FitOptions {
            depths: vec![5.0],
            tanks_wide: vec![1],
            ..FitOptions::default()
        };
        if let Some(c) = cap {
            o.max_tank_od_m = c;
        }
        fit(110.0, 26.0, "LNG", 4.0, &o).unwrap()
    };
    assert_eq!(run(None).tank_od_m, 16.0);
    assert!(run(Some(12.0)).tank_od_m <= 12.0 + 1e-6);
}

#[test]
fn a_tank_longer_than_seven_diameters_is_not_accepted_under_either_ruleset() {
    // A long, narrow hull has two tanks of about 60 m and 7 m across: L/D is about 9.
    for ruleset in IGC_RULESETS {
        let o = FitOptions {
            depths: vec![5.0],
            ruleset: ruleset.to_string(),
            ..FitOptions::default()
        };
        let f = fit(170.0, 8.0, "LNG", 4.0, &o).unwrap();
        assert!(slenderness(&f) > 7.0, "{ruleset}: L/D {}", slenderness(&f));
        assert!(!f.feasible, "{ruleset} accepted L/D {}", slenderness(&f));
        assert!(
            f.failed_checks
                .iter()
                .any(|m| m.starts_with("tank_length_over_diameter:")),
            "{ruleset}: {:?}",
            f.failed_checks
        );
    }
}

#[test]
fn no_feasible_fit_has_a_tank_longer_than_seven_diameters() {
    // Sweep hulls from short and wide to long and narrow. Some must be refused for L/D, or the
    // sweep proves nothing; every one that is accepted must hold the limit.
    let (mut accepted, mut refused_for_slenderness) = (0, 0);
    for ruleset in IGC_RULESETS {
        for loa in [
            40.0, 60.0, 80.0, 100.0, 120.0, 150.0, 180.0, 210.0, 240.0, 280.0,
        ] {
            for beam in [6.0, 8.0, 10.0, 12.0, 14.0, 18.0, 24.0, 30.0] {
                let o = FitOptions {
                    depths: vec![5.0],
                    ruleset: ruleset.to_string(),
                    ..FitOptions::default()
                };
                let f = fit(loa, beam, "LNG", 4.0, &o).unwrap();
                if f.engine_result.gas_basis.is_none() {
                    continue;
                }
                if f.feasible && f.payload_t > 0.0 {
                    accepted += 1;
                    assert!(
                        slenderness(&f) <= 7.0,
                        "{ruleset} {loa} x {beam}: L/D {}",
                        slenderness(&f)
                    );
                } else if slenderness(&f) > 7.0 {
                    refused_for_slenderness += 1;
                }
            }
        }
    }
    assert!(accepted > 20, "only {accepted} accepted");
    assert!(
        refused_for_slenderness > 5,
        "only {refused_for_slenderness} refused"
    );
}

// Label: property. This test states a rule of the bisection, with the lengths it relies on checked
// in the test itself, not a printed number.
//
// Issue #57: `smallest_viable_loa` must treat a length with exactly zero capacity as NOT viable
// (`volume_capacity_t > 0.0`). The mutant `>=` would treat it as viable.
//
// Worked out. Beam 12 m, LNG, draught cap 3 m, search range 10 m to 60 m:
//  - the first midpoint is 0.5 * (10 + 60) = 35 m;
//  - at 35 m the hull has no cargo area, so the capacity is exactly 0.0 (asserted below);
//  - at 60 m the capacity is positive (asserted below), so the search does not return infinity;
//  - the real code therefore moves `lo` up to 35 and can only return a length above 35 m;
//  - the mutant would move `hi` down to 35 and keep halving towards 10 m, so it could only return
//    a length at most 35 m (it would stop at 10.1 m).
// The answer must lie above the last length known to have zero capacity (35 m) and at most the
// first length known to have capacity (40 m, asserted below).
#[test]
fn the_bisection_treats_a_zero_capacity_midpoint_as_not_viable() {
    let o = FitOptions::for_smallest_viable_loa();
    let cap = |loa: f64| fit(loa, 12.0, "LNG", 3.0, &o).unwrap().volume_capacity_t;
    assert_eq!(
        cap(35.0),
        0.0,
        "35 m (the first midpoint) must have exactly zero capacity"
    );
    assert!(cap(40.0) > 0.0);
    assert!(cap(60.0) > 0.0);
    let loa = smallest_viable_loa(12.0, "LNG", 3.0, 10.0, 60.0, &o).unwrap();
    assert!(loa > 35.0 && loa <= 40.0, "got {loa}");
    assert!(cap(loa) > 0.0);
}

// Label: property. States the two facts that make the `&&` and `>` in the extra-clearance loop of
// `fit` equivalent mutants (issue #58; the reasoning is in the comment at that line).
//
// With a fill ratio of 0 every candidate has a tank (diameter above zero) but exactly zero
// capacity: the case the mutants `||` and `>=` would let into the search. Worked out: the cargo
// volume is the tank volume times the fill ratio, so it is 0.0 and the payload is 0.0 (it is
// capped by the capacity). The `feasible_geometry` check needs a positive cargo volume, so it
// fails and the result is infeasible.
#[test]
fn a_zero_capacity_arrangement_is_infeasible_and_carries_nothing() {
    let o = FitOptions {
        fill_ratio: 0.0,
        ..FitOptions::default()
    };
    let f = fit(90.0, 14.8, "LNG", 3.0, &o).unwrap();
    assert!(f.tank_od_m > 0.0, "the hull must have tanks for this case");
    assert_eq!(f.volume_capacity_t, 0.0);
    assert_eq!(f.payload_t, 0.0);
    assert!(!f.feasible);
    assert!(
        f.failed_checks
            .iter()
            .any(|c| c.starts_with("feasible_geometry")),
        "{:?}",
        f.failed_checks
    );
}

// Label: property. These two tests state the facts that make the guard `cs < 0.0` in `fit` (issue
// #59) an equivalent mutant; the reasoning is in the comment at that line.
//
// Worked out for the first: with a cap of 5 m and two tanks across, `cs` is
// (6 - 2 * 5) / 2 = -2 m, below zero. A 15 m hull has no room for tank heads, so the full-beam
// tank has no usable diameter and the guard is reached. No arrangement can be made, so the hull is
// reported infeasible with no capacity and no tank.
#[test]
fn a_hull_too_narrow_for_tanks_at_the_cap_gets_no_candidate() {
    let o = FitOptions {
        max_tank_od_m: 5.0,
        tanks_wide: vec![2],
        ..FitOptions::default()
    };
    let f = fit(15.0, 6.0, "LNG", 3.0, &o).unwrap();
    assert!(!f.feasible);
    assert_eq!(f.volume_capacity_t, 0.0);
    assert_eq!(f.tank_od_m, 0.0);
    assert_eq!(f.binding, "no cargo area");
}

// Worked out: IGC_2G needs d(V) less the insulation, 0.8 m less 0.35 m = 0.45 m at least (IGC
// 2.4.1.1: d is at least 0.8 m; 2.4.2: the insulation is inside d; the insulation is fixed at
// 350 mm). That is above any negative clearance.
#[test]
fn the_side_clearance_rule_is_never_below_0_45_m() {
    for ruleset in RULESETS {
        for total_m3 in [0.0, 1.0, 500.0, 2_000.0, 10_000.0, 100_000.0] {
            let per_tank = total_m3 / 2.0;
            let need = required_side_clearance(ruleset, 6.0, total_m3, per_tank, 350.0).unwrap();
            assert!(need >= 0.45 - 1e-12, "{ruleset} {total_m3}: {need}");
        }
    }
}

// --- Issue #215: the inland type-G floors are gone ---

/// Label: property. A 120 x 18 m hull with only a two-abreast arrangement on offer has a 0.48 m side
/// clearance. That used to fail the inland type-G side floor, and nothing else. It now passes, and
/// no check carries a type-G name.
#[test]
fn a_design_that_failed_only_the_type_g_floor_now_passes() {
    let o = FitOptions {
        ruleset: "IGC_2G".to_string(),
        fill_ratio: 0.98,
        depths: vec![5.75],
        tanks_wide: vec![2],
        extra_clearance_m: vec![],
        ..FitOptions::default()
    };
    let f = fit(120.0, 18.0, "LNG", 3.0, &o).unwrap();
    assert!(f.side_clearance_m < 0.8);
    assert!(f.feasible);
    assert!(f.failed_checks.is_empty());
}

#[test]
fn a_full_volume_check_does_not_remove_a_de_rated_candidate() {
    // Issue #418 (maintainer decision): `freeboard_ok` and `not_overloaded` test the full tank
    // volume. A candidate carries only its de-rated payload, so they do not gate the search; the
    // carried draught is gated by `freeboard_at_payload`. A 6.5 m hull at a 2.0 m draught cap, with
    // a 4.45 m required freeboard: the payload keeps 4.50 m, the full volume misses it, and the
    // arrangement passes.
    let o = FitOptions {
        depths: vec![6.5],
        required_freeboard_m: 4.45,
        ..FitOptions::default()
    };
    let f = fit(90.0, 14.8, "LPG_propane", 2.0, &o).unwrap();
    assert!(f.feasible, "{:?}", f.failed_checks);
    let full = f
        .engine_result
        .checks
        .iter()
        .find(|c| c.name == "freeboard_ok")
        .unwrap();
    assert!(!full.ok, "{}", full.message);
    // The full-volume check reads the user's freeboard, not a hidden 0.3 m.
    assert!(
        full.message.to_string().contains("required 4.45"),
        "{}",
        full.message
    );
    assert!(f.depth_m - f.draft_at_payload_m >= 4.45 - 1e-9);
}

#[test]
fn a_tank_put_on_the_cap_counts_the_gaps_between_tanks() {
    // Issue #422: with n tanks across on the diameter cap, the side clearance is what the beam
    // leaves after n caps and the n - 1 transverse gaps (1.0 m each), so the tanks reach the cap.
    // A 22 m beam, two tanks across, an 8 m cap: clearance (22 - 16 - 1) / 2 = 2.5 m, not 3 m.
    let o = FitOptions {
        depths: vec![6.0],
        tanks_wide: vec![2],
        max_tank_od_m: 8.0,
        ..FitOptions::default()
    };
    let f = fit(100.0, 22.0, "LNG", 3.0, &o).unwrap();
    assert_eq!(f.n_tanks_wide, 2);
    assert!((f.tank_od_m - 8.0).abs() < 1e-6, "{}", f.tank_od_m);
    assert!(
        (f.side_clearance_m - 2.5).abs() < 1e-6,
        "{}",
        f.side_clearance_m
    );
}

#[test]
fn property_the_smallest_viable_length_is_a_length_that_passes() {
    // Issue #422: the answer is rounded to 0.1 m, and the rounded length itself passes every
    // check; rounding never returns an untested length below the one the search found.
    for (beam, cargo, t) in [
        (11.4, "LNG", 3.0),
        (14.8, "LPG_propane", 3.5),
        (9.0, "LNG", 2.5),
    ] {
        let o = FitOptions::default();
        let l = smallest_viable_loa(beam, cargo, t, 20.0, 200.0, &o).unwrap();
        assert!(l.is_finite(), "{beam} {cargo}");
        assert_eq!((l * 10.0).round() / 10.0, l, "{l} is not on the 0.1 m grid");
        assert!(
            fit(l, beam, cargo, t, &o).unwrap().feasible,
            "{beam} {cargo}: {l} m does not pass"
        );
    }
}
