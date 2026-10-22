//! Seam 5: the GA view builder (spec 0003, issue #100). Every view of every fixture Design is checked
//! against the Drawing Format Standard itself, through `ovcd-draw`.

#[path = "support/dxf_reader.rs"]
mod dxf_reader;

use ovcd_core::pyfmt::fixed;
use ovcd_draw::entities::Shape;
use ovcd_draw::{NoFiles, Normalised, RunResult, run_value};
use ovcd_format::{CONVOY_NO_VIEWS, GaViews, Profile, RunOutcome, check, ga_views_of, run_design};
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
            // Issue #409: the ISO 7200 title block, with the view, the Option and the sheet.
            assert_eq!(document["title_block"]["template"], "iso7200");
            let fields = &document["title_block"]["fields"];
            assert_eq!(fields["sheet_count"], 3);
            assert!(fields["title"].as_str().is_some_and(|t| !t.is_empty()));
            assert!(
                document["notes"].to_string().contains("RULE SETS"),
                "{file}: the Rule Sets are named"
            );
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
fn the_drawn_hold_section_has_the_area_the_volume_model_uses() {
    // Issue #408: the volume model takes the top width at the coaming top and the slope over the
    // whole hold height. The drawn side therefore runs from the inner bottom to the coaming top.
    for file in DRY {
        for (_, r) in computed(file) {
            let g = &r["geometry"];
            let (top_w, bottom_w, hold_h) = (
                g["hold_top_width_m"].as_f64().unwrap(),
                g["hold_bottom_width_m"].as_f64().unwrap(),
                g["hold_height_m"].as_f64().unwrap(),
            );
            let model_area = 0.5 * (top_w + bottom_w) * hold_h;
            let v = views(&r);
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
            let pts: Vec<[f64; 2]> = hold["points_m"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| [p[0].as_f64().unwrap(), p[1].as_f64().unwrap()])
                .collect();
            let mut twice = 0.0;
            for i in 0..4 {
                let (a, b) = (pts[i], pts[(i + 1) % 4]);
                twice += a[0] * b[1] - b[0] * a[1];
            }
            let drawn_area = twice.abs() / 2.0;
            assert!(
                (drawn_area - model_area).abs() < 1e-9,
                "{file}: drawn {drawn_area} m2, model {model_area} m2"
            );
            let depth = r["dims"]["depth_m"].as_f64().unwrap();
            let coaming = r["drawing"]["hatch_coaming_height_m"].as_f64().unwrap();
            assert!(
                close(pts[0][1], depth + coaming),
                "{file}: top at the coaming top"
            );
        }
    }
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
            // tank band (#196), which is there only when the results give a wing tank width, and
            // is hidden only when the wing plate is below the deck (#407: above it, it is in view).
            let wing = r["drawing"]["wing_tank_width_m"].as_f64().unwrap_or(0.0) > 0.0;
            let below_deck = wing_plate_level(&r) <= r["dims"]["depth_m"].as_f64().unwrap();
            assert_eq!(
                of_layer(&v.side_elevation, "HIDDEN", "polyline").len(),
                usize::from(wing && below_deck),
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
    assert_eq!(ga_views_of(outcome), Err(CONVOY_NO_VIEWS.to_string()));
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

/// The wing plate level of a gas result, as the weights work it out: the tank bottom (double
/// bottom and support height) plus the wing tank height ratio times the tank outer diameter.
fn wing_plate_level(r: &Value) -> f64 {
    let d = &r["drawing"];
    d["double_bottom_m"].as_f64().unwrap()
        + d["tank_support_height_m"].as_f64().unwrap()
        + d["wing_tank_height_ratio"].as_f64().unwrap_or(0.0)
            * r["geometry"]["tank_outer_diameter_m"].as_f64().unwrap()
}

#[test]
fn the_drawn_wing_tank_top_is_the_wing_plate_level_even_above_the_deck() {
    // Issue #407: the wing tank top was clamped at the deck; the weights carry the wing plate at
    // its own level, and the side shell runs up to it. Both gas fixtures have it above the deck.
    for file in GAS {
        for (_, r) in computed(file) {
            let level = wing_plate_level(&r);
            let depth = r["dims"]["depth_m"].as_f64().unwrap();
            assert!(
                level > depth,
                "{file}: the fixture tests the case above the deck"
            );
            let v = views(&r);
            let ys: Vec<f64> = of_layer(&v.midship_section, "OBJECT", "polyline")
                .iter()
                .flat_map(|e| e["points_m"].as_array().unwrap().iter())
                .map(|p| p[1].as_f64().unwrap())
                .collect();
            assert!(
                ys.iter().any(|y| close(*y, level)),
                "{file}: no wing tank top at {level} m in the section"
            );
            let side = of_layer(&v.midship_section, "OBJECT", "line")
                .into_iter()
                .filter(|e| {
                    close(e["from_m"][1].as_f64().unwrap(), depth)
                        && close(e["to_m"][1].as_f64().unwrap(), level)
                })
                .count();
            assert_eq!(
                side, 2,
                "{file}: the side shell runs up to the wing plate on both sides"
            );
        }
    }
}

#[test]
fn the_weather_shield_is_drawn_dashed_from_the_wing_plate_to_the_tank_top_plus_the_clearance() {
    // Issue #407: the weights carry a shield over the tanks (sides and ends on the wing plate, or on
    // the deck where the Moulded Depth is higher; its top the tank top plus the clearance). The
    // drawing shows it dashed, in the section and the elevation, from the results' own values.
    for file in GAS {
        for (_, r) in computed(file) {
            let d = &r["drawing"];
            let (t, clearance) = (
                d["weather_shield_thickness_mm"].as_f64().unwrap(),
                d["weather_shield_clearance_m"].as_f64().unwrap(),
            );
            assert!(t > 0.0, "{file}: the fixture has a shield");
            let depth = r["dims"]["depth_m"].as_f64().unwrap();
            let od = r["geometry"]["tank_outer_diameter_m"].as_f64().unwrap();
            let bottom = d["double_bottom_m"].as_f64().unwrap()
                + d["tank_support_height_m"].as_f64().unwrap();
            let top = bottom + od + clearance;
            let foot = wing_plate_level(&r).max(depth);
            assert!(top > foot, "{file}: the shield stands above its foot");
            let v = views(&r);
            for (name, view) in [
                ("section", &v.midship_section),
                ("elevation", &v.side_elevation),
            ] {
                let shield: Vec<&Value> = of_layer(view, "ZONE", "polyline")
                    .into_iter()
                    .filter(|e| {
                        let ys: Vec<f64> = e["points_m"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|p| p[1].as_f64().unwrap())
                            .collect();
                        ys.len() == 4
                            && close(ys[0], foot)
                            && close(ys[1], top)
                            && close(ys[2], top)
                            && close(ys[3], foot)
                    })
                    .collect();
                assert_eq!(shield.len(), 1, "{file}: the shield in the {name}");
            }
        }
    }
}

fn x_lines(document: &Value, layer: &str) -> Vec<f64> {
    of_layer(document, layer, "line")
        .iter()
        .filter(|e| {
            close(
                e["from_m"][0].as_f64().unwrap(),
                e["to_m"][0].as_f64().unwrap(),
            )
        })
        .map(|e| e["from_m"][0].as_f64().unwrap())
        .collect()
}

#[test]
fn the_elevation_and_the_plan_are_drawn_at_one_scale() {
    // Issue #409: a length scaled off one view is the same off the other.
    for (file, r) in all_fixtures().into_iter().map(|(f, _, r)| (f, r)) {
        let v = views(&r);
        assert_eq!(
            v.side_elevation["viewports"][0]["scale"], v.plan["viewports"][0]["scale"],
            "{file}"
        );
        assert!(v.plan["viewports"][0]["scale"].is_number(), "{file}");
    }
}

#[test]
fn the_section_is_labelled_a_typical_section() {
    let (_, r) = computed("gas_igc_2g_exact.vdesign").remove(0);
    let notes = views(&r).midship_section["notes"].to_string();
    assert!(notes.contains("TYPICAL SECTION"), "{notes}");
}

#[test]
fn the_top_of_the_structure_is_drawn_at_the_draught_plus_the_air_draft() {
    // Issue #409: a dashed line at the top of the structure, and the air draft named.
    for file in GAS.iter().chain(DRY) {
        let (_, r) = computed(file).remove(0);
        let top = r["results"]["draught_m"].as_f64().unwrap()
            + r["results"]["air_draft_loaded_m"].as_f64().unwrap();
        let v = views(&r);
        for (name, view) in [
            ("elevation", &v.side_elevation),
            ("section", &v.midship_section),
        ] {
            let at = of_layer(view, "ZONE", "line")
                .iter()
                .filter(|e| close(e["from_m"][1].as_f64().unwrap(), top))
                .count();
            assert_eq!(
                at, 1,
                "{file}: the line at the top of the structure in the {name}"
            );
            // The air draft is a dimension from the waterline to the line, or, with no room for
            // one, a label that names it.
            let dim = entities(view).iter().any(|e| {
                e["type"] == "dimension"
                    && close(e["to"]["point_m"][1].as_f64().unwrap_or(f64::NAN), top)
            });
            assert!(
                dim || view.to_string().contains("AIR DRAFT"),
                "{file}: {name} gives the air draft"
            );
        }
    }
}

#[test]
fn a_gas_plan_shows_the_forward_separation_and_the_service_space() {
    for file in GAS {
        let (_, r) = computed(file).remove(0);
        let sep = &r["drawing"]["separation"];
        let (fwd, service) = (
            sep["fwd_separation_m"].as_f64().unwrap(),
            sep["service_space_m"].as_f64().unwrap(),
        );
        let x1 = r["dims"]["loa_m"].as_f64().unwrap()
            - r["geometry"]["bow_void_length_m"].as_f64().unwrap();
        let v = views(&r);
        for (name, view) in [("elevation", &v.side_elevation), ("plan", &v.plan)] {
            assert!(
                x_lines(view, "HIDDEN").iter().any(|x| close(*x, x1 + fwd)),
                "{file}: the forward end bulkhead in the {name}"
            );
            let text = view.to_string();
            assert!(text.contains("FWD SEPARATION"), "{file}: {name}");
            assert_eq!(
                text.contains("SERVICE SPACE"),
                service > 0.0,
                "{file}: {name}"
            );
        }
    }
}

#[test]
fn a_dry_bulk_hull_shows_a_bulkhead_between_each_two_holds() {
    for file in DRY {
        let (_, r) = computed(file).remove(0);
        let holds = r["drawing"]["hold_count"].as_u64().unwrap() as usize;
        assert!(holds >= 2, "{file}");
        let (x0, x1) = (
            r["geometry"]["stern_void_length_m"].as_f64().unwrap(),
            r["dims"]["loa_m"].as_f64().unwrap()
                - r["geometry"]["bow_void_length_m"].as_f64().unwrap(),
        );
        let v = views(&r);
        for (name, view) in [("elevation", &v.side_elevation), ("plan", &v.plan)] {
            let xs = x_lines(view, "HIDDEN");
            for i in 1..holds {
                let x = x0 + (x1 - x0) * i as f64 / holds as f64;
                assert!(
                    xs.iter().any(|a| close(*a, x)),
                    "{file}: bulkhead {i} in the {name}"
                );
            }
            assert!(
                view.to_string().contains(&format!("HOLDS ({holds})")),
                "{file}: {name}"
            );
        }
        // A gas hull has none.
    }
    let (_, gas) = computed("gas_igc_2g_exact.vdesign").remove(0);
    assert!(gas["drawing"]["hold_count"].is_null());
    assert!(!views(&gas).plan.to_string().contains("HOLDS"));
}

#[test]
fn a_ballast_waterline_is_drawn_when_the_results_have_a_ballast_condition_apart_from_the_others() {
    // Issue #409. The IS Code loading conditions give a "Ballast, departure" draught. It is drawn
    // when it stands at least 1 mm from the light and the loaded waterlines at the scale of the view.
    let (_, mut r) = computed("gas_igc_2g_exact.vdesign").remove(0);
    r["loading_conditions"] = serde_json::json!({"conditions": [
        {"role": "ballast_departure", "draught_m": 1.655}
    ]});
    let v = views(&r);
    for (name, view) in [
        ("elevation", &v.side_elevation),
        ("section", &v.midship_section),
    ] {
        let at = of_layer(view, "WATER", "line")
            .iter()
            .filter(|e| close(e["from_m"][1].as_f64().unwrap(), 1.655))
            .count();
        assert_eq!(at, 1, "{name}");
        assert!(view.to_string().contains("BALLAST DRAUGHT"), "{name}");
    }
    // Next to the light waterline it is not drawn twice.
    let light = r["results"]["light_draught_m"].as_f64().unwrap();
    r["loading_conditions"]["conditions"][0]["draught_m"] = serde_json::json!(light + 0.001);
    assert!(
        !views(&r)
            .side_elevation
            .to_string()
            .contains("BALLAST DRAUGHT")
    );
}

/// The results of a self-propelled Option as a new Design gets it: the new stern, machinery and a
/// lifting wheelhouse, every input from the Defaults Profile.
fn new_self_propelled(cargo: &str) -> Value {
    let raw = serde_json::json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": cargo, "source": "typed" },
            "hull_form": { "value": "raked_box", "source": "typed" },
            "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
            "length_overall_m": { "value": 76.5, "source": "typed" },
            "beam_m": { "value": 11.4, "source": "typed" },
            "moulded_depth_m": { "value": 3.5, "source": "typed" }
        }}]
    });
    let filled = ovcd_format::fill_as_new(&raw.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    match run_design(&filled.design.unwrap()).remove(0).outcome {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_tunnel_stern_the_raised_wheelhouse_and_the_blind_distance_are_drawn_from_the_results() {
    // Issue #356. A tunnel stern: the crown along the run in the elevation, each tunnel in the plan,
    // both hidden. The lifting wheelhouse raised, as a thin chain outline. The Blind Distances in
    // the notes.
    let r = new_self_propelled("bauxite");
    assert_eq!(r["geometry"]["stern_form"], "tunnel");
    let v = views(&r);
    let crown: Vec<f64> = r["drawing"]["stern"]["profile"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p[1].as_f64().unwrap())
        .collect();
    let drawn: Vec<Vec<f64>> = of_layer(&v.side_elevation, "HIDDEN", "polyline")
        .iter()
        .map(|e| {
            e["points_m"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| p[1].as_f64().unwrap())
                .collect()
        })
        .collect();
    assert!(
        drawn
            .iter()
            .any(|ys| ys.len() == crown.len() && ys.iter().zip(&crown).all(|(a, b)| close(*a, *b))),
        "the tunnel crown"
    );
    let tunnels = r["drawing"]["stern"]["tunnel_sections"]
        .as_array()
        .unwrap()
        .len();
    let closed = of_layer(&v.plan, "HIDDEN", "polyline")
        .iter()
        .filter(|e| e["closed"] == true && e["points_m"].as_array().unwrap().len() == 4)
        .filter(|e| {
            let x1 = e["points_m"][1][0].as_f64().unwrap();
            close(
                x1,
                r["drawing"]["stern"]["profile"]
                    .as_array()
                    .unwrap()
                    .last()
                    .unwrap()[0]
                    .as_f64()
                    .unwrap(),
            )
        })
        .count();
    assert_eq!(closed, tunnels, "one outline for each tunnel");
    let a = &r["drawing"]["aft_arrangement"];
    let (floor, top) = (
        a["wheelhouse_floor_raised_m"].as_f64().unwrap(),
        a["wheelhouse_top_raised_m"].as_f64().unwrap(),
    );
    assert!(a["wheelhouse_lift_m"].as_f64().unwrap() > 0.0);
    let raised = of_layer(&v.side_elevation, "CENTRE", "polyline")
        .iter()
        .filter(|e| {
            let ys: Vec<f64> = e["points_m"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| p[1].as_f64().unwrap())
                .collect();
            ys.iter().any(|y| close(*y, floor)) && ys.iter().any(|y| close(*y, top))
        })
        .count();
    assert_eq!(raised, 1, "the raised wheelhouse");
    assert!(v.side_elevation.to_string().contains("RAISED"));
    let notes = v.side_elevation["notes"].to_string();
    assert!(
        notes.contains("BLIND DISTANCE AHEAD OF THE BOW: LIGHT 250.0 M, LADEN"),
        "{notes}"
    );
}

#[test]
fn a_legacy_stern_draws_none_of_it() {
    // A file saved before the new stern has no tunnel, no lift and no blind distance to draw.
    let (_, r) = computed("gas_igc_2g_self_propelled.vdesign").remove(0);
    let v = views(&r);
    assert!(
        !v.side_elevation["notes"]
            .to_string()
            .contains("BLIND DISTANCE")
    );
    assert!(!v.side_elevation.to_string().contains("RAISED"));
}

#[test]
fn a_hull_without_a_shield_draws_none() {
    let (_, mut r) = computed("gas_igc_2g_exact.vdesign").remove(0);
    r["drawing"]["weather_shield_thickness_mm"] = serde_json::json!(0.0);
    let v = views(&r);
    for view in [&v.midship_section, &v.side_elevation] {
        assert!(
            of_layer(view, "ZONE", "polyline")
                .iter()
                .all(|e| e["points_m"].as_array().unwrap().len() != 4),
            "no shield"
        );
    }
}

#[test]
fn a_dome_is_drawn_only_at_the_diameter_the_results_give() {
    // Issue #406: no drawing proportion stands in for the core's dome diameter.
    let (_, r) = computed("gas_igc_2g_exact.vdesign").remove(0);
    assert!(r["drawing"]["dome_height_m"].as_f64().unwrap() > 0.0);
    assert!(ovcd_format::ga_views(&r).is_ok());
    let mut broken = r;
    broken["drawing"]["tank"]
        .as_object_mut()
        .unwrap()
        .remove("dome_diameter_m");
    let err = ovcd_format::ga_views(&broken).unwrap_err();
    assert!(err.contains("drawing.tank.dome_diameter_m"), "{err}");
}

#[test]
fn results_of_an_unsupported_or_unknown_cargo_class_are_refused() {
    // Issue #328: a class with no drawing is an error, never drawn as gas tanks or a dry hold.
    for name in ["dry_bulk_box_hold.vdesign", "gas_igc_2g_exact.vdesign"] {
        let (_, r) = computed(name).remove(0);
        for phase in ["liquid_bulk", "crude_oil", ""] {
            let mut other = r.clone();
            other["phase"] = serde_json::json!(phase);
            assert!(ovcd_format::ga_views(&other).is_err(), "{name} {phase:?}");
        }
        let mut missing = r;
        missing.as_object_mut().unwrap().remove("phase");
        assert!(ovcd_format::ga_views(&missing).is_err(), "{name}");
    }
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
