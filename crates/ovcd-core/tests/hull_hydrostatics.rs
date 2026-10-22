//! Seam: `hydrostatics::Hull`, the hull as its Sections Table (issue #448, spec 0008).
//!
//! Label: sourced - closed form, box: ∇ = L B T, KB = T/2, BM_T = B²/12T, Cb = 1; the raked box
//! against the `RakedBox` closed forms of `hull.rs` (prism and wedge); the Stern Form against
//! `SternGeometry::removed`. The free-trim float and the table are properties.

use ovcd_core::hull::HullGeometry;
use ovcd_core::hydrostatics::{Hull, HullSpec, TABLE_ROWS};
use ovcd_core::sections::Gravity;
use ovcd_core::stern_form::{HullDims, SternForm, SternFormInputs, geometry};

const L: f64 = 90.0;
const B: f64 = 14.0;
const D: f64 = 4.5;
const RHO: f64 = 1.0;

fn spec(rake: f64) -> HullSpec {
    HullSpec {
        loa: L,
        beam: B,
        depth: D,
        bow_rake_length_m: rake,
        bow_rake_rise_m: D,
        stern: None,
    }
}

fn rel(a: f64, b: f64) -> f64 {
    (a - b).abs() / b.abs().max(1.0)
}

#[test]
fn a_box_has_its_closed_form_hydrostatics() {
    let h = Hull::build(&spec(0.0)).unwrap();
    for t in [0.5, 2.0, 3.5] {
        let u = h.upright(t, RHO).unwrap();
        assert!(rel(u.immersed.volume_m3, L * B * t) < 1e-12, "{t}");
        assert!(rel(u.immersed.kb_m, t / 2.0) < 1e-12, "{t}");
        assert!(rel(u.bm_t_m, B * B / (12.0 * t)) < 1e-12, "{t}");
        assert!(rel(u.cb, 1.0) < 1e-12, "{t}");
        // The level draught for that displacement is the draught.
        let back = h.level_draught(u.displacement_t, RHO).unwrap();
        assert!((back - t).abs() < 1e-9, "{t}: {back}");
    }
}

#[test]
fn a_raked_box_matches_the_raked_box_closed_forms() {
    let rake = 1.6 * D;
    let h = Hull::build(&spec(rake)).unwrap();
    let closed = HullGeometry::new(L, B, D, rake, D, 0.0, 0.0);
    for t in [1.0, 2.5, 4.0] {
        assert!(
            rel(h.volume_at(t), closed.displaced_volume(t)) < 1e-12,
            "{t}"
        );
        let u = h.upright(t, RHO).unwrap();
        assert!(rel(u.immersed.waterplane_area_m2, closed.waterplane_area(t)) < 1e-12);
        // Cb on L_WL B_WL T: the raked bow shortens the waterline, so Cb < 1.
        assert!(u.cb < 1.0 && u.cb > 0.9, "{}", u.cb);
    }
}

#[test]
fn a_stern_form_removes_its_closed_form_volume() {
    let g = geometry(
        &SternFormInputs {
            form: SternForm::Transom,
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
            design_draught: 3.5,
        },
    );
    let square = Hull::build(&spec(1.6 * D)).unwrap();
    let mut s = spec(1.6 * D);
    s.stern = Some(g.clone());
    let cut = Hull::build(&s).unwrap();
    for t in [1.5, 3.0, 4.0] {
        let removed = square.volume_at(t) - cut.volume_at(t);
        assert!(
            (removed - g.removed(t).0).abs() <= 1e-12 * square.volume_at(t),
            "{t}"
        );
    }
}

#[test]
fn the_free_trim_float_is_level_when_g_is_over_b_and_trims_by_the_bow_when_g_is_forward() {
    let h = Hull::build(&spec(1.6 * D)).unwrap();
    let u = h.upright(3.0, RHO).unwrap();
    let mass = u.displacement_t;
    let over_b = Gravity {
        lcg_m: u.immersed.lcb_m,
        tcg_m: 0.0,
        kg_m: 3.0,
    };
    let f = h.float(mass, over_b, RHO).unwrap();
    assert!(f.trim_m.abs() < 1e-6, "{}", f.trim_m);
    assert!((f.draught_mid_m - 3.0).abs() < 1e-6, "{}", f.draught_mid_m);
    let forward = Gravity {
        lcg_m: u.immersed.lcb_m + 2.0,
        ..over_b
    };
    let f = h.float(mass, forward, RHO).unwrap();
    assert!(f.trim_m > 0.0);
    assert!(f.draught_fwd_m > f.draught_mid_m && f.draught_mid_m > f.draught_aft_m);
    assert!(((f.draught_aft_m + f.draught_fwd_m) / 2.0 - f.draught_mid_m).abs() < 1e-12);
}

#[test]
fn the_hydrostatic_table_has_ten_rows_from_a_tenth_of_the_depth_to_the_depth() {
    let h = Hull::build(&spec(1.6 * D)).unwrap();
    let rows = h.hydrostatic_table(RHO).unwrap();
    assert_eq!(rows.len(), TABLE_ROWS as usize);
    assert!(rel(rows[0].draught_m, 0.1 * D) < 1e-12);
    assert_eq!(rows[rows.len() - 1].draught_m, D);
    for w in rows.windows(2) {
        assert!(w[1].displacement_t > w[0].displacement_t);
    }
}
