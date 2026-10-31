//! Issue #740: a worked example for each scantling Rule of `abs-steel-barges` 1.0.0, worked by hand
//! from the printed formula of the paragraph it names.
//!
//! Label: sourced - ABS Rules for Building and Classing Steel Barges, Part 3 January 2026 and Part 5
//! July 2025, each paragraph read in the July 2025 Rules (eagle.org, `10-barge-rules-jul25.pdf`); the
//! January 2026 Rule Change Notice changes none of them. Each example names the paragraph, the inputs
//! and the hand sum.

use ovd_core::scantling_rules::{RULES, coefficient, get, value};

fn v(id: &str, ops: &[(&str, f64)]) -> f64 {
    value(&format!("abs-steel-barges/{id}"), ops)
}

fn close(got: f64, hand: f64) {
    assert!((got - hand).abs() < 1e-9, "{got} vs {hand}");
}

#[test]
fn rule_length_3_1_1_3_1() {
    // 96 percent of the length on a waterline at 85 percent of D.
    assert_eq!(
        coefficient("abs-steel-barges/3-1-1.3.1/rule-length", "value"),
        0.96
    );
    assert_eq!(
        coefficient("abs-steel-barges/3-1-1.3.1/rule-length-waterline", "value"),
        0.85
    );
}

#[test]
fn hull_girder_modulus_3_2_1_3_1() {
    // L = 100 m: C1 = 10.75 - ((300 - 100)/100)^1.5 = 10.75 - 2.828427 = 7.921573;
    // SMb = C1 x 0.01 x 100^2 x 20 x (0.9 + 0.7) = 25,349.03 cm2-m.
    let hand = (10.75 - libm::pow(2.0, 1.5)) * 0.01 * 100.0 * 100.0 * 20.0 * 1.6;
    close(
        v(
            "3-2-1.3.1/hull-girder-modulus",
            &[("l", 100.0), ("b", 20.0), ("cb", 0.9)],
        ),
        hand,
    );
    assert!((hand - 25_349.03).abs() < 0.01);
    // L = 60 m: C1 = 16.33 x 0.36 - 15.47 x 0.6 + 7.77 = 4.3308; L = 40 m: C1 = 4.11.
    let c60 = 16.33 * 0.36 - 15.47 * 0.6 + 7.77;
    let sm60 = v(
        "3-2-1.3.1/hull-girder-modulus",
        &[("l", 60.0), ("b", 10.0), ("cb", 0.8)],
    );
    assert!((sm60 - c60 * 0.01 * 3600.0 * 10.0 * 1.5).abs() < 1e-6);
    let sm40 = v(
        "3-2-1.3.1/hull-girder-modulus",
        &[("l", 40.0), ("b", 10.0), ("cb", 0.8)],
    );
    close(sm40, 4.11 * 0.01 * 1600.0 * 10.0 * 1.5);
}

#[test]
fn side_shell_3_2_2_3_1() {
    // L = 100 m, s = 700 mm: 0.07 x 100 + 0.007 x 700 = 11.9 mm; L = 160 m: 4.8 + 4.9 + 6.0 = 15.7.
    close(
        v("3-2-2.3.1/side-shell", &[("x", 100.0), ("s", 700.0)]),
        11.9,
    );
    close(
        v("3-2-2.3.1/side-shell", &[("x", 160.0), ("s", 700.0)]),
        15.7,
    );
    // 3.1.1: less 0.5 mm with longitudinal bulkheads within 0.2B of the side.
    let id = "abs-steel-barges/3-2-2.3.1.1/side-shell-reduction";
    assert_eq!(coefficient(id, "value"), 0.5);
    assert_eq!(
        coefficient(
            "abs-steel-barges/3-2-2.3.1.1/side-shell-reduction-breadth",
            "value"
        ),
        0.2
    );
}

#[test]
fn bottom_shell_3_2_2_3_5_1() {
    // L = 100 m: 0.045 x 100 + 0.007 x 700 + 1.8 = 11.2 mm. L = 140 m: 11.2 + 4.9 - 2.5 = 13.6.
    close(
        v("3-2-2.3.5.1/bottom-shell", &[("x", 100.0), ("s", 700.0)]),
        11.2,
    );
    close(
        v("3-2-2.3.5.1/bottom-shell", &[("x", 140.0), ("s", 700.0)]),
        13.6,
    );
    // L need not be greater than 225 m: L = 250 m gives the 225 m value, 18 + 4.9 - 2.5 = 20.4.
    close(
        v("3-2-2.3.5.1/bottom-shell", &[("x", 250.0), ("s", 700.0)]),
        20.4,
    );
}

#[test]
fn shell_ends_3_2_2_5_1() {
    // L = 60 m: 0.055 x 60 + 4.9 + 1.0 = 9.2 mm; L = 100 m: 1.5 + 0.01 x 700 + 2.2 = 10.7 mm.
    close(v("3-2-2.5.1/shell-ends", &[("x", 60.0), ("s", 700.0)]), 9.2);
    close(
        v("3-2-2.5.1/shell-ends", &[("x", 100.0), ("s", 700.0)]),
        10.7,
    );
}

#[test]
fn strength_deck_3_2_3_2_1_2() {
    // (a) s = 700 mm: 0.009 x 700 + 2.4 = 8.7 mm; s = 800 mm: 0.006 x 800 + 4.7 = 9.5 mm.
    close(
        v(
            "3-2-3.2.1.2(a)/strength-deck",
            &[("x", 700.0), ("s", 700.0)],
        ),
        8.7,
    );
    close(
        v(
            "3-2-3.2.1.2(a)/strength-deck",
            &[("x", 800.0), ("s", 800.0)],
        ),
        9.5,
    );
    // (b) L = 100 m: 700 x 148.76 / (2600 + 8681) = 9.2309 mm.
    close(
        v(
            "3-2-3.2.1.2(b)/strength-deck",
            &[("l", 100.0), ("s", 700.0)],
        ),
        700.0 * 148.76 / 11281.0,
    );
}

#[test]
fn deck_ends_3_2_3_2_7() {
    // 0.01 mm per mm of frame spacing: 7.0 mm at 700 mm.
    close(v("3-2-3.2.7/deck-ends", &[("x", 100.0), ("s", 700.0)]), 7.0);
}

#[test]
fn inner_bottom_5_1_2_5_17() {
    // L = 100 m: 0.037 x 100 + 0.009 x 700 - 0.5 = 9.5 mm, less 1 mm (longitudinal framing) = 8.5.
    close(
        v("5-1-2.5.17/inner-bottom", &[("x", 100.0), ("s", 700.0)]),
        8.5,
    );
}

#[test]
fn tank_and_watertight_bulkhead_plating() {
    // 3-2-7/3.1, s = 700, k = 1 (a > 2), h = 10 m: 700 x sqrt(10) / 254 + 2.5 = 11.2151 mm.
    let t = v(
        "3-2-7.3.1/tank-bulkhead-plating",
        &[("s", 700.0), ("aspect", 5.0), ("h", 10.0)],
    );
    close(t, 700.0 * 10.0_f64.sqrt() / 254.0 + 2.5);
    // a = 1: k = (3.075 - 2.077) / 1.272 = 0.784591; at h = 0.1 the minimum 700/150 + 2.5 governs.
    let k1 = (3.075 - 2.077) / 1.272;
    let t1 = v(
        "3-2-7.3.1/tank-bulkhead-plating",
        &[("s", 700.0), ("aspect", 1.0), ("h", 10.0)],
    );
    close(
        t1,
        (700.0 * k1 * 10.0_f64.sqrt() / 254.0 + 2.5).max(700.0 / 150.0 + 2.5),
    );
    // 3-2-6/5.1, h = 9 m: 700 x 3 / 290 + 1.5 = 8.7414 mm; at no head 700/200 + 2.5 = 6.0.
    close(
        v(
            "3-2-6.5.1/watertight-bulkhead-plating",
            &[("s", 700.0), ("aspect", 5.0), ("h", 9.0)],
        ),
        2100.0 / 290.0 + 1.5,
    );
    close(
        v(
            "3-2-6.5.1/watertight-bulkhead-plating",
            &[("s", 700.0), ("aspect", 5.0), ("h", 0.0)],
        ),
        6.0,
    );
}

#[test]
fn heads_5_2_1_3_and_3_2_6_5_3() {
    // 5-2-1/3: ha1 = 1.22 m (L <= 61), 0.02L (61 < L <= 122), 2.44 m (L > 122), plus the distance
    // to the deck: z = 2 m under D = 6 m gives 4 m.
    close(
        v(
            "5-2-1.3/tank-head",
            &[("l", 50.0), ("depth", 6.0), ("z", 2.0)],
        ),
        5.22,
    );
    close(
        v(
            "5-2-1.3/tank-head",
            &[("l", 100.0), ("depth", 6.0), ("z", 2.0)],
        ),
        6.0,
    );
    close(
        v(
            "5-2-1.3/tank-head",
            &[("l", 140.0), ("depth", 6.0), ("z", 2.0)],
        ),
        6.44,
    );
    // 3-2-6/5.3: under 6.10 m, 0.8 x 4 + 1.22 = 4.42 m; at 8 m, 8 m.
    close(
        v("3-2-6.5.3/bulkhead-head", &[("depth", 6.0), ("z", 2.0)]),
        4.42,
    );
    close(
        v("3-2-6.5.3/bulkhead-head", &[("depth", 10.0), ("z", 2.0)]),
        8.0,
    );
}

#[test]
fn web_minimum_3_2_4_3_11() {
    // In tanks, L = 140 m, a 300 mm web: max(0.01 x 300 + 3, 140/36 + 7.0) = 10.8889 mm.
    close(
        v(
            "3-2-4.3.11.1/web-in-tanks",
            &[("l", 140.0), ("depth_mm", 300.0)],
        ),
        140.0 / 36.0 + 7.0,
    );
    // Clear of tanks, L = 60 m: 60/60 + 6.35 = 7.35 mm; the depth term caps at 11.5 mm.
    close(
        v(
            "3-2-4.3.11.2/web-clear-of-tanks",
            &[("l", 60.0), ("depth_mm", 300.0)],
        ),
        7.35,
    );
    close(
        v(
            "3-2-4.3.11.2/web-clear-of-tanks",
            &[("l", 60.0), ("depth_mm", 2000.0)],
        ),
        11.5,
    );
    // L1 is not taken less than 29 m: L = 20 m in tanks gives 29/20 + 5.33 = 6.78 mm.
    close(
        v(
            "3-2-4.3.11.1/web-in-tanks",
            &[("l", 20.0), ("depth_mm", 100.0)],
        ),
        29.0 / 20.0 + 5.33,
    );
}

#[test]
fn double_bottom_girders_and_floors_5_1_2() {
    // L = 100 m: centre girder 0.056 x 100 + 5.5 = 11.1 mm (85 % at the ends), side girders
    // 0.036 x 100 + 4.5 = 8.1 mm at most 4.6 m apart, floors 0.036 x 100 + 6.0 = 9.6 mm.
    close(
        v("5-1-2.5.1.1/center-girder", &[("x", 100.0), ("s", 0.0)]),
        11.1,
    );
    assert_eq!(
        coefficient("abs-steel-barges/5-1-2.5.1.2/center-girder-ends", "value"),
        0.85
    );
    close(
        v("5-1-2.5.5/side-girders", &[("x", 100.0), ("s", 0.0)]),
        8.1,
    );
    assert_eq!(
        coefficient("abs-steel-barges/5-1-2.5.5/side-girder-spacing", "value"),
        4.6
    );
    close(
        v("5-1-2.5.7.1/solid-floors", &[("x", 100.0), ("s", 0.0)]),
        9.6,
    );
}

#[test]
fn deck_beams_5_1_1_3_1() {
    // h = 0.02L + 0.76 below L = 107 m (2.76 m at 100 m), 2.9 m from 107 m.
    close(
        v("5-1-1.3.1/deck-beam-head", &[("x", 100.0), ("s", 0.0)]),
        2.76,
    );
    close(
        v("5-1-1.3.1/deck-beam-head", &[("x", 107.0), ("s", 0.0)]),
        2.9,
    );
    // SM = 7.8 c h s l^2, c = 1/(1.709 - 0.651 k), k = 1: h = 2.9, s = 0.7, l = 3.66.
    let c = 1.0 / (1.709 - 0.651);
    close(
        v(
            "5-1-1.3.1/deck-longitudinals",
            &[("h", 2.9), ("s", 0.7), ("l", 3.66)],
        ),
        7.8 * c * 2.9 * 0.7 * 3.66 * 3.66,
    );
}

#[test]
fn section_moduli_of_stiffeners_and_primary_members() {
    // SM = 7.8 c h s l^2 (stiffeners) and 4.74 c h s l^2 (primary members), h = 5, s = 0.7, l = 3.
    let ops = [("h", 5.0), ("s", 0.7), ("l", 3.0)];
    let st = |c: f64| 7.8 * c * 5.0 * 0.7 * 3.0 * 3.0;
    let pr = |c: f64| 4.74 * c * 5.0 * 0.7 * 3.0 * 3.0;
    close(v("5-2-2.3.7/bottom-longitudinals", &ops), st(1.34));
    close(
        v("5-2-2.3.5/inner-bottom-longitudinals", &ops),
        0.85 * st(1.30),
    );
    close(v("3-2-7.3.3/tank-boundary-longitudinals", &ops), st(1.0));
    close(
        v("3-2-7.3.3/longitudinal-bulkhead-stiffeners", &ops),
        st(0.90),
    );
    close(v("5-2-1.7/side-longitudinals", &ops), st(1.25));
    close(v("3-2-6.5.3/bulkhead-stiffeners", &ops), st(0.56));
    close(v("3-2-6.5.5/bulkhead-webs", &ops), pr(1.0));
    close(v("5-2-1.11/side-webs", &ops), pr(1.75));
    close(v("5-2-1.11/deck-transverses", &ops), pr(2.0));
}

/// Every Rule of the table has a worked example above (its id is named in this file), and every id
/// carries its paragraph with a dot for the slash.
#[test]
fn every_rule_has_a_worked_example() {
    let me = include_str!("scantling_rules.rs");
    for r in RULES {
        let short = r.id.trim_start_matches("abs-steel-barges/");
        assert!(me.contains(short), "{} has no worked example", r.id);
        assert_eq!(
            get(r.id).paragraph.replace('/', "."),
            short.split('/').next().unwrap()
        );
    }
}
