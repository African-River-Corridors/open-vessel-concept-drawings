//! The engine's Message Ids agree with its English, and change no result (ADR 0014, spec 0005).
//!
//! Every fixture is opened and evaluated as the app does. Each `text_ids` entry of each result, and
//! each message `msg`, is formatted with the English Text Library (`locales/en/*.ftl`, by the Fluent
//! reference implementation) and must give the engine's own English exactly. And the results hash
//! is the same with `text_ids` removed or changed, so the language can never move a result.
//! Label: property

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use ovcd_format::{DesignSession, Profile, input_schema, results_hash};
use serde_json::{Map, Value, json};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn english() -> FluentBundle<FluentResource> {
    let mut b = FluentBundle::new(vec!["en".parse().unwrap()]);
    b.set_use_isolating(false);
    for entry in std::fs::read_dir(root().join("locales/en")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "ftl") {
            let text = std::fs::read_to_string(&path).unwrap();
            b.add_resource(FluentResource::try_new(text).unwrap_or_else(|(_, e)| panic!("{path:?}: {e:?}")))
                .unwrap_or_else(|e| panic!("{path:?}: {e:?}"));
        }
    }
    b
}

/// An input's label as the app shows it, from the schema (the session's own rewording).
fn label(key: &str) -> String {
    match key {
        "abreast" | "formation.abreast" => return "Abreast".into(),
        "in_line" | "formation.in_line" => return "In line".into(),
        "kind" => return "Kind".into(),
        _ => {}
    }
    let schema = input_schema(&Profile::built_in());
    schema["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["name"] == key)
        .and_then(|i| i["label"].as_str())
        .unwrap_or(key)
        .to_string()
}

fn say(b: &FluentBundle<FluentResource>, msg: &Value) -> String {
    let id = msg["id"].as_str().unwrap();
    let pattern = b
        .get_message(id)
        .and_then(|m| m.value())
        .unwrap_or_else(|| panic!("{id} is not in locales/en"));
    let mut args = FluentArgs::new();
    for (name, a) in msg["args"].as_object().unwrap() {
        let v: FluentValue = match a {
            Value::String(s) => s.clone().into(),
            Value::Number(n) => n.as_i64().unwrap().into(),
            Value::Object(o) if o.contains_key("num") => o["num"].as_str().unwrap().to_string().into(),
            Value::Object(o) if o.contains_key("cite") => o["cite"].as_str().unwrap().to_string().into(),
            Value::Object(o) if o.contains_key("input") => label(o["input"].as_str().unwrap()).into(),
            Value::Object(o) if o.contains_key("id") => {
                let inner = say(b, a);
                assert_eq!(Some(inner.as_str()), o["en"].as_str(), "nested {id}.{name}");
                inner.into()
            }
            other => panic!("{id}.{name}: unknown argument {other}"),
        };
        args.set(name.clone(), v);
    }
    let mut errors = vec![];
    let out = b.format_pattern(pattern, Some(&args), &mut errors).to_string();
    assert!(errors.is_empty(), "{id}: {errors:?}");
    out
}

fn evaluations() -> Vec<(String, Value)> {
    let dir = root().join("crates/ovcd-format/tests/fixtures");
    let mut out = vec![];
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let text = std::fs::read_to_string(&path).unwrap();
        let (session, _) = DesignSession::open(&text, Profile::built_in()).unwrap();
        out.push((path.file_name().unwrap().to_string_lossy().into_owned(), session.evaluate().to_json()));
    }
    out.push(("new design".into(), DesignSession::new_design(Profile::built_in()).unwrap().evaluate().to_json()));
    out
}

#[test]
fn every_message_id_gives_the_engine_english() {
    let b = english();
    let mut seen = 0;
    for (file, eval) in evaluations() {
        for o in eval["options"].as_array().unwrap() {
            if let Some(ids) = o["outcome"]["results"]["text_ids"].as_object() {
                for (en, msg) in ids {
                    assert_eq!(&say(&b, msg), en, "{file}: {}", msg["id"]);
                    seen += 1;
                }
            }
            for m in o["messages"].as_array().unwrap() {
                if !m["msg"].is_null() {
                    assert_eq!(say(&b, &m["msg"]), m["text"].as_str().unwrap(), "{file}: {}", m["msg"]["id"]);
                    seen += 1;
                }
            }
            if !o["outcome"]["msg"].is_null() {
                assert_eq!(say(&b, &o["outcome"]["msg"]), o["outcome"]["message"].as_str().unwrap(), "{file}");
                seen += 1;
            }
        }
    }
    assert!(seen > 20, "only {seen} messages were checked");
}

#[test]
fn every_result_has_text_ids_and_they_are_outside_the_hash() {
    let mut results = 0;
    for (file, eval) in evaluations() {
        for o in eval["options"].as_array().unwrap() {
            let outcome = &o["outcome"];
            if outcome["status"] != "results" {
                continue;
            }
            let r = &outcome["results"];
            assert!(r["text_ids"].as_object().is_some_and(|m| !m.is_empty()), "{file}: no text_ids");
            let hash = outcome["results_hash"].as_str().unwrap();
            let mut without = r.clone();
            without.as_object_mut().unwrap().remove("text_ids");
            let mut changed = r.clone();
            changed["text_ids"] = json!({ "anything": { "id": "x", "args": Map::new() } });
            assert_eq!(results_hash(r), hash, "{file}");
            assert_eq!(results_hash(&without), hash, "{file}: text_ids moved the hash");
            assert_eq!(results_hash(&changed), hash, "{file}: text_ids moved the hash");
            results += 1;
        }
    }
    assert!(results >= 6, "only {results} results were checked");
}
