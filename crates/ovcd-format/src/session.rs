//! The Design session (spec 0002): one Design and its Defaults Profile, edited through plain
//! values in and out. This is the seam between the desktop app and the engine. It has no Tauri
//! dependency and no file, clock or screen access: the app reads and writes the file itself.
//!
//! Every edit re-derives the Design from its own text with [`fill`], so the session cannot drift
//! from what the file would hold, and the defaults and ignored-input rules have one source.

use crate::profile::{HardRange, Profile, SoftRange};
use crate::run::{HashStatus, RunOutcome, hash_warning, run_design, stamp};
use crate::vdesign::{
    self, AppliesTo, Design, DesignOption, ENGINE_VERSION, Group, INPUTS, InputKind, InputSpec,
    KEYED_DEFAULTS, Note, Source, StoredInput,
};
use serde_json::{Map, Value, json};

/// The groups of the input table, in the order the app shows them.
pub const GROUPS: &[Group] = &[
    Group::Required,
    Group::HullAndContainment,
    Group::GasTanks,
    Group::WaterwayLimits,
    Group::SpeedAndPower,
];

/// The input schema: every input of an Option, with the Defaults Profile's values and ranges.
/// One source: the input table gives the kinds, groups and applicability; the profile gives the
/// defaults, ranges and basis.
pub fn input_schema(profile: &Profile) -> Value {
    let inputs: Vec<Value> = INPUTS.iter().map(|s| input_entry(s, profile)).collect();
    json!({
        "groups": GROUPS.iter().map(|g| g.as_str()).collect::<Vec<_>>(),
        "inputs": inputs,
    })
}

fn input_entry(spec: &InputSpec, profile: &Profile) -> Value {
    let name = spec.name;
    let mut entry = Map::new();
    entry.insert("name".into(), json!(name));
    entry.insert("kind".into(), json!(kind_name(spec.kind)));
    entry.insert("group".into(), json!(spec.group.as_str()));
    entry.insert("required".into(), json!(spec.required));
    entry.insert("applies_to".into(), applies_to(spec));
    entry.insert("unit".into(), json!(profile.unit(name)));
    entry.insert("choices".into(), choices(spec.kind));
    entry.insert("default".into(), default_of(name, profile));
    entry.insert("hard".into(), hard_of(name, profile));
    entry.insert("soft".into(), soft_of(name, profile));
    entry.insert("basis".into(), basis_of(name, profile));
    if matches!(spec.kind, InputKind::Formation) {
        let mut fields = Map::new();
        for (key, profile_name) in [
            ("abreast", "formation_abreast"),
            ("in_line", "formation_in_line"),
        ] {
            let mut field = Map::new();
            field.insert("unit".into(), json!(profile.unit(profile_name)));
            field.insert("hard".into(), hard_of(profile_name, profile));
            field.insert("soft".into(), soft_of(profile_name, profile));
            field.insert("basis".into(), basis_of(profile_name, profile));
            fields.insert(key.into(), Value::Object(field));
        }
        entry.insert("fields".into(), Value::Object(fields));
    }
    Value::Object(entry)
}

fn kind_name(kind: InputKind) -> &'static str {
    match kind {
        InputKind::Number { nullable: false } => "number",
        InputKind::Number { nullable: true } => "number_or_none",
        InputKind::Bool => "bool",
        InputKind::Choice(_) => "choice",
        InputKind::Cargo => "cargo",
        InputKind::Formation => "formation",
    }
}

/// The Cargo Containment keys an input applies to.
fn applies_to(spec: &InputSpec) -> Value {
    let keys: Vec<&str> = match spec.applies_to {
        AppliesTo::All => vdesign::CARGO_CONTAINMENTS
            .iter()
            .map(|(c, _)| *c)
            .collect(),
        AppliesTo::Only(c) => vec![c],
    };
    Value::Array(keys.into_iter().map(Value::from).collect())
}

/// The valid choices of a choice or Cargo input, each as a key and the name to show.
fn choices(kind: InputKind) -> Value {
    match kind {
        InputKind::Choice(keys) => Value::Array(
            keys.iter()
                .map(|k| json!({ "key": k, "name": k }))
                .collect(),
        ),
        InputKind::Cargo => Value::Array(
            ovcd_core::cargo::CARGO_DATABASE
                .iter()
                .map(|(key, props)| json!({ "key": key, "name": props.name }))
                .collect(),
        ),
        _ => Value::Null,
    }
}

fn default_of(name: &str, profile: &Profile) -> Value {
    if let Some((_, by, keys)) = KEYED_DEFAULTS.iter().find(|(n, _, _)| *n == name) {
        let values: Map<String, Value> = keys
            .iter()
            .filter_map(|key| {
                profile
                    .default_by(name, by, key)
                    .map(|v| ((*key).to_string(), v.clone()))
            })
            .collect();
        return json!({ "by": by, "values": values });
    }
    profile.default(name).cloned().unwrap_or(Value::Null)
}

fn hard_of(name: &str, profile: &Profile) -> Value {
    let Ok(h) = profile.hard(name) else {
        return Value::Null;
    };
    hard_json(&h)
}

fn hard_json(h: &HardRange) -> Value {
    let mut o = Map::new();
    for (key, bound) in [("gt", h.gt), ("ge", h.ge), ("lt", h.lt), ("le", h.le)] {
        if let Some(x) = bound {
            o.insert(key.into(), json!(x));
        }
    }
    if o.is_empty() {
        Value::Null
    } else {
        Value::Object(o)
    }
}

fn soft_of(name: &str, profile: &Profile) -> Value {
    match profile.soft(name) {
        Ok(Some(SoftRange { min, max })) => json!({ "min": min, "max": max }),
        _ => Value::Null,
    }
}

fn basis_of(name: &str, profile: &Profile) -> Value {
    match profile.basis(name) {
        "" => Value::Null,
        b => json!(b),
    }
}

/// A Design and the Defaults Profile its defaults and ranges come from.
#[derive(Debug, Clone)]
pub struct Session {
    design: Design,
    profile: Profile,
}

impl Default for Session {
    fn default() -> Session {
        Session::new()
    }
}

impl Session {
    /// A new session on the built-in Defaults Profile, on the starter Design of [`new_design`].
    pub fn new() -> Session {
        let profile = Profile::built_in();
        let design = vdesign::new_design(&profile).expect("the built-in profile makes a starter");
        Session { design, profile }
    }

    /// A new session on a loaded Defaults Profile.
    pub fn with_profile(profile: Profile) -> Session {
        let design = vdesign::new_design(&profile).expect("a checked profile makes a starter");
        Session { design, profile }
    }

    /// The name of the Defaults Profile in use.
    pub fn profile_name(&self) -> &str {
        self.profile.name()
    }

    /// The input schema of the Defaults Profile in use.
    pub fn input_schema(&self) -> Value {
        input_schema(&self.profile)
    }

    /// Load a Defaults Profile from its text, refusing one that loosens a hard range. Inputs
    /// already in the Design stay as they are; left-out inputs take the new defaults.
    pub fn load_profile(&mut self, text: &str) -> Result<Value, String> {
        self.profile = Profile::from_json(text)?;
        self.resync()?;
        self.state()
    }

    /// Go back to the built-in Defaults Profile.
    pub fn use_built_in_profile(&mut self) -> Result<Value, String> {
        self.profile = Profile::built_in();
        self.resync()?;
        self.state()
    }

    /// The same starter Design as `ovcd new`.
    pub fn new_design(&mut self) -> Result<Value, String> {
        self.design = vdesign::new_design(&self.profile)?;
        self.state()
    }

    /// Read a `.vdesign` text and fill it as [`fill`] does, and report each Option's results-hash
    /// status. A text with hard errors still opens: the Design is what the file holds, and
    /// [`evaluate`] says what is wrong.
    pub fn open(&mut self, text: &str) -> Result<Value, String> {
        let filled = crate::fill(text, &self.profile);
        let Some(design) = filled.design else {
            return Err(filled.report.errors.join("; "));
        };
        self.design = design;
        let mut out = self.state()?;
        out["options"] = Value::Array(self.hash_statuses());
        Ok(out)
    }

    /// Set one input of one Option to a typed value. The value is kept as typed even when it
    /// fails a check, so the field shows what was typed (spec 0002).
    pub fn set_input(&mut self, option: &str, name: &str, value: Value) -> Result<Value, String> {
        if !INPUTS.iter().any(|s| s.name == name) {
            return Err(format!("unknown input `{name}`"));
        }
        let stored = StoredInput {
            value: vdesign::normalise(value),
            source: Source::Typed,
        };
        {
            let o = self.option_mut(option)?;
            o.inputs.retain(|(n, _)| n != name);
            o.inputs.push((name.into(), stored));
        }
        self.resync()?;
        self.state()
    }

    /// Take one input of one Option back to its Defaults Profile value. A required input has no
    /// default and cannot be reset.
    pub fn reset_input(&mut self, option: &str, name: &str) -> Result<Value, String> {
        let spec = INPUTS
            .iter()
            .find(|s| s.name == name)
            .ok_or_else(|| format!("unknown input `{name}`"))?;
        if spec.required {
            return Err(format!(
                "`{name}` is required and has no default; give it a value"
            ));
        }
        self.option_mut(option)?.inputs.retain(|(n, _)| n != name);
        self.resync()?;
        self.state()
    }

    /// Set the Formation of one Option: the kind, and `abreast` and `in_line` for a pushed
    /// convoy. `abreast` and `in_line` are ignored for the other kinds.
    pub fn set_formation(
        &mut self,
        option: &str,
        kind: &str,
        abreast: Option<u64>,
        in_line: Option<u64>,
    ) -> Result<Value, String> {
        if !vdesign::FORMATION_KINDS.contains(&kind) {
            return Err(format!(
                "`formation.kind` must be one of: {}",
                vdesign::FORMATION_KINDS.join(", ")
            ));
        }
        let value = if kind == "pushed_convoy" {
            json!({
                "kind": kind,
                "abreast": abreast
                    .ok_or("`formation.abreast` must be a whole number for a pushed_convoy")?,
                "in_line": in_line
                    .ok_or("`formation.in_line` must be a whole number for a pushed_convoy")?,
            })
        } else {
            json!({ "kind": kind })
        };
        self.set_input(option, "formation", value)
    }

    /// Add an Option with the starter inputs of [`new_design`], under a new name.
    pub fn add_option(&mut self, name: &str) -> Result<Value, String> {
        self.check_new_name(name)?;
        let starter = vdesign::new_design(&self.profile)?;
        let mut option = starter.options.into_iter().next().expect("one Option");
        option.name = name.into();
        option.results_hash = None;
        self.design.options.push(option);
        self.resync()?;
        self.state()
    }

    /// Copy an Option, its inputs and its stored results hash.
    pub fn duplicate_option(&mut self, name: &str, new_name: &str) -> Result<Value, String> {
        self.check_new_name(new_name)?;
        let source = self
            .design
            .options
            .iter()
            .find(|o| o.name == name)
            .ok_or_else(|| format!("there is no Option '{name}'"))?;
        let mut option = source.clone();
        option.name = new_name.into();
        self.design.options.push(option);
        self.resync()?;
        self.state()
    }

    /// Give an Option another name.
    pub fn rename_option(&mut self, name: &str, new_name: &str) -> Result<Value, String> {
        self.check_new_name(new_name)?;
        let o = self.option_mut(name)?;
        o.name = new_name.into();
        self.resync()?;
        self.state()
    }

    /// Delete an Option. The last one stays: a Design needs at least one Option.
    pub fn delete_option(&mut self, name: &str) -> Result<Value, String> {
        if self.design.options.len() == 1 {
            return Err("a Design needs at least one Option".into());
        }
        let before = self.design.options.len();
        self.design.options.retain(|o| o.name != name);
        if self.design.options.len() == before {
            return Err(format!("there is no Option '{name}'"));
        }
        self.resync()?;
        self.state()
    }

    /// Validate and run every Option. Per Option: each input with the hard errors, soft warnings
    /// and cross-check messages it takes part in; and either the results (the same JSON object
    /// `ovcd run --json` gives) or the engine's message. An Option with hard errors is not run.
    pub fn evaluate(&self) -> Result<Value, String> {
        let checked = crate::check(&self.design.to_text(), &self.profile);
        let runs = run_design(&self.design);
        let mut file_errors = Vec::new();
        let mut file_warnings = Vec::new();
        for note in checked.notes.iter().filter(|n| n.option.is_none()) {
            if note.error {
                file_errors.push(json!(note.message));
            } else {
                file_warnings.push(json!(note.message));
            }
        }
        let mut options = Vec::new();
        for (option, run) in self.design.options.iter().zip(&runs) {
            let notes: Vec<&Note> = checked
                .notes
                .iter()
                .filter(|n| n.option.as_deref() == Some(option.name.as_str()))
                .collect();
            let hard = notes.iter().any(|n| n.error);
            let inputs: Vec<Value> = option
                .inputs
                .iter()
                .map(|(name, stored)| {
                    json!({
                        "name": name,
                        "value": stored.value,
                        "source": stored.source.as_str(),
                        "hard_errors": messages(&notes, name, |n| n.error && !n.cross_check),
                        "soft_warnings": messages(&notes, name, |n| !n.error),
                        "cross_checks": messages(&notes, name, |n| n.cross_check),
                    })
                })
                .collect();
            let (results, message, results_hash, hash_status) = if hard {
                // The results are not recomputed (spec 0002).
                (Value::Null, Value::Null, Value::Null, Value::Null)
            } else {
                match &run.outcome {
                    RunOutcome::Results { results, hash } => (
                        results.clone(),
                        Value::Null,
                        json!(hash),
                        json!(run.hash_status.as_str()),
                    ),
                    RunOutcome::NotAvailable(m) | RunOutcome::Failed(m) => {
                        (Value::Null, json!(m), Value::Null, Value::Null)
                    }
                }
            };
            options.push(json!({
                "name": option.name,
                "inputs": inputs,
                "errors": messages(&notes, "", |n| n.error && !n.cross_check && n.inputs.is_empty()),
                "warnings": messages(&notes, "", |n| !n.error && n.inputs.is_empty()),
                "results": results,
                "message": message,
                "results_hash": results_hash,
                "hash_status": hash_status,
            }));
        }
        Ok(json!({
            "profile": self.profile.name(),
            "engine_version": ENGINE_VERSION,
            "file_engine_version": self.design.engine_version,
            "errors": file_errors,
            "warnings": file_warnings,
            "options": options,
        }))
    }

    /// The saved `.vdesign` text, with each computable Option's results hash recomputed and
    /// written in. Refuses while any input has a hard error: the app may not save then.
    pub fn save(&mut self) -> Result<String, String> {
        let checked = crate::check(&self.design.to_text(), &self.profile);
        if !checked.report.is_ok() {
            return Err(format!(
                "the Design has hard errors: {}",
                checked.report.errors.join("; ")
            ));
        }
        for o in &mut self.design.options {
            o.results_hash = None;
        }
        stamp(&mut self.design);
        Ok(self.design.to_text())
    }

    fn check_new_name(&self, name: &str) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("an Option needs a non-empty string `name`".into());
        }
        if self.design.options.iter().any(|o| o.name == name) {
            return Err(format!("two Options are named '{name}'; names must differ"));
        }
        Ok(())
    }

    fn option_mut(&mut self, name: &str) -> Result<&mut DesignOption, String> {
        self.design
            .options
            .iter_mut()
            .find(|o| o.name == name)
            .ok_or_else(|| format!("there is no Option '{name}'"))
    }

    /// Re-derive the Design from its own text as [`fill`] does: write in the defaults of inputs
    /// the edits left out, drop the inputs that do not apply to the Cargo Containment, and keep
    /// the rest as typed. One code path, so a session and a saved file cannot differ.
    fn resync(&mut self) -> Result<(), String> {
        let filled = crate::fill(&self.design.to_text(), &self.profile);
        self.design = filled
            .design
            .ok_or_else(|| filled.report.errors.join("; "))?;
        Ok(())
    }

    /// The Design state every edit returns.
    fn state(&self) -> Result<Value, String> {
        Ok(json!({ "design": self.design.to_json() }))
    }

    /// Each Option's results-hash status against what this engine computes.
    fn hash_statuses(&self) -> Vec<Value> {
        run_design(&self.design)
            .iter()
            .map(|r| {
                let warning = match &r.hash_status {
                    HashStatus::Differs { stored, computed } => json!(hash_warning(
                        &r.name,
                        &self.design.engine_version,
                        stored,
                        computed
                    )),
                    _ => Value::Null,
                };
                json!({
                    "name": r.name,
                    "hash_status": r.hash_status.as_str(),
                    "hash_warning": warning,
                })
            })
            .collect()
    }
}

/// The messages of `notes` that are about `name` (or about no input, when `name` is empty) and
/// satisfy `keep`, each as the sentence [`crate::Report`] holds it.
fn messages<F>(notes: &[&Note], name: &str, keep: F) -> Vec<Value>
where
    F: Fn(&Note) -> bool,
{
    notes
        .iter()
        .filter(|n| {
            keep(n)
                && if name.is_empty() {
                    n.inputs.is_empty()
                } else {
                    n.inputs.iter().any(|i| i == name)
                }
        })
        .map(|n| json!(n.message))
        .collect()
}
