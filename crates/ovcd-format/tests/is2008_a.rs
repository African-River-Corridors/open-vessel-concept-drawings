//! Seam: the built-in Rule Set `is2008-a` 1.0.0 (spec 0004, issue #270).
//!
//! Label: sourced - IS Code 2008, resolution MSC.267(85), Part A 2.2.1 to 2.2.4 (MSC.267-85.pdf,
//! annex pp. 11-12)

use ovcd_core::rule_trace::{
    MeasureInfo, MeasureSource, MeasureValue, ObjectType, Outcome, evaluate,
};
use ovcd_core::rules::{ArgValue, MeasureRef, RuleSet};
use ovcd_format::rule_set::Library;
use ovcd_format::trace::measure_info;
use serde_json::Value;

fn set() -> RuleSet {
    Library::built_in()
        .find("is2008-a", "1.0.0")
        .expect("built in")
        .clone()
}

/// One loading condition where every Measure has the value given for its id (and arguments).
struct One(Vec<(&'static str, f64)>);

fn key(m: &MeasureRef) -> String {
    let mut k = m.id.clone();
    for (_, v) in &m.args {
        if let ArgValue::Number(x) = v {
            k.push_str(&format!(":{x}"));
        }
    }
    k
}

impl MeasureSource for One {
    fn def(&self, id: &str) -> Option<MeasureInfo> {
        measure_info(id)
    }
    fn objects(&self, object: ObjectType) -> Vec<String> {
        match object {
            ObjectType::LoadingCondition => vec!["Laden".into()],
            ObjectType::Option => vec!["Option".into()],
            _ => Vec::new(),
        }
    }
    fn value(&self, m: &MeasureRef, _: usize) -> Result<MeasureValue, String> {
        let k = key(m);
        self.0
            .iter()
            .find(|(id, _)| *id == k)
            .map(|(_, v)| MeasureValue::Number(*v))
            .ok_or(k)
    }
    fn input(&self, _: &str) -> Option<String> {
        None
    }
}

/// Values that meet every criterion with room to spare.
fn good() -> Vec<(&'static str, f64)> {
    vec![
        ("gz.area:0:30", 0.2),
        ("gz.area:0:40", 0.3),
        ("gz.area:30:40", 0.1),
        ("gz.max_from:30", 0.5),
        ("gz.angle_of_max", 30.0),
        ("gm0", 1.0),
    ]
}

fn outcome_of(rule: &str, measure: &'static str, value: f64) -> Outcome {
    let mut values = good();
    for v in values.iter_mut() {
        if v.0 == measure {
            v.1 = value;
        }
    }
    let s = set();
    let t = evaluate(&[&s], &One(values));
    t.rows
        .iter()
        .find(|r| r.rule.as_deref() == Some(rule))
        .unwrap()
        .outcome
}

#[test]
fn each_criterion_at_both_sides_of_its_limit() {
    // Worked by hand from the text: "not less than" and "at least" are `ge`, so the limit itself
    // passes and anything below it fails.
    for (rule, measure, limit, below) in [
        // 2.2.1: area to 30 degrees not less than 0.055 m-rad.
        ("is2008/A.2.2.1/area-0-30", "gz.area:0:30", 0.055, 0.0549),
        // 2.2.1: to 40 degrees not less than 0.09 m-rad.
        ("is2008/A.2.2.1/area-0-40", "gz.area:0:40", 0.09, 0.0899),
        // 2.2.1: between 30 and 40 degrees not less than 0.03 m-rad.
        ("is2008/A.2.2.1/area-30-40", "gz.area:30:40", 0.03, 0.0299),
        // 2.2.2: GZ at least 0.2 m at an angle of heel equal to or greater than 30 degrees.
        ("is2008/A.2.2.2/gz-30", "gz.max_from:30", 0.2, 0.199),
        // 2.2.3: maximum righting lever at an angle of heel not less than 25 degrees.
        ("is2008/A.2.2.3/angle-max-gz", "gz.angle_of_max", 25.0, 24.0),
        // 2.2.4: GM0 not less than 0.15 m.
        ("is2008/A.2.2.4/gm0", "gm0", 0.15, 0.149),
    ] {
        assert_eq!(
            outcome_of(rule, measure, limit),
            Outcome::Pass,
            "{rule} at {limit}"
        );
        assert_eq!(
            outcome_of(rule, measure, below),
            Outcome::Fail,
            "{rule} at {below}"
        );
    }
}

#[test]
fn the_set_holds_seven_rules_and_the_weather_criterion_is_not_evaluated() {
    let s = set();
    // Six criteria, the note of each condition's check (#272) and the weather criterion.
    assert_eq!(s.rules.len(), 8);
    let weather = s.rule("is2008/A.2.3/weather").unwrap();
    assert!(!weather.evaluated);
    let t = evaluate(&[&s], &One(good()));
    let w: Vec<_> = t
        .rows
        .iter()
        .filter(|r| r.rule.as_deref() == Some("is2008/A.2.3/weather"))
        .collect();
    assert_eq!(w.len(), 1);
    assert_eq!(w[0].outcome, Outcome::NotEvaluated);
    // A pass names it: no output says "meets every rule" while it is not evaluated.
    assert!(t.passes());
    assert_eq!(t.blocking_not_evaluated().len(), 1);
}

#[test]
fn the_down_flooding_finding_is_on_both_40_degree_rules() {
    let s = set();
    for id in ["is2008/A.2.2.1/area-0-40", "is2008/A.2.2.1/area-30-40"] {
        assert_eq!(s.rule(id).unwrap().finding.as_deref(), Some("#282"), "{id}");
    }
    assert_eq!(s.rule("is2008/A.2.2.1/area-0-30").unwrap().finding, None);
}

#[test]
fn the_limits_are_the_limits_the_engine_checks_with() {
    // The stability check of the gas engine still holds its own constants until `fit` reads the
    // trace (#277). Until then the set and the engine must agree on every limit and comparison.
    let c =
        ovcd_core::gas_stability::box_condition(10.0, 3.0, 1.5, 2.0, 0.0, "IS_CODE_2008").unwrap();
    let s = set();
    let ids = [
        "is2008/A.2.2.1/area-0-30",
        "is2008/A.2.2.1/area-0-40",
        "is2008/A.2.2.1/area-30-40",
        "is2008/A.2.2.2/gz-30",
        "is2008/A.2.2.3/angle-max-gz",
        "is2008/A.2.2.4/gm0",
    ];
    assert_eq!(c.criteria.len(), ids.len());
    for (criterion, id) in c.criteria.iter().zip(ids) {
        let rule = s.rule(id).unwrap();
        let Some(ovcd_core::rules::Limit::Constant { value, .. }) = &rule.limit else {
            panic!("{id}: a constant limit");
        };
        assert_eq!(*value, criterion.limit, "{id}");
        assert_eq!(rule.compare, Some(ovcd_core::rules::Compare::Ge), "{id}");
        assert_eq!(rule.tolerance, 0.0, "{id}");
    }
}

// --- through real runs --------------------------------------------------------------------------

fn results_of(file: &str) -> Vec<Value> {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(file),
    )
    .unwrap();
    let design = ovcd_format::check(&text, &ovcd_format::Profile::built_in())
        .design
        .unwrap();
    ovcd_format::run_design(&design)
        .into_iter()
        .filter_map(|r| match r.outcome {
            ovcd_format::RunOutcome::Results { results, .. } => Some(results),
            _ => None,
        })
        .collect()
}

/// The criterion name of the engine's check list for each evaluated rule id.
const NAMES: [(&str, &str); 6] = [
    ("is2008/A.2.2.1/area-0-30", "Area 0\u{2013}30\u{b0}"),
    ("is2008/A.2.2.1/area-0-40", "Area 0\u{2013}40\u{b0}"),
    ("is2008/A.2.2.1/area-30-40", "Area 30\u{2013}40\u{b0}"),
    ("is2008/A.2.2.2/gz-30", "GZ at \u{2265} 30\u{b0}"),
    ("is2008/A.2.2.3/angle-max-gz", "Angle of max GZ"),
    ("is2008/A.2.2.4/gm0", "Initial GM (fluid)"),
];

#[test]
fn every_gas_option_has_the_is_code_rows_of_each_loading_condition() {
    for file in [
        "gas_igc_2g_exact.vdesign",
        "gas_igc_2g_self_propelled.vdesign",
    ] {
        for results in results_of(file) {
            let rows = results["rule_trace"]["rows"].as_array().unwrap();
            let conditions = results["stability"]["conditions"].as_array().unwrap();
            assert_eq!(conditions.len(), 3, "{file}");
            for c in conditions {
                for (id, name) in NAMES {
                    let row = rows
                        .iter()
                        .find(|r| r["rule"] == id && r["object"] == c["name"])
                        .unwrap_or_else(|| panic!("{file}: no {id} row for {}", c["name"]));
                    let criterion = c["criteria"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|k| k["name"] == name)
                        .unwrap();
                    // The same number, bit for bit, and the same verdict.
                    assert_eq!(
                        row["value"].as_f64(),
                        criterion["value"].as_f64(),
                        "{file} {id}"
                    );
                    assert_eq!(
                        row["outcome"] == "pass",
                        criterion["ok"] == true,
                        "{file} {id}"
                    );
                    assert_eq!(row["rule_sets"][0], "is2008-a@1.0.0");
                }
            }
            // Six evaluated Rules per condition, and the weather Rule once.
            let is_code = rows
                .iter()
                .filter(|r| r["rule_sets"][0] == "is2008-a@1.0.0")
                .count();
            // ... and the note of each condition's own check (#272).
            assert_eq!(is_code, 7 * conditions.len() + 1, "{file}");
        }
    }
}

#[test]
fn a_dry_bulk_option_lists_the_is_code_rules_as_not_evaluated() {
    for results in results_of("dry_bulk_box_hold.vdesign") {
        let trace = &results["rule_trace"];
        assert_eq!(
            trace["rule_sets"],
            serde_json::json!(["is2008-a@1.0.0", "ovcd@1.0.0"])
        );
        let rows: Vec<&serde_json::Value> = trace["rows"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["rule_sets"][0] == "is2008-a@1.0.0")
            .collect();
        // The six criteria and the weather criterion block; the condition note does not.
        assert_eq!(rows.len(), 8);
        assert!(rows.iter().all(|r| r["outcome"] == "not evaluated"));
        assert_eq!(
            trace["verdict"]["blocking_not_evaluated"]
                .as_array()
                .unwrap()
                .len(),
            7
        );
        assert!(
            trace["verdict"]["summary"]
                .as_str()
                .unwrap()
                .contains("7 blocking Rule(s) not evaluated")
        );
    }
}
