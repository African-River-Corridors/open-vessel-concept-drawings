//! Seam: the end-bulkhead space aft and forward of the gas tanks is an input (issue #280). A new
//! Option takes the type G rule from the Defaults Profile, 0.20 m for domed tank ends (Lloyd's
//! Register inland waterways rules Pt 4 Ch 4 3.5.3; ADN 2025 9.3.1.11.3(a)). A file that stores none
//! keeps the 0.60 m aft and 0.50 m forward the engine used before, so its results do not move.
//!
//! Label: sourced - the 0.20 m and 0.50 m are the cited clause values; the rest is property.

use ovd_format::vdesign::{INPUTS, LEGACY_FIXED_INPUTS};
use ovd_format::{
    DesignSession, HashStatus, Profile, RunOutcome, Source, check, fill_as_new, run_design,
};
use serde_json::{Value, json};

const NAMES: [&str; 2] = ["aft_end_space_m", "fwd_end_space_m"];

fn raw_fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// A fixture as a file written before the two inputs existed.
fn old_fixture(name: &str) -> String {
    let mut file: Value = serde_json::from_str(&raw_fixture(name)).unwrap();
    for option in file["options"].as_array_mut().unwrap() {
        for key in NAMES {
            option["inputs"].as_object_mut().unwrap().remove(key);
        }
    }
    file.to_string()
}

fn cargo_of(text: &str) -> f64 {
    let design = check(text, &Profile::built_in()).design.unwrap();
    match run_design(&design).remove(0).outcome {
        RunOutcome::Results { results, .. } => results["results"]["cargo_t"].as_f64().unwrap(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_inputs_are_gas_only_in_the_gas_tanks_group() {
    for name in NAMES {
        let spec = INPUTS.iter().find(|s| s.name == name).unwrap();
        assert_eq!(spec.group.label(), "Gas tanks", "{name}");
        assert_eq!(spec.applies_to.containments(), ["type_c_tanks"], "{name}");
    }
}

#[test]
fn a_new_option_takes_the_type_g_rule_and_an_old_file_keeps_what_the_engine_used() {
    let profile = Profile::built_in();
    for name in NAMES {
        assert_eq!(
            profile.default(name).and_then(Value::as_f64),
            Some(0.20),
            "{name}"
        );
    }
    let legacy = |name: &str| {
        LEGACY_FIXED_INPUTS
            .iter()
            .find(|l| l.name == name)
            .unwrap()
            .value
    };
    assert_eq!(legacy("aft_end_space_m"), 0.60);
    assert_eq!(legacy("fwd_end_space_m"), 0.50);
    // A new gas Option holds the profile's value, marked default.
    let raw = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": {
            "cargo": { "value": "LNG", "source": "typed" },
            "hull_form": { "value": "raked_box", "source": "typed" },
            "formation": { "value": { "kind": "pushed_barge" }, "source": "typed" },
            "length_overall_m": { "value": 90.0, "source": "typed" },
            "beam_m": { "value": 14.8, "source": "typed" },
            "moulded_depth_m": { "value": 6.5, "source": "typed" }
        }}]
    });
    let filled = fill_as_new(&raw.to_string(), &profile);
    assert_eq!(filled.report.errors, Vec::<String>::new());
    let design = filled.design.unwrap();
    for name in NAMES {
        let stored = design.options[0].input(name).unwrap();
        assert_eq!(stored.value.as_f64(), Some(0.20), "{name}");
        assert_eq!(stored.source, Source::Default, "{name}");
    }
}

#[test]
fn a_gas_file_without_the_inputs_holds_the_old_values_with_one_note_and_the_same_hash() {
    for file in ["gas_igc_2g_exact", "gas_igc_2g_self_propelled"] {
        let text = old_fixture(file);
        let (_, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
        assert_eq!(opened.hash_status[0].1, HashStatus::Matches, "{file}");
        let notes = opened
            .report
            .warnings
            .iter()
            .filter(|w| w.contains("`aft_end_space_m`, `fwd_end_space_m`"))
            .count();
        assert_eq!(notes, 1, "{file}: {:?}", opened.report.warnings);
        let design = check(&text, &Profile::built_in()).design.unwrap();
        assert_eq!(
            design.options[0].input("aft_end_space_m").unwrap().value,
            json!(0.60)
        );
        assert_eq!(
            design.options[0].input("fwd_end_space_m").unwrap().value,
            json!(0.50)
        );
    }
}

#[test]
fn a_smaller_end_space_carries_more_cargo() {
    // The type G rule takes 0.60 + 0.50 m less than the old figures only where the end space, not the
    // 2.00 m opening setback, sets the length taken: at the forward end. Issue #740: on a hull whose
    // base arrangement passes every check (the 76.5 m pushed barge of `gas_igc_2g_exact` has none
    // since the rule length of ABS 3-1-1/3.1 lightened its hull, so its search shows the closest
    // one), the self-propelled gas fixture.
    let mut file: Value = serde_json::from_str(&raw_fixture("gas_igc_2g_self_propelled")).unwrap();
    let base = |f: &Value| {
        let mut f = f.clone();
        f["options"][0]
            .as_object_mut()
            .unwrap()
            .remove("results_hash");
        f
    };
    let old = cargo_of(&base(&file).to_string());
    file["options"][0]["inputs"]["fwd_end_space_m"] = json!({ "value": 0.20, "source": "typed" });
    let new = cargo_of(&base(&file).to_string());
    assert!(new > old, "{new} vs {old}");
}
