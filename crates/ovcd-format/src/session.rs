//! The Design session: the one interface between an app and the engine.
//!
//! It holds a Design and a Defaults Profile, and takes and returns strings and values. It reads no
//! file, clock or environment, so it is tested like the command-line tool, and an app shell does
//! the reading and writing. It adds no logic of its own for defaults, validation or results: it
//! calls [`fill`], [`check`], [`new_design`] and [`run_option`], so a Design gives here exactly what
//! `ovcd fill`, `ovcd validate` and `ovcd run` give.
//!
//! - [`input_schema`]: every input, for a Defaults Profile, as JSON a front end draws a form from.
//! - [`DesignSession::new_design`], [`DesignSession::open`]: start from the starter Design, or from
//!   a `.vdesign` text (left-out inputs are filled in, as `ovcd fill` does).
//! - Edits: [`set_input`](DesignSession::set_input), [`reset_input`](DesignSession::reset_input),
//!   [`set_formation`](DesignSession::set_formation), and add, duplicate, rename and delete an
//!   Option. Each returns the new Design state.
//! - [`DesignSession::evaluate`]: validate and run every Option.
//! - [`DesignSession::save`]: the `.vdesign` text, with each computed Option's results hash.
//! - [`DesignSession::load_profile`]: swap in a Defaults Profile, refusing one that loosens a hard
//!   range.
//!
//! A value the designer types is kept as typed even when it breaks a hard range, so a form shows
//! what was typed; [`evaluate`](DesignSession::evaluate) then reports the error, and the Option
//! is not run.

use crate::profile::Profile;
use crate::run::{HashStatus, RunOutcome, results_hash, run_design, run_option};
use crate::vdesign::{
    AppliesTo, CARGO_CONTAINMENTS, Design, DesignOption, ENGINE_VERSION, FORMATION_KINDS, Group,
    INPUTS, InputKind, KEYED_DEFAULTS, Report, Source, StoredInput, check, choice_label, fill,
    new_design,
};
use serde_json::{Map, Value, json};

/// The input schema for a Defaults Profile, as JSON. Built from the input table and the profile,
/// so it holds no fact of its own.
///
/// ```text
/// { "profile": name, "engine_version": v,
///   "groups": ["Required", ...],
///   "inputs": [ { "name", "group", "kind", "required", "applies_to": [containments],
///                 "unit", "nullable", "choices": [{ "value", "label", ... }],
///                 "default" | "default_by": { "by", "values" },
///                 "hard": { "gt"|"ge"|"lt"|"le" }, "soft": { "min", "max" }, "basis" } ] }
/// ```
///
/// `kind` is `number`, `bool`, `choice`, `cargo`, `formation`, `vessel_scheme` or `vessel_class`. The
/// `formation` input also holds `convoy`, the `abreast` and `in_line` ranges of a pushed convoy.
/// `vessel_scheme` and `vessel_class` are held only by an Option defined by a standard class; their
/// choices are the schemes, and the classes with the figures they set (`formation` is the Formation
/// kind the class gives, `length_max_m` and `beam_max_m` what picking it sets, `draught_limit_m` and
/// `air_draft_limit_m` the limits it fills, `reference_dwt_t` the deadweight shown beside the cargo).
pub fn input_schema(profile: &Profile) -> Value {
    let inputs: Vec<Value> = INPUTS.iter().map(|s| input_entry(s, profile)).collect();
    json!({
        "profile": profile.name(),
        "engine_version": ENGINE_VERSION,
        "groups": Group::ALL.iter().map(|g| g.label()).collect::<Vec<_>>(),
        "inputs": inputs,
    })
}

fn input_entry(spec: &crate::vdesign::InputSpec, profile: &Profile) -> Value {
    let name = spec.name;
    let mut e = Map::new();
    e.insert("name".into(), json!(name));
    e.insert("label".into(), json!(spec.label));
    e.insert("group".into(), json!(spec.group.label()));
    e.insert("required".into(), json!(spec.required));
    let applies: Vec<&str> = match spec.applies_to {
        AppliesTo::All => CARGO_CONTAINMENTS.iter().map(|(c, _)| *c).collect(),
        AppliesTo::Only(c) => vec![c],
    };
    e.insert("applies_to".into(), json!(applies));
    match spec.kind {
        InputKind::Number { nullable } => {
            e.insert("kind".into(), json!("number"));
            e.insert("nullable".into(), json!(nullable));
            add_ranges(&mut e, name, profile);
        }
        InputKind::Bool => {
            e.insert("kind".into(), json!("bool"));
        }
        InputKind::Choice(choices) => {
            e.insert("kind".into(), json!("choice"));
            let list: Vec<Value> = choices
                .iter()
                .map(|c| json!({ "value": c, "label": choice_label(c) }))
                .collect();
            e.insert("choices".into(), Value::Array(list));
        }
        InputKind::Cargo => {
            e.insert("kind".into(), json!("cargo"));
            let list: Vec<Value> = ovcd_core::cargo::CARGO_DATABASE
                .iter()
                .map(|(key, p)| {
                    json!({
                        "value": key,
                        "label": p.name,
                        "phase": p.phase,
                        "description": p.description,
                    })
                })
                .collect();
            e.insert("choices".into(), Value::Array(list));
        }
        InputKind::VesselScheme => {
            e.insert("kind".into(), json!("vessel_scheme"));
            e.insert("nullable".into(), json!(true));
            let list: Vec<Value> = ovcd_core::vessel_classes::schemes()
                .iter()
                .map(|s| json!({ "value": s, "label": s }))
                .collect();
            e.insert("choices".into(), Value::Array(list));
        }
        InputKind::VesselClass => {
            e.insert("kind".into(), json!("vessel_class"));
            e.insert("nullable".into(), json!(true));
            let list: Vec<Value> = ovcd_core::vessel_classes::CLASSES
                .iter()
                .filter(|c| c.pickable())
                .map(|c| {
                    let (abreast, in_line) = c.counts();
                    json!({
                        "value": c.id,
                        "label": c.id,
                        "scheme": c.family,
                        "formation": c.kind(),
                        "length_max_m": c.loa_max_m(),
                        "beam_max_m": c.beam_max_m(),
                        "draught_limit_m": c.draught_limit_m(),
                        "air_draft_limit_m": c.air_draft_limit_m(),
                        "reference_dwt_t": c.dwt_t.map(|(lo, hi)| [lo, hi]),
                        "abreast": abreast,
                        "in_line": in_line,
                        "confidence": c.confidence,
                    })
                })
                .collect();
            e.insert("choices".into(), Value::Array(list));
        }
        InputKind::Formation => {
            e.insert("kind".into(), json!("formation"));
            let kinds: Vec<Value> = FORMATION_KINDS
                .iter()
                .map(|k| json!({ "value": k, "label": choice_label(k) }))
                .collect();
            e.insert("choices".into(), Value::Array(kinds));
            let mut convoy = Map::new();
            for (key, profile_name) in [
                ("abreast", "formation_abreast"),
                ("in_line", "formation_in_line"),
            ] {
                let mut r = Map::new();
                add_ranges(&mut r, profile_name, profile);
                convoy.insert(key.into(), Value::Object(r));
            }
            e.insert("convoy".into(), Value::Object(convoy));
        }
    }
    if name == crate::vdesign::DEPTH_INPUT {
        e.insert("default_rule".into(), depth_rule(profile));
    }
    if !spec.required
        && !crate::class_input::UNSET_WHEN_ABSENT.contains(&name)
        && name != crate::vdesign::DEPTH_INPUT
    {
        match KEYED_DEFAULTS.iter().find(|(n, _, _)| *n == name) {
            Some((_, by, keys)) => {
                let values: Map<String, Value> = keys
                    .iter()
                    .filter_map(|k| {
                        profile
                            .default_by(name, by, k)
                            .map(|v| (k.to_string(), v.clone()))
                    })
                    .collect();
                e.insert("default_by".into(), json!({ "by": by, "values": values }));
            }
            None => {
                if let Some(d) = profile.default(name) {
                    e.insert("default".into(), d.clone());
                }
            }
        }
    }
    Value::Object(e)
}

/// The rule that gives a left-out moulded depth: the depth-to-beam ratio of the Cargo group times
/// the beam. The ratios and the basis come from the Defaults Profile; `groups` says which group
/// each Cargo is in (a liquid group exists in the profile but no Cargo is in it yet).
fn depth_rule(profile: &Profile) -> Value {
    let ratios: Map<String, Value> = crate::vdesign::DEPTH_GROUPS
        .iter()
        .filter_map(|g| {
            profile
                .default_by("depth_to_beam_ratio", "cargo_group", g)
                .map(|v| (g.to_string(), v.clone()))
        })
        .collect();
    let cargo_group: Map<String, Value> = ovcd_core::cargo::CARGO_DATABASE
        .iter()
        .filter_map(|(key, _)| {
            crate::vdesign::depth_group(key).map(|g| (key.to_string(), json!(g)))
        })
        .collect();
    let mut soft = Value::Null;
    if let Ok(Some(s)) = profile.soft("length_to_depth_ratio") {
        soft = json!(s.max);
    }
    json!({
        "form": "ratio x beam_m, to the millimetre",
        "by": "cargo_group",
        "ratios": ratios,
        "cargo_group": cargo_group,
        "length_to_depth_warning_above": soft,
        "basis": profile.basis("depth_to_beam_ratio"),
        "slenderness_basis": profile.basis("length_to_depth_ratio"),
    })
}

/// Unit, hard range, soft range and basis of a profile entry.
fn add_ranges(e: &mut Map<String, Value>, name: &str, profile: &Profile) {
    e.insert("unit".into(), json!(profile.unit(name)));
    if let Ok(h) = profile.hard(name) {
        let mut hard = Map::new();
        for (k, v) in [("gt", h.gt), ("ge", h.ge), ("lt", h.lt), ("le", h.le)] {
            if let Some(v) = v {
                hard.insert(k.into(), json!(v));
            }
        }
        e.insert("hard".into(), Value::Object(hard));
    }
    if let Ok(Some(s)) = profile.soft(name) {
        e.insert("soft".into(), json!({ "min": s.min, "max": s.max }));
    }
    e.insert("basis".into(), json!(profile.basis(name)));
}

/// What [`DesignSession::open`] found in the file.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenReport {
    /// What `ovcd fill` reports for the file: hard errors and warnings, including a warning for
    /// each input written in from the Defaults Profile.
    pub report: Report,
    /// Per Option: the file's stored results hash against this engine's.
    pub hash_status: Vec<(String, HashStatus)>,
}

/// What a message is about.
#[derive(Debug, Clone, PartialEq)]
pub enum Scope {
    /// One input: its range, type or choice.
    Input(String),
    /// A cross-check: the message names several inputs.
    CrossCheck(Vec<String>),
    /// The Option as a whole.
    Option,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// A hard error: the Option is not run.
    Error,
    /// A soft warning: accepted.
    Warning,
}

/// One validation message, as `ovcd validate` words it (without its `Option '<name>':` lead-in).
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub severity: Severity,
    pub scope: Scope,
    pub text: String,
}

/// One Option, validated and (when it has no hard error) run.
#[derive(Debug, Clone, PartialEq)]
pub struct OptionEvaluation {
    pub name: String,
    pub messages: Vec<Message>,
    /// `None` when the Option has a hard error and was not run.
    pub outcome: Option<RunOutcome>,
    /// The Vessel Class the Option sits in, with its reference deadweight; `None` for typed
    /// dimensions. See [`crate::class_input::summary`].
    pub vessel_class: Option<Value>,
}

/// The evaluation of a whole Design.
#[derive(Debug, Clone, PartialEq)]
pub struct Evaluation {
    pub options: Vec<OptionEvaluation>,
    /// Messages about the file as a whole, which no Option owns (an unknown top-level key, say).
    pub file_messages: Vec<Message>,
}

fn message_json(m: &Message) -> Value {
    let (scope, inputs) = match &m.scope {
        Scope::Input(n) => ("input", vec![n.clone()]),
        Scope::CrossCheck(ns) => ("cross_check", ns.clone()),
        Scope::Option => ("option", vec![]),
    };
    json!({
        "severity": match m.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        },
        "scope": scope,
        "inputs": inputs,
        "text": plain_text(&m.text),
    })
}

/// A message as the app shows it: each back-ticked input key is replaced by the input's label
/// (`formation` by Formation, `in_line` by In line). Other back-ticked names lose their ticks.
/// `ovcd validate` keeps the raw text; only the app's JSON is reworded.
fn plain_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('`') {
        let Some(len) = rest[start + 1..].find('`') else {
            break;
        };
        out.push_str(&rest[..start]);
        let key = &rest[start + 1..start + 1 + len];
        out.push_str(&plain_label(key));
        rest = &rest[start + len + 2..];
    }
    out.push_str(rest);
    out
}

fn plain_label(key: &str) -> String {
    match key {
        "abreast" | "formation.abreast" => return "Abreast".into(),
        "in_line" | "formation.in_line" => return "In line".into(),
        "kind" => return "Kind".into(),
        _ => {}
    }
    match INPUTS.iter().find(|s| s.name == key) {
        Some(s) => s.label.to_string(),
        None => key.to_string(),
    }
}

impl Evaluation {
    /// The evaluation as JSON. An Option's `results` is the object `ovcd run --json` gives.
    pub fn to_json(&self) -> Value {
        let options: Vec<Value> = self
            .options
            .iter()
            .map(|o| {
                let messages: Vec<Value> = o.messages.iter().map(message_json).collect();
                let outcome = match &o.outcome {
                    Some(RunOutcome::Results { results, hash }) => {
                        json!({ "status": "results", "results": results, "results_hash": hash })
                    }
                    Some(RunOutcome::NotAvailable(m)) => {
                        json!({ "status": "not_available", "message": m })
                    }
                    Some(RunOutcome::Failed(m)) => json!({ "status": "failed", "message": m }),
                    None => json!({ "status": "not_run" }),
                };
                json!({ "name": o.name, "messages": messages, "outcome": outcome,
                        "vessel_class": o.vessel_class })
            })
            .collect();
        let file_messages: Vec<Value> = self.file_messages.iter().map(message_json).collect();
        json!({ "options": options, "file_messages": file_messages })
    }
}

/// Name of the hidden-input memory entry that holds the last pushed-convoy counts of an Option.
const CONVOY_MEMORY: &str = "formation.convoy";

/// An open Design with its Defaults Profile. Every edit returns the new state.
#[derive(Debug, Clone)]
pub struct DesignSession {
    profile: Profile,
    /// The Design as the session holds it: every applicable input stored as used. A typed value
    /// that breaks a hard range stays in it, so a form shows what was typed.
    design: Design,
    /// Typed inputs an Option holds while its Cargo Containment does not use them: (Option, input,
    /// value). They stay out of the Design (and so out of a saved file, as `ovcd fill` leaves them
    /// out) and come back when the input applies again.
    hidden: Vec<(String, String, StoredInput)>,
}

impl DesignSession {
    /// A new Design: the starter Design `ovcd new` writes.
    pub fn new_design(profile: Profile) -> Result<DesignSession, String> {
        let design = new_design(&profile)?;
        Ok(DesignSession {
            profile,
            design,
            hidden: Vec::new(),
        })
    }

    /// Open a `.vdesign` text. Left-out inputs are written in from the profile, as `ovcd fill`
    /// does. A file with hard errors still opens, so the errors can be fixed; `Err` only when
    /// the text cannot be read as a Design (not JSON, no named Option).
    pub fn open(text: &str, profile: Profile) -> Result<(DesignSession, OpenReport), String> {
        let filled = fill(text, &profile);
        let mut design = filled.design.ok_or_else(|| {
            filled
                .report
                .errors
                .first()
                .cloned()
                .unwrap_or_else(|| "the file cannot be read as a Design".into())
        })?;
        if design.options.is_empty() {
            return Err("a Design needs at least one Option".into());
        }
        let raw: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        let raw_options = raw["options"].as_array().cloned().unwrap_or_default();
        if raw_options.len() != design.options.len() {
            let first = filled.report.errors.first().cloned().unwrap_or_default();
            return Err(format!("an Option cannot be read: {first}"));
        }
        // `fill` leaves out an input that breaks a hard range. Put each back as typed, so a form
        // shows it and an edit can fix it.
        for (option, raw_option) in design.options.iter_mut().zip(&raw_options) {
            let empty = Map::new();
            let raw_inputs = raw_option["inputs"].as_object().unwrap_or(&empty);
            for spec in INPUTS {
                if option.input(spec.name).is_none()
                    && let Some(raw_input) = raw_inputs.get(spec.name)
                    && let Ok(stored) = crate::vdesign::unmark(raw_input)
                {
                    option.inputs.push((spec.name.into(), stored));
                }
            }
            sort_inputs(option);
        }
        // The file's own hashes, before `fill` stamped any left out.
        let mut stored = design.clone();
        for (o, raw_option) in stored.options.iter_mut().zip(&raw_options) {
            o.results_hash = raw_option["results_hash"].as_str().map(String::from);
        }
        let hash_status = run_design(&stored)
            .into_iter()
            .map(|r| (r.name, r.hash_status))
            .collect();
        let mut session = DesignSession {
            profile,
            design,
            hidden: Vec::new(),
        };
        session.normalise();
        Ok((
            session,
            OpenReport {
                report: filled.report,
                hash_status,
            },
        ))
    }

    /// The Design as it is now.
    pub fn design(&self) -> &Design {
        &self.design
    }

    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    /// The input schema for the session's Defaults Profile.
    pub fn input_schema(&self) -> Value {
        input_schema(&self.profile)
    }

    /// Set an input to a typed value. The value is kept even when it breaks a hard range;
    /// [`evaluate`](Self::evaluate) reports it.
    pub fn set_input(&mut self, option: &str, name: &str, value: Value) -> Result<&Design, String> {
        let spec = INPUTS
            .iter()
            .find(|s| s.name == name)
            .ok_or_else(|| format!("unknown input `{name}`"))?;
        let i = self.index_of(option)?;
        if let AppliesTo::Only(c) = spec.applies_to
            && let Some(current) = containment_of(&self.design.options[i])
            && current != c
        {
            return Err(format!(
                "`{name}` is not used for a {current} Cargo Containment"
            ));
        }
        match name {
            "vessel_class" => self.pick_class(i, &value)?,
            "vessel_scheme" => self.pick_scheme(i, value),
            _ => self.put(i, name, value),
        }
        self.normalise();
        Ok(&self.design)
    }

    /// Define the Option by this class (an id, or null for typed dimensions): length, beam and
    /// Formation come from the class, marked `class`; the scheme is the class's. Depth stays typed.
    /// The limits the class fills follow when the Design is normalised.
    fn pick_class(&mut self, i: usize, value: &Value) -> Result<(), String> {
        let Some(id) = value.as_str() else {
            self.pick_scheme(i, Value::Null);
            return Ok(());
        };
        let class = ovcd_core::vessel_classes::find(id)
            .ok_or_else(|| format!("`{id}` is not a Vessel Class"))?;
        if !class.pickable() {
            return Err(format!(
                "`{}` cannot define a vessel: it gives no length or beam, or its Formation ({}) \
                 is not one a Design has",
                class.id, class.formation
            ));
        }
        let option = &mut self.design.options[i];
        option.results_hash = None;
        let mut write = |name: &str, value: Value, source: Source| {
            let stored = StoredInput { value, source };
            match option.inputs.iter_mut().find(|(n, _)| n == name) {
                Some((_, slot)) => *slot = stored,
                None => option.inputs.push((name.into(), stored)),
            }
        };
        write("vessel_scheme", json!(class.family), Source::Typed);
        write("vessel_class", json!(class.id), Source::Class);
        for (name, v) in crate::class_input::class_values(class) {
            write(name, v, Source::Class);
        }
        // A limit the class fills is refilled; one the designer typed stays.
        for (name, _) in crate::class_input::class_limits(class) {
            if option
                .input(name)
                .is_some_and(|s| s.source != Source::Typed)
            {
                option.inputs.retain(|(n, _)| n != name);
            }
        }
        // The beam changed, so a depth still on its default is worked out again.
        self.drop_dependent_defaults(i, "beam_m");
        // The convoy counts of the class replace those that were waiting for a return.
        self.hidden
            .retain(|(o, n, _)| !(o == &self.design.options[i].name && n == CONVOY_MEMORY));
        Ok(())
    }

    /// Pick the scheme, or null to go back to typed dimensions: what came from a class is then
    /// typed, and the Option holds no class. A new scheme drops the old class, and the vessel is
    /// placed again.
    fn pick_scheme(&mut self, i: usize, value: Value) {
        let option = &mut self.design.options[i];
        option.results_hash = None;
        option.inputs.retain(|(n, _)| n != "vessel_class");
        if value.is_null() {
            option.inputs.retain(|(n, _)| n != "vessel_scheme");
            for (_, s) in option.inputs.iter_mut() {
                if s.source == Source::Class {
                    s.source = Source::Typed;
                }
            }
        } else {
            self.put(i, "vessel_scheme", value);
        }
    }

    /// Set the Formation: its kind and, for a pushed convoy, the barges abreast and in line.
    /// Any other kind stores only the kind. The last convoy counts wait in the hidden-input
    /// memory and come back when the kind returns to a pushed convoy and no counts are given.
    pub fn set_formation(
        &mut self,
        option: &str,
        kind: &str,
        abreast: Option<u64>,
        in_line: Option<u64>,
    ) -> Result<&Design, String> {
        let i = self.index_of(option)?;
        let current = self.design.options[i]
            .input("formation")
            .map(|s| s.value.clone());
        let mut value = json!({ "kind": kind });
        if kind == "pushed_convoy" {
            let remembered = self
                .hidden
                .iter()
                .position(|(o, n, _)| o == option && n == CONVOY_MEMORY);
            let back = remembered.map(|h| self.hidden.remove(h).2.value);
            let count = |key: &str, given: Option<u64>| {
                given.or_else(|| back.as_ref().and_then(|b| b[key].as_u64()))
            };
            if let Some(a) = count("abreast", abreast) {
                value["abreast"] = json!(a);
            }
            if let Some(l) = count("in_line", in_line) {
                value["in_line"] = json!(l);
            }
        } else {
            // Remember the counts of the convoy being left (or the ones just sent).
            let mut memory = json!({});
            let was_convoy = current.as_ref().filter(|v| v["kind"] == "pushed_convoy");
            for (key, given) in [("abreast", abreast), ("in_line", in_line)] {
                let n = given.or_else(|| was_convoy.and_then(|v| v[key].as_u64()));
                if let Some(n) = n {
                    memory[key] = json!(n);
                }
            }
            if memory.as_object().is_some_and(|m| !m.is_empty()) {
                let entry = (
                    option.to_string(),
                    CONVOY_MEMORY.to_string(),
                    StoredInput {
                        value: memory,
                        source: Source::Typed,
                    },
                );
                match self
                    .hidden
                    .iter()
                    .position(|(o, n, _)| o == option && n == CONVOY_MEMORY)
                {
                    Some(h) => self.hidden[h] = entry,
                    None => self.hidden.push(entry),
                }
            }
        }
        self.put(i, "formation", value);
        self.normalise();
        Ok(&self.design)
    }

    /// Put an input back to its default from the Defaults Profile (marked default). A required
    /// input has no default.
    pub fn reset_input(&mut self, option: &str, name: &str) -> Result<&Design, String> {
        let spec = INPUTS
            .iter()
            .find(|s| s.name == name)
            .ok_or_else(|| format!("unknown input `{name}`"))?;
        if spec.required {
            return Err(format!("`{name}` is required and has no default"));
        }
        let i = self.index_of(option)?;
        self.design.options[i].inputs.retain(|(n, _)| n != name);
        self.design.options[i].results_hash = None;
        self.drop_dependent_defaults(i, name);
        self.normalise();
        Ok(&self.design)
    }

    /// Add an Option: the starter Option, with the given name or the next free "Option N".
    /// Returns the new Option's name.
    pub fn add_option(&mut self, name: Option<&str>) -> Result<String, String> {
        let name = match name {
            Some(n) => self.free_name(n)?,
            None => (self.design.options.len() + 1..)
                .map(|n| format!("Option {n}"))
                .find(|n| self.design.options.iter().all(|o| &o.name != n))
                .expect("an endless range"),
        };
        let mut option = new_design(&self.profile)?
            .options
            .pop()
            .expect("a starter Design has an Option");
        option.name = name.clone();
        self.design.options.push(option);
        self.normalise();
        Ok(name)
    }

    /// Duplicate an Option under the given name, or "<name> copy". Returns the new name.
    pub fn duplicate_option(
        &mut self,
        option: &str,
        new_name: Option<&str>,
    ) -> Result<String, String> {
        let i = self.index_of(option)?;
        let name = match new_name {
            Some(n) => self.free_name(n)?,
            None => {
                let base = format!("{option} copy");
                (1..)
                    .map(|n| {
                        if n == 1 {
                            base.clone()
                        } else {
                            format!("{base} {n}")
                        }
                    })
                    .find(|n| self.design.options.iter().all(|o| &o.name != n))
                    .expect("an endless range")
            }
        };
        let copied: Vec<(String, String, StoredInput)> = self
            .hidden
            .iter()
            .filter(|(owner, _, _)| owner == option)
            .map(|(_, input, v)| (name.clone(), input.clone(), v.clone()))
            .collect();
        self.hidden.extend(copied);
        let mut copy = self.design.options[i].clone();
        copy.name = name.clone();
        self.design.options.insert(i + 1, copy);
        Ok(name)
    }

    /// Rename an Option. The new name must be non-empty and differ from every other Option's.
    pub fn rename_option(&mut self, option: &str, new_name: &str) -> Result<&Design, String> {
        let i = self.index_of(option)?;
        let new_name = new_name.trim();
        if new_name != option {
            self.free_name(new_name)?;
        }
        self.design.options[i].name = new_name.to_string();
        for (owner, _, _) in &mut self.hidden {
            if owner == option {
                *owner = new_name.to_string();
            }
        }
        Ok(&self.design)
    }

    /// Delete an Option. The last Option of a Design cannot be deleted.
    pub fn delete_option(&mut self, option: &str) -> Result<&Design, String> {
        let i = self.index_of(option)?;
        if self.design.options.len() == 1 {
            return Err(
                "a Design needs at least one Option; the last one cannot be deleted".into(),
            );
        }
        self.design.options.remove(i);
        self.hidden.retain(|(owner, _, _)| owner != option);
        Ok(&self.design)
    }

    /// Load a Defaults Profile from its text. A profile that loosens a hard range, or is missing
    /// an entry, is refused and the session keeps its profile. Stored inputs never change; only
    /// inputs the Design lacks take the new profile's defaults.
    pub fn load_profile(&mut self, text: &str) -> Result<(), String> {
        self.profile = Profile::from_json(text)?;
        self.normalise();
        Ok(())
    }

    /// Validate and run every Option, as `ovcd validate` and `ovcd run` do.
    pub fn evaluate(&self) -> Evaluation {
        let checked = check(&self.design.to_text(), &self.profile);
        let names: Vec<&str> = self
            .design
            .options
            .iter()
            .map(|o| o.name.as_str())
            .collect();
        let (by_option, file_messages) = attach(&checked.report, &names);
        let options = by_option
            .into_iter()
            .enumerate()
            .map(|(i, messages)| {
                let has_error = messages.iter().any(|m| m.severity == Severity::Error);
                let outcome = match (&checked.design, has_error) {
                    (Some(d), false) => d.options.get(i).map(run_option),
                    _ => None,
                };
                OptionEvaluation {
                    name: names[i].to_string(),
                    messages,
                    outcome,
                    vessel_class: checked
                        .design
                        .as_ref()
                        .and_then(|d| d.options.get(i))
                        .and_then(crate::class_input::summary),
                }
            })
            .collect();
        Evaluation {
            options,
            file_messages,
        }
    }

    /// The `.vdesign` text, with each computed Option's results hash recomputed and the current
    /// engine version. Refused while the Design has a hard error, so a saved file always passes
    /// `ovcd validate`.
    pub fn save(&self) -> Result<String, String> {
        let checked = check(&self.design.to_text(), &self.profile);
        if !checked.report.is_ok() {
            return Err(format!(
                "the Design has {} hard error(s); fix them before saving: {}",
                checked.report.errors.len(),
                checked.report.errors.join("; ")
            ));
        }
        let mut design = checked.design.ok_or("the Design cannot be read")?;
        design.engine_version = ENGINE_VERSION.into();
        for o in &mut design.options {
            o.results_hash = match run_option(o) {
                RunOutcome::Results { results, .. } => Some(results_hash(&results)),
                _ => o.results_hash.take(),
            };
        }
        Ok(design.to_text())
    }

    fn index_of(&self, option: &str) -> Result<usize, String> {
        self.design
            .options
            .iter()
            .position(|o| o.name == option)
            .ok_or_else(|| format!("no Option named '{option}'"))
    }

    /// A trimmed, non-empty name no Option has.
    fn free_name(&self, name: &str) -> Result<String, String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("an Option needs a name".into());
        }
        if self.design.options.iter().any(|o| o.name == name) {
            return Err(format!(
                "an Option named '{name}' exists; names must differ"
            ));
        }
        Ok(name.to_string())
    }

    /// Store a typed input, and drop what its change leaves stale.
    fn put(&mut self, i: usize, name: &str, value: Value) {
        let option = &mut self.design.options[i];
        option.results_hash = None;
        let stored = StoredInput {
            value,
            source: Source::Typed,
        };
        match option.inputs.iter_mut().find(|(n, _)| n == name) {
            Some((_, slot)) => *slot = stored,
            None => option.inputs.push((name.into(), stored)),
        }
        self.drop_dependent_defaults(i, name);
    }

    /// A default chosen by another input follows that input: when it changes, an input still on
    /// its default is dropped, so the next fill chooses it again.
    fn drop_dependent_defaults(&mut self, i: usize, changed: &str) {
        // The default depth is the Cargo's ratio times the beam: when either changes, a depth still
        // on its default is dropped, so the next fill works it out again.
        if matches!(changed, "cargo" | "beam_m") {
            let option = &mut self.design.options[i];
            if option
                .input(crate::vdesign::DEPTH_INPUT)
                .is_some_and(|s| s.source == Source::Default)
            {
                option
                    .inputs
                    .retain(|(n, _)| n != crate::vdesign::DEPTH_INPUT);
            }
        }
        let mut changed = vec![changed.to_string()];
        while let Some(by_name) = changed.pop() {
            let by_name = if by_name == "cargo" {
                "cargo_phase".to_string()
            } else {
                by_name
            };
            for (input, by, _) in KEYED_DEFAULTS {
                if *by == by_name {
                    let option = &mut self.design.options[i];
                    let on_default = option
                        .input(input)
                        .is_some_and(|s| s.source == Source::Default);
                    if on_default {
                        option.inputs.retain(|(n, _)| n != input);
                        changed.push(input.to_string());
                    }
                }
            }
        }
    }

    /// Write in the inputs the Design lacks (with `fill`), and drop those its Cargo Containment
    /// does not use. A hard-range breach stays as typed.
    fn normalise(&mut self) {
        let filled = fill(&self.design.to_text(), &self.profile);
        let Some(filled_design) = filled.design else {
            return;
        };
        for option in &mut self.design.options {
            let Some(from_fill) = filled_design.options.iter().find(|o| o.name == option.name)
            else {
                continue;
            };
            // The filled Option knows the containment, even when it was a default left out.
            let applicable = containment_of(from_fill)
                .filter(|c| CARGO_CONTAINMENTS.iter().any(|(known, _)| known == c));
            let mut inputs: Vec<(String, StoredInput)> = Vec::new();
            for spec in INPUTS {
                let applies = match (spec.applies_to, applicable) {
                    (AppliesTo::Only(c), Some(current)) => c == current,
                    _ => true,
                };
                let held = self
                    .hidden
                    .iter()
                    .position(|(o, n, _)| *o == option.name && n == spec.name);
                if !applies {
                    // Hide a typed value rather than lose it.
                    if let Some(stored) = option.input(spec.name)
                        && stored.source == Source::Typed
                    {
                        let entry = (option.name.clone(), spec.name.to_string(), stored.clone());
                        match held {
                            Some(h) => self.hidden[h] = entry,
                            None => self.hidden.push(entry),
                        }
                    }
                    continue;
                }
                let mut kept = from_fill
                    .input(spec.name)
                    .or_else(|| option.input(spec.name))
                    .cloned();
                if let Some(h) = held {
                    let (_, _, back) = self.hidden.remove(h);
                    // A value typed since then wins; a default gives way to what was typed.
                    if kept.as_ref().is_none_or(|k| k.source == Source::Default) {
                        kept = Some(back);
                    }
                }
                if let Some(stored) = kept {
                    inputs.push((spec.name.into(), stored));
                }
            }
            option.inputs = inputs;
            if option.results_hash.is_none() {
                option.results_hash = from_fill.results_hash.clone();
            }
        }
        if filled_design.defaults_profile.is_some() {
            self.design.defaults_profile = filled_design.defaults_profile;
        }
    }
}

fn sort_inputs(option: &mut DesignOption) {
    option.inputs.sort_by_key(|(n, _)| {
        INPUTS
            .iter()
            .position(|s| s.name == n)
            .unwrap_or(usize::MAX)
    });
}

fn containment_of(option: &DesignOption) -> Option<&str> {
    option.input("cargo_containment")?.value.as_str()
}

/// Sort a report's messages into the Options they belong to. The inputs a message is about come
/// from the report as data, never from the wording of the text.
fn attach(report: &Report, names: &[&str]) -> (Vec<Vec<Message>>, Vec<Message>) {
    let mut file_level: Vec<Message> = Vec::new();
    let mut by_option: Vec<Vec<Message>> = names.iter().map(|_| Vec::new()).collect();
    let tagged = report
        .errors
        .iter()
        .zip(&report.error_inputs)
        .map(|(t, n)| (Severity::Error, t, n))
        .chain(
            report
                .warnings
                .iter()
                .zip(&report.warning_inputs)
                .map(|(t, n)| (Severity::Warning, t, n)),
        );
    for (severity, text, inputs) in tagged {
        // The longest name first, so "A" does not claim a message of "A: B".
        let owner = (0..names.len())
            .filter(|&i| text.starts_with(&format!("Option '{}': ", names[i])))
            .max_by_key(|&i| names[i].len());
        let scope = match inputs.as_slice() {
            [] => Scope::Option,
            [one] => Scope::Input(one.clone()),
            many => Scope::CrossCheck(many.to_vec()),
        };
        match owner {
            Some(i) => {
                let body = &text[format!("Option '{}': ", names[i]).len()..];
                by_option[i].push(Message {
                    severity,
                    scope,
                    text: body.to_string(),
                });
            }
            // No Option owns it: it is about the file.
            None => file_level.push(Message {
                severity,
                scope,
                text: text.clone(),
            }),
        }
    }
    (by_option, file_level)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_message_attaches_to_the_inputs_the_report_names_not_to_its_wording() {
        let mut report = Report::default();
        // The wording names `beam_m` and `cargo`, but the data says the message is about length.
        report.error(
            "Option 'A': reworded text about `beam_m` and `cargo`".into(),
            &["length_overall_m"],
        );
        report.error("Option 'A': another".into(), &["beam_m", "moulded_depth_m"]);
        report.warning("Option 'A': about the whole Option".into(), &[]);
        let (got, file) = attach(&report, &["A"]);
        assert_eq!(got[0][0].scope, Scope::Input("length_overall_m".into()));
        assert_eq!(
            got[0][1].scope,
            Scope::CrossCheck(vec!["beam_m".into(), "moulded_depth_m".into()])
        );
        assert_eq!(got[0][2].scope, Scope::Option);
        assert!(file.is_empty());
    }

    #[test]
    fn a_message_no_option_owns_is_a_file_message() {
        let mut report = Report::default();
        report.warning("unknown top-level key `extra` is ignored".into(), &[]);
        report.error("Option 'A': about A".into(), &["beam_m"]);
        let (by_option, file) = attach(&report, &["A"]);
        assert_eq!(by_option[0].len(), 1);
        assert_eq!(file.len(), 1);
        assert_eq!(file[0].severity, Severity::Warning);
        assert_eq!(file[0].text, "unknown top-level key `extra` is ignored");
    }
}
