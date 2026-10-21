//! Seam: released Rule Set versions, version classes, rule ids across versions and the provenance
//! gate (spec 0004, issue #273).

use ovcd_format::rule_set::{
    Library, RELEASED, Released, VersionClass, check_released, check_versions, gate, load,
    provenance, released, version_class,
};
use serde_json::{Value, json};

const EXAMPLE: &str = include_str!("../rules/examples/example-1.0.0.json");

fn example() -> Value {
    serde_json::from_str(EXAMPLE).unwrap()
}

fn with_version(change: impl FnOnce(&mut Value), version: &str) -> String {
    let mut v = example();
    change(&mut v);
    v["version"] = json!(version);
    v.to_string()
}

#[test]
fn the_built_in_sets_pass_the_gate() {
    let (errors, report) = gate(&Library::built_in());
    assert!(errors.is_empty(), "{errors:?}");
    for id in [
        "is2008-a@1.0.0",
        "igc-2g@1.0.0",
        "lr-inland-gas@1.0.0",
        "ovcd@1.0.0",
    ] {
        assert!(report.contains(id), "{report}");
    }
    released(RELEASED).unwrap();
}

#[test]
fn changing_a_number_in_a_released_set_fails() {
    let original = load(EXAMPLE).unwrap().set;
    let list = vec![Released {
        id: original.id.clone(),
        version: original.version.clone(),
        hash: original.hash.clone(),
    }];
    let mut lib = Library::default();
    lib.add_text(EXAMPLE).unwrap();
    assert!(check_released(&lib, &list).is_empty());
    // The same file with one limit moved: the released version no longer matches its hash.
    let mut v = example();
    v["rules"][0]["limit"]["value"] = json!(0.056);
    let mut lib = Library::default();
    lib.add_text(&v.to_string()).unwrap();
    let errors = check_released(&lib, &list);
    assert_eq!(errors.len(), 1);
    assert!(
        errors[0].contains("a released file never changes"),
        "{errors:?}"
    );
    // A released version that is gone fails too.
    assert!(check_released(&Library::default(), &list)[0].contains("not built in"));
}

#[test]
fn a_changed_citation_keeps_the_hash_so_the_released_version_still_matches() {
    let original = load(EXAMPLE).unwrap().set;
    let list = vec![Released {
        id: original.id.clone(),
        version: original.version.clone(),
        hash: original.hash.clone(),
    }];
    let mut v = example();
    v["rules"][0]["source"]["page"] = json!("p. 99");
    v["rules"][0]["verified"] = json!({"by": "expert-reviewer", "date": "d", "edition_held": "e", "locator_read": "l", "review": "#1"});
    let mut lib = Library::default();
    lib.add_text(&v.to_string()).unwrap();
    assert!(check_released(&lib, &list).is_empty());
}

fn rule_copy(v: &Value, i: usize, id: &str) -> Value {
    let mut r = v["rules"][i].clone();
    r["id"] = json!(id);
    r
}

#[test]
fn version_classes_follow_the_content_hash() {
    let base = load(EXAMPLE).unwrap().set;
    let class = |text: String| version_class(&base, &load(&text).unwrap().set);
    // A title only: the same hash, a patch.
    assert_eq!(
        class(with_version(|v| v["title"] = json!("x"), "1.0.1")),
        VersionClass::Patch
    );
    // A Rule added: minor.
    let added = with_version(
        |v| {
            let r = rule_copy(v, 3, "example/gas-arrangement/fill-ratio-2");
            v["rules"].as_array_mut().unwrap().push(r);
        },
        "1.1.0",
    );
    assert_eq!(class(added), VersionClass::Minor);
    // A limit moved, or a Rule removed: major.
    assert_eq!(
        class(with_version(
            |v| v["rules"][0]["limit"]["value"] = json!(0.06),
            "2.0.0"
        )),
        VersionClass::Major
    );
    assert_eq!(
        class(with_version(
            |v| drop(v["rules"].as_array_mut().unwrap().remove(3)),
            "2.0.0"
        )),
        VersionClass::Major
    );
    // A table row moved: major.
    assert_eq!(
        class(with_version(
            |v| v["tables"][0]["rows"][0]["a"] = json!(0.31),
            "2.0.0"
        )),
        VersionClass::Major
    );
}

fn library(texts: &[String]) -> Library {
    let mut lib = Library::default();
    for t in texts {
        lib.add_text(t).unwrap();
    }
    lib
}

#[test]
fn a_version_number_must_go_up_by_its_class() {
    let moved =
        |version: &str| with_version(|v| v["rules"][0]["limit"]["value"] = json!(0.06), version);
    // A changed limit as a minor or patch version: refused.
    for version in ["1.0.1", "1.1.0"] {
        let errors = check_versions(&library(&[EXAMPLE.to_string(), moved(version)]));
        assert_eq!(errors.len(), 1, "{version}: {errors:?}");
        assert!(errors[0].contains("Major"), "{errors:?}");
    }
    assert!(check_versions(&library(&[EXAMPLE.to_string(), moved("2.0.0")])).is_empty());
    // An added Rule as a patch: refused; as a minor: fine.
    let add = |version: &str| {
        with_version(
            |v| {
                let r = rule_copy(v, 3, "example/gas-arrangement/fill-ratio-2");
                v["rules"].as_array_mut().unwrap().push(r);
            },
            version,
        )
    };
    assert_eq!(
        check_versions(&library(&[EXAMPLE.to_string(), add("1.0.1")])).len(),
        1
    );
    assert!(check_versions(&library(&[EXAMPLE.to_string(), add("1.1.0")])).is_empty());
    // A citation only, as a patch: fine.
    assert!(
        check_versions(&library(&[
            EXAMPLE.to_string(),
            with_version(|v| v["title"] = json!("x"), "1.0.1")
        ]))
        .is_empty()
    );
}

#[test]
fn a_rule_id_keeps_its_meaning_and_is_never_reused() {
    // The same id on another Measure in a later version: refused, whatever the version number.
    let other_measure = with_version(
        |v| {
            v["rules"][0]["measure"] =
                json!({"id": "gz.area", "args": {"from_deg": 0, "to_deg": 40}})
        },
        "2.0.0",
    );
    let errors = check_versions(&library(&[EXAMPLE.to_string(), other_measure]));
    assert!(
        errors
            .iter()
            .any(|e| e.contains("changes its Measure or direction")),
        "{errors:?}"
    );
    // The same id in the other direction: refused.
    let flipped = with_version(
        |v| {
            v["rules"][0]["compare"] = json!("le");
            v["rules"][0]["limit"]["value"] = json!(5.0);
        },
        "2.0.0",
    );
    let errors = check_versions(&library(&[EXAMPLE.to_string(), flipped]));
    assert!(
        errors
            .iter()
            .any(|e| e.contains("changes its Measure or direction")),
        "{errors:?}"
    );
    // Dropped in 2.0.0, back in 3.0.0: refused.
    let dropped = with_version(
        |v| drop(v["rules"].as_array_mut().unwrap().remove(3)),
        "2.0.0",
    );
    let back = with_version(|_| {}, "3.0.0");
    let errors = check_versions(&library(&[EXAMPLE.to_string(), dropped, back]));
    assert!(
        errors.iter().any(|e| e.contains("is used again")),
        "{errors:?}"
    );
}

#[test]
fn the_provenance_gate_fails_on_a_regulation_rule_with_no_paragraph() {
    // The loader refuses such a file, so the fixture is built after loading.
    let mut set = load(EXAMPLE).unwrap().set;
    set.rules[0].source.locator = String::new();
    let mut lib = Library::default();
    lib.add(ovcd_format::rule_set::Loaded {
        set,
        warnings: Vec::new(),
    })
    .unwrap();
    let (errors, report) = provenance(&lib);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("cites no paragraph"), "{errors:?}");
    // The report counts the unverified blocking Rules and names the published ones.
    assert!(report.contains("example@1.0.0: 2 blocking Rule(s) unverified (0 OVCD Checks): example/1.1/area-0-30, example/1.3/weather"), "{report}");
}
