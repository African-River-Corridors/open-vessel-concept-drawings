//! Seam: the GA midship section draws the hull's own section (issue #450): the V-bottom, the chine
//! and a bilge radius from `drawing.midship_outline`, and the rectangle for a square box section.
//!
//! Label: property. Each test states a rule about the drawn outline; no number is a printed result.

use ovd_format::rule_set::Library;
use ovd_format::{Profile, RunOutcome, fill_as_new, ga_views_of, run_option_with};
use serde_json::{Value, json};

const B: f64 = 11.4;
const D: f64 = 5.0;

fn section(form: &str, extra: Value) -> Value {
    let mut inputs = json!({
        "cargo": { "value": "LNG", "source": "typed" },
        "hull_form": { "value": form, "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 90.0, "source": "typed" },
        "beam_m": { "value": B, "source": "typed" },
        "moulded_depth_m": { "value": D, "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    let design = fill_as_new(&file.to_string(), &Profile::built_in())
        .design
        .unwrap();
    let outcome = run_option_with(&design.options[0], &Library::built_in(), &[]);
    assert!(matches!(outcome, RunOutcome::Results { .. }), "{outcome:?}");
    ga_views_of(&outcome).expect("views").midship_section
}

/// The hull outline: the OUTLINE polyline that runs from the port deck edge to the starboard one.
fn outline(document: &Value) -> Vec<[f64; 2]> {
    document["viewports"][0]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["layer"] == "OUTLINE" && e["type"] == "polyline")
        .map(|e| {
            e["points_m"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| [p[0].as_f64().unwrap(), p[1].as_f64().unwrap()])
                .collect::<Vec<_>>()
        })
        .find(|p| p.first() == Some(&[-B / 2.0, D]) && p.last() == Some(&[B / 2.0, D]))
        .expect("the hull outline")
}

#[test]
fn a_square_section_is_the_rectangle() {
    assert_eq!(
        outline(&section("raked_box", json!({}))),
        [[-B / 2.0, D], [-B / 2.0, 0.0], [B / 2.0, 0.0], [B / 2.0, D]]
    );
}

#[test]
fn a_v_bottom_draws_the_flat_keel_the_deadrise_and_the_chine() {
    let o = outline(&section("v_bottom", json!({ "keel_half_width_m": 0.5 })));
    // Port deck edge, port chine, the flat keel (one line), starboard chine, starboard deck edge.
    assert_eq!(o.len(), 6, "{o:?}");
    assert_eq!(o[2], [-0.5, 0.0]);
    assert_eq!(o[3], [0.5, 0.0]);
    assert_eq!(o[1][0], -B / 2.0);
    assert!(o[1][1] > 0.0 && o[1][1] == o[4][1]);
}

#[test]
fn a_chine_and_a_bilge_radius_are_drawn_too() {
    let chine = outline(&section(
        "chine",
        json!({ "chine_half_breadth_m": 4.5, "chine_height_m": 0.6 }),
    ));
    assert!(
        chine.contains(&[4.5, 0.6]) && chine.contains(&[-4.5, 0.6]),
        "{chine:?}"
    );
    let round = outline(&section("raked_box", json!({ "bilge_radius_m": 1.0 })));
    // The arc's 16 chords on each side, the flat bottom as one line.
    assert_eq!(round.len(), 2 * 17 + 2, "{round:?}");
}
