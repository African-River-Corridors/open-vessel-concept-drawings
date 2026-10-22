//! Seam: the `shipd` Hull Form in a Design (issue #452): a new Option of it runs, its parameters
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
fn a_failed_constraint_and_a_bulb_are_named() {
    let filled = fill_as_new(
        &design("bauxite", "pushed_barge", json!({})),
        &Profile::built_in(),
    );
    let mut d: Value = serde_json::from_str(&filled.design.unwrap().to_text()).unwrap();
    let params = &mut d["options"][0]["inputs"]["shipd_parameters"]["value"];
    params["Lb"] = json!(0.6);
    params["Ls"] = json!(0.6);
    params["bit_BB"] = json!(1.0);
    let checked = check(&d.to_string(), &Profile::built_in());
    let errors = checked.report.errors.join("\n");
    assert!(
        errors.contains("fails ShipD constraint 0 (bow and stern tapers within LOA)"),
        "{errors}"
    );
    assert!(errors.contains("bit_BB and bit_SB must be 0"), "{errors}");
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
