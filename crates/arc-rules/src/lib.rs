//! A domain-free rules engine (spec 0004, ADR 0013, ADR 0019).
//!
//! Versioned, content-hashed Rule Sets; the strictest Rule governs; a Rule Trace on every result.
//! The caller declares its domain: its object types (string keys), its Measure registry, its
//! applicability vocabulary and its formula shapes ([`domain`]). The evaluator reads values only
//! through [`eval::MeasureSource`].
//!
//! `no_std` plus `alloc`: no file, clock, environment or randomness access. The Rule Set schema
//! 1.x is frozen for outside users: changes are additive only (see `README.md`).

#![no_std]

extern crate alloc;

pub mod domain;
pub mod eval;
#[cfg(feature = "load")]
pub mod load;
pub mod model;
