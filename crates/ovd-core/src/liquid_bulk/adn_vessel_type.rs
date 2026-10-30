//! The ADN Tank Vessel Type of a cargo, from the ADN 2025 3.2.3.3 flowchart (spec 0006 §3.1).
//!
//! Source read: ADN 2025, ECE/TRANS/351, 3.2.3.3 "Flowchart, schemes and criteria for determining
//! applicable special requirements" (flowchart for liquids of Classes 3, 6.1, 8 and 9 in tanks in
//! inland navigation), and 3.2.3.2 Table C, UN 3494, column (6). A crude oil has Class 3, so the
//! flowchart branches for corrosive acids, halogenated hydrocarbons and toxic substances (2.2.61)
//! are not asked of it: a crude grade carries no such flag. An unknown flag gives type C (spec 0006
//! decision D2).

use crate::liquid_bulk::crude_grade::CrudeGrade;

/// A yes or no that may not be known.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Flag {
    Unknown,
    No,
    Yes,
}

/// The aquatic toxicity class of ADN 2.2.9.1.10.2 that the flowchart asks about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Aquatic {
    Unknown,
    /// Not classified as acute or chronic aquatic toxic.
    None,
    /// N2: chronic 2 or 3.
    N2,
    /// N1: acute 1 or chronic 1.
    N1,
}

/// The ADN Tank Vessel Type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum AdnVesselType {
    /// Type N with open cargo tanks.
    #[cfg_attr(feature = "serde", serde(rename = "N_open"))]
    NOpen,
    /// Type N with closed cargo tanks.
    #[cfg_attr(feature = "serde", serde(rename = "N_closed"))]
    NClosed,
    /// Type C.
    #[cfg_attr(feature = "serde", serde(rename = "C"))]
    C,
}

impl AdnVesselType {
    pub const ALL: [AdnVesselType; 3] = [
        AdnVesselType::NOpen,
        AdnVesselType::NClosed,
        AdnVesselType::C,
    ];

    /// The text key in files and results.
    pub const fn key(self) -> &'static str {
        match self {
            AdnVesselType::NOpen => "N_open",
            AdnVesselType::NClosed => "N_closed",
            AdnVesselType::C => "C",
        }
    }

    /// The type of a text key. Anything else is an error.
    pub fn from_key(key: &str) -> Option<AdnVesselType> {
        AdnVesselType::ALL.iter().copied().find(|t| t.key() == key)
    }
}

/// The branch of the flowchart that decided the type. The text is for the reader of a result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Branch {
    /// UN 3494 (sour crude): Table C gives type C in column (6).
    Sour,
    /// A hazard flag is not known: type C by decision D2.
    UnknownDefaultsToC,
    /// Benzene above 10 %: flowchart, type C box.
    Benzene,
    /// N1 aquatic toxicity with vapour pressure at 50 deg C of 1 kPa or more: flowchart, type C box.
    AquaticN1HighVapourPressure,
    /// CMR, N2 or N1 with low vapour pressure: type N closed, tank walls distinct from the hull.
    ClosedDistinct,
    /// Flammable, flash point below 23 deg C, no other hazard: type N closed, walls may form the hull.
    ClosedHull,
    /// Flash point of 23 deg C or more and none of the hazards above: type N open.
    Open,
    /// The user set the type.
    Override,
}

impl Branch {
    pub const fn text(self) -> &'static str {
        match self {
            Branch::Sour => "UN 3494 sour crude oil is type C (ADN 2025 Table C, column 6)",
            Branch::UnknownDefaultsToC => {
                "a hazard flag is unknown, so type C (spec 0006 decision D2)"
            }
            Branch::Benzene => "more than 10 % benzene is type C (ADN 2025 3.2.3.3 flowchart)",
            Branch::AquaticN1HighVapourPressure => {
                "aquatic toxicity N1 with vapour pressure at 50 deg C of 1 kPa or more is type C (ADN 2025 3.2.3.3 flowchart)"
            }
            Branch::ClosedDistinct => {
                "CMR, aquatic toxicity N2 or N1 with low vapour pressure: type N closed, tank walls distinct from the hull (ADN 2025 3.2.3.3 flowchart)"
            }
            Branch::ClosedHull => {
                "flammable, flash point below 23 deg C, no other hazard: type N closed, tank walls may form the hull (ADN 2025 3.2.3.3 flowchart)"
            }
            Branch::Open => {
                "flash point of 23 deg C or more, no other hazard: type N open (ADN 2025 3.2.3.3 flowchart)"
            }
            Branch::Override => "set by the user",
        }
    }
}

/// The derived type, with what the flowchart says about the tank walls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdnVesselTypeResult {
    pub vessel_type: AdnVesselType,
    /// The cargo tank walls must be distinct from the vessel hull (a double hull around integral
    /// tanks, or independent tanks). Always true for type C.
    pub walls_distinct_from_hull: bool,
    pub branch: Branch,
}

/// The flags the flowchart reads for a Class 3 crude oil.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdnFlags {
    pub sour: bool,
    pub cmr: Flag,
    pub benzene_over_10_percent: Flag,
    pub aquatic: Aquatic,
    pub flash_point_below_23c: Flag,
    pub vapour_pressure_50c_at_least_1kpa: Flag,
}

impl AdnFlags {
    /// The flags of a Crude Grade.
    pub fn of(grade: &CrudeGrade) -> AdnFlags {
        AdnFlags {
            sour: grade.sour.value,
            cmr: grade.cmr.value,
            benzene_over_10_percent: grade.benzene_over_10_percent.value,
            aquatic: grade.aquatic.value,
            flash_point_below_23c: grade.flash_point_below_23c.value,
            vapour_pressure_50c_at_least_1kpa: grade.vapour_pressure_50c_at_least_1kpa.value,
        }
    }
}

fn type_c(branch: Branch) -> AdnVesselTypeResult {
    AdnVesselTypeResult {
        vessel_type: AdnVesselType::C,
        walls_distinct_from_hull: true,
        branch,
    }
}

/// The ADN Tank Vessel Type of a crude oil, by the 3.2.3.3 flowchart, in the order of its boxes: the
/// type C box first, then the type N closed boxes, then type N open. A flag that is not known and could
/// change the answer gives type C.
pub fn derive(flags: &AdnFlags) -> AdnVesselTypeResult {
    // Table C, UN 3494: type C for every packing group.
    if flags.sour {
        return type_c(Branch::Sour);
    }
    if flags.cmr == Flag::Unknown
        || flags.benzene_over_10_percent == Flag::Unknown
        || flags.aquatic == Aquatic::Unknown
        || flags.flash_point_below_23c == Flag::Unknown
        || (flags.aquatic == Aquatic::N1
            && flags.vapour_pressure_50c_at_least_1kpa == Flag::Unknown)
    {
        return type_c(Branch::UnknownDefaultsToC);
    }
    // Box 1: benzene, and N1 with vapour pressure at 50 deg C of 1 kPa or more.
    if flags.benzene_over_10_percent == Flag::Yes {
        return type_c(Branch::Benzene);
    }
    if flags.aquatic == Aquatic::N1 && flags.vapour_pressure_50c_at_least_1kpa == Flag::Yes {
        return type_c(Branch::AquaticN1HighVapourPressure);
    }
    let flash_below_23 = flags.flash_point_below_23c == Flag::Yes;
    // Box 2: CMR; N1 with vapour pressure below 1 kPa; and, with flash point below 23 deg C, N2.
    if flags.cmr == Flag::Yes
        || flags.aquatic == Aquatic::N1
        || (flash_below_23 && flags.aquatic == Aquatic::N2)
    {
        return AdnVesselTypeResult {
            vessel_type: AdnVesselType::NClosed,
            walls_distinct_from_hull: true,
            branch: Branch::ClosedDistinct,
        };
    }
    // Box 3: flammable, flash point below 23 deg C, no other hazard.
    if flash_below_23 {
        return AdnVesselTypeResult {
            vessel_type: AdnVesselType::NClosed,
            walls_distinct_from_hull: false,
            branch: Branch::ClosedHull,
        };
    }
    // Box 4: type N open; the walls are distinct from the hull when the crude is N2.
    AdnVesselTypeResult {
        vessel_type: AdnVesselType::NOpen,
        walls_distinct_from_hull: flags.aquatic == Aquatic::N2,
        branch: Branch::Open,
    }
}

/// The type to design for: the user's override when there is one, else the derived type.
pub fn effective(flags: &AdnFlags, user_override: Option<AdnVesselType>) -> AdnVesselTypeResult {
    match user_override {
        None => derive(flags),
        Some(t) => AdnVesselTypeResult {
            vessel_type: t,
            walls_distinct_from_hull: t != AdnVesselType::NOpen
                || derive(flags).walls_distinct_from_hull,
            branch: Branch::Override,
        },
    }
}
