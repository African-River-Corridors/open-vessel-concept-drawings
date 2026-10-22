//! Seam: `ovcd_core::damage_stability`, the lost buoyancy method on the Sections Table (issue #341).
//!
//! Label: sourced - closed forms for a wall-sided box barge, worked out here, and an independent
//! numerical integration of the box less its flooded boxes for the equilibrium conditions.
//!
//! The box hull L x B x D = 60 x 10 x 4 m floats at 1,200 t (density 1.0), KG 1.8 m, LCG at mid-length.
//! A flooded double bottom 5 x 10 x 1 m under the middle of the hull at 95 % permeability takes away
//! 47.5 m3 of buoyancy. Wall-sided (the deck edge not immersed, the bottom edge not out), the immersed
//! section is the beam B times the draught T at the centre line at every heel, so T = (1200 + 47.5) /
//! (L B) = 2.0791667 m at every heel. The transverse moment of the immersed volume is L B^3 tan(phi) / 12
//! and its vertical moment L (B T^2 / 2 + B^3 tan^2(phi) / 24); the flooded box lies on the centre line
//! (y_c = 0) at height 0.5 m, so GZ = (M_y cos(phi) + (M_z / V - KG) sin(phi)) with M_y the transverse moment
//! over the net volume V = 1200 and M_z the vertical moment less 47.5 x 0.5, over V.

use libm::{cos, sin, sqrt, tan};
use ovcd_core::damage_stability::{AdnCriteria, Floats, FloodedBox, assess, righting_lever_at};
use ovcd_core::hull::hull_form::{HullFormInputs, HullFormKind, HullShape, generate};
use ovcd_core::sections::{Gravity, SectionsTable, equilibrium};

const L: f64 = 60.0;
const B: f64 = 10.0;
const D: f64 = 4.0;
const MASS: f64 = 1200.0;
const KG: f64 = 1.8;

fn hull() -> SectionsTable {
    generate(&HullFormInputs {
        length_overall_m: L,
        beam_m: B,
        moulded_depth_m: D,
        form: HullFormKind::Box,
        shape: HullShape::default(),
    })
    .unwrap()
}

fn gravity() -> Gravity {
    Gravity {
        lcg_m: L / 2.0,
        tcg_m: 0.0,
        kg_m: KG,
    }
}

fn start(table: &SectionsTable) -> (f64, f64) {
    let eq = equilibrium(table, MASS, gravity(), 0.0, 1.0).unwrap();
    (eq.waterplane.c, eq.waterplane.trim_deg)
}

fn flooded(
    x: (f64, f64),
    y: (f64, f64),
    z: (f64, f64),
    permeability: f64,
    name: &str,
) -> FloodedBox {
    FloodedBox {
        name: name.to_string(),
        x_aft_m: x.0,
        x_fwd_m: x.1,
        y_min_m: y.0,
        y_max_m: y.1,
        z_min_m: z.0,
        z_max_m: z.1,
        permeability,
    }
}

fn centred_double_bottom(permeability: f64) -> FloodedBox {
    flooded((27.5, 32.5), (-5.0, 5.0), (0.0, 1.0), permeability, "DB")
}

#[test]
fn a_flooded_double_bottom_at_mid_length_sinks_the_hull_bodily_by_the_lost_volume_over_the_waterplane()
 {
    let t = hull();
    let s = start(&t);
    let out = assess(
        &t,
        &[centred_double_bottom(0.95)],
        MASS,
        gravity(),
        1.0,
        s,
        &AdnCriteria::ADN_2025,
    );
    assert_eq!(out.floats, Floats::Yes);
    // symmetric about the centre line and about mid-length: no heel, no trim
    assert!(out.heel_deg.abs() < 1e-6, "{}", out.heel_deg);
    assert!(out.trim_deg.abs() < 1e-6, "{}", out.trim_deg);
    // T = (1200 + 0.95 x 5 x 10 x 1) / (60 x 10)
    let want = (MASS + 0.95 * 5.0 * 10.0 * 1.0) / (L * B);
    assert!(
        (out.draught_mid_m - want).abs() < 1e-6,
        "{} {want}",
        out.draught_mid_m
    );
}

#[test]
fn the_righting_lever_of_the_damaged_box_follows_the_wall_sided_closed_form() {
    let t = hull();
    let s = start(&t);
    let boxes = [centred_double_bottom(0.95)];
    let lost = 0.95 * 5.0 * 10.0 * 1.0;
    let draught = (MASS + lost) / (L * B);
    for deg in [2.0_f64, 5.0, 8.0] {
        let tn = tan(deg.to_radians());
        let (sn, cs) = (sin(deg.to_radians()), cos(deg.to_radians()));
        let b3 = B * B * B;
        let my = L * b3 * tn / 12.0;
        let mz = L * (B * draught * draught / 2.0 + b3 * tn * tn / 24.0) - lost * 0.5;
        let want = (my / MASS) * cs + (mz / MASS - KG) * sn;
        let got = righting_lever_at(&t, &boxes, MASS, gravity(), 1.0, s, deg).unwrap();
        assert!((got - want).abs() < 1e-6, "{deg} deg: {got} against {want}");
    }
}

#[test]
fn a_flooded_space_with_no_permeability_changes_nothing() {
    let t = hull();
    let s = start(&t);
    let eq = equilibrium(&t, MASS, gravity(), 0.0, 1.0).unwrap();
    let out = assess(
        &t,
        &[centred_double_bottom(0.0)],
        MASS,
        gravity(),
        1.0,
        s,
        &AdnCriteria::ADN_2025,
    );
    assert!((out.waterplane_c_m - eq.waterplane.c).abs() < 1e-6);
    assert!(out.heel_deg.abs() < 1e-6);
}

/// An independent integral: the volume and centroid of the box hull below the plane `p . k < c`, less
/// `permeability` times each flooded box's part below it, on a fine grid of cells (no use of the
/// engine's cuts). Returns (volume, x, y, z of the centroid).
fn grid_buoyancy(
    c: f64,
    trim_deg: f64,
    heel_deg: f64,
    boxes: &[FloodedBox],
) -> (f64, f64, f64, f64) {
    let (th, ph) = (trim_deg.to_radians(), heel_deg.to_radians());
    let k = (-sin(th), -sin(ph) * cos(th), cos(ph) * cos(th));
    let (nx, ny, nz) = (240usize, 80usize, 80usize);
    let (dx, dy, dz) = (L / nx as f64, B / ny as f64, D / nz as f64);
    let (mut v, mut mx, mut my, mut mz) = (0.0, 0.0, 0.0, 0.0);
    for i in 0..nx {
        let x = (i as f64 + 0.5) * dx;
        for j in 0..ny {
            let y = -B / 2.0 + (j as f64 + 0.5) * dy;
            for l in 0..nz {
                let z = (l as f64 + 0.5) * dz;
                if k.0 * x + k.1 * y + k.2 * z >= c {
                    continue;
                }
                let mut w = 1.0;
                for b in boxes {
                    if x > b.x_aft_m
                        && x < b.x_fwd_m
                        && y > b.y_min_m
                        && y < b.y_max_m
                        && z > b.z_min_m
                        && z < b.z_max_m
                    {
                        w -= b.permeability;
                    }
                }
                let cell = w * dx * dy * dz;
                v += cell;
                mx += cell * x;
                my += cell * y;
                mz += cell * z;
            }
        }
    }
    (v, mx / v, my / v, mz / v)
}

#[test]
fn a_one_sided_damage_floats_where_the_volume_and_the_two_levers_balance_by_an_independent_integral()
 {
    // A starboard wing tank 10 m long, 1 m wide, the full depth, offset from mid-length: it heels and trims.
    let t = hull();
    let s = start(&t);
    let boxes = [flooded((15.0, 25.0), (4.0, 5.0), (0.0, 4.0), 0.95, "WB")];
    let out = assess(&t, &boxes, MASS, gravity(), 1.0, s, &AdnCriteria::ADN_2025);
    assert_eq!(out.floats, Floats::Yes);
    // heels to starboard (the damaged side) and trims by the stern (the lost buoyancy is aft of LCF)
    assert!(
        out.heel_deg > 0.5 && out.heel_deg < 20.0,
        "{}",
        out.heel_deg
    );
    assert!(out.trim_deg < 0.0, "{}", out.trim_deg);
    let (v, x, y, z) = grid_buoyancy(out.waterplane_c_m, out.trim_deg, out.heel_deg, &boxes);
    // the displaced volume is the mass over the density
    assert!((v - MASS).abs() / MASS < 2e-3, "volume {v} against {MASS}");
    // the centre of buoyancy is under G in the plane of the heel: B - G is along the earth vertical, so its
    // components perpendicular to the vertical are zero. With the earth vertical k, B - G = lambda k.
    let (th, ph) = (out.trim_deg.to_radians(), out.heel_deg.to_radians());
    let k = (-sin(th), -sin(ph) * cos(th), cos(ph) * cos(th));
    let bg = (x - L / 2.0, y - 0.0, z - KG);
    let along = bg.0 * k.0 + bg.1 * k.1 + bg.2 * k.2;
    let perp = (bg.0 - along * k.0, bg.1 - along * k.1, bg.2 - along * k.2);
    let perp_len = sqrt(perp.0 * perp.0 + perp.1 * perp.1 + perp.2 * perp.2);
    assert!(
        perp_len < 0.01,
        "B is {perp_len} m off the vertical through G"
    );
}

#[test]
fn the_verdict_is_the_criteria_applied_to_the_numbers_it_reports() {
    let t = hull();
    let s = start(&t);
    for boxes in [
        vec![centred_double_bottom(0.95)],
        vec![flooded((15.0, 25.0), (4.0, 5.0), (0.0, 4.0), 0.95, "WB")],
        vec![flooded((5.0, 55.0), (-5.0, 5.0), (0.0, 4.0), 0.95, "ALL")],
    ] {
        let c = AdnCriteria::ADN_2025;
        let out = assess(&t, &boxes, MASS, gravity(), 1.0, s, &c);
        let want = out.floats == Floats::Yes
            && out.heel_deg.abs() <= c.max_heel_deg
            && out.max_gz_m >= c.min_gz_m
            && out.area_m_rad >= c.min_area_m_rad;
        assert_eq!(out.ok, want, "{boxes:?} {out:?}");
    }
    // a harsher damage fails a criterion that a milder one meets: flooding the whole hull sinks it
    let all = [flooded((0.0, 60.0), (-5.0, 5.0), (0.0, 4.0), 0.95, "ALL")];
    let out = assess(&t, &all, MASS, gravity(), 1.0, s, &AdnCriteria::ADN_2025);
    assert!(!out.ok);
    assert_ne!(out.floats, Floats::Yes);
    // and the mild one passes
    let mild = assess(
        &t,
        &[centred_double_bottom(0.95)],
        MASS,
        gravity(),
        1.0,
        s,
        &AdnCriteria::ADN_2025,
    );
    assert!(mild.ok, "{mild:?}");
}

// ---- the damage cases of a layout -----------------------------------------------------------------

use ovcd_core::compartments::{Side, Space, SpaceKind};
use ovcd_core::liquid_bulk::damage::{AdnExtents, damage_cases, longitudinal_extent_m};

fn space(name: &str, kind: SpaceKind, x: (f64, f64), y: (f64, f64), z: (f64, f64)) -> Space {
    Space {
        name: name.to_string(),
        kind,
        side: Side::Starboard,
        x_aft: x.0,
        x_fwd: x.1,
        section: vec![(y.0, z.0), (y.1, z.0), (y.1, z.1), (y.0, z.1)],
    }
}

/// ADN 9.3.2.15.1 (a): the side damage reaches 0.79 m inboard of the side for type C. 9.3.3.15.1 (a): 0.59 m
/// for type N. A space 0.35 to 0.80 m inboard of the side, of a hull 10 m wide, is reached by the first and
/// not by the second at its inner part: a ballast space 0.65 to 1.00 m from the side.
#[test]
fn the_side_damage_reaches_079_m_inboard_for_type_c_and_059_m_for_type_n() {
    let spaces = vec![
        space(
            "WB1S",
            SpaceKind::WingBallast,
            (10.0, 50.0),
            (4.8, 5.0),
            (0.0, 4.0),
        ),
        space(
            "VD1S",
            SpaceKind::PeakBallast,
            (10.0, 50.0),
            (4.0, 4.35),
            (0.0, 4.0),
        ),
        space(
            "1",
            SpaceKind::CargoTank,
            (10.0, 50.0),
            (0.0, 4.0),
            (0.7, 4.0),
        ),
    ];
    let flooded = |ext: &AdnExtents| -> Vec<String> {
        damage_cases(&spaces, 60.0, 10.0, ext)
            .into_iter()
            .filter(|c| c.name.starts_with("side"))
            .flat_map(|c| c.flooded.into_iter().map(|f| f.name))
            .collect()
    };
    let c = flooded(&AdnExtents::TYPE_C);
    let n = flooded(&AdnExtents::TYPE_N);
    // 5.0 - 0.79 = 4.21 < 4.35: the inner space is reached by type C; 5.0 - 0.59 = 4.41 > 4.35: not by type N
    assert!(c.contains(&"VD1S".to_string()), "{c:?}");
    assert!(!n.contains(&"VD1S".to_string()), "{n:?}");
    assert!(n.contains(&"WB1S".to_string()) && c.contains(&"WB1S".to_string()));
    // the cargo tank is full at the maximum draught: its permeability is 0 and it is never listed as flooded
    assert!(!c.contains(&"1".to_string()) && !n.contains(&"1".to_string()));
}

#[test]
fn the_longitudinal_extent_is_a_tenth_of_the_length_but_not_under_5_m() {
    assert_eq!(longitudinal_extent_m(60.0), 6.0);
    assert_eq!(longitudinal_extent_m(40.0), 5.0);
    assert_eq!(longitudinal_extent_m(76.5), 7.65);
}

#[test]
fn a_ballast_space_is_cut_at_the_bulkheads_of_the_tanks_beside_it_and_a_window_floods_what_it_touches()
 {
    // two cargo tanks, 10 to 30 and 30 to 50, and a wing tank along both: the wing is two compartments.
    let spaces = vec![
        space(
            "1",
            SpaceKind::CargoTank,
            (10.0, 30.0),
            (0.0, 4.0),
            (0.7, 4.0),
        ),
        space(
            "2",
            SpaceKind::CargoTank,
            (30.0, 50.0),
            (0.0, 4.0),
            (0.7, 4.0),
        ),
        space(
            "WB1S",
            SpaceKind::WingBallast,
            (10.0, 50.0),
            (4.0, 5.0),
            (0.0, 4.0),
        ),
    ];
    let cases = damage_cases(&spaces, 60.0, 10.0, &AdnExtents::TYPE_C);
    let side: Vec<Vec<String>> = cases
        .iter()
        .filter(|c| c.name.starts_with("side"))
        .map(|c| c.flooded.iter().map(|f| f.name.clone()).collect())
        .collect();
    // a window of 6 m over the bulkhead at 30 m floods both halves; a window inside one tank floods one
    assert!(side.iter().any(|f| f == &["WB1S/1", "WB1S/2"]), "{side:?}");
    assert!(side.iter().any(|f| f == &["WB1S/1"]), "{side:?}");
    assert!(side.iter().any(|f| f == &["WB1S/2"]), "{side:?}");
    // no damage floods the whole length of the wing tank at once
    assert!(side.iter().all(|f| f.len() <= 2));
}
