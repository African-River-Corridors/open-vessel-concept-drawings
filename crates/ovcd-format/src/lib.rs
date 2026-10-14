//! The `.vdesign` format, the Defaults Profile and validation, and the parity `eval` dispatch
//! over every parity entry point.

mod eval;
mod gas_arrangement;
pub mod gas_stability;
pub mod parity;
pub mod profile;
pub mod vdesign;

pub use eval::{ENTRY_POINTS, EvalError, eval};
pub use profile::Profile;
pub use vdesign::{
    Checked, Design, DesignOption, Report, Source, StoredInput, check, fill, new_design,
};
