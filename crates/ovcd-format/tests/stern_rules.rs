//! Seam: the Rule Sets of the stern and the wheelhouse (issue #355, first slice): `solas-v22`,
//! `rheinschpv`, and the Blind Distance, speed and redundancy Rules of `es-trin-2025`.
//!
//! Label: sourced - SOLAS 1974 Chapter V regulation 22.1.1 (MSC.99(73), annex p. 133); RheinSchPV 1994
//! 1.07(2); ES-TRIN 2025/1 Articles 5.06(1), 7.02(2), 7.02(3), 7.12(2), 28.04(1)(a) and ESI-III-6 2.1
//! (ES_TRIN_2025_signed_en.pdf)

use ovcd_core::measured::Measured;
use ovcd_core::rule_trace::{Outcome, formula_value};
use ovcd_core::rules::{Compare, Limit};
use ovcd_format::rule_set::Library;
use ovcd_format::trace::{add_rule_set, trace_of};
use ovcd_format::{Profile, RunOutcome, fill_as_new, run_option_with};
use serde_json::{Value, json};

fn design(extra: Value) -> ovcd_format::Design {
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

fn results(design: &ovcd_format::Design) -> Value {
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

fn outcome_of(set: &str, version: &str, id: &str, measured: &Measured, option: &ovcd_format::DesignOption) -> Outcome {
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
    for id in ["solas/V.22.1.1/blind-zone-light", "solas/V.22.1.1/blind-zone-laden"] {
        let rule = set.rule(id).unwrap();
        assert_eq!(rule.compare, Some(Compare::Le), "{id}");
        assert_eq!(rule.consequence, ovcd_core::rules::Consequence::Warn, "{id}");
        assert_eq!(rule.finding.as_deref(), Some("#543"), "{id}");
        assert!(rule.verified.is_none(), "{id}");
        let Some(Limit::Formula { shape, coefficients, .. }) = &rule.limit else {
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
    assert!(!river["rule_trace"]["rule_sets"].to_string().contains("solas-v22"));
}

#[test]
fn rheinschpv_is_optional_and_limits_the_laden_blind_distance_to_350_m() {
    let lib = Library::built_in();
    let rule = lib
        .find("rheinschpv", "1.0.0")
        .unwrap()
        .rule("rheinschpv/1.07(2)/blind-zone-laden")
        .unwrap();
    assert_eq!(rule.consequence, ovcd_core::rules::Consequence::Block);
    // Both sides of the limit, through the trace.
    let d = design(json!({}));
    let at = |laden: f64| {
        let measured = Measured {
            blind_distance_laden_m: Some(laden),
            ..Measured::default()
        };
        outcome_of("rheinschpv", "1.0.0", "rheinschpv/1.07(2)/blind-zone-laden", &measured, &d.options[0])
    };
    assert_eq!(at(350.0), Outcome::Pass);
    assert_eq!(at(350.1), Outcome::Fail);
    assert_eq!(at(0.0), Outcome::Pass);
    // Not in any default stack; a Designer adds it, and it applies.
    let mut d = design(json!({}));
    assert!(!results(&d)["rule_trace"]["rule_sets"].to_string().contains("rheinschpv"));
    add_rule_set(&mut d.options[0], &lib, "rheinschpv").unwrap();
    let r = results(&d);
    assert!(r["rule_trace"]["rule_sets"].to_string().contains("rheinschpv@1.0.0"));
    assert_eq!(rows(&r, "rheinschpv/1.07(2)/blind-zone-laden")[0]["outcome"], "pass");
}
