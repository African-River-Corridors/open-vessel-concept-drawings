//! Seam: the Stern Form in a Design's results (issue #350): a new self-propelled Option gets
//! `geometry.stern_*` and the stern outlines in the drawing; a file saved before gets no new field
//! and keeps its hash. Issue #448: Cb is a result of the hull's sections, ∇ / (L_WL B_WL T).
//!
//! Label: property.

use ovd_format::{DesignSession, HashStatus, Profile, RunOutcome};
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

fn cb(r: &Value) -> f64 {
    r["dims"]["cb"].as_f64().unwrap()
}

#[test]
fn a_new_gas_option_has_a_transom_stern_and_the_block_coefficient_rule() {
    let mut s = new_option("LNG", 6.0);
    let r = results(&s);
    let g = &r["geometry"];
    assert_eq!(g["stern_form"], "transom", "{g}");
    // Spec 0007: a transom run of 1.0 x depth.
    assert_eq!(g["stern_run_length_m"], 6.0);
    assert_eq!(g["stern_rake_length_m"], 6.0);
    assert!(g["propeller_d_max_m"].as_f64().unwrap() > 0.0);
    assert!(r["drawing"]["stern"]["profile"].as_array().unwrap().len() > 1);
    // Cb of the raked box with the transom cut: below a box's 1, above the old fixed 0.85.
    let with_transom = cb(&r);
    assert!(with_transom > 0.85 && with_transom < 1.0, "{with_transom}");
    // Cb is not an input.
    assert!(
        s.set_input("Option 1", "block_coefficient", json!(0.85))
            .is_err()
    );
}

#[test]
fn a_new_dry_option_has_a_tunnel_stern_with_no_rake_for_the_drawings() {
    let r = results(&new_option("bauxite", 4.0));
    let g = &r["geometry"];
    assert_eq!(g["stern_form"], "tunnel", "{g}");
    assert!(g.get("stern_rake_length_m").is_none(), "{g}");
    let t = g["stern_transom_height_m"].as_f64().unwrap();
    assert_eq!(t, 0.0);
    assert_eq!(
        r["drawing"]["stern"]["tunnel_sections"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    // D_max under a tunnel is 0.80 of the design draught.
    let d_max = g["propeller_d_max_m"].as_f64().unwrap();
    assert!(d_max > 0.0 && d_max < 4.0, "{d_max}");
}

#[test]
fn a_barge_has_no_stern_form_and_the_cb_of_its_raked_box() {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    s.set_input("Option 1", "cargo", json!("bauxite")).unwrap();
    let r = results(&s);
    assert!(r["geometry"].get("stern_form").is_none());
    assert!(r["drawing"].get("stern").is_none());
    // A square stern and the raked bow: Cb on L_WL is just under 1.
    let c = cb(&r);
    assert!(c > 0.9 && c < 1.0, "{c}");
}

#[test]
fn a_file_saved_before_has_no_stern_fields_and_keeps_its_hash() {
    // The version 1 file's ruleset changed in version 2, so its stored hash differs by design
    // (`aft_block_inputs.rs`); it still gets no stern field.
    for (name, hash_kept) in [
        ("legacy/v2_gas_lpg_before_313.vdesign", true),
        ("fixtures/gas_igc_2g_self_propelled.vdesign", true),
        ("legacy/v1_adn_g_self_propelled.vdesign", false),
    ] {
        let path = format!("{}/tests/{name}", env!("CARGO_MANIFEST_DIR"));
        let (s, opened) =
            DesignSession::open(&std::fs::read_to_string(path).unwrap(), Profile::built_in())
                .unwrap();
        if hash_kept {
            assert_eq!(opened.hash_status[0].1, HashStatus::Matches, "{name}");
        }
        let r = results(&s);
        assert!(r["geometry"].get("stern_form").is_none(), "{name}");
        assert!(r["drawing"].get("stern").is_none(), "{name}");
        // Issue #448: the stored Cb is dropped; the hull's sections give it.
        assert!(cb(&r) > 0.9 && cb(&r) < 1.0, "{name}: {}", cb(&r));
    }
}
