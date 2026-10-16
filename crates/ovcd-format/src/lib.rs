//! The `.vdesign` format, the Defaults Profile and validation, running a Design, and the `eval`
//! dispatch over every entry point of the calculation core (the Python package's seam).

pub mod class_input;
mod cost;
mod eval;
pub mod ga;
mod gas_arrangement;
pub mod gas_stability;
mod objects;
pub mod profile;
mod run;
pub mod session;
mod simulate;
pub mod vdesign;

pub use eval::{ENTRY_POINTS, EvalError, eval, parse_case};
pub use ga::{GaViews, ga_views, ga_views_of};
pub use profile::Profile;
pub use run::{
    CONVOY_NOT_AVAILABLE, HashStatus, RunOption, RunOutcome, UNHASHED_KEYS, canonical_json,
    hash_warning, results_hash, run_design, run_option, simulate_params_of, stamp, summary_lines,
};
pub use session::{
    DesignSession, Evaluation, Message, OpenReport, OptionEvaluation, Scope, Severity, input_schema,
};
pub use vdesign::{
    Checked, Design, DesignOption, Report, Source, StoredInput, check, fill, fill_as_new,
    new_design,
};
