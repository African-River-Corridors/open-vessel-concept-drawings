//! IMO Type C horizontal cylindrical tank geometry and arrangement. Port of `core/tanks.py`.
//!
//! Behaviour is copied exactly, including the errors and their message text. Every `**` is
//! `libm::pow`, every float in a message prints as Python prints it, and every expression keeps
//! Python's evaluation order so the rounding is the same. Tank counts are `i64`: arithmetic on
//! them is checked, and a result that no 64-bit field can hold is an `OverflowError` with one
//! fixed message (the integer range rule in `parity/README.md`).

use alloc::format;
use alloc::string::{String, ToString};
use core::f64::consts::PI;

use libm::{fabs, pow, sqrt};

use crate::models::{BargeInputs, TankResult};
use crate::pressure_vessel::{PyError, calculate_wall_thicknesses};
use crate::pyfmt::{fixed, py_str};

/// The one message for a tank count product that no 64-bit field can hold.
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
    // Guard None values (auto-optimization may not have run yet).
    let side_clearance = inputs.side_clearance.unwrap_or(0.0);
    let double_bottom_height = inputs.double_bottom_height.unwrap_or(0.0);

    let n_wide = inputs.n_tanks_wide;
    let n_long = inputs.n_tanks_long;
    let n_total = n_wide.checked_mul(n_long).ok_or_else(overflow)?;

    // --- Transverse (beam) sizing ---
    let available_beam = inputs.beam - 2.0 * side_clearance;
    let wide_less_one = n_wide.checked_sub(1).ok_or_else(overflow)?;
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

    // --- Wall thickness determination ---
    let (wall_cyl_mm, wall_head_mm) = match inputs.tank_wall_thickness {
        // User-provided fixed thickness.
        Some(t) => (t, t),
        // Auto-calculate from the pressure vessel formula.
        None => calculate_wall_thicknesses(
            outer_diameter,
            inputs.design_pressure,
            &inputs.tank_material,
            &inputs.tank_head_type,
            inputs.weld_efficiency,
        )?,
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
    // Head length uses the full OD (the outer envelope including insulation).
    let hemisphere = inputs.tank_head_type == "hemisphere";
    let (head_length_outer, volume_one_head_inner, sa_one_head_shell);
    if hemisphere {
        head_length_outer = outer_radius;
        volume_one_head_inner = (2.0 / 3.0) * PI * pow(head_inner_radius, 3.0);
        sa_one_head_shell = 2.0 * PI * pow(shell_outer_radius, 2.0);
    } else if inputs.tank_head_type == "ellipsoidal_2to1" {
        head_length_outer = outer_radius / 2.0;
        volume_one_head_inner = (1.0 / 3.0) * PI * pow(head_inner_radius, 3.0);
        sa_one_head_shell = 1.084 * 2.0 * PI * pow(shell_outer_radius, 2.0) / 2.0;
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
    let long_less_one = n_long.checked_sub(1).ok_or_else(overflow)?;
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
        1.084 * PI * pow(outer_radius, 2.0)
    };
    let total_od_sa_per_tank = cylinder_sa_od + 2.0 * sa_one_head_od;
    let insulation_volume_per_tank = total_od_sa_per_tank * insulation_m;
    let insulation_density_t = inputs.insulation_density / 1000.0; // kg/m^3 to t/m^3
    let total_insulation_weight = n_total_f * insulation_volume_per_tank * insulation_density_t;

    // --- Tank top height above keel ---
    let tank_top_above_keel = double_bottom_height + inputs.tank_support_height + outer_diameter;

    Ok(TankResult {
        n_tanks_wide: n_wide,
        n_tanks_long: n_long,
        n_tanks_total: n_total,
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
        design_pressure: inputs.design_pressure,
        tank_material: if inputs.tank_wall_thickness.is_none() {
            inputs.tank_material.clone()
        } else {
            String::new()
        },
    })
}

/// Transverse free surface moment for a horizontal cylindrical tank: `cargo_density * I_fs`.
///
/// The free surface is a rectangle as wide as the chord at the fill level and as long as the
/// cylinder, so `I_fs = (1/12) * L * chord^3`. The caller divides by displacement.
pub fn free_surface_moment_cylindrical(
    inner_radius: f64,
    cylinder_length: f64,
    fill_ratio: f64,
    cargo_density: f64,
) -> f64 {
    let r = inner_radius;
    // Fill height from bottom of cylinder.
    let h = fill_ratio * 2.0 * r;

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
