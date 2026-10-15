//! Protective location checks per IGF Code and IGC Code for Type C tanks.
//! Port of `core/protective_location.py`. The ADN type G vessel type was removed in issue #175: the
//! engine applies IGC rules only to gas cargo tanks.

use alloc::format;
use alloc::string::{String, ToString};

/// Vessel/gas code classification controlling protective location rules (`VesselType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VesselType {
    Igf,
    Igc1g,
    Igc2g,
    Igc3g,
    /// IGC type 2G with d(Vc) applied exactly as written (per-tank volume, tank steel datum).
    Igc2gExact,
}

impl VesselType {
    /// `VesselType(value)`. The error text is Python's `ValueError` message.
    pub fn from_value(value: &str) -> Result<Self, String> {
        match value {
            "IGF" => Ok(Self::Igf),
            "IGC_1G" => Ok(Self::Igc1g),
            "IGC_2G" => Ok(Self::Igc2g),
            "IGC_3G" => Ok(Self::Igc3g),
            "IGC_2G_EXACT" => Ok(Self::Igc2gExact),
            other => Err(format!("{} is not a valid VesselType", py_repr_str(other))),
        }
    }

    /// The enum's value text.
    pub fn value(self) -> &'static str {
        match self {
            Self::Igf => "IGF",
            Self::Igc1g => "IGC_1G",
            Self::Igc2g => "IGC_2G",
            Self::Igc3g => "IGC_3G",
            Self::Igc2gExact => "IGC_2G_EXACT",
        }
    }
}

/// Python's `repr()` of a `str`, as used in the enum's `ValueError` message.
fn py_repr_str(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::new();
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            // Python escapes what `str.isprintable()` rejects: \xNN, \uNNNN or \UNNNNNNNN.
            c if crate::unprintable::is_unprintable(c) => {
                let cp = c as u32;
                if cp < 0x100 {
                    out.push_str(&format!("\\x{cp:02x}"));
                } else if cp < 0x10000 {
                    out.push_str(&format!("\\u{cp:04x}"));
                } else {
                    out.push_str(&format!("\\U{cp:08x}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// Result of protective location clearance checks.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ProtectiveLocationResult {
    pub vessel_type: String,
    /// Volume-based minimum distance (m)
    pub d_vc: f64,
    /// Vc used for d(Vc) lookup (m^3)
    pub total_tank_volume: f64,
    pub cs_required: f64,
    pub cs_actual: f64,
    pub cs_ok: bool,
    pub cb_required: f64,
    pub cb_actual: f64,
    pub cb_ok: bool,
}

/// Required side and bottom clearances from protective location rules.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RequiredClearances {
    pub cs_required: f64,
    pub cb_required: f64,
    pub d_vc: f64,
}

/// Python's `max(a, b)`: the first of equal items, and `b` only if `b > a` (so NaN never wins
/// unless it is first).
fn py_max(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// Python's `min(a, b)`: the first of equal items, and `b` only if `b < a`.
fn py_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// Volume-based minimum distance d(Vc) per IGC/IGF Code. `vc` is the total tank volume (m^3).
pub fn distance_d_vc(vc: f64) -> f64 {
    if vc <= 1000.0 {
        0.8
    } else if vc <= 5000.0 {
        1.0
    } else {
        2.0
    }
}

/// IGC 2.4.1.1 / IGF 5.3.3.4.2 d(Vc), exactly: the gross volume of ONE tank (m^3).
pub fn d_vc_exact(vc_individual_m3: f64) -> f64 {
    let v = vc_individual_m3;
    if v <= 1000.0 {
        return 0.8;
    }
    if v < 5000.0 {
        return 0.75 + v * 0.2 / 4000.0;
    }
    if v < 30000.0 {
        return 0.8 + v / 25000.0;
    }
    2.0
}

/// Required side clearance Cs from tank to hull side. `None` is Python's `None`, which the
/// function rejects like any unknown type.
pub fn required_side_protection(
    vessel_type: Option<VesselType>,
    beam: f64,
    d_vc: f64,
) -> Result<f64, String> {
    match vessel_type {
        Some(VesselType::Igf | VesselType::Igc1g) => Ok(py_max(py_min(beam / 5.0, 11.5), d_vc)),
        Some(VesselType::Igc2g) => Ok(d_vc),
        Some(VesselType::Igc3g) => Ok(0.8),
        Some(VesselType::Igc2gExact) => Ok(d_vc),
        None => Err("Unknown vessel type: None".to_string()),
    }
}

/// Required bottom clearance Cb from tank bottom to keel: the same formula for all types.
pub fn required_bottom_protection(beam: f64, d_vc: f64, _vessel_type: Option<VesselType>) -> f64 {
    py_max(py_min(beam / 15.0, 2.0), d_vc)
}

/// Compute the required Cs and Cb for a vessel type text, beam and d(Vc).
pub fn compute_required_clearances(
    vessel_type_str: &str,
    beam: f64,
    d_vc: f64,
) -> Result<RequiredClearances, String> {
    let vessel_type = VesselType::from_value(vessel_type_str)?;
    let cs_required = required_side_protection(Some(vessel_type), beam, d_vc)?;
    let cb_required = required_bottom_protection(beam, d_vc, Some(vessel_type));
    Ok(RequiredClearances {
        cs_required,
        cb_required,
        d_vc,
    })
}

/// Run the full protective location check.
pub fn check_protective_location(
    vessel_type_str: &str,
    beam: f64,
    total_tank_volume: f64,
    side_clearance: f64,
    double_bottom_height: f64,
    tank_support_height: f64,
) -> Result<ProtectiveLocationResult, String> {
    let vessel_type = VesselType::from_value(vessel_type_str)?;
    let d_vc = distance_d_vc(total_tank_volume);
    let cs_required = required_side_protection(Some(vessel_type), beam, d_vc)?;
    let cb_required = required_bottom_protection(beam, d_vc, Some(vessel_type));
    let cb_actual = double_bottom_height + tank_support_height;
    Ok(ProtectiveLocationResult {
        vessel_type: vessel_type_str.to_string(),
        d_vc,
        total_tank_volume,
        cs_required,
        cs_actual: side_clearance,
        cs_ok: side_clearance >= cs_required,
        cb_required,
        cb_actual,
        cb_ok: cb_actual >= cb_required,
    })
}
