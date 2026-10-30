//! Seam: the Design session's results carry what the Intact stability view draws (issue #231): for a
//! gas Design, the GZ points of each loading condition, with the values the criteria use; for a
//! dry-bulk Design, no stability at all. Adding to the results must not move a saved results hash.
//!
//! Label: property. Each test states a rule (a curve starts at zero heel, the governing condition is
//! one of the conditions, the stored hash still matches). No number here is a printed result.

use ovd_format::{DesignSession, HashStatus, Profile, RunOutcome, UNHASHED_KEYS, results_hash};
use serde_json::Value;

fn fixture_text(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The results of the first Option of a fixture, as the Design session gives them.
fn results_of(name: &str) -> Value {
    let (session, opened) = DesignSession::open(&fixture_text(name), Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches, "{name}");
    match session.evaluate().options[0].outcome.as_ref() {
        Some(RunOutcome::Results { results, .. }) => results.clone(),
        other => panic!("{name}: {other:?}"),
    }
}

#[test]
fn a_gas_design_carries_the_gz_points_of_every_loading_condition() {
    let results = results_of("gas_igc_2g_exact");
    let stability = &results["stability"];
    let conditions = stability["conditions"].as_array().expect("conditions");
    assert!(conditions.len() >= 2, "more than one loading condition");
    let mut names = Vec::new();
    for c in conditions {
        let name = c["name"].as_str().unwrap();
        names.push(name);
        let gz = c["gz"].as_array().expect("gz points");
        assert!(gz.len() > 10, "{name}: a curve has many points");
        // Each point is [heel in degrees, GZ in metres], heel rising from zero.
        assert_eq!(gz[0][0].as_i64(), Some(0), "{name}");
        let heels: Vec<i64> = gz.iter().map(|p| p[0].as_i64().unwrap()).collect();
        assert!(heels.windows(2).all(|w| w[0] < w[1]), "{name}: heel rises");
        assert!(gz.iter().all(|p| p[1].is_number()), "{name}");
        // The max-GZ summary is one of the points.
        let max = c["max_gz_m"].as_f64().unwrap();
        let at = c["max_gz_deg"].as_i64().unwrap();
        assert!(
            gz.iter()
                .any(|p| p[0].as_i64() == Some(at) && p[1].as_f64() == Some(max)),
            "{name}: max GZ is a point of the curve"
        );
        assert!(c["gm_fluid_m"].is_number(), "{name}");
        assert!(!c["criteria"].as_array().unwrap().is_empty(), "{name}");
    }
    let governing = stability["lowest_gm_condition"].as_str().unwrap();
    assert!(
        names.contains(&governing),
        "the governing condition is one of the conditions"
    );
}

#[test]
fn a_dry_bulk_design_carries_no_stability() {
    // Saved in the legacy conditions; a dry bulk design in the IS Code ones does carry it.
    let results = results_of("dry_bulk_sloped_hopper");
    assert!(results["stability"].is_null());
}

#[test]
fn a_gas_design_carries_the_criteria_angles_the_view_marks() {
    let results = results_of("gas_igc_2g_exact");
    assert_eq!(
        results["stability_marks"]["criteria_angles_deg"],
        serde_json::json!([30, 40])
    );
}

#[test]
fn a_dry_bulk_design_has_no_marks() {
    assert!(results_of("dry_bulk_sloped_hopper")["stability_marks"].is_null());
}

/// The stored hash of each example matches (checked in `results_of`), and no unhashed key changes a
/// hash, whatever it holds.
#[test]
fn the_unhashed_keys_do_not_change_the_results_hash() {
    for name in ["gas_igc_2g_exact", "dry_bulk_box_hold"] {
        let results = results_of(name);
        let hash = results_hash(&results);
        let mut edited = results.clone();
        for key in UNHASHED_KEYS {
            edited[key] = serde_json::json!({"anything": [1, 2, 3]});
        }
        assert_eq!(results_hash(&edited), hash, "{name}");
        // A key that is not on the list does change it.
        edited["a_result"] = serde_json::json!(1);
        assert_ne!(results_hash(&edited), hash, "{name}");
    }
}
