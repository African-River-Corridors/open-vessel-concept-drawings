//! The `.vdesign` format, the Defaults Profile and validation (later tickets), and the parity
//! `eval` dispatch over every parity entry point.

mod eval;
pub mod parity;

pub use eval::{ENTRY_POINTS, EvalError, eval};
