//! The ShipD-style parametric hull (spec 0008, "ShipD-style hull", issue #452): a Sections Table
//! generator written for OVD from the definitions of the Ship-D hull parameterisation, with the
//! bow and stern bulbs (issue #453).
//!
//! Sources, read for this module: Bagazinski, N. J., and Ahmed, F., "Ship-D: Ship Hull Dataset for
//! Design Optimization Using Machine Learning", Proc. ASME IDETC/CIE 2023 (arXiv:2305.08279, v2);
//! the dataset, Bagazinski and Ahmed 2024, "Ship-D", Harvard Dataverse, doi:10.7910/DVN/MMGAUS, V1
//! (CC0 1.0): `HullParameterizationOverview.pdf` (the 45 parameters, their scaling and ranges) and
//! `HullParameterization_ConstraintsOverview.pdf` (the 49 constraints). No code is copied. The
//! dataset's `HullParameterization.py` was read only to settle definitions the two PDFs leave open;
//! each one is listed here with the function read and the reading taken:
//!
//! - Scaling (`__init__`): `Bd`, `Bc` and `Bc_trans` are full breadths, halved here; `BOW(A)` and
//!   `DELTA_BOW(A)` scale by `Lb / (2 Dd^2)` and their `B` terms by `Lb / (2 Dd)` (the PDF says "by
//!   Lbb and Dd"; the reading is `Lb`); the stern `DELTA` terms likewise by `Ls`; `DRIFT(A)` by
//!   `60 / Dd^2` and `DRIFT(B)` by `60 / Dd`, `DRIFT(C)` in degrees; `Rk_trans` is a fraction of
//!   `Dd (1 - SK)` with `SK` still the fraction.
//! - Midship section (`GenCrossSection`, `halfBeam_MidBody`): a keel arc of radius `Rk` (centre on
//!   the centre line above the keel for `Rk > 0`, on the base line outboard for `Rk < 0`), the bottom
//!   line tangent to it at the deadrise, the chine point at half-breadth `Bc` on that line (its
//!   height is `Dc`), a straight side from the chine point to the deck edge, and a chine arc of
//!   radius `Rc` tangent to the bottom and the side. The transom section is the same with its own
//!   terms, raised by `SK_z`, to the deck edge at `Bs`.
//! - Bow (`GenBowForm`, `bowrise`, `keelrise_bow`, `delta_bow`, `drift`, `solve_waterline_bow`):
//!   the bow rake `x = A z^2 + B z + C` with `C` such that its least x over the rake is 0; the keel
//!   rise `x = Kappa_BOW Lb - sqrt(z / K)` through the rake at `BK_z`; `delta_bow(z) = Lb + A z^2 +
//!   B z + C` with `C` such that its largest offset is 0; at each height the half-breadth is the
//!   cubic from the stem (0 at the bow profile, slope `tan drift`) to the midship half-breadth at
//!   `delta_bow` (slope 0). The PDF's "scale applied twice" is in the vertex test of the bow rake:
//!   the vertex counts when it lies between `BK_z Dd` (the height scaled a second time) and `Dd`.
//!   This module keeps that test as written, since the dataset's hulls were made with it.
//! - Stern (`GenSternForm`, `transom`, `sternrise`, `delta_stern`, `halfBeam_Transom`,
//!   `solve_waterline_stern`): the transom `x = A z + B` with `B` such that its largest x is LOA;
//!   the stern rise `x = Lb + Lm + Kappa_STERN Ls + sqrt(z / K)` through the transom at `SK_z`;
//!   `delta_stern` with `C` such that its smallest offset is 0; at each height, from `delta_stern`
//!   (the midship half-breadth, slope 0) to the stern profile (the transom half-breadth), a quarter
//!   ellipse (`bit_EP_* = 1`) or a parabola (`= 0`), `bit_EP_T` above `SK_z` and `bit_EP_S` below.
//!   Below `SK_z` the transom half-breadth is 0.
//! - Constraints (`input_Constraints` and the four functions it joins): the values `g`, satisfied
//!   when `g <= 0`, in the PDF's order. The PDF writes "> 1" for "is strictly positive" (indices 3
//!   to 5 and 31 to 33); the reading is "> 0" (`g = -Rc`, and so on). Index 1 compares `WL` with `Dd`
//!   as lengths. The thirteen bulb constraints (36 to 48) are -1 when there is no bulb.
//! - Bulbs (`GenBulbForms`, `BB_profile`, `halfBeam_BB`, `SB_profile`, `halfBeam_SB`,
//!   `gen_waterline_bow_BB`, `gen_waterline_stern_SB`, `BulbformConstraints`): each bulb is an
//!   ellipsoid of its own. Its widest part is at the height `Hbb WL` (`Hsb HSBOA WL` for the stern
//!   bulb), where the half-width is `Bbb` (`Bsb`) of the midship half-breadth at that height (the
//!   overview PDF's reading; the code's comment says "of Bd at the FP"). Above that height the
//!   vertical radius is the rest of the design draught (`(1 - Hbb) WL`), below it the height itself,
//!   so the bulb closes at the keel line and at `WL` (`HSBOA WL`). It leads the bow `Lbb LOA` forward
//!   of the stem at the design draught, with its widest part `Lbbm` of that length from the front
//!   (`Lbbm` negative puts it behind the stem). The stern bulb starts at `Kappa_SB Ls` aft of the
//!   start of the stern taper: the code and its comment take `Ls`, the overview PDF says "of Lb";
//!   the reading is `Ls`, since the bulb must start on the stern taper. At each height below the
//!   design draught the waterline runs along the bulb to the half-width the hull's own waterline
//!   reaches at the bulb's widest part, with a quartic fillet between: it starts and ends where the
//!   fraction `Rbb / 2` (`Rsb / 2`) of the shorter of the two distances lies each side of the
//!   crossing, matches value and slope to the bulb and to the hull at its two ends, and has no
//!   curvature at the bulb end of the bow fillet and at the bulb end of the stern fillet. The
//!   dataset's code writes that last row with `6 x^2` where the second derivative has `6 x`; the row
//!   is written here from the definition of the curvature, and the hulls where the difference shows
//!   are in the benchmark report. Where the bulb is wider than the hull at a height, so that the two
//!   waterlines never meet, the waterline is the wider of the hull's and the bulb's narrowed to the
//!   hull's half-breadth.
//! - Two more places where the code departs from the definitions, kept as a [`Reading`] for the
//!   benchmark only (issue #648; every Design takes the definitions): the stern ellipse
//!   (`halfBeam_Stern` squares its vertical radius, so where the transom is wider than the midbody
//!   the ellipse is mirrored) and the bow bulb's crossing with the hull (`gen_waterline_bow_BB` takes
//!   the smallest root of the bow cubic, real or complex, aft of the stem and of the bulb's front;
//!   numpy orders a complex number by its real part first, so the crossing may be a complex root's
//!   real part or lie aft of `delta_bow`).
//! - The bulb constraints (36 to 48): the index order follows the constraints PDF; for the three
//!   "forward of `DELTA_BOW`" rows the code's order is at the base, at the design draught, at the
//!   vertex, the PDF's is at the base, at the vertex, at the design draught. All three must hold, so
//!   only the label differs. A bulb needs a zero deadrise or a positive keel radius (rows 36 to 38
//!   and 42 to 44 are 1, a failure, otherwise): the dataset's code does this by branch.
//!
//! Axes: the dataset's x runs aft from the stem; OVD's runs forward from the aft end, so a
//! Section's x is `L - x_ShipD`.

use crate::sections::{Section, SectionsError, SectionsTable};
use alloc::vec::Vec;
use core::f64::consts::PI;
use core::fmt;
use libm::{acos, cbrt, cos, fabs, sin, sqrt, tan};

/// The dataset's header names of the 45 parameters, in its column order.
pub const NAMES: [&str; 45] = [
    "LOA",
    "Lb",
    "Ls",
    "Bd",
    "Dd",
    "Bs",
    "WL",
    "Bc",
    "Beta",
    "Rc",
    "Rk",
    "Abow",
    "Bbow",
    "BK_z",
    "Kappa_bow",
    "Adel bow",
    "Bdel bow",
    "Adrft",
    "Bdrft",
    "Cdrft",
    "bit_EP_S",
    "bit_EP_T",
    "Atrans",
    "SK_z",
    "Kappa_stern",
    "Adel stern",
    "Bdel stern",
    "Beta trans",
    "Bc trans",
    "Rc Trans",
    "Rk trans",
    "bit_BB",
    "bit_SB",
    "Lbb",
    "Hbb",
    "Bbb",
    "Lbbm",
    "Rbb",
    "Kappa_SB",
    "Lsb",
    "HSBOA",
    "Hsb",
    "Bsb",
    "Lsbm",
    "Rsb",
];

/// The names of the 49 constraints, in the dataset's index order.
pub const CONSTRAINTS: [&str; 49] = [
    "bow and stern tapers within LOA",
    "design draught below the deck",
    "side meets the chine fillet above the chine",
    "chine radius positive",
    "chine half-breadth positive",
    "chine height positive",
    "chine fillet meets the bottom inboard of the chine",
    "keel fillet inboard of the chine fillet",
    "keel radius not 0",
    "bow keel rise starts aft of the midship breadth at the base",
    "drift angle below 90 degrees at the base",
    "drift angle 0 or more at the base",
    "drift angle below 90 degrees at the deck",
    "drift angle 0 or more at the deck",
    "drift angle below 90 degrees at its vertex",
    "drift angle 0 or more at its vertex",
    "bow-keel intersection at x 0 or more",
    "bow-keel intersection forward of the keel rise start",
    "bow-keel intersection height 0 or more",
    "bow-keel intersection height at most the depth",
    "bow taper length positive at the deck",
    "bow taper length positive at the bow-keel intersection",
    "bow taper length positive at the midship-breadth vertex",
    "bow taper length positive at the bow rake vertex",
    "stern rise starts aft of the midship breadth at the base",
    "stern taper positive at the stern-keel intersection",
    "stern taper positive at the midship-breadth vertex",
    "stern taper positive at the deck",
    "stern-keel intersection aft of the stern rise start",
    "transom chine inside the midship section",
    "transom side meets the chine fillet above the chine",
    "transom chine radius positive",
    "transom chine half-breadth positive",
    "transom chine height positive",
    "transom chine fillet meets the bottom inboard of the chine",
    "transom keel fillet inboard of the chine fillet",
    "bulbous bow lower radius below the keel radius",
    "bulbous bow length radius below the keel radius",
    "bulbous bow inside the hull section",
    "bulbous bow forward of the midship breadth at the base",
    "bulbous bow forward of the midship breadth at its vertex",
    "bulbous bow forward of the midship breadth at the design draught",
    "stern bulb lower radius below the keel radius",
    "stern bulb length radius below the keel radius",
    "stern bulb inside the hull section",
    "stern bulb height below the stern-keel intersection",
    "stern bulb aft of the midship breadth at the base",
    "stern bulb aft of the midship breadth at its height",
    "stern bulb aft of the midship breadth at its vertex",
];

/// The 45 ShipD parameters in the dataset's non-dimensional form, in [`NAMES`] order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShipdParameters(pub [f64; 45]);

impl ShipdParameters {
    /// The value of the parameter of this header name.
    pub fn get(&self, name: &str) -> Option<f64> {
        NAMES.iter().position(|n| *n == name).map(|i| self.0[i])
    }
}

/// One failed constraint: its index (0 to 48), its name and its value `g > 0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConstraintFailure {
    pub index: usize,
    pub name: &'static str,
    pub g: f64,
}

/// Why a ShipD hull cannot write its sections.
#[derive(Debug, Clone, PartialEq)]
pub enum ShipdError {
    /// The parameters fail these constraints.
    Constraints(Vec<ConstraintFailure>),
    /// Length, beam or depth is not a positive finite number, or a parameter is not finite.
    NotFinite,
    /// The sections the generator wrote were refused.
    Sections(SectionsError),
}

impl fmt::Display for ShipdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShipdError::Constraints(c) => {
                write!(f, "the ShipD parameters fail")?;
                for (i, x) in c.iter().enumerate() {
                    let sep = if i == 0 { " " } else { "; " };
                    write!(f, "{sep}constraint {} ({})", x.index, x.name)?;
                }
                Ok(())
            }
            ShipdError::NotFinite => {
                write!(
                    f,
                    "the ShipD hull needs finite parameters and a positive length, beam and depth"
                )
            }
            ShipdError::Sections(e) => write!(f, "{e}"),
        }
    }
}

/// A half-section built of a keel arc, a bottom line at the deadrise, a chine arc and a straight
/// side, in (y, z) with z from the base. `z0` is the height of its keel point (0 at midship, `SK_z`
/// at the transom).
#[derive(Debug, Clone, Copy)]
struct CrossSection {
    rk: f64,
    keel_centre: (f64, f64),
    /// The keel arc meets the bottom line.
    keel_tangent: (f64, f64),
    /// The bottom line: through `keel_tangent`, rising at the deadrise.
    tan_beta: f64,
    /// The chine point and the deck edge.
    chine: (f64, f64),
    deck: (f64, f64),
    rc: f64,
    chine_centre: (f64, f64),
    /// The chine arc meets the bottom line and the side.
    chine_bottom: (f64, f64),
    chine_side: (f64, f64),
}

impl CrossSection {
    fn new(z0: f64, rk: f64, beta_deg: f64, bc: f64, rc: f64, deck: (f64, f64)) -> Self {
        let beta = beta_deg * PI / 180.0;
        let keel_centre = if rk > 0.0 { (0.0, z0 + rk) } else { (-rk, z0) };
        let keel_tangent = (
            keel_centre.0 + rk * sin(beta),
            keel_centre.1 - rk * cos(beta),
        );
        let tan_beta = tan(beta);
        let chine = (bc, keel_tangent.1 + (bc - keel_tangent.0) * tan_beta);
        // The chine arc: its centre is at `rc` from the bottom line and from the side, inboard of
        // both.
        let (ub_y, ub_z) = (cos(beta), sin(beta));
        let (dy, dz) = (deck.0 - chine.0, deck.1 - chine.1);
        let len = sqrt(dy * dy + dz * dz);
        let (us_y, us_z) = (dy / len, dz / len);
        let (nb_y, nb_z) = (-ub_z, ub_y);
        let (ns_y, ns_z) = (-us_z, us_y);
        // Points on the two offset lines: chine + rc n; solve chine + rc nb + s ub = chine + rc ns
        // + t us for s.
        let (ry, rz) = (rc * (ns_y - nb_y), rc * (ns_z - nb_z));
        let det = ub_y * (-us_z) - ub_z * (-us_y);
        let s = (ry * (-us_z) - rz * (-us_y)) / det;
        let chine_centre = (
            chine.0 + rc * nb_y + s * ub_y,
            chine.1 + rc * nb_z + s * ub_z,
        );
        let chine_bottom = (chine_centre.0 - rc * nb_y, chine_centre.1 - rc * nb_z);
        let chine_side = (chine_centre.0 - rc * ns_y, chine_centre.1 - rc * ns_z);
        Self {
            rk,
            keel_centre,
            keel_tangent,
            tan_beta,
            chine,
            deck,
            rc,
            chine_centre,
            chine_bottom,
            chine_side,
        }
    }

    /// The half-breadth at height `z`, from the keel point to the deck.
    fn half_breadth(&self, z: f64) -> f64 {
        if z < self.keel_tangent.1 {
            let s = if self.rk >= 0.0 { 1.0 } else { -1.0 };
            let d = z - self.keel_centre.1;
            s * sqrt(fabs(self.rk * self.rk - d * d)) + self.keel_centre.0
        } else if z < self.chine_bottom.1 {
            self.keel_tangent.0 + (z - self.keel_tangent.1) / self.tan_beta
        } else if z < self.chine_side.1 {
            let d = z - self.chine_centre.1;
            sqrt(fabs(self.rc * self.rc - d * d)) + self.chine_centre.0
        } else {
            let (c, e) = (self.chine, self.deck);
            c.0 + (z - c.1) * (e.0 - c.0) / (e.1 - c.1)
        }
    }
}

/// A quadratic `a z^2 + b z` and the vertex height it may have.
fn vertex(a: f64, b: f64) -> f64 {
    if a == 0.0 { -1.0 } else { -b / (2.0 * a) }
}

/// A bulb's ellipsoid, bow or stern (`GenBulbForms`): its widest part is at height `rz_lo`, its
/// vertical radius is `rz_up` above that and `rz_lo` below it.
#[derive(Debug, Clone, Copy)]
struct Bulb {
    /// Where the bulb leaves the hull: the stem at the design draught (bow), `SBs` (stern).
    start: f64,
    rz_up: f64,
    rz_lo: f64,
    /// The half-width of its widest part.
    ry: f64,
    /// The x radius of the end ellipse, and the x of the widest part.
    rx: f64,
    cx: f64,
    /// The fillet radius, as a fraction.
    fillet: f64,
    /// The bulb exists below this height.
    top: f64,
}

impl Bulb {
    fn root(&self, z: f64) -> f64 {
        let r = if z >= self.rz_lo {
            self.rz_up
        } else {
            self.rz_lo
        };
        if r <= 0.0 {
            return 0.0;
        }
        let t = (z - self.rz_lo) / r;
        sqrt(fabs(1.0 - t * t))
    }

    /// The x radius of the end ellipse at height `z`.
    fn rx_at(&self, z: f64) -> f64 {
        self.root(z) * self.rx
    }

    /// The half-width of the widest part at height `z`.
    fn ry_at(&self, z: f64) -> f64 {
        self.root(z) * self.ry
    }

    /// The half-width at `x`; `aft` is true for the stern bulb, whose end ellipse is aft of `cx`.
    fn half_width(&self, z: f64, x: f64, aft: bool) -> f64 {
        let (ry, rx) = (self.ry_at(z), self.rx_at(z));
        if (aft && x <= self.cx) || (!aft && x >= self.cx) || rx <= 0.0 {
            return ry;
        }
        let t = (x - self.cx) / rx;
        sqrt(fabs(1.0 - t * t)) * ry
    }

    /// The slope `dy/dx` of the end ellipse at `x`.
    fn slope(&self, z: f64, x: f64, aft: bool) -> f64 {
        let (ry, rx) = (self.ry_at(z), self.rx_at(z));
        if (aft && x <= self.cx) || (!aft && x >= self.cx) || rx <= 0.0 {
            return 0.0;
        }
        let t = (x - self.cx) / rx;
        -ry * t / (rx * sqrt(fabs(1.0 - t * t)))
    }
}

/// The quartic `c0 + c1 t + c2 t^2 + c3 t^3 + c4 t^4` that starts at `(0, ya)` with slope `da` and no
/// curvature, and ends at `(h, yb)` with slope `db`.
fn quartic(ya: f64, da: f64, yb: f64, db: f64, h: f64) -> [f64; 5] {
    let a = yb - ya - da * h;
    let b = db - da;
    [
        ya,
        da,
        0.0,
        (4.0 * a - h * b) / (h * h * h),
        (h * b - 3.0 * a) / (h * h * h * h),
    ]
}

fn quartic_at(c: &[f64; 5], t: f64) -> f64 {
    c[0] + t * (c[1] + t * (c[2] + t * (c[3] + t * c[4])))
}

/// The smallest x in `[lo, hi]` where `f` is 0: the first sign change on an even grid, then
/// bisection.
fn first_root(f: impl Fn(f64) -> f64, lo: f64, hi: f64) -> Option<f64> {
    const N: u32 = 256;
    let (mut xa, mut fa) = (lo, f(lo));
    if fa == 0.0 {
        return Some(lo);
    }
    for i in 1..=N {
        let xb = lo + (hi - lo) * f64::from(i) / f64::from(N);
        let fb = f(xb);
        if fb == 0.0 || (fa < 0.0) != (fb < 0.0) {
            let (mut a, mut b, mut fa) = (xa, xb, fa);
            for _ in 0..60 {
                let m = 0.5 * (a + b);
                let fm = f(m);
                if (fm < 0.0) == (fa < 0.0) {
                    a = m;
                    fa = fm;
                } else {
                    b = m;
                }
            }
            return Some(0.5 * (a + b));
        }
        (xa, fa) = (xb, fb);
    }
    None
}

/// The fillet between a bulb and the hull at one height: bulb to `xr0`, the quartic to `xr1`, the
/// hull from there. The coefficients are in `t = x - xr0` (bow) or `t = xr1 - x` (stern).
#[derive(Debug, Clone, Copy)]
struct Fillet {
    xr0: f64,
    xr1: f64,
    coef: [f64; 5],
    /// The coefficients are in x itself (the dataset's row), not in the local `t`.
    absolute: bool,
}

impl Fillet {
    fn at(&self, x: f64, bow: bool) -> f64 {
        if self.absolute {
            quartic_at(&self.coef, x)
        } else if bow {
            quartic_at(&self.coef, x - self.xr0)
        } else {
            quartic_at(&self.coef, self.xr1 - x)
        }
    }
}

/// Which curvature row closes the fillet's quartic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilletRow {
    /// No curvature at the bulb end: the second derivative `12 a x^2 + 6 b x + 2 c`.
    Definition,
    /// The dataset's code as written: `12 a x^2 + 6 b x^2 + 2 c` at the same end. For the
    /// benchmark report only: it shows which hulls the difference reaches.
    DatasetCode,
}

/// Which reading the generator takes at a place where the dataset's code does not do what the
/// definitions say. (The bow-rake vertex test is not one of them: the module keeps it as the code
/// has it, since the dataset's hulls were made with it.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reading {
    /// The definitions. Every Design takes this.
    Definition,
    /// The dataset's code as written. For the benchmark only: it reproduces the dataset's values
    /// where the code departs from the definitions, so a hull it reaches is shown to be the
    /// dataset's, not OVD's.
    DatasetCode,
}

/// The real and imaginary parts of the roots of `a s^3 + b s^2 + c s + d` (a quadratic or a line
/// when the leading coefficients are 0).
fn cubic_roots(a: f64, b: f64, c: f64, d: f64) -> Vec<(f64, f64)> {
    let scale = fabs(a) + fabs(b) + fabs(c) + fabs(d);
    if fabs(a) <= 1e-14 * scale {
        if fabs(b) <= 1e-14 * scale {
            return if c == 0.0 {
                Vec::new()
            } else {
                alloc::vec![(-d / c, 0.0)]
            };
        }
        let disc = c * c - 4.0 * b * d;
        return if disc >= 0.0 {
            let r = sqrt(disc);
            alloc::vec![((-c + r) / (2.0 * b), 0.0), ((-c - r) / (2.0 * b), 0.0)]
        } else {
            let im = sqrt(-disc) / (2.0 * fabs(b));
            let re = -c / (2.0 * b);
            alloc::vec![(re, im), (re, -im)]
        };
    }
    // Depressed: s = t - B / 3, t^3 + p t + q = 0.
    let (bb, cc, dd) = (b / a, c / a, d / a);
    let shift = bb / 3.0;
    let p = cc - bb * bb / 3.0;
    let q = 2.0 * bb * bb * bb / 27.0 - bb * cc / 3.0 + dd;
    let disc = q * q / 4.0 + p * p * p / 27.0;
    if disc > 0.0 {
        let r = sqrt(disc);
        let (u, v) = (cbrt(-q / 2.0 + r), cbrt(-q / 2.0 - r));
        let re = -(u + v) / 2.0 - shift;
        let im = sqrt(3.0) / 2.0 * (u - v);
        alloc::vec![(u + v - shift, 0.0), (re, im), (re, -im)]
    } else if p == 0.0 {
        alloc::vec![(-shift, 0.0); 3]
    } else {
        let m = 2.0 * sqrt(-p / 3.0);
        let theta = acos((3.0 * q / (p * m)).clamp(-1.0, 1.0)) / 3.0;
        (0..3)
            .map(|k| (m * cos(theta - 2.0 * PI * f64::from(k) / 3.0) - shift, 0.0))
            .collect()
    }
}

/// The quartic in x through `(x0, y0)` and `(x1, y1)` with slopes `d0`, `d1` and the dataset's
/// curvature row at `xc` (`6 x^2` where the second derivative has `6 x`): the five-by-five system
/// solved by elimination with row pivoting.
fn quartic_dataset_row(x0: f64, y0: f64, d0: f64, x1: f64, y1: f64, d1: f64, xc: f64) -> [f64; 5] {
    let (c0, c1) = (x0 * x0 * x0, x1 * x1 * x1);
    let mut m = [
        [c0 * x0, c0, x0 * x0, x0, 1.0, y0],
        [c1 * x1, c1, x1 * x1, x1, 1.0, y1],
        [4.0 * c0, 3.0 * x0 * x0, 2.0 * x0, 1.0, 0.0, d0],
        [4.0 * c1, 3.0 * x1 * x1, 2.0 * x1, 1.0, 0.0, d1],
        [12.0 * xc * xc, 6.0 * xc * xc, 2.0, 0.0, 0.0, 0.0],
    ];
    for c in 0..5 {
        let mut p = c;
        for r in c + 1..5 {
            if fabs(m[r][c]) > fabs(m[p][c]) {
                p = r;
            }
        }
        m.swap(c, p);
        let pivot = m[c];
        for row in m.iter_mut().skip(c + 1) {
            let f = row[c] / pivot[c];
            for (v, p) in row.iter_mut().zip(pivot).skip(c) {
                *v -= f * p;
            }
        }
    }
    let mut a = [0.0; 5];
    for c in (0..5).rev() {
        let mut v = m[c][5];
        for k in c + 1..5 {
            v -= m[c][k] * a[k];
        }
        a[c] = v / m[c][c];
    }
    // a is [x^4 .. 1]: the highest power first; `quartic_at` takes the lowest first.
    [a[4], a[3], a[2], a[1], a[0]]
}

/// The waterline of the hull at one height: where it starts and ends, the midbody half-breadth and
/// the fillets of the bulbs at that height.
#[derive(Debug, Clone)]
pub struct Waterline {
    z: f64,
    start: f64,
    end: f64,
    y_mid: f64,
    bow: BowLine,
    stern: SternLine,
    stern_ellipse: Reading,
}

#[derive(Debug, Clone, Copy)]
enum BowLine {
    /// No bulb at this height.
    Plain,
    /// The bulb and the hull meet: a fillet between.
    Filleted(Fillet),
    /// The bulb is wider than the hull at every x: the wider of the hull's and the narrowed bulb's.
    Wider,
}

#[derive(Debug, Clone, Copy)]
enum SternLine {
    Plain,
    Filleted(Fillet),
}

/// The ShipD hull at its dimensions (m): every curve of the definitions, in the dataset's x (aft from
/// the stem).
#[derive(Debug, Clone)]
pub struct Shape {
    pub loa: f64,
    lb: f64,
    ls: f64,
    lm: f64,
    dd: f64,
    wl: f64,
    bc: f64,
    rc: f64,
    rk: f64,
    bow: [f64; 3],
    bk: (f64, f64),
    kappa_bow: f64,
    keel_bow: f64,
    delta_bow: [f64; 3],
    drift: [f64; 3],
    ep_s: bool,
    ep_t: bool,
    trans: [f64; 2],
    sk: (f64, f64),
    kappa_stern: f64,
    sternrise: f64,
    delta_stern: [f64; 3],
    bc_trans: f64,
    rc_trans: f64,
    mid: CrossSection,
    transom: CrossSection,
    beta: f64,
    bb: Option<Bulb>,
    sb: Option<Bulb>,
}

impl Shape {
    /// The shape of `p` at length `loa`, half the beam `beam / 2` at the deck and depth `depth`
    /// (the dataset's `LOA`, `Bd` and `Dd`: in a Design they come from the Option's main
    /// dimensions, not from `p`).
    pub fn new(p: &ShipdParameters, loa: f64, beam: f64, depth: f64) -> Self {
        let v = &p.0;
        let (lb, ls) = (v[1] * loa, v[2] * loa);
        let bd = beam / 2.0;
        let dd = depth;
        let bs = v[5] * bd;
        let bc = v[7] / 2.0 * loa;
        let rc = v[9] * bc;
        let rk = v[10] * dd;
        let mid = CrossSection::new(0.0, rk, v[8], bc, rc, (bd, dd));
        let mut bow = [v[11] * 0.5 * lb / (dd * dd), v[12] * 0.5 * lb / dd, 0.0];
        let bk_z = v[13] * dd;
        // The least x of the bow rake is 0 (the vertex test scales `BK_z` twice; see the header).
        let zv = vertex(bow[0], bow[1]);
        let at = |z: f64| bow[0] * z * z + bow[1] * z;
        let mut least = at(dd).min(at(bk_z));
        if zv >= bk_z * dd && zv <= dd {
            least = least.min(at(zv));
        }
        bow[2] = -least;
        let kappa_bow = v[14];
        let bk = (bow[0] * bk_z * bk_z + bow[1] * bk_z + bow[2], bk_z);
        let keel_bow = bk.1 / ((bk.0 - kappa_bow * lb) * (bk.0 - kappa_bow * lb));
        let mut delta_bow = [v[15] * 0.5 * lb / (dd * dd), v[16] * 0.5 * lb / dd, 0.0];
        let zv = vertex(delta_bow[0], delta_bow[1]);
        let at = |z: f64| delta_bow[0] * z * z + delta_bow[1] * z;
        let mut most = at(dd).max(0.0);
        if (0.0..=dd).contains(&zv) {
            most = most.max(at(zv));
        }
        delta_bow[2] = -most;
        let drift = [v[17] * 60.0 / (dd * dd), v[18] * 60.0 / dd, v[19]];
        let lm = loa - ls - lb;
        let sk_z = v[23] * dd;
        let a_trans = v[22];
        let b_trans = if a_trans >= 0.0 {
            loa - a_trans * dd
        } else {
            loa - a_trans * sk_z
        };
        let sk = (a_trans * sk_z + b_trans, sk_z);
        let kappa_stern = v[24];
        let start = lb + lm + ls * kappa_stern;
        let sternrise = sk.1 / ((sk.0 - start) * (sk.0 - start));
        let mut delta_stern = [v[25] * 0.5 * ls / (dd * dd), v[26] * 0.5 * ls / dd, 0.0];
        let zv = vertex(delta_stern[0], delta_stern[1]);
        let at = |z: f64| delta_stern[0] * z * z + delta_stern[1] * z;
        let mut least = at(dd).min(0.0);
        if (0.0..=dd).contains(&zv) {
            least = least.min(at(zv));
        }
        delta_stern[2] = -least;
        let bc_trans = v[28] / 2.0 * loa;
        let rc_trans = v[29] * bc_trans;
        let rk_trans = v[30] * dd * (1.0 - v[23]);
        let transom = CrossSection::new(sk_z, rk_trans, v[27], bc_trans, rc_trans, (bs, dd));
        let mut shape = Self {
            loa,
            lb,
            ls,
            lm,
            dd,
            wl: v[6] * dd,
            bc,
            rc,
            rk,
            bow,
            bk,
            kappa_bow,
            keel_bow,
            delta_bow,
            drift,
            ep_s: v[20] != 0.0,
            ep_t: v[21] != 0.0,
            trans: [a_trans, b_trans],
            sk,
            kappa_stern,
            sternrise,
            delta_stern,
            bc_trans,
            rc_trans,
            mid,
            transom,
            beta: v[8],
            bb: None,
            sb: None,
        };
        let wl = shape.wl;
        if v[31] != 0.0 {
            let fp = shape.bow_profile(wl);
            let zc = v[34] * wl;
            shape.bb = Some(Bulb {
                start: fp,
                rz_up: (1.0 - v[34]) * wl,
                rz_lo: zc,
                ry: shape.mid.half_breadth(zc) * v[35],
                rx: v[33] * loa * (1.0 - v[36]),
                cx: fp - loa * v[33] * v[36],
                fillet: v[37],
                top: wl,
            });
        }
        if v[32] != 0.0 {
            let start = v[38] * ls + shape.lm + shape.lb;
            let top = wl * v[40];
            let zc = v[41] * top;
            shape.sb = Some(Bulb {
                start,
                rz_up: (1.0 - v[41]) * top,
                rz_lo: zc,
                ry: shape.mid.half_breadth(zc) * v[42],
                rx: v[39] * loa * (1.0 - v[43]),
                cx: start + loa * v[39] * v[43],
                fillet: v[44],
                top,
            });
        }
        shape
    }

    fn bowrise(&self, z: f64) -> f64 {
        self.bow[0] * z * z + self.bow[1] * z + self.bow[2]
    }

    fn keelrise_bow(&self, z: f64) -> f64 {
        -sqrt(z / self.keel_bow) + self.kappa_bow * self.lb
    }

    /// The x of the stem at height `z`: the keel rise below `BK_z`, the bow rake above.
    pub fn bow_profile(&self, z: f64) -> f64 {
        if z <= self.bk.1 {
            self.keelrise_bow(z)
        } else {
            self.bowrise(z)
        }
    }

    fn delta_bow(&self, z: f64) -> f64 {
        self.lb + self.delta_bow[0] * z * z + self.delta_bow[1] * z + self.delta_bow[2]
    }

    fn drift(&self, z: f64) -> f64 {
        PI * (self.drift[0] * z * z + self.drift[1] * z + self.drift[2]) / 180.0
    }

    fn transom_x(&self, z: f64) -> f64 {
        self.trans[0] * z + self.trans[1]
    }

    fn sternrise_x(&self, z: f64) -> f64 {
        sqrt(z / self.sternrise) + self.lb + self.lm + self.ls * self.kappa_stern
    }

    /// The x of the stern at height `z`: the stern rise below `SK_z`, the transom above (the start
    /// of the stern bulb, where it has one).
    pub fn stern_profile(&self, z: f64) -> f64 {
        // With a stern bulb the taper leads into the bulb: it ends where the bulb starts.
        if let Some(b) = &self.sb
            && z <= b.top
        {
            return b.start;
        }
        if z <= self.sk.1 {
            self.sternrise_x(z)
        } else {
            self.transom_x(z)
        }
    }

    fn delta_stern(&self, z: f64) -> f64 {
        self.lb
            + self.lm
            + self.delta_stern[0] * z * z
            + self.delta_stern[1] * z
            + self.delta_stern[2]
    }

    fn transom_half_breadth(&self, z: f64) -> f64 {
        if z <= self.sk.1 {
            0.0
        } else {
            self.transom.half_breadth(z)
        }
    }

    /// The cubic waterline of the bow at height `z`: from the stem (0, slope `tan drift`) to the
    /// midship half-breadth (slope 0) at `delta_bow`.
    fn bow_y(&self, x: f64, z: f64) -> f64 {
        let x_bow = self.bow_profile(z);
        let h = self.delta_bow(z) - x_bow;
        let s = (x - x_bow) / h;
        let m = tan(self.drift(z));
        self.mid.half_breadth(z) * s * s * (3.0 - 2.0 * s) + m * h * s * (1.0 - s) * (1.0 - s)
    }

    fn bow_dydx(&self, x: f64, z: f64) -> f64 {
        let x_bow = self.bow_profile(z);
        let h = self.delta_bow(z) - x_bow;
        let s = (x - x_bow) / h;
        let m = tan(self.drift(z));
        self.mid.half_breadth(z) * 6.0 * s * (1.0 - s) / h + m * (1.0 - s) * (1.0 - 3.0 * s)
    }

    /// Whether the stern taper at height `z` is an ellipse (else a parabola).
    fn stern_ellipse(&self, z: f64) -> bool {
        if z >= self.sk.1 { self.ep_t } else { self.ep_s }
    }

    /// The stern waterline at height `z`: from the midship half-breadth at `delta_stern` to the
    /// transom's, a quarter ellipse or a parabola.
    fn stern_y(&self, x: f64, z: f64, ellipse: Reading) -> f64 {
        let y_mid = self.mid.half_breadth(z);
        let y_t = self.transom_half_breadth(z);
        let d = self.delta_stern(z);
        let u = (x - d) / (self.stern_profile(z) - d);
        if self.stern_ellipse(z) {
            // The dataset's code squares the vertical radius inside the root, so where the transom
            // is wider than the midbody its ellipse is mirrored: it starts at `2 y_t - y_mid`, a
            // step out from the midbody, and falls to the transom.
            let ry = match ellipse {
                Reading::Definition => y_mid - y_t,
                Reading::DatasetCode => fabs(y_mid - y_t),
            };
            ry * sqrt(fabs(1.0 - u * u)) + y_t
        } else {
            y_mid + (y_t - y_mid) * u * u
        }
    }

    fn stern_dydx(&self, x: f64, z: f64) -> f64 {
        let y_mid = self.mid.half_breadth(z);
        let y_t = self.transom_half_breadth(z);
        let d = self.delta_stern(z);
        let span = self.stern_profile(z) - d;
        let u = (x - d) / span;
        if self.stern_ellipse(z) {
            -(y_mid - y_t) * u / (span * sqrt(fabs(1.0 - u * u)))
        } else {
            2.0 * (y_t - y_mid) * u / span
        }
    }

    /// The x on the stern waterline at height `z` where its half-breadth is `y`.
    fn stern_x_at(&self, y: f64, z: f64) -> f64 {
        let y_mid = self.mid.half_breadth(z);
        let y_t = self.transom_half_breadth(z);
        let d = self.delta_stern(z);
        let span = self.stern_profile(z) - d;
        let k = if y_mid > y_t {
            ((y - y_t) / (y_mid - y_t)).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let u = if self.stern_ellipse(z) {
            sqrt(1.0 - k * k)
        } else {
            sqrt(1.0 - k)
        };
        d + span * u
    }

    /// The waterline at height `z` (the bulbs, where there are bulbs below `z`, are in it).
    pub fn waterline(&self, z: f64, row: FilletRow) -> Waterline {
        self.waterline_with(z, row, Reading::Definition, Reading::Definition)
    }

    /// The dataset's code reading of where a bow bulb meets the hull at height `z`: the smallest
    /// real part among the roots of the bow cubic less `y_b`, real or complex, that the code's
    /// comparison puts at or aft of both the stem and the bulb's front `front`. (numpy orders a
    /// complex number by its real part first, so a complex root passes on its real part, and a root
    /// aft of `delta_bow` is kept.)
    fn bow_crossing_dataset(&self, z: f64, y_b: f64, front: f64) -> Option<f64> {
        let x_bow = self.bow_profile(z);
        let h = self.delta_bow(z) - x_bow;
        let (y, mh) = (self.mid.half_breadth(z), tan(self.drift(z)) * h);
        // `bow_y` in s = (x - x_bow) / h: (mh - 2 y) s^3 + (3 y - 2 mh) s^2 + mh s.
        let lo = x_bow.max(front);
        cubic_roots(mh - 2.0 * y, 3.0 * y - 2.0 * mh, mh, -y_b)
            .into_iter()
            .map(|(re, im)| (x_bow + h * re, h * im))
            .filter(|&(re, im)| re > lo || (re == lo && im >= 0.0))
            .map(|(re, _)| re)
            .reduce(f64::min)
    }

    /// [`Shape::waterline`] with a [`Reading`] for the stern ellipse and for the bow bulb's
    /// crossing with the hull.
    pub fn waterline_with(
        &self,
        z: f64,
        row: FilletRow,
        stern_ellipse: Reading,
        bulb_crossing: Reading,
    ) -> Waterline {
        let y_mid = self.mid.half_breadth(z);
        let bb = self.bb.filter(|b| z > 0.0 && z < b.top);
        let sb = self.sb.filter(|b| z > 0.0 && z < b.top);
        let start = match &bb {
            Some(b) => b.cx - b.rx_at(z),
            None => self.bow_profile(z),
        };
        let end = match &sb {
            Some(b) => b.cx + b.rx_at(z),
            None => self.stern_profile(z),
        };
        let bow = match &bb {
            None => BowLine::Plain,
            Some(b) => {
                let x2 = self.delta_bow(z);
                let y_b = b.ry_at(z);
                let lo = self.bow_profile(z).max(start);
                let crossing = match bulb_crossing {
                    Reading::Definition => first_root(|x| self.bow_y(x, z) - y_b, lo, x2),
                    Reading::DatasetCode => self
                        .bow_crossing_dataset(z, y_b, start)
                        .or_else(|| first_root(|x| self.bow_y(x, z) - y_b, lo, x2)),
                };
                match crossing {
                    None => BowLine::Wider,
                    Some(x_int) => {
                        let dx = fabs((x_int - start).min(x2 - x_int));
                        let (xr0, xr1) = (x_int - dx * b.fillet / 2.0, x_int + dx * b.fillet / 2.0);
                        let h = xr1 - xr0;
                        let (y0, d0) = (b.half_width(z, xr0, false), b.slope(z, xr0, false));
                        let (y1, d1) = (self.bow_y(xr1, z), self.bow_dydx(xr1, z));
                        let (coef, absolute) = if h <= 0.0 {
                            ([0.0; 5], false)
                        } else if row == FilletRow::DatasetCode {
                            (quartic_dataset_row(xr0, y0, d0, xr1, y1, d1, xr0), true)
                        } else {
                            (quartic(y0, d0, y1, d1, h), false)
                        };
                        BowLine::Filleted(Fillet {
                            xr0,
                            xr1,
                            coef,
                            absolute,
                        })
                    }
                }
            }
        };
        let stern = match &sb {
            None => SternLine::Plain,
            Some(b) => {
                let x1 = self.delta_stern(z);
                let y_b = b.ry_at(z);
                let x_int = if y_b >= y_mid {
                    b.start
                } else {
                    self.stern_x_at(y_b, z)
                };
                let dx = fabs((x_int - x1).min(end - x_int));
                let (xr0, xr1) = (x_int - dx * b.fillet / 2.0, x_int + dx * b.fillet / 2.0);
                let h = xr1 - xr0;
                let (y0, d0) = (self.stern_y(xr0, z, stern_ellipse), self.stern_dydx(xr0, z));
                let (y1, d1) = (b.half_width(z, xr1, true), b.slope(z, xr1, true));
                let (coef, absolute) = if h <= 0.0 {
                    ([0.0; 5], false)
                } else if row == FilletRow::DatasetCode {
                    (quartic_dataset_row(xr0, y0, d0, xr1, y1, d1, xr1), true)
                } else {
                    // In `t = xr1 - x`, running from the bulb to the hull.
                    (quartic(y1, -d1, y0, -d0, h), false)
                };
                SternLine::Filleted(Fillet {
                    xr0,
                    xr1,
                    coef,
                    absolute,
                })
            }
        };
        Waterline {
            z,
            start,
            end,
            y_mid,
            bow,
            stern,
            stern_ellipse,
        }
    }

    /// The half-breadth at (x_ShipD, z); 0 outside the hull.
    pub fn half_breadth(&self, x: f64, z: f64) -> f64 {
        if !(0.0..=self.dd).contains(&z) {
            return 0.0;
        }
        self.waterline(z, FilletRow::Definition)
            .half_breadth(self, x)
    }

    /// The 49 constraint values `g` (satisfied when `g <= 0`), in the dataset's order.
    pub fn constraints(&self) -> [f64; 49] {
        let mut g = [-1.0; 49];
        let dd = self.dd;
        g[0] = -self.loa + self.ls + self.lb;
        g[1] = self.wl - dd;
        let m = &self.mid;
        g[2] = -m.chine_side.1 + m.chine.1;
        g[3] = -self.rc;
        g[4] = -self.bc;
        g[5] = -m.chine.1;
        g[6] = m.chine_bottom.0 - self.bc;
        g[7] = m.keel_tangent.0 - m.chine_bottom.0;
        g[8] = 1e-8 - fabs(self.rk);
        g[9] = self.kappa_bow * self.lb - self.delta_bow(0.0);
        g[10] = self.drift(0.0) - PI / 2.0;
        g[11] = -self.drift(0.0);
        g[12] = self.drift(dd) - PI / 2.0;
        g[13] = -self.drift(dd);
        let zv = vertex(self.drift[0], self.drift[1]);
        if (0.0..=dd).contains(&zv) {
            g[14] = self.drift(zv) - PI / 2.0;
            g[15] = -self.drift(zv);
        }
        g[16] = -self.bk.0;
        g[17] = self.bk.0 - self.kappa_bow * self.lb;
        g[18] = -self.bk.1;
        g[19] = self.bk.1 - dd;
        g[20] = -self.delta_bow(dd) + self.bow_profile(dd);
        g[21] = -self.delta_bow(self.bk.1) + self.bk.0;
        let zv = vertex(self.delta_bow[0], self.delta_bow[1]);
        if (0.0..=dd).contains(&zv) {
            g[22] = -self.delta_bow(zv) + self.bow_profile(zv);
        }
        let zv = vertex(self.bow[0], self.bow[1]);
        if (0.0..=dd).contains(&zv) {
            g[23] = -self.delta_bow(zv) + self.bow_profile(zv);
        }
        let start = self.lb + self.lm + self.ls * self.kappa_stern;
        g[24] = self.delta_stern(0.0) - start;
        g[25] = self.delta_stern(self.sk.1) - self.sk.0;
        let zv = vertex(self.delta_stern[0], self.delta_stern[1]);
        if (0.0..=dd).contains(&zv) {
            g[26] = self.delta_stern(zv) - self.stern_profile(zv);
        }
        g[27] = self.delta_stern(dd) - self.stern_profile(dd);
        g[28] = start - self.sk.0;
        let t = &self.transom;
        g[29] = self.bc_trans - self.mid.half_breadth(t.chine.1);
        g[30] = -t.chine_side.1 + t.chine.1;
        g[31] = -self.rc_trans;
        g[32] = -self.bc_trans;
        g[33] = -t.chine.1;
        g[34] = t.chine_bottom.0 - self.bc_trans;
        g[35] = t.keel_tangent.0 - t.chine_bottom.0;
        let mid_half = |z: f64| self.mid.half_breadth(z);
        // A bulb needs a zero deadrise or a positive keel radius.
        let bulb_rows = |b: &Bulb| {
            if self.beta == 0.0 {
                (-1.0, -1.0, b.ry - mid_half(b.rz_lo))
            } else if self.rk > 0.0 {
                (b.rz_lo - self.rk, b.ry - self.rk, b.ry - mid_half(b.rz_lo))
            } else {
                (1.0, 1.0, 1.0)
            }
        };
        if let Some(b) = &self.bb {
            (g[36], g[37], g[38]) = bulb_rows(b);
            g[39] = b.cx - self.delta_bow(0.0);
            let zv = vertex(self.delta_bow[0], self.delta_bow[1]);
            if (0.0..=self.wl).contains(&zv) {
                g[40] = b.cx - self.delta_bow(zv);
            }
            g[41] = b.cx - self.delta_bow(self.wl);
        }
        if let Some(b) = &self.sb {
            (g[42], g[43], g[44]) = bulb_rows(b);
            g[45] = if self.beta == 0.0 || self.rk > 0.0 {
                b.top - self.sk.1
            } else {
                1.0
            };
            g[46] = self.delta_stern(0.0) - b.cx;
            g[47] = self.delta_stern(b.top) - b.cx;
            let zv = vertex(self.delta_stern[0], self.delta_stern[1]);
            if (0.0..=b.top).contains(&zv) {
                g[48] = self.delta_stern(zv) - b.cx;
            }
        }
        g
    }

    /// The heights the Sections Table cuts at: 40 even levels from the base to the deck, and the
    /// knuckle heights (`BK_z`, `SK_z`, the ends of the keel and chine arcs, midship and transom).
    pub fn heights(&self, levels: u32) -> Vec<f64> {
        let dd = self.dd;
        let mut z: Vec<f64> = (0..=levels)
            .map(|k| {
                if k == levels {
                    dd
                } else {
                    dd * f64::from(k) / f64::from(levels)
                }
            })
            .collect();
        let (m, t) = (&self.mid, &self.transom);
        for k in [
            self.bk.1,
            self.sk.1,
            m.keel_tangent.1,
            m.chine_bottom.1,
            m.chine_side.1,
            t.keel_tangent.1,
            t.chine_bottom.1,
            t.chine_side.1,
        ] {
            if k > 0.0 && k < dd {
                z.push(k);
            }
        }
        // The bulbs: the height of the widest part, the top, and a hair below the top (where the
        // waterline still has the bulb).
        for b in [&self.bb, &self.sb].into_iter().flatten() {
            for k in [b.rz_lo, b.top, b.top - 1e-9 * dd] {
                if k > 0.0 && k <= dd {
                    z.push(k);
                }
            }
        }
        // Just above `SK_z` the transom begins: the waterline ends at the transom's half-breadth,
        // which may jump from 0 (a flat transom bottom).
        let above = self.sk.1 + 1e-9 * dd;
        if self.sk.1 > 0.0 && above < dd {
            z.push(above);
        }
        z.sort_by(f64::total_cmp);
        z.dedup();
        z
    }
}

impl Waterline {
    /// Where the waterline starts and ends (x_ShipD): the stem or the bulb's leading edge, the
    /// stern or the stern bulb's trailing edge.
    pub fn span(&self) -> (f64, f64) {
        (self.start, self.end)
    }

    /// The half-breadth at `x` on this waterline of `s`; 0 outside it.
    pub fn half_breadth(&self, s: &Shape, x: f64) -> f64 {
        let z = self.z;
        if x <= self.start || x >= self.end {
            return 0.0;
        }
        let y = if x < s.delta_bow(z) {
            match (&self.bow, &s.bb) {
                (BowLine::Filleted(f), Some(b)) => {
                    if x <= f.xr0 {
                        b.half_width(z, x, false)
                    } else if x < f.xr1 {
                        f.at(x, true)
                    } else {
                        s.bow_y(x, z)
                    }
                }
                (BowLine::Wider, Some(b)) => {
                    let bulb = b.half_width(z, x, false).min(self.y_mid);
                    let hull = if x >= s.bow_profile(z) {
                        s.bow_y(x, z)
                    } else {
                        0.0
                    };
                    bulb.max(hull)
                }
                _ => s.bow_y(x, z),
            }
        } else if x <= s.delta_stern(z) {
            self.y_mid
        } else {
            match (&self.stern, &s.sb) {
                (SternLine::Filleted(f), Some(b)) => {
                    if x <= f.xr0 {
                        s.stern_y(x, z, self.stern_ellipse)
                    } else if x < f.xr1 {
                        f.at(x, false)
                    } else {
                        b.half_width(z, x, true)
                    }
                }
                _ => s.stern_y(x, z, self.stern_ellipse),
            }
        };
        // A fillet may dip a hair below the centre line.
        y.max(0.0)
    }
}

/// How finely the generator cuts the hull: even stations over the length, even height levels, and
/// whether each height's waterline ends get a station of their own.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Resolution {
    pub stations: u32,
    pub levels: u32,
    pub waterline_ends: bool,
    /// The curvature row of the bulb fillets.
    pub fillet_row: FilletRow,
    /// The stern ellipse and the bow bulb's crossing with the hull.
    pub stern_ellipse: Reading,
    pub bulb_crossing: Reading,
}

/// A Design's cut: coarse enough that its hydrostatics and GZ curves stay fast. The benchmark's
/// report gives its accuracy beside the benchmark's own finer cut.
pub const DESIGN: Resolution = Resolution {
    stations: 41,
    levels: 20,
    waterline_ends: false,
    fillet_row: FilletRow::Definition,
    stern_ellipse: Reading::Definition,
    bulb_crossing: Reading::Definition,
};

/// The benchmark's cut: fine enough that the comparison is of the shape, not of the cut.
pub const BENCHMARK: Resolution = Resolution {
    stations: 101,
    levels: 200,
    waterline_ends: true,
    fillet_row: FilletRow::Definition,
    stern_ellipse: Reading::Definition,
    bulb_crossing: Reading::Definition,
};

/// The constraints `p` fails at these dimensions, with their index and name.
pub fn check(p: &ShipdParameters, loa: f64, beam: f64, depth: f64) -> Vec<ConstraintFailure> {
    Shape::new(p, loa, beam, depth)
        .constraints()
        .iter()
        .enumerate()
        // A NaN value fails too.
        .filter(|(_, g)| g.is_nan() || **g > 0.0)
        .map(|(index, &g)| ConstraintFailure {
            index,
            name: CONSTRAINTS[index],
            g,
        })
        .collect()
}

/// The Sections Table of the ShipD hull `p` at length `loa`, beam `beam` and depth `depth`, cut at
/// `r`: even stations over the length, plus the knuckles (and, if asked, the ends of each height's
/// waterline); points at even heights plus the knuckle heights. A Design takes [`DESIGN`].
pub fn sections(
    p: &ShipdParameters,
    loa: f64,
    beam: f64,
    depth: f64,
    r: Resolution,
) -> Result<SectionsTable, ShipdError> {
    let positive = |x: f64| x.is_finite() && x > 0.0;
    if !(positive(loa) && positive(beam) && positive(depth)) || p.0.iter().any(|v| !v.is_finite()) {
        return Err(ShipdError::NotFinite);
    }
    let shape = Shape::new(p, loa, beam, depth);
    let failures = check(p, loa, beam, depth);
    if !failures.is_empty() {
        return Err(ShipdError::Constraints(failures));
    }
    let heights = shape.heights(r.levels.max(1));
    let lines: Vec<Waterline> = heights
        .iter()
        .map(|&z| shape.waterline_with(z, r.fillet_row, r.stern_ellipse, r.bulb_crossing))
        .collect();
    // Stations in the dataset's x: even ones, the knuckles, and each waterline's two ends.
    let n = r.stations.max(2) - 1;
    let mut xs: Vec<f64> = (0..=n).map(|i| loa * f64::from(i) / f64::from(n)).collect();
    xs.extend([
        shape.lb,
        shape.lb + shape.lm,
        shape.bk.0,
        shape.sk.0,
        shape.kappa_bow * shape.lb,
        shape.lb + shape.lm + shape.ls * shape.kappa_stern,
    ]);
    // A bulb's start, its widest part and its tip (at the height of its widest part).
    for b in [&shape.bb, &shape.sb].into_iter().flatten() {
        xs.extend([b.cx, b.start, b.cx - b.rx, b.cx + b.rx]);
    }
    if r.waterline_ends {
        for l in &lines[1..] {
            xs.push(l.start);
            xs.push(l.end);
        }
    }
    // The stem or the stern may reach a little past the ends of LOA where a rake bulges (the
    // definitions allow it), and a bulb reaches past the stem: the hull keeps those stations.
    let (lo, hi) = lines.iter().fold((0.0_f64, loa), |(lo, hi), l| {
        (lo.min(l.start), hi.max(l.end))
    });
    xs.extend([lo, hi]);
    xs.retain(|x| x.is_finite() && (lo..=hi).contains(x));
    xs.sort_by(f64::total_cmp);
    xs.dedup();
    let dd = depth;
    let sections: Vec<Section> = xs
        .iter()
        .rev()
        .map(|&x| {
            // The keel on the centre line, then each height: at the base the half-breadth of the
            // flat of bottom (0 for a hull with deadrise or a keel arc), up to the deck edge, then
            // the deck at the centre line.
            let mut points = Vec::with_capacity(heights.len() + 2);
            points.push((0.0, 0.0));
            for (l, &z) in lines.iter().zip(&heights) {
                points.push((l.half_breadth(&shape, x), z));
            }
            points.push((0.0, dd));
            Section { x: loa - x, points }
        })
        .collect();
    let depths = alloc::vec![dd; sections.len()];
    SectionsTable::new(sections, depths).map_err(ShipdError::Sections)
}

/// The draughts of the Ship-D dataset's measures, as fractions of the depth.
pub const DRAUGHT_FRACTIONS: [f64; 10] =
    [0.12, 0.25, 0.33, 0.38, 0.50, 0.62, 0.67, 0.75, 0.88, 1.0];

/// The quantities the benchmark compares at one draught, in the dataset's form: divided by powers
/// of LOA, with x (LCB, LCF) measured aft from the stem.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measures {
    pub volume: f64,
    pub waterplane_area: f64,
    pub lcb: f64,
    pub lcf: f64,
    pub ixx: f64,
    pub iyy: f64,
    pub waterline_length: f64,
}

/// The [`Measures`] of a Sections Table at the upright draughts `draughts` (m), for a hull of length
/// `loa`.
pub fn measures(table: &SectionsTable, loa: f64, draughts: &[f64]) -> Vec<Measures> {
    draughts
        .iter()
        .map(|&t| {
            let i = crate::sections::immersed(table, &crate::sections::Waterplane::upright(t));
            let (l2, l3, l4) = (loa * loa, loa * loa * loa, loa * loa * loa * loa);
            Measures {
                volume: i.volume_m3 / l3,
                waterplane_area: i.waterplane_area_m2 / l2,
                lcb: (loa - i.lcb_m) / loa,
                lcf: (loa - i.lcf_m) / loa,
                ixx: i.i_t_m4 / l4,
                iyy: i.i_l_m4 / l4,
                waterline_length: i.l_wl_m / loa,
            }
        })
        .collect()
}

/// The [`Measures`] of the ShipD hull `p` at the dataset's draughts, taken the way the dataset took
/// its own (read from its `Calc_VolumeProperties` and the functions it calls; no code is copied):
/// 101 waterlines, evenly from 1e-5 m to the depth; on each, 1000 points shared between the bow
/// taper, the midbody and the stern taper by their length; the waterplane area, its centroid and
/// its second moments by trapezoids between the points; the volume and the LCB by the trapezoid rule
/// over the waterlines; each quantity at a draught by linear interpolation between waterlines. The
/// benchmark compares OVD's shape this way, so the comparison is of the shape, not of the
/// integration rule. `r` gives the readings; its cut is not used.
pub fn dataset_measures(
    p: &ShipdParameters,
    loa: f64,
    beam: f64,
    depth: f64,
    r: Resolution,
) -> Vec<Measures> {
    const WATERLINES: usize = 101;
    const POINTS: usize = 1000;
    let s = Shape::new(p, loa, beam, depth);
    let z0 = 1e-5;
    let zs: Vec<f64> = (0..WATERLINES)
        .map(|i| z0 + (depth - z0) * i as f64 / (WATERLINES - 1) as f64)
        .collect();
    let even = |a: f64, b: f64, n: usize| -> Vec<f64> {
        if n == 1 {
            return alloc::vec![a];
        }
        (0..n)
            .map(|i| a + (b - a) * i as f64 / (n - 1) as f64)
            .collect()
    };
    // Per waterline: area, LCF, Ixx, Iyy, waterline length.
    let mut wp: Vec<[f64; 5]> = Vec::with_capacity(WATERLINES);
    for &z in &zs {
        let wl = s.waterline_with(z, r.fillet_row, r.stern_ellipse, r.bulb_crossing);
        let (start, end) = wl.span();
        let (db, ds) = (s.delta_bow(z), s.delta_stern(z));
        let len = end - start;
        let share = |a: f64| (fabs(a / len * POINTS as f64) as usize) + 1;
        let (pb, mut ps) = (share(db - start), share(end - ds));
        let pm = if pb + ps > POINTS {
            ps -= pb + ps - POINTS;
            0
        } else {
            POINTS - pb - ps
        };
        let mut xs = even(start, db, pb);
        xs.extend_from_slice(&even(db, ds, pm + 2)[1..=pm]);
        xs.extend(even(ds, end, ps));
        // The ends of a waterline carry the stem's and the transom's half-breadth.
        let eps = 1e-12 * loa;
        let ys: Vec<f64> = xs
            .iter()
            .map(|&x| 2.0 * wl.half_breadth(&s, x.clamp(start + eps, end - eps)))
            .collect();
        let (mut area, mut moment) = (0.0, 0.0);
        for j in 1..xs.len() {
            let (dx, a, b) = (xs[j] - xs[j - 1], ys[j], ys[j - 1]);
            area += 0.5 * (a + b) * dx;
            if a + b > 0.0 {
                moment += (xs[j - 1] + dx / 3.0 * (2.0 * a + b) / (a + b)) * 0.5 * (a + b) * dx;
            }
        }
        let lcf = moment / area;
        let (mut ixx, mut iyy) = (0.0, 0.0);
        for j in 1..xs.len() {
            let (dx, a, b) = (xs[j] - xs[j - 1], ys[j], ys[j - 1]);
            if a + b > 0.0 {
                let cx = xs[j - 1] + dx / 3.0 * (2.0 * a + b) / (a + b);
                ixx += dx / 48.0 * (a + b) * (a * a + b * b);
                iyy += dx * dx * dx * (a * a + 4.0 * a * b + b * b) / (36.0 * (a + b))
                    + 0.5 * (a + b) * dx * (lcf - cx) * (lcf - cx);
            }
        }
        wp.push([area, lcf, ixx, iyy, xs[xs.len() - 1] - xs[0]]);
    }
    let mut volume = alloc::vec![0.5 * wp[0][0] * zs[0]; WATERLINES];
    let mut lcb = alloc::vec![0.0; WATERLINES];
    let mut moment = 0.0;
    for i in 1..WATERLINES {
        let dz = zs[i] - zs[i - 1];
        volume[i] = volume[i - 1] + 0.5 * (wp[i][0] + wp[i - 1][0]) * dz;
        moment += 0.5 * (wp[i][0] * wp[i][1] + wp[i - 1][0] * wp[i - 1][1]) * dz;
        lcb[i] = moment / volume[i];
    }
    let at = |f: &dyn Fn(usize) -> f64, t: f64| -> f64 {
        let i = zs
            .iter()
            .rposition(|&z| z < t)
            .unwrap_or(0)
            .min(WATERLINES - 2);
        let u = (t - zs[i]) / (zs[i + 1] - zs[i]);
        f(i) + u * (f(i + 1) - f(i))
    };
    let (l2, l3) = (loa * loa, loa * loa * loa);
    DRAUGHT_FRACTIONS
        .iter()
        .map(|fr| {
            let t = fr * depth;
            Measures {
                volume: at(&|i| volume[i], t) / l3,
                waterplane_area: at(&|i| wp[i][0], t) / l2,
                lcb: at(&|i| lcb[i], t) / loa,
                lcf: at(&|i| wp[i][1], t) / loa,
                ixx: at(&|i| wp[i][2], t) / (l2 * l2),
                iyy: at(&|i| wp[i][3], t) / (l2 * l2),
                waterline_length: at(&|i| wp[i][4], t) / loa,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Label: property. The fillet's quartic has the values, the slopes and the curvature it is
    /// given at its two ends, for any such data.
    #[test]
    fn the_quartic_meets_its_five_conditions() {
        let cases = [
            (0.2, 0.5, 0.35, -0.1, 0.4),
            (0.0, 0.0, 0.3, 0.0, 1.0),
            (0.31, -2.0, 0.05, 3.0, 0.07),
        ];
        for (ya, da, yb, db, h) in cases {
            let c = quartic(ya, da, yb, db, h);
            let d1 = |t: f64| c[1] + t * (2.0 * c[2] + t * (3.0 * c[3] + t * 4.0 * c[4]));
            let d2 = |t: f64| 2.0 * c[2] + t * (6.0 * c[3] + t * 12.0 * c[4]);
            let tol = 1e-9;
            assert!((quartic_at(&c, 0.0) - ya).abs() < tol);
            assert!((quartic_at(&c, h) - yb).abs() < tol, "{c:?}");
            assert!((d1(0.0) - da).abs() < tol);
            assert!((d1(h) - db).abs() < 1e-7 * (1.0 + db.abs()), "{c:?}");
            assert!(d2(0.0).abs() < tol);
        }
    }

    /// Label: property. The dataset's row is not the curvature condition: with the same data its
    /// quartic has the same values and slopes, and a second derivative at the chosen end that is
    /// not 0 (the row has `6 x^2` where the derivative has `6 x`).
    #[test]
    fn the_dataset_row_meets_its_four_conditions_and_not_the_curvature_one() {
        let (x0, y0, d0, x1, y1, d1) = (4.1, 0.2, 0.5, 4.5, 0.35, -0.1);
        let c = quartic_dataset_row(x0, y0, d0, x1, y1, d1, x0);
        let slope = |x: f64| c[1] + x * (2.0 * c[2] + x * (3.0 * c[3] + x * 4.0 * c[4]));
        let curv = |x: f64| 2.0 * c[2] + x * (6.0 * c[3] + x * 12.0 * c[4]);
        assert!((quartic_at(&c, x0) - y0).abs() < 1e-7);
        assert!((quartic_at(&c, x1) - y1).abs() < 1e-7);
        assert!((slope(x0) - d0).abs() < 1e-6);
        assert!((slope(x1) - d1).abs() < 1e-6);
        assert!(curv(x0).abs() > 1e-3, "{}", curv(x0));
    }

    /// Label: property. The first root of a function is the smallest, and none is found where
    /// there is none.
    #[test]
    fn first_root_finds_the_smallest_root() {
        let f = |x: f64| (x - 1.0) * (x - 2.0) * (x - 3.0);
        let r = first_root(f, 0.0, 4.0).unwrap();
        assert!((r - 1.0).abs() < 1e-9, "{r}");
        assert!(first_root(|x| x * x + 1.0, -2.0, 2.0).is_none());
    }
}
