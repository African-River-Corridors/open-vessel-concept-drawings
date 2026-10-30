//! Seam: the Deck Items of the Vessel Model and their meshes (spec 0009 "Deck Items", ADR 0021,
//! issue #688): the stern block's container units, the wheelhouse lowered and raised, and its lift
//! column, from `AftArrangementResult`.
//!
//! Label: property. Each test rebuilds a Deck Item from the results JSON and the hull mesh, which do
//! not share the model's construction: the units' footprints and top, the wheelhouse floors and
//! tops, the deck height under a footprint (the highest hull-mesh point over a grid), and the side
//! the general arrangement's plan draws a unit on. The fixtures with a stern block are run as they
//! are and with a cambered and sheered deck.

use std::path::Path;

use ovd_core::mesh::{Mesh, PartSource};
use ovd_core::vessel_model::{
    DeckItemKind, LIFT_COLUMN_SIDE_OVER_WHEELHOUSE_WIDTH, VesselModel, WheelhousePosition,
};
use ovd_format::{GaViews, Profile, RunOutcome, check, ga_views_of, option_vessel_run, run_option};
use serde_json::{Value, json};

/// The designs with a stern block, as `(name, file text)`: the fixtures and the website example,
/// and the gas fixture again with a cambered and sheered deck.
fn designs() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out = Vec::new();
    for path in [
        root.join("tests/fixtures/gas_igc_2g_self_propelled.vdesign"),
        root.join("tests/fixtures/crude_integral_tanks_self_propelled.vdesign"),
        root.join("../../website/assets/examples/lrg-gas-89.vdesign"),
    ] {
        let text = std::fs::read_to_string(&path).unwrap();
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        out.push((name, text));
    }
    let mut f: Value = serde_json::from_str(&out[0].1).unwrap();
    for (key, value) in [
        ("camber_m", 0.4),
        ("sheer_aft_m", 0.8),
        ("sheer_fwd_m", 1.2),
    ] {
        f["options"][0]["inputs"][key] = json!({ "value": value, "source": "typed" });
    }
    out.push((format!("{} + camber and sheer", out[0].0), f.to_string()));
    out
}

struct Case {
    at: String,
    results: Value,
    model: VesselModel,
    mesh: Mesh,
}

fn cases() -> Vec<Case> {
    let mut out = Vec::new();
    for (name, text) in designs() {
        let design = check(&text, &Profile::built_in())
            .design
            .unwrap_or_else(|| panic!("{name}: a valid Design"));
        for option in &design.options {
            let RunOutcome::Results { results, .. } = run_option(option) else {
                panic!("{name}: it runs");
            };
            let run = option_vessel_run(option).unwrap();
            out.push(Case {
                at: format!("{name} / {}", option.name),
                results,
                mesh: run.meshes(),
                model: run.model,
            });
        }
    }
    assert!(out.len() >= 4, "four designs");
    out
}

/// One run of the designs for every check: each runs the engine, so the checks share it.
#[test]
fn the_deck_items_of_every_stern_block_design() {
    let cases = cases();
    each_unit_stands_on_the_deck_under_it_up_to_the_units_top(&cases);
    the_wheelhouse_lowered_and_raised_are_the_stern_blocks(&cases);
    the_lift_column_runs_from_the_deck_to_the_raised_floor_when_the_wheelhouse_rises(&cases);
    a_unit_the_plan_draws_to_port_is_to_port_in_the_model(&cases);
    every_deck_item_mesh_is_closed_and_has_a_source(&cases);
}

/// The part of `m` in the group of `source`.
fn part(m: &Mesh, source: &PartSource) -> Mesh {
    let g = m
        .groups
        .iter()
        .find(|g| &g.source == source)
        .unwrap_or_else(|| panic!("no group for {source:?}"));
    Mesh {
        positions: m.positions.clone(),
        indices: m.indices[g.start as usize..(g.start + g.count) as usize].to_vec(),
        groups: Vec::new(),
    }
}

/// The bounding box of a mesh: ([x, y, z] min, [x, y, z] max).
fn bbox(m: &Mesh) -> ([f64; 3], [f64; 3]) {
    let mut lo = [f64::INFINITY; 3];
    let mut hi = [f64::NEG_INFINITY; 3];
    for &i in &m.indices {
        let v = m.vertex(i);
        for k in 0..3 {
            lo[k] = lo[k].min(v[k]);
            hi[k] = hi[k].max(v[k]);
        }
    }
    (lo, hi)
}

/// The hull's triangles whose plan overlaps the box `(x0, x1)` by `(y0, y1)`.
fn triangles_over(hull: &Mesh, (x0, x1): (f64, f64), (y0, y1): (f64, f64)) -> Vec<[[f64; 3]; 3]> {
    hull.indices
        .as_chunks::<3>()
        .0
        .iter()
        .map(|t| [hull.vertex(t[0]), hull.vertex(t[1]), hull.vertex(t[2])])
        .filter(|[a, b, c]| {
            let (xs, ys) = ([a[0], b[0], c[0]], [a[1], b[1], c[1]]);
            let min = |v: [f64; 3]| v[0].min(v[1]).min(v[2]);
            let max = |v: [f64; 3]| v[0].max(v[1]).max(v[2]);
            max(xs) >= x0 - 1e-6
                && min(xs) <= x1 + 1e-6
                && max(ys) >= y0 - 1e-6
                && min(ys) <= y1 + 1e-6
        })
        .collect()
}

/// The highest of `triangles` straight above `(x, y)`: the deck there.
fn hull_top_at(triangles: &[[[f64; 3]; 3]], x: f64, y: f64) -> Option<f64> {
    let mut top: Option<f64> = None;
    for [a, b, c] in triangles {
        let d = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
        if d.abs() < 1e-12 {
            continue;
        }
        let u = ((x - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (y - a[1])) / d;
        let v = ((b[0] - a[0]) * (y - a[1]) - (x - a[0]) * (b[1] - a[1])) / d;
        if u >= -1e-9 && v >= -1e-9 && u + v <= 1.0 + 1e-9 {
            let z = a[2] + u * (b[2] - a[2]) + v * (c[2] - a[2]);
            top = Some(top.map_or(z, |t: f64| t.max(z)));
        }
    }
    top
}

/// The highest deck point under a footprint, from the hull mesh on a 13 x 13 grid.
fn deck_top(hull: &Mesh, (x0, x1): (f64, f64), (y0, y1): (f64, f64)) -> f64 {
    let triangles = triangles_over(hull, (x0, x1), (y0, y1));
    let n = 12;
    let mut top = f64::NEG_INFINITY;
    for i in 0..=n {
        for j in 0..=n {
            let x = x0 + (x1 - x0) * i as f64 / n as f64;
            let y = y0 + (y1 - y0) * j as f64 / n as f64;
            if let Some(z) = hull_top_at(&triangles, x, y) {
                top = top.max(z);
            }
        }
    }
    top
}

/// The general arrangement's plan of a result: the centre of each EQUIP box, `[x, y]` on paper.
fn plan_boxes(results: &Value) -> Vec<[f64; 2]> {
    let views: GaViews = ga_views_of(&RunOutcome::Results {
        results: results.clone(),
        hash: String::new(),
    })
    .expect("views");
    views.plan["viewports"][0]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["type"] == "polyline" && e["layer"] == "EQUIP")
        .map(|e| {
            let pts = e["points_m"].as_array().unwrap();
            let n = pts.len() as f64;
            let sum = |k: usize| pts.iter().map(|p| p[k].as_f64().unwrap()).sum::<f64>() / n;
            [sum(0), sum(1)]
        })
        .collect()
}

fn each_unit_stands_on_the_deck_under_it_up_to_the_units_top(cases: &[Case]) {
    let mut units = 0;
    let mut above_side = 0;
    for c in cases {
        let a = &c.results["drawing"]["aft_arrangement"];
        let list = a["container_units"].as_array().unwrap();
        let hull = part(&c.mesh, &PartSource::Hull);
        let width = a["container_unit_width_m"].as_f64().unwrap();
        let top = a["container_top_above_keel_m"].as_f64().unwrap();
        let depth = c.results["dims"]["depth_m"].as_f64().unwrap();
        for (i, u) in list.iter().enumerate() {
            let (x0, x1, y_port_up) = (
                u[1].as_f64().unwrap(),
                u[2].as_f64().unwrap(),
                u[3].as_f64().unwrap(),
            );
            // `container_units` y is + to port; the model's is + to starboard.
            let (y0, y1) = (-y_port_up - width / 2.0, -y_port_up + width / 2.0);
            let base = deck_top(&hull, (x0, x1), (y0, y1));
            let (lo, hi) = bbox(&part(&c.mesh, &PartSource::Unit(i)));
            let at = format!("{} unit {}", c.at, i + 1);
            for (got, want) in [
                (lo[0], x0),
                (hi[0], x1),
                (lo[1], y0),
                (hi[1], y1),
                (lo[2], base),
                (hi[2], top),
            ] {
                assert!((got - want).abs() <= 1e-3, "{at}: {got} vs {want}");
            }
            if base > depth + 1e-3 {
                above_side += 1;
            }
            units += 1;
        }
    }
    assert!(units >= 9, "{units} units");
    assert!(above_side > 0, "a cambered or sheered deck lifts a unit");
}

fn the_wheelhouse_lowered_and_raised_are_the_stern_blocks(cases: &[Case]) {
    let mut checked = 0;
    for c in cases {
        let a = &c.results["drawing"]["aft_arrangement"];
        let n = |k: &str| a[k].as_f64().unwrap();
        let half = n("wheelhouse_width_m") / 2.0;
        for (position, floor, top) in [
            (
                WheelhousePosition::Lowered,
                n("wheelhouse_floor_lowered_m"),
                n("wheelhouse_top_lowered_m"),
            ),
            (
                WheelhousePosition::Raised,
                n("wheelhouse_floor_raised_m"),
                n("wheelhouse_top_raised_m"),
            ),
        ] {
            let (lo, hi) = bbox(&part(&c.mesh, &PartSource::Wheelhouse(position)));
            let at = format!("{} {position:?}", c.at);
            for (got, want) in [
                (lo[0], n("wheelhouse_x0_m")),
                (hi[0], n("wheelhouse_x1_m")),
                (lo[1], -half),
                (hi[1], half),
                (lo[2], floor),
                (hi[2], top),
            ] {
                assert!((got - want).abs() <= 1e-3, "{at}: {got} vs {want}");
            }
        }
        // The lowered top is the stern block's top that the light air draft takes (with the
        // units' top), and the light air draft is from a point at least that high.
        let lowered = c
            .model
            .outfit
            .deck_items
            .iter()
            .find(|d| d.kind == DeckItemKind::Wheelhouse(WheelhousePosition::Lowered))
            .unwrap();
        assert_eq!(lowered.z_top_m, n("wheelhouse_top_lowered_m"));
        assert_eq!(
            n("block_top_lowered_m"),
            lowered.z_top_m.max(n("container_top_above_keel_m")),
            "{}",
            c.at
        );
        // A liquid bulk hull reports no light air draft (`float:nan`).
        let r = &c.results["results"];
        if let (Some(air), Some(t)) = (
            r["air_draft_light_m"].as_f64(),
            r["light_draught_m"].as_f64(),
        ) {
            assert!(lowered.z_top_m <= air + t + 1e-9, "{}", c.at);
            checked += 1;
        }
    }
    assert!(checked > 0, "a design with a light air draft");
}

fn the_lift_column_runs_from_the_deck_to_the_raised_floor_when_the_wheelhouse_rises(
    cases: &[Case],
) {
    let mut columns = 0;
    for c in cases {
        let a = &c.results["drawing"]["aft_arrangement"];
        let n = |k: &str| a[k].as_f64().unwrap();
        let col = c
            .model
            .outfit
            .deck_items
            .iter()
            .find(|d| d.kind == DeckItemKind::LiftColumn);
        let hull = part(&c.mesh, &PartSource::Hull);
        let side = LIFT_COLUMN_SIDE_OVER_WHEELHOUSE_WIDTH * n("wheelhouse_width_m");
        let mid = (n("wheelhouse_x0_m") + n("wheelhouse_x1_m")) / 2.0;
        let foot = (mid - side / 2.0, mid + side / 2.0);
        let base = deck_top(&hull, foot, (-side / 2.0, side / 2.0));
        if n("wheelhouse_floor_raised_m") <= base + 1e-9 {
            assert!(col.is_none(), "{}: no lift, no column", c.at);
            continue;
        }
        assert!(col.is_some(), "{}", c.at);
        let (lo, hi) = bbox(&part(&c.mesh, &PartSource::LiftColumn));
        for (got, want) in [
            (lo[0], foot.0),
            (hi[0], foot.1),
            (lo[1], -side / 2.0),
            (hi[1], side / 2.0),
            (lo[2], base),
            (hi[2], n("wheelhouse_floor_raised_m")),
        ] {
            assert!((got - want).abs() <= 1e-3, "{}: {got} vs {want}", c.at);
        }
        columns += 1;
    }
    assert!(columns > 0, "a fixture lifts its wheelhouse");
}

fn a_unit_the_plan_draws_to_port_is_to_port_in_the_model(cases: &[Case]) {
    // The general arrangement's plan draws port up the sheet (+y on paper), as `container_units`
    // gives it; the model's y is + to starboard, so a unit drawn to port has y < 0.
    let mut port = 0;
    for c in cases {
        let boxes = plan_boxes(&c.results);
        for item in &c.model.outfit.deck_items {
            let DeckItemKind::Unit { index, .. } = item.kind else {
                continue;
            };
            let x = (item.x_aft_m + item.x_fwd_m) / 2.0;
            let y = (item.y_port_m + item.y_stbd_m) / 2.0;
            let drawn = boxes
                .iter()
                .find(|b| (b[0] - x).abs() < 1e-6 && (b[1] + y).abs() < 1e-6)
                .unwrap_or_else(|| panic!("{} unit {}: not in the plan", c.at, index + 1));
            if drawn[1] > 0.0 {
                assert!(y < 0.0, "{}: unit {} drawn to port", c.at, index + 1);
                port += 1;
            } else if drawn[1] < 0.0 {
                assert!(y > 0.0, "{}: unit {} drawn to starboard", c.at, index + 1);
            }
        }
    }
    assert!(port > 0, "a unit is drawn to port");
}

fn every_deck_item_mesh_is_closed_and_has_a_source(cases: &[Case]) {
    for c in cases {
        let items: Vec<_> = c
            .mesh
            .groups
            .iter()
            .filter(|g| g.layer == "deck_item")
            .collect();
        assert_eq!(items.len(), c.model.outfit.deck_items.len(), "{}", c.at);
        for g in &c.mesh.groups {
            assert_eq!(g.layer, g.source.layer(), "{}", c.at);
            assert_eq!(g.id, g.source.id(), "{}", c.at);
        }
        for g in items {
            let p = part(&c.mesh, &g.source);
            let (lo, hi) = bbox(&p);
            let box_volume = (hi[0] - lo[0]) * (hi[1] - lo[1]) * (hi[2] - lo[2]);
            assert!(
                (p.volume() - box_volume).abs() <= 1e-6 * box_volume.max(1.0),
                "{} {}: outward and closed",
                c.at,
                g.id
            );
            let mut edges = std::collections::HashMap::new();
            for t in p.indices.as_chunks::<3>().0 {
                for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                    *edges.entry((a, b)).or_insert(0) += 1;
                }
            }
            for (&(a, b), &n) in &edges {
                assert!(
                    n == 1 && edges.get(&(b, a)) == Some(&1),
                    "{} {}",
                    c.at,
                    g.id
                );
            }
        }
    }
}
