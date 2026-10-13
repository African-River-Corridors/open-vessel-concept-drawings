//! Seam: the public `ovcd_core::electric_energy` API. Expected values are what the frozen Python
//! printed for the same inputs (see `parity/reference/electric_energy.json`); the full case set
//! runs in `ovcd-format/tests/parity.rs`.

use ovcd_core::aft_arrangement::PyException;
use ovcd_core::electric_energy::{EnergyInputs, compute};

fn close(a: f64, b: f64) {
    assert!((a - b).abs() <= 1e-9 * b.abs().max(1.0), "{a} vs {b}");
}

#[test]
fn the_80_m_worked_example() {
    let r = compute(&EnergyInputs::default()).unwrap();
    close(r.installed_power_kw, 204.609808407916);
    assert_eq!(r.binding().unwrap().name, "Design channel");
    let [channel, sea] = &r.environments[..] else {
        panic!("two environments")
    };
    assert_eq!(channel.name, "Design channel");
    assert_eq!(channel.service_speed_knots, 6.0);
    close(channel.energy_kwh_per_km, 15.344508069948105);
    close(channel.energy_per_year_mwh, 939.0838938808241);
    assert_eq!(sea.name, "Open sea");
    close(sea.energy_kwh_per_km, 9.758854050089464);
    close(sea.energy_per_year_mwh, 597.2418678654752);
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
        "service 6.0 kn stays below the Schijf limit 9.4 kn"
    );
    assert_eq!(r.checks[1].message, "energy 15.3 kWh/km at service speed");
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
    // Frozen Python, target 9.0 kn: Design channel 0.9 x 9.358 = 8.42 kn, Open sea 9.0 kn.
    let inp = EnergyInputs {
        target_service_speed_knots: 9.0,
        ..EnergyInputs::default()
    };
    let r = compute(&inp).unwrap();
    let [channel, sea] = &r.environments[..] else {
        panic!("two environments")
    };
    assert_eq!(channel.service_speed_knots, 8.42);
    assert_eq!(sea.service_speed_knots, 9.0);
    close(channel.battery_power_kw, 542.5283351789959);
    assert!((sea.battery_power_kw - 359.119).abs() < 1e-3);
    assert_eq!(r.binding().unwrap().name, "Design channel");
    close(r.installed_power_kw, 651.0340022147951);
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
