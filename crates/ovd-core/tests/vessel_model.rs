//! Seam: `ovd_core::vessel_model::assemble` (issue #454, spec 0008 "Phase 2: the Vessel Model").
//!
//! Label: property - the model holds the hull it is given, the holds of the volume model split
//! evenly with a bulkhead between each two, the ballast tanks of `compartments::ballast_tanks`
//! unchanged, an inner bottom at the double-bottom height, and frames FR0, FR1, ... at the frame
//! spacing of the structure block (spec 0013, issue #739). Label: sourced - closed
//! form, the hold prisms sum to the volume model's hold volume (section area times cargo zone
//! length, `containment_inner_volume_m3`), and the hull clips them at the deck: the trapezoid from
//! the inner bottom to the deck times the cargo zone length (issue #455).

use ovd_core::barge_capacity::{BargeCapacityInputs, compute};
use ovd_core::compartments::{SpaceKind, ballast_tanks};
use ovd_core::hydrostatics::Hull;
use ovd_core::vessel_model::{StructureBlock, VesselModelInputs, assemble, frames_of};

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
        structure: StructureBlock::default(),
    }
}

#[test]
fn a_dry_bulk_barge_has_its_holds_bulkheads_ballast_tanks_and_decks() {
    for n in [1, 3, 4] {
        let inp = dry_bulk(n);
        let m = assemble(&inp);
        let r = &inp.result;
        assert_eq!(m.hull, inp.hull);
        assert_eq!(m.structure, StructureBlock::default());
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

/// Spec 0013 (issue #739): the frames are named reference planes FR0, FR1, ... at the frame
/// spacing, from the aft end of the hull to its forward end.
#[test]
fn frames_are_named_reference_planes_at_the_frame_spacing() {
    let mut inp = dry_bulk(3);
    let x0 = inp.hull.sections[0].x;
    let x1 = inp.hull.sections[inp.hull.sections.len() - 1].x;
    for spacing in [3.66, 2.5, 0.7] {
        inp.structure.frame_spacing_m = spacing;
        let m = assemble(&inp);
        assert_eq!(m.structure.frame_spacing_m, spacing);
        let n = ((x1 - x0) / spacing).floor() as usize + 1;
        assert_eq!(m.frames.len(), n, "spacing {spacing}");
        for (i, f) in m.frames.iter().enumerate() {
            assert_eq!(f.name, format!("FR{i}"));
            assert!((f.x_m - (x0 + spacing * i as f64)).abs() < 1e-9);
            assert!(f.x_m <= x1 + 1e-9);
        }
        // The last frame is less than one spacing from the forward end.
        assert!(x1 - m.frames[n - 1].x_m < spacing);
        assert_eq!(m.frames, frames_of(&inp.hull, spacing));
    }
}

/// A spacing that is not a positive finite number gives no frames (validation refuses it first).
#[test]
fn no_frames_without_a_positive_spacing() {
    let inp = dry_bulk(1);
    for s in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(frames_of(&inp.hull, s).is_empty(), "{s}");
    }
}

/// Spec 0013, "Format": the defaults of the structure block.
#[test]
fn the_structure_block_has_the_spec_defaults() {
    let s = StructureBlock::default();
    assert_eq!(s.frame_spacing_m, 3.66);
    assert_eq!(s.longitudinal_spacing_mm, 700.0);
    assert_eq!((s.stock_plate_width_m, s.stock_plate_length_m), (3.0, 12.0));
    assert_eq!(s.thickness_step_mm, 0.5);
    assert_eq!(s.steel_grade, ovd_core::vessel_model::SteelGrade::Ordinary);
    assert!(!s.structural_wing_deck);
    // The panel model's own defaults are the same numbers.
    assert_eq!(
        s.frame_spacing_m,
        ovd_core::hull_panels::DEFAULT_WEB_SPACING_M
    );
    assert_eq!(
        s.longitudinal_spacing_mm,
        ovd_core::hull_panels::DEFAULT_LONGITUDINAL_SPACING_MM
    );
}
