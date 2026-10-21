//! The Crude Grade: the properties of one crude oil that drive a design (spec 0006 §3.1).
//!
//! Every value says where it comes from. A value is `Sourced` (a document, edition and paragraph that
//! was read) or an `Estimate` (the basis is stated). The first release has one grade, the default
//! one, whose values are all estimates: a grade for a real crude is a data row added later.

use crate::liquid_bulk::adn_vessel_type::{Aquatic, Flag};

/// How well a value is known.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Status {
    /// Taken from a document that was read. The basis names it.
    Sourced,
    /// A working value. The basis says why it was chosen.
    Estimate,
}

impl Status {
    /// The text key in files and results.
    pub const fn key(self) -> &'static str {
        match self {
            Status::Sourced => "sourced",
            Status::Estimate => "estimate",
        }
    }
}

/// A value with its status and its basis.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Graded<T> {
    pub value: T,
    pub status: Status,
    /// Where the value comes from (sourced) or why it was chosen (estimate).
    pub basis: &'static str,
}

/// How volatile the crude is: it decides open or closed tanks, or pressure tanks (spec 0006 §2 row 11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum VapourPressureClass {
    Low,
    Volatile,
}

impl VapourPressureClass {
    pub const fn key(self) -> &'static str {
        match self {
            VapourPressureClass::Low => "low",
            VapourPressureClass::Volatile => "volatile",
        }
    }
}

/// The data of one Crude Grade.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CrudeGrade {
    pub name: &'static str,
    /// Density at `density_reference_c`, kg/m3: mass, draught and stability.
    pub density_kg_m3: Graded<f64>,
    /// The temperature the density is given at (deg C).
    pub density_reference_c: f64,
    /// Density for structure, kg/m3: scantlings and tank test head.
    pub design_density_kg_m3: Graded<f64>,
    /// UN 3494 sour crude (H2S): always ADN type C, no pump room below deck.
    pub sour: Graded<bool>,
    /// CMR: Category 1A or 1B (GHS chapters 3.5 to 3.7).
    pub cmr: Graded<Flag>,
    /// More than 10 % benzene.
    pub benzene_over_10_percent: Graded<Flag>,
    /// Aquatic toxicity class (ADN 2.2.9.1.10.2): none, N2 (chronic 2 or 3) or N1.
    pub aquatic: Graded<Aquatic>,
    /// Flash point below 23 deg C.
    pub flash_point_below_23c: Graded<Flag>,
    /// Vapour pressure at 50 deg C is 1 kPa or more.
    pub vapour_pressure_50c_at_least_1kpa: Graded<Flag>,
    /// Pour point (deg C). `None`: not known, no heating is assumed.
    pub pour_point_c: Option<Graded<f64>>,
    /// Carriage temperature (deg C). `None`: ambient.
    pub carriage_temperature_c: Option<Graded<f64>>,
    pub vapour_pressure_class: Graded<VapourPressureClass>,
}

const SPEC: &str = "spec 0006 section 3.1";

/// The Cargo key the default grade belongs to.
pub const CRUDE_OIL_KEY: &str = "crude_oil";

/// The default grade: what a design uses until the user gives the data of a real crude. The density
/// values are estimates inside the range of the published assays; every hazard flag is unknown, so
/// the ADN vessel type is C (decision D2).
pub static DEFAULT_GRADE: CrudeGrade = CrudeGrade {
    name: "Crude oil (default grade)",
    density_kg_m3: Graded {
        value: 870.0,
        status: Status::Estimate,
        basis: "Middle of the SG 0.77 to 0.97 range of crude assays (03-design-practice.md 1.1); spec 0006 section 3.1",
    },
    density_reference_c: 15.0,
    design_density_kg_m3: Graded {
        value: 1025.0,
        status: Status::Estimate,
        basis: "Designer practice 1.015 to 1.025 t/m3 for tank scantlings (03-design-practice.md 1.1); spec 0006 section 3.1",
    },
    sour: Graded {
        value: false,
        status: Status::Estimate,
        basis: "Not sour until the user says so; UN 3494 is always ADN type C (ADN 2025 3.2.3.2 Table C, column 6)",
    },
    cmr: Graded {
        value: Flag::Unknown,
        status: Status::Estimate,
        basis: "Not known: unknown gives type C (spec 0006 decision D2)",
    },
    benzene_over_10_percent: Graded {
        value: Flag::Unknown,
        status: Status::Estimate,
        basis: "Not known: unknown gives type C (spec 0006 decision D2)",
    },
    aquatic: Graded {
        value: Aquatic::Unknown,
        status: Status::Estimate,
        basis: "Not known: unknown gives type C (spec 0006 decision D2)",
    },
    flash_point_below_23c: Graded {
        value: Flag::Yes,
        status: Status::Estimate,
        basis: "Crude oil is Class 3, group F1 in ADN 2025 Table C (UN 1267); flash point below 23 deg C is the usual case",
    },
    vapour_pressure_50c_at_least_1kpa: Graded {
        value: Flag::Yes,
        status: Status::Estimate,
        basis: "Published crude assays give vapour pressure far above 1 kPa at 50 deg C (03-design-practice.md 1.3: Bakken 115 kPa maximum)",
    },
    pour_point_c: None,
    carriage_temperature_c: None,
    vapour_pressure_class: Graded {
        value: VapourPressureClass::Low,
        status: Status::Estimate,
        basis: SPEC,
    },
};

/// Every grade, by Cargo key. One for now; a second oil is a data row (spec 0006 section 8, ticket 15).
pub static CRUDE_GRADES: [(&str, &CrudeGrade); 1] = [(CRUDE_OIL_KEY, &DEFAULT_GRADE)];

/// The grade of a Cargo, if it has one.
pub fn grade_of(cargo_key: &str) -> Option<&'static CrudeGrade> {
    CRUDE_GRADES
        .iter()
        .find(|(k, _)| *k == cargo_key)
        .map(|(_, g)| *g)
}
