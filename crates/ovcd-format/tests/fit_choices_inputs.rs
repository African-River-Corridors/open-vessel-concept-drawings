//! Seam: the gas arrangement inputs and explanations of issue #675 through a Design: tanks across,
//! the least side space, the list of arrangements tried, the warning for no wing tank, and the
//! wheelhouse lift default with its cited range and datum.
//!
//! Label: sourced - BUTER Hebetechnik, lifting-shaft wheelhouse lift, product range (strokes 2.5 to
//! 8.9 m), read 2026-10-29: the soft range of the lift. The other tests state rules: an input
//! reaches the search, a file saved before keeps its results, the list is outside the results hash.

use ovcd_format::vdesign::{INPUTS, LEGACY_FIXED_INPUTS, LEGACY_FIXED_OTHER};
use ovcd_format::{
    Profile, RunOutcome, UNHASHED_KEYS, fill, fill_as_new, input_schema, results_hash, run_option,
};
use serde_json::{Value, json};

/// The hull of the issue: a self-propelled LNG barge, 140 x 29 x 9 m, coastal and river, at least
/// 4 m of freeboard, every other input from the Defaults Profile.
fn design(extra: Value) -> Value {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 140.0, "source": "typed" },
        "beam_m": { "value": 29.0, "source": "typed" },
        "moulded_depth_m": { "value": 9.0, "source": "typed" },
        "service_area": { "value": "coastal-and-river", "source": "typed" },
        "min_freeboard_m": { "value": 4.0, "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    })
}

fn run(file: &Value) -> Value {
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    match run_option(&filled.design.unwrap().options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn no_wing_warning(r: &Value) -> bool {
    r["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|w| w.as_str().unwrap().starts_with("No wing ballast tank"))
}

#[test]
fn the_two_inputs_are_gas_tank_inputs_that_leave_a_file_saved_before_as_it_was() {
    for name in ["tanks_across", "min_side_space_m"] {
        let spec = INPUTS.iter().find(|s| s.name == name).unwrap();
        assert_eq!(spec.group.label(), "Gas tanks", "{name}");
        assert_eq!(spec.applies_to.containments(), ["type_c_tanks"], "{name}");
    }
    // A file saved before: tanks across tried 1, 2 and 3 (none), and no least side space (0).
    let across = LEGACY_FIXED_OTHER
        .iter()
        .find(|l| l.name == "tanks_across")
        .unwrap();
    assert_eq!(across.value, "null");
    let side = LEGACY_FIXED_INPUTS
        .iter()
        .find(|l| l.name == "min_side_space_m")
        .unwrap();
    assert_eq!(side.value, 0.0);
    // A new Design gets the same: the search as before.
    let p = Profile::built_in();
    assert_eq!(p.default("tanks_across"), Some(&Value::Null));
    assert_eq!(
        p.default("min_side_space_m").and_then(Value::as_f64),
        Some(0.0)
    );
}

#[test]
fn a_fixed_number_of_tanks_across_reaches_the_search() {
    let auto = run(&design(json!({})));
    assert_eq!(
        auto["fit_tried"].as_array().unwrap().len(),
        3,
        "1, 2 and 3 across tried"
    );
    for n in [1, 2, 3] {
        let r = run(&design(json!({ "tanks_across": n as f64 })));
        assert_eq!(r["arrangement"]["n_wide"], n);
        let tried = r["fit_tried"].as_array().unwrap();
        assert!(tried.iter().all(|t| t["n_wide"] == n), "{n}: {tried:?}");
        // When it cannot pass the result says so, and the row says why.
        if r["arrangement"]["feasible"] == false {
            assert!(
                r["checks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|c| c["name"] == "Feasible arrangement")
            );
            assert_eq!(tried[0]["outcome"], "fails");
            assert!(!tried[0]["reasons"].as_array().unwrap().is_empty());
        }
    }
}

#[test]
fn tanks_across_is_a_whole_number_of_at_least_one() {
    for (v, why) in [(json!(2.5), "whole number"), (json!(0), "must be >= 1")] {
        let mut f = design(json!({}));
        f["options"][0]["inputs"]["tanks_across"] = json!({ "value": v, "source": "typed" });
        let filled = fill_as_new(&f.to_string(), &Profile::built_in());
        assert!(
            filled.report.errors.iter().any(|e| e.contains(why)),
            "{why}: {:?}",
            filled.report.errors
        );
    }
}

#[test]
fn the_wing_tank_warning_shows_with_no_wing_tank_and_a_least_side_space_clears_it() {
    // The issue's case: three across on the IGC minimum, which is under the wing tank gap.
    let plain = run(&design(json!({})));
    let side = plain["arrangement"]["side_clearance_m"].as_f64().unwrap();
    let chosen: Vec<&Value> = plain["fit_tried"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|t| t["outcome"] == "chosen")
        .collect();
    assert_eq!(chosen.len(), 1);
    assert_eq!(chosen[0]["wing_tank_width_m"].as_f64(), Some(0.0));
    assert!(side <= 0.6, "side {side} m");
    assert!(no_wing_warning(&plain), "{:?}", plain["warnings"]);

    let min = 2.0;
    let wide = run(&design(json!({ "min_side_space_m": min })));
    let side = wide["arrangement"]["side_clearance_m"].as_f64().unwrap();
    assert!(side + 1e-9 >= min, "side {side} m");
    assert!(!no_wing_warning(&wide), "{:?}", wide["warnings"]);
    for t in wide["fit_tried"].as_array().unwrap() {
        if t["outcome"] != "not_built" {
            assert!(t["side_clearance_m"].as_f64().unwrap() + 1e-9 >= min, "{t}");
            assert!(t["wing_tank_width_m"].as_f64().unwrap() > 0.0, "{t}");
        }
    }
}

#[test]
fn each_arrangement_lost_for_a_reason_and_the_list_is_not_hashed() {
    let r = run(&design(json!({})));
    assert!(UNHASHED_KEYS.contains(&"fit_tried"));
    let tried = r["fit_tried"].as_array().unwrap();
    for t in tried {
        match t["outcome"].as_str().unwrap() {
            "chosen" | "less_cargo" => assert!(t["reasons"].as_array().unwrap().is_empty(), "{t}"),
            "fails" | "not_built" => assert!(!t["reasons"].as_array().unwrap().is_empty(), "{t}"),
            other => panic!("outcome {other}"),
        }
    }
    // On this hull two across is turned down on the blind distance, with the lift needed.
    let two = tried.iter().find(|t| t["n_wide"] == 2).unwrap();
    assert_eq!(two["outcome"], "fails");
    let message = two["reasons"][0]["message"].as_str().unwrap();
    assert!(message.contains("lift needed"), "{message}");
    // The list moves no hash: the results without it hash the same.
    let mut without = r.clone();
    without.as_object_mut().unwrap().remove("fit_tried");
    assert_eq!(results_hash(&r), results_hash(&without));
    // Its English has its Message Ids, for the app.
    assert!(
        r["text_ids"].as_object().unwrap().contains_key(message),
        "{message}"
    );
}

#[test]
fn the_wheelhouse_lift_is_an_estimate_in_the_published_range_and_names_its_datum() {
    let p = Profile::built_in();
    // The default stays the original engine's 6.0 m (no source gives one value; #675 asks the
    // maintainer), inside the published range of strokes, which is its soft range.
    assert_eq!(
        p.default("wheelhouse_max_lift_m").and_then(Value::as_f64),
        Some(6.0)
    );
    let schema = input_schema(&p);
    let e = schema["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["name"] == "wheelhouse_max_lift_m")
        .unwrap()
        .clone();
    assert_eq!(e["soft"]["min"].as_f64(), Some(2.5), "{e}");
    assert_eq!(e["soft"]["max"].as_f64(), Some(8.9), "{e}");
    let basis = e["basis"].as_str().unwrap_or_default().to_string();
    assert!(basis.contains("BUTER Hebetechnik"), "{basis}");
    assert!(basis.contains("deck at moulded depth"), "{basis}");
    assert!(basis.contains("light air draft"), "{basis}");
    // A file saved before keeps the 6.0 m the engine used: its results do not move.
    let legacy = LEGACY_FIXED_INPUTS
        .iter()
        .find(|l| l.name == "wheelhouse_max_lift_m")
        .unwrap();
    assert_eq!(legacy.value, 6.0);
    // `fill` (not --as-new) of a file that leaves it out takes the legacy value.
    let mut f = design(json!({}));
    f["options"][0]["inputs"]
        .as_object_mut()
        .unwrap()
        .remove("wheelhouse_max_lift_m");
    let filled = fill(&f.to_string(), &Profile::built_in());
    let o = &filled.design.unwrap().options[0];
    assert_eq!(
        o.input("wheelhouse_max_lift_m").unwrap().value.as_f64(),
        Some(6.0)
    );
}
