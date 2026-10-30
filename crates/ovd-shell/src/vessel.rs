//! The Vessel Model calls of the 3D view (spec 0008, "the 3D viewer", issue #457), shared by the
//! desktop app (Tauri) and the web build (WebAssembly): the meshes, the section at an x, a Space's
//! values and the waterlines of an Option.
//!
//! No geometry here: every answer comes from `ovd-format` and `ovd-core`. The run of one Option
//! is kept until that Option changes, so a section plane that moves asks the core again without a
//! new run. Vertex data stays typed: `mesh_buffers` gives the f32 positions and u32 indices; the
//! JSON calls carry the group table and the values only.

use ovd_core::mesh::Mesh;
use ovd_format::{DesignOption, VesselRun, mesh_hash, option_vessel_run};
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
    let mut out = json!({
        "name": s.space.name,
        "kind": s.space.kind.key(),
        "side": s.space.side.key(),
        "volume_m3": s.volume_m3,
        "lcg_m": x,
        "tcg_m": y,
        "vcg_m": z,
    });
    // A Type C tank (ADR 0021): its Tank Layout values, for the pick panel (spec 0009).
    if let ovd_core::compartments::SpaceShape::Tank {
        axis_x0,
        axis_x1,
        outer_diameter,
        head,
        ..
    } = &s.space.shape
    {
        out["tank"] = json!({
            "outer_diameter_m": outer_diameter,
            "length_m": axis_x1 - axis_x0,
            "head_type": head.kind.key(),
            "head_length_m": head.length_m,
        });
    }
    Ok(out)
}

/// The Deck Items of the Outfit (spec 0009): each one's mesh group id, kind, label and box (m, y to
/// starboard), and the wheelhouse's lift and the light air draft, all from the core.
pub(crate) fn deck_items(k: &Kept) -> Value {
    use ovd_core::mesh::PartSource;
    use ovd_core::vessel_model::{DeckItemKind, WheelhousePosition};
    let outfit = &k.run.model.outfit;
    let items: Vec<Value> = outfit
        .deck_items
        .iter()
        .map(|d| {
            let kind = match d.kind {
                DeckItemKind::Unit { battery: true, .. } => "battery_unit",
                DeckItemKind::Unit { battery: false, .. } => "accommodation_unit",
                DeckItemKind::Wheelhouse(WheelhousePosition::Lowered) => "wheelhouse_lowered",
                DeckItemKind::Wheelhouse(WheelhousePosition::Raised) => "wheelhouse_raised",
                DeckItemKind::LiftColumn => "lift_column",
            };
            json!({
                "id": PartSource::of_deck_item(d).id(),
                "kind": kind,
                "label": d.label,
                "x_aft_m": d.x_aft_m,
                "x_fwd_m": d.x_fwd_m,
                "y_port_m": d.y_port_m,
                "y_stbd_m": d.y_stbd_m,
                "z_base_m": d.z_base_m,
                "z_top_m": d.z_top_m,
            })
        })
        .collect();
    let lift = outfit.wheelhouse_lift.map(|w| {
        json!({
            "needed_m": w.needed_m,
            "used_m": w.used_m,
            "allowed_m": w.allowed_m,
            "air_draft_light_m": w.air_draft_light_m,
            "air_draft_navigating_light_m": w.air_draft_navigating_light_m,
        })
    });
    json!({ "items": items, "wheelhouse": lift })
}

/// The propulsor parts of the Outfit (spec 0009): each mesh group's id, its part, the propulsor it
/// belongs to, and the values the pick panel shows (type, diameter, axis height, power), with the
/// part's centre in the mesh's axes (y + to starboard) and its mass item, all from the core's
/// Propulsor Layout. An empty list without a stern block.
pub(crate) fn propulsors(k: &Kept) -> Value {
    use ovd_core::mesh::PartSource;
    let Some(l) = &k.run.model.outfit.propulsors else {
        return json!([]);
    };
    let common = json!({
        "propulsor_type": l.propulsor_type,
        "diameter_m": l.diameter_m,
        "axis_z_m": l.axis_z_m,
        "unit_power_kw": l.unit_power_kw,
    });
    let part = |source: PartSource,
                part: &str,
                index: Option<usize>,
                at: [f64; 3],
                mass: (&str, f64, f64)| {
        let mut v = common.clone();
        v["id"] = json!(source.id());
        v["part"] = json!(part);
        v["index"] = json!(index);
        v["x_m"] = json!(at[0]);
        v["y_m"] = json!(at[1]);
        v["z_m"] = json!(at[2]);
        v["mass_item"] = json!(mass.0);
        v["lcg_m"] = json!(mass.1);
        v["kcg_m"] = json!(mass.2);
        v
    };
    let mut out = Vec::new();
    for (i, u) in l.propulsors.iter().enumerate() {
        let at = [u.x_m, -u.y_m, u.z_m];
        let mass = (u.mass_item, u.lcg_m, u.kcg_m);
        out.push(part(
            PartSource::Propulsor(i),
            "propulsor",
            Some(i),
            at,
            mass,
        ));
        if u.nozzle.is_some() {
            out.push(part(PartSource::Nozzle(i), "nozzle", Some(i), at, mass));
        }
        if u.azimuth.is_some() {
            out.push(part(PartSource::Leg(i), "leg", Some(i), at, mass));
        }
    }
    for (i, r) in l.rudders.iter().enumerate() {
        let mut v = part(
            PartSource::Rudder(i),
            "rudder",
            None,
            [r.x_m, -r.y_m, r.z_m],
            ("rudders and steering gear", r.lcg_m, r.kcg_m),
        );
        v["span_m"] = json!(r.span_m);
        v["chord_m"] = json!(r.chord_m);
        out.push(v);
    }
    if let Some(b) = &l.bow_thruster {
        let mut v = part(
            PartSource::BowThruster,
            "bow_thruster",
            None,
            [b.x_m, 0.0, b.z_m],
            ("bow thruster", b.lcg_m, b.kcg_m),
        );
        v["unit_power_kw"] = json!(b.power_kw);
        v["tunnel_diameter_m"] = json!(b.tunnel_diameter_m);
        out.push(v);
    }
    Value::Array(out)
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
