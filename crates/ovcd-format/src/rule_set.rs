//! OVCD's Rule Sets (spec 0004, "Rule Set (data)" and "Loading at run time"; ADR 0013, ADR 0019):
//! the built-in sets, the vocabulary OVCD gives the loader of the `arc-rules` kernel, and the
//! released-versions and provenance gate.
//!
//! The loader, the content hash, the library and the selection checks are the kernel's
//! ([`arc_rules::load`]). The schema beside the files (`rules/rule-set.schema.json`, the same file as
//! the kernel's) says the same as the loader: a file the schema refuses, the loader refuses too.

use crate::trace::{applicability_values, measure_info};
use arc_rules::domain::{Domain, FormulaShape, InputDef, InputScope, MeasureInfo};
pub use arc_rules::load::{
    ACCESS, Loaded, ROLES, Released, SCHEMA, Selection, VersionClass, canonical_json,
    check_released, check_versions, content_hash, content_json, describe, is_rule_id, released,
    rule_content_json, version_class,
};
use ovcd_core::rule_trace::{FORMULA_SHAPES, ObjectType};
use ovcd_core::rules::{Consequence, Rule};

/// The built-in Rule Sets: `(path, text)`, one file per set version, embedded so that every binary
/// and the WebAssembly build carry them.
const BUILT_IN: &[(&str, &str)] = &[
    (
        "rules/is2008-a/1.0.0.json",
        include_str!("../rules/is2008-a/1.0.0.json"),
    ),
    (
        "rules/igc-2g/1.0.0.json",
        include_str!("../rules/igc-2g/1.0.0.json"),
    ),
    (
        "rules/lr-inland-gas/1.0.0.json",
        include_str!("../rules/lr-inland-gas/1.0.0.json"),
    ),
    (
        "rules/lr-inland-gas/1.1.0.json",
        include_str!("../rules/lr-inland-gas/1.1.0.json"),
    ),
    (
        "rules/ovcd/1.0.0.json",
        include_str!("../rules/ovcd/1.0.0.json"),
    ),
    (
        "rules/ovcd/2.0.0.json",
        include_str!("../rules/ovcd/2.0.0.json"),
    ),
    (
        "rules/ovcd/2.1.0.json",
        include_str!("../rules/ovcd/2.1.0.json"),
    ),
];

/// What OVCD declares to the kernel's loader: its Measure registry (`ovcd rules --measures`), the
/// inputs an Applicability may name (each one value per Option), and its formula shapes.
#[derive(Debug, Clone, Copy, Default)]
pub struct Ovcd;

impl Domain for Ovcd {
    fn measure(&self, id: &str) -> Option<MeasureInfo> {
        measure_info(id)
    }

    fn input(&self, name: &str) -> Option<InputDef> {
        applicability_values(name).map(|values| InputDef {
            values,
            scope: InputScope::Design,
        })
    }

    fn shapes(&self) -> &[FormulaShape] {
        FORMULA_SHAPES
    }

    fn design_object(&self) -> &str {
        ObjectType::Option.key()
    }

    fn measure_list_hint(&self) -> Option<&str> {
        Some("`ovcd rules --measures` lists them")
    }
}

/// Read one Rule Set file against OVCD's vocabulary ([`arc_rules::load::load`]).
pub fn load(text: &str) -> Result<Loaded, String> {
    arc_rules::load::load(text, &Ovcd)
}

/// The Rule Sets known to a run: the built-in sets first, then each loaded file in order. The
/// kernel's library ([`arc_rules::load::Library`]), loading against OVCD's vocabulary.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Library(arc_rules::load::Library);

impl std::ops::Deref for Library {
    type Target = arc_rules::load::Library;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Library {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Library {
    /// The built-in sets only.
    pub fn built_in() -> Library {
        let mut lib = Library::default();
        for (path, text) in BUILT_IN {
            let loaded = load(text).unwrap_or_else(|e| panic!("built-in Rule Set {path}: {e}"));
            lib.add(loaded)
                .unwrap_or_else(|e| panic!("built-in Rule Set {path}: {e}"));
        }
        lib.check()
            .unwrap_or_else(|e| panic!("built-in Rule Sets: {e}"));
        lib
    }

    /// Load a file's text and add it. Returns the set's `id@version`.
    pub fn add_text(&mut self, text: &str) -> Result<String, String> {
        self.0.add_text(text, &Ovcd)
    }
}

// --- released versions and provenance (#273) ----------------------------------------------------

/// The list of released Rule Set versions: `{id, version, hash}` each. A version is released when a
/// tagged release ships it (`docs/agents/build-process.md`). A listed file never changes.
pub const RELEASED: &str = include_str!("../rules/RELEASED.json");

/// The provenance gate: a Rule of a published origin (`regulation`, `class`, `guideline`,
/// `national-practice`) with no paragraph is an error (the loader
/// refuses one too; the gate checks every set again). The report lists, per set, the blocking Rules
/// that are not verified: a notice, never a failure.
pub fn provenance(lib: &Library) -> (Vec<String>, String) {
    let mut errors = Vec::new();
    let mut report = String::new();
    for s in lib.sets() {
        for r in &s.rules {
            if r.origin.is_published() && r.source.locator.trim().is_empty() {
                errors.push(format!(
                    "{}: `{}` Rule `{}` cites no paragraph",
                    s.key(),
                    r.origin.key(),
                    r.id
                ));
            }
        }
        let blocking: Vec<&Rule> = s
            .rules
            .iter()
            .filter(|r| r.consequence == Consequence::Block && r.verified.is_none())
            .collect();
        // OVCD Checks are never verified; the published Rules are named.
        let published: Vec<&str> = blocking
            .iter()
            .filter(|r| r.origin.is_published())
            .map(|r| r.id.as_str())
            .collect();
        report.push_str(&format!(
            "{}: {} blocking Rule(s) unverified ({} OVCD Checks){}{}\n",
            s.key(),
            blocking.len(),
            blocking.len() - published.len(),
            if published.is_empty() { "" } else { ": " },
            published.join(", ")
        ));
    }
    (errors, report)
}

/// Every check of the CI gate: released versions, version classes and rule ids, provenance.
pub fn gate(lib: &Library) -> (Vec<String>, String) {
    let mut errors = match released(RELEASED) {
        Ok(list) => check_released(lib, &list),
        Err(e) => vec![e],
    };
    errors.extend(check_versions(lib));
    let (more, report) = provenance(lib);
    errors.extend(more);
    (errors, report)
}
