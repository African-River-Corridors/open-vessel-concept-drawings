//! The ATB tug and its Notch sized from the barge (spec 0007, "Formation and Tug", issue #362).
//!
//! An articulated tug-barge (ATB) is one barge with a Notch at its transom and one tug that sits in
//! it. Unless the designer types them, the tug comes from the barge by fixed ratios, and the Notch
//! from the tug. The ratios are estimates fitted to reference ATBs (research §10, Crowley sheets,
//! Yanakiev 2025 §4.1).

use crate::stern_form::Notch;

/// The tug length over the barge length: the middle of 0.24 to 0.32 (estimate, Crowley sheets).
pub const TUG_LENGTH_OVER_BARGE_LENGTH: f64 = 0.28;
/// The tug beam over the barge beam: the middle of 0.49 to 0.57 (estimate, Crowley sheets).
pub const TUG_BEAM_OVER_BARGE_BEAM: f64 = 0.53;
/// The mean tug beam over draught of ATB tugs (sourced: Yanakiev 2025 section 4.1).
pub const TUG_BEAM_OVER_DRAUGHT: f64 = 2.33;
/// The Notch depth over the tug length (estimate: one deep-notch ATB, derived).
pub const NOTCH_DEPTH_OVER_TUG_LENGTH: f64 = 0.50;
/// The clearance each side between the tug and the Notch wall, m (estimate: the coupler).
pub const NOTCH_CLEARANCE_EACH_SIDE_M: f64 = 0.15;

/// An ATB tug's main dimensions (m).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtbTug {
    pub length_m: f64,
    pub beam_m: f64,
    pub draught_m: f64,
}

/// What the designer typed of the tug; `None` takes the ratio.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TugTyped {
    pub length_m: Option<f64>,
    pub beam_m: Option<f64>,
    pub draught_m: Option<f64>,
}

/// The ATB tug of a barge of `barge_length_m` and `barge_beam_m`: 0.28 of the length, 0.53 of the
/// beam, and a draught of beam over 2.33; each typed dimension is kept. The draught follows the
/// tug's beam, typed or not.
pub fn tug_from_barge(barge_length_m: f64, barge_beam_m: f64, typed: TugTyped) -> AtbTug {
    let length_m = typed
        .length_m
        .unwrap_or(TUG_LENGTH_OVER_BARGE_LENGTH * barge_length_m);
    let beam_m = typed
        .beam_m
        .unwrap_or(TUG_BEAM_OVER_BARGE_BEAM * barge_beam_m);
    let draught_m = typed.draught_m.unwrap_or(beam_m / TUG_BEAM_OVER_DRAUGHT);
    AtbTug {
        length_m,
        beam_m,
        draught_m,
    }
}

/// The Notch of the barge for `tug`: half the tug's length deep, the tug's beam and 0.15 m each side
/// wide; a typed depth or width is kept.
pub fn notch_for_tug(tug: &AtbTug, depth_m: Option<f64>, width_m: Option<f64>) -> Notch {
    Notch {
        depth_m: depth_m.unwrap_or(NOTCH_DEPTH_OVER_TUG_LENGTH * tug.length_m),
        width_m: width_m.unwrap_or(tug.beam_m + 2.0 * NOTCH_CLEARANCE_EACH_SIDE_M),
    }
}

/// The length of the ATB's Equivalent Hull: the barge and the tug, less the Notch the tug sits in
/// (spec 0007, "Formations").
pub fn equivalent_hull_length(barge_length_m: f64, tug: &AtbTug, notch: &Notch) -> f64 {
    barge_length_m + tug.length_m - notch.depth_m
}
