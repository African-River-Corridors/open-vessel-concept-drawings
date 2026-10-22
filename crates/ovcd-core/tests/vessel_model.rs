//! Seam: `ovcd_core::vessel_model::assemble` (issue #454, spec 0008 "Phase 2: the Vessel Model").
//!
//! Label: property - the model holds the hull it is given, the holds of the volume model split
//! evenly with a bulkhead between each two, the ballast tanks of `compartments::ballast_tanks`
//! unchanged, an inner bottom at the double-bottom height and no frames. Label: sourced - closed
//! form, the hold prisms sum to the volume model's hold volume (section area times cargo zone
//! length, `containment_inner_volume_m3`), and the hull clips them at the deck: the trapezoid from
//! the inner bottom to the deck times the cargo zone length (issue #455).

use ovcd_core::barge_capacity::{BargeCapacityInputs, compute};
use ovcd_core::compartments::{SpaceKind, ballast_tanks};
use ovcd_core::hydrostatics::Hull;
use ovcd_core::vessel_model::{VesselModelInputs, assemble};

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
