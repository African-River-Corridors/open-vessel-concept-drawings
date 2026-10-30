//! Seam: `drawing.propulsors`, the Propulsor Layout (issue #691, spec 0009 "Propulsors"): one set
//! of position functions in `machinery.rs` places every propulsor, shaft line, rudder and the bow
//! thruster, and the mass items take their centres from them, so the layout adds geometry and moves
//! no result.
//!
//! Label: property - each part's mass centre equals its mass item's LCG and KCG, the disc axis is at
//! `powering.shaft_axis_height_m`, under a tunnel stern each disc lies on its tunnel's axis, and the
//! stored results hash of every self-propelled fixture still matches. The pinned numbers of the gas
//! fixture's thrusters are labelled `snapshot` where they are written.

use ovd_format::{DesignSession, HashStatus, Profile, RunOutcome, UNHASHED_KEYS, results_hash};
use serde_json::{Value, json};

const SELF_PROPELLED_FIXTURES: [&str; 2] = [
    "crude_integral_tanks_self_propelled",
    "gas_igc_2g_self_propelled",
];

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

fn results(s: &DesignSession) -> Value {
    match s.evaluate().options[0].outcome.as_ref().expect("it ran") {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

fn close(a: f64, b: &Value) -> bool {
    (a - b.as_f64().unwrap()).abs() < 1e-9
}

/// The mass item of `label` in the stern block's mass, as (lcg, kcg).
fn item(r: &Value, label: &str) -> (f64, f64) {
    let i = r["stern_block_mass"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["label"] == label)
        .unwrap_or_else(|| panic!("no mass item {label}: {}", r["stern_block_mass"]));
    (i["lcg_m"].as_f64().unwrap(), i["kcg_m"].as_f64().unwrap())
}

/// Every part of the layout sits at its mass item's centre (when the stern block is weighed).
fn assert_parts_at_their_mass_items(r: &Value) {
    let l = &r["drawing"]["propulsors"];
    let parts = l["propulsors"]
        .as_array()
        .unwrap()
        .iter()
        .chain(l["rudders"].as_array().unwrap())
        .chain(l["bow_thruster"].as_object().map(|_| &l["bow_thruster"]));
    for p in parts {
        let (lcg, kcg) = item(r, p["mass_item"].as_str().unwrap());
        assert!(close(lcg, &p["mass_centre"]["lcg_m"]), "{p}");
        assert!(close(kcg, &p["mass_centre"]["kcg_m"]), "{p}");
    }
}

#[test]
fn every_self_propelled_fixture_has_a_layout_and_keeps_its_results_hash() {
    for name in SELF_PROPELLED_FIXTURES {
        let (s, opened) = DesignSession::open(&fixture(name), Profile::built_in()).unwrap();
        // The results hash is unchanged: the masses call the same places they used.
        assert_eq!(opened.hash_status[0].1, HashStatus::Matches, "{name}");
        let r = results(&s);
        let l = &r["drawing"]["propulsors"];
        assert!(l.is_object(), "{name}: no drawing.propulsors");
        assert!(UNHASHED_KEYS.contains(&"drawing"));
        // Out of the hash: removing the layout leaves the hash as it is.
        let mut without = r.clone();
        without["drawing"]
            .as_object_mut()
            .unwrap()
            .remove("propulsors");
        assert_eq!(results_hash(&without), results_hash(&r), "{name}");
        assert!(!l["propulsors"].as_array().unwrap().is_empty(), "{name}");
        if r["stern_block_mass"].is_object() {
            assert_parts_at_their_mass_items(&r);
        } else {
            // The stern block is not weighed in this file: weigh it, and the layout is the same and
            // sits at the items.
            let mut weighed = s.clone();
            weighed
                .set_input("Option 1", "stern_block_mass_included", json!(true))
                .unwrap();
            let w = results(&weighed);
            assert_eq!(w["drawing"]["propulsors"], *l, "{name}");
            assert_parts_at_their_mass_items(&w);
        }
        // The axis is the powering result's, where the powering has one.
        if let Some(axis) = r["powering"]["shaft_axis_height_m"].as_f64() {
            for p in l["propulsors"].as_array().unwrap() {
                assert!(close(axis, &p["z_m"]), "{name}: {p}");
            }
            assert!(close(
                r["powering"]["propeller_diameter_m"].as_f64().unwrap(),
                &l["diameter_m"]
            ));
        }
    }
}

#[test]
fn the_pushed_fixtures_have_no_layout() {
    for name in [
        "crude_integral_tanks_pushed",
        "diesel_integral_tanks_pushed",
    ] {
        let (s, _) = DesignSession::open(&fixture(name), Profile::built_in()).unwrap();
        assert!(results(&s)["drawing"].get("propulsors").is_none(), "{name}");
    }
}

#[test]
fn the_gas_fixture_pins_its_three_default_thrusters() {
    let (s, _) =
        DesignSession::open(&fixture("gas_igc_2g_self_propelled"), Profile::built_in()).unwrap();
    let r = results(&s);
    let l = &r["drawing"]["propulsors"];
    // Label: snapshot - the default stern block's ducted azimuth thrusters at the thruster room's
    // centre, D = the envelope / 1.35, the axis at D / 2 and the unit's centre at D.
    assert_eq!(l["propulsor_type"], "unspecified");
    let units = l["propulsors"].as_array().unwrap();
    let offsets = r["drawing"]["aft_arrangement"]["thruster_offsets_m"]
        .as_array()
        .unwrap();
    assert_eq!(units.len(), offsets.len());
    assert_eq!(units.len(), 3);
    let d = l["diameter_m"].as_f64().unwrap();
    for (u, y) in units.iter().zip(offsets) {
        assert!(close(y.as_f64().unwrap(), &u["y_m"]), "{u}");
        assert!(close(d / 2.0, &u["z_m"]), "{u}");
        assert!(close(d, &u["mass_centre"]["kcg_m"]), "{u}");
        assert!(close(
            r["drawing"]["aft_arrangement"]["thruster_room_length_m"]
                .as_f64()
                .unwrap()
                / 2.0,
            &u["x_m"]
        ));
        assert!(close(d, &u["nozzle"]["inner_diameter_m"]));
        assert!(u["azimuth"].is_object());
    }
    assert!(l["rudders"].as_array().unwrap().is_empty());
}

#[test]
fn under_a_tunnel_stern_each_disc_lies_on_its_tunnels_axis() {
    let (s, _) = DesignSession::open(
        &fixture("crude_integral_tanks_self_propelled"),
        Profile::built_in(),
    )
    .unwrap();
    let r = results(&s);
    let l = &r["drawing"]["propulsors"];
    assert_eq!(l["propulsor_type"], "shaft_nozzle");
    // Each tunnel section is the arc over one tunnel: its axis is the middle of its y range.
    let axes: Vec<f64> = r["drawing"]["stern"]["tunnel_sections"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            let ys: Vec<f64> = s
                .as_array()
                .unwrap()
                .iter()
                .map(|p| p[0].as_f64().unwrap())
                .collect();
            let (lo, hi) = ys
                .iter()
                .fold((f64::MAX, f64::MIN), |(a, b), &y| (a.min(y), b.max(y)));
            (lo + hi) / 2.0
        })
        .collect();
    let units = l["propulsors"].as_array().unwrap();
    assert_eq!(units.len(), axes.len());
    for (u, axis) in units.iter().zip(&axes) {
        assert!(
            (u["y_m"].as_f64().unwrap() - axis).abs() < 1e-6,
            "{u} vs {axis}"
        );
        // A shaft propeller at the aft end of its shaft line, in a nozzle.
        assert!(close(u["shaft"]["x_aft_m"].as_f64().unwrap(), &u["x_m"]));
        assert!(u["nozzle"].is_object());
    }
    // Two rudders aft of each propeller, the bow thruster in the bow.
    assert_eq!(l["rudders"].as_array().unwrap().len(), 2 * units.len());
    for rd in l["rudders"].as_array().unwrap() {
        assert!(rd["x_m"].as_f64().unwrap() < units[0]["x_m"].as_f64().unwrap());
    }
    assert!(l["bow_thruster"].is_object());
}

fn new_self_propelled(propulsor: &str, stern: &str) -> DesignSession {
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
    s.set_input("Option 1", "propulsor_type", json!(propulsor))
        .unwrap();
    s.set_input("Option 1", "stern_form", json!(stern)).unwrap();
    s.set_input("Option 1", "stern_block_mass_included", json!(true))
        .unwrap();
    s
}

#[test]
fn every_propulsor_type_sits_at_its_mass_items_and_counts_propulsor_count() {
    for propulsor in [
        "azimuth_open",
        "azimuth_ducted",
        "shaft_open",
        "shaft_nozzle",
        "voith_schneider",
        "pump_jet",
    ] {
        for stern in ["transom", "tunnel"] {
            let s = new_self_propelled(propulsor, stern);
            let r = results(&s);
            let l = &r["drawing"]["propulsors"];
            assert_eq!(l["propulsor_type"], propulsor);
            assert_eq!(
                l["propulsors"].as_array().unwrap().len() as f64,
                2.0,
                "{propulsor} {stern}: the default propulsor_count"
            );
            assert_parts_at_their_mass_items(&r);
            let axis = r["powering"]["shaft_axis_height_m"].as_f64().unwrap();
            for p in l["propulsors"].as_array().unwrap() {
                assert!(close(axis, &p["z_m"]), "{propulsor} {stern}: {p}");
            }
            assert_eq!(
                l["propulsors"][0]["nozzle"].is_object(),
                propulsor == "azimuth_ducted" || propulsor == "shaft_nozzle",
                "{propulsor}"
            );
            assert_eq!(
                l["propulsors"][0]["azimuth"].is_object(),
                propulsor.starts_with("azimuth"),
                "{propulsor}"
            );
            assert_eq!(
                l["rudders"].as_array().unwrap().is_empty(),
                !propulsor.starts_with("shaft"),
                "{propulsor}"
            );
        }
    }
}

/// The athwartships offsets of the stern block (`drawing.aft_arrangement.thruster_offsets_m`).
fn stern_offsets(r: &Value) -> Vec<f64> {
    r["drawing"]["aft_arrangement"]["thruster_offsets_m"]
        .as_array()
        .unwrap()
        .iter()
        .map(|y| y.as_f64().unwrap())
        .collect()
}

fn layout_offsets(r: &Value) -> Vec<f64> {
    r["drawing"]["propulsors"]["propulsors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["y_m"].as_f64().unwrap())
        .collect()
}

#[test]
fn away_from_a_tunnel_the_stern_block_takes_propulsor_count_units_at_the_propulsor_pitch() {
    // Issue #702, option 1: with a specified Propulsor at a transom or pram stern, the stern block
    // lays out `propulsor_count` units at a pitch of the Propulsor's own envelope (1.35 D) plus the
    // thruster clearance (0.6 m by default), so its offsets and the Propulsor Layout agree.
    for propulsor in [
        "azimuth_open",
        "azimuth_ducted",
        "shaft_open",
        "shaft_nozzle",
        "voith_schneider",
        "pump_jet",
    ] {
        for stern in ["transom", "pram"] {
            for count in [2, 3] {
                let mut s = new_self_propelled(propulsor, stern);
                s.set_input("Option 1", "propulsor_count", json!(count))
                    .unwrap();
                let r = results(&s);
                let l = &r["drawing"]["propulsors"];
                let held = stern_offsets(&r);
                assert_eq!(held.len(), count, "{propulsor} {stern} {count}: {held:?}");
                let ys = layout_offsets(&r);
                assert_eq!(ys.len(), count, "{propulsor} {stern} {count}");
                for (a, b) in held.iter().zip(&ys) {
                    assert!((a - b).abs() < 1e-9, "{propulsor} {stern}: {held:?} {ys:?}");
                }
                let pitch = 1.35 * l["diameter_m"].as_f64().unwrap() + 0.6;
                for w in held.windows(2) {
                    assert!(
                        (w[1] - w[0] - pitch).abs() < 1e-9,
                        "{propulsor} {stern}: {held:?}, pitch {pitch}"
                    );
                }
                assert!((held[0] + held[count - 1]).abs() < 1e-9, "{held:?}");
                assert!(
                    l["warnings"].as_array().unwrap().is_empty(),
                    "{propulsor} {stern}: {l}"
                );
            }
        }
    }
}

#[test]
fn a_beam_that_takes_fewer_units_than_propulsor_count_spreads_them_and_warns() {
    // The stern block fits what the beam takes; the layout keeps `propulsor_count` units (the
    // masses count them), spreads them at the stern block's pitch and warns.
    let mut s = new_self_propelled("shaft_nozzle", "transom");
    // Four units of D = 1.6 m need 4 x 2.16 + 3 x 0.6 + 2 x 0.6 = 11.64 m; the beam is 11.4 m.
    s.set_input("Option 1", "propeller_diameter_m", json!(1.6))
        .unwrap();
    s.set_input("Option 1", "propulsor_count", json!(4))
        .unwrap();
    let r = results(&s);
    let l = &r["drawing"]["propulsors"];
    let held = stern_offsets(&r);
    assert!(
        held.len() < 4,
        "the case needs a beam that takes fewer: {held:?}"
    );
    let warnings = l["warnings"].as_array().unwrap();
    assert_eq!(warnings.len(), 1, "{l}");
    assert!(
        warnings[0]
            .as_str()
            .unwrap()
            .contains("propulsor places for 4 propulsors")
    );
    let ys = layout_offsets(&r);
    assert_eq!(ys.len(), 4);
    // Spread about the centre line at the stern block's pitch.
    assert!((ys[0] + ys[3]).abs() < 1e-9, "{ys:?}");
    let pitch = 1.35 * l["diameter_m"].as_f64().unwrap() + 0.6;
    assert!((ys[1] - ys[0] - pitch).abs() < 1e-9, "{ys:?}");
    // The layout's warning is not a result warning: the results warnings do not hold it.
    assert!(
        !r["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w.as_str().unwrap_or("").contains("propulsor places"))
    );
}
