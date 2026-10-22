//! The liquid bulk Rule Sets `adn-tank`, `marpol-i` and `normam-202` (issue #338, spec 0006 section 5.1).
//!
//! Label: sourced - ADN 2025 (ECE/TRANS/351) 9.3.2.11.1 (a) and (d), 9.3.2.11.3 (a), 9.3.2.11.7, 9.3.3.11.1,
//! 9.3.3.11.3, 9.3.3.11.7; MARPOL Annex I as revised by MEPC.117(52), regulations 19.6.1, 19.6.2 and 29;
//! NORMAM-202/DPC 5.23.6 (a) (II). Each limit below is worked out by hand from the text; a Rule is tried
//! one step inside and one step across its limit.

use ovcd_core::liquid_bulk::LiquidLayout;
use ovcd_core::liquid_bulk::PumpArrangement;
use ovcd_format::rule_set::Library;
use ovcd_format::trace::{LiquidTraceCase, liquid_trace};
use serde_json::Value;

fn case() -> LiquidTraceCase {
    LiquidTraceCase {
        loa_m: 76.5,
        beam_m: 11.4,
        depth_m: 3.5,
        zone_aft_m: 3.0,
        zone_fwd_m: 40.0,
        layout: LiquidLayout {
            double_side_m: 1.2,
            double_bottom_m: 0.9,
            tanks_in_line: 3,
            centreline_bulkhead: false,
            cofferdam_m: 0.8,
            pump_arrangement: PumpArrangement::Deepwell,
            slop_share: 0.04,
            fill_ratio: 0.98,
            heating_kw_per_m3: 0.0,
            lightship_k_t_per_m3: 0.1141,
        },
        cargo_density_t_m3: 0.87,
        deadweight_t: 1500.0,
        service_area: "river-only".to_string(),
        adn_vessel_type: "C".to_string(),
    }
}

fn trace(c: &LiquidTraceCase, sets: &[(&str, &str)]) -> Vec<Value> {
    let lib = Library::built_in();
    liquid_trace(c, &lib, sets).unwrap()["rows"]
        .as_array()
        .unwrap()
        .clone()
}

const ADN: &[(&str, &str)] = &[("adn-tank", "1.0.0")];
const BOTH: &[(&str, &str)] = &[("adn-tank", "1.0.0"), ("marpol-i", "1.0.0")];

fn outcomes<'a>(rows: &'a [Value], rule: &str) -> Vec<&'a str> {
    rows.iter()
        .filter(|r| r["rule"] == rule)
        .map(|r| r["outcome"].as_str().unwrap())
        .collect()
}

fn row<'a>(rows: &'a [Value], rule: &str) -> &'a Value {
    rows.iter()
        .find(|r| r["rule"] == rule)
        .unwrap_or_else(|| panic!("{rule}"))
}

/// The case moved so a Rule's governing value is `value`, and the outcome of the Rule's row.
fn at(rule: &str, edit: impl Fn(&mut LiquidTraceCase), sets: &[(&str, &str)]) -> &'static str {
    let mut c = case();
    c.service_area = "coastal-and-river".to_string();
    edit(&mut c);
    let rows = trace(&c, sets);
    let o = outcomes(&rows, rule);
    assert!(!o.is_empty(), "{rule} has no row");
    // A Rule with a row per tank: the worst row stands for it.
    if o.contains(&"warn") {
        "fail"
    } else if o.contains(&"pass") {
        "pass"
    } else {
        "other"
    }
}

// --- ADN type C, by hand --------------------------------------------------------------------------

#[test]
fn the_type_c_double_side_and_bottom_limits_are_1_00_and_0_70_and_0_60() {
    let side = "adn/9.3.2.11.7/wing-width-min";
    assert_eq!(at(side, |c| c.layout.double_side_m = 1.0, ADN), "pass");
    assert_eq!(at(side, |c| c.layout.double_side_m = 0.999, ADN), "fail");
    let bottom = "adn/9.3.2.11.7/double-bottom-depth";
    assert_eq!(at(bottom, |c| c.layout.double_bottom_m = 0.7, ADN), "pass");
    assert_eq!(
        at(bottom, |c| c.layout.double_bottom_m = 0.699, ADN),
        "fail"
    );
    let min = "adn/9.3.2.11.7/double-bottom-min";
    assert_eq!(at(min, |c| c.layout.double_bottom_m = 0.6, ADN), "pass");
    assert_eq!(at(min, |c| c.layout.double_bottom_m = 0.599, ADN), "fail");
}

#[test]
fn the_cofferdam_is_at_least_0_60_m() {
    let r = "adn/9.3.2.11.3(a)/cofferdam-width";
    assert_eq!(at(r, |c| c.layout.cofferdam_m = 0.6, ADN), "pass");
    assert_eq!(at(r, |c| c.layout.cofferdam_m = 0.59, ADN), "fail");
}

// L x B x H = 76.5 x 11.4 x 3.5 = 3,052.35 m3, in the second row of the table:
// 180 + (3,052.35 - 600) x 0.0635 = 335.724225 m3. With a double side of 1.0 m and a double bottom of
// 0.9 m a tank is (11.4 - 2 x 1.0) = 9.4 m wide and (3.5 - 0.9) = 2.6 m high: 24.44 m2 of section.
#[test]
fn the_cargo_tank_capacity_table_is_worked_out_by_hand() {
    let r = "adn/9.3.2.11.1(a)/cargo-tank-capacity";
    let limit: f64 = 180.0 + (76.5 * 11.4 * 3.5 - 600.0) * 0.0635;
    assert!((limit - 335.724225).abs() < 1e-9);
    // A tank of length l holds 24.44 l: set the zone so that the tank holds a little less and a little more.
    let zone_for = |volume: f64| {
        let tank_length = volume / 24.44;
        3.0 + 3.0 * tank_length + 2.0 * 0.8 // zone_aft + block + two cofferdams, no slop, no pump room
    };
    let edit = |volume: f64| {
        move |c: &mut LiquidTraceCase| {
            c.layout.double_side_m = 1.0;
            c.layout.slop_share = 0.0;
            c.zone_fwd_m = zone_for(volume);
        }
    };
    assert_eq!(at(r, edit(limit - 0.01), ADN), "pass");
    assert_eq!(at(r, edit(limit + 0.01), ADN), "fail");
}

#[test]
fn the_capacity_table_rows_at_600_and_3750_and_above() {
    // The limit is read off the Rule's row, by the same table: 0.3 LBH up to 600; 380 above 3,750.
    let limit_of = |loa: f64, beam: f64, depth: f64| -> f64 {
        let mut c = case();
        c.loa_m = loa;
        c.beam_m = beam;
        c.depth_m = depth;
        c.zone_fwd_m = loa - 2.0;
        c.layout.tanks_in_line = 1;
        row(&trace(&c, ADN), "adn/9.3.2.11.1(a)/cargo-tank-capacity")["limit"]
            .as_f64()
            .unwrap()
    };
    // 20 x 10 x 3 = 600: first row, 180.
    assert!((limit_of(20.0, 10.0, 3.0) - 180.0).abs() < 1e-9);
    // 100 x 10 x 3.99 = 3,990 > 3,750: 380.
    assert_eq!(limit_of(100.0, 10.0, 3.99), 380.0);
    // 3,750 exactly is "600 to 3,750": 180 + 3,150 x 0.0635 = 380.025.
    assert!((limit_of(100.0, 10.0, 3.75) - 380.025).abs() < 1e-9);
    // 20 x 10 x 2.5 = 500, up to 600: 0.3 x 500 = 150.
    assert!((limit_of(20.0, 10.0, 2.5) - 150.0).abs() < 1e-9);
}

#[test]
fn the_cargo_tank_length_is_10_m_up_to_50_m_and_0_20_l_above() {
    let r = "adn/9.3.2.11.1(d)/cargo-tank-length";
    let limit_of = |loa: f64| -> f64 {
        let mut c = case();
        c.loa_m = loa;
        c.zone_fwd_m = loa - 2.0;
        c.layout.tanks_in_line = 6;
        row(&trace(&c, ADN), r)["limit"].as_f64().unwrap()
    };
    assert_eq!(limit_of(50.0), 10.0);
    assert!((limit_of(50.1) - 10.02).abs() < 1e-9);
    assert!((limit_of(76.5) - 15.3).abs() < 1e-9);
    // 76.5 m: tanks of 15.29 m and 15.31 m. Three positions, no slop: block = 3 l.
    let edit = |len: f64| {
        move |c: &mut LiquidTraceCase| {
            c.layout.slop_share = 0.0;
            c.zone_fwd_m = 3.0 + 3.0 * len + 1.6;
        }
    };
    assert_eq!(at(r, edit(15.29), ADN), "pass");
    assert_eq!(at(r, edit(15.31), ADN), "fail");
}

// --- ADN type N ------------------------------------------------------------------------------------

#[test]
fn the_type_n_limits_are_0_60_and_0_50_and_the_type_c_rules_do_not_apply() {
    let mut c = case();
    c.adn_vessel_type = "N_closed".to_string();
    c.layout.double_side_m = 0.6;
    c.layout.double_bottom_m = 0.5;
    let rows = trace(&c, ADN);
    assert_eq!(outcomes(&rows, "adn/9.3.3.11.7/wing-width-min"), ["pass"]);
    assert_eq!(
        outcomes(&rows, "adn/9.3.3.11.7/double-bottom-depth"),
        ["pass"]
    );
    assert_eq!(
        outcomes(&rows, "adn/9.3.2.11.7/wing-width-min"),
        ["not applicable"]
    );
    c.layout.double_side_m = 0.59;
    c.layout.double_bottom_m = 0.49;
    let rows = trace(&c, ADN);
    assert_eq!(outcomes(&rows, "adn/9.3.3.11.7/wing-width-min"), ["warn"]);
    assert_eq!(
        outcomes(&rows, "adn/9.3.3.11.7/double-bottom-depth"),
        ["warn"]
    );
    // Both types give the same capacity table and length limit.
    assert!(!outcomes(&rows, "adn/9.3.3.11.1(a)/cargo-tank-capacity").is_empty());
    assert!(!outcomes(&rows, "adn/9.3.3.11.1(d)/cargo-tank-length").is_empty());
    assert!(!outcomes(&rows, "adn/9.3.3.11.3(a)/cofferdam-width").is_empty());
}

// --- MARPOL Annex I ------------------------------------------------------------------------------

#[test]
fn reg_19_6_1_the_double_bottom_is_b_over_15_and_at_least_0_76_m() {
    let r = "marpol/19.6.1/double-bottom-depth";
    // B = 14.4 m: B/15 = 0.96 m.
    let edit = |h: f64| {
        move |c: &mut LiquidTraceCase| {
            c.beam_m = 14.4;
            c.layout.double_bottom_m = h;
        }
    };
    assert_eq!(at(r, edit(0.96), BOTH), "pass");
    assert_eq!(at(r, edit(0.95), BOTH), "fail");
    // B = 9 m: B/15 = 0.6 m, so the minimum of 0.76 m governs.
    let edit = |h: f64| {
        move |c: &mut LiquidTraceCase| {
            c.beam_m = 9.0;
            c.layout.double_bottom_m = h;
        }
    };
    assert_eq!(at(r, edit(0.76), BOTH), "pass");
    assert_eq!(at(r, edit(0.75), BOTH), "fail");
}

#[test]
fn reg_19_6_2_the_wing_tank_is_0_4_plus_2_4_dw_over_20000_and_at_least_0_76_m() {
    let r = "marpol/19.6.2/double-side-width";
    // DW 4,000 t: w = 0.4 + 2.4 x 4,000 / 20,000 = 0.88 m.
    let edit = |w: f64| {
        move |c: &mut LiquidTraceCase| {
            c.deadweight_t = 4000.0;
            c.layout.double_side_m = w;
        }
    };
    assert_eq!(at(r, edit(0.88), BOTH), "pass");
    assert_eq!(at(r, edit(0.87), BOTH), "fail");
    // DW 1,000 t: w = 0.52 m, so 0.76 m governs.
    let edit = |w: f64| {
        move |c: &mut LiquidTraceCase| {
            c.deadweight_t = 1000.0;
            c.layout.double_side_m = w;
        }
    };
    assert_eq!(at(r, edit(0.76), BOTH), "pass");
    assert_eq!(at(r, edit(0.75), BOTH), "fail");
}

#[test]
fn reg_29_the_slop_tanks_are_at_least_3_per_cent() {
    let r = "marpol/29/slop-tank-capacity";
    assert_eq!(at(r, |c| c.layout.slop_share = 0.03, BOTH), "pass");
    assert_eq!(at(r, |c| c.layout.slop_share = 0.029, BOTH), "fail");
    // No slop tank: a share of nothing fails.
    assert_eq!(at(r, |c| c.layout.slop_share = 0.0, BOTH), "fail");
}

#[test]
fn the_larger_tanker_rules_are_listed_not_evaluated_from_5000_tonnes_and_do_not_apply_below() {
    let mut c = case();
    c.service_area = "coastal-and-river".to_string();
    c.deadweight_t = 6000.0;
    let rows = trace(&c, BOTH);
    for r in [
        "marpol/19.3.1/wing-width",
        "marpol/19.3.2/double-bottom-depth",
        "marpol/23/accidental-oil-outflow",
        "marpol/27/intact-stability",
        "marpol/28/damage-stability",
    ] {
        assert_eq!(outcomes(&rows, r), ["not evaluated"], "{r}");
    }
    // Regulation 19.6 is for tankers of less than 5,000 t.
    assert_eq!(
        outcomes(&rows, "marpol/19.6.1/double-bottom-depth"),
        ["not applicable"]
    );
    // Below 5,000 t it is the other way round.
    c.deadweight_t = 4000.0;
    let rows = trace(&c, BOTH);
    assert_eq!(
        outcomes(&rows, "marpol/19.3.1/wing-width"),
        ["not applicable"]
    );
    assert!(!outcomes(&rows, "marpol/19.6.1/double-bottom-depth").contains(&"not applicable"));
    // Under 600 t deadweight Regulation 19 does not apply at all.
    c.deadweight_t = 500.0;
    let rows = trace(&c, BOTH);
    assert_eq!(
        outcomes(&rows, "marpol/19.6.1/double-bottom-depth"),
        ["not applicable"]
    );
}

#[test]
fn marpol_is_for_coastal_service_only() {
    let mut c = case();
    c.service_area = "river-only".to_string();
    let lib = Library::built_in();
    let t = liquid_trace(&c, &lib, BOTH).unwrap();
    let rows = t["rows"].as_array().unwrap();
    assert!(rows.iter().all(|r| {
        r["rule"]
            .as_str()
            .is_none_or(|id| !id.starts_with("marpol/"))
    }));
}

// --- the strictest governs ----------------------------------------------------------------------

#[test]
fn on_coastal_and_river_service_below_5000_t_the_strictest_of_adn_and_marpol_governs() {
    // B = 11.4 m, DW 3,000 t: ADN type C double side 1.00 m against MARPOL 0.4 + 2.4 x 3,000 / 20,000 =
    // 0.76 m (the minimum): ADN governs. ADN double bottom 0.70 m against MARPOL max(11.4 / 15, 0.76) =
    // 0.76 m: MARPOL governs (research note 02, section 7).
    let mut c = case();
    c.service_area = "coastal-and-river".to_string();
    c.deadweight_t = 3000.0;
    c.layout.double_side_m = 1.0;
    c.layout.double_bottom_m = 0.72;
    let rows = trace(&c, BOTH);
    let governing = |rule: &str| row(&rows, rule)["governing"].as_bool().unwrap();
    assert!(governing("adn/9.3.2.11.7/wing-width-min"));
    assert!(!governing("marpol/19.6.2/double-side-width"));
    assert!(governing("marpol/19.6.1/double-bottom-depth"));
    assert!(!governing("adn/9.3.2.11.7/double-bottom-depth"));
    // The design meets the ADN limit of the bottom but not the MARPOL one.
    assert_eq!(
        row(&rows, "adn/9.3.2.11.7/double-bottom-depth")["outcome"],
        "pass"
    );
    assert_eq!(
        row(&rows, "marpol/19.6.1/double-bottom-depth")["outcome"],
        "warn"
    );
    assert!(
        (row(&rows, "marpol/19.6.1/double-bottom-depth")["limit"]
            .as_f64()
            .unwrap()
            - 0.76)
            .abs()
            < 1e-12
    );
    // The limits worked out by hand.
    assert_eq!(row(&rows, "adn/9.3.2.11.7/wing-width-min")["limit"], 1.0);
    assert!(
        (row(&rows, "marpol/19.6.2/double-side-width")["limit"]
            .as_f64()
            .unwrap()
            - 0.76)
            .abs()
            < 1e-12
    );
}

// --- NORMAM-202 ---------------------------------------------------------------------------------

#[test]
fn normam_202_double_bottom_is_the_greater_of_b_over_15_and_0_76_and_the_side_1_00_m() {
    let sets: &[(&str, &str)] = &[("normam-202", "1.0.0")];
    // B = 15 m: B/15 = 1.0 m.
    let mut c = case();
    c.beam_m = 15.0;
    c.layout.double_bottom_m = 1.0;
    c.layout.double_side_m = 1.0;
    let rows = trace(&c, sets);
    assert_eq!(
        outcomes(&rows, "normam/5.23.6(a)(II)/double-bottom"),
        ["pass"]
    );
    assert_eq!(
        outcomes(&rows, "normam/5.23.6(a)(II)/double-side"),
        ["pass"]
    );
    c.layout.double_bottom_m = 0.99;
    c.layout.double_side_m = 0.99;
    let rows = trace(&c, sets);
    assert_eq!(
        outcomes(&rows, "normam/5.23.6(a)(II)/double-bottom"),
        ["warn"]
    );
    assert_eq!(
        outcomes(&rows, "normam/5.23.6(a)(II)/double-side"),
        ["warn"]
    );
    // B = 10 m: 0.76 m governs.
    c.beam_m = 10.0;
    c.layout.double_bottom_m = 0.76;
    let rows = trace(&c, sets);
    assert_eq!(
        outcomes(&rows, "normam/5.23.6(a)(II)/double-bottom"),
        ["pass"]
    );
    // The rest is listed, not evaluated.
    for r in [
        "normam/5.23.8/standard-damage",
        "normam/5.23.9/damage-criteria",
        "normam/5.23.3(l)/opening-setback",
    ] {
        assert_eq!(outcomes(&rows, r), ["not evaluated"], "{r}");
    }
}

// --- the sets as data -----------------------------------------------------------------------------

fn set(id: &str) -> Value {
    let path = format!("{}/rules/{id}/1.0.0.json", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn every_rule_is_unverified_and_warns_and_cites_a_paragraph_and_a_page() {
    for id in ["adn-tank", "marpol-i", "normam-202"] {
        let s = set(id);
        assert_eq!(s["version"], "1.0.0");
        assert_eq!(s["status"], "draft");
        for src in s["sources"].as_array().unwrap() {
            assert_eq!(src["sha256"].as_str().unwrap().len(), 64, "{id}");
        }
        for r in s["rules"].as_array().unwrap() {
            let rid = r["id"].as_str().unwrap();
            assert_eq!(r["verified"], false, "{rid}");
            assert!(
                ["warn", "note"].contains(&r["consequence"].as_str().unwrap()),
                "{rid}"
            );
            assert!(
                r["source"]["locator"]
                    .as_str()
                    .is_some_and(|l| !l.is_empty()),
                "{rid}"
            );
            assert!(
                r["source"]["page"]
                    .as_str()
                    .is_some_and(|l| l.contains("p.")),
                "{rid}"
            );
            assert_eq!(r["codified"]["review"], "#338", "{rid}");
            // A Rule that is not evaluated says what is missing.
            if r["evaluated"] == false {
                assert!(r["gap"].as_str().is_some_and(|g| g.len() > 10), "{rid}");
            }
        }
    }
}

#[test]
fn normam_202_is_never_in_a_default_stack_and_marpol_names_coastal_service() {
    for containment in [
        "box_hold",
        "sloped_hopper_hold",
        "type_c_tanks",
        "integral_tanks",
    ] {
        let ids = ovcd_format::trace::default_stack_ids(Some(containment));
        assert!(!ids.contains(&"normam-202"), "{containment}");
    }
    assert_eq!(
        set("marpol-i")["applies_when"]["service_area"],
        serde_json::json!(["coastal-and-river"])
    );
}

// --- through a real run -----------------------------------------------------------------------------

#[test]
fn a_crude_option_gets_the_type_c_rules_from_its_crude_grade_and_the_type_n_rules_do_not_apply() {
    use ovcd_format::{Profile, RunOutcome, check, run_design_with};
    let path = format!(
        "{}/tests/fixtures/crude_integral_tanks_pushed.vdesign",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut design = check(
        &std::fs::read_to_string(path).unwrap(),
        &Profile::built_in(),
    )
    .design
    .unwrap();
    let lib = Library::built_in();
    ovcd_format::trace::add_rule_set(&mut design.options[0], &lib, "adn-tank@1.0.0").unwrap();
    let RunOutcome::Results { results, .. } = run_design_with(&design, &lib, &[]).remove(0).outcome
    else {
        panic!("a failed run");
    };
    let rows = results["rule_trace"]["rows"].as_array().unwrap();
    let of = |id: &str| -> Vec<&Value> { rows.iter().filter(|r| r["rule"] == id).collect() };
    // The default Crude Grade has unknown hazard flags, so its ADN Tank Vessel Type is C.
    assert_eq!(of("adn/3.2.3.3/vessel-type")[0]["value"], "C");
    assert_eq!(of("adn/9.3.2.11.7/wing-width-min")[0]["outcome"], "pass");
    assert_eq!(
        of("adn/9.3.3.11.7/wing-width-min")[0]["outcome"],
        "not applicable"
    );
    // 4 tank positions of 16.7 m each side of a centreline bulkhead: the 0.20 L length limit is 15.3 m.
    assert!(
        of("adn/9.3.2.11.1(d)/cargo-tank-length")
            .iter()
            .all(|r| r["outcome"] == "warn")
    );
    assert_eq!(of("adn/9.3.2.11.1(d)/cargo-tank-length").len(), 8);
    // The results hash does not move with a Rule Set.
    let plain = run_design_with(
        &check(
            &std::fs::read_to_string(format!(
                "{}/tests/fixtures/crude_integral_tanks_pushed.vdesign",
                env!("CARGO_MANIFEST_DIR")
            ))
            .unwrap(),
            &Profile::built_in(),
        )
        .design
        .unwrap(),
        &lib,
        &[],
    )
    .remove(0);
    let RunOutcome::Results { hash, .. } = plain.outcome else {
        panic!()
    };
    let RunOutcome::Results { hash: with, .. } =
        run_design_with(&design, &lib, &[]).remove(0).outcome
    else {
        panic!()
    };
    assert_eq!(hash, with);
}
