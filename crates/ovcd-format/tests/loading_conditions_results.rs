//! Seam: the IS Code 2008 loading conditions in a Design's results (issue #426). An Option with
//! `loading_conditions` = `is_code_2008` has results that carry the four standard conditions of IS Code
//! 2008 Part B 3.4.1.2 (and "Lightship, no ballast" for a barge), the ballast tanks and a ballast
//! plan per ballast condition. A file saved before keeps `legacy`, gets no new field, and keeps its
//! results hash.
//!
//! Label: property. tests/loading_conditions.rs in ovcd-core holds the hand-worked values.

use ovcd_format::{DesignSession, HashStatus, Profile, RunOutcome};
use serde_json::{Value, json};

fn results(s: &DesignSession) -> Value {
    match s.evaluate().options[0].outcome.as_ref().expect("it ran") {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

fn session(cargo: &str, formation: &str) -> DesignSession {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    for (n, v) in [
        ("cargo", json!(cargo)),
        ("length_overall_m", json!(110.0)),
        ("beam_m", json!(11.4)),
        ("moulded_depth_m", json!(6.0)),
    ] {
        s.set_input("Option 1", n, v).unwrap();
    }
    s.set_formation("Option 1", formation, None, None).unwrap();
    s.set_input("Option 1", "loading_conditions", json!("is_code_2008"))
        .unwrap();
    s
}

fn roles(r: &Value) -> Vec<String> {
    r["loading_conditions"]["conditions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["role"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn a_self_propelled_option_has_the_four_is_code_conditions() {
    for cargo in ["LNG", "bauxite"] {
        let r = results(&session(cargo, "self_propelled"));
        assert_eq!(
            roles(&r),
            [
                "full_load_departure",
                "full_load_arrival",
                "ballast_departure",
                "ballast_arrival"
            ],
            "{cargo}"
        );
        let lc = &r["loading_conditions"];
        let c = lc["conditions"].as_array().unwrap();
        let full = lc["consumables_t"].as_f64().unwrap();
        assert!(
            full > 0.0,
            "{cargo}: a self-propelled vessel carries stores"
        );
        // Part B 3.4.1.2: departure full, arrival 10 %.
        assert!((c[0]["consumables_t"].as_f64().unwrap() - full).abs() < 1e-9);
        assert!((c[1]["consumables_t"].as_f64().unwrap() - 0.1 * full).abs() < 1e-9);
        // Part B 3.4.2.2: the full load sails with the ballast tanks empty.
        assert_eq!(c[0]["ballast_t"], json!(0.0));
        // The stability view reads the same conditions.
        assert_eq!(r["stability"]["conditions"].as_array().unwrap().len(), 4);
        // The ballast tanks: count and capacity.
        let n = lc["ballast_tank_count"].as_u64().unwrap();
        assert_eq!(n as usize, lc["ballast_tanks"].as_array().unwrap().len());
        assert!(lc["ballast_capacity_t"].as_f64().unwrap() > 0.0);
        // The deadweight holds the cargo and the stores and fuel.
        let cargo_t = r["results"]["cargo_t"].as_f64().unwrap();
        let dwt = r["results"]["deadweight_t"].as_f64().unwrap();
        assert!((dwt - cargo_t - full).abs() < 1e-6, "{cargo}");
    }
}

#[test]
fn a_barge_also_has_lightship_no_ballast_and_carries_no_consumables() {
    let r = results(&session("bauxite", "pushed_barge"));
    assert_eq!(roles(&r).last().unwrap(), "lightship_no_ballast");
    assert_eq!(r["loading_conditions"]["consumables_t"], json!(0.0));
    // A pushed barge has an aft peak.
    let names: Vec<&str> = r["loading_conditions"]["ballast_tanks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"AP1C"), "{names:?}");
}

#[test]
fn each_ballast_condition_states_the_quantity_and_disposition_of_its_ballast() {
    let mut s = session("bauxite", "pushed_barge");
    s.set_input("Option 1", "ballast_plan", json!("user_set"))
        .unwrap();
    s.set_input(
        "Option 1",
        "ballast_departure_double_bottom_fill",
        json!(0.5),
    )
    .unwrap();
    let r = results(&s);
    let dep = &r["loading_conditions"]["conditions"][2];
    assert_eq!(dep["role"], "ballast_departure");
    let tanks = dep["ballast"].as_array().unwrap();
    assert_eq!(tanks.len(), 2, "both double-bottom tanks");
    let sum: f64 = tanks.iter().map(|t| t["mass_t"].as_f64().unwrap()).sum();
    assert!((sum - dep["ballast_t"].as_f64().unwrap()).abs() < 1e-9);
    assert!(tanks.iter().all(|t| t["fill"] == json!(0.5)));
    // A half-full tank has a free surface.
    assert!(dep["free_surface_m"].as_f64().unwrap() > 0.0);
    assert_eq!(dep["plan_found"], Value::Null);
}

#[test]
fn a_new_option_keeps_the_legacy_conditions_until_the_search_uses_the_new_ones() {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    s.set_input("Option 1", "cargo", json!("bauxite")).unwrap();
    let r = results(&s);
    assert!(r.get("loading_conditions").is_none());
}

#[test]
fn legacy_conditions_add_no_field() {
    let mut s = session("bauxite", "self_propelled");
    s.set_input("Option 1", "loading_conditions", json!("legacy"))
        .unwrap();
    let r = results(&s);
    assert!(r.get("loading_conditions").is_none());
    assert!(r.get("stability").is_none());
}

#[test]
fn a_file_saved_before_keeps_the_legacy_conditions_and_its_hash() {
    // (`v1_adn_g_self_propelled` is left out: its saved hash is of a retired ruleset, so it
    // differs by design; see tests/legacy_v1.rs.)
    for name in ["v2_gas_lpg_before_313", "v1_dry_bulk_box_hold"] {
        let path = format!("{}/tests/legacy/{name}.vdesign", env!("CARGO_MANIFEST_DIR"));
        let (s, opened) =
            DesignSession::open(&std::fs::read_to_string(path).unwrap(), Profile::built_in())
                .unwrap();
        if let Some((_, status)) = opened.hash_status.first() {
            assert!(!matches!(status, HashStatus::Differs { .. }), "{name}");
        }
        let r = results(&s);
        assert!(r.get("loading_conditions").is_none(), "{name}");
    }
}
