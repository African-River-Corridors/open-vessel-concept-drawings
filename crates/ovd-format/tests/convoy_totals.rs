//! A pushed convoy has results (issue #358): one barge, and the totals of every barge, with the
//! overall size checked against the lock of the waterway. The pusher is not in the totals yet.
//!
//! Label: property

use ovd_format::{DesignSession, HashStatus, Profile, RunOutcome};
use serde_json::{Value, json};

fn raw(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The results of the convoy fixture with `abreast` x `in_line` barges and `changes` typed.
fn results(abreast: u64, in_line: u64, changes: &[(&str, Value)]) -> Value {
    let mut f: Value = serde_json::from_str(&raw("pushed_convoy")).unwrap();
    f["options"][0]["inputs"]["formation"] = json!({
        "value": { "kind": "pushed_convoy", "abreast": abreast, "in_line": in_line },
        "source": "typed",
    });
    for (name, value) in changes {
        f["options"][0]["inputs"][*name] = json!({ "value": value, "source": "typed" });
    }
    let (session, _) = DesignSession::open(&f.to_string(), Profile::built_in()).unwrap();
    match session.evaluate().options[0].outcome.as_ref().unwrap() {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

fn num(v: &Value, path: &[&str]) -> f64 {
    let mut cur = v;
    for key in path {
        cur = &cur[*key];
    }
    cur.as_f64().unwrap_or_else(|| panic!("{path:?} in {cur}"))
}

#[test]
fn a_two_by_three_convoys_totals_are_six_times_one_barges() {
    let r = results(2, 3, &[]);
    let one = results(1, 1, &[]);
    let c = &r["convoy"];
    assert_eq!(c["barges"], 6);
    for key in [
        "cargo_t",
        "lightship_t",
        "deadweight_t",
        "displacement_t",
        "cargo_volume_m3",
    ] {
        // The barge of the convoy is the barge alone.
        assert_eq!(
            num(c, &["barge", key]),
            num(&one, &["convoy", "barge", key]),
            "{key}"
        );
        assert_eq!(
            num(c, &["totals", key]),
            6.0 * num(c, &["barge", key]),
            "{key}"
        );
    }
    // The overall size: barges in line x length, barges abreast x beam.
    assert_eq!(
        num(c, &["totals", "overall_length_m"]),
        3.0 * num(&r, &["dims", "loa_m"])
    );
    assert_eq!(
        num(c, &["totals", "overall_beam_m"]),
        2.0 * num(&r, &["dims", "beam_m"])
    );
    assert_eq!(c["pusher_included"], json!(false));
    // The results of the Option are the barge's own.
    assert_eq!(
        num(&r, &["results", "cargo_t"]),
        num(c, &["barge", "cargo_t"])
    );
}

#[test]
fn the_fixture_gives_results_and_not_the_old_message() {
    let (session, _) = DesignSession::open(&raw("pushed_convoy"), Profile::built_in()).unwrap();
    let evaluation = session.evaluate();
    assert!(matches!(
        evaluation.options[0].outcome,
        Some(RunOutcome::Results { .. })
    ));
    let (_, opened) = DesignSession::open(&raw("pushed_convoy"), Profile::built_in()).unwrap();
    assert_ne!(opened.hash_status[0].1, HashStatus::NotChecked);
}

#[test]
fn the_overall_size_is_checked_against_the_lock() {
    let find = |r: &Value, name: &str| {
        r["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == name)
            .cloned()
    };
    // No lock typed: no lock check.
    let none = results(2, 3, &[]);
    assert!(find(&none, "within lock length").is_none());
    assert!(find(&none, "within lock width").is_none());
    // 3 x 76.5 m = 229.5 m long and 2 x 11.4 m = 22.8 m wide.
    let wide = results(
        2,
        3,
        &[
            ("lock_length_m", json!(230.0)),
            ("lock_width_m", json!(22.8)),
        ],
    );
    assert_eq!(find(&wide, "within lock length").unwrap()["ok"], true);
    assert_eq!(find(&wide, "within lock width").unwrap()["ok"], true);
    let tight = results(
        2,
        3,
        &[
            ("lock_length_m", json!(229.0)),
            ("lock_width_m", json!(22.0)),
        ],
    );
    let len = find(&tight, "within lock length").unwrap();
    assert_eq!(len["ok"], false);
    assert_eq!(
        len["message"],
        "overall length 229.50 m <= lock length 229.0 m"
    );
    assert_eq!(find(&tight, "within lock width").unwrap()["ok"], false);
    // Each new text has a Message Id.
    assert_eq!(
        tight["text_ids"]["overall length 229.50 m <= lock length 229.0 m"]["id"],
        "check-lock-length"
    );
    assert_eq!(
        tight["text_ids"]["within lock width"]["id"],
        "name-check-lock-width"
    );
}

/// The results of a crude oil tank barge fixture run as a pushed convoy of `abreast` x `in_line`.
fn crude_convoy(abreast: u64, in_line: u64) -> Value {
    let mut f: Value = serde_json::from_str(&raw("crude_integral_tanks_pushed")).unwrap();
    f["options"][0]["inputs"]["formation"] = json!({
        "value": { "kind": "pushed_convoy", "abreast": abreast, "in_line": in_line },
        "source": "typed",
    });
    let (session, _) = DesignSession::open(&f.to_string(), Profile::built_in()).unwrap();
    match session.evaluate().options[0].outcome.as_ref().unwrap() {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

/// Issue #552. Label: property: the budget of a 2 x 3 convoy is that of six barges and one tug, and
/// the budget of one barge is still given.
#[test]
fn a_two_by_three_convoys_budget_is_six_barges_and_one_tug() {
    let r = crude_convoy(2, 3);
    let one = crude_convoy(1, 1);
    let (c, o) = (&r["cost"], &one["cost"]);
    assert!(c.is_object(), "the crude oil convoy has a budget");
    assert_eq!(c["barges"], 6);
    // The barge items and their subtotal are six times one barge's.
    assert_eq!(
        num(c, &["barge_subtotal"]),
        6.0 * num(o, &["barge_subtotal"])
    );
    let (items, one_items) = (
        c["barge"].as_array().unwrap(),
        o["barge"].as_array().unwrap(),
    );
    assert_eq!(items.len(), one_items.len());
    for (a, b) in items.iter().zip(one_items) {
        assert_eq!(a["item"], b["item"]);
        assert_eq!(
            num(a, &["amount"]),
            6.0 * num(b, &["amount"]),
            "{}",
            a["item"]
        );
    }
    // One tug for the convoy, and the margin and the total follow from the new subtotals.
    assert_eq!(c["tug"], o["tug"]);
    assert_eq!(c["tug_subtotal"], o["tug_subtotal"]);
    let margin = (num(c, &["yard_margin_fraction"])
        * (num(c, &["barge_subtotal"]) + num(c, &["tug_subtotal"])))
    .round_ties_even();
    assert_eq!(num(c, &["yard_margin"]), margin);
    assert_eq!(
        num(c, &["unit_total"]),
        num(c, &["barge_subtotal"]) + num(c, &["tug_subtotal"]) + margin + num(c, &["delivery"])
    );
    // The cost of one barge is kept, as the budget of a convoy of one barge gives it.
    assert_eq!(c["one_barge"]["barge_subtotal"], o["barge_subtotal"]);
    assert_eq!(c["one_barge"]["unit_total"], o["unit_total"]);
    // A convoy of one barge is that barge's budget.
    assert_eq!(o["barges"], 1);
    assert_eq!(o["unit_total"], o["one_barge"]["unit_total"]);
}

/// Issue #552. Label: property: a pushed barge, which is no convoy, keeps its budget as it was.
#[test]
fn a_pushed_barge_has_no_convoy_budget_keys() {
    let mut f: Value = serde_json::from_str(&raw("crude_integral_tanks_pushed")).unwrap();
    let (session, _) = DesignSession::open(&f.to_string(), Profile::built_in()).unwrap();
    f["x"] = Value::Null;
    match session.evaluate().options[0].outcome.as_ref().unwrap() {
        RunOutcome::Results { results, .. } => {
            assert!(results["cost"].is_object());
            assert!(results["cost"].get("one_barge").is_none());
            assert!(results["cost"].get("barges").is_none());
        }
        other => panic!("{other:?}"),
    }
}
