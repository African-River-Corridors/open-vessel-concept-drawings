//! Seam: the public `ovcd_core::gas_arrangement` API. Expected values are printed by the frozen
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
