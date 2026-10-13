//! Aft arrangement: the self-propelled vessel's stern block, derived from its equipment.
//! Port of `core/aft_arrangement.py`. The reasoning behind the arrangement lives in the Python file.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::checks::{Check, all_ok};
use crate::pyfmt::fixed;

// ISO 40 ft container envelope (ISO 668 series 1, 1AAA high-cube).
pub const ISO40_LENGTH_M: f64 = 12.192;
pub const ISO40_WIDTH_M: f64 = 2.438;
pub const ISO40_HEIGHT_HC_M: f64 = 2.896;
pub const ISO40_HEIGHT_STD_M: f64 = 2.591;

/// What has to fit in the stern block, and how much room each item needs.
/// `AftArrangementInputs::new(beam, depth)` gives the Python dataclass defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct AftArrangementInputs {
    /// Moulded beam (m).
    pub beam: f64,
    /// Moulded depth: main deck above keel (m).
    pub depth: f64,

    /// Requested number of thrusters. A maximum, not a promise: `compute` fits what the beam takes.
    pub n_thrusters: i64,
    pub thruster_envelope_m: f64,
    pub thruster_clear_m: f64,
    pub thruster_room_length_m: f64,

    pub n_battery_units: i64,
    pub n_accom_units: i64,
    pub container_length_m: f64,
    pub container_width_m: f64,
    pub container_height_m: f64,
    pub container_side_gap_m: f64,
    pub container_gap_m: f64,
    pub container_foundation_m: f64,

    pub wheelhouse_length_m: f64,
    pub wheelhouse_width_m: f64,
    pub wheelhouse_height_m: f64,
    pub wheelhouse_gap_m: f64,
    pub wheelhouse_eye_above_floor_m: f64,
    /// Height above keel the helmsman's eye must clear when raised. `None`: nothing to see over.
    pub sightline_obstruction_m: Option<f64>,
    pub sightline_margin_m: f64,
    pub wheelhouse_max_lift_m: f64,
}

impl AftArrangementInputs {
    pub fn new(beam: f64, depth: f64) -> Self {
        Self {
            beam,
            depth,
            n_thrusters: 3,
            thruster_envelope_m: 2.60,
            thruster_clear_m: 0.60,
            thruster_room_length_m: 3.50,
            n_battery_units: 1,
            n_accom_units: 2,
            container_length_m: ISO40_LENGTH_M,
            container_width_m: ISO40_WIDTH_M,
            container_height_m: ISO40_HEIGHT_HC_M,
            container_side_gap_m: 0.90,
            container_gap_m: 0.80,
            container_foundation_m: 0.30,
            wheelhouse_length_m: 3.60,
            wheelhouse_width_m: 6.40,
            wheelhouse_height_m: 3.40,
            wheelhouse_gap_m: 0.30,
            wheelhouse_eye_above_floor_m: 1.50,
            sightline_obstruction_m: None,
            sightline_margin_m: 0.50,
            wheelhouse_max_lift_m: 6.00,
        }
    }

    /// `__post_init__`: the `ValueError` message on failure.
    pub fn validate(&self) -> Result<(), String> {
        if self.beam <= 0.0 || self.depth <= 0.0 {
            return Err("beam and depth must be positive".to_string());
        }
        if self.n_thrusters < 1 {
            return Err("n_thrusters must be >= 1".to_string());
        }
        Ok(())
    }
}

/// The stern block, dimensioned. All x are metres forward of the stern (x = 0).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AftArrangementResult {
    pub length_m: f64,
    /// Longitudinal zones: (label, x_aft, x_fwd).
    pub zones: Vec<(String, f64, f64)>,
    pub thruster_offsets_m: Vec<f64>,
    pub thruster_room_length_m: f64,
    pub container_x0_m: f64,
    pub container_x1_m: f64,
    pub container_total_width_m: f64,
    pub container_top_above_keel_m: f64,
    pub container_labels: Vec<String>,
    pub container_unit_width_m: f64,
    pub container_side_gap_m: f64,
    pub container_rows: i64,
    pub container_per_row: i64,
    pub container_row_length_m: f64,
    pub container_row_gap_m: f64,
    pub wheelhouse_x0_m: f64,
    pub wheelhouse_x1_m: f64,
    pub wheelhouse_floor_lowered_m: f64,
    pub wheelhouse_top_lowered_m: f64,
    pub wheelhouse_floor_raised_m: f64,
    pub wheelhouse_top_raised_m: f64,
    pub wheelhouse_lift_m: f64,
    pub wheelhouse_width_m: f64,
    pub wheelhouse_height_m: f64,
    pub block_top_lowered_m: f64,
    pub checks: Vec<Check>,
    pub warnings: Vec<String>,
}

impl AftArrangementResult {
    /// The Python `ok` property.
    pub fn ok(&self) -> bool {
        all_ok(&self.checks)
    }
}

/// Python `max(a, b)`: the first of two equal items.
fn py_max(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// Python `min(a, b)`: the first of two equal items.
fn py_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// Python `a // b` on floats (CPython `float_floor_div`). Not `floor(a / b)`: it works from
/// `fmod`, so `1.0 // 0.1` is 9.0 although `1.0 / 0.1` rounds to 10.0.
fn py_floor_div(a: f64, b: f64) -> f64 {
    let modulo = libm::fmod(a, b);
    let mut div = (a - modulo) / b;
    // A remainder of the wrong sign moves the quotient down one. A zero remainder needs nothing.
    if modulo != 0.0 && (b < 0.0) != (modulo < 0.0) {
        div -= 1.0;
    }
    if div != 0.0 {
        let floored = libm::floor(div);
        if div - floored > 0.5 {
            floored + 1.0
        } else {
            floored
        }
    } else {
        libm::copysign(0.0, a / b)
    }
}

/// Build the stern block from its equipment. `Err` is a `ValueError` message.
pub fn compute(inp: &AftArrangementInputs) -> Result<AftArrangementResult, String> {
    inp.validate()?;
    let n_units = inp.n_battery_units + inp.n_accom_units;

    // How many units go abreast, and how many rows that needs.
    let per_row_fit = py_floor_div(
        inp.beam - inp.container_side_gap_m,
        inp.container_width_m + inp.container_side_gap_m,
    ) as i64;
    let n_per_row = core::cmp::max(1, core::cmp::min(n_units, per_row_fit));
    // `-(-n_units // n_per_row)`: ceiling division. `n_per_row` is at least 1, so `div_euclid`
    // is floor division here.
    let n_rows = -(-n_units).div_euclid(n_per_row);

    // Longitudinal build-up, stern (x = 0) forward.
    let mut zones: Vec<(String, f64, f64)> = Vec::new();
    let thr_x0 = 0.0;
    let thr_x1 = thr_x0 + inp.thruster_room_length_m;
    zones.push(("THRUSTER ROOM".to_string(), thr_x0, thr_x1));
    let x = thr_x1;

    let cont_x0 = x + inp.container_foundation_m;
    let cont_x1 = cont_x0
        + n_rows as f64 * inp.container_length_m
        + (n_rows - 1) as f64 * inp.container_gap_m;
    // `int(round(...))`: round half to even.
    let feet = libm::rint(inp.container_length_m / 0.3048) as i64;
    zones.push((
        format!("{feet} FT UNITS ({n_per_row} ABREAST x {n_rows})"),
        cont_x0,
        cont_x1,
    ));

    let wh_x0 = cont_x1 + inp.wheelhouse_gap_m;
    let wh_x1 = wh_x0 + inp.wheelhouse_length_m;
    zones.push(("WHEELHOUSE (LIFTING)".to_string(), wh_x0, wh_x1));
    let length = wh_x1;

    // Transverse: thrusters equally spaced, containers side by side.
    let pitch = inp.thruster_envelope_m + inp.thruster_clear_m;
    let usable = inp.beam - 2.0 * inp.thruster_clear_m;
    let n_fit = if pitch > 0.0 {
        py_floor_div(usable + inp.thruster_clear_m, pitch) as i64
    } else {
        0
    };
    let n_thr = core::cmp::max(1, core::cmp::min(inp.n_thrusters, n_fit));
    let span = (n_thr - 1) as f64 * pitch;
    let offsets: Vec<f64> = (0..n_thr).map(|i| -span / 2.0 + i as f64 * pitch).collect();
    let thruster_beam_needed = span + inp.thruster_envelope_m + 2.0 * inp.thruster_clear_m;
    let requested_beam_needed =
        (inp.n_thrusters - 1) as f64 * pitch + inp.thruster_envelope_m + 2.0 * inp.thruster_clear_m;

    let cont_w = n_per_row as f64 * inp.container_width_m
        + (n_per_row + 1) as f64 * inp.container_side_gap_m;
    let mut labels: Vec<String> = Vec::new();
    for _ in 0..inp.n_battery_units.max(0) {
        labels.push("BATTERY 40 FT".to_string());
    }
    for _ in 0..inp.n_accom_units.max(0) {
        labels.push("ACCOM 40 FT".to_string());
    }

    // Vertical: containers on deck; the wheelhouse stows at deck level.
    let deck = inp.depth;
    let cont_top = deck + inp.container_foundation_m + inp.container_height_m;

    let wh_floor_lo = deck;
    let wh_top_lo = wh_floor_lo + inp.wheelhouse_height_m;

    let mut lift = 0.0;
    if let Some(obstruction) = inp.sightline_obstruction_m {
        // The eye must sit `margin` above the tallest thing forward of it.
        let eye_lo = wh_floor_lo + inp.wheelhouse_eye_above_floor_m;
        let needed = obstruction + inp.sightline_margin_m;
        lift = py_max(0.0, needed - eye_lo);
    }
    let lift_used = py_min(lift, inp.wheelhouse_max_lift_m);
    let wh_floor_hi = wh_floor_lo + lift_used;
    let wh_top_hi = wh_floor_hi + inp.wheelhouse_height_m;

    let mut checks = alloc::vec![
        Check::new(
            "thrusters_fit_beam",
            thruster_beam_needed <= inp.beam,
            format!(
                "{} x {} m thrusters need {} m across a {} m beam",
                n_thr,
                fixed(inp.thruster_envelope_m, 2),
                fixed(thruster_beam_needed, 2),
                fixed(inp.beam, 2)
            ),
        ),
        Check::with_severity(
            "requested_thruster_count_fits",
            n_thr >= inp.n_thrusters,
            format!(
                "{} of {} requested thrusters fit; {} would need {} m of {} m beam",
                n_thr,
                inp.n_thrusters,
                inp.n_thrusters,
                fixed(requested_beam_needed, 2),
                fixed(inp.beam, 2)
            ),
            "warning",
        ),
        Check::new(
            "containers_fit_beam",
            cont_w <= inp.beam,
            format!(
                "{} x 40 ft units abreast need {} m across a {} m beam",
                n_per_row,
                fixed(cont_w, 2),
                fixed(inp.beam, 2)
            ),
        ),
        Check::with_severity(
            "units_fit_in_one_row",
            n_rows == 1,
            format!(
                "{} units in {} row(s) of {}; a second row adds {} m to the stern block and \
                 comes straight off the cargo zone",
                n_units,
                n_rows,
                n_per_row,
                fixed(inp.container_length_m + inp.container_gap_m, 2)
            ),
            "warning",
        ),
        Check::new(
            "wheelhouse_fits_beam",
            inp.wheelhouse_width_m <= inp.beam,
            format!(
                "wheelhouse {} m wide on a {} m beam",
                fixed(inp.wheelhouse_width_m, 2),
                fixed(inp.beam, 2)
            ),
        ),
    ];
    let mut warnings: Vec<String> = Vec::new();
    if let Some(obstruction) = inp.sightline_obstruction_m {
        let cleared = (wh_floor_hi + inp.wheelhouse_eye_above_floor_m)
            >= (obstruction + inp.sightline_margin_m - 1e-9);
        // A warning, not a gate: see the Python file.
        checks.push(Check::with_severity(
            "wheelhouse_sees_over_cargo",
            cleared,
            format!(
                "raised eye {} m >= obstruction {} m + {} m margin (lift used {} m of {} m)",
                fixed(wh_floor_hi + inp.wheelhouse_eye_above_floor_m, 2),
                fixed(obstruction, 2),
                fixed(inp.sightline_margin_m, 2),
                fixed(lift_used, 2),
                fixed(inp.wheelhouse_max_lift_m, 2)
            ),
            "warning",
        ));
        if lift > inp.wheelhouse_max_lift_m + 1e-9 {
            warnings.push(format!(
                "Sightline needs {} m of lift; the column gives {} m. Raise the column or \
                 lower the cargo top.",
                fixed(lift, 2),
                fixed(inp.wheelhouse_max_lift_m, 2)
            ));
        }
    }

    Ok(AftArrangementResult {
        length_m: length,
        zones,
        thruster_offsets_m: offsets,
        thruster_room_length_m: inp.thruster_room_length_m,
        container_x0_m: cont_x0,
        container_x1_m: cont_x1,
        container_total_width_m: cont_w,
        container_top_above_keel_m: cont_top,
        container_labels: labels,
        container_unit_width_m: inp.container_width_m,
        container_side_gap_m: inp.container_side_gap_m,
        container_rows: n_rows,
        container_per_row: n_per_row,
        container_row_length_m: inp.container_length_m,
        container_row_gap_m: inp.container_gap_m,
        wheelhouse_x0_m: wh_x0,
        wheelhouse_x1_m: wh_x1,
        wheelhouse_floor_lowered_m: wh_floor_lo,
        wheelhouse_top_lowered_m: wh_top_lo,
        wheelhouse_floor_raised_m: wh_floor_hi,
        wheelhouse_top_raised_m: wh_top_hi,
        wheelhouse_lift_m: lift_used,
        wheelhouse_width_m: inp.wheelhouse_width_m,
        wheelhouse_height_m: inp.wheelhouse_height_m,
        block_top_lowered_m: py_max(cont_top, wh_top_lo),
        checks,
        warnings,
    })
}
