//! Seam: `ovcd_core::sections::equilibrium` and `gz_curve`, free trim and the GZ curve on the
//! Sections Table (issue #444, spec 0008 "Exact polygon integration").
//!
//! Label: sourced - closed form, box; wall-sided formula. The box GZ is checked against
//! `box_stability::box_gz` at every angle. Below deck-edge immersion a box is wall-sided, so
//! GZ = sin φ (GM + ½ BM tan² φ) exactly: the wall-sided formula, which follows from the shift of
//! the two wedges of a section whose sides are vertical at the waterline. The trim is checked against
//! the first-order trim (LCG − LCB) L / BM_L of the box for a small trim. The rest are properties.

use ovcd_core::box_stability::{ANGLES_DEG, box_gz};
use ovcd_core::sections::{
    Gravity, Section, SectionsError, SectionsTable, Waterplane, equilibrium, gz_curve, immersed,
    righting_lever,
};

const L: f64 = 60.0;
const B: f64 = 11.4;
const D: f64 = 4.0;
const RHO: f64 = 1.0;

fn box_table() -> SectionsTable {
    let half = vec![(0.0, 0.0), (B / 2.0, 0.0), (B / 2.0, D), (0.0, D)];
    SectionsTable::new(
        vec![
            Section {
                x: 0.0,
                points: half.clone(),
            },
            Section { x: L, points: half },
        ],
        vec![D, D],
    )
    .unwrap()
}

fn mid(kg: f64) -> Gravity {
    Gravity {
        lcg_m: L / 2.0,
        tcg_m: 0.0,
        kg_m: kg,
    }
}

#[test]
fn the_box_gz_curve_is_box_gz_at_every_angle() {
    let t = box_table();
    for (draught, kg) in [(1.5, 3.0), (2.6, 2.2)] {
        let mass = RHO * L * B * draught;
        let gz = gz_curve(&t, mass, mid(kg), 0.0, RHO, ANGLES_DEG).unwrap();
        assert_eq!(gz.len(), 61);
        for (deg, v) in gz {
            let want = box_gz(B, D, draught, kg, f64::from(deg)).unwrap();
            assert!(
                (v - want).abs() <= 1e-12,
                "T {draught} {deg}: {v} vs {want}"
            );
        }
    }
}

#[test]
fn the_free_surface_comes_off_as_fsm_over_displacement_times_sin_phi() {
    let t = box_table();
    let (draught, kg, fsm) = (2.0, 3.0, 150.0);
    let mass = RHO * L * B * draught;
    let dry = gz_curve(&t, mass, mid(kg), 0.0, RHO, [10, 30]).unwrap();
    let wet = gz_curve(&t, mass, mid(kg), fsm, RHO, [10, 30]).unwrap();
    for (d, w) in dry.iter().zip(&wet) {
        let want = d.1 - fsm / mass * libm::sin(f64::from(d.0).to_radians());
        assert!((w.1 - want).abs() <= 1e-12);
    }
}

#[test]
fn a_wall_sided_box_follows_the_wall_sided_formula_and_gz_over_sin_tends_to_gm() {
    let t = box_table();
    let (draught, kg) = (2.0, 2.5);
    let mass = RHO * L * B * draught;
    let (kb, bm) = (draught / 2.0, B * B / (12.0 * draught));
    let gm = kb + bm - kg;
    // Deck edge immersion at atan(2 (D − T) / B) = 19.3 degrees; keel edge emergence at the same.
    for deg in [1, 2, 5, 10, 15, 19] {
        let phi = f64::from(deg).to_radians();
        let gz = gz_curve(&t, mass, mid(kg), 0.0, RHO, [deg]).unwrap()[0].1;
        let tan = libm::tan(phi);
        let want = libm::sin(phi) * (gm + 0.5 * bm * tan * tan);
        assert!((gz - want).abs() <= 1e-10, "{deg}: {gz} vs {want}");
    }
    let small = gz_curve(&t, mass, mid(kg), 0.0, RHO, [1]).unwrap()[0].1;
    let ratio = small / libm::sin(1.0_f64.to_radians());
    assert!((ratio - gm).abs() / gm < 1e-3, "{ratio} vs {gm}");
}

#[test]
fn a_fore_and_aft_symmetric_hull_with_lcg_at_mid_length_floats_level_and_gz_is_odd() {
    // A box with equal rakes at both ends.
    let (rl, rr) = (8.0, 1.5);
    let flat = vec![(0.0, 0.0), (B / 2.0, 0.0), (B / 2.0, D), (0.0, D)];
    let raked = vec![(0.0, rr), (B / 2.0, rr), (B / 2.0, D), (0.0, D)];
    let t = SectionsTable::new(
        vec![
            Section {
                x: 0.0,
                points: raked.clone(),
            },
            Section {
                x: rl,
                points: flat.clone(),
            },
            Section {
                x: L - rl,
                points: flat,
            },
            Section {
                x: L,
                points: raked,
            },
        ],
        vec![D; 4],
    )
    .unwrap();
    let mass = RHO * 1800.0;
    let g = mid(2.8);
    for deg in [0, 10, 25, 40, 60] {
        let e = equilibrium(&t, mass, g, f64::from(deg), RHO).unwrap();
        assert!(
            e.waterplane.trim_deg.abs() <= 1e-9,
            "{deg}: trim {}",
            e.waterplane.trim_deg
        );
        let neg = gz_curve(&t, mass, g, 0.0, RHO, [-deg]).unwrap()[0].1;
        let pos = righting_lever(&e, g);
        assert!((neg + pos).abs() <= 1e-12, "{deg}: {neg} vs {pos}");
    }
    assert!(gz_curve(&t, mass, g, 0.0, RHO, [0]).unwrap()[0].1.abs() <= 1e-12);
}

#[test]
fn an_lcg_forward_of_the_lcb_trims_the_box_by_the_bow_as_the_first_order_trim() {
    let t = box_table();
    let draught = 2.0;
    let mass = RHO * L * B * draught;
    // KG = KB, so GM_L = BM_L, the lever arm of the first-order trim.
    let lcg = L / 2.0 + 0.5;
    let g = Gravity {
        lcg_m: lcg,
        tcg_m: 0.0,
        kg_m: draught / 2.0,
    };
    let e = equilibrium(&t, mass, g, 0.0, RHO).unwrap();
    let ours = L * libm::tan(e.waterplane.trim_deg.to_radians());
    // First-order trim (LCG − LCB) L / BM_L, with LCB = L/2 and BM_L = L²/12T for a box.
    let first_order = (lcg - L / 2.0) * L / (L * L / (12.0 * draught));
    assert!(ours > 0.0 && first_order > 0.0);
    assert!(
        (ours - first_order).abs() / first_order < 0.01,
        "{ours} vs {first_order}"
    );
    close_volume(e.immersed.volume_m3, mass / RHO);
}

fn close_volume(a: f64, b: f64) {
    assert!((a - b).abs() / b <= 1e-12, "{a} vs {b}");
}

#[test]
fn a_trim_beyond_ten_degrees_is_an_error() {
    let t = box_table();
    let mass = RHO * L * B * 1.0;
    let g = Gravity {
        lcg_m: L - 1.0,
        tcg_m: 0.0,
        kg_m: 1.0,
    };
    assert_eq!(
        equilibrium(&t, mass, g, 0.0, RHO),
        Err(SectionsError::TrimOutOfRange)
    );
    assert_eq!(
        gz_curve(&t, mass, g, 0.0, RHO, [0, 1]),
        Err(SectionsError::TrimOutOfRange)
    );
}

#[test]
fn a_hull_too_heavy_to_float_or_with_no_mass_is_an_error() {
    let t = box_table();
    let g = mid(2.0);
    assert_eq!(
        equilibrium(&t, RHO * L * B * D * 1.01, g, 0.0, RHO),
        Err(SectionsError::Sinks)
    );
    assert_eq!(
        equilibrium(&t, 0.0, g, 0.0, RHO),
        Err(SectionsError::NotAfloat)
    );
}

#[test]
fn the_fixed_step_solve_gives_the_same_bits_twice() {
    let t = box_table();
    let g = Gravity {
        lcg_m: 31.3,
        tcg_m: 0.0,
        kg_m: 2.9,
    };
    let a = gz_curve(&t, 1500.0, g, 40.0, RHO, ANGLES_DEG).unwrap();
    let b = gz_curve(&t, 1500.0, g, 40.0, RHO, ANGLES_DEG).unwrap();
    for (p, q) in a.iter().zip(&b) {
        assert_eq!(p.1.to_bits(), q.1.to_bits());
    }
    let (e1, e2) = (
        equilibrium(&t, 1500.0, g, 17.0, RHO).unwrap(),
        equilibrium(&t, 1500.0, g, 17.0, RHO).unwrap(),
    );
    assert_eq!(e1.waterplane.c.to_bits(), e2.waterplane.c.to_bits());
    assert_eq!(
        e1.waterplane.trim_deg.to_bits(),
        e2.waterplane.trim_deg.to_bits()
    );
}

/// The reference: the bisection of the first draft of spec 0008, 80 steps on `c` inside 60 steps
/// on the trim, through the public cut. Slow, so only a few cases.
fn bisection_equilibrium(t: &SectionsTable, mass: f64, g: Gravity, heel: f64) -> (f64, f64) {
    let target = mass / RHO;
    let c_for = |trim: f64| {
        let k = Waterplane {
            heel_deg: heel,
            trim_deg: trim,
            c: 0.0,
        }
        .normal();
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for s in &t.sections {
            for &(y, z) in &s.points {
                for y in [y, -y] {
                    let d = k.0 * s.x + k.1 * y + k.2 * z;
                    lo = lo.min(d);
                    hi = hi.max(d);
                }
            }
        }
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            let wp = Waterplane {
                heel_deg: heel,
                trim_deg: trim,
                c: mid,
            };
            if immersed(t, &wp).volume_m3 < target {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    };
    let lever = |trim: f64| {
        let wp = Waterplane {
            heel_deg: heel,
            trim_deg: trim,
            c: c_for(trim),
        };
        let im = immersed(t, &wp);
        // Earth forward horizontal w = u × k, u = (0, k_z, −k_y) / cos θ.
        let (kx, ky, kz) = wp.normal();
        let ct = libm::cos(trim.to_radians());
        let (uy, uz) = (kz / ct, -ky / ct);
        let w = (uy * kz - uz * ky, uz * kx, -uy * kx);
        let b = (im.lcb_m, im.tcb_m, im.kb_m);
        (b.0 - g.lcg_m) * w.0 + (b.1 - g.tcg_m) * w.1 + (b.2 - g.kg_m) * w.2
    };
    let (mut lo, mut hi) = (-10.0, 10.0);
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if lever(mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let trim = 0.5 * (lo + hi);
    (c_for(trim), trim)
}

#[test]
fn the_secant_solve_agrees_with_an_80_by_60_bisection() {
    let t = box_table();
    let draught = 1.8;
    let mass = RHO * L * B * draught;
    let g = Gravity {
        lcg_m: L / 2.0 + 1.7,
        tcg_m: 0.0,
        kg_m: 3.1,
    };
    // Upright, before the deck edge, at it (19 to 21 degrees) and past it.
    for heel in [0.0, 12.0, 20.0, 35.0, 60.0] {
        let e = equilibrium(&t, mass, g, heel, RHO).unwrap();
        let (c, trim) = bisection_equilibrium(&t, mass, g, heel);
        assert!(
            (e.waterplane.c - c).abs() <= 1e-12,
            "{heel}: c {} vs {c}",
            e.waterplane.c
        );
        assert!(
            (e.waterplane.trim_deg - trim).abs() <= 1e-10,
            "{heel}: trim {} vs {trim}",
            e.waterplane.trim_deg
        );
    }
}
