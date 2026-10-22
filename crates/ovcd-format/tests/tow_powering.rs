//! The towed barge and its tug (issue #361, spec 0007 "Formations"): the tug sized by the towing
//! ratios, the barge and the tug as separate hulls with their resistances added, and no coupling force
//! and no Equivalent Hull in the result.
//!
//! Label: property - the ratios are the spec's; the sum of the parts is tested at the speeds of the
//! curves in `formation.rs` and in `ovcd-core/tests/formation.rs`.

use ovcd_format::rule_set::Library;
use ovcd_format::trace::add_rule_set;
use ovcd_format::{DesignSession, Profile, RunOutcome, fill_as_new, run_option_with};
use serde_json::{Value, json};

fn raw(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The results of the convoy fixture run as a towed barge with `changes` typed.
fn results(changes: &[(&str, Value)]) -> Value {
    let mut f: Value = serde_json::from_str(&raw("pushed_convoy")).unwrap();
    let inputs = &mut f["options"][0]["inputs"];
    inputs["formation"] = json!({ "value": { "kind": "towed_barge" }, "source": "typed" });
    for (name, value) in [
        ("tug_included", json!(true)),
        ("tug_stern_form", json!("transom")),
        ("tug_propulsor_type", json!("azimuth_ducted")),
        ("tug_design_speed_kn", json!(7.0)),
    ] {
        inputs[name] = json!({ "value": value, "source": "typed" });
    }
    for (name, value) in changes {
        inputs[*name] = json!({ "value": value, "source": "typed" });
    }
    f["options"][0]
        .as_object_mut()
        .unwrap()
        .remove("results_hash");
    let (session, _) = DesignSession::open(&f.to_string(), Profile::built_in()).unwrap();
    match session.evaluate().options[0].outcome.as_ref().unwrap() {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

fn num(v: &Value, path: &[&str]) -> f64 {
    let mut cur = v;
    for key in path {
        cur = &cur[*key];
    }
    cur.as_f64().unwrap_or_else(|| panic!("{path:?} in {cur}"))
}

#[test]
fn a_towed_barge_with_its_tug_gives_a_formation_block_of_separate_hulls() {
    let r = results(&[]);
    let f = &r["formation"];
    assert_eq!(f["tug_included"], true, "{f}");
    assert!(f.get("error").is_none(), "{f}");
    assert_eq!(f["barges_towed"], 1);
    // Separate hulls: no Equivalent Hull, and a hawser has no coupling force.
    assert!(f.get("equivalent_hull").is_none());
    assert!(f.get("coupling_force_kn").is_none());
    assert_eq!(f["sizing"]["converged"], true, "{f}");
    let notes = f["notes"].to_string();
    assert!(notes.contains("separate hulls"), "{notes}");
    assert!(num(f, &["tug", "installed_power_kw"]) > 0.0);
}

#[test]
fn the_towing_tug_follows_the_towing_ratios() {
    let r = results(&[]);
    let t = &r["formation"]["tug"];
    let (l, b, d) = (
        num(t, &["length_m"]),
        num(t, &["beam_m"]),
        num(t, &["draught_m"]),
    );
    // Beam over length 1 / 2.5 and draught 0.42 of the beam, whatever the power.
    assert!((b - l / 2.5).abs() < 1e-9, "{b} {l}");
    assert!((d - 0.42 * b).abs() < 1e-9, "{d} {b}");
    // Length 23 m x (P / 3,000 kW)^0.45 at the power of the last pass: the sizing stops when the
    // power moves by less than 1 %, so the final power gives the length within 0.5 %.
    let p = num(t, &["installed_power_kw"]);
    let expected = 23.0 * libm::pow(p / 3000.0, 0.45);
    assert!((l - expected).abs() / expected < 0.01, "{l} vs {expected}");
}

#[test]
fn the_allowance_multiplies_the_resistance_so_the_power_rises() {
    let one = results(&[]);
    let two = results(&[("formation_resistance_allowance", json!(2.0))]);
    assert!(
        num(&two["formation"], &["tug", "installed_power_kw"])
            > 1.5 * num(&one["formation"], &["tug", "installed_power_kw"])
    );
}

#[test]
fn with_the_tug_left_out_there_is_no_formation_block() {
    let r = results(&[("tug_included", json!(false))]);
    assert!(r.get("formation").is_none());
}

fn rule_rows(results: &Value, id: &str) -> Vec<Value> {
    results["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["rule"] == id && r["outcome"] != "not applicable")
        .cloned()
        .collect()
}

/// The results of a towed Design of `kind` and length overall `loa`, with `es-trin-2025` selected.
fn rule_results(kind: &str, loa: f64) -> Value {
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": "bauxite", "source": "typed" },
            "hull_form": { "value": "raked_box", "source": "typed" },
            "formation": { "value": { "kind": kind }, "source": "typed" },
            "length_overall_m": { "value": loa, "source": "typed" },
            "beam_m": { "value": 11.4, "source": "typed" },
            "moulded_depth_m": { "value": 3.5, "source": "typed" }
        }}]
    });
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    let mut design = filled.design.unwrap();
    let lib = Library::built_in();
    add_rule_set(&mut design.options[0], &lib, "es-trin-2025").unwrap();
    match run_option_with(&design.options[0], &lib, &[]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_towed_craft_over_86_m_gets_the_downstream_towing_warning_and_a_shorter_one_passes() {
    let id = "es-trin/21.05(2)/no-downstream-towing-over-86m";
    let long = rule_rows(&rule_results("towed_barge", 90.0), id);
    assert_eq!(long.len(), 1, "{long:?}");
    assert_eq!(long[0]["outcome"], "warn", "{:?}", long[0]);
    let short = rule_rows(&rule_results("towed_barge", 80.0), id);
    assert_eq!(short[0]["outcome"], "pass", "{:?}", short[0]);
    // A pushed barge is not towed: the Rule does not apply.
    assert!(rule_rows(&rule_results("pushed_barge", 90.0), id).is_empty());
}
