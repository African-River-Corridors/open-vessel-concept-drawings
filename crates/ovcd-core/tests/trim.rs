//! Seam: trim from the lightship items, the cargo and the hull (issue #349).
//!
//! Label: property. The trim is the legacy `calculate_trim` formula on the item LCGs; each test
//! states a rule of it, not a printed number.

use ovcd_core::aft_arrangement::SternMassInputs;
use ovcd_core::barge_capacity::{BargeCapacityInputs, compute};
use ovcd_core::gas_stability::{GasCapacityResult, assess};
use ovcd_core::hull::HullGeometry;
use ovcd_core::hydrostatics::{calculate_trim, trim};

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
fn on_a_box_hull_the_trim_is_the_legacy_trim_with_the_same_items() {
    let r = compute(&gas_hull(false)).unwrap();
    let b = r.gas_basis.unwrap();
    let (inputs, tanks, w) = (&b.inputs, &b.tanks, &b.weights);
    let hull = HullGeometry::new(inputs.loa, inputs.beam, inputs.depth, 0.0, 0.0, 0.0, 0.0);
    let draft = 3.0;
    let cargo = 1500.0;
    let legacy = calculate_trim(&hull, inputs, tanks, w, cargo, draft).unwrap();
    let stern = inputs.stern_non_tank_length.unwrap();
    let zone = (stern + inputs.loa - inputs.bow_non_tank_length.unwrap()) / 2.0;
    let items = [
        (w.hull_steel_total, inputs.loa / 2.0),
        (w.tank_shells + w.insulation, zone),
        (cargo, zone),
        (w.piping_machinery, inputs.loa / 2.0),
        (w.outfitting, stern / 2.0),
        (w.stores, stern / 2.0),
    ];
    let new = trim(&hull, draft, &items);
    assert!((new.LCG - legacy.LCG).abs() < 1e-9);
    assert!(
        (new.trim - legacy.trim).abs() < 1e-9,
        "{} {}",
        new.trim,
        legacy.trim
    );
    assert!((new.draft_aft - legacy.draft_aft).abs() < 1e-9);
    assert!((new.draft_fwd - legacy.draft_fwd).abs() < 1e-9);
}

#[test]
fn no_trim_when_the_lcg_is_the_lcb_and_a_heavy_stern_trims_by_the_stern() {
    let hull = HullGeometry::new(100.0, 12.0, 5.0, 0.0, 0.0, 0.0, 0.0);
    let draft = 3.0;
    let lcb = hull.LCB(draft);
    let level = trim(&hull, draft, &[(1000.0, lcb), (200.0, lcb)]);
    assert_eq!(level.trim, 0.0);
    assert_eq!(level.draft_aft, level.draft_fwd);
    let heavy_aft = trim(&hull, draft, &[(1000.0, lcb), (200.0, 5.0)]);
    assert!(heavy_aft.trim < 0.0);
    assert!(heavy_aft.draft_aft > heavy_aft.draft_fwd);
    assert!((heavy_aft.draft_aft + heavy_aft.draft_fwd) / 2.0 - draft < 1e-12);
}

#[test]
fn the_gas_conditions_carry_a_trim_only_with_the_stern_block_mass() {
    let run = |with| {
        let r = compute(&gas_hull(with)).unwrap();
        assess(
            &GasCapacityResult::try_from(&r).unwrap(),
            1500.0,
            0.85,
            1.0,
            "IS_CODE_2008",
        )
        .unwrap()
    };
    assert!(run(false).trims.is_empty());
    let with = run(true);
    assert_eq!(with.trims.len(), with.conditions.len());
    // The light hull, its stern block aft and no cargo forward of it, trims by the stern.
    let light = with
        .conditions
        .iter()
        .position(|c| c.name == "Light")
        .unwrap();
    assert!(with.trims[light].draft_aft > with.trims[light].draft_fwd);
}

#[test]
fn a_dry_bulk_hull_trims_laden_and_light_with_the_block() {
    let mut i = BargeCapacityInputs::new(90.0, 11.4, 4.0);
    i.cargo_type = "bauxite".into();
    i.self_propelled = true;
    i.lightship_k_t_per_m3 = Some(0.1093);
    assert!(compute(&i).unwrap().trim_conditions.is_empty());
    i.stern_block_mass = Some(SternMassInputs {
        battery_unit_energy_kwh: 5000.0,
        installed_power_kw: None,
    });
    let r = compute(&i).unwrap();
    let names: Vec<&str> = r.trim_conditions.iter().map(|(n, _)| *n).collect();
    assert_eq!(names, ["Laden", "Light"]);
    let light = &r.trim_conditions[1].1;
    assert!(light.draft_aft > light.draft_fwd);
}
