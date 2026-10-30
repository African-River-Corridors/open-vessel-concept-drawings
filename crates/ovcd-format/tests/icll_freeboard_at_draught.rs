//! Seam: the ICLL tabular freeboard Rules read the Freeboard of the Design at its laden Draught
//! (issue #659), not the Minimum Freeboard input, which is the ES-TRIN safety clearance.
//!
//! Label: sourced - Protocol of 1988 to the Load Lines Convention as revised by MSC.143(77),
//! regulation 3(8) (the freeboard assigned is measured down from the deck line to the load line)
//! and regulation 28(1), table 28.1 (MSC.143-77.pdf, annex pp. 42-44): 1,803 mm at 140 m.

use ovcd_format::rule_set::Library;
use ovcd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

fn results_of(typed: Value) -> Value {
    let mut inputs = json!({});
    for (k, v) in typed.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    match run_option(&filled.design.unwrap().options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

/// The rows of the ICLL tabular freeboard Rules that apply, as (rule, outcome, value, limit).
fn icll_rows(results: &Value) -> Vec<(String, String, f64, f64)> {
    results["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| {
            r["rule"].as_str().unwrap().starts_with("icll/28(1)/")
                && r["outcome"] != "not applicable"
        })
        .map(|r| {
            (
                r["rule"].as_str().unwrap().to_string(),
                r["outcome"].as_str().unwrap().to_string(),
                r["value"].as_f64().unwrap_or(f64::NAN),
                r["limit"].as_f64().unwrap_or(f64::NAN),
            )
        })
        .collect()
}

/// The 140 x 29 x 12 m self-propelled LNG barge of the review run, coastal, every other input at
/// its default (so the Minimum Freeboard is the 0.3 m safety clearance).
fn gas_barge() -> Value {
    json!({
        "cargo": "LNG", "hull_form": "raked_box", "formation": { "kind": "self_propelled" },
        "length_overall_m": 140.0, "beam_m": 29.0, "moulded_depth_m": 12.0,
        "cargo_containment": "type_c_tanks", "max_tank_od_m": 12.0, "tanks_in_line": 2.0,
        "aft_end_space_m": 0.6, "fwd_end_space_m": 0.5, "max_speed_kn": 12.0,
        "service_area": "coastal-and-river"
    })
}

fn infeasible(results: &Value) -> bool {
    results["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["name"] == "Feasible arrangement" && c["ok"] == false)
}

#[test]
fn a_default_coastal_gas_barge_is_checked_on_its_laden_freeboard_and_fits() {
    let r = results_of(gas_barge());
    let freeboard = r["results"]["freeboard_m"].as_f64().unwrap();
    // The laden draught of the review run is about 4 m: about 8 m of freeboard.
    assert!(freeboard > 7.0, "{freeboard}");
    let rows = icll_rows(&r);
    assert_eq!(rows.len(), 1, "{rows:?}");
    let (rule, outcome, value, limit) = &rows[0];
    assert_eq!(
        rule, "icll/28(1)/type-a-tabular-freeboard-laden",
        "{rows:?}"
    );
    assert_eq!(outcome, "pass", "{rows:?}");
    // Table 28.1 at 140 m: 1,803 mm.
    assert!((limit - 1.803).abs() < 1e-9, "{limit}");
    // The value is the Freeboard at the laden Draught, not the 0.3 m Minimum Freeboard.
    assert!((value - freeboard).abs() < 1e-6, "{value} vs {freeboard}");
    assert!(!infeasible(&r), "{}", r["checks"]);
}

#[test]
fn a_hull_loaded_to_the_safety_clearance_still_fails_for_the_right_reason() {
    // Dry bulk at 76.5 m, draught-bound: the hull loads to its depth less 0.3 m, so its laden
    // Freeboard is 0.3 m, below the table's 0.793 m. The verdict is the one #428 intended, now
    // read from the Freeboard the Design has.
    let r = results_of(json!({
        "cargo": "bauxite", "hull_form": "raked_box", "formation": { "kind": "self_propelled" },
        "length_overall_m": 76.5, "beam_m": 11.4, "moulded_depth_m": 3.5,
        "service_area": "coastal-and-river"
    }));
    let freeboard = r["results"]["freeboard_m"].as_f64().unwrap();
    assert!((freeboard - 0.3).abs() < 1e-6, "{freeboard}");
    let rows = icll_rows(&r);
    assert_eq!(rows.len(), 1, "{rows:?}");
    let (rule, outcome, value, limit) = &rows[0];
    assert_eq!(rule, "icll/28(1)/type-a-tabular-freeboard-laden");
    assert_eq!(outcome, "fail");
    assert!((value - freeboard).abs() < 1e-6, "{value}");
    assert!((limit - 0.793).abs() < 1e-9, "{limit}");
}

#[test]
fn a_barge_with_no_propulsion_gets_the_laden_freeboard_as_a_warning() {
    let mut barge = gas_barge();
    barge["formation"] = json!({ "kind": "pushed_barge" });
    let r = results_of(barge);
    let rows = icll_rows(&r);
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0].0, "icll/28(1)/type-a-tabular-freeboard-laden-barge");
    assert_eq!(rows[0].1, "pass");
    let freeboard = r["results"]["freeboard_m"].as_f64().unwrap();
    assert!((rows[0].2 - freeboard).abs() < 1e-6);
}

#[test]
fn version_1_0_0_is_kept_unchanged_for_files_saved_before() {
    // A file that stores icll-freeboard 1.0.0 keeps its verdicts (ADR 0013): 2.0.0 is a new
    // version with new rule ids, because the Measure changed.
    let lib = Library::built_in();
    let old = lib.find("icll-freeboard", "1.0.0").unwrap();
    let rule = old.rule("icll/28(1)/type-a-tabular-freeboard").unwrap();
    assert_eq!(rule.measure.id, "input.min_freeboard_m");
    let new = lib.latest("icll-freeboard").unwrap();
    assert_eq!(new.version, "2.0.0");
    assert!(new.rule("icll/28(1)/type-a-tabular-freeboard").is_none());
    for id in [
        "icll/28(1)/type-a-tabular-freeboard-laden",
        "icll/28(1)/type-a-tabular-freeboard-laden-barge",
    ] {
        assert_eq!(new.rule(id).unwrap().measure.id, "freeboard.laden", "{id}");
    }
}
