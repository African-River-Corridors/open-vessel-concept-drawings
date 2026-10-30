//! Seam: the gas-fuelled stern block of `ovd_core::aft_arrangement` (issue #180).
//!
//! Label: sourced - genset figures from Weichai Power, "Weichai Medium Speed Marine Engine"
//! (April 2023), slides 12 and 15 (engine dimensions, mass) and 30-31 (generating sets); block lengths are hand sums of the inputs,
//! written out in each test. Selection and battery-electric cases are properties.

use ovd_core::aft_arrangement::{
    AftArrangementInputs, GAS_GENSETS, PyException, compute, select_gensets,
};

fn gas(beam: f64, depth: f64, load: Option<f64>) -> AftArrangementInputs {
    let mut i = AftArrangementInputs::new(beam, depth);
    i.propulsion = "gas_fuelled".to_string();
    i.n_battery_units = 0;
    i.electrical_load_kwe = load;
    i
}

#[test]
fn the_catalogue_is_the_published_weichai_table() {
    assert_eq!(GAS_GENSETS.len(), 8);
    let g = &GAS_GENSETS[1];
    assert_eq!(
        (g.genset_model, g.kwe, g.rpm),
        ("6WH20LCD1200-1", 1050.0, 1000.0)
    );
    assert_eq!(
        (g.engine_length_m, g.engine_width_m, g.engine_height_m),
        (3.532, 1.365, 2.642)
    );
    assert_eq!(g.engine_mass_t, 9.8);
    let g = &GAS_GENSETS[7];
    assert_eq!(
        (g.genset_model, g.engine_model, g.kwe),
        ("CCFJ2200J-WMW", "8WH25LCD2450-1", 2200.0)
    );
    assert_eq!(
        (g.engine_length_m, g.engine_width_m, g.engine_height_m),
        (5.136, 1.71, 2.829)
    );
    // Envelope: engine + alternator 1.90 m long; width at least 1.90 m; + 0.25 m base frame.
    assert_eq!(g.length_m(), 5.136 + 1.90);
    assert_eq!(g.width_m(), 1.90);
    assert_eq!(g.height_m(), 2.829 + 0.25);
}

#[test]
fn selection_is_the_least_installed_power_with_one_set_in_reserve() {
    // 2 x 1050 = 2100 beats 3 x 900 = 2700.
    let (g, n) = select_gensets(1050.0).unwrap();
    assert_eq!((g.kwe, n), (1050.0, 2));
    // 3 x 1050 = 3150 beats 2 x 2200 = 4400 and 4 x 900 = 3600.
    let (g, n) = select_gensets(1910.0).unwrap();
    assert_eq!((g.kwe, n), (1050.0, 3));
    // 3 x 1200 = 4 x 900 = 3600: fewer sets win the tie.
    let (g, n) = select_gensets(2200.0).unwrap();
    assert_eq!((g.kwe, n), (1200.0, 3));
    // The biggest the catalogue carries: 5 of 6 x 2200 kWe.
    let (g, n) = select_gensets(11000.0).unwrap();
    assert_eq!((g.kwe, n), (2200.0, 6));
    assert!(select_gensets(11000.1).is_none());
}

#[test]
fn every_selection_carries_its_load_with_one_set_in_reserve() {
    let mut load = 50.0;
    while load <= 11000.0 {
        let (g, n) = select_gensets(load).unwrap();
        assert!((n - 1) as f64 * g.kwe >= load, "load {load}");
        assert!((2..=6).contains(&n));
        load += 137.0;
    }
}

#[test]
fn the_140_m_lng_barge_block_by_hand() {
    // 26 m beam, 7.0 m depth, 1910 kWe -> 3 x 6WH20LCD1200-1 (1050 kWe).
    //   below deck: thruster room 3.500 + engine room (3.532 + 1.90 + 2 x 1.20 = 7.832)
    //               + GVU room 2.000 = 13.332 m
    //   deck:       thruster room 3.500 + foundation 0.300 + 40 ft unit 12.192 = 15.992 m,
    //               wheelhouse at the forward end of the units, between them.
    let r = compute(&gas(26.0, 7.0, Some(1910.0))).unwrap();
    assert_eq!(r.propulsion, "gas_fuelled");
    assert_eq!(r.container_layout, "beside_wheelhouse");
    assert_eq!(
        (r.n_gensets, r.genset_kwe, r.installed_kwe),
        (3, 1050.0, 3150.0)
    );
    assert_eq!(r.genset_model, "6WH20LCD1200-1");
    assert!((r.below_deck_length_m - 13.332).abs() < 1e-9);
    assert!((r.deck_length_m - 15.992).abs() < 1e-9);
    assert!((r.length_m - 15.992).abs() < 1e-9);
    assert!((r.wheelhouse_x1_m - r.container_x1_m).abs() < 1e-12);
    assert!(r.container_labels.iter().all(|l| !l.contains("BATTERY")));
    assert_eq!(r.container_units.len(), 2);
    // Units either side of the cab: +- (3.20 + 0.90 + 1.219) m.
    assert!((r.container_units[0].3 - 5.319).abs() < 1e-9);
    assert!((r.container_units[1].3 + 5.319).abs() < 1e-9);
    assert_eq!(r.genset_offsets_m.len(), 3);
    assert!((r.genset_engine_mass_t - 3.0 * 9.8).abs() < 1e-9);
    assert!(r.ok());
    assert_eq!(r.warnings.len(), 1);
    assert!(r.warnings[0].contains("ESTIMATES"));
}

#[test]
fn the_block_is_as_long_as_the_longer_layer() {
    // A 9 m set: engine room 3.5 + 9 + 2.4 = 14.9, GVU to 16.9 m, longer than the 15.992 m deck.
    let mut i = gas(26.0, 7.0, None);
    i.genset_length_m = 9.0;
    let r = compute(&i).unwrap();
    assert!((r.below_deck_length_m - 16.9).abs() < 1e-9);
    assert_eq!(r.length_m, r.below_deck_length_m.max(r.deck_length_m));
    for beam in [12.0, 16.0, 20.0, 26.0, 34.0] {
        let r = compute(&gas(beam, 6.0, Some(1500.0))).unwrap();
        assert_eq!(r.length_m, r.below_deck_length_m.max(r.deck_length_m));
    }
}

#[test]
fn a_narrow_beam_puts_the_units_in_a_row_and_warns() {
    // 2 units + a 6.40 m cab + 4 walkways of 0.90 m = 14.876 m > 12 m.
    let r = compute(&gas(12.0, 5.0, None)).unwrap();
    assert_eq!(r.container_layout, "rows");
    let c = r
        .checks
        .iter()
        .find(|c| c.name == "accom_beside_wheelhouse")
        .unwrap();
    assert!(!c.ok);
    assert_eq!(c.severity, "warning");
    assert!(r.ok());
}

#[test]
fn gensets_too_wide_for_the_beam_fail() {
    let mut i = gas(6.0, 5.0, None);
    i.n_gensets = 4;
    let r = compute(&i).unwrap();
    let c = r
        .checks
        .iter()
        .find(|c| c.name == "gensets_fit_beam")
        .unwrap();
    assert!(!c.ok);
    assert!(!r.ok());
}

#[test]
fn invalid_gas_inputs_raise_value_errors() {
    let mut i = AftArrangementInputs::new(26.0, 7.0);
    i.propulsion = "diesel".to_string();
    assert!(matches!(compute(&i), Err(PyException::ValueError(m)) if m.contains("propulsion")));
    let mut i = gas(26.0, 7.0, None);
    i.n_battery_units = 1;
    assert!(matches!(compute(&i), Err(PyException::ValueError(m)) if m.contains("battery")));
    assert!(matches!(
        compute(&gas(26.0, 7.0, Some(20000.0))),
        Err(PyException::ValueError(_))
    ));
    let mut i = gas(26.0, 7.0, None);
    i.n_gensets = 1;
    assert!(matches!(compute(&i), Err(PyException::ValueError(_))));
}

#[test]
fn battery_electric_is_unchanged() {
    for (beam, depth) in [(8.0, 5.0), (14.8, 5.4), (26.0, 7.0)] {
        let r = compute(&AftArrangementInputs::new(beam, depth)).unwrap();
        assert_eq!(r.propulsion, "battery_electric");
        assert_eq!(r.container_layout, "rows");
        assert_eq!(r.n_gensets, 0);
        assert_eq!(r.length_m, r.deck_length_m);
        assert_eq!(r.engine_room_x1_m, 0.0);
        assert!(
            r.checks
                .iter()
                .all(|c| !c.name.starts_with("gensets") && c.name != "accom_beside_wheelhouse")
        );
    }
}
