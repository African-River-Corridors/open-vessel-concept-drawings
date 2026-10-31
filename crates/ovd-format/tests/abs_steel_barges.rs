//! The Rule Set `abs-steel-barges` 1.0.0 (issue #740, spec 0013): the scantling Rules of ABS Steel
//! Barges at the current edition, as data. The panel model (`ovd_core::hull_panels`) sizes with the
//! same Rules from `ovd_core::scantling_rules`; this file holds the two equal, so the file a reviewer
//! reads is the data the model uses. Each Rule is a `not evaluated` row of the Rule Trace of a coastal
//! Option, with its paragraph.
//!
//! Label: property

use ovd_core::rule_trace::formula_value;
use ovd_core::scantling_rules::{EDITION, RULES, get};
use ovd_format::{Profile, RunOutcome, check, fill_as_new, run_design};
use serde_json::{Value, json};

fn file() -> Value {
    let path = format!(
        "{}/rules/abs-steel-barges/1.0.0.json",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// The operands of each formula shape, as `rule_trace::FORMULA_SHAPES` registers them.
fn operands(shape: &str) -> &'static [&'static str] {
    ovd_core::rule_trace::FORMULA_SHAPES
        .iter()
        .find(|s| s.id == shape)
        .unwrap_or_else(|| panic!("shape {shape}"))
        .operands
}

#[test]
fn the_file_and_the_core_table_hold_the_same_rules_and_numbers() {
    let f = file();
    assert_eq!(f["id"], "abs-steel-barges");
    assert_eq!(f["version"], "1.0.0");
    assert_eq!(
        f["edition"]["edition"],
        "January 2026 (Part 3 January 2026, Part 5 July 2025)"
    );
    assert!(EDITION.ends_with("January 2026 (Part 3 January 2026, Part 5 July 2025)"));
    let rules = f["rules"].as_array().unwrap();
    let ids: Vec<&str> = rules.iter().map(|r| r["id"].as_str().unwrap()).collect();
    let core: Vec<&str> = RULES.iter().map(|r| r.id).collect();
    assert_eq!(ids, core);
    for r in rules {
        let id = r["id"].as_str().unwrap();
        let c = get(id);
        // The locator is the paragraph, with a dot for the slash.
        assert_eq!(
            r["source"]["locator"],
            c.paragraph.replace('/', "."),
            "{id}"
        );
        assert_eq!(r["source"]["edition"], f["edition"]["edition"], "{id}");
        assert_eq!(r["evaluated"], false, "{id}");
        assert_eq!(r["sizes"], true, "{id}");
        assert!(r["gap"].as_str().unwrap().contains("sizing Rule"), "{id}");
        match r["kind"].as_str().unwrap() {
            "limit" => {
                assert_eq!(c.shape, "value", "{id}");
                assert_eq!(c.coefficients.len(), 1, "{id}");
                // Bit for bit.
                assert_eq!(
                    r["limit"]["value"].as_f64().unwrap().to_bits(),
                    c.coefficients[0].1.to_bits(),
                    "{id}"
                );
            }
            "formula" => {
                let lim = r["limit"].as_object().unwrap();
                assert_eq!(lim["shape"], c.shape, "{id}");
                let ops = operands(c.shape);
                let mut coeffs: Vec<(&str, f64)> = lim
                    .iter()
                    .filter(|(k, _)| !["shape", "unit"].contains(&k.as_str()))
                    .filter(|(k, _)| !ops.contains(&k.as_str()))
                    .map(|(k, v)| (k.as_str(), v.as_f64().unwrap()))
                    .collect();
                coeffs.sort_by(|a, b| a.0.cmp(b.0));
                let mut want: Vec<(&str, f64)> = c.coefficients.to_vec();
                want.sort_by(|a, b| a.0.cmp(b.0));
                assert_eq!(coeffs.len(), want.len(), "{id}");
                for ((k, v), (wk, wv)) in coeffs.iter().zip(&want) {
                    assert_eq!((k, v.to_bits()), (wk, wv.to_bits()), "{id}");
                }
                // Every operand names a Measure id.
                for op in ops {
                    assert!(
                        lim[*op].as_str().unwrap().starts_with("hull_steel."),
                        "{id}: {op}"
                    );
                }
            }
            other => panic!("{id}: {other}"),
        }
    }
}

/// The file's numbers give the worked examples of `crates/ovd-core/tests/scantling_rules.rs` through
/// the kernel's formula shapes: the bottom shell at L = 100 m, s = 700 mm, 11.2 mm.
#[test]
fn a_formula_of_the_file_evaluates_to_its_worked_example() {
    let f = file();
    let r = f["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "abs-steel-barges/3-2-2.3.5.1/bottom-shell")
        .unwrap();
    let lim = r["limit"].as_object().unwrap();
    let coeffs: Vec<(String, f64)> = lim
        .iter()
        .filter_map(|(k, v)| v.as_f64().map(|x| (k.clone(), x)))
        .collect();
    let t = formula_value("linear-ls-split-capped", &coeffs, |op| match op {
        "x" => Ok(100.0),
        "s" => Ok(700.0),
        o => Err(o.to_string()),
    })
    .unwrap();
    assert!((t - 11.2).abs() < 1e-9, "{t}");
}

/// The Rule Trace rows of a new Option in `service_area`.
fn rows(cargo: &str, service_area: &str) -> Vec<Value> {
    let raw = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.3.0", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": cargo, "source": "typed" },
            "hull_form": { "value": "raked_box", "source": "typed" },
            "formation": { "value": { "kind": "pushed_barge" }, "source": "typed" },
            "service_area": { "value": service_area, "source": "typed" },
            "length_overall_m": { "value": 100.0, "source": "typed" },
            "beam_m": { "value": 20.0, "source": "typed" },
            "moulded_depth_m": { "value": 6.0, "source": "typed" }
        }}]
    });
    let filled = fill_as_new(&raw.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    let text = filled.design.unwrap().to_text();
    let design = check(&text, &Profile::built_in()).design.unwrap();
    match run_design(&design).remove(0).outcome {
        RunOutcome::Results { results, .. } => {
            results["rule_trace"]["rows"].as_array().unwrap().clone()
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_coastal_option_lists_every_scantling_rule_not_evaluated() {
    let r = rows("bauxite", "coastal-and-river");
    for rule in RULES {
        let outcomes: Vec<&str> = r
            .iter()
            .filter(|x| x["rule"] == rule.id)
            .map(|x| x["outcome"].as_str().unwrap())
            .collect();
        assert_eq!(outcomes, ["not evaluated"], "{}", rule.id);
    }
}

#[test]
fn a_river_only_option_does_not_have_the_set() {
    let r = rows("bauxite", "river-only");
    assert!(
        r.iter()
            .all(|x| !x["rule"].as_str().unwrap().starts_with("abs-steel-barges/"))
    );
}
