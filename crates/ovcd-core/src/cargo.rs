//! Cargo properties for LNG and LPG variants. Port of `core/cargo.py`.
//!
//! The database is compiled in, in the Python dict's order. Comments on the densities (sources and
//! history) live in the Python file.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// The Cargo Class: how a Cargo is carried, so which containment, capacity and stability path it
/// takes. Every branch on it is a `match`, so a new class is a compile error at each branch, never a
/// silent fall into the gas or the dry path (spec 0006 §6, ADR 0016).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum CargoClass {
    /// Liquefied gas in Type C tanks. Key `liquid_gas`.
    #[cfg_attr(feature = "serde", serde(rename = "liquid_gas"))]
    Gas,
    /// Dry bulk in a box or hopper hold. Key `dry_bulk`.
    #[cfg_attr(feature = "serde", serde(rename = "dry_bulk"))]
    DryBulk,
    /// Liquid bulk (crude oil) in cargo tanks. Key `liquid_bulk`. Crude oil has it: every path
    /// that has no liquid bulk code refuses it with a clear error until crude oil ticket 4.
    #[cfg_attr(feature = "serde", serde(rename = "liquid_bulk"))]
    LiquidBulk,
}

impl CargoClass {
    /// Every class, in a fixed order.
    pub const ALL: [CargoClass; 3] = [CargoClass::Gas, CargoClass::DryBulk, CargoClass::LiquidBulk];

    /// The text key in files and results (the old `phase` string).
    pub const fn key(self) -> &'static str {
        match self {
            CargoClass::Gas => "liquid_gas",
            CargoClass::DryBulk => "dry_bulk",
            CargoClass::LiquidBulk => "liquid_bulk",
        }
    }

    /// The class of a text key. An unknown key is an error, never a default class.
    pub fn from_key(key: &str) -> Result<CargoClass, String> {
        match CargoClass::ALL.iter().find(|c| c.key() == key) {
            Some(c) => Ok(*c),
            None => Err(format!(
                "Unknown cargo class '{key}'. Valid classes: liquid_gas, dry_bulk, liquid_bulk"
            )),
        }
    }
}

impl core::fmt::Display for CargoClass {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.key())
    }
}

/// How far a Cargo is built (spec 0006 §4): the app shows a tag for `development`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum CargoStatus {
    /// Complete: every check the Cargo needs exists.
    Released,
    /// Work in development: results are shown, and some checks do not exist yet.
    Development,
}

impl CargoStatus {
    /// The text key in the app data.
    pub const fn key(self) -> &'static str {
        match self {
            CargoStatus::Released => "released",
            CargoStatus::Development => "development",
        }
    }
}

/// Physical properties of a Cargo.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CargoProperties {
    pub name: &'static str,
    /// kg/m^3 (liquid density for gas; stowage density for dry bulk)
    pub density: f64,
    /// Storage temperature (deg C)
    pub temperature: f64,
    /// Storage pressure (barg)
    pub pressure: f64,
    pub description: &'static str,
    /// The Cargo Class (`liquid_gas`, `dry_bulk` or `liquid_bulk`).
    pub phase: CargoClass,
    /// `Released` or `Development`.
    pub status: CargoStatus,
}

/// `CARGO_DATABASE`: (type key, properties), in the Python dict's insertion order.
pub static CARGO_DATABASE: [(&str, CargoProperties); 7] = [
    (
        "LNG",
        CargoProperties {
            name: "LNG",
            density: 450.0,
            temperature: -162.0,
            pressure: 0.25,
            description: "Liquefied Natural Gas (methane-rich)",
            phase: CargoClass::Gas,
            status: CargoStatus::Released,
        },
    ),
    (
        "LNG_ethane",
        CargoProperties {
            name: "LNG / Ethane",
            density: 500.0,
            temperature: -162.0,
            pressure: 0.25,
            description: "LNG\u{2013}ethane blend at cryogenic temperature",
            phase: CargoClass::Gas,
            status: CargoStatus::Released,
        },
    ),
    (
        "LPG_propane",
        CargoProperties {
            name: "LPG (Propane)",
            density: 581.0,
            temperature: -42.1,
            pressure: 0.0,
            description: "Fully refrigerated liquefied propane (-42.1 C, near-atmospheric)",
            phase: CargoClass::Gas,
            status: CargoStatus::Released,
        },
    ),
    (
        "LPG_butane",
        CargoProperties {
            name: "LPG (Butane)",
            density: 600.0,
            temperature: -1.0,
            pressure: 0.0,
            description: "Cryogenic liquefied butane",
            phase: CargoClass::Gas,
            status: CargoStatus::Released,
        },
    ),
    (
        "LPG_mix",
        CargoProperties {
            name: "LPG (60/40 Propane/Butane)",
            density: 588.4,
            temperature: -42.1,
            pressure: 0.0,
            description: "Fully refrigerated LPG mix (60% propane / 40% butane by mass)",
            phase: CargoClass::Gas,
            status: CargoStatus::Released,
        },
    ),
    (
        "bauxite",
        CargoProperties {
            name: "Bauxite",
            density: 1500.0,
            temperature: 20.0,
            pressure: 0.0,
            description: "Dry bulk bauxite ore (stowage density, ~0.67 m3/t)",
            phase: CargoClass::DryBulk,
            status: CargoStatus::Released,
        },
    ),
    (
        "crude_oil",
        CargoProperties {
            name: "Crude oil",
            density: 870.0,
            temperature: 15.0,
            pressure: 0.0,
            description: "Crude oil, default grade (density at 15 C; see liquid_bulk::crude_grade)",
            phase: CargoClass::LiquidBulk,
            status: CargoStatus::Development,
        },
    ),
];

/// Look up cargo properties by type key. The error text is Python's `ValueError` message.
pub fn get_cargo(cargo_type: &str) -> Result<&'static CargoProperties, String> {
    match CARGO_DATABASE.iter().find(|(key, _)| *key == cargo_type) {
        Some((_, props)) => Ok(props),
        None => {
            let valid: Vec<&str> = CARGO_DATABASE.iter().map(|(key, _)| *key).collect();
            Err(format!(
                "Unknown cargo type '{cargo_type}'. Valid types: {}",
                valid.join(", ")
            ))
        }
    }
}
