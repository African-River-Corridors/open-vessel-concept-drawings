//! Seam: the tank head type as an input (issue #375). A new Design takes the head type of its
//! Cargo and carriage condition from the Defaults Profile; the head type reaches the wall, the
//! volume, the steel and the drawing; a file saved before keeps hemispherical heads.
//!
//! Label: property. The values compared are the engine's own (tests/head_area.rs in ovd-core holds
//! the sourced head area).

use ovd_format::{DesignSession, HashStatus, Profile, RunOutcome, Source, ga_views_of};
use serde_json::{Value, json};

fn new_design(cargo: &str, carriage: &str) -> DesignSession {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    for (name, value) in [
        ("cargo", json!(cargo)),
        ("length_overall_m", json!(110.0)),
        ("beam_m", json!(11.4)),
        ("moulded_depth_m", json!(6.0)),
    ] {
        s.set_input("Option 1", name, value).unwrap();
    }
    if !carriage.is_empty() {
        s.set_input("Option 1", "lpg_carriage_condition", json!(carriage))
            .unwrap();
    }
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    s
}

fn outcome(s: &DesignSession) -> RunOutcome {
    s.evaluate().options[0].outcome.clone().expect("it ran")
}

fn results(s: &DesignSession) -> Value {
    match outcome(s) {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

#[test]
fn a_new_design_takes_the_head_type_of_its_cargo() {
    for (cargo, carriage, head) in [
        ("LNG", "", "hemisphere"),
        ("LPG_propane", "fully_pressurised", "ellipsoidal_2to1"),
        ("LPG_butane", "semi_pressurised", "ellipsoidal_2to1"),
        ("LPG_mix", "fully_refrigerated", "ellipsoidal_2to1"),
    ] {
        let s = new_design(cargo, carriage);
        let i = s.design().options[0].input("tank_head_type").unwrap();
        assert_eq!((i.value.clone(), i.source), (json!(head), Source::Default));
        assert_eq!(results(&s)["drawing"]["tank"]["head_type"], head);
    }
    let rows = Profile::built_in();
    let row = rows
        .table_default(
            "tank_head_type",
            &[
                ("cargo_containment", "type_c_tanks"),
                ("cargo", "LPG_propane"),
                ("lpg_carriage_condition", "fully_pressurised"),
            ],
        )
        .unwrap();
    assert_eq!(row.status, "estimate");
    assert!(row.basis.contains("DESIGN 2006"));
}

#[test]
fn a_2_to_1_head_is_shorter_and_holds_less_than_a_hemisphere() {
    let mut s = new_design("LNG", "");
    let r0 = results(&s);
    s.set_input("Option 1", "tank_head_type", json!("ellipsoidal_2to1"))
        .unwrap();
    let r1 = results(&s);
    assert_eq!(r1["drawing"]["tank"]["head_type"], "ellipsoidal_2to1");
    // The ASME 2:1 head formula asks for about twice the hemisphere's head plate.
    assert!(
        f(&r1["drawing"]["tank"]["head_wall_thickness_mm"])
            > f(&r0["drawing"]["tank"]["head_wall_thickness_mm"])
    );
    assert_ne!(r1["results"], r0["results"]);
}

#[test]
fn the_plan_draws_half_ellipse_ends_half_a_radius_long() {
    let mut s = new_design("LNG", "");
    s.set_input("Option 1", "tank_head_type", json!("ellipsoidal_2to1"))
        .unwrap();
    let r = results(&s);
    let od = f(&r["geometry"]["tank_outer_diameter_m"]);
    let views = ga_views_of(&outcome(&s)).unwrap();
    let ends: Vec<f64> = views.plan["viewports"][0]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["type"] == "polyline" && e["points_m"].as_array().unwrap().len() == 25)
        .map(|e| {
            let xs: Vec<f64> = e["points_m"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| f(&p[0]))
                .collect();
            xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
                - xs.iter().cloned().fold(f64::INFINITY, f64::min)
        })
        .collect();
    assert!(!ends.is_empty(), "the 2:1 ends are drawn");
    // The full outline's ends are R/2 = OD/4 long; the outline cut at the deck is smaller.
    let longest = ends.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        (longest - od / 4.0).abs() < 1e-9,
        "{ends:?} against {}",
        od / 4.0
    );
    assert!(ends.iter().all(|w| *w <= od / 4.0 + 1e-9), "{ends:?}");
    // A hemispherical design draws arcs, as before.
    let s = new_design("LNG", "");
    let views = ga_views_of(&outcome(&s)).unwrap();
    assert!(
        !views.plan["viewports"][0]["entities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["type"] == "polyline" && e["points_m"].as_array().unwrap().len() == 25)
    );
}

#[test]
fn a_file_saved_before_keeps_hemispherical_heads_and_its_results() {
    let path = format!(
        "{}/tests/legacy/v2_gas_lpg_before_313.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).unwrap();
    let (s, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
    let i = s.design().options[0].input("tank_head_type").unwrap();
    assert_eq!(
        (i.value.clone(), i.source),
        (json!("hemisphere"), Source::Default)
    );
}
