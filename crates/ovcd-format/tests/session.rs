//! The Design session (spec 0002): behaviour a user or an agent can see — a Design or an edit
//! in, values and messages out. State is only read through what an operation returns.

use ovcd_format::Profile;
use ovcd_format::session::{GROUPS, Session, input_schema};
use ovcd_format::vdesign::INPUTS;
use ovcd_format::{Report, check, fill, new_design};
use serde_json::{Map, Value, json};

/// The six required inputs of a box-hold dry-bulk pushed barge, as a starter Design holds them.
fn required() -> Value {
    json!({
        "cargo": json!({ "value": "bauxite", "source": "typed" }),
        "hull_form": json!({ "value": "raked_box", "source": "typed" }),
        "formation": json!({ "value": { "kind": "pushed_barge" }, "source": "typed" }),
        "length_overall_m": json!({ "value": 76.5, "source": "typed" }),
        "beam_m": json!({ "value": 11.4, "source": "typed" }),
        "moulded_depth_m": json!({ "value": 3.5, "source": "typed" }),
    })
}

/// A hand-written file holding only the required inputs, as a designer would start one.
const HAND_WRITTEN: &str = r#"{
  "format": "vdesign",
  "format_version": 1,
  "engine_version": "0.2.0",
  "units": "SI",
  "options": [
    {
      "name": "A",
      "inputs": {
        "cargo": { "value": "bauxite", "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "pushed_barge" }, "source": "typed" },
        "length_overall_m": { "value": 76.5, "source": "typed" },
        "beam_m": { "value": 11.4, "source": "typed" },
        "moulded_depth_m": { "value": 3.5, "source": "typed" }
      }
    }
  ]
}
"#;

/// The `.vdesign` text of a returned session state, as `save` would write it.
fn text_of(state: &Value) -> String {
    let mut text = serde_json::to_string_pretty(&state["design"]).expect("JSON");
    text.push('\n');
    text
}

/// `ovcd validate` on the Design a session holds after `edit`.
fn check_after(edit: impl FnOnce(&mut Session) -> Result<Value, String>) -> (Value, Report) {
    let mut s = Session::new();
    let state = edit(&mut s).expect("edit");
    let eval = s.evaluate().expect("evaluate");
    let checked = check(&text_of(&state), &Profile::built_in());
    (eval, checked.report)
}

/// Every error sentence in `evaluate`, each once, as `Report` holds it. A cross-check sits on
/// both of its inputs; the report has the sentence once.
fn errors_of(eval: &Value) -> Vec<String> {
    let mut out: Vec<String> = eval["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e.as_str().unwrap().to_string())
        .collect();
    for option in eval["options"].as_array().unwrap() {
        out.extend(
            option["errors"]
                .as_array()
                .unwrap()
                .iter()
                .map(|e| e.as_str().unwrap().to_string()),
        );
        for input in option["inputs"].as_array().unwrap() {
            for key in ["hard_errors", "cross_checks"] {
                out.extend(
                    input[key]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|e| e.as_str().unwrap().to_string()),
                );
            }
        }
    }
    let mut seen = Vec::new();
    for message in out {
        if !seen.contains(&message) {
            seen.push(message);
        }
    }
    seen
}

/// Every warning sentence in `evaluate`, each once, as `Report` holds it.
fn warnings_of(eval: &Value) -> Vec<String> {
    let mut out: Vec<String> = eval["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w.as_str().unwrap().to_string())
        .collect();
    for option in eval["options"].as_array().unwrap() {
        out.extend(
            option["warnings"]
                .as_array()
                .unwrap()
                .iter()
                .map(|w| w.as_str().unwrap().to_string()),
        );
        for input in option["inputs"].as_array().unwrap() {
            out.extend(
                input["soft_warnings"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|w| w.as_str().unwrap().to_string()),
            );
        }
    }
    let mut seen = Vec::new();
    for message in out {
        if !seen.contains(&message) {
            seen.push(message);
        }
    }
    seen
}

fn input_of<'a>(option: &'a Value, name: &str) -> &'a Value {
    option["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["name"] == name)
        .unwrap_or_else(|| panic!("input {name}"))
}

#[test]
fn new_design_equals_what_ovcd_new_writes() {
    let mut s = Session::new();
    let state = s.new_design().unwrap();
    assert_eq!(
        state["design"],
        new_design(&Profile::built_in()).unwrap().to_json()
    );
    assert_eq!(state["design"]["options"][0]["name"], "Option 1");
    assert_eq!(
        state["design"]["options"][0]["inputs"]["cargo"],
        json!({ "value": "bauxite", "source": "typed" })
    );
}

#[test]
fn open_of_a_hand_written_file_equals_ovcd_fill() {
    let mut s = Session::new();
    let opened = s.open(HAND_WRITTEN).unwrap();
    let filled = fill(HAND_WRITTEN, &Profile::built_in());
    let filled = filled.design.expect("fill reads the file");
    assert_eq!(opened["design"], filled.to_json());
}

#[test]
fn a_saved_design_passes_validate_and_round_trips_through_open_unchanged() {
    let mut s = Session::new();
    s.set_input("Option 1", "length_overall_m", json!(80.0))
        .unwrap();
    let text = s.save().unwrap();

    let checked = check(&text, &Profile::built_in());
    assert!(checked.report.is_ok(), "{:?}", checked.report);
    assert!(checked.report.errors.is_empty());
    assert!(checked.report.warnings.is_empty());

    let mut reopened = Session::new();
    let opened = reopened.open(&text).unwrap();
    assert_eq!(
        opened["design"],
        serde_json::from_str::<Value>(&text).unwrap()
    );
    assert_eq!(reopened.save().unwrap(), text);
}

#[test]
fn hard_range_gives_the_validate_message_on_the_right_input() {
    let (eval, report) = check_after(|s| s.set_input("Option 1", "length_overall_m", json!(0.5)));

    let expected = "Option 'Option 1': `length_overall_m` = 0.5 is out of range: must be >= 1 \
                    (hard limit)";
    let hard = input_of(&eval["options"][0], "length_overall_m")["hard_errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e.as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(hard, vec![expected.to_string()]);
    assert_eq!(errors_of(&eval), report.errors);
    assert_eq!(report.errors, vec![expected.to_string()]);
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
}

#[test]
fn soft_range_gives_the_validate_message_on_the_right_input() {
    let (eval, report) = check_after(|s| s.set_input("Option 1", "length_overall_m", json!(12.0)));

    let expected = "Option 'Option 1': `length_overall_m` = 12 m is outside the usual range 20 \
                    to 250 m (soft limit); accepted";
    let soft = input_of(&eval["options"][0], "length_overall_m")["soft_warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w.as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(soft, vec![expected.to_string()]);
    assert_eq!(warnings_of(&eval), report.warnings);
    assert_eq!(report.warnings, vec![expected.to_string()]);
    assert!(report.errors.is_empty(), "{:?}", report.errors);
}

#[test]
fn cross_check_message_is_on_both_inputs() {
    let (eval, report) = check_after(|s| s.set_input("Option 1", "min_freeboard_m", json!(3.5)));

    let expected = "Option 'Option 1': `min_freeboard_m` = 3.5 must be less than \
                    `moulded_depth_m` = 3.5 (hard limit)";
    for name in ["min_freeboard_m", "moulded_depth_m"] {
        let crosses = input_of(&eval["options"][0], name)["cross_checks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap().to_string())
            .collect::<Vec<_>>();
        assert_eq!(crosses, vec![expected.to_string()], "{name}");
    }
    assert_eq!(errors_of(&eval), report.errors);
    assert_eq!(report.errors, vec![expected.to_string()]);
}

#[test]
fn every_edit_operation_on_a_new_design() {
    let mut s = Session::new();

    let state = s
        .set_input("Option 1", "max_speed_kn", json!(20.0))
        .unwrap();
    let stored = &state["design"]["options"][0]["inputs"]["max_speed_kn"];
    assert_eq!(stored, &json!({ "value": 20.0, "source": "typed" }));

    let state = s.reset_input("Option 1", "max_speed_kn").unwrap();
    let stored = &state["design"]["options"][0]["inputs"]["max_speed_kn"];
    assert_eq!(stored, &json!({ "value": 10.0, "source": "default" }));

    let state = s
        .set_formation("Option 1", "pushed_convoy", Some(3), Some(2))
        .unwrap();
    let formation = &state["design"]["options"][0]["inputs"]["formation"]["value"];
    assert_eq!(formation["kind"], "pushed_convoy");
    assert_eq!(formation["abreast"], 3);
    assert_eq!(formation["in_line"], 2);

    let state = s.add_option("Option 2").unwrap();
    assert_eq!(state["design"]["options"].as_array().unwrap().len(), 2);
    let added = state["design"]["options"][1]["inputs"].as_object().unwrap();
    for (name, stored) in required().as_object().unwrap() {
        assert_eq!(&added[name], stored, "{name}");
    }

    let state = s.duplicate_option("Option 1", "Option 3").unwrap();
    assert_eq!(state["design"]["options"].as_array().unwrap().len(), 3);
    assert_eq!(
        state["design"]["options"][2]["inputs"],
        state["design"]["options"][0]["inputs"]
    );

    let state = s.rename_option("Option 2", "B").unwrap();
    assert_eq!(state["design"]["options"][1]["name"], "B");

    let state = s.delete_option("B").unwrap();
    assert_eq!(state["design"]["options"].as_array().unwrap().len(), 2);

    assert_eq!(
        s.set_input("Nope", "max_speed_kn", json!(1)).unwrap_err(),
        "there is no Option 'Nope'"
    );
    assert_eq!(
        s.set_input("Option 1", "nope", json!(1)).unwrap_err(),
        "unknown input `nope`"
    );
    assert_eq!(
        s.reset_input("Option 1", "cargo").unwrap_err(),
        "`cargo` is required and has no default; give it a value"
    );
    assert_eq!(
        s.add_option("").unwrap_err(),
        "an Option needs a non-empty string `name`"
    );
    assert_eq!(
        s.rename_option("Option 3", "Option 1").unwrap_err(),
        "two Options are named 'Option 1'; names must differ"
    );
    assert_eq!(
        s.set_formation("Option 1", "convoy", None, None)
            .unwrap_err(),
        "`formation.kind` must be one of: self_propelled, pushed_barge, pushed_convoy"
    );
    assert_eq!(
        s.set_formation("Option 1", "pushed_convoy", Some(2), None)
            .unwrap_err(),
        "`formation.in_line` must be a whole number for a pushed_convoy"
    );
    assert_eq!(
        s.set_formation("Option 1", "pushed_convoy", None, Some(2))
            .unwrap_err(),
        "`formation.abreast` must be a whole number for a pushed_convoy"
    );
}

#[test]
fn deleting_the_last_option_is_refused() {
    let mut s = Session::new();
    assert_eq!(
        s.delete_option("Option 1").unwrap_err(),
        "a Design needs at least one Option"
    );
    assert_eq!(
        s.evaluate().unwrap()["options"].as_array().unwrap().len(),
        1
    );
}

#[test]
fn a_profile_that_loosens_a_hard_range_is_refused() {
    let mut s = Session::new();
    let mut root: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    root["inputs"]["length_overall_m"]["hard"] = json!({ "ge": 0.5, "le": 1000.0 });
    let err = s
        .load_profile(&serde_json::to_string(&root).unwrap())
        .unwrap_err();
    assert!(
        err.contains("profile loosens the hard range of `length_overall_m`"),
        "{err}"
    );
    assert!(
        err.contains("it must be >= 1 and <= 1000 at least"),
        "{err}"
    );

    // A tighter range is accepted, and shows in the schema.
    let mut root: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    root["inputs"]["length_overall_m"]["hard"] = json!({ "ge": 1.0, "le": 200.0 });
    let state = s
        .load_profile(&serde_json::to_string(&root).unwrap())
        .unwrap();
    assert_eq!(state["design"]["options"].as_array().unwrap().len(), 1);
}

#[test]
fn the_input_schema_lists_every_input_with_the_profile_ranges() {
    let profile = Profile::built_in();
    let schema = input_schema(&profile);
    let inputs = schema["inputs"].as_array().unwrap();
    assert_eq!(inputs.len(), INPUTS.len());
    assert_eq!(
        schema["groups"],
        json!(GROUPS.iter().map(|g| g.as_str()).collect::<Vec<_>>())
    );

    for (entry, spec) in inputs.iter().zip(INPUTS.iter()) {
        assert_eq!(entry["name"], json!(spec.name));
        assert_eq!(entry["group"], json!(spec.group.as_str()));
        assert_eq!(entry["required"], json!(spec.required));
        assert_eq!(entry["unit"], json!(profile.unit(spec.name)));
        let basis = profile.basis(spec.name);
        assert_eq!(
            entry["basis"],
            if basis.is_empty() {
                Value::Null
            } else {
                json!(basis)
            }
        );

        let hard = profile.hard(spec.name).unwrap();
        let mut expected_hard = Map::new();
        for (key, bound) in [
            ("gt", hard.gt),
            ("ge", hard.ge),
            ("lt", hard.lt),
            ("le", hard.le),
        ] {
            if let Some(x) = bound {
                expected_hard.insert(key.into(), json!(x));
            }
        }
        let expected_hard = if expected_hard.is_empty() {
            Value::Null
        } else {
            Value::Object(expected_hard)
        };
        assert_eq!(entry["hard"], expected_hard, "{}", spec.name);

        let expected_soft = match profile.soft(spec.name).unwrap() {
            Some(s) => json!({ "min": s.min, "max": s.max }),
            None => Value::Null,
        };
        assert_eq!(entry["soft"], expected_soft, "{}", spec.name);
    }

    // The Formation's counted fields carry the profile's ranges too.
    let formation = inputs.iter().find(|i| i["name"] == "formation").unwrap();
    for (key, profile_name) in [
        ("abreast", "formation_abreast"),
        ("in_line", "formation_in_line"),
    ] {
        let hard = profile.hard(profile_name).unwrap();
        let mut expected_hard = Map::new();
        for (k, bound) in [
            ("gt", hard.gt),
            ("ge", hard.ge),
            ("lt", hard.lt),
            ("le", hard.le),
        ] {
            if let Some(x) = bound {
                expected_hard.insert(k.into(), json!(x));
            }
        }
        assert_eq!(
            formation["fields"][key]["hard"],
            Value::Object(expected_hard),
            "{profile_name}"
        );
    }
}

#[test]
fn evaluate_on_a_new_design_agrees_with_check() {
    let mut s = Session::new();
    s.set_input("Option 1", "beam_m", json!(3.0)).unwrap();
    let eval = s.evaluate().unwrap();
    assert_eq!(eval["profile"], "built-in");
    assert_eq!(eval["options"].as_array().unwrap().len(), 1);
    let option = &eval["options"][0];
    assert!(option["results"].is_object());
    assert_eq!(option["errors"].as_array().unwrap().len(), 0);
    assert_eq!(option["message"], Value::Null);
    // A value below the soft range warns but is accepted, so the Option still runs.
    let soft = input_of(option, "beam_m")["soft_warnings"]
        .as_array()
        .unwrap();
    assert_eq!(soft.len(), 1);
}
