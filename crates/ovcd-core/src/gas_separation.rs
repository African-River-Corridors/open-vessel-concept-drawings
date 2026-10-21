//! Longitudinal separation between the gas cargo area and the rest of the vessel.
//! Port of `core/gas_separation.py`. The rule text and clause numbers live in the Python file.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::checks::{Check, all_ok};
use crate::text;
use crate::text::{Arg, Text};

// Rule constants, each with its clause in the Python file.
pub const COFFERDAM_MIN_M: f64 = 0.60;
pub const OPENING_SETBACK_M: f64 = 2.00;
/// The space between the cargo tanks and the hold end bulkhead where the tanks have plane ends
/// (inland class rules Pt 4 Ch 4 3.5.3; ADN 9.3.1.11.3(a)).
pub const END_BULKHEAD_SPACE_M: f64 = 0.50;
/// The same space for any other tank end, such as the domed heads of a Type C tank (3.5.3). The name
/// is kept from when the code keyed it to pressure tanks (issue #281).
pub const END_BULKHEAD_SPACE_PRESSURE_M: f64 = 0.20;

const CLAUSE_WARNING: &str = "Separation clauses are cited from the classification rules for \
inland-waterways ships (Pt 4 Ch 4 Sec 3, Jul 2022). Confirm against the ADN text before release.";

/// Inputs for the longitudinal cargo-area separation. `Default` is the Python dataclass default.
#[derive(Debug, Clone, PartialEq)]
pub struct GasSeparationInputs {
    pub cofferdam_m: f64,
    pub opening_setback_m: f64,
    pub fwd_end_bulkhead_m: f64,
    /// Kept so older callers still build. No check reads it: the end-bulkhead space is keyed to the
    /// tank end shape, as 3.5.3 says, not to the tank kind (issue #281).
    pub is_pressure_tank: bool,
    /// The tanks have plane ends. Every tank the engine models has domed heads (hemispherical or
    /// 2:1 ellipsoidal), so no caller sets it yet.
    pub plane_tank_ends: bool,
    /// Kept so older callers still build. No check reads it (the inland type-G floors went in
    /// issue #215).
    pub side_clearance_m: f64,
    /// Kept so older callers still build. No check reads it (see `side_clearance_m`).
    pub double_bottom_m: f64,
}

impl Default for GasSeparationInputs {
    fn default() -> Self {
        Self {
            cofferdam_m: COFFERDAM_MIN_M,
            opening_setback_m: OPENING_SETBACK_M,
            fwd_end_bulkhead_m: END_BULKHEAD_SPACE_M,
            is_pressure_tank: true,
            plane_tank_ends: false,
            side_clearance_m: 0.0,
            double_bottom_m: 0.0,
        }
    }
}

impl GasSeparationInputs {
    /// `__post_init__`: the `ValueError` message on failure.
    pub fn validate(&self) -> Result<(), String> {
        if self.cofferdam_m < 0.0 || self.opening_setback_m < 0.0 || self.fwd_end_bulkhead_m < 0.0 {
            return Err("separations must be non-negative".to_string());
        }
        Ok(())
    }
}

/// The length the gas code takes off the cargo area at each end.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GasSeparationResult {
    pub aft_separation_m: f64,
    pub fwd_separation_m: f64,
    pub cofferdam_m: f64,
    pub service_space_m: f64,
    pub total_length_taken_m: f64,
    pub checks: Vec<Check>,
    pub warnings: Vec<Text>,
}

impl GasSeparationResult {
    /// The Python `ok` property.
    pub fn ok(&self) -> bool {
        all_ok(&self.checks)
    }
}

/// Python `max(a, b)`: the first of two equal items.
fn py_max(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// Resolve the aft and forward separations, with the rules as checks. `Err` is a `ValueError`.
pub fn compute(inp: &GasSeparationInputs) -> Result<GasSeparationResult, String> {
    inp.validate()?;
    let aft = py_max(inp.cofferdam_m, inp.opening_setback_m);
    let fwd = inp.fwd_end_bulkhead_m;
    let service = py_max(0.0, aft - inp.cofferdam_m);

    let floor = if inp.plane_tank_ends {
        END_BULKHEAD_SPACE_M
    } else {
        END_BULKHEAD_SPACE_PRESSURE_M
    };

    let checks = alloc::vec![
        Check::new(
            "cofferdam_min_width",
            inp.cofferdam_m >= COFFERDAM_MIN_M - 1e-9,
            text!(
                "check-cofferdam-min-width",
                "cofferdam {cofferdam} m >= {min} m (cl. 3.6.13)",
                cofferdam = Arg::fixed(inp.cofferdam_m, 2),
                min = Arg::fixed(COFFERDAM_MIN_M, 2),
            ),
        ),
        Check::new(
            "opening_setback",
            aft >= OPENING_SETBACK_M - 1e-9,
            text!(
                "check-opening-setback",
                "aft separation {aft} m >= {min} m from the cargo area to any entrance or openable \
                 window (cl. 3.3.7 / 3.3.11)",
                aft = Arg::fixed(aft, 2),
                min = Arg::fixed(OPENING_SETBACK_M, 2),
            ),
        ),
        Check::new(
            "fwd_end_bulkhead_space",
            fwd >= floor - 1e-9,
            text!(
                "check-fwd-end-bulkhead-space",
                "forward end-bulkhead space {fwd} m >= {min} m ({tank_ends}, cl. 3.5.3)",
                fwd = Arg::fixed(fwd, 2),
                min = Arg::fixed(floor, 2),
                tank_ends = if inp.plane_tank_ends {
                    Text::fixed("check-fwd-end-bulkhead-space-plane-ends", "plane tank ends")
                } else {
                    Text::fixed("check-fwd-end-bulkhead-space-domed-ends", "domed tank ends")
                },
            ),
        ),
    ];

    Ok(GasSeparationResult {
        aft_separation_m: aft,
        fwd_separation_m: fwd,
        cofferdam_m: inp.cofferdam_m,
        service_space_m: service,
        total_length_taken_m: aft + fwd,
        checks,
        warnings: alloc::vec![Text::fixed("warn-separation-clauses", CLAUSE_WARNING)],
    })
}
