//! Best gas-tank arrangement for a hull: search, do not trust the single AUTO answer.
//! Port of `core/gas_arrangement.py`. The reasoning behind the search lives in the Python file.
//!
//! Behaviour is copied exactly, including the errors and their message text. Python's `x or 0.0`
//! on a float is `truthy_or_zero`, `x or y` on an optional float is `or_else`, a strict `>` with
//! the `1e-6` margin picks the winner (so the first of two near-equal candidates stays), and the
//! loops run in Python's order so the tie-breaks agree. Python's keyword defaults become
//! [`FitOptions`].
//!
//! Python's `ArrangementFit` also carries the engine's own input and result; so does
//! [`ArrangementFit`], because the Python bridge returns them and `gas_stability` reads the result.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::barge_capacity::{
    BargeCapacityInputs, BargeCapacityResult, MAX_TANK_OD_M, compute as capacity,
};
use crate::checks::all_ok;
use crate::journey_derating::{JourneyDeratingInputs, JourneyDeratingResult, compute as derate};
use crate::pressure_vessel::PyError;
use crate::protective_location::{
    ADN_G_MAX_L_OVER_D, ADN_G_TANK_CAP_9_3_4_M3, ADN_G_WIDE_TANK_FRACTION,
    compute_required_clearances, d_vc_exact, distance_d_vc,
};
use crate::pyfmt::{fixed, round_to};
use crate::tanks::INT_OVERFLOW_MESSAGE;

/// The rulesets `fit` accepts.
pub const RULESETS: [&str; 3] = ["IGC_2G", "IGC_2G_EXACT", "ADN_G"];
/// `round(3.5 + 0.25 * i, 2)` for `i` in `0..15`: every value is exact in binary, so the
/// rounding changes nothing.
pub const DEFAULT_DEPTHS_M: [f64; 15] = [
    3.5, 3.75, 4.0, 4.25, 4.5, 4.75, 5.0, 5.25, 5.5, 5.75, 6.0, 6.25, 6.5, 6.75, 7.0,
];
pub const DEFAULT_TANKS_WIDE: [i64; 3] = [1, 2, 3];
pub const DEFAULT_EXTRA_CLEARANCE_M: [f64; 4] = [0.5, 1.0, 1.5, 2.0];
/// Tank-to-tank gap the engine uses along the hull (a default, not a rule).
pub const ADN_LONGITUDINAL_GAP_M: f64 = 2.0;

/// The keyword arguments of `fit`. `Default` is the Python defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct FitOptions {
    pub ruleset: String,
    pub fill_ratio: f64,
    pub self_propelled: bool,
    pub depths: Vec<f64>,
    pub tanks_wide: Vec<i64>,
    pub extra_clearance_m: Vec<f64>,
    pub adn_tank_cap_m3: f64,
    pub block_coefficient: f64,
    pub max_tank_od_m: f64,
}

impl Default for FitOptions {
    fn default() -> Self {
        Self {
            ruleset: "IGC_2G".to_string(),
            fill_ratio: 0.95,
            self_propelled: true,
            depths: DEFAULT_DEPTHS_M.to_vec(),
            tanks_wide: DEFAULT_TANKS_WIDE.to_vec(),
            extra_clearance_m: DEFAULT_EXTRA_CLEARANCE_M.to_vec(),
            adn_tank_cap_m3: ADN_G_TANK_CAP_9_3_4_M3,
            block_coefficient: 0.85,
            max_tank_od_m: MAX_TANK_OD_M,
        }
    }
}

/// The winning arrangement and its payload.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrangementFit {
    pub loa_m: f64,
    pub beam_m: f64,
    pub depth_m: f64,
    pub cargo: String,
    pub ruleset: String,
    pub fill_ratio: f64,
    pub self_propelled: bool,
    pub t_max_m: f64,
    /// Per journey, after the draught cap.
    pub payload_t: f64,
    pub volume_capacity_t: f64,
    /// `"volume"`, `"draft"` or `"no cargo area"`.
    pub binding: String,
    pub draft_at_payload_m: f64,
    pub cargo_zone_m: f64,
    pub stern_void_m: f64,
    pub bow_void_m: f64,
    pub tank_od_m: f64,
    pub n_tanks: i64,
    pub n_tanks_wide: i64,
    pub n_tanks_long: i64,
    pub lightship_t: f64,
    pub air_draft_light_m: f64,
    pub side_clearance_m: f64,
    pub tank_inner_diameter_m: f64,
    pub containment_inner_volume_m3: f64,
    pub per_tank_volume_m3: f64,
    pub cargo_volume_m3: f64,
    pub density_t_m3: f64,
    pub failed_checks: Vec<String>,
    /// False: no candidate passed its error checks; see `failed_checks`.
    pub feasible: bool,
    pub candidates_tried: i64,
    pub engine_inputs: BargeCapacityInputs,
    pub engine_result: BargeCapacityResult,
}

impl FitOptions {
    /// The defaults of `smallest_viable_loa`: as `fit`, but `depths` is the first default depth.
    pub fn for_smallest_viable_loa() -> Self {
        Self {
            depths: vec![DEFAULT_DEPTHS_M[0]],
            ..Self::default()
        }
    }
}

fn value_error(message: String) -> PyError {
    PyError {
        kind: "ValueError",
        message,
    }
}

/// Python `x or 0.0` on a float: zero is false, so it gives `0.0`; NaN is true.
fn truthy_or_zero(x: f64) -> f64 {
    if x == 0.0 { 0.0 } else { x }
}

/// Python `x or 0.0` on an optional float: `None` and zero give `0.0`.
fn opt_or_zero(x: Option<f64>) -> f64 {
    truthy_or_zero(x.unwrap_or(0.0))
}

/// Python `int(x or 0)`.
fn opt_count(x: Option<i64>) -> i64 {
    x.unwrap_or(0)
}

/// Python `max(r.n_tanks_total or 1, 1)` as the float the divisions turn it into.
fn tank_divisor(n_tanks_total: Option<i64>) -> f64 {
    let n = match n_tanks_total {
        None | Some(0) => 1,
        Some(n) => n,
    };
    n.max(1) as f64
}

/// Python `a // b` on ints with `b != 0`. The one result no 64-bit field holds
/// (`i64::MIN // -1`) is the fixed `OverflowError`.
fn floor_div(a: i64, b: i64) -> Result<i64, PyError> {
    let q = a.checked_div(b).ok_or_else(|| PyError {
        kind: "OverflowError",
        message: INT_OVERFLOW_MESSAGE.to_string(),
    })?;
    Ok(if a % b != 0 && ((a < 0) != (b < 0)) {
        q - 1
    } else {
        q
    })
}

/// The code minimum for the engine's clearance datum (outer face of the insulation).
///
/// `Err` is the `ValueError` for a ruleset that is not a vessel type.
pub fn required_side_clearance(
    ruleset: &str,
    beam: f64,
    total_m3: f64,
    per_tank_m3: f64,
    insulation_mm: f64,
) -> Result<f64, PyError> {
    if ruleset == "IGC_2G_EXACT" {
        // Per-tank d, measured to steel: insulation sits inside d.
        return Ok(d_vc_exact(per_tank_m3) - insulation_mm / 1000.0);
    }
    compute_required_clearances(ruleset, beam, distance_d_vc(total_m3))
        .map(|c| c.cs_required)
        .map_err(value_error)
}

/// One built candidate: the inputs and the capacity result they gave.
type Built = (BargeCapacityInputs, BargeCapacityResult);

/// One scored candidate, as Python's `(payload, depth, result, journey, inputs)` tuple.
#[derive(Clone)]
struct Candidate {
    tonnage: f64,
    depth: f64,
    result: BargeCapacityResult,
    journey: JourneyDeratingResult,
    inputs: BargeCapacityInputs,
}

fn derate_candidate(
    r: &BargeCapacityResult,
    loa: f64,
    beam: f64,
    depth: f64,
    t_max: f64,
    block_coefficient: f64,
) -> Result<JourneyDeratingResult, PyError> {
    let mut ji = JourneyDeratingInputs::new(r, loa, beam, depth);
    ji.block_coefficient = block_coefficient;
    ji.design_depth_m = t_max;
    ji.ukc_m = 0.0;
    derate(&ji).map_err(value_error)
}

/// Python's `fit.build`, with its `nonlocal tried` as `&mut tried`.
struct Search<'a> {
    loa: f64,
    beam: f64,
    cargo: &'a str,
    o: &'a FitOptions,
    tried: i64,
}

impl Search<'_> {
    fn base_inputs(&self, depth: f64, nw: i64, nl: i64) -> BargeCapacityInputs {
        let mut inp = BargeCapacityInputs::new(self.loa, self.beam, depth);
        inp.cargo_type = self.cargo.to_string();
        inp.self_propelled = self.o.self_propelled;
        inp.vessel_type = self.o.ruleset.clone();
        inp.fill_ratio = self.o.fill_ratio;
        inp.block_coefficient = self.o.block_coefficient;
        inp.n_tanks_wide = Some(nw);
        inp.n_tanks_long = nl;
        inp
    }

    fn od_unusable(&self, r: &BargeCapacityResult) -> bool {
        let od = opt_or_zero(r.tank_outer_diameter_m);
        od == 0.0 || od > self.o.max_tank_od_m + 1e-6
    }

    fn build(&mut self, d: f64, nw: i64, nl: i64) -> Result<Option<Built>, PyError> {
        self.tried += 1;
        let mut inp = self.base_inputs(d, nw, nl);
        let mut r = capacity(&inp)?;
        if self.od_unusable(&r) {
            // Full-beam tank infeasible or over the cap: put the tank ON the cap and widen the
            // clearance: a legitimate design only if that clearance meets the rule.
            let cs = (self.beam - nw as f64 * self.o.max_tank_od_m) / 2.0;
            if cs < 0.0 {
                return Ok(None);
            }
            inp.side_clearance_override_m = Some(cs);
            r = capacity(&inp)?;
            if self.od_unusable(&r) {
                return Ok(None);
            }
            let total = opt_or_zero(r.containment_inner_volume_m3);
            // Mutation survivor (#146), `total / n` to `total % n` below: equivalent. The per-tank
            // volume matters only to IGC_2G_EXACT, whose rule d(V) - insulation is non-decreasing
            // in V, and continuous, so the loop that sets the clearance settles. We are here only when the full-beam tank is over the cap by more than the
            // 1e-6 of `od_unusable`, so the clearance `cs` is at least 5e-7 above the one the
            // full-beam attempt settled on, and that settled clearance is already at least the
            // rule for the full-beam tank. The cap tank is smaller, so its rule is smaller still,
            // and the check below passes whatever per-tank volume is given. The other way in, a
            // full-beam tank of zero diameter, needs a beam under 0.9 m, where `cs` is under the
            // 0.45 m floor of the rule for every volume the engine can build. Searched with the
            // mutant built in: 180 000 random inputs, 240 000 more at hull sizes up to 4 km, none
            // differ. Not a gap in the tests.
            let need = required_side_clearance(
                &self.o.ruleset,
                self.beam,
                total,
                total / tank_divisor(r.n_tanks_total),
                opt_or_zero(r.insulation_thickness_mm),
            )?;
            // Killed by `a_cap_clearance_a_hair_under_the_rule_is_accepted_not_refused` (`+` to `-`).
            if cs + 1e-9 < need {
                return Ok(None);
            }
        }
        if self.o.ruleset == "ADN_G" {
            let per_tank =
                opt_or_zero(r.containment_inner_volume_m3) / tank_divisor(r.n_tanks_total);
            let tank_len = (opt_or_zero(r.cargo_zone_length_m)
                - (nl - 1) as f64 * ADN_LONGITUDINAL_GAP_M)
                / nl as f64;
            let od = opt_or_zero(r.tank_outer_diameter_m);
            if per_tank > self.o.adn_tank_cap_m3 + 1e-6 || tank_len > ADN_G_MAX_L_OVER_D * od {
                return Ok(None);
            }
        }
        Ok(Some((inp, r)))
    }
}

/// Most cargo per journey at draught cap `t_max` over depth x arrangement.
///
/// Only candidates whose error-severity checks all pass can win. If none passes, the best-tonnage
/// candidate comes back with `feasible == false`; check it before using the payload.
pub fn fit(
    loa: f64,
    beam: f64,
    cargo: &str,
    t_max: f64,
    o: &FitOptions,
) -> Result<ArrangementFit, PyError> {
    if !RULESETS.contains(&o.ruleset.as_str()) {
        return Err(value_error(
            "ruleset must be one of ('IGC_2G', 'IGC_2G_EXACT', 'ADN_G')".to_string(),
        ));
    }
    let mut s = Search {
        loa,
        beam,
        cargo,
        o,
        tried: 0,
    };
    // Best candidate that passes its error checks, and best overall (reported only if none passes).
    let mut best: Option<Candidate> = None;
    let mut best_any: Option<Candidate> = None;

    for &d in &o.depths {
        for &nw in &o.tanks_wide {
            let mut cands: Vec<Built> = Vec::new();
            if o.ruleset == "ADN_G" {
                // Add tanks along the hull until one complies, then one more (shorter tanks lose
                // head length but can balance better).
                for nl in 2..25 {
                    if let Some(x) = s.build(d, nw, nl)? {
                        cands.push(x);
                        if cands.len() == 2 {
                            break;
                        }
                    }
                }
            } else if let Some(x) = s.build(d, nw, 2)? {
                cands.push(x);
            }
            let mut more: Vec<Built> = Vec::new();
            for (inp0, r0) in &cands {
                if opt_or_zero(r0.tank_outer_diameter_m) == 0.0 {
                    continue;
                }
                let Some(side) = r0.side_clearance_m else {
                    continue;
                };
                for &extra in &o.extra_clearance_m {
                    s.tried += 1;
                    let mut i2 = inp0.clone();
                    i2.side_clearance_override_m = Some(side + extra);
                    let r2 = capacity(&i2)?;
                    if opt_or_zero(r2.tank_outer_diameter_m) != 0.0 && r2.cargo_capacity_t > 0.0 {
                        more.push((i2, r2));
                    }
                }
            }
            for (inp, r) in cands.into_iter().chain(more) {
                let j = derate_candidate(&r, loa, beam, d, t_max, o.block_coefficient)?;
                let tonnage = j.achievable_tonnage_t;
                let passes = {
                    let mut all = r.checks.clone();
                    all.extend(j.checks.iter().cloned());
                    all_ok(&all)
                };
                let beats_any = best_any.as_ref().is_none_or(|b| tonnage > b.tonnage + 1e-6);
                let beats_best = passes && best.as_ref().is_none_or(|b| tonnage > b.tonnage + 1e-6);
                if beats_any || beats_best {
                    let cand = Candidate {
                        tonnage,
                        depth: d,
                        result: r,
                        journey: j,
                        inputs: inp,
                    };
                    match (beats_any, beats_best) {
                        (true, true) => {
                            best_any = Some(cand.clone());
                            best = Some(cand);
                        }
                        (true, false) => best_any = Some(cand),
                        _ => best = Some(cand),
                    }
                }
            }
        }
    }

    // Nothing passes: report the best failing candidate, flagged infeasible.
    let best = match best.or(best_any) {
        Some(b) => b,
        None => {
            // Nothing fitted: report the plain single-tank attempt, which carries nothing.
            let d = *o.depths.first().ok_or_else(|| PyError {
                kind: "IndexError",
                message: "list index out of range".to_string(),
            })?;
            let inp = s.base_inputs(d, 1, 2);
            let r = capacity(&inp)?;
            let j = derate_candidate(&r, loa, beam, d, t_max, o.block_coefficient)?;
            Candidate {
                tonnage: 0.0,
                depth: d,
                result: r,
                journey: j,
                inputs: inp,
            }
        }
    };

    let Candidate {
        depth: d,
        result: r,
        journey: j,
        inputs: inp,
        ..
    } = best;
    let n_total = opt_count(r.n_tanks_total);
    let n_wide = opt_count(r.n_tanks_wide);
    let mut failed: Vec<String> = r
        .checks
        .iter()
        .chain(j.checks.iter())
        .filter(|c| !c.ok)
        .map(|c| format!("{}: {}", c.name, c.message))
        .collect();
    let feasible = {
        let mut all = r.checks.clone();
        all.extend(j.checks.iter().cloned());
        all_ok(&all)
    };
    if o.ruleset == "ADN_G"
        // Killed by `a_tank_exactly_at_seventy_percent_of_beam_is_not_flagged_wide` (`>` to `>=`).
        && opt_or_zero(r.tank_outer_diameter_m) > ADN_G_WIDE_TANK_FRACTION * beam
    {
        failed.push(format!(
            "adn_wide_tank: tank {} m > 0.70 B; ADN 9.3.1.14.2 needs extra intact stability checks",
            fixed(opt_or_zero(r.tank_outer_diameter_m), 1)
        ));
    }
    let n_tanks_long = if n_wide != 0 {
        floor_div(n_total, n_wide)?
    } else {
        0
    };
    let containment = opt_or_zero(r.containment_inner_volume_m3);
    let tanks_for_division = n_total.max(1) as f64;
    Ok(ArrangementFit {
        loa_m: loa,
        beam_m: beam,
        depth_m: d,
        cargo: cargo.to_string(),
        ruleset: o.ruleset.clone(),
        fill_ratio: o.fill_ratio,
        self_propelled: o.self_propelled,
        t_max_m: t_max,
        payload_t: j.achievable_tonnage_t,
        volume_capacity_t: r.cargo_capacity_t,
        binding: if r.cargo_capacity_t <= 0.0 {
            "no cargo area".to_string()
        } else {
            j.binding_limit.clone()
        },
        draft_at_payload_m: j.draft_at_achievable_m,
        cargo_zone_m: opt_or_zero(r.cargo_zone_length_m),
        stern_void_m: opt_or_zero(r.stern_void_length_m),
        bow_void_m: opt_or_zero(r.bow_void_length_m),
        tank_od_m: opt_or_zero(r.tank_outer_diameter_m),
        n_tanks: n_total,
        n_tanks_wide: n_wide,
        n_tanks_long,
        lightship_t: r.lightship_t,
        air_draft_light_m: match r.air_draft_light_m {
            // Python `r.air_draft_light_m or r.air_draft_m`. Mutation survivor (#146), the guard
            // `x != 0.0` always true: no input found that tells it apart, so it is recorded as
            // equivalent. It differs only when the light air draft is exactly 0.0 and the loaded
            // air draft is not: the tank top level with the light waterline to the last bit, with
            // the loaded waterline above it. When the draught is clamped at the depth (a lightship
            // over the hull's displacement) both air drafts are the tank top less the depth, so
            // the two sides agree. Hill-climbing the light air draft down over length, beam,
            // depth, tank cap and draught cap, and 240 000 random inputs, found nothing under
            // 0.0146 m, and that only for a hull 15 mm deep. The guard stays as the port of
            // Python's `or`.
            Some(x) if x != 0.0 => x,
            _ => r.air_draft_m,
        },
        side_clearance_m: opt_or_zero(r.side_clearance_m),
        tank_inner_diameter_m: opt_or_zero(r.tank_inner_diameter_m),
        containment_inner_volume_m3: containment,
        per_tank_volume_m3: containment / tanks_for_division,
        cargo_volume_m3: r.cargo_volume_m3,
        density_t_m3: r.cargo_density_t_per_m3,
        failed_checks: failed,
        feasible,
        candidates_tried: s.tried,
        engine_inputs: inp,
        engine_result: r,
    })
}

/// Shortest LOA (to 0.1 m) at which the hull has any cargo area at all. Infinity when even `hi`
/// has none. Python defaults `depths` to the first default depth only for this function: start
/// from [`FitOptions::for_smallest_viable_loa`].
pub fn smallest_viable_loa(
    beam: f64,
    cargo: &str,
    t_max: f64,
    lo: f64,
    hi: f64,
    o: &FitOptions,
) -> Result<f64, PyError> {
    let (mut lo, mut hi) = (lo, hi);
    if fit(hi, beam, cargo, t_max, o)?.volume_capacity_t <= 0.0 {
        return Ok(f64::INFINITY);
    }
    while hi - lo > 0.1 {
        let mid = 0.5 * (lo + hi);
        if fit(mid, beam, cargo, t_max, o)?.volume_capacity_t > 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Ok(round_to(hi, 1))
}
