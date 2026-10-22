//! Seam: the angle of down-flooding (issue #282). IS Code 2008 Part A 2.2.1 (MSC.267(85), annex p.
//! 11): the area under the GZ curve is at least 0.09 m.rad up to 40 degrees "or the angle of
//! down-flooding if this angle is less than 40 degrees", and the area between 30 degrees and 40 degrees
//! (or the angle) at least 0.03 m.rad. The designer types the angle (`downflooding_angle_deg`);
//! with none the areas run to 40 degrees and the Rule Trace says so.
//!
//! Label: sourced - IS Code 2008 Part A 2.2.1 (the criteria and their ends); the rest is property.

use ovcd_core::box_stability::{area, area_end_deg, area_to};
use ovcd_format::vdesign::{INPUTS, LEGACY_FIXED_OTHER};
use ovcd_format::{DesignSession, HashStatus, Profile, RunOutcome, check, run_design};
use serde_json::{Value, json};

fn raw_fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// A new gas Option (the default Rule Set stack) with the angle of down-flooding set (or `null`),
/// and no stored hash.
fn with_angle(angle: Value) -> Value {
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
    let filled = ovcd_format::fill_as_new(&raw.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    let mut file: Value = serde_json::from_str(&filled.design.unwrap().to_text()).unwrap();
    let option = &mut file["options"][0];
    option["inputs"]["downflooding_angle_deg"] = json!({ "value": angle, "source": "typed" });
    option.as_object_mut().unwrap().remove("results_hash");
    file
}

fn results_of(file: &Value) -> Value {
    let design = check(&file.to_string(), &Profile::built_in())
        .design
        .unwrap();
    match run_design(&design).remove(0).outcome {
        RunOutcome::Results { results, .. } => results,
        other => panic!("{other:?}"),
    }
}

fn criterion(r: &Value, starts: &str) -> Value {
    r["stability"]["conditions"][0]["criteria"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"].as_str().unwrap().starts_with(starts))
        .cloned()
        .unwrap_or_else(|| panic!("{starts}: {}", r["stability"]["conditions"][0]["criteria"]))
}

#[test]
fn a_curve_is_integrated_to_a_fractional_angle_with_gz_interpolated_between_the_degrees() {
    // GZ = 0.01 m per degree, a straight line: the trapezoid is exact, k (deg^2 / 2) pi / 180.
    let gz: Vec<(i32, f64)> = (0..=60).map(|d| (d, 0.01 * f64::from(d))).collect();
    let exact =
        |a0: f64, a1: f64| 0.01 * (a1 * a1 - a0 * a0) / 2.0 * (std::f64::consts::PI / 180.0);
    assert!((area_to(&gz, 0, 35.5) - exact(0.0, 35.5)).abs() < 1e-12);
    assert!((area_to(&gz, 30, 37.25) - exact(30.0, 37.25)).abs() < 1e-12);
    // A whole angle is the plain area.
    assert!((area_to(&gz, 0, 40.0) - area(&gz, 0, 40)).abs() < 1e-15);
    // The end: 40 degrees, or the angle where it is less.
    assert_eq!(area_end_deg(None), 40.0);
    assert_eq!(area_end_deg(Some(55.0)), 40.0);
    assert_eq!(area_end_deg(Some(32.5)), 32.5);
}

#[test]
fn the_input_is_optional_for_every_option_and_an_old_file_reads_none() {
    let spec = INPUTS
        .iter()
        .find(|s| s.name == "downflooding_angle_deg")
        .unwrap();
    assert_eq!(spec.group.label(), "Loading conditions");
    assert!(spec.applies_to.containments().is_empty() || spec.applies_to.containments().len() > 1);
    assert_eq!(
        Profile::built_in().default("downflooding_angle_deg"),
        Some(&Value::Null)
    );
    let legacy = LEGACY_FIXED_OTHER
        .iter()
        .find(|l| l.name == "downflooding_angle_deg")
        .unwrap();
    assert_eq!(legacy.value, "null");
    // The stored fixture holds none and keeps its hash.
    let (_, opened) =
        DesignSession::open(&raw_fixture("gas_igc_2g_exact"), Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
}

#[test]
fn with_no_angle_the_areas_run_to_40_and_the_trace_says_the_angle_is_not_given() {
    let r = results_of(&with_angle(Value::Null));
    assert_eq!(
        criterion(&r, "Area 0")
            .get("name")
            .unwrap()
            .as_str()
            .is_some(),
        true
    );
    assert!(criterion(&r, "Area 30").is_object());
    let names: Vec<String> = r["stability"]["conditions"][0]["criteria"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap().to_string())
        .collect();
    assert!(
        names.contains(&"Area 0\u{2013}40\u{b0}".to_string()),
        "{names:?}"
    );
    let rows = r["rule_trace"]["rows"].as_array().unwrap();
    let note: Vec<&Value> = rows
        .iter()
        .filter(|row| row["rule"] == "is2008/A.2.2.1/downflooding-angle-not-given")
        .collect();
    assert!(!note.is_empty(), "the note row");
    assert!(
        note.iter().all(|row| row["outcome"] == "not evaluated"),
        "{note:?}"
    );
}

#[test]
fn a_typed_angle_below_40_ends_the_areas_there_and_the_note_goes() {
    let none = results_of(&with_angle(Value::Null));
    let typed = results_of(&with_angle(json!(35.0)));
    let area_of = |r: &Value, starts: &str| criterion(r, starts)["value"].as_f64().unwrap();
    assert_eq!(
        criterion(&typed, "Area 0\u{2013}35")["name"],
        "Area 0\u{2013}35\u{b0}"
    );
    assert_eq!(
        criterion(&typed, "Area 30\u{2013}35")["name"],
        "Area 30\u{2013}35\u{b0}"
    );
    // The areas are smaller: they end 5 degrees sooner, the curve being positive there.
    assert!(area_of(&typed, "Area 0\u{2013}35") < area_of(&none, "Area 0\u{2013}40"));
    assert!(area_of(&typed, "Area 30\u{2013}35") < area_of(&none, "Area 30\u{2013}40"));
    // The Rules read the same angle.
    let rows = typed["rule_trace"]["rows"].as_array().unwrap();
    let row = rows
        .iter()
        .find(|r| {
            r["rule"] == "is2008/A.2.2.1/area-to-40-or-downflooding"
                && r["outcome"] != "not applicable"
        })
        .unwrap();
    let cond0 = typed["stability"]["conditions"][0]["name"]
        .as_str()
        .unwrap();
    let _ = cond0;
    assert!((row["value"].as_f64().unwrap() - area_of(&typed, "Area 0\u{2013}35")).abs() < 1e-9);
    // With an angle typed the note row is not applicable.
    assert!(
        rows.iter()
            .filter(|r| r["rule"] == "is2008/A.2.2.1/downflooding-angle-not-given")
            .all(|r| r["outcome"] == "not applicable")
    );
}

#[test]
fn an_angle_of_30_or_less_leaves_no_area_between_30_degrees_and_the_angle() {
    let r = results_of(&with_angle(json!(28.0)));
    let c = criterion(&r, "Area 30\u{2013}28");
    assert_eq!(c["value"].as_f64(), Some(0.0));
    assert_eq!(c["ok"], false);
}
