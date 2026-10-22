//! Seam: the built-in Rule Sets `ovcd` and `lr-inland-gas` 1.0.0, and the link from each check of a
//! result to the Rule it reports (spec 0004, issue #272).
//!
//! Label: property

use ovcd_core::rules::Origin;
use ovcd_format::rule_set::{Library, describe};
use ovcd_format::trace::{CHECK_RULES, rule_of_check};
use serde_json::{Value, json};

fn fixture_json(name: &str) -> Value {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name),
    )
    .unwrap();
    serde_json::from_str(&text).unwrap()
}

/// Each computed Option's results, for a Design given as JSON.
fn results_of(design: &Value) -> Vec<Value> {
    let checked = ovcd_format::check(&design.to_string(), &ovcd_format::Profile::built_in());
    let design = checked.design.expect("a valid Design");
    ovcd_format::run_design(&design)
        .into_iter()
        .filter_map(|r| match r.outcome {
            ovcd_format::RunOutcome::Results { results, .. } => Some(results),
            _ => None,
        })
        .collect()
}

/// The fixtures, and variants that reach more checks: an air draft limit (low enough to fail), a
/// short gas hull with no feasible arrangement.
fn corpus() -> Vec<(String, Value)> {
    let mut out = Vec::new();
    for name in [
        "dry_bulk_box_hold.vdesign",
        "dry_bulk_sloped_hopper.vdesign",
        "gas_igc_2g_exact.vdesign",
        "gas_igc_2g_self_propelled.vdesign",
        "two_options.vdesign",
    ] {
        let d = fixture_json(name);
        out.push((name.to_string(), d.clone()));
        for limit in [4.0, 30.0] {
            let mut v = d.clone();
            for o in v["options"].as_array_mut().unwrap() {
                o["inputs"]["air_draft_limit_m"] = json!({"value": limit, "source": "typed"});
                o.as_object_mut().unwrap().remove("results_hash");
            }
            out.push((format!("{name} air draft {limit} m"), v));
        }
    }
    let mut short = fixture_json("gas_igc_2g_self_propelled.vdesign");
    short["options"][0]["inputs"]["length_overall_m"] = json!({"value": 45.0, "source": "typed"});
    short["options"][0]
        .as_object_mut()
        .unwrap()
        .remove("results_hash");
    out.push(("short gas hull".to_string(), short));
    out
}

#[test]
fn every_check_of_a_result_reports_exactly_one_rule() {
    let lib = Library::built_in();
    let mut seen = std::collections::BTreeSet::new();
    for (name, design) in corpus() {
        for results in results_of(&design) {
            let rows = results["rule_trace"]["rows"].as_array().unwrap();
            for check in results["checks"].as_array().unwrap() {
                let check_name = check["name"].as_str().unwrap();
                // The per-condition checks "Air draft" and "Ballast plan" of the IS Code loading
                // conditions (issue #426, part 2) are not linked to a Rule yet: a follow-up of the
                // rules lane. No fixture failed them before the examples were saved in those
                // conditions.
                if ["Air draft", "Ballast plan"].contains(&check_name) {
                    continue;
                }
                let id = rule_of_check(check_name)
                    .unwrap_or_else(|| panic!("{name}: no Rule for check `{check_name}`"));
                seen.insert(check_name.to_string());
                // The Rule exists in exactly one built-in set (in any of its versions).
                let mut holders: Vec<&str> = lib
                    .sets()
                    .iter()
                    .filter(|s| s.rule(id).is_some())
                    .map(|s| s.id.as_str())
                    .collect();
                holders.dedup();
                assert_eq!(holders.len(), 1, "{name}: {id}");
                // Its rows agree with the check: a pass when the check holds, else not a pass.
                let ok = check["ok"].as_bool().unwrap();
                let mine: Vec<&Value> = rows.iter().filter(|r| r["rule"] == id).collect();
                // A file saved before a Rule existed stores the older version of its set: a check
                // that a later version of the set mirrors (issue #280: the aft end space) has no
                // row there, and the stored set is the one the file asked for.
                let stored = results["rule_trace"]["rule_sets"].as_array().unwrap();
                let in_stack = stored.iter().any(|k| {
                    k.as_str()
                        .and_then(|k| k.split_once('@'))
                        .and_then(|(i, v)| lib.find(i, v))
                        .is_some_and(|set| set.rule(id).is_some())
                });
                if !in_stack {
                    continue;
                }
                assert!(!mine.is_empty(), "{name}: no row for {id}");
                if id == "is2008/A.2.2/condition" {
                    // A note: the check's own verdict, one row per loading condition.
                    let cond = check_name.trim_start_matches("Stability \u{2014} ");
                    let found = mine.iter().find(|r| {
                        r["object"]
                            .as_str()
                            .is_some_and(|o| o.to_lowercase() == cond)
                    });
                    // The dry bulk loading conditions stay out of the Rule Trace until a dry bulk
                    // Rule Set exists (issue #426, part 2): their checks are warnings with no row.
                    if found.is_none() && results["phase"] == "dry_bulk" {
                        continue;
                    }
                    let row = found.unwrap_or_else(|| panic!("{name}: no row for {cond}"));
                    assert_eq!(row["value"], json!(ok), "{name}: {check_name}");
                    continue;
                }
                for row in mine {
                    assert_eq!(
                        row["outcome"] == "pass",
                        ok,
                        "{name}: {check_name} {}",
                        row["outcome"]
                    );
                }
            }
        }
    }
    // The corpus reaches the checks a `.vdesign` run makes; a missing one means a weak corpus.
    for must in [
        "Cargo > 0",
        "feasible geometry",
        "air draft within limit",
        "intact gm at reported cargo",
        "aft end bulkhead space",
        "wheelhouse sees over cargo",
        "Feasible arrangement",
    ] {
        assert!(
            seen.contains(must),
            "the corpus never reaches `{must}`: {seen:?}"
        );
    }
}

#[test]
fn every_table_entry_names_a_rule_that_mirrors_that_check() {
    let lib = Library::built_in();
    for (check, id) in CHECK_RULES {
        let rule = lib
            .sets()
            .iter()
            .find_map(|s| s.rule(id))
            .unwrap_or_else(|| panic!("no Rule {id}"));
        if rule.measure.id == "check.ok" {
            let arg = &rule.measure.args[0];
            assert_eq!(arg.0, "name");
            assert_eq!(
                arg.1,
                ovcd_core::rules::ArgValue::Text(check.to_string()),
                "{id}"
            );
        }
    }
}

#[test]
fn ovcd_checks_are_never_shown_as_sourced() {
    // Sourced means a `regulation` or `class` origin and a verification record. An OVCD Check has
    // neither; the method-validity Rules stay unverified until their method is in the library.
    let lib = Library::built_in();
    for set in lib.sets() {
        for r in &set.rules {
            if !r.origin.is_published() {
                assert!(
                    r.verified.is_none(),
                    "{} is an OVCD Check with a verification record",
                    r.id
                );
            }
            if r.verified.is_some() {
                assert!(
                    r.origin.is_published() && !r.source.locator.trim().is_empty(),
                    "{}",
                    r.id
                );
            }
        }
    }
    let ovcd = lib.find("ovcd", "1.0.0").unwrap();
    assert!(
        ovcd.rules
            .iter()
            .all(|r| r.family() == "ovcd" && !r.origin.is_published())
    );
    assert!(
        ovcd.rules
            .iter()
            .any(|r| r.origin == Origin::MethodValidity)
    );
}

#[test]
fn the_separation_rules_keep_the_codes_tolerance_and_findings() {
    let lib = Library::built_in();
    let lr = lib.find("lr-inland-gas", "1.0.0").unwrap();
    for r in &lr.rules {
        assert_eq!(r.tolerance, 1e-9, "{}", r.id);
    }
    assert_eq!(
        lr.rule("lr-inland/3.6.13/cofferdam")
            .unwrap()
            .finding
            .as_deref(),
        Some("#280")
    );
    // #281 is fixed: the space follows the tank end shape, as 3.5.3 does.
    assert_eq!(lr.rule("lr-inland/3.5.3/end-space").unwrap().finding, None);
    // The limits are the code's constants.
    use ovcd_core::gas_separation::{COFFERDAM_MIN_M, OPENING_SETBACK_M};
    let constant = |id: &str| match &lr.rule(id).unwrap().limit {
        Some(ovcd_core::rules::Limit::Constant { value, .. }) => *value,
        _ => panic!("{id}"),
    };
    assert_eq!(constant("lr-inland/3.6.13/cofferdam"), COFFERDAM_MIN_M);
    assert_eq!(
        constant("lr-inland/3.3.7/opening-setback"),
        OPENING_SETBACK_M
    );
}

#[test]
fn rules_show_lists_the_unverified_count() {
    let lib = Library::built_in();
    for id in ["lr-inland-gas", "ovcd"] {
        let set = lib.lookup(id).unwrap();
        let unverified = set.rules.iter().filter(|r| r.verified.is_none()).count();
        assert!(
            describe(set).contains(&format!(
                "{} Rules, {unverified} unverified",
                set.rules.len()
            )),
            "{id}"
        );
    }
}
