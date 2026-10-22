//! Seam: `ovcd_core::vessel_model::assemble` (issue #454, spec 0008 "Phase 2: the Vessel Model").
//!
//! Label: property - the model holds the hull it is given, the holds of the volume model split
//! evenly with a bulkhead between each two, the ballast tanks of `compartments::ballast_tanks`
//! unchanged, an inner bottom at the double-bottom height and no frames. Label: sourced - closed
//! form, the hold prisms sum to the volume model's hold volume (section area times cargo zone
//! length, `containment_inner_volume_m3`).

use ovcd_core::barge_capacity::{BargeCapacityInputs, compute};
use ovcd_core::compartments::{SpaceKind, ballast_tanks};
use ovcd_core::hull::hull_form::{HullFormInputs, HullFormKind, HullShape, generate};
use ovcd_core::vessel_model::{VesselModelInputs, assemble};

fn dry_bulk(n_holds: i64) -> VesselModelInputs {
    let mut i = BargeCapacityInputs::new(80.0, 11.4, 4.0);
    i.cargo_type = "bauxite".to_string();
    i.hold_type = "sloped_hopper".to_string();
    i.n_holds = n_holds;
    let r = compute(&i).unwrap();
    let hull = generate(&HullFormInputs {
        length_overall_m: 80.0,
        beam_m: 11.4,
        moulded_depth_m: 4.0,
        form: HullFormKind::Box,
        shape: HullShape::default(),
    })
    .unwrap();
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
        let v: f64 = holds.iter().map(|s| s.volume_m3).sum();
        let want = r.containment_inner_volume_m3.unwrap();
        assert!((v - want).abs() <= 1e-12 * want, "{v} vs {want}");
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
        // A prism's centroid is at its mid-length.
        for s in &m.spaces {
            assert_eq!(s.centroid_m.0, (s.space.x_aft + s.space.x_fwd) / 2.0);
        }
    }
}
