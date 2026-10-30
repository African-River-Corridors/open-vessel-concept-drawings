//! Seam: `ovd_format::option_meshes` and `mesh_hash` (issue #456, spec 0008 "Phase 2: meshes and
//! the parity hash").
//!
//! Label: property - for every fixture Option the hull mesh is closed (every edge in exactly two
//! triangles, once in each direction) and holds the sections volume within the ruled-quad error
//! (the sum of half the tetrahedron volumes of the ruled quads) plus 1e-6 for the f32 rounding;
//! the group table names one hull and one group per Space; the hash is the same twice and a
//! one-vertex change changes it. Label: snapshot - the triangle count of each layer of each fixture
//! when the triangle budget landed (issue #690); each layer stays within 1.5 x it.

use std::collections::HashMap;
use std::path::Path;

use ovd_core::mesh::Mesh;
use ovd_core::sections::{SectionsTable, Waterplane, immersed};
use ovd_format::{Profile, check, mesh_hash, option_hull_table, option_meshes};

fn hull_part(m: &Mesh) -> Mesh {
    let g = m
        .groups
        .iter()
        .find(|g| g.layer == "hull")
        .expect("a hull group");
    let idx = &m.indices[g.start as usize..(g.start + g.count) as usize];
    Mesh {
        positions: m.positions.clone(),
        indices: idx.to_vec(),
        groups: Vec::new(),
    }
}

fn closed(m: &Mesh) -> Result<(), String> {
    let mut count: HashMap<(u32, u32), i32> = HashMap::new();
    for t in m.indices.as_chunks::<3>().0 {
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            *count.entry((a, b)).or_default() += 1;
        }
    }
    for (&(a, b), &n) in &count {
        if n != 1 || count.get(&(b, a)) != Some(&1) {
            return Err(format!("edge {:?} {:?}", m.vertex(a), m.vertex(b)));
        }
    }
    Ok(())
}

fn quad_error(t: &SectionsTable) -> f64 {
    let tet = |a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3]| {
        let (u, v, w) = (
            [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
            [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
            [d[0] - a[0], d[1] - a[1], d[2] - a[2]],
        );
        (u[0] * (v[1] * w[2] - v[2] * w[1]) - u[1] * (v[0] * w[2] - v[2] * w[0])
            + u[2] * (v[0] * w[1] - v[1] * w[0]))
            .abs()
            / 6.0
    };
    let mut e = 0.0;
    for p in t.sections.windows(2) {
        let (s0, s1) = (&p[0], &p[1]);
        for i in 0..s0.points.len() - 1 {
            let at =
                |s: &ovd_core::sections::Section, j: usize| [s.x, s.points[j].0, s.points[j].1];
            e += tet(at(s0, i), at(s0, i + 1), at(s1, i + 1), at(s1, i));
        }
    }
    e
}

#[test]
fn every_fixture_hull_mesh_is_closed_with_the_sections_volume_and_a_stable_hash() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "vdesign"))
        .collect();
    files.sort();
    let mut meshed = 0;
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap();
        let design = check(&text, &Profile::built_in())
            .design
            .expect("a valid fixture");
        for option in &design.options {
            let Ok(m) = option_meshes(option) else {
                continue;
            };
            let at = format!("{} / {}", path.display(), option.name);
            let hull = hull_part(&m);
            closed(&hull).unwrap_or_else(|e| panic!("{at}: not closed at {e}"));
            // The gas search may change the depth, so the gas hull is checked by its own volume.
            let table = option_hull_table(option).unwrap();
            if !path.to_string_lossy().contains("gas") {
                let v = immersed(&table, &Waterplane::upright(1e6)).volume_m3;
                let bound = quad_error(&table) + 1e-6 * v;
                assert!(
                    (hull.volume() - v).abs() <= bound,
                    "{at}: mesh {} vs sections {v}, bound {bound}",
                    hull.volume()
                );
            } else {
                assert!(hull.volume() > 0.0, "{at}");
            }
            assert_eq!(m.groups.iter().filter(|g| g.layer == "hull").count(), 1);
            assert!(m.groups.iter().any(|g| g.layer == "space"), "{at}");
            assert!(m.groups.iter().any(|g| g.layer == "waterline"), "{at}");
            let h = mesh_hash(&m);
            assert_eq!(h, mesh_hash(&m.clone()));
            let mut moved = m.clone();
            moved.positions[0] += 0.001;
            assert_ne!(
                h,
                mesh_hash(&moved),
                "{at}: a moved vertex changes the hash"
            );
            meshed += 1;
        }
    }
    assert!(meshed >= 10, "{meshed} Options meshed");
}

#[test]
fn every_space_mesh_of_every_fixture_is_closed_and_holds_the_space_volume() {
    // Issue #633: a Space the hull cuts is meshed as the Space inside the hull, so each Space mesh is
    // closed and its volume is the Space's clipped volume (a fixed hull and Spaces: the ruled-quad
    // error of a curved hull between its sections: 0.1 % for a pram stern, 0.5 % for the coarsely cut ShipD bow; the bound is 2 %).
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "vdesign"))
        .collect();
    files.sort();
    let (mut spaces, mut cut) = (0, 0);
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap();
        let design = check(&text, &Profile::built_in())
            .design
            .expect("a valid fixture");
        for option in &design.options {
            let Ok(m) = option_meshes(option) else {
                continue;
            };
            let model = ovd_format::option_vessel_model(option).unwrap();
            for g in m.groups.iter().filter(|g| g.layer == "space") {
                let at = format!("{} / {} / {}", path.display(), option.name, g.id);
                let part = Mesh {
                    positions: m.positions.clone(),
                    indices: m.indices[g.start as usize..(g.start + g.count) as usize].to_vec(),
                    groups: Vec::new(),
                };
                if part.indices.is_empty() {
                    continue;
                }
                closed(&part).unwrap_or_else(|e| panic!("{at}: not closed at {e}"));
                let s = model.spaces.iter().find(|s| s.space.name == g.id).unwrap();
                // A tank Space holds its interior volume, but its mesh is the outer shape (ADR 0021):
                // `tank_meshes.rs` checks it against the closed form at the outer diameter.
                if matches!(
                    s.space.shape,
                    ovd_core::compartments::SpaceShape::Tank { .. }
                ) {
                    continue;
                }
                assert!(
                    (part.volume() - s.volume_m3).abs() <= 2e-2 * s.volume_m3.max(1e-9) + 1e-5,
                    "{at}: mesh {} vs {}",
                    part.volume(),
                    s.volume_m3
                );
                spaces += 1;
                cut += usize::from(s.is_cut());
            }
        }
    }
    assert!(spaces >= 10, "{spaces} Space meshes");
    assert!(cut >= 1, "a fixture has a Space the hull cuts");
}

/// The triangles of each layer of each fixture and example Option when the triangle budget landed
/// (issue #690, spec 0009 "Performance budget"); the `propulsor` layer since issue #692. Label:
/// snapshot - what `option_meshes` printed.
const TRIANGLES: &[(&str, &str, &str, u32)] = &[
    ("gas-140x29-coastal-lng.vdesign", "A basis", "bulkhead", 36),
    ("gas-140x29-coastal-lng.vdesign", "A basis", "deck", 2),
    ("gas-140x29-coastal-lng.vdesign", "A basis", "deck_item", 60),
    ("gas-140x29-coastal-lng.vdesign", "A basis", "dome", 384),
    ("gas-140x29-coastal-lng.vdesign", "A basis", "hull", 38),
    (
        "gas-140x29-coastal-lng.vdesign",
        "A basis",
        "propulsor",
        1752,
    ),
    ("gas-140x29-coastal-lng.vdesign", "A basis", "space", 9410),
    ("gas-140x29-coastal-lng.vdesign", "A basis", "waterline", 16),
    (
        "gas-140x29-coastal-lng.vdesign",
        "A basis",
        "weather_shield",
        60,
    ),
    (
        "gas-140x29-coastal-lng.vdesign",
        "B profile",
        "bulkhead",
        36,
    ),
    ("gas-140x29-coastal-lng.vdesign", "B profile", "deck", 2),
    (
        "gas-140x29-coastal-lng.vdesign",
        "B profile",
        "deck_item",
        60,
    ),
    ("gas-140x29-coastal-lng.vdesign", "B profile", "dome", 384),
    ("gas-140x29-coastal-lng.vdesign", "B profile", "hull", 38),
    (
        "gas-140x29-coastal-lng.vdesign",
        "B profile",
        "propulsor",
        1752,
    ),
    ("gas-140x29-coastal-lng.vdesign", "B profile", "space", 9410),
    (
        "gas-140x29-coastal-lng.vdesign",
        "B profile",
        "waterline",
        16,
    ),
    (
        "gas-140x29-coastal-lng.vdesign",
        "B profile",
        "weather_shield",
        60,
    ),
    ("gas-140x29-coastal-lng.vdesign", "C deep", "bulkhead", 36),
    ("gas-140x29-coastal-lng.vdesign", "C deep", "deck", 2),
    ("gas-140x29-coastal-lng.vdesign", "C deep", "deck_item", 60),
    ("gas-140x29-coastal-lng.vdesign", "C deep", "dome", 384),
    ("gas-140x29-coastal-lng.vdesign", "C deep", "hull", 38),
    (
        "gas-140x29-coastal-lng.vdesign",
        "C deep",
        "propulsor",
        1752,
    ),
    ("gas-140x29-coastal-lng.vdesign", "C deep", "space", 9418),
    ("gas-140x29-coastal-lng.vdesign", "C deep", "waterline", 16),
    ("lrg-gas-89.vdesign", "LRG Gas 89", "bulkhead", 28),
    ("lrg-gas-89.vdesign", "LRG Gas 89", "deck", 0),
    ("lrg-gas-89.vdesign", "LRG Gas 89", "deck_item", 72),
    ("lrg-gas-89.vdesign", "LRG Gas 89", "dome", 768),
    ("lrg-gas-89.vdesign", "LRG Gas 89", "hull", 26),
    ("lrg-gas-89.vdesign", "LRG Gas 89", "propulsor", 2340),
    ("lrg-gas-89.vdesign", "LRG Gas 89", "space", 18506),
    ("lrg-gas-89.vdesign", "LRG Gas 89", "waterline", 9),
    (
        "crude_integral_tanks_coastal.vdesign",
        "Option 1",
        "bulkhead",
        32,
    ),
    (
        "crude_integral_tanks_coastal.vdesign",
        "Option 1",
        "deck",
        2,
    ),
    (
        "crude_integral_tanks_coastal.vdesign",
        "Option 1",
        "hull",
        26,
    ),
    (
        "crude_integral_tanks_coastal.vdesign",
        "Option 1",
        "space",
        218,
    ),
    (
        "crude_integral_tanks_coastal.vdesign",
        "Option 1",
        "waterline",
        18,
    ),
    (
        "crude_integral_tanks_pushed.vdesign",
        "Option 1",
        "bulkhead",
        28,
    ),
    ("crude_integral_tanks_pushed.vdesign", "Option 1", "deck", 2),
    (
        "crude_integral_tanks_pushed.vdesign",
        "Option 1",
        "hull",
        26,
    ),
    (
        "crude_integral_tanks_pushed.vdesign",
        "Option 1",
        "space",
        194,
    ),
    (
        "crude_integral_tanks_pushed.vdesign",
        "Option 1",
        "waterline",
        18,
    ),
    (
        "crude_integral_tanks_self_propelled.vdesign",
        "Option 1",
        "bulkhead",
        192,
    ),
    (
        "crude_integral_tanks_self_propelled.vdesign",
        "Option 1",
        "deck",
        2,
    ),
    (
        "crude_integral_tanks_self_propelled.vdesign",
        "Option 1",
        "deck_item",
        60,
    ),
    (
        "crude_integral_tanks_self_propelled.vdesign",
        "Option 1",
        "hull",
        1306,
    ),
    (
        "crude_integral_tanks_self_propelled.vdesign",
        "Option 1",
        "propulsor",
        1392,
    ),
    (
        "crude_integral_tanks_self_propelled.vdesign",
        "Option 1",
        "space",
        1508,
    ),
    (
        "crude_integral_tanks_self_propelled.vdesign",
        "Option 1",
        "waterline",
        175,
    ),
    (
        "diesel_integral_tanks_pushed.vdesign",
        "Option 1",
        "bulkhead",
        36,
    ),
    (
        "diesel_integral_tanks_pushed.vdesign",
        "Option 1",
        "deck",
        2,
    ),
    (
        "diesel_integral_tanks_pushed.vdesign",
        "Option 1",
        "hull",
        26,
    ),
    (
        "diesel_integral_tanks_pushed.vdesign",
        "Option 1",
        "space",
        242,
    ),
    (
        "diesel_integral_tanks_pushed.vdesign",
        "Option 1",
        "waterline",
        18,
    ),
    ("dry_bulk_box_hold.vdesign", "Option 1", "bulkhead", 16),
    ("dry_bulk_box_hold.vdesign", "Option 1", "deck", 2),
    ("dry_bulk_box_hold.vdesign", "Option 1", "hull", 26),
    ("dry_bulk_box_hold.vdesign", "Option 1", "space", 260),
    ("dry_bulk_box_hold.vdesign", "Option 1", "waterline", 15),
    ("dry_bulk_sloped_hopper.vdesign", "Option 1", "bulkhead", 16),
    ("dry_bulk_sloped_hopper.vdesign", "Option 1", "deck", 2),
    ("dry_bulk_sloped_hopper.vdesign", "Option 1", "hull", 26),
    ("dry_bulk_sloped_hopper.vdesign", "Option 1", "space", 290),
    ("dry_bulk_sloped_hopper.vdesign", "Option 1", "waterline", 6),
    ("gas_igc_2g_exact.vdesign", "Option 1", "bulkhead", 20),
    ("gas_igc_2g_exact.vdesign", "Option 1", "deck", 0),
    ("gas_igc_2g_exact.vdesign", "Option 1", "dome", 192),
    ("gas_igc_2g_exact.vdesign", "Option 1", "hull", 26),
    ("gas_igc_2g_exact.vdesign", "Option 1", "space", 4730),
    ("gas_igc_2g_exact.vdesign", "Option 1", "waterline", 15),
    ("gas_igc_2g_exact.vdesign", "Option 1", "weather_shield", 60),
    (
        "gas_igc_2g_self_propelled.vdesign",
        "Option 1",
        "bulkhead",
        28,
    ),
    ("gas_igc_2g_self_propelled.vdesign", "Option 1", "deck", 0),
    (
        "gas_igc_2g_self_propelled.vdesign",
        "Option 1",
        "deck_item",
        72,
    ),
    ("gas_igc_2g_self_propelled.vdesign", "Option 1", "dome", 192),
    ("gas_igc_2g_self_propelled.vdesign", "Option 1", "hull", 26),
    (
        "gas_igc_2g_self_propelled.vdesign",
        "Option 1",
        "propulsor",
        2340,
    ),
    (
        "gas_igc_2g_self_propelled.vdesign",
        "Option 1",
        "space",
        4730,
    ),
    (
        "gas_igc_2g_self_propelled.vdesign",
        "Option 1",
        "waterline",
        12,
    ),
    (
        "gas_igc_2g_self_propelled.vdesign",
        "Option 1",
        "weather_shield",
        60,
    ),
    ("offsets_dry_bulk.vdesign", "Option 1", "bulkhead", 16),
    ("offsets_dry_bulk.vdesign", "Option 1", "deck", 6),
    ("offsets_dry_bulk.vdesign", "Option 1", "hull", 44),
    ("offsets_dry_bulk.vdesign", "Option 1", "space", 390),
    ("offsets_dry_bulk.vdesign", "Option 1", "waterline", 10),
    ("pushed_convoy.vdesign", "Option 1", "bulkhead", 16),
    ("pushed_convoy.vdesign", "Option 1", "deck", 2),
    ("pushed_convoy.vdesign", "Option 1", "hull", 26),
    ("pushed_convoy.vdesign", "Option 1", "space", 260),
    ("pushed_convoy.vdesign", "Option 1", "waterline", 6),
    ("shipd_dry_bulk.vdesign", "Option 1", "bulkhead", 180),
    ("shipd_dry_bulk.vdesign", "Option 1", "deck", 62),
    ("shipd_dry_bulk.vdesign", "Option 1", "hull", 4372),
    ("shipd_dry_bulk.vdesign", "Option 1", "space", 7822),
    ("shipd_dry_bulk.vdesign", "Option 1", "waterline", 150),
    ("towed_barge.vdesign", "Towed barge", "bulkhead", 16),
    ("towed_barge.vdesign", "Towed barge", "deck", 2),
    ("towed_barge.vdesign", "Towed barge", "hull", 26),
    ("towed_barge.vdesign", "Towed barge", "space", 260),
    ("towed_barge.vdesign", "Towed barge", "waterline", 15),
    ("two_options.vdesign", "Base", "bulkhead", 16),
    ("two_options.vdesign", "Base", "deck", 2),
    ("two_options.vdesign", "Base", "hull", 26),
    ("two_options.vdesign", "Base", "space", 260),
    ("two_options.vdesign", "Base", "waterline", 6),
    ("two_options.vdesign", "Option 2", "bulkhead", 16),
    ("two_options.vdesign", "Option 2", "deck", 2),
    ("two_options.vdesign", "Option 2", "hull", 26),
    ("two_options.vdesign", "Option 2", "space", 260),
    ("two_options.vdesign", "Option 2", "waterline", 6),
];

/// Every fixture and example design, by file name.
fn budget_designs() -> Vec<std::path::PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files: Vec<_> = std::fs::read_dir(root.join("tests/fixtures"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "vdesign"))
        .collect();
    files.push(root.join("../../website/assets/examples/lrg-gas-89.vdesign"));
    files.push(root.join("../../examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign"));
    files.sort();
    files
}

/// Label: property (the 1.5 x bound and the 200,000 cap of spec 0009) over the snapshot above. Each
/// layer of each Option stays within 1.5 x its pinned count, so doubling a tessellation constant
/// (`TANK_SEGMENTS`, `HEAD_RINGS`, `DOME_SEGMENTS`, `PROPULSOR_SEGMENTS`) fails it; and no Option passes 200,000.
#[test]
fn every_fixture_layer_stays_within_its_triangle_budget() {
    let mut seen = 0;
    for path in budget_designs() {
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        let design = check(
            &std::fs::read_to_string(&path).unwrap(),
            &Profile::built_in(),
        )
        .design
        .expect("a valid design");
        for option in &design.options {
            let Ok(m) = option_meshes(option) else {
                continue;
            };
            let total = m.indices.len() / 3;
            assert!(
                total <= 200_000,
                "{file} / {}: {total} triangles",
                option.name
            );
            let mut layers: HashMap<&str, u32> = HashMap::new();
            for g in &m.groups {
                *layers.entry(g.layer.as_str()).or_default() += g.count / 3;
            }
            for (layer, count) in layers {
                let pinned = TRIANGLES
                    .iter()
                    .find(|(f, o, l, _)| *f == file && *o == option.name && *l == layer)
                    .unwrap_or_else(|| panic!("{file} / {} / {layer}: no budget", option.name))
                    .3;
                assert!(
                    f64::from(count) <= 1.5 * f64::from(pinned),
                    "{file} / {} / {layer}: {count} triangles over 1.5 x {pinned}",
                    option.name
                );
                seen += 1;
            }
        }
    }
    assert_eq!(seen, TRIANGLES.len(), "every pinned layer is checked");
}

/// Label: property - the largest design of the sweep (a 180 x 24 m gas barge, as in the core's
/// sweeps, with camber and sheer for the most Stations) stays under 200,000 triangles.
#[test]
fn the_largest_sweep_design_stays_under_200000_triangles() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let text =
        std::fs::read_to_string(root.join("tests/fixtures/gas_igc_2g_self_propelled.vdesign"))
            .unwrap();
    let mut f: serde_json::Value = serde_json::from_str(&text).unwrap();
    for (key, value) in [
        ("length_overall_m", 180.0),
        ("beam_m", 24.0),
        ("moulded_depth_m", 10.0),
        ("camber_m", 0.4),
        ("sheer_aft_m", 0.8),
        ("sheer_fwd_m", 1.2),
    ] {
        f["options"][0]["inputs"][key] = serde_json::json!({ "value": value, "source": "typed" });
    }
    let design = check(&f.to_string(), &Profile::built_in())
        .design
        .expect("a valid design");
    let m = option_meshes(&design.options[0]).expect("meshes");
    let tanks = m
        .groups
        .iter()
        .filter(|g| g.layer == "space" && g.id.starts_with("CT"))
        .count();
    assert!(tanks >= 2, "{tanks} tanks");
    assert!(
        m.indices.len() / 3 <= 200_000,
        "{} triangles",
        m.indices.len() / 3
    );
}
