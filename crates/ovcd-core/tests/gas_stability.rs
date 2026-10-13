//! Branches of `gas_stability` that the parity cases do not reach. Expected values were printed by
//! the frozen Python (`parity/frozen_python`), with its messages as Python 3.12 words them. The
//! parity cases themselves run in `crates/ovcd-format/tests/gas_stability.rs`.

use ovcd_core::gas_stability::{
    DEFAULT_BLOCK_COEFFICIENT, DEFAULT_CRITERIA_SET, DEFAULT_WATER_DENSITY, GasBasis,
    GasCapacityResult, PyError, PyNum, assess, box_gz, criteria_set_for, segment_fill,
};

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0)
}

/// The "narrow deep" case of the Python tests: 60 x 7 x 6 m, LPG propane, two tanks.
fn narrow_deep() -> GasCapacityResult {
    GasCapacityResult {
        gas_basis: Some(GasBasis {
            loa: 60.0,
            beam: 7.0,
            depth: 6.0,
            double_bottom_height: 0.0,
            tank_support_height: 0.5,
            outer_diameter: 6.1,
            inner_diameter: 5.35,
            total_tank_volume: 850.4955629214109,
            n_tanks_total: 2,
            cylinder_length: 15.35,
            steel_top_down: 252.0,
            tank_total: 179.0007365011633,
            insulation: 17.264591011876167,
            piping_machinery: 17.240029460046532,
            outfitting: 15.085025777540716,
        }),
        cargo_density_t_per_m3: 0.581,
        lightship_t: 480.59038275062665,
        has_aft_arrangement: false,
    }
}

const PAYLOAD: f64 = 484.2551636161929;

fn run(
    r: &GasCapacityResult,
    payload: f64,
    cset: &str,
) -> Result<ovcd_core::gas_stability::StabilityAssessment, PyError> {
    assess(
        r,
        payload,
        DEFAULT_BLOCK_COEFFICIENT,
        DEFAULT_WATER_DENSITY,
        cset,
    )
}

#[test]
fn a_barge_that_does_not_float_gets_one_failed_criterion_and_no_curve() {
    let a = run(&narrow_deep(), 5000.0, DEFAULT_CRITERIA_SET).unwrap();
    let laden = &a.conditions[0];
    assert_eq!(laden.name, "Laden");
    assert!(close(laden.draught_m, 15.35179378921744));
    assert!(laden.gz.is_empty());
    assert_eq!(laden.deck_edge_deg, None);
    assert_eq!(laden.gm_solid_m, -1.0);
    assert_eq!(laden.gm_fluid_m, -1.0);
    assert_eq!(laden.criteria.len(), 1);
    let c = &laden.criteria[0];
    assert_eq!(c.name, "Floats with freeboard");
    assert_eq!(c.source, "draught \u{2265} depth");
    assert!(!c.ok);
    assert_eq!(c.limit, 6.0);
    assert_eq!(c.value, PyNum::Float(laden.draught_m));
    assert!(close(a.conditions[1].draught_m, 8.348992668769263));
    assert!(a.conditions[1].gz.is_empty());
    // the light condition still floats, so it is assessed in full
    let light = &a.conditions[2];
    assert_eq!(light.gz.len(), 61);
    assert!(close(light.gm_fluid_m, 0.3270324499662216));
    assert_eq!(light.max_gz_deg, 27);
    assert!(close(light.deck_edge_deg.unwrap(), 53.054191472791246));
    assert!(!a.ok());
    assert!(!a.conditions[0].ok());
    assert!(light.ok());
    // lowest fluid GM; the two that do not float tie at -1, and the first of them wins
    assert_eq!(a.governing().name, "Laden");
}

#[test]
fn the_weight_split_note_appears_only_above_one_tonne_of_difference() {
    let mut r = narrow_deep();
    r.lightship_t += 1.5;
    let a = run(&r, PAYLOAD, DEFAULT_CRITERIA_SET).unwrap();
    assert_eq!(a.limitations.len(), 3);
    assert_eq!(
        a.limitations[2],
        "Weight split sums to 481 t vs lightship 482 t."
    );
    r.lightship_t -= 0.5;
    assert_eq!(
        run(&r, PAYLOAD, DEFAULT_CRITERIA_SET)
            .unwrap()
            .limitations
            .len(),
        2
    );
}

#[test]
fn the_stern_block_note_follows_the_aft_arrangement() {
    let mut r = narrow_deep();
    r.has_aft_arrangement = true;
    let a = run(&r, PAYLOAD, DEFAULT_CRITERIA_SET).unwrap();
    assert_eq!(a.limitations.len(), 3);
    assert!(a.limitations[2].starts_with("The self-propelled stern block's mass"));
}

#[test]
fn a_gas_assessment_with_no_cargo_has_the_same_three_conditions() {
    let a = run(&narrow_deep(), 0.0, DEFAULT_CRITERIA_SET).unwrap();
    for c in &a.conditions {
        assert!(close(c.draught_m, 1.3461915483210831));
        assert_eq!(c.fill_fraction, 0.0);
    }
}

#[test]
fn no_tank_volume_means_no_fill_and_no_free_surface() {
    let mut r = narrow_deep();
    r.gas_basis.as_mut().unwrap().total_tank_volume = 0.0;
    let a = run(&r, PAYLOAD, DEFAULT_CRITERIA_SET).unwrap();
    assert_eq!(a.conditions[0].fill_fraction, 0.0);
    assert_eq!(a.conditions[0].free_surface_m, 0.0);
    assert!(close(a.conditions[0].gm_fluid_m, 0.7397881002462028));
    assert_eq!(a.conditions[0].max_gz_deg, 60);
}

#[test]
fn a_zero_divisor_is_a_zero_division_error() {
    let mut r = narrow_deep();
    r.cargo_density_t_per_m3 = 0.0;
    assert_eq!(
        run(&r, PAYLOAD, DEFAULT_CRITERIA_SET),
        Err(PyError::ZeroDivision)
    );
    assert_eq!(
        assess(&narrow_deep(), PAYLOAD, 0.0, 1.0, DEFAULT_CRITERIA_SET),
        Err(PyError::ZeroDivision)
    );
    let e = PyError::ZeroDivision;
    assert_eq!(e.type_name(), "ZeroDivisionError");
    assert_eq!(e.message(), "float division by zero");
}

#[test]
fn a_bulk_result_is_refused() {
    let mut r = narrow_deep();
    r.gas_basis = None;
    let e = run(&r, PAYLOAD, DEFAULT_CRITERIA_SET).unwrap_err();
    assert_eq!(e.type_name(), "ValueError");
    assert_eq!(
        e.message(),
        "assess() needs a gas BargeCapacityResult (gas_basis is None)"
    );
}

#[test]
fn an_unknown_criteria_set_names_itself_the_way_python_repr_does() {
    let msg = |s: &str| {
        run(&narrow_deep(), PAYLOAD, s)
            .unwrap_err()
            .message()
            .to_string()
    };
    assert_eq!(msg("IMO_2020"), "unknown criteria set 'IMO_2020'");
    assert_eq!(msg("it's"), "unknown criteria set \"it's\"");
    assert_eq!(msg("it's \""), "unknown criteria set 'it\\'s \"'");
    assert_eq!(
        msg("a\\b\n\r\t\u{1}\u{7f}\u{e9}"),
        "unknown criteria set 'a\\\\b\\n\\r\\t\\x01\\x7f\u{e9}'"
    );
}

#[test]
fn an_unknown_criteria_set_is_not_an_error_when_nothing_floats() {
    let mut r = narrow_deep();
    r.gas_basis.as_mut().unwrap().depth = 0.1;
    // every condition has draught >= depth, so no criteria set is ever read
    let a = run(&r, PAYLOAD, "anything").unwrap();
    assert!(a.conditions.iter().all(|c| c.gz.is_empty()));
    assert_eq!(a.criteria_set, "anything");
}

#[test]
fn box_gz_at_no_heel_is_zero_and_an_infinite_heel_is_a_domain_error() {
    assert_eq!(box_gz(10.0, 5.0, 2.0, 3.0, 0.0), Ok(0.0));
    assert_eq!(box_gz(10.0, 5.0, 2.0, 3.0, -0.0), Ok(0.0));
    let e = box_gz(10.0, 5.0, 2.0, 3.0, f64::INFINITY).unwrap_err();
    assert_eq!(e, PyError::Value("math domain error".into()));
    assert!(box_gz(10.0, 5.0, 2.0, 3.0, f64::NAN).unwrap().is_nan());
}

#[test]
fn segment_fill_clamps_and_keeps_python_edge_behaviour() {
    assert_eq!(segment_fill(2.0, 0.0), Ok((0.0, 0.0)));
    assert_eq!(segment_fill(2.0, -0.5), Ok((0.0, 0.0)));
    assert_eq!(segment_fill(2.0, 1.0), Ok((4.0, 2.0)));
    assert_eq!(segment_fill(2.0, 1.5), Ok((4.0, 2.0)));
    let (h, y) = segment_fill(2.0, 1e-10).unwrap();
    assert!(close(h, 6.055650021430381e-07));
    assert!(close(y, 3.636841054976969e-07));
    // Python divides by zero here: sin(t) == t for the smallest t the bisection reaches
    assert_eq!(segment_fill(2.0, 1e-30), Err(PyError::ZeroDivision));
    assert_eq!(segment_fill(2.0, f64::NAN), Err(PyError::ZeroDivision));
}

#[test]
fn criteria_set_follows_the_ruleset() {
    assert_eq!(criteria_set_for("ADN_G"), "ADN_9_3_1_14_2");
    assert_eq!(criteria_set_for("IGC_2G"), "IS_CODE_2008");
    assert_eq!(criteria_set_for(""), "IS_CODE_2008");
}
