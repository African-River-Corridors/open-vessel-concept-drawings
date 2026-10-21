//! The Sections Table (ADR 0012, spec 0008): the master hull geometry, and the exact polygon
//! integrator that cuts it with a waterplane.
//!
//! A Section is the starboard half of a closed hull section at one Station. Its mirror closes it, so
//! the full section is a simple polygon. Between two Stations the hull is the ruled surface that joins
//! point `i` of one Section to point `i` of the next.
//!
//! The cut of one full section by the waterplane is a Sutherland-Hodgman clip against one half plane,
//! and the shoelace formula gives its area and first moments exactly. Along the length, each interval
//! is first split where a point line crosses the waterplane, and then integrated by Simpson's rule with
//! the mid-section taken from the ruled surface. That is exact (to rounding) wherever the immersed
//! area is at most cubic in x: a prismatic interval at any heel and trim, and the upright raked box.
//!
//! Axes: x forward from the aft end, y to starboard, z up from the base line. Heel is positive to
//! starboard (starboard goes down); trim is positive by the bow (bow goes down).
//!
//! Not yet wired into any result (issue #443).

use alloc::vec::Vec;
use core::fmt;
use libm::{cos, sin, sqrt};

/// One Station of the hull: the starboard half-section from the keel point on the centre line
/// `(0, z_keel)`, round the bottom and the side to the deck edge, and across the deck to the centre
/// line `(0, z_deck)`. Points are `(y, z)` in metres.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    /// Station, m from the aft end.
    pub x: f64,
    /// Half-section `(y >= 0, z)`, keel at `y = 0` first, deck at the centre line last.
    pub points: Vec<(f64, f64)>,
}

/// The master hull geometry: Sections in ascending x, with the same point count on each, so point
/// `i` of every Section lies on one longitudinal line. Two Sections may share an x (a step).
#[derive(Debug, Clone, PartialEq)]
pub struct SectionsTable {
    pub sections: Vec<Section>,
    /// Deck edge height per Section (the sheer), for the freeboard.
    pub depth_at_side: Vec<f64>,
}

/// Why a Sections Table is refused, or why an upright cut has no answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionsError {
    /// Fewer than two Sections.
    TooFewSections { count: usize },
    /// `depth_at_side` does not hold one value per Section.
    DepthCount { sections: usize, depths: usize },
    /// A Section has an x, a point or a deck edge height that is not finite.
    NotFinite { section: usize },
    /// A Section lies aft of the one before it.
    XDescending { section: usize },
    /// A Section has fewer than two points.
    TooFewPoints { section: usize },
    /// A Section's point count differs from the first Section's.
    PointCount {
        section: usize,
        expected: usize,
        found: usize,
    },
    /// A half-section does not start and end on the centre line (`y = 0`).
    OffCentreLine { section: usize },
    /// A point lies to port of the centre line.
    NegativeY { section: usize, point: usize },
    /// Two edges of a half-section cross.
    SelfCrossing { section: usize },
    /// The hull has no immersed volume or no waterplane at this draught.
    NotAfloat,
}

impl fmt::Display for SectionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            SectionsError::TooFewSections { count } => {
                write!(f, "a sections table needs at least 2 sections, not {count}")
            }
            SectionsError::DepthCount { sections, depths } => write!(
                f,
                "the sections table has {sections} sections but {depths} deck edge heights"
            ),
            SectionsError::NotFinite { section } => {
                write!(
                    f,
                    "section {section} holds a value that is not a finite number"
                )
            }
            SectionsError::XDescending { section } => {
                write!(f, "section {section} lies aft of the section before it")
            }
            SectionsError::TooFewPoints { section } => {
                write!(f, "section {section} needs at least 2 points")
            }
            SectionsError::PointCount {
                section,
                expected,
                found,
            } => write!(
                f,
                "section {section} has {found} points; every section needs {expected}"
            ),
            SectionsError::OffCentreLine { section } => write!(
                f,
                "section {section} does not start and end on the centre line (y = 0)"
            ),
            SectionsError::NegativeY { section, point } => {
                write!(f, "point {point} of section {section} has y below 0")
            }
            SectionsError::SelfCrossing { section } => {
                write!(f, "section {section} crosses itself")
            }
            SectionsError::NotAfloat => {
                write!(
                    f,
                    "the hull has no immersed volume or waterplane at this draught"
                )
            }
        }
    }
}

impl SectionsTable {
    /// A checked table: at least two Sections, x ascending, one deck edge height per Section, the
    /// same point count (at least two) on every Section, each half-section from and to the centre
    /// line with `y >= 0`, and no two edges of a half-section crossing.
    pub fn new(sections: Vec<Section>, depth_at_side: Vec<f64>) -> Result<Self, SectionsError> {
        if sections.len() < 2 {
            return Err(SectionsError::TooFewSections {
                count: sections.len(),
            });
        }
        if depth_at_side.len() != sections.len() {
            return Err(SectionsError::DepthCount {
                sections: sections.len(),
                depths: depth_at_side.len(),
            });
        }
        let expected = sections[0].points.len();
        for (i, s) in sections.iter().enumerate() {
            let finite = s.x.is_finite()
                && depth_at_side[i].is_finite()
                && s.points.iter().all(|p| p.0.is_finite() && p.1.is_finite());
            if !finite {
                return Err(SectionsError::NotFinite { section: i });
            }
            if i > 0 && s.x < sections[i - 1].x {
                return Err(SectionsError::XDescending { section: i });
            }
            if s.points.len() < 2 {
                return Err(SectionsError::TooFewPoints { section: i });
            }
            if s.points.len() != expected {
                return Err(SectionsError::PointCount {
                    section: i,
                    expected,
                    found: s.points.len(),
                });
            }
            if s.points[0].0 != 0.0 || s.points[s.points.len() - 1].0 != 0.0 {
                return Err(SectionsError::OffCentreLine { section: i });
            }
            if let Some(point) = s.points.iter().position(|p| p.0 < 0.0) {
                return Err(SectionsError::NegativeY { section: i, point });
            }
            if crosses_itself(&s.points) {
                return Err(SectionsError::SelfCrossing { section: i });
            }
        }
        Ok(Self {
            sections,
            depth_at_side,
        })
    }
}

/// `(b - a) x (c - a)`.
fn orient(a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> f64 {
    (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
}

/// True when two edges of the open chain cross at a point inside both. Touching or overlapping
/// edges do not count: a stem section with every point on the centre line is allowed.
fn crosses_itself(pts: &[(f64, f64)]) -> bool {
    let n = pts.len();
    for i in 0..n.saturating_sub(1) {
        for j in (i + 2)..n.saturating_sub(1) {
            let (a, b, c, d) = (pts[i], pts[i + 1], pts[j], pts[j + 1]);
            let (d1, d2) = (orient(a, b, c), orient(a, b, d));
            let (d3, d4) = (orient(c, d, a), orient(c, d, b));
            if d1 * d2 < 0.0 && d3 * d4 < 0.0 {
                return true;
            }
        }
    }
    false
}

/// The plane of the water surface relative to the hull: `p · k = c`, with the earth vertical in hull
/// axes `k = (−sin θ, −sin φ cos θ, cos φ cos θ)`. A point is immersed when `p · k < c`. Upright and
/// level, `c` is the draught.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Waterplane {
    /// Heel φ, degrees, positive to starboard.
    pub heel_deg: f64,
    /// Trim θ, degrees, positive by the bow.
    pub trim_deg: f64,
    /// Height of the plane along `k`, m.
    pub c: f64,
}

impl Waterplane {
    /// Upright and level at a draught above the base line.
    pub fn upright(draught: f64) -> Self {
        Self {
            heel_deg: 0.0,
            trim_deg: 0.0,
            c: draught,
        }
    }

    /// The earth vertical `k` in hull axes.
    pub fn normal(&self) -> (f64, f64, f64) {
        let (phi, theta) = (radians(self.heel_deg), radians(self.trim_deg));
        let (sp, cp, st, ct) = (sin(phi), cos(phi), sin(theta), cos(theta));
        (-st, -sp * ct, cp * ct)
    }
}

fn radians(deg: f64) -> f64 {
    deg * (core::f64::consts::PI / 180.0)
}

/// What one cut of the hull by a waterplane gives. Lengths in m, areas in m², volume in m³, second
/// moments in m⁴. Centroids are in hull axes. When nothing is immersed, every centroid is 0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Immersed {
    /// Immersed volume ∇.
    pub volume_m3: f64,
    /// Centre of buoyancy: x (LCB), y (TCB) and z (KB).
    pub lcb_m: f64,
    pub tcb_m: f64,
    pub kb_m: f64,
    /// Area of the waterplane, in its own plane.
    pub waterplane_area_m2: f64,
    /// Centroid of the waterplane: x (LCF), y (TCF) and z.
    pub lcf_m: f64,
    pub tcf_m: f64,
    pub vcf_m: f64,
    /// Second moment of the waterplane about its longitudinal axis through the centroid.
    pub i_t_m4: f64,
    /// Second moment of the waterplane about its transverse axis through the centroid.
    pub i_l_m4: f64,
    /// Wetted surface: the immersed ruled surface and the immersed end and step faces.
    pub wetted_surface_m2: f64,
    /// Length of the waterplane along x.
    pub l_wl_m: f64,
    /// Largest breadth of the waterplane.
    pub b_wl_m: f64,
    /// Immersed area of the section at mid-length of the waterplane.
    pub midship_area_m2: f64,
}

/// The upright hydrostatics at one draught, level. MCT uses KG = KB, the usual first estimate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Upright {
    pub draught_m: f64,
    pub immersed: Immersed,
    /// Δ = ρ∇, tonnes.
    pub displacement_t: f64,
    /// BM_T = I_T / ∇ and BM_L = I_L / ∇.
    pub bm_t_m: f64,
    pub bm_l_m: f64,
    /// KM_T = KB + BM_T and KM_L = KB + BM_L.
    pub km_t_m: f64,
    pub km_l_m: f64,
    /// Tonnes per cm immersion, ρ A_WP / 100.
    pub tpc_t: f64,
    /// Moment to change trim 1 cm, Δ BM_L / (100 L_WL), t·m.
    pub mct_1cm_tm: f64,
    /// Block coefficient ∇ / (L_WL B_WL T) (ES-TRIN 2025/1, Article 1.01, 4.9).
    pub cb: f64,
    /// Midship coefficient A_M / (B_WL T).
    pub cm: f64,
    /// Prismatic coefficient Cb / Cm.
    pub cp: f64,
    /// Waterplane coefficient A_WP / (L_WL B_WL).
    pub cwp: f64,
}

/// The waterplane in the form the cut needs.
struct Frame {
    kx: f64,
    ky: f64,
    kz: f64,
    c: f64,
    /// Unit direction of the waterline in a section plane, `(y, z)`.
    uy: f64,
    uz: f64,
    cos_trim: f64,
}

impl Frame {
    fn new(wp: &Waterplane) -> Self {
        let (kx, ky, kz) = wp.normal();
        let cos_trim = cos(radians(wp.trim_deg));
        Self {
            kx,
            ky,
            kz,
            c: wp.c,
            uy: kz / cos_trim,
            uz: -ky / cos_trim,
            cos_trim,
        }
    }

    /// `p · k − c`: negative when immersed.
    fn f(&self, x: f64, (y, z): (f64, f64)) -> f64 {
        self.kx * x + self.ky * y + self.kz * z - self.c
    }
}

/// The integrands of one cut of one full section, in the plane of the section.
#[derive(Debug, Clone, Copy, Default)]
struct Cut {
    area: f64,
    /// ∫y dA, ∫z dA.
    my: f64,
    mz: f64,
    /// Waterline chord: length, ∫y ds, ∫z ds, ∫s ds, ∫s² ds (s along the waterline).
    chord: f64,
    chord_y: f64,
    chord_z: f64,
    chord_s: f64,
    chord_s2: f64,
    /// Waterline breadth: from the first to the last crossing.
    breadth: f64,
    /// Immersed girth, with the slope of the surface along x.
    girth: f64,
}

/// The full section of a half-section: the half, then its mirror back to the keel.
fn full(half: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let n = half.len();
    let mut out = Vec::with_capacity(2 * n - 2);
    out.extend_from_slice(half);
    for &(y, z) in half[1..n - 1].iter().rev() {
        out.push((-y, z));
    }
    out
}

/// Interpolate two half-sections on the ruled surface: `(1 − t) a + t b`, exact at both ends.
fn lerp(a: &[(f64, f64)], b: &[(f64, f64)], t: f64) -> Vec<(f64, f64)> {
    a.iter()
        .zip(b)
        .map(|(p, q)| ((1.0 - t) * p.0 + t * q.0, (1.0 - t) * p.1 + t * q.1))
        .collect()
}

/// Cut one full section at station `x`. `wet` says which vertices are immersed (fixed for the whole
/// sub-interval, so an end of it takes the limit from inside); `tan` is the slope `(dy/dx, dz/dx)` of
/// each vertex's point line.
fn cut(fr: &Frame, x: f64, poly: &[(f64, f64)], tan: &[(f64, f64)], wet: &[bool]) -> Cut {
    let n = poly.len();
    let f: Vec<f64> = poly.iter().map(|&p| fr.f(x, p)).collect();
    let mut clipped: Vec<(f64, f64)> = Vec::with_capacity(n + 4);
    // (s, y, z) of each crossing of the waterline.
    let mut crossings: Vec<(f64, f64, f64)> = Vec::new();
    let mut out = Cut::default();
    for i in 0..n {
        let j = (i + 1) % n;
        let (p, q) = (poly[i], poly[j]);
        if wet[i] {
            clipped.push(p);
        }
        if wet[i] && wet[j] {
            out.girth += piece(p, q, tan[i], tan[j]);
        } else if wet[i] != wet[j] {
            let t = if f[i] == f[j] {
                0.5
            } else {
                (f[i] / (f[i] - f[j])).clamp(0.0, 1.0)
            };
            let r = (p.0 + t * (q.0 - p.0), p.1 + t * (q.1 - p.1));
            let tr = (
                tan[i].0 + t * (tan[j].0 - tan[i].0),
                tan[i].1 + t * (tan[j].1 - tan[i].1),
            );
            clipped.push(r);
            out.girth += if wet[i] {
                piece(p, r, tan[i], tr)
            } else {
                piece(r, q, tr, tan[j])
            };
            crossings.push((fr.uy * r.0 + fr.uz * r.1, r.0, r.1));
        }
    }
    let (mut a, mut my, mut mz) = (0.0, 0.0, 0.0);
    let m = clipped.len();
    for i in 0..m {
        let ((y0, z0), (y1, z1)) = (clipped[i], clipped[(i + 1) % m]);
        let c = y0 * z1 - y1 * z0;
        a += c;
        my += (y0 + y1) * c;
        mz += (z0 + z1) * c;
    }
    out.area = a / 2.0;
    out.my = my / 6.0;
    out.mz = mz / 6.0;
    crossings.sort_by(|p, q| p.0.total_cmp(&q.0));
    for w in crossings.as_chunks::<2>().0 {
        let ((s0, y0, z0), (s1, y1, z1)) = (w[0], w[1]);
        let len = s1 - s0;
        out.chord += len;
        out.chord_y += len * (y0 + y1) / 2.0;
        out.chord_z += len * (z0 + z1) / 2.0;
        out.chord_s += (s1 * s1 - s0 * s0) / 2.0;
        out.chord_s2 += (s1 * s1 * s1 - s0 * s0 * s0) / 3.0;
    }
    if let (Some(first), Some(last)) = (crossings.first(), crossings.last()) {
        out.breadth = last.0 - first.0;
    }
    out
}

/// Surface area per unit x of the ruled surface along the edge `p → q`: `|t × e|` with the
/// longitudinal tangent `t = (1, dy/dx, dz/dx)` at the middle of the edge and `e = q − p`.
fn piece(p: (f64, f64), q: (f64, f64), tp: (f64, f64), tq: (f64, f64)) -> f64 {
    let (dy, dz) = (q.0 - p.0, q.1 - p.1);
    let (ty, tz) = ((tp.0 + tq.0) / 2.0, (tp.1 + tq.1) / 2.0);
    let a = ty * dz - tz * dy;
    sqrt(a * a + dz * dz + dy * dy)
}

/// The integrands that Simpson's rule sums along x.
#[derive(Debug, Clone, Copy, Default)]
struct Sums {
    area: f64,
    x_area: f64,
    my: f64,
    mz: f64,
    chord: f64,
    x_chord: f64,
    x2_chord: f64,
    chord_y: f64,
    chord_z: f64,
    chord_s: f64,
    chord_s2: f64,
    girth: f64,
}

impl Sums {
    fn add(&mut self, w: f64, x: f64, c: &Cut) {
        self.area += w * c.area;
        self.x_area += w * x * c.area;
        self.my += w * c.my;
        self.mz += w * c.mz;
        self.chord += w * c.chord;
        self.x_chord += w * x * c.chord;
        self.x2_chord += w * x * x * c.chord;
        self.chord_y += w * c.chord_y;
        self.chord_z += w * c.chord_z;
        self.chord_s += w * c.chord_s;
        self.chord_s2 += w * c.chord_s2;
        self.girth += w * c.girth;
    }
}

/// One interval of the table, between two Sections at different x.
struct Interval<'a> {
    x0: f64,
    x1: f64,
    a: &'a [(f64, f64)],
    b: &'a [(f64, f64)],
    /// Slope of each vertex's point line, on the full section.
    tan: Vec<(f64, f64)>,
}

impl<'a> Interval<'a> {
    fn new(s0: &'a Section, s1: &'a Section) -> Self {
        let h = s1.x - s0.x;
        let half: Vec<(f64, f64)> = s0
            .points
            .iter()
            .zip(&s1.points)
            .map(|(p, q)| ((q.0 - p.0) / h, (q.1 - p.1) / h))
            .collect();
        Self {
            x0: s0.x,
            x1: s1.x,
            a: &s0.points,
            b: &s1.points,
            tan: full(&half),
        }
    }

    fn full_at(&self, x: f64) -> Vec<(f64, f64)> {
        full(&lerp(self.a, self.b, (x - self.x0) / (self.x1 - self.x0)))
    }

    /// The interval's ends and every x inside it where a point line crosses the waterplane. Each
    /// point line is straight, so `p · k − c` is linear along it and has one exact root.
    fn splits(&self, fr: &Frame) -> Vec<f64> {
        let (pa, pb) = (full(self.a), full(self.b));
        let mut xs = Vec::with_capacity(4);
        xs.push(self.x0);
        for (p, q) in pa.iter().zip(&pb) {
            let (f0, f1) = (fr.f(self.x0, *p), fr.f(self.x1, *q));
            if (f0 < 0.0) != (f1 < 0.0) && f0 != f1 {
                let r = self.x0 + (self.x1 - self.x0) * (f0 / (f0 - f1));
                if r > self.x0 && r < self.x1 {
                    xs.push(r);
                }
            }
        }
        xs.push(self.x1);
        xs.sort_by(f64::total_cmp);
        xs.dedup();
        xs
    }

    /// Which vertices are immersed on the open sub-interval `(a, b)`: none crosses inside it, so
    /// the sign at its middle holds throughout.
    fn wet(&self, fr: &Frame, a: f64, b: f64) -> Vec<bool> {
        let m = (a + b) / 2.0;
        self.full_at(m).iter().map(|&p| fr.f(m, p) < 0.0).collect()
    }
}

/// Cut the hull by a waterplane and integrate along the length.
pub fn immersed(table: &SectionsTable, wp: &Waterplane) -> Immersed {
    let fr = Frame::new(wp);
    let mut sums = Sums::default();
    let mut b_wl: f64 = 0.0;
    let mut wl: Option<(f64, f64)> = None;
    // End and step faces: the immersed area on each side of a step, and at both ends.
    let mut faces = 0.0;
    let mut pending_face: Option<f64> = Some(0.0);
    let mut last_area = 0.0;
    for pair in table.sections.windows(2) {
        let (s0, s1) = (&pair[0], &pair[1]);
        if s1.x <= s0.x {
            pending_face = Some(last_area);
            continue;
        }
        let iv = Interval::new(s0, s1);
        let xs = iv.splits(&fr);
        for w in xs.windows(2) {
            let (a, b) = (w[0], w[1]);
            let m = (a + b) / 2.0;
            let wet = iv.wet(&fr, a, b);
            let ca = cut(&fr, a, &iv.full_at(a), &iv.tan, &wet);
            let cm = cut(&fr, m, &iv.full_at(m), &iv.tan, &wet);
            let cb = cut(&fr, b, &iv.full_at(b), &iv.tan, &wet);
            let h = (b - a) / 6.0;
            sums.add(h, a, &ca);
            sums.add(4.0 * h, m, &cm);
            sums.add(h, b, &cb);
            if let Some(prev) = pending_face.take() {
                faces += (ca.area - prev).abs();
            }
            last_area = cb.area;
            if ca.chord > 0.0 || cm.chord > 0.0 || cb.chord > 0.0 {
                wl = Some(match wl {
                    None => (a, b),
                    Some((lo, hi)) => (lo.min(a), hi.max(b)),
                });
            }
            b_wl = b_wl.max(ca.breadth).max(cm.breadth).max(cb.breadth);
        }
    }
    faces += last_area.abs();

    let v = sums.area;
    let (lcb, tcb, kb) = if v > 0.0 {
        (sums.x_area / v, sums.my / v, sums.mz / v)
    } else {
        (0.0, 0.0, 0.0)
    };
    let l = sums.chord;
    let ct = fr.cos_trim;
    let (lcf, tcf, vcf, i_t, i_l) = if l > 0.0 {
        let (xf, sf) = (sums.x_chord / l, sums.chord_s / l);
        (
            xf,
            sums.chord_y / l,
            sums.chord_z / l,
            (sums.chord_s2 - sf * sf * l) / ct,
            (sums.x2_chord - xf * xf * l) / (ct * ct * ct),
        )
    } else {
        (0.0, 0.0, 0.0, 0.0, 0.0)
    };
    let (l_wl, midship) = match wl {
        Some((lo, hi)) => (hi - lo, station_cut(table, &fr, lo + (hi - lo) / 2.0).area),
        None => (0.0, 0.0),
    };
    Immersed {
        volume_m3: v,
        lcb_m: lcb,
        tcb_m: tcb,
        kb_m: kb,
        waterplane_area_m2: l / ct,
        lcf_m: lcf,
        tcf_m: tcf,
        vcf_m: vcf,
        i_t_m4: i_t,
        i_l_m4: i_l,
        wetted_surface_m2: sums.girth + faces,
        l_wl_m: l_wl,
        b_wl_m: b_wl,
        midship_area_m2: midship,
    }
}

/// The cut of the section at any x on the ruled surface. At a Section shared by two intervals the
/// interval aft of it is used.
fn station_cut(table: &SectionsTable, fr: &Frame, x: f64) -> Cut {
    for pair in table.sections.windows(2) {
        let (s0, s1) = (&pair[0], &pair[1]);
        if s1.x <= s0.x || x < s0.x || x > s1.x {
            continue;
        }
        let iv = Interval::new(s0, s1);
        let xs = iv.splits(fr);
        for w in xs.windows(2) {
            if x <= w[1] {
                let wet = iv.wet(fr, w[0], w[1]);
                return cut(fr, x, &iv.full_at(x), &iv.tan, &wet);
            }
        }
    }
    Cut::default()
}

/// The upright hydrostatics at one draught (m above the base line), level, in water of `density`
/// (t/m³).
pub fn upright(
    table: &SectionsTable,
    draught: f64,
    density: f64,
) -> Result<Upright, SectionsError> {
    let im = immersed(table, &Waterplane::upright(draught));
    let (v, awp) = (im.volume_m3, im.waterplane_area_m2);
    if !(v > 0.0 && awp > 0.0 && draught > 0.0 && im.l_wl_m > 0.0 && im.b_wl_m > 0.0) {
        return Err(SectionsError::NotAfloat);
    }
    let displacement = density * v;
    let (bm_t, bm_l) = (im.i_t_m4 / v, im.i_l_m4 / v);
    let cb = v / (im.l_wl_m * im.b_wl_m * draught);
    let cm = im.midship_area_m2 / (im.b_wl_m * draught);
    Ok(Upright {
        draught_m: draught,
        immersed: im,
        displacement_t: displacement,
        bm_t_m: bm_t,
        bm_l_m: bm_l,
        km_t_m: im.kb_m + bm_t,
        km_l_m: im.kb_m + bm_l,
        tpc_t: density * awp / 100.0,
        mct_1cm_tm: displacement * bm_l / (100.0 * im.l_wl_m),
        cb,
        cm,
        cp: cb / cm,
        cwp: awp / (im.l_wl_m * im.b_wl_m),
    })
}
