//! Seam: `ovcd_core::hull::hull_form::generate` with a bilge radius, sheer and camber (issue #449,
//! spec 0008 "Hull Forms").
//!
//! Label: sourced - closed form, bilge radius, sheer and camber. The bilge is a quarter circle of
//! radius r written as n chords: the chorded corner keeps the polygon (centre and arc points) of area
//! n r² sin(π / 2n) / 2 of the square corner r², and the true arc keeps π r² / 4. The sheer is the
//! parabola s(x) = s_end ((x − L/2) / (L/2))². The camber is the parabola c (1 − (y / b)²) across
//! the half-breadth b, written as n equal chords: by the trapezoid rule with a constant second
//! derivative its chorded area is the parabola's 2 c b / 3 less c b / (6 n²) on each half. The
//! GZ check is a property: with the same mass and the same GM, a wall-sided heel gives the same GZ.

use core::f64::consts::PI;
use ovcd_core::hull::hull_form::{
    BILGE_CHORDS, CAMBER_CHORDS, HullError, HullFormInputs, HullFormKind, HullShape, RakedBox,
    SHEER_INTERVALS, generate,
};
use ovcd_core::sections::{
    Gravity, SectionsTable, Waterplane, draught_for_volume, gz_curve, immersed, upright,
};

const L: f64 = 60.0;
const B: f64 = 10.0;
const D: f64 = 6.0;

fn inputs(form: HullFormKind, shape: HullShape) -> HullFormInputs {
    HullFormInputs {
        length_overall_m: L,
        beam_m: B,
        moulded_depth_m: D,
        form,
        shape,
    }
}

fn raked() -> HullFormKind {
    HullFormKind::RakedBox {
        bow_rake_length_m: 6.0,
        bow_rake_rise_m: 1.5,
        stern_rake_length_m: 4.0,
        stern_rake_rise_m: 1.0,
    }
}

fn bilge(r: f64) -> HullShape {
    HullShape {
        bilge_radius_m: r,
        ..HullShape::default()
    }
}

fn close(a: f64, b: f64, what: &str) {
    assert!(
        (a - b).abs() / b.abs().max(1.0) <= 1e-12,
        "{what}: {a} vs {b}"
    );
}

fn volume(t: &SectionsTable, draught: f64) -> f64 {
    immersed(t, &Waterplane::upright(draught)).volume_m3
}

/// The area one chorded bilge corner takes from the square section.
fn chorded_corner(r: f64) -> f64 {
    let n = f64::from(BILGE_CHORDS);
    r * r - n / 2.0 * r * r * libm::sin(PI / (2.0 * n))
}

#[test]
fn a_shape_of_zero_writes_the_same_table() {
    for form in [HullFormKind::Box, raked()] {
        let plain = generate(&inputs(form, HullShape::default())).unwrap();
        assert_eq!(plain.sections[0].points.len(), 4);
        assert!(plain.depth_at_side.iter().all(|&d| d == D));
    }
}

#[test]
fn the_bilge_volume_is_the_chorded_section_and_near_the_true_arc() {
    let r = 1.0;
    let t = generate(&inputs(HullFormKind::Box, bilge(r))).unwrap();
    assert_eq!(
        t.sections[0].points.len(),
        4 + BILGE_CHORDS as usize,
        "the keel, 17 arc points, the deck edge and the deck at the centre line"
    );
    // The chord error of the whole hull: the arc segments the chords cut off, both sides.
    let n = f64::from(BILGE_CHORDS);
    let chord_error = 2.0 * L * (PI * r * r / 4.0 - n / 2.0 * r * r * libm::sin(PI / (2.0 * n)));
    for draught in [1.0, 2.0, 3.0, 5.5] {
        let v = volume(&t, draught);
        close(
            v,
            L * (B * draught - 2.0 * chorded_corner(r)),
            "chorded volume",
        );
        let true_arc = L * (B * draught - 2.0 * r * r * (1.0 - PI / 4.0));
        assert!(
            (true_arc - v).abs() <= chord_error * (1.0 + 1e-9),
            "true arc {true_arc} vs {v}: more than the chord error {chord_error}"
        );
    }
}

#[test]
fn the_raked_box_takes_the_bilge_along_its_whole_length() {
    // The corner rides up the rakes with the keel; it stays below the waterline here.
    let r = 1.0;
    let t = generate(&inputs(raked(), bilge(r))).unwrap();
    let rb = RakedBox::new(L, B, D, 6.0, 1.5, 4.0, 1.0);
    for draught in [3.0, 4.5] {
        close(
            volume(&t, draught),
            rb.displaced_volume(draught) - 2.0 * chorded_corner(r) * L,
            "raked volume with a bilge",
        );
    }
}

#[test]
fn the_sheer_is_the_parabola_at_each_station_and_moves_nothing_below_the_deck_edge() {
    let (aft, fwd) = (0.8, 1.6);
    let shape = HullShape {
        sheer_aft_m: aft,
        sheer_fwd_m: fwd,
        ..HullShape::default()
    };
    for form in [HullFormKind::Box, raked()] {
        let plain = generate(&inputs(form, HullShape::default())).unwrap();
        let t = generate(&inputs(form, shape)).unwrap();
        for i in 0..=SHEER_INTERVALS {
            let x = L * f64::from(i) / f64::from(SHEER_INTERVALS);
            assert!(t.sections.iter().any(|s| s.x == x), "a station at {x}");
        }
        for (s, &edge) in t.sections.iter().zip(&t.depth_at_side) {
            let u = (s.x - L / 2.0) / (L / 2.0);
            let parabola = D + if s.x < L / 2.0 { aft } else { fwd } * u * u;
            close(edge, parabola, "depth at side");
            close(s.points[2].1, parabola, "deck edge point");
            assert_eq!(s.points[2].0, B / 2.0);
        }
        // The lowest deck edge is the moulded depth at mid-length.
        close(
            t.depth_at_side
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min),
            D,
            "lowest deck edge",
        );
        for draught in [0.5, 1.2, 3.0, D] {
            let (a, b) = (
                immersed(&plain, &Waterplane::upright(draught)),
                immersed(&t, &Waterplane::upright(draught)),
            );
            close(b.volume_m3, a.volume_m3, "volume below the deck edge");
            close(b.lcb_m, a.lcb_m, "LCB below the deck edge");
            close(b.kb_m, a.kb_m, "KB below the deck edge");
            close(
                b.waterplane_area_m2,
                a.waterplane_area_m2,
                "waterplane below the deck edge",
            );
        }
    }
}

#[test]
fn a_sheer_at_one_end_adds_stations_on_that_half_only() {
    let t = generate(&inputs(
        HullFormKind::Box,
        HullShape {
            sheer_fwd_m: 1.0,
            ..HullShape::default()
        },
    ))
    .unwrap();
    let half = SHEER_INTERVALS / 2;
    let xs: Vec<f64> = t.sections.iter().map(|s| s.x).collect();
    let mut want = vec![0.0];
    want.extend((half..=SHEER_INTERVALS).map(|i| L * f64::from(i) / f64::from(SHEER_INTERVALS)));
    assert_eq!(xs, want);
    assert!(
        t.depth_at_side[..2].iter().all(|&d| d == D),
        "flat aft of mid-length"
    );
}

#[test]
fn the_camber_area_and_the_volume_above_the_deck_edge_are_the_chorded_parabola() {
    let c = 0.4;
    let t = generate(&inputs(
        HullFormKind::Box,
        HullShape {
            camber_m: c,
            ..HullShape::default()
        },
    ))
    .unwrap();
    assert_eq!(
        t.sections[0].points.len(),
        3 + CAMBER_CHORDS as usize,
        "the keel, the bilge corner, the deck edge and 16 chords to the centre line"
    );
    let above = volume(&t, D + c + 1.0) - volume(&t, D);
    let n = f64::from(CAMBER_CHORDS);
    let chord_error = c * B / (6.0 * n * n);
    let parabola = 2.0 * c * B / 3.0;
    close(above / L, parabola - chord_error, "chorded camber area");
    assert!(
        (above / L - parabola).abs() <= chord_error * (1.0 + 1e-9),
        "camber area against the parabola"
    );
    close(volume(&t, D), L * B * D, "volume to the deck edge");
    // The deck points are on the parabola.
    let s = &t.sections[0];
    for p in &s.points[2..] {
        let u = p.0 / (B / 2.0);
        close(p.1, D + c * (1.0 - u * u), "deck point on the parabola");
    }
}

/// The GZ curve of a hull at `mass` with G at mid-length on the centre line and `kg`.
fn gz(t: &SectionsTable, mass: f64, kg: f64, angles: &[i32]) -> Vec<f64> {
    let g = Gravity {
        lcg_m: L / 2.0,
        tcg_m: 0.0,
        kg_m: kg,
    };
    gz_curve(t, mass, g, 0.0, 1.0, angles.iter().copied())
        .unwrap()
        .into_iter()
        .map(|(_, v)| v)
        .collect()
}

#[test]
fn a_bilge_lowers_gz_at_large_heel_and_keeps_it_at_small_heel() {
    let square = generate(&inputs(HullFormKind::Box, HullShape::default())).unwrap();
    let round = generate(&inputs(HullFormKind::Box, bilge(1.0))).unwrap();
    let mass = L * B * 3.0;
    // The same mass and the same GM: the bilge hull floats a little deeper, so its KM is higher,
    // and its G is raised by the same amount.
    let km = |t: &SectionsTable| {
        let draught = draught_for_volume(t, mass).unwrap();
        upright(t, draught, 1.0).unwrap().km_t_m
    };
    let kg = 3.5;
    let kg_round = kg + km(&round) - km(&square);
    // Wall-sided while the waterline stays between the bilge and the deck edge (about 20°).
    let small = [0, 2, 5, 10, 15];
    let (a, b) = (
        gz(&square, mass, kg, &small),
        gz(&round, mass, kg_round, &small),
    );
    for (i, (x, y)) in a.iter().zip(&b).enumerate() {
        assert!(
            (x - y).abs() <= 1e-9,
            "{}°: square {x} vs bilge {y}",
            small[i]
        );
    }
    assert!(b[0].abs() <= 1e-12, "GZ(0) = {}", b[0]);
    // Past the emergence of the bilge, the round bilge hull rights less.
    let large = [30, 40, 50, 60];
    let (a, b) = (
        gz(&square, mass, kg, &large),
        gz(&round, mass, kg_round, &large),
    );
    for (i, (x, y)) in a.iter().zip(&b).enumerate() {
        assert!(y < x, "{}°: bilge {y} not below square {x}", large[i]);
    }
}

#[test]
fn a_bad_shape_is_a_typed_error() {
    assert_eq!(
        generate(&inputs(HullFormKind::Box, bilge(-0.1))),
        Err(HullError::NegativeShape)
    );
    assert_eq!(
        generate(&inputs(
            HullFormKind::Box,
            HullShape {
                camber_m: f64::NAN,
                ..HullShape::default()
            }
        )),
        Err(HullError::NegativeShape)
    );
    assert_eq!(
        generate(&inputs(HullFormKind::Box, bilge(5.5))),
        Err(HullError::BilgeRadius {
            radius_m: 5.5,
            limit_m: 5.0
        })
    );
    // Half the beam is allowed: a round bottom.
    assert!(generate(&inputs(HullFormKind::Box, bilge(5.0))).is_ok());
}
