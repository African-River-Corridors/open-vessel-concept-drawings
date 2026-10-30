//! Seam: the text and the words `run` produces, and the Option-to-`simulate` mapping for each
//! Cargo Containment (issue #64). Mutation testing left these as survivors: nothing read the
//! summary lines, the status words, the short hash in the warning, or the mapping for a sloped
//! hopper.
//!
//! Label: property. The expected text is worked out from the format rules in `run.rs` (one decimal
//! for tonnes, two for metres, three for Cb, `n/a` for a missing number), not printed by the code.

use ovcd_format::{
    CONVOY_NOT_AVAILABLE, HashStatus, Profile, RunOutcome, eval, hash_warning, new_design,
    run_option, simulate_params_of, summary_lines,
};
use serde_json::{Value, json};

fn starter() -> ovcd_format::Design {
    new_design(&Profile::built_in()).unwrap()
}

fn set(design: &mut ovcd_format::Design, name: &str, value: Value) {
    let slot = design.options[0]
        .inputs
        .iter_mut()
        .find(|(n, _)| n == name)
        .unwrap_or_else(|| panic!("no input {name}"));
    slot.1.value = value;
}

#[test]
fn each_hash_status_has_its_own_word() {
    assert_eq!(HashStatus::NotStored.as_str(), "not_stored");
    assert_eq!(HashStatus::NotChecked.as_str(), "not_checked");
    assert_eq!(HashStatus::Matches.as_str(), "matches");
    let differs = HashStatus::Differs {
        stored: "a".into(),
        computed: "b".into(),
    };
    assert_eq!(differs.as_str(), "differs");
}

/// Worked out: only a pushed convoy is "not yet available". An Option that fails for another
/// reason (here: no Cargo Containment) is a failure that says why, not a convoy.
#[test]
fn an_option_without_a_cargo_containment_fails_and_is_not_a_convoy() {
    let mut design = starter();
    set(&mut design, "cargo_containment", Value::Null);
    let o = &design.options[0];
    assert_eq!(
        simulate_params_of(o),
        Err("the Option has no Cargo Containment".to_string())
    );
    assert_eq!(
        run_option(o),
        RunOutcome::Failed("the Option has no Cargo Containment".to_string())
    );
    assert!(run_option(o).is_failure());
    assert_ne!(
        run_option(o),
        RunOutcome::NotAvailable(CONVOY_NOT_AVAILABLE.to_string())
    );
}

/// Worked out: each Cargo Containment maps to its vessel, and a sloped hopper is computed as the
/// sloped-hopper barge `simulate` knows (not the box barge).
#[test]
fn a_sloped_hopper_hold_runs_as_the_sloped_hopper_barge() {
    for (containment, vessel) in [
        ("box_hold", "box_barge"),
        ("sloped_hopper_hold", "hopper_barge"),
    ] {
        let mut design = starter();
        set(&mut design, "cargo_containment", json!(containment));
        let params = simulate_params_of(&design.options[0]).unwrap();
        assert_eq!(params["vessel"], vessel, "{containment}");
    }
    let mut design = starter();
    set(
        &mut design,
        "cargo_containment",
        json!("sloped_hopper_hold"),
    );
    let RunOutcome::Results { results, .. } = run_option(&design.options[0]) else {
        panic!("a sloped hopper runs");
    };
    assert_eq!(results["vessel"], "Sloped-hopper dry bulk barge");
    // The same answer `simulate` gives for the params the Option maps to (`run` adds the budget).
    let params = simulate_params_of(&design.options[0]).unwrap();
    let case = json!({ "fn": "simulate", "args": { "params": params } });
    let mut without_cost = results.clone();
    without_cost.as_object_mut().unwrap().remove("cost");
    // And the Rule Trace (issue #269), which `simulate` does not give.
    without_cost.as_object_mut().unwrap().remove("rule_trace");
    // And the calibration label (issue #434), which `simulate` does not give either.
    without_cost.as_object_mut().unwrap().remove("calibration");
    // And the pusher of the Formation (issue #359): `simulate` gives the barge alone, and the notes
    // of the pusher are not in its `text_ids`.
    without_cost.as_object_mut().unwrap().remove("formation");
    // And the tug's GA block (issue #360), which comes with the pusher.
    if let Some(d) = without_cost["drawing"].as_object_mut() {
        d.remove("tug");
    }
    if let Some(ids) = without_cost["text_ids"].as_object_mut() {
        ids.retain(|_, v| {
            !v["id"]
                .as_str()
                .is_some_and(|id| id.contains("-formation-"))
        });
    }
    assert_eq!(without_cost, eval("simulate", &case).unwrap()["ok"]);
    // And not the box barge's answer: the hold shapes differ.
    let box_hold = run_option(&starter().options[0]);
    let RunOutcome::Results { results: boxed, .. } = box_hold else {
        panic!("a box hold runs");
    };
    assert_ne!(results["results"]["cargo_t"], boxed["results"]["cargo_t"]);
    assert!(results["geometry"]["hold_bottom_width_m"].as_f64().unwrap() > 0.0);
}

#[test]
fn a_gas_containment_maps_to_the_gas_barge() {
    let mut design = starter();
    set(&mut design, "cargo_containment", json!("type_c_tanks"));
    set(&mut design, "cargo", json!("LPG_propane"));
    let params = simulate_params_of(&design.options[0]).unwrap();
    assert_eq!(params["vessel"], "gas_barge");
}

/// The warning gives the first 12 characters of each hash; a shorter text is given whole.
#[test]
fn the_hash_warning_shortens_each_hash_to_twelve_characters() {
    let stored = "0123456789abcdef0123";
    let computed = "fedcba9876543210fedc";
    assert_eq!(
        hash_warning("Opt", "0.2.2", stored, computed),
        "Option 'Opt': results differ from the hash saved in the file (saved with engine version \
         0.2.2; stored hash 0123456789ab, now fedcba987654)"
    );
    assert_eq!(
        hash_warning("Opt", "0.2.2", "abc", "123456789012"),
        "Option 'Opt': results differ from the hash saved in the file (saved with engine version \
         0.2.2; stored hash abc, now 123456789012)"
    );
}

fn results_object() -> Value {
    json!({
        "cargo": "Bauxite",
        "vessel": "Box barge",
        "dims": { "loa_m": 76.5, "beam_m": 11.4, "depth_m": 3.5, "cb": 0.928 },
        "results": {
            "cargo_t": 2256.166305, "governs": "hold volume", "lightship_t": 333.62185,
            "displacement_t": 2589.78, "draught_m": 3.2, "draught_max_m": 3.7, "freeboard_m": 0.3,
            "air_draft_loaded_m": 4.5, "air_draft_light_m": 6.254
        },
        "checks": [
            { "name": "a", "ok": true, "message": "x" },
            { "name": "b", "ok": true, "message": "y" }
        ],
        "warnings": []
    })
}

/// Worked out: the first line names the Option, cargo, vessel and dimensions (1 decimal, Cb to 3);
/// then cargo to 1 decimal, lightship and displacement to 1, draughts and freeboard to 2, and the
/// two air drafts to 2.
#[test]
fn the_summary_lists_the_main_numbers_with_their_decimals() {
    assert_eq!(
        summary_lines("Option 1", &results_object()),
        [
            "Option 'Option 1': Bauxite in a Box barge (76.5 x 11.4 x 3.5 m, Cb 0.928)",
            "  Cargo: 2256.2 t (hold volume)",
            "  Lightship 333.6 t, displacement 2589.8 t",
            "  Draught 3.20 m (cap 3.70 m), freeboard 0.30 m",
            "  Air draft 4.50 m loaded, 6.25 m light",
            "  Checks: all 2 pass",
        ]
    );
}

#[test]
fn the_summary_names_the_failed_checks_and_lists_the_warnings() {
    let mut r = results_object();
    r["checks"] = json!([
        { "name": "a", "ok": true, "message": "x" },
        { "name": "b", "ok": false, "message": "y" },
        { "name": "c", "ok": false, "message": "z" },
        { "name": "d", "ok": null, "message": "n" }
    ]);
    r["warnings"] = json!(["first", 7]);
    let lines = summary_lines("O", &r);
    assert_eq!(lines[5], "  Checks: 2 of 4 fail: b; c");
    assert_eq!(lines[6], "  Warning: first");
    assert_eq!(lines[7], "  Warning: ?");
    assert_eq!(lines.len(), 8);
}

/// Issue #419: a failed check of severity "warning" reads as a warning, apart from the failures; a
/// check with no severity is an error.
#[test]
fn a_failed_warning_check_reads_as_a_warning_not_a_failure() {
    let mut r = results_object();
    r["checks"] = json!([
        { "name": "gates", "ok": false, "message": "x", "severity": "error" },
        { "name": "advice", "ok": false, "message": "y", "severity": "warning" },
        { "name": "plain", "ok": false, "message": "z" },
        { "name": "fine", "ok": true, "message": "OK", "severity": "warning" }
    ]);
    let lines = summary_lines("O", &r);
    assert_eq!(lines[5], "  Checks: 2 of 3 fail: gates; plain");
    assert_eq!(lines[6], "  Warning checks: 1 not met: advice");
}

/// A result with nothing in it still prints, with `?` for missing text and `n/a` for a missing number.
#[test]
fn the_summary_of_an_empty_result_uses_question_marks_and_not_available() {
    assert_eq!(
        summary_lines("E", &json!({})),
        [
            "Option 'E': ? in a ? (n/a x n/a x n/a m, Cb n/a)",
            "  Cargo: n/a t (?)",
            "  Lightship n/a t, displacement n/a t",
            "  Draught n/a m (cap n/a m), freeboard n/a m",
            "  Air draft n/a m loaded, n/a m light",
            "  Checks: all 0 pass",
        ]
    );
}
