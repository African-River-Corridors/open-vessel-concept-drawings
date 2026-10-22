//! Seam: the ES-TRIN 7.02(2) Blind Distance of a new self-propelled Option on the hull's sections
//! (issue #448 on #354 and #355). The wheelhouse lift is sized to the 250 m limit; the Rule compares
//! with no tolerance, so the sized value must be at or below 250 m.
//!
//! Label: sourced - ES-TRIN 2025/1 Article 7.02(2) (250 m, unladen).

use ovcd_format::rule_set::Library;
use ovcd_format::{Profile, RunOutcome, fill_as_new, run_option_with};
use serde_json::json;

#[test]
fn the_sized_light_blind_distance_is_at_most_250_m_on_the_sections_hull() {
    for (loa, beam, depth) in [(110.0, 11.4, 6.0), (90.0, 14.0, 4.5), (76.5, 11.4, 4.5)] {
        let file = json!({
            "format": "vdesign", "format_version": 3, "engine_version": "0.2.4", "units": "SI",
            "options": [{ "name": "Option 1", "inputs": {
                "cargo": { "value": "LNG", "source": "typed" },
                "hull_form": { "value": "raked_box", "source": "typed" },
                "formation": { "value": { "kind": "self_propelled" }, "source": "typed" },
                "length_overall_m": { "value": loa, "source": "typed" },
                "beam_m": { "value": beam, "source": "typed" },
                "moulded_depth_m": { "value": depth, "source": "typed" }
            }}]
        });
        let filled = fill_as_new(&file.to_string(), &Profile::built_in());
        let design = filled.design.unwrap();
        let results = match run_option_with(&design.options[0], &Library::built_in(), &[]) {
            RunOutcome::Results { results, .. } => results,
            other => panic!("{other:?}"),
        };
        let row = results["rule_trace"]["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["rule"] == "es-trin/7.02(2)/blind-zone-unladen")
            .unwrap_or_else(|| panic!("{loa} m: no blind distance row"))
            .clone();
        assert_eq!(row["tolerance"], json!(0.0));
        let value = row["value"].as_f64().unwrap();
        assert!(value <= 250.0, "{loa} x {beam} x {depth}: {value} m");
        assert_eq!(row["outcome"], "pass", "{loa} x {beam} x {depth}: {row}");
    }
}
