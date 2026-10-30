//! Seam: the angle of maximum GZ in a light condition with no ballast (issue #498, the maintainer's
//! option 2). IS Code 2008 Part A 2.2.3 (25 degrees) stays in every standard condition. In the
//! "Lightship, no ballast" condition OVD adds, and in the legacy "Light" of the Python API, a
//! failure where 2.2.3 is the only failing criterion is a warning with the ballast to sail light, not
//! a failure. The 2.2.3 alternative (approval of the Administration) is `not evaluated`, never a pass.
//!
//! Label: property. The texts read: IS Code 2008 Part A 2.2.3 (MSC.267(85)); the numbers of the hulls
//! are what the engine prints.

use ovd_core::measured::{LOADING_CONDITION_KINDS, loading_condition_kind};
use ovd_core::rules::Consequence;
use ovd_format::rule_set::Library;
use ovd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

fn results(cargo: &str, formation: &str, loa: f64, beam: f64, depth: f64) -> Value {
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": cargo, "source": "typed" },
            "hull_form": { "value": "raked_box", "source": "typed" },
            "formation": { "value": { "kind": formation }, "source": "typed" },
            "length_overall_m": { "value": loa, "source": "typed" },
            "beam_m": { "value": beam, "source": "typed" },
            "moulded_depth_m": { "value": depth, "source": "typed" },
            "may_sail_without_ballast": { "value": true, "source": "typed" }
        }}]
    });
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    match run_option(&filled.design.unwrap().options[0]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn rows(results: &Value, id: &str) -> Vec<Value> {
    results["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["rule"] == id)
        .cloned()
        .collect()
}

#[test]
fn the_kind_of_a_condition_is_read_from_its_name() {
    assert_eq!(
        loading_condition_kind("Lightship, no ballast"),
        "lightship-no-ballast"
    );
    assert_eq!(loading_condition_kind("Light"), "legacy-light");
    for name in [
        "Laden",
        "Half laden",
        "Full load, departure",
        "Ballast, arrival",
        "Box",
    ] {
        assert_eq!(loading_condition_kind(name), "standard", "{name}");
    }
    assert_eq!(LOADING_CONDITION_KINDS.len(), 3);
}

#[test]
fn is2008_a_2_0_0_applies_2_2_3_to_the_standard_conditions_and_keeps_1_0_0_as_it_was() {
    let lib = Library::built_in();
    let (old, new) = (
        lib.find("is2008-a", "1.0.0").unwrap(),
        lib.find("is2008-a", "2.0.0").unwrap(),
    );
    // Issue #282: the two areas to 40 degrees are replaced by the ones that end at the angle of
    // down-flooding, under new ids (the Measure changes).
    let replaced = ["is2008/A.2.2.1/area-0-40", "is2008/A.2.2.1/area-30-40"];
    for id in replaced {
        assert!(old.rule(id).is_some() && new.rule(id).is_none(), "{id}");
    }
    for r in old
        .rules
        .iter()
        .filter(|r| !replaced.contains(&r.id.as_str()))
    {
        let n = new.rule(&r.id).expect("kept");
        if r.id == "is2008/A.2.2.3/angle-max-gz" {
            // Only the Applicability changes: the limit, the Measure and the consequence do not.
            assert_eq!(n.limit, r.limit);
            assert_eq!(n.measure, r.measure);
            assert_eq!(n.consequence, Consequence::Block);
            assert_eq!(
                n.applies_when.conditions,
                vec![(
                    "loading_condition_kind".to_string(),
                    vec!["standard".to_string()]
                )]
            );
            assert!(r.applies_when.conditions.is_empty(), "1.0.0 is not touched");
        } else {
            assert_eq!(
                ovd_format::rule_set::rule_content_json(n),
                ovd_format::rule_set::rule_content_json(r),
                "{}",
                r.id
            );
        }
    }
    let alt = new.rule("is2008/A.2.2.3/alternative-criteria").unwrap();
    assert!(!alt.evaluated);
    assert!(
        alt.gap
            .as_deref()
            .unwrap()
            .contains("approval of the Administration")
    );
    let warn = lib
        .find("ovd", "3.0.0")
        .unwrap()
        .rule("ovd/stability/light-angle-of-max-gz")
        .unwrap();
    assert_eq!(warn.consequence, Consequence::Warn);
    assert_eq!(
        warn.applies_when.conditions,
        vec![(
            "loading_condition_kind".to_string(),
            vec![
                "lightship-no-ballast".to_string(),
                "legacy-light".to_string()
            ]
        )]
    );
}

#[test]
fn a_light_condition_that_fails_only_the_angle_is_a_warning_with_the_ballast_and_the_design_is_feasible()
 {
    // 90 x 14 x 6 m LNG, a pushed barge that may sail without ballast.
    let r = results("LNG", "pushed_barge", 90.0, 14.0, 6.0);
    assert_eq!(r["arrangement"]["feasible"], true);
    let check = r["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "Stability \u{2014} lightship, no ballast")
        .unwrap();
    assert_eq!(check["ok"], false);
    assert_eq!(check["severity"], "warning");
    let message = check["message"].as_str().unwrap();
    assert!(message.contains("warning, not a failure"), "{message}");
    assert!(message.contains("IS Code 2.2.3"), "{message}");
    assert!(message.contains("t of ballast"), "{message}");
    // No failed error check: the Design is feasible with this one warning.
    assert!(
        r["checks"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["ok"] == true || c["severity"] == "warning"),
        "{}",
        r["checks"]
    );
    // The Rule Trace: 2.2.3 does not apply to that condition; the OVD warning has its row; the
    // alternative is `not evaluated`, never a pass; every standard condition keeps 2.2.3.
    let angle = rows(&r, "is2008/A.2.2.3/angle-max-gz");
    assert!(
        angle
            .iter()
            .all(|x| x["object"] != "Lightship, no ballast" || x["outcome"] == "not applicable")
    );
    assert!(
        angle
            .iter()
            .any(|x| x["object"] == "Full load, departure" && x["outcome"] == "pass")
    );
    let warn = rows(&r, "ovd/stability/light-angle-of-max-gz");
    let light = warn
        .iter()
        .find(|x| x["object"] == "Lightship, no ballast")
        .unwrap();
    assert_eq!(light["outcome"], "warn");
    assert!(light["value"].as_f64().unwrap() < 25.0);
    let alt = rows(&r, "is2008/A.2.2.3/alternative-criteria");
    let alt_light = alt
        .iter()
        .find(|x| x["object"] == "Lightship, no ballast")
        .unwrap();
    assert_eq!(alt_light["outcome"], "not evaluated");
    // The trace verdict does not fail on the warning.
    assert_eq!(r["rule_trace"]["verdict"]["passes"], true);
}

#[test]
fn a_light_failure_with_another_criterion_failing_stays_an_error() {
    // 60 x 20 x 3.5 m LPG propane fails several criteria in more than one condition: no downgrade.
    let r = results("LPG_propane", "self_propelled", 60.0, 20.0, 3.5);
    assert_eq!(r["arrangement"]["feasible"], false);
    for c in r["checks"].as_array().unwrap() {
        let name = c["name"].as_str().unwrap();
        if name.starts_with("Stability") && c["severity"] == "warning" {
            // Only the light conditions may warn, and only with the single-criterion message.
            assert!(name.contains("light"), "{name}");
            assert!(
                c["message"]
                    .as_str()
                    .unwrap()
                    .contains("warning, not a failure"),
                "{name}"
            );
        }
    }
    assert!(
        r["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["ok"] == false && c["severity"] == "error"),
        "the infeasible hull still has failed error checks"
    );
}
