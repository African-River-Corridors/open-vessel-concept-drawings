//! Seam: the bilge radius, sheer and camber inputs reach the hull (issue #449). Before, they were
//! inputs only.
//!
//! Label: property. Each test states a rule: a round bilge takes area from the midship section, and
//! a sheer or a camber adds hull above the deck edge only, so the upright hydrostatics at the float
//! keep their values while the results above the deck edge move.

use ovd_format::{DesignSession, Profile, RunOutcome};
use serde_json::{Value, json};

fn raw_fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn results(file: &str, typed: &[(&str, f64)]) -> Value {
    let mut f: Value = serde_json::from_str(&raw_fixture(file)).unwrap();
    for (name, value) in typed {
        f["options"][0]["inputs"][*name] = json!({ "value": value, "source": "typed" });
    }
    let (session, _) = DesignSession::open(&f.to_string(), Profile::built_in()).unwrap();
    match session.evaluate().options[0]
        .outcome
        .as_ref()
        .expect("it ran")
    {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

const FILE: &str = "dry_bulk_box_hold";

#[test]
fn a_bilge_radius_takes_area_from_the_midship_section() {
    let base = results(FILE, &[]);
    let round = results(FILE, &[("bilge_radius_m", 1.0)]);
    let cm = |r: &Value| r["hydrostatics"]["cm"].as_f64().unwrap();
    assert!(
        (cm(&base) - 1.0).abs() < 1e-12,
        "a square bilge: Cm {}",
        cm(&base)
    );
    assert!(cm(&round) < 0.99, "Cm {}", cm(&round));
    assert_ne!(base["hydrostatics"], round["hydrostatics"]);
}

#[test]
fn a_sheer_or_a_camber_moves_only_what_is_above_the_deck_edge() {
    // A gas barge: its IS Code GZ curves heel the deck edge under.
    let file = "gas_igc_2g_exact";
    let base = results(file, &[]);
    for typed in [
        &[("sheer_aft_m", 0.5), ("sheer_fwd_m", 1.0)][..],
        &[("camber_m", 0.3)][..],
    ] {
        let r = results(file, typed);
        // The float is below the deck edge: the same to rounding (the sheer adds Stations).
        let (h0, h1) = (
            base["hydrostatics"].as_object().unwrap(),
            r["hydrostatics"].as_object().unwrap(),
        );
        assert_eq!(h0.len(), h1.len());
        for (key, v0) in h0 {
            let (x, y) = (v0.as_f64().unwrap(), h1[key].as_f64().unwrap());
            assert!(
                (x - y).abs() <= 1e-9 * x.abs().max(1.0),
                "{typed:?}: hydrostatics.{key} {x} vs {y}"
            );
        }
        assert_ne!(
            base, r,
            "{typed:?}: the hull above the deck edge is in the results"
        );
    }
}
