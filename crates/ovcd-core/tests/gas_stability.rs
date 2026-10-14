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

/// Python 3.12 `repr` escapes what `str.isprintable` rejects: `\xhh` below U+0100, `\uhhhh` below
/// U+10000, `\Uhhhhhhhh` above. Each expected message below was printed by the frozen Python.
#[test]
fn an_unknown_criteria_set_escapes_non_printable_non_ascii_text_like_python() {
    let msg = |s: &str| {
        run(&narrow_deep(), PAYLOAD, s)
            .unwrap_err()
            .message()
            .to_string()
    };
    assert_eq!(msg("\u{a0}"), "unknown criteria set '\\xa0'");
    assert_eq!(msg("\u{ad}"), "unknown criteria set '\\xad'");
    assert_eq!(msg("\u{2028}"), "unknown criteria set '\\u2028'");
    assert_eq!(msg("a\u{200b}b"), "unknown criteria set 'a\\u200bb'");
    assert_eq!(msg("\u{feff}"), "unknown criteria set '\\ufeff'");
    assert_eq!(msg("\u{e000}"), "unknown criteria set '\\ue000'");
    assert_eq!(msg("\u{378}"), "unknown criteria set '\\u0378'");
    assert_eq!(msg("\u{e0001}"), "unknown criteria set '\\U000e0001'");
    assert_eq!(msg("\u{10ffff}"), "unknown criteria set '\\U0010ffff'");
    // printable non-ASCII text stays as it is
    assert_eq!(
        msg("\u{e9}\u{4e2d}\u{1f600}"),
        "unknown criteria set '\u{e9}\u{4e2d}\u{1f600}'"
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
fn a_radius_whose_square_overflows_is_an_overflow_error() {
    // Python: segment_fill(1e155, 0.5) raises OverflowError(34, 'Numerical result out of range').
    let e = segment_fill(1e155, 0.5).unwrap_err();
    assert_eq!(e, PyError::Overflow);
    assert_eq!(e.type_name(), "OverflowError");
    assert_eq!(e.message(), "(34, 'Numerical result out of range')");
    // The clamps return before the square, as in Python.
    assert_eq!(segment_fill(1e155, 1.0), Ok((2e155, 1e155)));
    assert_eq!(segment_fill(1e155, 0.0), Ok((0.0, 0.0)));
    // Just below the limit the square is finite.
    assert!(segment_fill(1e154, 0.5).is_ok());
}

// --- boundaries: inputs that sit exactly on a comparison. Expected values: frozen Python. ---

#[test]
fn a_draught_exactly_equal_to_the_depth_does_not_float() {
    // Light draught is 1.3461915483210831; with depth set to that value, T >= D holds.
    let mut r = narrow_deep();
    r.gas_basis.as_mut().unwrap().depth = 1.3461915483210831;
    let a = run(&r, PAYLOAD, DEFAULT_CRITERIA_SET).unwrap();
    let light = &a.conditions[2];
    assert_eq!(light.draught_m, 1.3461915483210831);
    assert!(light.gz.is_empty());
    assert_eq!(light.criteria.len(), 1);
    assert_eq!(light.criteria[0].name, "Floats with freeboard");
    assert!(!light.criteria[0].ok);
}

#[test]
fn a_tank_exactly_seven_tenths_of_the_beam_is_not_wide() {
    // 0.70 * 10.0 == 7.0 in floating point, so `od > 0.70 * B` is false: one ADN criterion.
    let mut r = narrow_deep();
    {
        let b = r.gas_basis.as_mut().unwrap();
        b.beam = 10.0;
        b.outer_diameter = 7.0;
        b.inner_diameter = 6.25;
    }
    let a = run(&r, PAYLOAD, "ADN_9_3_1_14_2").unwrap();
    for c in &a.conditions {
        assert_eq!(c.criteria.len(), 1);
        assert_eq!(c.criteria[0].name, "Initial GM (fluid) > 0");
    }
    // Just wider than 0.70 B: the three-criterion set.
    r.gas_basis.as_mut().unwrap().outer_diameter = 7.0000001;
    let a = run(&r, PAYLOAD, "ADN_9_3_1_14_2").unwrap();
    for c in &a.conditions {
        assert_eq!(c.criteria.len(), 3);
    }
}

#[test]
fn a_weight_split_difference_of_exactly_one_tonne_adds_no_note() {
    let mut r = narrow_deep();
    {
        let b = r.gas_basis.as_mut().unwrap();
        b.steel_top_down = 252.0;
        b.tank_total = 179.0;
        b.insulation = 17.0;
        b.piping_machinery = 17.0;
        b.outfitting = 15.0;
    }
    // The split sums to 480 t exactly.
    r.lightship_t = 481.0;
    assert_eq!(
        run(&r, 484.0, DEFAULT_CRITERIA_SET)
            .unwrap()
            .limitations
            .len(),
        2
    );
    r.lightship_t = 481.5;
    let a = run(&r, 484.0, DEFAULT_CRITERIA_SET).unwrap();
    assert_eq!(
        a.limitations[2],
        "Weight split sums to 480 t vs lightship 482 t."
    );
}

#[test]
fn block_coefficient_and_water_density_both_scale_the_draught() {
    // Python: assess(r, payload, block_coefficient=0.8, water_density=1.025)
    let a = assess(&narrow_deep(), PAYLOAD, 0.8, 1.025, DEFAULT_CRITERIA_SET).unwrap();
    let draughts: Vec<f64> = a.conditions.iter().map(|c| c.draught_m).collect();
    let want = [2.8015259766748537, 2.0984842176501832, 1.3954424586255132];
    for (g, w) in draughts.iter().zip(want) {
        assert!(close(*g, w), "{g} vs {w}");
    }
}

#[test]
fn criteria_set_follows_the_ruleset() {
    assert_eq!(criteria_set_for("ADN_G"), "ADN_9_3_1_14_2");
    assert_eq!(criteria_set_for("IGC_2G"), "IS_CODE_2008");
    assert_eq!(criteria_set_for(""), "IS_CODE_2008");
}

#[test]
fn box_gz_with_no_displaced_area_has_no_centre_of_buoyancy() {
    // Python's area guard (`abs(a) < 1e-12`) gives a centroid of 0, so GZ = -kg * sin(phi).
    assert_eq!(box_gz(2.0, 1.0, 0.0, 0.3, 10.0), Ok(-0.0520944533000791));
    assert_eq!(box_gz(2.0, 1.0, -0.5, 0.3, 10.0), Ok(-0.0520944533000791));
    // An area below the guard, not only exactly zero.
    assert_eq!(box_gz(1.0, 1.0, 1e-13, 0.0, 10.0), Ok(0.0));
    assert_eq!(box_gz(10.0, 3.0, 1e-14, 1.0, 20.0), Ok(-0.3420201433256687));
}

#[test]
fn segment_fill_keeps_the_centroid_when_r_squared_times_the_angle_term_is_even() {
    // r^2 * (t - sin t) is an exact even number here; the area must still divide by two.
    assert_eq!(
        segment_fill(0.7978845608028654, 0.5),
        Ok((0.7978845608028653, 0.45925183582024687))
    );
    assert_eq!(
        segment_fill(1.5957691216057308, 0.5),
        Ok((1.5957691216057306, 0.9185036716404937))
    );
    // The centroid here is ill-conditioned (last digits differ), so compare within tolerance.
    let (h, y) = segment_fill(2.5231325220201604, 0.1).unwrap();
    assert!(close(h, 0.7896172845893725));
    assert!(close(y, 0.4691096033520923));
}

#[test]
fn an_assessment_where_every_criterion_passes_reports_ok() {
    // Reference Results assess/CN_II_IS_CODE_2008 and assess/sweep/003: every criterion ok.
    let a = run(&narrow_deep(), 0.0, DEFAULT_CRITERIA_SET).unwrap();
    assert!(a.conditions.iter().all(|c| c.criteria.iter().all(|k| k.ok)));
    assert!(a.ok());
}

#[test]
fn the_free_surface_chord_is_not_computed_for_an_empty_tank() {
    // Python skips the chord unless 0 < fill < 1. With no cargo the fill is 0, so an inner radius
    // of 1e160 (whose square overflows) is not squared there. Python returns ok, Light
    // gm_fluid_m = -2.625e+159.
    let mut r = narrow_deep();
    {
        let b = r.gas_basis.as_mut().unwrap();
        b.loa = 100.0;
        b.beam = 12.0;
        b.depth = 4.0;
        b.double_bottom_height = 0.5;
        b.tank_support_height = 0.2;
        b.outer_diameter = 2e160;
        b.inner_diameter = 2e160;
        b.total_tank_volume = 1000.0;
        b.n_tanks_total = 2;
        b.cylinder_length = 40.0;
        b.steel_top_down = 500.0;
        b.tank_total = 200.0;
        b.insulation = 10.0;
        b.piping_machinery = 50.0;
        b.outfitting = 40.0;
    }
    r.cargo_density_t_per_m3 = 0.5;
    r.lightship_t = 800.0;
    let a = run(&r, 0.0, DEFAULT_CRITERIA_SET).unwrap();
    assert!(close(a.conditions[2].gm_fluid_m, -2.625e159));
}
