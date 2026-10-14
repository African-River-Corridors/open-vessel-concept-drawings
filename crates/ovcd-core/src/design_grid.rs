//! Barge design-space sweep. Port of `core/design_grid.py`.
//!
//! Behaviour is copied exactly, including the errors and their message text. Python's
//! `x or 0.0` on a float is `x == 0.0`, `math.ceil` is `libm::ceil` with CPython's NaN and
//! infinity errors, and every expression keeps Python's evaluation order so the rounding is the
//! same. `except Exception` around the capacity call is a match on the [`PyError`] it returns;
//! the reason keeps the first 80 characters of the message (`str(exc)[:80]`).
//!
//! A tank count above the 64-bit range is the fixed `OverflowError` of the integer range rule
//! (`parity/README.md`).

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::barge_capacity::{BargeCapacityInputs, compute as capacity};
use crate::pressure_vessel::PyError;
use crate::tanks::INT_OVERFLOW_MESSAGE;

/// Overall Type-C tank length cap (m). ESTIMATE.
pub const MAX_TANK_LENGTH_M: f64 = 28.0;
/// Standard IGC-2G protective-location spacing per side (m).
pub const SIDE_CLEARANCE_M: f64 = 1.0;
/// Mirrors the `models.py` default.
pub const BOW_NON_TANK_PCT: f64 = 0.10;
/// Mirrors the `models.py` default.
pub const STERN_NON_TANK_PCT: f64 = 0.15;
/// Kotug E-Pusher pattern, for the total-LOA note (m).
pub const TUG_LOA_M: f64 = 16.0;

/// The text of a reason: `str(exc)[:80]`, counted in characters.
fn first_80(s: &str) -> String {
    s.chars().take(80).collect()
}

/// Tanks in a line for a given barge length (length-only, so curves stay smooth).
///
/// `ZeroDivisionError` when `max_tank_length_m` is zero. `math.ceil` raises `ValueError` on NaN
/// and `OverflowError` on infinity. A ceiling below 1 is 1, however negative.
pub fn n_tanks_for_length(barge_length_m: f64, max_tank_length_m: f64) -> Result<i64, PyError> {
    let available = (1.0 - BOW_NON_TANK_PCT - STERN_NON_TANK_PCT) * barge_length_m;
    if max_tank_length_m == 0.0 {
        return Err(PyError {
            kind: "ZeroDivisionError",
            message: "float division by zero".to_string(),
        });
    }
    let ratio = available / max_tank_length_m;
    if ratio.is_nan() {
        return Err(PyError {
            kind: "ValueError",
            message: "cannot convert float NaN to integer".to_string(),
        });
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
        return Err(PyError {
            kind: "OverflowError",
            message: INT_OVERFLOW_MESSAGE.to_string(),
        });
    }
    Ok(n as i64)
}

/// One barge length x beam point of the grid.
#[derive(Debug, Clone, PartialEq)]
pub struct GridCell {
    pub barge_length_m: f64,
    pub beam_m: f64,
    pub n_tanks_long: i64,
    pub feasible: bool,
    /// Volume-bound (1.0 m standard side clearance)
    pub cargo_t: Option<f64>,
    pub tank_outer_diameter_m: Option<f64>,
    pub tank_length_m: Option<f64>,
    /// Overall tank length / OD
    pub slenderness: Option<f64>,
    pub full_load_draft_m: Option<f64>,
    pub air_draft_m: Option<f64>,
    /// Why infeasible, if so
    pub reason: String,
}

impl GridCell {
    fn infeasible(barge_length_m: f64, beam_m: f64, n: i64, reason: String) -> Self {
        GridCell {
            barge_length_m,
            beam_m,
            n_tanks_long: n,
            feasible: false,
            cargo_t: None,
            tank_outer_diameter_m: None,
            tank_length_m: None,
            slenderness: None,
            full_load_draft_m: None,
            air_draft_m: None,
            reason,
        }
    }
}

/// The swept grid: the inputs echoed, then one cell per length (outer) and beam (inner).
#[derive(Debug, Clone, PartialEq)]
pub struct DesignGrid {
    pub barge_lengths_m: Vec<f64>,
    pub beams_m: Vec<f64>,
    pub depth_m: f64,
    pub cargo_type: String,
    pub clearance_basis: String,
    pub max_tank_length_m: f64,
    pub cells: Vec<GridCell>,
}

impl DesignGrid {
    /// Feasible cells for one barge length, ordered by beam (one plotted line).
    pub fn curve(&self, barge_length_m: f64) -> Vec<&GridCell> {
        self.cells
            .iter()
            .filter(|c| c.barge_length_m == barge_length_m && c.feasible)
            .collect()
    }
}

/// `_tank_overall_length`: one tank's overall length with a 2.0 m longitudinal gap.
fn tank_overall_length(barge_length_m: f64, n: i64) -> f64 {
    let available = (1.0 - BOW_NON_TANK_PCT - STERN_NON_TANK_PCT) * barge_length_m;
    // `n >= 1`, so `n - 1` cannot overflow; the int-to-float conversion rounds once, as Python's.
    (available - ((n - 1) as f64) * 2.0) / (n as f64)
}

fn cell(
    barge_length_m: f64,
    beam_m: f64,
    depth_m: f64,
    cargo_type: &str,
) -> Result<GridCell, PyError> {
    // Outside Python's `try`: an error here ends the sweep.
    let n = n_tanks_for_length(barge_length_m, MAX_TANK_LENGTH_M)?;
    let mut inputs = BargeCapacityInputs::new(barge_length_m, beam_m, depth_m);
    inputs.cargo_type = cargo_type.to_string();
    inputs.n_tanks_long = n;
    inputs.side_clearance_override_m = Some(SIDE_CLEARANCE_M);
    let r = match capacity(&inputs) {
        Ok(r) => r,
        Err(e) => {
            return Ok(GridCell::infeasible(
                barge_length_m,
                beam_m,
                n,
                first_80(&e.message),
            ));
        }
    };
    // `x or 0.0`: None and 0.0 both give 0.0.
    let od = match r.tank_outer_diameter_m {
        Some(x) if x != 0.0 => x,
        _ => 0.0,
    };
    // `a and b`: a falsy `a` (here 0.0) is the result, `b` only runs when `a` is truthy.
    let tank_len = if r.cargo_volume_m3 == 0.0 {
        r.cargo_volume_m3
    } else {
        tank_overall_length(barge_length_m, n)
    };
    // `barge_capacity` does not raise on impossible geometry; it returns a failed
    // `feasible_geometry` check with zero cargo. Read that, or every impossible cell would
    // report feasible with 0 t.
    if !r.ok() || r.cargo_capacity_t <= 0.0 {
        let why = r
            .checks
            .iter()
            .find(|c| !c.ok)
            .map_or("no cargo volume", |c| c.message.as_str());
        return Ok(GridCell::infeasible(
            barge_length_m,
            beam_m,
            n,
            first_80(why),
        ));
    }
    Ok(GridCell {
        barge_length_m,
        beam_m,
        n_tanks_long: n,
        feasible: true,
        cargo_t: Some(r.cargo_capacity_t),
        tank_outer_diameter_m: Some(od),
        tank_length_m: Some(tank_len),
        slenderness: if od != 0.0 { Some(tank_len / od) } else { None },
        full_load_draft_m: Some(r.full_load_draft_m),
        air_draft_m: Some(r.air_draft_m),
        reason: String::new(),
    })
}

/// Sweep every barge length against every beam. An empty list on either side gives no cells.
pub fn sweep(
    barge_lengths_m: &[f64],
    beams_m: &[f64],
    depth_m: f64,
    cargo_type: &str,
) -> Result<DesignGrid, PyError> {
    let mut grid = DesignGrid {
        barge_lengths_m: barge_lengths_m.to_vec(),
        beams_m: beams_m.to_vec(),
        depth_m,
        cargo_type: cargo_type.to_string(),
        clearance_basis: "1.0 m standard side clearance per side (IGC-2G protective location)"
            .to_string(),
        max_tank_length_m: MAX_TANK_LENGTH_M,
        cells: Vec::new(),
    };
    for &loa in barge_lengths_m {
        for &beam in beams_m {
            grid.cells.push(cell(loa, beam, depth_m, cargo_type)?);
        }
    }
    Ok(grid)
}
