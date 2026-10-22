//! Seam: trim from the lightship items, the cargo and the hull (issue #349).
//!
//! Label: property. The trim is the free-trim float of the hull's sections (issue #448); each test
//! states a rule of it, not a printed number.

use ovcd_core::aft_arrangement::SternMassInputs;
use ovcd_core::barge_capacity::{BargeCapacityInputs, compute};
use ovcd_core::gas_stability::{GasCapacityResult, assess};
use ovcd_core::hydrostatics::{Hull, HullSpec};
use ovcd_core::sections::Gravity;

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
fn with_free_trim_the_centre_of_buoyancy_is_under_the_centre_of_gravity() {
    // Issue #448: the trim is the free-trim float of the sections, so at equilibrium the LCB is the
    // LCG (a level hull: on the vertical through G).
    let r = compute(&gas_hull(true)).unwrap();
    let a = assess(
        &GasCapacityResult::try_from(&r).unwrap(),
        1500.0,
        1.0,
        "IS_CODE_2008",
    )
    .unwrap();
    for t in &a.trims {
        assert!(
            (t.LCB - t.LCG).abs() < 1e-3 * t.LCG.abs().max(1.0),
            "{} {}",
            t.LCB,
            t.LCG
        );
    }
}

#[test]
fn no_trim_when_g_is_over_b_and_a_heavy_stern_trims_by_the_stern() {
    let hull = Hull::build(&HullSpec {
        loa: 100.0,
        beam: 12.0,
        depth: 5.0,
        bow_rake_length_m: 0.0,
        bow_rake_rise_m: 5.0,
        stern: None,
        shape: Default::default(),
    })
    .unwrap();
    let u = hull.upright(3.0, 1.0).unwrap();
    let g = |lcg_m: f64| Gravity {
        lcg_m,
        tcg_m: 0.0,
        kg_m: 3.0,
    };
    let level = hull
        .float(u.displacement_t, g(u.immersed.lcb_m), 1.0)
        .unwrap();
    assert!(level.trim_m.abs() < 1e-9, "{}", level.trim_m);
    let heavy_aft = hull
        .float(u.displacement_t, g(u.immersed.lcb_m - 3.0), 1.0)
        .unwrap();
    assert!(heavy_aft.trim_m < 0.0);
    assert!(heavy_aft.draught_aft_m > heavy_aft.draught_fwd_m);
    // A box displaces the same at any small trim, so the mean draught stays.
    assert!(((heavy_aft.draught_aft_m + heavy_aft.draught_fwd_m) / 2.0 - 3.0).abs() < 1e-9);
}

#[test]
fn the_gas_conditions_carry_a_trim_only_with_the_stern_block_mass() {
    let run = |with| {
        let r = compute(&gas_hull(with)).unwrap();
        assess(
            &GasCapacityResult::try_from(&r).unwrap(),
            1500.0,
            1.0,
            "IS_CODE_2008",
        )
        .unwrap()
    };
    assert!(run(false).trims.is_empty());
    let with = run(true);
    assert_eq!(with.trims.len(), with.conditions.len());
    // Each condition trims to put its centre of buoyancy under its centre of gravity: by the bow
    // when G is forward of the level hull's LCB, by the stern when it is aft. Issue #448: the level
    // LCB is the sections' (the raked bow takes volume from forward), not mid-length.
    let r = compute(&gas_hull(true)).unwrap();
    let hull = ovcd_core::hydrostatics::Hull::build(r.hull_spec.as_ref().unwrap()).unwrap();
    for (c, t) in with.conditions.iter().zip(&with.trims) {
        let level = hull.level_draught(c.displacement_t, 1.0).unwrap();
        let lcb = hull.upright(level, 1.0).unwrap().immersed.lcb_m;
        assert!(t.trim != 0.0, "{}", c.name);
        assert_eq!(
            t.draft_fwd > t.draft_aft,
            t.LCG > lcb,
            "{}: LCG {} LCB {lcb}",
            c.name,
            t.LCG
        );
    }
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
