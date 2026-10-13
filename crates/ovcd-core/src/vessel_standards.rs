//! Standard inland-vessel classes. Port of `core/vessel_standards.py`.
//!
//! Reference data plus a validity check. The catalogue is compiled in, in the Python dict's order
//! (the notes and sources are copied from the Python file; its header comment explains the two
//! classification families). `get_standard`, `standards_for` and `validate_catalogue` copy the
//! Python behaviour, including the error texts.

use crate::checks::Check;
use crate::pystr::{repr_str, repr_str_list};
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

/// One standard inland-vessel or convoy class.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct VesselStandard {
    pub standard_id: String,
    /// "CEMT" | "CEMT-NL-2011" | "CN"
    pub family: String,
    pub class_name: String,
    /// "motor_vessel" | "pushed_convoy" | "coupled_unit"
    pub formation: String,
    /// (min, max) overall length
    pub loa_m: (f64, f64),
    pub beam_m: f64,
    pub draught_m: (f64, f64),
    pub dwt_t: (f64, f64),
    pub n_abreast: i64,
    pub n_in_line: i64,
    pub unit_beam_m: Option<f64>,
    pub min_bridge_clearance_m: Option<f64>,
    pub moulded_depth_m: Option<f64>,
    pub source: String,
    pub cargo_modes: Vec<String>,
    pub notes: String,
}

impl VesselStandard {
    pub fn loa_nominal_m(&self) -> f64 {
        0.5 * (self.loa_m.0 + self.loa_m.1)
    }

    pub fn draught_nominal_m(&self) -> f64 {
        0.5 * (self.draught_m.0 + self.draught_m.1)
    }

    pub fn dwt_nominal_t(&self) -> f64 {
        0.5 * (self.dwt_t.0 + self.dwt_t.1)
    }

    /// Design minimum bend radius at the long end of the class length range.
    pub fn r_min_m(&self, radius_multiple: f64) -> f64 {
        radius_multiple * self.loa_m.1
    }
}

/// A catalogue: (key, standard) pairs in insertion order, like the Python dict.
pub type Catalogue = Vec<(String, VesselStandard)>;

/// `VESSEL_STANDARDS`: the built-in catalogue.
pub fn built_in_catalogue() -> Catalogue {
    let rows: [(&str, VesselStandard); 12] = [
        (
            "CEMT_IV",
            VesselStandard {
                standard_id: "CEMT_IV".into(),
                family: "CEMT".into(),
                class_name: "IV".into(),
                formation: "motor_vessel".into(),
                loa_m: (80.0, 85.0),
                beam_m: 9.5,
                draught_m: (2.5, 2.5),
                dwt_t: (1000.0, 1500.0),
                n_abreast: 1,
                n_in_line: 1,
                unit_beam_m: None,
                min_bridge_clearance_m: Some(5.25),
                moulded_depth_m: None,
                source: "WG 141 Table 2.1 (p15)".into(),
                cargo_modes: vec!["dry_bulk".into(), "liquid_gas".into()],
                notes: "".into(),
            },
        ),
        (
            "CEMT_Va",
            VesselStandard {
                standard_id: "CEMT_Va".into(),
                family: "CEMT".into(),
                class_name: "Va".into(),
                formation: "motor_vessel".into(),
                loa_m: (95.0, 110.0),
                beam_m: 11.4,
                draught_m: (2.5, 4.5),
                dwt_t: (1500.0, 3000.0),
                n_abreast: 1,
                n_in_line: 1,
                unit_beam_m: None,
                min_bridge_clearance_m: Some(5.25),
                moulded_depth_m: None,
                source: "WG 141 Table 2.1 (p15)".into(),
                cargo_modes: vec!["dry_bulk".into(), "liquid_gas".into()],
                notes: "The workhorse European barge. The unit that pairs abreast into VIa.".into(),
            },
        ),
        (
            "CEMT_Vb",
            VesselStandard {
                standard_id: "CEMT_Vb".into(),
                family: "CEMT".into(),
                class_name: "Vb".into(),
                formation: "pushed_convoy".into(),
                loa_m: (172.0, 185.0),
                beam_m: 11.4,
                draught_m: (2.5, 4.5),
                dwt_t: (3200.0, 6000.0),
                n_abreast: 1,
                n_in_line: 2,
                unit_beam_m: Some(11.4),
                min_bridge_clearance_m: Some(7.0),
                moulded_depth_m: None,
                source: "WG 141 Table 2.1 (p15)".into(),
                cargo_modes: vec!["dry_bulk".into(), "liquid_gas".into()],
                notes: "2 IN LINE. R_min ~555 m at 3xLOA — a tight meander cannot usually deliver it.".into(),
            },
        ),
        (
            "CEMT_VIa",
            VesselStandard {
                standard_id: "CEMT_VIa".into(),
                family: "CEMT".into(),
                class_name: "VIa".into(),
                formation: "pushed_convoy".into(),
                loa_m: (95.0, 110.0),
                beam_m: 22.8,
                draught_m: (2.5, 4.5),
                dwt_t: (3200.0, 6000.0),
                n_abreast: 2,
                n_in_line: 1,
                unit_beam_m: Some(11.4),
                min_bridge_clearance_m: Some(7.0),
                moulded_depth_m: None,
                source: "WG 141 Table 2.1 (p15); c_C row in Table E.5 (p261)".into(),
                cargo_modes: vec!["dry_bulk".into(), "liquid_gas".into()],
                notes: "2 Va ABREAST. Short and wide, so WG 141 p101 names this class as the worst case for bend widening (c_C 0.6 loaded downstream, 1.6 empty downstream).".into(),
            },
        ),
        (
            "CEMT_VIa_motor_NL2011",
            VesselStandard {
                standard_id: "CEMT_VIa_motor_NL2011".into(),
                family: "CEMT-NL-2011".into(),
                class_name: "VIa".into(),
                formation: "motor_vessel".into(),
                loa_m: (110.0, 135.0),
                beam_m: 17.0,
                draught_m: (4.0, 4.0),
                dwt_t: (6000.0, 6000.0),
                n_abreast: 1,
                n_in_line: 1,
                unit_beam_m: None,
                min_bridge_clearance_m: Some(7.0),
                moulded_depth_m: None,
                source: "WG 141 Table 2.2 (p16) + Table 2.5 (p17)".into(),
                cargo_modes: vec!["dry_bulk".into(), "liquid_gas".into()],
                notes: "Post-CEMT'92 wide single hull. Shows Europe moving toward the Chinese beam-over-draught trade. 2,400 kW, 1,135 kW bow thruster (Table 2.5).".into(),
            },
        ),
        (
            "CN_I_motor",
            VesselStandard {
                standard_id: "CN_I_motor".into(),
                family: "CN".into(),
                class_name: "I".into(),
                formation: "motor_vessel".into(),
                loa_m: (95.0, 95.0),
                beam_m: 16.2,
                draught_m: (3.2, 3.2),
                dwt_t: (3000.0, 3000.0),
                n_abreast: 1,
                n_in_line: 1,
                unit_beam_m: None,
                min_bridge_clearance_m: None,
                moulded_depth_m: None,
                source: "WG 141 Table 2.3 (p16); channel rules Table A.2 (p163)".into(),
                cargo_modes: vec!["dry_bulk".into(), "liquid_gas".into()],
                notes: "SINGLE HULL at 3,000 t with no pairing: R_min only ~285 m at 3xLOA.".into(),
            },
        ),
        (
            "CN_II_motor",
            VesselStandard {
                standard_id: "CN_II_motor".into(),
                family: "CN".into(),
                class_name: "II".into(),
                formation: "motor_vessel".into(),
                loa_m: (90.0, 90.0),
                beam_m: 14.8,
                draught_m: (2.6, 2.6),
                dwt_t: (2000.0, 2000.0),
                n_abreast: 1,
                n_in_line: 1,
                unit_beam_m: None,
                min_bridge_clearance_m: None,
                moulded_depth_m: Some(6.5),
                source: "WG 141 Table 2.3 (p16)".into(),
                cargo_modes: vec!["dry_bulk".into(), "liquid_gas".into()],
                notes: "2,000 t on 2.6 m draught in a single standard hull. MODIFIED CLASS: moulded depth 6.5 m, not the draught+1.0 m rule of thumb. 6.5 m is where dry bulk stops being volume-bound and exactly consumes an example 4.40 m draught limit. Beyond it payload FALLS — the steel outweighs the cargo.".into(),
            },
        ),
        (
            "CN_III_motor",
            VesselStandard {
                standard_id: "CN_III_motor".into(),
                family: "CN".into(),
                class_name: "III".into(),
                formation: "motor_vessel".into(),
                loa_m: (85.0, 85.0),
                beam_m: 10.8,
                draught_m: (2.0, 2.0),
                dwt_t: (1000.0, 1000.0),
                n_abreast: 1,
                n_in_line: 1,
                unit_beam_m: None,
                min_bridge_clearance_m: None,
                moulded_depth_m: None,
                source: "WG 141 Table 2.3 (p16); channel rules Table A.2 (p163): width 30 m, R 480 m".into(),
                cargo_modes: vec!["dry_bulk".into(), "liquid_gas".into()],
                notes: "The SHALLOWEST standard class of real size: 1,000 t on 2.0 m draught. On a river where depth is the expensive axis this is the one to beat. Its L/B of 7.87 is the top of the Chinese band, so the Chinese width rule applies to it without extrapolation — unlike every European class.".into(),
            },
        ),
        (
            "CN_III_convoy_2ab",
            VesselStandard {
                standard_id: "CN_III_convoy_2ab".into(),
                family: "CN".into(),
                class_name: "III".into(),
                formation: "pushed_convoy".into(),
                loa_m: (167.0, 167.0),
                beam_m: 21.6,
                draught_m: (2.0, 2.0),
                dwt_t: (4000.0, 4000.0),
                n_abreast: 2,
                n_in_line: 2,
                unit_beam_m: Some(10.8),
                min_bridge_clearance_m: None,
                moulded_depth_m: None,
                source: "WG 141 Table 2.3 (p16); Table A.2 (p163): width 45 m, R 500 m".into(),
                cargo_modes: vec!["dry_bulk".into(), "liquid_gas".into()],
                notes: "DWT inferred as 4 x the 1,000 t Class III unit; WG 141 does not tabulate convoy deadweight. L/B 7.73, inside the Chinese band.".into(),
            },
        ),
        (
            "CN_IV_motor",
            VesselStandard {
                standard_id: "CN_IV_motor".into(),
                family: "CN".into(),
                class_name: "IV".into(),
                formation: "motor_vessel".into(),
                loa_m: (67.5, 67.5),
                beam_m: 10.8,
                draught_m: (1.6, 1.6),
                dwt_t: (500.0, 500.0),
                n_abreast: 1,
                n_in_line: 1,
                unit_beam_m: None,
                min_bridge_clearance_m: None,
                moulded_depth_m: None,
                source: "WG 141 Table 2.3 (p16); Table A.2 (p163): width 30 m, R 330 m".into(),
                cargo_modes: vec!["dry_bulk".into(), "liquid_gas".into()],
                notes: "1.6 m draught. Carried as the shallow-draught floor of the comparison.".into(),
            },
        ),
        (
            "CN_II_convoy_2ab",
            VesselStandard {
                standard_id: "CN_II_convoy_2ab".into(),
                family: "CN".into(),
                class_name: "II".into(),
                formation: "pushed_convoy".into(),
                loa_m: (186.0, 186.0),
                beam_m: 32.4,
                draught_m: (2.6, 2.6),
                dwt_t: (8000.0, 8000.0),
                n_abreast: 2,
                n_in_line: 2,
                unit_beam_m: Some(16.2),
                min_bridge_clearance_m: None,
                moulded_depth_m: None,
                source: "WG 141 Table 2.3 (p16); Table A.2 (p163): width 70 m, R 560 m".into(),
                cargo_modes: vec!["dry_bulk".into(), "liquid_gas".into()],
                notes: "4,000 t at only 2.6 m draught, but 186 m long -> R_min ~558 m. A tight meander cannot usually deliver that radius.".into(),
            },
        ),
        (
            "CN_I_convoy_2ab",
            VesselStandard {
                standard_id: "CN_I_convoy_2ab".into(),
                family: "CN".into(),
                class_name: "I".into(),
                formation: "pushed_convoy".into(),
                loa_m: (223.0, 223.0),
                beam_m: 32.4,
                draught_m: (3.5, 3.5),
                dwt_t: (12000.0, 12000.0),
                n_abreast: 2,
                n_in_line: 2,
                unit_beam_m: Some(16.2),
                min_bridge_clearance_m: None,
                moulded_depth_m: None,
                source: "WG 141 Table 2.3 (p16); Table A.2 (p163): width 70 m, R 670 m".into(),
                cargo_modes: vec!["dry_bulk".into(), "liquid_gas".into()],
                notes: "Chinese Table A.2 prices this at 70 m channel width against PIANC's 91-110 m for the same beam — the clearest single illustration of the two traditions disagreeing.".into(),
            },
        ),

    ];
    rows.into_iter().map(|(k, v)| (k.into(), v)).collect()
}

/// `get_standard`. The error is the text of Python's `KeyError` message (before `str()` wraps it
/// in `repr`, which is what `str(KeyError(msg))` gives).
pub fn get_standard(standard_id: &str) -> Result<VesselStandard, String> {
    let catalogue = built_in_catalogue();
    match catalogue.iter().find(|(k, _)| k == standard_id) {
        Some((_, v)) => Ok(v.clone()),
        None => {
            let mut known: Vec<&str> = catalogue.iter().map(|(k, _)| k.as_str()).collect();
            known.sort_unstable();
            Err(format!(
                "unknown vessel standard {}; known: {}",
                repr_str(standard_id),
                known.join(", ")
            ))
        }
    }
}

/// `standards_for`: every standard that can carry this cargo mode. The error is Python's
/// `ValueError` message.
pub fn standards_for(cargo_mode: &str) -> Result<Vec<VesselStandard>, String> {
    if cargo_mode != "dry_bulk" && cargo_mode != "liquid_gas" {
        return Err(format!(
            "cargo_mode must be dry_bulk or liquid_gas, got {}",
            repr_str(cargo_mode)
        ));
    }
    Ok(built_in_catalogue()
        .into_iter()
        .map(|(_, v)| v)
        .filter(|s| s.cargo_modes.iter().any(|m| m == cargo_mode))
        .collect())
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

    let bad = keys_where(&|v| {
        v.loa_m.0 > v.loa_m.1 || v.draught_m.0 > v.draught_m.1 || v.dwt_t.0 > v.dwt_t.1
    });
    checks.push(Check::new(
        "ranges_ordered",
        bad.is_empty(),
        if bad.is_empty() {
            "every (min, max) range is ordered".into()
        } else {
            format!("inverted range: {}", repr_str_list(&bad))
        },
    ));

    let bad = keys_where(&|v| v.beam_m <= 0.0 || v.loa_m.0 <= 0.0);
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
    let bad = keys_where(&|v| match v.unit_beam_m {
        Some(unit) if unit != 0.0 && v.n_abreast > 1 => {
            libm::fabs(unit * (v.n_abreast as f64) - v.beam_m) > 0.05
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
