//! Seam: the `powering` block of a Design's results (issue #353): a specified Propulsor sizes its
//! Installed Power (or takes a typed one), places the engines, and gives the service speed per
//! environment and the bollard pull; a battery follows the energy per trip, and the cost takes the
//! same battery and power; an `unspecified` Propulsor and a file saved before get no block.
//!
//! Label: property.

use ovcd_format::{DesignSession, HashStatus, Profile, RunOutcome};
use serde_json::{Value, json};

fn results(s: &DesignSession) -> Value {
    match s.evaluate().options[0].outcome.as_ref().expect("it ran") {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

fn new_option(cargo: &str, depth: f64) -> DesignSession {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    for (n, v) in [
        ("cargo", json!(cargo)),
        ("length_overall_m", json!(110.0)),
        ("beam_m", json!(11.4)),
        ("moulded_depth_m", json!(depth)),
    ] {
        s.set_input("Option 1", n, v).unwrap();
    }
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    s
}

fn zones(r: &Value) -> Vec<String> {
    r["drawing"]["aft_arrangement"]["zones"]
        .as_array()
        .unwrap()
        .iter()
        .map(|z| z[0].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn a_new_option_sizes_its_installed_power_and_places_its_engines() {
    let mut s = new_option("bauxite", 4.0);
    let r = results(&s);
    let pw = &r["powering"];
    assert_eq!(pw["installed_power_sized"], true, "{pw}");
    let sized = pw["installed_power_kw"].as_f64().unwrap();
    assert!(sized > 0.0);
    assert!(pw["bollard_pull_t"].as_f64().unwrap() > 0.0);
    // The engines are in the stern block now (#352 waited for the power).
    assert!(
        zones(&r).contains(&"ENGINE ROOM".to_string()),
        "{:?}",
        zones(&r)
    );
    let deep = &pw["environments"][0];
    assert_eq!(deep["name"], "Deep water");
    assert!(deep["service_speed_kn"].as_f64().unwrap() > 0.0);
    assert!(
        deep["fuel_kg_per_h"].as_f64().unwrap() > 0.0,
        "diesel-mechanical burns diesel"
    );
    // A typed Installed Power is used as typed.
    s.set_input("Option 1", "installed_power_kw", json!(1500.0))
        .unwrap();
    let pw = &results(&s)["powering"];
    assert_eq!(pw["installed_power_sized"], false);
    assert_eq!(pw["installed_power_kw"], 1500.0);
}

#[test]
fn a_battery_follows_the_energy_per_trip_and_the_cost_takes_it() {
    let mut s = new_option("LNG", 6.0);
    s.set_input("Option 1", "power_source", json!("battery_electric"))
        .unwrap();
    let before = results(&s);
    assert!(before["powering"]["battery_energy_kwh"].is_null());
    s.set_input("Option 1", "trip_length_km", json!(150.0))
        .unwrap();
    let r = results(&s);
    let pw = &r["powering"];
    let per_trip = pw["environments"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["energy_per_trip_kwh"].as_f64().unwrap())
        .fold(0.0, f64::max);
    let battery = pw["battery_energy_kwh"].as_f64().unwrap();
    // Sized at the design draught, as the power is: at least the laden trip x 1.25 / 0.8.
    assert!(battery >= per_trip * 1.25 / 0.8 - 1e-6, "{pw}");
    // As many 5,000 kWh units as the energy needs.
    let units = r["drawing"]["aft_arrangement"]["container_labels"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|l| l.as_str().unwrap().starts_with("BATTERY"))
        .count();
    assert_eq!(units as f64, (battery / 5000.0).ceil());
    // The cost takes the same battery and the Installed Power.
    let m = &r["cost_masses"]["masses"];
    assert!(
        (m["battery_kwh"].as_f64().unwrap() - battery).abs() < 1e-6,
        "{m}"
    );
    assert_eq!(m["propulsion_kw"], pw["installed_power_kw"]);
}

#[test]
fn no_block_without_a_specified_propulsor_or_in_a_file_saved_before() {
    let mut s = new_option("bauxite", 4.0);
    s.set_input("Option 1", "propulsor_type", json!("unspecified"))
        .unwrap();
    assert!(results(&s).get("powering").is_none());
    for name in [
        "legacy/v2_gas_lpg_before_313.vdesign",
        "fixtures/gas_igc_2g_self_propelled.vdesign",
    ] {
        let path = format!("{}/tests/{name}", env!("CARGO_MANIFEST_DIR"));
        let (s, opened) =
            DesignSession::open(&std::fs::read_to_string(path).unwrap(), Profile::built_in())
                .unwrap();
        assert_eq!(opened.hash_status[0].1, HashStatus::Matches, "{name}");
        assert!(results(&s).get("powering").is_none(), "{name}");
    }
}
