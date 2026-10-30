//! Seam: the Propulsor model (issue #351, spec 0007 "Propulsor model").
//!
//! The bollard pull of an open disc with kappa_b 0.77 at rho 1,025 kg/m^3 is checked against the
//! Wartsila estimate BP = 0.9 x k2 x (D P_D)^(2/3) / 9.81 t with k2 = 1 for an open propeller
//! (Wartsila Encyclopedia, "Bollard pull", as the research note sec. 3.2 gives it). The ideal
//! efficiency is the actuator-disc result eta_i = 2 / (1 + sqrt(1 + C_T)) (spec 0007).
//!
//! Label: sourced - spec 0007 "Propulsor model" and docs/research/stern-and-tugs.md sec. 3.2 (Wartsila bollard pull estimate); the hand values are worked here.

use ovd_core::propulsor::{
    PropulsorInputs, PropulsorType, blade_load, bollard_pull, ideal_efficiency, performance,
};

fn open(d: f64) -> PropulsorInputs {
    PropulsorInputs {
        propulsor_type: PropulsorType::ShaftOpen,
        count: 1,
        diameter_m: d,
        wake: 0.2,
        kappa: 1.0,
        kappa_b: 0.77,
        rho_kg_m3: 1025.0,
    }
}

#[test]
fn an_open_disc_with_kappa_b_0_77_is_the_wartsila_estimate_within_0_5_percent() {
    for (d, p) in [(1.2, 441.0), (2.4, 1900.0), (2.8, 1900.0), (3.0, 2500.0)] {
        let wartsila = 0.9 * libm::pow(d * p, 2.0 / 3.0) / 9.81;
        let bp = bollard_pull(&open(d), p);
        assert!(
            ((bp - wartsila) / wartsila).abs() < 0.005,
            "{d} {p}: {bp} {wartsila}"
        );
    }
    // Two units pull twice one.
    let mut two = open(2.0);
    two.count = 2;
    assert!((bollard_pull(&two, 1000.0) - 2.0 * bollard_pull(&open(2.0), 1000.0)).abs() < 1e-12);
}

#[test]
fn the_ideal_efficiency_tends_to_1_at_no_load_and_falls_as_the_load_rises() {
    assert!((ideal_efficiency(1e-9, false) - 1.0).abs() < 1e-9);
    let mut last = 1.0;
    for c_t in [0.1, 0.5, 1.0, 2.0, 5.0, 20.0] {
        let e = ideal_efficiency(c_t, false);
        assert!(e < last, "{c_t}");
        // The duct lowers the propeller's load, so its disc is more efficient.
        assert!(ideal_efficiency(c_t, true) > e, "{c_t}");
        last = e;
    }
    // Hand value: C_T = 3 gives 2 / (1 + 2) = 2/3.
    assert!((ideal_efficiency(3.0, false) - 2.0 / 3.0).abs() < 1e-12);
}

#[test]
fn the_curve_gives_thrust_advance_speed_and_delivered_power_per_speed() {
    let inp = PropulsorInputs {
        count: 2,
        ..open(2.0)
    };
    let curve = performance(&inp, &[0.0, 10.0], &[0.0, 50.0]);
    assert_eq!(curve.points[0].delivered_power_kw, 0.0);
    let p = &curve.points[1];
    let v = 10.0 * 1852.0 / 3600.0;
    assert!((p.thrust_per_unit_kn - 50.0 / (2.0 * 0.8)).abs() < 1e-12);
    assert!((p.advance_speed_m_s - 0.8 * v).abs() < 1e-12);
    let area = std::f64::consts::PI;
    let c_t = p.thrust_per_unit_kn * 1000.0 / (0.5 * 1025.0 * area * (0.8 * v) * (0.8 * v));
    assert!((p.thrust_loading - c_t).abs() < 1e-9);
    let eta_d = ideal_efficiency(c_t, false) * 0.98;
    assert!((p.eta_d - eta_d).abs() < 1e-12);
    assert!((p.delivered_power_kw - 50.0 * v / eta_d).abs() < 1e-9);
}

#[test]
fn the_blade_load_is_the_power_over_the_disc_with_the_veth_limits() {
    let (load, limit) = blade_load(&open(2.0), 1000.0);
    assert!((load - 1000.0 / std::f64::consts::PI).abs() < 1e-9);
    assert_eq!(limit, 400.0);
    let ducted = PropulsorInputs {
        propulsor_type: PropulsorType::AzimuthDucted,
        ..open(2.0)
    };
    assert_eq!(blade_load(&ducted, 1000.0).1, 500.0);
    assert_eq!(
        PropulsorType::from_key("shaft_nozzle"),
        Some(PropulsorType::ShaftNozzle)
    );
    assert_eq!(PropulsorType::from_key("unspecified"), None);
}
