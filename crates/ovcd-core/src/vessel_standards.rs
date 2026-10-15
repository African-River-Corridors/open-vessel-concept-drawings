//! Standard inland-vessel classes: the 0.1.0 lookup API over the Vessel Class catalogue.
//!
//! Since issue #94 the classes are `catalogue/vessel_classes.yaml`, the one source (see
//! [`crate::vessel_classes`]); this module presents them as [`VesselStandard`]s. The ids of the 12
//! classes that were listed by hand before stay valid as aliases. [`get_standard`],
//! [`standards_for`] and [`validate_catalogue`] keep the 0.1.0 contract and error texts, except where
//! the catalogue made a change (see `CHANGELOG.md`): a figure the catalogue does not give is `None`,
//! and `standards_for` no longer filters by Cargo.

use crate::checks::Check;
use crate::pystr::{repr_str, repr_str_list};
use crate::vessel_classes::{CLASSES, ClassRow};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// One standard inland-vessel or convoy class.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct VesselStandard {
    /// The catalogue id, e.g. `CEMT-Va`.
    pub standard_id: String,
    /// The scheme, e.g. `CEMT-1992`.
    pub family: String,
    /// The class label; the catalogue id.
    pub class_name: String,
    /// `motor_vessel`, `pushed_convoy`, `coupled_unit`, `pushed_barge`, ...
    pub formation: String,
    /// (min, max) overall length; `None` when the class gives none.
    pub loa_m: Option<(f64, f64)>,
    /// The largest beam of the class; `None` when the class gives none.
    pub beam_m: Option<f64>,
    pub draught_m: Option<(f64, f64)>,
    pub dwt_t: Option<(f64, f64)>,
    pub n_abreast: i64,
    pub n_in_line: i64,
    /// Beam of one barge: beam over barges abreast, for a convoy class.
    pub unit_beam_m: Option<f64>,
    /// The lowest bridge clearance the class gives: the bottom of its air-draft range.
    pub min_bridge_clearance_m: Option<f64>,
    /// No class sets one (no source gives it); kept so 0.1.0 code that builds a standard still works.
    pub moulded_depth_m: Option<f64>,
    pub source: String,
    /// Catalogue data. These do not filter classes.
    pub cargo_modes: Vec<String>,
    pub notes: String,
    /// (min, max) air draft the class allows.
    pub air_draft_m: Option<(f64, f64)>,
    pub region: String,
    pub waters: String,
    pub confidence: String,
}

fn mid(r: Option<(f64, f64)>) -> Option<f64> {
    r.map(|(lo, hi)| 0.5 * (lo + hi))
}

impl VesselStandard {
    pub fn loa_nominal_m(&self) -> Option<f64> {
        mid(self.loa_m)
    }

    pub fn draught_nominal_m(&self) -> Option<f64> {
        mid(self.draught_m)
    }

    pub fn dwt_nominal_t(&self) -> Option<f64> {
        mid(self.dwt_t)
    }

    /// Design minimum bend radius at the long end of the class length range.
    pub fn r_min_m(&self, radius_multiple: f64) -> Option<f64> {
        self.loa_m.map(|(_, hi)| radius_multiple * hi)
    }
}

impl From<&ClassRow> for VesselStandard {
    fn from(c: &ClassRow) -> Self {
        let (abreast, in_line) = c.counts();
        let beam = c.beam_max_m();
        VesselStandard {
            standard_id: c.id.to_string(),
            family: c.family.to_string(),
            class_name: c.id.to_string(),
            formation: c.formation.to_string(),
            loa_m: c.loa_m,
            beam_m: beam,
            draught_m: c.draught_m,
            dwt_t: c.dwt_t,
            n_abreast: abreast,
            n_in_line: in_line,
            unit_beam_m: beam
                .filter(|_| abreast * in_line > 1)
                .map(|b| b / abreast as f64),
            min_bridge_clearance_m: c.air_draft_limit_m(),
            moulded_depth_m: None,
            source: c.source.to_string(),
            cargo_modes: c.cargo_modes.iter().map(|m| m.to_string()).collect(),
            notes: c.notes.to_string(),
            air_draft_m: c.air_draft_m,
            region: c.region.to_string(),
            waters: c.waters.to_string(),
            confidence: c.confidence.to_string(),
        }
    }
}

/// A catalogue: (key, standard) pairs in insertion order, like the Python dict.
pub type Catalogue = Vec<(String, VesselStandard)>;

/// `VESSEL_STANDARDS`: the built-in catalogue, every class of `catalogue/vessel_classes.yaml`.
pub fn built_in_catalogue() -> Catalogue {
    CLASSES
        .iter()
        .map(|c| (c.id.to_string(), VesselStandard::from(c)))
        .collect()
}

/// `get_standard`. The error is the text of Python's `KeyError` message (before `str()` wraps it
/// in `repr`, which is what `str(KeyError(msg))` gives).
pub fn get_standard(standard_id: &str) -> Result<VesselStandard, String> {
    match crate::vessel_classes::find(standard_id) {
        Some(c) => Ok(VesselStandard::from(c)),
        None => {
            let mut known: Vec<&str> = CLASSES.iter().map(|c| c.id).collect();
            known.sort_unstable();
            Err(format!(
                "unknown vessel standard {}; known: {}",
                repr_str(standard_id),
                known.join(", ")
            ))
        }
    }
}

/// `standards_for`: the 0.1.0 function that listed the standards able to carry a cargo mode. Cargo
/// is never a property of a Vessel Class (GLOSSARY), so since issue #94 it still checks that the
/// mode is `dry_bulk` or `liquid_gas`, as before, and returns every class. The error is Python's
/// `ValueError` message.
pub fn standards_for(cargo_mode: &str) -> Result<Vec<VesselStandard>, String> {
    if cargo_mode != "dry_bulk" && cargo_mode != "liquid_gas" {
        return Err(format!(
            "cargo_mode must be dry_bulk or liquid_gas, got {}",
            repr_str(cargo_mode)
        ));
    }
    Ok(built_in_catalogue().into_iter().map(|(_, v)| v).collect())
}

/// `validate_catalogue`: integrity of the fleet catalogue, as a list of checks.
pub fn validate_catalogue(catalogue: &[(String, VesselStandard)]) -> Vec<Check> {
    let keys_where = |pred: &dyn Fn(&VesselStandard) -> bool| -> Vec<&str> {
        catalogue
            .iter()
            .filter(|(_, v)| pred(v))
            .map(|(k, _)| k.as_str())
            .collect()
    };
    let mut checks = Vec::new();

    let bad: Vec<&str> = catalogue
        .iter()
        .filter(|(k, v)| *k != v.standard_id)
        .map(|(k, _)| k.as_str())
        .collect();
    checks.push(Check::new(
        "unique_ids",
        bad.is_empty(),
        if bad.is_empty() {
            "every key matches its standard_id".into()
        } else {
            format!("key/standard_id mismatch: {}", repr_str_list(&bad))
        },
    ));

    let inverted = |r: Option<(f64, f64)>| r.is_some_and(|(lo, hi)| lo > hi);
    let bad = keys_where(&|v| inverted(v.loa_m) || inverted(v.draught_m) || inverted(v.dwt_t));
    checks.push(Check::new(
        "ranges_ordered",
        bad.is_empty(),
        if bad.is_empty() {
            "every (min, max) range is ordered".into()
        } else {
            format!("inverted range: {}", repr_str_list(&bad))
        },
    ));

    let bad = keys_where(&|v| {
        v.beam_m.is_some_and(|b| b <= 0.0) || v.loa_m.is_some_and(|(lo, _)| lo <= 0.0)
    });
    checks.push(Check::new(
        "positive_dimensions",
        bad.is_empty(),
        if bad.is_empty() {
            "all beams and lengths positive".into()
        } else {
            format!("non-positive dimension: {}", repr_str_list(&bad))
        },
    ));

    // A convoy's overall beam should be its unit beam times the number abreast.
    // Python: `v.unit_beam_m and v.n_abreast > 1` (None and 0.0 are falsy).
    let bad = keys_where(&|v| match (v.unit_beam_m, v.beam_m) {
        (Some(unit), Some(beam)) if unit != 0.0 && v.n_abreast > 1 => {
            libm::fabs(unit * (v.n_abreast as f64) - beam) > 0.05
        }
        _ => false,
    });
    checks.push(Check::new(
        "convoy_beam_consistent",
        bad.is_empty(),
        if bad.is_empty() {
            "convoy beam = unit beam x n_abreast".into()
        } else {
            format!(
                "beam does not reconcile with n_abreast: {}",
                repr_str_list(&bad)
            )
        },
    ));

    let bad = keys_where(&|v| v.source.is_empty());
    checks.push(Check::with_severity(
        "source_cited",
        bad.is_empty(),
        if bad.is_empty() {
            "every class cites its table".into()
        } else {
            format!("no source: {}", repr_str_list(&bad))
        },
        "warning",
    ));

    checks
}
