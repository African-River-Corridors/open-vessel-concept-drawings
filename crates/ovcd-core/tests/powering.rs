//! Seam: power and transmission (issue #353, spec 0007 "Power and transmission").
//!
//! The diesel-electric chain loss is the Wartsila figure spec 0007 gives: 15 % at 1 MW or less,
//! 9.4 % at 5 MW, interpolated on log power and held flat outside. The rest is checked on a made-up
//! resistance curve (R = 0.5 v^2 kN, v in kn): a typed Installed Power gives the speed where
//! P_needed = P_B / margin; a sized one meets the Design Speed in every environment.
//!
//! Label: sourced - spec 0007 "Power and transmission" (the Wartsila chain loss figures, the transmission efficiencies); the curve is made up for the property tests.

use ovcd_core::machinery::PowerSource;
use ovcd_core::powering::{
    CurvePoint, EnvironmentCurve, PoweringInputs, compute, electric_chain_loss,
    round_up_to_engine_table, source_efficiency,
};
use ovcd_core::propulsor::{PropulsorInputs, PropulsorType};

fn curve(name: &str, top_kn: f64, squat_from: f64) -> EnvironmentCurve {
    let mut points = Vec::new();
    let mut v = 1.0;
    while v <= top_kn + 1e-9 {
        points.push(CurvePoint {
            speed_kn: v,
            speed_through_water_kn: v,
            resistance_kn: 0.5 * v * v,
            squat_limited: v > squat_from,
        });
        v += 0.5;
    }
    EnvironmentCurve {
        name: name.into(),
        points,
    }
}

fn inputs(source: PowerSource, installed: Option<f64>) -> PoweringInputs {
    PoweringInputs {
        propulsor: PropulsorInputs {
            propulsor_type: PropulsorType::ShaftNozzle,
            count: 2,
            diameter_m: 1.6,
            wake: 0.3,
            kappa: 0.8,
            kappa_b: 0.95,
            rho_kg_m3: 1000.0,
        },
        power_source: source,
        design_speed_kn: 9.0,
        margin: 1.2,
        installed_power_kw: installed,
        trip_length_km: None,
    }
}

#[test]
fn the_diesel_electric_loss_is_15_percent_at_1_mw_and_9_4_at_5_mw_on_log_power() {
    assert_eq!(electric_chain_loss(500.0), 0.15);
    assert_eq!(electric_chain_loss(1000.0), 0.15);
    assert!((electric_chain_loss(5000.0) - 0.094).abs() < 1e-15);
    assert_eq!(electric_chain_loss(9000.0), 0.094);
    // Halfway on log power: sqrt(1,000 x 5,000) kW.
    let mid = libm::sqrt(1000.0 * 5000.0);
    assert!((electric_chain_loss(mid) - (0.15 + 0.094) / 2.0).abs() < 1e-12);
    assert!((source_efficiency(PowerSource::DieselElectric, 1000.0) - 0.85).abs() < 1e-15);
    assert_eq!(
        source_efficiency(PowerSource::DieselMechanical, 1000.0),
        0.97
    );
    assert_eq!(
        source_efficiency(PowerSource::BatteryElectric, 1000.0),
        0.92
    );
}

#[test]
fn a_typed_installed_power_gives_the_speed_where_the_need_is_p_b_over_the_margin() {
    let p = compute(
        &inputs(PowerSource::DieselMechanical, Some(600.0)),
        &[curve("Deep water", 14.0, 99.0)],
    );
    assert!(!p.sized);
    assert_eq!(p.installed_power_kw, 600.0);
    let e = &p.environments[0];
    assert!((e.needed_at_service_kw - 600.0 / 1.2).abs() < 1e-9, "{e:?}");
    // Between the two curve points either side, the need crosses P_B / margin.
    let below = e
        .points
        .iter()
        .rfind(|q| q.speed_kn <= e.service_speed_kn)
        .unwrap();
    let above = e
        .points
        .iter()
        .find(|q| q.speed_kn > e.service_speed_kn)
        .unwrap();
    assert!(below.needed_kw <= 500.0 && above.needed_kw > 500.0);
    assert!(e.fuel_kg_per_h > 0.0);
}

#[test]
fn a_sized_installed_power_meets_the_design_speed_in_every_environment() {
    // A channel that adds resistance: R x 1.4.
    let mut channel = curve("Channel", 14.0, 99.0);
    for q in &mut channel.points {
        q.resistance_kn *= 1.4;
    }
    let envs = [curve("Deep water", 14.0, 99.0), channel];
    let p = compute(&inputs(PowerSource::BatteryElectric, None), &envs);
    assert!(p.sized);
    for e in &p.environments {
        assert!(
            e.service_speed_kn >= 9.0 - 1e-9,
            "{}: {}",
            e.name,
            e.service_speed_kn
        );
    }
    // Sized on the highest need of the environments (the channel), not on deep water alone.
    let deep_only = compute(&inputs(PowerSource::BatteryElectric, None), &envs[..1]);
    assert!(p.installed_power_kw > deep_only.installed_power_kw);
    // An environment that cannot reach the design speed (squat from 7 kn) sizes at its fastest.
    let shallow = [curve("Deep water", 14.0, 99.0), curve("Channel", 14.0, 7.0)];
    let s = compute(&inputs(PowerSource::BatteryElectric, None), &shallow);
    assert!((s.environments[1].service_speed_kn - 7.0).abs() < 1e-9);
    assert!(s.bollard_pull_t > 0.0);
}

#[test]
fn engine_power_rounds_up_only_in_a_gap_of_the_engine_table() {
    assert_eq!(round_up_to_engine_table(500.0), 500.0);
    assert_eq!(round_up_to_engine_table(1200.0), 1200.0);
    assert_eq!(round_up_to_engine_table(3300.0), 3400.0);
    assert_eq!(round_up_to_engine_table(5000.0), 5000.0);
    let p = compute(
        &inputs(PowerSource::DieselMechanical, None),
        &[curve("Deep water", 30.0, 99.0)],
    );
    // A diesel Installed Power is two engines of the table.
    let per_engine = p.installed_power_kw / 2.0;
    assert_eq!(round_up_to_engine_table(per_engine), per_engine);
}
