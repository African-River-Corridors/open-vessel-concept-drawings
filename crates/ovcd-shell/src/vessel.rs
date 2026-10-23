//! The Vessel Model calls of the 3D view (spec 0008, "the 3D viewer", issue #457), shared by the
//! desktop app (Tauri) and the web build (WebAssembly): the meshes, the section at an x, a Space's
//! values and the waterlines of an Option.
//!
//! No geometry here: every answer comes from `ovcd-format` and `ovcd-core`. The run of one Option
//! is kept until that Option changes, so a section plane that moves asks the core again without a
//! new run. Vertex data stays typed: `mesh_buffers` gives the f32 positions and u32 indices; the
//! JSON calls carry the group table and the values only.

use ovcd_core::mesh::Mesh;
use ovcd_format::{DesignOption, VesselRun, mesh_hash, option_vessel_run};
use serde_json::{Value, json};

/// One Option's run, its meshes and their hash.
pub(crate) struct Kept {
    option: DesignOption,
    run: VesselRun,
    mesh: Mesh,
    hash: String,
}

impl Kept {
    fn new(option: &DesignOption) -> Result<Kept, String> {
        let run = option_vessel_run(option)?;
        let mesh = run.meshes();
        let hash = mesh_hash(&mesh);
        Ok(Kept {
            option: option.clone(),
            run,
            mesh,
            hash,
        })
    }
}

/// The kept run of `option` in `design`: the one held when the Option is unchanged, else a new run.
pub(crate) fn kept<'a>(
    cache: &'a mut Option<Kept>,
    options: &[DesignOption],
    name: &str,
) -> Result<&'a Kept, String> {
    let option = options
        .iter()
        .find(|o| o.name == name)
        .ok_or_else(|| format!("no Option named '{name}'"))?;
    if cache.as_ref().is_none_or(|k| k.option != *option) {
        *cache = Some(Kept::new(option)?);
    }
    cache.as_ref().ok_or_else(|| "no Vessel Model".to_string())
}

/// The vertex data: x, y, z per vertex (m, hull axes), and three indices per triangle.
pub(crate) fn buffers(k: &Kept) -> (Vec<f32>, Vec<u32>) {
    (k.mesh.positions.clone(), k.mesh.indices.clone())
}

/// The group table and the mesh hash: `{ hash, groups: [{ layer, id, start, count }] }`, where
/// `start` and `count` are a range of the indices.
pub(crate) fn groups(k: &Kept) -> Value {
    json!({
        "hash": k.hash,
        "vertex_count": k.mesh.positions.len() / 3,
        "index_count": k.mesh.indices.len(),
        "groups": k.mesh.groups.iter().map(|g| json!({
            "layer": g.layer,
            "id": g.id,
            "start": g.start,
            "count": g.count,
        })).collect::<Vec<_>>(),
    })
}

/// The hull's section outline at `x` (m from the aft end): the polygons of the full section, (y, z)
/// in metres, and the deck edge height. At a Station it is the table's Section (the first of two at
/// a step); between Stations the ruled surface.
pub(crate) fn section(k: &Kept, x: f64) -> Value {
    let (s, depth) = k.run.model.hull.section_at(x);
    let h = &s.points;
    let n = h.len();
    let rings: Vec<Vec<[f64; 2]>> = if h[0].0 > 0.0 {
        vec![
            h.iter().map(|&(y, z)| [y, z]).collect(),
            h.iter().rev().map(|&(y, z)| [-y, z]).collect(),
        ]
    } else {
        let mut full: Vec<[f64; 2]> = h.iter().map(|&(y, z)| [y, z]).collect();
        full.extend(h[1..n - 1].iter().rev().map(|&(y, z)| [-y, z]));
        vec![full]
    };
    json!({ "x_m": s.x, "depth_at_side_m": depth, "rings": rings })
}

/// A Space's name, kind, volume (m³) and centroid (m), from the core.
pub(crate) fn space(k: &Kept, name: &str) -> Result<Value, String> {
    let s = k
        .run
        .model
        .spaces
        .iter()
        .find(|s| s.space.name == name)
        .ok_or_else(|| format!("no Space named '{name}'"))?;
    let (x, y, z) = s.centroid_m;
    Ok(json!({
        "name": s.space.name,
        "kind": s.space.kind.key(),
        "side": s.space.side.key(),
        "volume_m3": s.volume_m3,
        "lcg_m": x,
        "tcg_m": y,
        "vcg_m": z,
    }))
}

/// The waterline of each floating position: its draughts and trim, and the index of its mesh group.
pub(crate) fn waterlines(k: &Kept) -> Value {
    Value::Array(
        k.run
            .waterlines
            .iter()
            .map(|w| {
                let group = k
                    .mesh
                    .groups
                    .iter()
                    .position(|g| g.layer == "waterline" && g.id == w.name);
                json!({
                    "name": w.name,
                    "draught_m": w.draught_m,
                    "draught_aft_m": w.draught_aft_m,
                    "draught_fwd_m": w.draught_fwd_m,
                    "trim_m": w.trim_m,
                    "group": group,
                })
            })
            .collect(),
    )
}
