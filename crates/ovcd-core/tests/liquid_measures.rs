//! Label: sourced - hand calculation (the volume and length of a box tank, L x B x H of ADN 2025
//! 9.3.2.11.1 (a), and the slop share of the tank volume), for the layout of each test.

use ovcd_core::compartments::SpaceKind;
use ovcd_core::liquid_bulk::{LiquidBulkInputs, PumpArrangement, compute};
use ovcd_core::measured::Measured;
use ovcd_core::rule_trace::{MeasureValue, ObjectType};
use ovcd_core::rules::MeasureRef;

fn measured(centreline: bool, slop: f64) -> Measured {
    let r = compute(&LiquidBulkInputs {
        loa_m: 60.0,
        beam_m: 10.0,
        depth_m: 4.0,
        zone_aft_m: 5.0,
        zone_fwd_m: 55.0,
        double_side_m: 1.0,
        double_bottom_m: 0.7,
        tanks_in_line: 2,
        centreline_bulkhead: centreline,
        cofferdam_m: 0.6,
        pump_arrangement: PumpArrangement::Deepwell,
        slop_share: slop,
        fill_ratio: 0.98,
        cargo_density_t_m3: 0.87,
        heating_kw_per_m3: 0.0,
        lightship_k_t_per_m3: 0.1141,
    })
    .unwrap();
    Measured {
        liquid: Some(r),
        deadweight_t: Some(1234.5),
        ..Measured::default()
    }
}

fn at(m: &Measured, id: &str, index: usize) -> f64 {
    let r = MeasureRef {
        id: id.to_string(),
        args: Vec::new(),
    };
    assert!(m.has(&r), "{id}");
    match m.value(&r, index).unwrap() {
        MeasureValue::Number(x) => x,
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_hull_measures() {
    let m = measured(false, 0.0);
    assert_eq!(at(&m, "hull.double_side_width", 0), 1.0);
    assert_eq!(at(&m, "hull.double_bottom_height", 0), 0.7);
    // L x B x H: 60 x 10 x 4.
    assert_eq!(at(&m, "hull.lbh", 0), 2400.0);
    assert_eq!(m.objects(ObjectType::Hull, "Option 1"), ["Hull"]);
}

// Zone 50 m, two cofferdams of 0.6 m, two tanks of 24.4 m; inner breadth 8 m, height 3.3 m.
#[test]
fn the_cargo_tank_measures_by_tank() {
    let m = measured(false, 0.0);
    assert_eq!(m.objects(ObjectType::CargoTank, "Option 1"), ["1C", "2C"]);
    for i in 0..2 {
        assert!((at(&m, "cargo-tank.length", i) - 24.4).abs() < 1e-9);
        assert!((at(&m, "cargo-tank.volume", i) - 24.4 * 8.0 * 3.3).abs() < 1e-9);
    }
    let two = measured(true, 0.0);
    assert_eq!(
        two.objects(ObjectType::CargoTank, "o"),
        ["1P", "1S", "2P", "2S"]
    );
    assert!((at(&two, "cargo-tank.volume", 3) - 24.4 * 4.0 * 3.3).abs() < 1e-9);
    // A tank that is not there has no value.
    let r = MeasureRef {
        id: "cargo-tank.volume".to_string(),
        args: Vec::new(),
    };
    assert!(m.value(&r, 2).is_err());
}

// The per-tank copies of the hull's numbers, so a Rule on one cargo tank can read a table on its own object.
#[test]
fn a_cargo_tank_carries_the_hull_numbers_its_rules_read() {
    let m = measured(true, 0.0);
    for i in 0..4 {
        assert_eq!(at(&m, "cargo-tank.vessel_lbh", i), 2400.0);
        assert_eq!(at(&m, "cargo-tank.vessel_length", i), 60.0);
    }
    let r = MeasureRef {
        id: "cargo-tank.vessel_lbh".to_string(),
        args: Vec::new(),
    };
    assert!(m.value(&r, 4).is_err());
}

#[test]
fn the_option_measures() {
    let m = measured(true, 0.05);
    let l = m.liquid.as_ref().unwrap();
    let cargo: f64 = l.of_kind(SpaceKind::CargoTank).map(|s| s.volume_m3()).sum();
    let slop: f64 = l.of_kind(SpaceKind::Slop).map(|s| s.volume_m3()).sum();
    assert_eq!(at(&m, "option.cargo_capacity_m3", 0), cargo);
    // The slop tanks are 5 % of the tank block, the cargo tanks the other 95 %.
    assert!((at(&m, "option.slop_capacity_share", 0) - slop / (cargo + slop)).abs() < 1e-12);
    assert!((at(&m, "option.slop_capacity_share", 0) - 0.05).abs() < 1e-9);
    assert_eq!(at(&m, "option.deadweight", 0), 1234.5);
    assert_eq!(m.objects(ObjectType::SlopTank, "o"), ["SL1P", "SL1S"]);
    let ballast = m.objects(ObjectType::BallastTank, "o");
    assert!(ballast.contains(&"DB1P".to_string()) && ballast.contains(&"WB1S".to_string()));
}

#[test]
fn no_liquid_layout_no_liquid_measure() {
    let m = Measured::default();
    for id in [
        "hull.double_side_width",
        "hull.double_bottom_height",
        "hull.lbh",
        "cargo-tank.volume",
        "cargo-tank.length",
        "cargo-tank.vessel_lbh",
        "cargo-tank.vessel_length",
        "option.cargo_capacity_m3",
        "option.slop_capacity_share",
        "option.deadweight",
    ] {
        let r = MeasureRef {
            id: id.to_string(),
            args: Vec::new(),
        };
        assert!(!m.has(&r), "{id}");
        assert!(m.value(&r, 0).is_err(), "{id}");
    }
    assert!(m.objects(ObjectType::Hull, "o").is_empty());
    assert!(m.objects(ObjectType::CargoTank, "o").is_empty());
}
