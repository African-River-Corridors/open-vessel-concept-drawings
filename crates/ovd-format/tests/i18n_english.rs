//! The engine's Message Ids agree with its English, and change no result (ADR 0015, spec 0005).
//!
//! Every fixture is opened and evaluated as the app does. Each `text_ids` entry of each result, and
//! each message `msg`, is formatted with the English Text Library (`locales/en/*.ftl`, by the Fluent
//! reference implementation) and must give the engine's own English exactly. And the results hash
//! is the same with `text_ids` removed or changed, so the language can never move a result.
//! Label: property

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use ovd_core::text::{Arg, Msg};
use ovd_format::{DesignSession, Profile, check, fill, input_schema, results_hash};
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
            b.add_resource(
                FluentResource::try_new(text).unwrap_or_else(|(_, e)| panic!("{path:?}: {e:?}")),
            )
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
            Value::Object(o) if o.contains_key("num") => {
                o["num"].as_str().unwrap().to_string().into()
            }
            Value::Object(o) if o.contains_key("cite") => {
                o["cite"].as_str().unwrap().to_string().into()
            }
            Value::Object(o) if o.contains_key("input") => {
                label(o["input"].as_str().unwrap()).into()
            }
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
    let out = b
        .format_pattern(pattern, Some(&args), &mut errors)
        .to_string();
    assert!(errors.is_empty(), "{id}: {errors:?}");
    out
}

fn evaluations() -> Vec<(String, Value)> {
    let dir = root().join("crates/ovd-format/tests/fixtures");
    let mut out = vec![];
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let text = std::fs::read_to_string(&path).unwrap();
        let (session, _) = DesignSession::open(&text, Profile::built_in()).unwrap();
        out.push((
            path.file_name().unwrap().to_string_lossy().into_owned(),
            session.evaluate().to_json(),
        ));
    }
    out.push((
        "new design".into(),
        DesignSession::new_design(Profile::built_in())
            .unwrap()
            .evaluate()
            .to_json(),
    ));
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
                    assert_eq!(
                        say(&b, &m["msg"]),
                        m["text"].as_str().unwrap(),
                        "{file}: {}",
                        m["msg"]["id"]
                    );
                    seen += 1;
                }
            }
            if !o["outcome"]["msg"].is_null() {
                assert_eq!(
                    say(&b, &o["outcome"]["msg"]),
                    o["outcome"]["message"].as_str().unwrap(),
                    "{file}"
                );
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
            assert!(
                r["text_ids"].as_object().is_some_and(|m| !m.is_empty()),
                "{file}: no text_ids"
            );
            let hash = outcome["results_hash"].as_str().unwrap();
            let mut without = r.clone();
            without.as_object_mut().unwrap().remove("text_ids");
            let mut changed = r.clone();
            changed["text_ids"] = json!({ "anything": { "id": "x", "args": Map::new() } });
            assert_eq!(results_hash(r), hash, "{file}");
            assert_eq!(
                results_hash(&without),
                hash,
                "{file}: text_ids moved the hash"
            );
            assert_eq!(
                results_hash(&changed),
                hash,
                "{file}: text_ids moved the hash"
            );
            results += 1;
        }
    }
    assert!(results >= 6, "only {results} results were checked");
}

/// A report message's Message Id as English: an input argument is the key, as the report writes it.
fn say_msg(b: &FluentBundle<FluentResource>, msg: &Msg) -> String {
    let pattern = b
        .get_message(msg.id)
        .and_then(|m| m.value())
        .unwrap_or_else(|| panic!("{} is not in locales/en", msg.id));
    let mut args = FluentArgs::new();
    for (name, a) in &msg.args {
        let v: FluentValue = match a {
            Arg::Num { text, .. } => text.clone().into(),
            Arg::Count(n) => (*n).into(),
            Arg::Text(t) | Arg::Cite(t) | Arg::Input(t) => t.clone().into(),
            Arg::Msg(inner) => {
                let text = say_msg(b, inner);
                assert_eq!(text, inner.english, "nested {}.{name}", msg.id);
                text.into()
            }
        };
        args.set(*name, v);
    }
    let mut errors = vec![];
    let out = b
        .format_pattern(pattern, Some(&args), &mut errors)
        .to_string();
    assert!(errors.is_empty(), "{}: {errors:?}", msg.id);
    out
}

/// Files that raise most validation messages: the fixtures, the old-format files, and a valid
/// Design with one fault each.
fn faulty_files() -> Vec<(String, String)> {
    let mut out = vec![];
    for dir in [
        "crates/ovd-format/tests/fixtures",
        "crates/ovd-format/tests/legacy",
    ] {
        for entry in std::fs::read_dir(root().join(dir)).unwrap() {
            let path = entry.unwrap().path();
            out.push((
                path.display().to_string(),
                std::fs::read_to_string(&path).unwrap(),
            ));
        }
    }
    let base: Value = serde_json::from_str(
        &std::fs::read_to_string(
            root().join("crates/ovd-format/tests/fixtures/dry_bulk_box_hold.vdesign"),
        )
        .unwrap(),
    )
    .unwrap();
    let gas: Value = serde_json::from_str(
        &std::fs::read_to_string(
            root().join("crates/ovd-format/tests/fixtures/gas_igc_2g_exact.vdesign"),
        )
        .unwrap(),
    )
    .unwrap();
    type Fault = (&'static str, fn(&mut Value));
    let faults: &[Fault] = &[
        ("not an object", |d| *d = json!([1])),
        ("top level", |d| {
            d["format"] = json!("x");
            d["format_version"] = json!(9);
            d["engine_version"] = json!("");
            d["units"] = json!("imperial");
            d["extra"] = json!(1);
        }),
        ("no version", |d| {
            d.as_object_mut().unwrap().remove("format_version");
        }),
        ("no options", |d| {
            d.as_object_mut().unwrap().remove("options");
        }),
        ("empty options", |d| d["options"] = json!([])),
        ("option shape", |d| {
            let o = d["options"][0].clone();
            d["options"] = json!([o.clone(), o, { "inputs": {} }, { "name": "B" }]);
            d["options"][0]["extra"] = json!(1);
            d["options"][0]["results_hash"] = json!("abc");
            d["options"][1]["results_hash"] = json!(5);
        }),
        ("input values", |d| {
            let i = &mut d["options"][0]["inputs"];
            i["block_coefficient"] = json!({ "value": "x", "source": "typed" });
            i["max_speed_kn"] = json!({ "value": 40.0, "source": "typed" });
            i["propulsive_efficiency"] = json!({ "value": 0.95, "source": "typed" });
            i["hull_form"] = json!("round");
            i["cargo_containment"] = json!("barrel");
            i["air_draft_limit_m"] = json!("high");
            i["optimise_depth"] = json!(3);
            i["unknown_thing"] = json!(1);
            i["min_freeboard_m"] = json!({ "value": 1, "source": "nobody" });
            i["pushed_stern_void_m"] = json!({ "source": "typed" });
            i["hold_fill_ratio"] = json!({ "value": 1, "source": "typed", "x": 1 });
            i.as_object_mut().unwrap().remove("cargo");
        }),
        ("cross checks", |d| {
            let i = &mut d["options"][0]["inputs"];
            i["length_overall_m"] = json!(140.0);
            i["moulded_depth_m"] = json!(2.0);
            i["min_freeboard_m"] = json!(2.5);
            i["beam_m"] = json!(1.5);
            i["channel_depth_m"] = json!(0.2);
            i["channel_under_keel_clearance_m"] = json!(0.5);
            i["n_holds"] = json!(1.5);
        }),
        ("missing inputs", |d| {
            let i = d["options"][0]["inputs"].as_object_mut().unwrap();
            i.remove("block_coefficient");
            i.remove("max_speed_kn");
        }),
        ("formation", |d| {
            d["options"][0]["inputs"]["formation"] = json!({ "kind": "pushed_barge", "abreast": 2 })
        }),
        ("formation kind", |d| {
            d["options"][0]["inputs"]["formation"] = json!({ "kind": "raft" })
        }),
        ("formation object", |d| {
            d["options"][0]["inputs"]["formation"] = json!(3)
        }),
        (
            "convoy",
            |d| {
                d["options"][0]["inputs"]["formation"] =
                    json!({ "kind": "pushed_convoy", "abreast": 1.5, "in_line": 2 })
            },
        ),
        (
            "convoy keys",
            |d| {
                d["options"][0]["inputs"]["formation"] =
                    json!({ "kind": "pushed_convoy", "abreast": 9, "in_line": 2, "x": 1 })
            },
        ),
        ("class", |d| {
            d["options"][0]["inputs"]["vessel_scheme"] = json!("nowhere");
            d["options"][0]["inputs"]["vessel_class"] = json!(7);
        }),
        ("class name", |d| {
            d["options"][0]["inputs"]["vessel_class"] = json!("ZZ")
        }),
    ];
    for (name, fault) in faults {
        let mut d = base.clone();
        fault(&mut d);
        out.push((format!("dry bulk, {name}"), d.to_string()));
    }
    let gas_faults: &[Fault] = &[
        ("cargo", |d| d["options"][0]["inputs"]["cargo"] = json!(5)),
        ("cargo name", |d| {
            d["options"][0]["inputs"]["cargo"] = json!("cheese")
        }),
        ("containment", |d| {
            d["options"][0]["inputs"]["cargo_containment"] = json!("box_hold")
        }),
        ("insulation", |d| {
            d["options"][0]["inputs"]["tank_insulation_type"] = json!("none");
            d["options"][0]["inputs"]["tank_insulation_thickness_mm"] = json!(100.0);
        }),
        ("hold input", |d| {
            d["options"][0]["inputs"]["hold_fill_ratio"] = json!(0.9)
        }),
        ("stress", |d| {
            d["options"][0]["inputs"]["tank_allowable_stress_mpa"] = json!(300.0)
        }),
    ];
    for (name, fault) in gas_faults {
        let mut d = gas.clone();
        fault(&mut d);
        out.push((format!("gas, {name}"), d.to_string()));
    }
    out
}

/// The id of a message and of every message inside it.
fn walk(msg: &Msg, ids: &mut std::collections::BTreeSet<&'static str>) {
    ids.insert(msg.id);
    for (_, a) in &msg.args {
        if let Arg::Msg(inner) = a {
            walk(inner, ids);
        }
    }
}

#[test]
fn every_validation_message_id_gives_the_engine_english() {
    let b = english();
    let profile = Profile::built_in();
    let mut ids = std::collections::BTreeSet::new();
    for (file, text) in faulty_files() {
        for checked in [check(&text, &profile), fill(&text, &profile)] {
            let r = &checked.report;
            assert_eq!(r.errors.len(), r.error_msgs.len(), "{file}");
            assert_eq!(r.warnings.len(), r.warning_msgs.len(), "{file}");
            assert_eq!(
                r.migration_notes.len(),
                r.migration_note_msgs.len(),
                "{file}"
            );
            let all = r
                .errors
                .iter()
                .zip(&r.error_msgs)
                .chain(r.warnings.iter().zip(&r.warning_msgs));
            for (en, msg) in all.chain(r.migration_notes.iter().zip(&r.migration_note_msgs)) {
                let msg = msg
                    .as_ref()
                    .unwrap_or_else(|| panic!("{file}: no Message Id for {en:?}"));
                assert_eq!(&say_msg(&b, msg), en, "{file}: {}", msg.id);
                walk(msg, &mut ids);
            }
        }
    }
    assert!(
        ids.len() >= 40,
        "only {} validation messages were reached: {ids:?}",
        ids.len()
    );
}
