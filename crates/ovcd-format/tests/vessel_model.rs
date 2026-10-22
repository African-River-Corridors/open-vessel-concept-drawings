//! Seam: `ovcd_format::option_vessel_model`, the Vessel Model of each fixture Option (issue #454,
//! spec 0008 "Phase 2: the Vessel Model").
//!
//! Label: property - the model has the hull the run floats, a bulkhead at every transverse line the
//! general arrangement's elevation draws, and every Space with a kind and a unique name. Label:
//! snapshot - the Spaces' prism volumes against the volumes the results give: the dry bulk hold
//! (its section from `geometry` times the cargo zone), each ballast tank (`loading_conditions`,
//! printed to 0.001 m³) and the liquid bulk cargo tanks (`liquid.cargo_gross_m3`).

use std::collections::HashSet;
use std::path::Path;
use std::sync::OnceLock;

use ovcd_core::compartments::SpaceKind;
use ovcd_core::vessel_model::VesselModel;
use ovcd_format::{
    GaViews, Profile, RunOutcome, check, ga_views_of, option_hull_table, option_vessel_model,
    run_option,
};
use serde_json::Value;

const FIXTURES: &[&str] = &[
    "crude_integral_tanks_coastal.vdesign",
    "crude_integral_tanks_pushed.vdesign",
    "crude_integral_tanks_self_propelled.vdesign",
    "dry_bulk_box_hold.vdesign",
    "dry_bulk_sloped_hopper.vdesign",
    "gas_igc_2g_exact.vdesign",
    "gas_igc_2g_self_propelled.vdesign",
    "offsets_dry_bulk.vdesign",
    "shipd_dry_bulk.vdesign",
    "two_options.vdesign",
];

type Case = (
    String,
    String,
    Value,
    VesselModel,
    ovcd_core::sections::SectionsTable,
);

/// Each computed Option of every fixture: (file, Option name, results, model, hull table), worked
/// once.
fn models() -> &'static [Case] {
    static MODELS: OnceLock<Vec<Case>> = OnceLock::new();
    MODELS.get_or_init(work_models)
}

fn work_models() -> Vec<Case> {
    let mut out = Vec::new();
    for file in FIXTURES {
        let text = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(file),
        )
        .expect("fixture");
        let design = check(&text, &Profile::built_in())
            .design
            .expect("a fixture is a valid Design");
        for option in &design.options {
            let RunOutcome::Results { results, .. } = run_option(option) else {
                continue;
            };
            let model = option_vessel_model(option).expect("a Vessel Model");
            let table = option_hull_table(option).expect("a hull");
            out.push((file.to_string(), option.name.clone(), results, model, table));
        }
    }
    assert!(out.len() >= FIXTURES.len(), "every fixture gives a model");
    out
}

fn elevation_x_lines(results: &Value) -> Vec<f64> {
    let views: GaViews = ga_views_of(&RunOutcome::Results {
        results: results.clone(),
        hash: String::new(),
    })
    .expect("views");
    views.side_elevation["viewports"][0]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["type"] == "line" && (e["layer"] == "HIDDEN" || e["layer"] == "ZONE"))
        .filter_map(|e| {
            let (a, b) = (e["from_m"][0].as_f64()?, e["to_m"][0].as_f64()?);
            ((a - b).abs() < 1e-9).then_some(a)
        })
        .collect()
}

#[test]
fn every_fixture_option_has_its_hull_the_ga_bulkheads_named_spaces_and_their_capacities() {
    let (mut lines, mut machinery) = (0, 0);
    for (file, name, results, model, table) in models() {
        let at = format!("{file} / {name}");
        let loa = results["dims"]["loa_m"].as_f64().unwrap();
        // The gas arrangement search may change the depth: its hull is the one it found.
        if !file.starts_with("gas") {
            assert_eq!(&model.hull, table, "{at}: hull");
        }
        let last = model.hull.sections.last().unwrap().x - model.hull.sections[0].x;
        assert!((last - loa).abs() < 1e-9, "{at}: hull length {last}");
        for x in elevation_x_lines(results) {
            if x > 1e-9 && x < loa - 1e-9 {
                lines += 1;
                assert!(
                    model.bulkheads.iter().any(|b| (b.x_m - x).abs() < 1e-9),
                    "{at}: the GA draws a bulkhead at x = {x}"
                );
            }
        }
        assert!(
            model.bulkheads.windows(2).all(|w| w[0].x_m < w[1].x_m),
            "{at}: bulkheads in ascending x, one per x"
        );
        let mut names = HashSet::new();
        for s in &model.spaces {
            assert!(!s.space.name.is_empty(), "{at}");
            assert!(
                names.insert(s.space.name.clone()),
                "{at}: {} twice",
                s.space.name
            );
            assert!(!s.space.kind.key().is_empty());
            assert!(s.volume_m3 > 0.0, "{at}: {}", s.space.name);
        }
        assert!(model.frames.is_empty());
        assert_eq!(model.decks[0].name, "main_deck");
        let kinds: Vec<SpaceKind> = model.spaces.iter().map(|s| s.space.kind).collect();
        let cargo = if file.starts_with("crude") {
            SpaceKind::CargoTank
        } else {
            SpaceKind::CargoHold
        };
        assert!(kinds.contains(&cargo), "{at}: no {}", cargo.key());
        if results["drawing"]["aft_arrangement"].is_object() {
            assert!(
                kinds.contains(&SpaceKind::Machinery),
                "{at}: no machinery space"
            );
            machinery += 1;
        }
    }
    // Not vacuous: the GA's lines were matched, and the self-propelled fixtures have an aft block.
    assert!(
        lines > 20 && machinery >= 2,
        "{lines} lines, {machinery} aft blocks"
    );
    the_space_volumes_are_the_capacities_the_results_give();
}

/// The Space volumes against the capacities the results give. Called from the test above, so the
/// fixtures run once.
fn the_space_volumes_are_the_capacities_the_results_give() {
    let mut checked = [0; 3];
    for (file, name, results, model, _) in models() {
        let at = format!("{file} / {name}");
        let volume = |kind: SpaceKind| -> f64 {
            model
                .spaces
                .iter()
                .filter(|s| s.space.kind == kind)
                .map(|s| s.volume_m3)
                .sum()
        };
        let g = &results["geometry"];
        if file.starts_with("dry") || file.starts_with("offsets") || file.starts_with("shipd") {
            let (wt, wb, h, l) = (
                g["hold_top_width_m"].as_f64().unwrap(),
                g["hold_bottom_width_m"].as_f64().unwrap(),
                g["hold_height_m"].as_f64().unwrap(),
                g["cargo_zone_length_m"].as_f64().unwrap(),
            );
            let hold = 0.5 * (wt + wb) * h * l;
            let v = volume(SpaceKind::CargoHold);
            assert!((v - hold).abs() <= 1e-9 * hold, "{at}: holds {v} vs {hold}");
            checked[0] += 1;
        }
        if let Some(gross) = results["liquid"]["cargo_gross_m3"].as_f64() {
            let v = volume(SpaceKind::CargoTank);
            assert!(
                (v - gross).abs() <= 1e-9 * gross,
                "{at}: tanks {v} vs {gross}"
            );
            checked[1] += 1;
        }
        if let Some(tanks) = results["loading_conditions"]["ballast_tanks"].as_array() {
            for t in tanks {
                let n = t["name"].as_str().unwrap();
                let s = model
                    .spaces
                    .iter()
                    .find(|s| s.space.name == n)
                    .unwrap_or_else(|| panic!("{at}: no Space {n}"));
                let c = t["capacity_m3"].as_f64().unwrap();
                assert!(
                    (s.volume_m3 - c).abs() <= 5e-4,
                    "{at}: {n} {} vs {c}",
                    s.volume_m3
                );
                checked[2] += 1;
            }
        }
    }
    assert!(checked.iter().all(|&n| n > 0), "{checked:?}");
}
