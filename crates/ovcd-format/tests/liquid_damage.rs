//! Damage stability of a liquid bulk hull (issue #341, spec 0006): the ADN 9.3.2.15 and 9.3.3.15 extents on
//! the laden departure condition, by the lost buoyancy method on the Sections Table, and the Rule rows that
//! read them.
//!
//! Label: sourced - ADN 2025 (ECE/TRANS/351) 9.3.2.15.1 and 9.3.3.15.1: the longitudinal extent is 0.10 L but
//! not less than 5.00 m; 9.3.2.15.2 and 9.3.3.15.2: at equilibrium the heel is at most 12 degrees and the
//! righting lever beyond it reaches 0.05 m with an area of 0.0065 m.rad up to 27 degrees. The physics of the
//! solver is checked against closed forms in `ovcd-core/tests/damage_stability.rs`; these tests are about
//! the layout cut into damages and the rows. No published damage case with righting lever figures was
//! found (the acceptance line of the ticket stays open).

use ovcd_format::rule_set::Library;
use ovcd_format::trace::{LiquidTraceCase, liquid_trace};
use ovcd_format::{Profile, RunOutcome, check, run_design};
use serde_json::{Value, json};

const FIXTURES: [&str; 3] = [
    "crude_integral_tanks_pushed",
    "crude_integral_tanks_self_propelled",
    "crude_integral_tanks_coastal",
];

fn raw_fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn typed(name: &str, edits: &[(&str, Value)]) -> String {
    let mut file: Value = serde_json::from_str(&raw_fixture(name)).unwrap();
    let inputs = file["options"][0]["inputs"].as_object_mut().unwrap();
    for (key, value) in edits {
        inputs.insert(
            (*key).to_string(),
            json!({ "value": value, "source": "typed" }),
        );
    }
    file.to_string()
}

fn results(text: &str) -> Value {
    let checked = check(text, &Profile::built_in());
    assert_eq!(checked.report.errors, Vec::<String>::new());
    let design = checked.design.unwrap();
    match run_design(&design).remove(0).outcome {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn damage_rows<'a>(r: &'a Value, rule: &str) -> Vec<&'a Value> {
    r["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["rule"] == rule)
        .collect()
}

#[test]
fn every_liquid_fixture_has_damage_cases_and_the_adn_rule_has_one_row_for_each() {
    for name in FIXTURES {
        let r = results(&raw_fixture(name));
        let cases = r["damage"]
            .as_array()
            .unwrap_or_else(|| panic!("{name}: no damage"));
        assert!(cases.len() >= 6, "{name}: {} cases", cases.len());
        // crude gives type C (unknown flags): only the rule of type C has rows
        let (rule, other) = (
            "adn/9.3.2.15.1/damage-stability",
            "adn/9.3.3.15.1/damage-stability",
        );
        let rows = damage_rows(&r, rule);
        assert_eq!(rows.len(), cases.len(), "{name}");
        for (row, case) in rows.iter().zip(cases) {
            assert_eq!(row["object"], case["name"], "{name}");
            // a warn Rule: a case that does not meet the criteria is a warning, not a failure
            let want = if case["ok"] == true { "pass" } else { "warn" };
            assert_eq!(row["outcome"], want, "{name} {}", case["name"]);
            assert_eq!(row["object_type"], "damage-case");
        }
        assert!(
            damage_rows(&r, other)
                .iter()
                .all(|row| row["outcome"] == "not applicable"),
            "{name}"
        );
        // the damage is not in the results hash
        assert!(ovcd_format::UNHASHED_KEYS.contains(&"damage"));
    }
}

#[test]
fn each_damage_is_0_10_l_long_and_floods_whole_compartments_the_extent_touches() {
    let r = results(&raw_fixture("crude_integral_tanks_pushed"));
    let loa = r["dims"]["loa_m"].as_f64().unwrap();
    let extent = (0.10 * loa).max(5.0);
    for case in r["damage"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        // "side (starboard) damage, 20.3 to 27.9 m": the window of the damage, to 0.1 m
        let nums: Vec<f64> = name
            .split(|c: char| !(c.is_ascii_digit() || c == '.'))
            .filter_map(|t| t.parse().ok())
            .collect();
        assert_eq!(nums.len(), 2, "{name}");
        assert!(
            (nums[1] - nums[0] - extent).abs() < 0.11,
            "{name}: {extent}"
        );
        let flooded: Vec<&str> = case["flooded"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(!flooded.is_empty(), "{name}");
        // The cargo tanks are full at the maximum draught, so the damage floods ballast and end spaces only.
        for f in &flooded {
            assert!(
                f.starts_with("WB")
                    || f.starts_with("DB")
                    || f.starts_with("CD")
                    || f.starts_with("PR"),
                "{name}: {f}"
            );
        }
        if name.starts_with("bottom") {
            // adjacent compartments athwartships flood too: a double bottom compartment comes with its twin
            for f in flooded.iter().filter(|f| f.starts_with("DB")) {
                let twin = if f.contains('P') {
                    f.replacen('P', "S", 1)
                } else {
                    f.replacen('S', "P", 1)
                };
                assert!(
                    flooded.contains(&twin.as_str()),
                    "{name}: {f} without {twin}"
                );
            }
        } else {
            // a side damage to starboard floods no port compartment
            assert!(
                flooded
                    .iter()
                    .all(|f| !f.contains('P') || f.starts_with("PR")),
                "{name}: {flooded:?}"
            );
        }
    }
}

#[test]
fn the_verdict_of_each_case_follows_its_numbers() {
    // A small hull with a narrow beam so that some damages heel past 12 degrees, and some do not.
    let text = typed(
        "crude_integral_tanks_pushed",
        &[
            ("length_overall_m", json!(30.0)),
            ("beam_m", json!(4.5)),
            ("moulded_depth_m", json!(2.0)),
            ("liquid_tanks_in_line", json!(2.0)),
        ],
    );
    let r = results(&text);
    let cases = r["damage"].as_array().unwrap();
    let (mut passes, mut fails) = (0, 0);
    for c in cases {
        let (heel, gz, area) = (
            c["heel_deg"].as_f64().unwrap(),
            c["max_gz_m"].as_f64().unwrap(),
            c["area_m_rad"].as_f64().unwrap(),
        );
        let want = c["floats"] == "yes" && heel.abs() <= 12.0 && gz >= 0.05 && area >= 0.0065;
        assert_eq!(c["ok"] == true, want, "{c}");
        if want { passes += 1 } else { fails += 1 }
    }
    assert!(passes >= 1 && fails >= 1, "{passes} pass, {fails} fail");
    // a side damage heels to the damaged side; a bottom damage does not heel
    for c in cases {
        let heel = c["heel_deg"].as_f64().unwrap();
        if c["name"].as_str().unwrap().starts_with("bottom") {
            assert!(heel.abs() < 0.01, "{c}");
        } else {
            assert!(heel > 0.1, "{c}");
        }
    }
}

#[test]
fn a_damaged_hull_is_no_safer_than_an_intact_one_and_a_wider_beam_heels_less() {
    let heel = |beam: f64| {
        let r = results(&typed(
            "crude_integral_tanks_pushed",
            &[("beam_m", json!(beam)), ("moulded_depth_m", json!(3.5))],
        ));
        r["damage"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["name"].as_str().unwrap().starts_with("side"))
            .map(|c| c["heel_deg"].as_f64().unwrap())
            .fold(0.0_f64, f64::max)
    };
    assert!(heel(14.0) < heel(9.0), "{} {}", heel(14.0), heel(9.0));
}

#[test]
fn the_layout_harness_has_no_loading_condition_so_the_rule_is_not_evaluated_with_that_reason() {
    use ovcd_core::liquid_bulk::{LiquidLayout, PumpArrangement};
    let case = LiquidTraceCase {
        loa_m: 76.5,
        beam_m: 11.4,
        depth_m: 3.5,
        zone_aft_m: 3.0,
        zone_fwd_m: 70.0,
        layout: LiquidLayout {
            double_side_m: 1.0,
            double_bottom_m: 0.7,
            tanks_in_line: 5,
            centreline_bulkhead: true,
            cofferdam_m: 0.6,
            pump_arrangement: PumpArrangement::Deepwell,
            slop_share: 0.0,
            fill_ratio: 0.98,
            heating_kw_per_m3: 0.0,
            lightship_k_t_per_m3: 0.1141,
        },
        cargo_density_t_m3: 0.87,
        deadweight_t: 1500.0,
        service_area: "river-only".to_string(),
        adn_vessel_type: "C".to_string(),
    };
    let trace = liquid_trace(&case, &Library::built_in(), &[("adn-tank", "1.0.0")]).unwrap();
    let rows: Vec<&Value> = trace["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["rule"] == "adn/9.3.2.15.1/damage-stability")
        .collect();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["outcome"], "not evaluated");
    assert!(
        rows[0]["gap"]
            .as_str()
            .unwrap()
            .contains("no loading condition")
    );
}

/// MARPOL Annex I regulation 28 (issue #341): the coastal fixture has `marpol-i` in its stack, so it has a
/// `marpol_damage` block and a row of `marpol/28/damage-stability` for each case; the river fixtures have none.
#[test]
fn the_marpol_rule_has_one_row_for_each_marpol_damage_and_only_under_the_coastal_stack() {
    let r = results(&raw_fixture("crude_integral_tanks_coastal"));
    let cases = r["marpol_damage"].as_array().expect("marpol_damage");
    assert!(cases.len() >= 6, "{} cases", cases.len());
    let rows = damage_rows(&r, "marpol/28/damage-stability");
    assert_eq!(rows.len(), cases.len());
    for (row, case) in rows.iter().zip(cases) {
        assert_eq!(row["object"], case["name"]);
        assert_eq!(row["object_type"], "marpol-damage-case");
        let want = if case["ok"] == true { "pass" } else { "warn" };
        assert_eq!(row["outcome"], want, "{}", case["name"]);
    }
    // the extents reach the cargo tanks: some case floods a cargo tank (its name starts with a digit: 1S, 2P)
    assert!(
        cases
            .iter()
            .any(|c| c["flooded"].as_array().unwrap().iter().any(|n| n
                .as_str()
                .unwrap()
                .starts_with(|ch: char| ch.is_ascii_digit()))),
        "no case floods a cargo tank"
    );
    // not in the results hash
    assert!(ovcd_format::UNHASHED_KEYS.contains(&"marpol_damage"));
    let river = results(&raw_fixture("crude_integral_tanks_pushed"));
    assert!(river.get("marpol_damage").is_none());
}
