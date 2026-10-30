//! The liquid cargo tank inputs and the crude oil run (issue #331, spec 0006 sections 3.2 and 4).
//!
//! Label: property. Every test states a rule: an input applies to `integral_tanks` only and has a
//! default with a status and a basis, a run keeps the tank volume and the cargo mass the layout gives,
//! and a gas or dry bulk result has no `liquid` block. No number here is a printed result.

use ovd_core::liquid_bulk::lightship::TANK_LIGHTSHIP_K_T_PER_M3;
use ovd_format::vdesign::{CARGO_CONTAINMENTS, INPUTS, KEYED_DEFAULTS};
use ovd_format::{DesignSession, HashStatus, Profile, RunOutcome, check, run_design};
use serde_json::{Value, json};

const NEW: [&str; 10] = [
    "double_side_m",
    "double_bottom_m",
    "liquid_tanks_in_line",
    "centreline_bulkhead",
    "cofferdam_m",
    "pump_arrangement",
    "pump_room_length_m",
    "slop_share",
    "heating_kw_per_m3",
    "liquid_fill_ratio",
];

fn raw_fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The fixture with some inputs typed over.
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

fn errors(text: &str) -> Vec<String> {
    check(text, &Profile::built_in()).report.errors
}

fn spaces<'a>(r: &'a Value, kind: &str) -> Vec<&'a Value> {
    r["liquid"]["spaces"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["kind"] == kind)
        .collect()
}

#[test]
fn the_inputs_apply_to_integral_tanks_only_in_the_liquid_tanks_group() {
    for name in NEW {
        let spec = INPUTS
            .iter()
            .find(|s| s.name == name)
            .unwrap_or_else(|| panic!("{name}"));
        assert_eq!(spec.group.label(), "Liquid cargo tanks", "{name}");
        assert_eq!(spec.applies_to.containments(), ["integral_tanks"], "{name}");
    }
    // Integral tanks carry the liquid bulk class and no other containment does.
    let carries: Vec<&str> = CARGO_CONTAINMENTS
        .iter()
        .filter(|(_, c)| c.key() == "liquid_bulk")
        .map(|(n, _)| *n)
        .collect();
    assert_eq!(carries, ["integral_tanks"]);
}

#[test]
fn every_default_has_a_status_and_a_basis_and_a_sourced_one_a_source() {
    let profile = Profile::built_in();
    let text: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    for name in NEW {
        let e = &text["inputs"][name];
        assert!(
            e["basis"].as_str().is_some_and(|b| b.len() > 30),
            "{name}: basis"
        );
        let status = e["status"]
            .as_str()
            .unwrap_or_else(|| panic!("{name}: status"));
        assert!(["sourced", "estimate"].contains(&status), "{name}");
        if status == "sourced" {
            assert!(
                e["source"].as_str().is_some_and(|s| s.contains("ADN")),
                "{name}"
            );
        }
        assert!(
            profile.default(name).is_some() || e["default_by_service_area"].is_object(),
            "{name}"
        );
    }
}

#[test]
fn the_profile_defaults_are_the_spec_values() {
    let profile = Profile::built_in();
    let num = |n: &str| profile.default(n).and_then(Value::as_f64).unwrap();
    assert_eq!(num("cofferdam_m"), 0.6);
    assert_eq!(num("double_side_m"), 1.0);
    assert_eq!(num("double_bottom_m"), 0.7);
    assert_eq!(num("liquid_fill_ratio"), 0.98);
    assert_eq!(num("heating_kw_per_m3"), 0.0);
    assert_eq!(
        profile.default("pump_arrangement"),
        Some(&json!("deepwell"))
    );
    // The lightship factor of integral tanks is the fitted one in the core.
    assert_eq!(
        profile
            .default_by(
                "lightship_factor_t_per_m3",
                "cargo_containment",
                "integral_tanks"
            )
            .and_then(Value::as_f64),
        Some(TANK_LIGHTSHIP_K_T_PER_M3)
    );
    // The slop share by Service Area: none on a river, 3 % at the coast (MARPOL Annex I Reg. 29).
    let slop = |area: &str| {
        profile
            .default_by("slop_share", "service_area", area)
            .and_then(Value::as_f64)
    };
    assert_eq!(slop("river-only"), Some(0.0));
    assert_eq!(slop("coastal-and-river"), Some(0.03));
}

#[test]
fn every_keyed_default_has_a_liquid_bulk_row() {
    let profile = Profile::built_in();
    for (name, by, keys) in KEYED_DEFAULTS {
        for key in *keys {
            assert!(
                profile.default_by(name, by, key).is_some(),
                "{name} by {by}: {key}"
            );
        }
    }
    assert_eq!(
        profile.default_by("cargo_containment", "cargo_phase", "liquid_bulk"),
        Some(&json!("integral_tanks"))
    );
}

#[test]
fn a_crude_oil_file_gets_its_containment_and_the_tank_inputs_from_the_profile() {
    let (_, opened) = DesignSession::open(
        &raw_fixture("crude_integral_tanks_pushed"),
        Profile::built_in(),
    )
    .unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    s.set_input("Option 1", "cargo", json!("crude_oil"))
        .unwrap();
    let option = &s.design().options[0];
    assert_eq!(
        option.input("cargo_containment").unwrap().value,
        json!("integral_tanks")
    );
    for name in NEW.iter().filter(|n| **n != "pump_room_length_m") {
        assert!(option.input(name).is_some(), "{name}");
    }
    // The pump room length is used only with a pump room.
    assert!(option.input("pump_room_length_m").is_none());
}

#[test]
fn the_run_gives_a_liquid_block_with_spaces_that_add_up_to_the_cargo_zone() {
    let r = results(&raw_fixture("crude_integral_tanks_pushed"));
    assert_eq!(r["phase"], "liquid_bulk");
    let l = &r["liquid"];
    let tanks = spaces(&r, "cargo_tank");
    assert_eq!(tanks.len(), 8, "4 positions, P and S");
    let volume = |kind: &str| -> f64 {
        spaces(&r, kind)
            .iter()
            .map(|s| s["volume_m3"].as_f64().unwrap())
            .sum()
    };
    let cargo_gross = volume("cargo_tank");
    assert_eq!(l["cargo_gross_m3"].as_f64().unwrap(), cargo_gross);
    // The cargo is the gross volume at the fill ratio, and the mass is that volume x the density.
    let fill = 0.98;
    assert!((l["cargo_volume_m3"].as_f64().unwrap() - cargo_gross * fill).abs() < 1e-9);
    assert!((r["results"]["cargo_volume_m3"].as_f64().unwrap() - cargo_gross * fill).abs() < 1e-9);
    assert!((l["cargo_volume_bound_t"].as_f64().unwrap() - cargo_gross * fill * 0.87).abs() < 1e-9);
    // Cargo, ballast and cofferdams fill the cargo zone: 11.4 m beam, 3.5 m depth.
    let zone = r["geometry"]["cargo_zone_length_m"].as_f64().unwrap();
    let total: f64 = l["spaces"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["volume_m3"].as_f64().unwrap())
        .sum();
    assert!(
        (total - 11.4 * 3.5 * zone).abs() < 1e-6,
        "{total} vs {}",
        11.4 * 3.5 * zone
    );
    assert_eq!(r["results"]["governs"], "tank volume");
    // The free surface of a slack tank is l b^3 / 12: take the first tank.
    let first = &l["free_surface"][0];
    let (len, b) = (
        first["length_m"].as_f64().unwrap(),
        first["breadth_m"].as_f64().unwrap(),
    );
    assert!((first["inertia_m4"].as_f64().unwrap() - len * b * b * b / 12.0).abs() < 1e-9);
}

#[test]
fn the_default_grade_is_adn_type_c_because_its_flags_are_unknown() {
    let r = results(&raw_fixture("crude_integral_tanks_pushed"));
    assert_eq!(r["liquid"]["adn_vessel_type"]["type"], "C");
    assert!(
        r["liquid"]["adn_vessel_type"]["basis"]
            .as_str()
            .unwrap()
            .contains("unknown")
    );
}

#[test]
fn the_lightship_is_the_hull_plus_the_outfit_items() {
    let r = results(&raw_fixture("crude_integral_tanks_pushed"));
    let items = r["liquid"]["lightship_items"].as_array().unwrap();
    let names: Vec<&str> = items.iter().map(|i| i["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["hull", "pumps", "piping", "vents and gauging"]);
    let sum: f64 = items.iter().map(|i| i["mass_t"].as_f64().unwrap()).sum();
    // A pushed barge: the stern block is on the tug, so the lightship is the sum of the items.
    assert!((r["results"]["lightship_t"].as_f64().unwrap() - sum).abs() < 1e-9);
    // The fixture stores the lightship factor it was saved with (0.1141); the Defaults Profile has moved on.
    let hull = 0.1141 * 76.5 * 11.4 * 3.5;
    assert!((items[0]["mass_t"].as_f64().unwrap() - hull).abs() < 1e-9);
}

#[test]
fn a_service_area_at_the_coast_gives_slop_tanks_and_a_pump_room_is_a_space() {
    let r = results(&typed(
        "crude_integral_tanks_pushed",
        &[
            ("service_area", json!("coastal-and-river")),
            ("slop_share", json!(0.03)),
            ("pump_arrangement", json!("pump_room")),
            ("pump_room_length_m", json!(4.0)),
        ],
    ));
    assert_eq!(spaces(&r, "slop").len(), 2);
    assert_eq!(spaces(&r, "pump_room").len(), 1);
    assert!((r["liquid"]["slop_share"].as_f64().unwrap() - 0.03).abs() < 1e-9);
    let plain = results(&raw_fixture("crude_integral_tanks_pushed"));
    assert!(spaces(&plain, "slop").is_empty() && spaces(&plain, "pump_room").is_empty());
}

#[test]
fn heating_adds_a_lightship_item() {
    let r = results(&typed(
        "crude_integral_tanks_pushed",
        &[("heating_kw_per_m3", json!(0.3))],
    ));
    let names: Vec<&str> = r["liquid"]["lightship_items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"heating"));
}

#[test]
fn a_self_propelled_crude_hull_has_its_stern_block_and_its_stability() {
    let r = results(&raw_fixture("crude_integral_tanks_self_propelled"));
    assert!(r["drawing"]["aft_arrangement"].is_object());
    // The conditions are in the results (`liquid_conditions_results.rs` tests them), and so is the
    // capital budget of the cargo tank items (`liquid_cost.rs` tests it, issue #334).
    assert!(r["stability"]["conditions"].is_array());
    assert!(r["cost_masses"]["liquid"].is_object());
}

#[test]
fn a_gas_or_dry_bulk_result_has_no_liquid_block() {
    for file in ["dry_bulk_box_hold", "gas_igc_2g_self_propelled"] {
        let r = results(&raw_fixture(file));
        assert!(r.get("liquid").is_none(), "{file}");
    }
}

#[test]
fn the_hard_ranges_refuse_what_cannot_be_laid_out() {
    let base = "crude_integral_tanks_pushed";
    for (key, value) in [
        ("double_side_m", json!(-0.1)),
        ("liquid_tanks_in_line", json!(0)),
        ("slop_share", json!(1.0)),
        ("liquid_fill_ratio", json!(0.0)),
        ("liquid_fill_ratio", json!(1.2)),
        ("cofferdam_m", json!(-1.0)),
        ("pump_arrangement", json!("sump")),
    ] {
        let e = errors(&typed(base, &[(key, value.clone())]));
        assert!(!e.is_empty(), "{key} = {value} is accepted");
        assert!(e.iter().any(|m| m.contains(key)), "{key}: {e:?}");
    }
}

#[test]
fn a_layout_that_does_not_fit_is_infeasible_with_a_reason_not_a_crash() {
    // 50 tank positions of at least 1 m in a 31 m cargo zone.
    let checked = check(
        &typed(
            "crude_integral_tanks_pushed",
            &[
                ("liquid_tanks_in_line", json!(50)),
                ("length_overall_m", json!(40.0)),
            ],
        ),
        &Profile::built_in(),
    );
    let design = checked.design.unwrap();
    let RunOutcome::Results { results, .. } = run_design(&design).remove(0).outcome else {
        panic!("a failed run")
    };
    let feasible = results["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "feasible geometry")
        .unwrap();
    assert_eq!(feasible["ok"], false);
    assert!(feasible["message"].as_str().unwrap().contains("too short"));
}

#[test]
fn the_fixtures_hold_the_crude_inputs_as_defaults() {
    for file in [
        "crude_integral_tanks_pushed",
        "crude_integral_tanks_self_propelled",
    ] {
        let raw: Value = serde_json::from_str(&raw_fixture(file)).unwrap();
        let inputs = raw["options"][0]["inputs"].as_object().unwrap();
        for name in NEW.iter().filter(|n| **n != "pump_room_length_m") {
            assert_eq!(inputs[*name]["source"], "default", "{file}: {name}");
        }
        assert!(raw["options"][0]["results_hash"].is_string());
    }
}
