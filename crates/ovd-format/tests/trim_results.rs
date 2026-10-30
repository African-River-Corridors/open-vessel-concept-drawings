//! Seam: the trim fields of a Design's results (issue #349): present per condition only when the
//! stern block mass is in the lightship, absent for a file saved before (its hash is unchanged).
//!
//! Label: property.

use ovd_format::{DesignSession, HashStatus, Profile, RunOutcome};
use serde_json::{Value, json};

fn results(s: &DesignSession) -> Value {
    match s.evaluate().options[0].outcome.as_ref().expect("it ran") {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_new_option_has_a_trim_per_condition_and_without_the_block_none() {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    for (n, v) in [
        ("cargo", json!("LNG")),
        ("length_overall_m", json!(110.0)),
        ("beam_m", json!(11.4)),
        ("moulded_depth_m", json!(6.0)),
        // The legacy conditions: their trim is there only with the stern block. The IS Code
        // conditions (issue #426), the default since part 3, always carry a trim.
        ("loading_conditions", json!("legacy")),
    ] {
        s.set_input("Option 1", n, v).unwrap();
    }
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    let r = results(&s);
    for c in r["stability"]["conditions"].as_array().unwrap() {
        let (t, aft, fwd) = (&c["trim_m"], &c["draught_aft_m"], &c["draught_fwd_m"]);
        assert!(t.is_number() && aft.is_number() && fwd.is_number(), "{c}");
        let mean = (aft.as_f64().unwrap() + fwd.as_f64().unwrap()) / 2.0;
        assert!(
            (mean - c["draught_m"].as_f64().unwrap()).abs() < 1e-9,
            "{c}"
        );
    }
    s.set_input("Option 1", "stern_block_mass_included", json!(false))
        .unwrap();
    let r = results(&s);
    for c in r["stability"]["conditions"].as_array().unwrap() {
        assert!(c.get("trim_m").is_none(), "{c}");
    }
}

#[test]
fn a_file_saved_before_has_no_trim_fields() {
    let path = format!(
        "{}/tests/legacy/v2_gas_lpg_before_313.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let (s, opened) =
        DesignSession::open(&std::fs::read_to_string(path).unwrap(), Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
    let r = results(&s);
    assert!(r.get("trim").is_none());
    for c in r["stability"]["conditions"].as_array().unwrap() {
        assert!(c.get("trim_m").is_none());
    }
}
