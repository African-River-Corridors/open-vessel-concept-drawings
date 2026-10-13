//! Python bindings for the `vessel-designer` wheel (`vessel_designer._native`).
//!
//! Skeleton. The module exposes only `version()`, so the release gate can prove that a wheel
//! carries a compiled extension on every target. The binding layer for the Rust core arrives with
//! the switch ticket. This crate is a workspace member but not a default member, so `cargo build`
//! and `cargo test` never need Python. The core never depends on PyO3: only this crate does.

use pyo3::prelude::*;

/// The version of the compiled core, the same as the Python package version.
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}
