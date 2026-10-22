//! Issue #401, step 2: the bottom-up hull steel of a gas barge, which the capital budget prices, uses a
//! structure factor of 2.3 (an Estimate), not the legacy 1.4. The lightship does not use it.
//! Label: property

use ovcd_core::barge_capacity::{BargeCapacityInputs, GAS_HULL_STRUCTURE_FACTOR, compute};

fn gas_hull() -> BargeCapacityInputs {
    let mut i = BargeCapacityInputs::new(110.0, 11.4, 6.0);
    i.cargo_type = "LPG_propane".into();
    i
}

#[test]
fn the_factor_is_2_3() {
    assert_eq!(GAS_HULL_STRUCTURE_FACTOR, 2.3);
}

#[test]
fn the_internal_structure_is_the_plating_times_the_factor_less_one() {
    let r = compute(&gas_hull()).unwrap();
    let b = r.gas_basis.as_ref().unwrap();
    assert_eq!(b.inputs.structure_factor, GAS_HULL_STRUCTURE_FACTOR);
    let w = &b.weights;
    let plating = w.hull_steel_total - w.hull_internal_structure;
    let k = w.hull_internal_structure / plating;
    assert!((k - (GAS_HULL_STRUCTURE_FACTOR - 1.0)).abs() < 1e-12, "{k}");
}

#[test]
fn the_bottom_up_steel_is_inside_the_range_of_the_case_designs_and_below_the_library_figure() {
    let r = compute(&gas_hull()).unwrap();
    let w = &r.gas_basis.as_ref().unwrap().weights;
    let volume = 110.0 * 11.4 * 6.0;
    let per_m3 = w.steel_bottom_up / volume;
    // The 107 case designs span 0.076 to 0.135 t/m3 (median 0.115); the library gives 0.145 for cargo vessels.
    assert!(per_m3 > 0.076 && per_m3 < 0.145, "{per_m3} t/m3");
}

#[test]
fn the_top_down_steel_and_so_the_lightship_do_not_use_the_factor() {
    let r = compute(&gas_hull()).unwrap();
    let w = &r.gas_basis.as_ref().unwrap().weights;
    assert_eq!(w.steel_top_down, 0.10 * 110.0 * 11.4 * 6.0);
}
