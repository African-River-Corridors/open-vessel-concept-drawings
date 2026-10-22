//! Seam: the machinery space and its mass (issue #352, spec 0007 "Machinery space and mass").
//!
//! Hand sums at one power, 1,000 kW over two propulsors of D = 2.0 m, a 200 kW bow thruster
//! (Hekkenberg 2013 Appendix B rates, as spec 0007 gives them):
//! - shafts in nozzles, diesel-mechanical: two 500 kW engines (the small-engine rule); shaft lines 2 x
//!   1.0 t/m x (2 D + A = 5.0 m) = 10 t; rudders 2 x (2 x 1 t/m x 2.0 m + 0.5 t) = 9 t; engines 4.9 t;
//!   gearboxes 2 t; bow thruster 5.4 kg/kW x 200 = 1.08 t; small items 1 t; piping 3 + 2.5 = 5.5 t.
//!   Total 33.48 t.
//! - diesel-electric: three 500 kW gensets (N + 1), 9 kg/kVA x 1,875 kVA = 16.875 t; switchboards 1
//!   kg/kVA x 1,250 = 1.25 t; converters 0.9 + 2 x 1.25 = 3.4 t; motors 6 t; shaft lines 2 x 4.0 = 8 t;
//!   rudders 9 t; bow thruster, small items and piping 7.58 t. Total 52.105 t.
//! - battery hybrid with 2,000 kWh: engines 4.9 t, gearboxes 2 t, PTI motors 8 kg/kWe x 1,000 = 8 t,
//!   battery room 12 t, shaft lines 10 t, rudders 9 t, the rest 7.58 t. Total 53.48 t.
//! - ducted azimuth units, battery-electric: units 2 x 1.5 x 2.0^3 = 24 t; electrical 10.65 t; the
//!   rest 7.58 t. Total 42.23 t.
//!
//! Label: sourced - spec 0007 "Machinery space and mass" and docs/research/stern-and-tugs.md sec. 4 (Hekkenberg 2013, Appendix B pp. 221-225; the engine table from the maker sheets); the sums are hand calculations (above).

use ovcd_core::aft_arrangement::{AftArrangementInputs, compute as aft};
use ovcd_core::barge_capacity::{BargeCapacityInputs, compute};
use ovcd_core::machinery::{MachineryInputs, PowerSource, arrange, engine_for, mass_items};
use ovcd_core::propulsor::PropulsorType;
use ovcd_core::weights::total_mass;

fn machinery(source: PowerSource, propulsor: PropulsorType) -> MachineryInputs {
    MachineryInputs {
        power_source: source,
        propulsor,
        propulsor_count: 2,
        propeller_diameter_m: 2.0,
        installed_power_kw: Some(1000.0),
        battery_energy_kwh: Some(2000.0),
        bow_thruster_kw: Some(200.0),
        bow_thruster_lcg_m: 100.0,
        stern_run_m: 0.0,
    }
}

fn sum(source: PowerSource, propulsor: PropulsorType) -> f64 {
    let inp = machinery(source, propulsor);
    let m = arrange(&inp, 11.4, 4.0, 3.5, None);
    total_mass(&mass_items(&inp, &m, 4.0, 1.0, 0.0))
}

#[test]
fn each_power_source_sums_to_the_hand_sum_of_the_rates() {
    use PowerSource::*;
    use PropulsorType::*;
    let cases = [
        (DieselMechanical, ShaftNozzle, 33.48),
        (DieselElectric, ShaftNozzle, 52.105),
        (BatteryHybrid, ShaftNozzle, 53.48),
        (BatteryElectric, AzimuthDucted, 42.23),
    ];
    for (source, propulsor, hand) in cases {
        let got = sum(source, propulsor);
        assert!(
            (got - hand).abs() < 1e-9,
            "{source:?}: {got} against {hand}"
        );
    }
}

#[test]
fn the_engine_is_the_first_of_the_table_that_covers_the_power() {
    assert_eq!(engine_for(1500.0).unwrap().model, "Cat 3512C");
    assert_eq!(engine_for(1903.0).unwrap().length_m, 2.59);
    let e = engine_for(2000.0).unwrap();
    assert_eq!((e.model.as_str(), e.length_m), ("Cat 3516C", 3.761));
    assert_eq!(engine_for(3000.0).unwrap().model, "Everllence 16V175D");
    assert_eq!(engine_for(4400.0).unwrap().length_m, 3.941);
    assert!(engine_for(4401.0).is_none());
    // The small-engine rule: 2.59 m x (P / 955)^(1/3).
    let small = engine_for(500.0).unwrap();
    assert!((small.length_m - 2.59 * libm::cbrt(500.0 / 955.0)).abs() < 1e-12);
}

#[test]
fn the_rooms_follow_the_power_source() {
    use PowerSource::*;
    let shaft = |s| {
        arrange(
            &machinery(s, PropulsorType::ShaftNozzle),
            11.4,
            4.0,
            3.5,
            None,
        )
    };
    let labels = |s| {
        shaft(s)
            .zones
            .iter()
            .map(|z| z.0.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        labels(DieselMechanical),
        ["AFT PEAK, STEERING GEAR", "SHAFT RUN", "ENGINE ROOM"]
    );
    assert_eq!(labels(DieselElectric)[2], "GENSET ROOM");
    assert_eq!(labels(BatteryHybrid)[3], "BATTERY ROOM");
    assert_eq!(labels(BatteryElectric).len(), 2);
    // A diesel engine room: A + engine + gearbox + C, drawn 1.4 times as long (issue #365).
    let m = shaft(DieselMechanical);
    let e = m.engine.clone().unwrap();
    assert!(
        (m.engine_room_x1_m - m.engine_room_x0_m - 1.4 * (1.0 + e.length_m + 1.2 + 0.8)).abs()
            < 1e-12
    );
    assert_eq!(m.n_engines, 2);
    // The hybrid's battery room: 2,000 kWh / 70 kWh/m3 over (depth - 1 m) x (beam - 2 m).
    let h = shaft(BatteryHybrid);
    let room = h.battery_room_x1_m - h.battery_room_x0_m;
    assert!((room - 2000.0 / 70.0 / (3.0 * 9.4)).abs() < 1e-12);
    // No power: no engine room, and a warning that says why.
    let mut none = machinery(DieselMechanical, PropulsorType::ShaftNozzle);
    none.installed_power_kw = None;
    let m = arrange(&none, 11.4, 4.0, 3.5, None);
    assert!(m.engine.is_none() && m.zones.len() == 2);
    assert!(m.warnings[0].to_string().contains("not sized"));
}

#[test]
fn with_no_machinery_the_stern_block_is_todays_for_both_power_sources() {
    for gas in [false, true] {
        let mut inp = AftArrangementInputs::new(11.4, 6.0);
        if gas {
            inp.propulsion = "gas_fuelled".into();
            inp.n_battery_units = 0;
        }
        let r = aft(&inp).unwrap();
        assert!(r.machinery.is_none());
        assert_eq!(r.zones[0], ("THRUSTER ROOM".to_string(), 0.0, 3.5));
        if gas {
            assert_eq!(r.zones[1].0, "ENGINE ROOM (GAS GENSETS)");
            assert_eq!(r.zones[1].1, 3.5);
        }
        // Today's length rule: the deck items, or for a gas block the longer layer.
        let expected = if gas {
            r.deck_length_m.max(r.below_deck_length_m)
        } else {
            r.wheelhouse_x1_m
        };
        assert_eq!(r.length_m, expected);
    }
}

fn dry_hull(power_kw: f64) -> BargeCapacityInputs {
    let mut i = BargeCapacityInputs::new(110.0, 11.4, 4.0);
    i.cargo_type = "bauxite".into();
    i.self_propelled = true;
    i.lightship_k_t_per_m3 = Some(0.1093);
    i.aft_design.n_battery_units = Some(0);
    i.aft_design.n_accom_units = Some(0);
    let mut m = machinery(PowerSource::DieselMechanical, PropulsorType::ShaftNozzle);
    m.installed_power_kw = Some(power_kw);
    m.stern_run_m = 4.0;
    i.machinery = Some(m);
    i
}

#[test]
fn the_stern_block_lengthens_with_power_and_the_cargo_zone_shortens_by_as_much() {
    let low = compute(&dry_hull(1000.0)).unwrap();
    let high = compute(&dry_hull(4000.0)).unwrap();
    let block = |r: &ovcd_core::barge_capacity::BargeCapacityResult| {
        r.aft_arrangement.as_ref().unwrap().length_m
    };
    let grow = block(&high) - block(&low);
    // 500 to 2,000 kW per engine: the small-engine rule to the Cat 3516C.
    // The engine room is drawn 1.4 times as long as its parts (issue #365).
    let expected = 1.4 * (3.761 - 2.59 * libm::cbrt(500.0 / 955.0));
    assert!((grow - expected).abs() < 1e-9, "{grow}");
    let zone = |r: &ovcd_core::barge_capacity::BargeCapacityResult| r.cargo_zone_length_m.unwrap();
    assert!((zone(&low) - zone(&high) - grow).abs() < 1e-9);
    // The machinery's mass is in the lightship only with the stern block mass.
    assert!(low.stern_block_items.is_empty());
}
