//! Seam: `simulate` with its inputs away from their defaults (issue #64). Mutation testing of the
//! port left survivors that were input-variety gaps: nothing set these inputs to anything but the
//! default, so a change in how they are passed went unnoticed.
//!
//! Label: property. Every test states a rule worked out from the physics or the code's contract,
//! not a printed number. Where a test compares with another run of `simulate`, the comparison is
//! the rule (an explicit default gives the same answer as leaving the input out).
//!
//! `adn_tank_cap_m3` no longer exists (#175 removed it): its replacement is `max_tank_od_m`, so the
//! "non-default tank cap" case here sets `max_tank_od_m`.

use ovcd_format::{eval, parse_case};
use serde_json::{Value, json};

/// The `ok` object `simulate` returns for these params.
fn sim(params: Value) -> Value {
    let case =
        parse_case(&json!({ "fn": "simulate", "args": { "params": params } }).to_string()).unwrap();
    eval("simulate", &case).unwrap()["ok"].clone()
}

/// A self-propelled sloped-hopper bauxite barge, 70 x 11 x 4 m, Cb 0.9, with `extra` params merged in.
fn hopper(extra: Value) -> Value {
    let mut p = json!({
        "vessel": "hopper_barge", "cargo": "bauxite", "loa_m": 70, "beam_m": 11, "depth_m": 4,
        "cb": 0.9, "self_propelled": true
    });
    for (k, v) in extra.as_object().unwrap() {
        p[k] = v.clone();
    }
    p
}

/// A gas barge, 90 x 30 x 6.5 m (wide enough for the tank cap to bind), with `extra` merged in.
fn gas(extra: Value) -> Value {
    let mut p = json!({
        "vessel": "gas_barge", "cargo": "LPG_propane", "loa_m": 90, "beam_m": 30, "depth_m": 6.5,
        "cb": 0.85
    });
    for (k, v) in extra.as_object().unwrap() {
        p[k] = v.clone();
    }
    p
}

fn num(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("not a number: {v}"))
}

fn close(a: f64, b: f64, what: &str) {
    assert!(
        (a - b).abs() <= 1e-12 * b.abs().max(1.0),
        "{what}: {a} vs {b}"
    );
}

/// The speed curve of environment `env` as `(kn, kw, squat_m)`.
fn points(v: &Value, env: &str) -> Vec<(f64, f64, f64)> {
    let e = v["curves"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["name"] == env)
        .unwrap_or_else(|| panic!("no {env} curve"));
    e["points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (num(&p["kn"]), num(&p["kw"]), num(&p["squat_m"])))
        .collect()
}

fn channel_env(v: &Value) -> &Value {
    v["curves"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["name"] == "Channel")
        .unwrap()
}

const CHANNEL: &str = "Channel";

// --- Channel environment ------------------------------------------------------------------

/// Worked out: an opposing current of c knots means a hull making V knots over the ground moves
/// V + c knots through the water, so the power and the squat at V over the ground are those of
/// V + c knots in still water.
#[test]
fn an_opposing_current_adds_its_speed_to_the_speed_through_the_water() {
    let still = sim(hopper(
        json!({ "channel": { "depth_m": 6.0, "bottom_width_m": 60.0 } }),
    ));
    let still_pts = points(&still, CHANNEL);
    for current in [0.5, 1.0, 2.0] {
        let moving = sim(hopper(json!({
            "channel": { "depth_m": 6.0, "bottom_width_m": 60.0, "current_kn": current }
        })));
        let pts = points(&moving, CHANNEL);
        assert!(pts.len() > 8, "{current}: {} points", pts.len());
        for (kn, kw, squat) in pts {
            // The still-water curve ends at the top speed (10 kn); compare up to there.
            let Some((_, kw0, squat0)) = still_pts
                .iter()
                .find(|(k, _, _)| (*k - (kn + current)).abs() < 1e-9)
            else {
                assert!(
                    kn + current > 10.0,
                    "no still-water point at {}",
                    kn + current
                );
                continue;
            };
            close(kw, *kw0, &format!("kw at {kn} kn, current {current}"));
            close(
                squat,
                *squat0,
                &format!("squat at {kn} kn, current {current}"),
            );
        }
    }
}

/// The deep-water curve has no current: it is the same with or without one in the channel.
#[test]
fn the_deep_water_curve_ignores_the_channel_current() {
    let a = sim(hopper(
        json!({ "channel": { "depth_m": 6.0, "bottom_width_m": 60.0 } }),
    ));
    let b = sim(hopper(json!({
        "channel": { "depth_m": 6.0, "bottom_width_m": 60.0, "current_kn": 2.0 }
    })));
    assert_eq!(points(&a, "Deep water"), points(&b, "Deep water"));
}

/// Worked out: the blockage ratio is the midship area of the hull, beam x draught x Cb, over the
/// channel section, depth x (bottom width + slope x depth). So the bank slope changes it in a
/// way that can be computed.
#[test]
fn the_bank_slope_widens_the_channel_section_in_the_blockage_ratio() {
    let (depth, bottom) = (6.0, 60.0);
    for slope in [0.0, 1.0, 2.5, 3.0] {
        let v = sim(hopper(json!({
            "channel": { "depth_m": depth, "bottom_width_m": bottom, "bank_slope": slope }
        })));
        let t = num(&v["results"]["draught_m"]);
        // The midship section is B x T (issue #396; it was B x T x Cb).
        let expected = 11.0 * t / (depth * (bottom + slope * depth));
        close(
            num(&channel_env(&v)["blockage"]),
            expected,
            &format!("slope {slope}"),
        );
    }
}

/// Issue #425: the midship section is B x T x Cm where no hull section is given. Issue #448: a
/// Design gives the hull's own midship section from its sections, so the blockage ratio is the
/// sections' and Cm does not move it (#535 takes the input out of format v3).
#[test]
fn the_hull_section_sets_the_blockage_ratio_and_the_midship_coefficient_does_not() {
    let channel = json!({ "depth_m": 6.0, "bottom_width_m": 60.0 });
    let bare = sim(hopper(json!({ "channel": channel })));
    assert!(num(&channel_env(&bare)["blockage"]) > 0.0);
    for cm in [0.5, 0.8, 0.95, 1.0] {
        let v = sim(hopper(
            json!({ "channel": channel, "midship_coefficient": cm }),
        ));
        assert_eq!(v, bare, "Cm {cm}");
    }
}

/// An input left out takes its default. Giving the default explicitly must change nothing, and
/// giving anything else must. Defaults: bank slope 3, current 0 kn, under-keel clearance 0.5 m,
/// propulsive efficiency 0.45, top speed 10 kn, minimum freeboard 0.3 m.
#[test]
fn an_explicit_default_gives_the_same_answer_as_leaving_the_input_out() {
    let channel = json!({ "depth_m": 4.0, "bottom_width_m": 40.0 });
    let bare = sim(hopper(json!({ "channel": channel })));
    let explicit = sim(hopper(json!({
        "channel": { "depth_m": 4.0, "bottom_width_m": 40.0, "bank_slope": 3.0, "current_kn": 0.0,
                     "ukc_m": 0.5 },
        "eta_d": 0.45, "speed_max_kn": 10.0, "freeboard_min_m": 0.3
    })));
    assert_eq!(bare, explicit);
    for (key, other) in [
        ("bank_slope", json!(1.0)),
        ("current_kn", json!(1.0)),
        ("ukc_m", json!(0.4)),
    ] {
        let mut ch = json!({ "depth_m": 4.0, "bottom_width_m": 40.0 });
        ch[key] = other;
        assert_ne!(sim(hopper(json!({ "channel": ch }))), bare, "{key}");
    }
    for (key, other) in [("eta_d", json!(0.5)), ("speed_max_kn", json!(8.0))] {
        let mut p = json!({ "channel": channel });
        p[key] = other;
        assert_ne!(sim(hopper(p)), bare, "{key}");
    }
}

/// Worked out: the hull limit is the depth less the minimum freeboard, 0.3 m unless given.
#[test]
fn the_minimum_freeboard_sets_the_hull_limit() {
    let barge = json!({ "depth_m": 2.5, "self_propelled": false });
    let cap = |extra: Value| {
        let mut p = hopper(barge.clone());
        for (k, v) in extra.as_object().unwrap() {
            p[k] = v.clone();
        }
        num(&sim(p)["results"]["draught_max_m"])
    };
    close(cap(json!({})), 2.2, "default 0.3 m");
    close(
        cap(json!({ "freeboard_min_m": 0.3 })),
        2.2,
        "explicit 0.3 m",
    );
    close(cap(json!({ "freeboard_min_m": 0.4 })), 2.1, "0.4 m");
    close(cap(json!({ "freeboard_min_m": 0 })), 2.5, "none");
}

/// Worked out: the power is the delivered power over the propulsive efficiency, so with the same
/// hull and speeds, a lower efficiency needs proportionally more power: kw x eta is constant.
#[test]
fn the_propulsive_efficiency_scales_the_power() {
    let a = sim(hopper(json!({ "eta_d": 0.45 })));
    let b = sim(hopper(json!({ "eta_d": 0.6 })));
    let (pa, pb) = (points(&a, "Deep water"), points(&b, "Deep water"));
    assert_eq!(pa.len(), pb.len());
    for (x, y) in pa.iter().zip(&pb) {
        close(x.1 * 0.45, y.1 * 0.6, &format!("kw at {} kn", x.0));
    }
}

/// The top speed sets the last point of every curve: 1 kn to `speed_max_kn` in 0.5 kn steps.
#[test]
fn the_top_speed_ends_the_deep_water_curve() {
    for top in [6.0, 10.0, 12.5] {
        let v = sim(hopper(json!({ "speed_max_kn": top })));
        let pts = points(&v, "Deep water");
        close(pts[0].0, 1.0, "first speed");
        close(pts.last().unwrap().0, top, "last speed");
    }
}

/// Worked out: squat is a channel effect. In a 3 m deep channel the hull squats, so every channel
/// point has squat above zero, and from a speed up its dynamic under-keel clearance is below the
/// safety margin and the points are flagged `squat_limited`, and stay flagged at higher speed.
/// Deep water (50 m) has no squat limit at all: zero squat, never flagged, no limiting speed, and
/// its curve runs to the top speed. The channel curve is cut at 95 % of its limiting speed.
#[test]
fn squat_applies_in_the_channel_and_never_in_deep_water() {
    let v = sim(hopper(json!({
        "speed_max_kn": 15,
        // Wide enough that squat limits the speed before 0.95 x the Schijf limit (issue #397;
        // it was 40 m).
        "channel": { "depth_m": 3.0, "bottom_width_m": 120.0 }
    })));
    let limit = num(&channel_env(&v)["limiting_speed_kn"]);
    let ch = points(&v, CHANNEL);
    let deep = points(&v, "Deep water");
    assert!(ch.iter().all(|p| p.2 > 0.0), "squat in every channel point");
    let last = ch.last().unwrap().0;
    assert!(
        last <= 0.95 * limit && last < 15.0,
        "{last} vs limit {limit}"
    );
    let flags: Vec<bool> = channel_env(&v)["points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["squat_limited"].as_bool().unwrap())
        .collect();
    let first = flags
        .iter()
        .position(|f| *f)
        .expect("some points are squat limited");
    assert!(first > 0 && flags[first..].iter().all(|f| *f), "{flags:?}");
    assert!(deep.iter().all(|p| p.2 == 0.0), "no squat in deep water");
    close(
        deep.last().unwrap().0,
        15.0,
        "deep water runs to the top speed",
    );
    let deep_env = &v["curves"][1];
    assert_eq!(deep_env["name"], "Deep water");
    assert_eq!(deep_env["limiting_speed_kn"], Value::Null);
    assert!(
        deep_env["points"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["squat_limited"] == json!(false))
    );
}

/// A channel is a curve only with both a depth and a bottom width; each is a truthy number.
#[test]
fn a_channel_curve_needs_a_depth_and_a_bottom_width() {
    let names = |v: &Value| -> Vec<String> {
        v["curves"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["name"].as_str().unwrap().to_string())
            .collect()
    };
    let only_depth = sim(hopper(json!({ "channel": { "depth_m": 6.0 } })));
    assert_eq!(names(&only_depth), ["Deep water"]);
    let only_width = sim(hopper(json!({ "channel": { "bottom_width_m": 60.0 } })));
    assert_eq!(names(&only_width), ["Deep water"]);
    let zero_width = sim(hopper(
        json!({ "channel": { "depth_m": 6.0, "bottom_width_m": 0 } }),
    ));
    assert_eq!(names(&zero_width), ["Deep water"]);
    let both = sim(hopper(
        json!({ "channel": { "depth_m": 6.0, "bottom_width_m": 60.0 } }),
    ));
    assert_eq!(names(&both), ["Channel", "Deep water"]);
}

/// Worked out: `h_over_T` is the channel depth over the loaded draught, and absent (null) without
/// a channel depth, or with a zero one.
#[test]
fn h_over_t_is_the_channel_depth_over_the_draught() {
    let v = sim(hopper(json!({ "channel": { "depth_m": 6.0 } })));
    close(
        num(&v["results"]["h_over_T"]),
        6.0 / num(&v["results"]["draught_m"]),
        "h/T",
    );
    for ch in [
        json!({}),
        json!({ "depth_m": 0 }),
        json!({ "depth_m": null }),
        json!(null),
    ] {
        let v = sim(hopper(json!({ "channel": ch })));
        assert_eq!(v["results"]["h_over_T"], Value::Null, "{ch}");
    }
}

// --- The draught limit -------------------------------------------------------------------

/// A shallow hull, 70 x 11 x 2.5 m with 0.5 m freeboard: the hull limit is 2.5 - 0.5 = 2.0 m
/// exactly, and the cargo is draught-bound, so `governs` names the limit that sets the draught.
fn shallow(extra: Value) -> Value {
    let mut p = hopper(json!({
        "depth_m": 2.5, "freeboard_min_m": 0.5, "self_propelled": false
    }));
    for (k, v) in extra.as_object().unwrap() {
        p[k] = v.clone();
    }
    p
}

/// Worked out: `min` of limits takes the first of equal values. The hull limit (2.0 m) comes
/// first, then the draught limit, then the channel limit (depth less under-keel clearance).
#[test]
fn the_first_of_equal_limits_is_the_one_named() {
    let hull = "draught \u{2014} Hull (depth \u{2212} min. freeboard)";
    let tie_hull = sim(shallow(json!({ "draught_limit_m": 2.0 })));
    assert_eq!(tie_hull["results"]["governs"], hull);
    close(num(&tie_hull["results"]["draught_max_m"]), 2.0, "cap");
    let tie_draught = sim(shallow(json!({
        "draught_limit_m": 1.5, "channel": { "depth_m": 2.0, "ukc_m": 0.5 }
    })));
    assert_eq!(
        tie_draught["results"]["governs"],
        "draught \u{2014} Draught limit"
    );
    let channel = sim(shallow(
        json!({ "channel": { "depth_m": 2.5, "ukc_m": 0.5 } }),
    ));
    assert_eq!(channel["results"]["governs"], hull);
    let lower = sim(shallow(
        json!({ "channel": { "depth_m": 2.4, "ukc_m": 0.5 } }),
    ));
    assert_eq!(
        lower["results"]["governs"],
        "draught \u{2014} Channel (depth \u{2212} under-keel clearance)"
    );
    close(num(&lower["results"]["draught_max_m"]), 1.9, "channel cap");
}

/// A zero draught limit, and a channel with no depth or a zero depth, are no limit at all.
#[test]
fn a_zero_draught_limit_and_a_zero_channel_depth_are_no_limit() {
    let none = sim(shallow(json!({})));
    for extra in [
        json!({ "draught_limit_m": 0 }),
        json!({ "draught_limit_m": null }),
        json!({ "channel": { "depth_m": 0 } }),
        json!({ "channel": {} }),
        json!({ "channel": null }),
    ] {
        let v = sim(shallow(extra.clone()));
        assert_eq!(v["results"], none["results"], "{extra}");
    }
}

/// The draught limit is `depth - ukc` for a channel, 0.5 m under keel by default.
#[test]
fn the_channel_limit_is_its_depth_less_the_under_keel_clearance() {
    // The channel is 2.3 m deep; the hull limit (2.0 m) caps the draught where the channel is
    // looser.
    for (ukc, expect) in [
        (None, 1.8),
        (Some(0.3), 2.0),
        (Some(0.8), 1.5),
        (Some(0.1), 2.0),
    ] {
        let mut ch = json!({ "depth_m": 2.3 });
        if let Some(u) = ukc {
            ch["ukc_m"] = json!(u);
        }
        let v = sim(shallow(json!({ "channel": ch })));
        close(
            num(&v["results"]["draught_max_m"]),
            expect,
            &format!("{ukc:?}"),
        );
    }
}

/// Worked out: the hold volume binds when the volume bound is at most the weight bound, so a tie
/// reports `hold volume`. The draught limit that makes the weight bound equal the volume bound is
/// found here to the last bit. Issue #448: the weight bound is the hull's displacement at the
/// draught limit less the lightship, so the limit is found by bisection on its bits, with the
/// weight bound `simulate` reports.
#[test]
fn a_tie_between_volume_and_weight_bound_is_reported_as_the_hold_volume() {
    // Not every hull has a draught limit that gives the tie to the last bit (the volume and the
    // subtraction round), so try lengths from 60 m up until one has.
    for loa in 60..=100 {
        let barge = |extra: Value| {
            let mut p = hopper(json!({ "self_propelled": false, "loa_m": loa }));
            for (k, v) in extra.as_object().unwrap() {
                p[k] = v.clone();
            }
            p
        };
        let weight = |t: f64| {
            num(&sim(barge(json!({ "draught_limit_m": t })))["results"]["cargo_weight_bound_t"])
        };
        let volume =
            num(&sim(barge(json!({ "draught_limit_m": 1.9 })))["results"]["cargo_volume_bound_t"]);
        // The weight bound rises with the draught limit: bisect on the bits of positive floats,
        // which order as the numbers do.
        // Below the hull's own limit, depth less freeboard: 4 - 0.3 m.
        let (mut lo, mut hi) = (0.5_f64.to_bits(), 3.6_f64.to_bits());
        if weight(f64::from_bits(lo)) >= volume || weight(f64::from_bits(hi)) <= volume {
            continue;
        }
        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;
            if weight(f64::from_bits(mid)) < volume {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let t = f64::from_bits(hi);
        if weight(t) != volume {
            continue;
        }
        let v = sim(barge(json!({ "draught_limit_m": t })));
        assert_eq!(
            v["results"]["cargo_weight_bound_t"],
            v["results"]["cargo_volume_bound_t"]
        );
        assert_eq!(v["results"]["governs"], "hold volume", "loa {loa}");
        return;
    }
    panic!("no hull from 60 to 100 m has an exact tie");
}

// --- Reading the inputs ------------------------------------------------------------------

/// Python `float(True)` is 1.0: a boolean is read as the number 1 or 0.
#[test]
fn a_boolean_is_read_as_the_number_one_or_zero() {
    let v = sim(hopper(json!({ "loa_m": true })));
    assert_eq!(v["dims"]["loa_m"], json!(1.0));
    let v = sim(gas(json!({ "fill_ratio": true })));
    assert_eq!(v["arrangement"]["fill_ratio"], json!(1.0));
}

/// Python truthiness decides `self_propelled`: zero, empty text and empty containers are false.
#[test]
fn self_propelled_follows_python_truthiness() {
    for (value, expected) in [
        (json!(false), false),
        (json!(0), false),
        (json!(0.0), false),
        (json!(""), false),
        (json!([]), false),
        (json!({}), false),
        (json!(null), false),
        (json!(true), true),
        (json!(1), true),
        (json!(0.5), true),
        (json!("no"), true),
        (json!("0"), true),
        (json!([0]), true),
        (json!({ "a": 1 }), true),
    ] {
        let v = sim(shallow(json!({ "self_propelled": value })));
        assert_eq!(v["self_propelled"], json!(expected), "{value}");
    }
}

/// A falsy fill ratio takes the ruleset default of 0.98; a truthy one is used as given.
#[test]
fn a_zero_fill_ratio_takes_the_default_and_another_is_used() {
    for falsy in [json!(0), json!(0.0), json!(null), json!(false), json!("")] {
        let v = sim(gas(json!({ "fill_ratio": falsy })));
        assert_eq!(v["arrangement"]["fill_ratio"], json!(0.98), "{falsy}");
    }
    let default = sim(gas(json!({})));
    assert_eq!(default["arrangement"]["fill_ratio"], json!(0.98));
    let lower = sim(gas(json!({ "fill_ratio": 0.9 })));
    assert_eq!(lower["arrangement"]["fill_ratio"], json!(0.9));
    assert!(num(&lower["results"]["cargo_t"]) < num(&default["results"]["cargo_t"]));
}

/// `cargo_options` lists every cargo with its id, name, phase and density in kg/m3.
#[test]
fn cargo_options_lists_every_cargo() {
    let case = parse_case(r#"{"fn":"cargo_options","args":{}}"#).unwrap();
    let options = eval("simulate", &case).unwrap()["ok"].clone();
    let items = options.as_array().unwrap();
    assert!(items.len() >= 6);
    let lng = items.iter().find(|o| o["id"] == "LNG").unwrap();
    assert_eq!(lng["name"], "LNG");
    assert_eq!(lng["phase"], "liquid_gas");
    assert_eq!(lng["density_kg_m3"], json!(450.0));
    assert!(items.iter().any(|o| o["phase"] == "dry_bulk"));
}

/// An error message shows a number as Python `repr` does: `1e+22` and `1e-07`, not the JSON text.
#[test]
fn a_number_in_an_error_message_is_written_as_python_writes_it() {
    for (text, shown) in [
        ("1e22", "1e+22"),
        ("1e-7", "1e-07"),
        ("5.0", "5.0"),
        ("7", "7"),
    ] {
        let params = format!(
            r#"{{"vessel":"gas_barge","cargo":"LPG_propane","loa_m":90,"beam_m":14.8,"depth_m":6.5,"cb":0.85,"ruleset":{text}}}"#
        );
        let case = parse_case(&format!(
            r#"{{"fn":"simulate","args":{{"params":{params}}}}}"#
        ))
        .unwrap();
        let out = eval("simulate", &case).unwrap();
        assert_eq!(
            out["ok"]["error"],
            format!("Unknown ruleset {shown}."),
            "{text}"
        );
    }
}

// --- The gas route -----------------------------------------------------------------------

/// Worked out: `max_tank_od_m` reaches the arrangement search, and the search never makes a tank
/// wider than it. So the tank outer diameter is at most the cap, a small cap on a wide hull
/// gives a smaller tank and less cargo, and the default cap (16 m) given explicitly changes
/// nothing. (Replaces the issue's "non-default ADN tank cap": that input was removed in #175.)
#[test]
fn max_tank_od_m_reaches_the_arrangement_search() {
    let default = sim(gas(json!({})));
    assert_eq!(sim(gas(json!({ "max_tank_od_m": 16.0 }))), default);
    // Since issue #264 the search keeps only arrangements that pass stability, so a smaller cap
    // does not always mean less cargo (the arrangement left after the cap can be a stabler one):
    // each cap holds the tank to the cap, and the smallest cap gives a smaller tank and less cargo.
    let (mut od, mut cargo) = (f64::INFINITY, f64::INFINITY);
    for cap in [12.0, 8.0, 6.0] {
        let v = sim(gas(json!({ "max_tank_od_m": cap })));
        od = num(&v["arrangement"]["tank_od_m"]);
        cargo = num(&v["results"]["cargo_t"]);
        assert!(od <= cap + 1e-6, "cap {cap}: tank OD {od}");
    }
    assert!(od < num(&default["arrangement"]["tank_od_m"]));
    assert!(cargo < num(&default["results"]["cargo_t"]));
}

/// Worked out: a hull too short for any tank carries nothing, so there is no loaded draught and
/// the draught reported is the light draught the engine gives; and it says why.
#[test]
fn a_hull_that_carries_nothing_reports_its_light_draught() {
    let v = sim(json!({
        "vessel": "gas_barge", "cargo": "LPG_propane", "loa_m": 20, "beam_m": 14.8, "depth_m": 6.5,
        "cb": 0.85
    }));
    assert_eq!(v["results"]["cargo_t"], json!(0.0));
    assert_eq!(
        v["results"]["governs"],
        "no cargo area (hull shorter than its fixed end spaces)"
    );
    let light = num(&v["results"]["light_draught_m"]);
    // Issue #448: with no tanks the lightship is 0 t, so the hull floats at no draught. The old
    // bisection stopped at its first close-enough midpoint, a draught of a few micrometres.
    assert_eq!(v["results"]["lightship_t"], json!(0.0));
    assert_eq!(light, 0.0);
    assert_eq!(v["results"]["draught_m"], json!(light));
    assert_eq!(v["checks"][0]["name"], "Cargo > 0");
    assert_eq!(v["checks"][0]["ok"], json!(false));
    assert_eq!(
        v["checks"][0]["message"],
        "Nothing fits: lengthen or widen the hull."
    );
    assert_eq!(v["stability"], Value::Null);
}
