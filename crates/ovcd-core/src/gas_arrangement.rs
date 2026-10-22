//! Best gas-tank arrangement for a hull: search, do not trust the single AUTO answer.
//! Port of `core/gas_arrangement.py`. The reasoning behind the search lives in the Python file.
//!
//! Only IGC rules apply to the gas tanks (issue #175): `IGC_2G`, the per-tank rule of IGC 2.4.1.1
//! with the datum of 2.4.2 (issue #237). Two tanks lie
//! along the hull. Every tank has to keep to the outer diameter cap (`FitOptions::max_tank_od_m`)
//! and to length over inner diameter at most [`MAX_TANK_LENGTH_OVER_OD`] (the diameter of a tank
//! shell is its internal diameter, ADN 2025 1.2.1; issue #416). A candidate over that is not
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

use crate::aft_arrangement::AftDesign;
use crate::barge_capacity::{
    BargeCapacityInputs, BargeCapacityResult, DEFAULT_BOW_RAKE_TO_DEPTH_RATIO,
    DEFAULT_MAX_TANK_OD_M, DEFAULT_PUSHED_STERN_VOID_M, DEFAULT_TANK_DOME_DIAMETER_M,
    DEFAULT_TANK_DOME_HEIGHT_M, DEFAULT_TANK_SUPPORT_HEIGHT_M, DEFAULT_TANK_WALL_THICKNESS_MM,
    DEFAULT_TANKS_IN_LINE, DEFAULT_WEATHER_SHIELD_CLEARANCE_M, DEFAULT_WEATHER_SHIELD_THICKNESS_MM,
    DEFAULT_WING_TANK_GAP_M, DEFAULT_WING_TANK_HEIGHT_RATIO, TankDesign, compute as capacity,
    dome_dimensions, dome_volume_m3,
};
use crate::checks::{Check, all_ok};
use crate::gas_stability::{self as gs, DEFAULT_WATER_DENSITY, GasCapacityResult};
use crate::journey_derating::{JourneyDeratingInputs, JourneyDeratingResult, compute as derate};
use crate::loading_conditions as lc;
use crate::measured::{Measured, deadweight_band};
use crate::models::DEFAULT_TANK_GAP_TRANSVERSE_M;
use crate::pressure_vessel::PyError;
use crate::protective_location::d_vc_exact;
use crate::pyfmt::{fixed, py_str, round_to};
use crate::rule_trace::{
    Failure, MeasureInfo, MeasureSource, MeasureValue, ObjectType, Outcome, Trace, evaluate,
    failures, measure_def,
};
use crate::rules::{ArgValue, MeasureRef, RuleSet};
use crate::tanks::INT_OVERFLOW_MESSAGE;
use crate::text;
use crate::text::Arg;

/// The rulesets `fit` accepts.
pub const RULESETS: [&str; 1] = ["IGC_2G"];
/// Largest overall tank length (heads included) over the inner shell diameter (issue #416), for
/// every pressure tank. A design limit of this tool. The figure has two sources, read in the
/// library: ADN 2025 9.3.1.11.1(b) ("Pressure tanks whose ratio of length to diameter exceeds 7
/// are prohibited") and the inland class rules Pt 4 Ch 4 3.5.1, now the class Rule
/// `lr-inland/3.5.1/tank-l-over-d` (issue #424). No IGC Code paragraph gives it.
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
    /// Cargo density (kg/m^3) in place of the cargo data's (issue #315); `None`: the cargo data's.
    pub cargo_density_kg_m3: Option<f64>,
    /// Issue #348: `Some` puts the stern block's mass into the lightship of a self-propelled hull.
    pub stern_block_mass: Option<crate::aft_arrangement::SternMassInputs>,
    /// Issue #350: the Stern Form, when it is not today's square stern.
    pub stern_geometry: Option<crate::stern_form::SternGeometry>,
    /// Issue #352: the machinery of a specified Propulsor (`BargeCapacityInputs::machinery`).
    pub machinery: Option<crate::machinery::MachineryInputs>,
    /// Issue #354: the Blind Distance limit (`BargeCapacityInputs::sightline_limit_m`).
    pub sightline_limit_m: Option<f64>,
    pub self_propelled: bool,
    pub depths: Vec<f64>,
    pub tanks_wide: Vec<i64>,
    pub extra_clearance_m: Vec<f64>,
    /// Issue #448: Cb only for the IGC 4.28.2 vertical acceleration; the hydrostatics come from
    /// each candidate's sections.
    pub block_coefficient: f64,
    /// Cap on a tank's outer diameter (m).
    pub max_tank_od_m: f64,
    /// Bow rake as a multiple of the moulded depth of each candidate.
    pub bow_rake_to_depth_ratio: f64,
    /// Issue #446: a typed bow rake length (m) for every candidate; `None`: the ratio.
    pub bow_rake_length_m: Option<f64>,
    /// Issue #446: the keel height at the bow (m); `None`: each candidate's moulded depth.
    pub bow_rake_rise_m: Option<f64>,
    /// Issue #449: bilge radius, sheer and camber of every candidate.
    pub hull_shape: crate::hull::hull_form::HullShape,
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
    /// Issue #316: `true` takes the dome diameter, or height, from the dome rule of each tank
    /// (`barge_capacity::dome_dimensions`) instead of the value above.
    pub tank_dome_diameter_by_rule: bool,
    pub tank_dome_height_by_rule: bool,
    /// Gap between the wing tank and the cargo tank, per side (m).
    pub wing_tank_gap_m: f64,
    /// Wing tank height as a fraction of the cargo tank outer diameter.
    pub wing_tank_height_ratio: f64,
    /// Plate thickness of the weather shield over the tanks (mm); 0: no shield (issue #410).
    pub weather_shield_thickness_mm: f64,
    /// Clearance from the tank top to the weather shield top (m, issue #410).
    pub weather_shield_clearance_m: f64,
    /// Forming allowance on the tank plate, % of the required thickness (issue #412).
    pub tank_forming_allowance_pct: f64,
    /// Wall thickness of each tank's dome (mm); 0: no dome mass (issue #412).
    pub tank_dome_wall_thickness_mm: f64,
    /// The end-bulkhead space aft and forward of the tanks (m): the type G rule for the tank end
    /// shape (`gas_separation::default_end_space`), or the Option's own values (issue #280).
    pub aft_end_space_m: f64,
    pub fwd_end_space_m: f64,
    /// Fixed tank wall thickness (mm); `None`: the pressure-vessel formula gives it (issue #292).
    pub tank_wall_thickness_mm: Option<f64>,
    /// The Cargo-dependent tank design values: pressure, steel, insulation, structure factor.
    pub tank_design: TankDesign,
    /// Water density for the draught of every candidate and its journey de-rating (t/m3).
    pub water_density_t_per_m3: f64,
    /// Least freeboard at the carried draught (m): a candidate that carries its payload deeper
    /// than the hull depth less this fails `freeboard_at_payload`.
    pub required_freeboard_m: f64,
    /// Air draft limit of the light condition (m), as the capacity check reads it. `None`: none.
    pub air_draft_limit_m: Option<f64>,
    /// The stern block's design values for every candidate (a self-propelled hull only).
    pub aft_design: AftDesign,
    /// The selected Rule Sets, in selection order (spec 0004, "How `fit` uses it", issue #277). A
    /// candidate passes when its Rule Trace has no `fail` row. Empty: the search judges by its own
    /// check list (see "THE CHECKS THE SEARCH USES"), as the Python API does.
    pub rules: Vec<RuleSet>,
    /// The Option's inputs, for the input Measures and the Applicability of [`FitOptions::rules`].
    pub rule_inputs: RuleInputs,
    /// Issue #426: `Some` checks every candidate's stability in the IS Code 2008 loading
    /// conditions (`loading_conditions`) in place of the legacy laden, half laden and light ones,
    /// and ranks it by the cargo it carries with its stores and fuel aboard. `None`: legacy.
    pub is_code: Option<IsCodeConditions>,
}

/// What the IS Code 2008 loading conditions of a candidate need beyond its own result (issue #426).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct IsCodeConditions {
    /// The stores and fuel of the full load (t). 0 for a barge.
    pub consumables_t: f64,
    pub may_sail_without_ballast: bool,
    /// The user's ballast fills by tank group, departure then arrival; `None`: the least ballast.
    pub user_ballast: Option<[lc::GroupFills; 2]>,
}

/// The Option's inputs as the Rule Trace reads them: each enumerated input's text value (for
/// Applicability), and each input Measure with its registry entry and value.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RuleInputs {
    pub applicability: Vec<(String, String)>,
    pub measures: Vec<(String, MeasureInfo, MeasureValue)>,
}

/// The Measures of one candidate of the search.
struct CandidateSource<'a> {
    measured: &'a Measured,
    inputs: &'a RuleInputs,
}

impl MeasureSource for CandidateSource<'_> {
    fn def(&self, id: &str) -> Option<MeasureInfo> {
        if let Some(d) = measure_def(id) {
            return Some(d.info());
        }
        self.inputs
            .measures
            .iter()
            .find(|(n, _, _)| n == id)
            .map(|(_, info, _)| info.clone())
    }
    fn objects(&self, object: &str) -> Vec<String> {
        self.measured.objects_by_key(object, "Candidate")
    }
    fn value(&self, m: &MeasureRef, index: usize) -> Result<MeasureValue, String> {
        // The Option's own values: each input Measure, and the Measures its typed inputs give
        // (issue #417).
        let own = self
            .inputs
            .measures
            .iter()
            .find(|(n, _, _)| *n == m.id)
            .map(|(_, _, v)| v.clone());
        if m.id.starts_with("input.") {
            return own.ok_or_else(|| format!("the Option has no `{}`", m.id));
        }
        if let Some(v) = own {
            return Ok(v);
        }
        self.measured.value(m, index)
    }
    fn input(&self, name: &str) -> Option<String> {
        if name == "deadweight_band" {
            return self
                .measured
                .deadweight_t
                .map(|dw| deadweight_band(dw).to_string());
        }
        self.inputs
            .applicability
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.clone())
    }
    fn object_input(&self, object: &str, index: usize, name: &str) -> Option<String> {
        self.measured
            .object_input(object, index, name)
            .or_else(|| self.input(name))
    }
    fn has(&self, m: &MeasureRef) -> bool {
        self.measured.has(m)
    }
}

/// What the Rule Trace of a candidate reads: its checks (named as a result lists them), its
/// stability conditions (none before the costly stage) and its tank and separation values.
fn candidate_measured(
    r: &BargeCapacityResult,
    j: &JourneyDeratingResult,
    inp: &BargeCapacityInputs,
    checks: &[Check],
    conditions: Vec<gs::Condition>,
) -> Measured {
    let plane = inp.separation.as_ref().is_some_and(|s| s.plane_tank_ends);
    Measured {
        conditions,
        deadweight_t: Some(j.achievable_tonnage_t),
        tank: r.tank_rules.clone(),
        separation: r.separation.clone().map(|s| (s, plane)),
        checks: checks
            .iter()
            .map(|c| (c.name.replace('_', " "), c.ok))
            .collect(),
        // The wheelhouse and the powering are worked out for the answer, not for a candidate.
        ..Measured::default()
    }
}

/// The whole Rule Trace of a candidate. Built once, for the answer.
fn candidate_trace(measured: &Measured, o: &FitOptions) -> Trace {
    let sets: Vec<&RuleSet> = o.rules.iter().collect();
    evaluate(
        &sets,
        &CandidateSource {
            measured,
            inputs: &o.rule_inputs,
        },
    )
}

/// The failing rows of a candidate's Rule Trace, without building the trace: the search calls this
/// on every candidate.
fn candidate_failures(measured: &Measured, o: &FitOptions) -> Vec<Failure> {
    let sets: Vec<&RuleSet> = o.rules.iter().collect();
    failures(
        &sets,
        &CandidateSource {
            measured,
            inputs: &o.rule_inputs,
        },
    )
}

/// The failing rows of a trace, grouped as the check list groups them: the rows of one loading
/// condition count once (one stability check per condition), the Rules on one engine check count
/// once (a class Rule and a tool limit can both report it, #424), each other failing Rule once.
fn failing_groups(fails: &[Failure]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for f in fails {
        let check = match (f.measure.id.as_str(), f.measure.args.as_slice()) {
            ("check.ok", [(_, ArgValue::Text(name))]) => Some(name),
            _ => None,
        };
        let key = match (
            f.object_type.as_deref().and_then(ObjectType::from_key),
            check,
        ) {
            (Some(ObjectType::LoadingCondition), _) => {
                format!(
                    "Stability \u{2014} {}",
                    f.object.clone().unwrap_or_default()
                )
            }
            (_, Some(name)) => format!("check {name}"),
            _ => f.rule.clone(),
        };
        if !out.contains(&key) {
            out.push(key);
        }
    }
    out
}

impl Default for FitOptions {
    fn default() -> Self {
        Self {
            ruleset: "IGC_2G".to_string(),
            fill_ratio: 0.95,
            cargo_density_kg_m3: None,
            stern_block_mass: None,
            stern_geometry: None,
            machinery: None,
            sightline_limit_m: None,
            self_propelled: true,
            depths: DEFAULT_DEPTHS_M.to_vec(),
            tanks_wide: DEFAULT_TANKS_WIDE.to_vec(),
            extra_clearance_m: DEFAULT_EXTRA_CLEARANCE_M.to_vec(),
            block_coefficient: 0.85,
            max_tank_od_m: DEFAULT_MAX_TANK_OD_M,
            bow_rake_to_depth_ratio: DEFAULT_BOW_RAKE_TO_DEPTH_RATIO,
            bow_rake_length_m: None,
            bow_rake_rise_m: None,
            hull_shape: crate::hull::hull_form::HullShape::default(),
            pushed_stern_void_m: DEFAULT_PUSHED_STERN_VOID_M,
            tanks_in_line: DEFAULT_TANKS_IN_LINE,
            tank_support_height_m: DEFAULT_TANK_SUPPORT_HEIGHT_M,
            tank_dome_height_m: DEFAULT_TANK_DOME_HEIGHT_M,
            tank_dome_diameter_m: DEFAULT_TANK_DOME_DIAMETER_M,
            tank_dome_diameter_by_rule: false,
            tank_dome_height_by_rule: false,
            wing_tank_gap_m: DEFAULT_WING_TANK_GAP_M,
            wing_tank_height_ratio: DEFAULT_WING_TANK_HEIGHT_RATIO,
            weather_shield_thickness_mm: DEFAULT_WEATHER_SHIELD_THICKNESS_MM,
            weather_shield_clearance_m: DEFAULT_WEATHER_SHIELD_CLEARANCE_M,
            // As the legacy capacity inputs: neither is weighed unless a Design asks (issue #412).
            tank_forming_allowance_pct: 0.0,
            tank_dome_wall_thickness_mm: 0.0,
            aft_end_space_m: crate::gas_separation::default_end_space(false),
            fwd_end_space_m: crate::gas_separation::default_end_space(false),
            tank_wall_thickness_mm: Some(DEFAULT_TANK_WALL_THICKNESS_MM),
            tank_design: TankDesign::default(),
            water_density_t_per_m3: DEFAULT_WATER_DENSITY,
            required_freeboard_m: DEFAULT_REQUIRED_FREEBOARD_M,
            air_draft_limit_m: None,
            aft_design: AftDesign::default(),
            rules: Vec::new(),
            rule_inputs: RuleInputs::default(),
            is_code: None,
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

/// The length over diameter check of a candidate's tank: the overall tank length, heads included,
/// over the inner diameter of the steel shell. ADN 2025 9.3.1.11.1 (b) and the inland class rules
/// Pt 4 Ch 4 3.5.1 set the ratio at 7; ADN 1.2.1 defines the diameter of a tank shell as its
/// internal diameter (maintainer decision on issue #416; it was the insulated outer diameter).
/// `None` when there is no tank (a bulk result, or a hull with no cargo area), which the other
/// checks already report.
pub fn length_over_diameter_check(r: &BargeCapacityResult) -> Option<Check> {
    let tanks = &r.gas_basis.as_ref()?.tanks;
    if tanks.outer_diameter <= 0.0 || tanks.inner_diameter <= 0.0 {
        return None;
    }
    let ratio = tanks.overall_tank_length / tanks.inner_diameter;
    Some(Check::new(
        "tank_length_over_diameter",
        ratio <= MAX_TANK_LENGTH_OVER_OD,
        text!(
            "check-tank-length-over-inner-diameter",
            "tank length {length} m / inner diameter {diameter} m = {ratio} <= {max}",
            length = Arg::fixed(tanks.overall_tank_length, 2),
            diameter = Arg::fixed(tanks.inner_diameter, 2),
            ratio = Arg::fixed(ratio, 2),
            max = Arg::py(MAX_TANK_LENGTH_OVER_OD),
        ),
    ))
}

// ---------------------------------------------------------------------------------------------
// THE CHECKS THE SEARCH USES (issue #264). This is the one list. `fit` and `smallest_viable_loa`
// keep a candidate only if every error-severity check below passes, then rank the passing ones by
// cargo. When the rules engine (#176) lands, it replaces this list with the Design's selected rules.
//
// Cheap checks (run first, from numbers the search has already computed):
//   - the capacity result's own checks: feasible geometry, cargo zone, the aft arrangement
//     (thrusters, containers, wheelhouse), the gas separation (the cofferdam, the opening
//     setback and the hold end space of the inland class rules; the IGC protective location
//     clearances are tank Rules, not checks of this list),
//     and the air draft limit of the light condition when one is set. Not `not_overloaded` or
//     `freeboard_ok` (`FULL_VOLUME_CHECKS`): they test the full tank volume, and a candidate is
//     de-rated to its payload, so they do not gate (maintainer decision on issue #418);
//   - the journey checks: the channel draught limit leaves a payload that floats;
//   - `tank_length_over_diameter`: overall tank length over inner shell diameter, at most 7;
//   - `freeboard_at_payload`: the hull depth less the carried draught, at least the required freeboard.
// Costly check (run only for a candidate that passed the cheap ones and could still win):
//   - stability: IS Code 2008 Part A 2.2, each loading condition: the legacy ones
//     (`gas_stability::assess`), or with `FitOptions::is_code` the Part B 3.4.1.2 ones
//     (`is_code_conditions`), with their ballast plan and air draft (issue #426).
// A check of warning severity is advisory: it never gates (`all_ok`), as everywhere else.
// ---------------------------------------------------------------------------------------------

/// The capacity checks that test the full tank volume, not the carried payload. They are
/// information: the search does not gate on them (issue #418). The carried draught is gated by
/// `freeboard_at_payload` and the journey's `floats` check.
pub const FULL_VOLUME_CHECKS: &[&str] = &["not_overloaded", "freeboard_ok"];

/// The cheap checks of a candidate: see the list above.
fn cheap_checks(
    r: &BargeCapacityResult,
    j: &JourneyDeratingResult,
    depth: f64,
    o: &FitOptions,
) -> Vec<Check> {
    let mut all: Vec<Check> = r
        .checks
        .iter()
        .filter(|c| !FULL_VOLUME_CHECKS.contains(&c.name.as_str()))
        .cloned()
        .collect();
    all.extend(j.checks.iter().cloned());
    all.extend(length_over_diameter_check(r));
    let freeboard = depth - j.draft_at_achievable_m;
    all.push(Check::new(
        "freeboard_at_payload",
        freeboard + 1e-9 >= o.required_freeboard_m,
        text!(
            "check-freeboard-at-payload",
            "freeboard {freeboard} m at the carried draught {draught} m >= required {required} m",
            freeboard = Arg::fixed(freeboard, 2),
            draught = Arg::fixed(j.draft_at_achievable_m, 2),
            required = Arg::py(o.required_freeboard_m),
        ),
    ));
    all
}

/// The criterion name of IS Code 2008 Part A 2.2.3, the angle of maximum GZ.
const ANGLE_CRITERION: &str = "Angle of max GZ";

/// Whether the condition is one of the two with no ballast that OVCD adds or keeps: "Lightship, no
/// ballast" of the IS Code conditions and the legacy "Light" (issue #498).
fn is_light_without_ballast(name: &str) -> bool {
    name == "Lightship, no ballast" || name == "Light"
}

/// One stability condition as a check, named and worded as the simulation lists it.
pub fn stability_check(c: &gs::Condition) -> Check {
    stability_check_with(c, false)
}

/// [`stability_check`] for a gas Design.
///
/// Issue #498: in a light condition with no ballast, a failure where the angle of maximum GZ
/// (2.2.3) is the only failing criterion is a warning, not a failure: the bilge emerges early on a
/// wide, shallow hull, and the vessel can sail light with ballast. IS Code 2.2.3 allows alternative
/// criteria only with the approval of the Administration, which OVCD does not model, so the
/// alternative is not taken as a pass. A dry-bulk Design keeps its checks as they were (all its
/// stability checks are warnings already).
pub fn gas_stability_check(c: &gs::Condition, warns: bool) -> Check {
    stability_check_with(c, warns)
}

/// The Rule that fails the angle of max GZ of every condition in the IS Code Rule Sets of 1.0.0.
const ANGLE_RULE: &str = "is2008/A.2.2.3/angle-max-gz";

/// Whether the angle of max GZ alone in a light condition with no ballast is a warning for these
/// Rule Sets (issue #498). It is when the Rule Sets say so: the IS Code Rule that holds the angle
/// names the kind of loading condition it applies to. A stack whose Rule applies to every
/// condition (`is2008-a` 1.0.0) keeps the failure, so a file saved before keeps its results. No
/// Rule Sets (the Python API) means the warning.
pub fn light_angle_warns(rules: &[RuleSet]) -> bool {
    let mut found = rules
        .iter()
        .flat_map(|set| set.rules.iter())
        .filter(|r| r.id == ANGLE_RULE)
        .peekable();
    if found.peek().is_none() {
        return true;
    }
    found.all(|r| {
        r.applies_when
            .conditions
            .iter()
            .any(|(input, _)| input == "loading_condition_kind")
    })
}

fn stability_check_with(c: &gs::Condition, gas: bool) -> Check {
    let bad: Vec<&gs::Criterion> = c.criteria.iter().filter(|x| !x.ok).collect();
    let gm = Arg::fixed(c.gm_fluid_m, 2);
    let max_gz = Arg::fixed(c.max_gz_m, 2);
    let angle = Arg::fixed(f64::from(c.max_gz_deg), 0);
    if gas && is_light_without_ballast(&c.name) && bad.len() == 1 && bad[0].name == ANGLE_CRITERION
    {
        let value = match bad[0].value {
            gs::PyNum::Int(i) => i as f64,
            gs::PyNum::Float(v) => v,
        };
        return Check::with_severity(
            format!("Stability \u{2014} {}", c.name.to_lowercase()),
            false,
            text!(
                "check-stability-light-angle",
                "GM {gm} m (fluid), max GZ {max_gz} m at {angle}\u{b0} \u{2014} warning, not a failure: \
                 angle of max GZ {value} < {limit} (IS Code 2.2.3) with no ballast; every other criterion holds. \
                 Sail light with ballast",
                gm = gm,
                max_gz = max_gz,
                angle = angle,
                value = Arg::fixed(value, 0),
                limit = Arg::fixed(bad[0].limit, 0),
            ),
            "warning",
        );
    }
    let message = if bad.is_empty() {
        text!(
            "check-stability",
            "GM {gm} m (fluid), max GZ {max_gz} m at {angle}\u{b0}",
            gm = gm,
            max_gz = max_gz,
            angle = angle,
        )
    } else {
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
        // The failed criteria are one English list (criterion names, values and limits).
        text!(
            "check-stability-fails",
            "GM {gm} m (fluid), max GZ {max_gz} m at {angle}\u{b0} \u{2014} fails: {fails}",
            gm = gm,
            max_gz = max_gz,
            angle = angle,
            fails = parts.join(", "),
        )
    };
    Check::new(
        format!("Stability \u{2014} {}", c.name.to_lowercase()),
        bad.is_empty(),
        message,
    )
}

/// The checks of one IS Code loading condition (issue #426): its stability, and when they fail,
/// its ballast plan (none meets every criterion) and its air draft.
/// The checks of every IS Code condition of a Design, in order. The "Lightship, no ballast" warning of
/// issue #498 also names the ballast the "Ballast, departure" condition needs to meet every criterion.
pub fn loading_condition_checks_of(conds: &[lc::LoadedCondition], warns: bool) -> Vec<Check> {
    let ballast = conds
        .iter()
        .find(|c| c.role == lc::Role::BallastDeparture && c.ok() && c.ballast_t > 0.0)
        .map(|c| c.ballast_t);
    let mut out = Vec::new();
    for c in conds {
        let mut checks = loading_condition_checks_with(c, warns);
        if let (Some(t), Some(first)) = (ballast, checks.first_mut())
            && c.role == lc::Role::LightshipNoBallast
            && first.severity == "warning"
        {
            let (gm, max_gz, angle) = (
                Arg::fixed(c.condition.gm_fluid_m, 2),
                Arg::fixed(c.condition.max_gz_m, 2),
                Arg::fixed(f64::from(c.condition.max_gz_deg), 0),
            );
            let name = first.name.clone();
            *first = Check::with_severity(
                name,
                false,
                text!(
                    "check-stability-light-angle-ballast",
                    "GM {gm} m (fluid), max GZ {max_gz} m at {angle}\u{b0} \u{2014} warning, not a failure: \
                 angle of max GZ {angle} < 25 (IS Code 2.2.3) with no ballast; every other criterion holds. \
                 To sail light, take {ballast} t of ballast (the Ballast, departure condition meets every criterion)",
                    gm = gm,
                    max_gz = max_gz,
                    angle = angle,
                    ballast = Arg::fixed(t, 0),
                ),
                "warning",
            );
        }
        out.extend(checks);
    }
    out
}

pub fn loading_condition_checks(c: &lc::LoadedCondition) -> Vec<Check> {
    loading_condition_checks_with(c, false)
}

fn loading_condition_checks_with(c: &lc::LoadedCondition, gas: bool) -> Vec<Check> {
    let mut out = vec![stability_check_with(&c.condition, gas)];
    if c.plan_found == Some(false) {
        out.push(Check::new(
            "Ballast plan",
            false,
            text!(
                "check-no-ballast-plan",
                "{condition}: no ballast plan meets every criterion, even with every ballast tank full.",
                condition = c.condition.name.clone(),
            ),
        ));
    }
    if c.air_draft_ok == Some(false) {
        out.push(Check::new(
            "Air draft",
            false,
            text!(
                "check-condition-air-draft",
                "{condition}: air draft {air} m is above the limit.",
                condition = c.condition.name.clone(),
                air = Arg::fixed(c.air_draft_m, 2),
            ),
        ));
    }
    out
}

/// The cargo of the IS Code full-load conditions (issue #426): the payload, but no more than the
/// displacement at the draught cap less the lightship and the stores and fuel. A draught-bound
/// cargo is that much less; a volume-bound one does not change.
pub fn is_code_cargo(
    r: &BargeCapacityResult,
    payload_t: f64,
    loa: f64,
    beam: f64,
    t_max: f64,
    o: &FitOptions,
    ic: &IsCodeConditions,
) -> f64 {
    // Issue #448: the hull's displacement at the draught cap, from its sections.
    let _ = (loa, beam);
    let displaced = r
        .hull_spec
        .as_ref()
        .and_then(|spec| crate::hydrostatics::Hull::build(spec).ok())
        .map(|h| h.volume_at(t_max))
        .unwrap_or(0.0);
    let room = displaced * o.water_density_t_per_m3 - r.lightship_t - ic.consumables_t;
    let room = if room > 0.0 { room } else { 0.0 };
    if room < payload_t { room } else { payload_t }
}

/// The IS Code loading conditions of a gas arrangement with `cargo_t` in the full loads (issue
/// #426): its lightship and cargo model, its ballast tanks and the stores and fuel of `ic`.
/// Empty for a result with no gas basis.
pub fn is_code_conditions(
    r: &BargeCapacityResult,
    inp: &BargeCapacityInputs,
    cargo_t: f64,
    o: &FitOptions,
    ic: &IsCodeConditions,
) -> Result<Vec<lc::LoadedCondition>, gs::PyError> {
    let Ok(basis) = GasCapacityResult::try_from(r) else {
        return Ok(Vec::new());
    };
    let Some((lightship, cargo)) = lc::gas_model(&basis) else {
        return Ok(Vec::new());
    };
    let cargo_at = |m: f64| cargo.at(m);
    let tanks = crate::compartments::ballast_tanks(inp, r);
    let ballast_plans = match &ic.user_ballast {
        Some([departure, arrival]) => [departure.plan(&tanks), arrival.plan(&tanks)],
        None => [lc::BallastPlan::Auto, lc::BallastPlan::Auto],
    };
    // The consumables tanks sit on the bottom at the aft end, in the stern block.
    let consumables = lc::Consumables {
        full_t: ic.consumables_t,
        tanks: crate::compartments::consumables_tanks(inp.beam, 0.0, ic.consumables_t),
        density: 1.0,
    };
    let Some(spec) = r.hull_spec.as_ref() else {
        return Ok(Vec::new());
    };
    let hull = crate::hydrostatics::Hull::build(spec)
        .map_err(|e| gs::PyError::Value(alloc::format!("{e}")))?;
    lc::assess(&lc::Inputs {
        loa: inp.loa,
        beam: inp.beam,
        depth: inp.depth,
        hull: &hull,
        water_density: o.water_density_t_per_m3,
        criteria_set: gs::criteria_set_for(&o.ruleset),
        lightship: &lightship,
        full_cargo_t: cargo_t,
        cargo_at: &cargo_at,
        consumables,
        ballast_tanks: &tanks,
        ballast_density: o.water_density_t_per_m3,
        ballast_plans,
        may_sail_without_ballast: ic.may_sail_without_ballast,
        air_draft_top_m: r.air_draft_top_above_keel_m,
        air_draft_limit_m: o.air_draft_limit_m,
    })
}

/// What the search ranks a candidate by: its payload, or with the IS Code conditions the cargo of
/// their full loads.
fn ranked_cargo(
    r: &BargeCapacityResult,
    j: &JourneyDeratingResult,
    inp: &BargeCapacityInputs,
    o: &FitOptions,
) -> f64 {
    match &o.is_code {
        Some(ic) => is_code_cargo(
            r,
            j.achievable_tonnage_t,
            inp.loa,
            inp.beam,
            j.t_max_m,
            o,
            ic,
        ),
        None => j.achievable_tonnage_t,
    }
}

/// The stability checks of a candidate, one per loading condition. None when it carries nothing or
/// has no gas tanks (the cheap checks report that). An assessment that cannot be made is one failed
/// check, so that a candidate the engine cannot judge never passes and never stops the search.
fn stability_checks(
    r: &BargeCapacityResult,
    j: &JourneyDeratingResult,
    inp: &BargeCapacityInputs,
    o: &FitOptions,
) -> Vec<Check> {
    stability_assessment(r, j, inp, o).0
}

/// The stability checks of a candidate and the loading conditions they come from (none when the
/// candidate carries nothing or the assessment failed).
fn stability_assessment(
    r: &BargeCapacityResult,
    j: &JourneyDeratingResult,
    inp: &BargeCapacityInputs,
    o: &FitOptions,
) -> (Vec<Check>, Vec<gs::Condition>) {
    if j.achievable_tonnage_t <= 0.0 || r.gas_basis.is_none() {
        return (Vec::new(), Vec::new());
    }
    let assessed = match &o.is_code {
        Some(ic) => is_code_conditions(r, inp, ranked_cargo(r, j, inp, o), o, ic)
            .map(|conds| {
                (
                    loading_condition_checks_of(&conds, light_angle_warns(&o.rules)),
                    conds.into_iter().map(|c| c.condition).collect(),
                )
            })
            .map_err(|e| e.message().to_string()),
        None => GasCapacityResult::try_from(r)
            .map_err(|m| m.to_string())
            .and_then(|basis| {
                gs::assess(
                    &basis,
                    j.achievable_tonnage_t,
                    o.water_density_t_per_m3,
                    gs::criteria_set_for(&o.ruleset),
                )
                .map_err(|e| e.message().to_string())
            })
            .map(|a| {
                (
                    a.conditions
                        .iter()
                        .map(|c| gas_stability_check(c, light_angle_warns(&o.rules)))
                        .collect(),
                    a.conditions,
                )
            }),
    };
    match assessed {
        Ok(a) => a,
        Err(m) => (
            vec![Check::new(
                "Stability \u{2014} not assessed",
                false,
                text!(
                    "check-stability-not-assessed",
                    "stability could not be assessed: {reason}",
                    reason = m,
                ),
            )],
            Vec::new(),
        ),
    }
}

/// Whether a candidate passes its costly stage: every check, stability included, or, with Rule Sets
/// selected, no `fail` row in its Rule Trace.
fn passes_all(c: &Candidate, o: &FitOptions) -> bool {
    if o.rules.is_empty() {
        return all_ok(&stability_checks(&c.result, &c.journey, &c.inputs, o));
    }
    candidate_failures(&full_measured(c, o), o).is_empty()
}

/// What the whole Rule Trace of a candidate reads: every check and the stability conditions.
fn full_measured(c: &Candidate, o: &FitOptions) -> Measured {
    let (stab, conditions) = stability_assessment(&c.result, &c.journey, &c.inputs, o);
    let mut checks = cheap_checks(&c.result, &c.journey, c.depth, o);
    checks.extend(stab);
    candidate_measured(&c.result, &c.journey, &c.inputs, &checks, conditions)
}

/// Every check of a candidate: the cheap ones, then stability.
fn search_checks(
    r: &BargeCapacityResult,
    j: &JourneyDeratingResult,
    inp: &BargeCapacityInputs,
    depth: f64,
    o: &FitOptions,
) -> Vec<Check> {
    let mut all = cheap_checks(r, j, depth, o);
    all.extend(stability_checks(r, j, inp, o));
    all
}

/// The checks the search applies to the answer beyond its capacity result's own checks and its
/// stability: the journey checks, tank length over diameter and freeboard at the payload. A
/// result lists them, so every check that can remove a candidate is visible (issue #418).
pub fn answer_search_checks(f: &ArrangementFit, o: &FitOptions) -> Result<Vec<Check>, PyError> {
    let r = &f.engine_result;
    let j = derate_candidate(
        r,
        f.loa_m,
        f.beam_m,
        f.depth_m,
        f.t_max_m,
        o.water_density_t_per_m3,
    )?;
    let own: Vec<&str> = r.checks.iter().map(|c| c.name.as_str()).collect();
    Ok(cheap_checks(r, &j, f.depth_m, o)
        .into_iter()
        .filter(|c| !own.contains(&c.name.as_str()))
        .collect())
}

/// One built candidate: the inputs and the capacity result they gave.
type Built = (BargeCapacityInputs, BargeCapacityResult);

/// One scored candidate, as Python's `(payload, depth, result, journey, inputs)` tuple.
#[derive(Clone)]
struct Candidate {
    /// How many cheap checks fail (see "THE CHECKS THE SEARCH USES"), counted as
    /// `closest_to_passing` counts: never more than the candidate fails in all.
    cheap_failed: usize,
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
    water_density: f64,
) -> Result<JourneyDeratingResult, PyError> {
    let mut ji = JourneyDeratingInputs::new(r, loa, beam, depth);
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
        inp.cargo_density_kg_m3 = self.o.cargo_density_kg_m3;
        inp.stern_block_mass = self.o.stern_block_mass.clone();
        inp.stern_geometry = self.o.stern_geometry.clone();
        inp.machinery = self.o.machinery.clone();
        inp.sightline_limit_m = self.o.sightline_limit_m;
        inp.block_coefficient = self.o.block_coefficient;
        inp.n_tanks_wide = Some(nw);
        inp.n_tanks_long = nl;
        inp.max_tank_od_m = self.o.max_tank_od_m;
        inp.bow_rake_to_depth_ratio = self.o.bow_rake_to_depth_ratio;
        inp.bow_rake_length_m = self.o.bow_rake_length_m;
        inp.bow_rake_rise_m = self.o.bow_rake_rise_m;
        inp.hull_shape = self.o.hull_shape;
        inp.pushed_stern_void_m = self.o.pushed_stern_void_m;
        inp.tank_support_height = self.o.tank_support_height_m;
        inp.dome_height = self.o.tank_dome_height_m;
        inp.dome_diameter = self.o.tank_dome_diameter_m;
        inp.dome_diameter_by_rule = self.o.tank_dome_diameter_by_rule;
        inp.dome_height_by_rule = self.o.tank_dome_height_by_rule;
        inp.wing_tank_gap_m = self.o.wing_tank_gap_m;
        inp.wing_tank_height_ratio = self.o.wing_tank_height_ratio;
        inp.weather_shield_thickness_mm = self.o.weather_shield_thickness_mm;
        inp.weather_shield_clearance_m = self.o.weather_shield_clearance_m;
        inp.tank_forming_allowance_pct = self.o.tank_forming_allowance_pct;
        inp.tank_dome_wall_thickness_mm = self.o.tank_dome_wall_thickness_mm;
        // Issue #280: end spaces of the Option's own, when they are not the engine's default.
        let default_space = crate::gas_separation::default_end_space(false);
        if self.o.aft_end_space_m != default_space || self.o.fwd_end_space_m != default_space {
            inp.separation = Some(crate::gas_separation::GasSeparationInputs {
                cofferdam_m: self.o.aft_end_space_m,
                fwd_end_bulkhead_m: self.o.fwd_end_space_m,
                ..Default::default()
            });
        }
        inp.tank_wall_thickness = self.o.tank_wall_thickness_mm;
        inp.tank_design = self.o.tank_design.clone();
        inp.water_density = self.o.water_density_t_per_m3;
        inp.air_draft_limit_m = self.o.air_draft_limit_m;
        inp.aft_design = self.o.aft_design.clone();
        // The user's freeboard, so the full-volume `freeboard_ok` the result shows uses it (#418).
        inp.required_freeboard = self.o.required_freeboard_m;
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
            // The n - 1 gaps between tanks side by side take beam too (issue #422), as the tank
            // layout counts them; without them a capped tank came out below the cap.
            let cs = (self.beam
                - nw as f64 * self.o.max_tank_od_m
                - (nw - 1) as f64 * DEFAULT_TANK_GAP_TRANSVERSE_M)
                / 2.0;
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
                total / tank_divisor(r.n_tanks_total) + {
                    let (d, h) = dome_dimensions(
                        opt_or_zero(r.tank_outer_diameter_m)
                            - 2.0 * opt_or_zero(r.insulation_thickness_mm) / 1000.0,
                        (!self.o.tank_dome_diameter_by_rule).then_some(self.o.tank_dome_diameter_m),
                        (!self.o.tank_dome_height_by_rule).then_some(self.o.tank_dome_height_m),
                    );
                    dome_volume_m3(d, h)
                },
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

/// Build every candidate and judge it by the whole check list. Every candidate gets the cheap
/// checks. The stability assessment, the costly check, runs only for the candidates that passed
/// the cheap ones, from the most cargo down, and stops at the first that passes it: a candidate
/// with less cargo cannot win, so its stability is not needed. Among candidates within 1e-6 t of
/// the winner the first built stays, as before. With `stop_at_first_pass` the search ends at the
/// first passing candidate (for a yes or no answer).
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
                let j = derate_candidate(&r, loa, beam, d, t_max, o.water_density_t_per_m3)?;
                let cheap = cheap_checks(&r, &j, d, o);
                // With Rule Sets: the trace before stability (its stability Rules are not yet
                // evaluated, so they cannot fail here).
                let cheap_failed = if o.rules.is_empty() {
                    cheap.iter().filter(|k| k.is_error() && !k.ok).count()
                } else {
                    failing_groups(&candidate_failures(
                        &candidate_measured(&r, &j, &inp, &cheap, Vec::new()),
                        o,
                    ))
                    .len()
                };
                pool.all.push(Candidate {
                    cheap_failed,
                    tonnage: ranked_cargo(&r, &j, &inp, o),
                    depth: d,
                    result: r,
                    journey: j,
                    inputs: inp,
                });
            }
        }
    }
    // The costly check, from the most cargo down (a stable sort keeps the build order of equals).
    let mut order: Vec<usize> = (0..pool.all.len())
        .filter(|&i| pool.all[i].cheap_failed == 0)
        .collect();
    order.sort_by(|&a, &b| pool.all[b].tonnage.total_cmp(&pool.all[a].tonnage));
    let mut top = 0.0;
    for i in order {
        let c = &pool.all[i];
        if pool.best.is_some() && c.tonnage < top - 1e-6 {
            break;
        }
        if passes_all(c, o) {
            match pool.best {
                None => {
                    top = c.tonnage;
                    pool.best = Some(i);
                    if stop_at_first_pass {
                        break;
                    }
                }
                Some(w) if i < w => pool.best = Some(i),
                Some(_) => {}
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
        // The cheap failures are a floor on all of them: skip the costly stage when even the floor
        // cannot beat the best so far. The answer is the one a full count of each gives.
        if let Some((bi, bf)) = best
            && (c.cheap_failed > bf
                || (c.cheap_failed == bf && c.tonnage <= pool[bi].tonnage + 1e-6))
        {
            continue;
        }
        let failed = if o.rules.is_empty() {
            search_checks(&c.result, &c.journey, &c.inputs, c.depth, o)
                .iter()
                .filter(|k| k.is_error() && !k.ok)
                .count()
        } else {
            failing_groups(&candidate_failures(&full_measured(c, o), o)).len()
        };
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
            let j = derate_candidate(&r, loa, beam, d, t_max, o.water_density_t_per_m3)?;
            Candidate {
                cheap_failed: 1,
                tonnage: 0.0,
                depth: d,
                result: r,
                journey: j,
                inputs: inp,
            }
        }
    };

    let checks = search_checks(&best.result, &best.journey, &best.inputs, best.depth, o);
    let mut failed: Vec<String> = checks
        .iter()
        .filter(|c| c.is_error() && !c.ok)
        .map(|c| format!("{}: {}", c.name, c.message))
        .collect();
    let feasible = if o.rules.is_empty() {
        all_ok(&checks)
    } else {
        // The failing rows no check of the list reports (a tank Rule, a Rule of a set loaded at
        // run time) join the failed checks, as "rule id: title (object)".
        let trace = candidate_trace(&full_measured(&best, o), o);
        for row in trace.rows.iter().filter(|r| r.outcome == Outcome::Fail) {
            let mirrored = row
                .measure
                .as_ref()
                .is_some_and(|m| m.id == "check.ok" || m.id.starts_with("separation."))
                || row.object_type.as_deref() == Some(ObjectType::LoadingCondition.key());
            let line = format!(
                "{}: {} ({})",
                row.rule.clone().unwrap_or_default(),
                row.title,
                row.object.clone().unwrap_or_default()
            );
            if !mirrored && !failed.contains(&line) {
                failed.push(line);
            }
        }
        trace.rows.iter().all(|r| r.outcome != Outcome::Fail)
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
    // The answer is on the 0.1 m grid, and a length is only returned once it has been tested
    // (issue #422): rounding to nearest could go below `hi`, where nothing was tested.
    let nearest = round_to(hi, 1);
    if nearest == hi || has_passing_arrangement(nearest, beam, cargo, t_max, o)? {
        return Ok(nearest);
    }
    let up = libm::ceil(hi * 10.0) / 10.0;
    if has_passing_arrangement(up, beam, cargo, t_max, o)? {
        return Ok(up);
    }
    Ok(hi)
}
