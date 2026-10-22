//! Seam: the Stern Form geometry (issue #350, spec 0007 "Stern geometry").
//!
//! The closed forms are checked against a strip integral of the same shape (2,000 strips) at three
//! draughts; the Heuser run lengths and the D_max rules against spec 0007 by hand: single screw
//! 1.9 D + 1.85 B, twin converging 2.5 D + 1.65 B, twin parallel 3.2 D + 1.45 B (Rotteveel 2019
//! sec. 2.3.1); D_max 0.80 T under a tunnel, 0.65 T at a transom or pram stern.
//!
//! The Stern Forms written into the Sections Table (issue #447, spec 0008 "Hull Forms"): the
//! transom exactly against the closed form; the pram against it within the chord error of its
//! Stations, worked here; the tunnel against an independent strip integral of the same shape.
//!
//! Label: sourced - spec 0007 "Stern geometry" (Heuser in Rotteveel 2019 sec. 2.3.1; MAN d/D below about 0.65); closed form, transom; the strip integrals, the pram chord error B h L / 6 n² and the tunnel chord deficit (r²/2)(2a0 − K sin(2a0/K)) are worked here.

use ovcd_core::hull::HullGeometry;
use ovcd_core::hull::hull_form::{HullFormInputs, HullFormKind, generate};
use ovcd_core::sections::{SectionsTable, upright};
use ovcd_core::stern_form::{
    HullDims, RUN_INTERVALS, SternForm, SternFormInputs, SternGeometry, TUNNEL_CHORDS, apply,
    apply_with, geometry, heuser_run_length, removed_by_strips, tunnel_fit_diameter,
    tunnel_section_area, tunnel_width,
};

fn inputs(form: SternForm) -> SternFormInputs {
    SternFormInputs {
        form,
        run_length_m: None,
        transom_height_m: None,
        tunnel_crown_height_m: None,
        propulsor_count: 2,
        propeller_diameter_m: None,
        max_unit_diameter_m: None,
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
fn each_drawn_tunnel_outline_ends_on_the_base_at_its_half_width() {
    // The default crown (1.1 D, the centre above the base) and a typed low crown (below it).
    let mut low = inputs(SternForm::Tunnel);
    low.tunnel_crown_height_m = Some(0.8);
    for i in [inputs(SternForm::Tunnel), low] {
        let g = geometry(&i, &HULL);
        let half = tunnel_width(g.propeller_diameter_m, g.tunnel_crown_height_m, 0.0) / 2.0;
        for (outline, y0) in g.tunnel_sections().iter().zip(&g.tunnel_offsets_m) {
            let (first, last) = (outline[0], outline[outline.len() - 1]);
            for (p, y) in [(first, y0 - half), (last, y0 + half)] {
                assert!(p[1].abs() < 1e-12, "{p:?}");
                assert!((p[0] - y).abs() < 1e-12, "{p:?} {y}");
            }
            // The crown is the highest point.
            let top = outline.iter().map(|p| p[1]).fold(f64::MIN, f64::max);
            assert!((top - g.tunnel_crown_height_m).abs() < 1e-12);
        }
    }
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
    // Issue #448: with free trim B is under G, so the tunnel's lost buoyancy aft shows as a trim
    // more by the stern, not as a moved LCB.
    for ((name, a), (_, b)) in square.trim_conditions.iter().zip(&tunnel.trim_conditions) {
        assert!(b.trim < a.trim, "{name}: {} {}", a.trim, b.trim);
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

#[test]
fn the_maker_range_caps_the_default_diameter_but_not_d_max_or_a_typed_d() {
    // The maintainer's decision on issue #352: a 0.65 T default above the largest unit takes it.
    let deep = HullDims {
        design_draught: 7.4,
        ..HULL
    };
    let mut i = inputs(SternForm::Transom);
    i.max_unit_diameter_m = Some(4.2);
    let g = geometry(&i, &deep);
    assert_eq!(g.propeller_diameter_m, 4.2);
    assert!((g.d_max_m - 0.65 * 7.4).abs() < 1e-12);
    // Below the cap the default stays D_max; a typed D is used as typed.
    assert!((geometry(&i, &HULL).propeller_diameter_m - 0.65 * 3.5).abs() < 1e-12);
    i.propeller_diameter_m = Some(4.5);
    assert_eq!(geometry(&i, &deep).propeller_diameter_m, 4.5);
}

// --- The Stern Forms in the Sections Table (issue #447) ---------------------------------------

/// The raked box of `HULL` with its bow rake and a square stern, as the generator writes it.
fn square_stern_table() -> SectionsTable {
    generate(&HullFormInputs {
        length_overall_m: HULL.loa,
        beam_m: HULL.beam,
        moulded_depth_m: HULL.depth,
        form: HullFormKind::RakedBox {
            bow_rake_length_m: 1.6 * HULL.depth,
            bow_rake_rise_m: HULL.depth,
            stern_rake_length_m: 0.0,
            stern_rake_rise_m: 0.0,
        },
    })
    .unwrap()
}

/// The volume and its first moment about the transom below an upright waterline.
fn volume_and_moment(t: &SectionsTable, draught: f64) -> (f64, f64) {
    let im = upright(t, draught, 1.0).unwrap().immersed;
    (im.volume_m3, im.volume_m3 * im.lcb_m)
}

/// What the stern removes from the square-stern table, by the sections.
fn removed_by_sections(g: &SternGeometry, intervals: u32, draught: f64) -> (f64, f64) {
    let base = square_stern_table();
    let mut cut = base.clone();
    apply_with(g, &mut cut, intervals).unwrap();
    let (v0, m0) = volume_and_moment(&base, draught);
    let (v1, m1) = volume_and_moment(&cut, draught);
    (v0 - v1, m0 - m1)
}

#[test]
fn a_transom_in_the_sections_removes_exactly_the_closed_form() {
    let g = geometry(&inputs(SternForm::Transom), &HULL);
    // Below and above the transom height (3.5 m).
    for t in [1.0, 2.5, 3.4, 3.6, 4.2] {
        let (v, m) = removed_by_sections(&g, RUN_INTERVALS, t);
        let (vc, mc) = g.removed(t);
        // Relative to the hull's own volume and moment: the removed part is their difference.
        let (v0, m0) = volume_and_moment(&square_stern_table(), t);
        assert!((v - vc).abs() <= 1e-12 * v0, "{t}: {v} {vc}");
        assert!((m - mc).abs() <= 1e-12 * m0, "{t}: {m} {mc}");
    }
}

#[test]
fn a_pram_in_the_sections_is_within_the_chord_error_and_converges_at_second_order() {
    let g = geometry(&inputs(SternForm::Pram), &HULL);
    let (b, h, l) = (g.beam, g.transom_height_m, g.run_m);
    // The chords lie above the parabola h (1 - x/L)²: over n intervals the area between them is
    // the trapezoid error L (L/n)² f'' / 12 with f'' = 2h / L², so B h L / (6 n²) of volume.
    let chord = |n: u32| b * h * l / (6.0 * f64::from(n * n));
    // Above the transom height the whole run is removed: the error is exactly the chord error.
    let (v0, _) = volume_and_moment(&square_stern_table(), 4.0);
    let (v, _) = removed_by_sections(&g, RUN_INTERVALS, 4.0);
    let (vc, _) = g.removed(4.0);
    assert!(
        (v - vc - chord(RUN_INTERVALS)).abs() <= 1e-12 * v0,
        "{v} {vc}"
    );
    // Below it, the error is at most that, and at most L times that for the moment.
    for t in [1.0, 2.5, 3.4] {
        let (v, m) = removed_by_sections(&g, RUN_INTERVALS, t);
        let (vc, mc) = g.removed(t);
        assert!((v - vc).abs() <= chord(RUN_INTERVALS), "{t}: {v} {vc}");
        assert!((m - mc).abs() <= l * chord(RUN_INTERVALS), "{t}: {m} {mc}");
        // Doubling the Stations divides the error by about four.
        let (v2, _) = removed_by_sections(&g, 2 * RUN_INTERVALS, t);
        let ratio = (v - vc) / (v2 - vc);
        assert!((3.5..4.5).contains(&ratio), "{t}: {ratio}");
    }
}

/// The removed area of the tunnels at `x` below `draught`, worked independently of the sections:
/// the same chorded outline (the arc of radius D, its crown falling linearly over the run, `K`
/// chords, the inner arc of a pair cut off at the centre line where it meets its mirror), its
/// polygon clipped at the waterline, by the shoelace formula. Both sides.
fn tunnel_area_at(g: &SternGeometry, x: f64, draught: f64) -> f64 {
    let r = g.propeller_diameter_m;
    let crown = g.tunnel_crown_height_m * (1.0 - x / g.run_m);
    let zc = crown - r;
    let a0 = libm::acos((-zc / r).clamp(-1.0, 1.0));
    let mut total = 0.0;
    for &y0 in g.tunnel_offsets_m.iter().filter(|&&y| y >= 0.0) {
        // The polygon of one tunnel, starboard half only for a centreline tunnel.
        let mut poly: Vec<(f64, f64)> = Vec::new();
        let k = f64::from(TUNNEL_CHORDS);
        if y0 == 0.0 {
            poly.push((0.0, 0.0));
            for j in 0..=TUNNEL_CHORDS / 2 {
                let a = a0 * f64::from(j) / (k / 2.0);
                poly.push((r * libm::sin(a), zc + r * libm::cos(a)));
            }
        } else {
            let start = if y0 < r && a0 > libm::asin(y0 / r) {
                -libm::asin(y0 / r)
            } else {
                -a0
            };
            if start > -a0 {
                poly.push((0.0, 0.0));
            }
            for j in 0..=TUNNEL_CHORDS {
                let a = start + (a0 - start) * f64::from(j) / k;
                let p = (y0 + r * libm::sin(a), zc + r * libm::cos(a));
                poly.push(if j == 0 && start > -a0 { (0.0, p.1) } else { p });
            }
        }
        total += 2.0 * shoelace(&clip_below(&poly, draught));
    }
    total
}

/// Keep the part of a closed polygon at or below `z`.
fn clip_below(poly: &[(f64, f64)], z: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for i in 0..poly.len() {
        let (p, q) = (poly[i], poly[(i + 1) % poly.len()]);
        let (pin, qin) = (p.1 <= z, q.1 <= z);
        if pin {
            out.push(p);
        }
        if pin != qin {
            let u = (z - p.1) / (q.1 - p.1);
            out.push((p.0 + u * (q.0 - p.0), z));
        }
    }
    out
}

fn shoelace(poly: &[(f64, f64)]) -> f64 {
    let n = poly.len();
    (0..n)
        .map(|i| {
            let (p, q) = (poly[i], poly[(i + 1) % n]);
            p.0 * q.1 - q.0 * p.1
        })
        .sum::<f64>()
        .abs()
        / 2.0
}

fn tunnel_by_strips(g: &SternGeometry, draught: f64, strips: u32) -> (f64, f64) {
    let dx = g.run_m / f64::from(strips);
    (0..strips).fold((0.0, 0.0), |(v, m), i| {
        let x = (f64::from(i) + 0.5) * dx;
        let a = tunnel_area_at(g, x, draught);
        (v + a * dx, m + a * x * dx)
    })
}

/// The removed area at `x` below `draught` of the ruled surface between the Stations the
/// generator wrote, worked from the table's points but not by its integrator: the box section less
/// the polygon interpolated at `x`, clipped at the waterline. Both sides.
fn ruled_area_at(t: &SectionsTable, x: f64, draught: f64) -> f64 {
    let ss = &t.sections;
    let j = ss.iter().position(|s| s.x > x).unwrap();
    let (a, b) = (&ss[j - 1], &ss[j]);
    let u = (x - a.x) / (b.x - a.x);
    let poly: Vec<(f64, f64)> = a
        .points
        .iter()
        .zip(&b.points)
        .map(|(p, q)| (p.0 + u * (q.0 - p.0), p.1 + u * (q.1 - p.1)))
        .collect();
    HULL.beam * draught.min(HULL.depth) - 2.0 * shoelace(&clip_below(&poly, draught))
}

fn ruled_by_strips(g: &SternGeometry, t: &SectionsTable, draught: f64, strips: u32) -> (f64, f64) {
    let dx = g.run_m / f64::from(strips);
    (0..strips).fold((0.0, 0.0), |(v, m), i| {
        let x = (f64::from(i) + 0.5) * dx;
        let a = ruled_area_at(t, x, draught);
        (v + a * dx, m + a * x * dx)
    })
}

/// The default twin tunnels, capped so they meet on the centre line, and a smaller pair that stays
/// clear of it.
fn tunnel_cases() -> [SternGeometry; 2] {
    let mut small = inputs(SternForm::Tunnel);
    small.propeller_diameter_m = Some(2.0);
    [
        geometry(&inputs(SternForm::Tunnel), &HULL),
        geometry(&small, &HULL),
    ]
}

#[test]
fn a_tunnel_in_the_sections_matches_a_strip_integral_of_its_ruled_surface() {
    for g in tunnel_cases() {
        let mut t = square_stern_table();
        apply(&g, &mut t).unwrap();
        for d in [1.0, 2.0, 3.5] {
            let (v, m) = removed_by_sections(&g, RUN_INTERVALS, d);
            let (vs, ms) = ruled_by_strips(&g, &t, d, 400);
            let what = format!("D {}: {d}", g.propeller_diameter_m);
            assert!(((v - vs) / vs).abs() < 1e-4, "{what}: {v} {vs}");
            assert!(((m - ms) / ms).abs() < 1e-4, "{what}: {m} {ms}");
        }
    }
}

#[test]
fn three_tunnels_that_overlap_are_written_as_their_union() {
    // Three tunnels at the capped D meet at the base, and their crowns above the centre make them
    // overlap above it: the section is the union. It matches the strip integral of its ruled
    // surface, and it removes less than three separate tunnels would.
    let mut i = inputs(SternForm::Tunnel);
    i.propulsor_count = 3;
    let g = geometry(&i, &HULL);
    let mut t = square_stern_table();
    apply(&g, &mut t).unwrap();
    let r = g.propeller_diameter_m;
    let separate = 3.0 * tunnel_section_area(r, g.tunnel_crown_height_m, g.tunnel_crown_height_m);
    let at_transom = HULL.beam * HULL.depth - 2.0 * shoelace(&t.sections[0].points);
    assert!(
        at_transom > 0.0 && at_transom < separate,
        "{at_transom} {separate}"
    );
    for d in [1.0, 2.0, 3.5] {
        let (v, m) = removed_by_sections(&g, RUN_INTERVALS, d);
        let (vs, ms) = ruled_by_strips(&g, &t, d, 400);
        assert!(((v - vs) / vs).abs() < 1e-4, "{d}: {v} {vs}");
        assert!(((m - ms) / ms).abs() < 1e-4, "{d}: {m} {ms}");
    }
}

#[test]
fn a_tunnel_in_the_sections_converges_at_second_order_to_the_smooth_shape() {
    // Between the 17 Stations the ruled surface is linear while the shape is not: at 16 intervals
    // the error is below 1e-3 of the volume and 2e-3 of the moment, and both fall by about four
    // when the intervals double.
    for g in tunnel_cases() {
        for d in [1.0, 2.0, 3.5] {
            let (vs, ms) = tunnel_by_strips(&g, d, 4000);
            let (v, m) = removed_by_sections(&g, RUN_INTERVALS, d);
            let (v2, m2) = removed_by_sections(&g, 2 * RUN_INTERVALS, d);
            let what = format!("D {}: {d}", g.propeller_diameter_m);
            assert!(((v - vs) / vs).abs() < 1e-3, "{what}: {v} {vs}");
            assert!(((m - ms) / ms).abs() < 2e-3, "{what}: {m} {ms}");
            for ratio in [(v - vs) / (v2 - vs), (m - ms) / (m2 - ms)] {
                assert!((3.5..4.5).contains(&ratio), "{what}: {ratio}");
            }
        }
    }
}

#[test]
fn at_the_transom_a_tunnel_is_the_circular_segment_less_its_chord_deficit() {
    // Two tunnels clear of each other and of the centre line, the crown under the waterline.
    let mut i = inputs(SternForm::Tunnel);
    i.propeller_diameter_m = Some(2.0);
    let g = geometry(&i, &HULL);
    let mut t = square_stern_table();
    apply(&g, &mut t).unwrap();
    let s = &t.sections[0];
    assert_eq!(s.x, 0.0);
    // The removed area of both sides: the box section less the section's polygon.
    let half = shoelace(&s.points);
    let removed = HULL.beam * HULL.depth - 2.0 * half;
    let (r, c) = (g.propeller_diameter_m, g.tunnel_crown_height_m);
    let a0 = libm::acos(-(c - r) / r);
    let k = f64::from(TUNNEL_CHORDS);
    // K equal chords inscribed in an arc of 2 a0 leave (r²/2)(2 a0 - K sin(2 a0 / K)) of it out.
    let deficit = r * r / 2.0 * (2.0 * a0 - k * libm::sin(2.0 * a0 / k));
    let n = g.tunnel_offsets_m.len() as f64;
    let segment = tunnel_section_area(r, c, c);
    let rel = ((removed + n * deficit) - n * segment).abs() / (n * segment);
    assert!(rel < 1e-12, "{removed} {deficit} {segment}: {rel}");
}

#[test]
fn the_legacy_stern_leaves_the_table_unchanged() {
    let mut i = inputs(SternForm::Transom);
    i.run_length_m = Some(0.0);
    let g = geometry(&i, &HULL);
    let base = square_stern_table();
    let mut t = base.clone();
    apply(&g, &mut t).unwrap();
    assert_eq!(t, base);
}

#[test]
fn each_form_keeps_one_point_count_and_writes_its_stations() {
    for (form, stations) in [
        (SternForm::Transom, 2),
        (SternForm::Pram, 17),
        (SternForm::Tunnel, 17),
    ] {
        let g = geometry(&inputs(form), &HULL);
        let base = square_stern_table();
        let mut t = base.clone();
        apply(&g, &mut t).unwrap();
        let in_run = t.sections.iter().filter(|s| s.x <= g.run_m).count();
        assert_eq!(in_run, stations, "{form:?}");
        // The Sections forward of the run are the base's, but for the tunnels' points on the bottom.
        assert_eq!(
            t.sections.len(),
            base.sections.len() + stations - 1,
            "{form:?}"
        );
    }
}

/// The tunnel's difference to the closed form of #350, which takes the immersed tunnel area as
/// falling linearly. It goes into the design-impact table of #448. Run with `--nocapture`.
#[test]
fn print_the_tunnel_difference_to_the_closed_form() {
    let g = geometry(&inputs(SternForm::Tunnel), &HULL);
    for t in [1.5, 2.5, 3.5] {
        let (v, m) = removed_by_sections(&g, RUN_INTERVALS, t);
        let (vc, mc) = g.removed(t);
        println!(
            "tunnel D {:.3} m, crown {:.3} m, run {:.3} m, draught {t} m: removed {v:.3} m3 (closed form {vc:.3}, {:+.2} %), moment {m:.1} m4 (closed form {mc:.1}, {:+.2} %)",
            g.propeller_diameter_m,
            g.tunnel_crown_height_m,
            g.run_m,
            100.0 * (v - vc) / vc,
            100.0 * (m - mc) / mc
        );
        assert!(v > 0.0 && vc > 0.0);
    }
}
