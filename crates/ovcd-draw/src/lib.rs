//! The Drawing Format Standard in Rust (ADR 0009, spec 0003).
//!
//! A drawing document in, normalised content out (clauses 4 to 18). The core takes text and values
//! and returns values. It opens no file, reads no clock and uses no network: a document that names
//! a file is given a [`pipeline::Files`] to read it through.

pub mod conformance;
pub mod data;
pub mod entities;
pub mod json;
pub mod numbers;
pub mod pipeline;
pub mod rules;
pub mod schema;

pub use pipeline::{Files, NoFiles, Normalised, RunResult, run_text, run_value};
