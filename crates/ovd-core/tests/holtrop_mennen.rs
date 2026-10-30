//! Seam: the public `ovd_core::holtrop_mennen` API (issue #723).
//! Label: sourced - J. Holtrop and G.G.J. Mennen, An approximate power prediction method,
//! International Shipbuilding Progress 29 (335), 1982, section 5 (numerical example, the table of
//! coefficients and powering characteristics); the 1984 form factor against OpenTNSim (TU Delft,
//! MIT), `opentnsim/energy/calculations.py::calculate_viscous_resistance` at commit cb6bdda; the
//! envelope from Ortigosa, López and García 2009, Journal of Maritime Research VI(III), Table 1.
//!
//! The example ship: L 205 m, B 32 m, T_F = T_A 10 m, ∇ 37,500 m³, lcb -0.75 % of L, A_BT 20 m²,
//! h_B 4 m, C_M 0.98, C_WP 0.75, A_T 16 m², S_APP 50 m² with 1 + k2 1.50, C_stern +10, 25 kn.
//! Sea water: ρ 1025 kg/m³, ν 1.1883e-6 m²/s (15 °C, the paper's trial condition), g 9.80665 m/s².
//!
//! Each figure is checked to the relative residual named beside it. The paper prints 4 or 5
//! significant figures, so 0.1 % covers its rounding. One figure is looser, and why: the paper's
//! R_A (221.98 kN) is 0.6 % above 0.5 ρ V² S C_A with its own printed C_A (0.000352); the formula
//! gives C_A 0.0003525, as printed, so the residual is in the paper, not in the transcription.

use ovd_core::constants::{GRAVITY, KNOTS_TO_MS};
use ovd_core::holtrop_mennen::{
    Components, Flow, HOLTROP_FULL_FORM, HullInputs, Variant, components, holtrop_envelope,
    wetted_surface_estimate,
};

fn example_hull(stern_shape: f64) -> HullInputs {
    let (l, b, t, vol, cm, cwp, abt) = (205.0, 32.0, 10.0, 37500.0, 0.98, 0.75, 20.0);
    HullInputs {
        lwl: l,
        beam: b,
        draught: t,
        draught_fwd: t,
        volume_m3: vol,
        lcb_pct: -0.75,
        cm,
        cwp,
        wetted_surface_m2: wetted_surface_estimate(l, b, t, vol, cm, cwp, abt),
        bulb_area_m2: abt,
        bulb_centre_m: 4.0,
        transom_area_m2: 16.0,
        appendage_area_m2: 50.0,
        appendage_factor: 1.5,
        stern_shape,
    }
}

fn example_flow() -> Flow {
    Flow {
        speed_ms: 25.0 * KNOTS_TO_MS,
        rho: 1025.0,
        nu: 1.1883e-6,
        g: GRAVITY,
    }
}

fn example(variant: Variant) -> Components {
    components(&example_hull(10.0), example_flow(), variant)
}

#[track_caller]
fn within(value: f64, paper: f64, rel: f64) {
    let r = (value - paper) / paper;
    assert!(
        r.abs() <= rel,
        "{value} vs {paper}: residual {:.4} %",
        100.0 * r
    );
}

#[test]
fn the_1982_example_hull_coefficients() {
    let c = example(Variant::Hm1982);
    within(c.froude, 0.2868, 1e-3);
    within(c.cp, 0.5833, 1e-3);
    within(c.run_length_m, 81.385, 1e-3);
    within(c.form_factor, 1.156, 1e-3);
    within(c.cf, 0.001390, 1e-3);
    within(c.c7, 0.1561, 1e-3);
    within(c.entrance_angle_deg, 12.08, 1e-3);
    within(c.c1, 1.398, 1e-3);
    within(c.c3, 0.02119, 1e-3);
    within(c.c2, 0.7595, 1e-3);
    within(c.c5, 0.9592, 1e-3);
    within(c.m1, -2.1274, 1e-3);
    within(c.c15, -1.69385, 1e-6);
    within(c.m_hump, -0.17087, 1e-3);
    within(c.lambda, 0.6513, 1e-3);
    within(c.pb, 0.6261, 1e-3);
    within(c.fni, 1.5084, 1e-3);
    within(c.fnt, 5.433, 1e-3);
    within(c.ca, 0.000352, 2e-3);
}

#[test]
fn the_1982_example_resistance_components_and_total() {
    let c = example(Variant::Hm1982);
    let s = example_hull(10.0).wetted_surface_m2;
    within(s, 7381.45, 1e-3);
    within(c.r_f_kn, 869.63, 1e-3);
    within(c.r_app_kn, 8.83, 1e-3);
    within(c.r_w_kn, 557.11, 1e-3);
    within(c.r_b_kn, 0.049, 1e-2); // printed to 2 significant figures
    assert_eq!(c.r_tr_kn, 0.0); // F_nT 5.43 > 5: c6 = 0
    within(c.r_a_kn, 221.98, 1e-2); // the paper's own 0.6 % (see the header)
    within(c.total_kn, 1793.26, 1e-3);
    // P_E = R V.
    within(c.total_kn * example_flow().speed_ms, 23063.0, 1e-3);
}

#[test]
fn the_1984_form_factor_agrees_with_opentnsim() {
    // OpenTNSim takes c14 = 1 + 0.0011 C_stern where the 1984 paper prints 1 + 0.011 C_stern, so
    // the two are compared at C_stern 0. OpenTNSim rounds the exponents (0.487, 1.068, 0.461,
    // 0.122, 0.365, -0.604): 0.05 % covers the rounding. OpenTNSim printed 1.160032248058631.
    let c = components(&example_hull(0.0), example_flow(), Variant::Hm1984);
    within(c.form_factor, 1.160032248058631, 5e-4);
}

#[test]
fn the_1984_wave_term_stays_close_to_the_1982_one_at_the_example_speed() {
    // Label: property. Holtrop 1984: the low-speed formula "closely resembles" the 1982 one and
    // changes only the coefficient of the humps and hollows.
    let a = example(Variant::Hm1982).r_w_kn;
    let b = example(Variant::Hm1984).r_w_kn;
    assert!(((b - a) / a).abs() < 0.02, "{a} vs {b}");
}

#[test]
fn a_finer_hull_of_the_same_dimensions_has_less_resistance() {
    // Label: property. The form term the bluff-barge fit lacks (issue #723).
    let flow = Flow {
        speed_ms: 10.0 * KNOTS_TO_MS,
        rho: 1025.0,
        nu: 1.19e-6,
        g: GRAVITY,
    };
    let mut last = f64::INFINITY;
    for cb in [0.84, 0.80, 0.76, 0.72] {
        let (l, b, t, cm) = (160.0, 28.0, 9.5, 0.99);
        let h = HullInputs::plain(l, b, t, cb * l * b * t, cm, (1.0 + 2.0 * cb) / 3.0);
        let r = components(&h, flow, Variant::Hm1984).total_kn;
        assert!(r < last, "Cb {cb}: {r} kN, not below {last} kN");
        last = r;
    }
}

#[test]
fn the_envelope_names_each_parameter_out_of_range() {
    assert!(HOLTROP_FULL_FORM.outside(0.20, 0.80, 6.0, 3.0).is_empty());
    let out = HOLTROP_FULL_FORM.outside(0.25, 0.96, 4.3, 6.7);
    let names: Vec<&str> = out.iter().map(|o| o.name).collect();
    assert_eq!(names, ["Fn", "Cp", "L/B", "B/T"]);
    // The general cargo row below Cp 0.725, the full-form row above.
    assert_eq!(holtrop_envelope(0.70).cp, (0.58, 0.72));
    assert_eq!(holtrop_envelope(0.80).cp, (0.73, 0.85));
}
