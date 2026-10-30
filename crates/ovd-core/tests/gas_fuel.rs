//! The gas a gas-fuelled Power Source burns, and the boil-off of a Type C tank (issue #464).
//!
//! Label: sourced - Wartsila 20DF Product Guide 1/2018 p. 3-1 and Note 4 p. 3-3 (fuel-gas consumption
//! 8048, 8219, 8326 and 8862 kJ/kWh at 100, 85, 75 and 50 % load; gas LCV 49,620 kJ/kg); IGC Code 7.2
//! (45 C air); NIST saturation table of methane (latent heat 511.77 kJ/kg at -162 C). The heat leak is
//! worked by hand below.

use ovd_core::gas_fuel::*;

#[test]
fn the_fuel_gas_consumption_is_the_four_published_values_and_is_held_at_the_ends() {
    assert_eq!(fuel_gas_kj_per_kwh(1.00), 8048.0);
    assert_eq!(fuel_gas_kj_per_kwh(0.85), 8219.0);
    assert_eq!(fuel_gas_kj_per_kwh(0.75), 8326.0);
    assert_eq!(fuel_gas_kj_per_kwh(0.50), 8862.0);
    // Held below 50 % and above 100 %.
    assert_eq!(fuel_gas_kj_per_kwh(0.10), 8862.0);
    assert_eq!(fuel_gas_kj_per_kwh(1.20), 8048.0);
    // Linear between two loads: the middle of 75 and 85 %.
    assert!((fuel_gas_kj_per_kwh(0.80) - (8326.0 + 8219.0) / 2.0).abs() < 1e-9);
}

#[test]
fn the_gas_use_is_the_shaft_energy_over_the_calorific_value() {
    // 1,000 kWe at full load: shaft = 1000 / 0.955 kW, at 8048 kJ/kWh, over 49,620 kJ/kg.
    let by_hand = 1000.0 / 0.955 * 8048.0 / 49_620.0;
    assert!((gas_use_kg_per_h(1000.0, 1000.0) - by_hand).abs() < 1e-9);
    assert!((by_hand - 169.8).abs() < 0.1);
    // No need, or no sets: no gas.
    assert_eq!(gas_use_kg_per_h(0.0, 1000.0), 0.0);
    assert_eq!(gas_use_kg_per_h(100.0, 0.0), 0.0);
    // A part load burns more per kWh.
    let part = gas_use_kg_per_h(500.0, 1000.0) / 500.0;
    let full = gas_use_kg_per_h(1000.0, 1000.0) / 1000.0;
    assert!(part > full);
}

#[test]
fn the_heat_leak_of_a_tank_is_the_conduction_of_a_cylinder_and_a_sphere() {
    // Steel radius 2 m, outer 2.5 m, cylinder 10 m, 100 K, 0.03 W/m.K, by hand:
    // cylinder 2 pi k L dT / ln(2.5/2) = 844.7 W; sphere 4 pi k dT / (1/2 - 1/2.5) = 377.0 W.
    let q = tank_heat_leak_kw(2.0, 2.5, 10.0, 100.0, 0.03);
    assert!((q - 1.2217).abs() < 1e-3, "{q}");
    // No insulation, no heat leak by this model.
    assert_eq!(tank_heat_leak_kw(2.0, 2.0, 10.0, 100.0, 0.03), 0.0);
    // A thicker insulation leaks less.
    assert!(tank_heat_leak_kw(2.0, 2.6, 10.0, 100.0, 0.03) < q);
}

#[test]
fn the_latent_heat_of_methane_is_the_nist_value_and_the_boil_off_follows() {
    assert_eq!(methane_latent_heat_kj_per_kg(-162.0), 511.77);
    assert_eq!(methane_latent_heat_kj_per_kg(-161.0), 509.94);
    // Between two rows: linear.
    assert!((methane_latent_heat_kj_per_kg(-161.5) - (511.77 + 509.94) / 2.0).abs() < 1e-9);
    // 1 kW over 511.77 kJ/kg is 3,600 / 511.77 kg/h.
    assert!((boil_off_kg_per_h(1.0, 511.77) - 3600.0 / 511.77).abs() < 1e-12);
    assert_eq!(UPPER_AMBIENT_AIR_C, 45.0);
}
