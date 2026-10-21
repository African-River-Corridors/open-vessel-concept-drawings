//! Seam: the Propulsor factors as inputs (issue #351): `propulsor_efficiency_factor` (kappa) and
//! `bollard_pull_factor` (kappa_b), held only with a specified Propulsor type, their defaults by the
//! type; and the `propulsor` entry point of the eval seam, which the calibration calls.
//!
//! Label: property.

use ovcd_format::{DesignSession, Profile, Source, eval, input_schema, parse_case};
use serde_json::{Value, json};

fn input(s: &DesignSession, name: &str) -> Option<(Value, Source)> {
    s.design().options[0]
        .input(name)
        .map(|i| (i.value.clone(), i.source))
}

#[test]
fn a_new_option_takes_the_factors_of_its_propulsor_type() {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    s.set_input("Option 1", "cargo", json!("LNG")).unwrap();
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    // A gas hull's default Propulsor: ducted azimuth units, kappa_b fitted on the ASD tugs.
    assert_eq!(
        input(&s, "propulsor_type").unwrap().0,
        json!("azimuth_ducted")
    );
    assert_eq!(
        input(&s, "bollard_pull_factor"),
        Some((json!(0.975), Source::Default))
    );
    assert_eq!(
        input(&s, "propulsor_efficiency_factor"),
        Some((json!(0.8), Source::Default))
    );
    s.set_input("Option 1", "propulsor_type", json!("shaft_nozzle"))
        .unwrap();
    assert_eq!(input(&s, "bollard_pull_factor").unwrap().0, json!(0.951));
    // With an unspecified Propulsor neither factor is held.
    s.set_input("Option 1", "propulsor_type", json!("unspecified"))
        .unwrap();
    assert!(input(&s, "bollard_pull_factor").is_none());
    assert!(input(&s, "propulsor_efficiency_factor").is_none());
}

#[test]
fn the_schema_shows_the_factors_as_estimates_held_with_a_specified_propulsor() {
    let schema = input_schema(&Profile::built_in());
    let find = |n: &str| {
        schema["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["name"] == n)
            .cloned()
            .unwrap_or_else(|| panic!("{n}"))
    };
    for name in ["propulsor_efficiency_factor", "bollard_pull_factor"] {
        let i = find(name);
        assert_eq!(i["default_status"], "estimate", "{i}");
        assert_eq!(i["applies_when"]["input"], "propulsor_type", "{i}");
    }
}

#[test]
fn the_eval_seam_gives_the_bollard_pull_and_the_blade_load() {
    let case = |f: &str| {
        parse_case(
            &json!({ "fn": f, "args": {
                "inputs": { "type": "shaft_open", "count": 1, "diameter_m": 2.0, "kappa_b": 0.77 },
                "pd_kw": 1000.0
            }})
            .to_string(),
        )
        .unwrap()
    };
    let bp = eval("propulsor", &case("bollard_pull")).unwrap()["ok"]
        .as_f64()
        .unwrap();
    let wartsila = 0.9 * libm::pow(2.0 * 1000.0, 2.0 / 3.0) / 9.81;
    assert!(((bp - wartsila) / wartsila).abs() < 0.005, "{bp}");
    let load = eval("propulsor", &case("blade_load")).unwrap();
    assert_eq!(load["ok"]["limit_kw_m2"], 400.0);
    let bad = parse_case(
        &json!({ "fn": "bollard_pull", "args": {
            "inputs": { "type": "unspecified", "count": 1, "diameter_m": 2.0 }, "pd_kw": 1.0
        }})
        .to_string(),
    )
    .unwrap();
    assert!(eval("propulsor", &bad).is_err());
}

/// `calibration/factors.yaml`, the block `name`: (type, value) in the order written.
fn fitted_factors(name: &str) -> Vec<(String, f64)> {
    let path = format!(
        "{}/../../calibration/factors.yaml",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).unwrap();
    let block = text.split(&format!("\n{name}:\n")).nth(1).unwrap();
    let mut out = Vec::new();
    let mut current = None;
    for line in block.lines() {
        if !line.starts_with(' ') {
            break;
        }
        if let Some(t) = line.strip_prefix("  ").and_then(|l| l.strip_suffix(':')) {
            if !t.starts_with(' ') {
                current = Some(t.to_string());
            }
        } else if let (Some(t), Some(v)) = (&current, line.strip_prefix("    value: ")) {
            out.push((t.clone(), v.trim().parse().unwrap()));
        }
    }
    out
}

#[test]
fn the_profile_takes_the_fitted_factors() {
    let profile: Value = serde_json::from_str(Profile::built_in_text()).unwrap();
    // kappa_b from the bollard pull cases (#351), kappa from the installed power cases (#353).
    for name in ["bollard_pull_factor", "propulsor_efficiency_factor"] {
        let fitted = fitted_factors(name);
        assert_eq!(fitted.len(), 6, "{name}: {fitted:?}");
        let entry = &profile["inputs"][name];
        assert_eq!(entry["status"], "estimate");
        for (t, v) in fitted {
            assert_eq!(
                entry["default_by_propulsor_type"][&t],
                json!(v),
                "{name} {t}"
            );
        }
    }
}
