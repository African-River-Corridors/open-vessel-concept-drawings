//! Seam: the public `ovd_core::electric_energy` API. Expected values are what the original Python
//! printed for the same inputs.
//! Label: snapshot
//! A value re-pinned after an agreed change is what the code printed then; the old value is named next to it.

use ovd_core::aft_arrangement::PyException;
use ovd_core::confined_water::LIMITING_SPEED_MARGIN;
use ovd_core::electric_energy::{EnergyInputs, compute};

#[track_caller]
fn close(a: f64, b: f64) {
    assert!((a - b).abs() <= 1e-9 * b.abs().max(1.0), "{a} vs {b}");
}

#[test]
fn the_80_m_worked_example() {
    let r = compute(&EnergyInputs::default()).unwrap();
    // Re-pinned (issues #396, #397); was 204.609808407916.
    close(r.installed_power_kw, 137.87023704447876); // Re-pinned (issue #526); was 137.93175346474166.
    assert_eq!(r.binding().unwrap().name, "Design channel");
    let [channel, sea] = &r.environments[..] else {
        panic!("two environments")
    };
    assert_eq!(channel.name, "Design channel");
    // Re-pinned (issues #396, #397); was 6.0: 0.9 x the lower Schijf limit 5.85 kn.
    assert_eq!(channel.service_speed_knots, 5.26);
    // Re-pinned (issues #396, #397, #398); was 15.344508069948105.
    close(channel.energy_kwh_per_km, 11.783462644747221); // Re-pinned (issue #526); was 11.785483829983919.
    // Re-pinned (issues #396, #397, #398); was 939.0838938808241.
    close(channel.energy_per_year_mwh, 721.1479138585299); // Re-pinned (issue #526); was 721.2716103950158.
    assert_eq!(sea.name, "Open sea");
    close(sea.energy_kwh_per_km, 9.761370137809697); // Re-pinned (issue #526); was 9.758854050089464.
    close(sea.energy_per_year_mwh, 597.3958524339535); // Re-pinned (issue #526); was 597.2418678654752.
}

#[test]
fn the_curve_runs_on_the_rounded_speed_grid() {
    let r = compute(&EnergyInputs::default()).unwrap();
    let speeds: Vec<f64> = r.environments[0]
        .curve
        .iter()
        .take(12)
        .map(|p| p.speed_knots)
        .collect();
    assert_eq!(
        speeds,
        [2.0, 2.2, 2.5, 2.8, 3.0, 3.2, 3.5, 3.8, 4.0, 4.2, 4.5, 4.8]
    );
}

#[test]
fn the_check_texts_match_python() {
    let r = compute(&EnergyInputs::default()).unwrap();
    assert_eq!(r.checks.len(), 3);
    assert_eq!(r.checks[0].name, "service_below_schijf");
    assert_eq!(
        r.checks[0].message,
        // Re-pinned (issues #396, #397); was "service 6.0 kn ... limit 9.4 kn".
        "service 5.3 kn stays below the Schijf limit 5.8 kn"
    );
    // Re-pinned (issues #396, #397); was 15.3.
    assert_eq!(r.checks[1].message, "energy 11.8 kWh/km at service speed");
    assert_eq!(r.checks[2].severity, "warning");
    assert!(r.ok());
}

#[test]
fn a_zero_drivetrain_efficiency_is_a_zero_division() {
    let inp = EnergyInputs {
        drivetrain_efficiency: 0.0,
        ..EnergyInputs::default()
    };
    assert_eq!(
        compute(&inp).unwrap_err(),
        PyException::ZeroDivisionError("float division by zero".into())
    );
}

#[test]
fn a_zero_draft_fails_in_the_resistance_model() {
    let inp = EnergyInputs {
        draft: 0.0,
        ..EnergyInputs::default()
    };
    assert_eq!(
        compute(&inp).unwrap_err(),
        PyException::ValueError("draft_m must be > 0".into())
    );
}

#[test]
fn a_fast_target_is_capped_at_the_schijf_fraction_and_binds_the_design_channel() {
    // Target 9.0 kn: Design channel 0.9 x 5.85 = 5.26 kn (issues #396, #397; the original Python
    // had 0.9 x 9.358 = 8.42 kn), Open sea 9.0 kn.
    let inp = EnergyInputs {
        target_service_speed_knots: 9.0,
        ..EnergyInputs::default()
    };
    let r = compute(&inp).unwrap();
    let [channel, sea] = &r.environments[..] else {
        panic!("two environments")
    };
    assert_eq!(channel.service_speed_knots, 5.26);
    assert_eq!(sea.service_speed_knots, 9.0);
    // Re-pinned (issues #396, #397, #398); was 542.5283351789959.
    close(channel.battery_power_kw, 114.89186420373231); // Re-pinned (issue #526); was 114.94312788728473.
    assert!((sea.battery_power_kw - 359.2209).abs() < 1e-3); // Re-pinned (issue #526); was 359.119.
    assert_eq!(r.binding().unwrap().name, "Design channel");
    // Re-pinned (issues #396, #397, #398); was 651.0340022147951.
    close(r.installed_power_kw, 431.0650519804802); // Re-pinned (issue #526); was 430.9425386458399.
}

#[test]
fn an_empty_sweep_fails_the_energy_check_and_ok_is_false() {
    let inp = EnergyInputs {
        speed_min_knots: 5.0,
        speed_max_knots: 1.0,
        ..EnergyInputs::default()
    };
    let r = compute(&inp).unwrap();
    assert!(r.environments[0].curve.is_empty());
    assert_eq!(r.environments[0].energy_kwh_per_km, 0.0);
    assert!(!r.ok());
}

// --- Inputs the earlier case set never varied. Expected values are the original Python's. ---

#[test]
fn a_higher_speed_floor_shortens_the_curve() {
    let inp = EnergyInputs {
        speed_min_knots: 3.0,
        ..EnergyInputs::default()
    };
    let r = compute(&inp).unwrap();
    // Re-pinned (issues #396, #397); was 24.
    assert_eq!(r.environments[0].curve.len(), 11);
    assert_eq!(r.environments[1].curve.len(), 29);
    assert_eq!(r.environments[0].curve[0].speed_knots, 3.0);
    // Python: the first point is at 3.0 kn.
    close(
        r.environments[0].curve[0].energy_kwh_per_km,
        // Re-pinned (issues #396, #397, #398); was 4.3140884885663775.
        4.334920432285545, // Re-pinned (issue #526); was 4.333678574225196.
    );
}

#[test]
fn a_better_propulsive_coefficient_cuts_the_installed_power() {
    let inp = EnergyInputs {
        propulsive_coefficient: 0.6,
        ..EnergyInputs::default()
    };
    let r = compute(&inp).unwrap();
    // Re-pinned (issues #396, #397); was 153.457356305937.
    close(r.installed_power_kw, 103.40267778335907); // Re-pinned (issue #526); was 103.44881509855627.
    let [channel, sea] = &r.environments[..] else {
        panic!("two environments")
    };
    // Re-pinned (issues #396, #397); was 127.88113025494751.
    close(channel.battery_power_kw, 86.16889815279923); // Re-pinned (issue #526); was 86.20734591546356.
    // Re-pinned (issues #396, #397); was 11.508381052461079.
    close(channel.energy_kwh_per_km, 8.837596983560418); // Re-pinned (issue #526); was 8.83911287248794.
    close(sea.battery_power_kw, 81.35125872850602); // Re-pinned (issue #526); was 81.33028965344562.
    close(sea.energy_kwh_per_km, 7.321027603357273); // Re-pinned (issue #526); was 7.3191405375671.
    // Re-pinned (issues #396, #397); was 28.
    assert_eq!(channel.curve.len(), 15);
    // Re-pinned (issues #396, #397, #398); was 1.7935078737878523.
    close(channel.curve[0].energy_kwh_per_km, 1.79828711288645); // Re-pinned (issue #526); was 1.7978095965118202.
}

#[test]
fn a_zero_speed_floor_gives_zero_energy_at_the_first_point() {
    let inp = EnergyInputs {
        speed_min_knots: 0.0,
        ..EnergyInputs::default()
    };
    let r = compute(&inp).unwrap();
    // Re-pinned (issues #396, #397); was 36.
    assert_eq!(r.environments[0].curve.len(), 23);
    assert_eq!(r.environments[0].curve[0].speed_knots, 0.0);
    assert_eq!(r.environments[0].curve[0].energy_kwh_per_km, 0.0);
    assert_eq!(r.environments[1].curve.len(), 41);
    // Re-pinned (issues #396, #397, #398); was 204.609808407916.
    close(r.installed_power_kw, 137.87023704447876); // Re-pinned (issue #526); was 137.93175346474166.
}

#[test]
fn a_zero_schijf_limit_fails_the_check_and_sizes_the_power_from_the_floor() {
    // Bottom width 5 m = top width 5 m: the confined limit is 0.0 kn.
    let inp = EnergyInputs {
        channel_bottom_width_m: 5.0,
        channel_top_width_m: 5.0,
        ..EnergyInputs::default()
    };
    let r = compute(&inp).unwrap();
    let channel = &r.environments[0];
    assert_eq!(channel.limiting_speed_knots, Some(0.0));
    assert_eq!(channel.service_speed_knots, 6.0);
    assert_eq!(r.checks[0].name, "service_below_schijf");
    assert!(!r.checks[0].ok);
    assert_eq!(r.checks[0].message, "no confined limit (open water)");
    assert!(!r.ok());
    // Re-pinned (issues #396, #397); was 328.4312595149517.
    close(r.installed_power_kw, 353.1430469050719); // Re-pinned (issue #526); was 352.94684093930033.
    // Re-pinned (issues #396, #397); was 24.630374033698683.
    close(channel.energy_kwh_per_km, 26.483609829094068); // Re-pinned (issue #526); was 26.468895558802824.
}

#[test]
fn a_service_speed_exactly_at_the_margin_of_the_limit_passes_and_leaves_an_empty_curve() {
    // Knife edge (issue #397): the limit is exactly 9.0 kn (the channel width solved for it), the
    // service speed is exactly 0.95 x 9.0 kn, the one margin on the limit, and the sweep starts
    // above it at 9.9 kn. Was a 1.0 x limit case on the old limit formula and its 1.0 margin.
    let inp = EnergyInputs {
        channel_bottom_width_m: 179.9658564723023, // Re-pinned (issue #526); was 179.68075521333895.
        channel_top_width_m: 203.9658564723023,
        speed_min_knots: 9.9,
        target_service_speed_knots: 9.5,
        service_speed_safety_fraction: LIMITING_SPEED_MARGIN,
        ..EnergyInputs::default()
    };
    let r = compute(&inp).unwrap();
    let channel = &r.environments[0];
    assert_eq!(channel.limiting_speed_knots, Some(9.0));
    assert_eq!(channel.service_speed_knots, 8.55);
    assert!(channel.curve.is_empty());
    assert!(r.checks[0].ok);
    assert_eq!(
        r.checks[0].message,
        "service 8.6 kn stays below the Schijf limit 9.0 kn"
    );
    assert!(!r.checks[1].ok);
    assert_eq!(r.environments[1].service_speed_knots, 9.5);
    // Issue #398: the open sea sizes the installed power, not the channel's empty curve (was 0.0).
    assert_eq!(r.sizing().unwrap().name, "Open sea");
    assert_eq!(
        r.installed_power_kw,
        r.environments[1].battery_power_kw * inp.power_margin
    );
}

// --- Knife-edge guards (issue #20). Expected values: the original Python. ---

#[test]
fn py_min_of_equal_items_keeps_the_first_signed_zero() {
    // min(-0.0, 0.0, 5.8) is the first of the equal items: -0.0, not 0.0.
    let inp = EnergyInputs {
        target_service_speed_knots: -0.0,
        speed_min_knots: 0.0,
        speed_max_knots: 0.0,
        ..EnergyInputs::default()
    };
    let r = compute(&inp).unwrap();
    for env in &r.environments {
        assert_eq!(env.service_speed_knots, 0.0);
        assert!(env.service_speed_knots.is_sign_negative());
    }
    assert_eq!(
        r.checks[0].message,
        // Re-pinned (issues #396, #397); was "... limit 9.4 kn".
        "service -0.0 kn stays below the Schijf limit 5.8 kn"
    );
}

#[test]
fn py_min_of_equal_items_keeps_a_positive_zero_when_it_comes_first() {
    let inp = EnergyInputs {
        target_service_speed_knots: 0.0,
        service_speed_safety_fraction: -0.0,
        ..EnergyInputs::default()
    };
    let r = compute(&inp).unwrap();
    assert!(r.environments[0].service_speed_knots.is_sign_positive());
    assert_eq!(
        r.checks[0].message,
        // Re-pinned (issues #396, #397); was "... limit 9.4 kn".
        "service 0.0 kn stays below the Schijf limit 5.8 kn"
    );
}

// --- Issue #398: the installed power follows the hardest environment. ---

#[test]
fn the_installed_power_follows_the_highest_battery_power_not_the_lowest_service_speed() {
    // An 8 kn target: the channel caps the service at 0.9 x its Schijf limit, so it has the
    // lowest service speed; the open sea runs at 8 kn and needs more battery power.
    let inp = EnergyInputs {
        target_service_speed_knots: 8.0,
        ..EnergyInputs::default()
    };
    let r = compute(&inp).unwrap();
    let [channel, sea] = &r.environments[..] else {
        panic!("two environments")
    };
    assert!(channel.service_speed_knots < sea.service_speed_knots);
    assert!(sea.battery_power_kw > channel.battery_power_kw);
    assert_eq!(r.binding().unwrap().name, "Design channel");
    assert_eq!(r.sizing().unwrap().name, "Open sea");
    assert_eq!(
        r.installed_power_kw,
        sea.battery_power_kw * inp.power_margin
    );
}
