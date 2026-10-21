//! Squat: how much deeper a moving vessel sits than a stationary one. Port of `core/squat.py`.
//!
//! Two empirical methods, Barrass II and ICORELS / PIANC, both reported; the larger is adopted.
//! The Python file holds the engineering notes. Copied exactly, quirks included: the `max` picks
//! the first of equal estimates, and the speed solver is a bisection that stops early on
//! tolerance, so its answer depends on the path taken.
//!
//! Known difference: Python raises `OverflowError` or `ZeroDivisionError` where a float power
//! overflows or underflows (a speed of 1e200, an Lpp of 1e-200); this port returns infinity or
//! NaN. Such inputs are far outside the valid range. The evaluation
//! format can carry these errors now, so a later change may port them.

use crate::checks::{Check, all_ok};
use crate::text;
use crate::text::{Arg, Text};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

pub const GRAVITY: f64 = 9.80665;

/// Under-keel clearance kept below the squatted keel: a working margin, not a standard's number.
pub const DEFAULT_SAFETY_MARGIN_M: f64 = 0.15;

/// Above this depth Froude number the ICORELS form is no longer meaningful.
pub const FNH_VALID_MAX: f64 = 0.70;

/// One vessel, one channel section, one speed. `SquatInputs.__init__` defaults are in `new`.
#[derive(Debug, Clone, PartialEq)]
pub struct SquatInputs {
    pub speed_through_water_kn: f64,
    pub draft_m: f64,
    pub beam_m: f64,
    pub lpp_m: f64,
    pub block_coefficient: f64,
    /// Midship section coefficient Cm: the section area is B x T x Cm (issue #425).
    pub midship_coefficient: f64,
    /// Bed to still-water surface.
    pub water_depth_m: f64,
    /// Wetted section of the channel; 0 = open water.
    pub channel_area_m2: f64,
    /// 0 means estimated from Cb x L x B x T.
    pub displacement_m3: f64,
    pub safety_margin_m: f64,
}

impl SquatInputs {
    /// The four required fields, with Python's defaults for the rest.
    pub fn new(speed_through_water_kn: f64, draft_m: f64, beam_m: f64, lpp_m: f64) -> Self {
        Self {
            speed_through_water_kn,
            draft_m,
            beam_m,
            lpp_m,
            block_coefficient: 0.85,
            midship_coefficient: crate::confined_water::DEFAULT_MIDSHIP_COEFFICIENT,
            water_depth_m: 0.0,
            channel_area_m2: 0.0,
            displacement_m3: 0.0,
            safety_margin_m: DEFAULT_SAFETY_MARGIN_M,
        }
    }

    /// `SquatInputs.__post_init__`: the same tests in the same order, the same messages.
    pub fn validate(&self) -> Result<(), String> {
        if self.speed_through_water_kn < 0.0 {
            return Err("speed_through_water_kn must be >= 0".to_string());
        }
        for (name, value) in [
            ("draft_m", self.draft_m),
            ("beam_m", self.beam_m),
            ("lpp_m", self.lpp_m),
        ] {
            if value <= 0.0 {
                return Err(format!("{name} must be > 0"));
            }
        }
        if !(0.3..=1.0).contains(&self.block_coefficient) {
            return Err("block_coefficient outside 0.3-1.0".to_string());
        }
        if !(self.midship_coefficient > 0.0 && self.midship_coefficient <= 1.0) {
            return Err("midship_coefficient outside 0-1".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SquatResult {
    pub method: &'static str,
    /// The adopted (larger) estimate.
    pub squat_m: f64,
    pub squat_barrass_m: f64,
    pub squat_icorels_m: Option<f64>,
    pub blockage_ratio: f64,
    pub froude_depth: f64,
    /// Water depth minus static draft.
    pub static_ukc_m: f64,
    /// What is left under the keel at speed.
    pub dynamic_ukc_m: f64,
    pub grounds: bool,
    pub checks: Vec<Check>,
    pub warnings: Vec<Text>,
}

impl SquatResult {
    pub fn ok(&self) -> bool {
        all_ok(&self.checks)
    }
}

/// `_barrass`: zero blockage is open water, where the form does not apply, and the answer is 0.
fn barrass(cb: f64, blockage: f64, v_kn: f64) -> f64 {
    if v_kn <= 0.0 || blockage <= 0.0 {
        return 0.0;
    }
    cb * libm::pow(blockage, 0.81) * libm::pow(v_kn, 2.08) / 20.0
}

/// `_icorels`: `None` above the validity limit.
fn icorels(vol_m3: f64, lpp: f64, fnh: f64) -> Option<f64> {
    if fnh <= 0.0 {
        return Some(0.0);
    }
    if fnh >= 0.99 {
        return None;
    }
    Some(
        2.4 * (vol_m3 / libm::pow(lpp, 2.0)) * libm::pow(fnh, 2.0)
            / libm::sqrt(1.0 - libm::pow(fnh, 2.0)),
    )
}

/// Python `max(a, b)`: `a` unless `b` is strictly greater.
fn py_max(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// Maximum squat by two methods; the larger is adopted. `Err` carries Python's `ValueError` text.
pub fn compute(inp: &SquatInputs) -> Result<SquatResult, String> {
    inp.validate()?;
    let v_ms = inp.speed_through_water_kn * 0.514444;
    let vol = if inp.displacement_m3 != 0.0 {
        inp.displacement_m3
    } else {
        inp.block_coefficient * inp.lpp_m * inp.beam_m * inp.draft_m
    };
    // Issue #396: the same midship section and blockage ratio as the resistance.
    let midship = crate::confined_water::midship_section_area(
        inp.beam_m,
        inp.draft_m,
        inp.midship_coefficient,
    );
    let blockage = crate::confined_water::blockage_ratio(midship, inp.channel_area_m2);
    let fnh = if inp.water_depth_m > 0.0 {
        v_ms / libm::sqrt(GRAVITY * inp.water_depth_m)
    } else {
        0.0
    };

    let s_b = barrass(inp.block_coefficient, blockage, inp.speed_through_water_kn);
    let s_i = if inp.water_depth_m > 0.0 {
        icorels(vol, inp.lpp_m, fnh)
    } else {
        None
    };

    // `max(cands, key=...)`: the first of equal estimates wins.
    let (mut method, mut squat) = ("Barrass II", s_b);
    if let Some(i) = s_i
        && i > squat
    {
        method = "ICORELS";
        squat = i;
    }

    let static_ukc = if inp.water_depth_m > 0.0 {
        inp.water_depth_m - inp.draft_m
    } else {
        f64::INFINITY
    };
    let dynamic = static_ukc - squat;

    let speed = inp.speed_through_water_kn;
    let mut checks: Vec<Check> = vec![
        Check::new(
            "keel_clears_bed",
            dynamic > 0.0,
            text!(
                "check-keel-clears-bed",
                "dynamic UKC {ukc} m = static {static_ukc} m - squat {squat} m ({method}) at {speed} kn",
                ukc = Arg::fixed(dynamic, 2),
                static_ukc = Arg::fixed(static_ukc, 2),
                squat = Arg::fixed(squat, 2),
                method = method,
                speed = Arg::fixed(speed, 2),
            ),
        ),
        Check::with_severity(
            "keel_clears_bed_with_margin",
            dynamic >= inp.safety_margin_m,
            text!(
                "check-keel-clears-bed-with-margin",
                "dynamic UKC {ukc} m against a {margin} m working margin",
                ukc = Arg::fixed(dynamic, 2),
                margin = Arg::fixed(inp.safety_margin_m, 2),
            ),
            "warning",
        ),
        Check::with_severity(
            "froude_depth_subcritical",
            fnh < FNH_VALID_MAX,
            text!(
                "check-froude-depth-subcritical",
                "depth Froude {froude}; above 0.7 the flow approaches critical and both squat forms \
                 lose validity",
                froude = Arg::fixed(fnh, 2),
            ),
            "warning",
        ),
    ];
    let mut warns: Vec<Text> = Vec::new();
    if blockage > 0.0 {
        checks.push(Check::with_severity(
            "blockage_within_barrass_range",
            blockage <= 0.35,
            text!(
                "check-blockage-within-barrass-range",
                "blockage {blockage}; Barrass is fitted below about 0.35 and extrapolates above it",
                blockage = Arg::fixed(blockage, 3),
            ),
            "warning",
        ));
    }
    if let Some(i) = s_i
        && s_b > 0.0
    {
        let spread = (i - s_b).abs() / py_max(s_b, 1e-9);
        checks.push(Check::with_severity(
            "methods_agree_within_50pct",
            spread <= 0.5,
            text!(
                "check-methods-agree-within-50pct",
                "Barrass {barrass} m vs ICORELS {icorels} m, {spread}% apart",
                barrass = Arg::fixed(s_b, 2),
                icorels = Arg::fixed(i, 2),
                spread = Arg::fixed(spread * 100.0, 0),
            ),
            "warning",
        ));
    }
    if s_i.is_none() && fnh > 0.0 {
        warns.push(text!(
            "warn-squat-icorels-not-evaluated",
            "ICORELS not evaluated: depth Froude {froude} at or above 0.99",
            froude = Arg::fixed(fnh, 2),
        ));
    }
    if inp.channel_area_m2 <= 0.0 {
        warns.push(text!(
            "warn-squat-open-water",
            "no channel area given, so blockage is 0 and Barrass returns 0 \u{2014} this is the \
             OPEN-WATER case and squat is then ICORELS only"
        ));
    }

    Ok(SquatResult {
        method,
        squat_m: squat,
        squat_barrass_m: s_b,
        squat_icorels_m: s_i,
        blockage_ratio: blockage,
        froude_depth: fnh,
        static_ukc_m: static_ukc,
        dynamic_ukc_m: dynamic,
        grounds: dynamic <= 0.0,
        checks,
        warnings: warns,
    })
}

/// The greatest speed through water that keeps `safety_margin_m` under the keel.
///
/// Bisection on [`compute`], so the governing method is used rather than one formula inverted.
/// Returns 0.0 if the vessel cannot float clear at rest. The loop stops as soon as the bracket is
/// within `tol_kn`, exactly as Python does, so the answer depends on the path taken.
#[allow(clippy::too_many_arguments)] // mirrors the Python signature
pub fn max_speed_for_ukc(
    draft_m: f64,
    beam_m: f64,
    lpp_m: f64,
    water_depth_m: f64,
    channel_area_m2: f64,
    block_coefficient: f64,
    safety_margin_m: f64,
    v_hi_kn: f64,
    tol_kn: f64,
) -> Result<f64, String> {
    let static_ukc = water_depth_m - draft_m;
    if static_ukc <= safety_margin_m {
        return Ok(0.0);
    }

    let ukc = |v: f64| -> Result<f64, String> {
        let inp = SquatInputs {
            block_coefficient,
            water_depth_m,
            channel_area_m2,
            ..SquatInputs::new(v, draft_m, beam_m, lpp_m)
        };
        Ok(compute(&inp)?.dynamic_ukc_m)
    };

    let (mut lo, mut hi) = (0.0_f64, v_hi_kn);
    if ukc(hi)? >= safety_margin_m {
        return Ok(hi);
    }
    while hi - lo > tol_kn {
        let mid = 0.5 * (lo + hi);
        if ukc(mid)? >= safety_margin_m {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Ok(lo)
}
