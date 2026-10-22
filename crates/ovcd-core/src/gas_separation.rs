//! Longitudinal separation between the gas cargo area and the rest of the vessel.
//! Port of `core/gas_separation.py`. The rule text and clause numbers live in the Python file.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::checks::{Check, all_ok};
use crate::text;
use crate::text::{Arg, Text};

// Rule constants, each with its clause in the Python file.
/// The width of the cofferdam of the type C (liquid cargo) part, 3.6.13. It is not a limit on a gas
/// Design (issue #280), and since issue #280 it is no longer the default end space either (see
/// [`default_end_space`]). The name stays for the original Python's `COFFERDAM_MIN_M`.
pub const COFFERDAM_MIN_M: f64 = 0.60;
pub const OPENING_SETBACK_M: f64 = 2.00;
/// The space between the cargo tanks and the hold end bulkhead where the tanks have plane ends
/// (inland class rules Pt 4 Ch 4 3.5.3; ADN 9.3.1.11.3(a)).
pub const END_BULKHEAD_SPACE_M: f64 = 0.50;
/// The same space for any other tank end, such as the domed heads of a Type C tank (3.5.3). The name
/// is kept from when the code keyed it to pressure tanks (issue #281).
pub const END_BULKHEAD_SPACE_PRESSURE_M: f64 = 0.20;

/// The default end-bulkhead space at each end of the cargo area (`cofferdam_m` aft, `fwd_end_bulkhead_m`
/// forward): the type G rule for the tank end shape, 0.50 m for plane ends and 0.20 m for any other
/// end, such as the domed heads of a Type C tank. Basis: Lloyd's Register inland waterways rules
/// Pt 4 Ch 4 3.5.3 and ADN 2025 9.3.1.11.3(a). Maintainer decision on issue #280 (2026-10-22): the
/// default follows the type G rule, not the 0.60 m type C cofferdam.
pub fn default_end_space(plane_tank_ends: bool) -> f64 {
    if plane_tank_ends {
        END_BULKHEAD_SPACE_M
    } else {
        END_BULKHEAD_SPACE_PRESSURE_M
    }
}

const CLAUSE_WARNING: &str = "Separation clauses are cited from the classification rules for \
inland-waterways ships (Pt 4 Ch 4 Sec 3, Jul 2022). Confirm against the ADN text before release.";

/// Inputs for the longitudinal cargo-area separation. `Default` is the Python dataclass default.
#[derive(Debug, Clone, PartialEq)]
pub struct GasSeparationInputs {
    pub cofferdam_m: f64,
    pub opening_setback_m: f64,
    pub fwd_end_bulkhead_m: f64,
    /// Issue #381: `true` measures the opening setback from the cargo area, whose boundary is the aft
    /// face of the end space (the cofferdam stands inside the cargo area, ADN 1.2.1), as 3.3.7 and
    /// 3.3.11 do: the aft separation is the end space plus the setback. `false` is the earlier
    /// reading, the setback measured from the tank end, so the end space counts toward it (the aft
    /// separation is the greater of the two). A file saved before keeps `false`.
    pub setback_from_cargo_area: bool,
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
            cofferdam_m: default_end_space(false),
            opening_setback_m: OPENING_SETBACK_M,
            fwd_end_bulkhead_m: default_end_space(false),
            setback_from_cargo_area: true,
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
    let aft = if inp.setback_from_cargo_area {
        inp.cofferdam_m + inp.opening_setback_m
    } else {
        py_max(inp.cofferdam_m, inp.opening_setback_m)
    };
    let fwd = inp.fwd_end_bulkhead_m;
    let service = py_max(0.0, aft - inp.cofferdam_m);

    let floor = if inp.plane_tank_ends {
        END_BULKHEAD_SPACE_M
    } else {
        END_BULKHEAD_SPACE_PRESSURE_M
    };

    let checks = alloc::vec![
        // Issue #280: the type G clause 3.5.3 (ADN 9.3.1.11.3(a)), not the type C cofferdam of
        // 3.6.13. The A-60 class of the bulkhead to the spaces outside the cargo area is not checked.
        Check::new(
            "aft_end_bulkhead_space",
            inp.cofferdam_m >= floor - 1e-9,
            text!(
                "check-aft-end-bulkhead-space",
                "aft end-bulkhead space {aft} m >= {min} m ({tank_ends}, cl. 3.5.3)",
                aft = Arg::fixed(inp.cofferdam_m, 2),
                min = Arg::fixed(floor, 2),
                tank_ends = if inp.plane_tank_ends {
                    Text::fixed("check-fwd-end-bulkhead-space-plane-ends", "plane tank ends")
                } else {
                    Text::fixed("check-fwd-end-bulkhead-space-domed-ends", "domed tank ends")
                },
            ),
        ),
        if inp.setback_from_cargo_area {
            // Issue #381: the distance from the cargo area (the aft face of the end space) to the
            // first opening, not the aft separation from the tank end.
            Check::new(
                "opening_setback",
                service >= OPENING_SETBACK_M - 1e-9,
                text!(
                    "check-opening-distance",
                    "distance {distance} m from the cargo area (end space included) to any entrance or \
                     openable window >= {min} m (cl. 3.3.7 / 3.3.11)",
                    distance = Arg::fixed(service, 2),
                    min = Arg::fixed(OPENING_SETBACK_M, 2),
                ),
            )
        } else {
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
            )
        },
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
