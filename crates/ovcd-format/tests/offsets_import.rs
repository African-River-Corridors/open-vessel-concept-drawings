//! Seam: the offsets import (issue #451): `DesignSession::import_offsets`, `ovcd hull import` and the
//! `offsets` Hull Form in a run.
//!
//! Label: property. Each test states a rule: a hull exported and imported again gives the same
//! results, each bad CSV or table is refused with its reason, and L, B and D come from the table.

use ovcd_core::offsets::to_rows;
use ovcd_format::offsets::{parse_csv, to_csv};
use ovcd_format::{DesignSession, Profile, RunOutcome, check, option_hull_table, run_option};
use serde_json::Value;

fn fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{name}.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn results(design: &ovcd_format::Design) -> (Value, String) {
    match run_option(&design.options[0]) {
        RunOutcome::Results { results, hash } => (results, hash),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_hull_exported_and_imported_again_gives_the_same_results() {
    for name in ["dry_bulk_box_hold", "gas_igc_2g_exact"] {
        let text = fixture(name);
        let design = check(&text, &Profile::built_in()).design.unwrap();
        let table = option_hull_table(&design.options[0]).unwrap();
        let csv = to_csv(&to_rows(&table));
        let (mut s, _) = DesignSession::open(&text, Profile::built_in()).unwrap();
        let warnings = s.import_offsets("Option 1", &csv).unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        let imported = s.design().clone();
        let form = imported.options[0].input("hull_form").unwrap();
        assert_eq!(form.value, "offsets");
        assert_eq!(
            option_hull_table(&imported.options[0]).unwrap(),
            table,
            "{name}"
        );
        let (a, ha) = results(&design);
        let (b, hb) = results(&imported);
        assert_eq!(a["hydrostatics"], b["hydrostatics"], "{name}");
        assert_eq!(a["results"], b["results"], "{name}");
        assert_eq!(ha, hb, "{name}");
    }
}

const BOX: &str =
    "station_x_m,y_m,z_m\n0,0,0\n0,5,0\n0,5,4\n0,0,4\n\n60,0,0\n60,5,0\n60,5,4\n60,0,4\n";

#[test]
fn each_bad_csv_or_table_is_refused_with_its_reason_and_the_option_is_left_as_it_was() {
    let text = fixture("dry_bulk_box_hold");
    for (csv, reason) in [
        (
            "x,y,z\n0,0,0\n",
            "starts with the header station_x_m,y_m,z_m",
        ),
        (
            "station_x_m,y_m,z_m\n0,0\n",
            "line 2 of the offsets CSV needs three numbers",
        ),
        (
            "station_x_m,y_m,z_m\n0,0,0\n0,5,0\n0,0,4\n",
            "at least 2 stations, not 1",
        ),
        (
            "station_x_m,y_m,z_m\n10,0,0\n10,5,0\n10,0,4\n0,0,0\n0,5,0\n0,0,4\n",
            "station 2 of the offsets table lies aft of the station before it",
        ),
        (
            "station_x_m,y_m,z_m\n0,0,0\n0,5,0\n0,0,4\n10,0,0\n10,5,0\n10,1,4\n",
            "station 2 of the offsets table does not start and end on the centre line",
        ),
        (
            "station_x_m,y_m,z_m\n0,0,0\n0,5,0\n0,0,4\n10,0,0\n10,-1,2\n10,0,4\n",
            "station 2 of the offsets table has a point with y below 0",
        ),
        (
            "station_x_m,y_m,z_m\n0,0,0\n0,5,0\n0,5,4\n0,0,4\n10,0,0\n10,5,4\n10,5,0\n10,0,4\n",
            "station 2 of the offsets table crosses itself",
        ),
    ] {
        let (mut s, _) = DesignSession::open(&text, Profile::built_in()).unwrap();
        let before = s.design().clone();
        let e = s.import_offsets("Option 1", csv).unwrap_err();
        assert!(e.contains(reason), "{csv:?}: {e}");
        assert_eq!(s.design(), &before);
    }
    assert_eq!(parse_csv(BOX).unwrap().len(), 8);
}

#[test]
fn an_offsets_option_takes_l_b_and_d_from_its_table() {
    let (mut s, _) =
        DesignSession::open(&fixture("dry_bulk_box_hold"), Profile::built_in()).unwrap();
    s.import_offsets("Option 1", BOX).unwrap();
    let o = &s.design().options[0];
    for (name, want) in [
        ("length_overall_m", 60.0),
        ("beam_m", 10.0),
        ("moulded_depth_m", 4.0),
    ] {
        assert_eq!(o.input(name).unwrap().value.as_f64(), Some(want), "{name}");
    }
    // A typed length that differs is noted, and the run takes the table's.
    s.set_input("Option 1", "length_overall_m", serde_json::json!(70.0))
        .unwrap();
    let text = s.design().to_text();
    let checked = check(&text, &Profile::built_in());
    assert!(
        checked
            .report
            .warnings
            .iter()
            .any(|w| w.contains("differs from the offsets table's 60")),
        "{:?}",
        checked.report.warnings
    );
    let (r, _) = results(&checked.design.unwrap());
    assert_eq!(r["dims"]["loa_m"].as_f64(), Some(60.0));
}

#[test]
fn a_table_of_unequal_stations_is_imported_with_the_resampling_warning() {
    let csv = "station_x_m,y_m,z_m\n0,0,0\n0,5,0\n0,5,4\n0,0,4\n60,0,0\n60,2,0\n60,5,0\n60,5,1\n60,5,4\n60,0,4\n";
    let (mut s, _) =
        DesignSession::open(&fixture("dry_bulk_box_hold"), Profile::built_in()).unwrap();
    let warnings = s.import_offsets("Option 1", csv).unwrap();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("1 station(s) are resampled by arc length to 6 points"));
}
