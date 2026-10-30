//! Seam: the barge of an ATB has a Notch in a run (issue #362, spec 0007): the Notch comes from the tug,
//! the tug from the barge, each unless typed; the Notch takes volume from the barge; any other
//! Formation is unchanged.
//!
//! Label: property. The closed forms of the Notch are in `crates/ovd-core/tests/atb.rs`.

use ovd_format::{DesignSession, Profile, RunOutcome, run_design};
use serde_json::{Value, json};

fn volume_at_depth(kind: &str, typed: &[(&str, f64)]) -> f64 {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    s.set_input("Option 1", "cargo", json!("bauxite")).unwrap();
    s.set_formation("Option 1", kind, None, None).unwrap();
    s.set_input("Option 1", "loading_conditions", json!("legacy"))
        .unwrap();
    for (name, v) in typed {
        s.set_input("Option 1", name, json!(v)).unwrap();
    }
    let design = s.design().clone();
    match run_design(&design).remove(0).outcome {
        RunOutcome::Results { results, .. } => {
            let row: &Value = &results["hydrostatic_table"][9];
            row["volume_m3"].as_f64().unwrap()
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn an_atb_barge_displaces_less_than_the_same_pushed_barge() {
    let pushed = volume_at_depth("pushed_barge", &[]);
    let atb = volume_at_depth("atb", &[]);
    assert!(atb < pushed, "{atb} vs {pushed}");
}

#[test]
fn a_typed_notch_is_kept_and_a_notch_of_zero_gives_the_pushed_barge_bit_for_bit() {
    let pushed = volume_at_depth("pushed_barge", &[]);
    let none = volume_at_depth("atb", &[("notch_depth_m", 0.0001), ("notch_width_m", 0.0)]);
    assert_eq!(none.to_bits(), pushed.to_bits());
    let shallow = volume_at_depth("atb", &[("notch_depth_m", 4.0)]);
    let deep = volume_at_depth("atb", &[("notch_depth_m", 12.0)]);
    let default = volume_at_depth("atb", &[]);
    assert!(
        deep < default && default < shallow,
        "{shallow} {default} {deep}"
    );
}

#[test]
fn a_larger_tug_cuts_a_larger_notch() {
    let default = volume_at_depth("atb", &[]);
    let long_tug = volume_at_depth("atb", &[("tug_length_m", 40.0)]);
    assert!(long_tug < default, "{long_tug} vs {default}");
}
