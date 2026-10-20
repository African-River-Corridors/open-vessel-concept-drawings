//! Seam: the allowable stress basis of the tank steel (issue #313). A new Design takes the IGC
//! 4.23.3.1 limit, the lesser of Rm/A and Re/B; a file saved before the basis was an input keeps
//! the stress it was worked with (S690QL1 at 285 MPa), so its results do not change.
//!
//! Label: property. Each test states a rule: the default basis, its effect on the wall, an old file
//! keeps its results, a stress above the IGC limit warns. The numbers compared are the material
//! database's own (tests/allowable_stress.rs in ovcd-core holds the sourced values).

use ovcd_core::pressure_vessel::{get_tank_material, igc_allowable_stress};
use ovcd_format::vdesign::{ALLOWABLE_STRESS_BASES, LEGACY_FIXED_OTHER};
use ovcd_format::{DesignSession, HashStatus, Profile, RunOutcome, Source, check};
use serde_json::{Value, json};

fn legacy(name: &str) -> String {
    let path = format!("{}/tests/legacy/{name}.vdesign", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn new_lpg(carriage: &str) -> DesignSession {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    for (name, value) in [
        ("cargo", json!("LPG_propane")),
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

fn tank(s: &DesignSession) -> Value {
    match s.evaluate().options[0].outcome.as_ref().expect("it ran") {
        RunOutcome::Results { results, .. } => results["drawing"]["tank"].clone(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_new_design_takes_the_igc_basis_and_the_igc_stress() {
    let s = new_lpg("fully_pressurised");
    let o = &s.design().options[0];
    let basis = o.input("tank_allowable_stress_basis").unwrap();
    assert_eq!(
        (basis.value.clone(), basis.source),
        (json!("igc_4_23_3_1"), Source::Default)
    );
    let t = tank(&s);
    assert_eq!(t["material"], "S690QL1");
    assert_eq!(t["allowable_stress_basis"], "igc_4_23_3_1");
    let s690 = get_tank_material("S690QL1").unwrap();
    assert_eq!(t["allowable_stress_mpa"], json!(igc_allowable_stress(s690)));
    assert_eq!(ALLOWABLE_STRESS_BASES[0], "igc_4_23_3_1");
}

#[test]
fn the_old_basis_gives_285_mpa_and_a_thinner_wall_for_s690ql1_only() {
    let wall = |t: &Value| t["cylinder_wall_thickness_mm"].as_f64().unwrap();
    let mut s = new_lpg("fully_pressurised");
    let igc = tank(&s);
    s.set_input(
        "Option 1",
        "tank_allowable_stress_basis",
        json!("published_design"),
    )
    .unwrap();
    let old = tank(&s);
    assert_eq!(old["allowable_stress_mpa"], 285.0);
    assert!(wall(&igc) > wall(&old), "{igc} {old}");
    // 9Ni (the semi-pressurised default) is 230 MPa on both bases.
    let mut s = new_lpg("semi_pressurised");
    let igc = tank(&s);
    s.set_input(
        "Option 1",
        "tank_allowable_stress_basis",
        json!("published_design"),
    )
    .unwrap();
    assert_eq!(tank(&s), {
        let mut t = igc.clone();
        t["allowable_stress_basis"] = json!("published_design");
        t
    });
    assert_eq!(igc["allowable_stress_mpa"], 230.0);
}

#[test]
fn a_file_without_the_basis_keeps_the_old_stress_and_its_results() {
    let file = "v2_gas_lpg_before_313";
    let text = legacy(file);
    let raw: Value = serde_json::from_str(&text).unwrap();
    let inputs = &raw["options"][0]["inputs"];
    assert!(inputs.get("tank_allowable_stress_basis").is_none());
    assert_eq!(inputs["tank_material"]["value"], "S690QL1");
    assert_eq!(inputs["tank_allowable_stress_mpa"]["value"], Value::Null);
    let checked = check(&text, &Profile::built_in());
    assert!(
        checked.report.errors.is_empty(),
        "{:?}",
        checked.report.errors
    );
    let o = &checked.design.as_ref().unwrap().options[0];
    let basis = o.input("tank_allowable_stress_basis").unwrap();
    assert_eq!(
        (basis.value.clone(), basis.source),
        (json!("published_design"), Source::Default)
    );
    let (s, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches, "{file}");
    assert_eq!(tank(&s)["allowable_stress_mpa"], 285.0);
    // The file is told why its stress is above the IGC limit.
    assert!(
        checked
            .report
            .warnings
            .iter()
            .any(|w| w.contains("above the IGC 4.23.3.1 limit") && w.contains("published_design")),
        "{:?}",
        checked.report.warnings
    );
}

#[test]
fn the_legacy_basis_is_the_old_one() {
    let row = LEGACY_FIXED_OTHER
        .iter()
        .find(|l| l.name == "tank_allowable_stress_basis")
        .expect("a file without the basis gets a fixed value");
    assert_eq!(row.value, "\"published_design\"");
}

#[test]
fn a_typed_stress_above_the_igc_limit_warns_and_one_below_does_not() {
    let mut s = new_lpg("fully_pressurised");
    let warns = |s: &DesignSession| {
        check(&s.save().unwrap(), &Profile::built_in())
            .report
            .warnings
            .iter()
            .any(|w| w.contains("above the IGC 4.23.3.1 limit"))
    };
    assert!(!warns(&s));
    s.set_input("Option 1", "tank_allowable_stress_mpa", json!(300.0))
        .unwrap();
    assert!(warns(&s));
    s.set_input("Option 1", "tank_allowable_stress_mpa", json!(200.0))
        .unwrap();
    assert!(!warns(&s));
}
