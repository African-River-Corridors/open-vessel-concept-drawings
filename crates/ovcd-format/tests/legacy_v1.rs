//! Seam 4: a version 1 `.vdesign` file opens in the current format (issue #175). Version 2
//! changed the gas tank inputs: the `ADN_G` ruleset and `adn_tank_cap_m3` are gone and
//! `max_tank_od_m` is new. A version 1 file is read as `IGC_2G`, drops the cap, takes the
//! Defaults Profile diameter cap (version 1 had a fixed 12 m), and says so in a warning. Each test reads the real files in
//! `tests/legacy/`, which are version 1 files as that version wrote them.
//!
//! Label: property

use ovcd_format::{DesignSession, HashStatus, Profile, Source, check, fill};
use serde_json::{Value, json};

fn legacy(name: &str) -> String {
    let path = format!("{}/tests/legacy/{name}.vdesign", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn has(list: &[String], text: &str) -> bool {
    list.iter().any(|m| m.contains(text))
}

const ADN: &str = "`ruleset` ADN_G is no longer offered";
const CAP: &str = "`adn_tank_cap_m3` is no longer an input";
const OD: &str = "this version 1 file has no `max_tank_od_m`; it now uses the default 16 m";

#[test]
fn the_fixture_really_is_a_version_1_file_that_names_adn_g() {
    let raw: Value = serde_json::from_str(&legacy("v1_adn_g_self_propelled")).unwrap();
    assert_eq!(raw["format_version"], 1);
    let inputs = &raw["options"][0]["inputs"];
    assert_eq!(inputs["ruleset"]["value"], "ADN_G");
    assert_eq!(inputs["adn_tank_cap_m3"]["value"], 1000.0);
    assert!(inputs.get("max_tank_od_m").is_none());
}

#[test]
fn validate_opens_it_as_igc_2g_without_the_cap_and_warns() {
    let checked = check(&legacy("v1_adn_g_self_propelled"), &Profile::built_in());
    assert_eq!(checked.report.errors, Vec::<String>::new());
    let w = &checked.report.warnings;
    assert!(has(w, ADN), "{w:?}");
    assert!(has(w, CAP), "{w:?}");
    assert!(has(w, OD), "{w:?}");
    let design = checked.design.unwrap();
    let o = &design.options[0];
    // Issue #274: the ruleset is no longer an input; the name is read, and dropped.
    assert!(o.input("ruleset").is_none());
    assert!(o.input("adn_tank_cap_m3").is_none());
    let od = o.input("max_tank_od_m").unwrap();
    assert_eq!(
        (od.value.clone(), od.source),
        (json!(16.0), Source::Default)
    );
}

#[test]
fn each_warning_names_its_option_and_the_input_it_is_about() {
    let r = check(&legacy("v1_adn_g_self_propelled"), &Profile::built_in()).report;
    assert_eq!(r.warnings.len(), r.warning_inputs.len());
    let about = |text: &str| {
        let i = r.warnings.iter().position(|m| m.contains(text)).unwrap();
        assert!(
            r.warnings[i].starts_with("Option 'Option 1': "),
            "{}",
            r.warnings[i]
        );
        r.warning_inputs[i].clone()
    };
    assert_eq!(about(ADN), ["ruleset"]);
    assert_eq!(about(CAP), ["adn_tank_cap_m3"]);
    assert_eq!(about(OD), ["max_tank_od_m"]);
}

#[test]
fn fill_writes_a_version_2_file_that_validates_without_those_warnings() {
    let p = Profile::built_in();
    let filled = fill(&legacy("v1_adn_g_self_propelled"), &p);
    assert!(filled.report.is_ok(), "{:?}", filled.report);
    assert!(has(&filled.report.warnings, ADN));
    let text = filled.design.unwrap().to_text();
    let saved: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(saved["format_version"], 3);
    // Issue #274: the ruleset is no longer an input, so it is not written.
    assert!(saved["options"][0]["inputs"].get("ruleset").is_none());
    assert!(
        saved["options"][0]["inputs"]
            .get("adn_tank_cap_m3")
            .is_none()
    );
    let again = check(&text, &p);
    assert_eq!(again.report.errors, Vec::<String>::new());
    assert_eq!(again.report.warnings, Vec::<String>::new());
}

#[test]
fn the_design_session_opens_it_with_the_warnings_and_runs_it_as_igc_2g() {
    let (session, opened) =
        DesignSession::open(&legacy("v1_adn_g_self_propelled"), Profile::built_in()).unwrap();
    assert!(opened.report.is_ok(), "{:?}", opened.report);
    let w = &opened.report.warnings;
    assert!(has(w, ADN) && has(w, CAP) && has(w, OD), "{w:?}");
    // The saved hash is of the ADN_G results; these are IGC_2G, so it differs.
    assert!(
        matches!(opened.hash_status[0].1, HashStatus::Differs { .. }),
        "{:?}",
        opened.hash_status
    );
    let e = session.evaluate();
    assert!(
        e.options[0].messages.is_empty(),
        "{:?}",
        e.options[0].messages
    );
    let outcome = e.options[0].outcome.as_ref().expect("it ran");
    let text = format!("{outcome:?}");
    assert!(text.contains("IGC_2G"), "{text}");
    assert!(!text.contains("ADN_G"), "{text}");
    // Saving writes the current format.
    let saved: Value = serde_json::from_str(&session.save().unwrap()).unwrap();
    assert_eq!(saved["format_version"], 3);
}

#[test]
fn a_bare_value_form_is_changed_too() {
    let file = json!({
        "format": "vdesign", "format_version": 1, "engine_version": "0.1.0",
        "options": [{ "name": "A", "inputs": {
            "cargo": "LNG", "hull_form": "raked_box", "length_overall_m": 80.0, "beam_m": 12.0,
            "moulded_depth_m": 5.0, "formation": { "kind": "self_propelled" },
            "ruleset": "ADN_G", "adn_tank_cap_m3": 380.0
        } }]
    });
    let r = fill(&file.to_string(), &Profile::built_in());
    assert!(r.report.is_ok(), "{:?}", r.report);
    assert!(has(&r.report.warnings, ADN) && has(&r.report.warnings, CAP));
    let o = &r.design.unwrap().options[0];
    assert!(o.input("ruleset").is_none());
    // Left out, so the Defaults Profile gives it under the new ruleset: 98 %.
    assert_eq!(o.input("fill_ratio").unwrap().value, json!(0.98));
}

#[test]
fn a_version_1_dry_bulk_file_gets_only_the_note_about_the_new_hold_inputs() {
    // Slice 1 of issue #4: the hold inputs are new, so the one warning is about them. The gas
    // changes of version 2 do not touch a dry-bulk file.
    let checked = check(&legacy("v1_dry_bulk_box_hold"), &Profile::built_in());
    assert_eq!(checked.report.errors, Vec::<String>::new());
    let w = &checked.report.warnings;
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(
        has(w, "fixed value the engine used before it was an input"),
        "{w:?}"
    );
    assert!(!has(w, ADN) && !has(w, CAP) && !has(w, OD), "{w:?}");
    let (_, opened) =
        DesignSession::open(&legacy("v1_dry_bulk_box_hold"), Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
}

#[test]
fn a_version_2_file_gets_no_such_help() {
    // Only version 1 files are changed. A current file naming ADN_G, or the cap, is refused.
    let mut raw: Value = serde_json::from_str(&legacy("v1_adn_g_self_propelled")).unwrap();
    raw["format_version"] = json!(2);
    let r = check(&raw.to_string(), &Profile::built_in()).report;
    assert!(
        has(&r.errors, "`ruleset` = \"ADN_G\" must be one of: IGC_2G"),
        "{r:?}"
    );
    assert!(has(&r.errors, "unknown input `adn_tank_cap_m3`"), "{r:?}");
}
