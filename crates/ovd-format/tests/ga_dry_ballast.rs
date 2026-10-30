//! Seam 5: the ballast tanks on the general arrangement of a dry bulk hull (issue #426, part 4).
//! The tanks are the Spaces of the compartment model (`drawing.ballast_spaces`); a Design saved with
//! the legacy loading conditions has none and draws none.

use ovd_format::{GaViews, Profile, RunOutcome, fill, ga_views_of, run_design};
use serde_json::Value;
use std::path::Path;

fn results(is_code: bool) -> Value {
    let mut text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/dry_bulk_box_hold.vdesign"),
    )
    .expect("fixture");
    // The fixture is saved in the IS Code conditions; the legacy case sets them back.
    if !is_code {
        let at = text.find("\"loading_conditions\"").expect("the input");
        let code = at + text[at..].find("is_code_2008").expect("is_code_2008");
        text.replace_range(code..code + "is_code_2008".len(), "legacy");
    }
    let design = fill(&text, &Profile::built_in())
        .design
        .expect("a fixture is a valid Design");
    run_design(&design)
        .into_iter()
        .find_map(|r| match r.outcome {
            RunOutcome::Results { results, .. } => Some(results),
            _ => None,
        })
        .expect("results")
}

fn views(results: &Value) -> GaViews {
    ga_views_of(&RunOutcome::Results {
        results: results.clone(),
        hash: String::new(),
    })
    .expect("a dry bulk hull has views")
}

/// Every text of a drawing document.
fn texts(document: &Value) -> Vec<String> {
    document["viewports"][0]["entities"]
        .as_array()
        .expect("entities")
        .iter()
        .flat_map(|e| {
            let mut out = Vec::new();
            for key in ["text", "label"] {
                if let Some(t) = e[key].as_str() {
                    out.push(t.to_string());
                }
            }
            out
        })
        .collect()
}

/// Label: snapshot - the names a reader sees on the views of a dry bulk hull in the IS Code conditions.
#[test]
fn the_views_name_the_ballast_tanks_of_a_dry_bulk_hull() {
    let r = results(true);
    let spaces = r["drawing"]["ballast_spaces"].as_array().expect("spaces");
    assert!(!spaces.is_empty());
    let v = views(&r);
    let plan = texts(&v.plan);
    assert!(plan.iter().any(|t| t == "WBT (BOTH SIDES)"), "{plan:?}");
    let section = texts(&v.midship_section);
    assert!(section.iter().any(|t| t == "DOUBLE BOTTOM"), "{section:?}");
    assert!(
        section.iter().any(|t| t == "WBT (BOTH SIDES)"),
        "{section:?}"
    );
    let side = texts(&v.side_elevation);
    assert!(side.iter().any(|t| t == "DOUBLE BOTTOM"), "{side:?}");
}

/// Label: property - a Design in the legacy conditions has no ballast tanks and draws none.
#[test]
fn a_legacy_design_draws_no_ballast_tanks() {
    let r = results(false);
    assert!(r["drawing"].get("ballast_spaces").is_none());
    let v = views(&r);
    for doc in [&v.plan, &v.midship_section, &v.side_elevation] {
        assert!(
            !texts(doc)
                .iter()
                .any(|t| t == "DOUBLE BOTTOM" || t == "WBT (BOTH SIDES)")
        );
    }
}

/// Label: property - the ballast tanks are drawing data: they never enter the results hash.
#[test]
fn the_ballast_tanks_are_not_hashed() {
    let mut r = results(true);
    let before = ovd_format::results_hash(&r);
    r["drawing"]["ballast_spaces"] = Value::Null;
    assert_eq!(ovd_format::results_hash(&r), before);
}
