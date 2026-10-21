//! Seam: the public `ovcd_core::protective_location` API. Every expected number is what CPython
//! 3.12 printed for the `core/protective_location.py`. These pin the branch edges and the
//! Python behaviours the port must copy.
//! Label: snapshot
//! The d(Vc) tests are the exception: each is marked `sourced` in its own doc comment, with its IGC Code citation.

use ovcd_core::protective_location::*;

// The d(Vc) tests below are labelled `sourced - IGC Code 2.4.1.1 (MSC.370(93))`: the expected
// numbers are worked by hand from the text of the rule, not printed by the code. Vc is the gross
// design volume of ONE cargo tank (domes and appendages included):
//   Vc <= 1,000 m3:            d = 0.8 m
//   1,000 < Vc < 5,000 m3:     d = 0.75 + Vc x 0.2 / 4,000 m
//   5,000 <= Vc < 30,000 m3:   d = 0.8 + Vc / 25,000 m
//   Vc >= 30,000 m3:           d = 2 m
// The three joins are continuous (0.8 at 1,000; 1.0 at 5,000; 2.0 at 30,000).

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-12
}

/// sourced - IGC Code 2.4.1.1 (MSC.370(93)): the 1,000 m3 boundary and either side of it.
#[test]
fn d_vc_at_1000_and_either_side() {
    assert_eq!(d_vc_exact(0.0), 0.8);
    assert_eq!(d_vc_exact(999.999), 0.8);
    assert_eq!(d_vc_exact(1000.0), 0.8);
    // 0.75 + 1,000.001 x 0.2 / 4,000 = 0.75 + 0.05000005 = 0.80000005
    assert!(close(d_vc_exact(1000.001), 0.80000005));
    // 0.75 + 3,232 x 0.2 / 4,000 = 0.75 + 0.1616 = 0.9116
    assert!(close(d_vc_exact(3232.0), 0.9116));
}

/// sourced - IGC Code 2.4.1.1 (MSC.370(93)): the 5,000 m3 boundary and either side of it.
#[test]
fn d_vc_at_5000_and_either_side() {
    // 0.75 + 4,999.999 x 0.2 / 4,000 = 0.75 + 0.24999995 = 0.99999995
    assert!(close(d_vc_exact(4999.999), 0.99999995));
    // 0.8 + 5,000 / 25,000 = 1.0
    assert!(close(d_vc_exact(5000.0), 1.0));
    // 0.8 + 5,000.001 / 25,000 = 0.8 + 0.20000004 = 1.00000004
    assert!(close(d_vc_exact(5000.001), 1.00000004));
    // 0.8 + 12,345.6 / 25,000 = 0.8 + 0.493824 = 1.293824
    assert!(close(d_vc_exact(12345.6), 1.293824));
}

/// sourced - IGC Code 2.4.1.1 (MSC.370(93)): the 30,000 m3 boundary and either side of it.
#[test]
fn d_vc_at_30000_and_either_side() {
    // 0.8 + 29,999.9 / 25,000 = 0.8 + 1.199996 = 1.999996
    assert!(close(d_vc_exact(29999.9), 1.999996));
    // At 30,000 the rule gives 2 m (and 0.8 + 30,000 / 25,000 = 2.0 too).
    assert_eq!(d_vc_exact(30000.0), 2.0);
    assert_eq!(d_vc_exact(30000.1), 2.0);
    assert_eq!(d_vc_exact(100000.0), 2.0);
}

/// sourced - IGC Code 2.4.2: d is measured plate to plate, excluding insulation, so the clear gap
/// needed from the outer face of the insulation is d less the insulation. For a 2,000 m3 tank with
/// 350 mm of insulation: d = 0.75 + 2,000 x 0.2 / 4,000 = 0.85 m, so 0.85 - 0.35 = 0.50 m.
#[test]
fn the_clear_gap_is_d_less_the_insulation() {
    assert!(close(d_vc_exact(2000.0) - 350.0 / 1000.0, 0.5));
}

#[test]
fn vessel_type_round_trips_and_rejects_unknown_text_with_pythons_message() {
    for t in ["IGF", "IGC_1G", "IGC_2G", "IGC_3G"] {
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
    assert_eq!(bottom(Igc2g, 3.0, 2.0), 2.0);
    assert_eq!(required_bottom_protection(14.8, 1.0, None), 1.0);
    assert_eq!(required_bottom_protection(60.0, 0.8, None), 2.0);
    // beam/15 == 1.0 and d_vc == 1.0: either way 1.0; beam/15 > 2 caps at 2.0.
    assert_eq!(required_bottom_protection(15.0, 2.0, None), 2.0);
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
            // sourced - IGC Code 2.4.1.1: d = 0.75 + 2,000 x 0.2 / 4,000 = 0.85 m; the bottom rule
            // is the larger of min(beam / 15, 2) = 0.98666.. and d.
            d_vc: 0.85,
            total_tank_volume: 2000.0,
            cs_required: 2.96,
            cs_actual: 1.0,
            cs_ok: false,
            cb_required: 14.8 / 15.0,
            cb_actual: 1.0,
            cb_ok: true,
        }
    );
}

#[test]
fn check_passes_exactly_at_the_limit_and_fails_just_below() {
    // cs_actual == cs_required passes (>=), as does cb_actual == cb_required.
    let r = check_protective_location("IGC_3G", 14.8, 30000.0, 0.8, 1.0, 1.0).unwrap();
    assert!(r.cs_ok && r.cb_ok);
    assert_eq!((r.d_vc, r.cb_required, r.cb_actual), (2.0, 2.0, 2.0));
    let r = check_protective_location("IGC_3G", 14.8, 30000.0, 0.7999, 1.0, 0.9999).unwrap();
    assert!(!r.cs_ok && !r.cb_ok);
}

#[test]
fn igc_2g_exact_is_no_longer_a_vessel_type() {
    // Issue #237: the one rule set is IGC_2G; the file layer reads IGC_2G_EXACT as IGC_2G.
    assert_eq!(
        VesselType::from_value("IGC_2G_EXACT"),
        Err("'IGC_2G_EXACT' is not a valid VesselType".to_string())
    );
}

#[test]
fn adn_g_is_no_longer_a_vessel_type() {
    // Issue #175: the engine applies IGC rules only.
    assert_eq!(
        VesselType::from_value("ADN_G"),
        Err("'ADN_G' is not a valid VesselType".to_string())
    );
    assert_eq!(
        check_protective_location("ADN_G", 20.0, 300.0, 1.25, 0.6, 0.5),
        Err("'ADN_G' is not a valid VesselType".to_string())
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

#[test]
fn enum_error_text_escapes_what_python_calls_unprintable() {
    // Expected strings are CPython 3.12 `repr()` output.
    assert_eq!(
        VesselType::from_value("\u{a0}"),
        Err("'\\xa0' is not a valid VesselType".to_string())
    );
    assert_eq!(
        VesselType::from_value("\u{85}\u{ad}"),
        Err("'\\x85\\xad' is not a valid VesselType".to_string())
    );
    assert_eq!(
        VesselType::from_value("\u{2028}\u{200b}\u{e000}"),
        Err("'\\u2028\\u200b\\ue000' is not a valid VesselType".to_string())
    );
    assert_eq!(
        VesselType::from_value("\u{e0001}\u{1F600}\u{e9}"),
        Err("'\\U000e0001\u{1F600}\u{e9}' is not a valid VesselType".to_string())
    );
}
