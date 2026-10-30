//! Seam: `ovd hull import` (issue #451), end to end on real files in a temporary folder.
//! Label: property
//! Each test states a rule of the command: the Option becomes an offsets hull with the table's
//! dimensions, a bad CSV writes nothing, and an existing file is never overwritten.

use std::path::PathBuf;
use std::process::{Command, Output};

fn ovd(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ovd"))
        .args(args)
        .output()
        .expect("ovd runs")
}

fn temp_dir(test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ovd-test-{}-{test}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../ovd-format/tests/fixtures/dry_bulk_box_hold.vdesign"
);

const CSV: &str =
    "station_x_m,y_m,z_m\n0,0,0\n0,5,0\n0,5,4\n0,0,4\n\n60,0,0\n60,5,0\n60,5,4\n60,0,4\n";

#[test]
fn an_import_writes_an_offsets_option_with_the_tables_dimensions() {
    let dir = temp_dir("hull-import");
    let csv = dir.join("hull.csv");
    std::fs::write(&csv, CSV).unwrap();
    let out_path = dir.join("out.vdesign");
    let out = ovd(&[
        "hull",
        "import",
        csv.to_str().unwrap(),
        "--option",
        "Option 1",
        FIXTURE,
        out_path.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let design: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out_path).unwrap()).unwrap();
    let inputs = &design["options"][0]["inputs"];
    assert_eq!(inputs["hull_form"]["value"], "offsets");
    assert_eq!(inputs["hull_offsets"]["value"].as_array().unwrap().len(), 8);
    assert_eq!(inputs["length_overall_m"]["value"], 60.0);
    assert_eq!(inputs["beam_m"]["value"], 10.0);
    assert_eq!(inputs["moulded_depth_m"]["value"], 4.0);
    // It runs, and the file it wrote is not overwritten by a second import.
    assert!(ovd(&["run", out_path.to_str().unwrap()]).status.success());
    let again = ovd(&[
        "hull",
        "import",
        csv.to_str().unwrap(),
        "--option",
        "Option 1",
        FIXTURE,
        out_path.to_str().unwrap(),
    ]);
    assert!(!again.status.success());
}

#[test]
fn a_bad_csv_writes_nothing_and_says_why() {
    let dir = temp_dir("hull-import-bad");
    let csv = dir.join("bad.csv");
    std::fs::write(&csv, "x,y,z\n0,0,0\n").unwrap();
    let out_path = dir.join("out.vdesign");
    let out = ovd(&[
        "hull",
        "import",
        csv.to_str().unwrap(),
        "--option",
        "Option 1",
        FIXTURE,
        out_path.to_str().unwrap(),
    ]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("starts with the header"));
    assert!(!out_path.exists());
}
