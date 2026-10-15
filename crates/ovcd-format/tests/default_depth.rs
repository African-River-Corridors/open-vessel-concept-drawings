//! Seam 4: the default moulded depth (issue #120). Moulded depth is keel to main deck at the side.
//! A left-out depth is the depth-to-beam ratio of the Cargo times the beam, from the Defaults
//! Profile, marked `default`. A very slender hull for its depth (length over depth above about 25)
//! is a soft warning. Each test states a rule the maintainer gave on the issue.
//!
//! Label: property

use ovcd_format::vdesign::{DEPTH_GROUPS, Source, depth_group};
use ovcd_format::{DesignSession, Profile, RunOutcome, check, fill, input_schema};
use serde_json::{Value, json};

fn file(cargo: &str, beam: f64, depth: Option<f64>) -> String {
    let mut inputs = json!({
        "cargo": cargo, "hull_form": "raked_box", "length_overall_m": 80.0, "beam_m": beam,
        "formation": { "kind": "self_propelled" }
    });
    if let Some(d) = depth {
        inputs["moulded_depth_m"] = json!(d);
    }
    json!({ "format": "vdesign", "format_version": 2, "engine_version": "0.1.0",
            "options": [{ "name": "A", "inputs": inputs }] })
    .to_string()
}

fn depth_of(text: &str, profile: &Profile) -> (Value, Source) {
    let o = &fill(text, profile).design.expect("design").options[0];
    let i = o.input("moulded_depth_m").expect("a depth").clone();
    (i.value, i.source)
}

#[test]
fn a_left_out_depth_is_the_ratio_of_the_cargo_times_the_beam_marked_default() {
    let p = Profile::built_in();
    assert_eq!(
        depth_of(&file("bauxite", 10.0, None), &p),
        (json!(3.3), Source::Default)
    );
    for gas in ["LNG", "LNG_ethane", "LPG_propane", "LPG_butane", "LPG_mix"] {
        assert_eq!(
            depth_of(&file(gas, 10.0, None), &p),
            (json!(5.5), Source::Default),
            "{gas}"
        );
    }
}

#[test]
fn the_default_is_rounded_to_the_millimetre() {
    // 0.33 x 11.4 = 3.7620000000000005 in floating point.
    assert_eq!(
        depth_of(&file("bauxite", 11.4, None), &Profile::built_in()).0,
        json!(3.762)
    );
}

#[test]
fn a_typed_depth_is_never_replaced_by_the_rule() {
    let p = Profile::built_in();
    assert_eq!(
        depth_of(&file("LNG", 10.0, Some(4.0)), &p),
        (json!(4.0), Source::Typed)
    );
}

#[test]
fn the_rule_lives_in_the_profile_as_data_and_an_edited_profile_changes_the_default() {
    let mut raw: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    raw["inputs"]["depth_to_beam_ratio"]["default_by_cargo_group"]["dry_bulk"] = json!(0.4);
    let edited = Profile::from_json(&raw.to_string()).unwrap();
    assert_eq!(
        depth_of(&file("bauxite", 10.0, None), &edited).0,
        json!(4.0)
    );
    assert_eq!(depth_of(&file("LNG", 10.0, None), &edited).0, json!(5.5));
}

#[test]
fn a_profile_that_leaves_out_a_cargo_groups_ratio_is_refused() {
    for group in DEPTH_GROUPS {
        let mut raw: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
        raw["inputs"]["depth_to_beam_ratio"]["default_by_cargo_group"]
            .as_object_mut()
            .unwrap()
            .remove(*group);
        let err = Profile::from_json(&raw.to_string()).unwrap_err();
        assert!(
            err.contains(&format!("depth-to-beam ratio for {group} Cargo")),
            "{err}"
        );
    }
    let mut raw: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    raw["inputs"]["depth_to_beam_ratio"]["default_by_cargo_group"]["gas"] = json!(5.0);
    assert!(
        Profile::from_json(&raw.to_string())
            .unwrap_err()
            .contains("out of range")
    );
}

#[test]
fn every_cargo_of_the_format_has_a_group_and_liquid_has_none_yet() {
    let mut seen = Vec::new();
    for (key, props) in ovcd_core::cargo::CARGO_DATABASE.iter() {
        let g = depth_group(key).unwrap_or_else(|| panic!("{key} has no depth group"));
        assert_eq!(
            g,
            if props.phase == "dry_bulk" {
                "dry_bulk"
            } else {
                "gas"
            },
            "{key}"
        );
        seen.push(g);
    }
    assert!(seen.contains(&"dry_bulk") && seen.contains(&"gas"));
    assert!(
        !seen.contains(&"liquid"),
        "a liquid Cargo now exists: map it"
    );
    assert_eq!(depth_group("no-such-cargo"), None);
}

#[test]
fn a_depth_on_its_default_follows_a_change_of_beam_or_cargo_and_a_typed_depth_does_not() {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    s.set_input("Option 1", "moulded_depth_m", json!(4.0))
        .unwrap();
    s.reset_input("Option 1", "moulded_depth_m").unwrap();
    let depth = |s: &DesignSession| {
        s.design().options[0]
            .input("moulded_depth_m")
            .unwrap()
            .clone()
    };
    assert_eq!(
        (depth(&s).value, depth(&s).source),
        (json!(3.762), Source::Default)
    );
    s.set_input("Option 1", "beam_m", json!(12.0)).unwrap();
    assert_eq!(depth(&s).value, json!(3.96));
    s.set_input("Option 1", "cargo", json!("LNG")).unwrap();
    assert_eq!(
        (depth(&s).value, depth(&s).source),
        (json!(6.6), Source::Default)
    );
    s.set_input("Option 1", "moulded_depth_m", json!(5.0))
        .unwrap();
    s.set_input("Option 1", "beam_m", json!(14.0)).unwrap();
    assert_eq!(
        (depth(&s).value, depth(&s).source),
        (json!(5.0), Source::Typed)
    );
}

#[test]
fn a_class_defined_option_picks_the_default_depth_up_like_any_other() {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    s.reset_input("Option 1", "moulded_depth_m").unwrap();
    s.set_input("Option 1", "vessel_class", json!("CEMT-IV"))
        .unwrap();
    let o = &s.design().options[0];
    assert_eq!(o.input("beam_m").unwrap().value, json!(9.5));
    let d = o.input("moulded_depth_m").unwrap();
    assert_eq!((d.value.clone(), d.source), (json!(3.135), Source::Default));
}

#[test]
fn a_depth_that_equals_the_rules_value_gives_the_same_results_as_the_rule() {
    let p = Profile::built_in();
    let hash = |text: &str| {
        let design = fill(text, &p).design.unwrap();
        match ovcd_format::run_option(&design.options[0]) {
            RunOutcome::Results { hash, .. } => hash,
            other => panic!("{other:?}"),
        }
    };
    assert_eq!(
        hash(&file("bauxite", 10.0, None)),
        hash(&file("bauxite", 10.0, Some(3.3)))
    );
}

#[test]
fn a_hull_over_about_25_times_its_depth_is_a_soft_warning_on_both_inputs() {
    let p = Profile::built_in();
    let warnings = |depth: f64, loa: f64| {
        let text = file("bauxite", 10.0, Some(depth)).replace("80.0", &format!("{loa:?}"));
        let r = fill(&text, &p).report;
        assert!(r.is_ok(), "{r:?}");
        r.warnings
            .iter()
            .zip(&r.warning_inputs)
            .filter(|(w, _)| w.contains("slender hull"))
            .map(|(w, i)| (w.clone(), i.clone()))
            .collect::<Vec<_>>()
    };
    let hit = warnings(4.0, 104.0); // 26
    assert_eq!(hit.len(), 1, "{hit:?}");
    assert_eq!(hit[0].1, ["length_overall_m", "moulded_depth_m"]);
    assert!(
        hit[0].0.contains("26.0 times")
            && hit[0].0.contains("over about 25")
            && hit[0].0.contains("soft limit")
    );
    assert!(warnings(4.0, 100.0).is_empty(), "exactly 25 is not over it");
    assert!(warnings(4.0, 80.0).is_empty());
}

#[test]
fn a_depth_from_the_rule_can_raise_the_same_warning_on_a_long_narrow_hull() {
    // L / (0.33 B) is over 25 when L/B is over 8.25: the rule's depth is rough for long hulls.
    let p = Profile::built_in();
    let text = file("bauxite", 10.0, None).replace("80.0", "90.0"); // L/B 9, L/D 27.3
    let r = fill(&text, &p).report;
    assert!(
        r.warnings.iter().any(|w| w.contains("slender hull")),
        "{:?}",
        r.warnings
    );
}

#[test]
fn check_says_a_depth_is_not_stored_like_any_other_default() {
    let r = check(&file("bauxite", 10.0, None), &Profile::built_in()).report;
    assert!(
        r.errors
            .iter()
            .any(|e| e.contains("inputs not stored in the file") && e.contains("moulded_depth_m")),
        "{r:?}"
    );
}

#[test]
fn the_schema_shows_the_rule_and_its_basis_and_depth_is_not_required() {
    let schema = input_schema(&Profile::built_in());
    let depth = schema["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["name"] == "moulded_depth_m")
        .unwrap();
    assert_eq!(depth["required"], json!(false));
    assert!(depth.get("default").is_none());
    let rule = &depth["default_rule"];
    for (group, want) in [("dry_bulk", 0.33), ("liquid", 0.30), ("gas", 0.55)] {
        assert_eq!(rule["ratios"][group].as_f64(), Some(want), "{group}");
    }
    assert_eq!(rule["cargo_group"]["bauxite"], "dry_bulk");
    assert_eq!(rule["cargo_group"]["LPG_butane"], "gas");
    assert_eq!(rule["length_to_depth_warning_above"], json!(25.0));
    let basis = rule["basis"].as_str().unwrap();
    assert!(
        basis.contains("reference vessels")
            && basis.contains("0.22 to 0.44")
            && basis.contains("0.44 to 0.66"),
        "{basis}"
    );
    // The old text is gone.
    let own = depth["basis"].as_str().unwrap();
    assert!(
        own.contains("keel to main deck at the side") && !own.contains("holds no depths"),
        "{own}"
    );
}
