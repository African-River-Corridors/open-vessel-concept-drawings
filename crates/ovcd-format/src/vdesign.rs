//! The `.vdesign` file: a Design holding a named list of Options and the engine version, as
//! versioned JSON, always SI (ADR 0007).
//!
//! Every input is stored **as used**, with a marker saying where it came from:
//!
//! ```json
//! "block_coefficient": { "value": 0.85, "source": "default" }
//! ```
//!
//! `source` is `typed` (the designer gave it) or `default` (the Defaults Profile gave it). A bare
//! value in a hand-written file counts as typed. A stored input is never replaced by a profile
//! value, so a later profile change never changes a saved file.
//!
//! Validation lives here only. The core copies Python exactly and adds no checks.

use crate::profile::Profile;
use serde_json::{Map, Value, json};

/// The only `.vdesign` format version so far.
pub const FORMAT_VERSION: u64 = 1;

/// The engine version written into every new Design. One version covers every `ovcd-*` crate.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Cargo Containments, and the Cargo phases each can carry (as in the Python `simulate`).
pub const CARGO_CONTAINMENTS: &[(&str, &str)] = &[
    ("box_hold", "dry_bulk"),
    ("sloped_hopper_hold", "dry_bulk"),
    ("type_c_tanks", "liquid_gas"),
];

/// Cargo phases, as the Python cargo database names them.
pub const CARGO_PHASES: &[&str] = &["dry_bulk", "liquid_gas"];

/// Inputs whose default is chosen by another value: `(input, by, keys)`. The Defaults Profile
/// must give `default_by_<by>[key]` for every key, because each is a lookup the schema can make.
pub const KEYED_DEFAULTS: &[(&str, &str, &[&str])] = &[
    ("cargo_containment", "cargo_phase", CARGO_PHASES),
    (
        "lightship_factor_t_per_m3",
        "cargo_containment",
        &["box_hold"],
    ),
    ("fill_ratio", "ruleset", RULESETS),
];

/// Hull Forms that exist today.
pub const HULL_FORMS: &[&str] = &["raked_box"];

/// The deepest hull the gas depth search (`optimise_depth`) tries, in m. It is fixed in the core
/// (`DEFAULT_DEPTHS_M`, 3.5 to 7.0 m).
pub const DEPTH_SEARCH_MAX_M: f64 = 7.0;

/// Gas tank rulesets.
pub const RULESETS: &[&str] = &["IGC_2G", "IGC_2G_EXACT", "ADN_G"];

/// Formation kinds. Only the first two are computed in spec 0001.
pub const FORMATION_KINDS: &[&str] = &["self_propelled", "pushed_barge", "pushed_convoy"];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputKind {
    /// A float. `nullable` inputs take `null` for "none".
    Number {
        nullable: bool,
    },
    Bool,
    /// One of a fixed list of names.
    Choice(&'static [&'static str]),
    /// A key of the built-in cargo database.
    Cargo,
    /// `{ "kind": ..., "abreast": n, "in_line": n }`.
    Formation,
}

/// Which Cargo Containments an input is used for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppliesTo {
    All,
    Only(&'static str),
}

/// One input of an Option.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InputSpec {
    pub name: &'static str,
    pub kind: InputKind,
    pub required: bool,
    pub applies_to: AppliesTo,
}

const fn spec(name: &'static str, kind: InputKind, required: bool) -> InputSpec {
    InputSpec {
        name,
        kind,
        required,
        applies_to: AppliesTo::All,
    }
}

const fn only(name: &'static str, kind: InputKind, containment: &'static str) -> InputSpec {
    InputSpec {
        name,
        kind,
        required: false,
        applies_to: AppliesTo::Only(containment),
    }
}

const NUM: InputKind = InputKind::Number { nullable: false };
const NUM_OR_NONE: InputKind = InputKind::Number { nullable: true };

/// Every input of an Option, in the order defaults are resolved (a default may depend on an
/// input above it). Names follow `GLOSSARY.md`.
pub const INPUTS: &[InputSpec] = &[
    spec("cargo", InputKind::Cargo, true),
    spec("hull_form", InputKind::Choice(HULL_FORMS), true),
    spec("formation", InputKind::Formation, true),
    spec("length_overall_m", NUM, true),
    spec("beam_m", NUM, true),
    spec("moulded_depth_m", NUM, true),
    spec(
        "cargo_containment",
        InputKind::Choice(&["box_hold", "sloped_hopper_hold", "type_c_tanks"]),
        false,
    ),
    spec("block_coefficient", NUM, false),
    spec("min_freeboard_m", NUM, false),
    only("lightship_factor_t_per_m3", NUM, "box_hold"),
    only("ruleset", InputKind::Choice(RULESETS), "type_c_tanks"),
    only("fill_ratio", NUM, "type_c_tanks"),
    only("adn_tank_cap_m3", NUM, "type_c_tanks"),
    only("optimise_depth", InputKind::Bool, "type_c_tanks"),
    spec("propulsive_efficiency", NUM, false),
    spec("max_speed_kn", NUM, false),
    spec("air_draft_limit_m", NUM_OR_NONE, false),
    spec("draught_limit_m", NUM_OR_NONE, false),
    spec("channel_depth_m", NUM_OR_NONE, false),
    spec("channel_under_keel_clearance_m", NUM, false),
    spec("channel_bottom_width_m", NUM_OR_NONE, false),
    spec("channel_bank_slope", NUM, false),
    spec("channel_current_kn", NUM, false),
];

/// Where a stored input came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// The designer gave it.
    Typed,
    /// The Defaults Profile gave it.
    Default,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Typed => "typed",
            Source::Default => "default",
        }
    }
}

/// One input as used, with its marker.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredInput {
    pub value: Value,
    pub source: Source,
}

/// One complete set of inputs within a Design (glossary: Option).
#[derive(Debug, Clone, PartialEq)]
pub struct DesignOption {
    pub name: String,
    /// Inputs in `INPUTS` order. Inputs that do not apply to the Cargo Containment are left out.
    pub inputs: Vec<(String, StoredInput)>,
}

impl DesignOption {
    pub fn input(&self, name: &str) -> Option<&StoredInput> {
        self.inputs.iter().find(|(n, _)| n == name).map(|(_, i)| i)
    }
}

/// One study saved as a single file (glossary: Design).
#[derive(Debug, Clone, PartialEq)]
pub struct Design {
    pub engine_version: String,
    /// The name of the Defaults Profile the defaults came from, for the record.
    pub defaults_profile: Option<String>,
    pub options: Vec<DesignOption>,
}

impl Design {
    /// The file's JSON.
    pub fn to_json(&self) -> Value {
        let options: Vec<Value> = self
            .options
            .iter()
            .map(|o| {
                let inputs: Map<String, Value> = o
                    .inputs
                    .iter()
                    .map(|(n, i)| {
                        (
                            n.clone(),
                            json!({ "value": i.value, "source": i.source.as_str() }),
                        )
                    })
                    .collect();
                json!({ "name": o.name, "inputs": inputs })
            })
            .collect();
        let mut root = json!({
            "format": "vdesign",
            "format_version": FORMAT_VERSION,
            "engine_version": self.engine_version,
            "units": "SI",
            "options": options,
        });
        if let Some(p) = &self.defaults_profile {
            root["defaults_profile"] = json!(p);
        }
        root
    }

    /// The file's text: pretty JSON with a final newline.
    pub fn to_text(&self) -> String {
        let mut text = serde_json::to_string_pretty(&self.to_json()).expect("JSON");
        text.push('\n');
        text
    }
}

/// Hard errors and soft warnings found in a Design, each a full sentence for a person.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Report {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl Report {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// The result of reading a `.vdesign` file: the Design as used (when it could be read) and
/// what validation found.
#[derive(Debug, Clone, PartialEq)]
pub struct Checked {
    pub design: Option<Design>,
    pub report: Report,
}

/// A starter Design: one Option with example required inputs (typed, for the designer to edit)
/// and every other input from the profile, marked as defaults.
pub fn new_design(profile: &Profile) -> Design {
    let typed = json!({
        "cargo": "bauxite",
        "hull_form": "raked_box",
        "formation": { "kind": "pushed_barge" },
        "length_overall_m": 76.5,
        "beam_m": 11.4,
        "moulded_depth_m": 3.5,
    });
    let mut report = Report::default();
    let option = read_option("Option 1", typed.as_object().unwrap(), profile, &mut report);
    debug_assert!(report.is_ok(), "{report:?}");
    Design {
        engine_version: ENGINE_VERSION.into(),
        defaults_profile: Some(profile.name().into()),
        options: vec![option],
    }
}

/// Read and validate a `.vdesign` file's text, without running it.
pub fn check(text: &str, profile: &Profile) -> Checked {
    let mut report = Report::default();
    let root: Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            report.errors.push(format!("the file is not JSON: {e}"));
            return Checked {
                design: None,
                report,
            };
        }
    };
    let design = read_design(&root, profile, &mut report);
    Checked { design, report }
}

fn read_design(root: &Value, profile: &Profile, report: &mut Report) -> Option<Design> {
    let Some(root) = root.as_object() else {
        report
            .errors
            .push("the file must hold a JSON object".into());
        return None;
    };
    if root.get("format").and_then(Value::as_str) != Some("vdesign") {
        report
            .errors
            .push("the file needs `\"format\": \"vdesign\"`".into());
    }
    match root.get("format_version").and_then(Value::as_u64) {
        Some(FORMAT_VERSION) => {}
        Some(v) => report.errors.push(format!(
            "format_version {v} is not supported; this tool reads version {FORMAT_VERSION}"
        )),
        None => report
            .errors
            .push("the file needs an integer `format_version`".into()),
    }
    let engine_version = match root.get("engine_version").and_then(Value::as_str) {
        Some(v) => v.to_string(),
        None => {
            report
                .errors
                .push("the file needs a string `engine_version`".into());
            String::new()
        }
    };
    if let Some(units) = root.get("units")
        && units != "SI"
    {
        report
            .errors
            .push(format!("`units` must be \"SI\", not {units}"));
    }
    let defaults_profile = root
        .get("defaults_profile")
        .and_then(Value::as_str)
        .map(String::from);
    for key in root.keys() {
        let known = [
            "format",
            "format_version",
            "engine_version",
            "units",
            "defaults_profile",
            "options",
        ];
        if !known.contains(&key.as_str()) {
            report
                .warnings
                .push(format!("unknown top-level key `{key}` is ignored"));
        }
    }
    let Some(raw_options) = root.get("options").and_then(Value::as_array) else {
        report
            .errors
            .push("the file needs an `options` list".into());
        return None;
    };
    if raw_options.is_empty() {
        report
            .errors
            .push("a Design needs at least one Option".into());
    }
    let mut options = Vec::new();
    for (i, raw) in raw_options.iter().enumerate() {
        let name = raw
            .get("name")
            .and_then(Value::as_str)
            .filter(|n| !n.trim().is_empty());
        let Some(name) = name else {
            report
                .errors
                .push(format!("Option {} needs a non-empty string `name`", i + 1));
            continue;
        };
        if options.iter().any(|o: &DesignOption| o.name == name) {
            report
                .errors
                .push(format!("two Options are named '{name}'; names must differ"));
        }
        let Some(inputs) = raw.get("inputs").and_then(Value::as_object) else {
            report
                .errors
                .push(format!("Option '{name}': needs an `inputs` object"));
            continue;
        };
        for key in raw.as_object().into_iter().flat_map(|o| o.keys()) {
            if !["name", "inputs", "results_hash"].contains(&key.as_str()) {
                report
                    .warnings
                    .push(format!("Option '{name}': unknown key `{key}` is ignored"));
            }
        }
        options.push(read_option(name, inputs, profile, report));
    }
    Some(Design {
        engine_version,
        defaults_profile,
        options,
    })
}

/// Split a stored input into its value and marker. A bare value counts as typed.
fn unmark(raw: &Value) -> Result<StoredInput, String> {
    if let Some(o) = raw.as_object()
        && o.contains_key("source")
    {
        let source = match o.get("source").and_then(Value::as_str) {
            Some("typed") => Source::Typed,
            Some("default") => Source::Default,
            _ => return Err("`source` must be \"typed\" or \"default\"".into()),
        };
        let value = o.get("value").ok_or("a marked input needs a `value`")?;
        if o.len() != 2 {
            return Err("a marked input holds only `value` and `source`".into());
        }
        return Ok(StoredInput {
            value: value.clone(),
            source,
        });
    }
    Ok(StoredInput {
        value: raw.clone(),
        source: Source::Typed,
    })
}

fn read_option(
    name: &str,
    raw: &Map<String, Value>,
    profile: &Profile,
    report: &mut Report,
) -> DesignOption {
    let at = |msg: String| format!("Option '{name}': {msg}");
    for key in raw.keys() {
        if !INPUTS.iter().any(|s| s.name == key) {
            report.errors.push(at(format!("unknown input `{key}`")));
        }
    }
    let mut inputs: Vec<(String, StoredInput)> = Vec::new();
    let value_of = |inputs: &[(String, StoredInput)], n: &str| -> Option<Value> {
        inputs
            .iter()
            .find(|(k, _)| k == n)
            .map(|(_, i)| i.value.clone())
    };
    for spec in INPUTS {
        let containment = value_of(&inputs, "cargo_containment");
        let applies = match spec.applies_to {
            AppliesTo::All => true,
            AppliesTo::Only(c) => containment.as_ref().and_then(Value::as_str) == Some(c),
        };
        if let Some(raw_input) = raw.get(spec.name) {
            let stored = match unmark(raw_input) {
                Ok(s) => s,
                Err(e) => {
                    report.errors.push(at(format!("`{}`: {e}", spec.name)));
                    continue;
                }
            };
            if !applies {
                let c = containment
                    .as_ref()
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                report.warnings.push(at(format!(
                    "`{}` is not used for a {c} Cargo Containment; ignored",
                    spec.name
                )));
                continue;
            }
            match check_value(spec, &stored.value, profile) {
                Ok((value, warnings)) => {
                    report.warnings.extend(warnings.into_iter().map(&at));
                    inputs.push((
                        spec.name.into(),
                        StoredInput {
                            value,
                            source: stored.source,
                        },
                    ));
                }
                Err(e) => report.errors.push(at(e)),
            }
            continue;
        }
        if spec.required {
            let msg = if spec.name == "moulded_depth_m" {
                "missing required input `moulded_depth_m`: moulded depth is required for now; \
                 a default rule by Cargo is planned"
                    .to_string()
            } else {
                format!("missing required input `{}`", spec.name)
            };
            report.errors.push(at(msg));
            continue;
        }
        if !applies {
            continue;
        }
        let default = match default_for(spec.name, &inputs, profile) {
            Ok(Some(d)) => d,
            // The input it depends on is missing or wrong; that error is already reported.
            Ok(None) => continue,
            Err(e) => {
                report.errors.push(at(e));
                continue;
            }
        };
        report.warnings.push(at(format!(
            "`{}` is not stored in the file; using the Defaults Profile value {}",
            spec.name,
            show(&default)
        )));
        inputs.push((
            spec.name.into(),
            StoredInput {
                value: default,
                source: Source::Default,
            },
        ));
    }
    cross_checks(&inputs, report, &at);
    DesignOption {
        name: name.into(),
        inputs,
    }
}

/// The profile's default for an input, given the inputs resolved so far. `Ok(None)` when an
/// input it depends on is missing or wrong (that error is reported elsewhere); `Err` when the
/// profile has no default for this case.
fn default_for(
    name: &str,
    inputs: &[(String, StoredInput)],
    profile: &Profile,
) -> Result<Option<Value>, String> {
    let got = |n: &str| {
        inputs
            .iter()
            .find(|(k, _)| k == n)
            .and_then(|(_, i)| i.value.as_str().map(String::from))
    };
    let (value, case) = match KEYED_DEFAULTS.iter().find(|(n, _, _)| *n == name) {
        Some((_, by, _)) => {
            let key = match *by {
                "cargo_phase" => {
                    let Some(cargo) = got("cargo") else {
                        return Ok(None);
                    };
                    let Ok(props) = ovcd_core::cargo::get_cargo(&cargo) else {
                        return Ok(None);
                    };
                    props.phase.to_string()
                }
                other => match got(other) {
                    Some(k) => k,
                    None => return Ok(None),
                },
            };
            (
                profile.default_by(name, by, &key),
                format!("`default_by_{by}` for {key}"),
            )
        }
        None => (profile.default(name), "a `default`".to_string()),
    };
    match value {
        Some(v) => Ok(Some(normalise(v.clone()))),
        None => Err(format!(
            "the Defaults Profile '{}' gives no default for `{name}` ({case} is missing)",
            profile.name()
        )),
    }
}

/// Store every number as a float, so `10` and `10.0` are the same input.
fn normalise(v: Value) -> Value {
    match v.as_f64() {
        Some(x) => json!(x),
        _ => v,
    }
}

/// Check one value against its type and ranges. Returns the value as stored and any soft
/// warnings, or the hard error.
fn check_value(
    spec: &InputSpec,
    v: &Value,
    profile: &Profile,
) -> Result<(Value, Vec<String>), String> {
    let name = spec.name;
    match spec.kind {
        InputKind::Number { nullable } => {
            if v.is_null() {
                return if nullable {
                    Ok((Value::Null, vec![]))
                } else {
                    Err(format!("`{name}` must be a number"))
                };
            }
            let x = v.as_f64().ok_or_else(|| {
                if nullable {
                    format!("`{name}` must be a number or null (none)")
                } else {
                    format!("`{name}` must be a number")
                }
            })?;
            let warnings = number_ranges(name, name, x, profile)?;
            Ok((json!(x), warnings))
        }
        InputKind::Bool => match v {
            Value::Bool(_) => Ok((v.clone(), vec![])),
            _ => Err(format!("`{name}` must be true or false")),
        },
        InputKind::Choice(choices) => {
            let s = v.as_str().unwrap_or("");
            if choices.contains(&s) {
                Ok((v.clone(), vec![]))
            } else if name == "hull_form" {
                Err(format!(
                    "`hull_form` = {v} is not supported; only raked_box exists for now"
                ))
            } else {
                Err(format!(
                    "`{name}` = {v} must be one of: {}",
                    choices.join(", ")
                ))
            }
        }
        InputKind::Cargo => {
            let s = v.as_str().ok_or("`cargo` must be a string")?;
            ovcd_core::cargo::get_cargo(s).map_err(|e| format!("`cargo`: {e}"))?;
            Ok((v.clone(), vec![]))
        }
        InputKind::Formation => check_formation(v, profile),
    }
}

fn check_formation(v: &Value, profile: &Profile) -> Result<(Value, Vec<String>), String> {
    let o = v
        .as_object()
        .ok_or("`formation` must be an object with a `kind`")?;
    let kind = o.get("kind").and_then(Value::as_str).unwrap_or("");
    if !FORMATION_KINDS.contains(&kind) {
        return Err(format!(
            "`formation.kind` must be one of: {}",
            FORMATION_KINDS.join(", ")
        ));
    }
    let mut warnings = Vec::new();
    if kind != "pushed_convoy" {
        if o.len() != 1 {
            return Err(format!(
                "`formation`: a {kind} holds only `kind`; `abreast` and `in_line` are for a \
                 pushed_convoy"
            ));
        }
        return Ok((json!({ "kind": kind }), warnings));
    }
    let mut out = json!({ "kind": kind });
    for (key, profile_name) in [
        ("abreast", "formation_abreast"),
        ("in_line", "formation_in_line"),
    ] {
        let label = format!("formation.{key}");
        let n = o
            .get(key)
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("`{label}` must be a whole number for a pushed_convoy"))?;
        warnings.extend(number_ranges(&label, profile_name, n as f64, profile)?);
        out[key] = json!(n);
    }
    if o.len() != 3 {
        return Err(
            "`formation`: a pushed_convoy holds only `kind`, `abreast` and `in_line`".into(),
        );
    }
    Ok((out, warnings))
}

/// Hard range (error) and soft range (warning) of a number.
fn number_ranges(
    label: &str,
    name: &str,
    x: f64,
    profile: &Profile,
) -> Result<Vec<String>, String> {
    let unit = profile.unit(name);
    let with_unit = |n: f64| {
        if unit.is_empty() || unit == "-" {
            format!("{n}")
        } else {
            format!("{n} {unit}")
        }
    };
    if let Some(bound) = profile.hard(name)?.breach(x) {
        return Err(format!(
            "`{label}` = {x} is out of range: must be {bound} (hard limit)"
        ));
    }
    let mut warnings = Vec::new();
    if let Some(s) = profile.soft(name)?
        && (x < s.min || x > s.max)
    {
        warnings.push(format!(
            "`{label}` = {} is outside the usual range {} to {} (soft limit); accepted",
            with_unit(x),
            s.min,
            with_unit(s.max)
        ));
    }
    Ok(warnings)
}

/// Checks across inputs.
fn cross_checks(
    inputs: &[(String, StoredInput)],
    report: &mut Report,
    at: &dyn Fn(String) -> String,
) {
    let get = |n: &str| inputs.iter().find(|(k, _)| k == n).map(|(_, i)| &i.value);
    if let (Some(cargo), Some(containment)) = (
        get("cargo").and_then(Value::as_str),
        get("cargo_containment").and_then(Value::as_str),
    ) && let Ok(props) = ovcd_core::cargo::get_cargo(cargo)
    {
        let carries = CARGO_CONTAINMENTS
            .iter()
            .any(|(c, phase)| *c == containment && *phase == props.phase);
        if !carries {
            let fits: Vec<&str> = CARGO_CONTAINMENTS
                .iter()
                .filter(|(_, phase)| *phase == props.phase)
                .map(|(c, _)| *c)
                .collect();
            report.errors.push(at(format!(
                "`cargo_containment` = {containment} cannot carry {cargo} ({}); use {}",
                props.phase,
                fits.join(" or ")
            )));
        }
    }
    if let (Some(fb), Some(d)) = (
        get("min_freeboard_m").and_then(Value::as_f64),
        get("moulded_depth_m").and_then(Value::as_f64),
    ) && fb >= d
    {
        report.errors.push(at(format!(
            "`min_freeboard_m` = {fb} must be less than `moulded_depth_m` = {d} (hard limit)"
        )));
    }
    // The depth search tries hulls up to a fixed depth; above it no searched hull keeps the
    // freeboard, and the core raises when no waterway limit caps the draught.
    if get("optimise_depth").and_then(Value::as_bool) == Some(true)
        && let Some(fb) = get("min_freeboard_m").and_then(Value::as_f64)
        && fb > DEPTH_SEARCH_MAX_M
    {
        report.errors.push(at(format!(
            "`min_freeboard_m` = {fb} must be at most {DEPTH_SEARCH_MAX_M}, the deepest hull \
             `optimise_depth` searches (hard limit)"
        )));
    }
    // A channel shallower than its own clearance leaves no water to float in.
    if let (Some(depth), Some(ukc)) = (
        get("channel_depth_m").and_then(Value::as_f64),
        get("channel_under_keel_clearance_m").and_then(Value::as_f64),
    ) && depth < ukc
    {
        report.errors.push(at(format!(
            "`channel_depth_m` = {depth} must be at least `channel_under_keel_clearance_m` = \
             {ukc} (hard limit)"
        )));
    }
}

/// Check every default in a profile against its own input's type and hard range.
pub(crate) fn check_profile_defaults(profile: &Profile) -> Result<(), String> {
    for spec in INPUTS.iter().filter(|s| !s.required) {
        let mut defaults: Vec<Value> = profile.default(spec.name).cloned().into_iter().collect();
        for (_, by, keys) in KEYED_DEFAULTS.iter().filter(|(n, _, _)| *n == spec.name) {
            for key in *keys {
                defaults.extend(profile.default_by(spec.name, by, key).cloned());
            }
        }
        for d in defaults {
            check_value(spec, &d, profile)
                .map_err(|e| format!("profile default is invalid: {e}"))?;
        }
    }
    Ok(())
}

/// An input value as a person reads it.
pub fn show(v: &Value) -> String {
    match v {
        Value::Null => "none".into(),
        Value::String(s) => s.clone(),
        Value::Number(n) => n.as_f64().map(|x| format!("{x}")).unwrap_or(n.to_string()),
        Value::Object(o) if o.contains_key("kind") => {
            let kind = o["kind"].as_str().unwrap_or("?");
            match (o.get("abreast"), o.get("in_line")) {
                (Some(a), Some(l)) => format!("{kind} ({a} abreast, {l} in line)"),
                _ => kind.into(),
            }
        }
        other => other.to_string(),
    }
}
