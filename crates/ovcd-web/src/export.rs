//! Drawing export of one Option: the hull view documents of its results, as SVG (font embedded),
//! PDF and DXF. Plain Rust, no wasm types, so it is tested natively. It holds no geometry: the
//! documents come from `ga_views_named`, as in `ovcd_shell::views`.

use ovcd_draw::render::{dxf::to_dxf, pdf::to_pdf, svg::SvgOptions, svg::to_svg};
use ovcd_draw::{NoFiles, Normalised, RunResult, run_value};
use ovcd_format::ga_views_named;
use ovcd_shell::AppState;
use serde_json::Value;

/// The view document of an Option (`side`, `plan` or `midship`), or the reason there is none.
fn document(state: &AppState, option: &str, view: &str) -> Result<Value, String> {
    let evaluation = state.session().evaluate();
    let found = evaluation
        .options
        .iter()
        .find(|o| o.name == option)
        .ok_or_else(|| format!("no Option named '{option}'"))?;
    let outcome = found
        .outcome
        .as_ref()
        .ok_or_else(|| "The Option has an error, so there are no views.".to_string())?;
    let views = ga_views_named(outcome, Some(option))?;
    match view {
        "side" => Ok(views.side_elevation),
        "plan" => Ok(views.plan),
        "midship" => Ok(views.midship_section),
        other => Err(format!(
            "unknown view '{other}' (use side, plan or midship)"
        )),
    }
}

fn normalised(document: &Value) -> Result<(Box<Normalised>, &Value), String> {
    match run_value(document, &NoFiles) {
        RunResult::Ok(norm) => Ok((norm, document)),
        RunResult::Errors(clauses) => {
            let list: Vec<String> = clauses.into_iter().collect();
            Err(format!(
                "The hull views break the Drawing Format Standard (clause {}). This is a fault in OVCD, not in the Design.",
                list.join(", ")
            ))
        }
    }
}

/// The view as a standalone SVG, with the font embedded.
pub fn svg(state: &AppState, option: &str, view: &str) -> Result<String, String> {
    let doc = document(state, option, view)?;
    let (norm, _) = normalised(&doc)?;
    Ok(to_svg(&norm, &SvgOptions { embed_font: true }))
}

/// The view as PDF bytes.
pub fn pdf(state: &AppState, option: &str, view: &str) -> Result<Vec<u8>, String> {
    let doc = document(state, option, view)?;
    let (norm, _) = normalised(&doc)?;
    Ok(to_pdf(&norm))
}

/// The view as DXF bytes (text).
pub fn dxf(state: &AppState, option: &str, view: &str) -> Result<Vec<u8>, String> {
    let doc = document(state, option, view)?;
    let (norm, doc) = normalised(&doc)?;
    Ok(to_dxf(doc, &norm, &NoFiles).text.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn open(fixture: &str) -> AppState {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../ovcd-format/tests/fixtures")
            .join(fixture);
        AppState::open(Path::new(fixture), &std::fs::read_to_string(path).unwrap()).unwrap()
    }

    fn first_option(state: &AppState) -> String {
        state.session().design().options[0].name.clone()
    }

    #[test]
    fn svg_embeds_the_font_in_each_view() {
        let state = open("gas_igc_2g_self_propelled.vdesign");
        let name = first_option(&state);
        for view in ["side", "plan", "midship"] {
            let svg = svg(&state, &name, view).unwrap();
            assert!(svg.contains("<svg"), "{view}");
            assert!(svg.contains("base64"), "{view}: the font is embedded");
        }
    }

    #[test]
    fn pdf_starts_with_the_pdf_header() {
        let state = open("dry_bulk_box_hold.vdesign");
        let bytes = pdf(&state, &first_option(&state), "side").unwrap();
        assert!(bytes.starts_with(b"%PDF-"));
    }

    #[test]
    fn dxf_is_text_with_an_eof_marker() {
        let state = open("dry_bulk_box_hold.vdesign");
        let bytes = dxf(&state, &first_option(&state), "plan").unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("EOF"));
    }

    #[test]
    fn an_unknown_view_or_option_is_an_error_string() {
        let state = open("dry_bulk_box_hold.vdesign");
        let name = first_option(&state);
        assert!(
            svg(&state, &name, "back")
                .unwrap_err()
                .contains("unknown view")
        );
        assert!(
            svg(&state, "nope", "side")
                .unwrap_err()
                .contains("no Option named")
        );
    }

    #[test]
    fn a_wide_option_name_exports_every_view() {
        // Issue #660: the export of "140 x 29 x 9m LNG" broke clause 14.6.
        let mut state = open("gas_igc_2g_self_propelled.vdesign");
        let name = "140 x 29 x 9m LNG";
        state.rename_option(&first_option(&state), name).unwrap();
        for view in ["side", "plan", "midship"] {
            assert!(svg(&state, name, view).unwrap().contains("<svg"), "{view}");
            assert!(
                pdf(&state, name, view).unwrap().starts_with(b"%PDF-"),
                "{view}"
            );
            assert!(!dxf(&state, name, view).unwrap().is_empty(), "{view}");
        }
    }

    #[test]
    fn a_convoy_gives_the_no_drawings_message() {
        let state = open("pushed_convoy.vdesign");
        let err = svg(&state, &first_option(&state), "side").unwrap_err();
        assert_eq!(err, ovcd_format::CONVOY_NO_VIEWS);
    }
}
