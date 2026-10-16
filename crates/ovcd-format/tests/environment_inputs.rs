//! Seam: the water density and the squat safety margin are inputs (issue #4, slice 6). Both were
//! fixed in the engine; the defaults are those values, so no design moves. The water density was
//! held in four places (the `simulate` constant, the capacity inputs, the stability default and the
//! journey de-rating); on the `ovcd run` path all four read the one input.
//!
//! Label: property. Every test states a rule: a default equals the old constant, an old file keeps
//! its results (its stored hash still matches), a draught scales by one over the density, or a
//! margin moves the squat flag. No number here is a printed result.

use ovcd_core::barge_capacity::BargeCapacityInputs;
use ovcd_core::gas_arrangement::FitOptions;
use ovcd_core::gas_stability::DEFAULT_WATER_DENSITY;
use ovcd_core::journey_derating::JourneyDeratingInputs;
use ovcd_core::resistance::EnvironmentInputs;
use ovcd_format::vdesign::{INPUTS, LEGACY_FIXED_INPUTS};
use ovcd_format::{DesignSession, HashStatus, Profile, RunOutcome, Source, check};
use serde_json::{Value, json};

const NEW: [&str; 2] = ["water_density_t_per_m3", "squat_safety_margin_m"];

fn raw_fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// A fixture as a file written before these inputs existed: the two keys taken out.
fn fixture(name: &str) -> String {
    let mut file: Value = serde_json::from_str(&raw_fixture(name)).unwrap();
    for option in file["options"].as_array_mut().unwrap() {
        let inputs = option["inputs"].as_object_mut().unwrap();
        for key in NEW {
            inputs.remove(key);
        }
    }
    file.to_string()
}

#[test]
fn each_default_equals_the_legacy_table_value_and_every_core_default() {
    let profile = Profile::built_in();
    let cap = BargeCapacityInputs::new(90.0, 14.0, 6.0);
    let fit = FitOptions::default();
    let probe = BargeCapacityInputs::new(90.0, 14.0, 6.0);
    let journey = {
        // `JourneyDeratingInputs` borrows a result; the density default does not depend on it.
        let r = ovcd_core::barge_capacity::compute(&probe);
        let r = r.ok().or_else(|| {
            let mut dry = BargeCapacityInputs::new(70.0, 11.0, 4.0);
            dry.cargo_type = "bauxite".into();
            dry.hold_type = "box".into();
            dry.lightship_k_t_per_m3 = Some(0.11);
            ovcd_core::barge_capacity::compute(&dry).ok()
        });
        JourneyDeratingInputs::new(&r.unwrap(), 90.0, 14.0, 6.0).water_density
    };
    let squat = EnvironmentInputs::default().squat_safety_margin_m;
    let want = |name: &str| {
        LEGACY_FIXED_INPUTS
            .iter()
            .find(|l| l.name == name)
            .unwrap_or_else(|| panic!("{name} is not in the legacy table"))
            .value
    };
    // Water density: the profile, the table and all four places the engine held it.
    let rho = profile
        .default("water_density_t_per_m3")
        .and_then(Value::as_f64)
        .unwrap();
    assert_eq!(rho, want("water_density_t_per_m3"));
    assert_eq!(rho, cap.water_density, "capacity inputs");
    assert_eq!(rho, DEFAULT_WATER_DENSITY, "stability default");
    assert_eq!(rho, journey, "journey de-rating");
    assert_eq!(rho, fit.water_density_t_per_m3, "gas search");
    // Squat margin.
    let margin = profile
        .default("squat_safety_margin_m")
        .and_then(Value::as_f64)
        .unwrap();
    assert_eq!(margin, want("squat_safety_margin_m"));
    assert_eq!(margin, squat, "resistance default");
    assert_eq!(
        margin,
        ovcd_core::squat::DEFAULT_SAFETY_MARGIN_M,
        "squat default"
    );
}

#[test]
fn the_inputs_apply_to_every_hull_in_the_waterway_group() {
    for name in NEW {
        let spec = INPUTS.iter().find(|s| s.name == name).unwrap();
        assert_eq!(spec.group.label(), "Waterway Limits", "{name}");
        assert_eq!(spec.applies_to.containments().len(), 3, "{name}");
        assert!(spec.formations.is_none(), "{name}");
    }
}

#[test]
fn old_files_get_the_fixed_values_keep_their_hashes_and_note_it_once() {
    for file in [
        "dry_bulk_box_hold",
        "dry_bulk_sloped_hopper",
        "gas_igc_2g_exact",
        "gas_igc_2g_self_propelled",
    ] {
        let text = fixture(file);
        let checked = check(&text, &Profile::built_in());
        assert_eq!(checked.report.errors, Vec::<String>::new(), "{file}");
        let design = checked.design.unwrap();
        for name in NEW {
            let stored = design.options[0].input(name).unwrap();
            assert_eq!(stored.source, Source::Default, "{file}: {name}");
        }
        let (_, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
        assert_eq!(opened.hash_status[0].1, HashStatus::Matches, "{file}");
        let notes = opened
            .report
            .warnings
            .iter()
            .filter(|w| w.contains("fixed value the engine used before it was an input"))
            .count();
        assert_eq!(notes, 1, "{file}: {:?}", opened.report.warnings);
    }
}

fn results(file: &str, changes: &[(&str, Value)]) -> Value {
    let mut f: Value = serde_json::from_str(&raw_fixture(file)).unwrap();
    for (name, value) in changes {
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

fn n(v: &Value, a: &str, b: &str) -> f64 {
    v[a][b].as_f64().unwrap_or_else(|| panic!("no {a}.{b}"))
}

fn close(a: f64, b: f64, tol: f64, what: &str) {
    assert!(
        (a - b).abs() <= tol * b.abs().max(1.0),
        "{what}: {a} vs {b}"
    );
}

const RHO: f64 = 1.025;

#[test]
fn a_dry_bulk_run_uses_the_one_density_everywhere() {
    let one = results("dry_bulk_box_hold", &[]);
    let sea = results(
        "dry_bulk_box_hold",
        &[("water_density_t_per_m3", json!(RHO))],
    );
    let dims = |v: &Value| {
        (
            n(v, "dims", "loa_m"),
            n(v, "dims", "beam_m"),
            n(v, "dims", "cb"),
        )
    };
    let (l, b, cb) = dims(&sea);
    // Draught bound (this hull): the cargo is what floats to the draught limit at this density.
    assert!(
        sea["results"]["governs"]
            .as_str()
            .unwrap()
            .starts_with("draught")
    );
    let t_max = n(&sea, "results", "draught_max_m");
    close(
        n(&sea, "results", "cargo_t") + n(&sea, "results", "lightship_t"),
        cb * l * b * t_max * RHO,
        1e-12,
        "cargo floats to the limit",
    );
    close(
        n(&sea, "results", "draught_m"),
        t_max,
        1e-12,
        "draught at the limit",
    );
    close(
        n(&sea, "results", "cargo_weight_bound_t") + n(&sea, "results", "lightship_t"),
        cb * l * b * t_max * RHO,
        1e-12,
        "weight bound",
    );
    // Volume bound (a part-filled hold): the displacement is lightship plus cargo whatever the
    // density, so the draught times the density does not change.
    let part = [("hold_fill_ratio", json!(0.3))];
    let v1 = results("dry_bulk_box_hold", &part);
    let v2 = results(
        "dry_bulk_box_hold",
        &[part[0].clone(), ("water_density_t_per_m3", json!(RHO))],
    );
    assert_eq!(v2["results"]["governs"], "hold volume");
    close(
        n(&v2, "results", "draught_m") * RHO,
        n(&v1, "results", "draught_m"),
        1e-12,
        "laden draught x density",
    );
    // The capacity core (light draught by bisection, to 0.001 m) reads the same density.
    let tol = 3e-3;
    assert!(
        (n(&sea, "results", "light_draught_m") * RHO - n(&one, "results", "light_draught_m")).abs()
            < tol,
        "light draught"
    );
    // The resistance curves read it too: the deep-water frictional resistance depends on it.
    assert_ne!(sea["curves"], one["curves"]);
}

#[test]
fn a_gas_run_uses_the_one_density_in_the_capacity_the_journey_and_the_stability() {
    let file = "gas_igc_2g_self_propelled";
    let one = results(file, &[]);
    let sea = results(file, &[("water_density_t_per_m3", json!(RHO))]);
    let (l, b, cb) = (
        n(&sea, "dims", "loa_m"),
        n(&sea, "dims", "beam_m"),
        n(&sea, "dims", "cb"),
    );
    // Capacity core: the laden draught is where lightship plus cargo floats, with this density.
    close(
        n(&sea, "results", "draught_m") * RHO,
        n(&sea, "results", "displacement_t") / (cb * l * b),
        3e-3,
        "laden draught",
    );
    // Stability: each condition's draught is its displacement over cb L B and the same density.
    let conditions = sea["stability"]["conditions"].as_array().unwrap();
    assert!(!conditions.is_empty());
    for c in conditions {
        close(
            c["draught_m"].as_f64().unwrap() * RHO,
            c["displacement_t"].as_f64().unwrap() / (cb * l * b),
            1e-12,
            "stability draught",
        );
    }
    let before = one["stability"]["conditions"].as_array().unwrap();
    assert_ne!(before[0]["draught_m"], conditions[0]["draught_m"]);
    // Journey de-rating: with a draught limit that binds, the cargo is what floats to the limit.
    let limited = results(
        file,
        &[
            ("water_density_t_per_m3", json!(RHO)),
            ("draught_limit_m", json!(2.2)),
        ],
    );
    assert!(
        limited["results"]["governs"]
            .as_str()
            .unwrap()
            .starts_with("draught"),
        "{}",
        limited["results"]["governs"]
    );
    close(
        n(&limited, "results", "cargo_t") + n(&limited, "results", "lightship_t"),
        cb * l * b * 2.2 * RHO,
        1e-9,
        "journey cargo",
    );
}

#[test]
fn a_larger_squat_margin_flags_at_least_as_many_speeds() {
    let flagged = |m: f64| {
        let r = results(
            "dry_bulk_box_hold",
            &[
                ("channel_depth_m", json!(4.0)),
                ("channel_bottom_width_m", json!(30.0)),
                ("squat_safety_margin_m", json!(m)),
            ],
        );
        let channel = r["curves"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["name"] == "Channel")
            .unwrap()
            .clone();
        channel["points"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p["squat_limited"] == true)
            .count()
    };
    let (small, default, large) = (flagged(0.05), flagged(0.15), flagged(0.6));
    assert!(
        small <= default && default <= large,
        "{small} {default} {large}"
    );
    assert!(small < large, "{small} {large}");
}

#[test]
fn the_ranges_refuse_what_the_engine_cannot_compute() {
    for (name, bad) in [
        ("water_density_t_per_m3", json!(0.0)),
        ("water_density_t_per_m3", json!(2.0)),
        ("squat_safety_margin_m", json!(-0.1)),
    ] {
        let mut f: Value = serde_json::from_str(&raw_fixture("dry_bulk_box_hold")).unwrap();
        f["options"][0]["inputs"][name] = json!({ "value": bad, "source": "typed" });
        let checked = check(&f.to_string(), &Profile::built_in());
        assert!(
            checked.report.errors.iter().any(|e| e.contains(name)),
            "{name}: {:?}",
            checked.report.errors
        );
    }
}
