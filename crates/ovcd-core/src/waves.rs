//! Added resistance in waves, the sea leg. Port of `core/waves.py`.
//!
//! STAwave-1, ITTC's simple estimate for head waves. The Python file holds the engineering notes.

use crate::checks::{Check, all_ok};
use crate::pyfmt::fixed;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub const GRAVITY: f64 = 9.80665;
pub const SEAWATER_DENSITY_T_M3: f64 = 1.025;

/// ITTC's stated validity for STAwave-1: waves short relative to the hull.
pub const LAMBDA_OVER_LPP_MAX: f64 = 0.5;

#[derive(Debug, Clone, PartialEq)]
pub struct WaveResistanceInputs {
    pub significant_wave_height_m: f64,
    pub beam_waterline_m: f64,
    pub lpp_m: f64,
    /// Bow length on the waterline; 0 means 0.2 x Lpp (a blunt inland barge bow).
    pub bow_waterline_length_m: f64,
    pub heading_deg_off_bow: f64,
    /// 0 means estimated from the wave height.
    pub peak_wave_period_s: f64,
    pub water_density_t_m3: f64,
    /// Carried for reporting; STAwave-1 is speed-free.
    pub speed_kn: f64,
}

impl WaveResistanceInputs {
    /// The three required fields, with Python's defaults for the rest.
    pub fn new(significant_wave_height_m: f64, beam_waterline_m: f64, lpp_m: f64) -> Self {
        Self {
            significant_wave_height_m,
            beam_waterline_m,
            lpp_m,
            bow_waterline_length_m: 0.0,
            heading_deg_off_bow: 0.0,
            peak_wave_period_s: 0.0,
            water_density_t_m3: SEAWATER_DENSITY_T_M3,
            speed_kn: 0.0,
        }
    }

    /// `WaveResistanceInputs.__post_init__`: same tests, same order, same messages.
    pub fn validate(&self) -> Result<(), String> {
        if self.significant_wave_height_m < 0.0 {
            return Err("significant_wave_height_m must be >= 0".to_string());
        }
        for (name, value) in [
            ("beam_waterline_m", self.beam_waterline_m),
            ("lpp_m", self.lpp_m),
        ] {
            if value <= 0.0 {
                return Err(format!("{name} must be > 0"));
            }
        }
        if !(0.0..=180.0).contains(&self.heading_deg_off_bow) {
            return Err("heading_deg_off_bow must be 0-180".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaveResistanceResult {
    pub added_resistance_kn: f64,
    /// Before the heading factor.
    pub added_resistance_head_seas_kn: f64,
    pub heading_factor: f64,
    pub bow_waterline_length_m: f64,
    pub wavelength_m: f64,
    pub lambda_over_lpp: f64,
    /// At the quoted speed, effective (thrust) power.
    pub added_power_kw: f64,
    pub checks: Vec<Check>,
    pub warnings: Vec<String>,
}

impl WaveResistanceResult {
    pub fn ok(&self) -> bool {
        all_ok(&self.checks)
    }
}

/// `_peak_period`: a fetch-limited-sea estimate, T_p ~ 4.0 x sqrt(H_s).
fn peak_period(hs: f64) -> f64 {
    if hs > 0.0 { 4.0 * libm::sqrt(hs) } else { 0.0 }
}

/// Python `a or b` on floats: `b` when `a` is zero.
fn or_default(a: f64, b: f64) -> f64 {
    if a != 0.0 { a } else { b }
}

/// STAwave-1 added resistance. `Err` carries Python's `ValueError` text.
pub fn compute(inp: &WaveResistanceInputs) -> Result<WaveResistanceResult, String> {
    inp.validate()?;
    let hs = inp.significant_wave_height_m;
    let b = inp.beam_waterline_m;
    let lbwl = or_default(inp.bow_waterline_length_m, 0.2 * inp.lpp_m);
    let rho = inp.water_density_t_m3 * 1000.0; // t/m3 -> kg/m3

    // STAwave-1, head seas. rho*g*Hs^2*B in N; /1000 -> kN
    let r_head =
        (1.0 / 16.0) * rho * GRAVITY * libm::pow(hs, 2.0) * b * libm::sqrt(b / lbwl) / 1000.0;

    // `math.radians`: x * (pi / 180).
    let mu = inp.heading_deg_off_bow * (core::f64::consts::PI / 180.0);
    // cos(90 deg) is 6.1e-17, not 0. Snap it, as Python does: abeam the term is zero.
    let c = libm::cos(mu);
    let hf = if c <= 1e-12 { 0.0 } else { libm::pow(c, 2.0) };
    let r = r_head * hf;

    let tp = or_default(inp.peak_wave_period_s, peak_period(hs));
    let lam = if tp > 0.0 {
        (GRAVITY * libm::pow(tp, 2.0)) / (2.0 * core::f64::consts::PI)
    } else {
        0.0
    };
    let lol = if inp.lpp_m > 0.0 {
        lam / inp.lpp_m
    } else {
        0.0
    };

    let v_ms = inp.speed_kn * 0.514444;
    let added_kw = r * v_ms; // kN * m/s = kW, effective (thrust) power

    let mut checks: Vec<Check> = Vec::new();
    let mut warns: Vec<String> = Vec::new();

    checks.push(Check::with_severity(
        "stawave1_short_wave_validity",
        0.0 < lol && lol < LAMBDA_OVER_LPP_MAX,
        format!(
            "lambda/Lpp = {} (lambda {} m on Lpp {} m). STAwave-1 assumes short waves, \
             lambda/Lpp < 0.5; outside that it UNDER-predicts, and near lambda/Lpp = 1 it can \
             do so by more than 2x",
            fixed(lol, 2),
            fixed(lam, 0),
            fixed(inp.lpp_m, 0)
        ),
        "warning",
    ));
    checks.push(Check::with_severity(
        "wave_height_within_screening_range",
        hs <= 3.0,
        format!(
            "H_s {} m; above about 3 m a screening formula is not the right tool and voluntary \
             speed reduction usually governs anyway",
            fixed(hs, 2)
        ),
        "warning",
    ));
    checks.push(Check::with_severity(
        "bow_length_supplied",
        inp.bow_waterline_length_m > 0.0,
        format!(
            "L_BWL not given, assumed {} m = 0.2 x Lpp (a blunt barge bow). A finer bow reduces \
             added resistance, so this is the conservative assumption",
            fixed(lbwl, 1)
        ),
        "warning",
    ));
    if inp.peak_wave_period_s <= 0.0 && hs > 0.0 {
        warns.push(format!(
            "no wave period given; T_p estimated at {} s from H_s, which sets only the validity \
             check, not the resistance",
            fixed(tp, 1)
        ));
    }
    if inp.heading_deg_off_bow > 60.0 {
        warns.push(
            "beam-ish seas: this module returns a FLOOR, because drift, rudder and \
             course-keeping loss is not modelled and it dominates off the bow"
                .to_string(),
        );
    }

    Ok(WaveResistanceResult {
        added_resistance_kn: r,
        added_resistance_head_seas_kn: r_head,
        heading_factor: hf,
        bow_waterline_length_m: lbwl,
        wavelength_m: lam,
        lambda_over_lpp: lol,
        added_power_kw: added_kw,
        checks,
        warnings: warns,
    })
}
