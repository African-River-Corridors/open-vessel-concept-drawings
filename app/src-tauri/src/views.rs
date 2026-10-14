//! The hull views: the GA documents of an Option (spec 0003), drawn to the Drawing Format Standard by
//! `ovcd-draw`, as SVG for the main panel.
//!
//! This module has no Tauri types. The front end only places and zooms what it returns: it holds
//! no geometry, no scale and no dimension.

use ovcd_draw::render::svg::{SvgOptions, to_svg};
use ovcd_draw::{NoFiles, RunResult, run_value};
use ovcd_format::{RunOutcome, ga_views_of};
use serde_json::{Value, json};

/// The three views of a computed Option as SVG (`side`, `plan`, `midship`), or the reason there are
/// none (`message`). The font is not embedded: the front end loads it once.
pub fn hull_views(outcome: &RunOutcome) -> Value {
    let views = match ga_views_of(outcome) {
        Ok(v) => v,
        Err(message) => return json!({ "message": message }),
    };
    let mut out = serde_json::Map::new();
    for (key, document) in [
        ("side", &views.side_elevation),
        ("plan", &views.plan),
        ("midship", &views.midship_section),
    ] {
        match run_value(document, &NoFiles) {
            RunResult::Ok(norm) => {
                out.insert(
                    key.to_string(),
                    Value::String(to_svg(&norm, &SvgOptions { embed_font: false })),
                );
            }
            RunResult::Errors(clauses) => {
                let list: Vec<String> = clauses.into_iter().collect();
                return json!({
                    "message": format!(
                        "The hull views break the Drawing Format Standard (clause {}). This is a fault in OVCD, not in the Design.",
                        list.join(", ")
                    )
                });
            }
        }
    }
    Value::Object(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ovcd_format::{CONVOY_NOT_AVAILABLE, Profile, check, run_design};
    use std::path::Path;

    fn outcome(fixture: &str) -> RunOutcome {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/ovcd-format/tests/fixtures")
            .join(fixture);
        let checked = check(
            &std::fs::read_to_string(path).unwrap(),
            &Profile::built_in(),
        );
        run_design(&checked.design.unwrap()).remove(0).outcome
    }

    #[test]
    fn a_computed_option_gets_three_svg_views() {
        for fixture in [
            "dry_bulk_box_hold.vdesign",
            "gas_adn_g_self_propelled.vdesign",
        ] {
            let v = hull_views(&outcome(fixture));
            for key in ["side", "plan", "midship"] {
                let svg = v[key]
                    .as_str()
                    .unwrap_or_else(|| panic!("{fixture}: no {key}: {v}"));
                assert!(svg.starts_with("<?xml"), "{fixture} {key}");
                assert!(
                    svg.contains("viewBox=\"0 0 420 297\""),
                    "an A3 landscape sheet"
                );
                assert!(
                    !svg.contains("base64"),
                    "the font is loaded once by the front end, not in each SVG"
                );
            }
        }
    }

    #[test]
    fn the_dimensions_on_the_views_are_the_numbers_of_the_results() {
        let v = hull_views(&outcome("dry_bulk_box_hold.vdesign"));
        // LOA 76.5 m, beam 11.4 m, depth 3.5 m
        assert!(v["side"].as_str().unwrap().contains(">76.50<"));
        assert!(v["plan"].as_str().unwrap().contains(">76.50<"));
        assert!(v["plan"].as_str().unwrap().contains(">11.40<"));
        assert!(v["midship"].as_str().unwrap().contains(">11.40<"));
        assert!(v["midship"].as_str().unwrap().contains(">3.50<"));
    }

    #[test]
    fn a_convoy_has_no_views_and_the_message_of_the_outputs_panel() {
        let v = hull_views(&outcome("pushed_convoy.vdesign"));
        assert_eq!(v, json!({ "message": CONVOY_NOT_AVAILABLE }));
    }

    #[test]
    fn the_same_results_give_the_same_svg() {
        let o = outcome("gas_igc_2g_exact.vdesign");
        assert_eq!(hull_views(&o), hull_views(&o));
    }
}
