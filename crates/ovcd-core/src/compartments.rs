//! Compartments: the typed Spaces of a hull (issue #426, ADR 0016).
//!
//! One compartment model for every liquid a hull holds: water ballast now, the crude oil cargo tanks
//! of the `liquid_bulk` module later (#330). A [`Space`] is a prism: a cross-section polygon in the
//! (y, z) plane, swept from `x_aft` to `x_fwd`. x is from the aft end of the hull, y from the centre
//! line (positive to starboard), z from the keel.
//!
//! A fill gives the liquid's volume, level, centroid and the transverse inertia of its free surface,
//! by clipping the section at the level. The volumes are moulded: no deduction for structure.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::barge_capacity::{BargeCapacityInputs, BargeCapacityResult};
use crate::box_stability::{area_centroid, clip_below};
use crate::cargo::CargoClass;
use crate::sections::SectionsTable;

/// What a Space holds. The liquid bulk module adds the cargo kinds (#330).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceKind {
    /// A double-bottom water ballast tank.
    DoubleBottomBallast,
    /// A wing (side) water ballast tank.
    WingBallast,
    /// A peak (end) water ballast tank across the full beam: the fore peak, or the aft peak of a
    /// pushed barge.
    PeakBallast,
    /// A tank of stores and fuel (consumable liquids).
    Consumables,
    /// A cargo tank (the liquid bulk module).
    CargoTank,
    /// A slop tank: tank washings and oily residues, not cargo.
    Slop,
    /// A cofferdam: an empty space that separates the cargo tanks from the rest of the hull.
    Cofferdam,
    /// A pump room: the cargo pumps below deck.
    PumpRoom,
    /// A dry cargo hold (issue #454): the hold of a dry bulk hull, or the hold that houses the Type C
    /// tanks of a gas hull.
    CargoHold,
    /// An empty space with no other use: the service space of a gas hull's aft separation.
    Void,
    /// A machinery space below deck: a thruster, drive, engine, genset, gas valve unit or battery
    /// room.
    Machinery,
    /// The empty aft peak of a self-propelled hull, with the steering gear.
    PeakVoid,
}

impl SpaceKind {
    pub fn key(self) -> &'static str {
        match self {
            SpaceKind::DoubleBottomBallast => "double_bottom_ballast",
            SpaceKind::WingBallast => "wing_ballast",
            SpaceKind::PeakBallast => "peak_ballast",
            SpaceKind::Consumables => "consumables",
            SpaceKind::CargoTank => "cargo_tank",
            SpaceKind::Slop => "slop",
            SpaceKind::Cofferdam => "cofferdam",
            SpaceKind::PumpRoom => "pump_room",
            SpaceKind::CargoHold => "cargo_hold",
            SpaceKind::Void => "void",
            SpaceKind::Machinery => "machinery",
            SpaceKind::PeakVoid => "peak_void",
        }
    }

    pub fn is_ballast(self) -> bool {
        matches!(
            self,
            SpaceKind::DoubleBottomBallast | SpaceKind::WingBallast | SpaceKind::PeakBallast
        )
    }
}

/// The side of the centre line a Space lies on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Port,
    Starboard,
    /// Across the centre line.
    Centre,
}

impl Side {
    pub fn key(self) -> &'static str {
        match self {
            Side::Port => "port",
            Side::Starboard => "starboard",
            Side::Centre => "centre",
        }
    }

    pub(crate) fn sign(self) -> f64 {
        match self {
            Side::Port => -1.0,
            Side::Starboard | Side::Centre => 1.0,
        }
    }

    pub(crate) fn letter(self) -> &'static str {
        match self {
            Side::Port => "P",
            Side::Starboard => "S",
            Side::Centre => "C",
        }
    }
}

/// One compartment: a prism of `section` from `x_aft` to `x_fwd`.
#[derive(Debug, Clone, PartialEq)]
pub struct Space {
    /// A short name, the same in every language: "DB1P", "WB1S".
    pub name: String,
    pub kind: SpaceKind,
    pub side: Side,
    pub x_aft: f64,
    pub x_fwd: f64,
    /// The cross-section, (y, z) corners in order. A simple polygon.
    pub section: Vec<(f64, f64)>,
}

/// A Space with its volume and centroid inside the hull (spec 0008, "One compartment model", issue
/// #455): the prism less what lies outside the hull. A Space wholly inside the hull keeps its prism
/// values exactly, and its fills are the prism's.
#[derive(Debug, Clone, PartialEq)]
pub struct ClippedSpace {
    pub space: Space,
    pub volume_m3: f64,
    /// The centroid (x, y, z), m in hull axes.
    pub centroid_m: (f64, f64, f64),
    /// The hull intervals that cut the Space; empty when it lies wholly inside.
    cut: Vec<HullPiece>,
}

/// One interval of the hull over the Space's length: the half-sections at its two ends, with the
/// part of the Space's length it covers.
#[derive(Debug, Clone, PartialEq)]
struct HullPiece {
    x0: f64,
    x1: f64,
    a: Vec<(f64, f64)>,
    b: Vec<(f64, f64)>,
    slot: bool,
    /// The part of the Space's length in this interval.
    from: f64,
    to: f64,
}

impl ClippedSpace {
    /// The Space as its prism: the section's area times the length, at the section's centroid and
    /// mid-length. For a Space that no hull cuts.
    pub fn prism(space: Space) -> Self {
        let (area, cy, cz) = area_centroid(&space.section);
        Self {
            volume_m3: area * space.length(),
            centroid_m: ((space.x_aft + space.x_fwd) / 2.0, cy, cz),
            space,
            cut: Vec::new(),
        }
    }

    /// Whether the hull cuts the Space.
    pub fn is_cut(&self) -> bool {
        !self.cut.is_empty()
    }

    /// The mass pressed full of a liquid of `density` (t/m³).
    pub fn capacity_t(&self, density: f64) -> f64 {
        self.volume_m3 * density
    }

    /// The liquid at `fraction` of the clipped volume (clamped to 0..1): the level by 100
    /// bisections, as [`Space::fill`]; the centroid and the free surface from the clipped shape.
    pub fn fill(&self, fraction: f64) -> Fill {
        if self.cut.is_empty() {
            return self.space.fill(fraction);
        }
        let fraction = fraction.clamp(0.0, 1.0);
        let (z_lo, z_hi) = self.space.z_range();
        if fraction <= 0.0 || self.volume_m3 <= 0.0 {
            return Fill {
                fraction: 0.0,
                volume_m3: 0.0,
                level_m: z_lo,
                lcg_m: self.centroid_m.0,
                tcg_m: 0.0,
                vcg_m: z_lo,
                free_surface_inertia_m4: 0.0,
            };
        }
        let level = if fraction >= 1.0 {
            z_hi
        } else {
            let target = fraction * self.volume_m3;
            let (mut lo, mut hi) = (z_lo, z_hi);
            for _ in 0..100 {
                let mid = 0.5 * (lo + hi);
                if self.below(mid, false).volume < target {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            0.5 * (lo + hi)
        };
        let m = self.below(level, fraction < 1.0);
        let v = m.volume;
        let (x, y, z) = if v > 0.0 {
            (m.mx / v, m.my / v, m.mz / v)
        } else {
            (self.centroid_m.0, 0.0, z_lo)
        };
        Fill {
            fraction,
            volume_m3: v,
            level_m: level,
            lcg_m: x,
            tcg_m: y,
            vcg_m: z,
            free_surface_inertia_m4: m.inertia,
        }
    }

    /// The clipped shape below `level`, and with `surface` the free surface inertia there.
    fn below(&self, level: f64, surface: bool) -> Moments {
        let clipper = clip_below(&self.space.section, level);
        let mut m = Moments::default();
        for piece in &self.cut {
            let fs = surface.then_some((level, self.space.section.as_slice()));
            m.add(&integrate(piece, &clipper, fs).moments);
        }
        m
    }
}

/// The volume, its first moments and the free surface inertia of a clipped shape.
#[derive(Debug, Clone, Copy, Default)]
struct Moments {
    volume: f64,
    mx: f64,
    my: f64,
    mz: f64,
    inertia: f64,
}

impl Moments {
    fn add(&mut self, o: &Moments) {
        self.volume += o.volume;
        self.mx += o.mx;
        self.my += o.my;
        self.mz += o.mz;
        self.inertia += o.inertia;
    }
}

/// Clip a Space by the hull (spec 0008, "One compartment model"): at each x, each polygon of the
/// hull's full section is clipped by the Space's convex section (Sutherland–Hodgman, one half
/// plane per edge of the Space), and the areas and moments are integrated along x by Simpson's rule.
/// Each hull interval is first split where a hull point line crosses the line of a Space edge and
/// where a Space corner crosses a hull edge, so the clipped shape keeps one topology on each part.
/// That is exact wherever the clipped area is at most cubic in x: a prism, and a box or raked box
/// hull. Where no part of the Space lies outside the hull, the prism's values are kept exactly.
pub fn clip(space: &Space, table: &SectionsTable) -> ClippedSpace {
    let mut pieces = Vec::new();
    let mut covered = 0.0;
    for pair in table.sections.windows(2) {
        let (s0, s1) = (&pair[0], &pair[1]);
        if s1.x <= s0.x {
            continue;
        }
        let (from, to) = (space.x_aft.max(s0.x), space.x_fwd.min(s1.x));
        if to <= from {
            continue;
        }
        covered += to - from;
        pieces.push(HullPiece {
            x0: s0.x,
            x1: s1.x,
            a: s0.points.clone(),
            b: s1.points.clone(),
            slot: s0.points[0].0 > 0.0,
            from,
            to,
        });
    }
    let prism = ClippedSpace::prism(space.clone());
    let mut m = Moments::default();
    let length = space.length();
    let mut inside = (covered - length).abs() <= 1e-12 * length;
    for piece in &pieces {
        let part = integrate(piece, &space.section, None);
        inside &= part.full_at_every_node;
        m.add(&part.moments);
    }
    if inside {
        return prism;
    }
    let v = m.volume;
    ClippedSpace {
        volume_m3: v,
        centroid_m: if v > 0.0 {
            (m.mx / v, m.my / v, m.mz / v)
        } else {
            prism.centroid_m
        },
        space: space.clone(),
        cut: pieces,
    }
}

/// What one hull interval gives, and whether the Space's whole section was inside at every node.
struct Part {
    moments: Moments,
    full_at_every_node: bool,
}

/// The hull's full section at `x` in one interval: one polygon, or two for a slotted section.
fn rings(p: &HullPiece, x: f64) -> Vec<Vec<(f64, f64)>> {
    let t = (x - p.x0) / (p.x1 - p.x0);
    let half: Vec<(f64, f64)> =
        p.a.iter()
            .zip(&p.b)
            .map(|(u, v)| ((1.0 - t) * u.0 + t * v.0, (1.0 - t) * u.1 + t * v.1))
            .collect();
    let n = half.len();
    if p.slot {
        let port: Vec<(f64, f64)> = half.iter().rev().map(|&(y, z)| (-y, z)).collect();
        vec![half, port]
    } else {
        let mut full = half.clone();
        full.extend(half[1..n - 1].iter().rev().map(|&(y, z)| (-y, z)));
        vec![full]
    }
}

/// The x in `[from, to]` of one hull interval where the clipped shape may change its topology: its
/// ends, where a hull point line crosses the line of a clipper edge (once, as both are straight),
/// and where a clipper corner crosses a hull edge (a quadratic in x).
fn splits(p: &HullPiece, clipper: &[(f64, f64)]) -> Vec<f64> {
    let (r0, r1) = (rings(p, p.x0), rings(p, p.x1));
    let h = p.x1 - p.x0;
    let mut xs = vec![p.from, p.to];
    let mut push = |t: f64| {
        let x = p.x0 + t * h;
        if x > p.from && x < p.to {
            xs.push(x);
        }
    };
    let k = clipper.len();
    for (a0, a1) in r0.iter().zip(&r1) {
        let n = a0.len();
        for j in 0..n {
            let (u0, u1) = (a0[j], a1[j]);
            // A hull point line against each clipper edge line: linear in t.
            for e in 0..k {
                let (a, b) = (clipper[e], clipper[(e + 1) % k]);
                let side = |q: (f64, f64)| (b.0 - a.0) * (q.1 - a.1) - (b.1 - a.1) * (q.0 - a.0);
                let (f0, f1) = (side(u0), side(u1));
                if (f0 < 0.0) != (f1 < 0.0) && f0 != f1 {
                    push(f0 / (f0 - f1));
                }
            }
            // Each clipper corner against the hull edge j -> j + 1: quadratic in t.
            let (w0, w1) = (a0[(j + 1) % n], a1[(j + 1) % n]);
            for &c in clipper {
                let f = |t: f64| {
                    let u = (u0.0 + t * (u1.0 - u0.0), u0.1 + t * (u1.1 - u0.1));
                    let w = (w0.0 + t * (w1.0 - w0.0), w0.1 + t * (w1.1 - w0.1));
                    (w.0 - u.0) * (c.1 - u.1) - (w.1 - u.1) * (c.0 - u.0)
                };
                let (f0, fm, f1) = (f(0.0), f(0.5), f(1.0));
                // f(t) = c0 + c1 t + c2 t², from three values.
                let c2 = 2.0 * (f0 - 2.0 * fm + f1);
                let c1 = f1 - f0 - c2;
                for t in quadratic_roots(c2, c1, f0) {
                    if t > 0.0 && t < 1.0 {
                        push(t);
                    }
                }
            }
        }
    }
    xs.sort_by(f64::total_cmp);
    xs.dedup();
    xs
}

/// The real roots of `a t² + b t + c`; a linear or constant one when `a` is (near) zero.
fn quadratic_roots(a: f64, b: f64, c: f64) -> Vec<f64> {
    let scale = a.abs().max(b.abs()).max(c.abs());
    if scale == 0.0 {
        return Vec::new();
    }
    if a.abs() <= 1e-12 * scale {
        return if b != 0.0 { vec![-c / b] } else { Vec::new() };
    }
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return Vec::new();
    }
    let q = -0.5 * (b + libm::copysign(libm::sqrt(disc), b));
    let mut out = vec![q / a];
    if q != 0.0 {
        out.push(c / q);
    }
    out
}

/// Clip `subject` by the convex polygon `clipper` (counter-clockwise), one half plane per edge.
fn clip_convex(subject: &[(f64, f64)], clipper: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut out = subject.to_vec();
    let k = clipper.len();
    for i in 0..k {
        if out.is_empty() {
            break;
        }
        let (a, b) = (clipper[i], clipper[(i + 1) % k]);
        let side = |p: (f64, f64)| (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0);
        let input = core::mem::take(&mut out);
        let n = input.len();
        for j in 0..n {
            let (p, q) = (input[j], input[(j + 1) % n]);
            let (sp, sq) = (side(p), side(q));
            if sp >= 0.0 {
                out.push(p);
            }
            if (sp >= 0.0) != (sq >= 0.0) {
                let t = sp / (sp - sq);
                out.push((p.0 + t * (q.0 - p.0), p.1 + t * (q.1 - p.1)));
            }
        }
    }
    out
}

/// Area and first moments `(A, ∫y dA, ∫z dA)` of a polygon by the shoelace formula.
fn shoelace(poly: &[(f64, f64)]) -> (f64, f64, f64) {
    let n = poly.len();
    let (mut a, mut my, mut mz) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let ((y0, z0), (y1, z1)) = (poly[i], poly[(i + 1) % n]);
        let c = y0 * z1 - y1 * z0;
        a += c;
        my += (y0 + y1) * c;
        mz += (z0 + z1) * c;
    }
    (a / 2.0, my / 6.0, mz / 6.0)
}

/// The free surface inertia per metre at `level`: the chords of the hull's section, with `below`
/// saying which corners are under the level, each cut to the Space's chord at that level, about
/// their joint centre line.
fn surface_inertia(
    hull: &[Vec<(f64, f64)>],
    below: &[Vec<bool>],
    level: f64,
    section: &[(f64, f64)],
) -> f64 {
    let crossings = |poly: &[(f64, f64)], wet: &dyn Fn(usize) -> bool| -> Vec<f64> {
        let n = poly.len();
        let mut ys = Vec::new();
        for i in 0..n {
            let (p, q) = (poly[i], poly[(i + 1) % n]);
            if wet(i) != wet((i + 1) % n) {
                let t = if q.1 == p.1 {
                    0.5
                } else {
                    ((level - p.1) / (q.1 - p.1)).clamp(0.0, 1.0)
                };
                ys.push(p.0 + t * (q.0 - p.0));
            }
        }
        ys.sort_by(f64::total_cmp);
        ys
    };
    let space = crossings(section, &|i| section[i].1 <= level);
    let (Some(&lo), Some(&hi)) = (space.first(), space.last()) else {
        return 0.0;
    };
    let mut ys: Vec<f64> = Vec::new();
    for (poly, wet) in hull.iter().zip(below) {
        for pair in crossings(poly, &|i| wet[i]).as_chunks::<2>().0 {
            let (a, b) = (pair[0].max(lo), pair[1].min(hi));
            if b > a {
                ys.extend([a, b]);
            }
        }
    }
    chords_inertia(&mut ys)
}

/// Integrate one hull interval: the clipped area and moments by Simpson's rule on each part between
/// the split points, and with `surface` (the level and the Space's whole section) the free surface
/// inertia at that level.
fn integrate(p: &HullPiece, clipper: &[(f64, f64)], surface: Option<(f64, &[(f64, f64)])>) -> Part {
    let full = shoelace(clipper).0;
    let xs = splits(p, clipper);
    let mut m = Moments::default();
    let mut whole = true;
    for w in xs.windows(2) {
        let (u, v) = (w[0], w[1]);
        let mid = 0.5 * (u + v);
        let h = (v - u) / 6.0;
        // Which hull corners are below the level: fixed on the part, so its ends take the limit
        // from inside it, as the hull integrator does (`sections`).
        let below: Option<Vec<Vec<bool>>> = surface.map(|(level, _)| {
            rings(p, mid)
                .iter()
                .map(|r| r.iter().map(|q| q.1 < level).collect())
                .collect()
        });
        for (x, wt) in [(u, h), (mid, 4.0 * h), (v, h)] {
            let hull = rings(p, x);
            let mut area = 0.0;
            for r in &hull {
                let (a, my, mz) = shoelace(&clip_convex(r, clipper));
                area += a;
                m.volume += wt * a;
                m.mx += wt * x * a;
                m.my += wt * my;
                m.mz += wt * mz;
            }
            whole &= (area - full).abs() <= 1e-12 * full.abs().max(1.0);
            if let (Some((level, section)), Some(below)) = (surface, below.as_ref()) {
                m.inertia += wt * surface_inertia(&hull, below, level, section);
            }
        }
    }
    Part {
        moments: m,
        full_at_every_node: whole,
    }
}
/// The liquid in a Space at one fill.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fill {
    /// The fraction of the Space's volume that is full, 0 to 1.
    pub fraction: f64,
    pub volume_m3: f64,
    /// The height of the liquid surface above the keel (m).
    pub level_m: f64,
    pub lcg_m: f64,
    pub tcg_m: f64,
    pub vcg_m: f64,
    /// The transverse moment of inertia of the free surface about its own centre line (m⁴). Zero
    /// for an empty or a pressed-full Space: there is no free surface.
    pub free_surface_inertia_m4: f64,
}

impl Space {
    pub fn length(&self) -> f64 {
        self.x_fwd - self.x_aft
    }

    pub fn volume_m3(&self) -> f64 {
        area_centroid(&self.section).0 * self.length()
    }

    /// The mass of the Space pressed full of a liquid of `density` (t/m³).
    pub fn capacity_t(&self, density: f64) -> f64 {
        self.volume_m3() * density
    }

    /// The lowest and highest z of the section.
    pub fn z_range(&self) -> (f64, f64) {
        self.section
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &(_, z)| {
                (lo.min(z), hi.max(z))
            })
    }

    /// The liquid at `fraction` of the volume (clamped to 0..1).
    pub fn fill(&self, fraction: f64) -> Fill {
        let fraction = fraction.clamp(0.0, 1.0);
        let (z_lo, z_hi) = self.z_range();
        let full = area_centroid(&self.section).0;
        let lcg = 0.5 * (self.x_aft + self.x_fwd);
        if fraction <= 0.0 || full <= 0.0 {
            return Fill {
                fraction: 0.0,
                volume_m3: 0.0,
                level_m: z_lo,
                lcg_m: lcg,
                tcg_m: 0.0,
                vcg_m: z_lo,
                free_surface_inertia_m4: 0.0,
            };
        }
        let level = if fraction >= 1.0 {
            z_hi
        } else {
            // The wetted area rises with the level: bisect.
            let target = fraction * full;
            let (mut lo, mut hi) = (z_lo, z_hi);
            for _ in 0..100 {
                let mid = 0.5 * (lo + hi);
                if area_centroid(&clip_below(&self.section, mid)).0 < target {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            0.5 * (lo + hi)
        };
        let wet = clip_below(&self.section, level);
        let (area, y, z) = area_centroid(&wet);
        let inertia = if fraction >= 1.0 {
            0.0
        } else {
            self.length() * chord_inertia(&self.section, level)
        };
        Fill {
            fraction,
            volume_m3: area * self.length(),
            level_m: level,
            lcg_m: lcg,
            tcg_m: y,
            vcg_m: z,
            free_surface_inertia_m4: inertia,
        }
    }
}

/// The second moment, per metre of length, of the chords of `poly` at `level`, about their joint
/// centre line: ∫ (y − ȳ)² dy over the free surface.
fn chord_inertia(poly: &[(f64, f64)], level: f64) -> f64 {
    let mut ys: Vec<f64> = Vec::new();
    for i in 0..poly.len() {
        let p = poly[i];
        let q = poly[(i + 1) % poly.len()];
        if (p.1 <= level) != (q.1 <= level) {
            let t = (level - p.1) / (q.1 - p.1);
            ys.push(p.0 + t * (q.0 - p.0));
        }
    }
    chords_inertia(&mut ys)
}

/// The second moment of the chords between the sorted crossings `ys` (in pairs) about their joint
/// centre line.
fn chords_inertia(ys: &mut [f64]) -> f64 {
    ys.sort_by(f64::total_cmp);
    let chords: Vec<(f64, f64)> = ys.as_chunks::<2>().0.iter().map(|&[a, b]| (a, b)).collect();
    let breadth: f64 = chords.iter().map(|(a, b)| b - a).sum();
    if breadth <= 0.0 {
        return 0.0;
    }
    let ybar = chords
        .iter()
        .map(|(a, b)| (b - a) * 0.5 * (a + b))
        .sum::<f64>()
        / breadth;
    chords
        .iter()
        .map(|(a, b)| {
            let (u, v) = (a - ybar, b - ybar);
            (v * v * v - u * u * u) / 3.0
        })
        .sum()
}

/// A rectangle section from `y0` to `y1` (|y|, outboard positive) and `z0` to `z1`, on `side`.
pub(crate) fn rect(side: Side, y0: f64, y1: f64, z0: f64, z1: f64) -> Vec<(f64, f64)> {
    poly(side, &[(y0, z0), (y1, z0), (y1, z1), (y0, z1)])
}

/// Mirror |y| corners to `side`, keeping the corners counter-clockwise in the (y, z) plane.
pub(crate) fn poly(side: Side, corners: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut out: Vec<(f64, f64)> = corners.iter().map(|&(y, z)| (side.sign() * y, z)).collect();
    if side == Side::Port {
        out.reverse();
    }
    out
}

/// A pair of Spaces, port and starboard, named `<prefix>1P` and `<prefix>1S`.
pub(crate) fn pair(
    prefix: &str,
    kind: SpaceKind,
    (x_aft, x_fwd): (f64, f64),
    section: impl Fn(Side) -> Vec<(f64, f64)>,
) -> Vec<Space> {
    [Side::Port, Side::Starboard]
        .into_iter()
        .map(|side| Space {
            name: alloc::format!("{prefix}1{}", side.letter()),
            kind,
            side,
            x_aft,
            x_fwd,
            section: section(side),
        })
        .filter(|s| s.volume_m3() > 0.0)
        .collect()
}

/// The water ballast tanks of a barge-capacity result: a double-bottom tank and a wing tank on each
/// side, the length of the cargo zone, and the peaks. One tank per side and kind: the engine has no
/// ballast bulkheads along the hull yet. A tank of no volume is left out.
///
/// - Type C tanks: the double bottom (its height), and the wing tank (its width) from the double
///   bottom up to the wing plate, as the weights model has them.
/// - Holds: the double bottom, and the wing space outboard of the hold side up to the deck. For a
///   sloped hopper hold the wing space takes in the hopper.
/// - The fore peak: the bow void, full beam and depth, to the bow. The rake cuts it: [`clip`] gives
///   its volume inside the hull (issue #455).
/// - Liquid bulk: the double bottom and the wing tanks of the tank block (`liquid_bulk`).
/// - The aft peak, for a pushed barge only: the stern void. A self-propelled hull has its machinery
///   there.
pub fn ballast_tanks(inp: &BargeCapacityInputs, r: &BargeCapacityResult) -> Vec<Space> {
    let half_b = inp.beam / 2.0;
    let stern = r.stern_void_length_m.unwrap_or(0.0);
    let bow = r.bow_void_length_m.unwrap_or(0.0);
    let zone = (stern, (inp.loa - bow).max(stern));
    let mut out = Vec::new();
    match r.phase {
        CargoClass::Gas => {
            let Some(g) = r.gas_basis.as_ref() else {
                return out;
            };
            let db = g.inputs.double_bottom_height.unwrap_or(0.0);
            let w = g.inputs.wing_tank_width.unwrap_or(0.0).clamp(0.0, half_b);
            let top = db
                + g.inputs.tank_support_height
                + g.inputs.wing_tank_height_ratio * g.tanks.outer_diameter;
            out.extend(pair("DB", SpaceKind::DoubleBottomBallast, zone, |s| {
                rect(s, 0.0, half_b, 0.0, db)
            }));
            out.extend(pair("WB", SpaceKind::WingBallast, zone, |s| {
                rect(s, half_b - w, half_b, db, top.max(db))
            }));
        }
        CargoClass::DryBulk => {
            let db = inp.double_bottom_height_bulk.clamp(0.0, inp.depth);
            let d = inp.depth;
            let w_top = r
                .hold_top_width_m
                .unwrap_or(inp.beam - 2.0 * inp.wing_void_width);
            let w_bot = r.hold_bottom_width_m.unwrap_or(w_top);
            let h = r.hold_height_m.unwrap_or(d - db);
            // The hold side runs from half the bottom width at the inner bottom to half the top
            // width at the hold top; at the deck it is part way.
            let at_deck = if h > 0.0 {
                w_bot / 2.0 + (w_top - w_bot) / 2.0 * ((d - db) / h)
            } else {
                w_top / 2.0
            };
            let (y_bot, y_deck) = ((w_bot / 2.0).min(half_b), at_deck.min(half_b));
            out.extend(pair("DB", SpaceKind::DoubleBottomBallast, zone, |s| {
                rect(s, 0.0, half_b, 0.0, db)
            }));
            out.extend(pair("WB", SpaceKind::WingBallast, zone, |s| {
                poly(s, &[(y_bot, db), (half_b, db), (half_b, d), (y_deck, d)])
            }));
        }
        // The liquid bulk module lays out its own Spaces: the ballast ones are the double bottom
        // and the wing tanks along the tank block (#330). The peaks follow, as for any hull.
        CargoClass::LiquidBulk => {
            if let Some(l) = r.liquid.as_ref() {
                out.extend(l.spaces.iter().filter(|s| s.kind.is_ballast()).cloned());
            }
        }
    }
    out.extend(peak("FP1C", (inp.loa - bow, inp.loa), half_b, inp.depth));
    if !inp.self_propelled {
        out.extend(peak("AP1C", (0.0, stern), half_b, inp.depth));
    }
    out
}

/// A peak tank, full beam and depth, from `x_aft` to `x_fwd`. None when it has no length.
fn peak(name: &str, (x_aft, x_fwd): (f64, f64), half_b: f64, depth: f64) -> Option<Space> {
    (x_fwd > x_aft && depth > 0.0).then(|| Space {
        name: name.into(),
        kind: SpaceKind::PeakBallast,
        side: Side::Centre,
        x_aft,
        x_fwd,
        section: vec![
            (-half_b, 0.0),
            (half_b, 0.0),
            (half_b, depth),
            (-half_b, depth),
        ],
    })
}

/// The water ballast tanks' total volume (m³).
pub fn total_volume_m3(spaces: &[Space]) -> f64 {
    spaces.iter().fold(0.0, |a, s| a + s.volume_m3())
}

/// A fill per Space, in order: every Space empty.
pub fn empty_fills(spaces: &[Space]) -> Vec<f64> {
    vec![0.0; spaces.len()]
}

/// The height of the consumables tanks (m). An Estimate for concept layout, not from a source.
pub const CONSUMABLES_TANK_HEIGHT_M: f64 = 1.0;

/// The consumables tanks of a self-propelled hull: one transverse pair on the bottom of the stern
/// zone, each half the beam wide and [`CONSUMABLES_TANK_HEIGHT_M`] high, as long as `volume_m3`
/// needs, from `x_aft`. A pair is the case IS Code 2008 Part B 3.1.4 asks for: at least one
/// transverse pair or one centreline tank with a free surface. Empty for no volume.
pub fn consumables_tanks(beam: f64, x_aft: f64, volume_m3: f64) -> Vec<Space> {
    let h = CONSUMABLES_TANK_HEIGHT_M;
    if volume_m3 <= 0.0 || beam <= 0.0 {
        return Vec::new();
    }
    let length = volume_m3 / (beam * h);
    pair("CT", SpaceKind::Consumables, (x_aft, x_aft + length), |s| {
        rect(s, 0.0, beam / 2.0, 0.0, h)
    })
}
