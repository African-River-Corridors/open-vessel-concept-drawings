//! The Vessel Model (spec 0008, "Phase 2: the Vessel Model", ADR 0018, issue #454): the hull, the
//! decks, the bulkheads and the Spaces of one Option, in hull axes (x forward from the aft end, y to
//! starboard, z up from the base line).
//!
//! No new inputs: [`assemble`] takes what the engine already placed. The hull is the Option's
//! Sections Table. The Spaces are the hold or tank layout, the Type C cargo tanks of a gas hull
//! (the Tank Layout, ADR 0021), `compartments::ballast_tanks` (double bottom, wing tanks, peaks),
//! the gas separation and the machinery spaces of the aft block. The bulkheads are the ones the
//! general arrangement draws. Each Space is clipped by the hull (issue #455): a Space wholly inside
//! keeps its prism volume and centroid, and a tank passes through unchanged.
//!
//! No result reads the model, so it moves no results hash.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::barge_capacity::{BargeCapacityInputs, BargeCapacityResult};
use crate::cargo::CargoClass;
use crate::compartments::{
    ClippedSpace, Side, Space, SpaceKind, SpaceShape, ballast_tanks, clip, tank_spaces,
};
use crate::sections::SectionsTable;
use crate::tanks::TankLayout;

/// What the model is assembled from: the capacity inputs and result the run worked with, and the
/// hull it floated.
#[derive(Debug, Clone, PartialEq)]
pub struct VesselModelInputs {
    pub capacity: BargeCapacityInputs,
    pub result: BargeCapacityResult,
    pub hull: SectionsTable,
}

/// A deck: a horizontal plane at `z_m` from `x_aft_m` to `x_fwd_m`, clipped by the hull.
#[derive(Debug, Clone, PartialEq)]
pub struct Deck {
    /// A key, the same in every language: `main_deck`, `inner_bottom`.
    pub name: String,
    /// The height at the side, m above the base line.
    pub z_m: f64,
    pub x_aft_m: f64,
    pub x_fwd_m: f64,
}

/// A transverse bulkhead: a plane at `x_m`, clipped by the hull.
#[derive(Debug, Clone, PartialEq)]
pub struct Bulkhead {
    /// A key, the same in every language: `cargo_zone_aft`, `hold_1_2`, `zone_3_fwd`.
    pub name: String,
    pub x_m: f64,
    /// False only for the end of a deck item of the aft block (the accommodation units, the
    /// wheelhouse), which the general arrangement draws as a line under the deck.
    pub watertight: bool,
}

/// The Outfit layer (ADR 0021): the parts that are neither hull, compartments nor structure, and
/// are not in the hydrostatics. Their masses are already in the lightship; the Outfit adds only
/// their shape. Empty until the Deck Items and the Propulsor Layout fill it (spec 0009).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Outfit {}

/// The Vessel Model of spec 0008, with the tank Spaces and the Outfit layer of ADR 0021.
#[derive(Debug, Clone, PartialEq)]
pub struct VesselModel {
    pub hull: SectionsTable,
    pub decks: Vec<Deck>,
    /// In ascending x; one per x.
    pub bulkheads: Vec<Bulkhead>,
    pub spaces: Vec<ClippedSpace>,
    /// The Tank Layout of a gas hull, with its domes; `None` for any other hull.
    pub tank_layout: Option<TankLayout>,
    pub outfit: Outfit,
    /// Frame stations; empty until phase 3.
    pub frames: Vec<f64>,
}

/// Two bulkheads closer than this (m) are one.
const SAME_X_M: f64 = 1e-9;

/// Assemble the Vessel Model of one Option.
pub fn assemble(inp: &VesselModelInputs) -> VesselModel {
    let (c, r) = (&inp.capacity, &inp.result);
    let tank_layout = gas_tank_layout(c, r);
    let mut spaces = spaces(c, r);
    if let (Some(layout), Some(g)) = (tank_layout.as_ref(), r.gas_basis.as_ref()) {
        // The Type C tanks follow the hold that houses them.
        let at = spaces
            .iter()
            .position(|s| s.kind == SpaceKind::CargoHold)
            .map_or(0, |i| i + 1);
        let tanks = tank_spaces(layout, g.tanks.volume_per_tank);
        spaces.splice(at..at, tanks);
    }
    VesselModel {
        hull: inp.hull.clone(),
        decks: decks(c, &spaces),
        bulkheads: bulkheads(c, r),
        spaces: spaces.iter().map(|s| clip(s, &inp.hull)).collect(),
        tank_layout,
        outfit: Outfit::default(),
        frames: Vec::new(),
    }
}

/// The Tank Layout of a gas result, with the dome in force; `None` for any other result.
fn gas_tank_layout(c: &BargeCapacityInputs, r: &BargeCapacityResult) -> Option<TankLayout> {
    let g = r.gas_basis.as_ref()?;
    let dome = crate::barge_capacity::dome_in_force(
        c,
        g.tanks.outer_diameter,
        g.inputs.insulation_thickness / 1000.0,
    );
    crate::tanks::arrangement_layout(&g.inputs, &g.tanks, Some(dome))
}

/// The ends of the cargo zone: aft of it the stern void, forward of it the bow void.
fn cargo_zone(c: &BargeCapacityInputs, r: &BargeCapacityResult) -> (f64, f64) {
    let stern = r.stern_void_length_m.unwrap_or(0.0);
    let bow = r.bow_void_length_m.unwrap_or(0.0);
    (stern, (c.loa - bow).max(stern))
}

/// A section across the centre line, from `-half` to `half` and `z0` to `z1`.
fn across(half: f64, z0: f64, z1: f64) -> Vec<(f64, f64)> {
    vec![(-half, z0), (half, z0), (half, z1), (-half, z1)]
}

fn centre(name: String, kind: SpaceKind, x: (f64, f64), section: Vec<(f64, f64)>) -> Space {
    Space {
        name,
        kind,
        side: Side::Centre,
        x_aft: x.0,
        x_fwd: x.1,
        shape: SpaceShape::Prism { section },
    }
}

/// The Spaces: the cargo spaces, the ballast tanks, the other Spaces of a liquid bulk tank block,
/// the gas separation and the machinery spaces. A Space of no length is left out.
fn spaces(c: &BargeCapacityInputs, r: &BargeCapacityResult) -> Vec<Space> {
    let half_b = c.beam / 2.0;
    let (x0, x1) = cargo_zone(c, r);
    let mut out = Vec::new();
    match r.phase {
        CargoClass::DryBulk => {
            // The hold of the volume model (`barge_capacity`): a trapezoid from the inner bottom to
            // the coaming top, split evenly into `n_holds`.
            let db = c.double_bottom_height_bulk;
            let w_top = r.hold_top_width_m.unwrap_or(0.0);
            let w_bot = r.hold_bottom_width_m.unwrap_or(w_top);
            let h = r.hold_height_m.unwrap_or(0.0);
            let n = c.n_holds.max(1);
            let length = (x1 - x0) / n as f64;
            for i in 0..n {
                let xa = x0 + length * i as f64;
                out.push(centre(
                    format!("H{}C", i + 1),
                    SpaceKind::CargoHold,
                    (xa, if i + 1 == n { x1 } else { xa + length }),
                    vec![
                        (-w_bot / 2.0, db),
                        (w_bot / 2.0, db),
                        (w_top / 2.0, db + h),
                        (-w_top / 2.0, db + h),
                    ],
                ));
            }
        }
        CargoClass::Gas => {
            // The hold that houses the Type C tanks: inboard of the wing tanks, from the double
            // bottom to the deck.
            if let Some(g) = r.gas_basis.as_ref() {
                let db = g.inputs.double_bottom_height.unwrap_or(0.0);
                let w = g.inputs.wing_tank_width.unwrap_or(0.0).clamp(0.0, half_b);
                out.push(centre(
                    "H1C".to_string(),
                    SpaceKind::CargoHold,
                    (x0, x1),
                    across(half_b - w, db, c.depth),
                ));
            }
        }
        CargoClass::LiquidBulk => {}
    }
    out.extend(ballast_tanks(c, r));
    if let Some(l) = r.liquid.as_ref() {
        out.extend(l.spaces.iter().filter(|s| !s.kind.is_ballast()).cloned());
    }
    // The gas separation aft of the cargo zone: the cofferdam, then the service space.
    if let Some(s) = r.separation.as_ref() {
        let start = x0 - s.aft_separation_m;
        let cofferdam = (start, start + s.cofferdam_m);
        let service = (cofferdam.1, cofferdam.1 + s.service_space_m);
        for (name, kind, x) in [
            ("CD1C", SpaceKind::Cofferdam, cofferdam),
            ("SV1C", SpaceKind::Void, service),
        ] {
            out.push(centre(
                name.to_string(),
                kind,
                x,
                across(half_b, 0.0, c.depth),
            ));
        }
    }
    // The machinery spaces below the deck of the aft block, full beam and depth.
    let mut m = 0;
    for (label, xa, xf) in below_deck_zones(r) {
        let (name, kind) = if label.starts_with("AFT PEAK") {
            ("APV1C".to_string(), SpaceKind::PeakVoid)
        } else {
            m += 1;
            (format!("M{m}C"), SpaceKind::Machinery)
        };
        out.push(centre(name, kind, (xa, xf), across(half_b, 0.0, c.depth)));
    }
    out.retain(|s| s.x_fwd > s.x_aft);
    out
}

/// The zones of the aft block below the deck: the machinery's own zones, or every zone but the
/// deck items (the accommodation units and the wheelhouse).
fn below_deck_zones(r: &BargeCapacityResult) -> Vec<(String, f64, f64)> {
    let Some(a) = r.aft_arrangement.as_ref() else {
        return Vec::new();
    };
    if let Some(m) = a.machinery.as_ref() {
        return m.zones.clone();
    }
    a.zones
        .iter()
        .filter(|z| !is_deck_item(a, z))
        .cloned()
        .collect()
}

fn is_deck_item(
    a: &crate::aft_arrangement::AftArrangementResult,
    (_, xa, xf): &(String, f64, f64),
) -> bool {
    *xa == a.container_x0_m || (*xa == a.wheelhouse_x0_m && *xf == a.wheelhouse_x1_m)
}

/// The main deck over the length, and the inner bottom over the double-bottom tanks.
fn decks(c: &BargeCapacityInputs, spaces: &[Space]) -> Vec<Deck> {
    let mut out = vec![Deck {
        name: "main_deck".to_string(),
        z_m: c.depth,
        x_aft_m: 0.0,
        x_fwd_m: c.loa,
    }];
    let db: Vec<&Space> = spaces
        .iter()
        .filter(|s| s.kind == SpaceKind::DoubleBottomBallast)
        .collect();
    if !db.is_empty() {
        let fold = |f: fn(f64, f64) -> f64, init: f64, g: fn(&Space) -> f64| {
            db.iter().fold(init, |a, s| f(a, g(s)))
        };
        let top = fold(f64::max, 0.0, |s| s.z_range().1);
        if top > 0.0 {
            out.push(Deck {
                name: "inner_bottom".to_string(),
                z_m: top,
                x_aft_m: fold(f64::min, f64::INFINITY, |s| s.x_aft),
                x_fwd_m: fold(f64::max, f64::NEG_INFINITY, |s| s.x_fwd),
            });
        }
    }
    out
}

/// The bulkheads the general arrangement draws: the ends of the cargo zone, the bulkheads between
/// the holds, the ends of each Space of a liquid bulk tank block, the gas separation's, and the
/// forward end of each zone of the aft block. Inside the hull only, one per x (the first named).
fn bulkheads(c: &BargeCapacityInputs, r: &BargeCapacityResult) -> Vec<Bulkhead> {
    let (x0, x1) = cargo_zone(c, r);
    let mut out: Vec<Bulkhead> = Vec::new();
    let mut add = |name: String, x: f64, watertight: bool| {
        if x > 0.0 && x < c.loa {
            out.push(Bulkhead {
                name,
                x_m: x,
                watertight,
            });
        }
    };
    add("cargo_zone_aft".to_string(), x0, true);
    add("cargo_zone_fwd".to_string(), x1, true);
    if r.phase == CargoClass::DryBulk {
        let n = c.n_holds.max(1);
        for i in 1..n {
            add(
                format!("hold_{i}_{}", i + 1),
                x0 + (x1 - x0) * i as f64 / n as f64,
                true,
            );
        }
    }
    if let Some(l) = r.liquid.as_ref() {
        for s in l.spaces.iter().filter(|s| !s.kind.is_ballast()) {
            add(format!("{}_aft", s.name), s.x_aft, true);
            add(format!("{}_fwd", s.name), s.x_fwd, true);
        }
    }
    if let Some(s) = r.separation.as_ref() {
        let start = x0 - s.aft_separation_m;
        if s.cofferdam_m > 0.0 {
            add("cofferdam_aft".to_string(), start, true);
            add("cofferdam_fwd".to_string(), start + s.cofferdam_m, true);
        }
        if s.fwd_separation_m > 0.0 {
            add("fwd_separation".to_string(), x1 + s.fwd_separation_m, true);
        }
    }
    if let Some(a) = r.aft_arrangement.as_ref() {
        for (i, z) in a.zones.iter().enumerate() {
            add(format!("zone_{}_fwd", i + 1), z.2, !is_deck_item(a, z));
        }
    }
    out.sort_by(|a, b| a.x_m.total_cmp(&b.x_m));
    out.dedup_by(|b, a| (b.x_m - a.x_m).abs() < SAME_X_M);
    out
}
