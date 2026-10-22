//! Seam: the angle of down-flooding worked out from the openings of a Design (issue #592). IS Code
//! 2008 Part A 2.2.1 (MSC.267(85), annex p. 11) ends the areas to 40 degrees at the angle of
//! down-flooding where it is less; Part B 2.2.3.5 says what an opening is. The designer lists the
//! deck openings (`downflooding_openings`); the engine works the angle of each loading condition
//! from the equilibrium; a typed angle wins.
//!
//! Label: property. Each test states a rule; the closed form of the angle is in
//! `crates/ovcd-core/tests/downflooding_openings.rs`.

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

/// A new gas Option, filled once.
fn base() -> &'static Value {
    static BASE: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
    BASE.get_or_init(|| {
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
        serde_json::from_str(&filled.design.unwrap().to_text()).unwrap()
    })
}

/// The Option with an angle and a list of openings set (each `null` for none), no hash.
fn with(angle: Value, openings: Value) -> Value {
    let mut file = base().clone();
    let option = &mut file["options"][0];
    option["inputs"]["downflooding_angle_deg"] = json!({ "value": angle, "source": "typed" });
    option["inputs"]["downflooding_openings"] = json!({ "value": openings, "source": "typed" });
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

fn names_of(r: &Value, condition: usize) -> Vec<String> {
    r["stability"]["conditions"][condition]["criteria"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap().to_string())
        .collect()
}

fn opening(x: f64, y: f64, h: f64) -> Value {
    json!({ "x_m": x, "y_m": y, "height_above_deck_m": h })
}

#[test]
fn the_input_is_optional_and_an_old_file_lists_none_and_keeps_its_hash() {
    let spec = INPUTS
        .iter()
        .find(|s| s.name == "downflooding_openings")
        .unwrap();
    assert_eq!(spec.group.label(), "Loading conditions");
    assert_eq!(
        Profile::built_in().default("downflooding_openings"),
        Some(&Value::Null)
    );
    let legacy = LEGACY_FIXED_OTHER
        .iter()
        .find(|l| l.name == "downflooding_openings")
        .unwrap();
    assert_eq!(legacy.value, "null");
    let (_, opened) =
        DesignSession::open(&raw_fixture("gas_igc_2g_exact"), Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
}

#[test]
fn an_opening_near_the_waterline_ends_the_areas_at_its_angle_in_each_condition() {
    // An opening at the deck edge, amidships: it is immersed at the heel where the deck edge is,
    // and that heel depends on the condition's draught.
    let r = results_of(&with(Value::Null, json!([opening(45.0, 7.4, 0.0)])));
    let mut angles = Vec::new();
    for (i, c) in r["stability"]["conditions"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let a = c["downflooding_deg"].as_f64();
        let names = names_of(&r, i);
        match a {
            Some(a) if a < 40.0 => {
                let end = if a == a.floor() {
                    format!("{a:.0}")
                } else {
                    format!("{a:.1}")
                };
                assert!(
                    names.contains(&format!("Area 0\u{2013}{end}\u{b0}")),
                    "{names:?}"
                );
            }
            _ => assert!(
                names.contains(&"Area 0\u{2013}40\u{b0}".to_string()),
                "{names:?}"
            ),
        }
        angles.push(a);
    }
    // A loaded barge has a lower deck edge than a light one: not every condition has the same angle.
    let found: Vec<f64> = angles.iter().flatten().copied().collect();
    assert!(found.len() >= 2, "{angles:?}");
    assert!(
        found.iter().any(|a| (a - found[0]).abs() > 0.5),
        "{angles:?}"
    );
}

#[test]
fn a_typed_angle_wins_over_the_openings() {
    let r = results_of(&with(json!(35.0), json!([opening(45.0, 7.4, 0.0)])));
    assert!(names_of(&r, 0).contains(&"Area 0\u{2013}35\u{b0}".to_string()));
    assert_eq!(r["stability"]["conditions"][0]["downflooding_deg"], 35.0);
}

#[test]
fn an_opening_that_never_immerses_gives_no_angle_and_the_areas_run_to_40() {
    // 20 m above the deck on the centre line: not immersed within 60 degrees.
    let r = results_of(&with(Value::Null, json!([opening(45.0, 0.0, 20.0)])));
    assert!(names_of(&r, 0).contains(&"Area 0\u{2013}40\u{b0}".to_string()));
    assert!(
        r["stability"]["conditions"][0]
            .get("downflooding_deg")
            .is_none()
    );
}

#[test]
fn with_no_openings_and_no_angle_the_results_have_no_angle_key() {
    let r = results_of(&with(Value::Null, Value::Null));
    for c in r["stability"]["conditions"].as_array().unwrap() {
        assert!(c.get("downflooding_deg").is_none());
    }
}

#[test]
fn a_listed_opening_takes_the_not_given_note_away() {
    let rows_of = |r: &Value| r["rule_trace"]["rows"].as_array().unwrap().clone();
    let listed = rows_of(&results_of(&with(
        Value::Null,
        json!([opening(45.0, 7.4, 0.0)]),
    )));
    assert!(
        listed
            .iter()
            .filter(|r| r["rule"] == "is2008/A.2.2.1/downflooding-angle-not-given")
            .all(|r| r["outcome"] == "not applicable")
    );
    let none = rows_of(&results_of(&with(Value::Null, Value::Null)));
    assert!(
        none.iter()
            .filter(|r| r["rule"] == "is2008/A.2.2.1/downflooding-angle-not-given")
            .any(|r| r["outcome"] == "not evaluated")
    );
}

#[test]
fn an_opening_outside_the_hull_or_a_bad_row_is_named() {
    let errors = |openings: Value| -> String {
        check(
            &with(Value::Null, openings).to_string(),
            &Profile::built_in(),
        )
        .report
        .errors
        .join("\n")
    };
    assert!(errors(json!([opening(95.0, 0.0, 1.0)])).contains("x_m"));
    assert!(errors(json!([opening(45.0, 9.0, 1.0)])).contains("y_m"));
    assert!(errors(json!([opening(45.0, 0.0, -1.0)])).contains("height_above_deck_m"));
    assert!(errors(json!([{ "x_m": 45.0 }])).contains("must be a list of openings"));
    assert_eq!(errors(json!([opening(45.0, 7.4, 0.5)])), "");
}
