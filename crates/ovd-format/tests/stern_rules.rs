//! Seam: the Rule Sets of the stern and the wheelhouse (issue #355, first slice): `solas-v22`,
//! `rheinschpv`, and the Blind Distance, speed and redundancy Rules of `es-trin-2025`.
//!
//! Label: sourced - SOLAS 1974 Chapter V regulation 22.1.1 (MSC.99(73), annex p. 133); RheinSchPV 1994
//! 1.07(2); ES-TRIN 2025/1 Articles 5.06(1), 7.02(2), 7.02(3), 7.12(2), 28.04(1)(a) and ESI-III-6 2.1
//! (ES_TRIN_2025_signed_en.pdf)

use ovd_core::measured::Measured;
use ovd_core::rule_trace::{Outcome, formula_value};
use ovd_core::rules::{Compare, Limit};
use ovd_format::rule_set::Library;
use ovd_format::trace::{add_rule_set, trace_of};
use ovd_format::{Profile, RunOutcome, fill_as_new, run_option_with};
use serde_json::{Value, json};

fn design(extra: Value) -> ovd_format::Design {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 110.0, "source": "typed" },
        "beam_m": { "value": 11.4, "source": "typed" },
        "moulded_depth_m": { "value": 6.0, "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    filled.design.unwrap()
}

fn results(design: &ovd_format::Design) -> Value {
    match run_option_with(&design.options[0], &Library::built_in(), &[]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn rows(results: &Value, id: &str) -> Vec<Value> {
    results["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["rule"] == id && r["outcome"] != "not applicable")
        .cloned()
        .collect()
}

fn outcome_of(
    set: &str,
    version: &str,
    id: &str,
    measured: &Measured,
    option: &ovd_format::DesignOption,
) -> Outcome {
    let lib = Library::built_in();
    let selection = lib.select(&[(set, version)]).unwrap();
    trace_of(&selection, option, measured)
        .rows
        .iter()
        .find(|r| r.rule.as_deref() == Some(id))
        .unwrap_or_else(|| panic!("{id}"))
        .outcome
}

#[test]
fn the_solas_limit_is_two_ship_lengths_or_500_m_whichever_is_less() {
    // SOLAS V/22.1.1, by hand: 2 L, at most 500 m.
    let lib = Library::built_in();
    let set = lib.find("solas-v22", "1.0.0").unwrap();
    for id in [
        "solas/V.22.1.1/blind-zone-light",
        "solas/V.22.1.1/blind-zone-laden",
    ] {
        let rule = set.rule(id).unwrap();
        assert_eq!(rule.compare, Some(Compare::Le), "{id}");
        assert_eq!(rule.consequence, ovd_core::rules::Consequence::Warn, "{id}");
        assert_eq!(rule.finding, None, "{id}");
        // Issue #543: ships of not less than 45 m, read as a length overall.
        assert_eq!(
            rule.applies_when.conditions,
            vec![(
                "length_band".to_string(),
                vec!["45-to-110-m".to_string(), "over-110-m".to_string()]
            )],
            "{id}"
        );
        assert!(rule.verified.is_none(), "{id}");
        let Some(Limit::Formula {
            shape,
            coefficients,
            ..
        }) = &rule.limit
        else {
            panic!("{id}: a formula");
        };
        let limit = |l: f64| formula_value(shape, coefficients, |_| Ok(l)).unwrap();
        assert_eq!(limit(45.0), 90.0);
        assert_eq!(limit(110.0), 220.0);
        assert_eq!(limit(249.9), 499.8);
        assert_eq!(limit(250.0), 500.0);
        assert_eq!(limit(300.0), 500.0);
    }
}

#[test]
fn a_coastal_option_is_checked_against_two_lengths_and_the_light_blind_distance_warns() {
    // 110 m: the limit is 220 m. The light Blind Distance is 250 m (the ES-TRIN limit the search
    // sizes to), the laden one 216 m: the light condition warns, the laden one passes.
    let r = results(&design(json!({ "service_area": "coastal-and-river" })));
    let sets = r["rule_trace"]["rule_sets"].to_string();
    assert!(sets.contains("solas-v22@1.0.0"), "{sets}");
    let light = rows(&r, "solas/V.22.1.1/blind-zone-light");
    let laden = rows(&r, "solas/V.22.1.1/blind-zone-laden");
    assert_eq!(light.len(), 1);
    assert_eq!(light[0]["outcome"], "warn", "{light:?}");
    assert_eq!(light[0]["limit"], json!(220.0));
    assert_eq!(laden[0]["outcome"], "pass", "{laden:?}");
    // A river-only Option does not take the set.
    let river = results(&design(json!({})));
    assert!(
        !river["rule_trace"]["rule_sets"]
            .to_string()
            .contains("solas-v22")
    );
}

#[test]
fn a_craft_under_45_m_is_outside_solas_v_22_and_one_of_45_m_is_inside() {
    // Issue #543: the Rules apply from a length overall of 45 m. The coastal Option takes the set
    // at both lengths; the Rules have no row below 45 m and a row at 45 m.
    for (loa, applies) in [(44.9, false), (45.0, true), (110.0, true), (140.0, true)] {
        let r = results(&design(
            json!({ "service_area": "coastal-and-river", "length_overall_m": loa }),
        ));
        let all = r["rule_trace"]["rows"].as_array().unwrap();
        for id in [
            "solas/V.22.1.1/blind-zone-light",
            "solas/V.22.1.1/blind-zone-laden",
        ] {
            let mine: Vec<&Value> = all.iter().filter(|row| row["rule"] == id).collect();
            assert!(!mine.is_empty(), "{loa} m: no row for {id}");
            assert_eq!(
                mine.iter().any(|row| row["outcome"] != "not applicable"),
                applies,
                "{loa} m: {id}"
            );
        }
    }
}

#[test]
fn rheinschpv_is_optional_and_limits_the_laden_blind_distance_to_350_m() {
    let lib = Library::built_in();
    let rule = lib
        .find("rheinschpv", "1.0.0")
        .unwrap()
        .rule("rheinschpv/1.07(2)/blind-zone-laden")
        .unwrap();
    assert_eq!(rule.consequence, ovd_core::rules::Consequence::Block);
    // Both sides of the limit, through the trace.
    let d = design(json!({}));
    let at = |laden: f64| {
        let measured = Measured {
            blind_distance_laden_m: Some(laden),
            ..Measured::default()
        };
        outcome_of(
            "rheinschpv",
            "1.0.0",
            "rheinschpv/1.07(2)/blind-zone-laden",
            &measured,
            &d.options[0],
        )
    };
    assert_eq!(at(350.0), Outcome::Pass);
    assert_eq!(at(350.1), Outcome::Fail);
    assert_eq!(at(0.0), Outcome::Pass);
    // Not in any default stack; a Designer adds it, and it applies.
    let mut d = design(json!({}));
    assert!(
        !results(&d)["rule_trace"]["rule_sets"]
            .to_string()
            .contains("rheinschpv")
    );
    add_rule_set(&mut d.options[0], &lib, "rheinschpv").unwrap();
    let r = results(&d);
    assert!(
        r["rule_trace"]["rule_sets"]
            .to_string()
            .contains("rheinschpv@1.0.0")
    );
    assert_eq!(
        rows(&r, "rheinschpv/1.07(2)/blind-zone-laden")[0]["outcome"],
        "pass"
    );
}

// --- es-trin-2025: Articles 7.02(2), 5.06(1), 28.04(1)(a), and the Rules the model cannot read ----

#[test]
fn the_unladen_blind_distance_is_250_m_at_249_and_251() {
    let d = design(json!({}));
    let at = |light: f64| {
        let measured = Measured {
            blind_distance_light_m: Some(light),
            ..Measured::default()
        };
        outcome_of(
            "es-trin-2025",
            "1.0.0",
            "es-trin/7.02(2)/blind-zone-unladen",
            &measured,
            &d.options[0],
        )
    };
    assert_eq!(at(249.0), Outcome::Pass);
    assert_eq!(at(250.0), Outcome::Pass);
    assert_eq!(at(251.0), Outcome::Fail);
    // The margin method has no Blind Distance: the Rule does not apply.
    assert_eq!(at_missing(&d), Outcome::NotApplicable);
}

fn at_missing(d: &ovd_format::Design) -> Outcome {
    outcome_of(
        "es-trin-2025",
        "1.0.0",
        "es-trin/7.02(2)/blind-zone-unladen",
        &Measured::default(),
        &d.options[0],
    )
}

#[test]
fn the_speed_is_13_km_h_at_12_9_and_13_1() {
    // 13 km/h in knots: 13 / 1.852 = 7.0194 kn.
    let d = design(json!({}));
    let at = |kmh: f64| {
        let measured = Measured {
            speeds_kn: vec![("Deep water".to_string(), kmh / 1.852)],
            ..Measured::default()
        };
        outcome_of(
            "es-trin-2025",
            "1.0.0",
            "es-trin/5.06(1)/speed-13kmh",
            &measured,
            &d.options[0],
        )
    };
    assert_eq!(at(12.9), Outcome::Fail);
    assert_eq!(at(13.1), Outcome::Pass);
    assert_eq!(at(13.0), Outcome::Pass);
    // Another environment only: no value for the deep water, so no pass.
    let measured = Measured {
        speeds_kn: vec![("Shallow".to_string(), 12.0)],
        ..Measured::default()
    };
    assert_eq!(
        outcome_of(
            "es-trin-2025",
            "1.0.0",
            "es-trin/5.06(1)/speed-13kmh",
            &measured,
            &d.options[0]
        ),
        Outcome::Fail
    );
}

#[test]
fn a_craft_over_110_m_needs_a_bow_thruster_with_one_or_more_propulsors() {
    let id = "es-trin/28.04(1)(a)/propulsion-redundancy";
    let run = |loa: f64, thruster: &str| {
        let r = results(&design(json!({
            "length_overall_m": loa, "beam_m": 14.0, "moulded_depth_m": 7.0,
            "propulsor_count": 1, "bow_thruster": thruster
        })));
        rows(&r, id)
    };
    // 110.0 m is not over 110 m; 110.1 m with no bow thruster fails; with one, passes.
    assert_eq!(run(110.0, "none")[0]["outcome"], "pass");
    assert_eq!(run(110.1, "none")[0]["outcome"], "fail");
    assert_eq!(run(110.1, "channel_jet")[0]["outcome"], "pass");
    assert_eq!(run(110.1, "tunnel")[0]["outcome"], "pass");
}

#[test]
fn the_rules_the_model_cannot_read_are_named_not_passed() {
    let r = results(&design(json!({})));
    // The coupling force is for a pushed Formation: a self-propelled Option has no such row.
    assert!(rows(&r, "es-trin/ESI-III-6.2.1/coupling-force").is_empty());
    for (id, gap) in [
        ("es-trin/7.02(3)/field-of-view", "no window geometry"),
        (
            "es-trin/7.12(2)/elevating-wheelhouse-stability",
            "no stability case",
        ),
    ] {
        let row = rows(&r, id);
        assert_eq!(row.len(), 1, "{id}");
        assert_eq!(row[0]["outcome"], "not evaluated", "{id}");
        assert!(
            row[0]["gap"].as_str().unwrap().contains(gap),
            "{id}: {}",
            row[0]
        );
    }
}

#[test]
fn a_new_self_propelled_option_lists_every_es_trin_rule_and_a_barge_has_no_wheelhouse_rows() {
    let r = results(&design(json!({})));
    for id in [
        "es-trin/4.01(1)/safety-clearance",
        "es-trin/4.02(1)/freeboard",
        "es-trin/7.02(2)/blind-zone-unladen",
        "es-trin/5.06(1)/speed-13kmh",
        "es-trin/28.04(1)(a)/propulsion-redundancy",
    ] {
        let row = rows(&r, id);
        assert_eq!(row.len(), 1, "{id}");
        assert_eq!(row[0]["outcome"], "pass", "{id}: {}", row[0]);
    }
}

#[test]
fn a_pushed_barge_has_the_coupling_note_and_no_wheelhouse_rows() {
    let mut d = design(json!({}));
    for (name, input) in &mut d.options[0].inputs {
        if name == "formation" {
            input.value = json!({ "kind": "pushed_barge" });
        }
    }
    let r = results(&d);
    // The coupling force is a note with the value of the Formation block, when it has its pusher
    // (`formation_powering.rs` checks both sides).
    let row = rows(&r, "es-trin/ESI-III-6.2.1/coupling-force");
    match r["formation"]["coupling_force_kn"].as_f64() {
        Some(force) => {
            assert_eq!(row[0]["outcome"], "note", "{}", row[0]);
            assert_eq!(row[0]["value"].as_f64(), Some(force));
        }
        None => assert!(row.is_empty(), "{row:?}"),
    }
    for id in [
        "es-trin/7.02(3)/field-of-view",
        "es-trin/7.12(2)/elevating-wheelhouse-stability",
        "es-trin/7.02(2)/blind-zone-unladen",
        "es-trin/28.04(1)(a)/propulsion-redundancy",
        "es-trin/5.06(1)/speed-13kmh",
    ] {
        assert!(rows(&r, id).is_empty(), "{id}");
    }
}

// --- ovd 3.0.0: the OVD Checks the results already carry (issue #355, second slice) ------------

#[test]
fn installed_power_covers_the_design_speed_at_9_69_and_9_7() {
    let d = design(json!({}));
    let at = |speeds: Vec<(&str, f64)>| {
        let measured = Measured {
            speeds_kn: speeds
                .into_iter()
                .map(|(n, k)| (n.to_string(), k))
                .collect(),
            design_speed_kn: Some(9.7),
            ..Measured::default()
        };
        outcome_of(
            "ovd",
            "3.0.0",
            "ovd/powering/installed-covers-design-speed",
            &measured,
            &d.options[0],
        )
    };
    assert_eq!(at(vec![("Deep water", 9.7)]), Outcome::Pass);
    assert_eq!(at(vec![("Deep water", 10.0)]), Outcome::Pass);
    assert_eq!(at(vec![("Deep water", 9.69)]), Outcome::Fail);
    // The slowest environment governs.
    assert_eq!(
        at(vec![("Deep water", 10.0), ("Channel", 9.0)]),
        Outcome::Fail
    );
}

#[test]
fn the_lift_and_the_channel_depth_rules_at_both_sides() {
    let d = design(json!({}));
    let lift = |ok: bool| {
        let measured = Measured {
            lift_enough: Some(ok),
            ..Measured::default()
        };
        outcome_of(
            "ovd",
            "3.0.0",
            "ovd/wheelhouse/lift-enough",
            &measured,
            &d.options[0],
        )
    };
    assert_eq!(lift(true), Outcome::Pass);
    assert_eq!(lift(false), Outcome::Fail);
    let depth = |h_over_t: f64| {
        let measured = Measured {
            depth_over_draught: Some(h_over_t),
            ..Measured::default()
        };
        outcome_of(
            "ovd",
            "3.0.0",
            "ovd/powering/shallow-water-validity",
            &measured,
            &d.options[0],
        )
    };
    assert_eq!(depth(2.0), Outcome::Pass);
    assert_eq!(depth(1.99), Outcome::Warn);
    // No channel: the Rule does not apply.
    assert_eq!(
        outcome_of(
            "ovd",
            "3.0.0",
            "ovd/powering/shallow-water-validity",
            &Measured::default(),
            &d.options[0]
        ),
        Outcome::NotApplicable
    );
}

#[test]
fn a_new_self_propelled_option_passes_the_new_checks_and_a_typed_weak_engine_or_shallow_channel_shows()
 {
    let r = results(&design(json!({})));
    for id in [
        "ovd/powering/installed-covers-design-speed",
        "ovd/wheelhouse/lift-enough",
    ] {
        let row = rows(&r, id);
        assert_eq!(row.len(), 1, "{id}");
        assert_eq!(row[0]["outcome"], "pass", "{id}: {}", row[0]);
    }
    assert!(
        rows(&r, "ovd/powering/shallow-water-validity").is_empty(),
        "no channel"
    );
    // A typed installed power far below the need.
    let weak = results(&design(json!({ "installed_power_kw": 50.0 })));
    assert_eq!(
        rows(&weak, "ovd/powering/installed-covers-design-speed")[0]["outcome"],
        "fail"
    );
    // A channel 3 m deep under a draught of about 2.3 m: h/T is 1.3, a warning.
    let shallow = results(&design(
        json!({ "channel_depth_m": 3.0, "channel_bottom_width_m": 40.0 }),
    ));
    let row = rows(&shallow, "ovd/powering/shallow-water-validity");
    assert_eq!(row.len(), 1);
    assert_eq!(row[0]["outcome"], "warn", "{}", row[0]);
}

// ---- #355: the propeller Checks of the `ovd` set -------------------------------------------------

fn row_of(results: &Value, id: &str) -> Value {
    let found = rows(results, id);
    assert_eq!(found.len(), 1, "{id}: {found:?}");
    found[0].clone()
}

#[test]
fn the_propeller_checks_read_the_diameter_the_axis_and_the_blade_load_of_the_results() {
    // The Stern Form's D_max, as a file saved before has it.
    let r = results(&design(json!({
        "propulsor_type": "shaft_open", "stern_form": "transom", "cargo": "LPG_propane",
        "propeller_diameter_basis": "stern_form"
    })));
    let pw = &r["powering"];
    let d = pw["propeller_diameter_m"].as_f64().unwrap();
    // The axis is half a diameter above the base; the blade load is the delivered power per disc.
    assert!((pw["shaft_axis_height_m"].as_f64().unwrap() - d / 2.0).abs() < 1e-12);
    assert_eq!(pw["propeller_ducted"], false);
    assert_eq!(pw["blade_load_limit_kw_m2"], 400.0);
    // Hand value: installed power over one unit, through the shaft line (0.99) and the source,
    // over pi D^2 / 4. The source efficiency is not known here, so bound it: at most the shaft power.
    let disc = std::f64::consts::PI * d * d / 4.0;
    let load = pw["blade_load_kw_m2"].as_f64().unwrap();
    assert!(
        load > 0.0 && load <= pw["installed_power_kw"].as_f64().unwrap() / disc,
        "{load}"
    );
    // The diameter is the Stern Form's largest, so the Check passes at exactly 1.
    let fits = row_of(&r, "ovd/propulsion/propeller-fits-stern");
    assert_eq!(fits["outcome"], "pass");
    assert!(
        (fits["value"].as_f64().unwrap() - 1.0).abs() < 1e-12,
        "{fits}"
    );
    // Blade load: the row value is the load over the limit.
    let blade = row_of(&r, "ovd/propulsion/blade-load");
    assert!(
        (blade["value"].as_f64().unwrap() - load / 400.0).abs() < 1e-12,
        "{blade}"
    );
    // Immersion: the lightest aft draught of the conditions, less the axis, over D and over 0.76.
    let aft = r["loading_conditions"]["conditions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["draught_aft_m"].as_f64().unwrap())
        .fold(f64::INFINITY, f64::min);
    let imm = row_of(&r, "ovd/propulsion/immersion-light");
    let want = (aft - d / 2.0) / d / 0.76;
    assert!(
        (imm["value"].as_f64().unwrap() - want).abs() < 1e-12,
        "{imm} {want}"
    );
    // D_max is the largest the design draught allows, so the axis is higher than the light draught
    // immerses: a warning, not a block.
    assert!(want < 1.0, "{want}");
    assert_eq!(imm["outcome"], "warn");
}

fn lightest_aft(r: &Value) -> f64 {
    r["loading_conditions"]["conditions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["draught_aft_m"].as_f64().unwrap())
        .fold(f64::INFINITY, f64::min)
}

#[test]
fn a_new_gas_option_sizes_its_propeller_to_the_light_draught_and_passes_the_immersion_check() {
    // The maintainer's decision on #355: a default self-propelled hull passes at the light draught.
    // A new gas Option: a transom stern with ducted azimuth units (limit 0.71).
    let r = results(&design(json!({ "cargo": "LPG_propane" })));
    assert_eq!(r["geometry"]["stern_form"], "transom");
    let d = r["powering"]["propeller_diameter_m"].as_f64().unwrap();
    let d_max = r["geometry"]["propeller_d_max_m"].as_f64().unwrap();
    assert!(d < d_max, "{d} {d_max}");
    // Rounded down to 0.05 m, and the axis (D/2 above the base) is 0.71 D under the lightest aft draught.
    assert!(((d / 0.05).round() * 0.05 - d).abs() < 1e-9, "{d}");
    let aft = lightest_aft(&r);
    assert!(aft - d / 2.0 >= 0.71 * d, "{aft} {d}");
    let imm = row_of(&r, "ovd/propulsion/immersion-light");
    assert_eq!(imm["outcome"], "pass", "{imm}");
    assert_eq!(
        row_of(&r, "ovd/propulsion/propeller-fits-stern")["outcome"],
        "pass"
    );
}

#[test]
fn the_stern_form_basis_and_a_typed_diameter_keep_their_diameter_and_the_warning() {
    let legacy = results(&design(
        json!({ "cargo": "LPG_propane", "propeller_diameter_basis": "stern_form" }),
    ));
    let d_max = legacy["geometry"]["propeller_d_max_m"].as_f64().unwrap();
    assert_eq!(
        legacy["powering"]["propeller_diameter_m"].as_f64(),
        Some(d_max)
    );
    assert_eq!(
        row_of(&legacy, "ovd/propulsion/immersion-light")["outcome"],
        "warn"
    );
    // A typed diameter is used as typed, though it warns.
    let typed = results(&design(
        json!({ "cargo": "LPG_propane", "propeller_diameter_m": 3.0 }),
    ));
    assert_eq!(
        typed["powering"]["propeller_diameter_m"].as_f64(),
        Some(3.0)
    );
    assert_eq!(
        row_of(&typed, "ovd/propulsion/immersion-light")["outcome"],
        "warn"
    );
    // A tunnel stern keeps D_max: the Check does not apply there.
    let tunnel = results(&design(
        json!({ "cargo": "LPG_propane", "stern_form": "tunnel" }),
    ));
    assert_eq!(
        tunnel["powering"]["propeller_diameter_m"].as_f64(),
        tunnel["geometry"]["propeller_d_max_m"].as_f64()
    );
}

#[test]
fn a_ducted_propeller_has_the_ducted_limits_and_a_tunnel_stern_has_no_immersion_check() {
    let ducted = results(&design(json!({
        "propulsor_type": "shaft_nozzle", "stern_form": "transom", "cargo": "LPG_propane"
    })));
    assert_eq!(ducted["powering"]["blade_load_limit_kw_m2"], 500.0);
    assert_eq!(ducted["powering"]["propeller_ducted"], true);
    let d = ducted["powering"]["propeller_diameter_m"].as_f64().unwrap();
    let aft = ducted["loading_conditions"]["conditions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["draught_aft_m"].as_f64().unwrap())
        .fold(f64::INFINITY, f64::min);
    let imm = row_of(&ducted, "ovd/propulsion/immersion-light");
    assert!((imm["value"].as_f64().unwrap() - (aft - d / 2.0) / d / 0.71).abs() < 1e-12);
    let tunnel = results(&design(json!({
        "propulsor_type": "shaft_open", "stern_form": "tunnel", "cargo": "LPG_propane"
    })));
    assert!(rows(&tunnel, "ovd/propulsion/immersion-light").is_empty());
    assert_eq!(rows(&tunnel, "ovd/propulsion/blade-load").len(), 1);
}

#[test]
fn a_typed_diameter_over_the_stern_forms_largest_warns() {
    let r = results(&design(json!({
        "propulsor_type": "shaft_open", "stern_form": "transom", "cargo": "LPG_propane",
        "propeller_diameter_m": 4.5
    })));
    let fits = row_of(&r, "ovd/propulsion/propeller-fits-stern");
    assert_eq!(fits["outcome"], "warn");
    assert!(
        (fits["value"].as_f64().unwrap() - 4.5 / 3.705).abs() < 1e-9,
        "{fits}"
    );
}

#[test]
fn a_file_with_no_specified_propulsor_has_no_propeller_numbers() {
    let r = results(&design(
        json!({ "cargo": "LPG_propane", "propulsor_type": "unspecified" }),
    ));
    assert!(r["powering"].is_null());
    // The diameter is the Stern Form's own, so that Check reads 1; the other two have no Propulsor.
    for id in [
        "ovd/propulsion/immersion-light",
        "ovd/propulsion/blade-load",
    ] {
        let all: Vec<_> = r["rule_trace"]["rows"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["rule"] == id)
            .collect();
        assert!(
            all.iter()
                .all(|row| row["outcome"] != "pass" && row["outcome"] != "fail"),
            "{id}: {all:?}"
        );
    }
}
