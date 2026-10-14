//! Seam: the public `ovcd_core::resistance` API. Expected values are what the frozen Python
//! printed for the same inputs (see `parity/reference/resistance.json`); the full case set runs
//! in `ovcd-format/tests/parity.rs`.

use ovcd_core::aft_arrangement::PyException;
use ovcd_core::pyfmt::round_to;
use ovcd_core::resistance::{ResistanceInputs, TugInputs, compute};

fn close(a: f64, b: f64) {
    assert!((a - b).abs() <= 1e-9 * b.abs().max(1.0), "{a} vs {b}");
}

#[test]
fn the_default_barge_has_three_environments_and_the_river_binds() {
    let r = compute(&ResistanceInputs::default()).unwrap();
    close(r.wetted_surface_area, 4590.0);
    assert_eq!(r.environments.len(), 3);
    let names: Vec<&str> = r.environments.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(
        names,
        ["Open Sea", "Confined shallow river", "Confined creek"]
    );
    let binding = r.binding_environment().unwrap();
    assert_eq!(binding.name, "Confined shallow river");
    close(r.binding_power_at_service_kw(), 66.18578807734983);
    assert!(r.ok());
}

#[test]
fn service_speed_and_what_governs_it() {
    let r = compute(&ResistanceInputs::default()).unwrap();
    let [sea, river, creek] = &r.environments[..] else {
        panic!("three environments")
    };
    assert_eq!(sea.limiting_speed_knots, None);
    assert_eq!(sea.service_speed_knots, 12.0);
    assert_eq!(sea.service_speed_governed_by, "power");
    assert_eq!(sea.squat_limited_speed_knots, None);
    assert_eq!(sea.speed_curve.len(), 21);
    close(river.limiting_speed_knots.unwrap(), 10.827543883575197);
    assert_eq!(river.service_speed_knots, 2.5);
    assert_eq!(river.service_speed_governed_by, "squat");
    close(river.squat_limited_speed_knots.unwrap(), 3.779296875);
    assert_eq!(river.speed_curve.len(), 17);
    assert_eq!(creek.service_speed_knots, 10.0);
    close(creek.blockage_ratio, 0.19124999999999998);
}

#[test]
fn one_speed_point_in_the_river() {
    let r = compute(&ResistanceInputs::default()).unwrap();
    let p = &r.environments[1].speed_curve[3];
    assert_eq!(p.speed_knots, 3.5);
    assert_eq!(p.speed_through_water_knots, 4.5);
    close(p.froude_number, 0.06610342521903137);
    close(p.resistance_shallow_factor, 1.1194701507773603);
    close(p.resistance_total_kn, 27.688735115992472);
    close(p.delivered_power_kw, 142.43085343666527);
    close(p.squat_m, 0.5020177291974529);
    assert!(p.squat_limited);
    assert!(p.power_available);
}

#[test]
fn the_check_texts_match_python() {
    let r = compute(&ResistanceInputs::default()).unwrap();
    let texts: Vec<(&str, bool, &str)> = r
        .checks
        .iter()
        .map(|c| (c.name.as_str(), c.ok, c.message.as_str()))
        .collect();
    assert_eq!(texts.len(), 8);
    assert_eq!(
        texts[0],
        (
            "power_within_installed[Open Sea]",
            true,
            "Open Sea: power 2291 kW <= installed 2400 kW"
        )
    );
    assert_eq!(
        texts[1].2,
        "Confined shallow river: service 2.5 kn <= 0.95 x Schijf limit 10.83 kn"
    );
    assert_eq!(
        texts[3].2,
        "Confined shallow river: blockage n = 0.377 must be in (0, 1)"
    );
    assert_eq!(texts[7].0, "coefficients_are_first_order");
    assert_eq!(r.checks[7].severity, "warning");
}

#[test]
fn a_nonpositive_waterline_length_raises_as_python_does() {
    let zero = ResistanceInputs {
        lwl: 0.0,
        ..ResistanceInputs::default()
    };
    assert_eq!(
        compute(&zero).unwrap_err(),
        PyException::ZeroDivisionError("float division by zero".into())
    );
    let negative = ResistanceInputs {
        lwl: -1.0,
        ..ResistanceInputs::default()
    };
    assert_eq!(
        compute(&negative).unwrap_err(),
        PyException::ValueError("math domain error".into())
    );
}

#[test]
fn zero_fuel_density_is_a_zero_division() {
    let inp = ResistanceInputs {
        tug: TugInputs {
            fuel_density_kg_per_litre: 0.0,
            ..TugInputs::default()
        },
        ..ResistanceInputs::default()
    };
    assert_eq!(
        compute(&inp).unwrap_err(),
        PyException::ZeroDivisionError("float division by zero".into())
    );
}

#[test]
fn a_zero_draft_in_a_shallow_channel_fails_in_squat_validation() {
    let inp = ResistanceInputs {
        draft: 0.0,
        ..ResistanceInputs::default()
    };
    assert_eq!(
        compute(&inp).unwrap_err(),
        PyException::ValueError("draft_m must be > 0".into())
    );
}

#[test]
fn python_round_is_half_even_on_the_exact_value() {
    // CPython 3.12: round(0.25, 1) == 0.2, round(0.75, 1) == 0.8, round(2.25, 1) == 2.2,
    // round(2.75, 1) == 2.8, round(0.35, 1) == 0.3 (0.35 is a hair under), round(2.675, 2) == 2.67,
    // round(0.125, 2) == 0.12, round(0.375, 2) == 0.38.
    assert_eq!(round_to(0.25, 1), 0.2);
    assert_eq!(round_to(0.75, 1), 0.8);
    assert_eq!(round_to(2.25, 1), 2.2);
    assert_eq!(round_to(2.75, 1), 2.8);
    assert_eq!(round_to(0.35, 1), 0.3);
    assert_eq!(round_to(2.675, 2), 2.67);
    assert_eq!(round_to(0.125, 2), 0.12);
    assert_eq!(round_to(0.375, 2), 0.38);
    assert!(round_to(f64::NAN, 1).is_nan());
    assert_eq!(round_to(f64::INFINITY, 1), f64::INFINITY);
    assert!(round_to(-0.04, 1).is_sign_negative());
}

#[test]
fn the_speed_grid_is_repeated_addition_then_rounding() {
    // Step 0.25 from 2.0: the rounded grid repeats Python's 2.2, 2.8, 3.2 (not 2.25, 2.75, 3.25).
    let inp = ResistanceInputs {
        speed_step_knots: 0.25,
        speed_max_knots: 3.5,
        ..ResistanceInputs::default()
    };
    let r = compute(&inp).unwrap();
    let speeds: Vec<f64> = r.environments[0]
        .speed_curve
        .iter()
        .map(|p| p.speed_knots)
        .collect();
    assert_eq!(speeds, [2.0, 2.2, 2.5, 2.8, 3.0, 3.2, 3.5]);
}

// --- Inputs the parity cases never vary. Expected values are the frozen Python's. ---

use ovcd_core::resistance::EnvironmentInputs;

fn env(name: &str, depth: f64, bottom: f64, surface: f64) -> EnvironmentInputs {
    EnvironmentInputs {
        name: name.to_string(),
        water_depth: depth,
        channel_width_bottom: bottom,
        channel_width_surface: surface,
        ..EnvironmentInputs::default()
    }
}

fn deep(name: &str) -> EnvironmentInputs {
    EnvironmentInputs {
        name: name.to_string(),
        is_deep_water: true,
        ..EnvironmentInputs::default()
    }
}

fn with_envs(envs: Vec<EnvironmentInputs>) -> ResistanceInputs {
    ResistanceInputs {
        environments: envs,
        ..ResistanceInputs::default()
    }
}

#[test]
fn a_supplied_wetted_area_is_used_not_computed() {
    let inp = ResistanceInputs {
        wetted_surface_area: 3000.0,
        ..ResistanceInputs::default()
    };
    let r = compute(&inp).unwrap();
    assert_eq!(r.wetted_surface_area, 3000.0);
    let p = &r.environments[0].speed_curve[0];
    close(p.resistance_frictional_kn, 3.3525227670168434);
    close(p.resistance_total_kn, 3.6877750437185277);
    close(r.environments[0].power_at_service_kw, 1497.2142889390188);
    close(r.environments[1].power_at_service_kw, 43.258685017875706);
}

#[test]
fn a_channel_can_govern_the_service_speed() {
    let mut e = env("Cut", 5.0, 40.0, 60.0);
    e.apply_squat_limit = false;
    let inp = ResistanceInputs {
        speed_step_knots: 0.1,
        ..with_envs(vec![e])
    };
    let r = compute(&inp).unwrap();
    let c = &r.environments[0];
    assert_eq!(c.service_speed_governed_by, "channel");
    assert_eq!(c.service_speed_knots, 8.4);
    close(c.limiting_speed_knots.unwrap(), 8.884025674746445);
    close(c.blockage_ratio, 0.4896);
    assert_eq!(c.speed_curve.len(), 65);
    close(c.power_at_service_kw, 1522.048057913056);
    assert_eq!(
        r.checks[0].message,
        "Cut: service 8.4 kn <= 0.95 x Schijf limit 8.88 kn"
    );
}

#[test]
fn a_zero_area_channel_is_ok_with_a_failed_blockage_check() {
    let r = compute(&with_envs(vec![env("Zero", 6.0, 0.0, 0.0)])).unwrap();
    let e = &r.environments[0];
    assert_eq!(e.blockage_ratio, 0.0);
    assert_eq!(e.limiting_speed_knots, Some(0.0));
    assert_eq!(e.service_speed_knots, 2.0);
    close(
        e.speed_curve[0].resistance_shallow_factor,
        1.0092189211920222,
    );
    close(e.speed_curve[0].resistance_total_kn, 5.6943116973673265);
    assert_eq!(r.checks[1].name, "blockage_in_range[Zero]");
    assert!(!r.checks[1].ok);
    assert_eq!(
        r.checks[1].message,
        "Zero: blockage n = 0.000 must be in (0, 1)"
    );
}

#[test]
fn a_channel_exactly_filled_by_the_hull_fails_the_blockage_check() {
    // Midship area equals channel area: n is exactly 1.0, and (0, 1) is open.
    let r = compute(&with_envs(vec![env("Plug", 6.0, 20.4, 20.4)])).unwrap();
    let e = &r.environments[0];
    assert_eq!(e.blockage_ratio, 1.0);
    close(
        e.speed_curve[0].resistance_shallow_factor,
        1.045183224284105,
    );
    assert!(!r.checks[1].ok);
    assert_eq!(
        r.checks[1].message,
        "Plug: blockage n = 1.000 must be in (0, 1)"
    );
}

#[test]
fn the_wave_heading_changes_the_added_resistance() {
    let mut e = env("Coast", 12.0, 400.0, 500.0);
    e.significant_wave_height_m = 1.5;
    e.wave_heading_deg_off_bow = 30.0;
    e.peak_wave_period_s = 7.0;
    let r = compute(&with_envs(vec![e.clone()])).unwrap();
    let p = &r.environments[0].speed_curve[0];
    close(p.resistance_total_kn, 30.581921358527396);
    close(p.delivered_power_kw, 69.91706820811774);
    assert_eq!(r.environments[0].service_speed_knots, 11.0);
    e.wave_heading_deg_off_bow = 0.0;
    let head = compute(&with_envs(vec![e])).unwrap();
    close(
        head.environments[0].speed_curve[0].resistance_total_kn,
        38.89178587876913,
    );
}

#[test]
fn a_zero_propulsive_efficiency_gives_zero_fuel_and_still_returns() {
    let inp = ResistanceInputs {
        tug: TugInputs {
            propulsive_efficiency: 0.0,
            ..TugInputs::default()
        },
        ..ResistanceInputs::default()
    };
    // Python returns normally; fuel and power follow the zero-eta branch.
    let r = compute(&inp).unwrap();
    let p = &r.environments[0].speed_curve[0];
    close(p.delivered_power_kw, 5.80479393641576);
    close(p.fuel_litres_per_nm, 0.6697839157402801);
    close(r.environments[1].power_at_service_kw, 29.783604634807425);
}

#[test]
fn zero_speed_over_ground_in_a_current_has_zero_fuel_per_mile() {
    let mut e = deep("Flow");
    e.current_speed_knots = 1.0;
    let inp = ResistanceInputs {
        speed_min_knots: 0.0,
        ..with_envs(vec![e])
    };
    let r = compute(&inp).unwrap();
    let p = &r.environments[0].speed_curve[0];
    assert_eq!(p.speed_knots, 0.0);
    assert_eq!(p.fuel_litres_per_nm, 0.0);
    close(p.resistance_total_kn, 1.5626058510450285);
    assert_eq!(r.environments[0].speed_curve.len(), 25);
    assert_eq!(r.environments[0].service_speed_knots, 11.0);
}

#[test]
fn a_service_speed_below_zero_is_the_sweep_floor_when_no_point_has_power() {
    let inp = ResistanceInputs {
        speed_min_knots: -1.0,
        speed_max_knots: 1.0,
        speed_step_knots: 1.0,
        tug: TugInputs {
            engine_power_kw: 0.001,
            ..TugInputs::default()
        },
        ..with_envs(vec![deep("Still")])
    };
    let r = compute(&inp).unwrap();
    let e = &r.environments[0];
    assert_eq!(e.speed_curve.len(), 3);
    assert_eq!(e.service_speed_knots, -1.0);
    assert_eq!(e.power_at_service_kw, 0.0);
}

#[test]
fn a_laminar_hull_uses_the_ittc_fallback_friction() {
    // Speed through water 0.0001 kn gives Rn < 1e5.
    let mut e = deep("Calm");
    e.current_speed_knots = -1.9999;
    let inp = ResistanceInputs {
        speed_min_knots: 2.0,
        speed_max_knots: 3.0,
        speed_step_knots: 1.0,
        ..with_envs(vec![e])
    };
    let r = compute(&inp).unwrap();
    let p = &r.environments[0].speed_curve[0];
    let want = 4.979645907838903e-08;
    assert!(
        (p.resistance_frictional_kn - want).abs() <= 1e-12,
        "{}",
        p.resistance_frictional_kn
    );
    close(p.resistance_total_kn, 5.477610498622793e-08);
    assert_eq!(r.environments[0].service_speed_knots, 3.0);
}

#[test]
fn an_underpowered_tug_fails_ok() {
    let inp = ResistanceInputs {
        tug: TugInputs {
            engine_power_kw: 1.0,
            ..TugInputs::default()
        },
        ..ResistanceInputs::default()
    };
    let r = compute(&inp).unwrap();
    assert!(!r.ok());
    assert!(!r.checks[0].ok);
    assert_eq!(
        r.checks[0].message,
        "Open Sea: power 13 kW <= installed 1 kW"
    );
    for e in &r.environments {
        assert_eq!(e.service_speed_knots, 2.0);
    }
}

#[test]
fn equal_service_speeds_bind_to_the_first_environment() {
    let a = env("First", 6.5, 80.0, 120.0);
    let b = env("Second", 6.5, 80.0, 120.0);
    let r = compute(&with_envs(vec![a, b])).unwrap();
    assert_eq!(r.environments[0].service_speed_knots, 4.5);
    assert_eq!(r.environments[1].service_speed_knots, 4.5);
    assert_eq!(r.binding_environment().unwrap().name, "First");
    close(r.binding_power_at_service_kw(), 142.69389106055436);
}

// --- Knife edges: the comparison sits exactly on its bound. ---

#[test]
fn froude_number_exactly_0_2_uses_the_residuary_fit_above_the_bound() {
    let mut e = deep("Fast");
    e.current_speed_knots = 11.615028223089526;
    let inp = ResistanceInputs {
        speed_min_knots: 2.0,
        speed_max_knots: 2.0,
        speed_step_knots: 1.0,
        ..with_envs(vec![e])
    };
    let r = compute(&inp).unwrap();
    let p = &r.environments[0].speed_curve[0];
    assert_eq!(p.froude_number, 0.2);
    close(p.resistance_residuary_kn, 27.529869204463804);
    close(p.resistance_total_kn, 211.06233056755585);
    assert!(!r.checks[0].ok);
}

#[test]
fn depth_froude_number_exactly_0_95_keeps_the_subcritical_factor() {
    let mut e = env("E", 6.5, 300.0, 340.0);
    e.current_speed_knots = 12.747344562397966;
    let inp = ResistanceInputs {
        speed_min_knots: 2.0,
        speed_max_knots: 2.0,
        speed_step_knots: 1.0,
        ..with_envs(vec![e])
    };
    let r = compute(&inp).unwrap();
    let p = &r.environments[0].speed_curve[0];
    assert_eq!(p.froude_depth, 0.95);
    close(p.resistance_shallow_factor, 6.106217307692308);
    close(p.resistance_total_kn, 1499.7916759260709);
}

#[test]
fn a_dynamic_clearance_exactly_on_the_margin_is_not_squat_limited() {
    let env_with = |margin: f64| {
        let mut e = env("Shoal", 7.0, 60.0, 90.0);
        e.squat_safety_margin_m = margin;
        compute(&with_envs(vec![e])).unwrap()
    };
    let base = env_with(0.15);
    let point = base.environments[0]
        .speed_curve
        .iter()
        .find(|p| p.speed_knots == 4.0)
        .unwrap();
    let ukc = point.dynamic_ukc_m;
    close(ukc, 0.733548423071147);
    let r = env_with(ukc);
    let p = r.environments[0]
        .speed_curve
        .iter()
        .find(|p| p.speed_knots == 4.0)
        .unwrap();
    assert_eq!(p.dynamic_ukc_m, ukc);
    assert!(!p.squat_limited);
}

#[test]
fn a_service_speed_exactly_at_95_percent_of_the_limit_passes() {
    let mut e = env("Cut", 5.0, 17.386400075809654, 37.386400075809654);
    e.apply_squat_limit = false;
    let inp = ResistanceInputs {
        speed_step_knots: 0.1,
        ..with_envs(vec![e])
    };
    let r = compute(&inp).unwrap();
    assert_eq!(r.environments[0].service_speed_knots, 5.0);
    assert_eq!(r.environments[0].service_speed_governed_by, "channel");
    assert!(r.checks[0].ok);
    assert_eq!(
        r.checks[0].message,
        "Cut: service 5.0 kn <= 0.95 x Schijf limit 5.26 kn"
    );
}

#[test]
fn power_exactly_equal_to_the_engine_power_passes() {
    let inp = ResistanceInputs {
        tug: TugInputs {
            engine_power_kw: 299.1120265395556,
            ..TugInputs::default()
        },
        ..with_envs(vec![deep("Deep")])
    };
    let r = compute(&inp).unwrap();
    assert_eq!(r.environments[0].service_speed_knots, 6.0);
    close(r.environments[0].power_at_service_kw, 299.1120265395556);
    assert!(r.checks[0].ok);
    assert!(r.ok());
}

// --- Knife-edge guards (issue #20). Expected values: frozen Python via parity/driver.run_case. ---

#[test]
fn squat_does_not_apply_at_depth_exactly_zero() {
    let mut e = env("x", 0.0, 20.0, 40.0);
    e.apply_squat_limit = true;
    let inp = ResistanceInputs {
        speed_min_knots: 5.0,
        speed_max_knots: 4.0,
        ..with_envs(vec![e])
    };
    let r = compute(&inp).unwrap();
    assert_eq!(r.environments[0].squat_limited_speed_knots, None);
    assert_eq!(r.environments[0].service_speed_governed_by, "channel");
}

#[test]
fn ittc_friction_is_turbulent_at_reynolds_exactly_1e5() {
    let inp = ResistanceInputs {
        loa: 0.25,
        lwl: 0.22161741835147747,
        beam: 0.1,
        draft: 0.05,
        speed_min_knots: 1.0,
        speed_max_knots: 1.0,
        ..with_envs(vec![EnvironmentInputs {
            name: "x".to_string(),
            water_density: 1.0,
            is_deep_water: true,
            ..EnvironmentInputs::default()
        }])
    };
    let r = compute(&inp).unwrap();
    let p = &r.environments[0].speed_curve[0];
    assert_eq!(p.resistance_frictional_kn, 4.984536e-05);
    assert_eq!(p.resistance_total_kn, 6.615974179695845e-05);
}

#[test]
fn the_shallow_water_factor_is_one_when_the_draft_is_zero() {
    let inp = ResistanceInputs {
        draft: 0.0,
        speed_min_knots: 4.0,
        speed_max_knots: 4.0,
        ..with_envs(vec![EnvironmentInputs {
            name: "x".to_string(),
            water_depth: 1.0,
            current_speed_knots: 3.0,
            apply_squat_limit: false,
            ..EnvironmentInputs::default()
        }])
    };
    let r = compute(&inp).unwrap();
    let p = &r.environments[0].speed_curve[0];
    assert_eq!(p.resistance_shallow_factor, 1.0);
    assert_eq!(p.resistance_total_kn, 38.784599026937556);
}
