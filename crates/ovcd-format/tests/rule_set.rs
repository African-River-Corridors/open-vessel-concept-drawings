//! Seam: the Rule Set loader, the content hash and the selection checks (spec 0004, issue #268).
//! Each refusal reason has a minimal bad file: the example set with one change.

use ovcd_format::rule_set::{Library, content_hash, describe, is_rule_id, load};
use serde_json::{Value, json};

const EXAMPLE: &str = include_str!("../rules/examples/example-1.0.0.json");

fn example() -> Value {
    serde_json::from_str(EXAMPLE).unwrap()
}

fn text(v: &Value) -> String {
    serde_json::to_string_pretty(v).unwrap()
}

/// The example with one change, which the loader must refuse with a message holding `expect`.
fn refused(change: impl FnOnce(&mut Value), expect: &str) {
    let mut v = example();
    change(&mut v);
    match load(&text(&v)) {
        Ok(_) => panic!("loaded, but expected a refusal with `{expect}`"),
        Err(e) => assert!(e.contains(expect), "message `{e}` lacks `{expect}`"),
    }
}

fn rule(v: &mut Value, i: usize) -> &mut serde_json::Map<String, Value> {
    v["rules"][i].as_object_mut().unwrap()
}

/// A minimal set with the given id, version, includes and rules (all OVCD Checks).
fn small_set(id: &str, version: &str, includes: Value, rules: Value) -> String {
    text(&json!({
        "id": id, "family": id, "title": "t", "version": version,
        "edition": {"document": "d"}, "sources": [], "status": "draft",
        "includes": includes, "rules": rules,
    }))
}

fn limit_rule(id: &str, measure: &str, compare: &str, value: f64) -> Value {
    json!({
        "id": id, "title": "t", "origin": "physical", "source": {},
        "codified": {"by": "agent", "date": "2026-10-20", "review": "#268"},
        "verified": false, "kind": "limit", "measure": {"id": measure},
        "compare": compare, "limit": {"value": value, "unit": "m"},
        "consequence": "warn", "evaluated": true, "since": "1.0.0",
    })
}

#[test]
fn the_example_set_loads() {
    let loaded = load(EXAMPLE).unwrap();
    let set = &loaded.set;
    assert_eq!(set.key(), "example@1.0.0");
    assert_eq!(set.rules.len(), 6);
    assert_eq!(set.hash.len(), 64);
    // Blocking and unverified: a warning each, never a refusal.
    assert_eq!(loaded.warnings.len(), 2, "{:?}", loaded.warnings);
    assert!(loaded.warnings[0].contains("example/1.1/area-0-30"));
}

#[test]
fn the_rule_id_grammar() {
    for ok in [
        "is2008/A.2.2.1/area-0-30",
        "adn/9.3.1.11.1(b)/tank-length",
        "ovcd/gas-arrangement/tank-l-over-d",
        "lr-inland/3.5.3/end-space-plane",
    ] {
        assert!(is_rule_id(ok), "{ok}");
    }
    for bad in [
        "",
        "is2008/A.2.2.1",
        "is2008/A.2.2.1/area/0",
        "IS2008/A.2.2.1/area",
        "is2008/.2.2.1/area",
        "is2008/A 2.2.1/area",
        "is2008/A.2.2.1/Area",
        "is2008/A.2.2.1/-area",
        "2008/A.2.2.1/area",
    ] {
        assert!(!is_rule_id(bad), "{bad}");
    }
}

#[test]
fn a_missing_field_is_refused() {
    refused(
        |v| drop(v.as_object_mut().unwrap().remove("family")),
        "`family` is missing",
    );
    refused(
        |v| drop(rule(v, 0).remove("consequence")),
        "consequence` is missing",
    );
    refused(|v| drop(rule(v, 0).remove("limit")), "limit` is missing");
    refused(|v| drop(rule(v, 4).remove("gap")), "gap` is missing");
}

#[test]
fn an_unknown_field_is_refused() {
    refused(
        |v| drop(rule(v, 0).insert("override".into(), json!(true))),
        "unknown field `override`",
    );
}

#[test]
fn a_malformed_rule_id_is_refused() {
    refused(
        |v| drop(rule(v, 0).insert("id".into(), json!("example/1.1"))),
        "rule id grammar",
    );
}

#[test]
fn a_repeated_rule_id_is_refused() {
    refused(
        |v| {
            let copy = v["rules"][0].clone();
            v["rules"].as_array_mut().unwrap().push(copy);
        },
        "is used twice",
    );
}

#[test]
fn a_rule_of_another_family_is_refused() {
    refused(
        |v| drop(rule(v, 3).insert("id".into(), json!("other/gas-arrangement/weld-factor"))),
        "is not the set's family",
    );
}

#[test]
fn a_regulation_rule_with_no_paragraph_is_refused() {
    refused(
        |v| {
            drop(
                v["rules"][0]["source"]
                    .as_object_mut()
                    .unwrap()
                    .remove("locator"),
            )
        },
        "must cite its paragraph",
    );
}

#[test]
fn a_paragraph_that_does_not_match_the_locator_is_refused() {
    refused(
        |v| v["rules"][0]["source"]["locator"] = json!("1.2"),
        "does not match the locator",
    );
    // Spaces are removed before the comparison.
    let mut v = example();
    v["rules"][0]["source"]["locator"] = json!(" 1.1 ");
    assert!(load(&text(&v)).is_ok());
}

#[test]
fn a_non_finite_limit_or_coefficient_is_refused() {
    // JSON has no NaN or infinity, but a number out of range reads as an infinity.
    let bad = EXAMPLE.replacen("\"value\": 0.055", "\"value\": 1e999", 1);
    let e = load(&bad).unwrap_err();
    assert!(e.contains("value` must be a finite number"), "{e}");
    let bad = EXAMPLE.replacen("\"b\": 0.2", "\"b\": -1e999", 1);
    let e = load(&bad).unwrap_err();
    assert!(e.contains("b` must be a finite number"), "{e}");
    refused(
        |v| v["rules"][0]["limit"]["value"] = json!("0.055"),
        "must be a number",
    );
}

#[test]
fn an_unknown_table_is_refused() {
    refused(
        |v| v["rules"][1]["limit"]["table"] = json!("nope"),
        "names table `nope`",
    );
    refused(
        |v| v["rules"][2]["limit"]["of"]["table"] = json!("nope"),
        "names table `nope`",
    );
}

#[test]
fn a_check_or_note_rule_with_a_limit_is_refused() {
    refused(
        |v| drop(rule(v, 4).insert("compare".into(), json!("ge"))),
        "has no comparison and no limit",
    );
}

#[test]
fn a_note_consequence_only_goes_with_a_note_rule() {
    refused(
        |v| drop(rule(v, 0).insert("consequence".into(), json!("note"))),
        "consequence `note`",
    );
    refused(
        |v| drop(rule(v, 5).insert("consequence".into(), json!("warn"))),
        "consequence `note`",
    );
}

#[test]
fn a_person_is_never_named() {
    refused(
        |v| v["rules"][0]["codified"]["by"] = json!("A. Person"),
        "never a person's name",
    );
    refused(
        |v| v["rules"][1]["verified"]["by"] = json!("A. Person"),
        "never a person's name",
    );
}

#[test]
fn an_undeclared_conflict_of_two_constant_criteria_is_refused() {
    let low = limit_rule("c/x/low", "hull.depth", "ge", 2.0);
    let high = limit_rule("c/x/high", "hull.depth", "le", 1.0);
    let set = small_set("c", "1.0.0", json!([]), json!([low, high]));
    assert!(load(&set).unwrap_err().contains("cannot both be met"));
    // Equal bounds: met by exactly the limit when both are non-strict, never when one is strict.
    let eq = small_set(
        "c",
        "1.0.0",
        json!([]),
        json!([
            limit_rule("c/x/low", "hull.depth", "ge", 1.0),
            limit_rule("c/x/high", "hull.depth", "le", 1.0)
        ]),
    );
    assert!(load(&eq).is_ok());
    let strict = small_set(
        "c",
        "1.0.0",
        json!([]),
        json!([
            limit_rule("c/x/low", "hull.depth", "gt", 1.0),
            limit_rule("c/x/high", "hull.depth", "le", 1.0)
        ]),
    );
    assert!(load(&strict).unwrap_err().contains("cannot both be met"));
    // Another Measure, or Applicabilities that never meet, is no conflict.
    let other = small_set(
        "c",
        "1.0.0",
        json!([]),
        json!([low.clone(), limit_rule("c/x/high", "hull.beam", "le", 1.0)]),
    );
    assert!(load(&other).is_ok());
    let mut apart = high.clone();
    apart["applies_when"] = json!({"cargo": ["LNG"]});
    let mut low_apart = low.clone();
    low_apart["applies_when"] = json!({"cargo": ["bauxite"]});
    assert!(
        load(&small_set(
            "c",
            "1.0.0",
            json!([]),
            json!([low_apart, apart])
        ))
        .is_ok()
    );
    // Declared with a reason: loaded.
    let mut v: Value = serde_json::from_str(&set).unwrap();
    v["known_conflicts"] = json!([{"rules": ["c/x/high", "c/x/low"], "reason": "test"}]);
    assert!(load(&text(&v)).is_ok());
}

#[test]
fn the_tolerance_is_part_of_the_conflict_check() {
    let mut low = limit_rule("c/x/low", "hull.depth", "ge", 1.0 + 1e-9);
    low["tolerance"] = json!(1e-9);
    let high = limit_rule("c/x/high", "hull.depth", "le", 1.0);
    // Effective lower bound (1 + 1e-9) - 1e-9 is not above 1.0.
    assert!(load(&small_set("c", "1.0.0", json!([]), json!([low, high]))).is_ok());
    refused(
        |v| v["rules"][0]["tolerance"] = json!(-1.0),
        "must not be negative",
    );
}

#[test]
fn the_content_hash_ignores_titles_citations_and_verification() {
    let base = load(EXAMPLE).unwrap().set.hash;
    for change in [
        |v: &mut Value| v["title"] = json!("Another title"),
        |v: &mut Value| v["rules"][0]["title"] = json!("Another title"),
        |v: &mut Value| v["rules"][0]["source"]["page"] = json!("p. 99"),
        |v: &mut Value| v["rules"][0]["codified"]["date"] = json!("2027-01-01"),
        |v: &mut Value| v["rules"][0]["verified"] = json!({"by": "expert-reviewer", "date": "d", "edition_held": "e", "locator_read": "l", "review": "#1"}),
        |v: &mut Value| v["rules"][4]["gap"] = json!("another gap text"),
        |v: &mut Value| v["rules"][5]["finding"] = json!("#1"),
        |v: &mut Value| v["tables"][0]["source"]["locator"] = json!("2.9"),
        |v: &mut Value| v["version"] = json!("1.0.1"),
    ] {
        let mut v = example();
        change(&mut v);
        assert_eq!(load(&text(&v)).unwrap().set.hash, base);
    }
}

#[test]
fn the_content_hash_changes_with_a_result_changing_field() {
    let base = load(EXAMPLE).unwrap().set.hash;
    for change in [
        |v: &mut Value| v["rules"][0]["limit"]["value"] = json!(0.056),
        |v: &mut Value| v["rules"][0]["compare"] = json!("gt"),
        |v: &mut Value| v["rules"][0]["tolerance"] = json!(1e-9),
        |v: &mut Value| v["rules"][0]["consequence"] = json!("warn"),
        |v: &mut Value| v["rules"][0]["sizes"] = json!(true),
        |v: &mut Value| v["rules"][0]["measure"]["args"]["to_deg"] = json!(40),
        |v: &mut Value| v["rules"][0]["applies_when"] = json!({"cargo": ["LNG"]}),
        |v: &mut Value| v["rules"][0]["id"] = json!("example/1.1/area-0-30b"),
        |v: &mut Value| v["applies_when"] = json!({}),
        |v: &mut Value| v["tables"][0]["rows"][0]["a"] = json!(0.81),
        |v: &mut Value| v["rules"][2]["limit"]["less"] = json!("input.other"),
        |v: &mut Value| {
            v["rules"][4]["evaluated"] = json!(true);
            v["rules"][4].as_object_mut().unwrap().remove("gap");
        },
    ] {
        let mut v = example();
        change(&mut v);
        let hash = load(&text(&v)).unwrap().set.hash;
        assert_ne!(hash, base);
        assert_eq!(hash, content_hash(&load(&text(&v)).unwrap().set));
    }
}

#[test]
fn the_same_version_with_other_content_is_refused() {
    let mut lib = Library::built_in();
    assert_eq!(lib.add_text(EXAMPLE).unwrap(), "example@1.0.0");
    // The same file again is the same set.
    lib.add_text(EXAMPLE).unwrap();
    assert_eq!(lib.sets().len(), 1);
    // A patch with only a new title is the same set too.
    let mut v = example();
    v["title"] = json!("Retitled");
    lib.add_text(&text(&v)).unwrap();
    let mut v = example();
    v["rules"][0]["limit"]["value"] = json!(0.06);
    assert!(
        lib.add_text(&text(&v))
            .unwrap_err()
            .contains("needs a new version")
    );
    // A new version is a separate set.
    v["version"] = json!("2.0.0");
    lib.add_text(&text(&v)).unwrap();
    assert_eq!(lib.lookup("example").unwrap().version, "2.0.0");
    assert_eq!(lib.lookup("example@1.0.0").unwrap().version, "1.0.0");
}

#[test]
fn an_unknown_includes_target_is_refused() {
    let mut lib = Library::default();
    let a = small_set(
        "a",
        "1.0.0",
        json!([{"id": "b", "version": "1.0.0"}]),
        json!([]),
    );
    lib.add_text(&a).unwrap();
    assert!(
        lib.check()
            .unwrap_err()
            .contains("includes b@1.0.0, which is not loaded")
    );
    assert!(
        lib.select(&[("a", "1.0.0")])
            .unwrap_err()
            .contains("which is not loaded")
    );
    // Loaded later, in any order: fine.
    lib.add_text(&small_set("b", "1.0.0", json!([]), json!([])))
        .unwrap();
    lib.check().unwrap();
}

#[test]
fn selection_closes_over_includes_in_order() {
    let mut lib = Library::default();
    lib.add_text(&small_set("c", "1.0.0", json!([]), json!([])))
        .unwrap();
    lib.add_text(&small_set(
        "b",
        "1.0.0",
        json!([{"id": "c", "version": "1.0.0"}]),
        json!([]),
    ))
    .unwrap();
    lib.add_text(&small_set(
        "a",
        "1.0.0",
        json!([{"id": "b", "version": "1.0.0"}]),
        json!([]),
    ))
    .unwrap();
    lib.add_text(&small_set("d", "1.0.0", json!([]), json!([])))
        .unwrap();
    let sel = lib
        .select(&[("a", "1.0.0"), ("d", "1.0.0"), ("c", "1.0.0")])
        .unwrap();
    let keys: Vec<String> = sel.sets.iter().map(|s| s.key()).collect();
    assert_eq!(keys, ["a@1.0.0", "b@1.0.0", "c@1.0.0", "d@1.0.0"]);
}

#[test]
fn an_include_cycle_is_refused() {
    let mut lib = Library::default();
    lib.add_text(&small_set(
        "a",
        "1.0.0",
        json!([{"id": "b", "version": "1.0.0"}]),
        json!([]),
    ))
    .unwrap();
    lib.add_text(&small_set(
        "b",
        "1.0.0",
        json!([{"id": "c", "version": "1.0.0"}]),
        json!([]),
    ))
    .unwrap();
    lib.add_text(&small_set(
        "c",
        "1.0.0",
        json!([{"id": "a", "version": "1.0.0"}]),
        json!([]),
    ))
    .unwrap();
    let e = lib.select(&[("a", "1.0.0")]).unwrap_err();
    assert!(
        e.contains("a@1.0.0 -> b@1.0.0 -> c@1.0.0 -> a@1.0.0"),
        "{e}"
    );
    assert!(lib.check().is_err());
}

#[test]
fn two_versions_of_one_set_are_refused() {
    let mut lib = Library::default();
    lib.add_text(&small_set("a", "1.0.0", json!([]), json!([])))
        .unwrap();
    lib.add_text(&small_set("a", "1.1.0", json!([]), json!([])))
        .unwrap();
    lib.add_text(&small_set(
        "b",
        "1.0.0",
        json!([{"id": "a", "version": "1.0.0"}]),
        json!([]),
    ))
    .unwrap();
    let e = lib.select(&[("a", "1.1.0"), ("b", "1.0.0")]).unwrap_err();
    assert!(e.contains("a@1.1.0") && e.contains("a@1.0.0"), "{e}");
}

#[test]
fn one_rule_id_with_two_meanings_is_refused_and_with_one_is_shared() {
    let r = |v: f64| {
        let mut r = limit_rule("x/p/r", "hull.depth", "ge", v);
        r["id"] = json!("x/p/r");
        r
    };
    let set = |id: &str, rule: Value| {
        let mut v: Value =
            serde_json::from_str(&small_set(id, "1.0.0", json!([]), json!([rule]))).unwrap();
        v["family"] = json!("x");
        text(&v)
    };
    let mut lib = Library::default();
    lib.add_text(&set("a", r(1.0))).unwrap();
    lib.add_text(&set("b", r(1.0))).unwrap();
    lib.add_text(&set("c", r(2.0))).unwrap();
    assert!(lib.select(&[("a", "1.0.0"), ("b", "1.0.0")]).is_ok());
    let e = lib.select(&[("a", "1.0.0"), ("c", "1.0.0")]).unwrap_err();
    assert!(e.contains("means different things"), "{e}");
}

#[test]
fn a_conflict_across_two_sets_is_a_warning() {
    let mut lib = Library::default();
    lib.add_text(&small_set(
        "a",
        "1.0.0",
        json!([]),
        json!([limit_rule("a/x/low", "hull.depth", "ge", 2.0)]),
    ))
    .unwrap();
    lib.add_text(&small_set(
        "b",
        "1.0.0",
        json!([]),
        json!([limit_rule("b/x/high", "hull.depth", "le", 1.0)]),
    ))
    .unwrap();
    let sel = lib.select(&[("a", "1.0.0"), ("b", "1.0.0")]).unwrap();
    assert_eq!(sel.warnings.len(), 1, "{:?}", sel.warnings);
    assert!(sel.warnings[0].contains("cannot both be met"));
}

#[test]
fn describe_lists_every_rule_with_its_status() {
    let set = load(EXAMPLE).unwrap().set;
    let d = describe(&set);
    for r in &set.rules {
        assert!(d.contains(&r.id), "{}", r.id);
    }
    assert!(d.contains("6 Rules, 5 unverified"));
    assert!(d.contains("verified by agent on 2026-10-20"));
    assert!(d.contains("not evaluated"));
}

#[test]
fn the_built_in_sets_load() {
    // None yet: the first come with the migration tickets. Every one must load and select.
    let lib = Library::built_in();
    for s in lib.sets() {
        lib.select(&[(&s.id, &s.version)]).unwrap();
    }
}
