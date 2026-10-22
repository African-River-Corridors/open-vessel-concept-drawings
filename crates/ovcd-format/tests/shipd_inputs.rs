//! Seam: the `shipd` Hull Form in a Design (issues #452 and #453): a new Option of it runs, its parameters
//! are checked as the engine reads them, and its L, B and D are the Option's.
//!
//! The tests run in the legacy loading conditions to keep CI fast: a ShipD Design in the default IS
//! Code conditions takes about 21 s a run (#567). A Design keeps the IS Code default.
//!
//! Label: property. Each test states a rule; no number here is a printed result.

use ovcd_format::rule_set::Library;
use ovcd_format::{Profile, RunOutcome, check, fill_as_new, run_option_with};
use serde_json::{Value, json};

fn design(cargo: &str, formation: &str, extra: Value) -> String {
    let mut inputs = json!({
        "cargo": { "value": cargo, "source": "typed" },
        "hull_form": { "value": "shipd", "source": "typed" },
        "formation": { "value": { "kind": formation }, "source": "typed" },
        "length_overall_m": { "value": 80.0, "source": "typed" },
        "beam_m": { "value": 14.0, "source": "typed" },
        "moulded_depth_m": { "value": 4.5, "source": "typed" },
        "loading_conditions": { "value": "legacy", "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    })
    .to_string()
}

fn run(text: &str) -> RunOutcome {
    let d = fill_as_new(text, &Profile::built_in()).design.unwrap();
    run_option_with(&d.options[0], &Library::built_in(), &[])
}

#[test]
fn a_new_shipd_option_takes_the_profile_hull_and_runs() {
    let text = design("bauxite", "pushed_barge", json!({}));
    let filled = fill_as_new(&text, &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    let d = filled.design.unwrap();
    let p = d.options[0].input("shipd_parameters").unwrap();
    assert_eq!(p.value.as_object().unwrap().len(), 42);
    match run(&text) {
        RunOutcome::Results { results, .. } => {
            let cb = results["hydrostatics"]["cb"].as_f64().unwrap();
            assert!(cb > 0.3 && cb < 0.95, "a shaped hull: Cb {cb}");
            assert!(results["hydrostatics"]["cm"].as_f64().unwrap() < 1.0);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_failed_constraint_is_named() {
    let filled = fill_as_new(
        &design("bauxite", "pushed_barge", json!({})),
        &Profile::built_in(),
    );
    let mut d: Value = serde_json::from_str(&filled.design.unwrap().to_text()).unwrap();
    let params = &mut d["options"][0]["inputs"]["shipd_parameters"]["value"];
    params["Lb"] = json!(0.6);
    params["Ls"] = json!(0.6);
    let checked = check(&d.to_string(), &Profile::built_in());
    let errors = checked.report.errors.join("\n");
    assert!(
        errors.contains("fails ShipD constraint 0 (bow and stern tapers within LOA)"),
        "{errors}"
    );
}

/// The parameters of a hull with both bulbs, from the committed benchmark sample (Set 1, dataset
/// row 763), as the `shipd_parameters` object, and the Option's dimensions at 80 m.
fn bulb_hull() -> (Value, [f64; 3]) {
    let sample = include_str!("../../../calibration/shipd/sample.csv");
    let names: Vec<&str> = sample.lines().next().unwrap().split(',').skip(2).collect();
    let line = sample
        .lines()
        .find(|l| l.starts_with("Constrained_Randomized_Set_1,763,"))
        .unwrap();
    let v: Vec<f64> = line
        .split(',')
        .skip(2)
        .map(|c| c.parse().unwrap())
        .collect();
    let mut o = serde_json::Map::new();
    for (n, x) in names.iter().zip(&v) {
        if !["LOA", "Bd", "Dd"].contains(n) {
            o.insert((*n).to_string(), json!(x));
        }
    }
    (Value::Object(o), [80.0, v[3] * 80.0, v[4] * 80.0])
}

#[test]
fn a_shipd_option_with_a_bulb_runs_and_is_checked_as_the_engine_reads_it() {
    let (params, [l, b, d]) = bulb_hull();
    let text = design(
        "bauxite",
        "pushed_barge",
        json!({ "length_overall_m": l, "beam_m": b, "moulded_depth_m": d, "shipd_parameters": params }),
    );
    let filled = fill_as_new(&text, &Profile::built_in())
        .design
        .unwrap()
        .to_text();
    let checked = check(&filled, &Profile::built_in());
    assert_eq!(checked.report.errors, Vec::<String>::new());
    match run(&text) {
        RunOutcome::Results { results, .. } => {
            let cb = results["hydrostatics"]["cb"].as_f64().unwrap();
            assert!(cb > 0.2 && cb < 0.95, "a shaped hull with bulbs: Cb {cb}");
        }
        other => panic!("{other:?}"),
    }
    // A bulb that breaks a constraint is named by it: a bow bulb wider than the hull (index 38).
    let mut bad = params_of(&filled);
    bad["Bbb"] = json!(1.05);
    let broken = text_with(&filled, &bad);
    let errors = check(&broken, &Profile::built_in())
        .report
        .errors
        .join("\n");
    assert!(errors.contains("fails ShipD constraint 38"), "{errors}");
}

fn params_of(text: &str) -> Value {
    let d: Value = serde_json::from_str(text).unwrap();
    d["options"][0]["inputs"]["shipd_parameters"]["value"].clone()
}

fn text_with(text: &str, params: &Value) -> String {
    let mut d: Value = serde_json::from_str(text).unwrap();
    d["options"][0]["inputs"]["shipd_parameters"]["value"] = params.clone();
    d.to_string()
}

#[test]
fn the_main_dimensions_set_the_hull() {
    let small = run(&design("bauxite", "pushed_barge", json!({})));
    let large = run(&design(
        "bauxite",
        "pushed_barge",
        json!({ "length_overall_m": 90.0 }),
    ));
    let disp = |o: &RunOutcome| match o {
        RunOutcome::Results { results, .. } => results["hydrostatic_table"][9]["volume_m3"]
            .as_f64()
            .unwrap(),
        other => panic!("{other:?}"),
    };
    assert!(disp(&large) > disp(&small));
}
