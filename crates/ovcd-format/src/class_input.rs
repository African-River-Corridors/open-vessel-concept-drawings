//! The Vessel Class inputs of an Option (issue #94): `vessel_scheme` and `vessel_class`.
//!
//! An Option is defined by typed dimensions (no scheme) or by a standard Vessel Class (a scheme).
//! The rules (the maintainer's answers on the issue; the classes and the ladder are in
//! `ovcd_core::vessel_classes`):
//!
//! - Length and beam decide the class. The Option keeps one: after every read the class is the
//!   smallest of the scheme's ladder (its Formation kind) whose length and beam maxima hold the
//!   vessel. Past the largest class a warning says so. The scheme never changes by itself.
//! - Picking a class sets length, beam and Formation (and a convoy's counts) from it. The file marks
//!   those inputs `class`. Depth stays typed.
//! - The class's draught fills the draught limit (the top of its range) and its air draft fills the
//!   air-draft limit (the bottom of its range, the lowest bridge clearance), marked `class`, unless
//!   the designer typed a limit; a typed limit above the class's is a warning. A class that gives no
//!   value fills nothing.
//! - The class's reference deadweight is shown beside the computed cargo and is never an input.
//!
//! Both inputs are absent from a file that uses typed dimensions; they have no Defaults Profile entry.

use crate::profile::Profile;
use crate::vdesign::{DesignOption, Report, Source, StoredInput, show};
use ovcd_core::vessel_classes::{self, ClassRow};
use serde_json::{Map, Value, json};

/// Inputs that a file holds only when it uses them: they have no default and are not "left out".
pub const UNSET_WHEN_ABSENT: &[&str] = &["vessel_scheme", "vessel_class"];

fn plain(v: &Value) -> &Value {
    v.as_object()
        .filter(|o| o.contains_key("source"))
        .and_then(|o| o.get("value"))
        .unwrap_or(v)
}

/// The Formation a vessel of this class has, as the file stores it.
pub fn formation_value(c: &ClassRow) -> Value {
    match c.kind() {
        Some("pushed_convoy") => {
            let (abreast, in_line) = c.counts();
            json!({ "kind": "pushed_convoy", "abreast": abreast, "in_line": in_line })
        }
        Some(kind) => json!({ "kind": kind }),
        None => Value::Null,
    }
}

/// The inputs a class sets when it is picked: length and beam (its maxima) and the Formation.
pub fn class_values(c: &ClassRow) -> Vec<(&'static str, Value)> {
    vec![
        ("length_overall_m", json!(c.loa_max_m().unwrap_or(f64::NAN))),
        ("beam_m", json!(c.beam_max_m().unwrap_or(f64::NAN))),
        ("formation", formation_value(c)),
    ]
}

const LIMIT_NAMES: [&str; 2] = ["draught_limit_m", "air_draft_limit_m"];

/// The limits a class fills: (input, class value).
pub fn class_limits(c: &ClassRow) -> [(&'static str, Option<f64>); 2] {
    [
        ("draught_limit_m", c.draught_limit_m()),
        ("air_draft_limit_m", c.air_draft_limit_m()),
    ]
}

/// Before the inputs are read, for `fill`: a class that is named with no length, beam or Formation
/// gives them (marked `class`), and names its scheme.
pub fn prefill(raw: &Map<String, Value>) -> Map<String, Value> {
    let mut out = raw.clone();
    let class = raw
        .get("vessel_class")
        .map(plain)
        .and_then(Value::as_str)
        .and_then(vessel_classes::find)
        .filter(|c| c.pickable());
    if let Some(c) = class {
        if !out.contains_key("vessel_scheme") {
            out.insert("vessel_scheme".into(), json!(c.family));
        }
        for (name, value) in class_values(c) {
            if !out.contains_key(name) {
                out.insert(name.into(), json!({ "value": value, "source": "class" }));
            }
        }
    }
    out
}

fn get<'a>(inputs: &'a [(String, StoredInput)], name: &str) -> Option<&'a StoredInput> {
    inputs.iter().find(|(n, _)| n == name).map(|(_, i)| i)
}

fn set(inputs: &mut Vec<(String, StoredInput)>, name: &str, value: Value, source: Source) {
    let stored = StoredInput { value, source };
    match inputs.iter_mut().find(|(n, _)| n == name) {
        Some((_, slot)) => *slot = stored,
        None => inputs.push((name.to_string(), stored)),
    }
}

/// An input that came from a class goes back to its default (a limit with no class value).
fn restore_default(inputs: &mut Vec<(String, StoredInput)>, name: &str, profile: &Profile) {
    if get(inputs, name).is_some_and(|i| i.source == Source::Class) {
        let value = profile.default(name).cloned().unwrap_or(Value::Null);
        set(inputs, name, value, Source::Default);
    }
}

/// After the inputs are read: place the vessel in a class, and fill the limits from it.
pub fn resolve(
    inputs: &mut Vec<(String, StoredInput)>,
    profile: &Profile,
    report: &mut Report,
    at: &dyn Fn(String) -> String,
) {
    let class_in = get(inputs, "vessel_class").cloned();
    let class_row = class_in
        .as_ref()
        .and_then(|i| i.value.as_str())
        .and_then(vessel_classes::find);
    let scheme = get(inputs, "vessel_scheme")
        .and_then(|i| i.value.as_str().map(String::from))
        .or_else(|| class_row.map(|c| c.family.to_string()));
    let Some(scheme) = scheme else {
        // Typed dimensions: nothing came from a class any more.
        for (_, i) in inputs.iter_mut() {
            if i.source == Source::Class {
                i.source = Source::Typed;
            }
        }
        inputs.retain(|(n, _)| n != "vessel_class");
        return;
    };
    if get(inputs, "vessel_scheme").is_none() {
        set(inputs, "vessel_scheme", json!(scheme), Source::Typed);
    }
    if let Some(c) = class_row
        && c.family != scheme
    {
        report.error(
            at(format!(
                "`vessel_class` {} belongs to the scheme {}, not {scheme}",
                c.id, c.family
            )),
            &["vessel_scheme", "vessel_class"],
        );
        return;
    }
    let kind = get(inputs, "formation")
        .and_then(|i| i.value.get("kind"))
        .and_then(Value::as_str)
        .map(String::from);
    let loa = get(inputs, "length_overall_m").and_then(|i| i.value.as_f64());
    let beam = get(inputs, "beam_m").and_then(|i| i.value.as_f64());
    let (Some(kind), Some(loa), Some(beam)) = (kind, loa, beam) else {
        return;
    };
    let Some(p) = vessel_classes::place(&scheme, &kind, loa, beam, class_row.map(|c| c.id)) else {
        inputs.retain(|(n, _)| n != "vessel_class");
        for name in LIMIT_NAMES {
            restore_default(inputs, name, profile);
        }
        report.warning(
            at(format!(
                "the scheme {scheme} has no class for a {kind} Formation, so no class applies"
            )),
            &["vessel_scheme", "formation"],
        );
        return;
    };
    let class = p.class;
    if let (Some(typed), Some(row)) = (&class_in, class_row)
        && typed.source == Source::Typed
        && row.id != class.id
    {
        report.warning(
            at(format!(
                "`vessel_class` {} does not hold this length and beam; the vessel sits in {}",
                row.id, class.id
            )),
            &["vessel_class", "length_overall_m", "beam_m"],
        );
    }
    set(inputs, "vessel_class", json!(class.id), Source::Class);
    if p.beyond_largest {
        report.warning(
            at(format!(
                "this length and beam are beyond the largest {scheme} class ({})",
                class.id
            )),
            &["length_overall_m", "beam_m", "vessel_scheme"],
        );
    }
    for (name, class_value) in class_limits(class) {
        let stored = get(inputs, name).cloned();
        let typed = stored.as_ref().filter(|i| i.source == Source::Typed);
        if let Some(typed) = typed {
            if let (Some(v), Some(limit)) = (typed.value.as_f64(), class_value)
                && v > limit
            {
                let what = if name == "draught_limit_m" {
                    format!(
                        "the draught of class {} ({} m, the top of its range)",
                        class.id,
                        show(&json!(limit))
                    )
                } else {
                    format!(
                        "the lowest bridge clearance of class {} ({} m)",
                        class.id,
                        show(&json!(limit))
                    )
                };
                report.warning(
                    at(format!("`{name}` = {} is above {what}", show(&typed.value))),
                    &[name],
                );
            }
        } else if let Some(limit) = class_value {
            set(inputs, name, json!(limit), Source::Class);
        } else {
            restore_default(inputs, name, profile);
        }
    }
}

/// What the Option's class is, for a front end: the class the vessel sits in and its reference
/// deadweight (shown beside the computed cargo, never an input). `None` for typed dimensions.
pub fn summary(option: &DesignOption) -> Option<Value> {
    let class = vessel_classes::find(option.input("vessel_class")?.value.as_str()?)?;
    let scheme = option.input("vessel_scheme")?.value.as_str()?.to_string();
    let loa = option.input("length_overall_m")?.value.as_f64()?;
    let beam = option.input("beam_m")?.value.as_f64()?;
    let kind = class.kind()?;
    let beyond = vessel_classes::place(&scheme, kind, loa, beam, Some(class.id))
        .is_some_and(|p| p.beyond_largest);
    let pair = |p: Option<(f64, f64)>| p.map_or(Value::Null, |(lo, hi)| json!([lo, hi]));
    Some(json!({
        "id": class.id,
        "scheme": scheme,
        "formation": class.formation,
        "length_max_m": class.loa_max_m(),
        "beam_max_m": class.beam_max_m(),
        "draught_m": pair(class.draught_m),
        "air_draft_m": pair(class.air_draft_m),
        "reference_dwt_t": pair(class.dwt_t),
        "confidence": class.confidence,
        "source": class.source,
        "notes": class.notes,
        "beyond_largest": beyond,
    }))
}
