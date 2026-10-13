//! Cargo properties for LNG and LPG variants. Port of `core/cargo.py`.
//!
//! The database is compiled in, in the Python dict's order. Comments on the densities (sources and
//! history) live in the Python file.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// Physical properties of a liquid gas cargo.
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
    /// "liquid_gas" (cryogenic tanks) or "dry_bulk" (hopper hold)
    pub phase: &'static str,
}

/// `CARGO_DATABASE`: (type key, properties), in the Python dict's insertion order.
pub static CARGO_DATABASE: [(&str, CargoProperties); 6] = [
    (
        "LNG",
        CargoProperties {
            name: "LNG",
            density: 450.0,
            temperature: -162.0,
            pressure: 0.25,
            description: "Liquefied Natural Gas (methane-rich)",
            phase: "liquid_gas",
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
            phase: "liquid_gas",
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
            phase: "liquid_gas",
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
            phase: "liquid_gas",
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
            phase: "liquid_gas",
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
            phase: "dry_bulk",
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
