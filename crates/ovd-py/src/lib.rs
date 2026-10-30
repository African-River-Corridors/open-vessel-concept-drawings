//! Python bindings for the `open-vessel-designer` wheel (`ovd._native`).
//!
//! One function does the work: `eval(entry_point, case_json) -> result_json`, the Rust core's seam
//! for every entry point. The Python package turns its public calls into that JSON and the result
//! back into its dataclasses (`ovd/_bridge.py`), so the maths lives only in the Rust
//! core. This crate is a workspace member but not a default member, so
//! `cargo build` and `cargo test` never need Python. The core never depends on PyO3: only this
//! crate does.

use ovd_format::EvalError;
use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;

/// The version of the compiled core, the same as the Python package version.
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Run one case through the Rust core. `case_json` is `{"fn": ..., "args": {...}}`; the result is
/// `{"ok": ...}` or `{"error": {"type": ..., "message": ...}}`, as `ovd_format::eval` writes them. Input
/// the core does not accept raises `TypeError`; an unknown entry point raises `RuntimeError`.
#[pyfunction]
fn eval(entry_point: &str, case_json: &str) -> PyResult<String> {
    let input = ovd_format::parse_case(case_json)
        .map_err(|e| PyValueError::new_err(format!("case is not JSON: {e}")))?;
    match ovd_format::eval(entry_point, &input) {
        Ok(v) => Ok(v.to_string()),
        Err(EvalError::BadInput(m)) => Err(PyTypeError::new_err(m)),
        Err(e) => Err(PyRuntimeError::new_err(e.to_string())),
    }
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(eval, m)?)?;
    Ok(())
}
