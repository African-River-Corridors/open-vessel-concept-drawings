//! Seam: the Stern Form geometry (issue #350, spec 0007 "Stern geometry").
//!
//! The closed forms are checked against a strip integral of the same shape (2,000 strips) at three
//! draughts; the Heuser run lengths and the D_max rules against spec 0007 by hand: single screw
//! 1.9 D + 1.85 B, twin converging 2.5 D + 1.65 B, twin parallel 3.2 D + 1.45 B (Rotteveel 2019
//! sec. 2.3.1); D_max 0.80 T under a tunnel, 0.65 T at a transom or pram stern.
//!
//! Label: sourced - spec 0007 "Stern geometry" (Heuser in Rotteveel 2019 sec. 2.3.1; MAN d/D below about 0.65); the strip integrals and the hand values are worked here.

use ovcd_core::hull::HullGeometry;
use ovcd_core::stern_form::{
    HullDims, SternForm, SternFormInputs, geometry, heuser_run_length, removed_by_strips,
    tunnel_fit_diameter, tunnel_width,
};

fn inputs(form: SternForm) -> SternFormInputs {
    SternFormInputs {
        form,
        run_length_m: None,
        transom_height_m: None,
        tunnel_crown_height_m: None,
        propulsor_count: 2,
        propeller_diameter_m: None,
        parallel_shafts: true,
    }
}

const HULL: HullDims = HullDims {
    loa: 110.0,
    beam: 11.4,
    depth: 4.5,
    design_draught: 3.5,
};

#[test]
fn each_form_matches_a_strip_integral_within_0_1_percent_at_three_draughts() {
    for form in [SternForm::Transom, SternForm::Pram, SternForm::Tunnel] {
        let g = geometry(&inputs(form), &HULL);
        for t in [1.5, 2.5, 3.5] {
            let (v, m) = g.removed(t);
            let (vs, ms) = removed_by_strips(&g, t, 2000);
            assert!(v > 0.0, "{form:?} at {t}");
            assert!(((v - vs) / vs).abs() < 1e-3, "{form:?} at {t}: {v} {vs}");
            assert!(((m - ms) / ms).abs() < 1e-3, "{form:?} at {t}: {m} {ms}");
        }
    }
}

#[test]
fn the_legacy_transom_removes_nothing_and_matches_the_raked_box() {
    let mut i = inputs(SternForm::Transom);
    i.run_length_m = Some(0.0);
    let g = geometry(&i, &HULL);
    assert!(g.is_legacy());
    assert_eq!(g.removed(3.0), (0.0, 0.0));
    assert_eq!(g.waterplane_cut(3.0), 0.0);
    assert_eq!(g.extra_wetted_surface(3.0), 0.0);
    // A transom run is the raked box's stern rake: the same removed volume.
    let i = inputs(SternForm::Transom);
    let g = geometry(&i, &HULL);
    let rise = g.transom_height_m;
    let raked = HullGeometry::new(HULL.loa, HULL.beam, HULL.depth, 0.0, 0.0, g.run_m, rise);
    let box_ = HullGeometry::new(HULL.loa, HULL.beam, HULL.depth, 0.0, 0.0, 0.0, 0.0);
    for t in [1.0, 2.0, 3.0] {
        let by_box = box_.displaced_volume(t) - raked.displaced_volume(t);
        assert!((g.removed(t).0 - by_box).abs() < 1e-9, "{t}");
    }
}

#[test]
fn d_max_is_0_80_t_under_a_tunnel_and_0_65_t_otherwise_and_a_typed_d_is_kept() {
    // A narrow-draught hull, so the tunnels fit at 0.80 T.
    let shallow = HullDims {
        design_draught: 2.5,
        ..HULL
    };
    let g = geometry(&inputs(SternForm::Tunnel), &shallow);
    assert!((g.d_max_m - 0.80 * 2.5).abs() < 1e-12);
    let t = HULL.design_draught;
    assert!((geometry(&inputs(SternForm::Transom), &HULL).d_max_m - 0.65 * t).abs() < 1e-12);
    assert!((geometry(&inputs(SternForm::Pram), &HULL).d_max_m - 0.65 * t).abs() < 1e-12);
    // None: the largest the stern allows. A typed larger D is used as typed.
    let g = geometry(&inputs(SternForm::Tunnel), &HULL);
    assert_eq!(g.propeller_diameter_m, g.d_max_m);
    let mut i = inputs(SternForm::Tunnel);
    i.propeller_diameter_m = Some(3.2);
    let g = geometry(&i, &HULL);
    assert_eq!(g.propeller_diameter_m, 3.2);
    assert!(g.propeller_diameter_m > g.d_max_m);
}

#[test]
fn the_run_and_the_tunnels_follow_spec_0007() {
    let (d, b) = (2.0, 11.4);
    assert!((heuser_run_length(d, b, 1, false) - (1.9 * d + 1.85 * b)).abs() < 1e-12);
    assert!((heuser_run_length(d, b, 2, false) - (2.5 * d + 1.65 * b)).abs() < 1e-12);
    assert!((heuser_run_length(d, b, 2, true) - (3.2 * d + 1.45 * b)).abs() < 1e-12);
    let g = geometry(&inputs(SternForm::Tunnel), &HULL);
    assert_eq!(g.transom_height_m, 0.0);
    assert_eq!(g.tunnel_offsets_m.len(), 2);
    assert!((g.tunnel_offsets_m[1] - g.tunnel_offsets_m[0] - 0.45 * HULL.beam).abs() < 1e-12);
    assert!((g.tunnel_crown_height_m - 1.1 * g.propeller_diameter_m).abs() < 1e-12);
    let mut i = inputs(SternForm::Tunnel);
    i.parallel_shafts = false;
    let g = geometry(&i, &HULL);
    assert!((g.tunnel_offsets_m[1] - g.tunnel_offsets_m[0] - 0.28 * HULL.beam).abs() < 1e-12);
    // Transom and pram runs: 1.0 and 1.6 x depth, transom at the design draught.
    let g = geometry(&inputs(SternForm::Transom), &HULL);
    assert_eq!(
        (g.run_m, g.transom_height_m),
        (HULL.depth, HULL.design_draught)
    );
    let g = geometry(&inputs(SternForm::Pram), &HULL);
    assert!((g.run_m - 1.6 * HULL.depth).abs() < 1e-12);
}

#[test]
fn a_tunnel_adds_wetted_surface_and_cuts_the_waterplane_below_its_crown() {
    let g = geometry(&inputs(SternForm::Tunnel), &HULL);
    assert!(g.extra_wetted_surface(3.0) > 0.0);
    assert!(g.waterplane_cut(0.5 * g.tunnel_crown_height_m) > 0.0);
    assert_eq!(g.waterplane_cut(g.tunnel_crown_height_m + 0.1), 0.0);
    assert_eq!(g.tunnel_sections().len(), 2);
    assert!(!g.profile().is_empty());
}

#[test]
fn d_max_under_a_tunnel_is_capped_so_the_tunnels_meet_but_do_not_overlap_or_leave_the_beam() {
    // 110 x 11.4 m at 3.5 m: 0.80 T = 2.8 m, but twin parallel tunnels 0.45 B apart fit only
    // 0.225 B / sqrt(0.99) = 2.578 m.
    let g = geometry(&inputs(SternForm::Tunnel), &HULL);
    let fit = 0.225 * HULL.beam / (1.0_f64 - 0.01).sqrt();
    assert!((g.d_max_m - fit).abs() < 1e-12, "{}", g.d_max_m);
    assert!(g.d_max_m < 0.80 * HULL.design_draught);
    // At the cap the two tunnels meet on the centreline and stay inside the beam.
    let r = g.propeller_diameter_m;
    let w = tunnel_width(r, g.tunnel_crown_height_m, 0.0);
    let gap = g.tunnel_offsets_m[1] - g.tunnel_offsets_m[0];
    assert!((w - gap).abs() < 1e-9, "{w} {gap}");
    assert!(g.tunnel_offsets_m[1] + w / 2.0 <= HULL.beam / 2.0 + 1e-9);
    // A wide converging pair is held by the beam, not the pitch; one tunnel by half the beam.
    let (b, s) = (16.3, 0.28 * 16.3);
    let half = (1.0_f64 - 0.01).sqrt();
    assert!((tunnel_fit_diameter(b, s, 2) - (s / 2.0) / half).abs() < 1e-12);
    assert!((tunnel_fit_diameter(b, 0.45 * b, 2) - 0.225 * b / half).abs() < 1e-12);
    assert!((tunnel_fit_diameter(b, 0.0, 1) - b / 2.0 / half).abs() < 1e-12);
    // A typed larger D is kept.
    let mut i = inputs(SternForm::Tunnel);
    i.propeller_diameter_m = Some(3.0);
    assert_eq!(geometry(&i, &HULL).propeller_diameter_m, 3.0);
}

#[test]
fn a_tunnel_stern_moves_the_lcb_of_a_dry_bulk_trim_forward() {
    use ovcd_core::aft_arrangement::SternMassInputs;
    use ovcd_core::barge_capacity::{BargeCapacityInputs, compute};
    let mut i = BargeCapacityInputs::new(110.0, 11.4, 4.0);
    i.cargo_type = "bauxite".into();
    i.self_propelled = true;
    i.lightship_k_t_per_m3 = Some(0.1093);
    i.stern_block_mass = Some(SternMassInputs {
        battery_unit_energy_kwh: 5000.0,
        installed_power_kw: None,
    });
    let square = compute(&i).unwrap();
    let hull = HullDims {
        loa: 110.0,
        beam: 11.4,
        depth: 4.0,
        design_draught: 3.7,
    };
    i.stern_geometry = Some(geometry(&inputs(SternForm::Tunnel), &hull));
    let tunnel = compute(&i).unwrap();
    for ((name, a), (_, b)) in square.trim_conditions.iter().zip(&tunnel.trim_conditions) {
        assert!(b.LCB > a.LCB, "{name}: {} {}", a.LCB, b.LCB);
        assert!(b.trim < a.trim, "{name}");
    }
}

/// The ADR 0012 hook: the sections generator will take the same `SternFormInputs` and hull. The
/// geometry is a pure function of them, so the same inputs always give the same outlines and
/// volumes that the generator's sections must then match.
#[test]
fn the_same_inputs_give_the_same_geometry_for_the_sections_generator() {
    for form in [SternForm::Transom, SternForm::Pram, SternForm::Tunnel] {
        let i = inputs(form);
        let (a, b) = (geometry(&i, &HULL), geometry(&i.clone(), &HULL));
        assert_eq!(a, b);
        assert_eq!(a.profile(), b.profile());
        assert_eq!(a.tunnel_sections(), b.tunnel_sections());
        assert_eq!(a.removed(3.0), b.removed(3.0));
    }
}
