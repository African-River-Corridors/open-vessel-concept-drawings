//! Seam: the public `ovcd_core::resistance` API. Expected values are what the original Python
//! printed for the same inputs.
//! Label: snapshot
//! A value re-pinned after an agreed change is what the code printed then; the old value is named next to it.

use ovcd_core::aft_arrangement::PyException;
use ovcd_core::pyfmt::round_to;
use ovcd_core::resistance::{ResistanceInputs, TugInputs, compute, residuary_ratio};

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
    // Re-pinned (issues #396, #397); was 66.18578807734983.
    close(r.binding_power_at_service_kw(), 66.60443130757577);
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
    // Issue #400: the open sea runs to the top of the speed grid (12 kn); it was labelled "power".
    assert_eq!(sea.service_speed_governed_by, "speed grid");
    assert_eq!(sea.squat_limited_speed_knots, None);
    assert_eq!(sea.speed_curve.len(), 21);
    // Re-pinned (issues #396, #397); was 10.827543883575197.
    close(river.limiting_speed_knots.unwrap(), 3.798949005299152);
    assert_eq!(river.service_speed_knots, 2.5);
    assert_eq!(river.service_speed_governed_by, "squat");
    close(river.squat_limited_speed_knots.unwrap(), 3.779296875);
    // Re-pinned (issues #396, #397); was 17: the sweep stops at 0.95 x the lower limit.
    assert_eq!(river.speed_curve.len(), 4);
    // Re-pinned (issues #396, #397); was 10.0.
    assert_eq!(creek.service_speed_knots, 7.0);
    // Re-pinned (issue #396, Am = B x T); was 0.19124999999999998, that is x Cb 0.85.
    close(creek.blockage_ratio, 0.225);
}

#[test]
fn one_speed_point_in_the_river() {
    let r = compute(&ResistanceInputs::default()).unwrap();
    let p = &r.environments[1].speed_curve[3];
    assert_eq!(p.speed_knots, 3.5);
    assert_eq!(p.speed_through_water_knots, 4.5);
    close(p.froude_number, 0.06610342521903137);
    // Re-pinned (issues #396, #397); was 1.1194701507773603.
    close(p.resistance_shallow_factor, 1.1306399397755689);
    // Re-pinned (issues #396, #397); was 27.688735115992472.
    close(p.resistance_total_kn, 27.965006286472686);
    // Re-pinned (issues #396, #397); was 142.43085343666527.
    close(p.delivered_power_kw, 143.85199233761548);
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
    // Re-pinned (issue #420: the squat and fit checks at each service speed); was 8.
    assert_eq!(texts.len(), 20);
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
        // Re-pinned (issues #396, #397); was "... limit 10.83 kn".
        "Confined shallow river: service 2.5 kn <= 0.95 x Schijf limit 3.80 kn"
    );
    assert_eq!(
        texts[3].2,
        // Re-pinned (issue #396, Am = B x T); was n = 0.377.
        "Confined shallow river: blockage n = 0.443 must be in (0, 1)"
    );
    assert_eq!(texts[19].0, "coefficients_are_first_order");
    assert_eq!(r.checks[19].severity, "warning");
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

// --- Inputs the earlier case set never varied. Expected values are the original Python's. ---

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
    // Re-pinned (issues #396, #397); was 43.258685017875706.
    close(r.environments[1].power_at_service_kw, 43.53230804416717);
}

#[test]
fn a_channel_can_govern_the_service_speed() {
    // Issue #397: wider than before (was 40 m to 60 m), so the lower Schijf limit still falls in
    // the middle of the speed range.
    let mut e = env("Cut", 5.0, 100.0, 120.0);
    e.apply_squat_limit = false;
    let inp = ResistanceInputs {
        speed_step_knots: 0.1,
        ..with_envs(vec![e])
    };
    let r = compute(&inp).unwrap();
    let c = &r.environments[0];
    assert_eq!(c.service_speed_governed_by, "channel");
    // Re-pinned (issues #396, #397) with the wider channel; were 8.4 kn, 8.884025674746445 kn,
    // 0.4896, 65 points and 1522.048057913056 kW.
    assert_eq!(c.service_speed_knots, 5.2);
    close(c.limiting_speed_knots.unwrap(), 5.501113013453215);
    close(c.blockage_ratio, 0.26181818181818184);
    assert_eq!(c.speed_curve.len(), 33);
    close(c.power_at_service_kw, 246.57198379488898);
    assert_eq!(
        r.checks[0].message,
        "Cut: service 5.2 kn <= 0.95 x Schijf limit 5.50 kn"
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
    // Midship area (B x T, issue #396) equals channel area: n is exactly 1.0, and (0, 1) is open.
    let r = compute(&with_envs(vec![env("Plug", 6.0, 24.0, 24.0)])).unwrap();
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
    // Re-pinned (issues #396, #397); was 30.581921358527396.
    close(p.resistance_total_kn, 30.582327201001185);
    // Re-pinned (issues #396, #397); was 69.91706820811774.
    close(p.delivered_power_kw, 69.91799605420003);
    assert_eq!(r.environments[0].service_speed_knots, 11.0);
    e.wave_heading_deg_off_bow = 0.0;
    let head = compute(&with_envs(vec![e])).unwrap();
    // Re-pinned (issues #396, #397); was 38.89178587876913.
    close(
        head.environments[0].speed_curve[0].resistance_total_kn,
        38.89219172124292,
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
    // Re-pinned (issues #396, #397); was 29.783604634807425.
    close(r.environments[1].power_at_service_kw, 29.971994088409097);
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
    // Re-pinned (issues #396, #397); was 142.69389106055436.
    close(r.binding_power_at_service_kw(), 143.42648336242655);
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
    // Re-pinned (issue #399); was 27.529869204463804.
    close(p.resistance_residuary_kn, 32.86276119841476);
    // Re-pinned (issue #399); was 211.06233056755585.
    close(p.resistance_total_kn, 216.39522256150678);
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
    // Re-pinned (issues #396, #397); was 6.106217307692308.
    close(p.resistance_shallow_factor, 6.124961538461538);
    // Re-pinned (issues #396, #397); was 1499.7916759260709.
    // Re-pinned (issue #399); was 1504.3955804159937.
    close(p.resistance_total_kn, 1542.3339836642363);
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
    // Issue #397: the width that puts 0.95 x the Schijf limit on 5.0 kn (was 17.386400075809654
    // under the old limit formula).
    let mut e = env("Cut", 5.0, 93.3280161725555, 113.3280161725555);
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

// --- Knife-edge guards (issue #20). Expected values: the original Python. ---

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
    // Re-pinned (issue #399); was 6.615974179695845e-05.
    assert_eq!(p.resistance_total_kn, 6.760809550673473e-05);
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

// --- Issue #399: the residuary fit is continuous at Fn 0.20. ---

#[test]
fn the_residuary_ratio_from_the_left_and_the_right_of_fn_0_20_agree() {
    let left = residuary_ratio(0.20 - 1e-12);
    let right = residuary_ratio(0.20);
    assert!((left - right).abs() <= 1e-9 * right, "{left} vs {right}");
    // It was 0.15 at Fn 0.20, a step down from 0.179 just below.
    assert!((right - 0.17905694150420949).abs() < 1e-15);
}

#[test]
fn the_residuary_ratio_never_falls_as_the_froude_number_grows() {
    let mut last = residuary_ratio(0.0);
    for i in 1..=400 {
        let r = residuary_ratio(f64::from(i) / 1000.0);
        assert!(r >= last, "Fn {}", f64::from(i) / 1000.0);
        last = r;
    }
}

// --- Issue #400: the speed governing labels. ---

#[test]
fn a_service_speed_at_the_top_of_the_speed_grid_is_labelled_speed_grid() {
    let inp = ResistanceInputs {
        speed_max_knots: 6.0,
        ..with_envs(vec![deep("Deep")])
    };
    let r = compute(&inp).unwrap();
    let e = &r.environments[0];
    assert_eq!(e.service_speed_knots, 6.0);
    assert_eq!(e.service_speed_governed_by, "speed grid");
}

#[test]
fn a_sweep_cut_by_the_channel_cap_is_labelled_channel_even_off_the_grid_step() {
    // 60 m wide: 0.95 x the limit is about 3.48 kn, so the 0.1 kn grid stops at 3.4 kn, more than
    // 0.05 kn below the cap. It was labelled "power".
    let mut e = env("Cut", 5.0, 60.0, 80.0);
    e.apply_squat_limit = false;
    let inp = ResistanceInputs {
        speed_step_knots: 0.1,
        ..with_envs(vec![e])
    };
    let r = compute(&inp).unwrap();
    let c = &r.environments[0];
    assert_eq!(c.service_speed_knots, 3.4);
    assert_eq!(c.service_speed_governed_by, "channel");
}

#[test]
fn a_service_speed_that_fails_squat_is_flagged() {
    // The default river: no speed point keeps the under-keel clearance at its depth, so the first
    // point stands in, and the flag says so.
    let mut e = env("Shoal", 6.3, 40.0, 60.0);
    e.squat_safety_margin_m = 0.5;
    let r = compute(&with_envs(vec![e])).unwrap();
    let flag = r
        .checks
        .iter()
        .find(|c| c.name == "service_speed_clears_squat[Shoal]")
        .expect("the flag");
    assert!(!flag.ok);
    assert_eq!(flag.severity, "warning");
    let e = &r.environments[0];
    let p = e
        .speed_curve
        .iter()
        .find(|p| p.speed_knots == e.service_speed_knots)
        .unwrap();
    assert!(p.squat_limited);
}

// --- Issue #420: the method checks at the service speed reach the results. ---

#[test]
fn each_channel_gets_the_squat_and_fit_checks_at_its_service_speed() {
    let r = compute(&ResistanceInputs::default()).unwrap();
    let names: Vec<&str> = r.checks.iter().map(|c| c.name.as_str()).collect();
    for env in ["Confined shallow river", "Confined creek"] {
        for check in [
            "keel_clears_bed",
            "keel_clears_bed_with_margin",
            "froude_depth_subcritical",
            "lackenby_fit_subcritical",
        ] {
            let name = format!("{check}[{env}]");
            assert!(names.contains(&name.as_str()), "{name}: {names:?}");
        }
    }
    // Deep water has no squat and no shallow-water fit.
    assert!(
        !names
            .iter()
            .any(|n| n.ends_with("[Open Sea]") && n.starts_with("keel_clears_bed"))
    );
    assert!(!names.contains(&"lackenby_fit_subcritical[Open Sea]"));
    // The squat checks are the ones squat.rs gives at the service speed through the water.
    let river = &r.environments[1];
    let point = river
        .speed_curve
        .iter()
        .find(|p| p.speed_knots == river.service_speed_knots)
        .unwrap();
    let keel = r
        .checks
        .iter()
        .find(|c| c.name == "keel_clears_bed[Confined shallow river]")
        .unwrap();
    assert!(
        keel.message
            .ends_with(&format!("at {:.2} kn", point.speed_through_water_knots)),
        "{}",
        keel.message
    );
    // The fit check is a warning: the method loses validity, the design does not fail.
    let fit = r
        .checks
        .iter()
        .find(|c| c.name == "lackenby_fit_subcritical[Confined creek]")
        .unwrap();
    assert_eq!(fit.severity, "warning");
    assert!(fit.ok);
}

#[test]
fn waves_give_the_stawave1_checks_at_the_service_speed() {
    let mut inp = ResistanceInputs::default();
    inp.environments[0].significant_wave_height_m = 1.0;
    let r = compute(&inp).unwrap();
    let name = format!("stawave1_short_wave_validity[{}]", inp.environments[0].name);
    let c = r
        .checks
        .iter()
        .find(|c| c.name == name)
        .expect("the wave check");
    assert_eq!(c.severity, "warning");
}

#[test]
fn the_lackenby_fit_check_fails_above_its_depth_froude_limit() {
    // Shallow open water (no channel width, so no Schijf cap) and a high speed: the service speed
    // sits above Fnh 0.95. In a channel the cap of 0.95 x the Schijf limit keeps it below.
    let mut inp = ResistanceInputs::default();
    let env = &mut inp.environments[1];
    env.water_depth = 3.0;
    env.channel_width_bottom = 0.0;
    env.channel_width_surface = 0.0;
    env.apply_squat_limit = false;
    inp.speed_min_knots = 10.0;
    inp.speed_max_knots = 10.0;
    let r = compute(&inp).unwrap();
    let name = format!("lackenby_fit_subcritical[{}]", inp.environments[1].name);
    let c = r
        .checks
        .iter()
        .find(|c| c.name == name)
        .expect("the fit check");
    assert!(!c.ok, "{}", c.message);
}
