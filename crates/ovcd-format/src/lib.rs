//! The `.vdesign` format, the Defaults Profile and validation, and the parity `eval` dispatch
//! over every parity entry point.

mod eval;
mod gas_arrangement;
pub mod gas_stability;
pub mod parity;
pub mod profile;
mod run;
mod simulate;
pub mod vdesign;

pub use eval::{ENTRY_POINTS, EvalError, eval};
pub use profile::Profile;
pub use run::{
    CONVOY_NOT_AVAILABLE, HashStatus, RunOption, RunOutcome, canonical_json, hash_warning,
    results_hash, run_design, run_option, simulate_params_of, stamp, summary_lines,
};
pub use vdesign::{
    Checked, Design, DesignOption, Report, Source, StoredInput, check, fill, new_design,
};
