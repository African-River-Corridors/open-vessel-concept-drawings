//! Seam: the design liquid pressure input of the gas tanks (issue #374). A new Design takes the
//! static liquid pressure; the IGC 4.28.2 sea-going accelerations are a choice; a file saved before
//! keeps the vapour pressure only, so its results do not change.
//!
//! Label: property. The values compared are the engine's own (tests/liquid_pressure.rs in ovcd-core
//! holds the sourced hand calculations).

use ovcd_core::tanks::liquid_pressure_mpa;
use ovcd_format::{DesignSession, HashStatus, Profile, RunOutcome, Source};
use serde_json::{Value, json};

fn new_design(cargo: &str) -> DesignSession {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    for (name, value) in [
        ("cargo", json!(cargo)),
        ("length_overall_m", json!(110.0)),
        ("beam_m", json!(11.4)),
        ("moulded_depth_m", json!(6.0)),
    ] {
        s.set_input("Option 1", name, value).unwrap();
    }
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    s
}

fn tank(s: &DesignSession) -> Value {
    match s.evaluate().options[0].outcome.as_ref().expect("it ran") {
        RunOutcome::Results { results, .. } => results["drawing"]["tank"].clone(),
        other => panic!("{other:?}"),
    }
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

#[test]
fn a_new_design_takes_the_static_liquid_pressure() {
    for cargo in ["LNG", "LPG_propane"] {
        let s = new_design(cargo);
        let i = s.design().options[0].input("tank_liquid_pressure").unwrap();
        assert_eq!(
            (i.value.clone(), i.source),
            (json!("static"), Source::Default)
        );
        let t = tank(&s);
        assert_eq!(t["liquid_pressure_accel"], 1.0, "{cargo}");
        let pgd = liquid_pressure_mpa(
            Some(1.0),
            f(&t["tank_steel_outer_diameter_m"]),
            f(&t["liquid_pressure_density_kg_m3"]),
        );
        assert_eq!(f(&t["liquid_pressure_mpa"]), pgd, "{cargo}");
        assert_eq!(
            f(&t["internal_pressure_mpa"]),
            f(&t["design_pressure_mpa"]) + pgd
        );
    }
}

#[test]
fn the_sea_going_accelerations_give_a_thicker_wall_and_none_a_thinner_one() {
    let mut s = new_design("LNG");
    let wall = |s: &DesignSession| f(&tank(s)["cylinder_wall_thickness_mm"]);
    let pressure = |s: &DesignSession| f(&tank(s)["internal_pressure_mpa"]);
    let (stat_wall, stat_p) = (wall(&s), pressure(&s));
    s.set_input("Option 1", "tank_liquid_pressure", json!("igc_4_28_2"))
        .unwrap();
    assert!(f(&tank(&s)["liquid_pressure_accel"]) > 1.0);
    assert!(pressure(&s) > stat_p);
    assert!(wall(&s) >= stat_wall);
    s.set_input("Option 1", "tank_liquid_pressure", json!("none"))
        .unwrap();
    let t = tank(&s);
    assert_eq!(t["liquid_pressure_mpa"], 0.0);
    assert!(pressure(&s) < stat_p);
}

#[test]
fn a_file_saved_before_keeps_the_vapour_pressure_only_and_its_results() {
    let path = format!(
        "{}/tests/legacy/v2_gas_lpg_before_313.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).unwrap();
    let (s, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
    let i = s.design().options[0].input("tank_liquid_pressure").unwrap();
    assert_eq!(
        (i.value.clone(), i.source),
        (json!("none"), Source::Default)
    );
    assert_eq!(tank(&s)["liquid_pressure_mpa"], 0.0);
}
