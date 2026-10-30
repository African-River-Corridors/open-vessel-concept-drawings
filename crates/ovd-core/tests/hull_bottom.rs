//! Seam: `ovd_core::hull::hull_form::generate` for the `v_bottom` and `chine` Hull Forms (issue
//! #450, spec 0008 "Hull Forms").
//!
//! Label: sourced - closed form, V-bottom and chine prism. The half-section is piecewise linear in
//! height, y(z), so over each segment from (z0, y0) to (z1, y1) the half area is the trapezoid
//! (y0 + y1) (z1 - z0) / 2, and its moment about the base is (z1 - z0) times
//! (2 z0 y0 + z0 y1 + z1 y0 + 2 z1 y1) / 6, worked here. A prism has the section's area times L,
//! KB at the area's centroid, and BM_T = L (2 y(T))^3 / 12 over the volume. The raked V-bottom is
//! the raked box (`RakedBox` closed forms) less the triangle under the deadrise line, which rides
//! up the rake with the keel.

use ovd_core::hull::hull_form::{
    Bottom, HullError, HullFormInputs, HullFormKind, HullShape, RakedBox, generate,
};
use ovd_core::sections::{Gravity, SectionsTable, gz_curve, upright};

const L: f64 = 60.0;
const B: f64 = 10.0;
const D: f64 = 6.0;

fn hull(form: HullFormKind, bottom: Bottom, bilge: f64) -> Result<SectionsTable, HullError> {
    generate(&HullFormInputs {
        length_overall_m: L,
        beam_m: B,
        moulded_depth_m: D,
        form,
        shape: HullShape {
            bilge_radius_m: bilge,
            bottom,
            ..HullShape::default()
        },
    })
}

fn close(a: f64, b: f64, what: &str) {
    assert!(
        (a - b).abs() / b.abs().max(1.0) <= 1e-12,
        "{what}: {a} vs {b}"
    );
}

/// The prism's volume, KB and BM_T at draught `t` for the half-section through `pts` (z, y),
/// ascending in z, from the keel to the deck.
fn prism(pts: &[(f64, f64)], t: f64) -> (f64, f64, f64) {
    let (mut area, mut moment, mut y_t) = (0.0, 0.0, 0.0);
    for w in pts.windows(2) {
        let ((z0, y0), (z1, y1)) = (w[0], w[1]);
        if z0 >= t {
            break;
        }
        // Cut the segment at the waterline.
        let (z1c, y1c) = if z1 > t {
            (t, y0 + (y1 - y0) * (t - z0) / (z1 - z0))
        } else {
            (z1, y1)
        };
        let dz = z1c - z0;
        area += (y0 + y1c) * dz / 2.0;
        moment += dz * (2.0 * z0 * y0 + z0 * y1c + z1c * y0 + 2.0 * z1c * y1c) / 6.0;
        y_t = y1c;
    }
    let volume = 2.0 * area * L;
    let kb = moment / area;
    let bm = L * libm::pow(2.0 * y_t, 3.0) / 12.0 / volume;
    (volume, kb, bm)
}

const DEADRISE: f64 = 10.0;
const KEEL: f64 = 0.5;

fn rise(deg: f64) -> f64 {
    (B / 2.0 - KEEL) * libm::tan(deg.to_radians())
}

#[test]
fn the_v_bottom_prism_is_the_trapezoid_and_triangle_closed_form() {
    let t = hull(
        HullFormKind::Box,
        Bottom::VBottom {
            deadrise_deg: DEADRISE,
            keel_half_width_m: KEEL,
        },
        0.0,
    )
    .unwrap();
    let h = rise(DEADRISE);
    let section = [(0.0, KEEL), (h, B / 2.0), (D, B / 2.0)];
    // Below the chine, at it, and above it.
    for draught in [0.3 * h, h, 1.0, 3.0, 5.5] {
        let u = upright(&t, draught, 1.0).unwrap();
        let (v, kb, bm) = prism(&section, draught);
        close(u.immersed.volume_m3, v, "volume");
        close(u.immersed.kb_m, kb, "KB");
        close(u.bm_t_m, bm, "BM_T");
    }
}

#[test]
fn the_chine_prism_is_the_closed_form_with_a_flared_side() {
    let (y_c, h_c) = (4.0, 1.2);
    let t = hull(
        HullFormKind::Box,
        Bottom::Chine {
            deadrise_deg: DEADRISE,
            keel_half_width_m: KEEL,
            chine_half_breadth_m: Some(y_c),
            chine_height_m: Some(h_c),
        },
        0.0,
    )
    .unwrap();
    let section = [(0.0, KEEL), (h_c, y_c), (D, B / 2.0)];
    for draught in [0.5, h_c, 2.0, 4.0, 5.9] {
        let u = upright(&t, draught, 1.0).unwrap();
        let (v, kb, bm) = prism(&section, draught);
        close(u.immersed.volume_m3, v, "volume");
        close(u.immersed.kb_m, kb, "KB");
        close(u.bm_t_m, bm, "BM_T");
    }
}

#[test]
fn a_chine_with_no_height_or_breadth_is_the_v_bottom() {
    let v = hull(
        HullFormKind::Box,
        Bottom::VBottom {
            deadrise_deg: DEADRISE,
            keel_half_width_m: KEEL,
        },
        0.0,
    )
    .unwrap();
    let c = hull(
        HullFormKind::Box,
        Bottom::Chine {
            deadrise_deg: DEADRISE,
            keel_half_width_m: KEEL,
            chine_half_breadth_m: None,
            chine_height_m: None,
        },
        0.0,
    )
    .unwrap();
    assert_eq!(v, c);
}

#[test]
fn a_v_bottom_with_no_deadrise_and_a_full_keel_is_the_box() {
    let bx = hull(HullFormKind::Box, Bottom::Flat, 0.0).unwrap();
    let v = hull(
        HullFormKind::Box,
        Bottom::VBottom {
            deadrise_deg: 0.0,
            keel_half_width_m: B / 2.0,
        },
        0.0,
    )
    .unwrap();
    for draught in [0.5, 2.0, 3.0, 5.5] {
        let (a, b) = (
            upright(&bx, draught, 1.0).unwrap(),
            upright(&v, draught, 1.0).unwrap(),
        );
        close(b.immersed.volume_m3, a.immersed.volume_m3, "volume");
        close(b.immersed.kb_m, a.immersed.kb_m, "KB");
        close(b.immersed.lcb_m, a.immersed.lcb_m, "LCB");
        close(b.bm_t_m, a.bm_t_m, "BM_T");
        close(
            b.immersed.waterplane_area_m2,
            a.immersed.waterplane_area_m2,
            "waterplane",
        );
    }
    let g = Gravity {
        lcg_m: L / 2.0,
        tcg_m: 0.0,
        kg_m: 3.0,
    };
    let angles = [0, 10, 30, 50];
    let (a, b) = (
        gz_curve(&bx, L * B * 3.0, g, 0.0, 1.0, angles).unwrap(),
        gz_curve(&v, L * B * 3.0, g, 0.0, 1.0, angles).unwrap(),
    );
    for ((deg, x), (_, y)) in a.iter().zip(&b) {
        assert!((x - y).abs() <= 1e-12, "{deg}°: box {x} vs V-bottom {y}");
    }
}

#[test]
fn the_raked_v_bottom_is_the_raked_box_less_the_triangle_under_the_deadrise() {
    let form = HullFormKind::RakedBox {
        bow_rake_length_m: 6.0,
        bow_rake_rise_m: 1.5,
        stern_rake_length_m: 4.0,
        stern_rake_rise_m: 1.0,
    };
    let t = hull(
        form,
        Bottom::VBottom {
            deadrise_deg: DEADRISE,
            keel_half_width_m: KEEL,
        },
        0.0,
    )
    .unwrap();
    let rb = RakedBox::new(L, B, D, 6.0, 1.5, 4.0, 1.0);
    let triangle = (B / 2.0 - KEEL) * rise(DEADRISE);
    // Above the highest chine (the bow rise plus the chine rise), every section loses the triangle.
    for draught in [3.0, 5.0] {
        close(
            upright(&t, draught, 1.0).unwrap().immersed.volume_m3,
            rb.displaced_volume(draught) - triangle * L,
            "raked V-bottom volume",
        );
    }
}

#[test]
fn a_bilge_radius_rounds_the_chine_and_takes_volume() {
    let bottom = Bottom::Chine {
        deadrise_deg: DEADRISE,
        keel_half_width_m: KEEL,
        chine_half_breadth_m: Some(4.0),
        chine_height_m: Some(1.2),
    };
    let sharp = hull(HullFormKind::Box, bottom, 0.0).unwrap();
    let round = hull(HullFormKind::Box, bottom, 0.8).unwrap();
    for draught in [2.0, 4.0] {
        let (a, b) = (
            upright(&sharp, draught, 1.0).unwrap().immersed.volume_m3,
            upright(&round, draught, 1.0).unwrap().immersed.volume_m3,
        );
        assert!(b < a && b > 0.98 * a, "{draught} m: {b} vs {a}");
    }
}

#[test]
fn a_chine_outside_the_beam_or_above_the_depth_is_a_typed_error() {
    let chine = |breadth, height| Bottom::Chine {
        deadrise_deg: DEADRISE,
        keel_half_width_m: KEEL,
        chine_half_breadth_m: breadth,
        chine_height_m: height,
    };
    assert_eq!(
        hull(HullFormKind::Box, chine(Some(5.5), None), 0.0),
        Err(HullError::ChineOutsideBeam {
            half_breadth_m: 5.5,
            half_beam_m: 5.0
        })
    );
    assert_eq!(
        hull(HullFormKind::Box, chine(Some(0.2), None), 0.0),
        Err(HullError::ChineOutsideBeam {
            half_breadth_m: 0.2,
            half_beam_m: 5.0
        })
    );
    assert_eq!(
        hull(HullFormKind::Box, chine(None, Some(6.5)), 0.0),
        Err(HullError::ChineAboveDepth {
            height_m: 6.5,
            depth_m: D
        })
    );
    // A V-bottom so steep its chine would stand above the deck.
    assert!(matches!(
        hull(
            HullFormKind::Box,
            Bottom::VBottom {
                deadrise_deg: 60.0,
                keel_half_width_m: KEEL
            },
            0.0
        ),
        Err(HullError::ChineAboveDepth { .. })
    ));
    assert_eq!(
        hull(
            HullFormKind::Box,
            Bottom::VBottom {
                deadrise_deg: 90.0,
                keel_half_width_m: KEEL
            },
            0.0
        ),
        Err(HullError::Deadrise { deg: 90.0 })
    );
    assert_eq!(
        hull(
            HullFormKind::Box,
            Bottom::VBottom {
                deadrise_deg: 5.0,
                keel_half_width_m: 5.5
            },
            0.0
        ),
        Err(HullError::KeelOutsideBeam {
            keel_half_width_m: 5.5,
            half_beam_m: 5.0
        })
    );
}

#[test]
fn the_section_at_a_station_is_that_section_and_between_two_the_ruled_surface() {
    let t = hull(
        HullFormKind::RakedBox {
            bow_rake_length_m: 6.0,
            bow_rake_rise_m: 1.5,
            stern_rake_length_m: 0.0,
            stern_rake_rise_m: 0.0,
        },
        Bottom::VBottom {
            deadrise_deg: DEADRISE,
            keel_half_width_m: KEEL,
        },
        0.0,
    )
    .unwrap();
    for (i, s) in t.sections.iter().enumerate() {
        let (at, edge) = t.section_at(s.x);
        assert_eq!(&at, s);
        assert_eq!(edge, t.depth_at_side[i]);
    }
    // Half-way down the bow rake the keel is at half the rise.
    let (mid, _) = t.section_at(L - 3.0);
    close(mid.points[0].1, 0.75, "keel half-way down the rake");
}
