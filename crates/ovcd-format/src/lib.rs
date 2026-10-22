//! The `.vdesign` format, the Defaults Profile and validation, running a Design, and the `eval`
//! dispatch over every entry point of the calculation core (the Python package's seam).

pub mod class_input;
mod convoy;
mod cost;
mod eval;
mod formation;
pub mod ga;
mod gas_arrangement;
pub mod gas_stability;
mod liquid;
mod objects;
pub mod offsets;
pub mod profile;
pub mod rule_set;
mod run;
pub mod session;
pub mod shipd;
mod simulate;
mod text_ids;
pub mod trace;
pub mod vdesign;

pub use eval::{ENTRY_POINTS, EvalError, eval, parse_case};
pub use ga::{CONVOY_NO_VIEWS, GaViews, ga_views, ga_views_for, ga_views_named, ga_views_of};
pub use profile::Profile;
pub use run::{
    CONVOY_NOT_AVAILABLE, HashStatus, RunOption, RunOutcome, UNHASHED_KEYS, canonical_json,
    hash_warning, mesh_hash, option_hull_table, option_meshes, option_vessel_model, results_hash,
    run_design, run_design_with, run_option, run_option_with, simulate_params_of, stamp,
    summary_lines,
};
pub use session::{
    DesignSession, Evaluation, Message, OpenReport, OptionEvaluation, Scope, Severity, input_schema,
};
pub use text_ids::msg_json;
pub use vdesign::{
    Checked, Design, DesignOption, Report, Source, StoredInput, check, fill, fill_as_new,
    new_design,
};
