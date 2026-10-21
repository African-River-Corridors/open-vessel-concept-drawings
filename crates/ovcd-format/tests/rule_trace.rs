//! Seam: the rule evaluator and the Rule Trace (spec 0004, issue #269), through Rule Sets loaded at
//! run time and a stand-in Measure source. The property tests draw cases from a fixed-seed
//! generator, so every run checks the same cases.
//!
//! Label: property

use ovcd_core::rule_trace::{
    Failure, MeasureInfo, MeasureSource, MeasureValue, ObjectType, Outcome, Trace, evaluate,
    failures, holds,
};
use ovcd_core::rules::{MeasureRef, RuleSet};
use ovcd_format::rule_set::{Library, load};
use ovcd_format::trace::measure_info;
use serde_json::{Value, json};

/// A stand-in Option: loading conditions with a value per Measure, and its inputs.
struct Fake {
    conditions: Vec<String>,
    /// Measure id -> one value (or failure) per loading condition.
    values: Vec<(String, Vec<Result<f64, String>>)>,
    inputs: Vec<(String, String)>,
}

impl MeasureSource for Fake {
    fn def(&self, id: &str) -> Option<MeasureInfo> {
        measure_info(id)
    }
    fn objects(&self, object: &str) -> Vec<String> {
        match ObjectType::from_key(object).expect("an OVCD object type") {
            ObjectType::LoadingCondition => self.conditions.clone(),
            ObjectType::Option => vec!["Option 1".into()],
            _ => Vec::new(),
        }
    }
    fn value(&self, m: &MeasureRef, index: usize) -> Result<MeasureValue, String> {
        let (_, v) = self
            .values
            .iter()
            .find(|(id, _)| *id == m.id)
            .ok_or_else(|| format!("no value for {}", m.id))?;
        v[index].clone().map(MeasureValue::Number)
    }
    fn input(&self, name: &str) -> Option<String> {
        self.inputs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
    }
}

fn gm_rule(id: &str, compare: &str, limit: f64, consequence: &str) -> Value {
    json!({
        "id": id, "title": id, "origin": "physical", "source": {},
        "codified": {"by": "agent", "date": "2026-10-21", "review": "#269"},
        "verified": false, "kind": "limit", "measure": {"id": "gm0"},
        "compare": compare, "limit": {"value": limit, "unit": "m"},
        "consequence": consequence, "evaluated": true, "since": "1.0.0",
    })
}

fn set_text(id: &str, rules: Vec<Value>) -> String {
    serde_json::to_string(&json!({
        "id": id, "family": id, "title": id, "version": "1.0.0",
        "edition": {"document": "test"}, "sources": [], "status": "draft",
        "known_conflicts": [], "rules": rules,
    }))
    .unwrap()
}

fn fake(gm: &[f64]) -> Fake {
    Fake {
        conditions: (0..gm.len()).map(|i| format!("C{i}")).collect(),
        values: vec![("gm0".into(), gm.iter().map(|x| Ok(*x)).collect())],
        inputs: vec![("cargo".into(), "LNG".into())],
    }
}

fn run(lib: &Library, chosen: &[(&str, &str)], source: &Fake) -> Trace {
    let sel = lib.select(chosen).unwrap();
    evaluate(&sel.sets, source)
}

#[test]
fn two_sets_on_one_requirement_the_stricter_governs_and_either_can_fail() {
    let mut lib = Library::default();
    lib.add_text(&set_text("a", vec![gm_rule("a/x/gm", "ge", 0.15, "block")]))
        .unwrap();
    lib.add_text(&set_text("b", vec![gm_rule("b/x/gm", "ge", 0.30, "block")]))
        .unwrap();
    let chosen = [("a", "1.0.0"), ("b", "1.0.0")];
    // GM 0.20: meets a (0.15), fails b (0.30).
    let t = run(&lib, &chosen, &fake(&[0.20]));
    assert_eq!(t.rows.len(), 2);
    let (a, b) = (&t.rows[0], &t.rows[1]);
    assert_eq!(a.rule.as_deref(), Some("a/x/gm"));
    assert_eq!((a.outcome, a.governing), (Outcome::Pass, false));
    assert_eq!((b.outcome, b.governing), (Outcome::Fail, true));
    assert!(!t.passes());
    // GM 0.40 meets both: both rows still appear, and b still governs.
    let t = run(&lib, &chosen, &fake(&[0.40]));
    assert!(t.passes());
    assert!(t.rows[1].governing && !t.rows[0].governing);
    // A non-blocking stricter Rule never governs over a blocking one.
    let mut lib = Library::default();
    lib.add_text(&set_text("a", vec![gm_rule("a/x/gm", "ge", 0.15, "block")]))
        .unwrap();
    lib.add_text(&set_text("w", vec![gm_rule("w/x/gm", "ge", 0.30, "warn")]))
        .unwrap();
    let t = run(&lib, &[("a", "1.0.0"), ("w", "1.0.0")], &fake(&[0.20]));
    assert!(t.rows[0].governing);
    assert_eq!(t.rows[1].outcome, Outcome::Warn);
    assert!(t.passes(), "a warning never fails the Option");
}

#[test]
fn a_shared_rule_is_evaluated_once_and_names_both_sets() {
    let mut lib = Library::default();
    let mut text: Value =
        serde_json::from_str(&set_text("a", vec![gm_rule("x/p/gm", "ge", 0.15, "block")])).unwrap();
    text["family"] = json!("x");
    lib.add_text(&text.to_string()).unwrap();
    text["id"] = json!("b");
    lib.add_text(&text.to_string()).unwrap();
    let t = run(&lib, &[("a", "1.0.0"), ("b", "1.0.0")], &fake(&[0.2, 0.1]));
    assert_eq!(t.rows.len(), 2, "one row per object, not per set");
    assert_eq!(t.rows[0].rule_sets, ["a@1.0.0", "b@1.0.0"]);
}

#[test]
fn a_set_that_does_not_apply_gives_one_row() {
    let mut v: Value =
        serde_json::from_str(&set_text("a", vec![gm_rule("a/x/gm", "ge", 0.15, "block")])).unwrap();
    v["applies_when"] = json!({"cargo": ["bauxite"]});
    let mut lib = Library::default();
    lib.add_text(&v.to_string()).unwrap();
    let t = run(&lib, &[("a", "1.0.0")], &fake(&[0.0, 0.0, 0.0]));
    assert_eq!(t.rows.len(), 1);
    assert_eq!(
        (t.rows[0].rule.clone(), t.rows[0].outcome),
        (None, Outcome::NotApplicable)
    );
    assert!(t.passes());
}

#[test]
fn a_measure_that_fails_never_passes() {
    let mut lib = Library::default();
    lib.add_text(&set_text(
        "a",
        vec![
            gm_rule("a/x/gm", "ge", 0.15, "block"),
            gm_rule("a/x/w", "ge", 0.15, "warn"),
        ],
    ))
    .unwrap();
    let mut f = fake(&[0.2]);
    f.values[0].1[0] = Err("no curve".into());
    let t = run(&lib, &[("a", "1.0.0")], &f);
    assert_eq!(t.rows[0].outcome, Outcome::Fail);
    assert_eq!(t.rows[0].gap.as_deref(), Some("no curve"));
    assert_eq!(t.rows[1].outcome, Outcome::Warn);
    // A non-finite value is a failure too, never a pass.
    for bad in [f64::NAN, f64::INFINITY] {
        let t = run(&lib, &[("a", "1.0.0")], &fake(&[bad]));
        assert_eq!(t.rows[0].outcome, Outcome::Fail, "{bad}");
    }
}

#[test]
fn the_comparison_keeps_the_tolerance_form_exactly() {
    // Spec 0004: ge is value >= limit - tolerance, and the mirror forms.
    use ovcd_core::rules::Compare::*;
    assert!(holds(Ge, 0.6 - 1e-9, 0.6, 1e-9));
    assert!(!holds(Ge, 0.6 - 2e-9, 0.6, 1e-9));
    assert!(holds(Ge, 0.055, 0.055, 0.0) && !holds(Gt, 0.055, 0.055, 0.0));
    assert!(holds(Le, 0.95, 0.95, 0.0) && !holds(Lt, 0.95, 0.95, 0.0));
}

// --- properties, on a fixed-seed generator -------------------------------------------------------

/// xorshift64*: a small, fixed generator, so the cases are the same on every run.
struct Gen(u64);

impl Gen {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    /// A value on a coarse grid, so ties and exact boundaries happen often.
    fn value(&mut self) -> f64 {
        self.below(9) as f64 * 0.05
    }
}

/// A set of 1 to `max` random gm0 Rules.
fn random_set(g: &mut Gen, id: &str, max: u64) -> RuleSet {
    let n = 1 + g.below(max) as usize;
    let rules: Vec<Value> = (0..n)
        .map(|i| {
            let compare = ["ge", "gt", "le", "lt"][g.below(4) as usize];
            let consequence = ["block", "warn"][g.below(2) as usize];
            gm_rule(&format!("{id}/x/r{i}"), compare, g.value(), consequence)
        })
        .collect();
    let mut v: Value = serde_json::from_str(&set_text(id, rules)).unwrap();
    // Random bounds may conflict: declare every pair, as the spec allows with a reason.
    let ids: Vec<String> = (0..n).map(|i| format!("{id}/x/r{i}")).collect();
    let mut pairs = Vec::new();
    for a in &ids {
        for b in &ids {
            if a < b {
                pairs.push(json!({"rules": [a, b], "reason": "random test set"}));
            }
        }
    }
    v["known_conflicts"] = json!(pairs);
    load(&v.to_string()).unwrap().set
}

fn trace_of(sets: &[&RuleSet], gm: &[f64]) -> Trace {
    evaluate(sets, &fake(gm))
}

#[test]
fn property_margin_sign_matches_the_outcome() {
    let mut g = Gen(0x5eed_0001);
    for _ in 0..300 {
        let set = random_set(&mut g, "p", 5);
        let gm: Vec<f64> = (0..3).map(|_| g.value()).collect();
        for r in trace_of(&[&set], &gm).rows {
            let m = r.margin.unwrap();
            let strict = r.compare.unwrap().is_strict();
            let met = if strict { m > 0.0 } else { m >= 0.0 };
            assert_eq!(met, r.outcome == Outcome::Pass, "{r:?}");
        }
    }
}

#[test]
fn property_every_rule_appears_once_per_object() {
    let mut g = Gen(0x5eed_0002);
    for _ in 0..200 {
        let objects = g.below(4) as usize;
        let set = random_set(&mut g, "p", 6);
        let gm: Vec<f64> = (0..objects).map(|_| g.value()).collect();
        let t = trace_of(&[&set], &gm);
        for rule in &set.rules {
            let rows: Vec<_> = t
                .rows
                .iter()
                .filter(|r| r.rule.as_deref() == Some(&rule.id))
                .collect();
            assert_eq!(rows.len(), objects.max(1), "{}", rule.id);
        }
    }
}

#[test]
fn property_a_value_meets_every_criterion_exactly_when_it_meets_the_governing_one() {
    let mut g = Gen(0x5eed_0003);
    for _ in 0..300 {
        let set = random_set(&mut g, "p", 6);
        let t = trace_of(&[&set], &[g.value()]);
        for lower in [true, false] {
            let req: Vec<_> = t
                .rows
                .iter()
                .filter(|r| r.compare.unwrap().is_lower_bound() == lower)
                .collect();
            if req.is_empty() {
                continue;
            }
            let any_block = req
                .iter()
                .any(|r| r.consequence == Some(ovcd_core::rules::Consequence::Block));
            let pool: Vec<_> = req
                .iter()
                .filter(|r| {
                    !any_block || r.consequence == Some(ovcd_core::rules::Consequence::Block)
                })
                .collect();
            let gov: Vec<_> = req.iter().filter(|r| r.governing).collect();
            assert_eq!(gov.len(), 1);
            assert!(pool.iter().any(|r| r.rule == gov[0].rule));
            let meets = |r: &&&ovcd_core::rule_trace::TraceRow| r.outcome == Outcome::Pass;
            assert_eq!(pool.iter().all(meets), meets(&gov[0]));
        }
    }
}

#[test]
fn property_adding_a_rule_never_turns_a_failing_option_into_a_passing_one() {
    let mut g = Gen(0x5eed_0004);
    for _ in 0..300 {
        let a = random_set(&mut g, "a", 4);
        let b = random_set(&mut g, "b", 4);
        let gm: Vec<f64> = (0..2).map(|_| g.value()).collect();
        if !trace_of(&[&a], &gm).passes() {
            assert!(!trace_of(&[&a, &b], &gm).passes());
        }
    }
}

#[test]
fn property_failures_are_exactly_the_failing_rows_of_the_trace() {
    // The arrangement search tests each candidate with `failures`, then builds the trace once for
    // the answer: the two must agree on every case, a Measure that cannot be computed included.
    let mut g = Gen(0x5eed_0006);
    for _ in 0..300 {
        let a = random_set(&mut g, "a", 4);
        let b = random_set(&mut g, "b", 4);
        let objects = g.below(4) as usize;
        let mut source = fake(&(0..objects).map(|_| g.value()).collect::<Vec<_>>());
        for v in &mut source.values[0].1 {
            if g.below(5) == 0 {
                *v = Err("cannot compute".into());
            }
        }
        for sets in [vec![&a], vec![&a, &b], vec![&b, &a, &b]] {
            let want: Vec<Failure> = evaluate(&sets, &source)
                .rows
                .into_iter()
                .filter(|r| r.outcome == Outcome::Fail)
                .map(|r| Failure {
                    rule: r.rule.unwrap(),
                    measure: r.measure.unwrap(),
                    object_type: r.object_type,
                    object: r.object,
                })
                .collect();
            assert_eq!(failures(&sets, &source), want);
        }
    }
}

#[test]
fn property_the_governing_row_does_not_change_when_only_the_value_changes() {
    let mut g = Gen(0x5eed_0005);
    for _ in 0..300 {
        let set = random_set(&mut g, "p", 6);
        let gov = |t: &Trace| -> Vec<Option<String>> {
            t.rows
                .iter()
                .filter(|r| r.governing)
                .map(|r| r.rule.clone())
                .collect()
        };
        let first = gov(&trace_of(&[&set], &[g.value()]));
        for _ in 0..3 {
            assert_eq!(gov(&trace_of(&[&set], &[g.value()])), first);
        }
    }
}

// --- through a real run -------------------------------------------------------------------------

fn fixture_design(name: &str) -> ovcd_format::Design {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name),
    )
    .unwrap();
    ovcd_format::check(&text, &ovcd_format::Profile::built_in())
        .design
        .unwrap()
}

fn example_library() -> (Library, Vec<(String, String)>) {
    let mut lib = Library::built_in();
    lib.add_text(include_str!("../rules/examples/example-1.0.0.json"))
        .unwrap();
    (lib, vec![("example".into(), "1.0.0".into())])
}

#[test]
fn every_run_has_a_rule_trace_outside_the_results_hash() {
    let (lib, extra) = example_library();
    for name in [
        "gas_igc_2g_exact.vdesign",
        "dry_bulk_box_hold.vdesign",
        "two_options.vdesign",
    ] {
        let design = fixture_design(name);
        let plain = ovcd_format::run_design(&design);
        let with = ovcd_format::run_design_with(&design, &lib, &extra);
        for (p, w) in plain.iter().zip(&with) {
            let (
                ovcd_format::RunOutcome::Results {
                    results: rp,
                    hash: hp,
                },
                ovcd_format::RunOutcome::Results {
                    results: rw,
                    hash: hw,
                },
            ) = (&p.outcome, &w.outcome)
            else {
                panic!("{name}: not computed");
            };
            assert_eq!(hp, hw, "{name}: a Rule Set moved the results hash");
            assert!(rp["rule_trace"].is_object(), "{name}");
            // The default stack first, then the loaded set.
            assert_eq!(
                rw["rule_trace"]["rule_sets"].as_array().unwrap().last(),
                Some(&json!("example@1.0.0")),
                "{name}"
            );
        }
    }
}

#[test]
fn the_gz_area_measure_is_the_number_the_stability_criteria_use() {
    let (lib, extra) = example_library();
    let design = fixture_design("gas_igc_2g_exact.vdesign");
    let run = &ovcd_format::run_design_with(&design, &lib, &extra)[0];
    let ovcd_format::RunOutcome::Results { results, .. } = &run.outcome else {
        panic!("not computed");
    };
    let conditions = results["stability"]["conditions"].as_array().unwrap();
    let rows: Vec<&Value> = results["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["rule"] == "example/1.1/area-0-30")
        .collect();
    assert_eq!(rows.len(), conditions.len());
    for (row, c) in rows.iter().zip(conditions) {
        assert_eq!(row["object"], c["name"]);
        let criterion = c["criteria"]
            .as_array()
            .unwrap()
            .iter()
            .find(|k| k["name"] == "Area 0\u{2013}30\u{b0}")
            .unwrap();
        assert_eq!(row["value"], criterion["value"], "bit for bit");
        assert_eq!(row["outcome"] == "pass", criterion["ok"] == true);
    }
}

#[test]
fn a_set_that_does_not_apply_to_a_dry_option_gives_one_row() {
    let (lib, extra) = example_library();
    let design = fixture_design("dry_bulk_box_hold.vdesign");
    let run = &ovcd_format::run_design_with(&design, &lib, &extra)[0];
    let ovcd_format::RunOutcome::Results { results, .. } = &run.outcome else {
        panic!("not computed");
    };
    let rows: Vec<&Value> = results["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["rule_sets"][0] == "example@1.0.0")
        .collect();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["outcome"], "not applicable");
    assert_eq!(rows[0]["rule"], Value::Null);
}

#[test]
fn property_a_box_trace_gives_the_verdicts_of_the_box_condition_criteria() {
    // The evaluation harness (issue #278) reads the Rule Trace of a box loading condition; each
    // is2008-a row must give the verdict and value of the criterion the stability check uses.
    let pairs = [
        ("Area 0–30°", "is2008/A.2.2.1/area-0-30"),
        ("Area 0–40°", "is2008/A.2.2.1/area-0-40"),
        ("Area 30–40°", "is2008/A.2.2.1/area-30-40"),
        ("GZ at ≥ 30°", "is2008/A.2.2.2/gz-30"),
        ("Angle of max GZ", "is2008/A.2.2.3/angle-max-gz"),
        ("Initial GM (fluid)", "is2008/A.2.2.4/gm0"),
    ];
    let mut g = Gen(0x5eed_0007);
    let lib = Library::built_in();
    for _ in 0..60 {
        let beam = 8.0 + g.below(20) as f64;
        let depth = 4.0 + g.below(6) as f64;
        let draught = depth * (0.3 + 0.1 * g.below(6) as f64);
        let kg = depth * (0.3 + 0.1 * g.below(7) as f64);
        let c =
            ovcd_core::gas_stability::box_condition(beam, depth, draught, kg, 0.0, "IS_CODE_2008")
                .unwrap();
        let t = ovcd_format::trace::box_trace(c.clone(), &lib, &[("is2008-a", "1.0.0")]).unwrap();
        let rows = t["rows"].as_array().unwrap();
        for (name, id) in pairs {
            let crit = c.criteria.iter().find(|x| x.name == name).unwrap();
            let row = rows.iter().find(|r| r["rule"] == id).unwrap();
            assert_eq!(row["object"], "Box");
            assert_eq!(
                row["outcome"] == "pass",
                crit.ok,
                "{id} on {beam} x {depth} at {draught}, KG {kg}"
            );
        }
        let note = rows
            .iter()
            .find(|r| r["rule"] == "is2008/A.2.2/condition")
            .unwrap();
        assert_eq!(note["value"], c.criteria.iter().all(|x| x.ok));
    }
}
