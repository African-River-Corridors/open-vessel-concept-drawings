//! Barge design-space sweep. Port of `core/design_grid.py`.
//!
//! Behaviour was copied exactly from Python, including the errors and their message text, until
//! issue #175 changed the tank rule (below). Python's `x or 0.0` on a float is `x == 0.0`, and
//! every expression keeps Python's evaluation order so the rounding is the same. `except Exception`
//! around the capacity call is a match on the [`PyError`] it returns; the reason keeps the first
//! 80 characters of the message (`str(exc)[:80]`).
//!
//! Every cell has two tanks along the hull, as `gas_arrangement::fit` has. There is no cap on tank
//! length (issue #175: the 28 m cap is gone). A tank is limited only by its outer diameter cap and
//! by length over diameter at most 7, which the capacity check applies: a hull too long for two
//! tanks is an infeasible cell.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::barge_capacity::{BargeCapacityInputs, compute as capacity};
use crate::gas_arrangement::length_over_diameter_check;
use crate::pressure_vessel::PyError;
use crate::tanks::{DEFAULT_FILLING_LIMIT, INT_OVERFLOW_MESSAGE};

/// Tanks along the hull in every cell.
pub const TANKS_LONG: i64 = 2;
/// Kept for the 0.1.0 API only: the grid no longer uses it. The side clearance of each cell is the
/// IGC 2.4.1.2 rule, d(Vc) per tank less the insulation (issue #283, built in #277).
pub const SIDE_CLEARANCE_M: f64 = 1.0;
/// Kotug E-Pusher pattern, for the total-LOA note (m).
pub const TUG_LOA_M: f64 = 16.0;

/// The text of a reason: `str(exc)[:80]`, counted in characters.
fn first_80(s: &str) -> String {
    s.chars().take(80).collect()
}

/// One barge length x beam point of the grid.
#[derive(Debug, Clone, PartialEq)]
pub struct GridCell {
    pub barge_length_m: f64,
    pub beam_m: f64,
    pub n_tanks_long: i64,
    pub feasible: bool,
    /// Volume-bound (1.0 m default side clearance)
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

fn cell(
    barge_length_m: f64,
    beam_m: f64,
    depth_m: f64,
    cargo_type: &str,
) -> Result<GridCell, PyError> {
    let n = TANKS_LONG;
    let mut inputs = BargeCapacityInputs::new(barge_length_m, beam_m, depth_m);
    inputs.cargo_type = cargo_type.to_string();
    inputs.n_tanks_long = n;
    // The Filling Limit, not the 0.95 default of the capacity inputs (issue #423): the grid has no
    // loading-limit inputs, so it fills as a `.vdesign` run does when none is typed.
    inputs.fill_ratio = DEFAULT_FILLING_LIMIT;
    // The side clearance per tank is the IGC 2.4.1.2 rule (d(Vc) less the insulation), as the
    // arrangement sizes it; the 1.0 m default spacing is gone (issue #283, built in #277).
    let r = match capacity(&inputs) {
        Ok(r) => r,
        // The fixed range-rule error has no Python twin: Python holds the big integer and
        // answers. Fail loudly, not as an infeasible cell.
        Err(e) if e.kind == "OverflowError" && e.message == INT_OVERFLOW_MESSAGE => {
            return Err(e);
        }
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
    // The engine's own overall tank length (heads included), the one its length over diameter
    // check uses; zero when no tank was built.
    let tank_len = match &r.gas_basis {
        Some(b) if r.cargo_volume_m3 != 0.0 => b.tanks.overall_tank_length,
        _ => 0.0,
    };
    // `barge_capacity` does not raise on impossible geometry; it returns a failed
    // `feasible_geometry` check with zero cargo. Read that, or every impossible cell would
    // report feasible with 0 t. The tank length over diameter check is added here, as `fit` adds
    // it to each candidate.
    let length_check = length_over_diameter_check(&r);
    if !r.ok() || r.cargo_capacity_t <= 0.0 || length_check.as_ref().is_some_and(|c| !c.ok) {
        let why = r
            .checks
            .iter()
            .chain(length_check.iter())
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
        clearance_basis: "side clearance per tank from IGC 2.4.1.2: d(Vc) less the insulation"
            .to_string(),
        cells: Vec::new(),
    };
    for &loa in barge_lengths_m {
        for &beam in beams_m {
            grid.cells.push(cell(loa, beam, depth_m, cargo_type)?);
        }
    }
    Ok(grid)
}
