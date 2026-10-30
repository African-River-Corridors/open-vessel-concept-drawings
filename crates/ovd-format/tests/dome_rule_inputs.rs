//! Seam: the tank dome default is the dome rule (issue #316). A new Design stores none for the dome
//! diameter and height (marked default, an `estimate` row of the Defaults Profile) and takes the
//! rule of each tank; a typed size is an override; a saved file that stores 2.0 m keeps it, and a
//! file that lacks the inputs keeps the old 2.0 m.
//!
//! Label: property. The values compared are the rule's own (tests/dome_rule.rs in ovd-core holds
//! the sourced points).

use ovd_core::barge_capacity::dome_dimensions;
use ovd_format::{DesignSession, HashStatus, Profile, RunOutcome, Source};
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

fn results(s: &DesignSession) -> Value {
    match s.evaluate().options[0].outcome.as_ref().expect("it ran") {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_new_design_stores_none_and_takes_the_rule_of_its_tank() {
    for cargo in ["LNG", "LPG_propane"] {
        let s = new_design(cargo);
        for name in ["tank_dome_diameter_m", "tank_dome_height_m"] {
            let i = s.design().options[0].input(name).unwrap();
            assert_eq!((i.value.clone(), i.source), (Value::Null, Source::Default));
        }
        let r = results(&s);
        let t = &r["drawing"]["tank"];
        let steel = t["tank_steel_outer_diameter_m"].as_f64().unwrap();
        let (d, h) = dome_dimensions(steel, None, None);
        assert_eq!(t["dome_diameter_m"], json!(d), "{cargo}");
        assert_eq!(t["dome_height_m"], json!(h), "{cargo}");
        assert_eq!(t["dome_source"], "rule");
        // The tank top and the drawing take the same dome.
        assert_eq!(r["drawing"]["dome_height_m"], json!(h), "{cargo}");
    }
}

#[test]
fn a_typed_dome_is_an_override() {
    let mut s = new_design("LNG");
    s.set_input("Option 1", "tank_dome_diameter_m", json!(3.0))
        .unwrap();
    let t = results(&s)["drawing"]["tank"].clone();
    assert_eq!(t["dome_diameter_m"], 3.0);
    // The height follows the diameter in force.
    assert_eq!(t["dome_height_m"], 3.0);
    assert_eq!(t["dome_source"], "rule and override");
    s.set_input("Option 1", "tank_dome_height_m", json!(1.0))
        .unwrap();
    let r = results(&s);
    assert_eq!(r["drawing"]["dome_height_m"], 1.0);
    assert_eq!(r["drawing"]["tank"]["dome_source"], "override");
}

#[test]
fn a_saved_file_keeps_its_2_m_dome_and_its_results() {
    let path = format!(
        "{}/tests/legacy/v2_gas_lpg_before_313.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).unwrap();
    let (s, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
    let r = results(&s);
    assert_eq!(r["drawing"]["dome_height_m"], 2.0);
    assert_eq!(r["drawing"]["tank"]["dome_diameter_m"], 2.0);
    assert_eq!(r["drawing"]["tank"]["dome_source"], "override");
}
