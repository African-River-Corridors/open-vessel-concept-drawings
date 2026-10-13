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
pub mod cargo;
pub mod checks;
pub mod gas_separation;
pub mod pyfmt;
