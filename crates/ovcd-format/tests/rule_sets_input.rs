//! Seam: format version 3 holds the Service Area and the selected Rule Sets of an Option (issue
//! #274, spec 0004, "File format (ADR 0007): version 3"). No result moves: a file saved before is
//! read with the stack the engine always checked it against, and its results hash is the same.
//!
//! Label: property. Each test states a rule: the schema, the reading of an older file, the four
//! cases of opening a stored set, the shape of the input, the saved file.

use ovcd_format::rule_set::Library;
use ovcd_format::vdesign::{INPUTS, RULE_SET_SOURCES, SERVICE_AREAS};
use ovcd_format::{
    DesignSession, HashStatus, Profile, RunOutcome, Source, check, fill_as_new, input_schema,
    run_option_with,
};
use serde_json::{Value, json};

fn legacy(name: &str) -> String {
    let path = format!("{}/tests/legacy/{name}.vdesign", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn rule_file(set: &str, version: &str) -> String {
    let path = format!("{}/rules/{set}/{version}.json", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn hand(cargo: &str) -> Value {
    json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.3", "units": "SI",
        "defaults_profile": "built-in",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": cargo, "source": "typed" },
            "hull_form": { "value": "raked_box", "source": "typed" },
            "formation": { "value": { "kind": "pushed_barge" }, "source": "typed" },
            "length_overall_m": { "value": 76.5, "source": "typed" },
            "beam_m": { "value": 11.4, "source": "typed" },
            "moulded_depth_m": { "value": 3.5, "source": "typed" }
        }}]
    })
}

/// A new dry-bulk Option, filled from the profile.
fn dry() -> ovcd_format::Design {
    let filled = fill_as_new(&hand("bauxite").to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    filled.design.unwrap()
}

fn stored(design: &ovcd_format::Design) -> Vec<(String, String, String)> {
    design.options[0]
        .input("rule_sets")
        .unwrap()
        .value
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            let t = |k: &str| e[k].as_str().unwrap().to_string();
            (t("id"), t("version"), t("source"))
        })
        .collect()
}

fn set_stored(design: &mut ovcd_format::Design, list: Value) {
    for (name, input) in &mut design.options[0].inputs {
        if name == "rule_sets" {
            input.value = list.clone();
        }
    }
}

fn trace_warnings(outcome: &RunOutcome) -> Vec<String> {
    match outcome {
        RunOutcome::Results { results, .. } => results["rule_trace"]["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|w| w.as_str().unwrap().to_string())
            .collect(),
        other => panic!("expected results, got {other:?}"),
    }
}

#[test]
fn the_schema_lists_the_two_new_inputs_and_not_the_ruleset() {
    let schema = input_schema(&Profile::built_in());
    let inputs = schema["inputs"].as_array().unwrap();
    assert!(inputs.iter().all(|i| i["name"] != "ruleset"));
    let area = inputs.iter().find(|i| i["name"] == "service_area").unwrap();
    assert_eq!(area["group"], "Rules");
    assert_eq!(area["default"], "river-only");
    assert_eq!(area["default_status"], "estimate");
    let values: Vec<&str> = area["choices"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["value"].as_str().unwrap())
        .collect();
    assert_eq!(values, SERVICE_AREAS);
    assert_eq!(SERVICE_AREAS, ["river-only", "coastal-and-river"]);
    let sets = inputs.iter().find(|i| i["name"] == "rule_sets").unwrap();
    assert_eq!(sets["group"], "Rules");
    assert_eq!(sets["kind"], "rule_sets");
    assert_eq!(
        sets["shape"]["columns"],
        json!(["id", "version", "hash", "source"])
    );
    // The default stack is its own key: `default_rule` is the depth rule, which the app reads by its
    // shape.
    assert!(sets.get("default_rule").is_none());
    assert_eq!(
        sets["default_stack"]["by"],
        json!(["service_area", "cargo_containment"])
    );
    assert_eq!(
        sets["default_stack"]["stacks"]["river-only"]["gas"],
        json!(["igc-2g", "is2008-a", "lr-inland-gas", "ovcd"])
    );
    assert_eq!(
        sets["default_stack"]["stacks"]["coastal-and-river"]["other"],
        json!(["is2008-a", "ovcd"])
    );
    // The app lists each selected set with its status and its unverified blocking Rules.
    let known = sets["known_sets"].as_array().unwrap();
    let ovcd = known
        .iter()
        .find(|k| k["id"] == "ovcd" && k["version"] == "2.1.0")
        .unwrap();
    assert_eq!(ovcd["status"], "draft");
    assert!(ovcd["unverified_blocking"].as_u64().unwrap() > 0);
    let is = known.iter().find(|k| k["id"] == "is2008-a").unwrap();
    assert_eq!(is["unverified_blocking"], 0);
    assert!(INPUTS.iter().all(|s| s.name != "ruleset"));
}

#[test]
fn a_file_saved_before_gets_the_default_stack_marked_default_with_one_warning_and_the_same_hash() {
    for (file, gas) in [
        ("v2_gas_lpg_before_313", true),
        ("v2_gas_lng_before_292", true),
        ("v1_dry_bulk_box_hold", false),
    ] {
        let text = legacy(file);
        let checked = check(&text, &Profile::built_in());
        assert_eq!(checked.report.errors, Vec::<String>::new(), "{file}");
        let design = checked.design.unwrap();
        let o = &design.options[0];
        let area = o.input("service_area").unwrap();
        assert_eq!(
            (area.value.clone(), area.source),
            (json!("river-only"), Source::Default)
        );
        let sets = o.input("rule_sets").unwrap();
        assert_eq!(sets.source, Source::Default, "{file}");
        let want: &[&str] = if gas {
            &["igc-2g", "is2008-a", "lr-inland-gas", "ovcd"]
        } else {
            &["is2008-a", "ovcd"]
        };
        let ids: Vec<String> = stored(&design).into_iter().map(|(id, _, _)| id).collect();
        assert_eq!(ids, want, "{file}");
        // Each entry holds the built-in version and its content hash, marked default.
        let lib = Library::built_in();
        for e in sets.value.as_array().unwrap() {
            let set = lib.latest(e["id"].as_str().unwrap()).unwrap();
            assert_eq!(e["version"], set.version.as_str());
            assert_eq!(e["hash"], set.hash.as_str());
            assert_eq!(e["source"], "default");
        }
        // One warning for the new inputs of the Option, and it names both.
        let notes: Vec<&String> = checked
            .report
            .warnings
            .iter()
            .filter(|w| w.contains("`rule_sets`"))
            .collect();
        assert_eq!(notes.len(), 1, "{file}: {:?}", checked.report.warnings);
        assert!(notes[0].contains("`service_area`"), "{}", notes[0]);
        assert!(notes[0].contains("results are unchanged"), "{}", notes[0]);
        // The results hash is the one the file stored.
        let (_, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
        assert_eq!(opened.hash_status[0].1, HashStatus::Matches, "{file}");
    }
}

#[test]
fn a_new_option_takes_the_default_stack_and_the_saved_file_is_version_3_and_validates() {
    let design = dry();
    let sets = stored(&design);
    assert_eq!(
        sets.iter()
            .map(|(i, _, s)| (i.as_str(), s.as_str()))
            .collect::<Vec<_>>(),
        [("is2008-a", "default"), ("ovcd", "default")]
    );
    assert_eq!(
        design.options[0].input("service_area").unwrap().value,
        json!("river-only")
    );
    let saved: Value = serde_json::from_str(&design.to_text()).unwrap();
    assert_eq!(saved["format_version"], 3);
    let again = check(&design.to_text(), &Profile::built_in());
    assert_eq!(again.report.errors, Vec::<String>::new());
    assert_eq!(again.report.warnings, Vec::<String>::new());
    // The default stack follows the Cargo.
    let gas = fill_as_new(&hand("LNG").to_string(), &Profile::built_in());
    assert_eq!(gas.report.errors, Vec::<String>::new());
    let ids: Vec<String> = stored(&gas.design.unwrap())
        .into_iter()
        .map(|(i, _, _)| i)
        .collect();
    assert_eq!(ids, ["igc-2g", "is2008-a", "lr-inland-gas", "ovcd"]);
}

#[test]
fn a_ruleset_input_of_any_version_is_read_with_the_name_dropped() {
    let mut file = hand("LNG");
    file["options"][0]["inputs"]["cargo_containment"] =
        json!({"value": "type_c_tanks", "source": "typed"});
    file["options"][0]["inputs"]["ruleset"] = json!({"value": "IGC_2G", "source": "default"});
    let checked = check(&file.to_string(), &Profile::built_in());
    assert!(
        !checked.report.errors.iter().any(|e| e.contains("ruleset")),
        "{:?}",
        checked.report.errors
    );
    assert!(
        checked.design.unwrap().options[0]
            .input("ruleset")
            .is_none()
    );
}

#[test]
fn the_run_checks_the_stored_stack_and_a_changed_stack_changes_the_trace() {
    let lib = Library::built_in();
    let mut design = dry();
    let names = |o: &RunOutcome| match o {
        RunOutcome::Results { results, .. } => results["rule_trace"]["rule_sets"].to_string(),
        other => panic!("{other:?}"),
    };
    let default = run_option_with(&design.options[0], &lib, &[]);
    assert!(names(&default).contains("is2008-a") && names(&default).contains("ovcd@2.1.0"));
    assert_eq!(trace_warnings(&default), Vec::<String>::new());
    // The Option stores the older ovcd version: the run uses it, with no swap.
    let old = lib.find("ovcd", "1.0.0").unwrap();
    set_stored(
        &mut design,
        json!([{"id": "ovcd", "version": "1.0.0", "hash": old.hash, "source": "typed"}]),
    );
    let outcome = run_option_with(&design.options[0], &lib, &[]);
    assert!(
        names(&outcome).contains("ovcd@1.0.0"),
        "{}",
        names(&outcome)
    );
    assert!(!names(&outcome).contains("is2008-a"), "{}", names(&outcome));
    // The newer built-in version has other content: one warning, never a swap.
    let w = trace_warnings(&outcome);
    assert!(
        w.iter()
            .any(|m| m.contains("ovcd@1.0.0") && m.contains("also built in")),
        "{w:?}"
    );
}

#[test]
fn opening_a_stored_set_has_four_cases() {
    let lib = Library::built_in();
    let latest = lib.latest("ovcd").unwrap().clone();
    let entry = |version: &str, hash: &str| json!([{"id": "ovcd", "version": version, "hash": hash, "source": "typed"}]);
    let run = |list: Value, lib: &Library| {
        let mut design = dry();
        set_stored(&mut design, list);
        run_option_with(&design.options[0], lib, &[])
    };
    // 1. The same version and hash: used, no warning.
    let same = run(entry(&latest.version, &latest.hash), &lib);
    assert_eq!(trace_warnings(&same), Vec::<String>::new());
    // 2. A newer built-in version with the same content hash: no warning (only a verification changed).
    let mut text: Value = serde_json::from_str(&rule_file("ovcd", &latest.version)).unwrap();
    text["version"] = json!("2.1.1");
    let mut with_patch = Library::built_in();
    let key = with_patch.add_text(&text.to_string()).unwrap();
    assert_eq!(key, "ovcd@2.1.1");
    assert_eq!(with_patch.latest("ovcd").unwrap().hash, latest.hash);
    let patched = run(entry(&latest.version, &latest.hash), &with_patch);
    assert_eq!(trace_warnings(&patched), Vec::<String>::new());
    // 3. A stored hash that differs from the built-in hash of that version: a warning.
    let other = "0".repeat(64);
    let differs = run(entry(&latest.version, &other), &lib);
    let w = trace_warnings(&differs);
    assert!(
        w.iter()
            .any(|m| m.contains("the rules differ from the file")),
        "{w:?}"
    );
    // 4. A stored version the engine lacks: the Option is refused, naming the set.
    let missing = run(entry("99.0.0", &latest.hash), &lib);
    match missing {
        RunOutcome::Failed(m) => {
            assert!(m.contains("ovcd@99.0.0") && m.contains("--rule-set"), "{m}")
        }
        other => panic!("{other:?}"),
    }
    // ... unless a loaded file supplies it.
    let mut text: Value = serde_json::from_str(&rule_file("ovcd", &latest.version)).unwrap();
    text["version"] = json!("99.0.0");
    let mut supplied = Library::built_in();
    supplied.add_text(&text.to_string()).unwrap();
    let ok = run(entry("99.0.0", &latest.hash), &supplied);
    assert!(matches!(ok, RunOutcome::Results { .. }), "{ok:?}");
}

#[test]
fn rule_sets_takes_only_its_shape() {
    let spec = INPUTS.iter().find(|s| s.name == "rule_sets").unwrap();
    let p = Profile::built_in();
    let ok = |v: Value| ovcd_format::vdesign::check_input_value(spec, &v, &p).is_ok();
    let e = |id: &str, source: &str| json!({"id": id, "version": "1.0.0", "hash": "ab", "source": source});
    assert!(ok(json!([e("ovcd", "default")])));
    assert!(ok(json!([e("is2008-a", "typed"), e("ovcd", "default")])));
    assert!(RULE_SET_SOURCES == ["default", "typed"]);
    assert!(!ok(json!(null)));
    assert!(!ok(json!("ovcd")));
    assert!(!ok(json!([])), "the ovcd set cannot be left out");
    assert!(
        !ok(json!([e("is2008-a", "default")])),
        "the ovcd set cannot be left out"
    );
    assert!(
        !ok(json!([e("ovcd", "default"), e("ovcd", "typed")])),
        "no set twice"
    );
    assert!(
        !ok(json!([e("ovcd", "estimate")])),
        "a source is default or typed"
    );
    assert!(!ok(
        json!([{"id": "ovcd", "version": "1.0.0", "source": "default"}])
    ));
    assert!(!ok(
        json!([{"id": "ovcd", "version": "1.0.0", "hash": "ab", "source": "default", "x": 1}])
    ));
}

#[test]
fn service_area_takes_its_two_values_only() {
    let spec = INPUTS.iter().find(|s| s.name == "service_area").unwrap();
    let p = Profile::built_in();
    let ok = |v: Value| ovcd_format::vdesign::check_input_value(spec, &v, &p).is_ok();
    assert!(ok(json!("river-only")) && ok(json!("coastal-and-river")));
    assert!(!ok(json!("ocean-only")));
}

#[test]
fn the_trace_names_the_service_area_of_the_option() {
    let mut design = dry();
    for (name, input) in &mut design.options[0].inputs {
        if name == "service_area" {
            input.value = json!("coastal-and-river");
        }
    }
    let outcome = run_option_with(&design.options[0], &Library::built_in(), &[]);
    match outcome {
        RunOutcome::Results { results, .. } => {
            assert_eq!(results["rule_trace"]["service_area"], "coastal-and-river");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_default_stack_follows_the_cargo_in_a_session_and_a_typed_one_stays() {
    let ids = |s: &DesignSession| -> Vec<String> {
        s.design().options[0]
            .input("rule_sets")
            .unwrap()
            .value
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["id"].as_str().unwrap().to_string())
            .collect()
    };
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    assert_eq!(ids(&s), ["is2008-a", "ovcd"]);
    s.set_input("Option 1", "cargo", json!("LNG")).unwrap();
    assert_eq!(ids(&s), ["igc-2g", "is2008-a", "lr-inland-gas", "ovcd"]);
    // A set the designer added stays when the Cargo changes; the default sets follow the Cargo.
    s.add_rule_set("Option 1", "ovcd@1.0.0").unwrap();
    assert_eq!(ids(&s), ["igc-2g", "is2008-a", "lr-inland-gas", "ovcd"]);
    s.set_input("Option 1", "cargo", json!("bauxite")).unwrap();
    assert_eq!(ids(&s), ["is2008-a", "ovcd"]);
    let ovcd = &s.design().options[0].input("rule_sets").unwrap().value[1];
    assert_eq!(
        (ovcd["version"].as_str(), ovcd["source"].as_str()),
        (Some("1.0.0"), Some("typed"))
    );
}

// --- issue #275: the stacks in the profile, the Service Area, adding and removing sets -------------

fn profile_with_coastal_gas_stack(stack: Value) -> Profile {
    let mut text: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    text["inputs"]["rule_sets"]["default_stacks"]["coastal-and-river"]["gas"] = stack;
    Profile::from_json(&text.to_string()).unwrap()
}

fn ids_of(s: &DesignSession) -> Vec<(String, String)> {
    s.design().options[0]
        .input("rule_sets")
        .unwrap()
        .value
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["id"].as_str().unwrap().to_string(),
                e["source"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

#[test]
fn the_profile_holds_the_default_stack_of_each_service_area_and_cargo() {
    let p = Profile::built_in();
    for area in ["river-only", "coastal-and-river"] {
        assert_eq!(
            p.default_stack(area, true).unwrap(),
            ["igc-2g", "is2008-a", "lr-inland-gas", "ovcd"]
        );
        assert_eq!(p.default_stack(area, false).unwrap(), ["is2008-a", "ovcd"]);
    }
    // A copy that leaves out a stack, or the `ovcd` set, is refused.
    let mut text: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    text["inputs"]["rule_sets"]["default_stacks"]["river-only"]["gas"] = json!(["igc-2g"]);
    assert!(
        Profile::from_json(&text.to_string())
            .unwrap_err()
            .contains("`ovcd`")
    );
    text["inputs"]["rule_sets"]["default_stacks"]["river-only"]
        .as_object_mut()
        .unwrap()
        .remove("gas");
    assert!(
        Profile::from_json(&text.to_string())
            .unwrap_err()
            .contains("river-only")
    );
    // A copy written before the stacks existed takes the built-in ones.
    let mut old: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    old["inputs"].as_object_mut().unwrap().remove("rule_sets");
    let loaded = Profile::from_json(&old.to_string()).unwrap();
    assert_eq!(
        loaded.default_stack("river-only", false).unwrap(),
        ["is2008-a", "ovcd"]
    );
}

#[test]
fn changing_the_service_area_resets_the_default_sets_and_keeps_the_typed_ones() {
    let profile = profile_with_coastal_gas_stack(json!(["igc-2g", "is2008-a", "ovcd"]));
    let mut s = DesignSession::new_design(profile).unwrap();
    s.set_input("Option 1", "cargo", json!("LNG")).unwrap();
    let names = |s: &DesignSession| ids_of(s).into_iter().map(|(i, _)| i).collect::<Vec<_>>();
    assert_eq!(names(&s), ["igc-2g", "is2008-a", "lr-inland-gas", "ovcd"]);
    // A set the designer adds is typed; the Service Area change resets the default ones.
    s.add_rule_set("Option 1", "lr-inland-gas@1.0.0").unwrap();
    s.set_input("Option 1", "service_area", json!("coastal-and-river"))
        .unwrap();
    assert_eq!(
        ids_of(&s),
        [
            ("igc-2g".to_string(), "default".to_string()),
            ("is2008-a".to_string(), "default".to_string()),
            ("ovcd".to_string(), "default".to_string()),
            ("lr-inland-gas".to_string(), "typed".to_string()),
        ]
    );
    assert_eq!(
        s.design().options[0].input("rule_sets").unwrap().source,
        Source::Typed
    );
    // Back to river-only: the profile's river stack, the typed set still there once.
    s.set_input("Option 1", "service_area", json!("river-only"))
        .unwrap();
    let list = ids_of(&s);
    assert_eq!(list.iter().filter(|(i, _)| i == "lr-inland-gas").count(), 1);
    // The typed entry takes the place of the default one with its id.
    assert_eq!(list[2], ("lr-inland-gas".to_string(), "typed".to_string()));
    // With nothing typed, the stack is marked default again.
    s.remove_rule_set("Option 1", "lr-inland-gas").unwrap();
    s.set_input("Option 1", "service_area", json!("river-only"))
        .unwrap();
    assert!(ids_of(&s).iter().all(|(_, src)| src == "default"));
}

#[test]
fn the_ovcd_set_and_a_set_an_included_set_requires_cannot_be_removed() {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    s.set_input("Option 1", "cargo", json!("LNG")).unwrap();
    let e = s.remove_rule_set("Option 1", "ovcd").unwrap_err();
    assert!(e.contains("cannot be removed"), "{e}");
    // igc-2g includes is2008-a.
    let e = s.remove_rule_set("Option 1", "is2008-a").unwrap_err();
    assert!(e.contains("required by igc-2g@"), "{e}");
    // Remove the including set first, then the included one goes.
    s.remove_rule_set("Option 1", "igc-2g").unwrap();
    s.remove_rule_set("Option 1", "is2008-a").unwrap();
    let e = s.remove_rule_set("Option 1", "is2008-a").unwrap_err();
    assert!(e.contains("not selected"), "{e}");
    assert_eq!(
        ids_of(&s)
            .iter()
            .map(|(i, _)| i.as_str())
            .collect::<Vec<_>>(),
        ["lr-inland-gas", "ovcd"]
    );
}

#[test]
fn adding_a_set_takes_a_known_id_and_version_and_replaces_another_version_of_it() {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    let e = s.add_rule_set("Option 1", "nope@1.0.0").unwrap_err();
    assert!(e.contains("not in this engine"), "{e}");
    let e = s.add_rule_set("Option 1", "ovcd@9.9.9").unwrap_err();
    assert!(e.contains("not in this engine"), "{e}");
    s.add_rule_set("Option 1", "ovcd@1.0.0").unwrap();
    let ovcd: Vec<&Value> = s.design().options[0]
        .input("rule_sets")
        .unwrap()
        .value
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["id"] == "ovcd")
        .collect();
    assert_eq!(ovcd.len(), 1);
    assert_eq!(ovcd[0]["version"], "1.0.0");
}

#[test]
fn the_trace_states_the_scope_with_the_service_area_the_sets_and_the_gaps() {
    let mut design = dry();
    let scope =
        |d: &ovcd_format::Design| match run_option_with(&d.options[0], &Library::built_in(), &[]) {
            RunOutcome::Results { results, .. } => {
                results["rule_trace"]["scope"].as_str().unwrap().to_string()
            }
            other => panic!("{other:?}"),
        };
    let river = scope(&design);
    assert!(
        river.starts_with("Service Area river-only; Rule Sets is2008-a@1.0.0, ovcd@"),
        "{river}"
    );
    assert!(
        river.contains("no statutory technical code was checked, only these Rule Sets"),
        "{river}"
    );
    // The count of blocking Rules not evaluated, named, as the verdict says it.
    assert!(
        river.contains("blocking Rule(s) not evaluated")
            || river.contains("no blocking Rule is left"),
        "{river}"
    );
    assert!(!river.contains("meets every rule"), "{river}");
    for (name, input) in &mut design.options[0].inputs {
        if name == "service_area" {
            input.value = json!("coastal-and-river");
        }
    }
    let coastal = scope(&design);
    assert!(
        coastal.starts_with("Service Area coastal-and-river; "),
        "{coastal}"
    );
    assert!(
        !coastal.contains("no statutory technical code"),
        "{coastal}"
    );
}
