//! Seam: the flags and the tank of `ga_adapter::to_ga` (issue #413). No flag is a constant: each
//! follows what it names. The drawn tank's cylinder holds only the volume the heads do not.
//!
//! Label: property

use ovd_core::barge_capacity::{BargeCapacityInputs, BargeCapacityResult, compute};
use ovd_core::ga_adapter::to_ga;
use ovd_core::gas_arrangement::{FitOptions, fit};
use ovd_core::gas_stability::{GasCapacityResult, assess};

fn gas() -> BargeCapacityInputs {
    let mut i = BargeCapacityInputs::new(90.0, 14.8, 6.5);
    i.cargo_type = "LPG_propane".into();
    i
}

#[test]
fn the_freeboard_flag_follows_the_draft_drawn() {
    let i = gas();
    let r = compute(&i).unwrap();
    let (_, ok) = to_ga(&i, &r, Some(i.depth - i.required_freeboard - 0.1), true).unwrap();
    assert!(ok.min_freeboard_ok);
    let (_, low) = to_ga(&i, &r, Some(i.depth - i.required_freeboard + 0.1), true).unwrap();
    assert!(!low.min_freeboard_ok);
}

#[test]
fn the_air_draft_flag_follows_the_limit() {
    let mut i = gas();
    let r = compute(&i).unwrap();
    assert!(to_ga(&i, &r, None, true).unwrap().1.air_draft_ok);
    i.air_draft_limit_m = Some(r.air_draft_m + 1.0);
    assert!(to_ga(&i, &r, None, true).unwrap().1.air_draft_ok);
    i.air_draft_limit_m = Some(r.air_draft_m - 1.0);
    assert!(!to_ga(&i, &r, None, true).unwrap().1.air_draft_ok);
}

#[test]
fn the_stability_flag_is_the_gas_stability_verdict() {
    // One result that passes and one that fails, both with a gas basis. Issue #448: on the hull's
    // sections the default arrangements below fail (the Light condition's GZ peaks below 25
    // degrees, #498), so the passing one is the arrangement `fit` finds for 100 x 16 x 6 m.
    let mut cases: Vec<(BargeCapacityInputs, BargeCapacityResult)> = Vec::new();
    for (loa, beam, depth) in [(90.0, 14.8, 6.5), (90.0, 11.4, 6.5), (110.0, 16.0, 4.0)] {
        let mut i = BargeCapacityInputs::new(loa, beam, depth);
        i.cargo_type = "LPG_propane".into();
        let r = compute(&i).unwrap();
        cases.push((i, r));
    }
    let o = FitOptions {
        depths: vec![6.0],
        fill_ratio: 0.98,
        ..FitOptions::default()
    };
    let f = fit(100.0, 16.0, "LPG_propane", 5.7, &o).unwrap();
    assert!(f.feasible, "{:?}", f.failed_checks);
    cases.push((f.engine_inputs.clone(), f.engine_result.clone()));
    let mut seen = (false, false);
    for (i, r) in &cases {
        let a = assess(
            &GasCapacityResult::try_from(r).unwrap(),
            r.cargo_capacity_t,
            i.water_density,
            "IS_CODE_2008",
        )
        .unwrap();
        let flag = to_ga(i, r, None, true).unwrap().1.stability_ok;
        assert_eq!(flag, a.ok(), "{} x {} x {}", i.loa, i.beam, i.depth);
        if flag {
            seen.0 = true;
        } else {
            seen.1 = true;
        }
    }
    assert_eq!(seen, (true, true));
}

#[test]
fn the_drawn_tank_holds_the_tank_volume_once() {
    let i = gas();
    let r = compute(&i).unwrap();
    let t = to_ga(&i, &r, None, true).unwrap().1.tanks.unwrap();
    assert!(
        (t.volume_cylinder + t.volume_heads - t.volume_per_tank).abs() <= 1e-9 * t.volume_per_tank,
        "{} + {} vs {}",
        t.volume_cylinder,
        t.volume_heads,
        t.volume_per_tank
    );
}
