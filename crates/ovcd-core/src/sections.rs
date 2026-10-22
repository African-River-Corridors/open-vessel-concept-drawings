//! The Sections Table (ADR 0012, spec 0008): the master hull geometry, and the exact polygon
//! integrator that cuts it with a waterplane.
//!
//! A Section is the starboard half of a closed hull section at one Station. Its mirror closes it, so
//! the full section is a simple polygon. Between two Stations the hull is the ruled surface that joins
//! point `i` of one Section to point `i` of the next.
//!
//! A slotted Section (the Notch of an ATB barge, issue #465) starts and ends at `y = y_in > 0`: a slot
//! of half width `y_in` on the centre line. The slot wall closes the half, so the full section is two
//! simple polygons, starboard and port, and every cut sums over the two. A ruled interval joins two
//! slotted or two whole Sections, never one of each: a slot ends at a step.
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
//! Not yet wired into any result (issues #443 and #444).

use alloc::vec::Vec;
use core::fmt;
use libm::{cos, fabs, sin, sqrt};

/// One Station of the hull: the starboard half-section from the keel point on the centre line
/// `(0, z_keel)`, round the bottom and the side to the deck edge, and across the deck to the centre
/// line `(0, z_deck)`. Points are `(y, z)` in metres.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    /// Station, m from the aft end.
    pub x: f64,
    /// Half-section `(y >= 0, z)`, keel at `y = 0` first, deck at the centre line last. A slotted
    /// half-section starts and ends at the slot edge `y = y_in` instead.
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
    /// A half-section does not start and end at one y: both on the centre line (`y = 0`), or both on
    /// the edge of a slot.
    OffCentreLine { section: usize },
    /// A ruled interval joins a slotted and a whole Section: a slot must end at a step.
    SlotWithoutStep { section: usize },
    /// A point lies to port of the centre line.
    NegativeY { section: usize, point: usize },
    /// Two edges of a half-section cross.
    SelfCrossing { section: usize },
    /// The hull has no immersed volume or no waterplane at this draught, or no mass to float.
    NotAfloat,
    /// The mass is more than the whole closed hull displaces.
    Sinks,
    /// The free-trim equilibrium needs a trim beyond ±10 degrees.
    TrimOutOfRange,
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
                "section {section} does not start and end on the centre line (y = 0) or on one slot edge"
            ),
            SectionsError::SlotWithoutStep { section } => write!(
                f,
                "section {section} and the section before it join a slotted and a whole section; a \
                 slot must end at a step (two sections at one x)"
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
            SectionsError::Sinks => write!(f, "the mass is more than the whole hull displaces"),
            SectionsError::TrimOutOfRange => write!(
                f,
                "the hull would trim more than {TRIM_LIMIT_DEG} degrees; the trim solver stops there"
            ),
        }
    }
}

impl SectionsTable {
    /// The half-section at `x` on the ruled surface between the Sections either side, and its deck
    /// edge height (spec 0008, "the section outline at that x"). At a Station, that Section; at two
    /// Sections sharing an x, the first. `x` outside the table takes the end Section.
    pub fn section_at(&self, x: f64) -> (Section, f64) {
        let ss = &self.sections;
        let last = ss.len() - 1;
        if x <= ss[0].x {
            return (ss[0].clone(), self.depth_at_side[0]);
        }
        if x >= ss[last].x {
            return (ss[last].clone(), self.depth_at_side[last]);
        }
        let j = ss.iter().position(|s| s.x > x).unwrap_or(last);
        let (a, b) = (&ss[j - 1], &ss[j]);
        if a.x == x {
            return (a.clone(), self.depth_at_side[j - 1]);
        }
        let u = (x - a.x) / (b.x - a.x);
        let lerp = |p: f64, q: f64| p + u * (q - p);
        let points = a
            .points
            .iter()
            .zip(&b.points)
            .map(|(p, q)| (lerp(p.0, q.0), lerp(p.1, q.1)))
            .collect();
        let edge = lerp(self.depth_at_side[j - 1], self.depth_at_side[j]);
        (Section { x, points }, edge)
    }

    /// A checked table: at least two Sections, x ascending, one deck edge height per Section, the
    /// same point count (at least two) on every Section, each half-section from and to the centre
    /// line (or from and to one slot edge) with `y >= 0`, no two edges of a half-section crossing,
    /// and no ruled interval from a slotted to a whole Section.
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
            if s.points[0].0 != s.points[s.points.len() - 1].0 {
                return Err(SectionsError::OffCentreLine { section: i });
            }
            if let Some(point) = s.points.iter().position(|p| p.0 < 0.0) {
                return Err(SectionsError::NegativeY { section: i, point });
            }
            if crosses_itself(&s.points) {
                return Err(SectionsError::SelfCrossing { section: i });
            }
            if i > 0 && s.x > sections[i - 1].x && slotted(s) != slotted(&sections[i - 1]) {
                return Err(SectionsError::SlotWithoutStep { section: i });
            }
        }
        Ok(Self {
            sections,
            depth_at_side,
        })
    }
}

/// True when the half-section starts off the centre line: a slot.
fn slotted(s: &Section) -> bool {
    s.points[0].0 > 0.0
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
            // Edges on one straight line (a flat of bottom, a straight side) differ by rounding
            // only: a point within 1e-12 of the other edge's line is on it, not across it.
            let scale = [a, b, c, d]
                .iter()
                .fold(1.0_f64, |m, p| m.max(fabs(p.0)).max(fabs(p.1)));
            let side = |p: (f64, f64), q: (f64, f64), r: (f64, f64)| {
                let len = sqrt((q.0 - p.0) * (q.0 - p.0) + (q.1 - p.1) * (q.1 - p.1));
                let o = orient(p, q, r);
                if len == 0.0 || fabs(o) <= 1e-12 * scale * len {
                    0.0
                } else {
                    o
                }
            };
            let (d1, d2) = (side(a, b, c), side(a, b, d));
            let (d3, d4) = (side(c, d, a), side(c, d, b));
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

/// Buffers reused from cut to cut, so the solvers do not allocate in their inner loops.
#[derive(Default)]
struct Work {
    /// The full section being cut.
    poly: Vec<(f64, f64)>,
    /// Slope `(dy/dx, dz/dx)` of each vertex's point line.
    tan: Vec<(f64, f64)>,
    /// Which vertices are immersed.
    wet: Vec<bool>,
    f: Vec<f64>,
    clipped: Vec<(f64, f64)>,
    /// Where the second polygon of a slotted section starts in `poly` (`poly.len()` when whole).
    split: usize,
    /// `(s, y, z)` of each crossing of the waterline.
    crossings: Vec<(f64, f64, f64)>,
    xs: Vec<f64>,
}

/// Write the full section of the half-section `(1 − t) a + t b` (exact at both ends): the half, then
/// its mirror back to the keel. A slotted half is closed by its slot wall, and its whole mirror
/// follows as the second polygon. Gives where the second polygon starts (`dst.len()` when whole).
fn fill_full(
    dst: &mut Vec<(f64, f64)>,
    a: &[(f64, f64)],
    b: &[(f64, f64)],
    t: f64,
    slot: bool,
) -> usize {
    let n = a.len();
    dst.clear();
    for (p, q) in a.iter().zip(b) {
        dst.push(((1.0 - t) * p.0 + t * q.0, (1.0 - t) * p.1 + t * q.1));
    }
    let mirrored = if slot { 0..n } else { 1..n - 1 };
    for i in mirrored.rev() {
        let (y, z) = dst[i];
        dst.push((-y, z));
    }
    if slot { n } else { dst.len() }
}

/// Cut the full section in `w.poly` at station `x`. `w.wet` says which vertices are immersed (fixed
/// for the whole sub-interval, so an end of it takes the limit from inside). With `area_only`, only
/// the area is worked out.
fn cut(fr: &Frame, x: f64, w: &mut Work, area_only: bool) -> Cut {
    let n = w.poly.len();
    w.f.clear();
    w.f.extend(w.poly.iter().map(|&p| fr.f(x, p)));
    w.clipped.clear();
    w.crossings.clear();
    let (poly, tan, wet, f) = (&w.poly, &w.tan, &w.wet, &w.f);
    let mut out = Cut::default();
    // One polygon, or two for a slotted section; each closes on itself.
    let rings = [(0, w.split), (w.split, n)];
    let mut ends = [0; 2];
    for (r, &(start, end)) in rings.iter().enumerate() {
        for i in start..end {
            let j = if i + 1 == end { start } else { i + 1 };
            let (p, q) = (poly[i], poly[j]);
            if wet[i] {
                w.clipped.push(p);
            }
            if wet[i] && wet[j] {
                if !area_only {
                    out.girth += piece(p, q, tan[i], tan[j]);
                }
            } else if wet[i] != wet[j] {
                let t = if f[i] == f[j] {
                    0.5
                } else {
                    (f[i] / (f[i] - f[j])).clamp(0.0, 1.0)
                };
                let r = (p.0 + t * (q.0 - p.0), p.1 + t * (q.1 - p.1));
                w.clipped.push(r);
                if !area_only {
                    let tr = (
                        tan[i].0 + t * (tan[j].0 - tan[i].0),
                        tan[i].1 + t * (tan[j].1 - tan[i].1),
                    );
                    out.girth += if wet[i] {
                        piece(p, r, tan[i], tr)
                    } else {
                        piece(r, q, tr, tan[j])
                    };
                    w.crossings.push((fr.uy * r.0 + fr.uz * r.1, r.0, r.1));
                }
            }
        }
        ends[r] = w.clipped.len();
    }
    let (mut a, mut my, mut mz) = (0.0, 0.0, 0.0);
    for (start, end) in [(0, ends[0]), (ends[0], ends[1])] {
        let m = end - start;
        for i in 0..m {
            let ((y0, z0), (y1, z1)) = (w.clipped[start + i], w.clipped[start + (i + 1) % m]);
            let c = y0 * z1 - y1 * z0;
            a += c;
            if !area_only {
                my += (y0 + y1) * c;
                mz += (z0 + z1) * c;
            }
        }
    }
    out.area = a / 2.0;
    if area_only {
        return out;
    }
    out.my = my / 6.0;
    out.mz = mz / 6.0;
    w.crossings.sort_by(|p, q| p.0.total_cmp(&q.0));
    for pair in w.crossings.as_chunks::<2>().0 {
        let ((s0, y0, z0), (s1, y1, z1)) = (pair[0], pair[1]);
        let len = s1 - s0;
        out.chord += len;
        out.chord_y += len * (y0 + y1) / 2.0;
        out.chord_z += len * (z0 + z1) / 2.0;
        out.chord_s += (s1 * s1 - s0 * s0) / 2.0;
        out.chord_s2 += (s1 * s1 * s1 - s0 * s0 * s0) / 3.0;
    }
    if let (Some(first), Some(last)) = (w.crossings.first(), w.crossings.last()) {
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
    /// Both Sections are slotted (a checked table never joins a slotted and a whole one).
    slot: bool,
}

impl<'a> Interval<'a> {
    fn new(s0: &'a Section, s1: &'a Section) -> Self {
        Self {
            x0: s0.x,
            x1: s1.x,
            a: &s0.points,
            b: &s1.points,
            slot: slotted(s0),
        }
    }

    /// Put the full section at `x` into `w.poly`.
    fn section_at(&self, x: f64, w: &mut Work) {
        w.split = fill_full(
            &mut w.poly,
            self.a,
            self.b,
            (x - self.x0) / (self.x1 - self.x0),
            self.slot,
        );
    }

    /// Put the slope of each vertex's point line into `w.tan`.
    fn slopes(&self, w: &mut Work) {
        let h = self.x1 - self.x0;
        let n = self.a.len();
        w.tan.clear();
        for (p, q) in self.a.iter().zip(self.b) {
            w.tan.push(((q.0 - p.0) / h, (q.1 - p.1) / h));
        }
        let mirrored = if self.slot { 0..n } else { 1..n - 1 };
        for i in mirrored.rev() {
            let (ty, tz) = w.tan[i];
            w.tan.push((-ty, tz));
        }
    }

    /// Put into `w.xs` the interval's ends and every x inside it where a point line crosses the
    /// waterplane. Each point line is straight, so `p · k − c` is linear along it and has one exact
    /// root.
    fn splits(&self, fr: &Frame, w: &mut Work) {
        let n = self.a.len();
        w.xs.clear();
        w.xs.push(self.x0);
        for (i, (p, q)) in self.a.iter().zip(self.b).enumerate() {
            let mirror = self.slot || (i > 0 && i < n - 1);
            for (pp, qq) in [(*p, *q), ((-p.0, p.1), (-q.0, q.1))]
                .into_iter()
                .take(if mirror { 2 } else { 1 })
            {
                let (f0, f1) = (fr.f(self.x0, pp), fr.f(self.x1, qq));
                if (f0 < 0.0) != (f1 < 0.0) && f0 != f1 {
                    let r = self.x0 + (self.x1 - self.x0) * (f0 / (f0 - f1));
                    if r > self.x0 && r < self.x1 {
                        w.xs.push(r);
                    }
                }
            }
        }
        w.xs.push(self.x1);
        w.xs.sort_by(f64::total_cmp);
        w.xs.dedup();
    }

    /// Set `w.wet` for the open sub-interval `(a, b)`: no point line crosses inside it, so the sign
    /// at its middle holds throughout.
    fn wet(&self, fr: &Frame, a: f64, b: f64, w: &mut Work) {
        let m = (a + b) / 2.0;
        self.section_at(m, w);
        w.wet.clear();
        w.wet.extend(w.poly.iter().map(|&p| fr.f(m, p) < 0.0));
    }
}

/// What the integration along the length gives.
struct Integral {
    sums: Sums,
    /// End and step faces: the immersed area on each side of a step, and at both ends.
    faces: f64,
    /// Aft and forward ends of the waterplane.
    wl: Option<(f64, f64)>,
    b_wl: f64,
}

/// Integrate the cuts along the length by Simpson's rule on each sub-interval.
fn integrate(table: &SectionsTable, fr: &Frame, area_only: bool, w: &mut Work) -> Integral {
    let mut out = Integral {
        sums: Sums::default(),
        faces: 0.0,
        wl: None,
        b_wl: 0.0,
    };
    let mut pending_face: Option<f64> = Some(0.0);
    let mut last_area = 0.0;
    let mut xs = core::mem::take(&mut w.xs);
    for pair in table.sections.windows(2) {
        let (s0, s1) = (&pair[0], &pair[1]);
        if s1.x <= s0.x {
            pending_face = Some(last_area);
            continue;
        }
        let iv = Interval::new(s0, s1);
        if !area_only {
            iv.slopes(w);
        }
        w.xs = xs;
        iv.splits(fr, w);
        xs = core::mem::take(&mut w.xs);
        for ab in xs.windows(2) {
            let (a, b) = (ab[0], ab[1]);
            let m = (a + b) / 2.0;
            iv.wet(fr, a, b, w);
            let cm = cut(fr, m, w, area_only);
            iv.section_at(a, w);
            let ca = cut(fr, a, w, area_only);
            iv.section_at(b, w);
            let cb = cut(fr, b, w, area_only);
            let h = (b - a) / 6.0;
            out.sums.add(h, a, &ca);
            out.sums.add(4.0 * h, m, &cm);
            out.sums.add(h, b, &cb);
            if let Some(prev) = pending_face.take() {
                out.faces += (ca.area - prev).abs();
            }
            last_area = cb.area;
            if ca.chord > 0.0 || cm.chord > 0.0 || cb.chord > 0.0 {
                out.wl = Some(match out.wl {
                    None => (a, b),
                    Some((lo, hi)) => (lo.min(a), hi.max(b)),
                });
            }
            out.b_wl = out.b_wl.max(ca.breadth).max(cm.breadth).max(cb.breadth);
        }
    }
    w.xs = xs;
    out.faces += last_area.abs();
    out
}

/// Cut the hull by a waterplane and integrate along the length.
pub fn immersed(table: &SectionsTable, wp: &Waterplane) -> Immersed {
    let fr = Frame::new(wp);
    let mut w = Work::default();
    let Integral {
        sums,
        faces,
        wl,
        b_wl,
    } = integrate(table, &fr, false, &mut w);
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
        Some((lo, hi)) => (
            hi - lo,
            station_cut(table, &fr, lo + (hi - lo) / 2.0, &mut w).area,
        ),
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
fn station_cut(table: &SectionsTable, fr: &Frame, x: f64, w: &mut Work) -> Cut {
    for pair in table.sections.windows(2) {
        let (s0, s1) = (&pair[0], &pair[1]);
        if s1.x <= s0.x || x < s0.x || x > s1.x {
            continue;
        }
        let iv = Interval::new(s0, s1);
        iv.slopes(w);
        iv.splits(fr, w);
        let xs = core::mem::take(&mut w.xs);
        let hit = xs.windows(2).find(|ab| x <= ab[1]).map(|ab| (ab[0], ab[1]));
        w.xs = xs;
        if let Some((a, b)) = hit {
            iv.wet(fr, a, b, w);
            iv.section_at(x, w);
            return cut(fr, x, w, false);
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

/// The most steps of the inner solve, on the height `c` for the volume (Illinois secant method). A
/// solve stops earlier when the volume is within [`VOLUME_TOLERANCE`] of the target. The stop is a
/// comparison of numbers the solve computes, so native and WebAssembly stop at the same step and
/// give the same bits.
pub const VOLUME_STEPS: u32 = 16;
/// The inner solve stops when the volume is within this share of the target volume.
pub const VOLUME_TOLERANCE: f64 = 1e-13;
/// The most steps of the outer solve, on the trim for the longitudinal moment (Illinois secant
/// method). A solve stops earlier when the lever is within [`TRIM_TOLERANCE_M`].
pub const TRIM_STEPS: u32 = 12;
/// The outer solve stops when B - G along the earth's forward horizontal is within this (m).
pub const TRIM_TOLERANCE_M: f64 = 1e-11;
/// The free-trim solve looks for the trim between −10 and +10 degrees.
pub const TRIM_LIMIT_DEG: f64 = 10.0;

/// The centre of gravity of the loaded hull: x (LCG), y (TCG) and z (KG), m in hull axes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gravity {
    pub lcg_m: f64,
    pub tcg_m: f64,
    pub kg_m: f64,
}

/// The floating position at one heel: the height of the waterplane, the trim, and the cut there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Equilibrium {
    pub waterplane: Waterplane,
    pub immersed: Immersed,
}

/// The height `c` that immerses `target` m³ at this heel and trim, by the Illinois secant method in
/// a bracket from the lowest to the highest point of the hull along `k`. At those ends the volume is
/// 0 and `total` (the whole hull), so the bracket costs no cut.
fn c_for_volume(
    table: &SectionsTable,
    heel_deg: f64,
    trim_deg: f64,
    target: f64,
    total: f64,
    w: &mut Work,
) -> f64 {
    let at = |c: f64| Waterplane {
        heel_deg,
        trim_deg,
        c,
    };
    let fr = Frame::new(&at(0.0));
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for s in &table.sections {
        for &(y, z) in &s.points {
            for p in [(y, z), (-y, z)] {
                let d = fr.f(s.x, p);
                lo = lo.min(d);
                hi = hi.max(d);
            }
        }
    }
    illinois(
        VOLUME_TOLERANCE * target,
        lo,
        -target,
        hi,
        total - target,
        VOLUME_STEPS,
        |c| integrate(table, &Frame::new(&at(c)), true, w).sums.area - target,
    )
    .0
}

/// The upright, level draught that immerses `volume` m³ (issue #448): the inner solve of
/// [`equilibrium`] at heel 0 and trim 0, at most [`VOLUME_STEPS`] steps.
pub fn draught_for_volume(table: &SectionsTable, volume: f64) -> Result<f64, SectionsError> {
    if volume.is_nan() || volume <= 0.0 {
        return Err(SectionsError::NotAfloat);
    }
    let total = immersed(table, &Waterplane::upright(f64::MAX / 4.0)).volume_m3;
    if volume >= total {
        return Err(SectionsError::Sinks);
    }
    let mut w = Work::default();
    Ok(c_for_volume(table, 0.0, 0.0, volume, total, &mut w))
}

/// The Illinois secant method on a bracket `[a, b]` with `f(a) <= 0 <= f(b)`, `f` rising: at most
/// `steps` steps, stopping at the first whose residual is within `tol`. Gives the last point and
/// what `f` returned there.
fn illinois<T>(
    tol: f64,
    mut a: f64,
    mut fa: f64,
    mut b: f64,
    mut fb: f64,
    steps: u32,
    mut f: impl FnMut(f64) -> T,
) -> (f64, Option<T>)
where
    T: Residual,
{
    let mut last = None;
    let mut x = b;
    for _ in 0..steps {
        x = if fb != fa {
            (a * fb - b * fa) / (fb - fa)
        } else {
            b
        };
        let r = f(x);
        let fx = r.residual();
        if fx.abs() <= tol {
            return (x, Some(r));
        }
        if (fx < 0.0) != (fb < 0.0) {
            a = b;
            fa = fb;
        } else {
            fa /= 2.0;
        }
        b = x;
        fb = fx;
        last = Some(r);
    }
    (x, last)
}

/// What a step of [`illinois`] gives: its residual, and anything the caller keeps.
trait Residual {
    fn residual(&self) -> f64;
}

impl Residual for f64 {
    fn residual(&self) -> f64 {
        *self
    }
}

/// Unit vectors of the earth frame in hull axes: `u` horizontal across (towards the low side at a
/// positive heel), `w` horizontal forward. `k` is the vertical.
fn earth_axes(wp: &Waterplane) -> ((f64, f64, f64), (f64, f64, f64)) {
    let fr = Frame::new(wp);
    let u = (0.0, fr.uy, fr.uz);
    // w = u × k.
    let w = (u.1 * fr.kz - u.2 * fr.ky, u.2 * fr.kx, -u.1 * fr.kx);
    (u, w)
}

fn dot(a: (f64, f64, f64), b: (f64, f64, f64)) -> f64 {
    a.0 * b.0 + a.1 * b.1 + a.2 * b.2
}

/// The floating position at a heel with free trim: the height `c` gives ρ∇ = Δ (inner solve), and
/// the trim puts the centre of buoyancy on the vertical through G in the longitudinal plane (outer
/// solve, between ±10 degrees). Both are Illinois secant solves with a cap on the steps and a stop
/// at a tolerance.
pub fn equilibrium(
    table: &SectionsTable,
    mass_t: f64,
    g: Gravity,
    heel_deg: f64,
    density: f64,
) -> Result<Equilibrium, SectionsError> {
    let target = mass_t / density;
    if target.is_nan() || target <= 0.0 {
        return Err(SectionsError::NotAfloat);
    }
    // The whole hull immersed: nothing heavier floats.
    let total = immersed(
        table,
        &Waterplane {
            heel_deg: 0.0,
            trim_deg: 0.0,
            c: f64::MAX / 4.0,
        },
    )
    .volume_m3;
    if target >= total {
        return Err(SectionsError::Sinks);
    }
    let gp = (g.lcg_m, g.tcg_m, g.kg_m);
    let mut w = Work::default();
    // B − G along the earth's forward horizontal, at a trim: it rises with the trim (bow down
    // moves B forward).
    let mut float_at = |trim_deg: f64| {
        let c = c_for_volume(table, heel_deg, trim_deg, target, total, &mut w);
        let waterplane = Waterplane {
            heel_deg,
            trim_deg,
            c,
        };
        let immersed = immersed(table, &waterplane);
        let (_, fwd) = earth_axes(&waterplane);
        let lever = dot((immersed.lcb_m, immersed.tcb_m, immersed.kb_m), fwd) - dot(gp, fwd);
        Floating {
            eq: Equilibrium {
                waterplane,
                immersed,
            },
            lever,
        }
    };
    let aft = float_at(-TRIM_LIMIT_DEG);
    let fwd = float_at(TRIM_LIMIT_DEG);
    if aft.lever > 0.0 || fwd.lever < 0.0 {
        return Err(SectionsError::TrimOutOfRange);
    }
    let (_, last) = illinois(
        TRIM_TOLERANCE_M,
        -TRIM_LIMIT_DEG,
        aft.lever,
        TRIM_LIMIT_DEG,
        fwd.lever,
        TRIM_STEPS,
        &mut float_at,
    );
    Ok(last.unwrap_or(fwd).eq)
}

/// The longitudinal centre of buoyancy (m from the stern) at the two ends of the trim range, at heel 0:
/// at -10 degrees (bow up) and at +10 degrees (bow down), for the displacement of `mass_t`. The
/// furthest aft and forward the hull can put its LCB at that displacement (issue #549). The LCB
/// moves forward with trim, so the first is the lesser.
pub fn lcb_reach(
    table: &SectionsTable,
    mass_t: f64,
    density: f64,
) -> Result<(f64, f64), SectionsError> {
    let target = mass_t / density;
    if target.is_nan() || target <= 0.0 {
        return Err(SectionsError::NotAfloat);
    }
    let total = immersed(
        table,
        &Waterplane {
            heel_deg: 0.0,
            trim_deg: 0.0,
            c: f64::MAX / 4.0,
        },
    )
    .volume_m3;
    if target >= total {
        return Err(SectionsError::Sinks);
    }
    let mut w = Work::default();
    let mut at = |trim_deg: f64| {
        let c = c_for_volume(table, 0.0, trim_deg, target, total, &mut w);
        immersed(
            table,
            &Waterplane {
                heel_deg: 0.0,
                trim_deg,
                c,
            },
        )
        .lcb_m
    };
    Ok((at(-TRIM_LIMIT_DEG), at(TRIM_LIMIT_DEG)))
}

/// One trim tried by the outer solve.
struct Floating {
    eq: Equilibrium,
    /// B − G along the earth's forward horizontal.
    lever: f64,
}

impl Residual for Floating {
    fn residual(&self) -> f64 {
        self.lever
    }
}

/// The righting lever at one equilibrium: the horizontal distance across from G to the vertical
/// through B, positive when righting.
pub fn righting_lever(eq: &Equilibrium, g: Gravity) -> f64 {
    let (u, _) = earth_axes(&eq.waterplane);
    let im = &eq.immersed;
    dot((im.lcb_m, im.tcb_m, im.kb_m), u) - dot((g.lcg_m, g.tcg_m, g.kg_m), u)
}

/// The GZ curve with free trim: at each heel (degrees) the equilibrium, its righting lever, less the
/// free surface term `FSM / Δ × sin φ` (FSM in t·m).
pub fn gz_curve(
    table: &SectionsTable,
    mass_t: f64,
    g: Gravity,
    free_surface_moment_tm: f64,
    density: f64,
    angles: impl IntoIterator<Item = i32>,
) -> Result<Vec<(i32, f64)>, SectionsError> {
    gz_curve_from(
        table,
        mass_t,
        g,
        free_surface_moment_tm,
        density,
        angles,
        None,
    )
}

/// [`gz_curve`] with the equilibrium at heel 0 already solved by the caller (the free-trim float
/// of the same mass and G): the same numbers, one solve less.
pub fn gz_curve_from(
    table: &SectionsTable,
    mass_t: f64,
    g: Gravity,
    free_surface_moment_tm: f64,
    density: f64,
    angles: impl IntoIterator<Item = i32>,
    heel0: Option<&Equilibrium>,
) -> Result<Vec<(i32, f64)>, SectionsError> {
    let mut out = Vec::new();
    for deg in angles {
        out.push((
            deg,
            gz_at(
                table,
                mass_t,
                g,
                free_surface_moment_tm,
                density,
                deg,
                heel0,
            )?,
        ));
    }
    Ok(out)
}

/// The GZ with free trim at one heel (degrees), less the free surface term: one point of
/// [`gz_curve_from`].
pub fn gz_at(
    table: &SectionsTable,
    mass_t: f64,
    g: Gravity,
    free_surface_moment_tm: f64,
    density: f64,
    deg: i32,
    heel0: Option<&Equilibrium>,
) -> Result<f64, SectionsError> {
    let fsc = free_surface_moment_tm / mass_t;
    let heel = f64::from(deg);
    let lever = match heel0 {
        Some(eq) if deg == 0 => righting_lever(eq, g),
        _ => righting_lever(&equilibrium(table, mass_t, g, heel, density)?, g),
    };
    Ok(lever - fsc * sin(radians(heel)))
}
