//! Pure calculation core for Open Vessel Concept Drawings.
//!
//! `no_std` plus `alloc`: no file, clock, environment or randomness access, so it builds for
//! `wasm32-unknown-unknown`. Modules mirror the Python core files one to one
//! (`src/vessel_designer/core/<name>.py` -> `<name>.rs`). Every transcendental uses `libm`;
//! `clippy.toml` bans the standard-library versions, `powi` and `mul_add`.

#![no_std]

extern crate alloc;
#[cfg(test)]
extern crate std;

pub mod aft_arrangement;
pub mod barge_capacity;
pub mod barge_cost;
pub mod box_stability;
pub mod cargo;
pub mod checks;
pub mod design_grid;
pub mod electric_energy;
pub mod ga_adapter;
pub mod gas_arrangement;
pub mod gas_separation;
pub mod gas_stability;
pub mod hull;
pub mod hydrostatics;
pub mod journey_derating;
pub mod lpg_saturation;
pub mod models;
pub mod pressure_vessel;
pub mod protective_location;
pub mod pyfmt;
pub mod pyint;
pub mod pystr;
mod pystr_tables;
pub mod resistance;
pub mod rules;
pub mod squat;
pub mod tanks;
pub mod text;
mod unprintable;
pub mod vessel_classes;
pub mod vessel_standards;
pub mod waves;
pub mod weights;
