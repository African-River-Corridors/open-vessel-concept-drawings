//! Seam: the typed openings and their Rules (issue #417, option 1 of the maintainer's decision, a
//! trial): IGC 3.2.4.1 in `igc-2g`, and the inland class clauses 3.3.7, 3.3.8, 3.3.11 and 3.4.1 in
//! `lr-inland-gas` 2.0.0, which also drops the type C cofferdam clause 3.6.13 from the gas stack
//! (#280). The engine does not place openings: the Designer types where they are.
//!
//! Label: sourced - IGC Code, resolution MSC.370(93), 3.1.1, 3.1.2, 3.2.1 and 3.2.4.1 (annex pp.
//! 32-33); Lloyd's Register inland waterways rules Pt 4 Ch 4 3.3.7, 3.3.8, 3.3.11, 3.4.1, 3.5.3 and
//! 3.6.13 (July 2022, read in the unofficial copy of the section and the November 2015 edition)

use ovcd_core::gas_separation::{GasSeparationInputs, compute};
use ovcd_core::measured::Measured;
use ovcd_core::rule_trace::formula_value;
use ovcd_core::rules::{Compare, Limit};
use ovcd_format::rule_set::Library;
use ovcd_format::trace::trace_of;
use ovcd_format::{Profile, RunOutcome, check, fill_as_new, input_schema, run_option};
use serde_json::{Value, json};

fn new_gas_design(cargo: &str, extra: Value) -> ovcd_format::Design {
    let mut inputs = json!({
        "cargo": { "value": cargo, "source": "typed" },
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

fn results_of(design: &ovcd_format::Design) -> Value {
    match run_option(&design.options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

/// The outcomes of one Rule in a result's trace, `not applicable` rows left out.
fn outcomes(results: &Value, id: &str) -> Vec<String> {
    results["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["rule"] == id && r["outcome"] != "not applicable")
        .map(|r| r["outcome"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn the_five_inputs_are_in_the_openings_group_for_type_c_tanks_with_estimate_defaults() {
    let p = Profile::built_in();
    let schema = input_schema(&p);
    let list = schema["inputs"].as_array().unwrap();
    assert!(
        schema["groups"]
            .as_array()
            .unwrap()
            .contains(&json!("Openings"))
    );
    for (name, default) in [
        ("deckhouse_front_opening", json!(false)),
        ("deckhouse_side_opening_distance_m", json!(5.0)),
        ("entrance_distance_m", json!(2.0)),
        ("engine_room_vent_inlet_distance_m", json!(2.0)),
        ("funnel_outlet_distance_m", json!(2.0)),
    ] {
        let e = list.iter().find(|i| i["name"] == name).unwrap();
        assert_eq!(e["group"], "Openings", "{name}");
        assert_eq!(e["applies_to"], json!(["type_c_tanks"]), "{name}");
        assert_eq!(e["default"], default, "{name}");
        assert_eq!(e["default_status"], "estimate", "{name}");
    }
}

#[test]
fn a_file_saved_before_reads_the_smallest_distances_the_rules_allow_with_one_note() {
    let text = std::fs::read_to_string(format!(
        "{}/tests/legacy/v2_gas_lpg_before_313.vdesign",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let checked = check(&text, &Profile::built_in());
    assert_eq!(checked.report.errors, Vec::<String>::new());
    let design = checked.design.unwrap();
    let o = &design.options[0];
    for (name, want) in [
        ("deckhouse_front_opening", json!(false)),
        ("deckhouse_side_opening_distance_m", json!(5.0)),
        ("entrance_distance_m", json!(2.0)),
        ("engine_room_vent_inlet_distance_m", json!(2.0)),
        ("funnel_outlet_distance_m", json!(2.0)),
    ] {
        let i = o.input(name).unwrap_or_else(|| panic!("{name}"));
        assert_eq!(
            (i.value.clone(), i.source),
            (want, ovcd_format::Source::Default),
            "{name}"
        );
    }
    let notes: Vec<&String> = checked
        .report
        .warnings
        .iter()
        .filter(|w| w.contains("`funnel_outlet_distance_m`"))
        .collect();
    assert_eq!(notes.len(), 1, "{:?}", checked.report.warnings);
    assert!(notes[0].contains("results are unchanged"), "{}", notes[0]);
}

#[test]
fn the_igc_side_opening_limit_is_4_percent_of_l_between_3_and_5_m() {
    // IGC 3.2.4.1, by hand: 4 % of L, not less than 3 m, not more than 5 m.
    let lib = Library::built_in();
    let set = lib.find("igc-2g", "1.1.0").unwrap();
    let rule = set.rule("igc/3.2.4.1/side-opening-distance").unwrap();
    assert_eq!(rule.compare, Some(Compare::Ge));
    let Some(Limit::Formula {
        shape,
        coefficients,
        ..
    }) = &rule.limit
    else {
        panic!("a formula");
    };
    let limit = |l: f64| formula_value(shape, coefficients, |_| Ok(l)).unwrap();
    assert_eq!(limit(60.0), 3.0); // 2.4, raised to 3
    assert_eq!(limit(75.0), 3.0); // 4 % is exactly 3.0
    assert_eq!(limit(75.0001), 0.04 * 75.0001); // above the floor
    assert_eq!(limit(110.0), 0.04 * 110.0); // 4.4
    assert_eq!(limit(125.0), 5.0); // 4 % is exactly 5.0
    assert_eq!(limit(200.0), 5.0); // 8, capped at 5
}

#[test]
fn the_inland_rules_are_the_two_metre_clauses_and_the_cofferdam_clause_is_gone() {
    let lib = Library::built_in();
    let (old, new) = (
        lib.find("lr-inland-gas", "1.1.0").unwrap(),
        lib.find("lr-inland-gas", "2.0.0").unwrap(),
    );
    // 1.1.0 is kept as it was; 2.0.0 drops 3.6.13 (type C liquid cargo, not type G) and keeps the rest.
    assert!(old.rule("lr-inland/3.6.13/cofferdam").is_some());
    assert!(new.rule("lr-inland/3.6.13/cofferdam").is_none());
    for r in &old.rules {
        if r.id != "lr-inland/3.6.13/cofferdam" {
            assert!(new.rule(&r.id).is_some(), "{} is kept", r.id);
        }
    }
    for (id, measure) in [
        (
            "lr-inland/3.3.7/entrance-setback",
            "input.entrance_distance_m",
        ),
        (
            "lr-inland/3.3.11/opening-setback",
            "input.entrance_distance_m",
        ),
        (
            "lr-inland/3.3.8/vent-inlet-setback",
            "input.engine_room_vent_inlet_distance_m",
        ),
        (
            "lr-inland/3.4.1/funnel-setback",
            "input.funnel_outlet_distance_m",
        ),
    ] {
        let r = new.rule(id).unwrap();
        assert_eq!(r.measure.id, measure, "{id}");
        assert_eq!(r.compare, Some(Compare::Ge), "{id}");
        let Some(Limit::Constant { value, .. }) = &r.limit else {
            panic!("{id}: a constant");
        };
        assert_eq!(*value, 2.0, "{id}");
        assert!(
            r.verified.is_none(),
            "{id}: unverified until a second reader"
        );
    }
}

#[test]
fn a_new_gas_option_meets_the_opening_rules_and_names_the_ones_the_model_cannot_check() {
    for cargo in ["LNG", "LPG_propane"] {
        let results = results_of(&new_gas_design(cargo, json!({})));
        let sets = results["rule_trace"]["rule_sets"].to_string();
        assert!(
            sets.contains("lr-inland-gas@2.0.0") && sets.contains("igc-2g@1.1.0"),
            "{sets}"
        );
        for id in [
            "igc/3.2.4.1/front-openings",
            "igc/3.2.4.1/side-opening-distance",
            "lr-inland/3.3.7/entrance-setback",
            "lr-inland/3.3.8/vent-inlet-setback",
            "lr-inland/3.3.11/opening-setback",
            "lr-inland/3.4.1/funnel-setback",
        ] {
            let o = outcomes(&results, id);
            assert!(
                !o.is_empty() && o.iter().all(|x| x == "pass"),
                "{cargo} {id}: {o:?}"
            );
        }
        for id in [
            "igc/3.1.1/hold-space-segregation",
            "igc/3.1.2/hold-space-segregation-means",
            "igc/3.2.1/no-accommodation-in-cargo-area",
        ] {
            assert_eq!(outcomes(&results, id), ["not evaluated"], "{cargo} {id}");
        }
        // The cofferdam clause is no longer in the stack.
        assert!(outcomes(&results, "lr-inland/3.6.13/cofferdam").is_empty());
    }
}

#[test]
fn a_typed_opening_that_breaks_a_rule_fails_that_rule_only() {
    for (input, value, rule) in [
        (
            "deckhouse_front_opening",
            json!(true),
            "igc/3.2.4.1/front-openings",
        ),
        (
            "deckhouse_side_opening_distance_m",
            json!(4.0),
            "igc/3.2.4.1/side-opening-distance",
        ),
        (
            "entrance_distance_m",
            json!(1.5),
            "lr-inland/3.3.11/opening-setback",
        ),
        (
            "entrance_distance_m",
            json!(1.5),
            "lr-inland/3.3.7/entrance-setback",
        ),
        (
            "engine_room_vent_inlet_distance_m",
            json!(1.99),
            "lr-inland/3.3.8/vent-inlet-setback",
        ),
        (
            "funnel_outlet_distance_m",
            json!(0.5),
            "lr-inland/3.4.1/funnel-setback",
        ),
    ] {
        let design = new_gas_design("LNG", json!({ input: value }));
        let results = results_of(&design);
        assert!(
            outcomes(&results, rule).iter().all(|o| o == "fail"),
            "{input}: {rule}: {:?}",
            outcomes(&results, rule)
        );
        assert!(!outcomes(&results, rule).is_empty(), "{rule}");
    }
}

#[test]
fn a_deep_cofferdam_with_a_door_on_its_aft_bulkhead_fails_the_opening_rule() {
    // Issue #381: the aft separation is max(cofferdam, opening setback), so a deep cofferdam gives
    // 3 m and the old Rule passes, though a door on the cofferdam's aft bulkhead is not 2 m from the
    // cargo area. The typed distance of the door says so.
    let design = new_gas_design("LNG", json!({ "entrance_distance_m": 0.5 }));
    let deep = compute(&GasSeparationInputs {
        cofferdam_m: 3.0,
        ..GasSeparationInputs::default()
    })
    .unwrap();
    assert_eq!(deep.aft_separation_m, 3.0);
    let measured = Measured {
        separation: Some((deep, false)),
        ..Measured::default()
    };
    let lib = Library::built_in();
    let selection = lib
        .select(&[("lr-inland-gas", "2.0.0")])
        .expect("the set is selected");
    let trace = trace_of(&selection, &design.options[0], &measured);
    let outcome = |id: &str| {
        trace
            .rows
            .iter()
            .find(|r| r.rule.as_deref() == Some(id))
            .unwrap_or_else(|| panic!("{id}"))
            .outcome
    };
    use ovcd_core::rule_trace::Outcome;
    assert_eq!(outcome("lr-inland/3.3.7/opening-setback"), Outcome::Pass);
    assert_eq!(outcome("lr-inland/3.3.11/opening-setback"), Outcome::Fail);
    assert_eq!(outcome("lr-inland/3.3.7/entrance-setback"), Outcome::Fail);
}
