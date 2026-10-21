//! Seam: the machinery in a Design's results (issue #352): a new self-propelled Option with a
//! specified Propulsor lays out its machinery below deck and weighs it; with no installed power the
//! engine room waits, with a warning; a file saved before keeps today's stern block and its hash.
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

fn zones(r: &Value) -> Vec<String> {
    r["drawing"]["aft_arrangement"]["zones"]
        .as_array()
        .unwrap()
        .iter()
        .map(|z| z[0].as_str().unwrap().to_string())
        .collect()
}

fn items(r: &Value) -> Vec<String> {
    r["stern_block_mass"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["label"].as_str().unwrap().to_string())
        .collect()
}

fn new_dry() -> DesignSession {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    for (n, v) in [
        ("cargo", json!("bauxite")),
        ("length_overall_m", json!(110.0)),
        ("beam_m", json!(11.4)),
        ("moulded_depth_m", json!(4.0)),
    ] {
        s.set_input("Option 1", n, v).unwrap();
    }
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    s
}

#[test]
fn a_typed_installed_power_places_the_engines_below_deck_and_in_the_lightship() {
    let mut s = new_dry();
    // A dry hull's default: shafts in nozzles, diesel-mechanical, a channel jet in the bow.
    let r = results(&s);
    assert_eq!(
        zones(&r)[..2],
        [
            "AFT PEAK, STEERING GEAR".to_string(),
            "SHAFT RUN".to_string()
        ]
    );
    assert!(!zones(&r).contains(&"ENGINE ROOM".to_string()));
    let warned = |r: &Value| {
        r["warnings"].as_array().unwrap().iter().any(|w| {
            w.as_str()
                .is_some_and(|w| w.contains("installed power is not sized"))
        })
    };
    assert!(warned(&r), "{}", r["warnings"]);
    assert!(items(&r).contains(&"rudders and steering gear".to_string()));
    assert!(items(&r).contains(&"bow thruster".to_string()));
    s.set_input("Option 1", "installed_power_kw", json!(1200.0))
        .unwrap();
    let r = results(&s);
    assert!(
        zones(&r).contains(&"ENGINE ROOM".to_string()),
        "{:?}",
        zones(&r)
    );
    assert!(!warned(&r));
    for label in ["engines", "gearboxes", "shaft lines"] {
        assert!(items(&r).contains(&label.to_string()), "{label}");
    }
}

#[test]
fn an_unspecified_propulsor_keeps_todays_stern_block() {
    let mut s = new_dry();
    s.set_input("Option 1", "propulsor_type", json!("unspecified"))
        .unwrap();
    let r = results(&s);
    assert_eq!(zones(&r)[0], "THRUSTER ROOM");
    assert!(items(&r).contains(&"thrusters".to_string()));
}

#[test]
fn a_file_saved_before_keeps_todays_stern_block_and_its_hash() {
    for name in [
        "legacy/v2_gas_lpg_before_313.vdesign",
        "fixtures/gas_igc_2g_self_propelled.vdesign",
    ] {
        let path = format!("{}/tests/{name}", env!("CARGO_MANIFEST_DIR"));
        let (s, opened) =
            DesignSession::open(&std::fs::read_to_string(path).unwrap(), Profile::built_in())
                .unwrap();
        assert_eq!(opened.hash_status[0].1, HashStatus::Matches, "{name}");
        let r = results(&s);
        assert_eq!(zones(&r)[0], "THRUSTER ROOM", "{name}");
    }
}
