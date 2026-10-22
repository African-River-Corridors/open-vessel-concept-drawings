//! The Defaults Profile: the default value and the allowed ranges of every input an Option may
//! leave out. A built-in profile ships with the tool; a user may copy it, edit it and load the copy.
//!
//! The profile holds values only. Which inputs exist, and their types, is fixed by the schema in
//! `vdesign.rs`.

use crate::vdesign::{
    INPUTS, InputKind, KEYED_DEFAULTS, LEGACY_FIXED_INPUTS, LEGACY_FIXED_OTHER, TABLE_DEFAULTS,
    TABLE_KEYS, TABLE_STATUSES, table_combinations,
};
use serde_json::{Map, Value};

const BUILT_IN: &str = include_str!("builtin_profile.json");

/// A loaded, checked Defaults Profile.
#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    name: String,
    inputs: Map<String, Value>,
}

/// The row of a default table that gave a value (see [`Profile::table_default`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TableHit<'a> {
    pub value: &'a Value,
    /// `sourced` or `estimate`.
    pub status: &'a str,
    /// The published source, for a `sourced` value (and often an `estimate` too).
    pub source: Option<&'a str>,
    /// How the value was chosen, always given.
    pub basis: &'a str,
}

/// Whether a table row's `when` fits the context: each key it lists holds one of the values named.
fn row_fits(row: &Value, context: &[(&str, &str)]) -> bool {
    let Some(when) = row.get("when").and_then(Value::as_object) else {
        return false;
    };
    when.iter().all(|(key, wanted)| {
        let Some((_, have)) = context.iter().find(|(k, _)| k == key) else {
            return false;
        };
        match wanted {
            Value::String(w) => w == have,
            Value::Array(list) => list.iter().any(|w| w.as_str() == Some(have)),
            _ => false,
        }
    })
}

/// Check the default table of one input (issue #292): its shape, the status and source of every
/// row, and that every combination of Cargo Containment, Cargo and carriage condition the schema
/// can make has a row.
fn check_table(name: &str, entry: &Map<String, Value>) -> Result<(), String> {
    let table = entry
        .get("default_by_table")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            format!(
                "profile gives no default table for `{name}` (`{name}.default_by_table` is missing)"
            )
        })?;
    let by: Vec<&str> = table
        .get("by")
        .and_then(Value::as_array)
        .map(|l| l.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    if by != TABLE_KEYS {
        return Err(format!(
            "profile `{name}.default_by_table.by` must be {}",
            TABLE_KEYS.join(", ")
        ));
    }
    let rows = table
        .get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("profile `{name}.default_by_table.rows` must be a list"))?;
    for (i, row) in rows.iter().enumerate() {
        let at = format!("`{name}.default_by_table.rows[{i}]`");
        let row = row
            .as_object()
            .ok_or_else(|| format!("profile {at} must be an object"))?;
        let when = row
            .get("when")
            .and_then(Value::as_object)
            .ok_or_else(|| format!("profile {at} needs an object `when`"))?;
        for (key, wanted) in when {
            if !TABLE_KEYS.contains(&key.as_str()) {
                return Err(format!("profile {at}.when has an unknown key `{key}`"));
            }
            let ok = match wanted {
                Value::String(_) => true,
                Value::Array(l) => !l.is_empty() && l.iter().all(Value::is_string),
                _ => false,
            };
            if !ok {
                return Err(format!(
                    "profile {at}.when.{key} must be a name or a list of names"
                ));
            }
        }
        if !row.contains_key("value") {
            return Err(format!("profile {at} needs a `value`"));
        }
        let status = row.get("status").and_then(Value::as_str).unwrap_or("");
        if !TABLE_STATUSES.contains(&status) {
            return Err(format!(
                "profile {at} needs a `status` of {}",
                TABLE_STATUSES.join(" or ")
            ));
        }
        let text = |k: &str| {
            row.get(k)
                .and_then(Value::as_str)
                .is_some_and(|t| !t.trim().is_empty())
        };
        if !text("basis") {
            return Err(format!(
                "profile {at} needs a `basis`: how the value was chosen"
            ));
        }
        if status == "sourced" && !text("source") {
            return Err(format!(
                "profile {at} is `sourced` but gives no `source`; a value with no source is an `estimate`"
            ));
        }
    }
    for combination in table_combinations() {
        let context: Vec<(&str, &str)> =
            combination.iter().map(|(k, v)| (*k, v.as_str())).collect();
        if !rows.iter().any(|row| row_fits(row, &context)) {
            let shown: Vec<String> = combination
                .iter()
                .filter(|(_, v)| !v.is_empty())
                .map(|(k, v)| format!("{k} {v}"))
                .collect();
            return Err(format!(
                "profile gives no default for `{name}` when {} (no row of `{name}.default_by_table` fits)",
                shown.join(", ")
            ));
        }
    }
    Ok(())
}

/// A hard range: the bounds an input must meet or be refused.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct HardRange {
    pub gt: Option<f64>,
    pub ge: Option<f64>,
    pub lt: Option<f64>,
    pub le: Option<f64>,
}

impl HardRange {
    /// The tighter lower bound, as (value, strict).
    fn lower(&self) -> Option<(f64, bool)> {
        match (self.gt, self.ge) {
            (Some(g), Some(e)) if e > g => Some((e, false)),
            (Some(g), _) => Some((g, true)),
            (None, e) => e.map(|e| (e, false)),
        }
    }

    /// The tighter upper bound, as (value, strict).
    fn upper(&self) -> Option<(f64, bool)> {
        match (self.lt, self.le) {
            (Some(l), Some(e)) if e < l => Some((e, false)),
            (Some(l), _) => Some((l, true)),
            (None, e) => e.map(|e| (e, false)),
        }
    }

    /// Whether every value this range accepts, `base` accepts too.
    pub fn within(&self, base: &HardRange) -> bool {
        let lower_ok = match (base.lower(), self.lower()) {
            (None, _) => true,
            (Some(_), None) => false,
            (Some((b, b_strict)), Some((s, s_strict))) => {
                s > b || (s == b && (s_strict || !b_strict))
            }
        };
        let upper_ok = match (base.upper(), self.upper()) {
            (None, _) => true,
            (Some(_), None) => false,
            (Some((b, b_strict)), Some((s, s_strict))) => {
                s < b || (s == b && (s_strict || !b_strict))
            }
        };
        lower_ok && upper_ok
    }

    /// The bounds as text: ">= 0.3 and <= 1".
    fn describe(&self) -> String {
        [
            (self.gt, ">"),
            (self.ge, ">="),
            (self.lt, "<"),
            (self.le, "<="),
        ]
        .into_iter()
        .filter_map(|(b, op)| b.map(|b| format!("{op} {b}")))
        .collect::<Vec<_>>()
        .join(" and ")
    }

    /// The first bound `x` breaks, as text ("> 0"), or `None` when `x` is inside.
    pub fn breach(&self, x: f64) -> Option<String> {
        let checks = [
            (self.gt, ">", x > self.gt.unwrap_or(f64::NEG_INFINITY)),
            (self.ge, ">=", x >= self.ge.unwrap_or(f64::NEG_INFINITY)),
            (self.lt, "<", x < self.lt.unwrap_or(f64::INFINITY)),
            (self.le, "<=", x <= self.le.unwrap_or(f64::INFINITY)),
        ];
        checks
            .into_iter()
            .find(|(bound, _, ok)| bound.is_some() && !ok)
            .map(|(bound, op, _)| format!("{op} {}", bound.unwrap()))
    }
}

/// A soft range: the usual span. An input outside it is accepted with a warning.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoftRange {
    pub min: f64,
    pub max: f64,
}

impl Profile {
    /// The built-in Defaults Profile.
    pub fn built_in() -> Profile {
        Profile::from_json(BUILT_IN).expect("the built-in Defaults Profile is valid")
    }

    /// The built-in profile's text, for a user to copy and edit.
    pub fn built_in_text() -> &'static str {
        BUILT_IN
    }

    /// Load a profile and check it covers every input it must, with well-formed ranges.
    pub fn from_json(text: &str) -> Result<Profile, String> {
        let mut profile = Profile::unchecked(text)?;
        profile.fill_added_inputs()?;
        profile.check()?;
        Ok(profile)
    }

    /// Parse a profile without checking it.
    fn unchecked(text: &str) -> Result<Profile, String> {
        let root: Value =
            serde_json::from_str(text).map_err(|e| format!("profile is not JSON: {e}"))?;
        let name = root
            .get("name")
            .and_then(Value::as_str)
            .ok_or("profile needs a string `name`")?
            .to_string();
        let inputs = root
            .get("inputs")
            .and_then(Value::as_object)
            .ok_or("profile needs an object `inputs`")?
            .clone();
        Ok(Profile { name, inputs })
    }

    /// A profile written before an input existed has no entry for it. Such an input takes the
    /// built-in profile's entry (the inputs of `LEGACY_FIXED_INPUTS`), so an older copy still loads.
    fn fill_added_inputs(&mut self) -> Result<(), String> {
        let built_in = Profile::unchecked(BUILT_IN)?;
        let added = LEGACY_FIXED_INPUTS
            .iter()
            .map(|l| l.name)
            .chain(LEGACY_FIXED_OTHER.iter().map(|l| l.name))
            .chain(TABLE_DEFAULTS.iter().copied())
            // Issue #275: a copy written before the default stacks has none.
            .chain(["rule_sets"]);
        for name in added {
            if !self.inputs.contains_key(name)
                && let Some(entry) = built_in.inputs.get(name)
            {
                self.inputs.insert(name.to_string(), entry.clone());
            }
        }
        // An input that now has a default table (issue #292) but whose entry in an older copy has
        // none takes the built-in table. A copy's own `fill_ratio` default (it was keyed by
        // ruleset) is kept: every row takes that value, marked an estimate from the copy.
        for name in TABLE_DEFAULTS {
            let Some(built) = built_in.inputs.get(*name).and_then(Value::as_object) else {
                continue;
            };
            let Some(entry) = self.inputs.get_mut(*name).and_then(Value::as_object_mut) else {
                continue;
            };
            if entry.contains_key("default_by_table") {
                continue;
            }
            let mut table = built
                .get("default_by_table")
                .cloned()
                .unwrap_or(Value::Null);
            let own = entry
                .get("default_by_ruleset")
                .and_then(|m| m.get("IGC_2G"))
                .and_then(Value::as_f64);
            if *name == "fill_ratio"
                && let Some(own) = own
                && let Some(rows) = table.get_mut("rows").and_then(Value::as_array_mut)
            {
                for row in rows.iter_mut().filter_map(Value::as_object_mut) {
                    if row.get("value").and_then(Value::as_f64) != Some(own) {
                        row.insert("value".into(), Value::from(own));
                        row.insert("status".into(), Value::from("estimate"));
                        row.remove("source");
                        row.insert(
                            "basis".into(),
                            Value::from(format!(
                                "the fill ratio of this profile copy (`default_by_ruleset.IGC_2G` = {own}), \
                                 which was one value for every Cargo before the table existed"
                            )),
                        );
                    }
                }
            }
            entry.insert("default_by_table".into(), table);
        }
        Ok(())
    }

    /// The default stacks (issue #275): each Service Area has a `gas` and an `other` list of set ids,
    /// and the `ovcd` set is in every one.
    fn check_default_stacks(&self) -> Result<(), String> {
        for area in crate::vdesign::SERVICE_AREAS {
            for kind in [true, false] {
                let stack = self.default_stack(area, kind).ok_or_else(|| {
                    format!(
                        "profile gives no default Rule Set stack for {area}, {} \
                         (`rule_sets.default_stacks.{area}.{}` is missing)",
                        if kind { "gas" } else { "other" },
                        if kind { "gas" } else { "other" }
                    )
                })?;
                if !stack.iter().any(|s| s == "ovcd") {
                    return Err(format!(
                        "profile default Rule Set stack for {area} must hold the `ovcd` set"
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn check(&self) -> Result<(), String> {
        for key in self.inputs.keys() {
            let known = INPUTS.iter().any(|s| s.name == key)
                || key == "formation_abreast"
                || key == "formation_in_line"
                || key == "depth_to_beam_ratio"
                || key == "length_to_depth_ratio";
            if !known {
                return Err(format!("profile has an unknown input `{key}`"));
            }
        }
        for spec in INPUTS.iter().filter(|s| {
            !s.required
                    && !crate::class_input::UNSET_WHEN_ABSENT.contains(&s.name)
                    // A rule, not a value: the depth is the ratio of the Cargo times the beam.
                    && s.name != crate::vdesign::DEPTH_INPUT
        }) {
            if spec.name == "rule_sets" {
                self.check_default_stacks()?;
                continue;
            }
            let entry = self
                .inputs
                .get(spec.name)
                .ok_or_else(|| format!("profile has no entry for `{}`", spec.name))?;
            let entry = entry
                .as_object()
                .ok_or_else(|| format!("profile `{}` must be an object", spec.name))?;
            // Every lookup the schema can make must find a value; a gap would leave an
            // Option silently short of an input.
            // Issue #347: a plain or keyed default may say whether it is sourced or an estimate.
            if let Some(status) = entry.get("status") {
                let status = status.as_str().unwrap_or("");
                if !TABLE_STATUSES.contains(&status) {
                    return Err(format!(
                        "profile `{}.status` must be one of {}, not `{status}`",
                        spec.name,
                        TABLE_STATUSES.join(", ")
                    ));
                }
                let text = |k: &str| entry.get(k).and_then(Value::as_str).unwrap_or("").trim();
                if status == "sourced" && text("source").is_empty() {
                    return Err(format!(
                        "profile `{}` says its default is sourced but gives no `source`",
                        spec.name
                    ));
                }
                if text("basis").is_empty() {
                    return Err(format!(
                        "profile `{}` gives a status but no `basis`",
                        spec.name
                    ));
                }
            }
            if TABLE_DEFAULTS.contains(&spec.name) {
                check_table(spec.name, entry)?;
                continue;
            }
            match KEYED_DEFAULTS.iter().find(|(n, _, _)| *n == spec.name) {
                Some((_, by, keys)) => {
                    let field = format!("default_by_{by}");
                    let table = entry.get(&field).and_then(Value::as_object);
                    for key in *keys {
                        if !table.is_some_and(|t| t.contains_key(*key)) {
                            return Err(format!(
                                "profile gives no default for `{}` when {by} is {key} \
                                 (`{}.{field}.{key}` is missing)",
                                spec.name, spec.name
                            ));
                        }
                    }
                }
                None => {
                    if !entry.contains_key("default") {
                        return Err(format!("profile gives no default for `{}`", spec.name));
                    }
                }
            }
        }
        // Hard ranges are physical limits and the only guard in front of the core, so a profile
        // may tighten one but never loosen it.
        let base = Profile::unchecked(BUILT_IN)?;
        let numbers = INPUTS
            .iter()
            .filter(|s| matches!(s.kind, InputKind::Number { .. }))
            .map(|s| s.name)
            .chain([
                "formation_abreast",
                "formation_in_line",
                "depth_to_beam_ratio",
                "length_to_depth_ratio",
            ]);
        for name in numbers {
            let (hard, base_hard) = (self.hard(name)?, base.hard(name)?);
            if !hard.within(&base_hard) {
                return Err(format!(
                    "profile loosens the hard range of `{name}`; it must be {} at least \
                     (a profile may tighten a hard range, never loosen it)",
                    base_hard.describe()
                ));
            }
            self.soft(name)?;
        }
        crate::vdesign::check_profile_defaults(self)
    }

    fn entry(&self, name: &str) -> Option<&Map<String, Value>> {
        self.inputs.get(name).and_then(Value::as_object)
    }

    /// The default stacks as the profile writes them: `{ <Service Area>: { gas: [..], other: [..] } }`.
    pub fn default_stacks_json(&self) -> Value {
        self.entry("rule_sets")
            .and_then(|e| e.get("default_stacks"))
            .cloned()
            .unwrap_or(Value::Null)
    }

    /// The default Rule Set stack for a Service Area and a Cargo (`gas`: a Type C tank Cargo
    /// Containment), as set ids in selection order (issue #275). `None` when the profile gives none.
    pub fn default_stack(&self, service_area: &str, gas: bool) -> Option<Vec<String>> {
        let list = self
            .entry("rule_sets")?
            .get("default_stacks")?
            .get(service_area)?
            .get(if gas { "gas" } else { "other" })?
            .as_array()?;
        list.iter().map(|v| v.as_str().map(String::from)).collect()
    }

    /// The plain default of an input, if the profile gives one.
    pub fn default(&self, name: &str) -> Option<&Value> {
        self.entry(name).and_then(|e| e.get("default"))
    }

    /// The rows of an input's default table (`default_by_table.rows`), empty when it has none.
    fn table_rows(&self, name: &str) -> &[Value] {
        self.entry(name)
            .and_then(|e| e.get("default_by_table"))
            .and_then(|t| t.get("rows"))
            .and_then(Value::as_array)
            .map_or(&[], Vec::as_slice)
    }

    /// The row of an input's default table that fits `context` (a value for each of
    /// [`TABLE_KEYS`]): the first whose `when` names every key it lists as one of the context's
    /// values. `None` when no row fits.
    pub fn table_default(&self, name: &str, context: &[(&str, &str)]) -> Option<TableHit<'_>> {
        self.table_rows(name)
            .iter()
            .find(|row| row_fits(row, context))
            .and_then(|row| {
                Some(TableHit {
                    value: row.get("value")?,
                    status: row.get("status").and_then(Value::as_str).unwrap_or(""),
                    source: row.get("source").and_then(Value::as_str),
                    basis: row.get("basis").and_then(Value::as_str).unwrap_or(""),
                })
            })
    }

    /// Every value in an input's default table, for the checks.
    pub fn table_values(&self, name: &str) -> Vec<Value> {
        self.table_rows(name)
            .iter()
            .filter_map(|r| r.get("value").cloned())
            .collect()
    }

    /// The rows of an input's default table as the schema shows them.
    pub fn table_json(&self, name: &str) -> Option<&Value> {
        self.entry(name).and_then(|e| e.get("default_by_table"))
    }

    /// A default chosen by another input's value: `default_by_<by>[key]`.
    pub fn default_by(&self, name: &str, by: &str, key: &str) -> Option<&Value> {
        self.entry(name)
            .and_then(|e| e.get(&format!("default_by_{by}")))
            .and_then(|m| m.get(key))
    }

    /// The unit text of an input ("m"), or "" when it has none.
    pub fn unit(&self, name: &str) -> &str {
        self.entry(name)
            .and_then(|e| e.get("unit"))
            .and_then(Value::as_str)
            .unwrap_or("")
    }

    /// The profile's note on where an input's ranges come from, or "" when it has none.
    /// The status of a plain or keyed default (issue #347): `sourced` with its source, or
    /// `estimate`; `None` when the entry gives none (a default table gives one per row).
    pub fn default_status(&self, name: &str) -> Option<(&str, Option<&str>)> {
        let e = self.entry(name)?;
        let status = e.get("status").and_then(Value::as_str)?;
        Some((status, e.get("source").and_then(Value::as_str)))
    }

    pub fn basis(&self, name: &str) -> &str {
        self.entry(name)
            .and_then(|e| e.get("basis"))
            .and_then(Value::as_str)
            .unwrap_or("")
    }

    pub fn hard(&self, name: &str) -> Result<HardRange, String> {
        let Some(h) = self.entry(name).and_then(|e| e.get("hard")) else {
            return Ok(HardRange::default());
        };
        let h = h
            .as_object()
            .ok_or_else(|| format!("profile `{name}.hard` must be an object"))?;
        let mut range = HardRange::default();
        for (k, v) in h {
            let x = v
                .as_f64()
                .ok_or_else(|| format!("profile `{name}.hard.{k}` must be a number"))?;
            match k.as_str() {
                "gt" => range.gt = Some(x),
                "ge" => range.ge = Some(x),
                "lt" => range.lt = Some(x),
                "le" => range.le = Some(x),
                _ => return Err(format!("profile `{name}.hard` has an unknown bound `{k}`")),
            }
        }
        Ok(range)
    }

    pub fn soft(&self, name: &str) -> Result<Option<SoftRange>, String> {
        let Some(s) = self.entry(name).and_then(|e| e.get("soft")) else {
            return Ok(None);
        };
        let bound = |k: &str| {
            s.get(k)
                .and_then(Value::as_f64)
                .ok_or_else(|| format!("profile `{name}.soft.{k}` must be a number"))
        };
        let (min, max) = (bound("min")?, bound("max")?);
        if min > max {
            return Err(format!("profile `{name}.soft` has min above max"));
        }
        Ok(Some(SoftRange { min, max }))
    }
}
