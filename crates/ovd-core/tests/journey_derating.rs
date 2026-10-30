//! Seam: the public `ovd_core::journey_derating` API. Expected values are printed by the original
//! Python (`core/journey_derating.py`), not worked out by hand. JSON cannot carry a NaN or an
//! infinity, so these tests, not the JSON seam, reach those paths.
//! Label: snapshot

use ovd_core::barge_capacity::{BargeCapacityInputs, compute as capacity};
use ovd_core::journey_derating::{JourneyDeratingInputs, compute};

fn bauxite() -> ovd_core::barge_capacity::BargeCapacityResult {
    let mut i = BargeCapacityInputs::new(100.0, 16.0, 6.0);
    i.cargo_type = "bauxite".to_string();
    capacity(&i).unwrap()
}

#[test]
fn a_shallow_channel_is_draft_bound_and_warns() {
    let cap = bauxite();
    let mut i = JourneyDeratingInputs::new(&cap, 100.0, 16.0, 6.0);
    i.design_depth_m = 2.25;
    i.ukc_m = 0.5;
    let r = compute(&i).unwrap();
    assert_eq!(r.binding_limit, "draft");
    assert_eq!(r.t_max_m, 1.75);
    // Re-pinned (#448); was 829.5246000318816.
    assert_eq!(r.achievable_tonnage_t, 1019.9246000318817);
    // Re-pinned (#448); was shed 3881 t, full-load draft 4.60 m.
    assert_eq!(
        r.warnings,
        [
            "Draft-bound: shed 3691 t of 4711 t volume capacity to float at T_max = 1.75 m (full-load draft 4.17 m)."
        ]
    );
    assert!(r.ok());
}

#[test]
fn a_deep_channel_is_volume_bound() {
    let cap = bauxite();
    let mut i = JourneyDeratingInputs::new(&cap, 100.0, 16.0, 6.0);
    i.design_depth_m = 5.5;
    i.ukc_m = 0.5;
    let r = compute(&i).unwrap();
    assert_eq!(r.binding_limit, "volume");
    assert_eq!(r.achievable_tonnage_t, r.cargo_capacity_t);
    assert_eq!(r.shed_tonnage_t, 0.0);
    assert!(r.warnings.is_empty());
}

#[test]
fn a_cap_shallower_than_the_bare_hull_floats_nothing_and_fails_the_floats_check() {
    // design depth below the UKC: t_max clamps to 0, nothing floats.
    let cap = bauxite();
    let mut i = JourneyDeratingInputs::new(&cap, 100.0, 16.0, 6.0);
    i.design_depth_m = 0.5;
    i.ukc_m = 0.5;
    let r = compute(&i).unwrap();
    assert_eq!(r.t_max_m, 0.0);
    assert_eq!(r.achievable_tonnage_t, 0.0);
    assert_eq!(r.volume_utilisation, 0.0);
    assert!(!r.ok());
}

#[test]
fn a_non_positive_principal_dimension_is_a_value_error() {
    let cap = bauxite();
    let i = JourneyDeratingInputs::new(&cap, 0.0, 16.0, 6.0);
    assert_eq!(
        compute(&i).unwrap_err(),
        "loa, beam, depth must be positive"
    );
}

#[test]
fn a_negative_ukc_is_a_value_error() {
    let cap = bauxite();
    let mut i = JourneyDeratingInputs::new(&cap, 100.0, 16.0, 6.0);
    i.ukc_m = -0.5;
    assert_eq!(
        compute(&i).unwrap_err(),
        "design_depth_m and ukc_m must be non-negative"
    );
}

#[test]
fn a_nan_dimension_passes_the_positivity_check_as_in_python() {
    // `nan <= 0` is False in Python, so a NaN loa is not refused.
    let cap = bauxite();
    let mut i = JourneyDeratingInputs::new(&cap, f64::NAN, 16.0, 6.0);
    i.design_depth_m = 2.25;
    assert!(compute(&i).is_ok());
}

// Pins for the mutation run of #17. Expected values printed by the original Python.

#[test]
fn each_principal_dimension_alone_is_refused_when_not_positive() {
    // The check is `loa <= 0 or beam <= 0 or depth <= 0`: any one is enough.
    let cap = bauxite();
    for (loa, beam, depth) in [(100.0, 0.0, 6.0), (100.0, 16.0, -1.0), (-1.0, 16.0, 6.0)] {
        let mut i = JourneyDeratingInputs::new(&cap, loa, beam, depth);
        i.design_depth_m = 2.25;
        assert_eq!(
            compute(&i).unwrap_err(),
            "loa, beam, depth must be positive"
        );
    }
}

#[test]
fn a_negative_design_depth_is_a_value_error_and_zero_is_not() {
    let cap = bauxite();
    let mut i = JourneyDeratingInputs::new(&cap, 100.0, 16.0, 6.0);
    i.design_depth_m = -0.5;
    assert_eq!(
        compute(&i).unwrap_err(),
        "design_depth_m and ukc_m must be non-negative"
    );
    // Zero design depth and zero UKC (the dataclass defaults) are allowed: nothing floats.
    i.design_depth_m = 0.0;
    let r = compute(&i).unwrap();
    assert_eq!(r.t_max_m, 0.0);
    assert_eq!(r.binding_limit, "draft");
}

#[test]
fn negative_zero_design_depth_gives_a_positive_zero_cap() {
    // Python `max(0.0, -0.0)` is the first item, `0.0`; a `>=` swap would return `-0.0`.
    let cap = bauxite();
    let mut i = JourneyDeratingInputs::new(&cap, 100.0, 16.0, 6.0);
    i.design_depth_m = -0.0;
    let r = compute(&i).unwrap();
    assert!(r.t_max_m.is_sign_positive(), "t_max_m = {}", r.t_max_m);
}

#[test]
fn water_density_scales_the_displacement_in_a_draft_bound_run() {
    // Original Python: 889.0246000318816 t at density 1.025 (829.5246000318816 t at 1.0).
    let cap = bauxite();
    let mut i = JourneyDeratingInputs::new(&cap, 100.0, 16.0, 6.0);
    i.design_depth_m = 2.25;
    i.ukc_m = 0.5;
    i.water_density = 1.025;
    let r = compute(&i).unwrap();
    // Re-pinned (#448); was 889.0246000318816.
    assert_eq!(r.achievable_tonnage_t, 1084.1846000318815);
}
