//! Meshes of the Vessel Model (spec 0008, "Phase 2: meshes and the parity hash", ADR 0018, issue
//! #456): triangles for the 3D view and the exports, in hull axes (x forward from the aft end, y to
//! starboard, z up from the base line), metres.
//!
//! The work is in f64; the positions are cast to f32 once, at the end. Triangles are
//! counter-clockwise seen from outside. Normals are left to the viewer.
//!
//! - **Hull:** two triangles for each ruled quad between neighbouring Sections and neighbouring
//!   points, both sides; the slot wall of a slotted Section; at a step (two Sections at one x) the
//!   quads between the two lie in the plane of the step and close it; end caps by ear clipping.
//!   Coincident vertices are welded and a triangle with a repeated vertex is dropped, as is each
//!   pair of triangles on one sheet of no thickness (both sides of a keel line). A vertex that lies
//!   inside an edge in the plane of a Section splits the triangle there, so every edge stays in
//!   exactly two triangles. A collinear triangle with three distinct vertices is kept where it joins
//!   such a point.
//! - **Decks and waterlines:** a strip across the hull at each Station, at a height that is fixed
//!   (a deck) or follows the trim (a waterline).
//! - **Bulkheads:** the hull's section at that x, by ear clipping.
//! - **Spaces:** each Space as its closed prism.
//!
//! No result reads a mesh, so meshes are outside the results hash. The parity check hashes them
//! (`ovcd_format::mesh_hash`).

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::compartments::Space;
use crate::sections::{Section, SectionsTable};
use crate::vessel_model::{Bulkhead, Deck, VesselModel};

/// A named part of a mesh: `count` indices from `start`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshGroup {
    /// `hull`, `deck`, `bulkhead`, `space` or `waterline`.
    pub layer: String,
    /// The part's name: a deck or bulkhead key, a Space name, a loading condition name.
    pub id: String,
    pub start: u32,
    pub count: u32,
}

/// Triangles: x, y, z per vertex; three indices per triangle.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mesh {
    pub positions: Vec<f32>,
    pub indices: Vec<u32>,
    pub groups: Vec<MeshGroup>,
}

impl Mesh {
    /// Add `part` as one group.
    pub fn append(&mut self, layer: &str, id: &str, part: &Mesh) {
        let base = (self.positions.len() / 3) as u32;
        let start = self.indices.len() as u32;
        self.positions.extend_from_slice(&part.positions);
        self.indices.extend(part.indices.iter().map(|i| i + base));
        self.groups.push(MeshGroup {
            layer: layer.to_string(),
            id: id.to_string(),
            start,
            count: part.indices.len() as u32,
        });
    }

    /// The vertex `i` as f64.
    pub fn vertex(&self, i: u32) -> [f64; 3] {
        let k = 3 * i as usize;
        [
            f64::from(self.positions[k]),
            f64::from(self.positions[k + 1]),
            f64::from(self.positions[k + 2]),
        ]
    }

    /// The enclosed volume by the divergence theorem: positive for a closed mesh oriented outward.
    pub fn volume(&self) -> f64 {
        let mut v = 0.0;
        for t in self.indices.as_chunks::<3>().0 {
            let (a, b, c) = (self.vertex(t[0]), self.vertex(t[1]), self.vertex(t[2]));
            v += a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
                + a[2] * (b[0] * c[1] - b[1] * c[0]);
        }
        v / 6.0
    }
}

/// Welds vertices by their exact coordinates and keeps the triangles.
#[derive(Default)]
struct Builder {
    points: Vec<[f64; 3]>,
    index: BTreeMap<[u64; 3], u32>,
    triangles: Vec<[u32; 3]>,
}

impl Builder {
    fn vertex(&mut self, p: [f64; 3]) -> u32 {
        // -0.0 and 0.0 are one point.
        let key = p.map(|v| (v + 0.0).to_bits());
        let next = self.points.len() as u32;
        *self.index.entry(key).or_insert_with(|| {
            self.points.push(p);
            next
        })
    }

    fn triangle(&mut self, a: [f64; 3], b: [f64; 3], c: [f64; 3]) {
        let (i, j, k) = (self.vertex(a), self.vertex(b), self.vertex(c));
        if i != j && j != k && k != i {
            self.triangles.push([i, j, k]);
        }
    }

    /// Two triangles of the quad `a b c d`, counter-clockwise.
    fn quad(&mut self, a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3]) {
        self.triangle(a, b, c);
        self.triangle(a, c, d);
    }

    /// The polygon `(y, z)` at `x`, by ear clipping; `forward` faces +x, else -x. A polygon with no
    /// area gives nothing.
    fn cap(&mut self, x: f64, ring: &[(f64, f64)], forward: bool) {
        for [a, b, c] in ear_clip(ring) {
            let p = |i: usize| [x, ring[i].0, ring[i].1];
            if forward {
                self.triangle(p(a), p(b), p(c));
            } else {
                self.triangle(p(a), p(c), p(b));
            }
        }
    }

    /// Split each triangle that has, inside one of its edges in a plane x = const, a vertex of that
    /// plane. Such a T-junction comes where a flat Section (a stem, a deck line) folds back on
    /// itself, or at a step: without the split that edge would have no opposite.
    fn split_t_junctions(&mut self) {
        let mut by_x: BTreeMap<u64, Vec<u32>> = BTreeMap::new();
        for (i, p) in self.points.iter().enumerate() {
            by_x.entry((p[0] + 0.0).to_bits())
                .or_default()
                .push(i as u32);
        }
        let mut work = core::mem::take(&mut self.triangles);
        while let Some(t) = work.pop() {
            match self.junction(&t, &by_x) {
                // A flat triangle whose third corner lies inside its own edge folds back on
                // itself: its edges cancel, so it is dropped.
                Some((e, v)) if v == t[(e + 2) % 3] => {}
                Some((e, v)) => {
                    let (a, b, c) = (t[e], t[(e + 1) % 3], t[(e + 2) % 3]);
                    work.push([a, v, c]);
                    work.push([v, b, c]);
                }
                None => self.triangles.push(t),
            }
        }
    }

    /// The first edge `e` (from corner `e` to the next) of `t` in a plane x = const with a vertex
    /// `v` of that plane strictly inside it.
    fn junction(&self, t: &[u32; 3], by_x: &BTreeMap<u64, Vec<u32>>) -> Option<(usize, u32)> {
        for e in 0..3 {
            let (a, b) = (
                self.points[t[e] as usize],
                self.points[t[(e + 1) % 3] as usize],
            );
            if a[0] != b[0] {
                continue;
            }
            let (dy, dz) = (b[1] - a[1], b[2] - a[2]);
            let len2 = dy * dy + dz * dz;
            for &v in by_x.get(&(a[0] + 0.0).to_bits())? {
                let p = self.points[v as usize];
                let (py, pz) = (p[1] - a[1], p[2] - a[2]);
                let along = py * dy + pz * dz;
                let off = py * dz - pz * dy;
                if along > 0.0 && along < len2 && off.abs() <= 1e-12 * len2 {
                    return Some((e, v));
                }
            }
        }
        None
    }

    /// Drop each pair of triangles on the same three vertices in opposite directions: the two sides
    /// of a sheet of no thickness (a keel or a stem with every point on the centre line), which
    /// enclose nothing.
    fn cancel_sheets(&mut self) {
        let canonical = |t: [u32; 3]| {
            let k = (0..3).min_by_key(|&k| t[k]).unwrap_or(0);
            [t[k], t[(k + 1) % 3], t[(k + 2) % 3]]
        };
        let mut open: BTreeMap<[u32; 3], Vec<usize>> = BTreeMap::new();
        let mut keep = vec![true; self.triangles.len()];
        for (i, &t) in self.triangles.iter().enumerate() {
            let [a, b, c] = canonical(t);
            if let Some(j) = open.get_mut(&[a, c, b]).and_then(Vec::pop) {
                keep[i] = false;
                keep[j] = false;
            } else {
                open.entry([a, b, c]).or_default().push(i);
            }
        }
        let mut k = keep.iter();
        self.triangles.retain(|_| *k.next().unwrap_or(&true));
    }

    fn mesh(mut self) -> Mesh {
        self.cancel_sheets();
        self.split_t_junctions();
        self.cancel_sheets();
        Mesh {
            positions: self
                .points
                .iter()
                .flat_map(|p| p.map(|v| v as f32))
                .collect(),
            indices: self.triangles.iter().flatten().copied().collect(),
            groups: Vec::new(),
        }
    }
}

fn slotted(s: &Section) -> bool {
    s.points[0].0 > 0.0
}

/// The full section as its polygons `(y, z)`, counter-clockwise: the half and its mirror back to the
/// keel, or for a slotted Section the half and the mirrored half.
fn rings(s: &Section) -> Vec<Vec<(f64, f64)>> {
    let h = &s.points;
    let n = h.len();
    if slotted(s) {
        vec![h.clone(), h.iter().rev().map(|&(y, z)| (-y, z)).collect()]
    } else {
        let mut full = h.clone();
        full.extend(h[1..n - 1].iter().rev().map(|&(y, z)| (-y, z)));
        vec![full]
    }
}

/// The hull as a closed mesh.
pub fn hull_mesh(table: &SectionsTable) -> Mesh {
    let mut b = Builder::default();
    let ss = &table.sections;
    for pair in ss.windows(2) {
        let (s0, s1) = (&pair[0], &pair[1]);
        let n = s0.points.len();
        let at = |s: &Section, i: usize, side: f64| [s.x, side * s.points[i].0, s.points[i].1];
        for side in [1.0, -1.0] {
            let mut quad = |a: [f64; 3], b2: [f64; 3], c: [f64; 3], d: [f64; 3]| {
                if side > 0.0 {
                    b.quad(a, b2, c, d);
                } else {
                    b.quad(a, d, c, b2);
                }
            };
            for i in 0..n - 1 {
                quad(
                    at(s0, i, side),
                    at(s0, i + 1, side),
                    at(s1, i + 1, side),
                    at(s1, i, side),
                );
            }
            if slotted(s0) || slotted(s1) {
                quad(
                    at(s0, n - 1, side),
                    at(s0, 0, side),
                    at(s1, 0, side),
                    at(s1, n - 1, side),
                );
            }
        }
    }
    let (first, last) = (&ss[0], &ss[ss.len() - 1]);
    for r in rings(first) {
        b.cap(first.x, &r, false);
    }
    for r in rings(last) {
        b.cap(last.x, &r, true);
    }
    b.mesh()
}

/// The largest |y| of the hull section at `x` at height `z`; 0 where the plane misses it.
fn half_breadth(table: &SectionsTable, x: f64, z: f64) -> f64 {
    let (s, _) = table.section_at(x);
    let mut y: f64 = 0.0;
    for r in rings(&s) {
        let n = r.len();
        for i in 0..n {
            let (p, q) = (r[i], r[(i + 1) % n]);
            if (p.1 <= z) != (q.1 <= z) {
                let t = (z - p.1) / (q.1 - p.1);
                y = y.max((p.0 + t * (q.0 - p.0)).abs());
            }
        }
    }
    y
}

/// A strip across the hull from `x0` to `x1`, at the height `z(x)`, facing up: at each Station in
/// between, from the port to the starboard side at that height.
fn strip(table: &SectionsTable, x0: f64, x1: f64, z: impl Fn(f64) -> f64) -> Mesh {
    let mut xs = vec![x0];
    xs.extend(
        table
            .sections
            .iter()
            .map(|s| s.x)
            .filter(|&x| x > x0 && x < x1),
    );
    xs.push(x1);
    xs.dedup();
    let mut b = Builder::default();
    for w in xs.windows(2) {
        let (xa, xb) = (w[0], w[1]);
        let (za, zb) = (z(xa), z(xb));
        let (ya, yb) = (half_breadth(table, xa, za), half_breadth(table, xb, zb));
        b.quad([xa, -ya, za], [xa, ya, za], [xb, yb, zb], [xb, -yb, zb]);
    }
    b.mesh()
}

/// A deck, clipped by the hull.
pub fn deck_mesh(table: &SectionsTable, deck: &Deck) -> Mesh {
    strip(table, deck.x_aft_m, deck.x_fwd_m, |_| deck.z_m)
}

/// The waterline of a floating position: the draught `aft` at the aft end of the table and `fwd` at
/// its forward end, a straight line between.
pub fn waterline_mesh(table: &SectionsTable, aft: f64, fwd: f64) -> Mesh {
    let ss = &table.sections;
    let (x0, x1) = (ss[0].x, ss[ss.len() - 1].x);
    strip(table, x0, x1, |x| aft + (fwd - aft) * (x - x0) / (x1 - x0))
}

/// A bulkhead: the hull's section at its x.
pub fn bulkhead_mesh(table: &SectionsTable, bulkhead: &Bulkhead) -> Mesh {
    let (s, _) = table.section_at(bulkhead.x_m);
    let mut b = Builder::default();
    for r in rings(&s) {
        b.cap(bulkhead.x_m, &r, true);
    }
    b.mesh()
}

/// A Space as its closed prism.
pub fn space_mesh(space: &Space) -> Mesh {
    let r = &space.section;
    let n = r.len();
    let (xa, xf) = (space.x_aft, space.x_fwd);
    let mut b = Builder::default();
    for i in 0..n {
        let (p, q) = (r[i], r[(i + 1) % n]);
        b.quad(
            [xa, p.0, p.1],
            [xa, q.0, q.1],
            [xf, q.0, q.1],
            [xf, p.0, p.1],
        );
    }
    b.cap(xa, r, false);
    b.cap(xf, r, true);
    b.mesh()
}

/// Every mesh of the model in a fixed order, one group each: the hull, the decks, the bulkheads,
/// the Spaces, and the waterline of each floating position `(name, aft draught, forward draught)`.
pub fn vessel_meshes(model: &VesselModel, waterlines: &[(String, f64, f64)]) -> Mesh {
    let mut m = Mesh::default();
    m.append("hull", "hull", &hull_mesh(&model.hull));
    for d in &model.decks {
        m.append("deck", &d.name, &deck_mesh(&model.hull, d));
    }
    for b in &model.bulkheads {
        m.append("bulkhead", &b.name, &bulkhead_mesh(&model.hull, b));
    }
    for s in &model.spaces {
        m.append("space", &s.space.name, &space_mesh(&s.space));
    }
    for (name, aft, fwd) in waterlines {
        m.append("waterline", name, &waterline_mesh(&model.hull, *aft, *fwd));
    }
    m
}

/// Twice the signed area of a polygon.
fn area2(ring: &[(f64, f64)]) -> f64 {
    let n = ring.len();
    (0..n)
        .map(|i| {
            let (p, q) = (ring[i], ring[(i + 1) % n]);
            p.0 * q.1 - q.0 * p.1
        })
        .sum()
}

fn cross(o: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
}

/// Triangles of a simple counter-clockwise polygon by ear clipping, as index triples. A polygon with
/// no area gives none. A corner on a straight edge is clipped as a flat ear when no other ear is
/// left, so every edge of the polygon is in a triangle.
pub fn ear_clip(ring: &[(f64, f64)]) -> Vec<[usize; 3]> {
    let scale = ring
        .iter()
        .fold(0.0_f64, |m, p| m.max(p.0.abs()).max(p.1.abs()))
        .max(1.0);
    if area2(ring) <= 1e-12 * scale * scale {
        return Vec::new();
    }
    let mut idx: Vec<usize> = (0..ring.len()).collect();
    let mut out = Vec::new();
    while idx.len() > 3 {
        let m = idx.len();
        let ear = (0..m).find(|&k| {
            let (a, b, c) = (idx[(k + m - 1) % m], idx[k], idx[(k + 1) % m]);
            let (pa, pb, pc) = (ring[a], ring[b], ring[c]);
            cross(pa, pb, pc) > 0.0
                && idx.iter().all(|&j| {
                    let p = ring[j];
                    j == a
                        || j == b
                        || j == c
                        || p == pa
                        || p == pb
                        || p == pc
                        || !(cross(pa, pb, p) >= 0.0
                            && cross(pb, pc, p) >= 0.0
                            && cross(pc, pa, p) >= 0.0)
                })
        });
        // No proper ear: a corner on a straight edge, or a repeated point.
        let k = ear
            .or_else(|| {
                (0..m).find(|&k| {
                    let (a, b, c) = (idx[(k + m - 1) % m], idx[k], idx[(k + 1) % m]);
                    cross(ring[a], ring[b], ring[c]) >= 0.0
                })
            })
            .unwrap_or(0);
        out.push([idx[(k + m - 1) % m], idx[k], idx[(k + 1) % m]]);
        idx.remove(k);
    }
    out.push([idx[0], idx[1], idx[2]]);
    out
}
