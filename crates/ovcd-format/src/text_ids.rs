//! The `text_ids` key of a result (ADR 0014, spec 0005).
//!
//! A result holds the engine's English. `text_ids` maps each English text of the result to its
//! Message Id and arguments, so the app can show the text in another language. It is a reading aid,
//! not a result: it is left out of the results hash (`UNHASHED_KEYS` in `run.rs`), and no other key of
//! the result changes when it is added.

use crate::eval::num;
use ovcd_core::checks::Check;
use ovcd_core::text::{Arg, Msg, Text};
use serde_json::{Map, Value, json};

/// The English texts of one result with their Message Ids, as they are found.
#[derive(Debug, Default)]
pub(crate) struct TextIds(Map<String, Value>);

impl TextIds {
    /// One text, if the engine gave it an id. The first id met for an English text is kept.
    pub(crate) fn text(&mut self, t: &Text) {
        if let Some(m) = &t.msg {
            self.add(&t.en, m);
        }
    }

    pub(crate) fn texts(&mut self, ts: &[Text]) {
        ts.iter().for_each(|t| self.text(t));
    }

    /// The messages of checks.
    pub(crate) fn checks(&mut self, cs: &[Check]) {
        for c in cs {
            if let Some(m) = &c.msg {
                self.add(&c.message, m);
            }
        }
    }

    fn add(&mut self, en: &str, m: &Msg) {
        if !self.0.contains_key(en) {
            self.0.insert(en.to_string(), msg_json(m, false));
        }
    }

    /// The finished key for `result`: what was found, and each fixed name the result holds.
    pub(crate) fn finish(mut self, result: &Value) -> Value {
        let names = fixed_names();
        let mut seen = Vec::new();
        strings(result, &mut seen);
        for s in seen {
            if let Some(t) = names.iter().find(|t| t.en == s) {
                self.text(t);
            }
        }
        Value::Object(self.0)
    }
}

/// A Message Id and its arguments as JSON. A message that is an argument of another also carries
/// its English, the text to fall back to.
pub fn msg_json(m: &Msg, nested: bool) -> Value {
    let args: Map<String, Value> = m
        .args
        .iter()
        .map(|(k, a)| ((*k).to_string(), arg_json(a)))
        .collect();
    let mut out = json!({ "id": m.id, "args": args });
    if nested {
        out["en"] = json!(m.english);
    }
    out
}

fn arg_json(a: &Arg) -> Value {
    match a {
        Arg::Num { value, text } => json!({ "num": text, "value": num(*value) }),
        Arg::Count(n) => json!(n),
        Arg::Text(t) => json!(t),
        Arg::Cite(t) => json!({ "cite": t }),
        Arg::Input(k) => json!({ "input": k }),
        Arg::Msg(m) => msg_json(m, true),
    }
}

/// Every string of a JSON value, depth first.
fn strings<'a>(v: &'a Value, out: &mut Vec<&'a str>) {
    match v {
        Value::String(s) => out.push(s),
        Value::Array(items) => items.iter().for_each(|i| strings(i, out)),
        Value::Object(map) => map.values().for_each(|i| strings(i, out)),
        _ => {}
    }
}

/// Names the engine writes from fixed English and also uses as keys (a stability condition, a
/// criterion, a check, a channel environment, a vessel), so their type stays text. Each is mapped
/// here to its Message Id when a result holds it.
fn fixed_names() -> Vec<Text> {
    vec![
        Text::fixed("name-condition-laden", "Laden"),
        Text::fixed("name-condition-half-laden", "Half laden"),
        Text::fixed("name-condition-light", "Light"),
        Text::fixed("name-criterion-area-0-30", "Area 0\u{2013}30\u{b0}"),
        Text::fixed("name-criterion-area-0-40", "Area 0\u{2013}40\u{b0}"),
        Text::fixed("name-criterion-area-30-40", "Area 30\u{2013}40\u{b0}"),
        Text::fixed("name-criterion-gz-at-30", "GZ at \u{2265} 30\u{b0}"),
        Text::fixed("name-criterion-angle-of-max-gz", "Angle of max GZ"),
        Text::fixed("name-criterion-initial-gm", "Initial GM (fluid)"),
        Text::fixed("name-criterion-floats", "Floats with freeboard"),
        Text::fixed("name-check-stability-laden", "Stability \u{2014} laden"),
        Text::fixed(
            "name-check-stability-half-laden",
            "Stability \u{2014} half laden",
        ),
        Text::fixed("name-check-stability-light", "Stability \u{2014} light"),
        Text::fixed(
            "name-check-stability-not-assessed",
            "Stability \u{2014} not assessed",
        ),
        Text::fixed("name-check-cargo", "Cargo > 0"),
        Text::fixed("name-check-feasible-arrangement", "Feasible arrangement"),
        Text::fixed("name-check-air-draft", "Air draft (empty, governing)"),
        Text::fixed("name-check-tank-length", "tank length over diameter"),
        Text::fixed("name-check-ok", "OK"),
        Text::fixed("name-curve-deep-water", "Deep water"),
        Text::fixed("name-governs-hold-volume", "hold volume"),
        Text::fixed("name-governs-tank-volume", "tank volume"),
        Text::fixed(
            "name-governs-no-cargo-area",
            "no cargo area (hull shorter than its fixed end spaces)",
        ),
        Text::fixed("name-vessel-box", "Box-hold dry bulk barge"),
        Text::fixed("name-vessel-hopper", "Sloped-hopper dry bulk barge"),
        Text::fixed("name-vessel-gas", "Gas barge, IMO Type C tanks"),
        Text::fixed(
            "name-cargo-lightship-alone",
            "Lightship alone exceeds the allowed draught.",
        ),
        Text::fixed(
            "name-cargo-nothing-fits",
            "Nothing fits: lengthen or widen the hull.",
        ),
    ]
}
