//! The ShipD-style parametric hull (spec 0008, "ShipD-style hull", issue #452): a Sections Table
//! generator written for OVCD from the definitions of the Ship-D hull parameterisation, without
//! bulbs.
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
//!   as lengths. The thirteen bulb constraints (36 to 48) are -1 when there is no bulb; the bulbs
//!   (and the bases of `Kappa_SB` and `Bbb` that the PDFs disagree on) come with #453.
//!
//! Axes: the dataset's x runs aft from the stem; OVCD's runs forward from the aft end, so a
//! Section's x is `L - x_ShipD`.

use crate::sections::{Section, SectionsError, SectionsTable};
use alloc::vec::Vec;
use core::f64::consts::PI;
use core::fmt;
use libm::{cos, fabs, sin, sqrt, tan};

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
    /// A bulb is asked for; the bulbs come with #453.
    Bulb,
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
            ShipdError::Bulb => write!(f, "a ShipD hull with a bulb is not built yet"),
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
    bulb: bool,
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
        Self {
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
            bulb: v[31] != 0.0 || v[32] != 0.0,
        }
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

    /// The x of the stern at height `z`: the stern rise below `SK_z`, the transom above.
    pub fn stern_profile(&self, z: f64) -> f64 {
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

    /// The half-breadth at (x_ShipD, z); 0 outside the hull.
    pub fn half_breadth(&self, x: f64, z: f64) -> f64 {
        if !(0.0..=self.dd).contains(&z) {
            return 0.0;
        }
        let (x_bow, x_stern) = (self.bow_profile(z), self.stern_profile(z));
        if x <= x_bow || x >= x_stern {
            return 0.0;
        }
        let y_mid = self.mid.half_breadth(z);
        let (d_bow, d_stern) = (self.delta_bow(z), self.delta_stern(z));
        if x < d_bow {
            // The cubic from the stem (0, slope tan drift) to the midship half-breadth (slope 0).
            let h = d_bow - x_bow;
            let s = (x - x_bow) / h;
            let m = tan(self.drift(z));
            return y_mid * s * s * (3.0 - 2.0 * s) + m * h * s * (1.0 - s) * (1.0 - s);
        }
        if x <= d_stern {
            return y_mid;
        }
        let y_t = self.transom_half_breadth(z);
        let span = x_stern - d_stern;
        let u = (x - d_stern) / span;
        let ellipse = if z >= self.sk.1 { self.ep_t } else { self.ep_s };
        if ellipse {
            (y_mid - y_t) * sqrt(fabs(1.0 - u * u)) + y_t
        } else {
            y_mid + (y_t - y_mid) * u * u
        }
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

/// How finely the generator cuts the hull: even stations over the length, even height levels, and
/// whether each height's waterline ends get a station of their own.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Resolution {
    pub stations: u32,
    pub levels: u32,
    pub waterline_ends: bool,
}

/// A Design's cut: coarse enough that its hydrostatics and GZ curves stay fast. The benchmark's
/// report gives its accuracy beside the benchmark's own finer cut.
pub const DESIGN: Resolution = Resolution {
    stations: 41,
    levels: 20,
    waterline_ends: false,
};

/// The benchmark's cut: fine enough that the comparison is of the shape, not of the cut.
pub const BENCHMARK: Resolution = Resolution {
    stations: 101,
    levels: 200,
    waterline_ends: true,
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
    if shape.bulb {
        return Err(ShipdError::Bulb);
    }
    let failures = check(p, loa, beam, depth);
    if !failures.is_empty() {
        return Err(ShipdError::Constraints(failures));
    }
    let heights = shape.heights(r.levels.max(1));
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
    if r.waterline_ends {
        for &z in &heights[1..] {
            xs.push(shape.bow_profile(z));
            xs.push(shape.stern_profile(z));
        }
    }
    // The stem or the stern may reach a little past the ends of LOA where a rake bulges (the
    // definitions allow it): the hull keeps those stations.
    let (lo, hi) = heights.iter().fold((0.0_f64, loa), |(lo, hi), &z| {
        (lo.min(shape.bow_profile(z)), hi.max(shape.stern_profile(z)))
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
            for &z in &heights {
                points.push((shape.half_breadth(x, z), z));
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
