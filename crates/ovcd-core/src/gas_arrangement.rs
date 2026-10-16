//! Best gas-tank arrangement for a hull: search, do not trust the single AUTO answer.
//! Port of `core/gas_arrangement.py`. The reasoning behind the search lives in the Python file.
//!
//! Only IGC rules apply to the gas tanks (issue #175): `IGC_2G`, the per-tank rule of IGC 2.4.1.1
//! with the datum of 2.4.2 (issue #237). Two tanks lie
//! along the hull. Every tank has to keep to the outer diameter cap (`FitOptions::max_tank_od_m`)
//! and to length over diameter at most [`MAX_TANK_LENGTH_OVER_OD`]. A candidate over that is not
//! feasible: it gets the failed check `tank_length_over_diameter`, so it can never win while a
//! compliant one exists, and a hull with none is reported infeasible, not given a payload.
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
    BargeCapacityInputs, BargeCapacityResult, DEFAULT_BOW_RAKE_TO_DEPTH_RATIO,
    DEFAULT_MAX_TANK_OD_M, DEFAULT_PUSHED_STERN_VOID_M, DEFAULT_TANK_DOME_DIAMETER_M,
    DEFAULT_TANK_DOME_HEIGHT_M, DEFAULT_TANK_SUPPORT_HEIGHT_M, DEFAULT_TANK_WALL_THICKNESS_MM,
    DEFAULT_TANKS_IN_LINE, DEFAULT_WING_TANK_GAP_M, DEFAULT_WING_TANK_HEIGHT_RATIO,
    compute as capacity, dome_volume_m3,
};
use crate::checks::{Check, all_ok};
use crate::gas_stability::{self as gs, DEFAULT_WATER_DENSITY, GasCapacityResult};
use crate::journey_derating::{JourneyDeratingInputs, JourneyDeratingResult, compute as derate};
use crate::pressure_vessel::PyError;
use crate::protective_location::d_vc_exact;
use crate::pyfmt::{fixed, py_str, round_to};
use crate::tanks::INT_OVERFLOW_MESSAGE;

/// The rulesets `fit` accepts.
pub const RULESETS: [&str; 1] = ["IGC_2G"];
/// Largest overall tank length (heads included) over outer diameter, for every pressure tank. A
/// design limit of this tool, the same under both rulesets. The figure is the one the removed
/// `ADN_G` ruleset used (the comments of that ruleset cite ADN 2025 9.3.1.11.1(b)); no IGC Code
/// paragraph is cited for it.
pub const MAX_TANK_LENGTH_OVER_OD: f64 = 7.0;
/// `round(3.5 + 0.25 * i, 2)` for `i` in `0..15`: every value is exact in binary, so the
/// rounding changes nothing.
pub const DEFAULT_DEPTHS_M: [f64; 15] = [
    3.5, 3.75, 4.0, 4.25, 4.5, 4.75, 5.0, 5.25, 5.5, 5.75, 6.0, 6.25, 6.5, 6.75, 7.0,
];
pub const DEFAULT_TANKS_WIDE: [i64; 3] = [1, 2, 3];
pub const DEFAULT_EXTRA_CLEARANCE_M: [f64; 4] = [0.5, 1.0, 1.5, 2.0];
/// The default least freeboard (m), the one `BargeCapacityInputs` uses.
pub const DEFAULT_REQUIRED_FREEBOARD_M: f64 = 0.3;

/// The keyword arguments of `fit`. `Default` is the Python defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct FitOptions {
    pub ruleset: String,
    pub fill_ratio: f64,
    pub self_propelled: bool,
    pub depths: Vec<f64>,
    pub tanks_wide: Vec<i64>,
    pub extra_clearance_m: Vec<f64>,
    pub block_coefficient: f64,
    /// Cap on a tank's outer diameter (m).
    pub max_tank_od_m: f64,
    /// Bow rake as a multiple of the moulded depth of each candidate.
    pub bow_rake_to_depth_ratio: f64,
    /// Aft void of a pushed barge (m).
    pub pushed_stern_void_m: f64,
    /// Tanks along the hull, for every candidate.
    pub tanks_in_line: i64,
    /// Height of the tank saddles above the inner bottom (m).
    pub tank_support_height_m: f64,
    /// Dome height above the tank top (m).
    pub tank_dome_height_m: f64,
    /// Dome diameter (m): with the dome height it gives the dome's volume, which counts in Vc
    /// only (IGC 2.4.1.1). A nominal value; no published source yet.
    pub tank_dome_diameter_m: f64,
    /// Gap between the wing tank and the cargo tank, per side (m).
    pub wing_tank_gap_m: f64,
    /// Wing tank height as a fraction of the cargo tank outer diameter.
    pub wing_tank_height_ratio: f64,
    /// Fixed tank wall thickness (mm).
    pub tank_wall_thickness_mm: f64,
    /// Water density for the draught of every candidate and its journey de-rating (t/m3).
    pub water_density_t_per_m3: f64,
    /// Least freeboard at the carried draught (m): a candidate that carries its payload deeper
    /// than the hull depth less this fails `freeboard_at_payload`.
    pub required_freeboard_m: f64,
    /// Air draft limit of the light condition (m), as the capacity check reads it. `None`: none.
    pub air_draft_limit_m: Option<f64>,
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
            block_coefficient: 0.85,
            max_tank_od_m: DEFAULT_MAX_TANK_OD_M,
            bow_rake_to_depth_ratio: DEFAULT_BOW_RAKE_TO_DEPTH_RATIO,
            pushed_stern_void_m: DEFAULT_PUSHED_STERN_VOID_M,
            tanks_in_line: DEFAULT_TANKS_IN_LINE,
            tank_support_height_m: DEFAULT_TANK_SUPPORT_HEIGHT_M,
            tank_dome_height_m: DEFAULT_TANK_DOME_HEIGHT_M,
            tank_dome_diameter_m: DEFAULT_TANK_DOME_DIAMETER_M,
            wing_tank_gap_m: DEFAULT_WING_TANK_GAP_M,
            wing_tank_height_ratio: DEFAULT_WING_TANK_HEIGHT_RATIO,
            tank_wall_thickness_mm: DEFAULT_TANK_WALL_THICKNESS_MM,
            water_density_t_per_m3: DEFAULT_WATER_DENSITY,
            required_freeboard_m: DEFAULT_REQUIRED_FREEBOARD_M,
            air_draft_limit_m: None,
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

/// The code minimum for the engine's clearance datum (outer face of the insulation): the d(Vc)
/// distance of IGC Code 2.4.1.1 (see `protective_location::d_vc_exact`) per individual tank, less
/// the insulation, because IGC 2.4.2 measures d plate to plate, from the moulded line to the
/// moulded line, excluding insulation.
///
/// `per_tank_m3` is Vc of one tank, dome included. `total_m3` is not used (the stepped reading,
/// which took the total volume, is removed); it stays so the signature is unchanged.
///
/// `Err` is the `ValueError` for a ruleset that is not a vessel type.
pub fn required_side_clearance(
    ruleset: &str,
    _beam: f64,
    _total_m3: f64,
    per_tank_m3: f64,
    insulation_mm: f64,
) -> Result<f64, PyError> {
    // An unknown name is the `ValueError` of the vessel types; of the known ones only the gas
    // rule set applies here.
    crate::protective_location::VesselType::from_value(ruleset).map_err(value_error)?;
    if !RULESETS.contains(&ruleset) {
        return Err(value_error(format!(
            "ruleset must be one of ('IGC_2G',), not {}",
            crate::pystr::repr_str(ruleset)
        )));
    }
    // IGC Code 2.4.1.1, per tank; 2.4.2, measured to the tank steel: the insulation sits inside d.
    Ok(d_vc_exact(per_tank_m3) - insulation_mm / 1000.0)
}

/// The length over diameter check of a candidate's tank. `None` when there is no tank (a bulk
/// result, or a hull with no cargo area), which the other checks already report.
pub fn length_over_diameter_check(r: &BargeCapacityResult) -> Option<Check> {
    let tanks = &r.gas_basis.as_ref()?.tanks;
    if tanks.outer_diameter <= 0.0 {
        return None;
    }
    let ratio = tanks.overall_tank_length / tanks.outer_diameter;
    Some(Check::new(
        "tank_length_over_diameter",
        ratio <= MAX_TANK_LENGTH_OVER_OD,
        format!(
            "tank length {} m / outer diameter {} m = {} <= {}",
            fixed(tanks.overall_tank_length, 2),
            fixed(tanks.outer_diameter, 2),
            fixed(ratio, 2),
            py_str(MAX_TANK_LENGTH_OVER_OD)
        ),
    ))
}

// ---------------------------------------------------------------------------------------------
// THE CHECKS THE SEARCH USES (issue #264). This is the one list. `fit` and `smallest_viable_loa`
// keep a candidate only if every error-severity check below passes, then rank the passing ones by
// cargo. When the rules engine (#176) lands, it replaces this list with the Design's selected rules.
//
// Cheap checks (run first, from numbers the search has already computed):
//   - the capacity result's own checks: feasible geometry, not overloaded, freeboard, cargo zone,
//     the aft arrangement (thrusters, containers, wheelhouse), the gas separation (protective
//     location clearances), and the air draft limit of the light condition when one is set;
//   - the journey checks: the channel draught limit leaves a payload that floats;
//   - `tank_length_over_diameter`: overall tank length over outer diameter, at most 7;
//   - `freeboard_at_payload`: the hull depth less the carried draught, at least the required freeboard.
// Costly check (run only for a candidate that passed the cheap ones and could still win):
//   - stability: IS Code 2008 Part A 2.2, each loading condition (`gas_stability::assess`).
// A check of warning severity is advisory: it never gates (`all_ok`), as everywhere else.
// ---------------------------------------------------------------------------------------------

/// The cheap checks of a candidate: see the list above.
fn cheap_checks(
    r: &BargeCapacityResult,
    j: &JourneyDeratingResult,
    depth: f64,
    o: &FitOptions,
) -> Vec<Check> {
    let mut all = r.checks.clone();
    all.extend(j.checks.iter().cloned());
    all.extend(length_over_diameter_check(r));
    let freeboard = depth - j.draft_at_achievable_m;
    all.push(Check::new(
        "freeboard_at_payload",
        freeboard + 1e-9 >= o.required_freeboard_m,
        format!(
            "freeboard {} m at the carried draught {} m >= required {} m",
            fixed(freeboard, 2),
            fixed(j.draft_at_achievable_m, 2),
            py_str(o.required_freeboard_m)
        ),
    ));
    all
}

/// One stability condition as a check, named and worded as the simulation lists it.
pub fn stability_check(c: &gs::Condition) -> Check {
    let bad: Vec<&gs::Criterion> = c.criteria.iter().filter(|x| !x.ok).collect();
    let mut message = format!(
        "GM {} m (fluid), max GZ {} m at {}\u{b0}",
        fixed(c.gm_fluid_m, 2),
        fixed(c.max_gz_m, 2),
        fixed(f64::from(c.max_gz_deg), 0)
    );
    if !bad.is_empty() {
        let parts: Vec<String> = bad
            .iter()
            .map(|x| {
                let v = match x.value {
                    gs::PyNum::Int(i) => i as f64,
                    gs::PyNum::Float(v) => v,
                };
                format!("{} {} < {}", x.name, fixed(v, 3), py_str(x.limit))
            })
            .collect();
        message.push_str(" \u{2014} fails: ");
        message.push_str(&parts.join(", "));
    }
    Check::new(
        format!("Stability \u{2014} {}", c.name.to_lowercase()),
        bad.is_empty(),
        message,
    )
}

/// The stability checks of a candidate, one per loading condition. None when it carries nothing or
/// has no gas tanks (the cheap checks report that). An assessment that cannot be made is one failed
/// check, so that a candidate the engine cannot judge never passes and never stops the search.
fn stability_checks(r: &BargeCapacityResult, j: &JourneyDeratingResult, o: &FitOptions) -> Vec<Check> {
    if j.achievable_tonnage_t <= 0.0 || r.gas_basis.is_none() {
        return Vec::new();
    }
    let assessed = GasCapacityResult::try_from(r)
        .map_err(|m| m.to_string())
        .and_then(|basis| {
            gs::assess(
                &basis,
                j.achievable_tonnage_t,
                o.block_coefficient,
                o.water_density_t_per_m3,
                gs::criteria_set_for(&o.ruleset),
            )
            .map_err(|e| e.message().to_string())
        });
    match assessed {
        Ok(a) => a.conditions.iter().map(stability_check).collect(),
        Err(m) => vec![Check::new(
            "Stability \u{2014} not assessed",
            false,
            format!("stability could not be assessed: {m}"),
        )],
    }
}

/// Every check of a candidate: the cheap ones, then stability.
fn search_checks(
    r: &BargeCapacityResult,
    j: &JourneyDeratingResult,
    depth: f64,
    o: &FitOptions,
) -> Vec<Check> {
    let mut all = cheap_checks(r, j, depth, o);
    all.extend(stability_checks(r, j, o));
    all
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
    water_density: f64,
) -> Result<JourneyDeratingResult, PyError> {
    let mut ji = JourneyDeratingInputs::new(r, loa, beam, depth);
    ji.block_coefficient = block_coefficient;
    ji.water_density = water_density;
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
        inp.max_tank_od_m = self.o.max_tank_od_m;
        inp.bow_rake_to_depth_ratio = self.o.bow_rake_to_depth_ratio;
        inp.pushed_stern_void_m = self.o.pushed_stern_void_m;
        inp.tank_support_height = self.o.tank_support_height_m;
        inp.dome_height = self.o.tank_dome_height_m;
        inp.dome_diameter = self.o.tank_dome_diameter_m;
        inp.wing_tank_gap_m = self.o.wing_tank_gap_m;
        inp.wing_tank_height_ratio = self.o.wing_tank_height_ratio;
        inp.tank_wall_thickness = Some(self.o.tank_wall_thickness_mm);
        inp.water_density = self.o.water_density_t_per_m3;
        inp.air_draft_limit_m = self.o.air_draft_limit_m;
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
            // Mutation survivor (#59), `cs < 0.0` to `cs == 0.0`: equivalent. `cs` is negative only
            // when the beam is under `nw` tank diameters at the cap. A full-beam tank over the cap
            // needs a beam over `nw` caps, so a negative `cs` is reached only with an unusable
            // full-beam tank of zero diameter (the tanks do not fit the hull). Under the mutant
            // the negative clearance goes on to `capacity`, which cannot make a tank over the cap
            // (the diameter is then at most the cap) and a candidate with it is turned down by
            // the rule check below: the rule needs at least 0.45 m (`d(V)` is at least 0.8 m, the
            // insulation is the fixed 350 mm), and `cs + 1e-9 < need` holds for every negative
            // `cs`. So `None` comes back either way. Searched with the mutant built in: 15 000
            // random inputs (both rulesets of the time, caps 0.2 to 8.2 m, beams 0.3 to 14.3 m, 1 to 5 tanks
            // wide), the branch reached in at least 10 068 of them, no difference in any field.
            // `a_hull_too_narrow_for_tanks_at_the_cap_gets_no_candidate` and
            // `the_side_clearance_rule_is_never_below_0_45_m` guard the two facts this rests on.
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
            // volume matters only through the rule d(V) - insulation, which is non-decreasing
            // in V (the dome adds the same volume to every tank), and continuous, so the loop that sets the clearance settles. We are here only when the full-beam tank is over the cap by more than the
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
                // Vc of one tank: the tank plus its dome (IGC 2.4.1.1).
                total / tank_divisor(r.n_tanks_total)
                    + dome_volume_m3(self.o.tank_dome_diameter_m, self.o.tank_dome_height_m),
                opt_or_zero(r.insulation_thickness_mm),
            )?;
            // Killed by `a_cap_clearance_a_hair_under_the_rule_is_accepted_not_refused` (`+` to `-`).
            if cs + 1e-9 < need {
                return Ok(None);
            }
        }
        Ok(Some((inp, r)))
    }
}

/// Every candidate the search built, and the index of the best one that passes every check.
struct Pool {
    all: Vec<Candidate>,
    best: Option<usize>,
}

/// Build every candidate and judge it by the whole check list. A candidate is judged by the cheap
/// checks first; the stability assessment runs only for one that passed them and is above the best
/// passing tonnage so far (a lower one cannot win, so its stability is not needed). With
/// `stop_at_first_pass` the search ends at the first passing candidate (for a yes or no answer).
fn search(s: &mut Search<'_>, t_max: f64, stop_at_first_pass: bool) -> Result<Pool, PyError> {
    let (loa, beam, o) = (s.loa, s.beam, s.o);
    let mut pool = Pool {
        all: Vec::new(),
        best: None,
    };
    for &d in &o.depths {
        for &nw in &o.tanks_wide {
            let mut cands: Vec<Built> = Vec::new();
            if let Some(x) = s.build(d, nw, o.tanks_in_line)? {
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
                    // Mutation survivors (#58), `&&` to `||` and `>` to `>=` below: equivalent. The
                    // two differ from the real test only for a candidate with a tank but zero
                    // capacity (a fill ratio of 0, or a product that underflows to 0); a tank
                    // diameter of 0 comes with zero capacity, so `||` adds nothing else. Such a
                    // candidate joins the search under the mutants and can never win: its payload is
                    // 0.0 (never above the 1e-6 margin over a payload that is 0.0 or more), and
                    // its `feasible_geometry` check fails (cargo volume must be positive), so it
                    // is never the best passing candidate either. It is also built last, after
                    // the base candidate, so it cannot win a tie. Searched with each mutant built
                    // in: 12 000 random inputs (rulesets, fill ratios from 0, tank caps, depths,
                    // clearances, 9 969 infeasible), and 175 824 direct capacity runs, none differ
                    // in any field. `a_zero_capacity_arrangement_is_infeasible_and_carries_nothing`
                    // guards the two facts this rests on. Not a gap in the tests.
                    if opt_or_zero(r2.tank_outer_diameter_m) != 0.0 && r2.cargo_capacity_t > 0.0 {
                        more.push((i2, r2));
                    }
                }
            }
            for (inp, r) in cands.into_iter().chain(more) {
                let j = derate_candidate(
                    &r,
                    loa,
                    beam,
                    d,
                    t_max,
                    o.block_coefficient,
                    o.water_density_t_per_m3,
                )?;
                let tonnage = j.achievable_tonnage_t;
                // Cheap checks first; stability only for a candidate that could still win.
                let could_win = pool
                    .best
                    .is_none_or(|i| tonnage > pool.all[i].tonnage + 1e-6);
                let wins = could_win
                    && all_ok(&cheap_checks(&r, &j, d, o))
                    && all_ok(&stability_checks(&r, &j, o));
                pool.all.push(Candidate {
                    tonnage,
                    depth: d,
                    result: r,
                    journey: j,
                    inputs: inp,
                });
                if wins {
                    pool.best = Some(pool.all.len() - 1);
                    if stop_at_first_pass {
                        return Ok(pool);
                    }
                }
            }
        }
    }
    Ok(pool)
}

/// The closest candidate to passing, when none passes: the fewest failed error checks, then the
/// most cargo (the first of two near-equal ones stays).
fn closest_to_passing(pool: &[Candidate], o: &FitOptions) -> Option<usize> {
    let mut best: Option<(usize, usize)> = None;
    for (i, c) in pool.iter().enumerate() {
        let failed = search_checks(&c.result, &c.journey, c.depth, o)
            .iter()
            .filter(|k| k.is_error() && !k.ok)
            .count();
        let better = match best {
            None => true,
            Some((bi, bf)) => failed < bf || (failed == bf && c.tonnage > pool[bi].tonnage + 1e-6),
        };
        if better {
            best = Some((i, failed));
        }
    }
    best.map(|(i, _)| i)
}

/// The arrangement that meets every check of the search, with the most cargo per journey at draught
/// cap `t_max` over depth x arrangement.
///
/// A candidate must pass every error-severity check (see "THE CHECKS THE SEARCH USES"), stability
/// included, to win. If none passes, the closest candidate to passing comes back with
/// `feasible == false` and its failing checks in `failed_checks`; it is not an answer, so check
/// `feasible` before using the payload.
pub fn fit(
    loa: f64,
    beam: f64,
    cargo: &str,
    t_max: f64,
    o: &FitOptions,
) -> Result<ArrangementFit, PyError> {
    if !RULESETS.contains(&o.ruleset.as_str()) {
        return Err(value_error(
            "ruleset must be one of ('IGC_2G',)".to_string(),
        ));
    }
    let mut s = Search {
        loa,
        beam,
        cargo,
        o,
        tried: 0,
    };
    let pool = search(&mut s, t_max, false)?;
    let pick = pool.best.or_else(|| closest_to_passing(&pool.all, o));
    let mut all = pool.all;
    // Nothing passes: report the closest failing candidate, flagged infeasible.
    let best = match pick {
        Some(i) => all.swap_remove(i),
        None => {
            // Nothing fitted: report the plain single-tank attempt, which carries nothing.
            let d = *o.depths.first().ok_or_else(|| PyError {
                kind: "IndexError",
                message: "list index out of range".to_string(),
            })?;
            let inp = s.base_inputs(d, 1, o.tanks_in_line);
            let r = capacity(&inp)?;
            let j = derate_candidate(
                &r,
                loa,
                beam,
                d,
                t_max,
                o.block_coefficient,
                o.water_density_t_per_m3,
            )?;
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
    let checks = search_checks(&r, &j, d, o);
    let failed: Vec<String> = checks
        .iter()
        .filter(|c| c.is_error() && !c.ok)
        .map(|c| format!("{}: {}", c.name, c.message))
        .collect();
    let feasible = all_ok(&checks);
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

/// Whether some arrangement of this hull meets every check of the search (the first one found
/// ends the search).
fn has_passing_arrangement(
    loa: f64,
    beam: f64,
    cargo: &str,
    t_max: f64,
    o: &FitOptions,
) -> Result<bool, PyError> {
    if !RULESETS.contains(&o.ruleset.as_str()) {
        return Err(value_error(
            "ruleset must be one of ('IGC_2G',)".to_string(),
        ));
    }
    let mut s = Search {
        loa,
        beam,
        cargo,
        o,
        tried: 0,
    };
    Ok(search(&mut s, t_max, true)?.best.is_some())
}

/// Step of the upward scan of `smallest_viable_loa` (m).
const VIABLE_LOA_SCAN_STEP_M: f64 = 2.0;

/// Shortest LOA (to 0.1 m) at which some arrangement of the hull meets every check of the search
/// (the same list as `fit`, stability included). Infinity when no LOA from `lo` to `hi` has one.
///
/// Whether a hull is viable is not monotone in its length: a long hull can fail length over
/// diameter, and stability moves with length. So the search scans upward from `lo` in steps of
/// [`VIABLE_LOA_SCAN_STEP_M`] to the first viable length, then halves the last step to 0.1 m. The
/// length it returns always has a passing arrangement. Python defaults `depths` to the first
/// default depth only for this function: start from [`FitOptions::for_smallest_viable_loa`].
pub fn smallest_viable_loa(
    beam: f64,
    cargo: &str,
    t_max: f64,
    lo: f64,
    hi: f64,
    o: &FitOptions,
) -> Result<f64, PyError> {
    let mut below = lo;
    let mut at = lo;
    loop {
        if has_passing_arrangement(at, beam, cargo, t_max, o)? {
            break;
        }
        if at >= hi {
            return Ok(f64::INFINITY);
        }
        below = at;
        at = (at + VIABLE_LOA_SCAN_STEP_M).min(hi);
    }
    let (mut lo, mut hi) = (below, at);
    while hi - lo > 0.1 {
        let mid = 0.5 * (lo + hi);
        if has_passing_arrangement(mid, beam, cargo, t_max, o)? {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Ok(round_to(hi, 1))
}
