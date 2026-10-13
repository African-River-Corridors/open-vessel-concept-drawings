//! Seam: the public `ovcd_core::protective_location` API. Every expected number is what CPython
//! 3.12 printed for the frozen `core/protective_location.py`. The parity cases cover the wide
//! sweep; these pin the branch edges and the Python behaviours the port must copy.

use ovcd_core::protective_location::*;

#[test]
fn d_vc_steps_at_1000_and_5000_inclusive_below() {
    assert_eq!(distance_d_vc(0.0), 0.8);
    assert_eq!(distance_d_vc(1000.0), 0.8);
    assert_eq!(distance_d_vc(1000.0001), 1.0);
    assert_eq!(distance_d_vc(5000.0), 1.0);
    assert_eq!(distance_d_vc(5000.1), 2.0);
}

#[test]
fn d_vc_of_nan_is_the_last_branch_as_in_python() {
    assert_eq!(distance_d_vc(f64::NAN), 2.0);
}

#[test]
fn d_vc_exact_is_linear_in_bands() {
    assert_eq!(d_vc_exact(0.0), 0.8);
    assert_eq!(d_vc_exact(1000.0), 0.8);
    assert_eq!(d_vc_exact(1000.5), 0.800025);
    assert_eq!(d_vc_exact(2500.0), 0.875);
    assert_eq!(d_vc_exact(4999.999), 0.9999999500000001);
    assert_eq!(d_vc_exact(5000.0), 1.0);
    assert_eq!(d_vc_exact(12345.6), 1.293824);
    assert_eq!(d_vc_exact(29999.9), 1.999996);
    assert_eq!(d_vc_exact(30000.0), 2.0);
}

#[test]
fn vessel_type_round_trips_and_rejects_unknown_text_with_pythons_message() {
    for t in ["IGF", "IGC_1G", "IGC_2G", "IGC_3G", "ADN_G", "IGC_2G_EXACT"] {
        assert_eq!(VesselType::from_value(t).unwrap().value(), t);
    }
    assert_eq!(
        VesselType::from_value("igc_2g"),
        Err("'igc_2g' is not a valid VesselType".to_string())
    );
    // Python's repr switches to double quotes for text holding a single quote.
    assert_eq!(
        VesselType::from_value("it's"),
        Err("\"it's\" is not a valid VesselType".to_string())
    );
    assert_eq!(
        VesselType::from_value("a\\b\nc\u{1}"),
        Err("'a\\\\b\\nc\\x01' is not a valid VesselType".to_string())
    );
}

fn side(t: VesselType, beam: f64, d_vc: f64) -> f64 {
    required_side_protection(Some(t), beam, d_vc).unwrap()
}

fn bottom(t: VesselType, beam: f64, d_vc: f64) -> f64 {
    required_bottom_protection(beam, d_vc, Some(t))
}

#[test]
fn side_protection_per_vessel_type() {
    use VesselType::*;
    // (type, beam, d_vc, expected) printed by CPython.
    assert_eq!(side(Igf, 14.8, 1.0), 2.96);
    assert_eq!(side(Igf, 80.0, 0.8), 11.5);
    assert_eq!(side(Igf, 3.0, 2.0), 2.0);
    assert_eq!(side(Igc1g, 18.0, 1.0), 3.6);
    assert_eq!(side(Igc1g, 300.0, 0.5), 11.5);
    assert_eq!(side(Igc2g, 14.8, 1.0), 1.0);
    assert_eq!(side(Igc2g, 300.0, 0.5), 0.5);
    assert_eq!(side(Igc3g, 14.8, 1.0), 0.8);
    assert_eq!(side(Igc3g, 3.0, 2.0), 0.8);
    assert_eq!(side(AdnG, 14.8, 1.0), 1.25);
    assert_eq!(side(AdnG, 300.0, 0.5), 1.25);
    assert_eq!(side(Igc2gExact, 80.0, 0.8), 0.8);
    assert_eq!(side(Igc2gExact, 3.0, 2.0), 2.0);
}

#[test]
fn side_protection_of_no_type_fails_like_python() {
    assert_eq!(
        required_side_protection(None, 1.0, 1.0),
        Err("Unknown vessel type: None".to_string())
    );
}

#[test]
fn bottom_protection_is_beam_over_15_capped_at_2_but_not_below_d_vc() {
    use VesselType::*;
    assert_eq!(bottom(Igf, 14.8, 1.0), 1.0);
    assert_eq!(bottom(Igc2g, 18.0, 1.0), 1.2);
    assert_eq!(bottom(Igc3g, 80.0, 0.8), 2.0);
    assert_eq!(bottom(Igc2gExact, 3.0, 2.0), 2.0);
    assert_eq!(required_bottom_protection(14.8, 1.0, None), 1.0);
    assert_eq!(required_bottom_protection(60.0, 0.8, None), 2.0);
    // beam/15 == 1.0 and d_vc == 1.0: either way 1.0; beam/15 > 2 caps at 2.0.
    assert_eq!(required_bottom_protection(15.0, 2.0, None), 2.0);
}

#[test]
fn adn_g_bottom_is_constant_and_ignores_beam_and_d_vc() {
    assert_eq!(bottom(VesselType::AdnG, 14.8, 1.0), 1.1);
    assert_eq!(bottom(VesselType::AdnG, 300.0, 0.5), 1.1);
    assert_eq!(bottom(VesselType::AdnG, 3.0, 2.0), 1.1);
}

#[test]
fn max_and_min_follow_python_with_nan() {
    // Python: min(nan / 5, 11.5) is nan (first item kept: 11.5 < nan is False), then
    // max(nan, 1.0) is nan (1.0 > nan is False). Rust's f64::max would give 1.0.
    let c = required_side_protection(Some(VesselType::Igf), f64::NAN, 1.0).unwrap();
    assert!(c.is_nan());
    // A NaN d_vc loses to the first item: max(2.96, nan) is 2.96.
    assert_eq!(
        required_side_protection(Some(VesselType::Igf), 14.8, f64::NAN).unwrap(),
        2.96
    );
    assert_eq!(
        required_bottom_protection(14.8, f64::NAN, None),
        14.8 / 15.0
    );
}

#[test]
fn compute_required_clearances_echoes_d_vc() {
    let r = compute_required_clearances("IGF", 14.8, 1.0).unwrap();
    assert_eq!(
        r,
        RequiredClearances {
            cs_required: 2.96,
            cb_required: 1.0,
            d_vc: 1.0
        }
    );
    assert_eq!(
        compute_required_clearances("IGC_9G", 14.8, 1.0),
        Err("'IGC_9G' is not a valid VesselType".to_string())
    );
}

#[test]
fn check_fails_side_and_passes_bottom() {
    let r = check_protective_location("IGC_1G", 14.8, 2000.0, 1.0, 0.5, 0.5).unwrap();
    assert_eq!(
        r,
        ProtectiveLocationResult {
            vessel_type: "IGC_1G".into(),
            d_vc: 1.0,
            total_tank_volume: 2000.0,
            cs_required: 2.96,
            cs_actual: 1.0,
            cs_ok: false,
            cb_required: 1.0,
            cb_actual: 1.0,
            cb_ok: true,
        }
    );
}

#[test]
fn check_passes_exactly_at_the_limit_and_fails_just_below() {
    // cs_actual == cs_required passes (>=), as does cb_actual == cb_required.
    let r = check_protective_location("IGC_3G", 14.8, 6000.0, 0.8, 1.0, 1.0).unwrap();
    assert!(r.cs_ok && r.cb_ok);
    assert_eq!((r.d_vc, r.cb_required, r.cb_actual), (2.0, 2.0, 2.0));
    let r = check_protective_location("IGC_3G", 14.8, 6000.0, 0.7999, 1.0, 0.9999).unwrap();
    assert!(!r.cs_ok && !r.cb_ok);
    let r = check_protective_location("ADN_G", 20.0, 300.0, 1.25, 0.6, 0.5).unwrap();
    assert!(r.cs_ok && r.cb_ok);
    assert_eq!(
        (r.cs_required, r.cb_required, r.cb_actual),
        (1.25, 1.1, 1.1)
    );
}

#[test]
fn check_rejects_an_unknown_vessel_type_before_anything_else() {
    assert_eq!(
        check_protective_location("igc_2g", 14.8, 2000.0, 1.0, 0.5, 0.5),
        Err("'igc_2g' is not a valid VesselType".to_string())
    );
}

#[test]
fn unknown_type_message_uses_pythons_repr_for_spaces_and_both_quotes() {
    // CPython: 'a b' keeps its space; a text with both quote kinds is single-quoted and the
    // single quote is escaped; DEL is shown as \x7f; a tab is shown as \t.
    let msg = |t: &str| VesselType::from_value(t).unwrap_err();
    assert_eq!(msg("a b"), "'a b' is not a valid VesselType");
    assert_eq!(msg("a'b\"c"), "'a\\'b\"c' is not a valid VesselType");
    assert_eq!(msg("a\u{7f}b"), "'a\\x7fb' is not a valid VesselType");
    assert_eq!(msg("tab\there"), "'tab\\there' is not a valid VesselType");
}

#[test]
fn max_keeps_the_first_of_equal_items_including_signed_zero() {
    // CPython: max(min(-0.0 / 5, 11.5), 0.0) is -0.0 (the first of the equal items), and so is
    // the bottom rule. A port that returns the second item would give +0.0.
    let s = required_side_protection(Some(VesselType::Igf), -0.0, 0.0).unwrap();
    assert!(s == 0.0 && s.is_sign_negative());
    let b = required_bottom_protection(-0.0, 0.0, Some(VesselType::Igf));
    assert!(b == 0.0 && b.is_sign_negative());
}
