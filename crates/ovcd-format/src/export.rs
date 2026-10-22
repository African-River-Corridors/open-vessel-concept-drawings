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

// --- The panel mesh and the offsets (issue #461, spec 0008 "Phase 2: exports") ------------------
//
// Binary STL ("StereoLithography Interface Specification", 3D Systems 1989: an 80 byte header, a
// u32 triangle count, then per triangle a normal and three vertices as little-endian f32 and a u16
// attribute) or Wavefront OBJ (text, `v` and `f` lines, one-based indices), of the hull to the deck
// in hull axes: x forward from the aft end, y to starboard, z up from the base line, metres.
// Triangles are counter-clockwise seen from outside, as in `ovcd_core::mesh`.

use std::collections::BTreeMap;

/// `mesh` with the vertices that coincide as f32 welded and the triangles that collapse dropped. The
/// f64 vertices of the model that differ by less than an f32 can hold (sections a nanometre apart)
/// are one point in a file, so an exported mesh is welded as the file holds it.
pub fn weld_f32(mesh: &Mesh) -> Mesh {
    let mut map: BTreeMap<[u32; 3], u32> = BTreeMap::new();
    let mut out = Mesh::default();
    let ids: Vec<u32> = mesh
        .positions
        .as_chunks::<3>()
        .0
        .iter()
        .map(|p| {
            let key = p.map(|v| (v + 0.0).to_bits());
            let next = (out.positions.len() / 3) as u32;
            *map.entry(key).or_insert_with(|| {
                out.positions.extend(p.iter().map(|v| v + 0.0));
                next
            })
        })
        .collect();
    for t in mesh.indices.as_chunks::<3>().0 {
        let tri = t.map(|i| ids[i as usize]);
        if tri[0] != tri[1] && tri[1] != tri[2] && tri[2] != tri[0] {
            out.indices.extend(tri);
        }
    }
    out
}

/// The hull of `mesh` (its `hull` group) as a mesh of its own, with the unused vertices dropped.
pub fn hull_part(mesh: &Mesh) -> Mesh {
    let Some(g) = mesh.groups.iter().find(|g| g.layer == "hull") else {
        return Mesh::default();
    };
    let idx = &mesh.indices[g.start as usize..(g.start + g.count) as usize];
    let mut map: BTreeMap<u32, u32> = BTreeMap::new();
    let mut out = Mesh::default();
    for &i in idx {
        let next = map.len() as u32;
        let j = *map.entry(i).or_insert_with(|| {
            let k = 3 * i as usize;
            out.positions.extend_from_slice(&mesh.positions[k..k + 3]);
            next
        });
        out.indices.push(j);
    }
    out
}

/// The part of the closed `hull` below the waterplane, with the waterplane closed by a lid. The
/// plane has the draught `aft` at x = `x_aft` and `fwd` at `x_fwd`, a straight line between (the
/// floating position of a loading condition, trimmed). A vertex on the plane is below it. The cut
/// points are shared by the two triangles of an edge, so the result is closed; the lid is the
/// outline of the cut by ear clipping, counter-clockwise seen from above. A hull wholly below the
/// plane is returned whole (with no lid); one wholly above gives an empty mesh.
pub fn clip_below(hull: &Mesh, x_aft: f64, fwd_x: f64, aft: f64, fwd: f64) -> Mesh {
    let plane = |x: f64| aft + (fwd - aft) * (x - x_aft) / (fwd_x - x_aft);
    let n = hull.positions.len() / 3;
    let height: Vec<f64> = (0..n as u32)
        .map(|i| {
            let v = vertex(hull, i);
            v[2] - plane(v[0])
        })
        .collect();
    let mut out = Mesh::default();
    let mut kept: BTreeMap<u32, u32> = BTreeMap::new();
    let mut cuts: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    let mut take = |out: &mut Mesh, i: u32| -> u32 {
        let next = (out.positions.len() / 3) as u32;
        *kept.entry(i).or_insert_with(|| {
            let k = 3 * i as usize;
            out.positions.extend_from_slice(&hull.positions[k..k + 3]);
            next
        })
    };
    // The vertex where the edge a-b crosses the plane, once for each edge.
    let mut cut = |out: &mut Mesh, a: u32, b: u32| -> u32 {
        let key = (a.min(b), a.max(b));
        if let Some(&i) = cuts.get(&key) {
            return i;
        }
        let (lo, hi) = key;
        let (pa, pb) = (vertex(hull, lo), vertex(hull, hi));
        let (ha, hb) = (height[lo as usize], height[hi as usize]);
        let t = ha / (ha - hb);
        let p = [
            pa[0] + t * (pb[0] - pa[0]),
            pa[1] + t * (pb[1] - pa[1]),
            pa[2] + t * (pb[2] - pa[2]),
        ];
        let i = (out.positions.len() / 3) as u32;
        out.positions.extend(p.map(|v| v as f32));
        cuts.insert(key, i);
        i
    };
    for t in hull.indices.as_chunks::<3>().0 {
        let inside = t.map(|i| height[i as usize] <= 0.0);
        match inside.iter().filter(|b| **b).count() {
            3 => {
                let tri = t.map(|i| take(&mut out, i));
                out.indices.extend(tri);
            }
            0 => {}
            _ => {
                // Sutherland-Hodgman against the plane: the polygon of 3 or 4 points.
                let mut poly: Vec<u32> = Vec::new();
                for k in 0..3 {
                    let (a, b) = (t[k], t[(k + 1) % 3]);
                    let (ia, ib) = (inside[k], inside[(k + 1) % 3]);
                    if ia {
                        poly.push(take(&mut out, a));
                    }
                    if ia != ib && height[a as usize] != 0.0 && height[b as usize] != 0.0 {
                        poly.push(cut(&mut out, a, b));
                    }
                }
                for k in 1..poly.len().saturating_sub(1) {
                    let tri = [poly[0], poly[k], poly[k + 1]];
                    if tri[0] != tri[1] && tri[1] != tri[2] && tri[2] != tri[0] {
                        out.indices.extend(tri);
                    }
                }
            }
        }
    }
    lid(&mut out);
    out
}

fn vertex(m: &Mesh, i: u32) -> [f64; 3] {
    let k = 3 * i as usize;
    [
        f64::from(m.positions[k]),
        f64::from(m.positions[k + 1]),
        f64::from(m.positions[k + 2]),
    ]
}

/// Close the open outline of `m` (edges in one triangle only, with no opposite) with triangles.
fn lid(m: &mut Mesh) {
    let mut edges: BTreeMap<(u32, u32), i32> = BTreeMap::new();
    for t in m.indices.as_chunks::<3>().0 {
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            *edges.entry((a, b)).or_default() += 1;
        }
    }
    // The lid runs each outline edge the other way: b to a.
    let mut next: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for (&(a, b), &c) in &edges {
        if c > 0 && !edges.contains_key(&(b, a)) {
            next.entry(b).or_default().push(a);
        }
    }
    let mut starts: Vec<u32> = next.keys().copied().collect();
    starts.sort_unstable();
    for s in starts {
        while let Some(first) = next.get_mut(&s).and_then(Vec::pop) {
            let mut ring = vec![s];
            let mut at = first;
            while at != s {
                ring.push(at);
                match next.get_mut(&at).and_then(Vec::pop) {
                    Some(n) => at = n,
                    None => break,
                }
            }
            if ring.len() < 3 {
                continue;
            }
            let pts: Vec<(f64, f64)> = ring
                .iter()
                .map(|&i| {
                    let v = vertex(m, i);
                    (v[0], v[1])
                })
                .collect();
            // In the (x, y) plane the ring is counter-clockwise, or clockwise: mirror it for the ear
            // clipper (which wants counter-clockwise) and turn the triangles back.
            let area: f64 = (0..pts.len())
                .map(|i| {
                    let (p, q) = (pts[i], pts[(i + 1) % pts.len()]);
                    p.0 * q.1 - q.0 * p.1
                })
                .sum();
            let mirrored = area < 0.0;
            let flat: Vec<(f64, f64)> = if mirrored {
                pts.iter().map(|&(x, y)| (x, -y)).collect()
            } else {
                pts
            };
            for tri in ovcd_core::mesh::ear_clip(&flat) {
                let [a, b, c] = tri.map(|k| ring[k]);
                if mirrored {
                    m.indices.extend([a, c, b]);
                } else {
                    m.indices.extend([a, b, c]);
                }
            }
        }
    }
}

/// The unit normal of a triangle, or zero for one with no area.
fn normal(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> [f32; 3] {
    let (u, v) = (
        [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
        [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
    );
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len == 0.0 {
        [0.0; 3]
    } else {
        n.map(|x| (x / len) as f32)
    }
}

/// Binary STL of `mesh`: all its triangles.
pub fn stl_binary(mesh: &Mesh) -> Vec<u8> {
    let triangles = mesh.indices.as_chunks::<3>().0;
    let mut out = Vec::with_capacity(84 + 50 * triangles.len());
    let mut header = [0u8; 80];
    let label = b"OVCD hull, metres, x forward y starboard z up";
    header[..label.len()].copy_from_slice(label);
    out.extend_from_slice(&header);
    out.extend_from_slice(&(triangles.len() as u32).to_le_bytes());
    for t in triangles {
        let (a, b, c) = (vertex(mesh, t[0]), vertex(mesh, t[1]), vertex(mesh, t[2]));
        for v in normal(a, b, c) {
            out.extend_from_slice(&v.to_le_bytes());
        }
        for i in t {
            let k = 3 * *i as usize;
            for x in &mesh.positions[k..k + 3] {
                out.extend_from_slice(&x.to_le_bytes());
            }
        }
        out.extend_from_slice(&0u16.to_le_bytes());
    }
    out
}

/// Wavefront OBJ of `mesh`: `v` lines, then `f` lines with one-based indices. No normals (a reader
/// takes them from the winding).
pub fn obj_text(mesh: &Mesh) -> String {
    use std::fmt::Write as _;
    let mut out = String::from("# OVCD hull, metres, x forward y starboard z up\n");
    for v in mesh.positions.as_chunks::<3>().0 {
        let _ = writeln!(out, "v {} {} {}", v[0], v[1], v[2]);
    }
    for t in mesh.indices.as_chunks::<3>().0 {
        let _ = writeln!(out, "f {} {} {}", t[0] + 1, t[1] + 1, t[2] + 1);
    }
    out
}

/// The closed hull mesh of an Option to the deck, or with `below` (a loading condition's name) the
/// part under that condition's waterline with the lid.
pub fn option_panel_mesh(
    option: &crate::DesignOption,
    below: Option<&str>,
) -> Result<Mesh, String> {
    let (model, waterlines) = crate::option_model_waterlines(option)?;
    let hull = weld_f32(&ovcd_core::mesh::hull_mesh(&model.hull));
    let Some(name) = below else {
        return Ok(hull);
    };
    let (_, aft, fwd) = waterlines
        .iter()
        .find(|(n, _, _)| n == name)
        .ok_or_else(|| {
            let names: Vec<&str> = waterlines.iter().map(|(n, _, _)| n.as_str()).collect();
            format!(
                "no condition named `{name}` (the Option has {})",
                names.join(", ")
            )
        })?;
    let ss = &model.hull.sections;
    Ok(clip_below(&hull, ss[0].x, ss[ss.len() - 1].x, *aft, *fwd))
}

/// The offsets CSV of an Option's hull, in the import format of #451.
pub fn option_offsets_csv(option: &crate::DesignOption) -> Result<String, String> {
    let table = crate::option_hull_table(option)?;
    Ok(crate::offsets::to_csv(&ovcd_core::offsets::to_rows(&table)))
}

/// The formats `ovcd export` writes.
pub const FORMATS: [&str; 4] = ["gltf", "stl", "obj", "offsets-csv"];

/// The bytes of an Option's export in `format` (one of [`FORMATS`]). `below` (a loading condition's
/// name) clips an `stl` or `obj` at that waterline; it is an error for the others.
pub fn option_export(
    format: &str,
    option: &crate::DesignOption,
    below: Option<&str>,
) -> Result<Vec<u8>, String> {
    if below.is_some() && !matches!(format, "stl" | "obj") {
        return Err("--below clips a mesh: use it with --format stl or obj".to_string());
    }
    match format {
        "gltf" => option_glb(option),
        "stl" => option_panel_mesh(option, below).map(|m| stl_binary(&m)),
        "obj" => option_panel_mesh(option, below).map(|m| obj_text(&m).into_bytes()),
        "offsets-csv" => option_offsets_csv(option).map(String::into_bytes),
        other => Err(format!(
            "unknown format `{other}` (use {})",
            FORMATS.join(", ")
        )),
    }
}
