//! The geometry and the sizing loop of a pushed Formation (issue #359, spec 0007 "Formations").
//!
//! A pushed barge or convoy with its pusher is run as one Equivalent Hull: the barges and the pusher
//! as one block, with the resistance method of the single hull (the method is `unverified`). The
//! pusher's size follows its installed power by the ratios of the spec, and the power follows the
//! resistance of the Equivalent Hull, which holds the pusher: the loop runs until the power moves less
//! than 1 %, at most five times.

use libm::pow;

/// The most passes of the sizing loop (spec 0007).
pub const MAX_PASSES: usize = 5;
/// The loop stops when the installed power moves by less than this share (spec 0007).
pub const POWER_TOLERANCE: f64 = 0.01;

/// The reference power of the pusher ratios (kW).
const REF_PUSHER_KW: f64 = 1400.0;
/// The reference power of the towing tug ratios (kW).
const REF_TOWING_KW: f64 = 3000.0;

/// The pusher length by the ratio fitted to the reference pushers (an Estimate, research 6.2):
/// 19.5 x (P / 1,400 kW)^0.67 (m).
pub fn pusher_length_m(power_kw: f64) -> f64 {
    19.5 * pow(power_kw.max(1.0) / REF_PUSHER_KW, 0.67)
}

/// The pusher beam by the same ratio family: 9.5 x (P / 1,400 kW)^0.45 (m).
pub fn pusher_beam_m(power_kw: f64) -> f64 {
    9.5 * pow(power_kw.max(1.0) / REF_PUSHER_KW, 0.45)
}

/// The pusher draught: 2.0 m, capped by the draught limit (the reference pushers span 1.4 to 2.65 m).
pub fn pusher_draught_m(draught_limit_m: Option<f64>) -> f64 {
    match draught_limit_m {
        Some(limit) if limit > 0.0 && limit < 2.0 => limit,
        _ => 2.0,
    }
}

/// The towing tug length by the ratio fitted to the reference tugs (an Estimate, research 7.3 and 10,
/// issue #361): 23 x (P / 3,000 kW)^0.45 (m).
pub fn towing_tug_length_m(power_kw: f64) -> f64 {
    23.0 * pow(power_kw.max(1.0) / REF_TOWING_KW, 0.45)
}

/// The towing tug beam: its length over 2.5 (an Estimate, as the length).
pub fn towing_tug_beam_m(length_m: f64) -> f64 {
    length_m / 2.5
}

/// The towing tug draught: 0.42 of its beam (an Estimate from the tug sheets).
pub fn towing_tug_draught_m(beam_m: f64) -> f64 {
    0.42 * beam_m
}

/// The pusher's moulded depth (m): its draught plus this (an Estimate; the inputs hold no depth).
pub const PUSHER_FREEBOARD_M: f64 = 1.0;

/// A pusher's main dimensions (m).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TugDims {
    pub length_m: f64,
    pub beam_m: f64,
    pub draught_m: f64,
}

impl TugDims {
    /// The moulded depth: the draught plus the freeboard estimate.
    pub fn depth_m(&self) -> f64 {
        self.draught_m + PUSHER_FREEBOARD_M
    }
}

/// The Equivalent Hull of a pushed Formation: the barges in line and abreast, and the pusher at the
/// stern.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EquivalentHull {
    pub length_m: f64,
    pub beam_m: f64,
    pub draught_m: f64,
    /// (sum of the displaced volumes) / (L x B x T).
    pub block_coefficient: f64,
    pub displaced_volume_m3: f64,
}

/// The Equivalent Hull of `abreast` x `in_line` identical barges and a pusher (spec 0007): length =
/// barges in line x barge length + pusher length; beam = barges abreast x barge beam; draught = the
/// barge's; block coefficient = the displaced volumes over L x B x T.
pub fn pushed_equivalent_hull(
    (barge_l, barge_b, barge_t): (f64, f64, f64),
    barge_volume_m3: f64,
    (abreast, in_line): (u64, u64),
    tug: &TugDims,
    tug_volume_m3: f64,
) -> EquivalentHull {
    let n = (abreast * in_line) as f64;
    let length_m = in_line as f64 * barge_l + tug.length_m;
    let beam_m = abreast as f64 * barge_b;
    let volume = n * barge_volume_m3 + tug_volume_m3;
    EquivalentHull {
        length_m,
        beam_m,
        draught_m: barge_t,
        block_coefficient: volume / (length_m * beam_m * barge_t),
        displaced_volume_m3: volume,
    }
}

/// The Equivalent Hull of an ATB (spec 0007, issue #362): the barge and the tug as one hull, of
/// length barge length + tug length - the Notch depth (the tug sits in the Notch), the barge's beam
/// and draught, and the block coefficient of the displaced volumes (the barge's, which the Notch has
/// already taken its share from, and the tug's) over L x B x T.
pub fn atb_equivalent_hull(
    (barge_l, barge_b, barge_t): (f64, f64, f64),
    barge_volume_m3: f64,
    tug: &TugDims,
    tug_volume_m3: f64,
    notch_depth_m: f64,
) -> EquivalentHull {
    let length_m = barge_l + tug.length_m - notch_depth_m;
    let volume = barge_volume_m3 + tug_volume_m3;
    EquivalentHull {
        length_m,
        beam_m: barge_b,
        draught_m: barge_t,
        block_coefficient: volume / (length_m * barge_b * barge_t),
        displaced_volume_m3: volume,
    }
}

/// The coupling force between a pusher and the pushed craft (ES-TRIN 2025/1, ESI-III-6, 2.1):
/// F_SB = 270 x P_B x L_S / B_S x 10^-3 kN, with P_B the installed power (kW), L_S the distance from
/// the stern of the pusher to the coupling point (m) and B_S the width of the pushing craft (m).
pub fn coupling_force_kn(power_kw: f64, l_s_m: f64, b_s_m: f64) -> f64 {
    270.0 * power_kw * l_s_m / b_s_m * 1e-3
}

/// The result of the sizing loop.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sized {
    pub power_kw: f64,
    /// The passes run.
    pub passes: usize,
    /// The power moved by less than [`POWER_TOLERANCE`] at the last pass.
    pub converged: bool,
}

/// Run the loop. `power_of(p)` gives the installed power the Formation needs when the pusher is
/// sized for the power `p`; the first pass starts at `start_kw`. The loop stops when the power
/// moves by less than 1 %, or after [`MAX_PASSES`] passes, and then reports it did not converge.
pub fn size_loop(start_kw: f64, mut power_of: impl FnMut(f64) -> f64) -> Sized {
    let mut p = start_kw;
    for pass in 1..=MAX_PASSES {
        let next = power_of(p);
        let moved = if p > 0.0 {
            (next - p).abs() / p
        } else {
            f64::INFINITY
        };
        p = next;
        if moved < POWER_TOLERANCE {
            return Sized {
                power_kw: p,
                passes: pass,
                converged: true,
            };
        }
    }
    Sized {
        power_kw: p,
        passes: MAX_PASSES,
        converged: false,
    }
}

/// The resistance of a tow (spec 0007, issue #361): the barges and the tug are separate hulls, so their
/// resistances add at each speed, and the Formation resistance allowance multiplies the sum.
/// `barge_kn` is the resistance of one barge, `barges` the number towed, `tug_kn` the tug's.
pub fn tow_resistance_kn(barge_kn: f64, barges: u64, tug_kn: f64, allowance: f64) -> f64 {
    (barges as f64 * barge_kn + tug_kn) * allowance
}
