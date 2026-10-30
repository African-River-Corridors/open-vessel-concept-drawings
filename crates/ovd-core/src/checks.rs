//! Shared validity-check primitive. Port of `core/checks.py`.
//!
//! Every module expresses its validity rules as a list of `Check`, not as buried `if`/error logic.
//! `severity` separates a hard failure ("error") from an advisory ("warning"). It stays text, as in
//! Python: only the exact text `"error"` gates a result, and any other text behaves as a warning.

use crate::text::{Msg, Text};
use alloc::string::String;

/// A single validity assertion about a design or calculation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Check {
    pub name: String,
    pub ok: bool,
    pub message: String,
    /// `"error"` (gates the result) or `"warning"` (advisory).
    pub severity: String,
    /// The Message Id of `message`, where the engine gives one (ADR 0015). Not part of the result:
    /// it is not compared and not serialised.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub msg: Option<Msg>,
}

/// Checks are equal when their name, outcome, English message and severity are.
impl PartialEq for Check {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.ok == other.ok
            && self.message == other.message
            && self.severity == other.severity
    }
}

impl Eq for Check {}

impl Check {
    /// A check with the default severity, `"error"`.
    pub fn new(name: impl Into<String>, ok: bool, message: impl Into<Text>) -> Self {
        Self::with_severity(name, ok, message, "error")
    }

    pub fn with_severity(
        name: impl Into<String>,
        ok: bool,
        message: impl Into<Text>,
        severity: impl Into<String>,
    ) -> Self {
        let message = message.into();
        Self {
            name: name.into(),
            ok,
            message: message.en,
            severity: severity.into(),
            msg: message.msg,
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
