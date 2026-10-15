//! Seam: the public `ovcd_core::gas_arrangement` API. Expected values are printed by the original
//! Python (`core/gas_arrangement.py`), not worked out by hand.

use ovcd_core::gas_arrangement::{
    DEFAULT_DEPTHS_M, FitOptions, fit, required_side_clearance, smallest_viable_loa,
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
fn adn_g_adds_tanks_along_the_hull_and_flags_a_wide_tank() {
    let o = FitOptions {
        depths: vec![4.5],
        ruleset: "ADN_G".to_string(),
        adn_tank_cap_m3: 380.0,
        ..FitOptions::default()
    };
    let f = fit(110.0, 11.4, "LNG", 3.0, &o).unwrap();
    assert_eq!((f.n_tanks, f.n_tanks_long), (7, 7));
    assert!(
        f.failed_checks
            .last()
            .unwrap()
            .starts_with("adn_wide_tank: tank 8.9 m > 0.70 B")
    );
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
        "ruleset must be one of ('IGC_2G', 'IGC_2G_EXACT', 'ADN_G')"
    );
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
fn a_tank_exactly_at_seventy_percent_of_beam_is_not_flagged_wide() {
    // `fit` flags a tank wider than 0.70 B, not one equal to it (survivor: `>` to `>=`). The tank
    // sits on the cap, 7.0 m, which is 0.70 x 10.0 m exactly.
    let o = FitOptions {
        depths: vec![4.5],
        ruleset: "ADN_G".to_string(),
        tanks_wide: vec![1],
        adn_tank_cap_m3: 380.0,
        max_tank_od_m: 7.0,
        ..FitOptions::default()
    };
    let f = fit(60.0, 10.0, "LNG", 3.0, &o).unwrap();
    assert_eq!(0.7_f64 * 10.0, 7.0);
    assert_eq!(f.tank_od_m, 7.0);
    assert_eq!(f.payload_t, 127.86772973962577); // ce96e66
    assert_eq!((f.n_tanks, f.n_tanks_wide, f.n_tanks_long), (2, 1, 2));
    assert_eq!(f.candidates_tried, 27);
    assert!(f.feasible);
    // ce96e66: these two, and no `adn_wide_tank: ...` line.
    assert_eq!(
        f.failed_checks,
        vec![
            "requested_thruster_count_fits: 2 of 3 requested thrusters fit; 3 would need 10.20 m of 10.00 m beam",
            "units_fit_in_one_row: 3 units in 2 row(s) of 2; a second row adds 12.99 m to the stern block and comes straight off the cargo zone",
        ]
    );
}

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
