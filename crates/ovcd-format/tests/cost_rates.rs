//! Seam: the 14 cost rates are inputs (issue #233) and the capital budget is an output. The rates
//! default to the example rates file, an edit moves that Option's budget and nothing hashed, the
//! budget is kept out of the results hash, and a file saved before the inputs existed opens with the
//! same results hash.
//!
//! Label: property. Every test states a rule (a default equals the example file, a total is the sum
//! of its parts, a hash does not move). No number here is a printed result.

use ovcd_core::barge_cost::example_rates;
use ovcd_format::vdesign::{COST_RATES_BASIS, COST_RATES_BASIS_USER, INPUTS, LEGACY_FIXED_INPUTS};
use ovcd_format::{
    DesignSession, HashStatus, Profile, RunOutcome, UNHASHED_KEYS, check, input_schema,
    results_hash,
};
use serde_json::{Value, json};

/// The cost-rate inputs, in the order of the core's rates.
const RATES: [&str; 14] = [
    "hull_steel_rate",
    "tank_shell_rate",
    "tank_supports_rate",
    "insulation_rate",
    "piping_rate",
    "outfitting_rate",
    "cargo_system_lump",
    "paint_anodes_misc",
    "battery_rate_per_kwh",
    "propulsion_rate_per_kw",
    "wheelhouse_accom_lump",
    "tug_outfitting_lump",
    "yard_margin_fraction",
    "delivery_lump",
];

const FIXTURES: [&str; 6] = [
    "dry_bulk_box_hold",
    "dry_bulk_sloped_hopper",
    "gas_igc_2g_exact",
    "gas_igc_2g_self_propelled",
    "pushed_convoy",
    "two_options",
];

fn raw_fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// A fixture as a file written before the cost rates existed: the 14 keys taken out.
fn old_fixture(name: &str) -> String {
    let mut file: Value = serde_json::from_str(&raw_fixture(name)).unwrap();
    for option in file["options"].as_array_mut().unwrap() {
        let inputs = option["inputs"].as_object_mut().unwrap();
        for key in RATES {
            inputs.remove(key);
        }
    }
    file.to_string()
}

fn open(text: &str) -> DesignSession {
    DesignSession::open(text, Profile::built_in()).unwrap().0
}

fn results(s: &DesignSession, option: usize) -> Value {
    match s.evaluate().options[option].outcome.as_ref() {
        Some(RunOutcome::Results { results, .. }) => results.clone(),
        other => panic!("{other:?}"),
    }
}

/// The rates of the core's example file, by input name.
fn example() -> Vec<(&'static str, f64)> {
    let r = example_rates().unwrap();
    vec![
        ("hull_steel_rate", r.hull_steel_rate),
        ("tank_shell_rate", r.tank_shell_rate),
        ("tank_supports_rate", r.tank_supports_rate),
        ("insulation_rate", r.insulation_rate),
        ("piping_rate", r.piping_rate),
        ("outfitting_rate", r.outfitting_rate),
        ("cargo_system_lump", r.cargo_system_lump),
        ("paint_anodes_misc", r.paint_anodes_misc),
        ("battery_rate_per_kwh", r.battery_rate_per_kwh),
        ("propulsion_rate_per_kw", r.propulsion_rate_per_kw),
        ("wheelhouse_accom_lump", r.wheelhouse_accom_lump),
        ("tug_outfitting_lump", r.tug_outfitting_lump),
        ("yard_margin_fraction", r.yard_margin_fraction),
        ("delivery_lump", r.delivery_lump),
    ]
}

#[test]
fn every_cost_rate_is_an_input_in_the_cost_rates_group_with_the_example_file_default() {
    let example = example();
    assert_eq!(example.iter().map(|(n, _)| *n).collect::<Vec<_>>(), RATES);
    let profile = Profile::built_in();
    let schema = input_schema(&profile);
    assert!(
        schema["groups"]
            .as_array()
            .unwrap()
            .contains(&json!("Cost rates")),
        "the group exists"
    );
    assert!(
        schema["group_notes"]["Cost rates"]
            .as_str()
            .unwrap()
            .contains("replace with yours"),
        "the group carries its basis"
    );
    for (name, want) in example {
        assert!(INPUTS.iter().any(|s| s.name == name), "{name} is an input");
        let entry = schema["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["name"] == name)
            .unwrap();
        assert_eq!(entry["group"], "Cost rates", "{name}");
        assert_eq!(entry["default"].as_f64(), Some(want), "{name} default");
        assert_eq!(
            profile.default(name).and_then(Value::as_f64),
            Some(want),
            "{name}"
        );
        assert!(
            LEGACY_FIXED_INPUTS
                .iter()
                .any(|l| l.name == name && l.value == want),
            "{name}"
        );
    }
}

#[test]
fn a_new_design_holds_the_example_rates_and_its_budget_uses_them() {
    let s = DesignSession::new_design(Profile::built_in()).unwrap();
    for (name, want) in example() {
        assert_eq!(
            s.design().options[0].input(name).unwrap().value.as_f64(),
            Some(want),
            "{name}"
        );
    }
    // The starter Design is dry bulk: no budget. A gas Design has one, with the example currency.
    assert!(results(&s, 0)["cost"].is_null());
    let cost = &results(&open(&raw_fixture("gas_igc_2g_exact")), 0)["cost"].clone();
    assert_eq!(cost["currency"], example_rates().unwrap().currency);
    assert_eq!(
        cost["basis"],
        json!(format!("{COST_RATES_BASIS} Current estimates."))
    );
}

#[test]
fn a_budget_is_the_sum_of_its_parts() {
    for file in ["gas_igc_2g_exact", "gas_igc_2g_self_propelled"] {
        let cost = results(&open(&raw_fixture(file)), 0)["cost"].clone();
        let sum = |key: &str| -> f64 {
            cost[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| r["amount"].as_f64().unwrap())
                .sum()
        };
        // Each part is rounded to whole currency units, so the sums agree to within one unit per item.
        let items = |key: &str| cost[key].as_array().unwrap().len() as f64;
        assert!(
            (sum("barge") - cost["barge_subtotal"].as_f64().unwrap()).abs() <= items("barge"),
            "{file}"
        );
        assert!(
            (sum("tug") - cost["tug_subtotal"].as_f64().unwrap()).abs() <= items("tug"),
            "{file}"
        );
        let total = cost["barge_subtotal"].as_f64().unwrap()
            + cost["tug_subtotal"].as_f64().unwrap()
            + cost["yard_margin"].as_f64().unwrap()
            + cost["delivery"].as_f64().unwrap();
        assert!(
            (total - cost["unit_total"].as_f64().unwrap()).abs() <= 2.0,
            "{file}"
        );
    }
}

#[test]
fn editing_one_rate_changes_that_options_budget_and_nothing_hashed() {
    // A gas Design with a second Option that is a copy of the first.
    let mut s = open(&raw_fixture("gas_igc_2g_exact"));
    let first = s.design().options[0].name.clone();
    s.duplicate_option(&first, None).unwrap();
    let before: Vec<Value> = (0..2).map(|i| results(&s, i)).collect();
    s.set_input(&first, "hull_steel_rate", json!(5000.0))
        .unwrap();
    let after: Vec<Value> = (0..2).map(|i| results(&s, i)).collect();
    assert_ne!(
        before[0]["cost"]["unit_total"], after[0]["cost"]["unit_total"],
        "the edited Option moves"
    );
    assert_eq!(
        before[1]["cost"], after[1]["cost"],
        "the other Option does not"
    );
    // Nothing that is hashed moved: every results key except the unhashed ones is the same.
    let hashed = |v: &Value| {
        let mut v = v.clone();
        for k in UNHASHED_KEYS {
            v.as_object_mut().unwrap().remove(k);
        }
        v
    };
    assert_eq!(hashed(&before[0]), hashed(&after[0]));
    assert_eq!(results_hash(&before[0]), results_hash(&after[0]));
}

#[test]
fn the_budget_and_its_masses_are_not_in_the_results_hash() {
    assert!(UNHASHED_KEYS.contains(&"cost") && UNHASHED_KEYS.contains(&"cost_masses"));
    for file in ["gas_igc_2g_exact", "gas_igc_2g_self_propelled"] {
        let r = results(&open(&raw_fixture(file)), 0);
        let mut other = r.clone();
        other["cost"] = json!({"anything": 1});
        other["cost_masses"] = json!(null);
        assert_eq!(results_hash(&r), results_hash(&other), "{file}");
    }
}

#[test]
fn every_saved_fixture_keeps_its_results_hash_with_the_rates_in_it() {
    for file in FIXTURES {
        let (_, opened) = DesignSession::open(&raw_fixture(file), Profile::built_in()).unwrap();
        for (option, status) in &opened.hash_status {
            // A pushed convoy is not computed yet and stores no hash.
            let want = if file == "pushed_convoy" {
                HashStatus::NotStored
            } else {
                HashStatus::Matches
            };
            assert_eq!(*status, want, "{file}/{option}");
        }
    }
}

#[test]
fn a_file_saved_before_the_rates_existed_opens_with_the_example_rates_and_the_same_hash() {
    for file in FIXTURES {
        let text = old_fixture(file);
        let checked = check(&text, &Profile::built_in());
        assert_eq!(checked.report.errors, Vec::<String>::new(), "{file}");
        let (session, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
        let (_, with_rates) = DesignSession::open(&raw_fixture(file), Profile::built_in()).unwrap();
        // The same verdict on the stored hash as the file that holds the rates.
        assert_eq!(opened.hash_status, with_rates.hash_status, "{file}");
        for option in &session.design().options {
            for (name, want) in example() {
                let stored = option.input(name).unwrap();
                assert_eq!(stored.value.as_f64(), Some(want), "{file}: {name}");
            }
        }
        assert!(
            opened
                .report
                .warnings
                .iter()
                .any(|w| w.contains("hull_steel_rate")),
            "{file}: a note says the rates were filled in"
        );
    }
}

#[test]
fn a_gas_budget_takes_its_masses_from_the_design_and_says_which() {
    let cost = results(&open(&raw_fixture("gas_igc_2g_exact")), 0)["cost"].clone();
    let list = |k: &str| -> Vec<String> {
        cost[k]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect()
    };
    let (from, default) = (list("from_design"), list("core_default"));
    assert!(from.contains(&"tank_shell_t".to_string()));
    assert!(
        default.contains(&"battery_kwh".to_string()),
        "the tug is not sized by the engine"
    );
    // Every mass is in exactly one list.
    assert_eq!(
        from.len() + default.len(),
        cost["masses"].as_object().unwrap().len()
    );
    // The tank shell mass is the design's, not the core's default.
    assert_ne!(cost["masses"]["tank_shell_t"].as_f64(), Some(215.6));
}

#[test]
fn a_dry_bulk_design_has_no_budget() {
    // The core models a gas barge: a Cargo that is not gas has no `cost` and no masses.
    for file in ["dry_bulk_box_hold", "dry_bulk_sloped_hopper"] {
        let r = results(&open(&raw_fixture(file)), 0);
        assert!(r["cost"].is_null(), "{file}");
        assert!(r["cost_masses"].is_null(), "{file}");
    }
}

#[test]
fn the_cost_rates_do_not_reach_the_engine() {
    // Two different rate sets give the same engine results, key by key, apart from the budget.
    let mut s = open(&raw_fixture("gas_igc_2g_exact"));
    let name = s.design().options[0].name.clone();
    let before = results(&s, 0);
    for rate in RATES {
        s.set_input(&name, rate, json!(1.0)).unwrap();
    }
    let after = results(&s, 0);
    for key in before
        .as_object()
        .unwrap()
        .keys()
        .filter(|k| !UNHASHED_KEYS.contains(&k.as_str()))
    {
        assert_eq!(before[key], after[key], "{key}");
    }
    assert_ne!(before["cost"], after["cost"]);
}

#[test]
fn the_budget_shows_the_steel_difference_between_lightship_and_cost() {
    // Issue #401: the lightship uses the top-down hull steel; the budget prices the bottom-up one,
    // the hull steel, wing plate and weather shield masses it lists. The difference is shown.
    let cost = results(&open(&raw_fixture("gas_igc_2g_exact")), 0)["cost"].clone();
    let steel = &cost["steel"];
    let m = |k: &str| cost["masses"][k].as_f64().unwrap();
    let bottom_up = steel["bottom_up_t"].as_f64().unwrap();
    let top_down = steel["top_down_t"].as_f64().unwrap();
    let priced = m("hull_steel_t") + m("wing_plate_t") + m("weather_shield_t");
    assert!(
        (bottom_up - priced).abs() <= 1e-9 * priced,
        "{bottom_up} vs {priced}"
    );
    assert!(top_down > 0.0);
    assert_eq!(steel["delta_t"].as_f64().unwrap(), top_down - bottom_up);
}

fn items(cost: &Value, key: &str) -> Vec<String> {
    cost[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["item"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn a_self_propelled_gas_option_has_no_tug_line() {
    // Issue #403: a self-propelled vessel has no tug. Its own battery, propulsion and wheelhouse
    // are barge items; the tug's hull steel and outfitting lump are not costed.
    let cost = results(&open(&raw_fixture("gas_igc_2g_self_propelled")), 0)["cost"].clone();
    assert!(items(&cost, "tug").is_empty());
    assert_eq!(cost["tug_subtotal"], json!(0));
    let barge = items(&cost, "barge");
    for own in ["battery", "propulsion", "wheelhouse_accommodation"] {
        assert!(barge.contains(&own.to_string()), "{own}: {barge:?}");
    }
    assert!(cost["masses"].get("tug_hull_steel_t").is_none());
    let listed: Vec<&str> = cost["core_default"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(!listed.contains(&"tug_hull_steel_t"), "{listed:?}");
}

#[test]
fn a_pushed_gas_option_keeps_the_tug_line() {
    let cost = results(&open(&raw_fixture("gas_igc_2g_exact")), 0)["cost"].clone();
    assert_eq!(
        items(&cost, "tug"),
        [
            "hull_steel",
            "battery",
            "propulsion",
            "wheelhouse_accommodation",
            "outfitting"
        ]
    );
    assert!(cost["tug_subtotal"].as_f64().unwrap() > 0.0);
    assert!(!items(&cost, "barge").contains(&"battery".to_string()));
}

#[test]
fn typed_rates_change_the_basis_line() {
    // Issue #404: "Example rates" only while every rate is the example file's.
    let mut s = open(&raw_fixture("gas_igc_2g_exact"));
    let cost = results(&s, 0)["cost"].clone();
    assert_eq!(cost["rates"], json!("example"));
    assert_eq!(
        cost["basis"],
        json!(format!("{COST_RATES_BASIS} Current estimates."))
    );
    let first = s.design().options[0].name.clone();
    s.set_input(&first, "piping_rate", json!(4500.0)).unwrap();
    let cost = results(&s, 0)["cost"].clone();
    assert_eq!(
        cost["rates"],
        json!("user"),
        "a typed rate, even at the example value"
    );
    assert_eq!(
        cost["basis"],
        json!(format!("{COST_RATES_BASIS_USER} Current estimates."))
    );
}

// --- Currency and price base date (issue #429) ---------------------------------------------

const CURRENT: &str = "Current estimates.";

fn gas_session() -> (DesignSession, String) {
    let s = open(&raw_fixture("gas_igc_2g_exact"));
    let first = s.design().options[0].name.clone();
    (s, first)
}

#[test]
fn the_default_currency_is_usd_and_the_default_price_base_date_is_none() {
    let profile = Profile::built_in();
    assert_eq!(profile.default("currency"), Some(&json!("USD")));
    assert_eq!(profile.default("price_base_date"), Some(&Value::Null));
    for (name, want) in [("currency", json!("USD")), ("price_base_date", Value::Null)] {
        assert!(INPUTS.iter().any(|s| s.name == name), "{name}");
        assert_eq!(
            ovcd_format::vdesign::legacy_value(name, None),
            Some(want),
            "{name}"
        );
    }
    // The budget of an old Design says "current estimates" and USD, as the maintainer decided.
    let (s, _) = gas_session();
    let cost = results(&s, 0)["cost"].clone();
    assert_eq!(cost["currency"], json!("USD"));
    assert_eq!(cost["price_base_date"], Value::Null);
    assert_eq!(
        cost["basis"],
        json!(format!("{COST_RATES_BASIS} {CURRENT}"))
    );
}

#[test]
fn a_typed_currency_and_price_base_date_show_in_the_budget_and_the_basis_line() {
    let (mut s, first) = gas_session();
    s.set_input(&first, "currency", json!("EUR")).unwrap();
    s.set_input(&first, "price_base_date", json!("2026-10"))
        .unwrap();
    let cost = results(&s, 0)["cost"].clone();
    assert_eq!(cost["currency"], json!("EUR"));
    assert_eq!(cost["price_base_date"], json!("2026-10"));
    // The currency is typed, so the rates are the user's: the example rates are in USD.
    assert_eq!(cost["rates"], json!("user"));
    assert_eq!(
        cost["basis"],
        json!(format!("{COST_RATES_BASIS_USER} Prices as of 2026-10."))
    );
}

#[test]
fn the_currency_and_the_date_change_no_figure_and_nothing_hashed() {
    let (mut s, first) = gas_session();
    let before = results(&s, 0);
    s.set_input(&first, "currency", json!("GBP")).unwrap();
    s.set_input(&first, "price_base_date", json!("2026-10-21"))
        .unwrap();
    let after = results(&s, 0);
    assert_eq!(results_hash(&before), results_hash(&after));
    for key in ["barge", "tug", "unit_total", "barge_subtotal"] {
        assert_eq!(before["cost"][key], after["cost"][key], "{key}");
    }
}

#[test]
fn a_bad_currency_or_date_is_refused() {
    let (mut s, first) = gas_session();
    for (name, bad) in [
        ("currency", json!("usd")),
        ("currency", json!("US")),
        ("currency", json!("DOLLAR")),
        ("currency", json!(3)),
        ("currency", Value::Null),
        ("price_base_date", json!("next week")),
        ("price_base_date", json!("2026-13")),
        ("price_base_date", json!("2026-02-30")),
        ("price_base_date", json!("2026-1-5")),
        ("price_base_date", json!(2026)),
    ] {
        s.set_input(&first, name, bad.clone()).unwrap();
        let errors = check(&s.design().to_text(), &Profile::built_in())
            .report
            .errors;
        assert!(!errors.is_empty(), "{name} = {bad}");
    }
    for good in [
        json!("2026"),
        json!("2026-02"),
        json!("2028-02-29"),
        Value::Null,
    ] {
        s.set_input(&first, "price_base_date", good.clone())
            .unwrap();
        s.set_input(&first, "currency", json!("USD")).unwrap();
        let errors = check(&s.design().to_text(), &Profile::built_in())
            .report
            .errors;
        assert!(errors.is_empty(), "{good}: {errors:?}");
    }
}

#[test]
fn a_file_saved_before_the_two_inputs_opens_with_usd_and_no_date() {
    for file in FIXTURES {
        let mut v: Value = serde_json::from_str(&raw_fixture(file)).unwrap();
        for option in v["options"].as_array_mut().unwrap() {
            let inputs = option["inputs"].as_object_mut().unwrap();
            inputs.remove("currency");
            inputs.remove("price_base_date");
        }
        let text = v.to_string();
        assert_eq!(
            check(&text, &Profile::built_in()).report.errors,
            Vec::<String>::new(),
            "{file}"
        );
        let (session, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
        let (_, whole) = DesignSession::open(&raw_fixture(file), Profile::built_in()).unwrap();
        assert_eq!(opened.hash_status, whole.hash_status, "{file}");
        for option in &session.design().options {
            let c = option.input("currency").unwrap();
            assert_eq!(
                (&c.value, c.source),
                (&json!("USD"), ovcd_format::Source::Default)
            );
            assert_eq!(option.input("price_base_date").unwrap().value, Value::Null);
        }
    }
}

// --- Tank internals (issue #402) -------------------------------------------------------------

#[test]
fn with_a_share_of_0_the_cost_is_todays_cost_and_a_share_moves_it_by_the_rate_difference() {
    let (mut s, first) = {
        let s = open(&raw_fixture("gas_igc_2g_exact"));
        let first = s.design().options[0].name.clone();
        (s, first)
    };
    s.set_input(&first, "tank_internals_fraction", json!(0.0))
        .unwrap();
    let zero = results(&s, 0)["cost"].clone();
    // Today's pricing: the whole surplus at the supports rate.
    let surplus = zero["masses"]["tank_supports_t"].as_f64().unwrap();
    let r = example_rates().unwrap();
    assert_eq!(zero["tank_internals"]["fraction"], json!(0.0));
    s.set_input(&first, "tank_internals_fraction", json!(0.5))
        .unwrap();
    let half = results(&s, 0)["cost"].clone();
    // Half of the surplus moves from the supports rate to the tank shell rate.
    let moved = 0.5 * surplus;
    assert!((half["masses"]["tank_supports_t"].as_f64().unwrap() - (surplus - moved)).abs() < 1e-9);
    let delta = half["barge_subtotal"].as_f64().unwrap() - zero["barge_subtotal"].as_f64().unwrap();
    let want = moved * (r.tank_shell_rate - r.tank_supports_rate);
    assert!((delta - want).abs() <= 3.0, "{delta} vs {want}");
    assert!(delta > 0.0);
}

#[test]
fn the_results_say_the_share_is_an_estimate_with_no_source() {
    let cost = results(&open(&raw_fixture("gas_igc_2g_exact")), 0)["cost"].clone();
    let t = &cost["tank_internals"];
    assert_eq!(t["status"], json!("estimate"));
    assert!(t["source"].is_null());
    assert!(t["basis"].as_str().unwrap().contains("not a sourced value"));
    assert_eq!(t["fraction"], json!(0.5));
}

#[test]
fn the_share_changes_no_hash_and_a_file_saved_before_keeps_its_cost() {
    let (mut s, first) = {
        let s = open(&raw_fixture("gas_igc_2g_exact"));
        let first = s.design().options[0].name.clone();
        (s, first)
    };
    let before = results(&s, 0);
    s.set_input(&first, "tank_internals_fraction", json!(0.9))
        .unwrap();
    assert_eq!(results_hash(&before), results_hash(&results(&s, 0)));
    // A file with no such input reads it as 0: the old pricing.
    let mut v: Value = serde_json::from_str(&raw_fixture("gas_igc_2g_exact")).unwrap();
    for o in v["options"].as_array_mut().unwrap() {
        o["inputs"]
            .as_object_mut()
            .unwrap()
            .remove("tank_internals_fraction");
    }
    let old = open(&v.to_string());
    let c = results(&old, 0)["cost"].clone();
    assert_eq!(c["tank_internals"]["fraction"], json!(0.0));
    assert_eq!(
        old.design().options[0]
            .input("tank_internals_fraction")
            .unwrap()
            .source,
        ovcd_format::Source::Default
    );
}
