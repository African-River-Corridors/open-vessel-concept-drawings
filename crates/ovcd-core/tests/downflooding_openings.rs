//! Seam: `Hull::gz_with_downflooding` and `sections::opening_immersion_deg` (issue #592): the heel
//! at which the lowest deck opening is immersed at the free-trim equilibrium.
//!
//! Label: sourced - closed form. A wall-sided box of beam B, depth D at draught T floats at a heel
//! with its waterline through the centre-line point at height T, while neither the deck edge is
//! immersed nor the bottom edge emerges. A point (y, z) is then immersed when
//! (z - T) cos(phi) - y sin(phi) <= 0. For an opening at the deck edge (y = B/2, z = D) that is
//! tan(phi) = (D - T) / (B / 2), worked here.

use ovcd_core::hull::hull_form::HullShape;
use ovcd_core::hydrostatics::{Hull, HullSpec, Opening};
use ovcd_core::sections::Gravity;

const L: f64 = 60.0;
const B: f64 = 10.0;
const D: f64 = 4.5;
const T: f64 = 3.0;

fn hull(openings: Vec<Opening>) -> Hull {
    Hull::build(&HullSpec {
        loa: L,
        beam: B,
        depth: D,
        bow_rake_length_m: 0.0,
        bow_rake_rise_m: 0.0,
        stern: None,
        shape: HullShape::default(),
        downflooding_deg: None,
        openings,
    })
    .unwrap()
}

fn angle(openings: Vec<Opening>) -> Option<f64> {
    let h = hull(openings);
    let g = Gravity {
        lcg_m: L / 2.0,
        tcg_m: 0.0,
        kg_m: 2.4,
    };
    let (gz, a) = h
        .gz_with_downflooding(L * B * T, g, 0.0, 1.0, 0..=60, None)
        .unwrap();
    assert_eq!(gz.len(), 61);
    a
}

fn at(x: f64, y: f64, h: f64) -> Opening {
    Opening {
        x_m: x,
        y_m: y,
        height_above_deck_m: h,
    }
}

#[test]
fn an_opening_at_the_deck_edge_is_immersed_where_the_deck_edge_is() {
    let exact = libm::atan((D - T) / (B / 2.0)) * 180.0 / std::f64::consts::PI;
    // Linear between whole degrees: within 0.01 degree of the closed form.
    let a = angle(vec![at(30.0, B / 2.0, 0.0)]).unwrap();
    assert!((a - exact).abs() < 0.01, "{a} vs {exact}");
    // On the other side of the hull: the same.
    let p = angle(vec![at(30.0, -B / 2.0, 0.0)]).unwrap();
    assert!((p - a).abs() < 1e-9, "{p} vs {a}");
}

#[test]
fn the_lowest_opening_counts_and_a_higher_one_immerses_later() {
    let low = angle(vec![at(30.0, 4.0, 0.0)]).unwrap();
    let high = angle(vec![at(30.0, 4.0, 0.5)]).unwrap();
    assert!(high > low, "{high} vs {low}");
    // Two openings: the earlier of the two.
    let both = angle(vec![at(30.0, 4.0, 0.5), at(30.0, 4.0, 0.0)]).unwrap();
    assert!((both - low).abs() < 1e-9);
    // A point nearer the centre line immerses later.
    assert!(angle(vec![at(30.0, 2.0, 0.0)]).unwrap() > low);
}

#[test]
fn no_opening_and_one_never_immersed_give_none() {
    assert_eq!(angle(vec![]), None);
    assert_eq!(angle(vec![at(30.0, 0.0, 30.0)]), None);
}

#[test]
fn an_opening_below_the_waterline_is_immersed_upright() {
    // A point 1 m under the draught line, at the deck height minus 2 m: below zero height above the
    // deck is not an opening of this model, so use a negative height to put it under water.
    assert_eq!(angle(vec![at(30.0, 1.0, -(D - T) - 0.5)]), Some(0.0));
}
