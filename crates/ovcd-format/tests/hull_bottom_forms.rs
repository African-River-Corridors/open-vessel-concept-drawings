//! Seam: the `v_bottom` and `chine` Hull Forms run end to end (issue #450). A new Option of each form
//! takes the Defaults Profile values, and its hull is the form's sections.
//!
//! Label: property. Each test states a rule: the form runs, its midship section is fuller than a
//! V and less full than the box, and the inputs reach the hull.

use ovcd_format::rule_set::Library;
use ovcd_format::{Profile, RunOutcome, fill_as_new, run_option_with};
use serde_json::{Value, json};

fn results(cargo: &str, form: &str, extra: Value) -> Result<Value, String> {
    let mut inputs = json!({
        "cargo": { "value": cargo, "source": "typed" },
        "hull_form": { "value": form, "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 90.0, "source": "typed" },
        "beam_m": { "value": 11.4, "source": "typed" },
        "moulded_depth_m": { "value": 5.0, "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new(), "{form}");
    let design = filled.design.unwrap();
    match run_option_with(&design.options[0], &Library::built_in(), &[]) {
        RunOutcome::Results { results, .. } => Ok(results),
        RunOutcome::Failed(message) => Err(message),
        other => panic!("{other:?}"),
    }
}

fn cm(r: &Value) -> f64 {
    r["hydrostatics"]["cm"].as_f64().unwrap()
}

#[test]
fn a_new_option_of_each_form_runs_and_its_midship_section_is_the_forms() {
    let raked = results("LNG", "raked_box", json!({})).unwrap();
    let v = results("LNG", "v_bottom", json!({})).unwrap();
    let chine = results("LNG", "chine", json!({})).unwrap();
    assert!((cm(&raked) - 1.0).abs() < 1e-12, "{}", cm(&raked));
    assert!(cm(&v) < 1.0, "V-bottom Cm {}", cm(&v));
    // The default chine (no breadth, no height) is the V-bottom.
    assert!((cm(&chine) - cm(&v)).abs() < 1e-12);
    // A chine inboard of the side, with a flared side above it, is fuller than the V-bottom.
    let flared = results(
        "LNG",
        "chine",
        json!({ "chine_half_breadth_m": 4.5, "chine_height_m": 0.6 }),
    )
    .unwrap();
    assert!(cm(&flared) != cm(&v));
    // The deadrise reaches the hull.
    let steep = results("LNG", "v_bottom", json!({ "deadrise_deg": 12.0 })).unwrap();
    assert!(cm(&steep) < cm(&v));
}

#[test]
fn a_chine_outside_the_beam_is_refused_with_its_reason() {
    let r = results("LNG", "chine", json!({ "chine_half_breadth_m": 6.0 }));
    let message = r.expect_err("refused");
    assert!(message.contains("chine half-breadth"), "{message}");
}

#[test]
fn a_dry_bulk_v_bottom_takes_a_transom_and_a_tunnel_stern_says_why_not() {
    // The dry bulk default stern is a tunnel stern, which needs a flat bottom wider than the
    // tunnels; the default keel flat is narrower.
    let message = results("bauxite", "v_bottom", json!({})).expect_err("refused");
    assert!(
        message.contains("takes a transom or a pram stern"),
        "{message}"
    );
    for form in ["v_bottom", "chine"] {
        let r = results("bauxite", form, json!({ "stern_form": "transom" })).unwrap();
        assert!(cm(&r) < 1.0, "{form}");
    }
}

#[test]
fn the_drawing_gets_the_midship_outline_of_a_shaped_section_only() {
    let raked = results("LNG", "raked_box", json!({})).unwrap();
    assert!(raked["drawing"].get("midship_outline").is_none());
    let v = results("LNG", "v_bottom", json!({})).unwrap();
    let pts: Vec<(f64, f64)> = v["drawing"]["midship_outline"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (p[0].as_f64().unwrap(), p[1].as_f64().unwrap()))
        .collect();
    // The keel on the centre line, the flat keel, the chine at the side, the deck edge.
    assert_eq!(pts.len(), 4, "{pts:?}");
    assert_eq!(pts[0], (0.0, 0.0));
    assert_eq!(pts[1].1, 0.0);
    assert_eq!(pts[2].0, 11.4 / 2.0);
    assert!(pts[2].1 > 0.0);
    assert_eq!(pts[3], (11.4 / 2.0, 5.0));
}
