//! IMO Type C horizontal cylindrical tank geometry and arrangement. Port of `core/tanks.py`.
//!
//! Behaviour is copied exactly, including the errors and their message text. Every `**` is
//! `libm::pow`, every float in a message prints as Python prints it, and every expression keeps
//! Python's evaluation order so the rounding is the same. Tank counts are `i64`. A count that
//! only feeds a float product (`n_wide - 1`, `n_long - 1`, `n_wide * n_long`) is held in `i128`
//! and rounded once to a float, as Python's int-to-float conversion does. Only a count that a
//! 64-bit result field must carry can overflow: that is an `OverflowError` with one fixed message,
//! raised after the rest of the calculation succeeds (the integer range rule:
//! Python holds a count of any size, a 64-bit field cannot).

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::f64::consts::PI;

use libm::{fabs, log, pow, sqrt};

use crate::models::{BargeInputs, TankResult};
use crate::pressure_vessel::{PyError, calculate_wall_thicknesses_with};
use crate::pyfmt::{fixed, py_str};

/// The one message for a tank count product that no 64-bit field can hold.
/// The Filling Limit by default: 98 % of a cargo tank's volume (IGC Code 15.3, as `simulate` and
/// GLOSSARY.md use it). A cargo with a typed loading-limit basis is loaded to its Loading Limit.
pub const DEFAULT_FILLING_LIMIT: f64 = 0.98;

pub const INT_OVERFLOW_MESSAGE: &str = "integer does not fit in 64 bits";

fn value_error(message: String) -> PyError {
    PyError {
        kind: "ValueError",
        message,
    }
}

fn overflow() -> PyError {
    PyError {
        kind: "OverflowError",
        message: INT_OVERFLOW_MESSAGE.to_string(),
    }
}

/// Python float `/` by an `int` count: an exactly-zero divisor raises `ZeroDivisionError`.
fn div_count(a: f64, n: i64) -> Result<f64, PyError> {
    if n == 0 {
        return Err(PyError {
            kind: "ZeroDivisionError",
            message: "float division by zero".to_string(),
        });
    }
    Ok(a / (n as f64))
}

/// `float - None`: the `TypeError` Python raises when a field `post_init` fills is still `None`.
pub(crate) fn none_operand_error() -> PyError {
    PyError {
        kind: "TypeError",
        message: "unsupported operand type(s) for -: 'float' and 'NoneType'".to_string(),
    }
}

/// `str(x)` of an `Optional[float]`: `None` prints as `None`.
fn opt_str(x: Option<f64>) -> String {
    match x {
        Some(v) => py_str(v),
        None => "None".to_string(),
    }
}

/// The surface area of a 2:1 semi-ellipsoidal head of radius `radius` (m^2), straight flange left
/// out (issue #314): half an oblate spheroid with semi-axes a = R and c = R/2. The whole spheroid
/// has S = 2 pi a^2 + (pi c^2 / e) ln((1 + e) / (1 - e)), e = sqrt(1 - c^2 / a^2) (MathWorld,
/// "Oblate Spheroid", eq. 11), so the head is 1.380173 pi R^2 = 4.335941 R^2 = 1.083985 D^2. The
/// original engine took 1.084 pi R^2, the 1.084 D^2 rule with pi R^2 in place of D^2: a factor of
/// pi/4, 21.5 % low. The caller picks the surface: the steel outer radius for the steel, the tank
/// outer radius (outside of the insulation) for the insulation.
pub fn ellipsoidal_2to1_head_area(radius: f64) -> f64 {
    let a = radius;
    let c = radius / 2.0;
    let e = sqrt(1.0 - pow(c, 2.0) / pow(a, 2.0));
    (2.0 * PI * pow(a, 2.0) + PI * pow(c, 2.0) / e * log((1.0 + e) / (1.0 - e))) / 2.0
}

/// The design liquid pressure of IGC 4.28.1.2 (MPa): Pgd = a x Z x rho / 1.02e5, with a the
/// dimensionless acceleration, Z the largest liquid height (m) and rho the cargo density (kg/m^3).
/// Issue #374: Z is the diameter the wall formula takes (the steel outer diameter), the full height
/// of a horizontal cylinder, a little over the inner diameter (on the safe side by twice the wall).
/// `None`: no liquid pressure, as the original engine did. IGC 4.23.2.2 and 4.13.2.4 add it to the
/// vapour pressure: Peq = Po + Pgd.
pub fn liquid_pressure_mpa(accel: Option<f64>, z_m: f64, density_kg_m3: f64) -> f64 {
    match accel {
        Some(a) => a * z_m * density_kg_m3 / 1.02e5,
        None => 0.0,
    }
}

/// Calculate tank dimensions, volumes, and weights for the given barge inputs.
///
/// 1. Available beam = beam - 2 * side_clearance
/// 2. Total width for tanks = available_beam - (n_wide - 1) * gap_transverse
/// 3. Outer diameter = total_width / n_wide
/// 4. Available length = LOA - non_tank_length
/// 5. Total length for tanks = available_length - (n_long - 1) * gap_longitudinal
/// 6. Length per tank = total_length / n_long
/// 7. Cylinder length = tank_length - 2 * head_length
pub fn calculate_tank_arrangement(inputs: &BargeInputs) -> Result<TankResult, PyError> {
    let (mut result, n_total) = calculate_tank_arrangement_wide(inputs)?;
    result.n_tanks_total = i64::try_from(n_total).map_err(|_| overflow())?;
    Ok(result)
}

/// [`calculate_tank_arrangement`] without the 64-bit limit on the tank count product.
///
/// Returns the exact `n_wide * n_long` beside the result. `TankResult::n_tanks_total` holds that
/// product clamped to the `i64` range, so read the second value when it can matter. Callers that
/// only need the geometry and weights (the weights chain) use this, because Python answers there
/// without ever carrying the product.
pub fn calculate_tank_arrangement_wide(
    inputs: &BargeInputs,
) -> Result<(TankResult, i128), PyError> {
    // Guard None values (auto-optimization may not have run yet).
    let side_clearance = inputs.side_clearance.unwrap_or(0.0);
    let double_bottom_height = inputs.double_bottom_height.unwrap_or(0.0);

    let n_wide = inputs.n_tanks_wide;
    let n_long = inputs.n_tanks_long;
    let n_total = i128::from(n_wide) * i128::from(n_long);

    // --- Transverse (beam) sizing ---
    let available_beam = inputs.beam - 2.0 * side_clearance;
    let wide_less_one = i128::from(n_wide) - 1;
    let tank_width_total = available_beam - (wide_less_one as f64) * inputs.tank_gap_transverse;
    let outer_diameter = div_count(tank_width_total, n_wide)?;

    if outer_diameter <= 0.0 {
        return Err(value_error(format!(
            "Tank outer diameter is {}m - not enough beam. Beam={}m, side_clearance={}m, \
             n_wide={}, gap_transverse={}m",
            fixed(outer_diameter, 2),
            py_str(inputs.beam),
            opt_str(inputs.side_clearance),
            n_wide,
            py_str(inputs.tank_gap_transverse),
        )));
    }

    let outer_radius = outer_diameter / 2.0;

    // Issue #577: the design pressure is the IGC 4.23.1.2 minimum of this tank: Po = 0.2 + A C
    // (rho_r)^1.5, with C from the outer diameter and the overall tank length (neither depends on
    // the wall, so there is nothing to iterate) and sigma_m the allowable stress of the wall formula.
    let design_pressure = if inputs.design_pressure_igc_minimum {
        let bow = inputs.bow_non_tank_length.ok_or_else(none_operand_error)?;
        let stern = inputs
            .stern_non_tank_length
            .ok_or_else(none_operand_error)?;
        let tank_length_total = inputs.loa
            - bow
            - stern
            - ((i128::from(inputs.n_tanks_long) - 1) as f64) * inputs.tank_gap_longitudinal;
        let overall = div_count(tank_length_total, inputs.n_tanks_long)?;
        let sigma = match inputs.tank_allowable_stress {
            Some(f) => f,
            None => {
                crate::pressure_vessel::get_tank_material(&inputs.tank_material)?.allowable_stress
            }
        };
        0.2 + crate::pressure_vessel::igc_dynamic_pressure_term_mpa(
            sigma,
            outer_diameter,
            overall,
            inputs.liquid_pressure_density / 1000.0,
        )
    } else {
        inputs.design_pressure
    };

    // --- Wall thickness determination ---
    let (wall_cyl_mm, wall_head_mm) = match inputs.tank_wall_thickness {
        // User-provided fixed thickness.
        Some(t) => (t, t),
        // Auto-calculate from the pressure vessel formula.
        // The formula is for the shell, so it wants the steel outer diameter: the diameter without
        // the insulation (issue #292). The original engine gave it the insulated diameter, and the
        // Python API still does (`wall_formula_on_steel_diameter` false).
        None => {
            let formula_diameter = if inputs.wall_formula_on_steel_diameter {
                outer_diameter - 2.0 * inputs.insulation_thickness / 1000.0
            } else {
                outer_diameter
            };
            calculate_wall_thicknesses_with(
                formula_diameter,
                design_pressure
                    + liquid_pressure_mpa(
                        inputs.liquid_pressure_accel,
                        formula_diameter,
                        inputs.liquid_pressure_density,
                    ),
                &inputs.tank_material,
                &inputs.tank_head_type,
                inputs.weld_efficiency,
                inputs.tank_allowable_stress,
            )?
        }
    };

    let wall_cyl_m = wall_cyl_mm / 1000.0;
    let wall_head_m = wall_head_mm / 1000.0;
    let insulation_m = inputs.insulation_thickness / 1000.0;

    // Inner geometry: OD -> insulation -> steel shell -> cargo space.
    let shell_outer_diameter = outer_diameter - 2.0 * insulation_m;
    let shell_outer_radius = shell_outer_diameter / 2.0;
    let inner_diameter = shell_outer_diameter - 2.0 * wall_cyl_m;
    let inner_radius = inner_diameter / 2.0;
    let head_inner_radius = shell_outer_radius - wall_head_m;

    if inner_radius <= 0.0 {
        return Err(value_error(
            "Tank inner radius is non-positive. Wall too thick for diameter.".to_string(),
        ));
    }
    if head_inner_radius <= 0.0 {
        return Err(value_error(
            "Head inner radius is non-positive. Wall too thick for diameter.".to_string(),
        ));
    }

    // --- Head geometry ---
    // Head length is the outer envelope including insulation: the steel head depth plus the
    // insulation over its crown.
    let hemisphere = inputs.tank_head_type == "hemisphere";
    let (head_length_outer, volume_one_head_inner, sa_one_head_shell);
    if hemisphere {
        head_length_outer = outer_radius;
        volume_one_head_inner = (2.0 / 3.0) * PI * pow(head_inner_radius, 3.0);
        sa_one_head_shell = 2.0 * PI * pow(shell_outer_radius, 2.0);
    } else if inputs.tank_head_type == "ellipsoidal_2to1" {
        // Issue #411: the steel head is R_s/2 deep (R_s the shell outer radius) and the insulation
        // adds its full thickness at the crown, so the head is R_s/2 + t_ins long, not
        // (R_s + t_ins)/2. A hemisphere's R_s + t_ins is the outer radius, as above.
        head_length_outer = shell_outer_radius / 2.0 + insulation_m;
        volume_one_head_inner = (1.0 / 3.0) * PI * pow(head_inner_radius, 3.0);
        sa_one_head_shell = ellipsoidal_2to1_head_area(shell_outer_radius);
    } else {
        return Err(value_error(format!(
            "Unknown tank head type: {}",
            inputs.tank_head_type
        )));
    }

    // --- Longitudinal sizing ---
    let bow = inputs.bow_non_tank_length.ok_or_else(none_operand_error)?;
    let stern = inputs
        .stern_non_tank_length
        .ok_or_else(none_operand_error)?;
    let available_length = inputs.loa - bow - stern;
    let long_less_one = i128::from(n_long) - 1;
    let tank_length_total =
        available_length - (long_less_one as f64) * inputs.tank_gap_longitudinal;
    let overall_tank_length = div_count(tank_length_total, n_long)?;

    let cylinder_length = overall_tank_length - 2.0 * head_length_outer;

    if cylinder_length <= 0.0 {
        return Err(value_error(format!(
            "Cylinder length is {}m - tank heads don't fit. Available length per tank={}m, \
             head length={}m each",
            fixed(cylinder_length, 2),
            fixed(overall_tank_length, 2),
            fixed(head_length_outer, 2),
        )));
    }

    // --- Interior volumes ---
    let volume_cylinder = PI * pow(inner_radius, 2.0) * cylinder_length;
    let volume_heads = 2.0 * volume_one_head_inner;

    let volume_per_tank = volume_cylinder + volume_heads;
    let n_total_f = n_total as f64;
    let total_tank_volume = n_total_f * volume_per_tank;
    let cargo_volume = total_tank_volume * inputs.fill_ratio;

    // --- Tank shell weight (steel) ---
    let cylinder_sa_shell_outer = 2.0 * PI * shell_outer_radius * cylinder_length;
    let cylinder_sa_inner = 2.0 * PI * inner_radius * cylinder_length;
    let cylinder_shell_volume = (cylinder_sa_shell_outer + cylinder_sa_inner) / 2.0 * wall_cyl_m;

    // Head shells (2 per tank, using head wall thickness).
    let head_shell_volume = 2.0 * sa_one_head_shell * wall_head_m;

    let shell_volume_per_tank = cylinder_shell_volume + head_shell_volume;
    let total_shell_weight = n_total_f * shell_volume_per_tank * inputs.tank_material_density;

    // --- Insulation weight ---
    let cylinder_sa_od = 2.0 * PI * outer_radius * cylinder_length;
    let sa_one_head_od = if hemisphere {
        2.0 * PI * pow(outer_radius, 2.0)
    } else {
        ellipsoidal_2to1_head_area(outer_radius)
    };
    let total_od_sa_per_tank = cylinder_sa_od + 2.0 * sa_one_head_od;
    let insulation_volume_per_tank = total_od_sa_per_tank * insulation_m;
    let insulation_density_t = inputs.insulation_density / 1000.0; // kg/m^3 to t/m^3
    let total_insulation_weight = n_total_f * insulation_volume_per_tank * insulation_density_t;

    // --- Tank top height above keel ---
    let tank_top_above_keel = double_bottom_height + inputs.tank_support_height + outer_diameter;

    let result = TankResult {
        n_tanks_wide: n_wide,
        n_tanks_long: n_long,
        n_tanks_total: n_total.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64,
        outer_diameter,
        inner_diameter,
        overall_tank_length,
        cylinder_length,
        head_length: head_length_outer,
        volume_per_tank,
        volume_cylinder,
        volume_heads,
        total_tank_volume,
        cargo_volume,
        tank_shell_weight: total_shell_weight,
        insulation_weight: total_insulation_weight,
        tank_top_above_keel,
        cylinder_wall_thickness: wall_cyl_mm,
        head_wall_thickness: wall_head_mm,
        design_pressure,
        tank_material: if inputs.tank_wall_thickness.is_none() {
            inputs.tank_material.clone()
        } else {
            String::new()
        },
    };
    Ok((result, n_total))
}

/// Transverse free surface moment for a horizontal cylindrical tank: `cargo_density * I_fs`.
///
/// The free surface is a rectangle as wide as the chord at the fill level and as long as the
/// cylinder, so `I_fs = (1/12) * L * chord^3`. The caller divides by displacement.
///
/// `fill_ratio` is a volume ratio. Issue #413: the liquid height is the height of the circular
/// segment that holds that share of the section (`gas_stability::segment_fill`, as the gas
/// stability takes it), not `fill_ratio * 2r`, which read the volume ratio as a height ratio and
/// gave about a quarter of the true moment at 0.98 fill. NaN when the segment cannot be formed
/// (a radius whose square overflows).
pub fn free_surface_moment_cylindrical(
    inner_radius: f64,
    cylinder_length: f64,
    fill_ratio: f64,
    cargo_density: f64,
) -> f64 {
    let r = inner_radius;
    // Fill height from bottom of cylinder: the segment that holds `fill_ratio` of the section.
    let h = match crate::gas_stability::segment_fill(r, fill_ratio) {
        Ok((h, _)) => h,
        Err(_) => return f64::NAN,
    };

    // Distance from centre to liquid surface (negative if more than half full).
    let d = r - h;

    // Empty or completely full: no free surface.
    if fabs(d) >= r {
        return 0.0;
    }
    let half_chord = sqrt(pow(r, 2.0) - pow(d, 2.0));
    let chord = 2.0 * half_chord;

    let i_fs = (1.0 / 12.0) * cylinder_length * pow(chord, 3.0);

    cargo_density * i_fs
}

// ---- the Tank Layout (spec 0009, ADR 0021) ---------------------------------------------------------

/// The cargo zone along the hull: from the stern void forward to the bow void (m from the aft end).
/// The tanks and the cargo lie in it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CargoZone {
    pub x_aft_m: f64,
    pub x_fwd_m: f64,
}

impl CargoZone {
    pub fn new(loa: f64, bow_void_m: f64, stern_void_m: f64) -> Self {
        Self {
            x_aft_m: stern_void_m,
            x_fwd_m: loa - bow_void_m,
        }
    }

    /// `loa - bow void - stern void`.
    pub fn length(&self) -> f64 {
        self.x_fwd_m - self.x_aft_m
    }

    /// The middle of the zone: the LCG of the tanks and of the cargo (`gas_stability`,
    /// `loading_conditions`).
    pub fn centre(&self) -> f64 {
        (self.x_aft_m + self.x_fwd_m) / 2.0
    }

    /// The overall length of one of `n_long` tanks in a line with `gap_m` between them: the length
    /// `calculate_tank_arrangement` gives a tank.
    pub fn tank_length(&self, n_long: i64, gap_m: f64) -> f64 {
        (self.length() - (n_long - 1) as f64 * gap_m) / n_long as f64
    }
}

/// The tank bottom above the keel: the double bottom plus the support height.
pub fn tank_bottom(double_bottom_m: f64, support_height_m: f64) -> f64 {
    double_bottom_m + support_height_m
}

/// The height of a tank's axis above the keel: the tank bottom plus half the outer diameter. The
/// KCG of the tanks (`gas_stability`, `loading_conditions`).
pub fn tank_axis_z(tank_bottom_m: f64, outer_diameter_m: f64) -> f64 {
    tank_bottom_m + outer_diameter_m / 2.0
}

/// One tank's place along the hull: its axis from `x0` to `x1`, and its centre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TankAxisX {
    pub x0_m: f64,
    pub x1_m: f64,
    /// `stern void + (i + 1/2) L + i g`: the point at which the IGC accelerations act on tank `i`.
    pub centre_m: f64,
}

/// The `n_long` tanks of overall length `length_m` in a line from `x_aft_m` forward, `gap_m` apart.
pub fn tank_axes_x(x_aft_m: f64, length_m: f64, gap_m: f64, n_long: usize) -> Vec<TankAxisX> {
    (0..n_long)
        .map(|i| {
            let x0 = x_aft_m + i as f64 * (length_m + gap_m);
            TankAxisX {
                x0_m: x0,
                x1_m: x0 + length_m,
                centre_m: x_aft_m + (i as f64 + 0.5) * length_m + i as f64 * gap_m,
            }
        })
        .collect()
}

/// The axis y of `n_wide` tanks of outer diameter `od_m` across `beam_m` less the side clearance on
/// each side, `gap_m` apart (m, positive to starboard; the first is the port one).
pub fn tank_axes_y(
    beam_m: f64,
    side_clearance_m: f64,
    od_m: f64,
    gap_m: f64,
    n_wide: usize,
) -> Vec<f64> {
    let available = beam_m - 2.0 * side_clearance_m;
    (0..n_wide)
        .map(|j| -available / 2.0 + od_m / 2.0 + j as f64 * (od_m + gap_m))
        .collect()
}

/// The shape of a Tank Head (GLOSSARY.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadKind {
    Hemisphere,
    Ellipsoidal2To1,
}

impl HeadKind {
    /// The head of a `tank_head_type` key; `None` for a key the engine does not model.
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "hemisphere" => Some(HeadKind::Hemisphere),
            "ellipsoidal_2to1" => Some(HeadKind::Ellipsoidal2To1),
            _ => None,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            HeadKind::Hemisphere => "hemisphere",
            HeadKind::Ellipsoidal2To1 => "ellipsoidal_2to1",
        }
    }
}

/// A Tank Head: its shape, and its length along the axis outside the insulation (`TankResult`'s
/// `head_length`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TankHead {
    pub kind: HeadKind,
    pub length_m: f64,
}

/// What places the tanks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TankPlacement {
    pub zone: CargoZone,
    pub beam_m: f64,
    pub side_clearance_m: f64,
    pub n_wide: usize,
    pub n_long: usize,
    pub gap_transverse_m: f64,
    pub gap_longitudinal_m: f64,
    /// The tank bottom above the keel ([`tank_bottom`]).
    pub tank_bottom_m: f64,
    pub outer_diameter_m: f64,
    /// The overall length of one tank, heads included.
    pub tank_length_m: f64,
}

/// One cargo tank of a Tank Layout.
#[derive(Debug, Clone, PartialEq)]
pub struct TankPlace {
    /// A short name, the same in every language: "CT1C", "CT2P", "CT1.3".
    pub name: String,
    /// Along (0: the aft one) and across (0: the port one).
    pub row: usize,
    pub column: usize,
    /// The axis, m in hull axes (x forward from the aft end, y to starboard, z up from the keel).
    pub axis: TankAxisX,
    pub y_m: f64,
    pub z_m: f64,
}

/// A tank dome: a cylinder standing on the tank top over the middle of the tank.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DomePlace {
    pub x_m: f64,
    pub y_m: f64,
    /// The tank top under it, above the keel.
    pub z_base_m: f64,
    pub diameter_m: f64,
    pub height_m: f64,
}

/// The Tank Layout (GLOSSARY.md): where each cargo tank and dome is. The capacity, the loading
/// conditions, the stability, the weights, the drawings and the Vessel Model read it.
#[derive(Debug, Clone, PartialEq)]
pub struct TankLayout {
    pub zone: CargoZone,
    pub outer_diameter_m: f64,
    pub head: TankHead,
    /// Row by row from aft, each row from port to starboard.
    pub tanks: Vec<TankPlace>,
    /// One per tank, in the same order; empty with no dome.
    pub domes: Vec<DomePlace>,
}

/// The name of the tank in `row` and `column` of a layout `n_wide` across: "CT1C" with one tank
/// across, "CT1P" and "CT1S" with two, "CT1.1" to "CT1.n" (port to starboard) with more.
fn tank_name(row: usize, column: usize, n_wide: usize) -> String {
    match (n_wide, column) {
        (1, _) => format!("CT{}C", row + 1),
        (2, 0) => format!("CT{}P", row + 1),
        (2, _) => format!("CT{}S", row + 1),
        _ => format!("CT{}.{}", row + 1, column + 1),
    }
}

/// The Tank Layout of a placement, a head and a dome `(diameter, height)` (`None` or a zero size:
/// no dome).
pub fn tank_layout(p: &TankPlacement, head: TankHead, dome: Option<(f64, f64)>) -> TankLayout {
    let xs = tank_axes_x(
        p.zone.x_aft_m,
        p.tank_length_m,
        p.gap_longitudinal_m,
        p.n_long,
    );
    let ys = tank_axes_y(
        p.beam_m,
        p.side_clearance_m,
        p.outer_diameter_m,
        p.gap_transverse_m,
        p.n_wide,
    );
    let z = tank_axis_z(p.tank_bottom_m, p.outer_diameter_m);
    let mut tanks = Vec::with_capacity(xs.len() * ys.len());
    for (row, axis) in xs.iter().enumerate() {
        for (column, y) in ys.iter().enumerate() {
            tanks.push(TankPlace {
                name: tank_name(row, column, p.n_wide),
                row,
                column,
                axis: *axis,
                y_m: *y,
                z_m: z,
            });
        }
    }
    let domes = match dome {
        Some((diameter_m, height_m)) if diameter_m > 0.0 && height_m > 0.0 => tanks
            .iter()
            .map(|t| DomePlace {
                x_m: t.axis.x0_m + p.tank_length_m / 2.0,
                y_m: t.y_m,
                z_base_m: p.tank_bottom_m + p.outer_diameter_m,
                diameter_m,
                height_m,
            })
            .collect(),
        _ => Vec::new(),
    };
    TankLayout {
        zone: p.zone,
        outer_diameter_m: p.outer_diameter_m,
        head,
        tanks,
        domes,
    }
}

impl TankPlacement {
    /// The placement of a converged arrangement: its inputs and its tanks.
    pub fn of(inputs: &BargeInputs, tanks: &TankResult) -> Self {
        Self {
            zone: CargoZone::new(
                inputs.loa,
                inputs.bow_non_tank_length.unwrap_or(0.0),
                inputs.stern_non_tank_length.unwrap_or(0.0),
            ),
            beam_m: inputs.beam,
            side_clearance_m: inputs.side_clearance.unwrap_or(0.0),
            n_wide: usize::try_from(tanks.n_tanks_wide).unwrap_or(0),
            n_long: usize::try_from(tanks.n_tanks_long).unwrap_or(0),
            gap_transverse_m: inputs.tank_gap_transverse,
            gap_longitudinal_m: inputs.tank_gap_longitudinal,
            tank_bottom_m: tank_bottom(
                inputs.double_bottom_height.unwrap_or(0.0),
                inputs.tank_support_height,
            ),
            outer_diameter_m: tanks.outer_diameter,
            tank_length_m: tanks.overall_tank_length,
        }
    }
}

/// The Tank Layout of a converged arrangement (`GasBasis`): its inputs, its tanks and the dome in
/// force `(diameter, height)`. `None` for a head the engine does not model.
pub fn arrangement_layout(
    inputs: &BargeInputs,
    tanks: &TankResult,
    dome: Option<(f64, f64)>,
) -> Option<TankLayout> {
    let head = TankHead {
        kind: HeadKind::from_key(&inputs.tank_head_type)?,
        length_m: tanks.head_length,
    };
    Some(tank_layout(&TankPlacement::of(inputs, tanks), head, dome))
}
