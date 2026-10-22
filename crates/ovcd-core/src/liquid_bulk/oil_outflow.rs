//! The accidental oil outflow of a liquid bulk hull: the mean oil outflow parameter OM of MARPOL Annex I
//! regulation 23 (issue #341, spec 0006), by the damaged tank method of 23.4 to 23.9.
//!
//! The text is Res. MEPC.117(52), regulation 23 (assumptions 4, combination 5, side outflow 6, bottom outflow
//! 7, probabilities 8 and 9). The explanatory notes, Res. MEPC.122(52) part C 1, work the same method for a
//! tank barge: `tests/oil_outflow.rs` runs that barge and reproduces its figures.
//!
//! Assumptions that are the model's, not the text's:
//! - The ship is a prism: the greatest breadth at or below the load line (BS) and at or below the waterline
//!   dB (BB) are the beam, and the bottom shell is flat at the base line, so `z` of 9.2 is the lowest point of
//!   the tank. A cargo tank is bounded by non-oil compartments below when that lowest point is above the base
//!   line (the factor CDB of 7.4).
//! - L is the overall length of the hull, and x runs from its aft end.
//! - The load line draught dS is the draught of the full load of the run, and the deadweight is that of the run.
//! - There is an inert gas overpressure of 5 kPa (7.3.2): the text allows 0 where no inert gas system is fitted,
//!   which would retain more oil, so 5 kPa does not understate the outflow.
//! - The two longitudinal bulkheads of C3 = 0.77 (6) are not looked for: C3 is 1.0.
//! - Fuel tanks in the cargo block (4.2) are not modelled.
//! - The side distance y is the least distance of the tank to the side shell; the 1.5 h point of a mid-deck
//!   tanker (the explanatory notes, figure 20) is not used.

use alloc::string::String;
use alloc::vec::Vec;

use crate::box_stability::{area_centroid, clip_below};
use crate::compartments::{Space, SpaceKind};

const PSA: [f64; 21] = [
    0.0, 0.023, 0.068, 0.117, 0.167, 0.217, 0.267, 0.317, 0.367, 0.417, 0.467, 0.517, 0.567, 0.617,
    0.667, 0.717, 0.767, 0.817, 0.867, 0.917, 0.967,
];
const PSF: [f64; 21] = [
    0.967, 0.917, 0.867, 0.817, 0.767, 0.717, 0.667, 0.617, 0.567, 0.517, 0.467, 0.417, 0.367,
    0.317, 0.267, 0.217, 0.167, 0.117, 0.068, 0.023, 0.0,
];
const PSL: [f64; 21] = [
    0.0, 0.0, 0.001, 0.003, 0.007, 0.013, 0.021, 0.034, 0.055, 0.085, 0.123, 0.172, 0.226, 0.285,
    0.347, 0.413, 0.482, 0.553, 0.626, 0.7, 0.775,
];
const PSU: [f64; 21] = [
    0.968, 0.952, 0.931, 0.905, 0.873, 0.836, 0.789, 0.733, 0.67, 0.599, 0.525, 0.452, 0.383,
    0.317, 0.255, 0.197, 0.143, 0.092, 0.046, 0.013, 0.0,
];
const PBA: [f64; 21] = [
    0.0, 0.002, 0.008, 0.017, 0.029, 0.042, 0.058, 0.076, 0.096, 0.119, 0.143, 0.171, 0.203, 0.242,
    0.289, 0.344, 0.409, 0.482, 0.565, 0.658, 0.761,
];
const PBF: [f64; 21] = [
    0.969, 0.953, 0.936, 0.916, 0.894, 0.87, 0.842, 0.81, 0.775, 0.734, 0.687, 0.63, 0.563, 0.489,
    0.413, 0.333, 0.252, 0.17, 0.089, 0.026, 0.0,
];
const PBP: [f64; 21] = [
    0.844, 0.794, 0.744, 0.694, 0.644, 0.594, 0.544, 0.494, 0.444, 0.394, 0.344, 0.297, 0.253,
    0.211, 0.171, 0.133, 0.097, 0.063, 0.032, 0.009, 0.0,
];
const PBS: [f64; 21] = [
    0.0, 0.009, 0.032, 0.063, 0.097, 0.133, 0.171, 0.211, 0.253, 0.297, 0.344, 0.394, 0.444, 0.494,
    0.544, 0.594, 0.644, 0.694, 0.744, 0.794, 0.844,
];

/// Linear interpolation in a table of 21 values at ratios 0, 0.05, ... 1.00 (regulation 8.2 and 9.2). A ratio
/// outside 0 to 1 takes the end value.
fn interp(table: &[f64; 21], ratio: f64) -> f64 {
    let x = ratio.clamp(0.0, 1.0) * 20.0;
    let i = (x as usize).min(19);
    let f = x - i as f64;
    table[i] + f * (table[i + 1] - table[i])
}

/// PSy of 8.3: the probability that the damage lies entirely outboard of the tank, from `y / BS`.
fn p_sy(y_over_b: f64) -> f64 {
    let r = y_over_b.max(0.0);
    let p = if r <= 0.05 {
        (24.96 - 199.6 * r) * r
    } else if r < 0.1 {
        let s = r - 0.05;
        0.749 + (5.0 - 44.4 * s) * s
    } else {
        0.888 + 0.56 * (r - 0.1)
    };
    p.min(1.0)
}

/// PBz of 9.3: the probability that the damage lies entirely below the tank, from `z / DS`.
fn p_bz(z_over_d: f64) -> f64 {
    let r = z_over_d.max(0.0);
    let p = if r <= 0.1 {
        (14.5 - 67.0 * r) * r
    } else {
        0.78 + 1.1 * (r - 0.1)
    };
    p.min(1.0)
}

/// What the regulation needs of the hull and the loading.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OilOutflowInputs {
    /// L, m.
    pub length_m: f64,
    /// BS and BB, m.
    pub beam_m: f64,
    /// DS, the moulded depth at mid-length, m.
    pub depth_m: f64,
    /// dS, the load line draught, m.
    pub draught_m: f64,
    /// The deadweight, t.
    pub deadweight_t: f64,
    /// The inert gas overpressure p of 7.3.2, kPa.
    pub overpressure_kpa: f64,
}

/// One cargo or slop tank and what its damage does.
#[derive(Debug, Clone, PartialEq)]
pub struct TankOutflow {
    pub name: String,
    /// The oil in the tank at 98 % filling and a permeability of 0.99 (4.4, 4.5), m3.
    pub oil_m3: f64,
    /// The probability of breaching the tank by a side damage, from the port and from the starboard side.
    pub ps_port: f64,
    pub ps_starboard: f64,
    /// The probability of breaching the tank by a bottom damage, centre of damage to port and to starboard.
    pub pb_port: f64,
    pub pb_starboard: f64,
    /// The outflow by a bottom damage at tides 0 and -2.5 m, m3.
    pub ob_0_m3: f64,
    pub ob_25_m3: f64,
    /// CDB (7.4).
    pub cdb: f64,
}

/// The mean oil outflow of the hull.
#[derive(Debug, Clone, PartialEq)]
pub struct OilOutflow {
    /// C: the oil in the cargo tanks at 98 % filling, m3.
    pub c_m3: f64,
    /// The nominal density of the cargo oil, kg/m3 (4.4).
    pub rho_n: f64,
    /// OMS, OMB(0), OMB(2.5), OMB, m3.
    pub oms_m3: f64,
    pub omb_0_m3: f64,
    pub omb_25_m3: f64,
    pub omb_m3: f64,
    /// OM, the mean oil outflow parameter (5.1).
    pub om: f64,
    /// The largest OM 23.3.1 allows for C.
    pub limit: f64,
    pub tanks: Vec<TankOutflow>,
}

impl OilOutflow {
    /// OM over its limit: at most 1 meets 23.3.1.
    pub fn ratio(&self) -> f64 {
        self.om / self.limit
    }
}

/// The largest OM of 23.3.1 for a tanker of 5,000 tonnes deadweight and above, by the cargo volume C (m3):
/// 0.015 up to 200,000 m3, falling to 0.012 at 400,000 m3.
pub fn om_limit(c_m3: f64) -> f64 {
    if c_m3 <= 200_000.0 {
        0.015
    } else if c_m3 < 400_000.0 {
        0.012 + (0.003 / 200_000.0) * (400_000.0 - c_m3)
    } else {
        0.012
    }
}

/// The y extent of the part of a section at or below `z_max`, or `None` when none is.
fn y_extent_below(section: &[(f64, f64)], z_max: f64) -> Option<(f64, f64)> {
    let mut y: Vec<f64> = section
        .iter()
        .filter(|&&(_, z)| z <= z_max + 1e-9)
        .map(|&(y, _)| y)
        .collect();
    for i in 0..section.len() {
        let (p, q) = (section[i], section[(i + 1) % section.len()]);
        if (p.1 < z_max) != (q.1 < z_max) {
            let t = (z_max - p.1) / (q.1 - p.1);
            y.push(p.0 + t * (q.0 - p.0));
        }
    }
    let lo = y.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = y.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    (lo <= hi).then_some((lo, hi))
}

/// The hydrostatic balance of 7.3.2: the height of the cargo above the lowest point of the tank, m (negative
/// when the sea is higher than the oil can balance).
fn cargo_height_m(inp: &OilOutflowInputs, tide_m: f64, zl_m: f64, rho_n: f64) -> f64 {
    ((inp.draught_m + tide_m - zl_m) * 1025.0 - 1000.0 * inp.overpressure_kpa / 9.81) / rho_n
}

/// OM of regulation 23.5 to 23.9 for the cargo and slop tanks of `spaces`. `None` when there is no cargo
/// volume or the deadweight is not above zero.
pub fn oil_outflow(spaces: &[Space], inp: &OilOutflowInputs) -> Option<OilOutflow> {
    const FILL: f64 = 0.98;
    const PERMEABILITY: f64 = 0.99;
    let tanks: Vec<&Space> = spaces
        .iter()
        .filter(|s| matches!(s.kind, SpaceKind::CargoTank | SpaceKind::Slop))
        .collect();
    let oil = |s: &Space| FILL * PERMEABILITY * s.volume_m3();
    let c: f64 = tanks.iter().map(|s| oil(s)).sum();
    if c <= 0.0 || inp.deadweight_t <= 0.0 || inp.length_m <= 0.0 || inp.beam_m <= 0.0 {
        return None;
    }
    let rho_n = 1000.0 * inp.deadweight_t / c;
    let half = 0.5 * inp.beam_m;
    let d_b = 0.3 * inp.depth_m;
    let mut out = Vec::new();
    let (mut oms_p, mut oms_s) = (0.0, 0.0);
    let (mut omb0_p, mut omb0_s, mut omb25_p, mut omb25_s) = (0.0, 0.0, 0.0, 0.0);
    for s in &tanks {
        let (z_lo, z_hi) = s.z_range();
        let (y_lo, y_hi) = s
            .section
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), &(y, _)| {
                (a.min(y), b.max(y))
            });
        let volume = oil(s);
        // longitudinal and vertical factors of 8.1, 9.1 (the same for both sides)
        let p_sl =
            1.0 - interp(&PSF, (s.x_fwd) / inp.length_m) - interp(&PSA, s.x_aft / inp.length_m);
        let p_sv = 1.0
            - interp(&PSU, z_hi.min(inp.depth_m) / inp.depth_m)
            - interp(&PSL, z_lo / inp.depth_m);
        let p_bl =
            1.0 - interp(&PBF, s.x_fwd / inp.length_m) - interp(&PBA, s.x_aft / inp.length_m);
        let p_bv = 1.0 - p_bz(z_lo / inp.depth_m);
        // y: the least distance of the tank to the side shell, from each side
        let ps = |y: f64| (p_sl * p_sv * (1.0 - p_sy(y / inp.beam_m))).max(0.0);
        let ps_stbd = ps(half - y_hi);
        let ps_port = ps(half + y_lo);
        // Yp and Ys: from the port-most and starboard-most point at or below dB to the plane BB/2 to
        // starboard (centre of damage to starboard); mirrored for the other side
        let (below_lo, below_hi) = y_extent_below(&s.section, d_b).unwrap_or((y_lo, y_hi));
        let pb = |yp: f64, ys: f64| {
            let p_bt = 1.0 - interp(&PBP, yp / inp.beam_m) - interp(&PBS, ys / inp.beam_m);
            (p_bl * p_bt * p_bv).max(0.0)
        };
        let pb_stbd = pb(half - below_lo, half - below_hi);
        let pb_port = pb(half + below_hi, half + below_lo);
        // 7.3.2, 7.3.3: the oil left in the tank is the part below the balance level, at the permeability
        let ob = |tide: f64| {
            let h = cargo_height_m(inp, tide, z_lo, rho_n);
            let left = if h <= 0.0 {
                0.0
            } else {
                PERMEABILITY * area_centroid(&clip_below(&s.section, z_lo + h)).0 * s.length()
            };
            let mut o = (volume - left.min(volume)).max(0.0);
            if z_lo < 1e-9 {
                o = o.max(0.01 * volume);
            }
            o
        };
        let (ob0, ob25) = (ob(0.0), ob(-2.5));
        let cdb = if z_lo < 1e-9 { 1.0 } else { 0.6 };
        oms_p += ps_port * volume;
        oms_s += ps_stbd * volume;
        omb0_p += pb_port * ob0 * cdb;
        omb0_s += pb_stbd * ob0 * cdb;
        omb25_p += pb_port * ob25 * cdb;
        omb25_s += pb_stbd * ob25 * cdb;
        out.push(TankOutflow {
            name: s.name.clone(),
            oil_m3: volume,
            ps_port,
            ps_starboard: ps_stbd,
            pb_port,
            pb_starboard: pb_stbd,
            ob_0_m3: ob0,
            ob_25_m3: ob25,
            cdb,
        });
    }
    // 6.2 and 6.3 of the explanatory notes: the port and the starboard damage are averaged. C3 = 1.0.
    let oms = 0.5 * (oms_p + oms_s);
    let omb_0 = 0.5 * (omb0_p + omb0_s);
    let omb_25 = 0.5 * (omb25_p + omb25_s);
    let omb = 0.7 * omb_0 + 0.3 * omb_25;
    let om = (0.4 * oms + 0.6 * omb) / c;
    Some(OilOutflow {
        c_m3: c,
        rho_n,
        oms_m3: oms,
        omb_0_m3: omb_0,
        omb_25_m3: omb_25,
        omb_m3: omb,
        om,
        limit: om_limit(c),
        tanks: out,
    })
}

/// The longest cargo tank that regulation 23.3.2 allows an oil tanker of less than 5,000 tonnes deadweight, m:
/// 10 m or the value below, whichever is the greater, for the cargo tank number `index` of `spaces` (counting
/// the cargo tanks in layout order), a hull of length `length_m` and breadth `beam_m`.
///
/// The longitudinal bulkheads are those of the cargo tanks the tank lies abreast of: one tank across has
/// none (23.3.2.1: `(0.5 bi/B + 0.1) L`, not above `0.2 L`), two have a centre line bulkhead (23.3.2.2:
/// `(0.25 bi/B + 0.15) L`), three or more have two or more bulkheads (23.3.2.3: the outer tanks `0.2 L`; a
/// centre tank `0.2 L` where `bi/B` is 0.2 or more, else the first or second formula, the second when the
/// number across is even, which puts a bulkhead on the centre line). `bi` is the least distance of the tank to
/// the side shell. `None` for an index with no cargo tank.
pub fn max_cargo_tank_length_m(
    spaces: &[Space],
    index: usize,
    length_m: f64,
    beam_m: f64,
) -> Option<f64> {
    let tanks: Vec<&Space> = spaces
        .iter()
        .filter(|s| s.kind == SpaceKind::CargoTank)
        .collect();
    let tank = tanks.get(index)?;
    let y_of = |s: &Space| {
        s.section
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), &(y, _)| {
                (a.min(y), b.max(y))
            })
    };
    let half = 0.5 * beam_m;
    let (lo, hi) = y_of(tank);
    let bi = (half - hi).min(half + lo).max(0.0);
    let ratio = bi / beam_m;
    // the tanks abreast: those that share a length of the hull with this one
    let abreast: Vec<&&Space> = tanks
        .iter()
        .filter(|t| t.x_aft < tank.x_fwd - 1e-9 && t.x_fwd > tank.x_aft + 1e-9)
        .collect();
    let across = abreast.len();
    let outer = {
        let lo_all = abreast
            .iter()
            .map(|t| y_of(t).0)
            .fold(f64::INFINITY, f64::min);
        let hi_all = abreast
            .iter()
            .map(|t| y_of(t).1)
            .fold(f64::NEG_INFINITY, f64::max);
        (lo - lo_all).abs() < 1e-9 || (hi - hi_all).abs() < 1e-9
    };
    let no_bulkhead = (0.5 * ratio + 0.1) * length_m;
    let centre_bulkhead = (0.25 * ratio + 0.15) * length_m;
    let formula = match across {
        0 | 1 => no_bulkhead.min(0.2 * length_m),
        2 => centre_bulkhead,
        _ if outer => 0.2 * length_m,
        _ if ratio >= 0.2 => 0.2 * length_m,
        _ if across.is_multiple_of(2) => centre_bulkhead,
        _ => no_bulkhead,
    };
    Some(formula.max(10.0))
}
