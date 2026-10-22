//! Seam: the freeboard Rules of issue #428 (the maintainer's option A): `es-trin-2025` in the river
//! stack (and the coastal one), `icll-freeboard` in the coastal stack, on the input
//! `min_freeboard_m`, now labelled as the safety clearance it is. No file saved before changes: its
//! stored stack stays.
//!
//! Label: sourced - ES-TRIN 2025/1 Articles 4.01 and 4.02 (ES_TRIN_2025_signed_en.pdf, Chapter 4,
//! p. 17); Protocol of 1988 to the Load Lines Convention as revised by MSC.143(77), regulations
//! 27(14) and 28(1), table 28.1 (MSC.143-77.pdf, annex pp. 42-44)

use ovcd_core::rule_trace::{MeasureValue, table_value};
use ovcd_core::rules::{Compare, Limit};
use ovcd_format::rule_set::Library;
use ovcd_format::{Profile, RunOutcome, fill_as_new, input_schema, run_option};
use serde_json::{Value, json};

fn results_of(cargo: &str, extra: Value) -> Value {
    let mut inputs = json!({
        "cargo": { "value": cargo, "source": "typed" },
        "hull_form": { "value": "raked_box", "source": "typed" },
        "formation": { "value": { "kind": "pushed_barge" }, "source": "typed" },
        "length_overall_m": { "value": 76.5, "source": "typed" },
        "beam_m": { "value": 11.4, "source": "typed" },
        "moulded_depth_m": { "value": 3.5, "source": "typed" }
    });
    for (k, v) in extra.as_object().unwrap() {
        inputs[k] = json!({ "value": v, "source": "typed" });
    }
    let file = json!({
        "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
        "options": [{ "name": "Option 1", "inputs": inputs }]
    });
    let filled = fill_as_new(&file.to_string(), &Profile::built_in());
    assert_eq!(filled.report.errors, Vec::<String>::new());
    match run_option(&filled.design.unwrap().options[0]) {
        RunOutcome::Results { results, .. } => results,
        // An infeasible Option still has its trace in the failure; read it through a feasible one.
        other => panic!("{other:?}"),
    }
}

fn outcomes(results: &Value, id: &str) -> Vec<String> {
    results["rule_trace"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["rule"] == id && r["outcome"] != "not applicable")
        .map(|r| r["outcome"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn the_input_is_labelled_as_the_safety_clearance_and_its_basis_says_it_is_not_an_icll_freeboard() {
    let p = Profile::built_in();
    let schema = input_schema(&p);
    let e = schema["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["name"] == "min_freeboard_m")
        .unwrap()
        .clone();
    assert_eq!(e["label"], "Minimum freeboard (safety clearance)");
    let basis = p.basis("min_freeboard_m");
    assert!(basis.contains("ES-TRIN 2025/1 Article 4.01(1)"), "{basis}");
    assert!(basis.contains("not an ICLL freeboard"), "{basis}");
    assert_eq!(e["default"], json!(0.3));
}

#[test]
fn the_es_trin_limits_are_300_mm_500_mm_for_an_open_hold_and_150_mm() {
    let lib = Library::built_in();
    let set = lib.find("es-trin-2025", "1.0.0").unwrap();
    let constant = |id: &str| match &set.rule(id).unwrap().limit {
        Some(Limit::Constant { value, .. }) => *value,
        other => panic!("{id}: {other:?}"),
    };
    assert_eq!(constant("es-trin/4.01(1)/safety-clearance"), 0.3);
    assert_eq!(
        constant("es-trin/4.01(1)/safety-clearance-covered-holds"),
        0.3
    );
    assert_eq!(constant("es-trin/4.02(1)/freeboard"), 0.15);
    // An open hold is measured to the top of the coaming: the limit is the clearance less it.
    for (id, clearance) in [
        ("es-trin/4.01(1)/safety-clearance-open-holds", 0.3),
        ("es-trin/4.01(2)/safety-clearance-uncovered-holds", 0.5),
    ] {
        match &set.rule(id).unwrap().limit {
            Some(Limit::Formula {
                shape,
                coefficients,
                operands,
                ..
            }) => {
                assert_eq!(shape, "at-least", "{id}");
                assert!(coefficients.contains(&("a".to_string(), clearance)), "{id}");
                assert!(
                    format!("{operands:?}").contains("input.hatch_coaming_height_m"),
                    "{id}"
                );
            }
            other => panic!("{id}: {other:?}"),
        }
    }
    // The freeboard Rules of Chapter 4 (the set holds the wheelhouse and speed Rules too, #355).
    for r in set.rules.iter().filter(|r| r.id.starts_with("es-trin/4.0")) {
        assert_eq!(r.compare, Some(Compare::Ge), "{}", r.id);
        assert_eq!(r.measure.id, "input.min_freeboard_m", "{}", r.id);
        assert!(r.source.page.ends_with("Chapter 4, p. 17"), "{}", r.id);
    }
    // Only the 150 mm Rule has a second reader; the 4.01 Rules changed after it.
    assert!(
        set.rule("es-trin/4.02(1)/freeboard")
            .unwrap()
            .verified
            .is_some()
    );
    assert_eq!(
        set.rule("es-trin/4.01(2)/safety-clearance-uncovered-holds")
            .unwrap()
            .applies_when
            .conditions,
        vec![
            ("cargo_class".to_string(), vec!["dry_bulk".to_string()]),
            ("hold_covering".to_string(), vec!["open".to_string()])
        ]
    );
}

#[test]
fn the_icll_table_gives_the_printed_values_at_each_length_and_interpolates_between() {
    // Table 28.1, by hand from the printed table (mm): 24 m 200, 36 m 300, 90 m 984, 100 m 1135,
    // 230 m 2872, 365 m 3433. Intermediate lengths: linear interpolation (the text of regulation 28).
    let lib = Library::built_in();
    let set = lib.find("icll-freeboard", "1.0.0").unwrap();
    let table = set.table("icll-type-a-freeboard").unwrap();
    let at = |l: f64| table_value(table, &MeasureValue::Number(l)).unwrap();
    for (l, mm) in [
        (24.0, 200.0),
        (25.0, 208.0),
        (36.0, 300.0),
        (90.0, 984.0),
        (100.0, 1135.0),
        (104.0, 1196.0),
        (105.0, 1212.0),
        (230.0, 2872.0),
        (294.0, 3237.0),
        (365.0, 3433.0),
    ] {
        assert!((at(l) - mm / 1000.0).abs() < 1e-12, "{l}: {}", at(l));
    }
    assert!((at(36.5) - 0.304).abs() < 1e-12); // (300 + 308) / 2 mm
    assert!((at(90.25) - (0.984 + 0.25 * 0.015)).abs() < 1e-12);
    // Both sides of the first row: the table starts at 24 m (the Convention is for ships of 24 m and
    // over); below it there is no value, taken as none.
    assert_eq!(at(23.999), 0.0);
    assert!((at(24.0) - 0.2).abs() < 1e-12);
    // The table never falls with the length.
    let mut last = 0.0;
    for l in 24..=365 {
        let v = at(l as f64);
        assert!(v >= last, "{l}");
        last = v;
    }
    let rule = set.rule("icll/28(1)/type-a-tabular-freeboard").unwrap();
    assert_eq!(rule.measure.id, "input.min_freeboard_m");
    assert!(rule.verified.is_none());
}

#[test]
fn an_open_hold_river_design_is_measured_to_the_coaming_top() {
    // Covered holds and tank vessels: 300 mm to the deck edge. Open holds: the clearance is to the
    // top of the coaming, so the freeboard needs only the clearance less the coaming height.
    let open = |fb: f64, coaming: f64| {
        results_of(
            "bauxite",
            json!({ "hold_covering": "open", "min_freeboard_m": fb, "hatch_coaming_height_m": coaming }),
        )
    };
    let id2 = "es-trin/4.01(2)/safety-clearance-uncovered-holds";
    let id1 = "es-trin/4.01(1)/safety-clearance-open-holds";
    // 300 mm freeboard with a 600 mm coaming: 900 mm to the top, which meets both.
    let ok = open(0.3, 0.6);
    assert_eq!(outcomes(&ok, id2), ["pass"]);
    assert_eq!(outcomes(&ok, id1), ["pass"]);
    // 300 mm with a 100 mm coaming: 400 mm, which meets 300 mm and not 500 mm.
    let low = open(0.3, 0.1);
    assert_eq!(outcomes(&low, id1), ["pass"]);
    assert_eq!(outcomes(&low, id2), ["fail"]);
    // 400 mm with a 100 mm coaming is exactly 500 mm.
    assert_eq!(outcomes(&open(0.4, 0.1), id2), ["pass"]);
    // The deck-edge 300 mm Rules do not apply to an open hold.
    for id in [
        "es-trin/4.01(1)/safety-clearance",
        "es-trin/4.01(1)/safety-clearance-covered-holds",
    ] {
        assert!(outcomes(&ok, id).is_empty(), "{id}");
    }
    // Covered: 300 mm to the deck edge, and no 500 mm Rule.
    let covered = results_of("bauxite", json!({ "min_freeboard_m": 0.4 }));
    assert!(outcomes(&covered, id2).is_empty());
    assert!(outcomes(&covered, id1).is_empty());
    assert_eq!(
        outcomes(&covered, "es-trin/4.01(1)/safety-clearance-covered-holds"),
        ["pass"]
    );
    let thin = results_of("bauxite", json!({ "min_freeboard_m": 0.25 }));
    assert_eq!(
        outcomes(&thin, "es-trin/4.01(1)/safety-clearance-covered-holds"),
        ["fail"]
    );
    // The default is 300 mm, covered: the new Rules pass for a new river Option.
    let new = results_of("bauxite", json!({}));
    for id in [
        "es-trin/4.01(1)/safety-clearance-covered-holds",
        "es-trin/4.02(1)/freeboard",
    ] {
        assert_eq!(outcomes(&new, id), ["pass"], "{id}");
    }
    assert!(outcomes(&new, "icll/28(1)/type-a-tabular-freeboard").is_empty());
}

#[test]
fn a_coastal_option_is_checked_against_the_icll_table_and_shows_the_value() {
    // 76.5 m: the table gives 0.793 m (773 mm at 75 m, 786 mm at 76 m, 800 mm at 77 m: 793 mm at
    // 76.5 m). A self-propelled vessel is blocked below it; a barge with no propulsion of its own
    // gets a warning, because regulation 27(14)(c) lets an unmanned barge take 25 % less.
    let coastal = |fb: f64, kind: &str| {
        results_of(
            "bauxite",
            json!({
                "service_area": "coastal-and-river", "min_freeboard_m": fb,
                "formation": { "kind": kind }
            }),
        )
    };
    let row_of = |r: &Value, id: &str| {
        r["rule_trace"]["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["rule"] == id && row["outcome"] != "not applicable")
            .cloned()
            .unwrap_or_else(|| panic!("{id}"))
    };
    let blocked = "icll/28(1)/type-a-tabular-freeboard";
    let barge = "icll/28(1)/type-a-tabular-freeboard-barge";
    let sp = |fb| coastal(fb, "self_propelled");
    let low = sp(0.3);
    let row = row_of(&low, blocked);
    assert_eq!(row["outcome"], "fail");
    assert_eq!(row["consequence"], "block");
    assert!(
        (row["limit"].as_f64().unwrap() - 0.793).abs() < 1e-9,
        "{row}"
    );
    assert!(outcomes(&low, barge).is_empty());
    assert_eq!(outcomes(&sp(0.8), blocked), ["pass"]);
    assert_eq!(outcomes(&sp(0.792), blocked), ["fail"]);
    assert_eq!(outcomes(&sp(0.793), blocked), ["pass"]);
    let sets = low["rule_trace"]["rule_sets"].to_string();
    assert!(
        sets.contains("icll-freeboard@1.0.0") && sets.contains("es-trin-2025@1.0.0"),
        "{sets}"
    );
    // A pushed barge: the same value, as a warning.
    let pushed = coastal(0.3, "pushed_barge");
    let row = row_of(&pushed, barge);
    assert_eq!(row["outcome"], "warn");
    assert!(outcomes(&pushed, blocked).is_empty());
    assert_eq!(outcomes(&coastal(0.8, "pushed_barge"), barge), ["pass"]);
}
