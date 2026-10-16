//! Seam 5: the GA view builder (spec 0003, issue #100). Every view of every fixture Design is checked
//! against the Drawing Format Standard itself, through `ovcd-draw`.

#[path = "../../ovcd-draw/tests/support/dxf_reader.rs"]
mod dxf_reader;

use ovcd_core::pyfmt::fixed;
use ovcd_draw::entities::Shape;
use ovcd_draw::{NoFiles, Normalised, RunResult, run_value};
use ovcd_format::{
    CONVOY_NOT_AVAILABLE, GaViews, Profile, RunOutcome, check, ga_views_of, run_design,
};
use serde_json::Value;
use std::path::Path;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name),
    )
    .expect("fixture")
}

/// The results of every Option of a fixture that can be computed, with the Option's name.
fn computed(name: &str) -> Vec<(String, Value)> {
    let checked = check(&fixture(name), &Profile::built_in());
    let design = checked.design.expect("a fixture is a valid Design");
    run_design(&design)
        .into_iter()
        .filter_map(|r| match r.outcome {
            RunOutcome::Results { results, .. } => Some((r.name, results)),
            _ => None,
        })
        .collect()
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
        RunResult::Errors(e) => {
            panic!("the standard refuses the document (clause {e:?}): {document}")
        }
    }
}

const DRY: &[&str] = &[
    "dry_bulk_box_hold.vdesign",
    "dry_bulk_sloped_hopper.vdesign",
];
const GAS: &[&str] = &[
    "gas_igc_2g_self_propelled.vdesign",
    "gas_igc_2g_exact.vdesign",
];

fn all_fixtures() -> Vec<(String, String, Value)> {
    DRY.iter()
        .chain(GAS)
        .chain(&["two_options.vdesign"])
        .flat_map(|f| {
            computed(f)
                .into_iter()
                .map(move |(name, r)| (f.to_string(), name, r))
        })
        .collect()
}

#[test]
fn every_document_of_every_fixture_validates() {
    let all = all_fixtures();
    assert!(all.len() >= 6, "{} computed Options", all.len());
    for (file, option, results) in &all {
        for (view, document) in views(results).all() {
            let norm = normalise(document);
            assert_eq!(norm.sheet.size, "A3", "{file} {option} {view}");
            assert_eq!(norm.sheet.orientation, "landscape");
            assert_eq!(norm.viewports.len(), 1, "one viewport");
            assert_eq!(document["status"], "concept");
            assert!(document.get("title_block").is_none(), "no title block");
        }
    }
}

/// The texts of the dimensions and leaders of a view: what a reader of the drawing reads.
fn read_texts(norm: &Normalised) -> Vec<String> {
    norm.entities
        .iter()
        .filter(|e| e.origin.ends_with(".dimension") || e.origin.ends_with(".leader"))
        .filter_map(|e| {
            if let Shape::Text { text, .. } = &e.shape {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect()
}

/// Is `value` (two decimals) written on the drawing, as a dimension or in a label?
fn written(texts: &[String], value: f64) -> bool {
    let v = fixed(value, 2);
    texts
        .iter()
        .any(|t| *t == v || t.ends_with(&format!(" {v} m")))
}

#[test]
fn the_numbers_on_the_drawings_are_the_numbers_in_the_results() {
    for (file, option, r) in all_fixtures() {
        let get = |a: &str, b: &str| r[a][b].as_f64().unwrap();
        let (loa, beam, depth) = (
            get("dims", "loa_m"),
            get("dims", "beam_m"),
            get("dims", "depth_m"),
        );
        let (draught, freeboard) = (get("results", "draught_m"), get("results", "freeboard_m"));
        let v = views(&r);
        let side = read_texts(&normalise(&v.side_elevation));
        let plan = read_texts(&normalise(&v.plan));
        let section = read_texts(&normalise(&v.midship_section));
        let who = format!("{file} {option}");
        for (name, value) in [
            ("loa", loa),
            ("depth", depth),
            ("draught", draught),
            ("freeboard", freeboard),
        ] {
            assert!(
                written(&side, value),
                "{who}: side elevation does not show {name} {value}: {side:?}"
            );
        }
        for (name, value) in [("loa", loa), ("beam", beam)] {
            assert!(
                written(&plan, value),
                "{who}: plan does not show {name} {value}: {plan:?}"
            );
        }
        for (name, value) in [
            ("beam", beam),
            ("depth", depth),
            ("draught", draught),
            ("freeboard", freeboard),
        ] {
            assert!(
                written(&section, value),
                "{who}: midship section does not show {name} {value}: {section:?}"
            );
        }
    }
}

/// The entities of a document's viewport, as JSON.
fn entities(document: &Value) -> &Vec<Value> {
    document["viewports"][0]["entities"].as_array().unwrap()
}

fn of_layer<'a>(document: &'a Value, layer: &str, kind: &str) -> Vec<&'a Value> {
    entities(document)
        .iter()
        .filter(|e| e["layer"] == layer && e["type"] == kind)
        .collect()
}

fn xs(points: &Value) -> Vec<f64> {
    points
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p[0].as_f64().unwrap())
        .collect()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn dry_bulk_shows_its_own_containment() {
    for file in DRY {
        for (_, r) in computed(file) {
            let g = &r["geometry"];
            // The hold floor is the inner bottom (#193), not `depth - hold_height_m`: the core's hold
            // height runs on up to the top of the coaming.
            let (inner_bottom, top_w, bottom_w) = (
                r["drawing"]["double_bottom_m"].as_f64().unwrap(),
                g["hold_top_width_m"].as_f64().unwrap(),
                g["hold_bottom_width_m"].as_f64().unwrap(),
            );
            let depth = r["dims"]["depth_m"].as_f64().unwrap();
            let v = views(&r);
            // side elevation: the hold in dashed hidden lines, from the stern void to the bow void
            let hidden = of_layer(&v.side_elevation, "HIDDEN", "polyline");
            assert_eq!(hidden.len(), 1, "{file}");
            let ys: Vec<f64> = hidden[0]["points_m"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| p[1].as_f64().unwrap())
                .collect();
            assert!(
                ys.iter().any(|y| close(*y, inner_bottom)) && ys.iter().any(|y| close(*y, depth)),
                "{file}: the hold spans the inner bottom to the deck"
            );
            let hx = xs(&hidden[0]["points_m"]);
            assert!(close(
                hx.iter().cloned().fold(f64::INFINITY, f64::min),
                g["stern_void_length_m"].as_f64().unwrap()
            ));
            // midship section: the hold as a box or as a sloped hopper
            let cut = of_layer(&v.midship_section, "OUTLINE", "polyline");
            let hold = cut
                .iter()
                .find(|p| {
                    p["points_m"].as_array().unwrap().len() == 4
                        && p["closed"] == false
                        && xs(&p["points_m"])
                            .iter()
                            .any(|x| close(x.abs(), top_w / 2.0))
                })
                .expect("the hold section");
            let widths: Vec<f64> = xs(&hold["points_m"]).iter().map(|x| x.abs()).collect();
            assert!(close(widths[0], top_w / 2.0) && close(widths[3], top_w / 2.0));
            assert!(
                close(widths[1], bottom_w / 2.0) && close(widths[2], bottom_w / 2.0),
                "{file}: the hold bottom is {bottom_w} wide"
            );
            // plan: the hold opening
            assert!(
                !of_layer(&v.plan, "OBJECT", "polyline").is_empty(),
                "{file}: plan shows the hold opening"
            );
            assert!(
                of_layer(&v.midship_section, "OUTLINE", "circle").is_empty(),
                "no tanks in a hold"
            );
        }
    }
}

#[test]
fn gas_shows_its_tank_arrangement() {
    for file in GAS {
        for (_, r) in computed(file) {
            let a = &r["arrangement"];
            let (n_wide, n_long) = (
                a["n_wide"].as_u64().unwrap() as usize,
                a["n_long"].as_u64().unwrap() as usize,
            );
            let od = r["geometry"]["tank_outer_diameter_m"].as_f64().unwrap();
            let v = views(&r);
            let circles = of_layer(&v.midship_section, "OUTLINE", "circle");
            assert_eq!(
                circles.len(),
                n_wide,
                "{file}: one circle for each tank across"
            );
            assert!(
                circles
                    .iter()
                    .all(|c| close(c["radius_m"].as_f64().unwrap(), od / 2.0))
            );
            // A head is one arc centre. Its arc may be split between HIDDEN and OBJECT where the
            // tank stands through the deck (#195), and the plan may draw a head twice (its widest
            // outline and the part above the deck), so heads are counted by distinct centre and
            // radius.
            let hidden_arcs = |d: &Value| {
                let mut heads: Vec<String> = ["HIDDEN", "OBJECT"]
                    .iter()
                    .flat_map(|layer| of_layer(d, layer, "arc"))
                    .map(|e| e["centre_m"].to_string())
                    .collect();
                heads.sort();
                heads.dedup();
                heads.len()
            };
            assert_eq!(
                hidden_arcs(&v.side_elevation),
                2 * n_long,
                "{file}: two heads for each tank along"
            );
            assert_eq!(
                hidden_arcs(&v.plan),
                2 * n_long * n_wide,
                "{file}: two heads for each tank of the plan"
            );
            // No hold in a gas barge. The one hidden polyline there may be is the wing ballast
            // tank band (#196), which is there only when the results give a wing tank width.
            let wing = r["drawing"]["wing_tank_width_m"].as_f64().unwrap_or(0.0) > 0.0;
            assert_eq!(
                of_layer(&v.side_elevation, "HIDDEN", "polyline").len(),
                usize::from(wing),
                "no hold in a gas barge"
            );
        }
    }
}

#[test]
fn the_hull_is_drawn_to_the_results() {
    for (file, option, r) in all_fixtures() {
        let loa = r["dims"]["loa_m"].as_f64().unwrap();
        let depth = r["dims"]["depth_m"].as_f64().unwrap();
        let v = views(&r);
        let outline = of_layer(&v.side_elevation, "OUTLINE", "polyline");
        let pts = outline[0]["points_m"].as_array().unwrap();
        let top = pts
            .iter()
            .map(|p| p[1].as_f64().unwrap())
            .fold(f64::NEG_INFINITY, f64::max);
        let right = pts
            .iter()
            .map(|p| p[0].as_f64().unwrap())
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(close(top, depth) && close(right, loa), "{file} {option}");
        // the waterline is at the draught
        let wl = of_layer(&v.side_elevation, "WATER", "line");
        assert!(wl.iter().any(|l| close(
            l["from_m"][1].as_f64().unwrap(),
            r["results"]["draught_m"].as_f64().unwrap()
        )));
    }
}

#[test]
fn a_convoy_gets_no_views_and_the_message_of_the_outputs_panel() {
    let checked = check(&fixture("pushed_convoy.vdesign"), &Profile::built_in());
    let runs = run_design(&checked.design.unwrap());
    let outcome = &runs[0].outcome;
    assert_eq!(ga_views_of(outcome), Err(CONVOY_NOT_AVAILABLE.to_string()));
    assert_eq!(
        ga_views_of(&RunOutcome::Failed("Cargo does not fit".into())),
        Err("Cargo does not fit".to_string())
    );
}

#[test]
fn the_same_results_give_the_same_documents() {
    for (_, _, r) in all_fixtures() {
        assert_eq!(views(&r), views(&r));
    }
}

#[test]
fn results_that_lack_a_value_are_refused_with_a_message() {
    let (_, r) = computed("dry_bulk_box_hold.vdesign").remove(0);
    let mut broken = r.clone();
    broken["dims"].as_object_mut().unwrap().remove("beam_m");
    let err = ovcd_format::ga_views(&broken).unwrap_err();
    assert!(err.contains("dims.beam_m"), "{err}");
    let mut zero = r;
    zero["dims"]["loa_m"] = serde_json::json!(0.0);
    assert!(ovcd_format::ga_views(&zero).unwrap_err().contains("loa"));
}

/// The `simulate` params of a fixture's first Option.
fn base_params(file: &str) -> serde_json::Map<String, Value> {
    let checked = check(&fixture(file), &Profile::built_in());
    ovcd_format::simulate_params_of(&checked.design.unwrap().options[0]).expect("params")
}

/// The results of `simulate` for params, if the Option can be computed.
fn simulate(params: &serde_json::Map<String, Value>) -> Option<Value> {
    let input = serde_json::json!({"fn": "simulate", "args": {"params": params}});
    let out = ovcd_format::eval("simulate", &input).ok()?;
    let result = out.get("ok").cloned().unwrap_or(out);
    result.get("dims").is_some().then_some(result)
}

#[test]
fn a_sweep_of_hull_sizes_always_draws_a_valid_sheet() {
    let mut drawn = 0;
    for file in [
        "dry_bulk_box_hold.vdesign",
        "dry_bulk_sloped_hopper.vdesign",
        "gas_igc_2g_self_propelled.vdesign",
        "gas_igc_2g_exact.vdesign",
    ] {
        for loa in [
            12.0, 20.0, 30.0, 55.0, 76.5, 95.0, 135.0, 170.0, 250.0, 400.0,
        ] {
            for (beam, depth) in [
                (3.0, 1.5),
                (6.0, 2.5),
                (9.5, 3.0),
                (11.4, 4.5),
                (14.0, 5.5),
                (16.0, 7.0),
                (30.0, 9.0),
            ] {
                let mut p = base_params(file);
                p.insert("loa_m".into(), serde_json::json!(loa));
                p.insert("beam_m".into(), serde_json::json!(beam));
                p.insert("depth_m".into(), serde_json::json!(depth));
                let Some(results) = simulate(&p) else {
                    continue;
                };
                let Ok(v) = ovcd_format::ga_views(&results) else {
                    continue;
                };
                for (view, document) in v.all() {
                    match run_value(document, &NoFiles) {
                        RunResult::Ok(_) => {}
                        RunResult::Errors(e) => panic!(
                            "{file} loa {loa} beam {beam} depth {depth} {view}: the standard refuses the document (clause {e:?})"
                        ),
                    }
                }
                drawn += 1;
            }
        }
    }
    assert!(drawn >= 40, "only {drawn} combinations could be drawn");
}

#[test]
fn the_dxf_of_every_view_reads_back_to_the_same_entities_at_the_stated_scale() {
    for (file, option, r) in all_fixtures() {
        for (view, document) in views(&r).all() {
            let norm = normalise(document);
            let dxf = ovcd_draw::render::dxf::to_dxf(document, &norm, &NoFiles);
            assert!(
                dxf.warnings.is_empty(),
                "{file} {option} {view}: {:?}",
                dxf.warnings
            );
            let back = dxf_reader::read_normalised(&dxf.text);
            let mut want = norm.entities.clone();
            let mut got = back.entities;
            want.sort_by(ovcd_draw::entities::compare);
            got.sort_by(ovcd_draw::entities::compare);
            assert_eq!(got, want, "{file} {option} {view}");
            assert_eq!(
                back.viewports["main"].n, norm.viewports[0].scale,
                "{file} {option} {view}: the viewport holds the stated scale"
            );
        }
    }
}
