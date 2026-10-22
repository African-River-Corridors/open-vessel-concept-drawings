//! The model exports of spec 0008, "Phase 2: exports" (issue #460): the Vessel Model's meshes as a
//! glTF 2.0 binary (`.glb`). The writers are plain functions of the mesh, so the command line, the
//! web crate and the shell write the same bytes.
//!
//! glTF 2.0 (Khronos, "glTF 2.0 Specification", sections 3 and 5): a GLB is a 12 byte header, a JSON
//! chunk and a BIN chunk, each chunk a length, a type and data padded to four bytes (JSON with
//! spaces, BIN with zeros). The one buffer holds the f32 positions then the u32 indices. Each mesh
//! group is one node with one mesh of one triangle primitive; its indices accessor is the group's
//! range of the shared index view. Normals are left to the viewer (flat, as the specification says
//! for a primitive with none).

use ovcd_core::mesh::Mesh;
use serde_json::{Value, json};

/// The layers of a mesh in the order they appear in a scene, with the name of the layer's node.
const LAYERS: [(&str, &str); 5] = [
    ("hull", "hull"),
    ("deck", "decks"),
    ("bulkhead", "bulkheads"),
    ("space", "spaces"),
    ("waterline", "waterlines"),
];

/// The colour of a layer: base colour factor (RGBA) and whether it blends.
fn material(layer: &str) -> ([f64; 4], bool) {
    match layer {
        "hull" => ([0.55, 0.60, 0.66, 1.0], false),
        "deck" => ([0.78, 0.70, 0.50, 1.0], false),
        "bulkhead" => ([0.70, 0.72, 0.80, 1.0], false),
        "space" => ([0.25, 0.60, 0.90, 0.5], true),
        _ => ([0.10, 0.40, 0.90, 0.8], true),
    }
}

/// The `.glb` of `mesh`: one node for each layer, named for it (`hull`, `decks`, `bulkheads`,
/// `spaces`, `waterlines`), holding one child node for each part, named for the part (a deck or
/// bulkhead key, a Space name, a loading condition). The hull is the layer's own node, with its mesh.
/// One material for each layer. An empty mesh gives a scene with no nodes.
pub fn gltf_glb(mesh: &Mesh) -> Vec<u8> {
    let mut bin: Vec<u8> = Vec::with_capacity(4 * (mesh.positions.len() + mesh.indices.len()));
    for p in &mesh.positions {
        bin.extend_from_slice(&p.to_le_bytes());
    }
    let index_offset = bin.len();
    for i in &mesh.indices {
        bin.extend_from_slice(&i.to_le_bytes());
    }
    while !bin.len().is_multiple_of(4) {
        bin.push(0);
    }
    let vertex_count = mesh.positions.len() / 3;
    let (mut lo, mut hi) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
    for v in mesh.positions.as_chunks::<3>().0 {
        for (k, x) in v.iter().enumerate() {
            lo[k] = lo[k].min(f64::from(*x));
            hi[k] = hi[k].max(f64::from(*x));
        }
    }
    let mut accessors: Vec<Value> = Vec::new();
    let mut meshes: Vec<Value> = Vec::new();
    let mut nodes: Vec<Value> = Vec::new();
    let mut materials: Vec<Value> = Vec::new();
    let mut buffer_views: Vec<Value> = Vec::new();
    let mut scene_nodes: Vec<usize> = Vec::new();
    if vertex_count > 0 && !mesh.indices.is_empty() {
        buffer_views.push(json!({
            "buffer": 0, "byteOffset": 0, "byteLength": 12 * vertex_count, "target": 34962
        }));
        buffer_views.push(json!({
            "buffer": 0, "byteOffset": index_offset, "byteLength": 4 * mesh.indices.len(),
            "target": 34963
        }));
        accessors.push(json!({
            "bufferView": 0, "componentType": 5126, "count": vertex_count, "type": "VEC3",
            "min": lo, "max": hi
        }));
        for (layer, layer_node) in LAYERS {
            let groups: Vec<_> = mesh
                .groups
                .iter()
                .filter(|g| g.layer == layer && g.count > 0)
                .collect();
            if groups.is_empty() {
                continue;
            }
            let (colour, blend) = material(layer);
            let material_index = materials.len();
            materials.push(json!({
                "name": layer,
                "pbrMetallicRoughness": {
                    "baseColorFactor": colour, "metallicFactor": 0.0, "roughnessFactor": 0.9
                },
                "alphaMode": if blend { "BLEND" } else { "OPAQUE" },
                "doubleSided": true
            }));
            let mut primitive_node = |name: &str, start: u32, count: u32| -> usize {
                let accessor = accessors.len();
                accessors.push(json!({
                    "bufferView": 1, "byteOffset": 4 * start as usize, "componentType": 5125,
                    "count": count, "type": "SCALAR"
                }));
                let mesh_index = meshes.len();
                meshes.push(json!({
                    "name": name,
                    "primitives": [{
                        "attributes": { "POSITION": 0 }, "indices": accessor,
                        "material": material_index, "mode": 4
                    }]
                }));
                nodes.push(json!({ "name": name, "mesh": mesh_index }));
                nodes.len() - 1
            };
            if layer == "hull" {
                for g in &groups {
                    scene_nodes.push(primitive_node(layer_node, g.start, g.count));
                }
            } else {
                let children: Vec<usize> = groups
                    .iter()
                    .map(|g| primitive_node(&g.id, g.start, g.count))
                    .collect();
                nodes.push(json!({ "name": layer_node, "children": children }));
                scene_nodes.push(nodes.len() - 1);
            }
        }
    }
    let mut root = json!({
        "asset": { "version": "2.0", "generator": "OVCD" },
        "scene": 0,
        "scenes": [{ "nodes": scene_nodes }],
        "nodes": nodes,
        "buffers": [{ "byteLength": bin.len() }],
    });
    if !meshes.is_empty() {
        root["meshes"] = Value::Array(meshes);
        root["materials"] = Value::Array(materials);
        root["accessors"] = Value::Array(accessors);
        root["bufferViews"] = Value::Array(buffer_views);
    }
    let mut json_bytes = serde_json::to_vec(&root).expect("a glTF document is JSON");
    while !json_bytes.len().is_multiple_of(4) {
        json_bytes.push(b' ');
    }
    let total = 12 + 8 + json_bytes.len() + 8 + bin.len();
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(b"JSON");
    out.extend_from_slice(&json_bytes);
    out.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    out.extend_from_slice(b"BIN\0");
    out.extend_from_slice(&bin);
    out
}

/// The `.glb` of an Option's Vessel Model.
pub fn option_glb(option: &crate::DesignOption) -> Result<Vec<u8>, String> {
    crate::option_meshes(option).map(|m| gltf_glb(&m))
}

/// The SHA-256 of `bytes`, as hex: what the parity check compares between native and WebAssembly.
pub fn bytes_hash(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
