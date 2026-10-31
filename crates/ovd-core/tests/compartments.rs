//! Seam: `ovd_core::compartments`, the typed Spaces (issue #426).
//! Label: sourced - hand calculation: the volume, centroid and free-surface inertia of a
//! rectangular and a trapezoidal prism (I = l b³ / 12 for a rectangular free surface).

use ovd_core::barge_capacity::{BargeCapacityInputs, compute};
use ovd_core::compartments::{Side, Space, SpaceKind, SpaceShape, ballast_tanks, gas_hold_bounds};

fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-9, "{a} != {b}");
}

/// A box tank 10 m long, 4 m wide (y 1 to 5) and 2 m high (z 0 to 2), starboard.
fn box_tank() -> Space {
    Space {
        name: "DB1S".into(),
        kind: SpaceKind::DoubleBottomBallast,
        side: Side::Starboard,
        x_aft: 5.0,
        x_fwd: 15.0,
        shape: SpaceShape::Prism {
            section: vec![(1.0, 0.0), (5.0, 0.0), (5.0, 2.0), (1.0, 2.0)],
        },
    }
}

#[test]
fn a_box_tank_has_its_volume_and_capacity() {
    let t = box_tank();
    close(t.volume_m3(), 80.0);
    close(t.capacity_t(1.025), 82.0);
}

#[test]
fn a_half_full_box_tank_has_the_hand_worked_level_centroid_and_free_surface() {
    let f = box_tank().fill(0.5);
    close(f.volume_m3, 40.0);
    assert!((f.level_m - 1.0).abs() < 1e-9);
    close(f.lcg_m, 10.0);
    close(f.tcg_m, 3.0);
    assert!((f.vcg_m - 0.5).abs() < 1e-9);
    // l b³ / 12 = 10 × 4³ / 12.
    assert!((f.free_surface_inertia_m4 - 10.0 * 64.0 / 12.0).abs() < 1e-6);
}

#[test]
fn an_empty_or_pressed_full_tank_has_no_free_surface() {
    let t = box_tank();
    assert_eq!(t.fill(0.0).free_surface_inertia_m4, 0.0);
    assert_eq!(t.fill(0.0).volume_m3, 0.0);
    let full = t.fill(1.0);
    assert_eq!(full.free_surface_inertia_m4, 0.0);
    close(full.volume_m3, 80.0);
    close(full.vcg_m, 1.0);
}

#[test]
fn a_trapezoid_section_has_the_hand_worked_area_and_centroid() {
    // Corners (1,0) (3,0) (3,2) (2,2): area (2 + 1) / 2 × 2 = 3; z̄ = h(a + 2b) / (3(a + b)) with
    // the bottom a = 2 and the top b = 1: 2 × 4 / 9.
    let t = Space {
        name: "WB1S".into(),
        kind: SpaceKind::WingBallast,
        side: Side::Starboard,
        x_aft: 0.0,
        x_fwd: 1.0,
        shape: SpaceShape::Prism {
            section: vec![(1.0, 0.0), (3.0, 0.0), (3.0, 2.0), (2.0, 2.0)],
        },
    };
    close(t.volume_m3(), 3.0);
    close(t.fill(1.0).vcg_m, 8.0 / 9.0);
}

#[test]
fn a_hopper_wing_tank_at_half_height_has_the_chord_of_its_free_surface() {
    // The hold side runs from y 1 at z 0 to y 2 at z 2; the shell is at y 3. At z 1 the free
    // surface runs from y 1.5 to 3: b = 1.5, I = 4 × 1.5³ / 12.
    let t = Space {
        name: "WB1S".into(),
        kind: SpaceKind::WingBallast,
        side: Side::Starboard,
        x_aft: 0.0,
        x_fwd: 4.0,
        shape: SpaceShape::Prism {
            section: vec![(1.0, 0.0), (3.0, 0.0), (3.0, 2.0), (2.0, 2.0)],
        },
    };
    // The wetted area below z 1 is (2 + 1.5) / 2 × 1 = 1.75, of 3.
    let f = t.fill(1.75 / 3.0);
    assert!((f.level_m - 1.0).abs() < 1e-9);
    assert!((f.free_surface_inertia_m4 - 4.0 * 3.375 / 12.0).abs() < 1e-6);
}

#[test]
fn a_dry_hopper_barge_has_a_double_bottom_and_a_wing_tank_each_side() {
    let mut i = BargeCapacityInputs::new(80.0, 11.4, 4.0);
    i.cargo_type = "bauxite".to_string();
    i.hold_type = "sloped_hopper".to_string();
    let r = compute(&i).unwrap();
    let tanks = ballast_tanks(&i, &r);
    let names: Vec<&str> = tanks.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["DB1P", "DB1S", "WB1P", "WB1S", "FP1C"]);
    let l = r.cargo_zone_length_m.unwrap();
    // Double bottom: half beam × its height × the cargo zone, each side.
    close(tanks[0].volume_m3(), 5.7 * i.double_bottom_height_bulk * l);
    // Port and starboard mirror each other.
    close(tanks[2].volume_m3(), tanks[3].volume_m3());
    close(tanks[2].fill(0.5).tcg_m, -tanks[3].fill(0.5).tcg_m);
    // The wing tank runs from the inner bottom to the deck, outboard of the hold side.
    let (z_lo, z_hi) = tanks[3].z_range();
    close(z_lo, i.double_bottom_height_bulk);
    close(z_hi, 4.0);
}

#[test]
fn a_default_gas_barge_has_no_wing_tank_because_the_clearance_is_below_the_wing_gap() {
    // IGC_2G gives a side clearance of about 0.55 m; the wing tank is the clearance less the wing
    // gap, so it has no width (issue #283). The double bottom, one pair per hold (spec 0012 D6:
    // two tanks in line), and the fore peak remain.
    let mut i = BargeCapacityInputs::new(90.0, 14.8, 6.5);
    i.cargo_type = "LPG_propane".to_string();
    let r = compute(&i).unwrap();
    let names: Vec<String> = ballast_tanks(&i, &r).into_iter().map(|t| t.name).collect();
    assert_eq!(names, ["DB1P", "DB1S", "DB2P", "DB2S", "FP1C"]);
}

#[test]
fn a_gas_barge_with_a_wing_tank_has_it_up_to_the_wing_plate() {
    let mut i = BargeCapacityInputs::new(90.0, 14.8, 6.5);
    i.cargo_type = "LPG_propane".to_string();
    i.side_clearance_override_m = Some(2.0);
    let r = compute(&i).unwrap();
    let tanks = ballast_tanks(&i, &r);
    let g = r.gas_basis.as_ref().unwrap();
    let db = g.inputs.double_bottom_height.unwrap();
    let top = db
        + g.inputs.tank_support_height
        + g.inputs.wing_tank_height_ratio * g.tanks.outer_diameter;
    let wb = tanks.iter().find(|t| t.name == "WB1S").unwrap();
    let (z_lo, z_hi) = wb.z_range();
    close(z_lo, db);
    close(z_hi, top);
    // The 2 m clearance less the gap.
    let width = 2.0 - i.wing_tank_gap_m;
    close(
        wb.section()
            .iter()
            .map(|p| p.0)
            .fold(f64::INFINITY, f64::min),
        7.4 - width,
    );
    assert!(
        tanks
            .iter()
            .all(|t| t.kind.is_ballast() && t.volume_m3() > 0.0)
    );
}

#[test]
fn the_fore_peak_is_the_whole_bow_void_before_the_hull_clips_it() {
    let mut i = BargeCapacityInputs::new(80.0, 11.4, 4.0);
    i.cargo_type = "bauxite".to_string();
    let r = compute(&i).unwrap();
    let fp = ballast_tanks(&i, &r)
        .into_iter()
        .find(|t| t.name == "FP1C")
        .unwrap();
    let bow = r.bow_void_length_m.unwrap();
    close(fp.volume_m3(), 11.4 * 4.0 * bow);
    close(fp.x_aft, 80.0 - bow);
    close(fp.x_fwd, 80.0);
}

#[test]
fn only_a_pushed_barge_has_an_aft_peak() {
    let mut i = BargeCapacityInputs::new(80.0, 11.4, 4.0);
    i.cargo_type = "bauxite".to_string();
    i.self_propelled = false;
    let r = compute(&i).unwrap();
    let ap = ballast_tanks(&i, &r)
        .into_iter()
        .find(|t| t.name == "AP1C")
        .unwrap();
    close(ap.x_aft, 0.0);
    close(ap.x_fwd, r.stern_void_length_m.unwrap());
    close(ap.fill(0.5).tcg_m, 0.0);
}

/// Spec 0012 D6: one hold per tank in line, a watertight bulkhead midway between the heads of two
/// tanks in line, and the double bottom and the wing tanks cut at the same bulkheads.
fn gas_with_wings() -> (
    BargeCapacityInputs,
    ovd_core::barge_capacity::BargeCapacityResult,
) {
    let mut i = BargeCapacityInputs::new(90.0, 14.8, 6.5);
    i.cargo_type = "LPG_propane".to_string();
    i.side_clearance_override_m = Some(2.0);
    let r = compute(&i).unwrap();
    (i, r)
}

#[test]
fn a_gas_hold_bulkhead_stands_midway_between_the_heads_of_two_tanks_in_line() {
    let (i, r) = gas_with_wings();
    let g = r.gas_basis.as_ref().unwrap();
    let layout = ovd_core::tanks::arrangement_layout(&g.inputs, &g.tanks, None).unwrap();
    let rows: Vec<_> = layout.tanks.iter().filter(|t| t.column == 0).collect();
    assert!(rows.len() >= 2, "a test of subdivision needs tanks in line");
    let holds = gas_hold_bounds(&i, &r);
    assert_eq!(holds.len(), rows.len());
    // the holds run end to end over the cargo zone
    close(holds[0].0, r.stern_void_length_m.unwrap_or(0.0));
    close(
        holds[holds.len() - 1].1,
        i.loa - r.bow_void_length_m.unwrap_or(0.0),
    );
    for k in 1..holds.len() {
        assert_eq!(holds[k].0, holds[k - 1].1);
        close(
            holds[k].0,
            0.5 * (rows[k - 1].axis.x1_m + rows[k].axis.x0_m),
        );
    }
    // each tank lies in its own hold
    for (h, t) in holds.iter().zip(&rows) {
        assert!(
            h.0 <= t.axis.x0_m && t.axis.x1_m <= h.1,
            "{h:?} {:?}",
            t.axis
        );
    }
}

#[test]
fn a_gas_double_bottom_and_wing_tank_are_cut_at_the_hold_bulkheads() {
    let (i, r) = gas_with_wings();
    let holds = gas_hold_bounds(&i, &r);
    let n = holds.len();
    let tanks = ballast_tanks(&i, &r);
    let mut want: Vec<String> = Vec::new();
    for prefix in ["DB", "WB"] {
        for k in 1..=n {
            want.push(format!("{prefix}{k}P"));
            want.push(format!("{prefix}{k}S"));
        }
    }
    want.push("FP1C".to_string());
    let names: Vec<String> = tanks.iter().map(|t| t.name.clone()).collect();
    assert_eq!(names, want);
    for (k, h) in holds.iter().enumerate() {
        for prefix in ["DB", "WB"] {
            for side in ["P", "S"] {
                let t = tanks
                    .iter()
                    .find(|t| t.name == format!("{prefix}{}{side}", k + 1))
                    .unwrap();
                assert_eq!((t.x_aft, t.x_fwd), *h);
            }
        }
    }
    // the cut keeps the total: half beam x double bottom x cargo zone, each side
    let g = r.gas_basis.as_ref().unwrap();
    let db = g.inputs.double_bottom_height.unwrap();
    let l = holds[n - 1].1 - holds[0].0;
    let total: f64 = tanks
        .iter()
        .filter(|t| t.kind == SpaceKind::DoubleBottomBallast)
        .map(|t| t.volume_m3())
        .sum();
    assert!((total - 2.0 * 7.4 * db * l).abs() < 1e-9 * total);
}
