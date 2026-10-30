//! Seam: the Tank Layout (`tanks::tank_layout`, spec 0009, ADR 0021). One function places the Type C
//! tanks for the capacity, the loading conditions, the stability, the weights and the GA drawer, and
//! a tank Space passes through `clip` unchanged.
//!
//! Label: property. Each test states a relation between the layout and what the engine already
//! uses (the cargo LCG, the tank KCG, the zone), over a sweep of hulls, cargoes, heads and tank
//! counts. No fixed number.

use ovcd_core::barge_capacity::{BargeCapacityInputs, BargeCapacityResult, compute};
use ovcd_core::compartments::{ClippedSpace, SpaceKind, SpaceShape, clip, tank_spaces};
use ovcd_core::gas_stability::GasCapacityResult;
use ovcd_core::hull::hull_form::{HullFormInputs, HullFormKind, HullShape, generate};
use ovcd_core::loading_conditions::gas_model;
use ovcd_core::tanks::{HeadKind, TankLayout, arrangement_layout};

/// Every gas result of the sweep, with its Tank Layout.
fn sweep() -> Vec<(String, BargeCapacityResult, TankLayout)> {
    let mut out = Vec::new();
    for (loa, beam, depth) in [
        (50.0, 10.0, 4.0),
        (70.0, 10.0, 5.0),
        (90.0, 14.8, 6.5),
        (110.0, 11.4, 6.0),
        (140.0, 29.0, 9.0),
        (180.0, 24.0, 10.0),
    ] {
        for cargo in ["LPG_propane", "LNG"] {
            for head in ["hemisphere", "ellipsoidal_2to1"] {
                for n_long in [1, 2, 3] {
                    for n_wide in [None, Some(2)] {
                        let mut i = BargeCapacityInputs::new(loa, beam, depth);
                        i.cargo_type = cargo.to_string();
                        i.tank_design.head_type = head.to_string();
                        i.n_tanks_long = n_long;
                        i.n_tanks_wide = n_wide;
                        let Ok(r) = compute(&i) else { continue };
                        let Some(g) = r.gas_basis.as_ref() else {
                            continue;
                        };
                        let layout = arrangement_layout(&g.inputs, &g.tanks, Some((2.0, 2.0)))
                            .expect("a head the engine models");
                        let case = format!(
                            "{loa} x {beam} x {depth} {cargo} {head} {n_long} long {n_wide:?} wide"
                        );
                        out.push((case, r, layout));
                    }
                }
            }
        }
    }
    assert!(out.len() > 100, "the sweep must reach many designs");
    out
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0)
}

#[test]
fn the_mean_tank_axis_x_is_the_cargo_lcg_and_the_axis_z_is_the_tank_kcg() {
    for (case, r, layout) in sweep() {
        let (items, cargo) =
            gas_model(&GasCapacityResult::try_from(&r).unwrap()).expect("a gas model");
        let n = layout.tanks.len() as f64;
        let mean_x = layout.tanks.iter().map(|t| t.axis.centre_m).sum::<f64>() / n;
        let mean_mid = layout
            .tanks
            .iter()
            .map(|t| (t.axis.x0_m + t.axis.x1_m) / 2.0)
            .sum::<f64>()
            / n;
        assert!(
            close(mean_x, cargo.lcg_m),
            "{case}: {mean_x} vs {}",
            cargo.lcg_m
        );
        assert!(close(mean_mid, cargo.lcg_m), "{case}: {mean_mid}");
        // The tanks' mass item (`gas_model`'s second item) stands at the axis height, exactly.
        for t in &layout.tanks {
            assert_eq!(t.z_m, items[1].kcg_m, "{case}: {}", t.name);
        }
        assert_eq!(items[1].lcg_m, cargo.lcg_m, "{case}");
        // Both read the layout, so check them against the inputs too: the middle of the zone
        // between the voids, and the bottom clearance plus half the diameter.
        let g = r.gas_basis.as_ref().unwrap();
        let (bow, stern) = (
            g.inputs.bow_non_tank_length.unwrap(),
            g.inputs.stern_non_tank_length.unwrap(),
        );
        assert!(
            close(cargo.lcg_m, stern + (g.inputs.loa - bow - stern) / 2.0),
            "{case}"
        );
        let z = g.inputs.double_bottom_height.unwrap()
            + g.inputs.tank_support_height
            + g.tanks.outer_diameter / 2.0;
        assert!(close(items[1].kcg_m, z), "{case}");
    }
}

#[test]
fn the_tanks_fill_the_zone_and_do_not_overlap() {
    for (case, r, layout) in sweep() {
        let g = r.gas_basis.as_ref().unwrap();
        assert_eq!(
            layout.tanks.len() as i64,
            g.tanks.n_tanks_total,
            "{case}: one place per tank"
        );
        let (z, od) = (&layout.zone, layout.outer_diameter_m);
        let tol = 1e-9 * z.x_fwd_m.max(1.0);
        for t in &layout.tanks {
            assert!(t.axis.x0_m >= z.x_aft_m - tol, "{case}: {}", t.name);
            assert!(t.axis.x1_m <= z.x_fwd_m + tol, "{case}: {}", t.name);
            assert!(close(
                t.axis.x1_m - t.axis.x0_m,
                g.tanks.overall_tank_length
            ));
            // Inside the beam less the side clearance.
            let room = g.inputs.beam / 2.0 - g.inputs.side_clearance.unwrap_or(0.0);
            assert!(t.y_m.abs() + od / 2.0 <= room + 1e-9, "{case}: {}", t.name);
        }
        // The first and last tanks reach the ends of the zone.
        let (first, last) = (layout.tanks.first().unwrap(), layout.tanks.last().unwrap());
        assert!(close(first.axis.x0_m, z.x_aft_m), "{case}");
        assert!(close(last.axis.x1_m, z.x_fwd_m), "{case}");
        for (i, a) in layout.tanks.iter().enumerate() {
            for b in &layout.tanks[i + 1..] {
                let apart_x = a.axis.x1_m <= b.axis.x0_m + tol || b.axis.x1_m <= a.axis.x0_m + tol;
                let apart_y = (a.y_m - b.y_m).abs() >= od - 1e-9;
                assert!(
                    apart_x || apart_y,
                    "{case}: {} and {} overlap",
                    a.name,
                    b.name
                );
            }
        }
        let names: std::collections::HashSet<_> = layout.tanks.iter().map(|t| &t.name).collect();
        assert_eq!(names.len(), layout.tanks.len(), "{case}: unique names");
    }
}

#[test]
fn each_dome_stands_on_the_top_of_its_tank_over_its_middle() {
    for (case, _, layout) in sweep() {
        assert_eq!(layout.domes.len(), layout.tanks.len(), "{case}");
        for (d, t) in layout.domes.iter().zip(&layout.tanks) {
            assert!(close(d.x_m, (t.axis.x0_m + t.axis.x1_m) / 2.0), "{case}");
            assert_eq!(d.y_m, t.y_m);
            assert!(
                close(d.z_base_m, t.z_m + layout.outer_diameter_m / 2.0),
                "{case}"
            );
        }
    }
}

#[test]
fn no_dome_size_gives_no_domes() {
    let (_, r, _) = sweep().remove(0);
    let g = r.gas_basis.as_ref().unwrap();
    for dome in [None, Some((0.0, 2.0)), Some((2.0, 0.0))] {
        let layout = arrangement_layout(&g.inputs, &g.tanks, dome).unwrap();
        assert!(layout.domes.is_empty(), "{dome:?}");
    }
}

#[test]
fn the_head_is_the_arrangements() {
    for (case, r, layout) in sweep() {
        let g = r.gas_basis.as_ref().unwrap();
        assert_eq!(layout.head.kind.key(), g.inputs.tank_head_type, "{case}");
        assert_eq!(layout.head.length_m, g.tanks.head_length, "{case}");
    }
    assert_eq!(HeadKind::from_key("torispherical"), None);
}

#[test]
fn clip_returns_a_tank_space_unchanged() {
    // A raked bow, so the hull cuts the prisms near the bow: a tank stays whole all the same.
    let hull = generate(&HullFormInputs {
        length_overall_m: 90.0,
        beam_m: 14.8,
        moulded_depth_m: 6.5,
        form: HullFormKind::RakedBox {
            bow_rake_length_m: 6.4,
            bow_rake_rise_m: 6.5,
            stern_rake_length_m: 0.0,
            stern_rake_rise_m: 0.0,
        },
        shape: HullShape::default(),
    })
    .unwrap();
    let mut i = BargeCapacityInputs::new(90.0, 14.8, 6.5);
    i.cargo_type = "LPG_propane".to_string();
    let r = compute(&i).unwrap();
    let g = r.gas_basis.as_ref().unwrap();
    let layout = arrangement_layout(&g.inputs, &g.tanks, None).unwrap();
    let spaces = tank_spaces(&layout, g.tanks.volume_per_tank);
    assert_eq!(spaces.len(), layout.tanks.len());
    for s in &spaces {
        let c = clip(s, &hull);
        assert_eq!(c, ClippedSpace::prism(s.clone()));
        assert!(!c.is_cut());
        assert_eq!(c.space, *s);
        assert_eq!(c.volume_m3, g.tanks.volume_per_tank);
        assert_eq!(s.kind, SpaceKind::CargoTank);
        let SpaceShape::Tank {
            axis_x0,
            axis_x1,
            y,
            z,
            ..
        } = s.shape
        else {
            panic!("a tank shape");
        };
        assert_eq!(c.centroid_m, ((axis_x0 + axis_x1) / 2.0, y, z));
    }
}
