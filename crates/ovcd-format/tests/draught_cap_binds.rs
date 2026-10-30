//! Seam: the draught cap binds the draught the Design reports (issue #680). A draught-bound hull
//! never reports a draught below its cap's waterline, so its freeboard is never less than the
//! Minimum Freeboard it asks for.
//!
//! Label: property - the draught cap is the least of the hull's depth less the Minimum
//! Freeboard, the draught limit and the channel's depth less its under-keel clearance
//! (`docs/basis/08-waterway-and-route.md`, step 3.1, WR-A08).

use ovcd_format::{Profile, RunOutcome, fill_as_new, run_option};
use serde_json::{Value, json};

fn results_of(typed: Value) -> Value {
    let mut inputs = json!({});
    for (k, v) in typed.as_object().unwrap() {
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
        other => panic!("{other:?}"),
    }
}

fn dry_bulk(kind: &str, fb: f64) -> Value {
    json!({
        "cargo": "bauxite", "hull_form": "raked_box", "formation": { "kind": kind },
        "length_overall_m": 76.5, "beam_m": 11.4, "moulded_depth_m": 3.5,
        "min_freeboard_m": fb
    })
}

#[test]
fn a_draught_bound_dry_bulk_hull_keeps_its_minimum_freeboard() {
    // 76.5 x 11.4 x 3.5 m bauxite, self-propelled: at 0.8 m the cap is 2.70 m and the hull
    // reported 2.737 m before the fix (freeboard 0.763 m).
    let mut seen: Vec<String> = Vec::new();
    for kind in ["self_propelled", "pushed_barge"] {
        for fb in [0.3, 0.5, 0.8, 1.0] {
            let r = results_of(dry_bulk(kind, fb));
            let res = &r["results"];
            let governs = res["governs"].as_str().unwrap_or_default();
            let t = res["draught_m"].as_f64().unwrap();
            let t_max = res["draught_max_m"].as_f64().unwrap();
            let freeboard = res["freeboard_m"].as_f64().unwrap();
            seen.push(format!(
                "{kind} {fb}: {governs}, T {t:.4}, cap {t_max:.4}, F {freeboard:.4}"
            ));
            if governs.starts_with("draught") || t > t_max {
                assert!(t <= t_max + 1e-6 && freeboard >= fb - 1e-6, "{seen:#?}");
                // The cargo is the most the cap allows: the reported draught is at the cap.
                assert!(t >= t_max - 1e-4, "{seen:#?}");
            }
        }
    }
}
