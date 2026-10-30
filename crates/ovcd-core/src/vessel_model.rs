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
/// their shape. The Propulsor Layout joins it later (spec 0009).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Outfit {
    /// The Deck Items of the stern block: the container units, then the wheelhouse lowered and
    /// raised, then its lift column. Empty without a stern block.
    pub deck_items: Vec<DeckItem>,
}

/// The two positions of a lifting wheelhouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WheelhousePosition {
    Lowered,
    Raised,
}

/// What a Deck Item is, and where its size comes from in the stern block's result
/// (`AftArrangementResult`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeckItemKind {
    /// A container unit: entry `index` of `container_units`, an accommodation or a battery unit.
    Unit {
        index: usize,
        battery: bool,
    },
    Wheelhouse(WheelhousePosition),
    /// The column the wheelhouse rises on: from the deck to the raised floor.
    LiftColumn,
}

/// A Deck Item (GLOSSARY.md): an Outfit part that stands on the deck, as a box in hull axes (x
/// forward from the aft end, y to starboard, z up from the keel).
#[derive(Debug, Clone, PartialEq)]
pub struct DeckItem {
    pub kind: DeckItemKind,
    /// The unit's label from the stern block ("ACCOM 40 FT"); the kind's name for the others.
    pub label: String,
    pub x_aft_m: f64,
    pub x_fwd_m: f64,
    pub y_port_m: f64,
    pub y_stbd_m: f64,
    pub z_base_m: f64,
    pub z_top_m: f64,
}

/// The weather shield over the tanks (issue #410), as the weights carry it (`weights.rs`) and the
/// general arrangement draws it: a top plate over the cargo zone, the full beam wide, at the tank top
/// plus the clearance; side walls at the hull sides and end walls at the ends of the zone, standing
/// on the wing plate or on the deck where the deck is higher. All plates `thickness_m` thick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeatherShield {
    pub x_aft_m: f64,
    pub x_fwd_m: f64,
    pub half_width_m: f64,
    /// The level its walls stand on, above the keel.
    pub z_foot_m: f64,
    /// The top of its top plate, above the keel.
    pub z_top_m: f64,
    pub thickness_m: f64,
}

/// The plan size of the wheelhouse lift column, square, over the wheelhouse width. An Estimate
/// (spec 0009, D2; basis DR-A45): no published proportion is held.
pub const LIFT_COLUMN_SIDE_OVER_WHEELHOUSE_WIDTH: f64 = 0.25;

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
    /// The weather shield over the tanks of a gas hull (issue #410); `None` without one.
    pub weather_shield: Option<WeatherShield>,
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
        weather_shield: weather_shield(r),
        tank_layout,
        outfit: Outfit {
            deck_items: deck_items(r, &inp.hull),
        },
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

/// The deck height under a footprint: the highest point of the deck from `x0` to `x1` and from `y0`
/// to `y1` (y to starboard). Hull Forms have camber and sheer, so a unit stands on the highest point
/// under it and never cuts into the deck (spec 0009). The deck is the top of each half-section, from
/// the deck edge to the centre line; between Sections it is the ruled surface. The highest point is
/// at a Station or an end of the footprint, and at a point of the deck line or an edge of the
/// footprint, so those are the points tried. With no deck under the footprint (a slot), the deck
/// edge height.
pub fn deck_height_over(table: &SectionsTable, (x0, x1): (f64, f64), (y0, y1): (f64, f64)) -> f64 {
    let (a, b) = (y0.abs(), y1.abs());
    let (lo, hi) = if y0 <= 0.0 && y1 >= 0.0 {
        (0.0, a.max(b))
    } else {
        (a.min(b), a.max(b))
    };
    let mut xs = vec![x0, x1];
    xs.extend(
        table
            .sections
            .iter()
            .map(|s| s.x)
            .filter(|x| *x > x0 && *x < x1),
    );
    let mut top = f64::NEG_INFINITY;
    for x in xs {
        let (section, edge) = table.section_at(x);
        let deck = deck_line(&section.points, edge);
        let mut ys = vec![lo, hi];
        ys.extend(deck.iter().map(|p| p.0).filter(|y| *y > lo && *y < hi));
        let mut any = false;
        for y in ys {
            if let Some(z) = deck_z(&deck, y) {
                top = top.max(z);
                any = true;
            }
        }
        if !any {
            top = top.max(edge);
        }
    }
    top
}

/// The deck line of a half-section: its points from the deck edge to the end (the centre line, or
/// the slot edge), the run of last points at or above the deck edge height.
fn deck_line(points: &[(f64, f64)], edge: f64) -> Vec<(f64, f64)> {
    let mut start = points.len();
    while start > 0 && points[start - 1].1 >= edge - 1e-9 {
        start -= 1;
    }
    points[start..].to_vec()
}

/// The deck height at half-breadth `y` on a deck line (y falling from the deck edge inboard);
/// `None` where the line does not reach `y`.
fn deck_z(deck: &[(f64, f64)], y: f64) -> Option<f64> {
    let tol = 1e-9;
    for w in deck.windows(2) {
        let ((ya, za), (yb, zb)) = (w[0], w[1]);
        let (y_lo, y_hi) = (ya.min(yb), ya.max(yb));
        if y >= y_lo - tol && y <= y_hi + tol {
            if (ya - yb).abs() <= tol {
                return Some(za.max(zb));
            }
            return Some(za + (zb - za) * (y - ya) / (yb - ya));
        }
    }
    match deck {
        [(ya, za)] if (ya - y).abs() <= tol => Some(*za),
        _ => None,
    }
}

/// The Deck Items of the stern block (spec 0009, "Deck Items"): each container unit on its footprint
/// from the deck to the units' top, the wheelhouse lowered and raised, and the lift column from the
/// deck to the raised floor. Empty without a stern block.
fn deck_items(r: &BargeCapacityResult, table: &SectionsTable) -> Vec<DeckItem> {
    let Some(a) = r.aft_arrangement.as_ref() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let half_unit = a.container_unit_width_m / 2.0;
    for (index, (label, x_aft, x_fwd, y_centre)) in a.container_units.iter().enumerate() {
        // `container_units` gives y + = port; the model's y is + to starboard. Convert once, here.
        let y = -y_centre;
        let (y_port, y_stbd) = (y - half_unit, y + half_unit);
        out.push(DeckItem {
            kind: DeckItemKind::Unit {
                index,
                battery: label.starts_with("BATTERY"),
            },
            label: label.clone(),
            x_aft_m: *x_aft,
            x_fwd_m: *x_fwd,
            y_port_m: y_port,
            y_stbd_m: y_stbd,
            z_base_m: deck_height_over(table, (*x_aft, *x_fwd), (y_port, y_stbd)),
            z_top_m: a.container_top_above_keel_m,
        });
    }
    let half_wh = a.wheelhouse_width_m / 2.0;
    if !(a.wheelhouse_x1_m > a.wheelhouse_x0_m && half_wh > 0.0) {
        return out;
    }
    for (position, floor, top) in [
        (
            WheelhousePosition::Lowered,
            a.wheelhouse_floor_lowered_m,
            a.wheelhouse_top_lowered_m,
        ),
        (
            WheelhousePosition::Raised,
            a.wheelhouse_floor_raised_m,
            a.wheelhouse_top_raised_m,
        ),
    ] {
        out.push(DeckItem {
            kind: DeckItemKind::Wheelhouse(position),
            label: match position {
                WheelhousePosition::Lowered => "wheelhouse_lowered",
                WheelhousePosition::Raised => "wheelhouse_raised",
            }
            .to_string(),
            x_aft_m: a.wheelhouse_x0_m,
            x_fwd_m: a.wheelhouse_x1_m,
            y_port_m: -half_wh,
            y_stbd_m: half_wh,
            z_base_m: floor,
            z_top_m: top,
        });
    }
    let half_col = LIFT_COLUMN_SIDE_OVER_WHEELHOUSE_WIDTH * a.wheelhouse_width_m / 2.0;
    let mid = (a.wheelhouse_x0_m + a.wheelhouse_x1_m) / 2.0;
    let (x_aft, x_fwd) = (mid - half_col, mid + half_col);
    let base = deck_height_over(table, (x_aft, x_fwd), (-half_col, half_col));
    if a.wheelhouse_floor_raised_m > base {
        out.push(DeckItem {
            kind: DeckItemKind::LiftColumn,
            label: "lift_column".to_string(),
            x_aft_m: x_aft,
            x_fwd_m: x_fwd,
            y_port_m: -half_col,
            y_stbd_m: half_col,
            z_base_m: base,
            z_top_m: a.wheelhouse_floor_raised_m,
        });
    }
    out
}

/// The weather shield of a gas result, when its inputs are set: a plate thickness, a wing tank, a
/// wing plate under the tank top (the weights' condition), and a top above the deck (the general
/// arrangement draws it only then). `None` otherwise.
fn weather_shield(r: &BargeCapacityResult) -> Option<WeatherShield> {
    let g = r.gas_basis.as_ref()?;
    let i = &g.inputs;
    let thickness = i.weather_shield_thickness / 1000.0;
    if !(thickness > 0.0 && i.wing_tank_width? > 0.0 && i.wing_tank_height_ratio < 1.0) {
        return None;
    }
    let od = g.tanks.outer_diameter;
    let bottom =
        crate::tanks::tank_bottom(i.double_bottom_height.unwrap_or(0.0), i.tank_support_height);
    let wing_top = bottom + i.wing_tank_height_ratio * od;
    let top = bottom + od + i.weather_shield_clearance;
    if top <= i.depth {
        return None;
    }
    let zone = crate::tanks::CargoZone::new(
        i.loa,
        i.bow_non_tank_length.unwrap_or(0.0),
        i.stern_non_tank_length.unwrap_or(0.0),
    );
    Some(WeatherShield {
        x_aft_m: zone.x_aft_m,
        x_fwd_m: zone.x_fwd_m,
        half_width_m: i.beam / 2.0,
        z_foot_m: wing_top.max(i.depth),
        z_top_m: top,
        thickness_m: thickness,
    })
}
