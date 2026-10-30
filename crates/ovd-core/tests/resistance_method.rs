//! Seam: the resistance method choice of `ovd_core::resistance` (issue #723).
//! Label: property

use ovd_core::resistance::{
    AppliedMethod, EnvironmentInputs, ResistanceInputs, ResistanceMethod, compute,
};

fn deep_water(method: ResistanceMethod, cb: f64) -> ResistanceInputs {
    ResistanceInputs {
        block_coefficient: cb,
        method,
        environments: vec![EnvironmentInputs {
            name: "Deep".to_string(),
            is_deep_water: true,
            ..EnvironmentInputs::default()
        }],
        ..ResistanceInputs::default()
    }
}

fn has_method_check(r: &ovd_core::resistance::ResistanceResults) -> bool {
    r.checks
        .iter()
        .any(|c| c.name.starts_with("resistance_method_in_range["))
}

#[test]
fn the_default_is_the_barge_fit_and_adds_no_check() {
    assert_eq!(
        ResistanceInputs::default().method,
        ResistanceMethod::BargeFit
    );
    let r = compute(&deep_water(ResistanceMethod::BargeFit, 0.85)).unwrap();
    assert_eq!(r.method_used, AppliedMethod::BargeFit);
    assert!(!has_method_check(&r));
}

#[test]
fn by_block_coefficient_keeps_the_barge_fit_above_cp_0_85() {
    let fit = compute(&deep_water(ResistanceMethod::BargeFit, 0.95)).unwrap();
    let auto = compute(&deep_water(ResistanceMethod::ByBlockCoefficient, 0.95)).unwrap();
    assert_eq!(auto.method_used, AppliedMethod::BargeFit);
    assert_eq!(fit.environments, auto.environments);
    // The choice is reported, with the method that applied.
    assert!(has_method_check(&auto));
    let c = auto
        .checks
        .iter()
        .find(|c| c.name.starts_with("resistance_method_in_range["))
        .unwrap();
    assert!(c.message.starts_with("bluff-barge fit"), "{}", c.message);
    assert_eq!(c.severity, "warning");
}

#[test]
fn by_block_coefficient_takes_holtrop_mennen_at_cp_0_85_and_below() {
    let fit = compute(&deep_water(ResistanceMethod::BargeFit, 0.78)).unwrap();
    let auto = compute(&deep_water(ResistanceMethod::ByBlockCoefficient, 0.78)).unwrap();
    assert_eq!(auto.method_used, AppliedMethod::Holtrop1984);
    assert_ne!(
        fit.environments[0].speed_curve,
        auto.environments[0].speed_curve
    );
    let forced = compute(&deep_water(ResistanceMethod::HoltropMennen, 0.78)).unwrap();
    assert_eq!(auto.environments, forced.environments);
    // The default barge (125 x 24 x 6 m: L/B 5.2, B/T 4.0) is outside the B/T range, so the check
    // warns and names it.
    let c = auto
        .checks
        .iter()
        .find(|c| c.name.starts_with("resistance_method_in_range["))
        .unwrap();
    assert!(!c.ok);
    assert!(
        c.message.contains("B/T 4.00 (range 2.40 to 3.20)"),
        "{}",
        c.message
    );
}

#[test]
fn holtrop_mennen_on_a_box_says_it_has_no_value() {
    let e = compute(&deep_water(ResistanceMethod::HoltropMennen, 1.0)).unwrap_err();
    assert!(
        e.message().contains("by_block_coefficient"),
        "{}",
        e.message()
    );
}

#[test]
fn under_holtrop_mennen_a_finer_hull_needs_less_power() {
    let mut last = f64::INFINITY;
    for cb in [0.84, 0.80, 0.76] {
        let r = compute(&deep_water(ResistanceMethod::HoltropMennen, cb)).unwrap();
        let p = r.environments[0]
            .speed_curve
            .iter()
            .find(|p| p.speed_knots == 10.0)
            .unwrap()
            .delivered_power_kw;
        assert!(p < last, "Cb {cb}: {p} kW, not below {last} kW");
        last = p;
    }
}

#[test]
fn the_method_names_round_trip() {
    for m in [
        ResistanceMethod::BargeFit,
        ResistanceMethod::HoltropMennen,
        ResistanceMethod::ByBlockCoefficient,
        ResistanceMethod::BargeFormFactor,
    ] {
        assert_eq!(ResistanceMethod::parse(m.as_str()), Some(m));
    }
    assert_eq!(ResistanceMethod::parse("holtrop"), None);
}

fn power_at_10_kn(r: &ovd_core::resistance::ResistanceResults) -> f64 {
    r.environments[0]
        .speed_curve
        .iter()
        .find(|p| p.speed_knots == 10.0)
        .unwrap()
        .delivered_power_kw
}

#[test]
fn barge_form_factor_ranks_hulls_and_reports_its_range() {
    // Issue #735: a finer hull needs less power, at every fullness the barge fit covers.
    let mut last = f64::INFINITY;
    for cb in [0.97, 0.92, 0.85, 0.80] {
        let r = compute(&deep_water(ResistanceMethod::BargeFormFactor, cb)).unwrap();
        assert_eq!(r.method_used, AppliedMethod::BargeFormFactor);
        let p = power_at_10_kn(&r);
        assert!(p < last, "Cb {cb}: {p} kW, not below {last} kW");
        last = p;
        // More resistance than the bluff-barge fit: its form factor is above 1.10.
        let fit = compute(&deep_water(ResistanceMethod::BargeFit, cb)).unwrap();
        assert!(p > power_at_10_kn(&fit), "Cb {cb}");
    }
    // The default barge (L/B 5.2) is outside the range of the barge data (L/B from 7.59).
    let r = compute(&deep_water(ResistanceMethod::BargeFormFactor, 0.85)).unwrap();
    let c = r
        .checks
        .iter()
        .find(|c| c.name.starts_with("resistance_method_in_range["))
        .unwrap();
    assert!(!c.ok);
    assert!(c.message.starts_with("barge form factor"), "{}", c.message);
    assert!(
        c.message.contains("L/B 5.21 (range 7.59 to 16.29)"),
        "{}",
        c.message
    );
}

#[test]
fn barge_form_factor_adds_the_millward_term_in_shallow_water() {
    use ovd_core::resistance::millward_form_factor_increase;
    assert_eq!(millward_form_factor_increase(6.0, 0.0), 0.0);
    // dk = 0.644 (T/h)^1.72; at h/T = 1 it is the constant.
    assert!((millward_form_factor_increase(6.0, 6.0) - 0.644).abs() < 1e-12);
    let shallow = |method| ResistanceInputs {
        block_coefficient: 0.85,
        method,
        environments: vec![EnvironmentInputs {
            name: "Shallow".to_string(),
            water_depth: 9.0,
            channel_width_bottom: 1000.0,
            channel_width_surface: 1000.0,
            apply_squat_limit: false,
            ..EnvironmentInputs::default()
        }],
        ..ResistanceInputs::default()
    };
    let ratio = |method| {
        let deep = power_at_10_kn(&compute(&deep_water(method, 0.85)).unwrap());
        power_at_10_kn(&compute(&shallow(method)).unwrap()) / deep
    };
    assert!(ratio(ResistanceMethod::BargeFormFactor) > ratio(ResistanceMethod::BargeFit));
}
