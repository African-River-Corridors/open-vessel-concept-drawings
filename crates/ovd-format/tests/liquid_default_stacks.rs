//! The default Rule Set stacks of a liquid bulk (crude oil) Option, by Service Area (issue #339, spec 0006
//! section 5.1 and spec 0004, "Service Area and default stacks").
//!
//! Label: property. Every test states a rule: the stack a new Option takes, which sets a fixture's run
//! checks it against, and that a gas or dry bulk Option keeps the stack it had.

use ovd_format::{DesignSession, Profile, RunOutcome, check, run_design};
use serde_json::{Value, json};

fn ids(s: &DesignSession) -> Vec<String> {
    s.design().options[0]
        .input("rule_sets")
        .unwrap()
        .value
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap().to_string())
        .collect()
}

const RIVER: [&str; 4] = ["is2008-a", "es-trin-2025", "adn-tank", "ovd"];
const COASTAL: [&str; 8] = [
    "is2008-a",
    "es-trin-2025",
    "icll-freeboard",
    "solas-v22",
    "abs-barges",
    "adn-tank",
    "marpol-i",
    "ovd",
];

fn crude_session(area: &str) -> DesignSession {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    s.set_input("Option 1", "cargo", json!("crude_oil"))
        .unwrap();
    s.set_input("Option 1", "service_area", json!(area))
        .unwrap();
    s
}

#[test]
fn a_crude_oil_option_takes_the_liquid_stack_of_its_service_area() {
    assert_eq!(ids(&crude_session("river-only")), RIVER);
    assert_eq!(ids(&crude_session("coastal-and-river")), COASTAL);
}

#[test]
fn the_service_area_changes_the_stack_and_the_cargo_changes_it_back() {
    let mut s = crude_session("river-only");
    s.set_input("Option 1", "service_area", json!("coastal-and-river"))
        .unwrap();
    assert_eq!(ids(&s), COASTAL);
    s.set_input("Option 1", "cargo", json!("bauxite")).unwrap();
    assert_eq!(
        ids(&s),
        [
            "is2008-a",
            "es-trin-2025",
            "icll-freeboard",
            "solas-v22",
            "abs-barges",
            "ovd"
        ]
    );
    s.set_input("Option 1", "cargo", json!("crude_oil"))
        .unwrap();
    assert_eq!(ids(&s), COASTAL);
    // A set the designer adds stays: the optional national set.
    s.add_rule_set("Option 1", "normam-202@1.0.0").unwrap();
    s.set_input("Option 1", "service_area", json!("river-only"))
        .unwrap();
    let mut want: Vec<&str> = RIVER.to_vec();
    want.push("normam-202");
    assert_eq!(ids(&s), want);
}

#[test]
fn the_profile_holds_the_liquid_stacks_and_the_national_set_is_in_no_default_stack() {
    let p = Profile::built_in();
    assert_eq!(p.default_liquid_stack("river-only").unwrap(), RIVER);
    assert_eq!(
        p.default_liquid_stack("coastal-and-river").unwrap(),
        COASTAL
    );
    for area in ["river-only", "coastal-and-river"] {
        for stack in [
            p.default_stack(area, true).unwrap(),
            p.default_stack(area, false).unwrap(),
            p.default_liquid_stack(area).unwrap(),
        ] {
            assert!(!stack.iter().any(|s| s == "normam-202"), "{area}");
            assert_eq!(stack.last().unwrap(), "ovd");
        }
        // The liquid stack is the other stack with the liquid sets added.
        let other = p.default_stack(area, false).unwrap();
        let liquid = p.default_liquid_stack(area).unwrap();
        assert!(other.iter().all(|s| liquid.contains(s)), "{area}");
        assert!(liquid.contains(&"adn-tank".to_string()));
        assert_eq!(
            liquid.contains(&"marpol-i".to_string()),
            area == "coastal-and-river"
        );
    }
}

#[test]
fn a_profile_written_before_the_liquid_stack_takes_the_other_stack_with_the_liquid_sets_added() {
    let mut text: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    for area in ["river-only", "coastal-and-river"] {
        text["inputs"]["rule_sets"]["default_stacks"][area]
            .as_object_mut()
            .unwrap()
            .remove("liquid_bulk");
    }
    let older = Profile::from_json(&text.to_string()).unwrap();
    assert_eq!(older.default_liquid_stack("river-only").unwrap(), RIVER);
    assert_eq!(
        older.default_liquid_stack("coastal-and-river").unwrap(),
        COASTAL
    );
    // A liquid stack without `ovd` is refused.
    let mut bad: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    bad["inputs"]["rule_sets"]["default_stacks"]["river-only"]["liquid_bulk"] = json!(["adn-tank"]);
    let e = Profile::from_json(&bad.to_string()).unwrap_err();
    assert!(e.contains("liquid bulk") && e.contains("ovd"), "{e}");
}

fn run_fixture(name: &str) -> Value {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let design = check(
        &std::fs::read_to_string(path).unwrap(),
        &Profile::built_in(),
    )
    .design
    .unwrap();
    match run_design(&design).remove(0).outcome {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn outcomes(r: &Value, prefix: &str) -> Vec<String> {
    r["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["rule"]
                .as_str()
                .is_some_and(|id| id.starts_with(prefix))
        })
        .map(|row| row["outcome"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn the_river_fixture_shows_the_adn_rows_and_no_marpol_row() {
    let r = run_fixture("crude_integral_tanks_pushed");
    assert_eq!(r["rule_trace"]["service_area"], "river-only");
    assert_eq!(
        r["rule_trace"]["rule_sets"],
        json!([
            "is2008-a@1.0.0",
            "es-trin-2025@1.0.0",
            "adn-tank@1.0.0",
            "ovd@3.0.0"
        ])
    );
    assert!(!outcomes(&r, "adn/9.3.2.11.7").is_empty());
    assert!(outcomes(&r, "marpol/").is_empty());
    assert!(outcomes(&r, "icll/").is_empty());
    assert!(outcomes(&r, "normam/").is_empty());
    // The default Crude Grade is type C: the type C rows are evaluated, the type N rows do not apply.
    assert!(
        outcomes(&r, "adn/9.3.2.11.7")
            .iter()
            .all(|o| o != "not applicable")
    );
    assert!(
        outcomes(&r, "adn/9.3.3.11.7")
            .iter()
            .all(|o| o == "not applicable")
    );
}

#[test]
fn the_coastal_fixture_shows_the_marpol_and_icll_rows_besides_the_adn_ones() {
    let r = run_fixture("crude_integral_tanks_coastal");
    assert_eq!(r["rule_trace"]["service_area"], "coastal-and-river");
    assert_eq!(
        r["rule_trace"]["rule_sets"],
        json!([
            "is2008-a@1.0.0",
            "es-trin-2025@1.0.0",
            "icll-freeboard@1.0.0",
            "solas-v22@1.0.0",
            "adn-tank@1.0.0",
            "marpol-i@1.0.0",
            "ovd@3.0.0"
        ])
    );
    assert!(!outcomes(&r, "adn/9.3.2.11.7").is_empty());
    assert!(!outcomes(&r, "marpol/29").is_empty());
    assert!(!outcomes(&r, "icll/").is_empty());
    // The coastal Default for the slop share is 3 per cent: Reg. 29 passes on it.
    assert!(
        outcomes(&r, "marpol/29").iter().all(|o| o == "pass"),
        "{:?}",
        outcomes(&r, "marpol/29")
    );
    // A tanker of this size is under 5,000 t deadweight: Reg. 19.6 applies, Reg. 19.3 does not.
    assert!(
        outcomes(&r, "marpol/19.6.1")
            .iter()
            .all(|o| o != "not applicable")
    );
    assert!(
        outcomes(&r, "marpol/19.3.1")
            .iter()
            .all(|o| o == "not applicable")
    );
    // The strictest governs on the double bottom: MARPOL 0.76 m over ADN 0.70 m.
    let governing: Vec<&str> = r["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["governing"] == true && row["measure"]["id"] == "hull.double_bottom_height"
        })
        .map(|row| row["rule"].as_str().unwrap())
        .collect();
    assert_eq!(governing, ["marpol/19.6.1/double-bottom-depth"]);
}

#[test]
fn a_gas_and_a_dry_bulk_option_keep_their_stacks() {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    s.set_input("Option 1", "cargo", json!("LNG")).unwrap();
    assert_eq!(
        ids(&s),
        ["igc-2g", "is2008-a", "lr-inland-gas", "es-trin-2025", "ovd"]
    );
    s.set_input("Option 1", "cargo", json!("bauxite")).unwrap();
    assert_eq!(ids(&s), ["is2008-a", "es-trin-2025", "ovd"]);
}
