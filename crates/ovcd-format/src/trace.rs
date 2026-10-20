//! The Rule Trace of an Option (spec 0004, "Rule Trace"): the Measures of one computed Option, the
//! default stack of Rule Sets, and the trace as JSON. The evaluator is `ovcd_core::rule_trace`.

use crate::profile::Profile;
use crate::rule_set::{Library, Selection};
use crate::simulate::Measured;
use crate::vdesign::{DesignOption, INPUTS, InputKind};
use ovcd_core::box_stability::area;
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
        InputKind::Formation | InputKind::VesselScheme | InputKind::VesselClass => return None,
    };
    let unit = match Profile::built_in().unit(name) {
        "" => "-".to_string(),
        u => u.to_string(),
    };
    Some(MeasureInfo {
        unit,
        object: ObjectType::Option,
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
        vec!["igc-2g", "lr-inland-gas", "ovcd"]
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

/// The Measures of one computed Option.
pub struct OptionMeasures<'a> {
    pub option: &'a DesignOption,
    pub measured: &'a Measured,
}

fn arg(m: &MeasureRef, name: &str) -> Result<f64, String> {
    match m.args.iter().find(|(k, _)| k == name) {
        Some((_, ArgValue::Number(x))) => Ok(*x),
        _ => Err(format!("`{}` needs the number argument `{name}`", m.id)),
    }
}

/// A heel argument: a whole number of degrees on the 0 to 60 degree grid.
fn deg(m: &MeasureRef, name: &str) -> Result<i32, String> {
    let x = arg(m, name)?;
    if x.fract() == 0.0 && (0.0..=60.0).contains(&x) {
        Ok(x as i32)
    } else {
        Err(format!(
            "`{}` argument `{name}` must be a whole number of degrees from 0 to 60",
            m.id
        ))
    }
}

impl MeasureSource for OptionMeasures<'_> {
    fn def(&self, id: &str) -> Option<MeasureInfo> {
        measure_info(id)
    }

    fn objects(&self, object: ObjectType) -> Vec<String> {
        match object {
            ObjectType::Option => vec![self.option.name.clone()],
            ObjectType::LoadingCondition => self
                .measured
                .conditions
                .iter()
                .map(|c| c.name.clone())
                .collect(),
            ObjectType::Hull | ObjectType::Tank => Vec::new(),
        }
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
        let c = self
            .measured
            .conditions
            .get(index)
            .ok_or_else(|| format!("no loading condition {index}"))?;
        if c.gz.is_empty() {
            return Err(format!(
                "{}: the barge does not float with freeboard, so it has no GZ curve",
                c.name
            ));
        }
        match m.id.as_str() {
            "gz.area" => {
                let (from, to) = (deg(m, "from_deg")?, deg(m, "to_deg")?);
                if from >= to {
                    return Err("`gz.area` needs from_deg below to_deg".to_string());
                }
                Ok(MeasureValue::Number(area(&c.gz, from, to)))
            }
            "gz.max_from" => {
                let from = deg(m, "from_deg")?;
                // `max()` keeps the first of equal values, as the IS Code criterion does.
                let mut best: Option<f64> = None;
                for &(d, g) in &c.gz {
                    if d >= from && best.is_none_or(|b| g > b) {
                        best = Some(g);
                    }
                }
                best.map(MeasureValue::Number)
                    .ok_or_else(|| format!("no GZ at {from} degrees or more"))
            }
            "gz.angle_of_max" => Ok(MeasureValue::Number(f64::from(c.max_gz_deg))),
            "gm0" => Ok(MeasureValue::Number(c.gm_fluid_m)),
            other => Err(format!("no value for Measure `{other}`")),
        }
    }

    fn input(&self, name: &str) -> Option<String> {
        if name == "service_area" {
            return Some(DEFAULT_SERVICE_AREA.to_string());
        }
        self.option
            .input(name)
            .and_then(|i| i.value.as_str())
            .map(str::to_string)
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
        "object_type": r.object_type.map(ObjectType::key),
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
