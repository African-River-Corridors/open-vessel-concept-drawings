//! Seam: the capital budget of a tank barge (issue #334, spec 0006 section 4). The cargo tank items are
//! lump sums priced on quantities read from the tank layout, with example rates that are inputs. The
//! budget is an output, kept out of the results hash. A gas budget keeps its items, and a dry bulk
//! Option still has none.
//!
//! Label: property. Every test states a rule (a line is its quantity times its rate, an edit moves one
//! line, the hash does not move). No number here is a printed result.

use ovd_format::vdesign::INPUTS;
use ovd_format::{DesignSession, Profile, RunOutcome, UNHASHED_KEYS, input_schema, results_hash};
use serde_json::{Value, json};

const RATES: [&str; 5] = [
    "tank_coating_rate",
    "cargo_pump_lump",
    "cargo_piping_lump",
    "cargo_heating_rate_per_kw",
    "vents_gauging_lump",
];

const CRUDE: [&str; 2] = [
    "crude_integral_tanks_pushed",
    "crude_integral_tanks_self_propelled",
];

fn raw_fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn open(name: &str) -> DesignSession {
    DesignSession::open(&raw_fixture(name), Profile::built_in())
        .unwrap()
        .0
}

fn results(s: &DesignSession) -> Value {
    match s.evaluate().options[0].outcome.as_ref() {
        Some(RunOutcome::Results { results, .. }) => results.clone(),
        other => panic!("{other:?}"),
    }
}

fn line(r: &Value, item: &str) -> f64 {
    r["cost"]["barge"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["item"] == item)
        .unwrap_or_else(|| panic!("no `{item}` line"))["amount"]
        .as_f64()
        .unwrap()
}

fn input(s: &DesignSession, name: &str) -> f64 {
    s.design().options[0]
        .input(name)
        .unwrap()
        .value
        .as_f64()
        .unwrap()
}

#[test]
fn the_cargo_tank_rates_are_cost_rate_inputs_of_the_tank_containment_only() {
    let schema = input_schema(&Profile::built_in());
    for name in RATES {
        let spec = INPUTS.iter().find(|s| s.name == name).unwrap();
        assert_eq!(spec.applies_to.containments(), ["integral_tanks"], "{name}");
        let entry = schema["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["name"] == name)
            .unwrap();
        assert_eq!(entry["group"], "Cost rates", "{name}");
        assert!(entry["default"].as_f64().unwrap() > 0.0, "{name}");
    }
}

#[test]
fn a_tank_barge_budget_lists_the_cargo_tank_items_and_each_line_is_its_quantity_times_its_rate() {
    for file in CRUDE {
        let s = open(file);
        let r = results(&s);
        let masses = &r["cost_masses"]["liquid"];
        let items: Vec<&str> = r["cost"]["barge"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["item"].as_str().unwrap())
            .collect();
        for want in [
            "hull_steel",
            "tank_coating",
            "cargo_pumps",
            "cargo_piping",
            "cargo_heating",
            "vents_gauging",
            "paint_anodes_misc",
        ] {
            assert!(items.contains(&want), "{file}: {want} in {items:?}");
        }
        // No Type C item is priced on a tank barge.
        assert!(!items.contains(&"type_c_tanks"), "{file}");
        // Each line is rounded to a whole unit, so a sum of lines is within half a unit for each line.
        let near = |a: f64, b: f64| (a - b).abs() <= 6.0 + 1e-9 * b.abs();
        let tanks = masses["cargo_tank_count"].as_f64().unwrap();
        assert!(tanks >= 1.0);
        assert!(near(
            line(&r, "hull_steel"),
            masses["hull_steel_t"].as_f64().unwrap() * input(&s, "hull_steel_rate")
        ));
        assert!(near(
            line(&r, "tank_coating"),
            masses["coating_area_m2"].as_f64().unwrap() * input(&s, "tank_coating_rate")
        ));
        assert!(near(
            line(&r, "cargo_pumps"),
            tanks * input(&s, "cargo_pump_lump")
        ));
        assert!(near(
            line(&r, "cargo_piping"),
            input(&s, "cargo_piping_lump")
        ));
        assert!(near(
            line(&r, "cargo_heating"),
            masses["heating_kw"].as_f64().unwrap() * input(&s, "cargo_heating_rate_per_kw")
        ));
        assert!(near(
            line(&r, "vents_gauging"),
            tanks * input(&s, "vents_gauging_lump")
        ));
        // The total is the sum of the parts: the barge and tug items, the margin and the delivery.
        let money = |v: &Value| v.as_f64().unwrap();
        let sum = |key: &str| {
            r["cost"][key]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| money(&row["amount"]))
                .sum::<f64>()
        };
        assert!(
            near(sum("barge"), money(&r["cost"]["barge_subtotal"])),
            "{file}"
        );
        let total = money(&r["cost"]["barge_subtotal"])
            + money(&r["cost"]["tug_subtotal"])
            + money(&r["cost"]["yard_margin"])
            + money(&r["cost"]["delivery"]);
        assert!(near(total, money(&r["cost"]["unit_total"])), "{file}");
    }
}

#[test]
fn the_coating_area_is_the_inside_surface_of_the_cargo_and_slop_tanks() {
    // A box tank of length x, breadth y and height z has the surface 2(xy + xz + yz). The breadth and the
    // height come from the beam, the depth and the double hull of the inputs, and the length from the
    // Space, so the area is worked out here and not by the core's code.
    let s = open(CRUDE[0]);
    let r = results(&s);
    let beam = r["dims"]["beam_m"].as_f64().unwrap();
    let depth = r["dims"]["depth_m"].as_f64().unwrap();
    let inner_half = beam / 2.0 - input(&s, "double_side_m");
    let z = depth - input(&s, "double_bottom_m");
    let mut area = 0.0;
    let mut tanks = 0;
    for sp in r["liquid"]["spaces"].as_array().unwrap() {
        if sp["kind"] != "cargo_tank" && sp["kind"] != "slop" {
            continue;
        }
        tanks += 1;
        let y = if sp["side"] == "centre" {
            2.0 * inner_half
        } else {
            inner_half
        };
        let x = sp["x_fwd_m"].as_f64().unwrap() - sp["x_aft_m"].as_f64().unwrap();
        area += 2.0 * (x * y + x * z + y * z);
    }
    assert!(tanks >= 2);
    let got = r["cost_masses"]["liquid"]["coating_area_m2"]
        .as_f64()
        .unwrap();
    assert!((got - area).abs() < 1e-6 * area, "{got} against {area}");
}

#[test]
fn editing_one_cargo_tank_rate_moves_its_line_and_nothing_else() {
    for (name, item) in [
        ("tank_coating_rate", "tank_coating"),
        ("cargo_pump_lump", "cargo_pumps"),
        ("cargo_piping_lump", "cargo_piping"),
        ("vents_gauging_lump", "vents_gauging"),
    ] {
        let mut s = open(CRUDE[0]);
        let before = results(&s);
        let was = input(&s, name);
        let first = s.design().options[0].name.clone();
        s.set_input(&first, name, json!(was * 2.0)).unwrap();
        let after = results(&s);
        assert!(
            (line(&after, item) - 2.0 * line(&before, item)).abs() <= 1.0,
            "{name} doubles {item}"
        );
        for other in ["hull_steel", "paint_anodes_misc"] {
            assert_eq!(
                line(&before, other),
                line(&after, other),
                "{name} leaves {other}"
            );
        }
        // Nothing hashed moved.
        assert_eq!(results_hash(&before), results_hash(&after), "{name}");
    }
}

#[test]
fn heating_is_priced_when_the_design_has_heating() {
    let mut s = open(CRUDE[0]);
    let first = s.design().options[0].name.clone();
    assert_eq!(
        line(&results(&s), "cargo_heating"),
        0.0,
        "no heating, no cost"
    );
    s.set_input(&first, "heating_kw_per_m3", json!(0.3))
        .unwrap();
    let r = results(&s);
    let kw = r["cost_masses"]["liquid"]["heating_kw"].as_f64().unwrap();
    assert!(kw > 0.0);
    let want = kw * input(&s, "cargo_heating_rate_per_kw");
    assert!((line(&r, "cargo_heating") - want).abs() <= 1.0);
}

#[test]
fn a_self_propelled_tank_barge_prices_its_own_power_and_a_pushed_one_has_a_tug() {
    let pushed = results(&open(CRUDE[0]));
    assert!(
        !pushed["cost"]["tug"].as_array().unwrap().is_empty(),
        "a pushed barge has a tug"
    );
    let own = results(&open(CRUDE[1]));
    assert!(
        own["cost"]["tug"].as_array().unwrap().is_empty(),
        "a self-propelled hull has none"
    );
    let kw = own["powering"]["installed_power_kw"].as_f64().unwrap();
    let rate = input(&open(CRUDE[1]), "propulsion_rate_per_kw");
    assert!((line(&own, "propulsion") - kw * rate).abs() <= 1.0);
    assert!(
        own["cost"]["from_design"]
            .as_array()
            .unwrap()
            .contains(&json!("propulsion_kw"))
    );
}

#[test]
fn the_tank_barge_budget_and_its_quantities_are_not_in_the_results_hash() {
    assert!(UNHASHED_KEYS.contains(&"cost") && UNHASHED_KEYS.contains(&"cost_masses"));
    let r = results(&open(CRUDE[0]));
    let mut other = r.clone();
    other["cost"] = json!({"anything": 1});
    other["cost_masses"] = json!(null);
    assert_eq!(results_hash(&r), results_hash(&other));
}

#[test]
fn gas_keeps_its_type_c_items_and_dry_bulk_still_has_no_budget() {
    let gas = results(&open("gas_igc_2g_exact"));
    let items: Vec<&str> = gas["cost"]["barge"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["item"].as_str().unwrap())
        .collect();
    assert_eq!(
        items,
        [
            "hull_steel",
            "type_c_tanks",
            "tank_supports",
            "insulation",
            "cargo_system",
            "piping_machinery",
            "outfitting",
            "paint_anodes_misc"
        ]
    );
    assert!(results(&open("dry_bulk_box_hold"))["cost"].is_null());
}
