//! Label: property
//!
//! The differential test against the Lean model (`proofs/lean`, ADR 0014). It generates random
//! groups of constant-limit Rules and random Measure values, writes them in the oracle's JSON
//! form, runs the oracle, and compares the verdict, each Rule's outcome and each governing Rule
//! with this crate's evaluator.
//!
//! It needs the built oracle, so it runs only when `ARC_RULES_LEAN_ORACLE` names it (the weekly
//! Lean job sets it); otherwise it passes with a note. `ARC_RULES_LEAN_CASES` sets the number of
//! cases (default 400), `ARC_RULES_LEAN_SEED` the seed.
//!
//! The model leaves out what the oracle cannot say: tolerances, tables, formulas, Applicability.
//! A key with no value is "not evaluated" in the model; here its Rules are written with
//! `evaluated: false`, and the model's governing Rule of such a key is not compared (the
//! evaluator marks governing rows only among worked-out limits).

use std::collections::BTreeMap;
use std::process::Command;

use arc_rules::domain::{MeasureInfo, MethodStatus, ValueKind};
use arc_rules::eval::{MeasureSource, MeasureValue, Outcome, evaluate};
use arc_rules::model::{
    Applicability, Citation, Codified, Compare, Consequence, Kind, Limit, MeasureRef, Origin, Rule,
    RuleSet, SetStatus,
};
use serde_json::{Value, json};

/// xorshift64*: a fixed, dependency-free generator, so a failing seed can be replayed.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }
}

/// A number on a coarse decimal grid, so that ties are common. Written and read as the same
/// decimal text on both sides.
fn grid(rng: &mut Rng) -> (String, f64) {
    let text = format!("{:.1}", rng.below(11) as f64 / 10.0);
    let v: f64 = text.parse().unwrap();
    (text, v)
}

struct OneObject {
    values: BTreeMap<String, f64>,
}

impl MeasureSource for OneObject {
    fn def(&self, _id: &str) -> Option<MeasureInfo> {
        Some(MeasureInfo {
            unit: "-".into(),
            object: "thing".into(),
            value: ValueKind::Number,
            args: Vec::new(),
            method_status: MethodStatus::Verified,
        })
    }
    fn objects(&self, object: &str) -> Vec<String> {
        if object == "thing" {
            vec!["x".into()]
        } else {
            Vec::new()
        }
    }
    fn value(&self, m: &MeasureRef, _: usize) -> Result<MeasureValue, String> {
        self.values
            .get(&m.id)
            .map(|v| MeasureValue::Number(*v))
            .ok_or_else(|| "no value".into())
    }
    fn input(&self, _: &str) -> Option<String> {
        None
    }
}

fn rule(
    id: String,
    measure: &str,
    compare: Compare,
    limit: f64,
    block: bool,
    evaluated: bool,
) -> Rule {
    Rule {
        id,
        title: "random".into(),
        origin: Origin::Physical,
        source: Citation::default(),
        codified: Codified {
            by: "agent".into(),
            date: "2026-10-21".into(),
            review: "lean".into(),
        },
        verified: None,
        applies_when: Applicability::default(),
        kind: Kind::Limit,
        measure: MeasureRef {
            id: measure.into(),
            args: Vec::new(),
        },
        compare: Some(compare),
        limit: Some(Limit::Constant {
            value: limit,
            unit: "-".into(),
        }),
        tolerance: 0.0,
        consequence: if block {
            Consequence::Block
        } else {
            Consequence::Warn
        },
        sizes: false,
        evaluated,
        gap: (!evaluated).then(|| "no value in this case".into()),
        finding: None,
        since: "1.0.0".into(),
    }
}

/// One random case: the oracle's JSON, and the evaluator's answer in the oracle's form.
fn case(rng: &mut Rng, n: usize) -> (Value, Value, Vec<String>) {
    let n_keys = 1 + rng.below(3) as usize;
    let mut values = BTreeMap::new();
    let mut value_json = serde_json::Map::new();
    let mut missing = Vec::new();
    for j in 0..n_keys {
        let id = format!("m{j}");
        if rng.chance(15) {
            missing.push(format!("{id}@x"));
            continue;
        }
        let (text, v) = grid(rng);
        values.insert(id.clone(), v);
        value_json.insert(format!("{id}@x"), serde_json::from_str(&text).unwrap());
    }
    let mut rules = Vec::new();
    let mut rules_json = Vec::new();
    for i in 0..1 + rng.below(6) as usize {
        let key = format!("m{}", rng.below(n_keys as u64));
        let lower = rng.chance(50);
        let strict = rng.chance(30);
        let block = rng.chance(70);
        let (text, limit) = grid(rng);
        let compare = match (lower, strict) {
            (true, false) => Compare::Ge,
            (true, true) => Compare::Gt,
            (false, false) => Compare::Le,
            (false, true) => Compare::Lt,
        };
        // Zero-padded ids sort in list order: the evaluator breaks a tie by rule id, the model by
        // list order.
        let id = format!("t/1/r{i:03}");
        let evaluated = values.contains_key(&key);
        rules.push(rule(id.clone(), &key, compare, limit, block, evaluated));
        rules_json.push(json!({
            "id": id,
            "key": format!("{key}@x"),
            "dir": if lower { "lower" } else { "upper" },
            "limit": serde_json::from_str::<Value>(&text).unwrap(),
            "strict": strict,
            "blocking": block,
        }));
    }
    let set = RuleSet {
        id: "t".into(),
        family: "t".into(),
        title: "random".into(),
        version: "1.0.0".into(),
        edition: Vec::new(),
        sources: Vec::new(),
        status: SetStatus::Draft,
        applies_when: Applicability::default(),
        includes: Vec::new(),
        known_conflicts: Vec::new(),
        tables: Vec::new(),
        rules,
        hash: String::new(),
    };
    let trace = evaluate(&[&set], &[], &OneObject { values });
    let mut outcomes = serde_json::Map::new();
    let mut governing = serde_json::Map::new();
    for r in &trace.rows {
        let id = r.rule.clone().unwrap();
        let outcome = match r.outcome {
            Outcome::Pass => "pass",
            Outcome::Fail => "fail",
            Outcome::Warn => "warn",
            Outcome::NotEvaluated => "not evaluated",
            other => panic!("case {n}: unexpected outcome {other:?}"),
        };
        outcomes.insert(id.clone(), json!(outcome));
        if r.governing {
            let dir = if r.compare.unwrap().is_lower_bound() {
                "lower"
            } else {
                "upper"
            };
            let key = format!("{}@x|{dir}", r.measure.as_ref().unwrap().id);
            governing.insert(key, json!(id));
        }
    }
    let name = format!("random case {n}");
    let input = json!({"name": name, "rules": rules_json, "values": value_json});
    let rust = json!({
        "name": name,
        "ok": trace.passes(),
        "outcomes": outcomes,
        "governing": governing,
    });
    (input, rust, missing)
}

fn env_number(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[test]
fn the_evaluator_agrees_with_the_lean_model() {
    let Ok(oracle) = std::env::var("ARC_RULES_LEAN_ORACLE") else {
        eprintln!("ARC_RULES_LEAN_ORACLE is not set: the Lean differential test is skipped");
        return;
    };
    let cases = env_number("ARC_RULES_LEAN_CASES", 400) as usize;
    let mut rng = Rng(env_number("ARC_RULES_LEAN_SEED", 0x0dd5_eed5) | 1);
    let mut inputs = Vec::new();
    let mut expected = Vec::new();
    for n in 0..cases {
        let (input, rust, missing) = case(&mut rng, n);
        inputs.push(input);
        expected.push((rust, missing));
    }
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("lean-oracle-cases.json");
    std::fs::write(&path, json!({"cases": inputs}).to_string()).unwrap();
    let out = Command::new(&oracle)
        .arg(&path)
        .output()
        .expect("run the oracle");
    assert!(
        out.status.success(),
        "the oracle failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let got: Value = serde_json::from_slice(&out.stdout).expect("the oracle prints JSON");
    let got = got["cases"].as_array().expect("a list of cases");
    assert_eq!(got.len(), expected.len());
    for ((rust, missing), lean) in expected.iter().zip(got) {
        let name = &rust["name"];
        assert_eq!(&lean["name"], name);
        assert_eq!(lean["ok"], rust["ok"], "{name}: the verdict differs");
        assert_eq!(
            lean["outcomes"], rust["outcomes"],
            "{name}: an outcome differs"
        );
        // The model names a governing Rule for a key with no value too; the evaluator does not.
        let mut lean_gov = lean["governing"].as_object().unwrap().clone();
        lean_gov.retain(|k, _| !missing.iter().any(|m| k.starts_with(&format!("{m}|"))));
        assert_eq!(
            Value::Object(lean_gov),
            rust["governing"],
            "{name}: a governing Rule differs"
        );
    }
    eprintln!("{cases} random cases agree with the Lean model");
}
