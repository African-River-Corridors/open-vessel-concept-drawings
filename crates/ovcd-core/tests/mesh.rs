//! Seam: `ovcd_core::mesh`, the meshes of the Vessel Model (issue #456, spec 0008 "Phase 2: meshes
//! and the parity hash").
//!
//! Label: property - the hull mesh is closed (every edge in exactly two triangles, once in each
//! direction) for a box, a raked box, a hull with bilge radius, sheer and camber, each Stern Form
//! and a barge with a Notch (issue #465); its volume by the divergence theorem is the sections
//! volume within the ruled-quad error. A ruled quad and its two triangles differ by half the
//! volume of the tetrahedron of its four corners (the ruled surface halves it), so the bound is the
//! sum of those half volumes, plus the f32 rounding (1e-6 of the volume). Label: sourced - closed
//! form, a box: the triangles are the quads, so the mesh volume is L B D.

use std::collections::HashMap;

use ovcd_core::compartments::{Side, Space, SpaceKind};
use ovcd_core::hull::hull_form::{Bottom, HullFormInputs, HullFormKind, HullShape, generate};
use ovcd_core::mesh::{Mesh, hull_mesh, space_mesh, waterline_mesh};
use ovcd_core::sections::{SectionsTable, Waterplane, immersed};
use ovcd_core::stern_form::{
    HullDims, Notch, SternForm, SternFormInputs, apply, apply_notch, geometry,
};

const L: f64 = 80.0;
const B: f64 = 11.4;
const D: f64 = 4.5;

fn hull(form: HullFormKind, shape: HullShape) -> SectionsTable {
    generate(&HullFormInputs {
        length_overall_m: L,
        beam_m: B,
        moulded_depth_m: D,
        form,
        shape,
    })
    .unwrap()
}

fn raked() -> HullFormKind {
    HullFormKind::RakedBox {
        bow_rake_length_m: 7.2,
        bow_rake_rise_m: D,
        stern_rake_length_m: 3.0,
        stern_rake_rise_m: 1.5,
    }
}

fn square_stern() -> SectionsTable {
    hull(
        HullFormKind::RakedBox {
            bow_rake_length_m: 7.2,
            bow_rake_rise_m: D,
            stern_rake_length_m: 0.0,
            stern_rake_rise_m: 0.0,
        },
        HullShape::default(),
    )
}

fn with_stern(form: SternForm) -> SectionsTable {
    let g = geometry(
        &SternFormInputs {
            form,
            run_length_m: None,
            transom_height_m: None,
            tunnel_crown_height_m: None,
            propulsor_count: 2,
            propeller_diameter_m: None,
            max_unit_diameter_m: None,
            parallel_shafts: true,
        },
        &HullDims {
            loa: L,
            beam: B,
            depth: D,
            design_draught: 3.2,
        },
    );
    let mut t = square_stern();
    apply(&g, &mut t).unwrap();
    t
}

fn cases() -> Vec<(&'static str, SectionsTable)> {
    let shaped = HullShape {
        bilge_radius_m: 1.2,
        sheer_aft_m: 0.3,
        sheer_fwd_m: 0.6,
        camber_m: 0.15,
        ..HullShape::default()
    };
    let vee = HullShape {
        bottom: Bottom::VBottom {
            deadrise_deg: 8.0,
            keel_half_width_m: 0.5,
        },
        ..HullShape::default()
    };
    let mut notched = hull(HullFormKind::Box, HullShape::default());
    apply_notch(
        &Notch {
            depth_m: 10.0,
            width_m: 4.0,
        },
        &mut notched,
    )
    .unwrap();
    let mut notched_raked = hull(raked(), HullShape::default());
    apply_notch(
        &Notch {
            depth_m: 2.0,
            width_m: 4.0,
        },
        &mut notched_raked,
    )
    .unwrap();
    vec![
        ("box", hull(HullFormKind::Box, HullShape::default())),
        ("raked box", hull(raked(), HullShape::default())),
        ("bilge, sheer, camber", hull(raked(), shaped)),
        ("V-bottom", hull(raked(), vee)),
        ("transom", with_stern(SternForm::Transom)),
        ("pram", with_stern(SternForm::Pram)),
        ("tunnel", with_stern(SternForm::Tunnel)),
        ("notched box", notched),
        ("notch in a stern rake", notched_raked),
    ]
}

/// Every undirected edge is in exactly two triangles, once in each direction.
fn assert_closed(m: &Mesh, what: &str) {
    let mut count: HashMap<(u32, u32), i32> = HashMap::new();
    for t in m.indices.as_chunks::<3>().0 {
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            assert_ne!(a, b, "{what}: a triangle with a repeated vertex");
            *count.entry((a, b)).or_default() += 1;
        }
    }
    for (&(a, b), &n) in &count {
        assert_eq!(n, 1, "{what}: edge {a}-{b} used {n} times in one direction");
        assert_eq!(
            count.get(&(b, a)).copied().unwrap_or(0),
            1,
            "{what}: edge {a}-{b} has no opposite: {:?} {:?}",
            m.vertex(a),
            m.vertex(b)
        );
    }
}

/// The sum of the half tetrahedron volumes of the ruled quads: the bound on the difference between
/// the triangles and the ruled surface.
fn quad_error(t: &SectionsTable) -> f64 {
    let tet = |a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3]| {
        let (u, v, w) = (
            [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
            [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
            [d[0] - a[0], d[1] - a[1], d[2] - a[2]],
        );
        (u[0] * (v[1] * w[2] - v[2] * w[1]) - u[1] * (v[0] * w[2] - v[2] * w[0])
            + u[2] * (v[0] * w[1] - v[1] * w[0]))
            .abs()
            / 6.0
    };
    let mut e = 0.0;
    for p in t.sections.windows(2) {
        let (s0, s1) = (&p[0], &p[1]);
        let n = s0.points.len();
        for i in 0..n - 1 {
            let at =
                |s: &ovcd_core::sections::Section, j: usize| [s.x, s.points[j].0, s.points[j].1];
            // Both sides.
            e += 2.0 * tet(at(s0, i), at(s0, i + 1), at(s1, i + 1), at(s1, i)) / 2.0;
        }
    }
    e
}

#[test]
fn every_hull_mesh_is_closed_and_has_the_sections_volume() {
    for (what, t) in cases() {
        let m = hull_mesh(&t);
        assert_closed(&m, what);
        let v = m.volume();
        let sections = immersed(&t, &Waterplane::upright(1e6)).volume_m3;
        let bound = quad_error(&t) + 1e-6 * sections;
        assert!(
            (v - sections).abs() <= bound,
            "{what}: mesh {v} vs sections {sections}, bound {bound}"
        );
        assert!(m.groups.is_empty() && m.positions.len() % 3 == 0);
    }
}

#[test]
fn a_box_mesh_has_the_box_volume() {
    let m = hull_mesh(&hull(HullFormKind::Box, HullShape::default()));
    let v = L * B * D;
    assert!((m.volume() - v).abs() <= 1e-6 * v, "{} vs {v}", m.volume());
}

#[test]
fn a_space_prism_is_closed_and_has_its_volume() {
    let s = Space {
        name: "DB1S".into(),
        kind: SpaceKind::DoubleBottomBallast,
        side: Side::Starboard,
        x_aft: 5.0,
        x_fwd: 45.0,
        section: vec![(1.0, 0.0), (5.7, 0.0), (5.7, 1.2), (1.0, 1.2)],
    };
    let m = space_mesh(&s);
    assert_closed(&m, "space");
    assert!((m.volume() - s.volume_m3()).abs() <= 1e-6 * s.volume_m3());
}

#[test]
fn a_waterline_spans_the_hull_at_its_draughts() {
    let t = hull(raked(), HullShape::default());
    let m = waterline_mesh(&t, 2.0, 3.0);
    let zs: Vec<f32> = m.positions.chunks(3).map(|p| p[2]).collect();
    let (lo, hi) = zs
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &z| (a.min(z), b.max(z)));
    assert_eq!((lo, hi), (2.0, 3.0));
    let ys: f32 = m
        .positions
        .chunks(3)
        .map(|p| p[1].abs())
        .fold(0.0, f32::max);
    assert_eq!(ys, (B / 2.0) as f32);
}
