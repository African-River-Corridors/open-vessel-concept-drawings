//! The gas consumption and the boil-off check of a gas-fuelled Option (issue #464).
//!
//! Label: property

use ovcd_core::gas_fuel::gas_use_kg_per_h;
use ovcd_format::rule_set::Library;
use ovcd_format::{Profile, RunOutcome, fill_as_new, run_option_with};
use serde_json::{Value, json};

fn results(cargo: &str, extra: Value) -> Value {
    let mut inputs = json!({
        "cargo": { "value": cargo, "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
        "length_overall_m": { "value": 110.0, "source": "typed" },
        "beam_m": { "value": 11.4, "source": "typed" },
        "moulded_depth_m": { "value": 6.0, "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    let design = filled.design.unwrap();
    match run_option_with(&design.options[0], &Library::built_in(), &[]) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn check<'a>(r: &'a Value, name: &str) -> Option<&'a Value> {
    r["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
}

#[test]
fn a_gas_fuelled_lng_option_gets_its_gas_use_per_environment() {
    let r = results("LNG", json!({}));
    assert_eq!(r["drawing"]["aft_arrangement"]["propulsion"], "gas_fuelled");
    let aft = &r["drawing"]["aft_arrangement"];
    let (kwe, n) = (
        aft["genset_kwe"].as_f64().unwrap(),
        aft["n_gensets"].as_f64().unwrap(),
    );
    for e in r["powering"]["environments"].as_array().unwrap() {
        let needed = e["power_at_service_kw"].as_f64().unwrap();
        // The sets that run: as many as the need asks for.
        let running = (needed / kwe).ceil().clamp(1.0, n) * kwe;
        let want = gas_use_kg_per_h(needed, running);
        assert_eq!(e["fuel_gas_kg_per_h"].as_f64().unwrap(), want, "{e}");
        assert!(want > 0.0);
    }
    assert!(
        r["powering"]["fuel_gas"]["basis"]
            .as_str()
            .unwrap()
            .contains("Wartsila 20DF")
    );
}

#[test]
fn the_boil_off_check_compares_the_boil_off_with_the_gas_used_and_has_message_ids() {
    let r = results("LNG", json!({}));
    let b = &r["boil_off"];
    assert_eq!(b["evaluated"], true);
    let boil = b["boil_off_kg_per_h"].as_f64().unwrap();
    // The heat leak over the latent heat of methane at -162 C.
    let kw = b["heat_leak_kw"].as_f64().unwrap();
    assert!((boil - kw * 3600.0 / 511.77).abs() < 1e-9);
    assert_eq!(b["ambient_air_c"], 45.0);
    let used = b["gas_use_kg_per_h"].as_f64().unwrap();
    let c = check(&r, "boil-off within gas use").unwrap();
    assert_eq!(c["ok"], boil <= used);
    assert_eq!(c["severity"], "warning");
    let message = c["message"].as_str().unwrap();
    let id = r["text_ids"][message]["id"].as_str().unwrap();
    assert!(id.starts_with("check-boil-off-"), "{id}");
    assert_eq!(
        r["text_ids"]["boil-off within gas use"]["id"],
        "name-check-boil-off-within-gas-use"
    );
    // IGC 16.1: methane may fuel the gensets.
    assert_eq!(check(&r, "cargo as fuel").unwrap()["ok"], true);
}

#[test]
fn another_cargo_gas_is_flagged_by_igc_16_1_and_its_boil_off_is_not_worked_out() {
    let r = results("LPG_propane", json!({}));
    assert!(r["powering"].is_object());
    assert_eq!(r["drawing"]["aft_arrangement"]["propulsion"], "gas_fuelled");
    let c = check(&r, "cargo as fuel").unwrap();
    assert_eq!(c["ok"], false);
    assert_eq!(c["severity"], "warning");
    assert!(c["message"].as_str().unwrap().contains("16.9.1"));
    assert_eq!(r["boil_off"]["evaluated"], false);
}

#[test]
fn a_hull_that_is_not_gas_fuelled_gets_none_of_it() {
    let r = results("LNG", json!({ "power_source": "diesel_mechanical" }));
    assert!(r.get("boil_off").is_none());
    assert!(check(&r, "cargo as fuel").is_none());
    for e in r["powering"]["environments"]
        .as_array()
        .into_iter()
        .flatten()
    {
        assert!(e.get("fuel_gas_kg_per_h").is_none());
    }
}
