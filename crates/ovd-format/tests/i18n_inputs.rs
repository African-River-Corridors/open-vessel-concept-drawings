//! The English of the input schema in the Text Library (spec 0005).
//!
//! The app shows the schema's labels under derived Message Ids: `group-<group>`, `group-note-<group>`,
//! `input-<name>`, `input-<name>-default-by` and `choice-<input>-<value>`. This test writes the English
//! of each from the schema and compares it with `locales/en/inputs.ftl`, so the file is never stale.
//! Vessel Scheme and Vessel Class choices are standard designations and are not translated; `basis`
//! texts are citations and are not translated either.
//!
//! To write the file again after a label changes: `OVD_WRITE_TEXT=1 cargo test -p ovd-format --test i18n_inputs`.
//! Label: property

use ovd_format::Profile;
use ovd_format::session::input_schema;
use serde_json::Value;
use std::path::Path;

/// A Message Id piece from an English name or a key: lower case, words joined by `-`.
fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

/// A Fluent value that reads as `text`: braces are literal, and space at either end is kept.
fn fluent(text: &str) -> String {
    let body = text
        .replace('{', "\u{0}")
        .replace('}', "{ \"}\" }")
        .replace('\u{0}', "{ \"{\" }");
    assert!(
        !body.contains('\n'),
        "a schema label has a line break: {text:?}"
    );
    assert_eq!(
        body.trim(),
        body,
        "a schema label starts or ends with a space: {text:?}"
    );
    body
}

fn render() -> String {
    let schema = input_schema(&Profile::built_in());
    let mut out = String::from(
        "# The input schema's labels, choices and group names (spec 0005).\n\
         # Written by crates/ovd-format/tests/i18n_inputs.rs from the schema. Do not edit: change the\n\
         # label in the Rust source and run `OVD_WRITE_TEXT=1 cargo test -p ovd-format --test i18n_inputs`.\n",
    );
    out.push_str("\n## Groups\n\n");
    for g in schema["groups"].as_array().unwrap() {
        let g = g.as_str().unwrap();
        out.push_str(&format!("group-{} = {}\n", slug(g), fluent(g)));
        if let Some(note) = schema["group_notes"][g].as_str() {
            out.push_str(&format!("group-note-{} = {}\n", slug(g), fluent(note)));
        }
    }
    out.push_str("\n## Inputs\n");
    for i in schema["inputs"].as_array().unwrap() {
        let name = i["name"].as_str().unwrap();
        out.push_str(&format!(
            "\ninput-{} = {}\n",
            slug(name),
            fluent(i["label"].as_str().unwrap())
        ));
        if let Some(by) = i["default_by"]["by"].as_str() {
            out.push_str(&format!(
                "input-{}-default-by = {}\n",
                slug(name),
                fluent(by)
            ));
        }
        let kind = i["kind"].as_str().unwrap();
        if kind == "vessel_scheme" || kind == "vessel_class" {
            continue;
        }
        for c in i["choices"].as_array().map(Vec::as_slice).unwrap_or(&[]) {
            let value = match &c["value"] {
                Value::String(s) => s.clone(),
                v => v.to_string(),
            };
            out.push_str(&format!(
                "choice-{}-{} = {}\n",
                slug(name),
                slug(&value),
                fluent(c["label"].as_str().unwrap())
            ));
        }
    }
    out
}

#[test]
fn the_text_library_has_the_english_of_the_input_schema() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../locales/en/inputs.ftl");
    let want = render();
    if std::env::var_os("OVD_WRITE_TEXT").is_some() {
        std::fs::write(&path, &want).unwrap();
    }
    let have = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        have == want,
        "locales/en/inputs.ftl is not the schema's English: run OVD_WRITE_TEXT=1 cargo test -p ovd-format --test i18n_inputs"
    );
}

#[test]
fn every_derived_id_is_unique() {
    let text = render();
    let mut ids: Vec<&str> = text
        .lines()
        .filter_map(|l| l.split_once(" = ").map(|(k, _)| k))
        .collect();
    let n = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), n, "two schema entries give one Message Id");
}
