//! Seam: the stern block mass in a Design's results (issue #348). A new self-propelled Option has
//! `stern_block_mass_included` true, so its lightship carries the block and the results show the
//! breakdown; a file saved before keeps false, gets no new field, and keeps its results hash.
//!
//! Label: property. tests/stern_block_mass.rs in ovd-core holds the sourced rates.

use ovd_format::{DesignSession, HashStatus, Profile, RunOutcome};
use serde_json::{Value, json};

fn results(s: &DesignSession) -> Value {
    match s.evaluate().options[0].outcome.as_ref().expect("it ran") {
        RunOutcome::Results { results, .. } => results.clone(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_new_self_propelled_option_carries_the_block_and_shows_it() {
    let mut s = DesignSession::new_design(Profile::built_in()).unwrap();
    for (n, v) in [
        ("cargo", json!("LNG")),
        ("length_overall_m", json!(110.0)),
        ("beam_m", json!(11.4)),
        ("moulded_depth_m", json!(6.0)),
    ] {
        s.set_input("Option 1", n, v).unwrap();
    }
    s.set_formation("Option 1", "self_propelled", None, None)
        .unwrap();
    let with = results(&s);
    let block = with["stern_block_mass"]["total_t"].as_f64().unwrap();
    assert!(block > 50.0, "{block}");
    let sum: f64 = with["stern_block_mass"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["mass_t"].as_f64().unwrap())
        .sum();
    assert!((sum - block).abs() < 1e-9);
    s.set_input("Option 1", "stern_block_mass_included", json!(false))
        .unwrap();
    let without = results(&s);
    assert!(without.get("stern_block_mass").is_none());
}

#[test]
fn a_file_saved_before_has_no_block_mass_and_keeps_its_hash() {
    let path = format!(
        "{}/tests/legacy/v2_gas_lpg_before_313.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let (s, opened) =
        DesignSession::open(&std::fs::read_to_string(path).unwrap(), Profile::built_in()).unwrap();
    assert_eq!(opened.hash_status[0].1, HashStatus::Matches);
    assert!(results(&s).get("stern_block_mass").is_none());
}
