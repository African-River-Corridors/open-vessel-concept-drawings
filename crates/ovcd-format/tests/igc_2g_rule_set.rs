//! Seam: the one rule set `IGC_2G` and how files that name the old ones open (issue #237).
//!
//! `IGC_2G` is the per-tank d(Vc) of IGC Code 2.4.1.1 with the datum of 2.4.2. `IGC_2G_EXACT` is no
//! longer offered: a file that names it opens as `IGC_2G`, with a warning. A version 1 file that
//! names `IGC_2G` meant the old stepped reading, so it opens as `IGC_2G` with a warning that the
//! results may change. A file that lacks the dome diameter gets the default, with a warning that
//! says the results may change (the dome now counts in Vc).
//!
//! Label: property

use ovcd_format::{DesignSession, Profile, RunOutcome, Source, check};
use serde_json::{Value, json};

fn raw(dir: &str, name: &str) -> Value {
    let path = format!("{}/tests/{dir}/{name}.vdesign", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}")))
        .unwrap()
}

fn has(list: &[String], text: &str) -> bool {
    list.iter().any(|m| m.contains(text))
}

const EXACT_NOTE: &str = "IGC_2G_EXACT is now IGC_2G: the same per-tank rule of IGC 2.4.1.1";
const STEPPED_NOTE: &str = "IGC_2G now uses the per-tank rule of IGC 2.4.1.1";

/// A version 2 gas file with the ruleset set to `name`.
fn v2_with(name: &str) -> Value {
    let mut f = raw("fixtures", "gas_igc_2g_exact");
    f["options"][0]["inputs"]["ruleset"] = json!({ "value": name, "source": "typed" });
    f
}

/// A version 1 gas file with the ruleset set to `name`.
fn v1_with(name: &str) -> Value {
    let mut f = raw("legacy", "v1_adn_g_self_propelled");
    f["options"][0]["inputs"]["ruleset"]["value"] = json!(name);
    f
}

fn ruleset_of(f: &Value) -> (Value, Vec<String>, Vec<String>) {
    let c = check(&f.to_string(), &Profile::built_in());
    let design = c.design.expect("the file opens");
    let r = design.options[0].input("ruleset").unwrap();
    assert_eq!(r.source, Source::Typed);
    (r.value.clone(), c.report.errors, c.report.warnings)
}

#[test]
fn a_file_that_names_igc_2g_exact_opens_as_igc_2g_in_either_version() {
    for f in [v2_with("IGC_2G_EXACT"), v1_with("IGC_2G_EXACT")] {
        let (value, errors, warnings) = ruleset_of(&f);
        assert_eq!(errors, Vec::<String>::new());
        assert_eq!(value, json!("IGC_2G"), "{}", f["format_version"]);
        assert!(has(&warnings, EXACT_NOTE), "{warnings:?}");
        assert!(!has(&warnings, STEPPED_NOTE), "{warnings:?}");
    }
}

#[test]
fn the_exact_name_may_be_a_bare_string_too() {
    let mut f = v2_with("IGC_2G_EXACT");
    f["options"][0]["inputs"]["ruleset"] = json!("IGC_2G_EXACT");
    let c = check(&f.to_string(), &Profile::built_in());
    assert_eq!(c.report.errors, Vec::<String>::new());
    assert!(
        has(&c.report.warnings, EXACT_NOTE),
        "{:?}",
        c.report.warnings
    );
}

#[test]
fn a_version_1_file_that_names_igc_2g_opens_with_a_results_may_change_note() {
    let (value, errors, warnings) = ruleset_of(&v1_with("IGC_2G"));
    assert_eq!(errors, Vec::<String>::new());
    assert_eq!(value, json!("IGC_2G"));
    assert!(has(&warnings, STEPPED_NOTE), "{warnings:?}");
    assert!(has(&warnings, "results may change"), "{warnings:?}");
    assert!(!has(&warnings, EXACT_NOTE), "{warnings:?}");
}

#[test]
fn a_version_2_file_that_names_igc_2g_needs_no_ruleset_note() {
    // Version 2 is unreleased and written by this build: its IGC_2G is the per-tank rule.
    let (value, errors, warnings) = ruleset_of(&v2_with("IGC_2G"));
    assert_eq!(errors, Vec::<String>::new());
    assert_eq!(value, json!("IGC_2G"));
    assert!(!has(&warnings, "IGC_2G now uses"), "{warnings:?}");
    assert!(!has(&warnings, "IGC_2G_EXACT is now"), "{warnings:?}");
}

#[test]
fn a_version_1_adn_g_file_still_opens_as_igc_2g() {
    let (value, errors, warnings) = ruleset_of(&v1_with("ADN_G"));
    assert_eq!(errors, Vec::<String>::new());
    assert_eq!(value, json!("IGC_2G"));
    assert!(
        has(&warnings, "`ruleset` ADN_G is no longer offered"),
        "{warnings:?}"
    );
}

#[test]
fn igc_2g_exact_is_not_a_choice_and_the_profile_default_is_igc_2g() {
    let profile = Profile::built_in();
    assert_eq!(profile.default("ruleset"), Some(&json!("IGC_2G")));
    assert_eq!(
        profile.default_by("fill_ratio", "ruleset", "IGC_2G"),
        Some(&json!(0.98))
    );
    assert_eq!(ovcd_format::vdesign::RULESETS, ["IGC_2G"]);
}

#[test]
fn a_file_without_the_dome_diameter_gets_2_m_marked_default_and_a_results_may_change_note() {
    let mut f = raw("fixtures", "gas_igc_2g_exact");
    f["options"][0]["inputs"]
        .as_object_mut()
        .unwrap()
        .remove("tank_dome_diameter_m");
    let c = check(&f.to_string(), &Profile::built_in());
    assert_eq!(c.report.errors, Vec::<String>::new());
    let d = c.design.unwrap();
    let dia = d.options[0].input("tank_dome_diameter_m").unwrap();
    assert_eq!(
        (dia.value.as_f64(), dia.source),
        (Some(2.0), Source::Default)
    );
    let w = &c.report.warnings;
    assert!(
        has(
            w,
            "this file has no `tank_dome_diameter_m`; it now holds 2.0 m"
        ),
        "{w:?}"
    );
    assert!(has(w, "so the results may change"), "{w:?}");
    // The note of the inputs that kept their old results does not claim that for the dome.
    assert!(
        !w.iter()
            .any(|m| m.contains("tank_dome_diameter_m") && m.contains("results are unchanged")),
        "{w:?}"
    );
}

#[test]
fn the_dome_diameter_is_a_gas_tank_input_with_a_basis_and_a_range() {
    use ovcd_format::vdesign::INPUTS;
    let spec = INPUTS
        .iter()
        .find(|s| s.name == "tank_dome_diameter_m")
        .unwrap();
    assert_eq!(spec.group.label(), "Gas tanks");
    assert_eq!(spec.applies_to.containments(), ["type_c_tanks"]);
    let mut f = raw("fixtures", "gas_igc_2g_exact");
    f["options"][0]["inputs"]["tank_dome_diameter_m"] = json!({ "value": -0.1, "source": "typed" });
    let c = check(&f.to_string(), &Profile::built_in());
    assert!(
        has(&c.report.errors, "tank_dome_diameter_m"),
        "{:?}",
        c.report.errors
    );
}

#[test]
fn the_profile_basis_of_the_dome_diameter_says_what_it_is_and_cites_the_rule() {
    let basis = Profile::built_in()
        .basis("tank_dome_diameter_m")
        .to_string();
    assert!(basis.contains("no published source yet"), "{basis}");
    assert!(basis.contains("IGC 2.4.1.1"), "{basis}");
    assert!(basis.contains("Counts in Vc only"), "{basis}");
}

fn clearance(dome_diameter_m: f64) -> f64 {
    let mut f = raw("fixtures", "gas_igc_2g_exact");
    f["options"][0]["inputs"]["tank_dome_diameter_m"] =
        json!({ "value": dome_diameter_m, "source": "typed" });
    let (session, _) = DesignSession::open(&f.to_string(), Profile::built_in()).unwrap();
    match session.evaluate().options[0]
        .outcome
        .as_ref()
        .expect("it ran")
    {
        RunOutcome::Results { results, .. } => {
            results["geometry"]["side_clearance_m"].as_f64().unwrap()
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_dome_diameter_reaches_the_engine_and_raises_the_clearance_through_vc() {
    let (cs_none, cs_wide) = (clearance(0.0), clearance(6.0));
    // d(Vc) rises with Vc (IGC 2.4.1.1), and a wider dome adds to Vc.
    assert!(cs_wide > cs_none, "{cs_wide} vs {cs_none}");
}
