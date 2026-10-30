//! Seam: the propulsor meshes of the Vessel Model and their glTF nodes (issue #692, spec 0009
//! "Propulsors" in the view), from the Propulsor Layout of #691.
//!
//! Label: property. For every self-propelled fixture and example: each propulsor part's bounding
//! box is rebuilt from `drawing.propulsors` in the results JSON and compared with its mesh's box to
//! 1 mm, and every `propulsor` group has such a source; each propeller disc (with an azimuth unit's
//! pod on the same centre) has its centroid at its layout place, the disc centre on the shaft axis
//! (`powering.shaft_axis_height_m` where the result has one); each rudder and the bow thruster
//! have their centroid at their mass item's LCG and KCG; every part is closed and faces out; and
//! each part is a named glTF node with the propulsor material. The pinned counts and places of one
//! azimuth gas fixture and one tunnel-stern shaft fixture are labelled `snapshot` where they are
//! written.
//!
//! The spec's "each propulsor mesh's centroid equals its mass item's LCG and KCG" is read as: the
//! disc's centroid equals its layout place, and that place's mass centre is the mass item's (the
//! #691 test, `propulsor_layout.rs`). An azimuth unit's mass centre is at the base + D (spec 0007)
//! and a shaft line's at the middle of the shaft, while the disc is on the axis at D / 2 at the
//! shaft's aft end, so the two cannot be one point.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ovcd_core::machinery::layout_rates::DISC_THICKNESS_OVER_D;
use ovcd_core::mesh::{Mesh, PartSource};
use ovcd_format::export::option_glb;
use ovcd_format::{Profile, RunOutcome, check, option_vessel_run, run_option};
use serde_json::Value;

struct Case {
    at: String,
    results: Value,
    mesh: Mesh,
}

fn design_files() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files: Vec<PathBuf> = Vec::new();
    for dir in [
        root.join("tests/fixtures"),
        root.join("../../website/assets/examples"),
        root.join("../../examples/gas-140x29-coastal-lng"),
    ] {
        for e in std::fs::read_dir(&dir).unwrap() {
            let p = e.unwrap().path();
            if p.extension().is_some_and(|x| x == "vdesign") {
                files.push(p);
            }
        }
    }
    files.sort();
    files
}

/// Every computed Option with a Propulsor Layout.
fn cases() -> Vec<Case> {
    let mut out = Vec::new();
    for path in design_files() {
        let text = std::fs::read_to_string(&path).unwrap();
        let design = check(&text, &Profile::built_in())
            .design
            .expect("a valid design");
        for option in &design.options {
            let RunOutcome::Results { results, .. } = run_option(option) else {
                continue;
            };
            if !results["drawing"]["propulsors"].is_object() {
                continue;
            }
            let run = option_vessel_run(option).expect("a vessel run");
            out.push(Case {
                at: format!(
                    "{} / {}",
                    path.file_name().unwrap().to_string_lossy(),
                    option.name
                ),
                results,
                mesh: run.meshes(),
            });
        }
    }
    assert!(out.len() >= 5, "{} self-propelled designs", out.len());
    out
}

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

fn bbox(m: &Mesh) -> [f64; 6] {
    let mut b = [
        f64::INFINITY,
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    for &i in &m.indices {
        let v = m.vertex(i);
        for k in 0..3 {
            b[k] = b[k].min(v[k]);
            b[k + 3] = b[k + 3].max(v[k]);
        }
    }
    b
}

/// The centroid of a closed, outward mesh, by the divergence theorem.
fn centroid(m: &Mesh) -> [f64; 3] {
    let (mut v, mut c) = (0.0, [0.0; 3]);
    for t in m.indices.as_chunks::<3>().0 {
        let (a, b, d) = (m.vertex(t[0]), m.vertex(t[1]), m.vertex(t[2]));
        let w = (a[0] * (b[1] * d[2] - b[2] * d[1]) - a[1] * (b[0] * d[2] - b[2] * d[0])
            + a[2] * (b[0] * d[1] - b[1] * d[0]))
            / 6.0;
        v += w;
        for k in 0..3 {
            c[k] += w * (a[k] + b[k] + d[k]) / 4.0;
        }
    }
    c.map(|x| x / v)
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

fn f(v: &Value, k: &str) -> f64 {
    v[k].as_f64()
        .unwrap_or_else(|| panic!("no number {k} in {v}"))
}

/// The box of each propulsor part, from `drawing.propulsors` alone: the results' y is + to port,
/// the mesh's + to starboard.
fn expected_boxes(results: &Value) -> Vec<(PartSource, [f64; 6])> {
    let l = &results["drawing"]["propulsors"];
    let d = f(l, "diameter_m");
    let mut out = Vec::new();
    for (i, u) in l["propulsors"].as_array().unwrap().iter().enumerate() {
        let (x, y, z) = (f(u, "x_m"), -f(u, "y_m"), f(u, "z_m"));
        let r = d / 2.0;
        let mut half_x = DISC_THICKNESS_OVER_D * d / 2.0;
        let mut half_r = r;
        if let Some(a) = u["azimuth"].as_object() {
            let a = Value::Object(a.clone());
            half_x = half_x.max(f(&a, "pod_length_m") / 2.0);
            half_r = half_r.max(f(&a, "pod_diameter_m") / 2.0);
            let (c, t) = (f(&a, "leg_chord_m") / 2.0, f(&a, "leg_thickness_m") / 2.0);
            out.push((
                PartSource::Leg(i),
                [
                    x - c,
                    y - t,
                    z + f(&a, "pod_diameter_m") / 2.0,
                    x + c,
                    y + t,
                    f(&a, "leg_top_z_m"),
                ],
            ));
        }
        out.push((
            PartSource::Propulsor(i),
            [
                x - half_x,
                y - half_r,
                z - half_r,
                x + half_x,
                y + half_r,
                z + half_r,
            ],
        ));
        if let Some(n) = u["nozzle"].as_object() {
            let n = Value::Object(n.clone());
            let (h, ro) = (f(&n, "length_m") / 2.0, f(&n, "outer_diameter_m") / 2.0);
            out.push((
                PartSource::Nozzle(i),
                [x - h, y - ro, z - ro, x + h, y + ro, z + ro],
            ));
        }
    }
    for (i, r) in l["rudders"].as_array().unwrap().iter().enumerate() {
        let (x, y, z) = (f(r, "x_m"), -f(r, "y_m"), f(r, "z_m"));
        let (c, t, s) = (
            f(r, "chord_m") / 2.0,
            f(r, "thickness_m") / 2.0,
            f(r, "span_m") / 2.0,
        );
        out.push((
            PartSource::Rudder(i),
            [x - c, y - t, z - s, x + c, y + t, z + s],
        ));
    }
    if let Some(b) = l["bow_thruster"].as_object() {
        let b = Value::Object(b.clone());
        let (x, z) = (f(&b, "x_m"), f(&b, "z_m"));
        let (r, h) = (
            f(&b, "tunnel_diameter_m") / 2.0,
            f(&b, "tunnel_length_m") / 2.0,
        );
        out.push((PartSource::BowThruster, [x - r, -h, z - r, x + r, h, z + r]));
    }
    out
}

#[test]
fn every_propulsor_part_has_its_box_from_the_results_and_a_source() {
    let mut parts = 0;
    for c in cases() {
        let expected = expected_boxes(&c.results);
        for (source, want) in &expected {
            let got = bbox(&part(&c.mesh, source));
            for k in 0..6 {
                assert!(
                    (got[k] - want[k]).abs() <= 1e-3,
                    "{}: {source:?} [{k}] mesh {} vs results {}",
                    c.at,
                    got[k],
                    want[k]
                );
            }
            parts += 1;
        }
        // Every propulsor group is a part the results account for.
        for g in c.mesh.groups.iter().filter(|g| g.layer == "propulsor") {
            assert_eq!(g.id, g.source.id(), "{}", c.at);
            assert!(
                expected.iter().any(|(s, _)| *s == g.source),
                "{}: {} has no source in the results",
                c.at,
                g.id
            );
        }
        // As many propeller discs as the layout has propulsors.
        let discs = c
            .mesh
            .groups
            .iter()
            .filter(|g| matches!(g.source, PartSource::Propulsor(_)))
            .count();
        assert_eq!(
            discs,
            c.results["drawing"]["propulsors"]["propulsors"]
                .as_array()
                .unwrap()
                .len(),
            "{}",
            c.at
        );
    }
    assert!(parts >= 20, "{parts} parts");
}

#[test]
fn each_disc_sits_at_its_layout_place_and_each_rudder_and_bow_thruster_at_its_mass_item() {
    let mut checked = 0;
    for c in cases() {
        let l = &c.results["drawing"]["propulsors"];
        let axis = c.results["powering"]["shaft_axis_height_m"].as_f64();
        for (i, u) in l["propulsors"].as_array().unwrap().iter().enumerate() {
            let got = centroid(&part(&c.mesh, &PartSource::Propulsor(i)));
            let want = [f(u, "x_m"), -f(u, "y_m"), f(u, "z_m")];
            for k in 0..3 {
                assert!(
                    (got[k] - want[k]).abs() <= 1e-3,
                    "{}: propulsor {i} [{k}] {} vs {}",
                    c.at,
                    got[k],
                    want[k]
                );
            }
            if let Some(axis) = axis {
                assert!((got[2] - axis).abs() <= 1e-3, "{}: axis", c.at);
            }
            checked += 1;
        }
        // The mass items' centres, when the stern block is weighed; else the layout's own, which
        // the #691 test pins to the mass items.
        let item = |label: &str, fallback: &Value| -> (f64, f64) {
            c.results["stern_block_mass"]["items"]
                .as_array()
                .and_then(|items| items.iter().find(|i| i["label"] == label))
                .map_or((f(fallback, "lcg_m"), f(fallback, "kcg_m")), |i| {
                    (f(i, "lcg_m"), f(i, "kcg_m"))
                })
        };
        let mut sources: Vec<(PartSource, &Value)> = l["rudders"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, r)| (PartSource::Rudder(i), r))
            .collect();
        if l["bow_thruster"].is_object() {
            sources.push((PartSource::BowThruster, &l["bow_thruster"]));
        }
        for (source, p) in sources {
            let (lcg, kcg) = item(p["mass_item"].as_str().unwrap(), &p["mass_centre"]);
            let got = centroid(&part(&c.mesh, &source));
            assert!((got[0] - lcg).abs() <= 1e-3, "{}: {source:?} x", c.at);
            assert!((got[2] - kcg).abs() <= 1e-3, "{}: {source:?} z", c.at);
            checked += 1;
        }
    }
    assert!(checked >= 15, "{checked}");
}

#[test]
fn every_propulsor_part_is_closed_and_faces_out() {
    for c in cases() {
        for g in c.mesh.groups.iter().filter(|g| g.layer == "propulsor") {
            let m = part(&c.mesh, &g.source);
            closed(&m).unwrap_or_else(|e| panic!("{}: {} not closed at {e}", c.at, g.id));
            assert!(m.volume() > 0.0, "{}: {} faces in", c.at, g.id);
        }
    }
}

fn case(name: &str) -> Case {
    cases()
        .into_iter()
        .find(|c| c.at.starts_with(name))
        .unwrap_or_else(|| panic!("no case {name}"))
}

fn ids(c: &Case) -> Vec<String> {
    c.mesh
        .groups
        .iter()
        .filter(|g| g.layer == "propulsor")
        .map(|g| g.id.clone())
        .collect()
}

#[test]
fn the_azimuth_gas_fixture_is_pinned() {
    let c = case("gas_igc_2g_self_propelled.vdesign");
    // Label: snapshot - the default stern block's three ducted azimuth thrusters: a disc, a nozzle
    // and a leg each; no rudder, no bow thruster.
    assert_eq!(
        ids(&c),
        [
            "propulsor_1",
            "nozzle_1",
            "leg_1",
            "propulsor_2",
            "nozzle_2",
            "leg_2",
            "propulsor_3",
            "nozzle_3",
            "leg_3"
        ]
    );
    let places: Vec<[f64; 3]> = (0..3)
        .map(|i| centroid(&part(&c.mesh, &PartSource::Propulsor(i))))
        .collect();
    let want = PINNED_AZIMUTH;
    for (got, want) in places.iter().zip(want) {
        for k in 0..3 {
            assert!((got[k] - want[k]).abs() <= 1e-3, "{got:?} vs {want:?}");
        }
    }
}

#[test]
fn the_tunnel_stern_shaft_fixture_is_pinned() {
    let c = case("crude_integral_tanks_self_propelled.vdesign");
    // Label: snapshot - two shaft propellers in nozzles under a tunnel stern, two rudders each,
    // and a bow thruster.
    assert_eq!(
        ids(&c),
        [
            "propulsor_1",
            "nozzle_1",
            "propulsor_2",
            "nozzle_2",
            "rudder_1",
            "rudder_2",
            "rudder_3",
            "rudder_4",
            "bow_thruster"
        ]
    );
    let places: Vec<[f64; 3]> = (0..2)
        .map(|i| centroid(&part(&c.mesh, &PartSource::Propulsor(i))))
        .collect();
    for (got, want) in places.iter().zip(PINNED_SHAFT) {
        for k in 0..3 {
            assert!((got[k] - want[k]).abs() <= 1e-3, "{got:?} vs {want:?}");
        }
    }
}

/// Label: snapshot - the disc centres (x, y to starboard, z) of the gas fixture's thrusters: at the
/// middle of its 3.5 m thruster room, 3.2 m apart (the 2.6 m envelope and 0.6 m clearance), the axis at
/// D / 2 with D = 2.6 / 1.35 m.
const PINNED_AZIMUTH: [[f64; 3]; 3] = [
    [1.75, 3.2, 1.3 / 1.35],
    [1.75, 0.0, 1.3 / 1.35],
    [1.75, -3.2, 1.3 / 1.35],
];
/// Label: snapshot - the disc centres of the crude fixture's two shaft propellers: at the aft end of
/// the shaft lines (the 2.5 m aft peak), on the tunnel axes 2.7 m off the centre line, the axis at
/// D / 2 of the Stern Form's D (2.7136 m).
const PINNED_SHAFT: [[f64; 3]; 2] = [[2.5, 2.7, 1.356_801], [2.5, -2.7, 1.356_801]];

#[test]
fn every_propulsor_part_is_a_named_gltf_node_with_the_propulsor_material() {
    let mut seen = 0;
    for path in design_files() {
        let design = check(
            &std::fs::read_to_string(&path).unwrap(),
            &Profile::built_in(),
        )
        .design
        .expect("a valid design");
        for option in &design.options {
            let Ok(run) = option_vessel_run(option) else {
                continue;
            };
            let mesh = run.meshes();
            if !mesh.groups.iter().any(|g| g.layer == "propulsor") {
                continue;
            }
            let glb = option_glb(option).unwrap();
            let json_len = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
            let doc: Value = serde_json::from_slice(&glb[20..20 + json_len]).unwrap();
            let nodes = doc["nodes"].as_array().unwrap();
            let layer = nodes
                .iter()
                .find(|n| n["name"] == "propulsors")
                .expect("a propulsors node");
            let children: Vec<&str> = layer["children"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| {
                    nodes[i.as_u64().unwrap() as usize]["name"]
                        .as_str()
                        .unwrap()
                })
                .collect();
            for g in mesh.groups.iter().filter(|g| g.layer == "propulsor") {
                assert!(children.contains(&g.id.as_str()), "{}", g.id);
                let node = nodes.iter().find(|n| n["name"] == g.id.as_str()).unwrap();
                let m = &doc["meshes"][node["mesh"].as_u64().unwrap() as usize];
                let mat = m["primitives"][0]["material"].as_u64().unwrap() as usize;
                assert_eq!(doc["materials"][mat]["name"], "propulsor", "{}", g.id);
                seen += 1;
            }
        }
    }
    assert!(seen >= 20, "{seen}");
}
