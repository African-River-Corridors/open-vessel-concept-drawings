//! Seam 4: a `.vdesign` file in, a Design as used and its validation report out. Defaults, hard
//! and soft ranges, the depth message, the convoy Formation, and the Defaults Profile.

use ovcd_format::{Profile, Source, check, fill, new_design};
use serde_json::{Value, json};

fn design_with(inputs: Value) -> String {
    json!({
        "format": "vdesign",
        "format_version": 2,
        "engine_version": "0.1.0",
        "options": [{ "name": "A", "inputs": inputs }]
    })
    .to_string()
}

fn required(cargo: &str) -> Value {
    json!({
        "cargo": cargo,
        "hull_form": "raked_box",
        "length_overall_m": 76.5,
        "beam_m": 11.4,
        "moulded_depth_m": 3.5,
        "formation": { "kind": "self_propelled" }
    })
}

fn with(mut inputs: Value, name: &str, value: Value) -> Value {
    inputs[name] = value;
    inputs
}

// `errors` and `warnings` fill in the left-out inputs first, as `ovcd fill` does, so each test
// sees only the problem it is about. `check` alone also refuses the left-out inputs.

fn errors(inputs: Value) -> Vec<String> {
    fill(&design_with(inputs), &Profile::built_in())
        .report
        .errors
}

fn warnings(inputs: Value) -> Vec<String> {
    fill(&design_with(inputs), &Profile::built_in())
        .report
        .warnings
}

fn has(list: &[String], text: &str) -> bool {
    list.iter().any(|m| m.contains(text))
}

fn used(inputs: Value, name: &str) -> (Value, Source) {
    let checked = check(&design_with(inputs), &Profile::built_in());
    let design = checked.design.expect("design read");
    let input = design.options[0]
        .input(name)
        .expect("input present")
        .clone();
    (input.value, input.source)
}

#[test]
fn a_new_design_round_trips_through_its_own_text_unchanged() {
    let profile = Profile::built_in();
    let design = new_design(&profile).unwrap();
    let checked = check(&design.to_text(), &profile);
    assert_eq!(checked.report.errors, Vec::<String>::new());
    assert_eq!(checked.report.warnings, Vec::<String>::new());
    assert_eq!(checked.design.unwrap(), design);
}

#[test]
fn gas_cargo_defaults_to_type_c_tanks_with_its_rule_sets_and_fill_ratio() {
    let inputs = required("LNG");
    assert_eq!(
        used(inputs.clone(), "cargo_containment"),
        (json!("type_c_tanks"), Source::Default)
    );
    let (sets, source) = used(inputs.clone(), "rule_sets");
    assert_eq!(source, Source::Default);
    assert_eq!(sets[0]["id"], "igc-2g");
    assert_eq!(
        used(inputs.clone(), "service_area"),
        (json!("river-only"), Source::Default)
    );
    assert_eq!(
        used(inputs.clone(), "fill_ratio"),
        (json!(0.98), Source::Default)
    );
    assert_eq!(
        used(inputs.clone(), "max_tank_od_m"),
        (json!(16.0), Source::Default)
    );
}

#[test]
fn a_box_hold_has_no_gas_inputs_and_a_tank_barge_has_no_lightship_factor() {
    let design = check(&design_with(required("bauxite")), &Profile::built_in())
        .design
        .unwrap();
    assert!(
        design.options[0]
            .input("lightship_factor_t_per_m3")
            .is_some()
    );
    assert!(design.options[0].input("max_tank_od_m").is_none());
    let design = check(&design_with(required("LNG")), &Profile::built_in())
        .design
        .unwrap();
    assert!(
        design.options[0]
            .input("lightship_factor_t_per_m3")
            .is_none()
    );
    assert!(design.options[0].input("max_tank_od_m").is_some());
}

#[test]
fn an_input_not_used_by_the_cargo_containment_is_ignored_with_a_warning() {
    let w = warnings(with(required("bauxite"), "max_tank_od_m", json!(12.0)));
    assert!(
        has(
            &w,
            "`max_tank_od_m` is not used for a box_hold Cargo Containment; ignored"
        ),
        "{w:?}"
    );
}

#[test]
fn fill_writes_a_left_out_input_in_with_a_warning() {
    let w = warnings(required("bauxite"));
    assert!(
        has(
            &w,
            "`max_speed_kn` was not stored in the file; wrote in the Defaults Profile value \
             10, marked default"
        ),
        "{w:?}"
    );
}

#[test]
fn check_refuses_a_file_that_leaves_out_optional_inputs_and_names_them() {
    let e = check(&design_with(required("bauxite")), &Profile::built_in())
        .report
        .errors;
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(
        has(
            &e,
            "Option 'A': inputs not stored in the file: cargo_containment, min_freeboard_m, \
             lightship_factor_t_per_m3, propulsive_efficiency"
        ),
        "{e:?}"
    );
    assert!(has(&e, "run `ovcd fill`"), "{e:?}");
}

#[test]
fn a_filled_file_stores_every_input_as_used_and_then_passes_check() {
    let profile = Profile::built_in();
    let filled = fill(&design_with(required("LNG")), &profile);
    assert!(filled.report.is_ok(), "{:?}", filled.report);
    let design = filled.design.unwrap();
    assert_eq!(
        design.options[0].input("fill_ratio").unwrap().source,
        Source::Default
    );
    assert_eq!(design.defaults_profile.as_deref(), Some("built-in"));
    let text = design.to_text();
    let checked = check(&text, &profile);
    assert_eq!(checked.report.errors, Vec::<String>::new());
    assert_eq!(checked.design.unwrap(), design);
}

#[test]
fn a_profile_edit_never_changes_the_inputs_of_a_filled_file() {
    let text = fill(&design_with(required("bauxite")), &Profile::built_in())
        .design
        .unwrap()
        .to_text();
    let edited = edited_profile(|p| {
        p["inputs"]["max_speed_kn"]["default"] = json!(12.0);
        p["inputs"]["min_freeboard_m"]["default"] = json!(0.5);
    })
    .unwrap();
    let before = check(&text, &Profile::built_in()).design.unwrap();
    let after = check(&text, &edited);
    assert!(after.report.is_ok(), "{:?}", after.report);
    assert_eq!(after.design.unwrap(), before);
    assert_eq!(
        before.options[0].input("max_speed_kn").unwrap().value,
        json!(10.0)
    );
}

#[test]
fn a_stored_default_is_kept_even_when_the_profile_now_says_otherwise() {
    let inputs = with(
        required("bauxite"),
        "water_density_t_per_m3",
        json!({ "value": 1.02, "source": "default" }),
    );
    assert_eq!(
        used(inputs, "water_density_t_per_m3"),
        (json!(1.02), Source::Default)
    );
}

#[test]
fn a_bare_value_counts_as_typed() {
    assert_eq!(
        used(required("bauxite"), "beam_m"),
        (json!(11.4), Source::Typed)
    );
}

#[test]
fn an_unknown_input_is_refused() {
    let e = errors(with(required("bauxite"), "vessel", json!("box_barge")));
    assert!(has(&e, "Option 'A': unknown input `vessel`"), "{e:?}");
}

#[test]
fn a_cargo_containment_that_cannot_carry_the_cargo_is_refused() {
    let e = errors(with(
        required("bauxite"),
        "cargo_containment",
        json!("type_c_tanks"),
    ));
    assert!(
        has(
            &e,
            "`cargo_containment` = type_c_tanks cannot carry bauxite (dry_bulk); use box_hold or \
             sloped_hopper_hold"
        ),
        "{e:?}"
    );
}

#[test]
fn an_unknown_cargo_is_refused_with_the_core_message() {
    let e = errors(required("gravel"));
    assert!(
        has(&e, "`cargo`: Unknown cargo type 'gravel'. Valid types: LNG"),
        "{e:?}"
    );
}

#[test]
fn freeboard_at_or_above_depth_is_a_hard_error() {
    let e = errors(with(required("bauxite"), "min_freeboard_m", json!(3.5)));
    assert!(
        has(
            &e,
            "`min_freeboard_m` = 3.5 must be less than `moulded_depth_m` = 3.5 (hard limit)"
        ),
        "{e:?}"
    );
}

#[test]
fn a_text_where_a_number_belongs_is_refused() {
    let e = errors(with(required("bauxite"), "beam_m", json!("NaN")));
    assert!(has(&e, "`beam_m` must be a number"), "{e:?}");
}

#[test]
fn a_length_under_one_metre_is_refused_but_a_short_one_only_warns() {
    let e = errors(with(required("bauxite"), "length_overall_m", json!(0.999)));
    assert!(
        has(
            &e,
            "`length_overall_m` = 0.999 is out of range: must be >= 1"
        ),
        "{e:?}"
    );
    assert!(errors(with(required("bauxite"), "length_overall_m", json!(1.0))).is_empty());
    let small = with(required("bauxite"), "length_overall_m", json!(12.0));
    assert!(errors(small.clone()).is_empty());
    assert!(has(
        &warnings(small),
        "`length_overall_m` = 12 m is outside the usual range"
    ));
}

#[test]
fn an_optional_limit_takes_null_for_none_but_not_zero() {
    let none = with(required("bauxite"), "draught_limit_m", json!(null));
    assert_eq!(used(none, "draught_limit_m"), (Value::Null, Source::Typed));
    let e = errors(with(required("bauxite"), "draught_limit_m", json!(0.0)));
    assert!(
        has(&e, "`draught_limit_m` = 0 is out of range: must be > 0"),
        "{e:?}"
    );
}

#[test]
fn a_hull_form_with_no_generator_yet_is_refused_as_not_built() {
    // Issue #446: every Hull Form of spec 0008 is a choice; raked_box, v_bottom, chine (#450) and
    // offsets (#451) and shipd (#452) are computed for now.
    {
        let form = "box";
        let e = errors(with(required("bauxite"), "hull_form", json!(form)));
        assert!(has(&e, "is not built yet"), "{form}: {e:?}");
    }
    let e = errors(with(required("bauxite"), "hull_form", json!("catamaran")));
    assert!(has(&e, "must be one of"), "{e:?}");
}

#[test]
fn a_convoy_needs_whole_numbers_abreast_and_in_line() {
    let e = errors(with(
        required("bauxite"),
        "formation",
        json!({ "kind": "pushed_convoy", "abreast": 0, "in_line": 2 }),
    ));
    assert!(
        has(&e, "`formation.abreast` = 0 is out of range: must be >= 1"),
        "{e:?}"
    );
    let e = errors(with(
        required("bauxite"),
        "formation",
        json!({ "kind": "pushed_convoy", "abreast": 2 }),
    ));
    assert!(
        has(&e, "`formation.in_line` must be a whole number"),
        "{e:?}"
    );
    let big = with(
        required("bauxite"),
        "formation",
        json!({ "kind": "pushed_convoy", "abreast": 8, "in_line": 2 }),
    );
    assert!(errors(big.clone()).is_empty());
    assert!(has(
        &warnings(big),
        "`formation.abreast` = 8 barges is outside"
    ));
}

#[test]
fn file_level_problems_are_hard_errors() {
    let p = Profile::built_in();
    let r = check(
        r#"{"format": "vdesign", "format_version": 4, "engine_version": "0.1.0", "options": []}"#,
        &p,
    )
    .report;
    assert!(
        has(
            &r.errors,
            "format_version 4 is not supported; this tool reads versions 1 to 3"
        ),
        "{r:?}"
    );
    assert!(
        has(&r.errors, "a Design needs at least one Option"),
        "{r:?}"
    );
    let two = json!({
        "format": "vdesign", "format_version": 2, "engine_version": "0.1.0",
        "options": [
            { "name": "A", "inputs": required("bauxite") },
            { "name": "A", "inputs": required("bauxite") }
        ]
    });
    let r = check(&two.to_string(), &p).report;
    assert!(has(&r.errors, "two Options are named 'A'"), "{r:?}");
}

#[test]
fn several_options_are_each_checked_by_name() {
    let p = Profile::built_in();
    let two = json!({
        "format": "vdesign", "format_version": 2, "engine_version": "0.1.0",
        "options": [
            { "name": "Short", "inputs": required("bauxite") },
            { "name": "Wide", "inputs": with(required("LNG"), "beam_m", json!(-1.0)) }
        ]
    });
    let checked = fill(&two.to_string(), &p);
    assert_eq!(checked.design.unwrap().options.len(), 2);
    assert!(has(&checked.report.errors, "Option 'Wide': `beam_m` = -1"));
    assert!(!has(&checked.report.errors, "Option 'Short'"));
}

#[test]
fn every_built_in_default_lies_inside_its_own_soft_range() {
    // Otherwise `ovcd new` would warn about its own defaults.
    let profile = Profile::built_in();
    for cargo in ["bauxite", "LNG"] {
        for containment in ["box_hold", "sloped_hopper_hold", "type_c_tanks"] {
            let inputs = with(required(cargo), "cargo_containment", json!(containment));
            let r = check(&design_with(inputs), &profile).report;
            let range_warnings: Vec<_> = r
                .warnings
                .iter()
                .filter(|w| w.contains("soft limit"))
                .collect();
            assert!(
                range_warnings.is_empty(),
                "{cargo} {containment}: {range_warnings:?}"
            );
        }
    }
}

#[test]
fn a_profile_with_a_default_outside_its_hard_range_is_refused() {
    let mut p: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    p["inputs"]["water_density_t_per_m3"]["default"] = json!(1.5);
    let err = Profile::from_json(&p.to_string()).unwrap_err();
    assert!(
        err.contains("`water_density_t_per_m3` = 1.5 is out of range"),
        "{err}"
    );
}

#[test]
fn a_profile_missing_a_default_is_refused() {
    let mut p: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    p["inputs"].as_object_mut().unwrap().remove("max_speed_kn");
    let err = Profile::from_json(&p.to_string()).unwrap_err();
    assert!(err.contains("no entry for `max_speed_kn`"), "{err}");
}

#[test]
fn an_edited_profile_range_changes_warnings_but_never_the_stored_inputs() {
    let profile = Profile::built_in();
    let saved = new_design(&profile).unwrap().to_text();
    let mut p: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    p["inputs"]["max_speed_kn"]["default"] = json!(12.0);
    p["inputs"]["max_speed_kn"]["soft"] = json!({ "min": 11.0, "max": 14.0 });
    let edited = Profile::from_json(&p.to_string()).unwrap();
    let a = check(&saved, &profile).design.unwrap();
    let checked = check(&saved, &edited);
    assert_eq!(checked.design.unwrap(), a);
    assert!(
        has(
            &checked.report.warnings,
            "`max_speed_kn` = 10 kn is outside"
        ),
        "{:?}",
        checked.report.warnings
    );
}

#[test]
fn a_whole_number_default_in_an_edited_profile_still_round_trips() {
    let mut p: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    p["inputs"]["max_speed_kn"]["default"] = json!(12);
    let edited = Profile::from_json(&p.to_string()).unwrap();
    let design = new_design(&edited).unwrap();
    assert_eq!(check(&design.to_text(), &edited).design.unwrap(), design);
}

fn edited_profile(edit: impl FnOnce(&mut Value)) -> Result<Profile, String> {
    let mut p: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    edit(&mut p);
    Profile::from_json(&p.to_string())
}

#[test]
fn a_profile_missing_any_keyed_default_is_refused() {
    use ovcd_format::vdesign::KEYED_DEFAULTS;
    for (name, by, keys) in KEYED_DEFAULTS {
        for key in *keys {
            let err = edited_profile(|p| {
                p["inputs"][*name][format!("default_by_{by}")]
                    .as_object_mut()
                    .unwrap()
                    .remove(*key);
            })
            .unwrap_err();
            assert!(
                err.contains(&format!("no default for `{name}` when {by} is {key}")),
                "{err}"
            );
        }
    }
}

#[test]
fn a_profile_with_only_a_keyed_default_for_a_plain_input_is_refused() {
    let err = edited_profile(|p| {
        let e = p["inputs"]["max_speed_kn"].as_object_mut().unwrap();
        e.remove("default");
        e.insert("default_by_ruleset".into(), json!({ "IGC_2G": 10.0 }));
    })
    .unwrap_err();
    assert!(err.contains("no default for `max_speed_kn`"), "{err}");
}

#[test]
fn every_cargo_phase_has_a_keyed_default_lookup() {
    use ovcd_format::vdesign::{CARGO_CONTAINMENTS, CARGO_PHASES};
    for (_, props) in ovcd_core::cargo::CARGO_DATABASE.iter() {
        // Liquid bulk (crude oil) has no containment yet: its keyed defaults come with the
        // `integral_tanks` containment (crude oil ticket 4).
        if props.phase == ovcd_core::cargo::CargoClass::LiquidBulk {
            continue;
        }
        assert!(CARGO_PHASES.contains(&props.phase.key()), "{}", props.phase);
    }
    for (_, phase) in CARGO_CONTAINMENTS {
        assert!(CARGO_PHASES.contains(&phase.key()), "{phase}");
    }
}

// The knife edges of the range table: a hard `lt`/`gt` bound excludes the bound itself; a soft
// range includes both ends.

#[test]
fn a_hard_lt_bound_refuses_the_bound_itself() {
    let e = errors(with(
        required("bauxite"),
        "propulsive_efficiency",
        json!(1.0),
    ));
    assert!(
        has(
            &e,
            "`propulsive_efficiency` = 1 is out of range: must be < 1"
        ),
        "{e:?}"
    );
    assert!(
        errors(with(
            required("bauxite"),
            "propulsive_efficiency",
            json!(0.9999)
        ))
        .is_empty()
    );
}

#[test]
fn a_hard_gt_bound_refuses_the_bound_itself() {
    let e = errors(with(required("bauxite"), "air_draft_limit_m", json!(0.0)));
    assert!(
        has(&e, "`air_draft_limit_m` = 0 is out of range: must be > 0"),
        "{e:?}"
    );
    assert!(
        errors(with(
            required("bauxite"),
            "air_draft_limit_m",
            json!(0.0001)
        ))
        .is_empty()
    );
}

#[test]
fn a_soft_range_includes_both_ends() {
    for beam in [4.0, 35.0] {
        let w = warnings(with(required("bauxite"), "beam_m", json!(beam)));
        assert!(!has(&w, "`beam_m`"), "{beam}: {w:?}");
    }
    for beam in [3.9999, 35.0001] {
        let w = warnings(with(required("bauxite"), "beam_m", json!(beam)));
        assert!(has(&w, "`beam_m` ="), "{beam}: {w:?}");
    }
}

#[test]
fn a_profile_soft_range_with_min_equal_to_max_is_accepted() {
    let profile = edited_profile(|p| {
        p["inputs"]["water_density_t_per_m3"]["soft"] = json!({ "min": 1.0, "max": 1.0 });
    })
    .expect("min == max is a valid soft range");
    let checked = check(&design_with(required("bauxite")), &profile);
    assert!(!has(&checked.report.warnings, "`water_density_t_per_m3` ="));
    let err = edited_profile(|p| {
        p["inputs"]["water_density_t_per_m3"]["soft"] = json!({ "min": 1.05, "max": 1.0 });
    })
    .unwrap_err();
    assert!(err.contains("min above max"), "{err}");
}

#[test]
fn a_profile_may_set_the_convoy_ranges() {
    for key in ["formation_abreast", "formation_in_line"] {
        let profile = edited_profile(|p| {
            p["inputs"][key]["soft"] = json!({ "min": 1.0, "max": 2.0 });
        })
        .unwrap_or_else(|e| panic!("{key}: {e}"));
        let convoy = with(
            required("bauxite"),
            "formation",
            json!({ "kind": "pushed_convoy", "abreast": 3, "in_line": 3 }),
        );
        let w = check(&design_with(convoy), &profile).report.warnings;
        let label = key.replace("formation_", "formation.");
        assert!(has(&w, &format!("`{label}` = 3")), "{key}: {w:?}");
    }
}

#[test]
fn a_profile_with_an_unknown_input_is_refused() {
    let err = edited_profile(|p| {
        p["inputs"]["formation_sideways"] = json!({ "default": 1.0 });
    })
    .unwrap_err();
    assert!(err.contains("unknown input `formation_sideways`"), "{err}");
}

#[test]
fn a_profile_with_a_keyed_default_outside_its_hard_range_is_refused() {
    // Issue #292: the fill ratio is a table; a value of a row is held to the hard range too.
    let err = edited_profile(|p| {
        p["inputs"]["fill_ratio"]["default_by_table"]["rows"][0]["value"] = json!(1.5);
    })
    .unwrap_err();
    assert!(err.contains("`fill_ratio` = 1.5 is out of range"), "{err}");
}

#[test]
fn a_new_design_records_the_name_of_the_profile_it_came_from() {
    let built_in = new_design(&Profile::built_in()).unwrap();
    assert_eq!(built_in.defaults_profile.as_deref(), Some("built-in"));
    let mine = edited_profile(|p| p["name"] = json!("my-yard")).expect("profile loads");
    assert_eq!(mine.name(), "my-yard");
    assert_eq!(
        new_design(&mine).unwrap().defaults_profile.as_deref(),
        Some("my-yard")
    );
}

#[test]
fn show_writes_a_formation_as_a_person_reads_it() {
    use ovcd_format::vdesign::show;
    assert_eq!(
        show(&json!({ "kind": "pushed_convoy", "abreast": 2, "in_line": 3 })),
        "pushed_convoy (2 abreast, 3 in line)"
    );
    assert_eq!(show(&json!({ "kind": "self_propelled" })), "self_propelled");
    assert_eq!(show(&json!({ "kind": "pushed_barge" })), "pushed_barge");
    // An object that is not a formation is shown as its JSON text.
    assert_eq!(show(&json!({ "a": 1 })), r#"{"a":1}"#);
    assert_eq!(show(&Value::Null), "none");
    assert_eq!(show(&json!(11.4)), "11.4");
    assert_eq!(show(&json!("bauxite")), "bauxite");
}

// A profile may tighten a hard range but never loosen it. Built-in: `water_density_t_per_m3` is
// `>= 0.9, <= 1.1`; `propulsive_efficiency` is `> 0, < 1`.

fn loads_with_hard(name: &str, hard: Value) -> bool {
    let mut p: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    p["inputs"][name]["hard"] = hard;
    match Profile::from_json(&p.to_string()) {
        Ok(_) => true,
        Err(e) => {
            assert!(e.contains("profile loosens the hard range"), "{e}");
            false
        }
    }
}

#[test]
fn a_profile_hard_range_is_refused_when_it_admits_a_value_the_built_in_refuses() {
    let looser = [
        ("water_density_t_per_m3", json!({ "gt": 0.8, "le": 1.1 })),
        ("water_density_t_per_m3", json!({ "ge": 0.9, "le": 1.5 })),
        (
            "water_density_t_per_m3",
            json!({ "ge": 0.85, "gt": 0.8, "le": 1.1 }),
        ),
        ("propulsive_efficiency", json!({ "gt": 0, "lt": 2 })),
        // Same value, but the bound now admits it: `>= 0` where the built-in says `> 0`.
        ("propulsive_efficiency", json!({ "ge": 0, "lt": 1 })),
        ("propulsive_efficiency", json!({ "gt": 0, "le": 1 })),
    ];
    for (name, hard) in looser {
        assert!(!loads_with_hard(name, hard.clone()), "{name} {hard} loaded");
    }
}

#[test]
fn a_profile_hard_range_that_admits_only_values_the_built_in_admits_loads() {
    let tighter = [
        ("water_density_t_per_m3", json!({ "ge": 0.9, "le": 1.1 })),
        ("water_density_t_per_m3", json!({ "gt": 0.9, "le": 1.05 })),
        // Both bounds apply, so the tighter one governs, whichever kind it is.
        (
            "water_density_t_per_m3",
            json!({ "gt": 0.5, "ge": 0.95, "le": 1.1 }),
        ),
        (
            "water_density_t_per_m3",
            json!({ "gt": 0.95, "ge": 0.5, "le": 1.1 }),
        ),
        (
            "water_density_t_per_m3",
            json!({ "ge": 0.9, "lt": 2, "le": 1.05 }),
        ),
        (
            "water_density_t_per_m3",
            json!({ "ge": 0.9, "lt": 1.05, "le": 2 }),
        ),
        // Two bounds at the same value: the strict one governs.
        (
            "propulsive_efficiency",
            json!({ "gt": 0, "ge": 0, "lt": 1 }),
        ),
        (
            "propulsive_efficiency",
            json!({ "gt": 0, "lt": 1, "le": 1 }),
        ),
        ("propulsive_efficiency", json!({ "gt": 0, "lt": 0.5 })),
    ];
    for (name, hard) in tighter {
        assert!(loads_with_hard(name, hard.clone()), "{name} {hard} refused");
    }
}

#[test]
fn a_profile_whose_defaults_break_a_cross_check_cannot_make_a_new_design() {
    // Loads: a 4 m freeboard suits a deep hull. But the 3.5 m starter hull cannot keep it.
    let deep = edited_profile(|p| p["inputs"]["min_freeboard_m"]["default"] = json!(4.0))
        .expect("profile loads");
    let err = new_design(&deep).unwrap_err();
    assert!(
        err.contains("`min_freeboard_m` = 4 must be less than `moulded_depth_m` = 3.5"),
        "{err}"
    );
}

// Physically impossible results. The original Python copies them faithfully (the core adds no
// checks), so validation refuses the inputs that lead there. Rule: the dry-bulk hold section
// must not be negative. Its width is the beam less the two fixed 1 m wing voids; its height is
// the moulded depth less the 1.2 m double bottom plus the 1 m coaming (
// `barge_capacity._bulk_volumetric`). A negative width or height gives a negative cargo mass
// and draught. A zero section gives zero cargo, which the engine reports as a failed check.

fn bulk(containment: &str, beam: f64, depth: f64) -> Value {
    let mut inputs = with(required("bauxite"), "cargo_containment", json!(containment));
    inputs["beam_m"] = json!(beam);
    inputs["moulded_depth_m"] = json!(depth);
    inputs["min_freeboard_m"] = json!(0.0);
    inputs
}

#[test]
fn a_one_metre_beam_dry_bulk_hold_is_refused_for_its_negative_hold_width() {
    for containment in ["box_hold", "sloped_hopper_hold"] {
        let e = errors(bulk(containment, 1.0, 3.5));
        assert!(
            has(
                &e,
                "`beam_m` = 1 leaves a hold width of -1 m after the two 1 m wing voids; a \
                 negative hold gives a negative cargo mass and draught (hard limit)"
            ),
            "{containment}: {e:?}"
        );
    }
}

#[test]
fn a_dry_bulk_hold_width_of_exactly_zero_passes_and_just_below_is_refused() {
    for containment in ["box_hold", "sloped_hopper_hold"] {
        assert_eq!(errors(bulk(containment, 2.0, 3.5)), Vec::<String>::new());
        let below = f64::from_bits(2.0f64.to_bits() - 1);
        assert!(
            has(
                &errors(bulk(containment, below, 3.5)),
                "leaves a hold width"
            ),
            "{containment}"
        );
    }
}

#[test]
fn a_moulded_depth_that_leaves_a_negative_hold_height_is_refused() {
    for containment in ["box_hold", "sloped_hopper_hold"] {
        let e = errors(bulk(containment, 11.4, 0.1));
        assert!(
            has(
                &e,
                "`moulded_depth_m` = 0.1 leaves a hold height of -0.1 m (depth less the 1.2 m \
                 double bottom, plus the 1 m coaming); a negative hold gives a negative cargo \
                 mass and draught (hard limit)"
            ),
            "{containment}: {e:?}"
        );
        assert_eq!(errors(bulk(containment, 11.4, 0.2)), Vec::<String>::new());
    }
}

#[test]
fn a_one_metre_beam_gas_barge_is_not_refused_by_the_hold_rule() {
    // Type C tanks have no wing voids; a too-narrow gas hull returns zero cargo, not a negative.
    let mut inputs = required("LNG");
    inputs["beam_m"] = json!(1.0);
    assert!(!has(&errors(inputs), "hold"));
}

// ---- the report names the inputs each message is about, as data ----------------------------

#[test]
fn every_message_names_its_inputs_as_data() {
    let report = fill(
        &design_with(with(
            with(
                with(required("LNG"), "beam_m", json!(-1.0)),
                "min_freeboard_m",
                json!(9.0),
            ),
            "not_an_input",
            json!(1),
        )),
        &Profile::built_in(),
    )
    .report;
    assert!(!report.errors.is_empty() && !report.warnings.is_empty());
    assert_eq!(report.errors.len(), report.error_inputs.len());
    assert_eq!(report.warnings.len(), report.warning_inputs.len());
    let at = |text: &str| {
        let i = report
            .errors
            .iter()
            .position(|e| e.contains(text))
            .unwrap_or_else(|| panic!("no error with {text}: {:?}", report.errors));
        report.error_inputs[i].clone()
    };
    assert_eq!(at("`beam_m` = -1"), ["beam_m"]);
    assert_eq!(at("unknown input"), Vec::<String>::new());
    assert_eq!(
        at("`min_freeboard_m` = 9"),
        ["min_freeboard_m", "moulded_depth_m"]
    );
    // A warning about one input names that input.
    let soft = fill(
        &design_with(with(required("bauxite"), "length_overall_m", json!(500.0))),
        &Profile::built_in(),
    )
    .report;
    assert_eq!(soft.warning_inputs.len(), soft.warnings.len());
    let i = soft
        .warnings
        .iter()
        .position(|w| w.contains("`length_overall_m`") && w.contains("soft limit"))
        .expect("a soft-range warning");
    assert_eq!(soft.warning_inputs[i], ["length_overall_m"]);
}

// Issue #139: a duplicate JSON key is ambiguous, so it is a hard error that names the key.

fn text_with_duplicate(find: &str, replace: &str) -> String {
    let text = serde_json::to_string_pretty(
        &serde_json::from_str::<Value>(&design_with(required("bauxite"))).unwrap(),
    )
    .unwrap();
    assert!(text.contains(find), "{find} not in {text}");
    text.replacen(find, replace, 1)
}

#[test]
fn a_duplicate_input_key_is_a_hard_error_naming_the_key() {
    // The second "beam_m" would win in a plain JSON read (the review's example: 14.8 then 30.0).
    let text = text_with_duplicate(
        "\"beam_m\": 11.4",
        "\"beam_m\": 14.8,\n \"beam_m\": {\"value\": 30.0, \"source\": \"typed\"}",
    );
    let r = fill(&text, &Profile::built_in()).report;
    assert!(has(&r.errors, "`beam_m` appears twice"), "{r:?}");
    assert!(has(&r.errors, "options[0].inputs"), "{r:?}");
}

#[test]
fn a_duplicate_top_level_key_is_a_hard_error() {
    let text = text_with_duplicate(
        "\"engine_version\": \"0.1.0\"",
        "\"engine_version\": \"0.1.0\", \"engine_version\": \"0.2.2\"",
    );
    let r = check(&text, &Profile::built_in()).report;
    assert!(has(&r.errors, "`engine_version` appears twice"), "{r:?}");
}

#[test]
fn a_duplicate_results_hash_is_a_hard_error() {
    let h = "ab".repeat(32);
    let text = text_with_duplicate(
        "\"name\": \"A\"",
        &format!("\"name\": \"A\", \"results_hash\": \"{h}\", \"results_hash\": \"{h}\""),
    );
    let r = check(&text, &Profile::built_in()).report;
    assert!(has(&r.errors, "`results_hash` appears twice"), "{r:?}");
}

#[test]
fn a_duplicate_key_inside_a_value_is_a_hard_error() {
    let text = text_with_duplicate(
        "\"kind\": \"self_propelled\"",
        "\"kind\": \"self_propelled\", \"kind\": \"pushed_barge\"",
    );
    let r = check(&text, &Profile::built_in()).report;
    assert!(has(&r.errors, "`kind` appears twice"), "{r:?}");
}

#[test]
fn the_same_key_in_different_objects_is_not_a_duplicate() {
    let two = json!({
        "format": "vdesign", "format_version": 2, "engine_version": "0.1.0",
        "options": [
            { "name": "A", "inputs": required("bauxite") },
            { "name": "B", "inputs": required("bauxite") }
        ]
    });
    let r = fill(&two.to_string(), &Profile::built_in()).report;
    assert!(!has(&r.errors, "appears twice"), "{r:?}");
}
