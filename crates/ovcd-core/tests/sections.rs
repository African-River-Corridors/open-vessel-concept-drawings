//! Seam: `ovcd_core::sections`, the Sections Table and the exact polygon integrator (issue #443,
//! spec 0008 "The Sections Table" and "Exact polygon integration").
//!
//! The box helper here is for these tests only; the real generators come with #445.
//!
//! Label: sourced - closed form, box. Upright: ∇ = LBT, KB = T/2, BM_T = B²/12T, BM_L = L²/12T,
//! LCB = LCF = L/2, Cb = Cm = Cwp = 1. Trimmed at zero heel, the immersed depth is linear in x:
//! T(x) = (c + x sin θ) / cos θ, so ∇ = B ∫T dx and LCB = ∫x T dx / ∫T dx (a prism and a wedge).
//! Heeled, the centre of buoyancy is checked against `box_stability::box_gz`'s polygon result. The
//! rest are properties.

use ovcd_core::box_stability::box_gz;
use ovcd_core::sections::{Section, SectionsError, SectionsTable, Waterplane, immersed, upright};

const L: f64 = 60.0;
const B: f64 = 11.4;
const D: f64 = 4.0;

fn box_half(b: f64, d: f64) -> Vec<(f64, f64)> {
    vec![(0.0, 0.0), (b / 2.0, 0.0), (b / 2.0, d), (0.0, d)]
}

/// A box L x B x D as a Sections Table: two stations, a rectangle at each.
fn box_table_at(stations: &[f64], b: f64, d: f64) -> SectionsTable {
    let sections = stations
        .iter()
        .map(|&x| Section {
            x,
            points: box_half(b, d),
        })
        .collect();
    SectionsTable::new(sections, vec![d; stations.len()]).unwrap()
}

fn box_table() -> SectionsTable {
    box_table_at(&[0.0, L], B, D)
}

fn rel(a: f64, b: f64) -> f64 {
    (a - b).abs() / b.abs().max(1.0)
}

fn close(a: f64, b: f64, what: &str) {
    assert!(rel(a, b) <= 1e-12, "{what}: {a} vs {b}");
}

#[test]
fn an_upright_box_matches_its_closed_forms_at_several_draughts() {
    let t = box_table();
    for draught in [0.25, 1.0, 2.5, 3.2, 3.99] {
        let u = upright(&t, draught, 1.0).unwrap();
        let im = u.immersed;
        close(im.volume_m3, L * B * draught, "volume");
        close(im.kb_m, draught / 2.0, "KB");
        close(u.bm_t_m, B * B / (12.0 * draught), "BM_T");
        close(u.bm_l_m, L * L / (12.0 * draught), "BM_L");
        close(im.lcb_m, L / 2.0, "LCB");
        close(im.lcf_m, L / 2.0, "LCF");
        close(im.waterplane_area_m2, L * B, "A_WP");
        close(im.l_wl_m, L, "L_WL");
        close(im.b_wl_m, B, "B_WL");
        close(im.midship_area_m2, B * draught, "A_M");
        close(u.cb, 1.0, "Cb");
        close(u.cm, 1.0, "Cm");
        close(u.cp, 1.0, "Cp");
        close(u.cwp, 1.0, "Cwp");
        close(u.km_t_m, draught / 2.0 + B * B / (12.0 * draught), "KM_T");
        close(u.tpc_t, L * B / 100.0, "TPC");
        // Bottom, two sides and the two end faces.
        close(
            im.wetted_surface_m2,
            L * B + 2.0 * L * draught + 2.0 * B * draught,
            "wetted surface",
        );
        assert!(im.tcb_m.abs() <= 1e-12 && im.tcf_m.abs() <= 1e-12);
    }
}

/// The height `c` that immerses `target` m³ at this heel and trim, by 80 bisections, as `box_gz`.
fn c_for_volume(t: &SectionsTable, heel: f64, trim: f64, target: f64) -> f64 {
    let k = Waterplane {
        heel_deg: heel,
        trim_deg: trim,
        c: 0.0,
    }
    .normal();
    let dots: Vec<f64> = t
        .sections
        .iter()
        .flat_map(|s| {
            s.points.iter().flat_map(move |&(y, z)| {
                [k.0 * s.x + k.1 * y + k.2 * z, k.0 * s.x - k.1 * y + k.2 * z]
            })
        })
        .collect();
    let (mut lo, mut hi) = (
        dots.iter().copied().fold(f64::INFINITY, f64::min),
        dots.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    );
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        let v = immersed(
            t,
            &Waterplane {
                heel_deg: heel,
                trim_deg: trim,
                c: mid,
            },
        )
        .volume_m3;
        if v < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

#[test]
fn a_heeled_box_has_the_centre_of_buoyancy_of_the_box_gz_polygon() {
    let t = box_table();
    for draught in [1.5, 2.6] {
        for deg in 0..=60 {
            let heel = f64::from(deg);
            let c = c_for_volume(&t, heel, 0.0, L * B * draught);
            let wp = Waterplane {
                heel_deg: heel,
                trim_deg: 0.0,
                c,
            };
            let im = immersed(&t, &wp);
            close(im.volume_m3, L * B * draught, "volume");
            // Horizontal distance of B from the centre line, across the waterline: with KG = 0,
            // that is box_gz (box_gz heels to port, so its GZ is our B's offset to starboard).
            let (_, ky, kz) = wp.normal();
            let s_b = kz * im.tcb_m - ky * im.kb_m;
            let want = box_gz(B, D, draught, 0.0, heel).unwrap();
            assert!(
                (s_b - want).abs() <= 1e-12,
                "T {draught} heel {deg}: {s_b} vs {want}"
            );
            close(im.lcb_m, L / 2.0, "LCB");
        }
    }
}

#[test]
fn a_trimmed_box_matches_the_prism_and_wedge_closed_form() {
    let t = box_table();
    for (c, trim) in [(2.0, 1.5), (2.0, -1.5), (1.2, 0.8), (3.0, -2.0)] {
        let im = immersed(
            &t,
            &Waterplane {
                heel_deg: 0.0,
                trim_deg: trim,
                c,
            },
        );
        let (s, ct) = (libm::sin(trim.to_radians()), libm::cos(trim.to_radians()));
        // Immersed depth at both ends stays between the keel and the deck.
        let (t_aft, t_fwd) = (c / ct, (c + s * L) / ct);
        assert!(t_aft > 0.0 && t_aft < D && t_fwd > 0.0 && t_fwd < D);
        let area = c * L + s * L * L / 2.0;
        let moment = c * L * L / 2.0 + s * L * L * L / 3.0;
        close(im.volume_m3, B * area / ct, "volume");
        close(im.lcb_m, moment / area, "LCB");
        assert!(im.tcb_m.abs() <= 1e-12);
    }
}

#[test]
fn volume_rises_with_the_height_of_the_waterplane() {
    let t = box_table();
    for (heel, trim) in [(0.0, 0.0), (20.0, 0.0), (45.0, 3.0), (-30.0, -2.0)] {
        let mut last = -1.0;
        for i in -20..=120 {
            let c = f64::from(i) * 0.05;
            let v = immersed(
                &t,
                &Waterplane {
                    heel_deg: heel,
                    trim_deg: trim,
                    c,
                },
            )
            .volume_m3;
            assert!(v >= last, "heel {heel} trim {trim} c {c}: {v} < {last}");
            last = v;
        }
    }
}

#[test]
fn splitting_a_prismatic_interval_changes_nothing() {
    let (one, three) = (box_table(), box_table_at(&[0.0, 17.0, 41.5, L], B, D));
    for wp in [
        Waterplane::upright(2.2),
        Waterplane {
            heel_deg: 25.0,
            trim_deg: 0.0,
            c: 2.0,
        },
        Waterplane {
            heel_deg: 10.0,
            trim_deg: 1.5,
            c: 1.8,
        },
    ] {
        let (a, b) = (immersed(&one, &wp), immersed(&three, &wp));
        for (x, y, what) in [
            (a.volume_m3, b.volume_m3, "volume"),
            (a.lcb_m, b.lcb_m, "LCB"),
            (a.tcb_m, b.tcb_m, "TCB"),
            (a.kb_m, b.kb_m, "KB"),
            (a.waterplane_area_m2, b.waterplane_area_m2, "A_WP"),
            (a.lcf_m, b.lcf_m, "LCF"),
            (a.i_t_m4, b.i_t_m4, "I_T"),
            (a.i_l_m4, b.i_l_m4, "I_L"),
            (a.wetted_surface_m2, b.wetted_surface_m2, "wetted surface"),
        ] {
            close(x, y, what);
        }
    }
}

#[test]
fn a_symmetric_table_has_no_transverse_offset_upright() {
    // A section with a sloped bottom and a flared side.
    let half = vec![(0.0, 0.0), (3.0, 0.4), (5.0, 1.5), (5.6, 4.0), (0.0, 4.2)];
    let t = SectionsTable::new(
        vec![
            Section {
                x: 0.0,
                points: half.clone(),
            },
            Section {
                x: 30.0,
                points: half,
            },
        ],
        vec![4.0, 4.0],
    )
    .unwrap();
    for draught in [0.3, 1.0, 2.5, 3.9] {
        let im = immersed(&t, &Waterplane::upright(draught));
        assert!(im.volume_m3 > 0.0);
        assert!(im.tcb_m.abs() <= 1e-12, "{draught}: TCB {}", im.tcb_m);
        assert!(im.tcf_m.abs() <= 1e-12, "{draught}: TCF {}", im.tcf_m);
    }
}

#[test]
fn moving_the_table_along_x_moves_only_the_longitudinal_centres() {
    let shift = 23.75;
    let (a_t, b_t) = (box_table(), box_table_at(&[shift, L + shift], B, D));
    for (heel, trim, c) in [(0.0, 0.0, 2.0), (30.0, 0.0, 1.5), (5.0, 1.2, 2.1)] {
        let s = libm::sin(f64::to_radians(trim));
        let wa = Waterplane {
            heel_deg: heel,
            trim_deg: trim,
            c,
        };
        // k_x = −sin θ, so the same immersion after the shift needs c − sin θ · shift.
        let wb = Waterplane {
            c: c - s * shift,
            ..wa
        };
        let (a, b) = (immersed(&a_t, &wa), immersed(&b_t, &wb));
        close(a.volume_m3, b.volume_m3, "volume");
        close(a.lcb_m + shift, b.lcb_m, "LCB");
        close(a.lcf_m + shift, b.lcf_m, "LCF");
        close(a.tcb_m, b.tcb_m, "TCB");
        close(a.kb_m, b.kb_m, "KB");
        close(a.i_l_m4, b.i_l_m4, "I_L");
    }
}

#[test]
fn a_dry_hull_has_no_upright_hydrostatics() {
    assert_eq!(
        upright(&box_table(), 0.0, 1.0),
        Err(SectionsError::NotAfloat)
    );
    assert_eq!(
        upright(&box_table(), -1.0, 1.0),
        Err(SectionsError::NotAfloat)
    );
}

fn sec(x: f64, points: Vec<(f64, f64)>) -> Section {
    Section { x, points }
}

#[test]
fn the_constructor_refuses_each_bad_table() {
    let good = box_half(B, D);
    let check = |sections: Vec<Section>, depths: Vec<f64>, want: SectionsError| {
        assert_eq!(SectionsTable::new(sections, depths), Err(want));
    };
    check(
        vec![sec(0.0, good.clone())],
        vec![D],
        SectionsError::TooFewSections { count: 1 },
    );
    check(
        vec![sec(0.0, good.clone()), sec(L, good.clone())],
        vec![D],
        SectionsError::DepthCount {
            sections: 2,
            depths: 1,
        },
    );
    check(
        vec![sec(0.0, good.clone()), sec(f64::NAN, good.clone())],
        vec![D, D],
        SectionsError::NotFinite { section: 1 },
    );
    check(
        vec![sec(L, good.clone()), sec(0.0, good.clone())],
        vec![D, D],
        SectionsError::XDescending { section: 1 },
    );
    check(
        vec![sec(0.0, vec![(0.0, 0.0)]), sec(L, vec![(0.0, 0.0)])],
        vec![D, D],
        SectionsError::TooFewPoints { section: 0 },
    );
    check(
        vec![sec(0.0, good.clone()), sec(L, good[..3].to_vec())],
        vec![D, D],
        SectionsError::PointCount {
            section: 1,
            expected: 4,
            found: 3,
        },
    );
    check(
        vec![
            sec(0.0, good.clone()),
            sec(L, vec![(0.0, 0.0), (B / 2.0, 0.0), (B / 2.0, D), (0.5, D)]),
        ],
        vec![D, D],
        SectionsError::OffCentreLine { section: 1 },
    );
    check(
        vec![
            sec(0.0, vec![(0.0, 0.0), (-1.0, 0.0), (B / 2.0, D), (0.0, D)]),
            sec(L, good.clone()),
        ],
        vec![D, D],
        SectionsError::NegativeY {
            section: 0,
            point: 1,
        },
    );
    // A bow-tie: the side runs back below the bottom.
    check(
        vec![
            sec(0.0, good.clone()),
            sec(L, vec![(0.0, 0.0), (B / 2.0, D), (B / 2.0, 0.0), (0.0, D)]),
        ],
        vec![D, D],
        SectionsError::SelfCrossing { section: 1 },
    );
}

#[test]
fn a_stem_section_on_the_centre_line_and_a_step_are_allowed() {
    // A wedge bow: the last station is a line on the centre line (zero area), and a transom step
    // at x = 0, where the aft section is a point line and the next is the transom.
    let stem = vec![(0.0, 0.0), (0.0, 0.0), (0.0, D), (0.0, D)];
    let t = SectionsTable::new(
        vec![
            sec(0.0, stem.clone()),
            sec(0.0, box_half(B, D)),
            sec(L - 10.0, box_half(B, D)),
            sec(L, stem),
        ],
        vec![D; 4],
    )
    .unwrap();
    let draught = 2.0;
    let im = immersed(&t, &Waterplane::upright(draught));
    // A prism of L − 10 and a wedge of length 10 whose breadth falls linearly to 0.
    let v = B * draught * (L - 10.0) + B * draught * 10.0 / 2.0;
    close(im.volume_m3, v, "volume");
    close(im.l_wl_m, L, "L_WL");
    // The transom face counts in the wetted surface; the stem has none.
    let side_wedge = 2.0 * draught * libm::sqrt(100.0 + (B / 2.0) * (B / 2.0));
    let ws =
        B * (L - 10.0) + B * 10.0 / 2.0 + 2.0 * draught * (L - 10.0) + side_wedge + B * draught;
    close(im.wetted_surface_m2, ws, "wetted surface");
}
