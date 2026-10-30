//! Seam: the tank, dome and weather shield meshes (spec 0009, "Meshes and the parity hash", issue
//! #686).
//!
//! Label: property. A tank mesh is closed, faces out, spans its axis and its outer diameter exactly,
//! and holds the closed-form volume at the outer diameter (the cylinder plus two half-ellipsoid
//! heads, worked in the test) to 0.5 %. A dome and the weather shield are closed and face out.

use core::f64::consts::PI;
use std::collections::HashMap;

use ovcd_core::mesh::{Mesh, TANK_SEGMENTS, dome_mesh, tank_mesh, weather_shield_mesh};
use ovcd_core::tanks::{DomePlace, HeadKind, TankHead};
use ovcd_core::vessel_model::WeatherShield;

fn closed(m: &Mesh) -> Result<(), String> {
    let mut count: HashMap<(u32, u32), i32> = HashMap::new();
    for t in m.indices.as_chunks::<3>().0 {
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            *count.entry((a, b)).or_default() += 1;
        }
    }
    for (&(a, b), &n) in &count {
        if n != 1 || count.get(&(b, a)) != Some(&1) {
            return Err(format!("edge {:?} {:?}", m.vertex(a), m.vertex(b)));
        }
    }
    Ok(())
}

fn bbox(m: &Mesh) -> ([f64; 3], [f64; 3]) {
    let mut lo = [f64::INFINITY; 3];
    let mut hi = [f64::NEG_INFINITY; 3];
    for &i in &m.indices {
        let v = m.vertex(i);
        for k in 0..3 {
            lo[k] = lo[k].min(v[k]);
            hi[k] = hi[k].max(v[k]);
        }
    }
    (lo, hi)
}

#[test]
fn a_tank_mesh_is_closed_spans_its_axis_and_diameter_and_holds_its_volume() {
    let mut worst: f64 = 0.0;
    for (kind, head_over_r) in [
        (HeadKind::Hemisphere, 1.0),
        (HeadKind::Ellipsoidal2To1, 0.5),
    ] {
        for (od, length) in [(4.0, 12.0), (7.4, 26.0), (8.4, 24.8), (12.0, 84.0)] {
            let r = od / 2.0;
            let h = head_over_r * r;
            let (x0, yc, zc) = (10.0, -1.5, 5.2);
            let m = tank_mesh(
                (x0, x0 + length),
                (yc, zc),
                od,
                TankHead { kind, length_m: h },
            );
            let at = format!("{kind:?} OD {od} L {length}");
            closed(&m).unwrap_or_else(|e| panic!("{at}: not closed at {e}"));
            let (lo, hi) = bbox(&m);
            for (got, want) in [
                (lo[0], x0),
                (hi[0], x0 + length),
                (lo[1], yc - r),
                (hi[1], yc + r),
                (lo[2], zc - r),
                (hi[2], zc + r),
            ] {
                // f32 positions.
                assert!(
                    (got - want).abs() <= 1e-5 * want.abs().max(1.0),
                    "{at}: {got} vs {want}"
                );
            }
            let closed_form = PI * r * r * (length - 2.0 * h) + 2.0 * (2.0 / 3.0) * PI * r * r * h;
            let v = m.volume();
            assert!(v > 0.0, "{at}: faces out");
            let error = (v - closed_form).abs() / closed_form;
            worst = worst.max(error);
            assert!(error <= 0.005, "{at}: {v} vs {closed_form}");
        }
    }
    // 48 segments lose (1 - sin(2 pi / 48) 48 / (2 pi)) of the section: about 0.29 %.
    assert!(worst > 0.002, "the bound is not slack: {worst}");
    assert_eq!(TANK_SEGMENTS, 48);
}

#[test]
fn a_dome_is_a_closed_cylinder_from_the_shell_to_its_top() {
    let (r, axis_z) = (3.7, 5.0);
    let dome = DomePlace {
        x_m: 20.0,
        y_m: 0.0,
        z_base_m: axis_z + r,
        diameter_m: 1.26,
        height_m: 1.26,
    };
    let m = dome_mesh(&dome, r, axis_z);
    closed(&m).unwrap();
    assert!(m.volume() > 0.0);
    let (lo, hi) = bbox(&m);
    let foot = axis_z + (r * r - 0.63 * 0.63_f64).sqrt();
    assert!((lo[2] - foot).abs() < 1e-5 && (hi[2] - (axis_z + r + 1.26)).abs() < 1e-5);
    assert!((lo[0] - 19.37).abs() < 1e-5 && (hi[0] - 20.63).abs() < 1e-5);
}

#[test]
fn the_weather_shield_is_five_closed_plates_that_fill_its_box_shell() {
    let w = WeatherShield {
        x_aft_m: 8.0,
        x_fwd_m: 70.0,
        half_width_m: 5.7,
        z_foot_m: 4.5,
        z_top_m: 9.4,
        thickness_m: 0.006,
    };
    let m = weather_shield_mesh(&w);
    closed(&m).unwrap();
    let (lo, hi) = bbox(&m);
    for (got, want) in [
        (lo[0], 8.0),
        (hi[0], 70.0),
        (lo[1], -5.7),
        (hi[1], 5.7),
        (lo[2], 4.5),
        (hi[2], 9.4),
    ] {
        assert!((got - want).abs() < 1e-5, "{got} vs {want}");
    }
    // The volume of the plates: the shell of the box, less the open bottom.
    let (l, b, h, t) = (62.0, 11.4, 4.9, 0.006);
    let plates = l * b * t + 2.0 * l * t * (h - t) + 2.0 * t * (b - 2.0 * t) * (h - t);
    assert!(
        (m.volume() - plates).abs() <= 1e-3 * plates,
        "{} vs {plates}",
        m.volume()
    );
}
