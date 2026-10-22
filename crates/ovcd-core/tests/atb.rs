//! Seam: `ovcd_core::atb` and the Notch in a hull (issue #362, spec 0007 "Formation and Tug").
//!
//! Label: sourced - the ratios of spec 0007 (tug 0.28 of the barge length and 0.53 of its beam, tug
//! B/T 2.33, Notch 0.50 of the tug length deep and the tug beam plus 0.30 m wide); the Notch volume and
//! LCB are closed forms. A box barge of length L and beam B at a draught T below its depth, with a
//! rectangular Notch of depth d and width w cut through its full depth at the transom, displaces
//! (L B - d w) T, and its centre of buoyancy is at
//! (L B T (L / 2) - d w T (d / 2)) / ((L B - d w) T) from the aft end.

use ovcd_core::atb::{AtbTug, TugTyped, equivalent_hull_length, notch_for_tug, tug_from_barge};
use ovcd_core::hull::hull_form::HullShape;
use ovcd_core::hydrostatics::{Hull, HullSpec};
use ovcd_core::sections::{Waterplane, immersed};
use ovcd_core::stern_form::Notch;

const L: f64 = 100.0;
const B: f64 = 20.0;
const D: f64 = 6.0;

fn close(a: f64, b: f64, what: &str) {
    assert!(
        (a - b).abs() <= 1e-9 * b.abs().max(1.0),
        "{what}: {a} vs {b}"
    );
}

#[test]
fn the_tug_comes_from_the_barge_by_the_spec_ratios() {
    let tug = tug_from_barge(L, B, TugTyped::default());
    close(tug.length_m, 28.0, "length");
    close(tug.beam_m, 10.6, "beam");
    close(tug.draught_m, 10.6 / 2.33, "draught");
    let notch = notch_for_tug(&tug, None, None);
    close(notch.depth_m, 14.0, "notch depth");
    close(notch.width_m, 10.9, "notch width");
}

#[test]
fn a_typed_dimension_is_kept_and_the_rest_follow() {
    let tug = tug_from_barge(
        L,
        B,
        TugTyped {
            length_m: Some(30.0),
            beam_m: None,
            draught_m: Some(4.0),
        },
    );
    assert_eq!(tug.length_m, 30.0);
    assert_eq!(tug.draught_m, 4.0);
    close(tug.beam_m, 10.6, "beam from the barge");
    // The draught follows a typed beam.
    let wide = tug_from_barge(
        L,
        B,
        TugTyped {
            beam_m: Some(12.0),
            ..TugTyped::default()
        },
    );
    close(wide.draught_m, 12.0 / 2.33, "draught from the typed beam");
    // A typed Notch is kept, each part alone too.
    let n = notch_for_tug(&tug, Some(9.0), None);
    assert_eq!(n.depth_m, 9.0);
    close(n.width_m, 10.6 + 0.3, "width from the tug");
    let n = notch_for_tug(&tug, None, Some(12.0));
    close(n.depth_m, 15.0, "depth from the tug");
    assert_eq!(n.width_m, 12.0);
}

#[test]
fn the_equivalent_hull_is_the_barge_and_the_tug_less_the_notch() {
    let tug = AtbTug {
        length_m: 28.0,
        beam_m: 10.6,
        draught_m: 4.5,
    };
    let notch = Notch {
        depth_m: 14.0,
        width_m: 10.9,
    };
    close(equivalent_hull_length(L, &tug, &notch), 114.0, "length");
}

fn box_barge(notch: Option<Notch>) -> Hull {
    Hull::build(&HullSpec {
        loa: L,
        beam: B,
        depth: D,
        bow_rake_length_m: 0.0,
        bow_rake_rise_m: 0.0,
        stern: None,
        shape: HullShape {
            notch,
            ..HullShape::default()
        },
    })
    .unwrap()
}

#[test]
fn the_notch_volume_and_lcb_match_the_closed_form() {
    let (d, w, t) = (14.0, 10.9, 4.0);
    let hull = box_barge(Some(Notch {
        depth_m: d,
        width_m: w,
    }));
    let im = immersed(&hull.table, &Waterplane::upright(t));
    close(im.volume_m3, (L * B - d * w) * t, "volume");
    let lcb = (L * B * t * (L / 2.0) - d * w * t * (d / 2.0)) / ((L * B - d * w) * t);
    close(im.lcb_m, lcb, "LCB");
    // Waterplane area: the barge less the Notch.
    close(im.waterplane_area_m2, L * B - d * w, "waterplane area");
}

#[test]
fn a_notch_of_zero_gives_the_plain_barge_bit_for_bit() {
    let plain = box_barge(None);
    for notch in [
        Notch {
            depth_m: 0.0,
            width_m: 10.9,
        },
        Notch {
            depth_m: 14.0,
            width_m: 0.0,
        },
    ] {
        let hull = box_barge(Some(notch));
        assert_eq!(hull.table, plain.table);
        assert_eq!(
            hull.volume_at(4.0).to_bits(),
            plain.volume_at(4.0).to_bits()
        );
    }
}
