//! The Rule Trace of an Option (spec 0004, "Rule Trace"): the Measures of one computed Option, the
//! default stack of Rule Sets, and the trace as JSON. The evaluator is `ovcd_core::rule_trace`.

use crate::profile::Profile;
use crate::rule_set::{Library, Selection};
use crate::simulate::Measured;
use crate::vdesign::{DesignOption, INPUTS, InputKind};
use ovcd_core::gas_arrangement::RuleInputs;
use ovcd_core::rule_trace::{
    ArgDef, MeasureInfo, MeasureSource, MeasureValue, MethodStatus, ObjectType, Outcome, Trace,
    TraceRow, ValueKind, evaluate, measure_def,
};
use ovcd_core::rules::{ArgValue, MeasureRef};
use serde_json::{Value, json};

/// The Service Area every Option has until `.vdesign` format version 3 (#274) stores one.
pub const DEFAULT_SERVICE_AREA: &str = "river-only";

/// The registry entry of a Measure: a computed Measure of the core, or `input.<name>` for an input
/// of the schema that is a number, a yes/no or a choice. The unit of an input Measure is its unit in
/// the built-in Defaults Profile (`-` for none).
pub fn measure_info(id: &str) -> Option<MeasureInfo> {
    if let Some(def) = measure_def(id) {
        return Some(def.info());
    }
    let name = id.strip_prefix("input.")?;
    let spec = INPUTS.iter().find(|s| s.name == name)?;
    let value = match spec.kind {
        InputKind::Number { .. } => ValueKind::Number,
        InputKind::Bool => ValueKind::Bool,
        InputKind::Choice(_) | InputKind::Cargo => ValueKind::Text,
        InputKind::Formation
        | InputKind::VesselScheme
        | InputKind::VesselClass
        | InputKind::ParameterSet
        | InputKind::OffsetsTable => return None,
    };
    // Parsed once: a run asks this for every input.
    static PROFILE: std::sync::OnceLock<Profile> = std::sync::OnceLock::new();
    let unit = match PROFILE.get_or_init(Profile::built_in).unit(name) {
        "" => "-".to_string(),
        u => u.to_string(),
    };
    Some(MeasureInfo {
        unit,
        object: ObjectType::Option.key().to_string(),
        value,
        args: Vec::new(),
        // An input Measure echoes the value used; there is no method to check.
        method_status: MethodStatus::Verified,
    })
}

/// The default stack of Rule Set ids of an Option, before format version 3 (spec 0004, "Service
/// Area and default stacks", `river-only`). Only the ids the library holds are used.
pub fn default_stack(option: &DesignOption) -> Vec<&'static str> {
    let gas = option
        .input("cargo_containment")
        .and_then(|i| i.value.as_str())
        == Some("type_c_tanks");
    if gas {
        // `igc-2g` includes `is2008-a` (IGC 2.2.2); naming it here too keeps the IS Code rows in the
        // stack whatever the gas sets hold. A set selected twice is checked once.
        vec!["igc-2g", "is2008-a", "lr-inland-gas", "ovcd"]
    } else {
        vec!["is2008-a", "ovcd"]
    }
}

/// The sets a run of one Option checks: the default stack at its highest built-in version, then
/// each extra set (loaded with `--rule-set`). An extra set with the id of a set in the stack
/// replaces it for the run.
pub fn stack_for<'a>(
    library: &'a Library,
    option: &DesignOption,
    extra: &[(String, String)],
) -> Result<Selection<'a>, String> {
    let mut chosen: Vec<(String, String)> = Vec::new();
    for id in default_stack(option) {
        if let Some(set) = library.latest(id) {
            chosen.push((set.id.clone(), set.version.clone()));
        }
    }
    for (id, version) in extra {
        chosen.retain(|(i, _)| i != id);
        chosen.push((id.clone(), version.clone()));
    }
    let pairs: Vec<(&str, &str)> = chosen
        .iter()
        .map(|(i, v)| (i.as_str(), v.as_str()))
        .collect();
    library.select(&pairs)
}

/// The derived, read-only inputs an Applicability can name (spec 0004 additions of spec 0006
/// §5.2), with their values. The engine works them out; a user never types them.
pub const DERIVED_INPUTS: &[(&str, &[&str])] = &[
    // Until `.vdesign` format version 3 stores one, every Option is `river-only`.
    ("service_area", &["river-only", "coastal-and-river"]),
    ("cargo_class", &["gas", "dry_bulk", "liquid_bulk"]),
    ("deadweight_band", DEADWEIGHT_BANDS),
];

pub use ovcd_core::measured::{DEADWEIGHT_BANDS, deadweight_band};

/// The `cargo_class` value of a Cargo Class.
pub fn cargo_class_value(class: ovcd_core::cargo::CargoClass) -> &'static str {
    match class {
        ovcd_core::cargo::CargoClass::Gas => "gas",
        ovcd_core::cargo::CargoClass::DryBulk => "dry_bulk",
        ovcd_core::cargo::CargoClass::LiquidBulk => "liquid_bulk",
    }
}

/// The values an Applicability may give for an input: a derived input, a choice input of the
/// schema, or `cargo` (the Cargo keys). `None` for an input that cannot be named.
pub fn applicability_values(input: &str) -> Option<Vec<String>> {
    if let Some((_, values)) = DERIVED_INPUTS.iter().find(|(n, _)| *n == input) {
        return Some(values.iter().map(|v| v.to_string()).collect());
    }
    let spec = INPUTS.iter().find(|s| s.name == input)?;
    match spec.kind {
        InputKind::Choice(values) => Some(values.iter().map(|v| v.to_string()).collect()),
        InputKind::Cargo => Some(
            ovcd_core::cargo::CARGO_DATABASE
                .iter()
                .map(|(k, _)| k.to_string())
                .collect(),
        ),
        _ => None,
    }
}

/// The Measures of one computed Option.
pub struct OptionMeasures<'a> {
    pub option: &'a DesignOption,
    pub measured: &'a Measured,
}

impl MeasureSource for OptionMeasures<'_> {
    fn def(&self, id: &str) -> Option<MeasureInfo> {
        measure_info(id)
    }

    fn objects(&self, object: &str) -> Vec<String> {
        self.measured.objects_by_key(object, &self.option.name)
    }

    fn value(&self, m: &MeasureRef, index: usize) -> Result<MeasureValue, String> {
        if let Some(name) = m.id.strip_prefix("input.") {
            let stored = self
                .option
                .input(name)
                .ok_or_else(|| format!("the Option has no input `{name}`"))?;
            return match &stored.value {
                Value::Number(n) => n
                    .as_f64()
                    .map(MeasureValue::Number)
                    .ok_or_else(|| format!("`{name}` is not a number")),
                Value::Bool(b) => Ok(MeasureValue::Bool(*b)),
                Value::String(s) => Ok(MeasureValue::Text(s.clone())),
                _ => Err(format!("`{name}` has no value")),
            };
        }
        self.measured.value(m, index)
    }

    fn has(&self, measure: &MeasureRef) -> bool {
        self.measured.has(measure)
    }

    fn missing(&self, object: &str) -> Option<String> {
        let object = ObjectType::from_key(object)?;
        let gas = self
            .option
            .input("cargo_containment")
            .and_then(|i| i.value.as_str())
            == Some("type_c_tanks");
        if object == ObjectType::Tank && gas && self.measured.tank.is_none() {
            return Some("no tank fits the hull, so there is no tank to check".to_string());
        }
        if object != ObjectType::LoadingCondition || !self.measured.conditions.is_empty() {
            return None;
        }
        Some(if gas {
            "no stability loading conditions: the barge carries no cargo".to_string()
        } else {
            "intact stability is not computed for this Cargo yet (a first-order GM only)"
                .to_string()
        })
    }

    fn input(&self, name: &str) -> Option<String> {
        match name {
            "service_area" => return Some(DEFAULT_SERVICE_AREA.to_string()),
            "cargo_class" => {
                let cargo = self.option.input("cargo")?.value.as_str()?;
                let props = ovcd_core::cargo::get_cargo(cargo).ok()?;
                return Some(cargo_class_value(props.phase).to_string());
            }
            "deadweight_band" => {
                return self
                    .measured
                    .deadweight_t
                    .map(|dw| deadweight_band(dw).to_string());
            }
            _ => {}
        }
        self.option
            .input(name)
            .and_then(|i| i.value.as_str())
            .map(str::to_string)
    }
}

/// The Option's inputs as the arrangement search reads them for its Rule Sets (#277): every input
/// with a text value, the derived `service_area` and `cargo_class` (the search works out
/// `deadweight_band` per candidate), and every input Measure with its registry entry and value.
pub fn rule_inputs(option: &DesignOption) -> RuleInputs {
    let source = OptionMeasures {
        option,
        measured: &Measured::default(),
    };
    let mut applicability: Vec<(String, String)> = Vec::new();
    for spec in INPUTS {
        if let Some(v) = source.input(spec.name) {
            applicability.push((spec.name.to_string(), v));
        }
    }
    for name in ["service_area", "cargo_class"] {
        if let Some(v) = source.input(name) {
            applicability.push((name.to_string(), v));
        }
    }
    let mut measures = Vec::new();
    for spec in INPUTS {
        let id = format!("input.{}", spec.name);
        if let (Some(info), Ok(value)) = (measure_info(&id), source.value(&bare(&id), 0)) {
            measures.push((id, info, value));
        }
    }
    RuleInputs {
        applicability,
        measures,
    }
}

fn bare(id: &str) -> MeasureRef {
    MeasureRef {
        id: id.to_string(),
        args: Vec::new(),
    }
}

/// The Rule Trace of a computed Option.
pub fn trace_of(selection: &Selection, option: &DesignOption, measured: &Measured) -> Trace {
    let source = OptionMeasures { option, measured };
    evaluate(&selection.sets, &source)
}

fn value_json(v: &MeasureValue) -> Value {
    match v {
        MeasureValue::Number(x) => json!(x),
        MeasureValue::Bool(b) => json!(b),
        MeasureValue::Text(t) => json!(t),
    }
}

fn measure_json(m: &MeasureRef) -> Value {
    let args: serde_json::Map<String, Value> = m
        .args
        .iter()
        .map(|(k, v)| {
            let v = match v {
                ArgValue::Number(x) => json!(x),
                ArgValue::Text(t) => json!(t),
            };
            (k.clone(), v)
        })
        .collect();
    json!({"id": m.id, "args": args})
}

fn row_json(r: &TraceRow) -> Value {
    json!({
        "rule": r.rule,
        "rule_sets": r.rule_sets,
        "title": r.title,
        "citation": r.citation,
        "origin": r.origin.map(|o| o.key()),
        "rule_status": if r.verified { "verified" } else { "unverified" },
        "method_status": r.method_status.map(MethodStatus::key),
        "measure": r.measure.as_ref().map(measure_json),
        "object_type": r.object_type,
        "object": r.object,
        "inputs": r.inputs.iter().map(|u| json!({"measure": u.measure, "value": value_json(&u.value)})).collect::<Vec<_>>(),
        "value": r.value.as_ref().map(value_json),
        "unit": r.unit,
        "compare": r.compare.map(|c| c.key()),
        "limit": r.limit,
        "tolerance": r.tolerance,
        "margin": r.margin,
        "consequence": r.consequence.map(|c| c.key()),
        "outcome": r.outcome.key(),
        "governing": r.governing,
        "sizes": r.sizes,
        "gap": r.gap,
        "finding": r.finding,
    })
}

/// The trace as JSON: the selected sets, every row, and the verdict. A pass with blocking Rules
/// not evaluated names them; nothing here says "meets every rule" while one is not evaluated.
pub fn trace_json(t: &Trace, warnings: &[String]) -> Value {
    let not_evaluated: Vec<Value> = t
        .blocking_not_evaluated()
        .iter()
        .map(|r| json!({"rule": r.rule, "title": r.title}))
        .collect();
    let failing = t.rows.iter().filter(|r| r.outcome == Outcome::Fail).count();
    let summary = if !t.passes() {
        format!("fails {failing} blocking Rule check(s)")
    } else if not_evaluated.is_empty() {
        "meets every evaluated blocking Rule; no blocking Rule is left not evaluated".to_string()
    } else {
        let names: Vec<String> = t
            .blocking_not_evaluated()
            .iter()
            .map(|r| r.title.clone())
            .collect();
        format!(
            "meets every evaluated blocking Rule; {} blocking Rule(s) not evaluated: {}",
            names.len(),
            names.join(", ")
        )
    };
    json!({
        "rule_sets": t.sets,
        "service_area": DEFAULT_SERVICE_AREA,
        "warnings": warnings,
        "rows": t.rows.iter().map(row_json).collect::<Vec<_>>(),
        "verdict": {
            "passes": t.passes(),
            "failing_rows": failing,
            "blocking_not_evaluated": not_evaluated,
            "summary": summary,
        },
    })
}

/// The arguments a Measure takes, for `ovcd rules --measures`.
pub fn args_text(args: &[ArgDef]) -> String {
    let names: Vec<&str> = args.iter().map(|a| a.name).collect();
    names.join(", ")
}

/// The Measure registry as text, for `ovcd rules --measures`.
pub fn measures_text() -> String {
    let mut out = String::from("Computed Measures:\n");
    for m in ovcd_core::rule_trace::MEASURES {
        out.push_str(&format!(
            "{}  {}  {}  args [{}]  method {}\n  {}\n",
            m.id,
            m.object.key(),
            m.unit,
            args_text(m.args),
            m.method_status.key(),
            m.method
        ));
    }
    out.push_str("\nInput Measures (object option, method verified):\n");
    for spec in INPUTS {
        let id = format!("input.{}", spec.name);
        if let Some(info) = measure_info(&id) {
            let kind = match info.value {
                ValueKind::Number => "number",
                ValueKind::Bool => "yes/no",
                ValueKind::Text => "text",
            };
            out.push_str(&format!("{id}  {}  {kind}\n", info.unit));
        }
    }
    out
}

/// The Rule each check of a result reports (spec 0004, "Each check gains the id of the Rule it
/// reports"). The `checks` list is in the results hash, so the id is not written into each check;
/// this table holds the link instead, and a test keeps it complete. A name that starts with
/// `Stability — ` is the stability check of one loading condition.
pub const CHECK_RULES: &[(&str, &str)] = &[
    ("Cargo > 0", "ovcd/capacity/cargo-positive"),
    ("feasible geometry", "ovcd/capacity/feasible-geometry"),
    ("cargo zone positive", "ovcd/capacity/cargo-zone-positive"),
    (
        "air draft within limit",
        "ovcd/capacity/air-draft-within-limit",
    ),
    ("intact gm at reported cargo", "ovcd/capacity/intact-gm"),
    (
        "Feasible arrangement",
        "ovcd/gas-arrangement/feasible-arrangement",
    ),
    (
        "tank length over diameter",
        "ovcd/gas-arrangement/tank-l-over-d",
    ),
    ("thrusters fit beam", "ovcd/aft-block/thrusters-fit-beam"),
    (
        "requested thruster count fits",
        "ovcd/aft-block/thruster-count",
    ),
    ("containers fit beam", "ovcd/aft-block/containers-fit-beam"),
    ("units fit in one row", "ovcd/aft-block/units-in-one-row"),
    (
        "wheelhouse fits beam",
        "ovcd/aft-block/wheelhouse-fits-beam",
    ),
    (
        "accom beside wheelhouse",
        "ovcd/aft-block/accom-beside-wheelhouse",
    ),
    ("gensets fit beam", "ovcd/aft-block/gensets-fit-beam"),
    (
        "gensets fit under deck",
        "ovcd/aft-block/gensets-under-deck",
    ),
    (
        "gensets carry load with one in reserve",
        "ovcd/aft-block/gensets-reserve",
    ),
    (
        "wheelhouse sees over cargo",
        "ovcd/aft-block/wheelhouse-sightline",
    ),
    ("cofferdam min width", "lr-inland/3.6.13/cofferdam"),
    ("opening setback", "lr-inland/3.3.7/opening-setback"),
    ("fwd end bulkhead space", "lr-inland/3.5.3/end-space"),
    // The checks only the arrangement search makes (`fit`, #277).
    ("not overloaded", "ovcd/capacity/not-overloaded"),
    ("freeboard ok", "ovcd/capacity/freeboard"),
    ("capacity is volume bound", "ovcd/journey/volume-bound"),
    ("floats", "ovcd/journey/floats"),
    ("ukc consistent", "ovcd/journey/ukc-consistent"),
    ("utilisation reasonable", "ovcd/journey/utilisation"),
    (
        "freeboard at payload",
        "ovcd/gas-arrangement/freeboard-at-payload",
    ),
    ("Stability \u{2014} not assessed", "ovcd/stability/assessed"),
];

/// The id of the Rule a check reports, or `None` for a check no Rule reports.
pub fn rule_of_check(name: &str) -> Option<&'static str> {
    if name.starts_with("Stability \u{2014} ") && name != "Stability \u{2014} not assessed" {
        return Some("is2008/A.2.2/condition");
    }
    CHECK_RULES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, id)| *id)
}

/// What a box section gives the Rule Trace (the evaluation harness, issue #278): one loading
/// condition and its stability check, nothing else. It has no inputs, so a set or Rule that applies
/// only for some input value does not apply.
struct BoxMeasures<'a> {
    measured: &'a Measured,
}

impl MeasureSource for BoxMeasures<'_> {
    fn def(&self, id: &str) -> Option<MeasureInfo> {
        measure_info(id)
    }

    fn objects(&self, object: &str) -> Vec<String> {
        self.measured.objects_by_key(object, "Box")
    }

    fn value(&self, m: &MeasureRef, index: usize) -> Result<MeasureValue, String> {
        self.measured.value(m, index)
    }

    fn input(&self, _name: &str) -> Option<String> {
        None
    }

    fn has(&self, measure: &MeasureRef) -> bool {
        self.measured.has(measure)
    }
}

/// The Rule Trace of one box loading condition under the sets `chosen` (`(id, version)`, in
/// selection order) of `library`, as `trace_json` writes it. The condition is named "Box". The
/// harness compares its rows with a case's expected verdicts, keyed by rule id.
pub fn box_trace(
    mut condition: ovcd_core::gas_stability::Condition,
    library: &Library,
    chosen: &[(&str, &str)],
) -> Result<Value, String> {
    condition.name = "Box".to_string();
    let check = ovcd_core::gas_arrangement::stability_check(&condition);
    let measured = Measured {
        checks: vec![(check.name, check.ok)],
        conditions: vec![condition],
        ..Measured::default()
    };
    let selection = library.select(chosen)?;
    let trace = evaluate(
        &selection.sets,
        &BoxMeasures {
            measured: &measured,
        },
    );
    Ok(trace_json(&trace, &[]))
}
