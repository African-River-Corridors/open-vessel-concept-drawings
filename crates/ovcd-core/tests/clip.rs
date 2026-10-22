//! Seam: `ovcd_core::compartments::clip`, a Space clipped by the hull (issue #455, spec 0008 "One
//! compartment model").
//!
//! Label: sourced - closed form, prism and wedge. A fore peak tank over the bow void of a raked box:
//! the box less the wedge under the rake, whose bottom rises linearly from the base at `L - l_r` to
//! `r` at the bow. Volume `B (D b - r l_r / 2)`; first moments of the box less the wedge; at a level
//! `h <= r`: `V(h) = B (h b - l_r (h - h² / 2r))` and the free surface `B³/12 (b - l_r + l_r h / r)`.
//! Label: property - a Space wholly inside the hull keeps its prism values and fills bit for bit.

use ovcd_core::compartments::{Side, Space, SpaceKind, clip};
use ovcd_core::hull::hull_form::{HullFormInputs, HullFormKind, HullShape, generate};
use ovcd_core::sections::SectionsTable;

const L: f64 = 80.0;
const B: f64 = 11.4;
const D: f64 = 4.0;
const RAKE: f64 = 6.4;

fn raked(rise: f64) -> SectionsTable {
    generate(&HullFormInputs {
        length_overall_m: L,
        beam_m: B,
        moulded_depth_m: D,
        form: HullFormKind::RakedBox {
            bow_rake_length_m: RAKE,
            bow_rake_rise_m: rise,
            stern_rake_length_m: 0.0,
            stern_rake_rise_m: 0.0,
        },
        shape: HullShape::default(),
    })
    .unwrap()
}

fn tank(x_aft: f64, x_fwd: f64, section: Vec<(f64, f64)>) -> Space {
    Space {
        name: "T".into(),
        kind: SpaceKind::PeakBallast,
        side: Side::Centre,
        x_aft,
        x_fwd,
        section,
    }
}

fn close(a: f64, b: f64, scale: f64, what: &str) {
    assert!((a - b).abs() <= 1e-12 * scale, "{what}: {a} vs {b}");
}

#[test]
fn a_space_inside_the_hull_keeps_its_prism_values_and_fills_exactly() {
    let t = raked(D);
    for s in [
        tank(
            5.0,
            60.0,
            vec![(1.0, 0.0), (5.0, 0.0), (5.0, 2.0), (1.0, 2.0)],
        ),
        tank(
            10.0,
            30.0,
            vec![(-2.0, 1.0), (2.0, 1.0), (3.0, 3.5), (-3.0, 3.5)],
        ),
        // Up to the start of the rake: the hull bottom is on the base the whole length.
        tank(
            L - RAKE - 5.0,
            L - RAKE,
            vec![(-B / 2.0, 0.0), (B / 2.0, 0.0), (B / 2.0, D), (-B / 2.0, D)],
        ),
    ] {
        let c = clip(&s, &t);
        assert!(!c.is_cut());
        assert_eq!(c.volume_m3, s.volume_m3());
        for f in [0.0, 0.3, 0.5, 1.0] {
            assert_eq!(c.fill(f), s.fill(f));
        }
    }
}

#[test]
fn a_fore_peak_in_a_rake_is_the_box_less_the_wedge() {
    let bow = 9.0;
    let xs = L - RAKE;
    for rise in [D, 2.5] {
        let t = raked(rise);
        let s = tank(
            L - bow,
            L,
            vec![(-B / 2.0, 0.0), (B / 2.0, 0.0), (B / 2.0, D), (-B / 2.0, D)],
        );
        let c = clip(&s, &t);
        assert!(c.is_cut());
        // The wedge: section B r s over s = (x - xs) / l_r in 0..1.
        let wedge = B * rise * RAKE / 2.0;
        let v = B * D * bow - wedge;
        close(c.volume_m3, v, v, "volume");
        let box_mx = B * D * bow * (L - bow / 2.0);
        // ∫ x B r (x - xs) / l_r dx over the rake = B r l_r (xs / 2 + l_r / 3).
        let wedge_mx = B * rise * RAKE * (xs / 2.0 + RAKE / 3.0);
        close(c.centroid_m.0, (box_mx - wedge_mx) / v, L, "LCG");
        // ∫ B z_b² / 2 dx = B r² l_r / 6.
        let wedge_mz = B * rise * rise * RAKE / 6.0;
        close(
            c.centroid_m.2,
            (B * D * D / 2.0 * bow - wedge_mz) / v,
            D,
            "VCG",
        );
        close(c.centroid_m.1, 0.0, B, "TCG");
        // Half full: the level h solves V(h) = v / 2 (h below the rise for these cases).
        let f = c.fill(0.5);
        let h = f.level_m;
        assert!(h < rise, "{h}");
        let vh = B * (h * bow - RAKE * (h - h * h / (2.0 * rise)));
        close(vh, v / 2.0, v, "V(h)");
        close(f.volume_m3, v / 2.0, v, "fill volume");
        let fs = B * B * B / 12.0 * (bow - RAKE + RAKE * h / rise);
        close(f.free_surface_inertia_m4, fs, fs, "free surface");
        let full = c.fill(1.0);
        close(full.volume_m3, v, v, "full");
        assert_eq!(full.free_surface_inertia_m4, 0.0);
    }
}

#[test]
fn a_space_past_the_end_of_the_hull_counts_only_the_part_inside() {
    let t = raked(D);
    let s = tank(
        -5.0,
        5.0,
        vec![(-1.0, 0.0), (1.0, 0.0), (1.0, 1.0), (-1.0, 1.0)],
    );
    let c = clip(&s, &t);
    assert!(c.is_cut());
    close(c.volume_m3, 10.0, 10.0, "volume");
    close(c.centroid_m.0, 2.5, L, "LCG");
}
