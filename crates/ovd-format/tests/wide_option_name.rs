//! A wide Option name (issue #660): every drawing of an Option keeps to clause 14.6 of the Drawing
//! Format Standard ("text MUST fit; never cut, shortened or made smaller") for any name the title
//! block can hold. The document number of a name too wide for its cell is a short id of the name; the
//! full name stays in the supplementary title. A name too wide for the supplementary title gets a
//! message that names the field and says what fits, not a fault of the standard.
//!
//! Label: property - every fixture, every drawing, widths by the standard's own table.

use ovd_draw::rules::text_width;
use ovd_draw::{NoFiles, RunResult, run_value};
use ovd_format::{
    Profile, RunOutcome, arrangement_plan_named, check, ga_views_named, option_name_fits,
    run_design, tug_ga_named,
};
use serde_json::Value;
use std::path::Path;

const FIXTURES: [&str; 13] = [
    "crude_integral_tanks_coastal.vdesign",
    "crude_integral_tanks_pushed.vdesign",
    "crude_integral_tanks_self_propelled.vdesign",
    "diesel_integral_tanks_pushed.vdesign",
    "dry_bulk_box_hold.vdesign",
    "dry_bulk_sloped_hopper.vdesign",
    "gas_igc_2g_exact.vdesign",
    "gas_igc_2g_self_propelled.vdesign",
    "offsets_dry_bulk.vdesign",
    "pushed_convoy.vdesign",
    "shipd_dry_bulk.vdesign",
    "towed_barge.vdesign",
    "two_options.vdesign",
];

/// Names that fit the supplementary title but not the document number cell as `<NAME>-GA-<n>`.
const WIDE: [&str; 4] = [
    "140 x 29 x 9m LNG",
    "D river-only xyz",
    "Coastal LNG barge, 140 x 29 x 9 m",
    "WWWWWWWWWWWWWWWWWWWWWWWW",
];

fn outcome(fixture: &str) -> RunOutcome {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(fixture);
    let checked = check(
        &std::fs::read_to_string(path).unwrap(),
        &Profile::built_in(),
    );
    run_design(&checked.design.unwrap()).remove(0).outcome
}

/// Every drawing document of an Option: the three views, the arrangement plan, the tug GA.
fn documents(outcome: &RunOutcome, name: &str) -> Vec<(String, Value)> {
    let mut out = Vec::new();
    if let Ok(v) = ga_views_named(outcome, Some(name)) {
        out.extend(v.all().into_iter().map(|(k, d)| (k.to_string(), d.clone())));
    }
    if let Ok(d) = arrangement_plan_named(outcome, Some(name)) {
        out.push(("arrangement-plan".into(), d));
    }
    if let Ok(d) = tug_ga_named(outcome, Some(name)) {
        out.push(("tug-ga".into(), d));
    }
    out
}

fn fields(document: &Value) -> &Value {
    &document["title_block"]["fields"]
}

#[test]
fn every_drawing_of_a_wide_name_keeps_to_the_standard_and_names_the_option() {
    let mut drawn = 0;
    for fixture in FIXTURES {
        let o = outcome(fixture);
        let usual = documents(&o, "Option 1").len();
        for name in WIDE {
            assert!(option_name_fits(name).is_ok(), "{name}");
            let docs = documents(&o, name);
            assert_eq!(
                docs.len(),
                usual,
                "{fixture} '{name}': a drawing went missing"
            );
            for (view, document) in docs {
                if let RunResult::Errors(e) = run_value(&document, &NoFiles) {
                    panic!("{fixture} '{name}' {view}: the standard refuses it (clause {e:?})");
                }
                let supplementary = fields(&document)["supplementary_title"].as_str().unwrap();
                assert!(
                    supplementary.starts_with(&name.to_ascii_uppercase()),
                    "{fixture} '{name}' {view}: the full name is not in '{supplementary}'"
                );
                drawn += 1;
            }
        }
    }
    assert!(drawn > 0, "no drawing was checked");
}

#[test]
fn a_wide_name_gets_a_short_id_that_is_the_same_on_each_run_and_differs_by_name() {
    let o = outcome("gas_igc_2g_self_propelled.vdesign");
    let number = |name: &str| {
        let v = ga_views_named(&o, Some(name)).unwrap();
        fields(&v.side_elevation)["document_number"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let a = number("140 x 29 x 9m LNG");
    assert!(a.starts_with("OPTION-") && a.ends_with("-GA-1"), "{a}");
    assert_eq!(a, number("140 x 29 x 9m LNG"));
    assert_ne!(a, number("140 x 29 x 9m LPG"));
}

#[test]
fn a_name_that_fits_keeps_its_document_number() {
    // Byte parity for every name that drew before issue #660: `<NAME>-GA-<n>` (spec 0003).
    let o = outcome("gas_igc_2g_self_propelled.vdesign");
    for (name, number) in [
        ("Option 1", "OPTION-1-GA-1"),
        ("140x29x9m LNG", "140X29X9M-LNG-GA-1"),
        ("D river-only xy", "D-RIVER-ONLY-XY-GA-1"),
    ] {
        let v = ga_views_named(&o, Some(name)).unwrap();
        assert_eq!(fields(&v.side_elevation)["document_number"], number);
        assert_eq!(
            fields(&v.plan)["document_number"],
            number.replace("-GA-1", "-GA-2")
        );
    }
}

#[test]
fn the_short_id_fits_its_cell_for_every_digit_and_kind() {
    // The cell of the document number is 70 mm less 1 mm padding each side, at 5 mm (iso7200).
    for digit in "0123456789ABCDEF".chars() {
        for kind in ["GA", "AP", "TUG"] {
            let id = format!("OPTION-{}-{kind}-3", digit.to_string().repeat(6));
            assert!(text_width(&id, 5.0).is_some_and(|w| w <= 68.0), "{id}");
        }
    }
}

#[test]
fn a_name_too_wide_for_the_title_block_says_so_and_what_fits() {
    let name = "A very long descriptive Option name that no title block can hold";
    let message = option_name_fits(name).unwrap_err();
    assert!(message.contains("too wide"), "{message}");
    assert!(message.contains("title block"), "{message}");
    assert!(!message.contains("fault in OVD"), "{message}");
    let o = outcome("gas_igc_2g_self_propelled.vdesign");
    let Err(e) = ga_views_named(&o, Some(name)) else {
        panic!("a name too wide drew")
    };
    assert_eq!(e, message);
    // The part that fits, by the message's own count, does fit.
    let n: usize = message
        .split("the first ")
        .nth(1)
        .and_then(|s| s.split(' ').next())
        .and_then(|s| s.parse().ok())
        .expect("the message gives a count");
    let prefix: String = name.chars().take(n).collect();
    assert!(option_name_fits(prefix.trim()).is_ok(), "'{prefix}'");
    let longer: String = name.chars().take(n + 1).collect();
    assert!(option_name_fits(longer.trim()).is_err(), "'{longer}'");
}
