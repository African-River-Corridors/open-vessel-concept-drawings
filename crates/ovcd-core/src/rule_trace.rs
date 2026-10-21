//! The rule evaluator and the Rule Trace (spec 0004, "Domain model (precise)", "Composition:
//! strictest governs" and "Rule Trace"; ADR 0013).
//!
//! The evaluator reads plain typed data: the selected Rule Sets, in selection order, and a
//! [`MeasureSource`] that gives the objects of an Option and the value of a Measure on one of them.
//! It returns one row per Rule per object, marks the governing row of each Requirement, and gives
//! the Option's verdict. It decides nothing about physics: Measures do that.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::rules::{
    Applicability, Compare, Consequence, Kind, Limit, MeasureRef, Operand, Origin, RowWhen, Rule,
    RuleSet, Table,
};

/// The type of object a Measure is computed for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    Option,
    Hull,
    Tank,
    LoadingCondition,
}

impl ObjectType {
    pub const fn key(self) -> &'static str {
        match self {
            ObjectType::Option => "option",
            ObjectType::Hull => "hull",
            ObjectType::Tank => "tank",
            ObjectType::LoadingCondition => "loading-condition",
        }
    }
}

/// What a Measure gives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Number,
    Bool,
    Text,
}

/// Whether a Measure's method has been checked against a source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodStatus {
    Verified,
    Unverified,
}

impl MethodStatus {
    pub const fn key(self) -> &'static str {
        match self {
            MethodStatus::Verified => "verified",
            MethodStatus::Unverified => "unverified",
        }
    }
}

/// An argument a Measure takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgDef {
    pub name: &'static str,
    pub text: bool,
}

/// One entry of the Measure registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeasureDef {
    pub id: &'static str,
    pub unit: &'static str,
    pub object: ObjectType,
    pub value: ValueKind,
    pub args: &'static [ArgDef],
    /// The method, cited: what the Measure computes and how.
    pub method: &'static str,
    pub method_status: MethodStatus,
}

const fn num(name: &'static str) -> ArgDef {
    ArgDef { name, text: false }
}

/// The computed Measures of the core. Input Measures (`input.<name>`) are added by the caller, which
/// knows the inputs and their units.
pub static MEASURES: &[MeasureDef] = &[
    MeasureDef {
        id: "gz.area",
        unit: "m.rad",
        object: ObjectType::LoadingCondition,
        value: ValueKind::Number,
        args: &[num("from_deg"), num("to_deg")],
        method: "Trapezoid area under the GZ curve on the 1 degree grid, from from_deg to to_deg",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "gz.max_from",
        unit: "m",
        object: ObjectType::LoadingCondition,
        value: ValueKind::Number,
        args: &[num("from_deg")],
        method: "Largest GZ on the 1 degree grid at a heel of from_deg or more",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "gz.angle_of_max",
        unit: "deg",
        object: ObjectType::LoadingCondition,
        value: ValueKind::Number,
        args: &[],
        method: "Heel of the largest GZ on the 1 degree grid, 0 to 60 degrees; the first of equal values",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "gm0",
        unit: "m",
        object: ObjectType::LoadingCondition,
        value: ValueKind::Number,
        args: &[],
        method: "Initial metacentric height of the box section, less the free surface correction",
        method_status: MethodStatus::Unverified,
    },
];

/// A Measure of the core registry.
pub fn measure_def(id: &str) -> Option<&'static MeasureDef> {
    MEASURES.iter().find(|m| m.id == id)
}

/// What the evaluator and the loader need to know of a Measure, core or input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasureInfo {
    pub unit: String,
    pub object: ObjectType,
    pub value: ValueKind,
    pub args: Vec<ArgDef>,
    pub method_status: MethodStatus,
}

impl MeasureDef {
    pub fn info(&self) -> MeasureInfo {
        MeasureInfo {
            unit: self.unit.to_string(),
            object: self.object,
            value: self.value,
            args: self.args.to_vec(),
            method_status: self.method_status,
        }
    }
}

/// The value of a Measure on one object.
#[derive(Debug, Clone, PartialEq)]
pub enum MeasureValue {
    Number(f64),
    Bool(bool),
    Text(String),
}

/// What the evaluator needs from an Option: its objects, the values of Measures on them, its input
/// values (for Applicability) and the registry entry of any Measure (core or input).
pub trait MeasureSource {
    /// The registry entry of a Measure id, or `None` for an unknown id.
    fn def(&self, id: &str) -> Option<MeasureInfo>;
    /// The labels of the Option's objects of one type, in their fixed order.
    fn objects(&self, object: ObjectType) -> Vec<String>;
    /// The value of a Measure instance on object `index` of its type, or why it cannot be computed.
    fn value(&self, measure: &MeasureRef, index: usize) -> Result<MeasureValue, String>;
    /// The text value of an enumerated input, for Applicability. `None` when the Option lacks it.
    fn input(&self, name: &str) -> Option<String>;
}

/// The outcome of one Rule on one object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Pass,
    Fail,
    Warn,
    Note,
    NotApplicable,
    NotEvaluated,
}

impl Outcome {
    pub const fn key(self) -> &'static str {
        match self {
            Outcome::Pass => "pass",
            Outcome::Fail => "fail",
            Outcome::Warn => "warn",
            Outcome::Note => "note",
            Outcome::NotApplicable => "not applicable",
            Outcome::NotEvaluated => "not evaluated",
        }
    }
}

/// A value a limit used: a Measure (a table's key, a formula operand) and its value.
#[derive(Debug, Clone, PartialEq)]
pub struct UsedValue {
    pub measure: String,
    pub value: MeasureValue,
}

/// One row of the Rule Trace.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceRow {
    /// `None` for the one row of a set that does not apply.
    pub rule: Option<String>,
    /// Every selected set that holds the Rule, as `id@version`.
    pub rule_sets: Vec<String>,
    pub title: String,
    pub citation: String,
    pub origin: Option<Origin>,
    /// Whether the Rule was verified against its text.
    pub verified: bool,
    pub method_status: Option<MethodStatus>,
    pub measure: Option<MeasureRef>,
    pub object_type: Option<ObjectType>,
    pub object: Option<String>,
    pub value: Option<MeasureValue>,
    pub unit: String,
    pub compare: Option<Compare>,
    pub limit: Option<f64>,
    pub tolerance: f64,
    pub margin: Option<f64>,
    pub inputs: Vec<UsedValue>,
    pub consequence: Option<Consequence>,
    pub outcome: Outcome,
    pub governing: bool,
    pub sizes: bool,
    pub gap: Option<String>,
    pub finding: Option<String>,
}

/// The Rule Trace of one Option, and its verdict.
#[derive(Debug, Clone, PartialEq)]
pub struct Trace {
    /// The selected sets, in selection order, as `id@version`.
    pub sets: Vec<String>,
    pub rows: Vec<TraceRow>,
}

impl Trace {
    /// The Option passes when no row fails.
    pub fn passes(&self) -> bool {
        !self.rows.iter().any(|r| r.outcome == Outcome::Fail)
    }

    /// The blocking Rules that are not evaluated, each once, in trace order.
    pub fn blocking_not_evaluated(&self) -> Vec<&TraceRow> {
        let mut out: Vec<&TraceRow> = Vec::new();
        for r in &self.rows {
            if r.outcome == Outcome::NotEvaluated
                && r.consequence == Some(Consequence::Block)
                && !out.iter().any(|o| o.rule == r.rule)
            {
                out.push(r);
            }
        }
        out
    }
}

/// Whether an Option's inputs meet an Applicability: every input it names has one of its values.
pub fn applies(a: &Applicability, source: &dyn MeasureSource) -> bool {
    a.conditions
        .iter()
        .all(|(input, values)| source.input(input).is_some_and(|v| values.contains(&v)))
}

/// Whether `value` meets a Criterion, in exactly the form spec 0004 gives, so that a migrated check
/// stays bit-identical: `ge` is `value >= limit - tolerance`, and so on.
pub fn holds(compare: Compare, value: f64, limit: f64, tolerance: f64) -> bool {
    match compare {
        Compare::Ge => value >= limit - tolerance,
        Compare::Gt => value > limit - tolerance,
        Compare::Le => value <= limit + tolerance,
        Compare::Lt => value < limit + tolerance,
    }
}

/// The Margin: `value - limit` for a lower bound, `limit - value` for an upper bound. Positive
/// passes.
pub fn margin(compare: Compare, value: f64, limit: f64) -> f64 {
    if compare.is_lower_bound() {
        value - limit
    } else {
        limit - value
    }
}

/// Whether Criterion `a` is stricter than `b` (same direction): its effective bound lets fewer
/// values pass; at an equal effective bound, a strict comparison is stricter.
pub fn stricter(a: (Compare, f64, f64), b: (Compare, f64, f64)) -> bool {
    let (ca, la, ta) = a;
    let (cb, lb, tb) = b;
    if ca.is_lower_bound() {
        let (ea, eb) = (la - ta, lb - tb);
        ea > eb || (ea == eb && ca.is_strict() && !cb.is_strict())
    } else {
        let (ea, eb) = (la + ta, lb + tb);
        ea < eb || (ea == eb && ca.is_strict() && !cb.is_strict())
    }
}

/// The value of a table row shape at `v`. Each shape keeps the order of operations of the code it
/// replaces, so results stay bit-identical.
pub fn row_shape(shape: &str, coefficients: &[(String, f64)], v: f64) -> Result<f64, String> {
    let c = |name: &str| -> Result<f64, String> {
        coefficients
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, x)| *x)
            .ok_or_else(|| format!("row shape `{shape}` needs coefficient `{name}`"))
    };
    match shape {
        "const" => c("a"),
        "a+v*b/c" => Ok(c("a")? + v * c("b")? / c("c")?),
        "a+v/c" => Ok(c("a")? + v / c("c")?),
        other => Err(format!("unknown table row shape `{other}`")),
    }
}

/// The table row shapes and the coefficients each takes.
pub const ROW_SHAPES: &[(&str, &[&str])] = &[
    ("const", &["a"]),
    ("a+v*b/c", &["a", "b", "c"]),
    ("a+v/c", &["a", "c"]),
];

/// The formula shapes: id, operands, coefficients.
pub const FORMULA_SHAPES: &[(&str, &[&str], &[&str])] = &[
    // `of - less`: a value less an input (IGC 2.4.1.2 side distance less the insulation).
    ("less-input", &["of", "less"], &[]),
];

/// The value of a table at a key: the first row whose condition holds.
pub fn table_value(table: &Table, key: &MeasureValue) -> Result<f64, String> {
    for row in &table.rows {
        let hit = match (&row.when, key) {
            (RowWhen::Bounds(bounds), MeasureValue::Number(v)) => bounds
                .iter()
                .all(|b| holds_strictly(b.compare, *v, b.value)),
            (RowWhen::OneOf(values), MeasureValue::Text(t)) => values.contains(t),
            _ => {
                return Err(format!(
                    "table `{}` cannot be read at a value of this kind",
                    table.id
                ));
            }
        };
        if hit {
            let v = match key {
                MeasureValue::Number(v) => *v,
                _ => 0.0,
            };
            return row_shape(&row.shape, &row.coefficients, v);
        }
    }
    Err(format!("table `{}` has no row for this value", table.id))
}

/// A table bound with no tolerance: the side of the boundary exactly as the text gives it.
fn holds_strictly(compare: Compare, v: f64, bound: f64) -> bool {
    match compare {
        Compare::Ge => v >= bound,
        Compare::Gt => v > bound,
        Compare::Le => v <= bound,
        Compare::Lt => v < bound,
    }
}

fn number(v: MeasureValue, id: &str) -> Result<f64, String> {
    match v {
        MeasureValue::Number(x) if x.is_finite() => Ok(x),
        MeasureValue::Number(_) => Err(format!("`{id}` gave a value that is not finite")),
        _ => Err(format!("`{id}` is not a number")),
    }
}

/// A Measure with no arguments, by id.
fn bare(id: &str) -> MeasureRef {
    MeasureRef {
        id: id.to_string(),
        args: Vec::new(),
    }
}

/// The limit of a Rule on one object, and the values it used.
fn limit_of(
    rule: &Rule,
    set: &RuleSet,
    source: &dyn MeasureSource,
    index: usize,
) -> Result<(f64, Vec<UsedValue>), String> {
    let mut used = Vec::new();
    let read = |id: &str, used: &mut Vec<UsedValue>| -> Result<MeasureValue, String> {
        let v = source.value(&bare(id), index)?;
        used.push(UsedValue {
            measure: id.to_string(),
            value: v.clone(),
        });
        Ok(v)
    };
    let table = |id: &str| {
        set.table(id)
            .ok_or_else(|| format!("table `{id}` is not in the set"))
    };
    let value = match &rule.limit {
        Some(Limit::Constant { value, .. }) => *value,
        Some(Limit::Table { table: t, at, .. }) => {
            let key = read(at, &mut used)?;
            table_value(table(t)?, &key)?
        }
        Some(Limit::Formula {
            shape, operands, ..
        }) => {
            let mut get = |name: &str| -> Result<f64, String> {
                let (_, op) = operands
                    .iter()
                    .find(|(k, _)| k == name)
                    .ok_or_else(|| format!("formula `{shape}` needs operand `{name}`"))?;
                match op {
                    Operand::Measure(id) => number(read(id, &mut used)?, id),
                    Operand::TableAt { table: t, at } => {
                        let key = read(at, &mut used)?;
                        table_value(table(t)?, &key)
                    }
                }
            };
            match shape.as_str() {
                "less-input" => {
                    let of = get("of")?;
                    of - get("less")?
                }
                other => return Err(format!("unknown formula shape `{other}`")),
            }
        }
        None => return Err("the Rule has no limit".to_string()),
    };
    if !value.is_finite() {
        return Err("the limit is not finite".to_string());
    }
    Ok((value, used))
}

fn citation(rule: &Rule) -> String {
    let s = &rule.source;
    let mut parts: Vec<&str> = Vec::new();
    for p in [&s.document, &s.edition, &s.locator] {
        if !p.is_empty() {
            parts.push(p);
        }
    }
    parts.join(", ")
}

/// A row for a Rule with nothing filled in yet.
fn row_of(rule: &Rule, sets: &[String], def: Option<&MeasureInfo>) -> TraceRow {
    let (compare, unit) = match &rule.limit {
        Some(Limit::Constant { unit, .. })
        | Some(Limit::Table { unit, .. })
        | Some(Limit::Formula { unit, .. }) => (rule.compare, unit.clone()),
        None => (None, def.map_or(String::new(), |d| d.unit.clone())),
    };
    TraceRow {
        rule: Some(rule.id.clone()),
        rule_sets: sets.to_vec(),
        title: rule.title.clone(),
        citation: citation(rule),
        origin: Some(rule.origin),
        verified: rule.verified.is_some(),
        method_status: def.map(|d| d.method_status),
        measure: Some(rule.measure.clone()),
        object_type: def.map(|d| d.object),
        object: None,
        value: None,
        unit,
        compare,
        limit: None,
        tolerance: rule.tolerance,
        margin: None,
        inputs: Vec::new(),
        consequence: Some(rule.consequence),
        outcome: Outcome::NotApplicable,
        governing: false,
        sizes: rule.sizes,
        gap: rule.gap.clone(),
        finding: rule.finding.clone(),
    }
}

/// The outcome of a failed test: `fail` for a blocking Rule, `warn` otherwise.
fn failed(rule: &Rule) -> Outcome {
    match rule.consequence {
        Consequence::Block => Outcome::Fail,
        Consequence::Warn | Consequence::Note => Outcome::Warn,
    }
}

/// Evaluate the selected sets on one Option. `sets` must be in selection order (the loader's
/// selection gives it). A Rule id held by two sets with the same content is evaluated once and its
/// rows name both sets.
pub fn evaluate(sets: &[&RuleSet], source: &dyn MeasureSource) -> Trace {
    let keys: Vec<String> = sets.iter().map(|s| s.key()).collect();
    // Each Rule once, with every set that holds it, in selection order.
    let mut rules: Vec<(&Rule, &RuleSet, Vec<String>, usize)> = Vec::new();
    let mut set_rows: Vec<(usize, TraceRow)> = Vec::new();
    for (si, set) in sets.iter().enumerate() {
        if !applies(&set.applies_when, source) {
            set_rows.push((si, set_not_applicable(set)));
            continue;
        }
        for rule in &set.rules {
            match rules.iter_mut().find(|(r, ..)| r.id == rule.id) {
                Some((_, _, holders, _)) => holders.push(set.key()),
                None => rules.push((rule, set, vec![set.key()], si)),
            }
        }
    }
    let mut rows: Vec<(usize, usize, TraceRow)> = Vec::new();
    for (order, (rule, set, holders, si)) in rules.iter().enumerate() {
        for row in rule_rows(rule, set, holders, source) {
            rows.push((*si, order, row));
        }
    }
    mark_governing(&mut rows);
    // Rows in selection order; a set that does not apply keeps its place.
    let mut out: Vec<(usize, usize, TraceRow)> = rows;
    for (si, row) in set_rows {
        out.push((si, 0, row));
    }
    out.sort_by_key(|(si, order, _)| (*si, *order));
    Trace {
        sets: keys,
        rows: out.into_iter().map(|(_, _, r)| r).collect(),
    }
}

fn set_not_applicable(set: &RuleSet) -> TraceRow {
    TraceRow {
        rule: None,
        rule_sets: vec![set.key()],
        title: set.title.clone(),
        citation: String::new(),
        origin: None,
        verified: false,
        method_status: None,
        measure: None,
        object_type: None,
        object: None,
        value: None,
        unit: String::new(),
        compare: None,
        limit: None,
        tolerance: 0.0,
        margin: None,
        inputs: Vec::new(),
        consequence: None,
        outcome: Outcome::NotApplicable,
        governing: false,
        sizes: false,
        gap: None,
        finding: None,
    }
}

/// The rows of one Rule: one per object of its type.
fn rule_rows(
    rule: &Rule,
    set: &RuleSet,
    holders: &[String],
    source: &dyn MeasureSource,
) -> Vec<TraceRow> {
    let def = source.def(&rule.measure.id);
    let base = row_of(rule, holders, def.as_ref());
    if !applies(&rule.applies_when, source) {
        return vec![base];
    }
    let Some(def) = def else {
        // Only a Rule that is not evaluated may name a Measure the engine lacks: one row.
        let mut row = base;
        row.outcome = if rule.evaluated {
            failed(rule)
        } else {
            Outcome::NotEvaluated
        };
        if rule.evaluated {
            row.gap = Some(format!("unknown Measure `{}`", rule.measure.id));
        }
        return vec![row];
    };
    let objects = source.objects(def.object);
    if objects.is_empty() {
        return vec![base];
    }
    let mut out = Vec::new();
    for (index, label) in objects.into_iter().enumerate() {
        let mut row = base.clone();
        row.object = Some(label);
        if !rule.evaluated {
            row.outcome = Outcome::NotEvaluated;
            out.push(row);
            continue;
        }
        let value = source.value(&rule.measure, index);
        match (rule.kind, value) {
            (_, Err(reason)) => {
                row.outcome = failed(rule);
                row.gap = Some(reason);
            }
            (Kind::Note, Ok(v)) => {
                row.value = Some(v);
                row.outcome = Outcome::Note;
            }
            (Kind::Check, Ok(v)) => {
                row.outcome = match v {
                    MeasureValue::Bool(true) => Outcome::Pass,
                    MeasureValue::Bool(false) => failed(rule),
                    _ => {
                        row.gap = Some(format!("`{}` is not a yes/no Measure", rule.measure.id));
                        failed(rule)
                    }
                };
                row.value = Some(v);
            }
            (Kind::Limit | Kind::Table | Kind::Formula, Ok(v)) => {
                let compare = rule.compare.expect("a limit Rule has a comparison");
                match (
                    number(v.clone(), &rule.measure.id),
                    limit_of(rule, set, source, index),
                ) {
                    (Ok(x), Ok((limit, used))) => {
                        row.limit = Some(limit);
                        row.margin = Some(margin(compare, x, limit));
                        row.outcome = if holds(compare, x, limit, rule.tolerance) {
                            Outcome::Pass
                        } else {
                            failed(rule)
                        };
                        row.inputs = used;
                    }
                    (Err(reason), _) | (_, Err(reason)) => {
                        row.outcome = failed(rule);
                        row.gap = Some(reason);
                    }
                }
                row.value = Some(v);
            }
        }
        out.push(row);
    }
    out
}

/// Mark the governing row of each Requirement: the rows with a worked-out limit, on the same Measure
/// instance and object, in the same direction. Of the blocking rows (all rows if none blocks), the
/// strictest Criterion governs; ties go to selection order, then rule id. Chosen from the Criteria,
/// never from the value.
fn mark_governing(rows: &mut [(usize, usize, TraceRow)]) {
    let n = rows.len();
    let mut done = vec![false; n];
    for i in 0..n {
        if done[i] || rows[i].2.limit.is_none() {
            continue;
        }
        let key = |r: &TraceRow| {
            (
                r.measure.clone(),
                r.object.clone(),
                r.compare.map(Compare::is_lower_bound),
            )
        };
        let k = key(&rows[i].2);
        let members: Vec<usize> = (i..n)
            .filter(|&j| rows[j].2.limit.is_some() && key(&rows[j].2) == k)
            .collect();
        for &j in &members {
            done[j] = true;
        }
        let any_block = members
            .iter()
            .any(|&j| rows[j].2.consequence == Some(Consequence::Block));
        let candidates: Vec<usize> = members
            .into_iter()
            .filter(|&j| !any_block || rows[j].2.consequence == Some(Consequence::Block))
            .collect();
        let mut best = candidates[0];
        for &j in &candidates[1..] {
            let (a, b) = (&rows[j], &rows[best]);
            let crit = |r: &TraceRow| (r.compare.unwrap(), r.limit.unwrap(), r.tolerance);
            let better = if stricter(crit(&a.2), crit(&b.2)) {
                true
            } else if stricter(crit(&b.2), crit(&a.2)) {
                false
            } else {
                (a.0, &a.2.rule) < (b.0, &b.2.rule)
            };
            if better {
                best = j;
            }
        }
        rows[best].2.governing = true;
    }
}
