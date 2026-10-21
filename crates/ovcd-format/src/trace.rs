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

/// The deadweight bands, at the thresholds of MARPOL Annex I Reg. 19 (MEPC.117(52)): 19.1, 600
/// tonnes deadweight and above; 19.2 and 19.6, 5,000 tonnes; 19.3.4, 20,000 tonnes.
pub const DEADWEIGHT_BANDS: &[&str] = &[
    "under-600-t",
    "600-to-5000-t",
    "5000-to-20000-t",
    "20000-t-and-above",
];

/// The band of a deadweight, t. "x tonnes deadweight and above" is in the band that starts at x.
pub fn deadweight_band(dw_t: f64) -> &'static str {
    if dw_t < 600.0 {
        DEADWEIGHT_BANDS[0]
    } else if dw_t < 5000.0 {
        DEADWEIGHT_BANDS[1]
    } else if dw_t < 20000.0 {
        DEADWEIGHT_BANDS[2]
    } else {
        DEADWEIGHT_BANDS[3]
    }
}

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
            ObjectType::Tank => match &self.measured.tank {
                Some(t) => (1..=t.n_tanks.max(0))
                    .map(|i| format!("Tank {i}"))
                    .collect(),
                None => Vec::new(),
            },
            // No Measure on these objects is computed yet.
            ObjectType::Hull
            | ObjectType::CargoTank
            | ObjectType::BallastTank
            | ObjectType::SlopTank => Vec::new(),
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
        if m.id == "check.ok" {
            let name = match m.args.iter().find(|(k, _)| k == "name") {
                Some((_, ArgValue::Text(n))) => n,
                _ => return Err("`check.ok` needs the text argument `name`".to_string()),
            };
            return self
                .measured
                .checks
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, ok)| MeasureValue::Bool(*ok))
                .ok_or_else(|| format!("the result has no check `{name}`"));
        }
        if let Some(name) = m.id.strip_prefix("separation.") {
            let (s, pressure) = self
                .measured
                .separation
                .as_ref()
                .ok_or_else(|| "the Option has no gas cargo-area separation".to_string())?;
            return match name {
                "cofferdam" => Ok(MeasureValue::Number(s.cofferdam_m)),
                "aft" => Ok(MeasureValue::Number(s.aft_separation_m)),
                "fwd" => Ok(MeasureValue::Number(s.fwd_separation_m)),
                "tank_end" => Ok(MeasureValue::Text(
                    if *pressure { "pressure" } else { "standard" }.to_string(),
                )),
                other => Err(format!("no value for Measure `separation.{other}`")),
            };
        }
        if let Some(name) = m.id.strip_prefix("tank.") {
            let t = self
                .measured
                .tank
                .as_ref()
                .ok_or_else(|| "the Option has no Type C tanks".to_string())?;
            let number = |x: f64| Ok(MeasureValue::Number(x));
            return match name {
                "vc" => number(t.vc_m3),
                "side_gap" => number(t.side_gap_m),
                "bottom_gap" => number(t.bottom_gap_m),
                "insulation" => number(t.insulation_m),
                // The thinner wall; the first of equal, as Python's `min`.
                "wall_thickness" => number(if t.head_wall_mm < t.cylinder_wall_mm {
                    t.head_wall_mm
                } else {
                    t.cylinder_wall_mm
                }),
                "weld_factor" => number(t.weld_factor),
                "allowable_stress" => number(t.allowable_stress_mpa),
                "rm" => number(t.tensile_strength_mpa),
                "re" => number(t.yield_strength_mpa),
                "material" => Ok(MeasureValue::Text(t.material.clone())),
                other => Err(format!("no value for Measure `tank.{other}`")),
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
            "stability.condition_ok" => {
                // The check `simulate` lists for the condition (`ga::stability_check`).
                let name = format!("Stability \u{2014} {}", c.name.to_lowercase());
                self.measured
                    .checks
                    .iter()
                    .find(|(n, _)| *n == name)
                    .map(|(_, ok)| MeasureValue::Bool(*ok))
                    .ok_or_else(|| format!("the result has no check `{name}`"))
            }
            "gm0" => Ok(MeasureValue::Number(c.gm_fluid_m)),
            other => Err(format!("no value for Measure `{other}`")),
        }
    }

    fn has(&self, measure: &MeasureRef) -> bool {
        match measure.id.as_str() {
            // A check the engine did not run for this Option: the Rule does not apply to it.
            "check.ok" => measure.args.iter().any(|(k, v)| {
                k == "name"
                    && matches!(v, ArgValue::Text(n) if self.measured.checks.iter().any(|(c, _)| c == n))
            }),
            id if id.starts_with("separation.") => self.measured.separation.is_some(),
            _ => true,
        }
    }

    fn missing(&self, object: ObjectType) -> Option<String> {
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
    (
        "Air draft (empty, governing)",
        "ovcd/capacity/air-draft-empty",
    ),
    ("gm positive", "ovcd/capacity/hopper-gm"),
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
];

/// The id of the Rule a check reports, or `None` for a check no Rule reports.
pub fn rule_of_check(name: &str) -> Option<&'static str> {
    if name.starts_with("Stability \u{2014} ") {
        return Some("is2008/A.2.2/condition");
    }
    CHECK_RULES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, id)| *id)
}
