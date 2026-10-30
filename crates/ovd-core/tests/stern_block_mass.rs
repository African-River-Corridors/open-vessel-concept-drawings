//! Seam: the stern block's mass in the lightship (issue #348).
//!
//! The rates are those of spec 0007 ("Machinery space and mass"), from Hekkenberg 2013, Appendix B
//! pp. 221-225, as the research note gives them: a ducted azimuth unit 1.5 x D^3 t, D the thruster
//! envelope / 1.35; batteries 6 kg/kWh; accommodation 0.17 t per m^3 of unit volume; wheelhouse 8 t
//! and its column 5 t; a genset 9 kg/kVA (kVA = kWe / 0.8) of which the engine is 4.9 kg/kW; small
//! items 1 kg/kW; piping and cabling 3 t + 2.5 kg/kW.
//!
//! Hand sums, today's battery-electric block on an 11.4 x 6 m hull (three 2.60 m thrusters, one
//! battery unit of 5,000 kWh, two 40 ft high-cube accommodation units of 12.192 x 2.438 x 2.896 m):
//! thrusters 3 x 1.5 x (2.6 / 1.35)^3 = 32.146 t; battery 0.006 x 5000 = 30 t; accommodation
//! 2 x 0.17 x 86.081 = 29.268 t; wheelhouse 8 t; column 5 t; small items 0 t (no power); piping 3 t.
//! Total 107.414 t.
//!
//! Label: sourced - spec 0007 "Machinery space and mass" and docs/research/stern-and-tugs.md sec. 4.1 (Hekkenberg 2013, Appendix B pp. 221-225; Corvus battery string); the sums are hand calculations (above).

use ovd_core::aft_arrangement::{
    AftArrangementInputs, SternMassInputs, compute as aft, stern_block_mass,
};
use ovd_core::barge_capacity::{BargeCapacityInputs, compute};
use ovd_core::gas_stability::{GasCapacityResult, assess};
use ovd_core::weights::{total_mass, vertical_moment};

fn mass(items: &[ovd_core::weights::MassItem], label: &str) -> f64 {
    items
        .iter()
        .filter(|i| i.label == label)
        .map(|i| i.mass_t)
        .sum()
}

#[test]
fn a_battery_electric_block_is_the_hand_sum_of_the_rates() {
    let inp = AftArrangementInputs::new(11.4, 6.0);
    let res = aft(&inp).unwrap();
    let items = stern_block_mass(
        &inp,
        &res,
        &SternMassInputs {
            battery_unit_energy_kwh: 5000.0,
            installed_power_kw: None,
        },
        0.5,
    );
    let d: f64 = 2.6 / 1.35;
    assert!((mass(&items, "thrusters") - 3.0 * 1.5 * d * d * d).abs() < 1e-9);
    assert!((mass(&items, "thrusters") - 32.146).abs() < 5e-4);
    assert_eq!(mass(&items, "battery unit"), 30.0);
    let unit = 12.192 * 2.438 * 2.896;
    assert!((mass(&items, "accommodation unit") - 2.0 * 0.17 * unit).abs() < 1e-9);
    assert_eq!(mass(&items, "wheelhouse"), 8.0);
    assert_eq!(mass(&items, "lift column"), 5.0);
    assert_eq!(mass(&items, "small items"), 0.0);
    assert_eq!(mass(&items, "piping and cabling"), 3.0);
    assert!(
        (total_mass(&items) - 107.414).abs() < 5e-4,
        "{}",
        total_mass(&items)
    );
    // The centres: thrusters at the base + D, in the thruster room; the column at the deck.
    let t = items.iter().find(|i| i.label == "thrusters").unwrap();
    assert!((t.kcg_m - d).abs() < 1e-12);
    assert_eq!(t.lcg_m, res.thruster_room_length_m / 2.0);
    let c = items.iter().find(|i| i.label == "lift column").unwrap();
    assert_eq!(c.kcg_m, 6.0);
}

#[test]
fn a_gas_fuelled_block_adds_its_gensets_and_scales_the_piping_with_their_power() {
    let mut inp = AftArrangementInputs::new(11.4, 6.0);
    inp.propulsion = "gas_fuelled".into();
    inp.n_battery_units = 0;
    inp.electrical_load_kwe = Some(1500.0);
    let res = aft(&inp).unwrap();
    let items = stern_block_mass(
        &inp,
        &res,
        &SternMassInputs {
            battery_unit_energy_kwh: 5000.0,
            installed_power_kw: None,
        },
        0.5,
    );
    let n = res.n_gensets as f64;
    let alternator = 0.009 * res.genset_kwe / 0.8 - 0.0049 * res.genset_kwe;
    assert!((mass(&items, "gensets") - (res.genset_engine_mass_t + n * alternator)).abs() < 1e-9);
    assert_eq!(mass(&items, "battery unit"), 0.0);
    let kw = res.installed_kwe;
    assert!(kw > 0.0);
    assert!((mass(&items, "small items") - 0.001 * kw).abs() < 1e-9);
    assert!((mass(&items, "piping and cabling") - (3.0 + 0.0025 * kw)).abs() < 1e-9);
    // A typed installed power replaces the genset power.
    let typed = stern_block_mass(
        &inp,
        &res,
        &SternMassInputs {
            battery_unit_energy_kwh: 5000.0,
            installed_power_kw: Some(800.0),
        },
        0.5,
    );
    assert!((mass(&typed, "piping and cabling") - (3.0 + 0.0025 * 800.0)).abs() < 1e-9);
}

fn gas_hull(with_mass: bool) -> BargeCapacityInputs {
    let mut i = BargeCapacityInputs::new(110.0, 11.4, 6.0);
    i.cargo_type = "LNG".into();
    i.self_propelled = true;
    if with_mass {
        i.stern_block_mass = Some(SternMassInputs {
            battery_unit_energy_kwh: 5000.0,
            installed_power_kw: None,
        });
    }
    i
}

#[test]
fn the_lightship_rises_by_exactly_the_stern_block_and_the_kg_moves_by_its_moment() {
    let without = compute(&gas_hull(false)).unwrap();
    let with = compute(&gas_hull(true)).unwrap();
    assert!(without.stern_block_items.is_empty());
    let block = total_mass(&with.stern_block_items);
    assert!(block > 50.0, "{block}");
    assert_eq!(with.lightship_t, without.lightship_t + block);
    // The light condition (no cargo): KG x displacement grows by the block's vertical moment.
    let light = |r| {
        let a = assess(
            &GasCapacityResult::try_from(r).unwrap(),
            0.0,
            1.0,
            "IS_CODE_2008",
        )
        .unwrap();
        let c = a
            .conditions
            .iter()
            .find(|c| c.name == "Light")
            .unwrap()
            .clone();
        (c.kg_m, c.displacement_t, a.limitations)
    };
    let (kg0, disp0, lim0) = light(&without);
    let (kg1, disp1, lim1) = light(&with);
    assert!((disp1 - (disp0 + block)).abs() < 1e-6);
    let moment = vertical_moment(&with.stern_block_items);
    assert!((kg1 * disp1 - (kg0 * disp0 + moment)).abs() < 1e-6);
    // The "not in the lightship" limitation goes when the block is in.
    let says = |l: &Vec<ovd_core::text::Text>| {
        l.iter()
            .any(|t| t.to_string().contains("stern block's mass is not in"))
    };
    assert!(says(&lim0));
    assert!(!says(&lim1));
}

#[test]
fn a_dry_bulk_hull_takes_the_block_too() {
    let mut i = BargeCapacityInputs::new(90.0, 11.4, 4.0);
    i.cargo_type = "bauxite".into();
    i.self_propelled = true;
    i.lightship_k_t_per_m3 = Some(0.1093);
    let without = compute(&i).unwrap();
    i.stern_block_mass = Some(SternMassInputs {
        battery_unit_energy_kwh: 5000.0,
        installed_power_kw: None,
    });
    let with = compute(&i).unwrap();
    let block = total_mass(&with.stern_block_items);
    assert!(block > 0.0);
    assert_eq!(with.lightship_t, without.lightship_t + block);
}
