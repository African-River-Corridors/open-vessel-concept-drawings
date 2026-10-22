//! The ATB tug and its powering (issue #362, spec 0007 "Formations"): the tug sized from the barge,
//! the Notch from the tug, the Equivalent Hull of the barge and the tug less the Notch, the power
//! and speed in each environment.
//!
//! Label: property - the ratios and the Equivalent Hull length are the spec's.

use ovcd_format::{DesignSession, Profile, RunOutcome};
use serde_json::{Value, json};

fn raw(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The results of the convoy fixture run as `kind` (an ATB or a pushed barge) with `changes` typed.
fn results(kind: &str, changes: &[(&str, Value)]) -> Value {
    let mut f: Value = serde_json::from_str(&raw("pushed_convoy")).unwrap();
    let inputs = &mut f["options"][0]["inputs"];
    inputs["formation"] = json!({ "value": { "kind": kind }, "source": "typed" });
    inputs["tug_included"] = json!({ "value": true, "source": "typed" });
    if kind == "atb" {
        for (name, value) in [
            ("tug_stern_form", json!("transom")),
            ("tug_propulsor_type", json!("azimuth_ducted")),
            ("formation_resistance_allowance", json!(1.05)),
            ("tug_design_speed_kn", json!(11.0)),
        ] {
            inputs[name] = json!({ "value": value, "source": "typed" });
        }
    }
    for (name, value) in changes {
        inputs[*name] = json!({ "value": value, "source": "typed" });
    }
    f["options"][0]
        .as_object_mut()
        .unwrap()
        .remove("results_hash");
    let (session, _) = DesignSession::open(&f.to_string(), Profile::built_in()).unwrap();
    match session.evaluate().options[0].outcome.as_ref().unwrap() {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

fn num(v: &Value, path: &[&str]) -> f64 {
    let mut cur = v;
    for key in path {
        cur = &cur[*key];
    }
    cur.as_f64().unwrap_or_else(|| panic!("{path:?} in {cur}"))
}

fn close(a: f64, b: f64, what: &str) {
    assert!(
        (a - b).abs() <= 1e-9 * b.abs().max(1.0),
        "{what}: {a} vs {b}"
    );
}

#[test]
fn the_atb_tug_comes_from_the_barge_and_the_equivalent_hull_is_shorter_by_the_notch() {
    let r = results("atb", &[]);
    let f = &r["formation"];
    let (l, b) = (num(&r, &["dims", "loa_m"]), num(&r, &["dims", "beam_m"]));
    close(num(f, &["tug", "length_m"]), 0.28 * l, "tug length");
    close(num(f, &["tug", "beam_m"]), 0.53 * b, "tug beam");
    close(
        num(f, &["tug", "draught_m"]),
        0.53 * b / 2.33,
        "tug draught",
    );
    // The Notch: half the tug's length deep.
    close(num(f, &["notch_depth_m"]), 0.5 * 0.28 * l, "notch depth");
    close(
        num(f, &["equivalent_hull", "length_m"]),
        l + 0.28 * l - 0.5 * 0.28 * l,
        "Equivalent Hull length",
    );
    close(
        num(f, &["equivalent_hull", "beam_m"]),
        b,
        "Equivalent Hull beam",
    );
    close(
        num(f, &["equivalent_hull", "draught_m"]),
        num(&r, &["results", "draught_m"]),
        "Equivalent Hull draught",
    );
    close(num(f, &["resistance_allowance"]), 1.05, "allowance");
    assert!(num(f, &["tug", "installed_power_kw"]) > 0.0);
    assert!(
        f.get("coupling_force_kn").is_none(),
        "an ATB has no coupling force"
    );
    for e in f["environments"].as_array().unwrap() {
        assert!(e["service_speed_kn"].as_f64().is_some(), "{e}");
    }
}

#[test]
fn a_typed_tug_and_notch_are_kept() {
    let r = results(
        "atb",
        &[
            ("tug_length_m", json!(40.0)),
            ("tug_beam_m", json!(5.0)),
            ("notch_depth_m", json!(15.0)),
        ],
    );
    let f = &r["formation"];
    let l = num(&r, &["dims", "loa_m"]);
    close(num(f, &["tug", "length_m"]), 40.0, "tug length");
    close(num(f, &["tug", "beam_m"]), 5.0, "tug beam");
    close(
        num(f, &["tug", "draught_m"]),
        5.0 / 2.33,
        "draught follows the beam",
    );
    close(num(f, &["notch_depth_m"]), 15.0, "notch depth");
    close(
        num(f, &["equivalent_hull", "length_m"]),
        l + 40.0 - 15.0,
        "length",
    );
}

#[test]
fn a_larger_allowance_needs_more_power_and_a_pushed_formation_is_unchanged() {
    let one = results("atb", &[("formation_resistance_allowance", json!(1.0))]);
    let more = results("atb", &[("formation_resistance_allowance", json!(1.2))]);
    assert!(
        num(&more["formation"], &["tug", "installed_power_kw"])
            > num(&one["formation"], &["tug", "installed_power_kw"])
    );
    // A pushed barge keeps its block: a coupling force, no Notch.
    let pushed = results("pushed_barge", &[]);
    assert!(pushed["formation"].get("coupling_force_kn").is_some());
    assert!(pushed["formation"].get("notch_depth_m").is_none());
}

/// A reference ATB: name, barge length, beam and depth, the tug's typed inputs, the speed (kn) and
/// the published installed power (kW).
type Reference = (
    &'static str,
    f64,
    f64,
    f64,
    Vec<(&'static str, Value)>,
    f64,
    f64,
);

/// The three reference ATBs of `calibration/atb.yaml`: name, barge length, beam and depth, the tug's
/// typed dimensions and propulsor, the speed and the published installed power in kW (1 hp = 0.7457 kW).
#[test]
fn the_atb_powering_against_the_reference_vessels() {
    // Label: snapshot - the barge depths are estimates (the sheets give none), so the band is wide.
    // The model needs the still-water power in deep water at the service speed, with the tug's
    // margin; the published installed power also covers shallow water, seaway, bollard pull and the
    // owner's margin. The ratio the model gives is 0.29 to 0.43 of the published power: a band of 0.2
    // to 0.6 holds if the powering drifts either way.
    let refs: [Reference; 3] = [
        (
            "Aveogan",
            121.9,
            25.9,
            7.9,
            vec![
                ("tug_length_m", json!(39.0)),
                ("tug_beam_m", json!(12.8)),
                ("tug_draught_m", json!(5.64)),
                ("tug_propulsor_type", json!("azimuth_ducted")),
                ("tug_propeller_diameter_m", json!(2.8)),
            ],
            11.0,
            6700.0 * 0.7457,
        ),
        (
            "550 Class",
            145.4,
            22.6,
            7.0,
            vec![
                ("tug_length_m", json!(37.1)),
                ("tug_beam_m", json!(12.8)),
                ("tug_propulsor_type", json!("shaft_open")),
            ],
            12.0,
            9280.0 * 0.7457,
        ),
        (
            "Kim M. Bouchard",
            191.4,
            27.6,
            9.0,
            vec![
                ("tug_length_m", json!(45.7)),
                ("tug_beam_m", json!(15.2)),
                ("tug_propulsor_type", json!("shaft_nozzle")),
                ("tug_propeller_diameter_m", json!(3.76)),
            ],
            12.0,
            10000.0 * 0.7457,
        ),
    ];
    for (name, l, b, d, tug, speed, published_kw) in refs {
        let mut changes = vec![
            ("length_overall_m", json!(l)),
            ("beam_m", json!(b)),
            ("moulded_depth_m", json!(d)),
            ("tug_design_speed_kn", json!(speed)),
            ("tug_power_source", json!("diesel_mechanical")),
            ("tug_propulsor_count", json!(2)),
        ];
        changes.extend(tug);
        let r = results("atb", &changes);
        let f = &r["formation"];
        assert!(f.get("error").is_none(), "{name}: {f}");
        let ratio = num(f, &["tug", "installed_power_kw"]) / published_kw;
        assert!((0.2..=0.6).contains(&ratio), "{name}: ratio {ratio}");
    }
}
