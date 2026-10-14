//! Unified barge design and volume-bound cargo capacity. Port of `core/barge_capacity.py`.
//!
//! Behaviour is copied exactly, including the errors and their message text. The reasoning
//! behind the design (what is derived, what is an estimate, why the light air draft governs)
//! lives in the Python file. Python's two-argument `max` and `min` keep their first-of-equal
//! rule (`py_max`, `py_min`), `x or 0.0` on a float is `truthy_or_zero`, every `**` is
//! `libm::pow` (inside the modules this one calls), and every expression keeps Python's
//! evaluation order so the rounding is the same.
//!
//! Only a `ValueError` out of the gas branch is caught: it becomes a failed `feasible_geometry`
//! check, as in Python. Any other exception, and every `ValueError` from anywhere else, comes
//! back as a [`PyError`]. A tank count that no 64-bit result field can hold is an
//! `OverflowError` with one fixed message, raised after the rest of the calculation succeeds
//! (the integer range rule: Python holds a count of any size, a 64-bit field cannot).

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use libm::{fabs, sqrt};

use crate::aft_arrangement::{
    AftArrangementInputs, AftArrangementResult, PyException, compute as compute_aft_arrangement,
};
use crate::cargo::get_cargo;
use crate::checks::{Check, all_ok};
use crate::gas_separation::{
    GasSeparationInputs, GasSeparationResult, compute as compute_gas_separation,
};
use crate::hull::HullGeometry;
use crate::hydrostatics::find_equilibrium_draft;
use crate::models::{BargeInputs, TankResult, WeightBreakdown};
use crate::pressure_vessel::PyError;
use crate::protective_location::{compute_required_clearances, d_vc_exact, distance_d_vc};
use crate::pyfmt::{fixed, py_str};
use crate::pystr::repr_str;
use crate::tanks::{INT_OVERFLOW_MESSAGE, calculate_tank_arrangement_wide};
use crate::weights::calculate_weights;

/// Maximum gas tank outer diameter (m): the design rule that caps tank OD and goes to more tanks
/// across above it.
pub const MAX_TANK_OD_M: f64 = 12.0;

/// Inputs for the unified barge capacity calculation. [`BargeCapacityInputs::new`] gives the
/// Python dataclass defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct BargeCapacityInputs {
    // --- Scenario fundamentals ---
    /// Length overall (m)
    pub loa: f64,
    /// Moulded beam (m)
    pub beam: f64,
    /// Moulded depth (m)
    pub depth: f64,
    /// Key into the cargo database (gas, or "bauxite")
    pub cargo_type: String,

    // --- Common assumptions ---
    pub block_coefficient: f64,
    /// t/m^3
    pub water_density: f64,
    /// Minimum freeboard (m)
    pub required_freeboard: f64,
    /// Bridge clearance limit (m), checked against the LIGHT condition.
    pub air_draft_limit_m: Option<f64>,
    /// Ballast the empty leg can carry (t)
    pub ballast_capacity_t: f64,

    // --- Arrangement ---
    pub aft: Option<AftArrangementInputs>,
    pub separation: Option<GasSeparationInputs>,
    /// A self-propelled vessel carries the stern block; a pushed barge does not.
    pub self_propelled: bool,
    /// Aft void of a pushed barge (m)
    pub pushed_stern_void_m: f64,
    /// Bow bottom rake (m). `None`: 1.6 x moulded depth.
    pub bow_rake_length_m: Option<f64>,
    /// Dry-bulk end-bulkhead allowance at each end of the hold (m)
    pub bulk_end_bulkhead_m: f64,

    // --- Gas branch (IGC/IGF) ---
    pub vessel_type: String,
    pub side_clearance_override_m: Option<f64>,
    /// `None`: auto, the fewest tanks across that keeps each tank's OD within the cap.
    pub n_tanks_wide: Option<i64>,
    pub n_tanks_long: i64,
    /// mm; `None`: auto pressure-vessel formula
    pub tank_wall_thickness: Option<f64>,
    pub fill_ratio: f64,
    pub dome_height: f64,
    pub tank_support_height: f64,

    // --- Bulk branch ---
    /// "sloped_hopper", "box" or "container"
    pub hold_type: String,
    pub lightship_k_t_per_m3: Option<f64>,
    pub wing_void_width: f64,
    pub double_bottom_height_bulk: f64,
    pub hopper_slope_h_per_v: f64,
    pub coaming_height: f64,
    pub n_holds: i64,
    pub hold_fill_ratio: f64,
    pub bow_void_fraction: f64,
    pub stern_void_fraction: f64,
    pub inner_bottom_t_mm: f64,
    pub hopper_plate_t_mm: f64,
    pub bulkhead_t_mm: f64,
    pub coaming_t_mm: f64,
    pub steel_density: f64,
    pub bulk_structure_factor: f64,
    pub bulk_steel_rate_per_cbm: f64,
    pub bulk_piping_factor: f64,
    pub bulk_outfitting_factor: f64,
}

impl BargeCapacityInputs {
    /// The dataclass defaults, with the three required principal dimensions.
    pub fn new(loa: f64, beam: f64, depth: f64) -> Self {
        Self {
            loa,
            beam,
            depth,
            cargo_type: "LNG_ethane".to_string(),
            block_coefficient: 0.85,
            water_density: 1.0,
            required_freeboard: 0.3,
            air_draft_limit_m: None,
            ballast_capacity_t: 0.0,
            aft: None,
            separation: None,
            self_propelled: true,
            pushed_stern_void_m: 3.0,
            bow_rake_length_m: None,
            bulk_end_bulkhead_m: 0.50,
            vessel_type: "IGC_2G".to_string(),
            side_clearance_override_m: None,
            n_tanks_wide: None,
            n_tanks_long: 2,
            tank_wall_thickness: Some(25.0),
            fill_ratio: 0.95,
            dome_height: 2.0,
            tank_support_height: 0.5,
            hold_type: "sloped_hopper".to_string(),
            lightship_k_t_per_m3: None,
            wing_void_width: 1.0,
            double_bottom_height_bulk: 1.2,
            hopper_slope_h_per_v: 1.0,
            coaming_height: 1.0,
            n_holds: 3,
            hold_fill_ratio: 0.95,
            bow_void_fraction: 0.10,
            stern_void_fraction: 0.15,
            inner_bottom_t_mm: 14.0,
            hopper_plate_t_mm: 14.0,
            bulkhead_t_mm: 11.0,
            coaming_t_mm: 10.0,
            steel_density: 7.85,
            bulk_structure_factor: 1.40,
            bulk_steel_rate_per_cbm: 0.10,
            bulk_piping_factor: 0.04,
            bulk_outfitting_factor: 0.035,
        }
    }
}

/// The converged gas arrangement and its weight split, for the gas stability check.
#[derive(Debug, Clone, PartialEq)]
pub struct GasBasis {
    pub inputs: BargeInputs,
    pub tanks: TankResult,
    pub weights: WeightBreakdown,
}

/// Volume-bound capacity and the resulting full-load hydrostatics.
#[derive(Debug, Clone, PartialEq)]
pub struct BargeCapacityResult {
    pub cargo_type: String,
    /// "liquid_gas" or "dry_bulk"
    pub phase: String,
    pub cargo_density_t_per_m3: f64,

    // Headline outputs
    pub cargo_capacity_t: f64,
    pub full_load_draft_m: f64,
    pub air_draft_m: f64,

    // Supporting
    pub cargo_volume_m3: f64,
    pub lightship_t: f64,
    pub displacement_t: f64,
    pub deadweight_t: f64,
    pub freeboard_m: f64,
    pub air_draft_top_above_keel_m: f64,
    pub overloaded: bool,

    // Gas design detail (`None` for bulk)
    pub tank_outer_diameter_m: Option<f64>,
    pub n_tanks_wide: Option<i64>,
    pub max_tank_od_m: Option<f64>,
    pub n_tanks_total: Option<i64>,
    pub side_clearance_m: Option<f64>,
    pub bottom_clearance_m: Option<f64>,

    // Bulk design detail (`None` for gas)
    pub hold_height_m: Option<f64>,
    pub hold_top_width_m: Option<f64>,
    pub hold_bottom_width_m: Option<f64>,
    pub hold_section_area_m2: Option<f64>,

    /// Transverse metacentric height (bulk hopper only)
    pub gm_m: Option<f64>,

    // --- Arrangement ---
    pub bow_void_length_m: Option<f64>,
    pub stern_void_length_m: Option<f64>,
    pub cargo_zone_length_m: Option<f64>,
    pub bow_rake_length_m: Option<f64>,
    pub aft_arrangement: Option<AftArrangementResult>,
    pub separation: Option<GasSeparationResult>,

    // --- Cargo volume build-up ---
    pub containment_inner_volume_m3: Option<f64>,
    pub fill_ratio_applied: Option<f64>,
    pub insulation_thickness_mm: Option<f64>,
    pub tank_wall_thickness_mm: Option<f64>,
    pub tank_inner_diameter_m: Option<f64>,

    // --- Light / ballast load conditions ---
    pub light_draft_m: Option<f64>,
    pub air_draft_light_m: Option<f64>,
    pub ballast_draft_m: Option<f64>,
    pub air_draft_ballast_m: Option<f64>,
    pub air_draft_navigating_light_m: Option<f64>,
    pub navigating_top_above_keel_m: Option<f64>,

    /// Gas only: the converged arrangement, for gas stability.
    pub gas_basis: Option<GasBasis>,

    pub checks: Vec<Check>,
    pub warnings: Vec<String>,
}

impl BargeCapacityResult {
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

/// Python `x or 0.0` on a float: zero is false, so it gives `0.0`; NaN is true.
fn truthy_or_zero(x: f64) -> f64 {
    if x == 0.0 { 0.0 } else { x }
}

fn value_error(message: String) -> PyError {
    PyError {
        kind: "ValueError",
        message,
    }
}

fn zero_division() -> PyError {
    PyError {
        kind: "ZeroDivisionError",
        message: "float division by zero".to_string(),
    }
}

fn overflow() -> PyError {
    PyError {
        kind: "OverflowError",
        message: INT_OVERFLOW_MESSAGE.to_string(),
    }
}

impl From<PyException> for PyError {
    fn from(e: PyException) -> Self {
        PyError {
            kind: e.type_name(),
            message: e.message().to_string(),
        }
    }
}

/// Fewest tanks across the beam that keeps each tank's OD within `max_od_m`.
///
/// Inverts the arrangement geometry in `tanks::calculate_tank_arrangement`:
/// `OD = (beam - 2*cs - (n-1)*gap) / n <= max_od`, so `n >= (beam - 2*cs + gap) / (max_od + gap)`.
/// Always at least 1, so a narrow hull is unaffected.
///
/// `ZeroDivisionError` when `max_od_m + gap` is exactly zero. `math.ceil` raises `ValueError` on
/// NaN and `OverflowError` on infinity. A count above the 64-bit range is the fixed
/// `OverflowError`; a ceiling below 1 is 1, however negative.
pub fn tanks_wide_for_beam(
    beam_m: f64,
    side_clearance_m: f64,
    tank_gap_transverse_m: f64,
    max_od_m: f64,
) -> Result<i64, PyError> {
    let usable = beam_m - 2.0 * side_clearance_m;
    if usable <= 0.0 {
        return Ok(1);
    }
    let divisor = max_od_m + tank_gap_transverse_m;
    if divisor == 0.0 {
        return Err(zero_division());
    }
    let ratio = (usable + tank_gap_transverse_m) / divisor;
    if ratio.is_nan() {
        return Err(value_error(
            "cannot convert float NaN to integer".to_string(),
        ));
    }
    if ratio.is_infinite() {
        return Err(PyError {
            kind: "OverflowError",
            message: "cannot convert float infinity to integer".to_string(),
        });
    }
    let n = libm::ceil(ratio);
    if n < 1.0 {
        return Ok(1);
    }
    // 2^63: the first float that no `i64` holds.
    if n >= 9_223_372_036_854_775_808.0 {
        return Err(overflow());
    }
    Ok(n as i64)
}

/// What the volume branch reports besides the four numbers it returns.
#[derive(Debug, Clone, Default)]
struct Detail {
    tank_outer_diameter_m: Option<f64>,
    n_tanks_wide: Option<i64>,
    max_tank_od_m: Option<f64>,
    n_tanks_total: Option<i64>,
    side_clearance_m: Option<f64>,
    bottom_clearance_m: Option<f64>,
    hold_height_m: Option<f64>,
    hold_top_width_m: Option<f64>,
    hold_bottom_width_m: Option<f64>,
    hold_section_area_m2: Option<f64>,
    containment_inner_volume_m3: Option<f64>,
    fill_ratio_applied: Option<f64>,
    insulation_thickness_mm: Option<f64>,
    tank_wall_thickness_mm: Option<f64>,
    tank_inner_diameter_m: Option<f64>,
    gas_basis: Option<GasBasis>,
}

/// (cargo volume, lightship, top above keel, detail) of a volume branch. `n_total` is the exact
/// tank count product of the gas branch, which no 64-bit field may hold.
struct Volumetric {
    cargo_volume: f64,
    lightship: f64,
    h_top: f64,
    detail: Detail,
    n_total: i128,
}

/// IGC/IGF protective-location clearances, then the tank arrangement, then cargo volume.
///
/// Resolves the volume and d(Vc) circular dependency by iterating.
fn gas_volumetric(
    inp: &BargeCapacityInputs,
    bow_void_m: f64,
    stern_void_m: f64,
) -> Result<Volumetric, PyError> {
    let mut base = BargeInputs::new(inp.loa, inp.beam, inp.depth, inp.depth, inp.depth * 10.0);
    base.vessel_type = inp.vessel_type.clone();
    base.cargo_type = inp.cargo_type.clone();
    // The cargo zone is bounded by the derived voids, not by a fraction of LOA.
    base.bow_non_tank_length = Some(bow_void_m);
    base.stern_non_tank_length = Some(stern_void_m);
    // `inp.n_tanks_wide or 1`
    base.n_tanks_wide = match inp.n_tanks_wide {
        Some(n) if n != 0 => n,
        _ => 1,
    };
    base.n_tanks_long = inp.n_tanks_long;
    base.tank_wall_thickness = inp.tank_wall_thickness;
    base.fill_ratio = inp.fill_ratio;
    base.dome_height = inp.dome_height;
    base.tank_support_height = inp.tank_support_height;
    base.post_init()?;

    let mut d_vc = 1.0;
    let mut bi = base.clone();
    let mut tanks: Option<(TankResult, i128)> = None;
    let mut cs = 0.0;
    let mut cb = 0.0;
    let mut db = 0.0;
    let exact = inp.vessel_type == "IGC_2G_EXACT";
    // The engine measures clearance to the OUTER face of the insulation (OD includes it).
    let ins_m = truthy_or_zero(base.insulation_thickness) / 1000.0;
    let rounds = if exact { 40 } else { 8 };
    for _ in 0..rounds {
        if exact {
            cs = py_max(d_vc - ins_m, 0.0);
            cb = py_max(py_min(inp.beam / 15.0, 2.0), d_vc) - ins_m;
        } else {
            let req = compute_required_clearances(&inp.vessel_type, inp.beam, d_vc)
                .map_err(value_error)?;
            cs = req.cs_required;
            cb = req.cb_required;
        }
        if let Some(over) = inp.side_clearance_override_m {
            cs = over;
        }
        db = py_max(0.0, cb - inp.tank_support_height);
        // AUTO tank arrangement: cs is only known inside this loop.
        let n_wide = match inp.n_tanks_wide {
            Some(n) => n,
            None => tanks_wide_for_beam(
                inp.beam,
                cs,
                truthy_or_zero(base.tank_gap_transverse),
                MAX_TANK_OD_M,
            )?,
        };
        // `dataclasses.replace`: copy, set, run `__post_init__` again.
        bi = base.clone();
        bi.side_clearance = Some(cs);
        bi.double_bottom_height = Some(db);
        bi.n_tanks_wide = n_wide;
        bi.post_init()?;
        let (t, n_total) = calculate_tank_arrangement_wide(&bi)?;
        let new_d_vc = if exact {
            // Per-tank gross volume, linear bands: a continuous map, so it converges.
            let count = if n_total > 1 { n_total } else { 1 };
            d_vc_exact(t.total_tank_volume / (count as f64))
        } else {
            distance_d_vc(t.total_tank_volume)
        };
        tanks = Some((t, n_total));
        if fabs(new_d_vc - d_vc) < 1e-9 {
            break;
        }
        d_vc = new_d_vc;
    }
    let (tanks, n_total) = tanks.expect("the loop runs at least eight times");

    let hull = HullGeometry::new(inp.loa, inp.beam, inp.depth, 0.0, 0.0, 0.0, 0.0);
    let weights = calculate_weights(&bi, &hull, &tanks)?;
    let lightship = weights.lightship;
    let h_top = db + inp.tank_support_height + tanks.outer_diameter + inp.dome_height;
    let detail = Detail {
        tank_outer_diameter_m: Some(tanks.outer_diameter),
        n_tanks_wide: Some(tanks.n_tanks_wide),
        max_tank_od_m: Some(MAX_TANK_OD_M),
        n_tanks_total: Some(tanks.n_tanks_total),
        side_clearance_m: Some(cs),
        bottom_clearance_m: Some(cb),
        containment_inner_volume_m3: Some(tanks.total_tank_volume),
        fill_ratio_applied: Some(inp.fill_ratio),
        insulation_thickness_mm: Some(bi.insulation_thickness),
        tank_wall_thickness_mm: Some(tanks.cylinder_wall_thickness),
        tank_inner_diameter_m: Some(tanks.inner_diameter),
        gas_basis: Some(GasBasis {
            inputs: bi,
            tanks: tanks.clone(),
            weights,
        }),
        ..Detail::default()
    };
    Ok(Volumetric {
        cargo_volume: tanks.cargo_volume,
        lightship,
        h_top,
        detail,
        n_total,
    })
}

/// Trapezoidal hopper hold geometry, then cargo volume and hopper structural weight.
fn bulk_volumetric(
    inp: &BargeCapacityInputs,
    bow_void_m: f64,
    stern_void_m: f64,
) -> Result<Volumetric, PyError> {
    let d_db = inp.double_bottom_height_bulk;
    let s = inp.hopper_slope_h_per_v;
    let h_hold = (inp.depth - d_db) + inp.coaming_height;
    let w_top = inp.beam - 2.0 * inp.wing_void_width;
    let w_bot = if inp.hold_type == "sloped_hopper" {
        py_max(1.0, w_top - 2.0 * s * h_hold)
    } else if inp.hold_type == "box" || inp.hold_type == "container" {
        w_top // vertical inner sides
    } else {
        return Err(value_error(format!(
            "unknown hold_type {}",
            repr_str(&inp.hold_type)
        )));
    };
    let area = 0.5 * (w_top + w_bot) * h_hold;
    let l_cargo = py_max(0.0, inp.loa - bow_void_m - stern_void_m);
    let hold_volume = area * l_cargo;
    let cargo_volume = hold_volume * inp.hold_fill_ratio;

    // --- Hopper structural steel (plating-area method) ---
    let t_ib = inp.inner_bottom_t_mm / 1000.0;
    let t_hop = inp.hopper_plate_t_mm / 1000.0;
    let t_bhd = inp.bulkhead_t_mm / 1000.0;
    let t_coam = inp.coaming_t_mm / 1000.0;
    let inner_bottom = l_cargo * inp.beam * t_ib;
    let hopper_slopes = 2.0 * (h_hold * sqrt(s * s + 1.0)) * l_cargo * t_hop;
    let wing_bulkheads = 2.0 * h_hold * l_cargo * t_bhd;
    // `(n_holds + 1)` is an exact integer that Python rounds once to a float.
    let holds_plus_one = (i128::from(inp.n_holds) + 1) as f64;
    let transverse_bhd = holds_plus_one * area * t_bhd;
    let coaming = 2.0 * inp.coaming_height * l_cargo * t_coam;
    let steel_vol = inner_bottom + hopper_slopes + wing_bulkheads + transverse_bhd + coaming;
    let hopper_steel = steel_vol * inp.steel_density * inp.bulk_structure_factor;

    let hull_steel_td = inp.bulk_steel_rate_per_cbm * inp.loa * inp.beam * inp.depth;
    let base_steel = hull_steel_td + hopper_steel;
    let mut lightship = base_steel * (1.0 + inp.bulk_piping_factor + inp.bulk_outfitting_factor);
    if inp.hold_type != "sloped_hopper" {
        let k = inp.lightship_k_t_per_m3.ok_or_else(|| {
            value_error(format!(
                "hold_type {} needs lightship_k_t_per_m3 (see calibration/factors.yaml)",
                repr_str(&inp.hold_type)
            ))
        })?;
        lightship = k * inp.loa * inp.beam * inp.depth;
    }

    let h_top = inp.depth + inp.coaming_height;
    let detail = Detail {
        hold_height_m: Some(h_hold),
        hold_top_width_m: Some(w_top),
        hold_bottom_width_m: Some(w_bot),
        hold_section_area_m2: Some(area),
        containment_inner_volume_m3: Some(hold_volume),
        fill_ratio_applied: Some(inp.hold_fill_ratio),
        ..Detail::default()
    };
    Ok(Volumetric {
        cargo_volume,
        lightship,
        h_top,
        detail,
        n_total: 0,
    })
}

/// First-order transverse intact GM for the bulk hopper at full load: `GM = KB + BM_t - KG`.
///
/// Solid bulk has no liquid free surface. KG combines lightship (at 0.5 x moulded depth) and
/// cargo (the centroid of the filled trapezoidal hold above the double bottom).
fn bulk_gm(
    inp: &BargeCapacityInputs,
    detail: &Detail,
    draft: f64,
    lightship: f64,
    cargo_t: f64,
    hull: &HullGeometry,
) -> f64 {
    // The bulk branch always sets these.
    let w_top = detail.hold_top_width_m.unwrap_or(f64::NAN);
    let w_bot = detail.hold_bottom_width_m.unwrap_or(f64::NAN);
    let h = detail.hold_height_m.unwrap_or(f64::NAN);
    // Trapezoid centroid above its base: h/3 * (2*top + bot)/(top + bot).
    let zc = if (w_top + w_bot) > 0.0 {
        (h / 3.0) * (2.0 * w_top + w_bot) / (w_top + w_bot)
    } else {
        h / 2.0
    };
    let kg_cargo = inp.double_bottom_height_bulk + zc;
    let kg_light = 0.5 * inp.depth;
    let total = lightship + cargo_t;
    let kg = if total > 0.0 {
        (lightship * kg_light + cargo_t * kg_cargo) / total
    } else {
        kg_light
    };
    hull.KB(draft) + hull.BM_transverse(draft) - kg
}

/// `replace(aft_in, sightline_obstruction_m=...)` and friends keep the other fields.
struct Arrangement {
    bow_void_m: f64,
    stern_void_m: f64,
    rake_m: f64,
    aft_inputs: AftArrangementInputs,
    separation: Option<GasSeparationResult>,
}

/// Resolve the two non-cargo end lengths from the equipment and the code, not a percentage.
///
/// The aft block's length does not depend on the cargo, so it is resolved here; only the
/// wheelhouse lift does, and that is a second pass once the cargo top is known.
fn arrangement(inp: &BargeCapacityInputs, is_gas: bool) -> Result<Arrangement, PyError> {
    // The mix of 40 ft units differs by cargo: dry bulk carries a second battery, gas a second
    // accommodation unit.
    let mut aft_in = match &inp.aft {
        Some(a) => a.clone(),
        None => {
            let mut a = AftArrangementInputs::new(inp.beam, inp.depth);
            a.n_battery_units = if is_gas { 1 } else { 2 };
            a.n_accom_units = if is_gas { 2 } else { 1 };
            a
        }
    };
    // Keep the block's beam and depth honest even if a caller passed an `aft` built elsewhere.
    aft_in.beam = inp.beam;
    aft_in.depth = inp.depth;
    let aft0 = compute_aft_arrangement(&aft_in)?;
    let block_len = if inp.self_propelled {
        aft0.length_m
    } else {
        inp.pushed_stern_void_m
    };

    let rake = match inp.bow_rake_length_m {
        Some(r) => r,
        None => 1.6 * inp.depth,
    };

    let (aft_sep, fwd_sep, sep) = if is_gas {
        let sep_in = inp.separation.clone().unwrap_or_default();
        let sep = compute_gas_separation(&sep_in).map_err(value_error)?;
        (sep.aft_separation_m, sep.fwd_separation_m, Some(sep))
    } else {
        (inp.bulk_end_bulkhead_m, inp.bulk_end_bulkhead_m, None)
    };

    Ok(Arrangement {
        bow_void_m: rake + fwd_sep,
        stern_void_m: block_len + aft_sep,
        rake_m: rake,
        aft_inputs: aft_in,
        separation: sep,
    })
}

const PUSHED_WARNING: &str = "PUSHED barge: crew, battery and propulsion are on the tug and are \
NOT in this hull's lightship, capacity or air draft. The tug is a separate vessel and must be \
costed and air-draft-gated separately.";

const BULK_GM_WARNING: &str = "Bulk GM is first-order (assumed lightship KG); \
cargo-shift/angle-of-repose not modelled \u{2014} confirm with a full intact-stability booklet.";

/// Run the unified barge capacity chain (volume-bound).
pub fn compute(inp: &BargeCapacityInputs) -> Result<BargeCapacityResult, PyError> {
    // Python builds the nested inputs, and so runs their `__post_init__` checks, before
    // `compute` starts: aft first, then separation, and before the cargo lookup.
    if let Some(a) = &inp.aft {
        a.validate().map_err(value_error)?;
    }
    if let Some(sep) = &inp.separation {
        sep.validate().map_err(value_error)?;
    }
    let props = get_cargo(&inp.cargo_type).map_err(value_error)?;
    let density_t = props.density / 1000.0;
    let is_gas = props.phase != "dry_bulk";

    let arr = arrangement(inp, is_gas)?;
    let (bow_void, stern_void, rake) = (arr.bow_void_m, arr.stern_void_m, arr.rake_m);
    let cargo_zone = inp.loa - bow_void - stern_void;

    let mut geometry_error: Option<String> = None;
    let vol = if props.phase == "dry_bulk" {
        bulk_volumetric(inp, bow_void, stern_void)?
    } else {
        match gas_volumetric(inp, bow_void, stern_void) {
            Ok(v) => v,
            // A vessel too short for its own stern block, or too narrow for its tanks, is
            // infeasible, not an exception: it comes back as a failed check.
            Err(e) if e.kind == "ValueError" => {
                geometry_error = Some(e.message);
                Volumetric {
                    cargo_volume: 0.0,
                    lightship: 0.0,
                    h_top: inp.depth,
                    detail: Detail {
                        max_tank_od_m: Some(MAX_TANK_OD_M),
                        containment_inner_volume_m3: Some(0.0),
                        fill_ratio_applied: Some(inp.fill_ratio),
                        ..Detail::default()
                    },
                    n_total: 0,
                }
            }
            Err(e) => return Err(e),
        }
    };
    let Volumetric {
        cargo_volume,
        lightship,
        mut h_top,
        detail,
        n_total,
    } = vol;

    // Second pass on the aft block: the wheelhouse has to see over the cargo, and the cargo top
    // is only known now. The length is unchanged by the lift.
    let cargo_top_above_keel = h_top;
    let mut aft_second = arr.aft_inputs.clone();
    aft_second.sightline_obstruction_m = Some(cargo_top_above_keel);
    let aft = compute_aft_arrangement(&aft_second)?;

    // The tallest fixed point of the vessel with the wheelhouse lowered for transit. A pushed
    // barge carries none of it: the block is on the tug.
    let (aft, h_top_navigating) = if inp.self_propelled {
        h_top = py_max(h_top, aft.block_top_lowered_m);
        let nav = py_max(h_top, aft.wheelhouse_top_raised_m);
        (Some(aft), nav)
    } else {
        (None, h_top)
    };

    let cargo_capacity = cargo_volume * density_t;

    // --- Full-load hydrostatics (volume-bound load) ---
    let hull = HullGeometry::new(inp.loa, inp.beam, inp.depth, 0.0, 0.0, 0.0, 0.0);
    let cb = inp.block_coefficient;
    let rho = inp.water_density;
    let total_weight = lightship + cargo_capacity;

    // Box hull displacement plateaus at moulded depth; a load that needs more is overloaded.
    let max_float_volume = hull.displaced_volume(inp.depth);
    let divisor = cb * rho;
    if divisor == 0.0 {
        return Err(zero_division());
    }
    let required_volume = total_weight / divisor;
    let overloaded = required_volume > max_float_volume;

    let draft = find_equilibrium_draft(&hull, total_weight, rho, inp.depth, cb, 0.001)?;
    let draft = py_min(draft, inp.depth); // clamp to the gunwale
    let displacement = cb * hull.displaced_volume(draft) * rho;
    let freeboard = inp.depth - draft;
    let air_draft = h_top - draft;

    // --- Light / ballast conditions ---
    let light_draft = find_equilibrium_draft(&hull, lightship, rho, inp.depth, cb, 0.001)?;
    let light_draft = py_min(light_draft, inp.depth);
    let air_draft_light = h_top - light_draft;

    let mut ballast_draft = light_draft;
    if inp.ballast_capacity_t > 0.0 {
        ballast_draft = find_equilibrium_draft(
            &hull,
            lightship + inp.ballast_capacity_t,
            rho,
            inp.depth,
            cb,
            0.001,
        )?;
        ballast_draft = py_min(ballast_draft, inp.depth);
    }
    let air_draft_ballast = h_top - ballast_draft;
    let air_draft_navigating_light = h_top_navigating - light_draft;

    let feasible_message = match &geometry_error {
        Some(m) if !m.is_empty() => m.clone(),
        _ => "cargo volume must be positive (tank/hold fits)".to_string(),
    };
    let mut checks = vec![
        Check::new("feasible_geometry", cargo_volume > 0.0, feasible_message),
        Check::new(
            "not_overloaded",
            !overloaded,
            "volume-bound load must float within the moulded depth",
        ),
        Check::new(
            "freeboard_ok",
            freeboard >= inp.required_freeboard,
            format!(
                "freeboard {} m >= required {} m",
                fixed(freeboard, 2),
                py_str(inp.required_freeboard)
            ),
        ),
        Check::new(
            "cargo_zone_positive",
            cargo_zone > 0.0,
            format!(
                "cargo zone {} m = LOA {} - bow void {} m - stern void {} m",
                fixed(cargo_zone, 2),
                fixed(inp.loa, 1),
                fixed(bow_void, 2),
                fixed(stern_void, 2)
            ),
        ),
    ];
    // Roll up the arrangement and gas-code checks so one `ok` covers the whole vessel.
    if let Some(a) = &aft {
        checks.extend(a.checks.iter().cloned());
    }
    let sep = match arr.separation {
        Some(_) => {
            // Re-run with the as-designed clearances so the type-G floors are checked against
            // real numbers rather than zeros.
            let mut sep_in = inp.separation.clone().unwrap_or_default();
            sep_in.side_clearance_m = truthy_or_zero(detail.side_clearance_m.unwrap_or(0.0));
            sep_in.double_bottom_m = truthy_or_zero(detail.bottom_clearance_m.unwrap_or(0.0));
            let sep = compute_gas_separation(&sep_in).map_err(value_error)?;
            checks.extend(sep.checks.iter().cloned());
            Some(sep)
        }
        None => None,
    };
    // Bridge clearance is gated on the LIGHT (or ballasted-light) condition, never the laden one.
    if let Some(limit) = inp.air_draft_limit_m {
        let ballasted = inp.ballast_capacity_t > 0.0;
        let governing = if ballasted {
            air_draft_ballast
        } else {
            air_draft_light
        };
        let label = if ballasted {
            "ballasted light"
        } else {
            "light"
        };
        checks.push(Check::new(
            "air_draft_within_limit",
            governing <= limit,
            format!(
                "{label} air draft {} m <= limit {} m (laden is {} m and is NOT the governing case)",
                fixed(governing, 2),
                fixed(limit, 2),
                fixed(air_draft, 2)
            ),
        ));
    }
    let mut warnings: Vec<String> = match &aft {
        Some(a) => a.warnings.clone(),
        None => vec![PUSHED_WARNING.to_string()],
    };
    if let Some(s) = &sep {
        warnings.extend(s.warnings.iter().cloned());
    }
    let mut gm = None;
    if props.phase == "dry_bulk" && draft > 0.0 && !overloaded {
        let g = bulk_gm(inp, &detail, draft, lightship, cargo_capacity, &hull);
        checks.push(Check::with_severity(
            "gm_positive",
            g >= 0.15,
            format!(
                "hopper intact GM {} m >= 0.15 m (first-order; KG_light @0.5D)",
                fixed(g, 2)
            ),
            "warning",
        ));
        warnings.push(BULK_GM_WARNING.to_string());
        gm = Some(g);
    }

    // A tank count that no 64-bit field can hold: after everything else has succeeded.
    if i64::try_from(n_total).is_err() {
        return Err(overflow());
    }

    Ok(BargeCapacityResult {
        cargo_type: props.name.to_string(),
        phase: props.phase.to_string(),
        cargo_density_t_per_m3: density_t,
        cargo_capacity_t: cargo_capacity,
        full_load_draft_m: draft,
        air_draft_m: air_draft,
        cargo_volume_m3: cargo_volume,
        lightship_t: lightship,
        displacement_t: displacement,
        deadweight_t: displacement - lightship,
        freeboard_m: freeboard,
        air_draft_top_above_keel_m: h_top,
        overloaded,
        tank_outer_diameter_m: detail.tank_outer_diameter_m,
        n_tanks_wide: detail.n_tanks_wide,
        max_tank_od_m: detail.max_tank_od_m,
        n_tanks_total: detail.n_tanks_total,
        side_clearance_m: detail.side_clearance_m,
        bottom_clearance_m: detail.bottom_clearance_m,
        hold_height_m: detail.hold_height_m,
        hold_top_width_m: detail.hold_top_width_m,
        hold_bottom_width_m: detail.hold_bottom_width_m,
        hold_section_area_m2: detail.hold_section_area_m2,
        gm_m: gm,
        bow_void_length_m: Some(bow_void),
        stern_void_length_m: Some(stern_void),
        cargo_zone_length_m: Some(cargo_zone),
        bow_rake_length_m: Some(rake),
        aft_arrangement: aft,
        separation: sep,
        containment_inner_volume_m3: detail.containment_inner_volume_m3,
        fill_ratio_applied: detail.fill_ratio_applied,
        insulation_thickness_mm: detail.insulation_thickness_mm,
        tank_wall_thickness_mm: detail.tank_wall_thickness_mm,
        tank_inner_diameter_m: detail.tank_inner_diameter_m,
        light_draft_m: Some(light_draft),
        air_draft_light_m: Some(air_draft_light),
        ballast_draft_m: Some(ballast_draft),
        air_draft_ballast_m: Some(air_draft_ballast),
        air_draft_navigating_light_m: Some(air_draft_navigating_light),
        navigating_top_above_keel_m: Some(h_top_navigating),
        gas_basis: detail.gas_basis,
        checks,
        warnings,
    })
}
