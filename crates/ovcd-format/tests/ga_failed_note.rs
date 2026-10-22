//! The failed-checks note (issue #214): a long list of failed checks stays one note on one line, so
//! the notes strip numbers 1..N with no gap, and the line fits the strip by the standard's own text
//! width (clauses 11.7.1 and 14.6). Checked on the normalised content.
//!
//! Label: property - structure of the strip and widths by the standard's table; no number is fixed.

use ovcd_draw::entities::Shape;
use ovcd_draw::rules::text_width;
use ovcd_draw::{NoFiles, Normalised, RunResult, run_value};
use ovcd_format::{GaViews, Profile, RunOutcome, check, ga_views_of, run_design};
use serde_json::{Value, json};

fn gas_self_propelled() -> Value {
    gas_hull("LNG", 76.5, 11.4, 3.5)
}

/// A hull on which no arrangement passes every check (issue #264): the closest one is drawn and
/// its failed checks make the note. 60 x 20 x 3.5 m LPG propane fails the stability of the full load
/// and ballast conditions (issue #498: 60 x 10 x 6 m, whose only failure was the angle of max GZ with
/// no ballast, is feasible now, with a warning).
fn gas_nothing_passes() -> Value {
    gas_hull("LPG_propane", 60.0, 20.0, 3.5)
}

fn gas_hull(cargo: &str, loa: f64, beam: f64, depth: f64) -> Value {
    let t = |v: Value| json!({"value": v, "source": "typed"});
    let design = json!({"format": "vdesign", "format_version": 2, "engine_version": "0.2.2",
        "units": "SI", "options": [{"name": "Option 1", "inputs": {
            "cargo": t(json!(cargo)), "hull_form": t(json!("raked_box")),
            "formation": t(json!({"kind": "self_propelled"})), "length_overall_m": t(json!(loa)),
            "beam_m": t(json!(beam)), "moulded_depth_m": t(json!(depth)),
            "cargo_containment": t(json!("type_c_tanks"))}}]});
    let checked = check(&design.to_string(), &Profile::built_in());
    match run_design(&checked.design.expect("a valid Design"))
        .remove(0)
        .outcome
    {
        RunOutcome::Results { results, .. } => results,
        other => panic!("no results: {other:?}"),
    }
}

fn views(results: &Value) -> GaViews {
    ga_views_of(&RunOutcome::Results {
        results: results.clone(),
        hash: String::new(),
    })
    .expect("views")
}

fn normalise(document: &Value) -> Normalised {
    match run_value(document, &NoFiles) {
        RunResult::Ok(n) => *n,
        RunResult::Errors(e) => panic!("the standard refuses the document (clause {e:?})"),
    }
}

/// The note lines of the strip as drawn: text, in order, with the strip's width.
fn note_lines(n: &Normalised) -> Vec<(String, f64)> {
    n.entities
        .iter()
        .filter(|e| e.origin == "sheet.notes")
        .filter_map(|e| match &e.shape {
            Shape::Text { text, height, .. } if (*height - 2.5).abs() < 1e-9 => {
                Some((text.clone(), text_width(text, *height).expect("width")))
            }
            _ => None,
        })
        .collect()
}

fn check_strip(what: &str, document: &Value, failed: usize) {
    let n = normalise(document);
    let lines = note_lines(&n);
    let strip_width = n.sheet.drawing_space.w - 4.0;
    // the numbers run 1..N with no gap, and nothing else is a numbered line
    for (i, (text, width)) in lines.iter().enumerate() {
        assert!(
            text.starts_with(&format!("{}. ", i + 1)),
            "{what}: line {} is '{text}'",
            i + 1
        );
        assert!(*width <= strip_width, "{what}: '{text}' is {width} mm wide");
    }
    let fails: Vec<_> = lines
        .iter()
        .filter(|(t, _)| {
            t.split_once(". ")
                .is_some_and(|(_, r)| r.starts_with("DESIGN FAILS"))
        })
        .collect();
    assert_eq!(fails.len(), usize::from(failed > 0), "{what}: {lines:?}");
    if let Some((text, _)) = fails.first() {
        assert!(
            text.contains(&format!("DESIGN FAILS {failed} CHECK")),
            "{what}: {text}"
        );
    }
    // the notes are the title, what the vessel is, the Rule Sets, the scope and the one
    // failed-checks note
    assert_eq!(
        lines.len(),
        4 + usize::from(failed > 0),
        "{what}: {lines:?}"
    );
}

#[test]
fn the_failed_checks_of_a_hull_where_nothing_passes_make_one_numbered_note() {
    let r = gas_nothing_passes();
    let failed = r["checks"]
        .as_array()
        .unwrap()
        .iter()
        // A failed warning is not a failure (issue #418).
        .filter(|c| c["ok"] == false && c["severity"] != "warning")
        .count();
    // "Feasible arrangement" says plainly that none passes; the failed light stability is listed too.
    assert!(failed >= 2, "{failed} failed checks");
    for (view, doc) in views(&r).all() {
        check_strip(view, doc, failed);
    }
}

#[test]
fn the_lng_self_propelled_case_now_passes_and_makes_no_failed_checks_note() {
    // Before issue #264 this hull got a 10.5 m tank that failed six checks; the search now returns
    // an arrangement that passes, so the strip has no failed-checks note.
    let r = gas_self_propelled();
    let failed = r["checks"]
        .as_array()
        .unwrap()
        .iter()
        // A failed warning is not a failure (issue #418).
        .filter(|c| c["ok"] == false && c["severity"] != "warning")
        .count();
    assert_eq!(failed, 0);
    for (view, doc) in views(&r).all() {
        check_strip(view, doc, failed);
    }
}

#[test]
fn twenty_long_names_make_one_numbered_note_that_fits() {
    let mut r = gas_self_propelled();
    let checks: Vec<Value> = (0..20)
        .map(|i| json!({"name": format!("A VERY LONG CHECK NAME FOR THE TEST NUMBER {i}"), "ok": false, "message": ""}))
        .collect();
    r["checks"] = Value::Array(checks);
    for (view, doc) in views(&r).all() {
        check_strip(view, doc, 20);
        let n = normalise(doc);
        let note = note_lines(&n).pop().unwrap().0;
        assert!(note.contains(" MORE"), "{view}: {note}");
    }
}
