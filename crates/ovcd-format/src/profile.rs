//! The Defaults Profile: the default value and the allowed ranges of every input an Option may
//! leave out. A built-in profile ships with the tool; a user may copy it, edit it and load the copy.
//!
//! The profile holds values only. Which inputs exist, and their types, is fixed by the schema in
//! `vdesign.rs`.

use crate::vdesign::{INPUTS, InputKind, KEYED_DEFAULTS};
use serde_json::{Map, Value};

const BUILT_IN: &str = include_str!("builtin_profile.json");

/// A loaded, checked Defaults Profile.
#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    name: String,
    inputs: Map<String, Value>,
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
        let profile = Profile::unchecked(text)?;
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

    pub fn name(&self) -> &str {
        &self.name
    }

    fn check(&self) -> Result<(), String> {
        for key in self.inputs.keys() {
            let known = INPUTS.iter().any(|s| s.name == key)
                || key == "formation_abreast"
                || key == "formation_in_line";
            if !known {
                return Err(format!("profile has an unknown input `{key}`"));
            }
        }
        for spec in INPUTS.iter().filter(|s| !s.required) {
            let entry = self
                .inputs
                .get(spec.name)
                .ok_or_else(|| format!("profile has no entry for `{}`", spec.name))?;
            let entry = entry
                .as_object()
                .ok_or_else(|| format!("profile `{}` must be an object", spec.name))?;
            // Every lookup the schema can make must find a value; a gap would leave an
            // Option silently short of an input.
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
            .chain(["formation_abreast", "formation_in_line"]);
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

    /// The plain default of an input, if the profile gives one.
    pub fn default(&self, name: &str) -> Option<&Value> {
        self.entry(name).and_then(|e| e.get("default"))
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
