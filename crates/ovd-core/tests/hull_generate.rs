//! Seam: `ovd_core::hull::hull_form::generate`, the box and raked box generators (issue #445,
//! spec 0008 "Hull Forms").
//!
//! Label: sourced - closed form, raked box. The volume, waterplane area and BM_T are the closed forms
//! of `RakedBox` (`hull.rs`). The LCB is worked here by hand from the prism and the two wedges, and
//! also checked against `RakedBox::LCB` (fixed for the stern rake in issue #414). The box is the closed form of
//! `tests/sections.rs`.

use ovd_core::hull::hull_form::{
    HullError, HullFormInputs, HullFormKind, HullShape, RakedBox, generate,
};
use ovd_core::sections::{Waterplane, immersed, upright};

const L: f64 = 60.0;
const B: f64 = 10.0;
const D: f64 = 4.0;
const BOW_L: f64 = 6.0;
const BOW_R: f64 = 2.0;
const STERN_L: f64 = 4.0;
const STERN_R: f64 = 1.5;

fn raked_inputs(bl: f64, br: f64, sl: f64, sr: f64) -> HullFormInputs {
    HullFormInputs {
        length_overall_m: L,
        beam_m: B,
        moulded_depth_m: D,
        form: HullFormKind::RakedBox {
            bow_rake_length_m: bl,
            bow_rake_rise_m: br,
            stern_rake_length_m: sl,
            stern_rake_rise_m: sr,
        },
        shape: HullShape::default(),
    }
}

fn close(a: f64, b: f64, what: &str) {
    assert!(
        (a - b).abs() / b.abs().max(1.0) <= 1e-12,
        "{what}: {a} vs {b}"
    );
}

/// Draughts below, at and above each rake rise.
const DRAUGHTS: [f64; 7] = [0.5, 1.0, 1.5, 1.8, 2.0, 2.7, 3.9];

#[test]
fn the_raked_box_matches_the_raked_box_closed_forms() {
    let t = generate(&raked_inputs(BOW_L, BOW_R, STERN_L, STERN_R)).unwrap();
    let rb = RakedBox::new(L, B, D, BOW_L, BOW_R, STERN_L, STERN_R);
    assert_eq!(
        t.sections.iter().map(|s| s.x).collect::<Vec<_>>(),
        vec![0.0, STERN_L, L - BOW_L, L]
    );
    for t_m in DRAUGHTS {
        let u = upright(&t, t_m, 1.0).unwrap();
        close(u.immersed.volume_m3, rb.displaced_volume(t_m), "volume");
        close(
            u.immersed.waterplane_area_m2,
            rb.waterplane_area(t_m),
            "waterplane area",
        );
        close(u.bm_t_m, rb.BM_transverse(t_m), "BM_T");
    }
}

/// Immersed profile area and its first moment about x = 0 of the bow wedge, keel rising from the
/// base at `x0` to `rise` at `x0 + len`.
fn bow_wedge(t: f64, x0: f64, len: f64, rise: f64) -> (f64, f64) {
    if t <= rise {
        // A triangle: depth t at x0, nothing at x1.
        let run = t * len / rise;
        let a = t * run / 2.0;
        (a, a * (x0 + run / 3.0))
    } else {
        // A trapezoid: depth t at x0, t − rise at x0 + len. About x0: ∫ (t − rise s / len) s ds.
        let a = len * (t - rise / 2.0);
        let m = t * len * len / 2.0 - rise * len * len / 3.0;
        (a, a * x0 + m)
    }
}

/// The same for the stern wedge, keel falling from `rise` at x = 0 to the base at `len`.
fn stern_wedge(t: f64, len: f64, rise: f64) -> (f64, f64) {
    if t <= rise {
        let run = t * len / rise;
        let a = t * run / 2.0;
        (a, a * (len - run / 3.0))
    } else {
        // ∫0^len (t − rise (len − x) / len) x dx = t len² / 2 − rise len² / 6.
        let a = len * (t - rise / 2.0);
        (a, t * len * len / 2.0 - rise * len * len / 6.0)
    }
}

#[test]
fn the_raked_box_lcb_is_the_prism_and_wedges_by_hand() {
    let t = generate(&raked_inputs(BOW_L, BOW_R, STERN_L, STERN_R)).unwrap();
    for t_m in DRAUGHTS {
        let par = L - BOW_L - STERN_L;
        let (a_mid, m_mid) = (t_m * par, t_m * par * (STERN_L + par / 2.0));
        let (a_bow, m_bow) = bow_wedge(t_m, L - BOW_L, BOW_L, BOW_R);
        let (a_st, m_st) = stern_wedge(t_m, STERN_L, STERN_R);
        let lcb = (m_mid + m_bow + m_st) / (a_mid + a_bow + a_st);
        let im = immersed(&t, &Waterplane::upright(t_m));
        close(im.lcb_m, lcb, "LCB");
        close(im.volume_m3, B * (a_mid + a_bow + a_st), "volume");
        close(
            RakedBox::new(L, B, D, BOW_L, BOW_R, STERN_L, STERN_R).LCB(t_m),
            lcb,
            "RakedBox::LCB",
        );
    }
}

#[test]
fn a_raked_box_with_no_rakes_is_the_box() {
    let flat = generate(&raked_inputs(0.0, 0.0, 0.0, 0.0)).unwrap();
    let bx = generate(&HullFormInputs {
        length_overall_m: L,
        beam_m: B,
        moulded_depth_m: D,
        form: HullFormKind::Box,
        shape: HullShape::default(),
    })
    .unwrap();
    assert_eq!(flat, bx);
    assert_eq!(bx.sections.len(), 2);
    let rb = RakedBox::new(L, B, D, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(rb.sections().unwrap(), bx);
    for t_m in DRAUGHTS {
        let u = upright(&bx, t_m, 1.0).unwrap();
        close(u.immersed.volume_m3, rb.displaced_volume(t_m), "volume");
        close(u.immersed.volume_m3, L * B * t_m, "box volume");
        close(u.immersed.kb_m, t_m / 2.0, "KB");
        close(u.bm_t_m, B * B / (12.0 * t_m), "BM_T");
        close(u.immersed.lcb_m, L / 2.0, "LCB");
        close(u.cb, 1.0, "Cb");
    }
}

#[test]
fn a_bad_rake_or_dimension_is_a_typed_error() {
    assert_eq!(
        generate(&raked_inputs(40.0, 1.0, 30.0, 1.0)),
        Err(HullError::RakeLongerThanHull {
            rakes_m: 70.0,
            length_m: L
        })
    );
    assert_eq!(
        generate(&raked_inputs(-1.0, 1.0, 0.0, 0.0)),
        Err(HullError::NegativeRake)
    );
    assert_eq!(
        generate(&raked_inputs(6.0, 1.0, 4.0, -0.5)),
        Err(HullError::NegativeRake)
    );
    assert_eq!(
        generate(&raked_inputs(6.0, 4.5, 4.0, 1.0)),
        Err(HullError::RiseAboveDepth {
            rise_m: 4.5,
            depth_m: D
        })
    );
    let mut bad = raked_inputs(6.0, 1.0, 4.0, 1.0);
    bad.beam_m = 0.0;
    assert_eq!(generate(&bad), Err(HullError::Dimension));
    bad.beam_m = f64::NAN;
    assert_eq!(generate(&bad), Err(HullError::Dimension));
}
