//! Seam: the public `ovcd_core::gas_arrangement` API. Expected values are printed by the original
//! Python (`core/gas_arrangement.py`), not worked out by hand.

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
    assert_eq!(f.payload_t, 2058.228164213275);
    assert_eq!((f.n_tanks, f.n_tanks_wide, f.n_tanks_long), (2, 1, 2));
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
    assert_eq!(f.failed_checks.len(), 3);
}

#[test]
fn an_unknown_ruleset_is_a_value_error() {
    let o = FitOptions {
        ruleset: "IGF".to_string(),
        ..FitOptions::default()
    };
    let e = fit(90.0, 14.8, "LNG", 3.0, &o).unwrap_err();
    assert_eq!(e.kind, "ValueError");
    assert_eq!(
        e.message,
        "ruleset must be one of ('IGC_2G', 'IGC_2G_EXACT')"
    );
}

#[test]
fn adn_g_is_no_longer_a_ruleset() {
    // Issue #175: only the two IGC rulesets are left.
    let o = FitOptions {
        ruleset: "ADN_G".to_string(),
        ..FitOptions::default()
    };
    let e = fit(90.0, 14.8, "LNG", 3.0, &o).unwrap_err();
    assert_eq!(e.kind, "ValueError");
    assert_eq!(
        e.message,
        "ruleset must be one of ('IGC_2G', 'IGC_2G_EXACT')"
    );
    assert_eq!(RULESETS, ["IGC_2G", "IGC_2G_EXACT"]);
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
        35.6
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

#[test]
fn the_exact_rule_measures_to_steel_so_insulation_sits_inside() {
    let c = required_side_clearance("IGC_2G_EXACT", 14.8, 2000.0, 1000.0, 350.0).unwrap();
    assert_eq!(c, 0.45000000000000007);
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

const IGC_RULESETS: [&str; 2] = ["IGC_2G", "IGC_2G_EXACT"];

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
    // The default cap, 16 m, gives a 16 m tank; the old constant, 12 m, gave a 12 m one.
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
    assert_eq!(run(Some(12.0)).tank_od_m, 12.0);
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
        for loa in [40.0, 60.0, 80.0, 100.0, 120.0, 150.0, 180.0] {
            for beam in [8.0, 10.0, 12.0, 14.0, 18.0, 24.0, 30.0] {
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
