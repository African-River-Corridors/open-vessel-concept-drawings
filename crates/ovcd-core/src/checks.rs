//! Shared validity-check primitive. Port of `core/checks.py`.
//!
//! Every module expresses its validity rules as a list of `Check`, not as buried `if`/error logic.
//! `severity` separates a hard failure ("error") from an advisory ("warning"). It stays text, as in
//! Python: only the exact text `"error"` gates a result, and any other text behaves as a warning.

use alloc::string::String;

/// A single validity assertion about a design or calculation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Check {
    pub name: String,
    pub ok: bool,
    pub message: String,
    /// `"error"` (gates the result) or `"warning"` (advisory).
    pub severity: String,
}

impl Check {
    /// A check with the default severity, `"error"`.
    pub fn new(name: impl Into<String>, ok: bool, message: impl Into<String>) -> Self {
        Self::with_severity(name, ok, message, "error")
    }

    pub fn with_severity(
        name: impl Into<String>,
        ok: bool,
        message: impl Into<String>,
        severity: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            ok,
            message: message.into(),
            severity: severity.into(),
        }
    }

    /// `Check.is_error`: severity is exactly `"error"`.
    pub fn is_error(&self) -> bool {
        self.severity == "error"
    }
}

/// `all_ok`: true if no error-severity check has failed (warnings do not gate).
pub fn all_ok(checks: &[Check]) -> bool {
    checks.iter().filter(|c| c.is_error()).all(|c| c.ok)
}
