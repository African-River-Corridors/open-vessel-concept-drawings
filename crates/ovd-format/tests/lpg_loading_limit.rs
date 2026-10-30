//! Seam: the density and the loading limit of an LPG Cargo by carriage condition (issue #315).
//!
//! A new Design loads an LPG Cargo at its loading temperature, at the saturated liquid density
//! there, to the loading limit of IGC 15.5.1, LL = FL x rho_R / rho_L. A file saved before keeps the
//! refrigerated density and the fill ratio, so its results do not change.
//!
//! Label: property. Each test states a rule: the reference temperature of each carriage condition,
//! the loading limit formula on the engine's own tables (tests/lpg_saturation.rs in ovd-core holds
//! the sourced worked case), the cargo mass, the old files, the soft check.

use ovd_core::lpg_saturation::{
    ATMOSPHERE_BAR, liquid_density_kg_m3, loading_limit, saturation_temperature_c,
};
use ovd_format::vdesign::{LOADING_LIMIT_BASES, LPG_CARGOS};
use ovd_format::{DesignSession, HashStatus, Profile, RunOutcome, Source, check};
use serde_json::{Value, json};

fn new_design(cargo: &str, carriage: &str) -> DesignSession {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    for (name, value) in [
        ("cargo", json!(cargo)),
        ("length_overall_m", json!(110.0)),
        ("beam_m", json!(11.4)),
        ("moulded_depth_m", json!(6.0)),
        ("lpg_carriage_condition", json!(carriage)),
    ] {
        s.set_input("Option 1", name, value).unwrap();
    }
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    s
}

fn results(s: &DesignSession) -> Value {
    match s.evaluate().options[0].outcome.as_ref().expect("it ran") {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

fn value(s: &DesignSession, name: &str) -> (Value, Source) {
    let i = s.design().options[0].input(name).unwrap();
    (i.value.clone(), i.source)
}

#[test]
fn a_new_lpg_design_takes_the_igc_basis_and_the_table_loading_temperature() {
    for (carriage, t) in [
        ("fully_pressurised", 15.0),
        ("semi_pressurised", -10.0),
        ("fully_refrigerated", -42.1),
    ] {
        let s = new_design("LPG_propane", carriage);
        assert_eq!(
            value(&s, "lpg_loading_limit_basis"),
            (json!("igc_15_1_3"), Source::Default)
        );
        assert_eq!(
            value(&s, "lpg_loading_temperature_c"),
            (json!(t), Source::Default),
            "{carriage}"
        );
    }
    assert_eq!(
        LOADING_LIMIT_BASES,
        ["igc_15_1_3", "igc_15_5_2", "refrigerated_density"]
    );
}

#[test]
fn fully_pressurised_takes_the_marvs_saturation_temperature_and_loads_below_the_filling_limit() {
    let s = new_design("LPG_propane", "fully_pressurised");
    let r = results(&s);
    let c = &r["drawing"]["cargo_loading"];
    let marvs = r["drawing"]["tank"]["design_pressure_mpa"]
        .as_f64()
        .unwrap()
        * 10.0
        + ATMOSPHERE_BAR;
    let t_r = saturation_temperature_c("LPG_propane", marvs).unwrap();
    assert_eq!(c["reference_temperature_c"], json!(t_r));
    let (rho_r, rho_l) = (
        liquid_density_kg_m3("LPG_propane", t_r).unwrap(),
        liquid_density_kg_m3("LPG_propane", 15.0).unwrap(),
    );
    let fl = c["filling_limit"].as_f64().unwrap();
    assert_eq!(fl, 0.98);
    let ll = c["loading_limit"].as_f64().unwrap();
    assert_eq!(ll, loading_limit(fl, rho_r, rho_l));
    assert!(ll < fl);
    // The cargo is the loaded volume at the loaded density: the tank volume x FL x rho_R.
    let cargo_t = r["results"]["cargo_t"].as_f64().unwrap();
    let volume = r["results"]["cargo_volume_m3"].as_f64().unwrap();
    assert!(
        (cargo_t - volume * rho_l / 1000.0).abs() < 1e-6,
        "{cargo_t}"
    );
}

#[test]
fn with_temperature_control_the_loading_limit_is_the_filling_limit() {
    for carriage in ["semi_pressurised", "fully_refrigerated"] {
        for cargo in LPG_CARGOS {
            let r = results(&new_design(cargo, carriage));
            let c = &r["drawing"]["cargo_loading"];
            assert_eq!(c["loading_limit"], c["filling_limit"], "{cargo} {carriage}");
            assert_eq!(
                c["reference_temperature_c"], c["loading_temperature_c"],
                "{cargo} {carriage}"
            );
        }
    }
}

#[test]
fn the_15_5_2_basis_takes_45_c_and_loads_more_than_15_1_3_1() {
    let mut s = new_design("LPG_butane", "fully_pressurised");
    let ll = |s: &DesignSession| {
        results(s)["drawing"]["cargo_loading"]["loading_limit"]
            .as_f64()
            .unwrap()
    };
    let by_marvs = ll(&s);
    s.set_input("Option 1", "lpg_loading_limit_basis", json!("igc_15_5_2"))
        .unwrap();
    let r = results(&s);
    assert_eq!(
        r["drawing"]["cargo_loading"]["reference_temperature_c"],
        45.0
    );
    assert!(ll(&s) > by_marvs);
}

#[test]
fn the_old_basis_keeps_the_cargo_data_density_and_the_fill() {
    let mut s = new_design("LPG_propane", "fully_pressurised");
    s.set_input(
        "Option 1",
        "lpg_loading_limit_basis",
        json!("refrigerated_density"),
    )
    .unwrap();
    let r = results(&s);
    assert!(r["drawing"].get("cargo_loading").is_none());
    let cargo_t = r["results"]["cargo_t"].as_f64().unwrap();
    let volume = r["results"]["cargo_volume_m3"].as_f64().unwrap();
    assert!((cargo_t - volume * 0.581).abs() < 1e-6);
}

#[test]
fn an_lng_design_has_no_loading_inputs_and_does_not_move() {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    s.set_input("Option 1", "cargo", json!("LNG")).unwrap();
    s.set_input("Option 1", "length_overall_m", json!(110.0))
        .unwrap();
    s.set_input("Option 1", "beam_m", json!(11.4)).unwrap();
    let o = &s.design().options[0];
    assert!(o.input("lpg_loading_limit_basis").is_none());
    assert!(o.input("lpg_loading_temperature_c").is_none());
}

#[test]
fn a_file_saved_before_keeps_its_results() {
    let path = format!(
        "{}/tests/legacy/v2_gas_lpg_before_313.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).unwrap();
    let checked = check(&text, &Profile::built_in());
    let o = &checked.design.as_ref().unwrap().options[0];
    let basis = o.input("lpg_loading_limit_basis").unwrap();
    assert_eq!(
        (basis.value.clone(), basis.source),
        (json!("refrigerated_density"), Source::Default)
    );
    let (_, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
}

fn simulate(params: &Value) -> Value {
    let text = json!({"fn": "simulate", "args": {"params": params}}).to_string();
    ovd_format::eval("simulate", &ovd_format::parse_case(&text).unwrap()).unwrap()
}

#[test]
fn simulate_params_without_the_basis_keep_the_old_behaviour() {
    let base = json!({
        "vessel": "gas_barge", "cargo": "LPG_propane", "loa_m": 110.0, "beam_m": 11.4, "depth_m": 6.0, "cb": 0.85,
        "lpg_carriage_condition": "fully_pressurised",
    });
    let old = simulate(&base);
    assert!(old.get("ok").is_some(), "{old}");
    let mut with = base.clone();
    with["lpg_loading_limit_basis"] = json!("refrigerated_density");
    assert_eq!(simulate(&with), old);
    with["lpg_loading_limit_basis"] = json!("igc_15_1_3");
    assert_ne!(simulate(&with), old);
}

#[test]
fn loading_warmer_than_the_marvs_allows_warns_and_stays_at_the_filling_limit() {
    let mut s = new_design("LPG_propane", "fully_pressurised");
    s.set_input("Option 1", "lpg_loading_temperature_c", json!(60.0))
        .unwrap();
    let checked = check(&s.save().unwrap(), &Profile::built_in());
    assert!(
        checked
            .report
            .warnings
            .iter()
            .any(|w| w.contains("above the MARVS")),
        "{:?}",
        checked.report.warnings
    );
    let r = results(&s);
    let c = &r["drawing"]["cargo_loading"];
    assert_eq!(c["loading_limit"], c["filling_limit"]);
}
