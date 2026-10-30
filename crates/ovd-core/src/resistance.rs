//! Hull resistance, service speed and fuel efficiency for the ATB barge. Port of
//! `core/resistance.py`.
//!
//! ITTC 1957 friction, a residuary fit for bluff bodies, Schijf's limiting speed and the Lackenby
//! shallow-water increase for a confined channel, plus the added resistance in waves
//! ([`crate::waves`]) and the squat limit ([`crate::squat`]). The Python file holds the
//! engineering notes. The port has been changed since, each change with its issue: one midship
//! section and the Schijf limit (#396, #397), a residuary fit continuous at Fn 0.20 (#399), the
//! speed governing labels (#400).
//!
//! g and the knot come from [`crate::constants`], the same values as [`crate::squat`] and
//! [`crate::waves`] (issue #526). The Python original used g = 9.81 m/s^2 and 1 kn = 0.5144 m/s.
//!
//! Python behaviours copied exactly:
//! - the speed grid is built by repeated addition, then `round(s, 1)` (half to even, on the exact
//!   binary value): a step of 0.25 gives 2.0, 2.2, 2.5, 2.8, not 2.25 or 2.75;
//! - `min` over environments returns the first of equal items;
//! - `math.sqrt` of a negative raises `ValueError("math domain error")` and a float `/` by zero
//!   raises `ZeroDivisionError("float division by zero")` (Python 3.12 wording; 3.14 says
//!   "division by zero");
//! - a squat or wave `ValueError` from a speed point comes out unchanged.
//!
//! Known differences from Python, on inputs far outside any valid range:
//! Python raises `OverflowError` where a float `**` overflows, and this port returns infinity; a
//! speed range with an infinite top or a step of zero or less never ends in Python (it runs out of
//! memory) and never ends here either.

use crate::aft_arrangement::PyException;
use crate::checks::{Check, all_ok};
use crate::confined_water::LIMITING_SPEED_MARGIN;
use crate::holtrop_mennen::{self, Envelope, Flow, HullInputs, Variant};
use crate::pyfmt::round_to;
use crate::squat::{self, SquatInputs};
use crate::text;
use crate::text::{Arg, Text};
use crate::waves::{self, WaveResistanceInputs};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// The depth Froude number up to which the Lackenby shallow-water term is used; above it a fixed
/// factor of 5 stands in (an OVD choice, no source).
pub const LACKENBY_FNH_MAX: f64 = 0.95;

// --- Physical constants ---
pub use crate::constants::{GRAVITY, KNOTS_TO_MS};
pub const KW_PER_KN_MS: f64 = 1.0; // 1 kN x 1 m/s = 1 kW

/// Tug propulsion characteristics.
#[derive(Debug, Clone, PartialEq)]
pub struct TugInputs {
    /// Total installed power (kW).
    pub engine_power_kw: f64,
    /// Overall propulsive coefficient.
    pub propulsive_efficiency: f64,
    /// Specific fuel consumption (g/kWh).
    pub sfc_g_per_kwh: f64,
    /// Marine diesel density.
    pub fuel_density_kg_per_litre: f64,
}

impl Default for TugInputs {
    fn default() -> Self {
        Self {
            engine_power_kw: 2400.0,
            propulsive_efficiency: 0.45,
            sfc_g_per_kwh: 195.0,
            fuel_density_kg_per_litre: 0.845,
        }
    }
}

/// Channel and waterway parameters for one operating environment.
#[derive(Debug, Clone, PartialEq)]
pub struct EnvironmentInputs {
    pub name: String,
    pub water_depth: f64,
    /// Bottom width of the trapezoidal channel (m).
    pub channel_width_bottom: f64,
    pub channel_width_surface: f64,
    /// Horizontal : vertical.
    pub bank_slope: f64,
    /// Opposing current (knots).
    pub current_speed_knots: f64,
    /// t/m^3.
    pub water_density: f64,
    pub is_deep_water: bool,
    /// Significant wave height. Zero in a river.
    pub significant_wave_height_m: f64,
    pub wave_heading_deg_off_bow: f64,
    pub peak_wave_period_s: f64,
    /// Under-keel clearance to keep below the squatted keel.
    pub squat_safety_margin_m: f64,
    /// False skips the squat limit.
    pub apply_squat_limit: bool,
}

impl Default for EnvironmentInputs {
    fn default() -> Self {
        Self {
            name: "Open Sea".to_string(),
            water_depth: 50.0,
            channel_width_bottom: 1000.0,
            channel_width_surface: 1000.0,
            bank_slope: 3.0,
            current_speed_knots: 0.0,
            water_density: 1.025,
            is_deep_water: false,
            significant_wave_height_m: 0.0,
            wave_heading_deg_off_bow: 0.0,
            peak_wave_period_s: 0.0,
            squat_safety_margin_m: 0.15,
            apply_squat_limit: true,
        }
    }
}

/// `default_environments`: the three standard operating environments.
pub fn default_environments() -> Vec<EnvironmentInputs> {
    vec![
        EnvironmentInputs {
            name: "Open Sea".to_string(),
            water_depth: 50.0,
            channel_width_bottom: 1000.0,
            channel_width_surface: 1000.0,
            current_speed_knots: 0.0,
            water_density: 1.025,
            is_deep_water: true,
            ..EnvironmentInputs::default()
        },
        EnvironmentInputs {
            name: "Confined shallow river".to_string(),
            water_depth: 6.5,
            channel_width_bottom: 40.0,
            channel_width_surface: 60.0,
            bank_slope: 3.0,
            current_speed_knots: 1.0,
            water_density: 1.0,
            ..EnvironmentInputs::default()
        },
        EnvironmentInputs {
            name: "Confined creek".to_string(),
            water_depth: 8.0,
            channel_width_bottom: 60.0,
            channel_width_surface: 100.0,
            bank_slope: 3.0,
            current_speed_knots: 0.5,
            water_density: 1.0,
            ..EnvironmentInputs::default()
        },
    ]
}

/// How the calm-water residuary resistance is worked out (issue #723, ADR 0023).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResistanceMethod {
    /// ITTC-1957 friction x (1 + Cr/Cf(Fn)), the bluff-barge fit [`residuary_ratio`]: no hull-form
    /// term. The method of every Design before issue #723, and still the default.
    #[default]
    BargeFit,
    /// Holtrop 1984 ([`crate::holtrop_mennen`]) at any fullness, with a warning outside its range.
    HoltropMennen,
    /// Holtrop 1984 up to C_P [`FORM_METHOD_CP_MAX`], the bluff-barge fit above it.
    ByBlockCoefficient,
    /// Issue #735: ITTC-1957 friction x (1 + k1 + dk + w(Fn)): the Holtrop 1984 form factor, the
    /// Millward (1989) shallow-water increase of k, and the Froude-number rise of the bluff-barge
    /// fit for the waves. No fitted constant; checked against barge towing-tank data
    /// (`calibration/resistance_cases.csv`), see [`BARGE_FORM_FACTOR_ENVELOPE`].
    BargeFormFactor,
}

/// The values of the `resistance_method` input, in the order of [`ResistanceMethod`].
pub const RESISTANCE_METHODS: &[&str] = &[
    "barge_fit",
    "holtrop_mennen",
    "by_block_coefficient",
    "barge_form_factor",
];

impl ResistanceMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BargeFit => "barge_fit",
            Self::HoltropMennen => "holtrop_mennen",
            Self::ByBlockCoefficient => "by_block_coefficient",
            Self::BargeFormFactor => "barge_form_factor",
        }
    }

    /// The method of an input value; `None` for a value that names none.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "barge_fit" => Some(Self::BargeFit),
            "holtrop_mennen" => Some(Self::HoltropMennen),
            "by_block_coefficient" => Some(Self::ByBlockCoefficient),
            "barge_form_factor" => Some(Self::BargeFormFactor),
            _ => None,
        }
    }
}

/// The prismatic coefficient up to which `by_block_coefficient` uses Holtrop–Mennen: the top of
/// its full-form range ([`holtrop_mennen::HOLTROP_FULL_FORM`]).
pub const FORM_METHOD_CP_MAX: f64 = 0.85;

/// The declared range of the bluff-barge fit: C_P from 0.85 to 1. An Estimate (no source): the fit
/// was set for bluff barges, Cb about 0.9 to 1, and has no hull-form term, so it cannot rank
/// finer hulls.
pub const BARGE_FIT_ENVELOPE: Envelope = Envelope {
    froude_max: None,
    cp: (0.85, 1.0),
    l_over_b: (0.0, f64::INFINITY),
    b_over_t: (0.0, f64::INFINITY),
};

/// The declared range of `barge_form_factor` (issue #735): the range of the towing-tank cases it
/// was checked against (`calibration/resistance_cases.csv`), rounded out at the third figure: Fn up
/// to 0.137, C_P 0.768 to 0.877, L/B 7.59 to 16.29, B/T 4.05 to 8.12, depth over draught 1.2 and more. Sourced: Zentari (2023),
/// Figs 5.9 and 5.10; Tabaczek and Zawislak (2018), Table 1 and Fig. 4. Inside it the method's CT
/// fell 0 to 36 % below the tests (mean about 17 % below); see `calibration/resistance_results.md`.
pub const BARGE_FORM_FACTOR_ENVELOPE: Envelope = Envelope {
    froude_max: Some(0.137),
    cp: (0.768, 0.877),
    l_over_b: (7.59, 16.29),
    b_over_t: (4.05, 8.12),
};

/// Millward (1989), "The effect of water depth on hull form factor", International Shipbuilding
/// Progress 36(407): the increase of the form factor k in water of depth `depth` over a hull of
/// draught `draught`, dk = 0.644 (T/h)^1.72. Zero in deep water (`depth` not above 0).
pub fn millward_form_factor_increase(draught: f64, depth: f64) -> f64 {
    if depth <= 0.0 || draught <= 0.0 {
        return 0.0;
    }
    0.644 * libm::pow(draught / depth, 1.72)
}

/// Issue #735: CT / CF of `barge_form_factor`: 1 + k1 (Holtrop 1984, from `form_factor`) + dk
/// (Millward, from the draught and the depth; 0 in deep water) + the rise of the bluff-barge fit
/// above its low-speed value, residuary_ratio(Fn) - residuary_ratio(0), for the waves.
pub fn barge_form_factor_ratio(form_factor: f64, froude: f64, draught: f64, depth: f64) -> f64 {
    form_factor
        + millward_form_factor_increase(draught, depth)
        + (residuary_ratio(froude) - residuary_ratio(0.0))
}

/// The method that gives the resistance of one hull: the bluff-barge fit, Holtrop 1984 or the
/// barge form factor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppliedMethod {
    BargeFit,
    Holtrop1984,
    BargeFormFactor,
}

impl AppliedMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BargeFit => "bluff-barge fit",
            Self::Holtrop1984 => "Holtrop-Mennen 1984",
            Self::BargeFormFactor => "barge form factor",
        }
    }

    /// The declared validity range at a prismatic coefficient.
    pub fn envelope(self, cp: f64) -> Envelope {
        match self {
            Self::BargeFit => BARGE_FIT_ENVELOPE,
            Self::Holtrop1984 => holtrop_mennen::holtrop_envelope(cp),
            Self::BargeFormFactor => BARGE_FORM_FACTOR_ENVELOPE,
        }
    }
}

/// All inputs for the resistance calculation.
#[derive(Debug, Clone, PartialEq)]
pub struct ResistanceInputs {
    pub loa: f64,
    /// Waterline length (m).
    pub lwl: f64,
    pub beam: f64,
    pub draft: f64,
    pub block_coefficient: f64,
    /// Midship section coefficient Cm: the section area is B x T x Cm (issue #425).
    pub midship_coefficient: f64,
    /// m^2; 0 = auto-calculate.
    pub wetted_surface_area: f64,
    /// Issue #350: wetted surface the Stern Form adds over the box (m^2), added to the computed one.
    pub wetted_surface_extra_m2: f64,
    /// Tonnes; 0 = auto from Cb. Carried, not used.
    pub displacement_tonnes: f64,
    /// Issue #448: the hull's own midship section area (m²), for the blockage and squat; `None`:
    /// B x T.
    pub midship_area_m2: Option<f64>,
    pub tug: TugInputs,
    pub environments: Vec<EnvironmentInputs>,
    pub speed_min_knots: f64,
    pub speed_max_knots: f64,
    pub speed_step_knots: f64,
    /// Issue #723: the residuary resistance method. The default keeps the bluff-barge fit.
    pub method: ResistanceMethod,
    /// Issue #723: the hull's waterplane coefficient, for Holtrop–Mennen; `None`: (1 + 2 Cb) / 3.
    pub waterplane_coefficient: Option<f64>,
}

impl Default for ResistanceInputs {
    fn default() -> Self {
        Self {
            loa: 130.0,
            lwl: 125.0,
            beam: 24.0,
            draft: 6.0,
            block_coefficient: 0.85,
            midship_coefficient: crate::confined_water::DEFAULT_MIDSHIP_COEFFICIENT,
            wetted_surface_area: 0.0,
            wetted_surface_extra_m2: 0.0,
            displacement_tonnes: 0.0,
            midship_area_m2: None,
            tug: TugInputs::default(),
            environments: default_environments(),
            speed_min_knots: 2.0,
            speed_max_knots: 12.0,
            speed_step_knots: 0.5,
            method: ResistanceMethod::BargeFit,
            waterplane_coefficient: None,
        }
    }
}

// --- Results ---

/// Resistance and power at one speed in one environment.
#[derive(Debug, Clone, PartialEq)]
pub struct SpeedPoint {
    /// Speed over ground.
    pub speed_knots: f64,
    /// Speed over ground plus the current.
    pub speed_through_water_knots: f64,
    pub froude_number: f64,
    pub froude_depth: f64,
    pub resistance_frictional_kn: f64,
    pub resistance_residuary_kn: f64,
    /// Multiplier from shallow water.
    pub resistance_shallow_factor: f64,
    pub resistance_total_kn: f64,
    pub effective_power_kw: f64,
    pub delivered_power_kw: f64,
    pub fuel_litres_per_hour: f64,
    pub fuel_litres_per_nm: f64,
    /// Delivered power within the engine power.
    pub power_available: bool,
    /// STAwave-1 added resistance.
    pub resistance_waves_kn: f64,
    pub squat_m: f64,
    /// Clearance left under the keel at this speed; infinity where squat does not apply.
    pub dynamic_ukc_m: f64,
    /// True where squat, not power, forbids this speed.
    pub squat_limited: bool,
}

/// Resistance results for one operating environment.
#[derive(Debug, Clone, PartialEq)]
pub struct EnvironmentResult {
    pub name: String,
    /// Schijf critical speed; `None` for deep water.
    pub limiting_speed_knots: Option<f64>,
    pub service_speed_knots: f64,
    /// Am / Ac.
    pub blockage_ratio: f64,
    pub speed_curve: Vec<SpeedPoint>,
    pub resistance_at_service_kn: f64,
    pub power_at_service_kw: f64,
    pub fuel_at_service_lph: f64,
    pub fuel_at_service_lpnm: f64,
    /// Fastest speed keeping the margin.
    pub squat_limited_speed_knots: Option<f64>,
    /// `power`, `squat` or `channel`.
    pub service_speed_governed_by: &'static str,
}

/// Complete resistance calculation output.
#[derive(Debug, Clone, PartialEq)]
pub struct ResistanceResults {
    pub inputs: ResistanceInputs,
    /// Issue #723: the method that gave the residuary resistance of this hull.
    pub method_used: AppliedMethod,
    pub wetted_surface_area: f64,
    pub environments: Vec<EnvironmentResult>,
    pub checks: Vec<Check>,
    pub warnings: Vec<Text>,
}

impl ResistanceResults {
    /// True if no error-severity check has failed (warnings do not gate).
    pub fn ok(&self) -> bool {
        all_ok(&self.checks)
    }

    /// The environment with the lowest service speed; the first of equal ones (Python `min`).
    pub fn binding_environment(&self) -> Option<&EnvironmentResult> {
        let mut best: Option<&EnvironmentResult> = None;
        for e in &self.environments {
            match best {
                Some(b) if e.service_speed_knots < b.service_speed_knots => best = Some(e),
                Some(_) => {}
                None => best = Some(e),
            }
        }
        best
    }

    /// Delivered power at service speed in the binding environment; 0 with no environments.
    pub fn binding_power_at_service_kw(&self) -> f64 {
        self.binding_environment()
            .map_or(0.0, |e| e.power_at_service_kw)
    }
}

/// `math.sqrt`: a negative argument raises `ValueError`.
fn py_sqrt(x: f64) -> Result<f64, PyException> {
    if x < 0.0 {
        Err(PyException::ValueError("math domain error".to_string()))
    } else {
        Ok(libm::sqrt(x))
    }
}

/// Float `a / b`: a zero divisor raises `ZeroDivisionError`.
fn py_div(a: f64, b: f64) -> Result<f64, PyException> {
    if b == 0.0 {
        Err(PyException::ZeroDivisionError(
            "float division by zero".to_string(),
        ))
    } else {
        Ok(a / b)
    }
}

/// Python `min(a, b)`: `a` unless `b` is strictly smaller.
fn py_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

fn squat_applies(env: &EnvironmentInputs) -> bool {
    env.apply_squat_limit && env.water_depth > 0.0 && !env.is_deep_water
}

/// The residuary to frictional resistance ratio Cr/Cf of a bluff-body barge at Froude number
/// `fn_`: an Estimate, fitted, not from a source. 0.10 below Fn 0.10, rising as (Fn - 0.10)^1.5 to
/// Fn 0.20, then as (Fn - 0.20)^2. The last part starts where the middle one ends, so the fit is
/// continuous (issue #399): it began at 0.15, a step down from about 0.179. The higher start is
/// the conservative one of the two ways to close the step.
pub fn residuary_ratio(fn_: f64) -> f64 {
    let at_0_20 = 0.10 + 2.5 * libm::pow(0.10, 1.5);
    if fn_ < 0.10 {
        0.10
    } else if fn_ < 0.20 {
        0.10 + 2.5 * libm::pow(fn_ - 0.10, 1.5)
    } else {
        at_0_20 + 8.0 * libm::pow(fn_ - 0.20, 2.0)
    }
}

/// Hull resistance, power and fuel consumption calculator.
pub struct ResistanceCalculator<'a> {
    pub inputs: &'a ResistanceInputs,
}

impl<'a> ResistanceCalculator<'a> {
    pub fn new(inputs: &'a ResistanceInputs) -> Self {
        Self { inputs }
    }

    /// Midship section coefficient: the midship area over B x T.
    fn cm(&self) -> f64 {
        let inp = self.inputs;
        self.midship_section_area() / (inp.beam * inp.draft)
    }

    /// Prismatic coefficient Cb / Cm.
    pub fn prismatic(&self) -> f64 {
        self.inputs.block_coefficient / self.cm()
    }

    /// The method that gives the residuary resistance of this hull (issue #723).
    pub fn applied_method(&self) -> AppliedMethod {
        match self.inputs.method {
            ResistanceMethod::BargeFit => AppliedMethod::BargeFit,
            ResistanceMethod::HoltropMennen => AppliedMethod::Holtrop1984,
            ResistanceMethod::BargeFormFactor => AppliedMethod::BargeFormFactor,
            ResistanceMethod::ByBlockCoefficient => {
                if self.prismatic() <= FORM_METHOD_CP_MAX {
                    AppliedMethod::Holtrop1984
                } else {
                    AppliedMethod::BargeFit
                }
            }
        }
    }

    /// The hull as Holtrop–Mennen reads it: no bulb, no immersed transom, no appendages, a normal
    /// afterbody, lcb estimated from C_P, the given wetted surface (issue #723).
    fn holtrop_hull(&self, wetted_surface: f64) -> HullInputs {
        let inp = self.inputs;
        let cb = inp.block_coefficient;
        let cwp = inp
            .waterplane_coefficient
            .unwrap_or_else(|| holtrop_mennen::cwp_estimate(cb));
        let mut h = HullInputs::plain(
            inp.lwl,
            inp.beam,
            inp.draft,
            cb * inp.lwl * inp.beam * inp.draft,
            self.cm(),
            cwp,
        );
        h.wetted_surface_m2 = wetted_surface;
        h
    }

    /// Wetted surface of a box barge: bottom plus two sides, times a 1.02 rake form factor.
    fn compute_wetted_surface(&self) -> f64 {
        let inp = self.inputs;
        let s = inp.lwl * inp.beam + 2.0 * inp.lwl * inp.draft;
        s * 1.02
    }

    /// Kinematic viscosity of water (m^2/s).
    fn kinematic_viscosity(&self, water_density: f64) -> f64 {
        if water_density >= 1.02 {
            1.19e-6
        } else {
            1.14e-6
        }
    }

    /// ITTC 1957: Cf = 0.075 / (log10(Rn) - 2)^2.
    fn ittc_1957_cf(&self, speed_ms: f64, lwl: f64, nu: f64) -> f64 {
        let rn = speed_ms * lwl / nu;
        if rn < 1e5 {
            return 0.008; // Laminar fallback
        }
        0.075 / libm::pow(libm::log10(rn) - 2.0, 2.0)
    }

    /// Frictional resistance (kN).
    fn frictional_resistance(
        &self,
        speed_ms: f64,
        wetted_surface: f64,
        water_density_kg_m3: f64,
        nu: f64,
    ) -> f64 {
        let cf = self.ittc_1957_cf(speed_ms, self.inputs.lwl, nu);
        let rf = 0.5 * water_density_kg_m3 * libm::pow(speed_ms, 2.0) * wetted_surface * cf;
        rf / 1000.0 // N to kN
    }

    /// Residuary resistance for a bluff-body barge (kN): Cr/Cf as an empirical function of Fn.
    fn residuary_resistance(&self, speed_ms: f64, frictional_kn: f64) -> Result<f64, PyException> {
        let fn_ = py_div(speed_ms, py_sqrt(GRAVITY * self.inputs.lwl)?)?;
        Ok(frictional_kn * residuary_ratio(fn_))
    }

    /// Trapezoidal channel cross-section area (m^2).
    fn channel_cross_section(&self, env: &EnvironmentInputs) -> f64 {
        (env.channel_width_bottom + env.channel_width_surface) / 2.0 * env.water_depth
    }

    /// Midship section area of the vessel (m^2): the hull's own (issue #448), or B x T x Cm, as
    /// squat has it (issues #396, #425).
    fn midship_section_area(&self) -> f64 {
        self.inputs
            .midship_area_m2
            .unwrap_or(crate::confined_water::midship_section_area(
                self.inputs.beam,
                self.inputs.draft,
                self.inputs.midship_coefficient,
            ))
    }

    /// Schijf (1949) critical speed in a confined waterway (knots); `None` for deep water.
    fn schijf_limiting_speed(&self, env: &EnvironmentInputs) -> Result<Option<f64>, PyException> {
        if env.is_deep_water {
            return Ok(None);
        }
        let ac = self.channel_cross_section(env);
        let am = self.midship_section_area();
        if ac <= 0.0 || am >= ac {
            return Ok(Some(0.0));
        }
        // Issue #397: the first critical speed of Schijf, on the water depth.
        let n = crate::confined_water::blockage_ratio(am, ac);
        let fnh_crit = crate::confined_water::critical_froude_depth(n);
        let v_limit = fnh_crit * py_sqrt(GRAVITY * env.water_depth)?;
        Ok(Some(v_limit / KNOTS_TO_MS))
    }

    /// Shallow and confined water resistance factor: Lackenby depth term plus a blockage term.
    /// Public for the calibration harness (issue #735).
    pub fn shallow_water_factor(
        &self,
        speed_ms: f64,
        env: &EnvironmentInputs,
    ) -> Result<f64, PyException> {
        if env.is_deep_water {
            return Ok(1.0);
        }
        let h = env.water_depth;
        let t = self.inputs.draft;
        if h <= 0.0 || t <= 0.0 {
            return Ok(1.0);
        }
        let fnh = py_div(speed_ms, py_sqrt(GRAVITY * h)?)?;
        let th_ratio = t / h;

        // Lackenby: delta = a * (T/h)^b * Fnh^c, fitted a = 1.4, b = 1.5, c = 2.5
        let lackenby = if fnh < LACKENBY_FNH_MAX {
            1.4 * libm::pow(th_ratio, 1.5) * libm::pow(fnh, 2.5)
        } else {
            5.0 // near critical speed
        };

        // Blockage correction
        let ac = self.channel_cross_section(env);
        let am = self.midship_section_area();
        let blockage = if ac > 0.0 {
            let n = am / ac;
            2.0 * n * libm::pow(fnh, 2.0)
        } else {
            0.0
        };
        Ok(1.0 + lackenby + blockage)
    }

    /// Fuel in litres per hour and litres per nautical mile.
    fn fuel_consumption(
        &self,
        delivered_power_kw: f64,
        speed_knots: f64,
    ) -> Result<(f64, f64), PyException> {
        let tug = &self.inputs.tug;
        let fuel_g_per_hr = delivered_power_kw * tug.sfc_g_per_kwh;
        let fuel_l_per_hr = py_div(fuel_g_per_hr, tug.fuel_density_kg_per_litre * 1000.0)?;
        let fuel_l_per_nm = if speed_knots > 0.0 {
            fuel_l_per_hr / speed_knots
        } else {
            0.0
        };
        Ok((fuel_l_per_hr, fuel_l_per_nm))
    }

    /// Resistance and power at one speed in one environment.
    fn compute_speed_point(
        &self,
        speed_knots: f64,
        env: &EnvironmentInputs,
        wetted_surface: f64,
    ) -> Result<SpeedPoint, PyException> {
        let speed_over_ground = speed_knots;
        let speed_through_water = speed_knots + env.current_speed_knots;
        let v = speed_through_water * KNOTS_TO_MS; // m/s

        if v <= 0.0 {
            return Ok(SpeedPoint {
                speed_knots,
                speed_through_water_knots: speed_through_water,
                froude_number: 0.0,
                froude_depth: 0.0,
                resistance_frictional_kn: 0.0,
                resistance_residuary_kn: 0.0,
                resistance_shallow_factor: 1.0,
                resistance_total_kn: 0.0,
                effective_power_kw: 0.0,
                delivered_power_kw: 0.0,
                fuel_litres_per_hour: 0.0,
                fuel_litres_per_nm: 0.0,
                power_available: true,
                resistance_waves_kn: 0.0,
                squat_m: 0.0,
                dynamic_ukc_m: f64::INFINITY,
                squat_limited: false,
            });
        }

        let fn_ = py_div(v, py_sqrt(GRAVITY * self.inputs.lwl)?)?;
        let fnh = py_div(v, py_sqrt(GRAVITY * env.water_depth)?)?;

        let nu = self.kinematic_viscosity(env.water_density);
        let rho = env.water_density * 1000.0; // t/m^3 to kg/m^3

        // Deep-water resistance components
        let rf = self.frictional_resistance(v, wetted_surface, rho, nu);
        let rr = match self.applied_method() {
            AppliedMethod::BargeFit => self.residuary_resistance(v, rf)?,
            // Issue #723: everything Holtrop–Mennen adds to the ITTC-1957 friction.
            AppliedMethod::Holtrop1984 => {
                let flow = Flow {
                    speed_ms: v,
                    rho,
                    nu,
                    g: GRAVITY,
                };
                let hull = self.holtrop_hull(wetted_surface);
                let total = holtrop_mennen::components(&hull, flow, Variant::Hm1984).total_kn;
                if !total.is_finite() {
                    // A hull this full leaves 1 - Cp - 0.0225 lcb (the entrance angle) or 1 - Cp
                    // (the form factor) at or below zero, where the formulas have no value.
                    return Err(PyException::ValueError(format!(
                        "Holtrop-Mennen has no value for this hull (Cp {:.3}, lcb {:.2} % of L): \
                         it needs 1 - Cp - 0.0225 lcb > 0; use resistance_method \
                         by_block_coefficient, which takes the bluff-barge fit above Cp {}",
                        hull.cp(),
                        hull.lcb_pct,
                        FORM_METHOD_CP_MAX
                    )));
                }
                total - rf
            }
            // Issue #735: (1 + k1 + dk + w(Fn)) x the ITTC-1957 friction.
            AppliedMethod::BargeFormFactor => {
                let flow = Flow {
                    speed_ms: v,
                    rho,
                    nu,
                    g: GRAVITY,
                };
                let hull = self.holtrop_hull(wetted_surface);
                let k1 = holtrop_mennen::components(&hull, flow, Variant::Hm1984).form_factor;
                if !k1.is_finite() {
                    return Err(PyException::ValueError(format!(
                        "the barge form factor has no value for this hull (Cp {:.3}): the \
                         Holtrop form factor needs Cp below 1",
                        hull.cp()
                    )));
                }
                let depth = if env.is_deep_water {
                    0.0
                } else {
                    env.water_depth
                };
                rf * (barge_form_factor_ratio(k1, fn_, self.inputs.draft, depth) - 1.0)
            }
        };
        let r_deep = rf + rr;

        // Shallow/confined correction
        let shallow_factor = self.shallow_water_factor(v, env)?;
        let mut r_total = r_deep * shallow_factor;

        // Added resistance in waves. Zero in a river.
        let mut r_waves = 0.0;
        if env.significant_wave_height_m > 0.0 {
            r_waves = waves::compute(&self.wave_inputs(env, speed_through_water))
                .map_err(PyException::ValueError)?
                .added_resistance_kn;
        }
        r_total += r_waves;

        // Squat: a depth term. It decides whether this speed is available at all.
        let mut sq = 0.0;
        let mut dyn_ukc = f64::INFINITY;
        let mut squat_limited = false;
        if squat_applies(env) {
            let sr = squat::compute(&self.squat_inputs(env, speed_through_water))
                .map_err(PyException::ValueError)?;
            sq = sr.squat_m;
            dyn_ukc = sr.dynamic_ukc_m;
            squat_limited = dyn_ukc < env.squat_safety_margin_m;
        }

        // Power
        let pe = r_total * v; // kW (kN x m/s)
        let eta = self.inputs.tug.propulsive_efficiency;
        let pd = if eta > 0.0 { pe / eta } else { pe };

        // Fuel
        let (lph, lpnm) = self.fuel_consumption(pd, speed_over_ground)?;

        Ok(SpeedPoint {
            speed_knots,
            speed_through_water_knots: speed_through_water,
            froude_number: fn_,
            froude_depth: fnh,
            resistance_frictional_kn: rf,
            resistance_residuary_kn: rr,
            resistance_shallow_factor: shallow_factor,
            resistance_total_kn: r_total,
            effective_power_kw: pe,
            delivered_power_kw: pd,
            fuel_litres_per_hour: lph,
            fuel_litres_per_nm: lpnm,
            power_available: pd <= self.inputs.tug.engine_power_kw,
            resistance_waves_kn: r_waves,
            squat_m: sq,
            dynamic_ukc_m: dyn_ukc,
            squat_limited,
        })
    }

    /// Full speed sweep for one environment.
    /// The STAwave-1 inputs of one environment at a speed through the water.
    fn wave_inputs(
        &self,
        env: &EnvironmentInputs,
        speed_through_water: f64,
    ) -> WaveResistanceInputs {
        WaveResistanceInputs {
            heading_deg_off_bow: env.wave_heading_deg_off_bow,
            peak_wave_period_s: env.peak_wave_period_s,
            water_density_t_m3: env.water_density,
            speed_kn: speed_through_water,
            ..WaveResistanceInputs::new(
                env.significant_wave_height_m,
                self.inputs.beam,
                self.inputs.lwl,
            )
        }
    }

    /// The squat inputs of one environment at a speed through the water.
    fn squat_inputs(&self, env: &EnvironmentInputs, speed_through_water: f64) -> SquatInputs {
        SquatInputs {
            block_coefficient: self.inputs.block_coefficient,
            midship_coefficient: self.inputs.midship_coefficient,
            water_depth_m: env.water_depth,
            channel_area_m2: self.channel_cross_section(env),
            safety_margin_m: env.squat_safety_margin_m,
            midship_area_m2: self.inputs.midship_area_m2,
            ..SquatInputs::new(
                speed_through_water,
                self.inputs.draft,
                self.inputs.beam,
                self.inputs.lwl,
            )
        }
    }

    /// The method checks at the service speed of one environment (issue #420): the squat checks
    /// where squat applies, the STAwave-1 checks where there are waves, and the validity of the
    /// shallow-water fit. Each is named for its environment, as `keel_clears_bed[<environment>]`.
    fn service_point_checks(
        &self,
        env: &EnvironmentInputs,
        out: &EnvironmentResult,
    ) -> Result<Vec<Check>, PyException> {
        let Some(point) = out
            .speed_curve
            .iter()
            .find(|p| p.speed_knots == out.service_speed_knots)
        else {
            return Ok(Vec::new());
        };
        let stw = point.speed_through_water_knots;
        let mut checks: Vec<Check> = Vec::new();
        if squat_applies(env) {
            checks.extend(
                squat::compute(&self.squat_inputs(env, stw))
                    .map_err(PyException::ValueError)?
                    .checks,
            );
        }
        if env.significant_wave_height_m > 0.0 {
            checks.extend(
                waves::compute(&self.wave_inputs(env, stw))
                    .map_err(PyException::ValueError)?
                    .checks,
            );
        }
        if !env.is_deep_water && env.water_depth > 0.0 && self.inputs.draft > 0.0 {
            // The Lackenby term is a fit up to a depth Froude number of 0.95; above it the code
            // takes a fixed factor (an OVD choice, no source), so the fit no longer holds.
            checks.push(Check::with_severity(
                "lackenby_fit_subcritical",
                point.froude_depth < LACKENBY_FNH_MAX,
                text!(
                    "check-lackenby-fit-subcritical",
                    "depth Froude {froude} at the service speed; the shallow-water fit holds below \
                     {limit}, above it a fixed factor stands in",
                    froude = Arg::fixed(point.froude_depth, 2),
                    limit = Arg::fixed(LACKENBY_FNH_MAX, 2),
                ),
                "warning",
            ));
        }
        if self.inputs.method != ResistanceMethod::BargeFit {
            checks.push(self.method_range_check(point.froude_number));
        }
        for c in &mut checks {
            c.name = format!("{}[{}]", c.name, out.name);
        }
        Ok(checks)
    }

    /// Issue #723: the resistance method used and whether the hull and the service speed are in its
    /// declared range (a warning). Only for a Design that chose a method, so the results of a
    /// Design saved before the choice do not change.
    fn method_range_check(&self, froude: f64) -> Check {
        let inp = self.inputs;
        let method = self.applied_method();
        let cp = self.prismatic();
        let (lb, bt) = (inp.lwl / inp.beam, inp.beam / inp.draft);
        let envelope = method.envelope(cp);
        let outside = envelope.outside(froude, cp, lb, bt);
        let list = if outside.is_empty() {
            "none".to_string()
        } else {
            outside
                .iter()
                .map(|o| {
                    if o.high.is_finite() {
                        format!(
                            "{} {:.2} (range {:.2} to {:.2})",
                            o.name, o.value, o.low, o.high
                        )
                    } else {
                        format!("{} {:.2} (range from {:.2})", o.name, o.value, o.low)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        };
        Check::with_severity(
            "resistance_method_in_range",
            outside.is_empty(),
            text!(
                "check-resistance-method-in-range",
                "{method} at the service speed: Fn {froude}, Cp {cp}, L/B {lb}, B/T {bt}; outside \
                 its range: {outside}",
                method = method.as_str(),
                froude = Arg::fixed(froude, 3),
                cp = Arg::fixed(cp, 3),
                lb = Arg::fixed(lb, 2),
                bt = Arg::fixed(bt, 2),
                outside = list,
            ),
            "warning",
        )
    }

    fn compute_environment(
        &self,
        env: &EnvironmentInputs,
        wetted_surface: f64,
    ) -> Result<EnvironmentResult, PyException> {
        let inp = self.inputs;
        let limiting = self.schijf_limiting_speed(env)?;

        // Build the speed range: repeated addition, then rounding.
        let mut speeds: Vec<f64> = Vec::new();
        let mut s = inp.speed_min_knots;
        while s <= inp.speed_max_knots + 0.01 {
            speeds.push(round_to(s, 1));
            s += inp.speed_step_knots;
        }

        // Cap at the margin of the limiting speed for river environments
        let mut max_eval = inp.speed_max_knots;
        if let Some(l) = limiting
            && l > 0.0
        {
            max_eval = py_min(max_eval, l * LIMITING_SPEED_MARGIN);
        }

        let mut curve: Vec<SpeedPoint> = Vec::new();
        let mut service_speed = 0.0;
        let mut service_point: Option<usize> = None;

        for &spd in &speeds {
            if spd > max_eval {
                break;
            }
            let pt = self.compute_speed_point(spd, env, wetted_surface)?;
            // A speed is only a SERVICE speed if the vessel can both push it and float at it.
            if pt.power_available && !pt.squat_limited && spd > service_speed {
                service_speed = spd;
                service_point = Some(curve.len());
            }
            curve.push(pt);
        }

        // Blockage ratio
        let ac = self.channel_cross_section(env);
        let am = self.midship_section_area();
        // Open water has no channel: m = 0 (maintainer decision on issue #396).
        let blockage = if env.is_deep_water {
            0.0
        } else {
            crate::confined_water::blockage_ratio(am, ac)
        };

        // What actually governed the service speed.
        let sq_limit = if squat_applies(env) {
            Some(
                squat::max_speed_for_ukc_with(
                    (inp.draft, inp.beam, inp.lwl),
                    env.water_depth,
                    ac,
                    (inp.block_coefficient, inp.midship_area_m2),
                    env.squat_safety_margin_m,
                    20.0,
                    0.01,
                )
                .map_err(PyException::ValueError)?,
            )
        } else {
            None
        };
        // Ask why the next step up was refused. With no next step the sweep ended: at the channel
        // cap, or at the top of the speed grid (issue #400; it was labelled "power").
        let mut governed = "power";
        let next = curve.iter().find(|p| p.speed_knots > service_speed + 1e-9);
        if let Some(n0) = next {
            if n0.squat_limited {
                governed = "squat";
            }
        } else if let Some(l) = limiting
            && (service_speed >= l * LIMITING_SPEED_MARGIN - 0.05 || max_eval < inp.speed_max_knots)
        {
            governed = "channel";
        } else {
            governed = "speed grid";
        }

        if service_point.is_none() && !curve.is_empty() {
            service_point = Some(0);
            service_speed = curve[0].speed_knots;
        }
        let sp = service_point.map(|i| &curve[i]);

        Ok(EnvironmentResult {
            name: env.name.clone(),
            limiting_speed_knots: limiting,
            service_speed_knots: service_speed,
            blockage_ratio: blockage,
            resistance_at_service_kn: sp.map_or(0.0, |p| p.resistance_total_kn),
            power_at_service_kw: sp.map_or(0.0, |p| p.delivered_power_kw),
            fuel_at_service_lph: sp.map_or(0.0, |p| p.fuel_litres_per_hour),
            fuel_at_service_lpnm: sp.map_or(0.0, |p| p.fuel_litres_per_nm),
            speed_curve: curve,
            squat_limited_speed_knots: sq_limit,
            service_speed_governed_by: governed,
        })
    }

    /// Run resistance calculations for all environments.
    pub fn calculate(&self) -> Result<ResistanceResults, PyException> {
        let ws = if self.inputs.wetted_surface_area > 0.0 {
            self.inputs.wetted_surface_area
        } else if self.inputs.wetted_surface_extra_m2 != 0.0 {
            self.compute_wetted_surface() + self.inputs.wetted_surface_extra_m2
        } else {
            self.compute_wetted_surface()
        };
        let mut env_results = Vec::new();
        for env in &self.inputs.environments {
            env_results.push(self.compute_environment(env, ws)?);
        }
        Ok(ResistanceResults {
            inputs: self.inputs.clone(),
            method_used: self.applied_method(),
            wetted_surface_area: ws,
            environments: env_results,
            checks: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

/// Validity checks for the resistance model, as data.
fn build_checks(inputs: &ResistanceInputs, environments: &[EnvironmentResult]) -> Vec<Check> {
    let mut checks: Vec<Check> = Vec::new();

    for env in environments {
        // Service speed must stay under the Schijf limit.
        if let Some(l) = env.limiting_speed_knots
            && l > 0.0
        {
            let ok = env.service_speed_knots <= LIMITING_SPEED_MARGIN * l + 1e-6;
            checks.push(Check::new(
                format!("service_speed_below_limiting[{}]", env.name),
                ok,
                text!(
                    "check-service-speed-below-limiting",
                    "{environment}: service {service} kn <= 0.95 x Schijf limit {limit} kn",
                    environment = &env.name,
                    service = Arg::fixed(env.service_speed_knots, 1),
                    limit = Arg::fixed(l, 2),
                ),
            ));
        }

        // Delivered power at service must be within installed engine power.
        let ok_pwr = env.power_at_service_kw <= inputs.tug.engine_power_kw + 1e-6;
        checks.push(Check::new(
            format!("power_within_installed[{}]", env.name),
            ok_pwr,
            text!(
                "check-power-within-installed",
                "{environment}: power {power} kW <= installed {installed} kW",
                environment = &env.name,
                power = Arg::fixed(env.power_at_service_kw, 0),
                installed = Arg::fixed(inputs.tug.engine_power_kw, 0),
            ),
        ));

        // Blockage ratio must be a sane fraction for confined environments.
        if env.limiting_speed_knots.is_some() {
            let ok_block = 0.0 < env.blockage_ratio && env.blockage_ratio < 1.0;
            checks.push(Check::new(
                format!("blockage_in_range[{}]", env.name),
                ok_block,
                text!(
                    "check-blockage-in-range",
                    "{environment}: blockage n = {blockage} must be in (0, 1)",
                    environment = &env.name,
                    blockage = Arg::fixed(env.blockage_ratio, 3),
                ),
            ));
        }

        // Issue #400: a service speed the vessel cannot run without touching the bed is flagged.
        // It happens when no speed point is free of squat, and the first point stands in.
        if env
            .speed_curve
            .iter()
            .any(|p| p.speed_knots == env.service_speed_knots && p.squat_limited)
        {
            checks.push(Check::with_severity(
                format!("service_speed_clears_squat[{}]", env.name),
                false,
                text!(
                    "check-service-speed-fails-squat",
                    "{environment}: at the service speed {service} kn the squat leaves less than \
                     the under-keel clearance",
                    environment = &env.name,
                    service = Arg::fixed(env.service_speed_knots, 1),
                ),
                "warning",
            ));
        }
    }

    // Standing advisory: the empirical coefficients are uncalibrated first-order estimates.
    checks.push(Check::with_severity(
        "coefficients_are_first_order",
        true,
        text!(
            "check-coefficients-are-first-order",
            "Residuary Cr/Cf fit, Lackenby shallow-water, blockage and 1.02 form factor \
             are uncalibrated first-order estimates (RS1) \u{2014} ITTC-1957 friction is standard."
        ),
        "warning",
    ));

    checks
}

/// Pure entry point: runs the calculator and attaches the validity checks. `Err` carries the
/// Python exception the same input raises.
pub fn compute(inputs: &ResistanceInputs) -> Result<ResistanceResults, PyException> {
    let calc = ResistanceCalculator::new(inputs);
    let mut result = calc.calculate()?;
    result.checks = build_checks(inputs, &result.environments);
    // Issue #420: the squat, wave and fit checks at each service speed.
    let mut method = Vec::new();
    for (env, out) in inputs.environments.iter().zip(&result.environments) {
        method.extend(calc.service_point_checks(env, out)?);
    }
    let advisory = result.checks.len() - 1;
    result.checks.splice(advisory..advisory, method);
    Ok(result)
}
