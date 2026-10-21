//! Drive the GA drawings from `barge_capacity`. Port of `core/ga_adapter.py`.
//!
//! Converts geometry only: nothing here recomputes a capacity, a weight or a draft. Behaviour is
//! copied exactly, including the errors and their message text. Python's `x or default` on a
//! float or an `Optional[float]` is `truthy_or` (a zero, and `None`, take the default; NaN
//! does not), `max(0.1, x)` is `py_max` (the first of two equal items; a NaN `x` loses), every
//! `**` is `libm::pow` with Python's `OverflowError` when it leaves the finite range, and every
//! expression keeps Python's evaluation order so the rounding is the same.
//!
//! Python's drawing structures carry `None` where the Rust model structs hold a plain value (a
//! bulk result has no tank count across, and a drawing carries no weights, stability or trim
//! result), so [`GaInputs`] and [`GaResults`] hold what Python carries instead of reusing
//! [`BargeResults`](crate::models::BargeResults).

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::barge_capacity::{BargeCapacityInputs, BargeCapacityResult};
use crate::models::{BargeInputs, StabilityResult, TankResult, WeightBreakdown};
use crate::pressure_vessel::PyError;
use crate::protective_location::ProtectiveLocationResult;
use crate::text::Text;

/// `BargeInputs` as the drawing carries it.
///
/// `inputs.n_tanks_wide` is the tank model's own `i64`. Python's drawing inputs take the capacity
/// result's `Optional[int]` as it is, so a bulk result carries `None`. [`n_tanks_wide`] holds
/// what Python carries; `inputs.n_tanks_wide` is 0 when that is `None`, and nothing reads it.
///
/// [`n_tanks_wide`]: GaInputs::n_tanks_wide
#[derive(Debug, Clone, PartialEq)]
pub struct GaInputs {
    pub inputs: BargeInputs,
    pub n_tanks_wide: Option<i64>,
}

/// `BargeResults` as the drawing carries it: Python's `None` where a drawing has no value.
#[derive(Debug, Clone, PartialEq)]
pub struct GaResults {
    pub inputs: GaInputs,
    /// `None` for a dry-bulk result.
    pub tanks: Option<TankResult>,
    pub cargo_type: String,
    /// kg/m^3 (the field name is Python's)
    pub cargo_density: f64,
    pub cargo_mass: f64,
    /// Always `None`: a drawing carries no weight breakdown.
    pub weights: Option<WeightBreakdown>,
    pub displacement: f64,
    pub deadweight: f64,
    pub draft: f64,
    pub freeboard: f64,
    pub min_freeboard_ok: bool,
    pub air_draft: f64,
    pub air_draft_ok: bool,
    /// Always `None`.
    pub stability: Option<StabilityResult>,
    pub stability_ok: bool,
    /// Python passes the float `0.0` here, not a trim result.
    pub trim: f64,
    pub total_depth_required: f64,
    pub air_draft_unladen: f64,
    pub light_draft: f64,
    pub air_draft_navigating_unladen: f64,
    /// Always `None`.
    pub protective_location: Option<ProtectiveLocationResult>,
    pub binding_constraints: Vec<String>,
    pub warnings: Vec<Text>,
}

/// Python `x or default` for a float: a zero (and so a `-0.0`) takes the default.
fn truthy_or(x: f64, default: f64) -> f64 {
    if x == 0.0 { default } else { x }
}

/// Python `x or default` for an `Optional[float]`: `None` and zero take the default.
fn opt_truthy_or(x: Option<f64>, default: f64) -> f64 {
    truthy_or(x.unwrap_or(0.0), default)
}

/// Python `max(a, b)`: the first of two equal items; `b` only if `b > a`.
fn py_max(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// Float `x ** y`: a finite base with a finite exponent that gives an infinity is Python's
/// `OverflowError`.
fn py_pow(x: f64, y: f64) -> Result<f64, PyError> {
    let r = libm::pow(x, y);
    if r.is_infinite() && x.is_finite() && y.is_finite() {
        Err(PyError {
            kind: "OverflowError",
            message: "(34, 'Numerical result out of range')".to_string(),
        })
    } else {
        Ok(r)
    }
}

/// Whether the capacity result is a liquefied-gas one.
pub fn is_gas(res: &BargeCapacityResult) -> bool {
    res.phase == crate::cargo::CargoClass::Gas
}

fn type_error(message: &str) -> PyError {
    PyError {
        kind: "TypeError",
        message: message.to_string(),
    }
}

/// A `TankResult` carrying `barge_capacity`'s tank geometry, for the drawing to read.
fn tank_result(
    inp: &BargeCapacityInputs,
    res: &BargeCapacityResult,
) -> Result<TankResult, PyError> {
    let od = res.tank_outer_diameter_m.ok_or_else(|| PyError {
        kind: "ValueError",
        message:
            "no tank geometry: this is a dry-bulk result, use a hopper view rather than the tank GA"
                .to_string(),
    })?;
    // The cargo space is what is left after INSULATION and then the steel shell come off the
    // outer diameter. Read the inner diameter the capacity model already reported; fall back to
    // the build-up only if it is absent.
    let wall = opt_truthy_or(inp.tank_wall_thickness, 25.0) / 1000.0;
    let insul = res.insulation_thickness_mm.unwrap_or(350.0) / 1000.0;
    let idia = match res.tank_inner_diameter_m {
        Some(d) => d,
        None => od - 2.0 * insul - 2.0 * wall,
    };
    let idia = py_max(0.1, idia);
    // The cylindrical length is DETERMINED by the volume the capacity model computed. Python
    // runs `a / b` before it evaluates `max(...)`, so the zero divisor wins over a missing count.
    if inp.fill_ratio == 0.0 {
        return Err(PyError {
            kind: "ZeroDivisionError",
            message: "float division by zero".to_string(),
        });
    }
    let n_total = res
        .n_tanks_total
        .ok_or_else(|| type_error("'>' not supported between instances of 'int' and 'NoneType'"))?;
    // `max(n, 1)` on an int, then the int is turned into a float for the division.
    let n_for_volume = if n_total < 1 { 1 } else { n_total };
    let v_tank = res.cargo_volume_m3 / inp.fill_ratio / (n_for_volume as f64);
    let half = idia / 2.0;
    let area = core::f64::consts::PI * py_pow(half, 2.0)?;
    let cyl = v_tank / area;
    // Hemispherical heads.
    let head = idia / 2.0;
    let n_wide = res.n_tanks_wide.ok_or_else(|| {
        type_error("a gas result always carries a tank count across (barge_capacity)")
    })?;
    let volume_heads = (4.0 / 3.0) * core::f64::consts::PI * py_pow(half, 3.0)?;
    let bottom = res
        .bottom_clearance_m
        .ok_or_else(|| type_error("unsupported operand type(s) for +: 'NoneType' and 'float'"))?;
    Ok(TankResult {
        n_tanks_wide: n_wide,
        n_tanks_long: inp.n_tanks_long,
        n_tanks_total: n_total,
        outer_diameter: od,
        inner_diameter: idia,
        overall_tank_length: cyl + 2.0 * head,
        cylinder_length: cyl,
        head_length: head,
        volume_per_tank: v_tank,
        volume_cylinder: area * cyl,
        volume_heads,
        // `v_tank * res.n_tanks_total`: the int is turned into a float for the product.
        total_tank_volume: v_tank * (n_total as f64),
        cargo_volume: res.cargo_volume_m3,
        // Not drawn; never invent one.
        tank_shell_weight: f64::NAN,
        insulation_weight: f64::NAN,
        tank_top_above_keel: bottom + od,
        cylinder_wall_thickness: opt_truthy_or(inp.tank_wall_thickness, 25.0),
        head_wall_thickness: 0.0,
        design_pressure: 0.0,
        tank_material: String::new(),
    })
}

/// The drawing inputs and results for `svg_drawings`, built from a capacity result.
///
/// `laden_draft_m` overrides the full-load draft with the channel-limited one from
/// `journey_derating`. `motor_vessel` true sets `tug_length` to 0, which the drawing reads as
/// self-propelled.
pub fn to_ga(
    inp: &BargeCapacityInputs,
    res: &BargeCapacityResult,
    laden_draft_m: Option<f64>,
    motor_vessel: bool,
) -> Result<(GaInputs, GaResults), PyError> {
    let draft = laden_draft_m.unwrap_or(res.full_load_draft_m);
    // 0 = no bridge clearance passed.
    let mut gi = BargeInputs::new(inp.loa, inp.beam, inp.depth, draft, 0.0);
    gi.vessel_type = inp.vessel_type.clone();
    gi.cargo_type = inp.cargo_type.clone();
    gi.fill_ratio = inp.fill_ratio;
    // DERIVED voids from the capacity result.
    gi.bow_non_tank_length = res.bow_void_length_m;
    gi.stern_non_tank_length = res.stern_void_length_m;
    gi.bow_rake_length = opt_truthy_or(res.bow_rake_length_m, 0.0);
    gi.aft_arrangement = res.aft_arrangement.clone();
    gi.separation = res.separation.clone();
    gi.n_tanks_wide = res.n_tanks_wide.unwrap_or(0);
    gi.n_tanks_long = inp.n_tanks_long;
    gi.tank_wall_thickness = Some(opt_truthy_or(inp.tank_wall_thickness, 25.0));
    gi.insulation_thickness = res.insulation_thickness_mm.unwrap_or(350.0);
    // The dome in force: by the dome rule (issue #316) it is the converged tank's.
    gi.dome_height = res
        .gas_basis
        .as_ref()
        .map_or(inp.dome_height, |b| b.inputs.dome_height);
    gi.tank_support_height = inp.tank_support_height;
    gi.side_clearance = res.side_clearance_m;
    gi.double_bottom_height = Some(opt_truthy_or(
        res.bottom_clearance_m,
        inp.double_bottom_height_bulk,
    ));
    gi.block_coefficient = inp.block_coefficient;
    gi.water_density = inp.water_density;
    gi.tug_length = if motor_vessel { 0.0 } else { 25.0 };
    gi.post_init()?;
    let gi = GaInputs {
        inputs: gi,
        n_tanks_wide: res.n_tanks_wide,
    };
    let tanks = if is_gas(res) {
        Some(tank_result(inp, res)?)
    } else {
        None
    };
    let gr = GaResults {
        inputs: gi.clone(),
        tanks,
        cargo_type: res.cargo_type.clone(),
        cargo_density: res.cargo_density_t_per_m3 * 1000.0,
        cargo_mass: res.cargo_capacity_t,
        weights: None,
        displacement: res.displacement_t,
        deadweight: res.deadweight_t,
        draft,
        freeboard: inp.depth - draft,
        min_freeboard_ok: true,
        air_draft: res.air_draft_m,
        air_draft_ok: true,
        stability: None,
        stability_ok: true,
        trim: 0.0,
        total_depth_required: inp.depth,
        air_draft_unladen: opt_truthy_or(res.air_draft_light_m, 0.0),
        light_draft: opt_truthy_or(res.light_draft_m, 0.0),
        air_draft_navigating_unladen: opt_truthy_or(res.air_draft_navigating_light_m, 0.0),
        protective_location: None,
        binding_constraints: Vec::new(),
        warnings: res.warnings.clone(),
    };
    Ok((gi, gr))
}
