//! Seam: the gas tank arrangement values are inputs (issue #4, slice 3): tanks in line, support
//! height, dome height, wing tank gap and height ratio, and the fixed wall thickness. They were
//! fixed in the engine; the defaults are those values, so no design moves.
//!
//! Label: property. Every test states a rule: a default equals the old constant, an old file keeps
//! its results (its stored hash still matches), an input reaches the core and the drawing object,
//! or the file refuses what it should. No number here is a printed result.

use ovd_core::barge_capacity::BargeCapacityInputs;
use ovd_core::gas_arrangement::FitOptions;
use ovd_core::models::BargeInputs;
use ovd_format::vdesign::{INPUTS, LEGACY_FIXED_INPUTS};
use ovd_format::{DesignSession, HashStatus, Profile, RunOutcome, Source, check, ga_views};
use serde_json::{Value, json};

const NEW: [&str; 8] = [
    "tanks_in_line",
    "tank_support_height_m",
    "tank_dome_height_m",
    "wing_tank_gap_m",
    "wing_tank_height_ratio",
    "tank_wall_thickness_mm",
    // Issue #410, part 1b: the weather shield.
    "weather_shield_thickness_mm",
    "weather_shield_clearance_m",
];

fn raw_fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// A fixture as a file written before these inputs existed: the six keys taken out. Its stored
/// results hash is the one the old engine gave, so a match proves the results did not move.
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

fn core_default(name: &str) -> f64 {
    let c = BargeCapacityInputs::new(90.0, 14.0, 6.0);
    let b = BargeInputs::new(90.0, 14.0, 6.0, 6.0, 60.0);
    let f = FitOptions::default();
    let (cap, fit) = match name {
        "tanks_in_line" => (c.n_tanks_long as f64, f.tanks_in_line as f64),
        "tank_support_height_m" => (c.tank_support_height, f.tank_support_height_m),
        "tank_dome_height_m" => (c.dome_height, f.tank_dome_height_m),
        "wing_tank_gap_m" => (c.wing_tank_gap_m, f.wing_tank_gap_m),
        "wing_tank_height_ratio" => (c.wing_tank_height_ratio, f.wing_tank_height_ratio),
        "weather_shield_thickness_mm" => {
            (c.weather_shield_thickness_mm, f.weather_shield_thickness_mm)
        }
        "weather_shield_clearance_m" => {
            (c.weather_shield_clearance_m, f.weather_shield_clearance_m)
        }
        "tank_wall_thickness_mm" => (
            c.tank_wall_thickness.unwrap(),
            f.tank_wall_thickness_mm.unwrap(),
        ),
        other => panic!("{other}"),
    };
    assert_eq!(cap, fit, "{name}: capacity default vs search default");
    // The two the models module also holds must say the same.
    match name {
        "wing_tank_gap_m" => assert_eq!(cap, b.wing_tank_gap),
        "wing_tank_height_ratio" => assert_eq!(cap, b.wing_tank_height_ratio),
        "tank_support_height_m" => assert_eq!(cap, b.tank_support_height),
        "tank_dome_height_m" => assert_eq!(cap, b.dome_height),
        "tanks_in_line" => assert_eq!(cap, b.n_tanks_long as f64),
        "weather_shield_thickness_mm" => assert_eq!(cap, b.weather_shield_thickness),
        "weather_shield_clearance_m" => assert_eq!(cap, b.weather_shield_clearance),
        _ => {}
    }
    cap
}

#[test]
fn each_default_equals_the_legacy_table_value_and_the_core_default() {
    let profile = Profile::built_in();
    for name in NEW {
        let legacy = LEGACY_FIXED_INPUTS
            .iter()
            .find(|l| l.name == name)
            .unwrap_or_else(|| panic!("{name} is not in the legacy table"));
        // Issue #292: the wall thickness is an optional override. The profile default is none (the
        // formula gives the thickness); a file that lacks it keeps the 25 mm the engine used.
        // Issue #316: the dome sizes are optional overrides too; none is the dome rule, and a file
        // that lacks them keeps the 2.0 m the engine used.
        if matches!(name, "tank_dome_height_m" | "tank_dome_diameter_m") {
            let row = profile
                .table_default(
                    name,
                    &[("cargo_containment", "type_c_tanks"), ("cargo", "LNG")],
                )
                .unwrap_or_else(|| panic!("{name} has a default table"));
            assert_eq!(row.value, &Value::Null, "{name}");
            assert_eq!(row.status, "estimate", "{name}");
            assert!(row.basis.contains("dome rule"), "{name}");
            assert_eq!(legacy.value, core_default(name), "{name}: table vs core");
            continue;
        }
        if name == "tank_wall_thickness_mm" {
            assert_eq!(profile.default(name), Some(&Value::Null), "{name}");
            assert_eq!(legacy.value, core_default(name), "{name}: table vs core");
            continue;
        }
        let from_profile = profile.default(name).and_then(Value::as_f64).unwrap();
        assert_eq!(from_profile, legacy.value, "{name}: profile vs table");
        assert_eq!(legacy.value, core_default(name), "{name}: table vs core");
    }
}

#[test]
fn the_inputs_are_gas_only_in_the_gas_tanks_group() {
    for name in NEW {
        let spec = INPUTS.iter().find(|s| s.name == name).unwrap();
        assert_eq!(spec.group.label(), "Gas tanks", "{name}");
        assert_eq!(spec.applies_to.containments(), ["type_c_tanks"], "{name}");
        assert!(spec.formations.is_none(), "{name}");
    }
}

#[test]
fn gas_files_without_the_inputs_open_with_one_note_and_the_same_results() {
    for file in ["gas_igc_2g_exact", "gas_igc_2g_self_propelled"] {
        let text = fixture(file);
        let checked = check(&text, &Profile::built_in());
        assert_eq!(checked.report.errors, Vec::<String>::new(), "{file}");
        let design = checked.design.unwrap();
        let o = &design.options[0];
        for name in NEW {
            let stored = o.input(name).unwrap_or_else(|| panic!("{file}: {name}"));
            assert_eq!(stored.source, Source::Default, "{name}");
            assert_eq!(stored.value.as_f64(), Some(core_default(name)), "{name}");
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

#[test]
fn a_dry_bulk_file_holds_none_of_them_and_the_stored_gas_fixtures_hold_all() {
    let dry = check(&fixture("dry_bulk_sloped_hopper"), &Profile::built_in())
        .design
        .unwrap();
    for name in NEW {
        assert!(dry.options[0].input(name).is_none(), "{name}");
    }
    let raw: Value = serde_json::from_str(&raw_fixture("gas_igc_2g_exact")).unwrap();
    // The fixture was saved before the two weather shield inputs (issue #410) existed.
    for name in &NEW[..6] {
        assert_eq!(raw["options"][0]["inputs"][name]["source"], "default");
    }
}

#[test]
fn a_whole_number_of_at_least_one_tank_in_line() {
    for (v, why) in [(json!(2.5), "whole number"), (json!(0), "must be >= 1")] {
        let mut f: Value = serde_json::from_str(&raw_fixture("gas_igc_2g_exact")).unwrap();
        f["options"][0]["inputs"]["tanks_in_line"] = json!({ "value": v, "source": "typed" });
        let checked = check(&f.to_string(), &Profile::built_in());
        assert!(
            checked.report.errors.iter().any(|e| e.contains(why)),
            "{why}: {:?}",
            checked.report.errors
        );
    }
}

#[test]
fn the_ranges_refuse_what_the_engine_cannot_compute() {
    for (name, bad) in [
        ("tank_support_height_m", json!(-0.1)),
        ("tank_dome_height_m", json!(-0.1)),
        ("wing_tank_gap_m", json!(-0.1)),
        ("wing_tank_height_ratio", json!(0)),
        ("tank_wall_thickness_mm", json!(0)),
    ] {
        let mut f: Value = serde_json::from_str(&raw_fixture("gas_igc_2g_exact")).unwrap();
        f["options"][0]["inputs"][name] = json!({ "value": bad, "source": "typed" });
        let checked = check(&f.to_string(), &Profile::built_in());
        assert!(
            checked.report.errors.iter().any(|e| e.contains(name)),
            "{name}: {:?}",
            checked.report.errors
        );
    }
}

fn results(file: &str, changes: &[(&str, Value)]) -> Value {
    let mut f: Value = serde_json::from_str(&raw_fixture(file)).unwrap();
    // These tests are about the legacy conditions; the fixtures are saved in the IS Code ones.
    f["options"][0]["inputs"]["loading_conditions"] =
        json!({ "value": "legacy", "source": "typed" });
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

fn drawing(v: &Value, key: &str) -> f64 {
    v["drawing"][key]
        .as_f64()
        .unwrap_or_else(|| panic!("no drawing.{key}"))
}

const SELF: &str = "gas_igc_2g_self_propelled";

#[test]
fn the_drawing_object_follows_each_input_and_the_ga_views_follow_the_drawing_object() {
    let base = results(SELF, &[]);
    // Support, dome and wing values come back as typed.
    let r = results(
        SELF,
        &[
            ("tank_support_height_m", json!(0.7)),
            ("tank_dome_height_m", json!(2.5)),
            ("wing_tank_gap_m", json!(0.4)),
            ("wing_tank_height_ratio", json!(0.5)),
        ],
    );
    assert_eq!(drawing(&r, "tank_support_height_m"), 0.7);
    assert_eq!(drawing(&r, "dome_height_m"), 2.5);
    assert_eq!(drawing(&r, "wing_tank_gap_m"), 0.4);
    assert_eq!(drawing(&r, "wing_tank_height_ratio"), 0.5);
    // The double bottom is the bottom clearance less the support, and not below zero.
    let cb = r["geometry"]["bottom_clearance_m"].as_f64().unwrap();
    assert_eq!(drawing(&r, "double_bottom_m"), (cb - 0.7).max(0.0));
    // The wing width is the side clearance less the gap, and not below zero. The clearance is the
    // one the IGC 2.4.1.2 rule gives the arrangement (issue #283, built in #277); until then the
    // engine used the default 1.0 m, a latent defect reported in the PR for issue #237.
    let cs = r["geometry"]["side_clearance_m"].as_f64().unwrap();
    assert_eq!(drawing(&r, "wing_tank_width_m"), (cs - 0.4).max(0.0));
    // The views are drawn from those values (ga.rs reads the drawing object), so they change.
    let (a, b) = (ga_views(&base).unwrap(), ga_views(&r).unwrap());
    assert_ne!(a.midship_section, b.midship_section);
    assert_ne!(a.side_elevation, b.side_elevation);
}

#[test]
fn the_dome_raises_the_air_draft_and_the_wall_takes_volume() {
    let base = results(SELF, &[]);
    let tall = results(SELF, &[("tank_dome_height_m", json!(6.0))]);
    let air = |v: &Value| v["results"]["air_draft_light_m"].as_f64().unwrap();
    assert!(air(&tall) > air(&base), "{} vs {}", air(&tall), air(&base));
    // Cargo volume is the inner volume: a thicker wall leaves less.
    let thick = results(SELF, &[("tank_wall_thickness_mm", json!(40.0))]);
    let vol = |v: &Value| v["results"]["cargo_volume_m3"].as_f64().unwrap();
    assert!(vol(&thick) < vol(&base));
}

#[test]
fn the_wing_ratio_and_the_support_move_the_lightship_and_the_air_draft_the_expected_way() {
    let base = results(SELF, &[]);
    let ls = |v: &Value| v["results"]["lightship_t"].as_f64().unwrap();
    // A taller wing tank raises the wing plate above D, and so the side plating up to it, a
    // lightship item since issue #410 (the hull steel stays weighed to D).
    let tall = results(SELF, &[("wing_tank_height_ratio", json!(0.9))]);
    assert!(ls(&tall) >= ls(&base), "{} vs {}", ls(&tall), ls(&base));
    // A higher support raises the tank: its bottom (the top above the keel less the diameter)
    // rises by the support's rise. The search may then fit a smaller tank (issue #448: on the
    // hull's sections it does here), so the top itself need not rise.
    let up = results(SELF, &[("tank_support_height_m", json!(1.4))]);
    let num = |v: &Value, path: [&str; 2]| v[path[0]][path[1]].as_f64().unwrap();
    let bottom = |v: &Value| {
        num(v, ["results", "air_draft_light_m"]) + num(v, ["results", "light_draught_m"])
            - num(v, ["geometry", "tank_outer_diameter_m"])
    };
    let rise = 1.4 - num(&base, ["drawing", "tank_support_height_m"]);
    assert!(rise > 0.0);
    assert!(
        (bottom(&up) - bottom(&base) - rise).abs() < 1e-9,
        "{} {}",
        bottom(&up),
        bottom(&base)
    );
}

#[test]
fn tanks_in_line_sets_the_tanks_along_the_hull_and_the_length_over_diameter_check_follows() {
    let two = results(SELF, &[]);
    assert_eq!(two["arrangement"]["n_long"], 2);
    // Three tanks along the hull: each is shorter, the count is three, and a hull that is long
    // for its diameter with one tank in line breaks the length over diameter limit.
    let three = results(SELF, &[("tanks_in_line", json!(3))]);
    assert_eq!(three["arrangement"]["n_long"], 3);
    let n_total = |v: &Value| v["geometry"]["n_tanks_total"].as_i64().unwrap();
    assert_eq!(
        n_total(&three),
        3 * three["arrangement"]["n_wide"].as_i64().unwrap()
    );
    let one = results(SELF, &[("tanks_in_line", json!(1))]);
    assert_eq!(one["arrangement"]["n_long"], 1);
    let failing = |v: &Value| {
        v["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["name"] == "tank length over diameter" && c["ok"] == false)
    };
    // The fixture hull: two tanks pass; one tank is twice as long and does not.
    assert!(!failing(&two));
    assert!(failing(&one), "{:?}", one["checks"]);
}

#[test]
fn the_weather_shield_inputs_move_the_lightship_and_its_centre() {
    // Issue #410, part 1b. The self-propelled fixture's wing plate stands above D, so its shield is
    // a lightship item. No shield weighs less; a higher shield weighs more and sits higher.
    let base = results(SELF, &[]);
    let ls = |v: &Value| v["results"]["lightship_t"].as_f64().unwrap();
    let none = results(SELF, &[("weather_shield_thickness_mm", json!(0.0))]);
    assert!(ls(&none) < ls(&base), "{} vs {}", ls(&none), ls(&base));
    let thick = results(SELF, &[("weather_shield_thickness_mm", json!(12.0))]);
    assert!(ls(&thick) > ls(&base), "{} vs {}", ls(&thick), ls(&base));
    let high = results(SELF, &[("weather_shield_clearance_m", json!(3.0))]);
    assert!(ls(&high) > ls(&base), "{} vs {}", ls(&high), ls(&base));
    // Typed at their defaults, the results are the same as leaving them out.
    let same = results(
        SELF,
        &[
            ("weather_shield_thickness_mm", json!(6.0)),
            ("weather_shield_clearance_m", json!(1.5)),
        ],
    );
    assert_eq!(same["results"], base["results"]);
}

/// Issue #412: the tank forming allowance and the dome wall. Unlike the inputs above, their
/// defaults are NOT the legacy values: the engine weighed neither, so a file saved before keeps 0
/// and its results do not move, while a new Design takes the Estimate.
const WEIGHED: [&str; 2] = ["tank_forming_allowance_pct", "tank_dome_wall_thickness_mm"];

#[test]
fn the_forming_allowance_and_dome_wall_default_to_an_estimate_and_a_saved_file_keeps_none() {
    let profile = Profile::built_in();
    let c = BargeCapacityInputs::new(90.0, 14.0, 6.0);
    let f = FitOptions::default();
    for (name, cap, fit, estimate) in [
        (
            WEIGHED[0],
            c.tank_forming_allowance_pct,
            f.tank_forming_allowance_pct,
            ovd_core::barge_capacity::DEFAULT_TANK_FORMING_ALLOWANCE_PCT,
        ),
        (
            WEIGHED[1],
            c.tank_dome_wall_thickness_mm,
            f.tank_dome_wall_thickness_mm,
            ovd_core::barge_capacity::DEFAULT_TANK_DOME_WALL_THICKNESS_MM,
        ),
    ] {
        // The legacy API weighs neither; the Defaults Profile holds the Estimate.
        assert_eq!((cap, fit), (0.0, 0.0), "{name}");
        assert_eq!(
            profile.default(name).and_then(Value::as_f64),
            Some(estimate)
        );
        assert_eq!(profile.default_status(name).map(|s| s.0), Some("estimate"));
        let legacy = LEGACY_FIXED_INPUTS.iter().find(|l| l.name == name).unwrap();
        assert_eq!(legacy.value, 0.0, "{name}");
        let spec = INPUTS.iter().find(|s| s.name == name).unwrap();
        assert_eq!(spec.group.label(), "Gas tanks", "{name}");
        assert_eq!(spec.applies_to.containments(), ["type_c_tanks"], "{name}");
    }
}

#[test]
fn a_file_saved_before_the_two_inputs_keeps_its_results_hash() {
    for file in ["gas_igc_2g_exact", "gas_igc_2g_self_propelled"] {
        let text = raw_fixture(file);
        let (_, opened) = DesignSession::open(&text, Profile::built_in()).unwrap();
        assert_eq!(opened.hash_status[0].1, HashStatus::Matches, "{file}");
    }
}

#[test]
fn the_forming_allowance_and_the_dome_wall_add_to_the_lightship() {
    // Issue #412. At 0 the tank is weighed at the required thickness and the dome has no mass; a
    // larger allowance or a thicker dome wall weighs more, and the lightship KG rises with the dome.
    let ls = |v: &Value| v["results"]["lightship_t"].as_f64().unwrap();
    let plain = results(
        SELF,
        &[
            ("tank_forming_allowance_pct", json!(0.0)),
            ("tank_dome_wall_thickness_mm", json!(0.0)),
        ],
    );
    // A file that lacks the two inputs keeps the legacy 0.
    assert_eq!(results(SELF, &[])["results"], plain["results"]);
    let base = results(
        SELF,
        &[
            ("tank_forming_allowance_pct", json!(5.0)),
            ("tank_dome_wall_thickness_mm", json!(25.0)),
        ],
    );
    assert!(ls(&base) > ls(&plain), "{} vs {}", ls(&base), ls(&plain));
    let formed = results(
        SELF,
        &[
            ("tank_forming_allowance_pct", json!(10.0)),
            ("tank_dome_wall_thickness_mm", json!(0.0)),
        ],
    );
    assert!(ls(&formed) > ls(&plain));
    let domed = results(
        SELF,
        &[
            ("tank_forming_allowance_pct", json!(0.0)),
            ("tank_dome_wall_thickness_mm", json!(40.0)),
        ],
    );
    let domed_less = results(
        SELF,
        &[
            ("tank_forming_allowance_pct", json!(0.0)),
            ("tank_dome_wall_thickness_mm", json!(20.0)),
        ],
    );
    assert!(ls(&domed) > ls(&domed_less) && ls(&domed_less) > ls(&plain));
}
