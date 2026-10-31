//! Seam: `ovd_core::vessel_model::assemble` (issue #454, spec 0008 "Phase 2: the Vessel Model").
//!
//! Label: property - the model holds the hull it is given, the holds of the volume model split
//! evenly with a bulkhead between each two, the ballast tanks of `compartments::ballast_tanks`
//! unchanged, an inner bottom at the double-bottom height and no frames. Label: sourced - closed
//! form, the hold prisms sum to the volume model's hold volume (section area times cargo zone
//! length, `containment_inner_volume_m3`), and the hull clips them at the deck: the trapezoid from
//! the inner bottom to the deck times the cargo zone length (issue #455).

use ovd_core::barge_capacity::{BargeCapacityInputs, compute};
use ovd_core::compartments::{SpaceKind, ballast_tanks, gas_hold_bounds};
use ovd_core::hydrostatics::Hull;
use ovd_core::vessel_model::{VesselModelInputs, assemble};

fn dry_bulk(n_holds: i64) -> VesselModelInputs {
    let mut i = BargeCapacityInputs::new(80.0, 11.4, 4.0);
    i.cargo_type = "bauxite".to_string();
    i.hold_type = "sloped_hopper".to_string();
    i.n_holds = n_holds;
    let r = compute(&i).unwrap();
    // The hull the result was worked on: the raked bow of the inputs.
    let hull = Hull::build(r.hull_spec.as_ref().unwrap()).unwrap().table;
    VesselModelInputs {
        capacity: i,
        result: r,
        hull,
    }
}

#[test]
fn a_dry_bulk_barge_has_its_holds_bulkheads_ballast_tanks_and_decks() {
    for n in [1, 3, 4] {
        let inp = dry_bulk(n);
        let m = assemble(&inp);
        let r = &inp.result;
        assert_eq!(m.hull, inp.hull);
        assert!(m.frames.is_empty());
        let holds: Vec<_> = m
            .spaces
            .iter()
            .filter(|s| s.space.kind == SpaceKind::CargoHold)
            .collect();
        assert_eq!(holds.len(), n as usize);
        // The prisms are the volume model's hold, to the coaming top.
        let prism: f64 = holds.iter().map(|s| s.space.volume_m3()).sum();
        let want = r.containment_inner_volume_m3.unwrap();
        assert!((prism - want).abs() <= 1e-12 * want, "{prism} vs {want}");
        // The hull (closed at the deck) clips the coaming part off: the trapezoid to the deck.
        let (db, d) = (inp.capacity.double_bottom_height_bulk, 4.0);
        let (wt, wb, h) = (
            r.hold_top_width_m.unwrap(),
            r.hold_bottom_width_m.unwrap(),
            r.hold_height_m.unwrap(),
        );
        let w_deck = wb + (wt - wb) * (d - db) / h;
        let below = 0.5 * (wb + w_deck) * (d - db) * r.cargo_zone_length_m.unwrap();
        let v: f64 = holds.iter().map(|s| s.volume_m3).sum();
        assert!((v - below).abs() <= 1e-12 * below, "{v} vs {below}");
        // One bulkhead at each end of the cargo zone and one between each two holds.
        let (x0, x1) = (holds[0].space.x_aft, holds[holds.len() - 1].space.x_fwd);
        for h in &holds[1..] {
            assert!(
                m.bulkheads
                    .iter()
                    .any(|b| b.x_m == h.space.x_aft && b.watertight)
            );
        }
        assert!(m.bulkheads.iter().any(|b| b.x_m == x0));
        assert!(m.bulkheads.iter().any(|b| b.x_m == x1));
        // The zones of the aft block have their own.
        let cargo = m.bulkheads.iter().filter(|b| !b.name.starts_with("zone_"));
        assert_eq!(cargo.count(), n as usize + 1, "{:?}", m.bulkheads);
        // The ballast tanks as the loading conditions use them.
        let ballast: Vec<_> = m
            .spaces
            .iter()
            .filter(|s| s.space.kind.is_ballast())
            .map(|s| s.space.clone())
            .collect();
        assert_eq!(ballast, ballast_tanks(&inp.capacity, r));
        let decks: Vec<(&str, f64)> = m.decks.iter().map(|d| (d.name.as_str(), d.z_m)).collect();
        assert_eq!(
            decks,
            [
                ("main_deck", 4.0),
                ("inner_bottom", inp.capacity.double_bottom_height_bulk)
            ]
        );
        // A Space the hull does not cut keeps its prism centroid, at mid-length; the fore peak in
        // the bow rake is cut.
        for s in m.spaces.iter().filter(|s| !s.is_cut()) {
            assert_eq!(s.centroid_m.0, (s.space.x_aft + s.space.x_fwd) / 2.0);
        }
        assert!(
            m.spaces
                .iter()
                .any(|s| s.space.name == "FP1C" && s.is_cut())
        );
    }
}

/// Spec 0012 D6 on a gas barge with wing tanks: one hold per tank in line between watertight
/// bulkheads, each Type C tank in its hold, and above each wing tank, outboard of the hold, a void
/// strip to the deck. The holds, wing tanks and strips fill the cargo zone above the double bottom:
/// B x (D - db) x the cargo zone length.
#[test]
fn a_gas_barge_has_one_hold_per_tank_in_line_and_a_void_strip_above_each_wing_tank() {
    let mut i = BargeCapacityInputs::new(90.0, 14.8, 6.5);
    i.cargo_type = "LPG_propane".to_string();
    i.side_clearance_override_m = Some(2.0);
    // a wing tank that stops below the deck
    i.wing_tank_height_ratio = 0.4;
    let r = compute(&i).unwrap();
    let g = r.gas_basis.as_ref().unwrap();
    let top = g.inputs.double_bottom_height.unwrap()
        + g.inputs.tank_support_height
        + g.inputs.wing_tank_height_ratio * g.tanks.outer_diameter;
    assert!(top < 6.5, "wing top {top}");
    let hull = Hull::build(r.hull_spec.as_ref().unwrap()).unwrap().table;
    let inp = VesselModelInputs {
        capacity: i,
        result: r,
        hull,
    };
    let m = assemble(&inp);
    let holds_x = gas_hold_bounds(&inp.capacity, &inp.result);
    assert!(holds_x.len() >= 2);
    let of = |k: SpaceKind| -> Vec<&ovd_core::compartments::Space> {
        m.spaces
            .iter()
            .filter(|s| s.space.kind == k)
            .map(|s| &s.space)
            .collect()
    };
    let holds = of(SpaceKind::CargoHold);
    let names: Vec<&str> = holds.iter().map(|h| h.name.as_str()).collect();
    let want: Vec<String> = (1..=holds_x.len()).map(|k| format!("H{k}C")).collect();
    assert_eq!(names, want);
    for (h, x) in holds.iter().zip(&holds_x) {
        assert_eq!((h.x_aft, h.x_fwd), *x);
    }
    // a watertight bulkhead at each joint
    for x in &holds_x[1..] {
        assert!(
            m.bulkheads.iter().any(|b| b.x_m == x.0 && b.watertight),
            "no bulkhead at {}: {:?}",
            x.0,
            m.bulkheads
        );
    }
    // each tank in its hold, and the tanks follow the last hold in the list
    let layout = m.tank_layout.as_ref().unwrap();
    for t in &layout.tanks {
        let h = &holds[t.row];
        assert!(h.x_aft <= t.axis.x0_m && t.axis.x1_m <= h.x_fwd);
    }
    let last_hold = m
        .spaces
        .iter()
        .rposition(|s| s.space.kind == SpaceKind::CargoHold)
        .unwrap();
    assert_eq!(m.spaces[last_hold + 1].space.kind, SpaceKind::CargoTank);
    // a void strip each side per hold, from the wing tank top to the deck, as wide as the wing tank
    let strips: Vec<_> = of(SpaceKind::Void)
        .into_iter()
        .filter(|s| s.name.starts_with("WV"))
        .collect();
    assert_eq!(strips.len(), 2 * holds_x.len());
    let wings = of(SpaceKind::WingBallast);
    for s in &strips {
        let w = wings
            .iter()
            .find(|w| w.name[2..] == s.name[2..])
            .expect("its wing tank");
        assert_eq!((s.x_aft, s.x_fwd), (w.x_aft, w.x_fwd));
        assert_eq!(s.side, w.side);
        let (wz, sz) = (w.z_range(), s.z_range());
        assert_eq!(sz, (wz.1, 6.5));
        let ys = |sp: &ovd_core::compartments::Space| {
            sp.section()
                .iter()
                .fold((f64::MAX, f64::MIN), |(a, b), &(y, _)| (a.min(y), b.max(y)))
        };
        assert_eq!(ys(s), ys(w));
    }
    // the holds, wing tanks and strips fill the zone above the double bottom
    let g = inp.result.gas_basis.as_ref().unwrap();
    let db = g.inputs.double_bottom_height.unwrap();
    let l = holds_x[holds_x.len() - 1].1 - holds_x[0].0;
    let sum: f64 = holds
        .iter()
        .chain(&wings)
        .chain(&strips)
        .map(|s| s.volume_m3())
        .sum();
    let want = 14.8 * (6.5 - db) * l;
    assert!((sum - want).abs() < 1e-9 * want, "{sum} vs {want}");
}
