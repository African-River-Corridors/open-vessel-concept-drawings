//! Rule Set files: the loader, the content hash, the library of loaded sets and the selection checks
//! (spec 0004, "Rule Set (data)" and "Loading at run time"; ADR 0013).
//!
//! A Rule Set file is JSON. The loader checks it field by field and builds the plain typed structs of
//! `ovcd_core::rules`; the core parses nothing. The schema beside the files
//! (`rules/rule-set.schema.json`) says the same as this loader: a file the schema refuses, the loader
//! refuses too. The loader also checks what a schema cannot: rule id against locator, repeated ids,
//! table names, and conflicts between two constant Criteria.

use crate::run::canonical_json;
use crate::trace::{applicability_values, measure_info};
use ovcd_core::rule_trace::{
    FORMULA_SHAPES, FREE_UNIT_OPERANDS, ObjectType, ROW_SHAPES, ValueKind,
};
use ovcd_core::rules::{
    Applicability, ArgValue, Bound, Citation, Codified, Compare, Consequence, Include, Kind,
    KnownConflict, Limit, MeasureRef, Operand, Origin, RowWhen, Rule, RuleSet, SetStatus, Source,
    Table, TableRow, Verification,
};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

/// The JSON Schema of a Rule Set file.
pub const SCHEMA: &str = include_str!("../rules/rule-set.schema.json");

/// The built-in Rule Sets: `(path, text)`, one file per set version, embedded so that every binary
/// and the WebAssembly build carry them. The first sets come with the migration tickets.
const BUILT_IN: &[(&str, &str)] = &[];

/// The roles that may write or verify a Rule. A person's name never goes in a Rule Set: the
/// repository is public.
pub const ROLES: &[&str] = &["maintainer", "expert-reviewer", "agent"];

/// The values of `source.access`.
pub const ACCESS: &[&str] = &["free", "paywalled", "sign-up"];

/// A loaded set, with the warnings the loader gave.
#[derive(Debug, Clone, PartialEq)]
pub struct Loaded {
    pub set: RuleSet,
    pub warnings: Vec<String>,
}

/// Read one Rule Set file. Refuses it with a message that names the field when anything is wrong.
pub fn load(text: &str) -> Result<Loaded, String> {
    let value: Value =
        serde_json::from_str(text).map_err(|e| format!("not a valid JSON file: {e}"))?;
    let mut set = parse_set(&value)?;
    check_set(&set)?;
    check_measures(&set)?;
    set.hash = content_hash(&set);
    let warnings = set
        .rules
        .iter()
        .filter(|r| r.consequence == Consequence::Block && r.verified.is_none())
        .map(|r| format!("{}: blocking Rule `{}` is unverified", set.key(), r.id))
        .collect();
    Ok(Loaded { set, warnings })
}

// --- reading the JSON ---------------------------------------------------------------------------

/// A JSON object with the path it was read at, for messages.
struct Obj<'a> {
    map: &'a Map<String, Value>,
    at: String,
}

impl<'a> Obj<'a> {
    fn of(v: &'a Value, at: &str) -> Result<Obj<'a>, String> {
        match v {
            Value::Object(map) => Ok(Obj {
                map,
                at: at.to_string(),
            }),
            _ => Err(format!("`{at}` must be an object")),
        }
    }

    /// Refuse a field this object does not have in the schema.
    fn only(&self, known: &[&str]) -> Result<(), String> {
        match self.map.keys().find(|k| !known.contains(&k.as_str())) {
            Some(k) => Err(format!("`{}` has an unknown field `{k}`", self.here())),
            None => Ok(()),
        }
    }

    fn here(&self) -> &str {
        if self.at.is_empty() {
            "the file"
        } else {
            &self.at
        }
    }

    fn path(&self, name: &str) -> String {
        if self.at.is_empty() {
            name.to_string()
        } else {
            format!("{}.{name}", self.at)
        }
    }

    fn get(&self, name: &str) -> Option<&'a Value> {
        self.map.get(name)
    }

    fn need(&self, name: &str) -> Result<&'a Value, String> {
        self.get(name)
            .ok_or_else(|| format!("`{}` is missing", self.path(name)))
    }

    fn text(&self, name: &str) -> Result<String, String> {
        match self.need(name)? {
            Value::String(s) if !s.trim().is_empty() => Ok(s.clone()),
            Value::String(_) => Err(format!("`{}` must not be empty", self.path(name))),
            _ => Err(format!("`{}` must be text", self.path(name))),
        }
    }

    fn text_or_empty(&self, name: &str) -> Result<String, String> {
        match self.get(name) {
            None => Ok(String::new()),
            Some(Value::String(s)) => Ok(s.clone()),
            Some(_) => Err(format!("`{}` must be text", self.path(name))),
        }
    }

    fn bool(&self, name: &str) -> Result<bool, String> {
        self.need(name)?
            .as_bool()
            .ok_or_else(|| format!("`{}` must be true or false", self.path(name)))
    }

    fn number(&self, name: &str) -> Result<f64, String> {
        number(self.need(name)?, &self.path(name))
    }

    fn array(&self, name: &str) -> Result<&'a Vec<Value>, String> {
        self.need(name)?
            .as_array()
            .ok_or_else(|| format!("`{}` must be a list", self.path(name)))
    }

    fn array_or_empty(&self, name: &str) -> Result<&'a [Value], String> {
        match self.get(name) {
            None => Ok(&[]),
            Some(v) => v
                .as_array()
                .map(Vec::as_slice)
                .ok_or_else(|| format!("`{}` must be a list", self.path(name))),
        }
    }

    fn choice<T: Copy>(
        &self,
        name: &str,
        all: &[T],
        key: fn(T) -> &'static str,
    ) -> Result<T, String> {
        let got = self.text(name)?;
        all.iter().copied().find(|c| key(*c) == got).ok_or_else(|| {
            let names: Vec<&str> = all.iter().map(|c| key(*c)).collect();
            format!(
                "`{}` is `{got}`; it must be one of {}",
                self.path(name),
                names.join(", ")
            )
        })
    }
}

/// A finite number.
fn number(v: &Value, at: &str) -> Result<f64, String> {
    match v.as_f64() {
        Some(x) if x.is_finite() => Ok(x),
        // A number out of the f64 range reads as an infinity, or as no f64 at all.
        _ if v.is_number() => Err(format!("`{at}` must be a finite number")),
        _ => Err(format!("`{at}` must be a number")),
    }
}

fn semver(o: &Obj, name: &str) -> Result<String, String> {
    let v = o.text(name)?;
    if is_semver(&v) {
        Ok(v)
    } else {
        Err(format!(
            "`{}` is `{v}`; it must be a version number such as 1.0.0",
            o.path(name)
        ))
    }
}

fn parse_set(v: &Value) -> Result<RuleSet, String> {
    let o = Obj::of(v, "")?;
    o.only(&[
        "$schema",
        "id",
        "family",
        "title",
        "version",
        "edition",
        "sources",
        "status",
        "applies_when",
        "includes",
        "known_conflicts",
        "tables",
        "rules",
    ])?;
    let id = o.text("id")?;
    if !is_word(&id) {
        return Err(format!(
            "`id` is `{id}`; a Rule Set id is lowercase letters, digits and hyphens, starting with a letter"
        ));
    }
    let family = o.text("family")?;
    if !is_word(&family) {
        return Err(format!(
            "`family` is `{family}`; a family is lowercase letters, digits and hyphens, starting with a letter"
        ));
    }
    let version = semver(&o, "version")?;
    let edition = {
        let e = Obj::of(o.need("edition")?, "edition")?;
        e.text("document")?;
        let mut fields = Vec::new();
        for (k, v) in e.map {
            match v {
                Value::String(s) => fields.push((k.clone(), s.clone())),
                _ => return Err(format!("`edition.{k}` must be text")),
            }
        }
        fields.sort();
        fields
    };
    let mut sources = Vec::new();
    for (i, s) in o.array("sources")?.iter().enumerate() {
        let s = Obj::of(s, &format!("sources[{i}]"))?;
        s.only(&["document", "edition", "url", "access", "sha256"])?;
        let access = s.text("access")?;
        if !ACCESS.contains(&access.as_str()) {
            return Err(format!(
                "`{}` is `{access}`; it must be one of {}",
                s.path("access"),
                ACCESS.join(", ")
            ));
        }
        let sha256 = s.text("sha256")?;
        if sha256.len() != 64 || !sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!(
                "`{}` must be 64 hexadecimal digits",
                s.path("sha256")
            ));
        }
        sources.push(Source {
            document: s.text("document")?,
            edition: s.text("edition")?,
            url: s.text_or_empty("url")?,
            access,
            sha256: sha256.to_ascii_lowercase(),
        });
    }
    let status = match o.text("status")?.as_str() {
        "draft" => SetStatus::Draft,
        "stable" => SetStatus::Stable,
        other => {
            return Err(format!("`status` is `{other}`; it must be draft or stable"));
        }
    };
    let applies_when = applicability(o.get("applies_when"), "applies_when")?;
    let mut includes = Vec::new();
    for (i, inc) in o.array_or_empty("includes")?.iter().enumerate() {
        let inc = Obj::of(inc, &format!("includes[{i}]"))?;
        inc.only(&["id", "version"])?;
        includes.push(Include {
            id: inc.text("id")?,
            version: semver(&inc, "version")?,
        });
    }
    let mut known_conflicts = Vec::new();
    for (i, k) in o.array_or_empty("known_conflicts")?.iter().enumerate() {
        let k = Obj::of(k, &format!("known_conflicts[{i}]"))?;
        k.only(&["rules", "reason"])?;
        let ids: Vec<String> = k
            .array("rules")?
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
        let [a, b] = <[String; 2]>::try_from(ids)
            .map_err(|_| format!("`{}` must be a list of two rule ids", k.path("rules")))?;
        known_conflicts.push(KnownConflict {
            rules: [a, b],
            reason: k.text("reason")?,
        });
    }
    let mut tables = Vec::new();
    for (i, t) in o.array_or_empty("tables")?.iter().enumerate() {
        tables.push(parse_table(t, &format!("tables[{i}]"))?);
    }
    let mut rules = Vec::new();
    for (i, r) in o.array("rules")?.iter().enumerate() {
        rules.push(parse_rule(r, &format!("rules[{i}]"))?);
    }
    Ok(RuleSet {
        id,
        family,
        title: o.text("title")?,
        version,
        edition,
        sources,
        status,
        applies_when,
        includes,
        known_conflicts,
        tables,
        rules,
        hash: String::new(),
    })
}

fn applicability(v: Option<&Value>, at: &str) -> Result<Applicability, String> {
    let Some(v) = v else {
        return Ok(Applicability::default());
    };
    let o = Obj::of(v, at)?;
    let mut conditions = Vec::new();
    for (input, values) in o.map {
        let list = values
            .as_array()
            .ok_or_else(|| format!("`{at}.{input}` must be a list of values"))?;
        if list.is_empty() {
            return Err(format!(
                "`{at}.{input}` is empty; a condition with no value can never be met"
            ));
        }
        let mut texts = Vec::new();
        for value in list {
            match value.as_str() {
                Some(s) if !s.is_empty() => texts.push(s.to_string()),
                _ => return Err(format!("`{at}.{input}` must hold only text values")),
            }
        }
        conditions.push((input.clone(), texts));
    }
    conditions.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(Applicability { conditions })
}

fn citation(v: &Value, at: &str) -> Result<Citation, String> {
    let o = Obj::of(v, at)?;
    o.only(&["document", "edition", "locator", "also", "page", "access"])?;
    let mut also = Vec::new();
    for (i, a) in o.array_or_empty("also")?.iter().enumerate() {
        match a.as_str() {
            Some(s) if !s.is_empty() => also.push(s.to_string()),
            _ => return Err(format!("`{at}.also[{i}]` must be text")),
        }
    }
    let access = o.text_or_empty("access")?;
    if !access.is_empty() && !ACCESS.contains(&access.as_str()) {
        return Err(format!(
            "`{at}.access` is `{access}`; it must be one of {}",
            ACCESS.join(", ")
        ));
    }
    Ok(Citation {
        document: o.text_or_empty("document")?,
        edition: o.text_or_empty("edition")?,
        locator: o.text_or_empty("locator")?,
        also,
        page: o.text_or_empty("page")?,
        access,
    })
}

fn role(o: &Obj, name: &str) -> Result<String, String> {
    let by = o.text(name)?;
    if ROLES.contains(&by.as_str()) {
        Ok(by)
    } else {
        Err(format!(
            "`{}` is `{by}`; it must be a role ({}), never a person's name",
            o.path(name),
            ROLES.join(", ")
        ))
    }
}

fn parse_rule(v: &Value, at: &str) -> Result<Rule, String> {
    let o = Obj::of(v, at)?;
    o.only(&[
        "id",
        "title",
        "origin",
        "source",
        "codified",
        "verified",
        "applies_when",
        "kind",
        "measure",
        "compare",
        "limit",
        "tolerance",
        "consequence",
        "sizes",
        "evaluated",
        "gap",
        "finding",
        "since",
    ])?;
    let id = o.text("id")?;
    // From here on, messages name the Rule by its id.
    let o = Obj {
        map: o.map,
        at: format!("{at} ({id})"),
    };
    if !is_rule_id(&id) {
        return Err(format!(
            "`{}` is `{id}`; a rule id is <family>/<paragraph>/<slug> (spec 0004, rule id grammar)",
            o.path("id")
        ));
    }
    let origin = o.choice("origin", &Origin::ALL, Origin::key)?;
    let source = citation(o.need("source")?, &o.path("source"))?;
    let codified = {
        let c = Obj::of(o.need("codified")?, &o.path("codified"))?;
        c.only(&["by", "date", "review"])?;
        Codified {
            by: role(&c, "by")?,
            date: c.text("date")?,
            review: c.text("review")?,
        }
    };
    let verified = match o.need("verified")? {
        Value::Bool(false) => None,
        v @ Value::Object(_) => {
            let c = Obj::of(v, &o.path("verified"))?;
            c.only(&["by", "date", "edition_held", "locator_read", "review"])?;
            Some(Verification {
                by: role(&c, "by")?,
                date: c.text("date")?,
                edition_held: c.text("edition_held")?,
                locator_read: c.text("locator_read")?,
                review: c.text("review")?,
            })
        }
        _ => {
            return Err(format!(
                "`{}` must be false or a verification record",
                o.path("verified")
            ));
        }
    };
    let kind = o.choice("kind", &Kind::ALL, Kind::key)?;
    let measure = {
        let m = Obj::of(o.need("measure")?, &o.path("measure"))?;
        m.only(&["id", "args"])?;
        let mut args = Vec::new();
        if let Some(a) = m.get("args") {
            let a = Obj::of(a, &m.path("args"))?;
            for (name, value) in a.map {
                let arg = match value {
                    Value::String(s) => ArgValue::Text(s.clone()),
                    v => ArgValue::Number(number(v, &a.path(name))?),
                };
                args.push((name.clone(), arg));
            }
            args.sort_by(|x, y| x.0.cmp(&y.0));
        }
        MeasureRef {
            id: m.text("id")?,
            args,
        }
    };
    let (compare, limit) = match kind {
        Kind::Check | Kind::Note => {
            for field in ["compare", "limit"] {
                if o.get(field).is_some() {
                    return Err(format!(
                        "`{}`: a `{}` Rule has no comparison and no limit",
                        o.path(field),
                        kind.key()
                    ));
                }
            }
            (None, None)
        }
        Kind::Limit | Kind::Table | Kind::Formula => {
            let compare = o.choice("compare", &Compare::ALL, Compare::key)?;
            let limit = parse_limit(kind, o.need("limit")?, &o.path("limit"))?;
            (Some(compare), Some(limit))
        }
    };
    let tolerance = match o.get("tolerance") {
        None => 0.0,
        Some(v) => {
            let t = number(v, &o.path("tolerance"))?;
            if t < 0.0 {
                return Err(format!("`{}` must not be negative", o.path("tolerance")));
            }
            t
        }
    };
    let consequence = o.choice("consequence", &Consequence::ALL, Consequence::key)?;
    if (kind == Kind::Note) != (consequence == Consequence::Note) {
        return Err(format!(
            "`{}`: a `note` Rule has consequence `note`, and only a `note` Rule has it",
            o.path("consequence")
        ));
    }
    let sizes = match o.get("sizes") {
        None => false,
        Some(_) => o.bool("sizes")?,
    };
    let evaluated = o.bool("evaluated")?;
    let gap = match o.get("gap") {
        None | Some(Value::Null) => None,
        Some(_) => Some(o.text("gap")?),
    };
    if !evaluated && gap.is_none() {
        return Err(format!(
            "`{}` is missing: a Rule that is not evaluated says what the engine lacks",
            o.path("gap")
        ));
    }
    if evaluated && gap.is_some() {
        return Err(format!(
            "`{}`: only a Rule that is not evaluated has a gap",
            o.path("gap")
        ));
    }
    let finding = match o.get("finding") {
        None | Some(Value::Null) => None,
        Some(_) => {
            let f = o.text("finding")?;
            if !(f.len() > 1 && f.starts_with('#') && f[1..].bytes().all(|b| b.is_ascii_digit())) {
                return Err(format!(
                    "`{}` is `{f}`; it must be an issue number such as #282",
                    o.path("finding")
                ));
            }
            Some(f)
        }
    };
    Ok(Rule {
        id,
        title: o.text("title")?,
        origin,
        source,
        codified,
        verified,
        applies_when: applicability(o.get("applies_when"), &o.path("applies_when"))?,
        kind,
        measure,
        compare,
        limit,
        tolerance,
        consequence,
        sizes,
        evaluated,
        gap,
        finding,
        since: semver(&o, "since")?,
    })
}

fn parse_limit(kind: Kind, v: &Value, at: &str) -> Result<Limit, String> {
    let o = Obj::of(v, at)?;
    let unit = o.text("unit")?;
    match kind {
        Kind::Limit => {
            o.only(&["value", "unit"])?;
            Ok(Limit::Constant {
                value: o.number("value")?,
                unit,
            })
        }
        Kind::Table => {
            o.only(&["table", "at", "unit"])?;
            Ok(Limit::Table {
                table: o.text("table")?,
                at: o.text("at")?,
                unit,
            })
        }
        Kind::Formula => {
            let shape = o.text("shape")?;
            let mut coefficients = Vec::new();
            let mut operands = Vec::new();
            for (name, value) in o.map {
                if name == "shape" || name == "unit" {
                    continue;
                }
                let path = o.path(name);
                match value {
                    Value::String(s) if !s.is_empty() => {
                        operands.push((name.clone(), Operand::Measure(s.clone())));
                    }
                    Value::Object(_) => {
                        let t = Obj::of(value, &path)?;
                        t.only(&["table", "at"])?;
                        operands.push((
                            name.clone(),
                            Operand::TableAt {
                                table: t.text("table")?,
                                at: t.text("at")?,
                            },
                        ));
                    }
                    v => coefficients.push((name.clone(), number(v, &path)?)),
                }
            }
            coefficients.sort_by(|a, b| a.0.cmp(&b.0));
            operands.sort_by(|a, b| a.0.cmp(&b.0));
            Ok(Limit::Formula {
                shape,
                unit,
                coefficients,
                operands,
            })
        }
        Kind::Check | Kind::Note => Err(format!("`{at}`: a `{}` Rule has no limit", kind.key())),
    }
}

fn parse_table(v: &Value, at: &str) -> Result<Table, String> {
    let o = Obj::of(v, at)?;
    o.only(&["id", "source", "input", "unit", "rows"])?;
    let id = o.text("id")?;
    let input = Obj::of(o.need("input")?, &o.path("input"))?;
    input.only(&["name", "unit"])?;
    let mut rows = Vec::new();
    for (i, r) in o.array("rows")?.iter().enumerate() {
        let r = Obj::of(r, &format!("{at}.rows[{i}]"))?;
        let w = Obj::of(r.need("when")?, &r.path("when"))?;
        let when = if let Some(list) = w.get("in") {
            w.only(&["in"])?;
            let texts: Vec<String> = list
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            if texts.is_empty() {
                return Err(format!("`{}` must be a list of text values", w.path("in")));
            }
            RowWhen::OneOf(texts)
        } else {
            w.only(&["ge", "gt", "le", "lt"])?;
            let mut bounds = Vec::new();
            for c in Compare::ALL {
                if let Some(v) = w.get(c.key()) {
                    bounds.push(Bound {
                        compare: c,
                        value: number(v, &w.path(c.key()))?,
                    });
                }
            }
            if bounds.is_empty() {
                return Err(format!(
                    "`{}` needs a bound (ge, gt, le, lt) or a list (in)",
                    w.here()
                ));
            }
            RowWhen::Bounds(bounds)
        };
        let mut coefficients = Vec::new();
        for (name, value) in r.map {
            if name == "when" || name == "shape" {
                continue;
            }
            coefficients.push((name.clone(), number(value, &r.path(name))?));
        }
        coefficients.sort_by(|a, b| a.0.cmp(&b.0));
        rows.push(TableRow {
            when,
            shape: r.text("shape")?,
            coefficients,
        });
    }
    if rows.is_empty() {
        return Err(format!("`{}` has no rows", o.path("rows")));
    }
    Ok(Table {
        id,
        source: citation(o.need("source")?, &o.path("source"))?,
        input: input.text("name")?,
        input_unit: input.text_or_empty("unit")?,
        unit: o.text("unit")?,
        rows,
    })
}

// --- checks a schema cannot make ------------------------------------------------------------------

/// `[a-z][a-z0-9-]*`.
fn is_word(s: &str) -> bool {
    let mut b = s.bytes();
    matches!(b.next(), Some(c) if c.is_ascii_lowercase())
        && b.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

/// The rule id grammar of spec 0004:
/// `^[a-z][a-z0-9-]*/[A-Za-z0-9][A-Za-z0-9.()-]*/[a-z0-9][a-z0-9-]*$`.
pub fn is_rule_id(id: &str) -> bool {
    let parts: Vec<&str> = id.split('/').collect();
    let [family, paragraph, slug] = parts.as_slice() else {
        return false;
    };
    let mut p = paragraph.bytes();
    let mut s = slug.bytes();
    is_word(family)
        && matches!(p.next(), Some(c) if c.is_ascii_alphanumeric())
        && p.all(|c| c.is_ascii_alphanumeric() || b".()-".contains(&c))
        && matches!(s.next(), Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit())
        && s.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

/// `major.minor.patch`, digits only.
fn is_semver(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

/// The checks across fields and Rules.
fn check_set(set: &RuleSet) -> Result<(), String> {
    let mut seen: Vec<&str> = Vec::new();
    for t in &set.tables {
        if seen.contains(&t.id.as_str()) {
            return Err(format!("table `{}` is defined twice", t.id));
        }
        seen.push(&t.id);
    }
    let mut ids: Vec<&str> = Vec::new();
    for r in &set.rules {
        if ids.contains(&r.id.as_str()) {
            return Err(format!("rule id `{}` is used twice in the set", r.id));
        }
        ids.push(&r.id);
        if r.family() != set.family {
            return Err(format!(
                "rule `{}`: its family `{}` is not the set's family `{}`",
                r.id,
                r.family(),
                set.family
            ));
        }
        if r.origin.is_published() {
            if r.source.locator.trim().is_empty() {
                return Err(format!(
                    "rule `{}`: a `{}` Rule must cite its paragraph (`source.locator`)",
                    r.id,
                    r.origin.key()
                ));
            }
            let locator: String = r
                .source
                .locator
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect();
            if locator != r.paragraph() {
                return Err(format!(
                    "rule `{}`: the paragraph segment `{}` does not match the locator `{}`",
                    r.id,
                    r.paragraph(),
                    r.source.locator
                ));
            }
        }
        let tables: Vec<&str> = match &r.limit {
            Some(Limit::Table { table, .. }) => vec![table.as_str()],
            Some(Limit::Formula { operands, .. }) => operands
                .iter()
                .filter_map(|(_, o)| match o {
                    Operand::TableAt { table, .. } => Some(table.as_str()),
                    Operand::Measure(_) => None,
                })
                .collect(),
            _ => Vec::new(),
        };
        for t in tables {
            if set.table(t).is_none() {
                return Err(format!(
                    "rule `{}` names table `{t}`, which the set does not define",
                    r.id
                ));
            }
        }
    }
    for k in &set.known_conflicts {
        for id in &k.rules {
            if set.rule(id).is_none() {
                return Err(format!(
                    "`known_conflicts` names rule `{id}`, which the set does not have"
                ));
            }
        }
    }
    for (a, b) in constant_conflicts(&set.rules, &set.rules) {
        let declared = set.known_conflicts.iter().any(|k| {
            (k.rules[0] == a.id && k.rules[1] == b.id) || (k.rules[0] == b.id && k.rules[1] == a.id)
        });
        if !declared {
            return Err(format!(
                "rules `{}` and `{}` cannot both be met (spec 0004, Coherence); declare the pair in `known_conflicts` with a reason, or fix a limit",
                a.id, b.id
            ));
        }
    }
    Ok(())
}

/// The Measure checks, against the registry (`ovcd rules --measures`): each Measure id is known
/// (a Rule that is not evaluated may name one the engine lacks), its arguments are the ones it
/// takes, a `check` Rule names a yes/no Measure, a limit's unit is the Measure's unit, and every
/// table row shape and formula shape is a known one with its coefficients and operands.
fn check_measures(set: &RuleSet) -> Result<(), String> {
    check_applicability(&set.applies_when, "the set")?;
    for r in &set.rules {
        check_applicability(&r.applies_when, &format!("rule `{}`", r.id))?;
    }
    for t in &set.tables {
        for (i, row) in t.rows.iter().enumerate() {
            let Some((_, coeffs)) = ROW_SHAPES.iter().find(|(s, _)| *s == row.shape) else {
                return Err(format!(
                    "table `{}` row {i}: unknown row shape `{}`",
                    t.id, row.shape
                ));
            };
            let got: Vec<&str> = row.coefficients.iter().map(|(k, _)| k.as_str()).collect();
            if !same_names(&got, coeffs) {
                return Err(format!(
                    "table `{}` row {i}: shape `{}` takes the coefficients {}",
                    t.id,
                    row.shape,
                    coeffs.join(", ")
                ));
            }
        }
    }
    for r in &set.rules {
        let Some(info) = measure_info(&r.measure.id) else {
            if r.evaluated {
                return Err(format!(
                    "rule `{}`: unknown Measure `{}` (`ovcd rules --measures` lists them)",
                    r.id, r.measure.id
                ));
            }
            continue;
        };
        let names: Vec<&str> = r.measure.args.iter().map(|(k, _)| k.as_str()).collect();
        let wanted: Vec<&str> = info.args.iter().map(|a| a.name).collect();
        if !same_names(&names, &wanted) {
            return Err(format!(
                "rule `{}`: Measure `{}` takes the arguments [{}]",
                r.id,
                r.measure.id,
                wanted.join(", ")
            ));
        }
        for (name, value) in &r.measure.args {
            let text = info.args.iter().any(|a| a.name == name && a.text);
            if text != matches!(value, ArgValue::Text(_)) {
                return Err(format!(
                    "rule `{}`: argument `{name}` of `{}` must be {}",
                    r.id,
                    r.measure.id,
                    if text { "text" } else { "a number" }
                ));
            }
        }
        match r.kind {
            Kind::Check if info.value != ValueKind::Bool => {
                return Err(format!(
                    "rule `{}`: a `check` Rule needs a yes/no Measure; `{}` is not one",
                    r.id, r.measure.id
                ));
            }
            Kind::Limit | Kind::Table | Kind::Formula if info.value != ValueKind::Number => {
                return Err(format!(
                    "rule `{}`: a limit needs a number Measure; `{}` is not one",
                    r.id, r.measure.id
                ));
            }
            _ => {}
        }
        let Some(limit) = &r.limit else {
            continue;
        };
        let unit = match limit {
            Limit::Constant { unit, .. }
            | Limit::Table { unit, .. }
            | Limit::Formula { unit, .. } => unit,
        };
        if *unit != info.unit {
            return Err(format!(
                "rule `{}`: the limit is in `{unit}`, but `{}` is in `{}`",
                r.id, r.measure.id, info.unit
            ));
        }
        let table_at = |t: &str, at: &str| -> Result<(), String> {
            let table = set.table(t).expect("checked in check_set");
            if table.unit != *unit {
                return Err(format!(
                    "rule `{}`: table `{t}` is in `{}`, the limit in `{unit}`",
                    r.id, table.unit
                ));
            }
            operand(r, info.object, at, Some(&table.input_unit))
        };
        match limit {
            Limit::Constant { .. } => {}
            Limit::Table { table, at, .. } => table_at(table, at)?,
            Limit::Formula {
                shape,
                coefficients,
                operands,
                ..
            } => {
                let Some((_, ops, coeffs)) = FORMULA_SHAPES.iter().find(|(s, ..)| s == shape)
                else {
                    return Err(format!("rule `{}`: unknown formula shape `{shape}`", r.id));
                };
                let got: Vec<&str> = operands.iter().map(|(k, _)| k.as_str()).collect();
                let got_c: Vec<&str> = coefficients.iter().map(|(k, _)| k.as_str()).collect();
                if !same_names(&got, ops) || !same_names(&got_c, coeffs) {
                    return Err(format!(
                        "rule `{}`: formula `{shape}` takes the operands [{}] and the coefficients [{}]",
                        r.id,
                        ops.join(", "),
                        coeffs.join(", ")
                    ));
                }
                for (name, op) in operands {
                    let free = FREE_UNIT_OPERANDS.contains(&(shape.as_str(), name.as_str()));
                    match op {
                        // The coefficients convert this operand, so any number Measure will do.
                        Operand::Measure(id) if free => operand(r, info.object, id, None)?,
                        Operand::Measure(id) => operand(r, info.object, id, Some(unit))?,
                        Operand::TableAt { table, at } => table_at(table, at)?,
                    }
                }
            }
        }
    }
    Ok(())
}

/// Each input an Applicability names can be named (a choice input, `cargo`, or a derived input), and
/// each value is one of its values. So a typo never makes a Rule silently not apply.
fn check_applicability(a: &Applicability, whose: &str) -> Result<(), String> {
    for (input, values) in &a.conditions {
        let known = applicability_values(input).ok_or_else(|| {
            format!("{whose}: `applies_when` names `{input}`, which is not an input it can name")
        })?;
        if let Some(v) = values.iter().find(|v| !known.contains(v)) {
            return Err(format!(
                "{whose}: `applies_when.{input}` has `{v}`; its values are {}",
                known.join(", ")
            ));
        }
    }
    Ok(())
}

/// A Measure a limit reads (a formula operand, a table key): known, with no arguments, a number in
/// `unit` (`Some("")`: a text key; `None`: a number in any unit), and on the Rule's object or on
/// the Option.
fn operand(r: &Rule, object: ObjectType, id: &str, unit: Option<&str>) -> Result<(), String> {
    let info = measure_info(id)
        .ok_or_else(|| format!("rule `{}`: its limit reads unknown Measure `{id}`", r.id))?;
    if !info.args.is_empty() {
        return Err(format!(
            "rule `{}`: its limit reads `{id}`, which takes arguments",
            r.id
        ));
    }
    let unit_ok = match unit {
        Some("") => info.value == ValueKind::Text,
        Some(u) => info.value == ValueKind::Number && info.unit == u,
        None => info.value == ValueKind::Number,
    };
    if !unit_ok {
        return Err(format!(
            "rule `{}`: its limit reads `{id}` in `{}`, where `{}` is needed",
            r.id,
            info.unit,
            unit.unwrap_or("a number")
        ));
    }
    if info.object != ObjectType::Option && info.object != object {
        return Err(format!(
            "rule `{}`: its limit reads `{id}`, a Measure of another object",
            r.id
        ));
    }
    Ok(())
}

fn same_names(got: &[&str], wanted: &[&str]) -> bool {
    got.len() == wanted.len() && wanted.iter().all(|w| got.contains(w))
}

/// The effective bound of a constant Criterion: limit − tolerance for a lower bound, limit +
/// tolerance for an upper bound.
fn constant_bound(r: &Rule) -> Option<(Compare, f64)> {
    match (r.compare, &r.limit) {
        (Some(c), Some(Limit::Constant { value, .. })) => Some((
            c,
            if c.is_lower_bound() {
                value - r.tolerance
            } else {
                value + r.tolerance
            },
        )),
        _ => None,
    }
}

/// Pairs (lower bound, upper bound) of constant Criteria on the same Measure instance, whose
/// Applicabilities overlap, that no value can meet together: the lower effective bound is above the
/// upper, or equal with a strict side.
fn constant_conflicts<'a>(lows: &'a [Rule], highs: &'a [Rule]) -> Vec<(&'a Rule, &'a Rule)> {
    let mut out = Vec::new();
    for a in lows {
        let Some((ca, ba)) = constant_bound(a) else {
            continue;
        };
        if !ca.is_lower_bound() {
            continue;
        }
        for b in highs {
            let Some((cb, bb)) = constant_bound(b) else {
                continue;
            };
            if cb.is_lower_bound()
                || a.measure != b.measure
                || !a.applies_when.overlaps(&b.applies_when)
            {
                continue;
            }
            if ba > bb || (ba == bb && (ca.is_strict() || cb.is_strict())) {
                out.push((a, b));
            }
        }
    }
    out
}

// --- the content hash ---------------------------------------------------------------------------

fn applicability_json(a: &Applicability) -> Value {
    Value::Object(
        a.conditions
            .iter()
            .map(|(k, v)| (k.clone(), json!(v)))
            .collect(),
    )
}

fn pairs_json(pairs: &[(String, f64)]) -> Value {
    Value::Object(pairs.iter().map(|(k, v)| (k.clone(), json!(v))).collect())
}

fn limit_json(l: &Limit) -> Value {
    match l {
        Limit::Constant { value, unit } => json!({"value": value, "unit": unit}),
        Limit::Table { table, at, unit } => json!({"table": table, "at": at, "unit": unit}),
        Limit::Formula {
            shape,
            unit,
            coefficients,
            operands,
        } => {
            let ops: Map<String, Value> = operands
                .iter()
                .map(|(k, o)| {
                    let v = match o {
                        Operand::Measure(m) => json!(m),
                        Operand::TableAt { table, at } => json!({"table": table, "at": at}),
                    };
                    (k.clone(), v)
                })
                .collect();
            json!({"shape": shape, "unit": unit, "coefficients": pairs_json(coefficients), "operands": ops})
        }
    }
}

/// The result-changing content of a set, as JSON (spec 0004, "Content hash"): rule ids,
/// `applies_when` of the set and of each Rule, `kind`, `measure`, `compare`, `limit`, `tolerance`,
/// `consequence`, `sizes`, `evaluated`, the tables (without their citations) and `includes`.
/// Titles, citations, `codified`, `verified`, gaps, findings and `since` are left out.
pub fn content_json(set: &RuleSet) -> Value {
    let rules: Vec<Value> = set.rules.iter().map(rule_content_json).collect();
    let tables: Vec<Value> = set
        .tables
        .iter()
        .map(|t| {
            let rows: Vec<Value> = t
                .rows
                .iter()
                .map(|r| {
                    let when = match &r.when {
                        RowWhen::Bounds(bs) => Value::Object(
                            bs.iter()
                                .map(|b| (b.compare.key().to_string(), json!(b.value)))
                                .collect(),
                        ),
                        RowWhen::OneOf(v) => json!({"in": v}),
                    };
                    json!({"when": when, "shape": r.shape, "coefficients": pairs_json(&r.coefficients)})
                })
                .collect();
            json!({"id": t.id, "input": t.input, "input_unit": t.input_unit, "unit": t.unit, "rows": rows})
        })
        .collect();
    let includes: Vec<Value> = set
        .includes
        .iter()
        .map(|i| json!({"id": i.id, "version": i.version}))
        .collect();
    json!({
        "applies_when": applicability_json(&set.applies_when),
        "includes": includes,
        "tables": tables,
        "rules": rules,
    })
}

/// The result-changing content of one Rule: two Rules with the same id are the same Rule when this
/// is equal.
pub fn rule_content_json(r: &Rule) -> Value {
    let args: Map<String, Value> = r
        .measure
        .args
        .iter()
        .map(|(k, v)| {
            let v = match v {
                ArgValue::Number(x) => json!(x),
                ArgValue::Text(s) => json!(s),
            };
            (k.clone(), v)
        })
        .collect();
    json!({
        "id": r.id,
        "applies_when": applicability_json(&r.applies_when),
        "kind": r.kind.key(),
        "measure": {"id": r.measure.id, "args": args},
        "compare": r.compare.map(Compare::key),
        "limit": r.limit.as_ref().map(limit_json),
        "tolerance": r.tolerance,
        "consequence": r.consequence.key(),
        "sizes": r.sizes,
        "evaluated": r.evaluated,
    })
}

/// SHA-256 of the canonical JSON of [`content_json`], in hex.
pub fn content_hash(set: &RuleSet) -> String {
    Sha256::digest(canonical_json(&content_json(set)).as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

// --- the library and the selection -------------------------------------------------------------

/// The Rule Sets known to a run: the built-in sets first, then each loaded file in order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Library {
    sets: Vec<RuleSet>,
    /// Warnings from loading (unverified blocking Rules).
    pub warnings: Vec<String>,
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

    /// Add a loaded set. A set whose id and version are already known with the same hash is the
    /// same set (nothing changes); with another hash it is refused, because a changed set needs a
    /// new version. The `includes` targets are checked by [`Library::check`] and [`Library::select`],
    /// so files can load in any order.
    pub fn add(&mut self, loaded: Loaded) -> Result<(), String> {
        let set = loaded.set;
        if let Some(known) = self.find(&set.id, &set.version) {
            if known.hash == set.hash {
                return Ok(());
            }
            return Err(format!(
                "{} is already loaded with other content (hash {} against {}); a changed Rule Set needs a new version",
                set.key(),
                short(&set.hash),
                short(&known.hash)
            ));
        }
        self.warnings.extend(loaded.warnings);
        self.sets.push(set);
        Ok(())
    }

    /// Load a file's text and add it. Returns the set's `id@version`.
    pub fn add_text(&mut self, text: &str) -> Result<String, String> {
        let loaded = load(text)?;
        let key = loaded.set.key();
        self.add(loaded)?;
        Ok(key)
    }

    /// Check every set on its own: each `includes` target is loaded, and no set includes itself
    /// through others. Run it once every file is loaded.
    pub fn check(&self) -> Result<(), String> {
        for s in &self.sets {
            self.select(&[(&s.id, &s.version)])?;
        }
        Ok(())
    }

    /// Every set, in loading order.
    pub fn sets(&self) -> &[RuleSet] {
        &self.sets
    }

    pub fn find(&self, id: &str, version: &str) -> Option<&RuleSet> {
        self.sets
            .iter()
            .find(|s| s.id == id && s.version == version)
    }

    /// The highest version of a set id.
    pub fn latest(&self, id: &str) -> Option<&RuleSet> {
        self.sets
            .iter()
            .filter(|s| s.id == id)
            .max_by_key(|s| version_key(&s.version))
    }

    /// `id` (its highest version) or `id@version`.
    pub fn lookup(&self, spec: &str) -> Option<&RuleSet> {
        match spec.split_once('@') {
            Some((id, version)) => self.find(id, version),
            None => self.latest(spec),
        }
    }

    /// The selection of the given sets, closed under `includes`, in selection order: each set,
    /// then the sets it includes just after it. Refused for an include cycle, two versions of one
    /// set id, or one rule id with two different contents. Conflicts between constant Criteria of
    /// two different sets are warnings: the user chose both sets.
    pub fn select(&self, chosen: &[(&str, &str)]) -> Result<Selection<'_>, String> {
        let mut order: Vec<&RuleSet> = Vec::new();
        for (id, version) in chosen {
            let set = self
                .find(id, version)
                .ok_or_else(|| format!("Rule Set {id}@{version} is not loaded"))?;
            self.visit(set, &mut Vec::new(), &mut order)?;
        }
        for (i, a) in order.iter().enumerate() {
            if let Some(b) = order[..i].iter().find(|b| b.id == a.id) {
                return Err(format!(
                    "two versions of Rule Set `{}` are selected: {} and {}",
                    a.id,
                    b.key(),
                    a.key()
                ));
            }
        }
        for (i, a) in order.iter().enumerate() {
            for b in &order[..i] {
                for ra in &a.rules {
                    if let Some(rb) = b.rule(&ra.id)
                        && rule_content_json(ra) != rule_content_json(rb)
                    {
                        return Err(format!(
                            "rule `{}` means different things in {} and {}",
                            ra.id,
                            b.key(),
                            a.key()
                        ));
                    }
                }
            }
        }
        let mut warnings = Vec::new();
        for (i, a) in order.iter().enumerate() {
            for b in &order[..i] {
                let pairs = constant_conflicts(&a.rules, &b.rules)
                    .into_iter()
                    .chain(constant_conflicts(&b.rules, &a.rules));
                for (lo, hi) in pairs {
                    if lo.id == hi.id {
                        continue;
                    }
                    warnings.push(format!(
                        "rules `{}` and `{}` of {} and {} cannot both be met; both are checked",
                        lo.id,
                        hi.id,
                        b.key(),
                        a.key()
                    ));
                }
            }
        }
        Ok(Selection {
            sets: order,
            warnings,
        })
    }

    fn visit<'a>(
        &'a self,
        set: &'a RuleSet,
        path: &mut Vec<String>,
        order: &mut Vec<&'a RuleSet>,
    ) -> Result<(), String> {
        let key = set.key();
        if path.contains(&key) {
            path.push(key);
            return Err(format!(
                "the Rule Sets include each other: {}",
                path.join(" -> ")
            ));
        }
        if order.iter().any(|s| s.key() == key) {
            return Ok(());
        }
        order.push(set);
        path.push(key);
        for inc in &set.includes {
            let target = self.find(&inc.id, &inc.version).ok_or_else(|| {
                format!(
                    "{} includes {}@{}, which is not loaded",
                    set.key(),
                    inc.id,
                    inc.version
                )
            })?;
            self.visit(target, path, order)?;
        }
        path.pop();
        Ok(())
    }
}

/// The selected sets, in selection order, with the warnings of the selection.
#[derive(Debug, Clone, PartialEq)]
pub struct Selection<'a> {
    pub sets: Vec<&'a RuleSet>,
    pub warnings: Vec<String>,
}

fn version_key(v: &str) -> (u64, u64, u64) {
    let mut it = v.split('.').map(|p| p.parse::<u64>().unwrap_or(0));
    (
        it.next().unwrap_or(0),
        it.next().unwrap_or(0),
        it.next().unwrap_or(0),
    )
}

fn short(hash: &str) -> &str {
    &hash[..hash.len().min(12)]
}

/// A plain-text listing of one set and its Rules, for `ovcd rules show`.
pub fn describe(set: &RuleSet) -> String {
    let mut out = format!(
        "{} {}: {}\nfamily {}, status {}, hash {}\n",
        set.id,
        set.version,
        set.title,
        set.family,
        set.status.key(),
        set.hash
    );
    for (k, v) in &set.edition {
        out.push_str(&format!("edition {k}: {v}\n"));
    }
    for inc in &set.includes {
        out.push_str(&format!("includes {}@{}\n", inc.id, inc.version));
    }
    let unverified = set.rules.iter().filter(|r| r.verified.is_none()).count();
    out.push_str(&format!(
        "{} Rules, {} unverified\n\n",
        set.rules.len(),
        unverified
    ));
    for r in &set.rules {
        let test = match (&r.compare, &r.limit) {
            (Some(c), Some(Limit::Constant { value, unit })) => {
                format!("{} {value} {unit}", c.key())
            }
            (Some(c), Some(Limit::Table { table, at, unit })) => {
                format!("{} table {table} at {at} ({unit})", c.key())
            }
            (Some(c), Some(Limit::Formula { shape, unit, .. })) => {
                format!("{} formula {shape} ({unit})", c.key())
            }
            _ => r.kind.key().to_string(),
        };
        let status = match &r.verified {
            Some(v) => format!("verified by {} on {}", v.by, v.date),
            None => "unverified".to_string(),
        };
        let evaluated = if r.evaluated { "" } else { ", not evaluated" };
        out.push_str(&format!(
            "{}\n  {}\n  {} | {} {} | {} | {}{}\n",
            r.id,
            r.title,
            r.origin.key(),
            r.measure.id,
            test,
            r.consequence.key(),
            status,
            evaluated
        ));
        if !r.source.locator.is_empty() {
            out.push_str(&format!(
                "  cites {} {}\n",
                r.source.document, r.source.locator
            ));
        }
    }
    out
}
