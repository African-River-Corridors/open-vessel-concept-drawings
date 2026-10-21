//! Seam: the public `ovcd_core::squat` API. Expected values are what the original Python printed
//! for the same inputs.
//! Label: snapshot

use ovcd_core::squat::{DEFAULT_SAFETY_MARGIN_M, SquatInputs, compute, max_speed_for_ukc};

fn cn_ii(speed: f64) -> SquatInputs {
    SquatInputs {
        water_depth_m: 4.8,
        channel_area_m2: 209.5,
        ..SquatInputs::new(speed, 4.4, 14.8, 87.3)
    }
}

#[test]
fn a_stationary_vessel_does_not_squat() {
    let r = compute(&cn_ii(0.0)).unwrap();
    assert_eq!(r.method, "Barrass II");
    assert_eq!(r.squat_m, 0.0);
    assert!(!r.grounds);
    assert!(r.ok());
    assert_eq!(
        r.checks[0].message,
        "dynamic UKC 0.40 m = static 0.40 m - squat 0.00 m (Barrass II) at 0.00 kn"
    );
}

#[test]
fn the_laden_motor_grounds_at_six_knots_in_a_4_8_m_channel() {
    let r = compute(&cn_ii(6.0)).unwrap();
    assert_eq!(r.method, "Barrass II");
    assert!((r.squat_m - 0.68531955807082).abs() < 1e-12);
    assert!((r.squat_icorels_m.unwrap() - 0.34486994034316343).abs() < 1e-12);
    assert!(r.grounds);
    assert!(!r.ok());
    assert_eq!(
        r.checks[0].message,
        "dynamic UKC -0.29 m = static 0.40 m - squat 0.69 m (Barrass II) at 6.00 kn"
    );
    assert_eq!(
        r.checks[4].message,
        "Barrass 0.69 m vs ICORELS 0.34 m, 50% apart"
    );
}

#[test]
fn open_water_squat_is_icorels_only_and_says_so() {
    let inp = SquatInputs {
        channel_area_m2: 0.0,
        ..cn_ii(8.0)
    };
    let r = compute(&inp).unwrap();
    assert_eq!(r.method, "ICORELS");
    assert_eq!(r.squat_barrass_m, 0.0);
    assert_eq!(r.warnings.len(), 1);
    assert!(r.warnings[0].starts_with("no channel area given, so blockage is 0"));
    assert!(r.warnings[0].contains(" \u{2014} this is the OPEN-WATER case"));
}

#[test]
fn no_water_depth_gives_an_infinite_static_ukc_printed_as_python_does() {
    let r = compute(&SquatInputs::new(5.0, 4.4, 14.8, 87.3)).unwrap();
    assert!(r.static_ukc_m.is_infinite());
    assert_eq!(r.squat_icorels_m, None);
    assert!(r.checks[0].message.contains("static inf m"));
}

#[test]
fn bad_inputs_fail_with_the_python_messages_in_the_python_order() {
    let msg = |inp: SquatInputs| compute(&inp).unwrap_err();
    assert_eq!(
        msg(SquatInputs::new(-1.0, 4.4, 14.8, 87.3)),
        "speed_through_water_kn must be >= 0"
    );
    assert_eq!(
        msg(SquatInputs::new(6.0, 0.0, 14.8, 87.3)),
        "draft_m must be > 0"
    );
    assert_eq!(
        msg(SquatInputs::new(6.0, 4.4, 0.0, 0.0)),
        "beam_m must be > 0"
    );
    assert_eq!(
        msg(SquatInputs::new(6.0, 4.4, 14.8, -2.0)),
        "lpp_m must be > 0"
    );
    let cb = SquatInputs {
        block_coefficient: 0.2,
        ..SquatInputs::new(6.0, 4.4, 14.8, 87.3)
    };
    assert_eq!(msg(cb), "block_coefficient outside 0.3-1.0");
}

#[test]
fn bisection_stops_early_on_tolerance_like_python() {
    // Python stops at 6.46484375, not at the true root.
    let v = max_speed_for_ukc(
        4.4,
        14.8,
        87.3,
        5.28,
        235.5,
        0.85,
        DEFAULT_SAFETY_MARGIN_M,
        20.0,
        0.01,
    );
    assert_eq!(v, Ok(6.46484375));
}

#[test]
fn a_vessel_that_cannot_float_clear_at_rest_has_speed_zero() {
    // static 0.40 m is above the 0.15 m margin, so use a tighter channel.
    let v = max_speed_for_ukc(4.4, 14.8, 87.3, 4.5, 209.5, 0.85, 0.15, 20.0, 0.01);
    assert_eq!(v, Ok(0.0));
}

#[test]
fn the_static_check_runs_before_input_validation() {
    // Python returns 0.0 here before it ever builds SquatInputs; a zero beam only fails later.
    assert_eq!(
        max_speed_for_ukc(4.4, 0.0, 87.3, 4.5, 209.5, 0.85, 0.15, 20.0, 0.01),
        Ok(0.0)
    );
    assert_eq!(
        max_speed_for_ukc(4.4, 0.0, 87.3, 5.28, 235.5, 0.85, 0.15, 20.0, 0.01),
        Err("beam_m must be > 0".to_string())
    );
}

#[test]
fn a_clear_vessel_at_the_top_speed_returns_the_top_speed() {
    let v = max_speed_for_ukc(1.0, 5.0, 50.0, 40.0, 5000.0, 0.85, 0.15, 20.0, 0.01);
    assert_eq!(v, Ok(20.0));
}

#[test]
fn the_method_spread_divides_by_at_least_one_nanometre_like_python() {
    // Barrass is 7.9e-11 m here, below Python's 1e-9 floor, so the spread is 0.66 -> "1% apart"
    // (dividing by the Barrass value itself would print 8%).
    let r = compute(&cn_ii(0.0001)).unwrap();
    assert_eq!(r.method, "ICORELS");
    assert_eq!(
        r.checks.last().unwrap().message,
        "Barrass 0.00 m vs ICORELS 0.00 m, 1% apart"
    );
}
