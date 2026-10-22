//! Seam: `ovcd_format::export::gltf_glb` and `option_glb` (issue #460, spec 0008 "Phase 2: exports").
//!
//! Label: property. For every fixture Option the `.glb` is a well-formed binary glTF 2.0 container
//! (glTF 2.0 Specification, "Binary glTF Layout": a header of magic `glTF`, version 2 and the total
//! length, then a JSON chunk and a BIN chunk, each length-prefixed and padded to four bytes); its
//! buffer holds the mesh's positions and indices byte for byte; each part is a named node whose
//! indices accessor is the part's range; node names are the layer names and the part names (the
//! Space names among them); the bytes are the same twice. The Khronos validator runs in CI
//! (`tools/gltf_validate.mjs`).

use std::path::Path;

use ovcd_core::mesh::Mesh;
use ovcd_format::export::{bytes_hash, gltf_glb, option_glb};
use ovcd_format::{Profile, check, option_meshes};
use serde_json::Value;

/// The JSON and BIN chunks of a `.glb`, after the container checks.
fn chunks(glb: &[u8]) -> (Value, Vec<u8>) {
    assert_eq!(&glb[0..4], b"glTF");
    assert_eq!(u32::from_le_bytes(glb[4..8].try_into().unwrap()), 2);
    assert_eq!(
        u32::from_le_bytes(glb[8..12].try_into().unwrap()) as usize,
        glb.len()
    );
    let mut at = 12;
    let mut out = Vec::new();
    while at < glb.len() {
        let len = u32::from_le_bytes(glb[at..at + 4].try_into().unwrap()) as usize;
        let kind = &glb[at + 4..at + 8];
        assert_eq!(len % 4, 0, "chunk of type {kind:?} is not padded");
        out.push((kind.to_vec(), glb[at + 8..at + 8 + len].to_vec()));
        at += 8 + len;
    }
    assert_eq!(at, glb.len());
    assert_eq!(out[0].0, b"JSON");
    assert_eq!(out[1].0, b"BIN\0");
    (serde_json::from_slice(&out[0].1).unwrap(), out[1].1.clone())
}

/// The names of the nodes under `node` (itself first).
fn names(doc: &Value, i: usize, out: &mut Vec<String>) {
    let n = &doc["nodes"][i];
    out.push(n["name"].as_str().unwrap().to_string());
    for c in n["children"].as_array().into_iter().flatten() {
        names(doc, c.as_u64().unwrap() as usize, out);
    }
}

fn check_glb(mesh: &Mesh, glb: &[u8]) -> Vec<String> {
    let (doc, bin) = chunks(glb);
    assert_eq!(doc["asset"]["version"], "2.0");
    assert_eq!(
        doc["buffers"][0]["byteLength"].as_u64().unwrap() as usize,
        bin.len()
    );
    // The buffer holds the positions, then the indices, as little-endian f32 and u32.
    let pos_bytes = 4 * mesh.positions.len();
    for (k, p) in mesh.positions.iter().enumerate() {
        assert_eq!(&bin[4 * k..4 * k + 4], &p.to_le_bytes());
    }
    for (k, i) in mesh.indices.iter().enumerate() {
        assert_eq!(
            &bin[pos_bytes + 4 * k..pos_bytes + 4 * k + 4],
            &i.to_le_bytes()
        );
    }
    // The position accessor: count, and the min and max over the positions.
    let pos = &doc["accessors"][0];
    assert_eq!(
        pos["count"].as_u64().unwrap() as usize,
        mesh.positions.len() / 3
    );
    for k in 0..3 {
        let col = mesh
            .positions
            .iter()
            .skip(k)
            .step_by(3)
            .map(|v| f64::from(*v));
        let (lo, hi) = col.fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), v| {
            (a.min(v), b.max(v))
        });
        assert_eq!(pos["min"][k].as_f64().unwrap(), lo);
        assert_eq!(pos["max"][k].as_f64().unwrap(), hi);
    }
    // Each mesh is one triangle primitive over its group's range of the index view.
    let mut meshed = 0;
    for m in doc["meshes"].as_array().unwrap() {
        let p = &m["primitives"][0];
        assert_eq!(p["mode"], 4);
        assert_eq!(p["attributes"]["POSITION"], 0);
        let a = &doc["accessors"][p["indices"].as_u64().unwrap() as usize];
        let (offset, count) = (
            a["byteOffset"].as_u64().unwrap() as u32 / 4,
            a["count"].as_u64().unwrap() as u32,
        );
        assert_eq!(count % 3, 0);
        let name = m["name"].as_str().unwrap();
        assert!(
            mesh.groups.iter().any(|g| g.start == offset
                && g.count == count
                && (g.id == name || g.layer == name)),
            "{name}: {offset} {count}"
        );
        meshed += 1;
    }
    assert!(meshed > 0);
    let mut all = Vec::new();
    for n in doc["scenes"][0]["nodes"].as_array().unwrap() {
        names(&doc, n.as_u64().unwrap() as usize, &mut all);
    }
    all
}

#[test]
fn every_fixture_option_writes_a_well_formed_glb_with_named_nodes() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "vdesign"))
        .collect();
    files.sort();
    let mut written = 0;
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap();
        let design = check(&text, &Profile::built_in())
            .design
            .expect("a valid fixture");
        for option in &design.options {
            let Ok(mesh) = option_meshes(option) else {
                continue;
            };
            let at = format!("{} / {}", path.display(), option.name);
            let glb = option_glb(option).unwrap();
            let nodes = check_glb(&mesh, &glb);
            // Node names: the layer of the hull, and each part and layer of the others.
            assert!(nodes.contains(&"hull".to_string()), "{at}: {nodes:?}");
            for g in &mesh.groups {
                if g.count > 0 && g.layer != "hull" {
                    assert!(nodes.contains(&g.id), "{at}: no node {}: {nodes:?}", g.id);
                }
            }
            for (layer, node) in [
                ("deck", "decks"),
                ("space", "spaces"),
                ("bulkhead", "bulkheads"),
            ] {
                let has = mesh.groups.iter().any(|g| g.layer == layer && g.count > 0);
                assert_eq!(nodes.contains(&node.to_string()), has, "{at}: {node}");
            }
            // The same bytes twice.
            assert_eq!(
                bytes_hash(&glb),
                bytes_hash(&option_glb(option).unwrap()),
                "{at}"
            );
            written += 1;
        }
    }
    assert!(written >= 8, "{written}");
}

#[test]
fn an_empty_mesh_gives_a_scene_with_no_nodes() {
    let glb = gltf_glb(&Mesh::default());
    let (doc, bin) = chunks(&glb);
    assert!(bin.is_empty());
    assert_eq!(doc["scenes"][0]["nodes"].as_array().unwrap().len(), 0);
    assert!(doc.get("meshes").is_none());
}

#[test]
fn a_group_of_no_triangles_has_no_node() {
    let mut m = Mesh::default();
    m.append(
        "hull",
        "hull",
        &Mesh {
            positions: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            indices: vec![0, 1, 2],
            groups: Vec::new(),
        },
    );
    m.append("space", "Empty", &Mesh::default());
    let (doc, _) = chunks(&gltf_glb(&m));
    let mut all = Vec::new();
    for n in doc["scenes"][0]["nodes"].as_array().unwrap() {
        names(&doc, n.as_u64().unwrap() as usize, &mut all);
    }
    assert_eq!(all, ["hull"]);
}
