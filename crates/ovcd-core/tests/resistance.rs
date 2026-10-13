//! Seam: the public `ovcd_core::resistance` API. Expected values are what the frozen Python
//! printed for the same inputs (see `parity/reference/resistance.json`); the full case set runs
//! in `ovcd-format/tests/parity.rs`.

use ovcd_core::aft_arrangement::PyException;
use ovcd_core::pyfmt::round_to;
use ovcd_core::resistance::{ResistanceInputs, TugInputs, compute};

fn close(a: f64, b: f64) {
    assert!((a - b).abs() <= 1e-9 * b.abs().max(1.0), "{a} vs {b}");
}

#[test]
fn the_default_barge_has_three_environments_and_the_river_binds() {
    let r = compute(&ResistanceInputs::default()).unwrap();
    close(r.wetted_surface_area, 4590.0);
    assert_eq!(r.environments.len(), 3);
    let names: Vec<&str> = r.environments.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(
        names,
        ["Open Sea", "Confined shallow river", "Confined creek"]
    );
    let binding = r.binding_environment().unwrap();
    assert_eq!(binding.name, "Confined shallow river");
    close(r.binding_power_at_service_kw(), 66.18578807734983);
    assert!(r.ok());
}

#[test]
fn service_speed_and_what_governs_it() {
    let r = compute(&ResistanceInputs::default()).unwrap();
    let [sea, river, creek] = &r.environments[..] else {
        panic!("three environments")
    };
    assert_eq!(sea.limiting_speed_knots, None);
    assert_eq!(sea.service_speed_knots, 12.0);
    assert_eq!(sea.service_speed_governed_by, "power");
    assert_eq!(sea.squat_limited_speed_knots, None);
    assert_eq!(sea.speed_curve.len(), 21);
    close(river.limiting_speed_knots.unwrap(), 10.827543883575197);
    assert_eq!(river.service_speed_knots, 2.5);
    assert_eq!(river.service_speed_governed_by, "squat");
    close(river.squat_limited_speed_knots.unwrap(), 3.779296875);
    assert_eq!(river.speed_curve.len(), 17);
    assert_eq!(creek.service_speed_knots, 10.0);
    close(creek.blockage_ratio, 0.19124999999999998);
}

#[test]
fn one_speed_point_in_the_river() {
    let r = compute(&ResistanceInputs::default()).unwrap();
    let p = &r.environments[1].speed_curve[3];
    assert_eq!(p.speed_knots, 3.5);
    assert_eq!(p.speed_through_water_knots, 4.5);
    close(p.froude_number, 0.06610342521903137);
    close(p.resistance_shallow_factor, 1.1194701507773603);
    close(p.resistance_total_kn, 27.688735115992472);
    close(p.delivered_power_kw, 142.43085343666527);
    close(p.squat_m, 0.5020177291974529);
    assert!(p.squat_limited);
    assert!(p.power_available);
}

#[test]
fn the_check_texts_match_python() {
    let r = compute(&ResistanceInputs::default()).unwrap();
    let texts: Vec<(&str, bool, &str)> = r
        .checks
        .iter()
        .map(|c| (c.name.as_str(), c.ok, c.message.as_str()))
        .collect();
    assert_eq!(texts.len(), 8);
    assert_eq!(
        texts[0],
        (
            "power_within_installed[Open Sea]",
            true,
            "Open Sea: power 2291 kW <= installed 2400 kW"
        )
    );
    assert_eq!(
        texts[1].2,
        "Confined shallow river: service 2.5 kn <= 0.95 x Schijf limit 10.83 kn"
    );
    assert_eq!(
        texts[3].2,
        "Confined shallow river: blockage n = 0.377 must be in (0, 1)"
    );
    assert_eq!(texts[7].0, "coefficients_are_first_order");
    assert_eq!(r.checks[7].severity, "warning");
}

#[test]
fn a_nonpositive_waterline_length_raises_as_python_does() {
    let zero = ResistanceInputs {
        lwl: 0.0,
        ..ResistanceInputs::default()
    };
    assert_eq!(
        compute(&zero).unwrap_err(),
        PyException::ZeroDivisionError("float division by zero".into())
    );
    let negative = ResistanceInputs {
        lwl: -1.0,
        ..ResistanceInputs::default()
    };
    assert_eq!(
        compute(&negative).unwrap_err(),
        PyException::ValueError("math domain error".into())
    );
}

#[test]
fn zero_fuel_density_is_a_zero_division() {
    let inp = ResistanceInputs {
        tug: TugInputs {
            fuel_density_kg_per_litre: 0.0,
            ..TugInputs::default()
        },
        ..ResistanceInputs::default()
    };
    assert_eq!(
        compute(&inp).unwrap_err(),
        PyException::ZeroDivisionError("float division by zero".into())
    );
}

#[test]
fn a_zero_draft_in_a_shallow_channel_fails_in_squat_validation() {
    let inp = ResistanceInputs {
        draft: 0.0,
        ..ResistanceInputs::default()
    };
    assert_eq!(
        compute(&inp).unwrap_err(),
        PyException::ValueError("draft_m must be > 0".into())
    );
}

#[test]
fn python_round_is_half_even_on_the_exact_value() {
    // CPython 3.12: round(0.25, 1) == 0.2, round(0.75, 1) == 0.8, round(2.25, 1) == 2.2,
    // round(2.75, 1) == 2.8, round(0.35, 1) == 0.3 (0.35 is a hair under), round(2.675, 2) == 2.67,
    // round(0.125, 2) == 0.12, round(0.375, 2) == 0.38.
    assert_eq!(round_to(0.25, 1), 0.2);
    assert_eq!(round_to(0.75, 1), 0.8);
    assert_eq!(round_to(2.25, 1), 2.2);
    assert_eq!(round_to(2.75, 1), 2.8);
    assert_eq!(round_to(0.35, 1), 0.3);
    assert_eq!(round_to(2.675, 2), 2.67);
    assert_eq!(round_to(0.125, 2), 0.12);
    assert_eq!(round_to(0.375, 2), 0.38);
    assert!(round_to(f64::NAN, 1).is_nan());
    assert_eq!(round_to(f64::INFINITY, 1), f64::INFINITY);
    assert!(round_to(-0.04, 1).is_sign_negative());
}

#[test]
fn the_speed_grid_is_repeated_addition_then_rounding() {
    // Step 0.25 from 2.0: the rounded grid repeats Python's 2.2, 2.8, 3.2 (not 2.25, 2.75, 3.25).
    let inp = ResistanceInputs {
        speed_step_knots: 0.25,
        speed_max_knots: 3.5,
        ..ResistanceInputs::default()
    };
    let r = compute(&inp).unwrap();
    let speeds: Vec<f64> = r.environments[0]
        .speed_curve
        .iter()
        .map(|p| p.speed_knots)
        .collect();
    assert_eq!(speeds, [2.0, 2.2, 2.5, 2.8, 3.0, 3.2, 3.5]);
}
