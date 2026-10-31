//! Seam: `ovd_core::damage_stability`, the lost buoyancy method on the Sections Table (issue #341).
//!
//! Label: sourced - closed forms for a wall-sided box barge, worked out here, and an independent
//! numerical integration of the box less its flooded boxes for the equilibrium conditions; for the Flooded
//! Volumes of spec 0012, closed forms of a tank and a hold, worked out at the section below.
//!
//! The box hull L x B x D = 60 x 10 x 4 m floats at 1,200 t (density 1.0), KG 1.8 m, LCG at mid-length.
//! A flooded double bottom 5 x 10 x 1 m under the middle of the hull at 95 % permeability takes away
//! 47.5 m3 of buoyancy. Wall-sided (the deck edge not immersed, the bottom edge not out), the immersed
//! section is the beam B times the draught T at the centre line at every heel, so T = (1200 + 47.5) /
//! (L B) = 2.0791667 m at every heel. The transverse moment of the immersed volume is L B^3 tan(phi) / 12
//! and its vertical moment L (B T^2 / 2 + B^3 tan^2(phi) / 24); the flooded box lies on the centre line
//! (y_c = 0) at height 0.5 m, so GZ = (M_y cos(phi) + (M_z / V - KG) sin(phi)) with M_y the transverse moment
//! over the net volume V = 1200 and M_z the vertical moment less 47.5 x 0.5, over V.

use libm::{acos, cos, sin, sqrt, tan};
use ovd_core::damage_stability::{
    DamageCriteria, Floats, FloodedVolume, IntactInclusion, assess, righting_lever_at,
};
use ovd_core::hull::hull_form::{HullFormInputs, HullFormKind, HullShape, generate};
use ovd_core::sections::{Gravity, SectionsTable, equilibrium};

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
) -> FloodedVolume {
    FloodedVolume::from_box(name, x, y, z, permeability)
}

fn centred_double_bottom(permeability: f64) -> FloodedVolume {
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
        &DamageCriteria::ADN_2025,
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
        &DamageCriteria::ADN_2025,
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
    boxes: &[FloodedVolume],
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
                    let (y0, y1, z0, z1) = box_bounds(b);
                    if x > b.x_aft_m && x < b.x_fwd_m && y > y0 && y < y1 && z > z0 && z < z1 {
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

/// The (y, z) bounds of a box's section.
fn box_bounds(b: &FloodedVolume) -> (f64, f64, f64, f64) {
    let SpaceShape::Prism { section } = &b.shape else {
        panic!("not a box: {b:?}");
    };
    section.iter().fold(
        (f64::MAX, f64::MIN, f64::MAX, f64::MIN),
        |(y0, y1, z0, z1), &(y, z)| (y0.min(y), y1.max(y), z0.min(z), z1.max(z)),
    )
}

#[test]
fn a_one_sided_damage_floats_where_the_volume_and_the_two_levers_balance_by_an_independent_integral()
 {
    // A starboard wing tank 10 m long, 1 m wide, the full depth, offset from mid-length: it heels and trims.
    let t = hull();
    let s = start(&t);
    let boxes = [flooded((15.0, 25.0), (4.0, 5.0), (0.0, 4.0), 0.95, "WB")];
    let out = assess(
        &t,
        &boxes,
        MASS,
        gravity(),
        1.0,
        s,
        &DamageCriteria::ADN_2025,
    );
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
        let c = DamageCriteria::ADN_2025;
        let out = assess(&t, &boxes, MASS, gravity(), 1.0, s, &c);
        let want = out.floats == Floats::Yes
            && out.heel_deg.abs() <= c.max_heel_deg
            && out.max_gz_m >= c.min_gz_m
            && out.area_m_rad >= c.min_area_m_rad;
        assert_eq!(out.ok, want, "{boxes:?} {out:?}");
    }
    // a harsher damage fails a criterion that a milder one meets: flooding the whole hull sinks it
    let all = [flooded((0.0, 60.0), (-5.0, 5.0), (0.0, 4.0), 0.95, "ALL")];
    let out = assess(&t, &all, MASS, gravity(), 1.0, s, &DamageCriteria::ADN_2025);
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
        &DamageCriteria::ADN_2025,
    );
    assert!(mild.ok, "{mild:?}");
}

// ---- the damage cases of a layout -----------------------------------------------------------------

use ovd_core::compartments::{Side, Space, SpaceKind, SpaceShape};
use ovd_core::liquid_bulk::damage::{AdnExtents, damage_cases, longitudinal_extent_m};

fn space(name: &str, kind: SpaceKind, x: (f64, f64), y: (f64, f64), z: (f64, f64)) -> Space {
    Space {
        name: name.to_string(),
        kind,
        side: Side::Starboard,
        x_aft: x.0,
        x_fwd: x.1,
        shape: SpaceShape::Prism {
            section: vec![(y.0, z.0), (y.1, z.0), (y.1, z.1), (y.0, z.1)],
        },
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

// ---- MARPOL Annex I regulation 28 (issue #341) ----
// Label: sourced - MEPC.117(52), regulation 28.2 (extents), 28.3.2 and 28.3.3 (criteria), worked by hand here.

use ovd_core::liquid_bulk::damage::{MarpolExtents, marpol_damage_cases};

/// 28.2.1 and 28.2.2 by hand for L = 90 m, B = 14 m: 90^(2/3) = 20.083, a third is 6.694 m (under 14.5 m and
/// over 5 m); B/5 = 2.8 m (under 11.5 m); B/6 = 2.333 m (under 10 m and 5 m); B/15 = 0.9333 m (under 6 m).
#[test]
fn the_marpol_extents_follow_regulation_28_2_by_hand() {
    let e = MarpolExtents::of(90.0, 14.0);
    assert!((e.side_long_m - 6.6943).abs() < 1e-3, "{e:?}");
    assert_eq!(e.side_inboard_m, 2.8);
    assert!((e.bottom_fwd_long_m - 6.6943).abs() < 1e-3);
    assert!((e.bottom_fwd_width_m - 14.0 / 6.0).abs() < 1e-12);
    assert_eq!(e.bottom_aft_long_m, 5.0);
    assert!((e.bottom_aft_width_m - 14.0 / 6.0).abs() < 1e-12);
    assert!((e.bottom_height_m - 14.0 / 15.0).abs() < 1e-12);
    // the caps: a very large hull takes the metre limits (L = 400 m: (1/3) 400^(2/3) = 18.8 m, over 14.5 m)
    let big = MarpolExtents::of(400.0, 120.0);
    assert_eq!(big.side_long_m, 14.5);
    assert_eq!(big.side_inboard_m, 11.5);
    assert_eq!(big.bottom_fwd_width_m, 10.0);
    assert_eq!(big.bottom_aft_width_m, 5.0);
    assert_eq!(big.bottom_height_m, 6.0);
}

/// The MARPOL side damage reaches the cargo tanks: the strip is B/5 = 2.0 m deep for a hull 10 m wide (3.0 to
/// 5.0 m from the centre line), so the cargo tank "IN" (4.0 to 3.9 m, 1.0 m inboard of the side) floods at 95 %
/// with the wing tank, not at 0 as under the ADN extents.
#[test]
fn the_marpol_side_damage_reaches_the_cargo_tanks() {
    let spaces = vec![
        space(
            "OUT",
            SpaceKind::CargoTank,
            (10.0, 50.0),
            (0.0, 3.9),
            (0.0, 4.0),
        ),
        space(
            "IN",
            SpaceKind::CargoTank,
            (10.0, 50.0),
            (3.9, 4.0),
            (0.0, 4.0),
        ),
        space(
            "WB",
            SpaceKind::WingBallast,
            (10.0, 50.0),
            (4.0, 5.0),
            (0.0, 4.0),
        ),
    ];
    let cases = marpol_damage_cases(&spaces, 60.0, 10.0);
    let side: Vec<Vec<String>> = cases
        .iter()
        .filter(|c| c.name.starts_with("side"))
        .map(|c| c.flooded.iter().map(|f| f.name.clone()).collect())
        .collect();
    assert!(!side.is_empty());
    for f in &side {
        assert!(
            f.contains(&"WB".to_string()) && f.contains(&"IN".to_string()),
            "{f:?}"
        );
    }
    assert!(
        cases
            .iter()
            .flat_map(|c| c.flooded.iter())
            .all(|f| (f.permeability - 0.95).abs() < 1e-12)
    );
}

/// 28.1.3: at 150 m or less the damage is between adjacent bulkheads, so it never floods two bays of a
/// wing tank when both bays are longer than the damage; 28.1.1: above 150 m it is anywhere in the length.
#[test]
fn a_hull_of_150_m_or_less_floods_one_bay_and_a_longer_hull_floods_across_a_bulkhead() {
    let make = |loa: f64| {
        vec![
            space(
                "T1",
                SpaceKind::CargoTank,
                (10.0, 60.0),
                (0.0, 4.0),
                (0.7, 4.0),
            ),
            space(
                "T2",
                SpaceKind::CargoTank,
                (60.0, 110.0),
                (0.0, 4.0),
                (0.7, 4.0),
            ),
            space(
                "W",
                SpaceKind::WingBallast,
                (10.0, loa - 10.0),
                (4.0, 5.0),
                (0.0, 4.0),
            ),
        ]
    };
    let two_bays = |spaces: &[ovd_core::compartments::Space], loa: f64| {
        marpol_damage_cases(spaces, loa, 10.0)
            .iter()
            .filter(|c| c.name.starts_with("side"))
            .any(|c| {
                c.flooded.iter().any(|f| f.name == "W/1")
                    && c.flooded.iter().any(|f| f.name == "W/2")
            })
    };
    // 120 m: bays of 50 m, the damage is (1/3) 120^(2/3) = 8.2 m: one bay at a time
    assert!(!two_bays(&make(120.0), 120.0));
    // 160 m: the damage is anywhere in the length, so it can straddle the bulkhead at 60 m
    let spaces = vec![
        space(
            "T1",
            SpaceKind::CargoTank,
            (10.0, 80.0),
            (0.0, 4.0),
            (0.7, 4.0),
        ),
        space(
            "T2",
            SpaceKind::CargoTank,
            (80.0, 150.0),
            (0.0, 4.0),
            (0.7, 4.0),
        ),
        space(
            "W",
            SpaceKind::WingBallast,
            (10.0, 150.0),
            (4.0, 5.0),
            (0.0, 4.0),
        ),
    ];
    assert!(two_bays(&spaces, 160.0));
}

/// 28.2.2: the bottom damage is B/6 wide: a bottom space 2.0 m wide at the centre line is reached, and a
/// double bottom compartment more than B/15 above the base line is not.
#[test]
fn the_marpol_bottom_damage_is_b_over_6_wide_and_b_over_15_high() {
    let spaces = vec![
        space(
            "DB",
            SpaceKind::DoubleBottomBallast,
            (10.0, 50.0),
            (-5.0, 5.0),
            (0.0, 0.5),
        ),
        space(
            "HIGH",
            SpaceKind::PumpRoom,
            (10.0, 50.0),
            (-5.0, 5.0),
            (1.0, 2.0),
        ),
    ];
    // B = 15 m: height 1.0 m (the pump room starts at 1.0 m: not reached), width 2.5 m
    let cases = marpol_damage_cases(&spaces, 60.0, 15.0);
    let bottom: Vec<_> = cases
        .iter()
        .filter(|c| c.name.starts_with("bottom"))
        .collect();
    assert!(!bottom.is_empty());
    assert!(
        bottom
            .iter()
            .all(|c| c.flooded.iter().all(|f| f.name == "DB"))
    );
}

/// 28.3.3: a range of 20 degrees beyond the equilibrium, a lever of 0.1 m and 0.0175 m.rad within it. The
/// closed-form box of the ADN test above has a flooded double bottom: it meets the ADN criteria and also these.
#[test]
fn the_marpol_criteria_need_the_range_the_lever_and_the_area() {
    use ovd_core::damage_stability::DamageCriteria;
    let t = hull();
    let s = start(&t);
    let flood = [flooded((27.5, 32.5), (-5.0, 5.0), (0.0, 1.0), 0.95, "DB")];
    let m = assess(
        &t,
        &flood,
        MASS,
        gravity(),
        1.0,
        s,
        &DamageCriteria::MARPOL_I_28,
    );
    assert_eq!(m.floats, Floats::Yes);
    assert!(m.ok, "{m:?}");
    // the curve is read to the equilibrium plus 20 degrees
    assert!(
        (m.end_deg - (m.heel_deg.abs() + 20.0)).abs() < 1e-6,
        "{m:?}"
    );
    assert!(m.max_gz_m >= 0.1 && m.area_m_rad >= 0.0175);
    // the same hull with a stricter lever fails the verdict and not the float
    let strict = DamageCriteria {
        min_gz_m: 5.0,
        ..DamageCriteria::MARPOL_I_28
    };
    let n = assess(&t, &flood, MASS, gravity(), 1.0, s, &strict);
    assert_eq!(n.floats, Floats::Yes);
    assert!(!n.ok);
}

// ---- Flooded Volumes: prisms, tanks and intact inclusions (spec 0012, D7) ----------------------------
//
// Closed forms worked out here. A Type C tank of outer radius R and overall length Lt with a head of
// length a at each end (half an ellipsoid of semi-axes a, R, R) has the outer volume
// pi R^2 (Lt - 2 a) + (4/3) pi R^2 a. The tank and a box are each symmetric about their centre point, so a
// plane through that point, at any heel and trim, leaves exactly half of each below it.

use ovd_core::sections::Waterplane;
use ovd_core::tanks::{HeadKind, TankHead};

const PI: f64 = core::f64::consts::PI;

/// A hold 20 m long, 8 m wide, from the double bottom (1 m) to the deck (4 m), centred at (30, 0, 2.5).
fn hold(permeability: f64, inclusions: Vec<IntactInclusion>) -> FloodedVolume {
    FloodedVolume {
        inclusions,
        ..FloodedVolume::from_box("H1", (20.0, 40.0), (-4.0, 4.0), (1.0, 4.0), permeability)
    }
}

/// A tank on the hold's centre: axis 21 to 39 m, at y 0 and z 2.5, outer diameter 2 m.
fn tank_shape(kind: HeadKind, head_m: f64) -> SpaceShape {
    SpaceShape::Tank {
        axis_x0: 21.0,
        axis_x1: 39.0,
        y: 0.0,
        z: 2.5,
        outer_diameter: 2.0,
        head: TankHead {
            kind,
            length_m: head_m,
        },
        inner_volume_m3: 0.0,
    }
}

fn tank(kind: HeadKind, head_m: f64) -> IntactInclusion {
    IntactInclusion {
        name: "CT1C".to_string(),
        x_aft_m: 21.0,
        x_fwd_m: 39.0,
        shape: tank_shape(kind, head_m),
    }
}

fn tank_volume(head_m: f64) -> f64 {
    PI * (18.0 - 2.0 * head_m) + 4.0 / 3.0 * PI * head_m
}

/// The waterplane at a heel and trim through the point (x, y, z).
fn plane_through(heel_deg: f64, trim_deg: f64, p: (f64, f64, f64)) -> Waterplane {
    let wp = Waterplane {
        heel_deg,
        trim_deg,
        c: 0.0,
    };
    let (kx, ky, kz) = wp.normal();
    Waterplane {
        c: kx * p.0 + ky * p.1 + kz * p.2,
        ..wp
    }
}

#[test]
fn a_box_is_the_prism_with_four_corners() {
    let b = FloodedVolume::from_box("DB", (27.5, 32.5), (-5.0, 5.0), (0.0, 1.0), 0.95);
    assert_eq!(
        b.shape,
        SpaceShape::Prism {
            section: vec![(-5.0, 0.0), (5.0, 0.0), (5.0, 1.0), (-5.0, 1.0)]
        }
    );
    assert_eq!((b.x_aft_m, b.x_fwd_m, b.permeability), (27.5, 32.5, 0.95));
    assert!(b.inclusions.is_empty());
}

#[test]
fn an_intact_tank_is_subtracted_from_the_flooded_hold_at_any_plane_through_their_centre() {
    for (kind, head_m) in [
        (HeadKind::Hemisphere, 1.0),
        (HeadKind::Ellipsoidal2To1, 0.5),
    ] {
        let full = hold(1.0, vec![]);
        let with_tank = hold(1.0, vec![tank(kind, head_m)]);
        let deep = plane_through(0.0, 0.0, (30.0, 0.0, 1e6));
        // wholly below: the hold less the tank's outer volume
        assert!((full.water_m3(&deep) - 480.0).abs() < 1e-9);
        let want = 480.0 - tank_volume(head_m);
        let got = with_tank.water_m3(&deep);
        assert!((got - want).abs() < 1e-9, "{kind:?}: {got} against {want}");
        // any plane through the common centre: half of each
        for (heel, trim) in [
            (0.0, 0.0),
            (10.0, 0.0),
            (0.0, 2.0),
            (25.0, -3.0),
            (60.0, 5.0),
        ] {
            let wp = plane_through(heel, trim, (30.0, 0.0, 2.5));
            let got = with_tank.water_m3(&wp);
            let want = 0.5 * (480.0 - tank_volume(head_m));
            assert!(
                (got - want).abs() < 1e-6,
                "{kind:?} heel {heel} trim {trim}: {got} against {want}"
            );
        }
        // the permeability scales the water, the tank stays out of it
        let wp = plane_through(10.0, 1.0, (30.0, 0.0, 2.5));
        let p = FloodedVolume {
            permeability: 0.95,
            ..with_tank.clone()
        };
        assert!((p.water_m3(&wp) - 0.95 * with_tank.water_m3(&wp)).abs() < 1e-9);
    }
}

/// A plane at distance h from the tank axis, parallel to it (any heel, no trim; the tank is a body of
/// revolution about its axis, so the heel does not matter). The cylinder keeps the circular segment
/// R^2 acos(-h / R) + h sqrt(R^2 - h^2) along its length; the two heads together are an ellipsoid of semi-axes
/// a, R, R, which is a sphere of radius R scaled by a / R along x, so they keep (a / R) pi (R + h)^2 (2 R - h) / 3.
#[test]
fn a_flooded_tank_cut_parallel_to_its_axis_keeps_the_closed_form_segment() {
    let r: f64 = 1.0;
    for (kind, head_m) in [
        (HeadKind::Hemisphere, 1.0),
        (HeadKind::Ellipsoidal2To1, 0.5),
    ] {
        let breached = FloodedVolume {
            name: "CT1C".to_string(),
            x_aft_m: 21.0,
            x_fwd_m: 39.0,
            shape: tank_shape(kind, head_m),
            permeability: 1.0,
            inclusions: vec![],
        };
        for heel in [0.0_f64, 30.0, -70.0] {
            for h in [0.4_f64, -0.7, 0.95] {
                let (sp, cp) = (sin(heel.to_radians()), cos(heel.to_radians()));
                // the point at distance h from the axis along the earth vertical (-sin phi, cos phi)
                let wp = plane_through(heel, 0.0, (30.0, -sp * h, 2.5 + cp * h));
                let segment = r * r * acos(-h / r) + h * sqrt(r * r - h * h);
                let heads = head_m / r * PI * (r + h) * (r + h) * (2.0 * r - h) / 3.0;
                let want = (18.0 - 2.0 * head_m) * segment + heads;
                let got = breached.water_m3(&wp);
                assert!(
                    (got - want).abs() < 1e-6,
                    "{kind:?} heel {heel} h {h}: {got} against {want}"
                );
            }
        }
    }
}

#[test]
fn a_flooded_tank_takes_its_outer_volume_of_water_when_wholly_below() {
    let breached = FloodedVolume {
        name: "CT1C".to_string(),
        x_aft_m: 21.0,
        x_fwd_m: 39.0,
        shape: tank_shape(HeadKind::Hemisphere, 1.0),
        permeability: 0.95,
        inclusions: vec![],
    };
    let deep = plane_through(0.0, 0.0, (30.0, 0.0, 1e6));
    assert!((breached.water_m3(&deep) - 0.95 * tank_volume(1.0)).abs() < 1e-9);
    let dry = plane_through(0.0, 0.0, (30.0, 0.0, 1.0));
    assert_eq!(breached.water_m3(&dry), 0.0);
}

/// The box hull at 1,800 t with the hold flooded (permeability 1) and an intact tank in it, the tank wholly
/// below the waterline. Wall-sided, upright, level: 600 T - 160 (T - 1) + V_tank = 1800, so
/// T = (1800 - 160 - V_tank) / 440 = 3.6035 m, above the tank top (3.5 m). Without the tank T = 1640 / 440.
#[test]
fn the_damaged_hull_floats_on_the_intact_tank_in_its_flooded_hold() {
    let t = hull();
    let mass = 1800.0;
    let g = Gravity {
        lcg_m: L / 2.0,
        tcg_m: 0.0,
        kg_m: KG,
    };
    let eq = equilibrium(&t, mass, g, 0.0, 1.0).unwrap();
    let s = (eq.waterplane.c, eq.waterplane.trim_deg);
    let crit = DamageCriteria::ADN_2025;
    let with_tank = assess(
        &t,
        &[hold(1.0, vec![tank(HeadKind::Hemisphere, 1.0)])],
        mass,
        g,
        1.0,
        s,
        &crit,
    );
    assert_eq!(with_tank.floats, Floats::Yes);
    assert!(with_tank.heel_deg.abs() < 1e-6 && with_tank.trim_deg.abs() < 1e-6);
    let want = (mass - 160.0 - tank_volume(1.0)) / 440.0;
    assert!(want > 3.5);
    assert!(
        (with_tank.draught_mid_m - want).abs() < 1e-6,
        "{} {want}",
        with_tank.draught_mid_m
    );
    let without = assess(&t, &[hold(1.0, vec![])], mass, g, 1.0, s, &crit);
    assert!((without.draught_mid_m - (mass - 160.0) / 440.0).abs() < 1e-6);
}
