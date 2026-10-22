//! Journey / channel de-rating: where volume-bound capacity meets the channel draft cap.
//! Port of `core/journey_derating.py`. The reasoning lives in the Python file.
//!
//! Behaviour is copied exactly, including the error text. Python's two-argument `max` and `min`
//! keep their first-of-equal rule (`py_max`, `py_min`) and every expression keeps Python's
//! evaluation order so the rounding is the same. Python raises `ValueError` in the inputs'
//! `__post_init__`; here [`compute`] runs the same check first and returns the message.

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::barge_capacity::BargeCapacityResult;
use crate::checks::{Check, all_ok};
use crate::hydrostatics::Hull;
use crate::pyfmt::percent;
use crate::text;
use crate::text::{Arg, Text};

/// A volume-bound barge design and a channel, to be de-rated to a per-journey load.
/// [`JourneyDeratingInputs::new`] gives the Python dataclass defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct JourneyDeratingInputs<'a> {
    /// From `barge_capacity::compute`.
    pub capacity: &'a BargeCapacityResult,
    pub loa: f64,
    pub beam: f64,
    pub depth: f64,
    /// t/m^3
    pub water_density: f64,
    /// Channel design depth below minimum water level (m).
    pub design_depth_m: f64,
    /// Under-keel clearance already in `design_depth_m` (m).
    pub ukc_m: f64,
    /// Advisory: below this the hull is over-built for the channel.
    pub utilisation_floor: f64,
}

impl<'a> JourneyDeratingInputs<'a> {
    /// The dataclass defaults, with the capacity result and the three principal dimensions.
    pub fn new(capacity: &'a BargeCapacityResult, loa: f64, beam: f64, depth: f64) -> Self {
        Self {
            capacity,
            loa,
            beam,
            depth,
            water_density: 1.0,
            design_depth_m: 0.0,
            ukc_m: 0.0,
            utilisation_floor: 0.5,
        }
    }

    /// Python's `__post_init__`. The `Err` text is the `ValueError` message.
    pub fn validate(&self) -> Result<(), String> {
        if self.loa <= 0.0 || self.beam <= 0.0 || self.depth <= 0.0 {
            return Err("loa, beam, depth must be positive".to_string());
        }
        if self.design_depth_m < 0.0 || self.ukc_m < 0.0 {
            return Err("design_depth_m and ukc_m must be non-negative".to_string());
        }
        Ok(())
    }
}

/// The per-journey load after the channel cap.
#[derive(Debug, Clone, PartialEq)]
pub struct JourneyDeratingResult {
    /// Echo: volume-bound capacity.
    pub cargo_capacity_t: f64,
    /// Channel draft cap used (design depth minus UKC).
    pub t_max_m: f64,
    /// Echo from `barge_capacity`.
    pub full_load_draft_m: f64,
    /// Headline: per-journey payload after de-rating.
    pub achievable_tonnage_t: f64,
    /// `"volume"` or `"draft"`.
    pub binding_limit: String,
    pub volume_utilisation: f64,
    pub draft_at_achievable_m: f64,
    pub shed_tonnage_t: f64,
    pub checks: Vec<Check>,
    pub warnings: Vec<Text>,
}

impl JourneyDeratingResult {
    /// The Python `ok` property.
    pub fn ok(&self) -> bool {
        all_ok(&self.checks)
    }
}

/// Python `max(a, b)`: the first of two equal items; `b` only if `b > a`.
fn py_max(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// Python `min(a, b)`: the first of two equal items; `b` only if `b < a`.
fn py_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// Apply the channel draft cap to a volume-bound barge design. `Err` is a `ValueError` message.
pub fn compute(inp: &JourneyDeratingInputs<'_>) -> Result<JourneyDeratingResult, String> {
    inp.validate()?;
    let cap = inp.capacity;
    let cargo_capacity = cap.cargo_capacity_t;
    let full_load_draft = cap.full_load_draft_m;
    let lightship = cap.lightship_t;

    let t_max = py_max(0.0, inp.design_depth_m - inp.ukc_m);
    // Issue #448: the hull the capacity was worked on, as its sections.
    let spec = cap
        .hull_spec
        .clone()
        .unwrap_or(crate::hydrostatics::HullSpec {
            loa: inp.loa,
            beam: inp.beam,
            depth: inp.depth,
            bow_rake_length_m: cap.bow_rake_length_m.unwrap_or(0.0),
            bow_rake_rise_m: inp.depth,
            stern: None,
        });
    let hull = Hull::build(&spec).map_err(|e| alloc::format!("{e}"))?;
    let rho = inp.water_density;

    let (achievable, binding, draft_at);
    if t_max >= full_load_draft {
        // Floats fully laden: volume-bound.
        achievable = cargo_capacity;
        binding = "volume";
        draft_at = full_load_draft;
    } else {
        // Too deep: shed cargo until the draft is `t_max` (weight- and draft-bound).
        let float_draft = py_min(t_max, inp.depth);
        let displacement_at_tmax = hull.volume_at(float_draft) * rho;
        achievable = py_max(
            0.0,
            py_min(cargo_capacity, displacement_at_tmax - lightship),
        );
        binding = "draft";
        draft_at = t_max;
    }

    let utilisation = if cargo_capacity > 0.0 {
        achievable / cargo_capacity
    } else {
        0.0
    };
    let shed = py_max(0.0, cargo_capacity - achievable);

    let checks = vec![
        Check::new(
            "capacity_is_volume_bound",
            !cap.overloaded,
            text!(
                "check-capacity-is-volume-bound",
                "input capacity must be a feasible volume-bound design (not overloaded)"
            ),
        ),
        Check::new(
            "floats",
            achievable > 0.0,
            text!(
                "check-floats",
                "channel must float some cargo (T_max {t_max} m vs lightship draft)",
                t_max = Arg::fixed(t_max, 2),
            ),
        ),
        Check::new(
            "ukc_consistent",
            inp.design_depth_m >= inp.ukc_m,
            text!(
                "check-ukc-consistent",
                "design depth must be >= UKC (pass depth + UKC, never a pre-reduced T_max)"
            ),
        ),
        Check::with_severity(
            "utilisation_reasonable",
            utilisation >= inp.utilisation_floor,
            text!(
                "check-utilisation-reasonable",
                "volume utilisation {utilisation} >= floor {floor}",
                utilisation = Arg::num(utilisation, percent(utilisation, 0)),
                floor = Arg::num(inp.utilisation_floor, percent(inp.utilisation_floor, 0)),
            ),
            "warning",
        ),
    ];
    let mut warnings = Vec::new();
    if binding == "draft" {
        warnings.push(text!(
            "warn-journey-draft-bound",
            "Draft-bound: shed {shed} t of {capacity} t volume capacity to float at T_max = {t_max} m (full-load draft {full_load_draft} m).",
            shed = Arg::fixed(shed, 0),
            capacity = Arg::fixed(cargo_capacity, 0),
            t_max = Arg::fixed(t_max, 2),
            full_load_draft = Arg::fixed(full_load_draft, 2),
        ));
    }

    Ok(JourneyDeratingResult {
        cargo_capacity_t: cargo_capacity,
        t_max_m: t_max,
        full_load_draft_m: full_load_draft,
        achievable_tonnage_t: achievable,
        binding_limit: binding.to_string(),
        volume_utilisation: utilisation,
        draft_at_achievable_m: draft_at,
        shed_tonnage_t: shed,
        checks,
        warnings,
    })
}
