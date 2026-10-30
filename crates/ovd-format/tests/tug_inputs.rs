//! Seam: the tug of a barge Formation as inputs (issue #357, spec 0007 "Formation and Tug"): the
//! Formation kinds `atb` and `towed_barge`, the "Tug" group, the defaults by Formation kind, the
//! `reference` source a tug preset fills, and the ATB and towed-barge Vessel Classes. Inputs only:
//! no result moves.
//!
//! Label: property.

use ovd_format::{DesignSession, HashStatus, Profile, Source, input_schema};
use serde_json::{Value, json};

fn input(s: &DesignSession, name: &str) -> Option<(Value, Source)> {
    s.design().options[0]
        .input(name)
        .map(|i| (i.value.clone(), i.source))
}

fn barge(kind: &str) -> DesignSession {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    s.set_input("Option 1", "cargo", json!("bauxite")).unwrap();
    s.set_formation("Option 1", kind, None, None).unwrap();
    s
}

#[test]
fn the_schema_lists_the_tug_inputs_and_the_two_new_formation_kinds() {
    let schema = input_schema(&Profile::built_in());
    let inputs = schema["inputs"].as_array().unwrap();
    let formation = inputs.iter().find(|i| i["name"] == "formation").unwrap();
    let kinds: Vec<&str> = formation["choices"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["value"].as_str().unwrap())
        .collect();
    assert!(
        kinds.contains(&"atb") && kinds.contains(&"towed_barge"),
        "{kinds:?}"
    );
    assert!(schema["groups"].as_array().unwrap().contains(&json!("Tug")));
    let tug: Vec<&Value> = inputs.iter().filter(|i| i["group"] == "Tug").collect();
    for name in [
        "tug_included",
        "tug_preset",
        "tug_length_m",
        "tug_propulsor_type",
        "formation_resistance_allowance",
        "notch_depth_m",
    ] {
        let i = tug
            .iter()
            .find(|i| i["name"] == name)
            .unwrap_or_else(|| panic!("{name}"));
        assert_eq!(i["default_status"], "estimate", "{name}");
    }
    let notch = tug.iter().find(|i| i["name"] == "notch_depth_m").unwrap();
    assert_eq!(notch["applies_to_formation"], json!(["atb"]));
}

#[test]
fn the_tug_defaults_follow_the_formation_kind() {
    let pusher = barge("pushed_barge");
    assert_eq!(input(&pusher, "tug_included").unwrap().0, json!(true));
    assert_eq!(input(&pusher, "tug_stern_form").unwrap().0, json!("tunnel"));
    assert_eq!(
        input(&pusher, "tug_propulsor_type").unwrap().0,
        json!("shaft_nozzle")
    );
    let atb = barge("atb");
    assert_eq!(input(&atb, "tug_stern_form").unwrap().0, json!("transom"));
    assert_eq!(
        input(&atb, "formation_resistance_allowance").unwrap().0,
        json!(1.05)
    );
    assert!(input(&atb, "notch_depth_m").is_some());
    assert!(input(&pusher, "notch_depth_m").is_none());
    // A self-propelled hull has no tug.
    let sp = barge("self_propelled");
    assert!(input(&sp, "tug_included").is_none());
}

#[test]
fn a_preset_fills_the_tug_inputs_marked_reference_and_typing_one_marks_it_typed() {
    let mut s = barge("pushed_barge");
    s.set_input("Option 1", "tug_preset", json!("cds-4115"))
        .unwrap();
    assert_eq!(
        input(&s, "tug_installed_power_kw"),
        Some((json!(3990.0), Source::Reference))
    );
    assert_eq!(
        input(&s, "tug_propulsor_count"),
        Some((json!(3.0), Source::Reference))
    );
    assert_eq!(
        input(&s, "tug_length_m"),
        Some((json!(41.0), Source::Reference))
    );
    // The file keeps the marker.
    let text = s.save().unwrap();
    assert!(text.contains("\"reference\""), "{text}");
    // Typing a value makes it typed; the others stay reference.
    s.set_input("Option 1", "tug_length_m", json!(40.0))
        .unwrap();
    assert_eq!(input(&s, "tug_length_m").unwrap().1, Source::Typed);
    assert_eq!(input(&s, "tug_beam_m").unwrap().1, Source::Reference);
    // `none` keeps the preset's values, now typed.
    s.set_input("Option 1", "tug_preset", json!("none"))
        .unwrap();
    assert_eq!(input(&s, "tug_beam_m"), Some((json!(15.5), Source::Typed)));
    assert!(
        s.set_input("Option 1", "tug_preset", json!("no-such-tug"))
            .is_err()
    );
}

#[test]
fn an_atb_and_a_towed_barge_vessel_class_can_be_picked() {
    use ovd_core::vessel_classes::CLASSES;
    for kind in ["atb", "towed_barge"] {
        let class = CLASSES
            .iter()
            .find(|c| c.formation == kind && c.pickable())
            .unwrap_or_else(|| panic!("no pickable {kind} class"));
        let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
        s.set_input("Option 1", "cargo", json!("bauxite")).unwrap();
        s.set_input("Option 1", "vessel_class", json!(class.id))
            .unwrap();
        let f = input(&s, "formation").unwrap();
        assert_eq!(f.0["kind"], kind, "{}", class.id);
        assert_eq!(f.1, Source::Class);
    }
}

#[test]
fn a_file_saved_before_has_no_tug_and_keeps_its_hash() {
    for name in [
        "fixtures/dry_bulk_box_hold.vdesign",
        "legacy/v2_gas_lpg_before_313.vdesign",
    ] {
        let path = format!("{}/tests/{name}", env!("CARGO_MANIFEST_DIR"));
        let (s, opened) =
            DesignSession::open(&std::fs::read_to_string(path).unwrap(), Profile::built_in())
                .unwrap();
        assert_eq!(opened.hash_status[0].1, HashStatus::Matches, "{name}");
        if let Some((v, _)) = input(&s, "tug_included") {
            assert_eq!(v, json!(false), "{name}");
        }
    }
}
