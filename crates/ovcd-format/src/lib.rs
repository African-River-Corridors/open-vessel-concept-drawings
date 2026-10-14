//! The `.vdesign` format, the Defaults Profile and validation, running a Design, and the `eval`
//! dispatch over every entry point of the calculation core (the Python package's seam).

mod eval;
mod gas_arrangement;
pub mod gas_stability;
pub mod profile;
mod run;
pub mod session;
mod simulate;
pub mod vdesign;

pub use eval::{ENTRY_POINTS, EvalError, eval, parse_case};
pub use profile::Profile;
pub use run::{
    CONVOY_NOT_AVAILABLE, HashStatus, RunOption, RunOutcome, canonical_json, hash_warning,
    results_hash, run_design, run_option, simulate_params_of, stamp, summary_lines,
};
pub use vdesign::{
    Checked, Design, DesignOption, Group, Note, Report, Source, StoredInput, check, fill,
    new_design,
};
