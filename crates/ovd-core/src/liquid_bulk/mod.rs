//! Liquid bulk cargo: crude oil on tank barges and small coastal tankers (spec 0006, ADR 0016).
//!
//! One interface: [`compute`] turns a hull, a cargo and a tank layout into typed Spaces, volumes,
//! masses, free surfaces and a lightship. Callers read the [`LiquidBulkResult`] and nothing inside:
//! the layout, the clipping to the inner hull and the free surface stay in the submodules. The rules
//! read their Measures from the result only.
//!
//! The Spaces are those of [`crate::compartments`] (one Space model for every liquid a hull holds).
//! x is from the aft end of the hull, y from the centre line (positive to starboard), z from the
//! keel. Volumes are moulded: no deduction for structure.
//!
//! The Crude Grade data and the ADN Tank Vessel Type are in [`crude_grade`] and [`adn_vessel_type`].

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::compartments::{Space, SpaceKind};
use crate::loading_conditions::Item;
use crate::pressure_vessel::PyError;

pub mod adn_vessel_type;
pub mod cost;
pub mod crude_grade;
pub mod damage;
mod free_surface;
mod layout;
pub mod lightship;
pub mod oil_outflow;

pub use free_surface::TankFreeSurface;

/// How the cargo is pumped out (decision D4 of spec 0006).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PumpArrangement {
    /// One deepwell pump per tank, on deck: no pump room.
    Deepwell,
    /// A pump room below deck, this long (m), at the aft end of the cargo zone.
    PumpRoom { length_m: f64 },
}

/// The tank layout: what a Design chooses about the cargo tanks (spec 0006 section 3.2). Lengths in m.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidLayout {
    /// The width of the double side (m).
    pub double_side_m: f64,
    /// The height of the double bottom (m).
    pub double_bottom_m: f64,
    /// The number of cargo tanks along the hull, at least 1.
    pub tanks_in_line: u32,
    /// A centreline bulkhead splits every tank into a port and a starboard tank.
    pub centreline_bulkhead: bool,
    /// The length of a cofferdam (m). One at each end of the tank block.
    pub cofferdam_m: f64,
    pub pump_arrangement: PumpArrangement,
    /// The slop tanks' share of the cargo tank volume, 0 to under 1.
    pub slop_share: f64,
    /// The fraction of a cargo tank's volume that is filled, 0 to 1.
    pub fill_ratio: f64,
    /// The installed heating (kW per m³ of cargo tank volume); 0 for none.
    pub heating_kw_per_m3: f64,
    /// The hull lightship factor, t per m³ of L x B x D (see [`lightship`]).
    pub lightship_k_t_per_m3: f64,
}

/// The inputs of [`compute`]. Lengths in m, x from the aft end of the hull.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidBulkInputs {
    pub loa_m: f64,
    pub beam_m: f64,
    pub depth_m: f64,
    /// The cargo zone: the part of the hull the cargo block takes, from the aft end of the zone to
    /// the fore end. The caller takes it from the end arrangement (stern block or void, bow rake).
    pub zone_aft_m: f64,
    pub zone_fwd_m: f64,
    /// The width of the double side (m).
    pub double_side_m: f64,
    /// The height of the double bottom (m).
    pub double_bottom_m: f64,
    /// The number of cargo tanks along the hull, at least 1.
    pub tanks_in_line: u32,
    /// A centreline bulkhead splits every tank into a port and a starboard tank.
    pub centreline_bulkhead: bool,
    /// The length of a cofferdam (m). One at each end of the tank block.
    pub cofferdam_m: f64,
    pub pump_arrangement: PumpArrangement,
    /// The slop tanks' share of the cargo tank volume, 0 to under 1. They take that share of the
    /// tank block, at its aft end, ahead of the cargo tanks.
    pub slop_share: f64,
    /// The fraction of a cargo tank's volume that is filled, 0 to 1.
    pub fill_ratio: f64,
    /// The cargo density at the carriage temperature (t/m³).
    pub cargo_density_t_m3: f64,
    /// The installed heating (kW per m³ of cargo tank volume); 0 for none.
    pub heating_kw_per_m3: f64,
    /// The hull lightship factor, t per m³ of L x B x D.
    pub lightship_k_t_per_m3: f64,
}

impl LiquidBulkInputs {
    /// The inputs of a hull, a cargo zone, a cargo density (t/m³) and a layout.
    pub fn of(
        (loa_m, beam_m, depth_m): (f64, f64, f64),
        (zone_aft_m, zone_fwd_m): (f64, f64),
        cargo_density_t_m3: f64,
        l: &LiquidLayout,
    ) -> LiquidBulkInputs {
        LiquidBulkInputs {
            loa_m,
            beam_m,
            depth_m,
            zone_aft_m,
            zone_fwd_m,
            double_side_m: l.double_side_m,
            double_bottom_m: l.double_bottom_m,
            tanks_in_line: l.tanks_in_line,
            centreline_bulkhead: l.centreline_bulkhead,
            cofferdam_m: l.cofferdam_m,
            pump_arrangement: l.pump_arrangement,
            slop_share: l.slop_share,
            fill_ratio: l.fill_ratio,
            cargo_density_t_m3,
            heating_kw_per_m3: l.heating_kw_per_m3,
            lightship_k_t_per_m3: l.lightship_k_t_per_m3,
        }
    }
}

/// The result of [`compute`].
#[derive(Debug, Clone, PartialEq)]
pub struct LiquidBulkResult {
    /// Every Space, in a fixed order: the cargo tanks (1P, 1S, 2P, ... or 1, 2, ...), the slop
    /// tanks, the double bottom and wing ballast tanks, the cofferdams, then the pump room.
    pub spaces: Vec<Space>,
    /// The number of cargo tank positions along the hull (a P and S pair is one).
    pub tanks_in_line: u32,
    /// The gross volume of the cargo tanks, slop excluded (m³).
    pub cargo_gross_m3: f64,
    /// The gross volume of the slop tanks (m³).
    pub slop_gross_m3: f64,
    /// The cargo the tanks hold at the fill ratio (m³): the gross volume times the fill ratio.
    pub cargo_volume_m3: f64,
    /// The cargo mass when the tanks are full to the fill ratio (t): the volume-bound cargo.
    pub cargo_mass_t: f64,
    /// The slop volume over the cargo and slop volume together.
    pub slop_share_actual: f64,
    /// The length of the shortest and the longest cargo tank (m).
    pub cargo_tank_length_min_m: f64,
    pub cargo_tank_length_max_m: f64,
    /// The widest cargo tank (m).
    pub cargo_tank_breadth_max_m: f64,
    /// The width of the double side (m): the layout's, the same along the tank block. A Measure of
    /// the hull (`hull.double_side_width`).
    pub double_side_width_m: f64,
    /// The height of the double bottom (m) (`hull.double_bottom_height`).
    pub double_bottom_height_m: f64,
    /// The overall length of the hull (m): the L of ADN 2025 9.3.2.11.1 (d).
    pub loa_m: f64,
    /// The extreme breadth of the hull (m).
    pub beam_m: f64,
    /// L x B x H of the hull (m³), as ADN 2025 9.3.2.11.1 (a) defines it for the tank capacity table:
    /// L the overall length of the hull, B the extreme breadth, H the moulded depth within the cargo
    /// area (`hull.lbh`).
    pub lbh_m3: f64,
    /// The free surface of each cargo and slop tank at a slack level.
    pub free_surface: Vec<TankFreeSurface>,
    /// The lightship (t): the hull and the outfit.
    pub lightship_t: f64,
    /// The lightship as mass items (t, KG, LCG), for the stability: the hull, then each outfit item.
    pub lightship_items: Vec<(String, Item)>,
}

impl LiquidBulkResult {
    /// The Spaces of one kind, in order.
    pub fn of_kind(&self, kind: SpaceKind) -> impl Iterator<Item = &Space> {
        self.spaces.iter().filter(move |s| s.kind == kind)
    }
}

fn value_error(message: String) -> PyError {
    PyError {
        kind: "ValueError",
        message,
    }
}

fn check_inputs(inp: &LiquidBulkInputs) -> Result<(), PyError> {
    let positive = [
        ("loa_m", inp.loa_m),
        ("beam_m", inp.beam_m),
        ("depth_m", inp.depth_m),
        ("cargo_density_t_m3", inp.cargo_density_t_m3),
    ];
    for (name, v) in positive {
        if !(v.is_finite() && v > 0.0) {
            return Err(value_error(format!("{name} must be a positive number")));
        }
    }
    let not_negative = [
        ("double_side_m", inp.double_side_m),
        ("double_bottom_m", inp.double_bottom_m),
        ("cofferdam_m", inp.cofferdam_m),
        ("heating_kw_per_m3", inp.heating_kw_per_m3),
        ("lightship_k_t_per_m3", inp.lightship_k_t_per_m3),
    ];
    for (name, v) in not_negative {
        if !(v.is_finite() && v >= 0.0) {
            return Err(value_error(format!("{name} must not be negative")));
        }
    }
    if inp.tanks_in_line < 1 {
        return Err(value_error("tanks_in_line must be at least 1".into()));
    }
    if !(inp.slop_share.is_finite() && (0.0..1.0).contains(&inp.slop_share)) {
        return Err(value_error(
            "slop_share must be 0 or more and under 1".into(),
        ));
    }
    if !(inp.fill_ratio.is_finite() && inp.fill_ratio > 0.0 && inp.fill_ratio <= 1.0) {
        return Err(value_error(
            "fill_ratio must be over 0 and at most 1".into(),
        ));
    }
    if let PumpArrangement::PumpRoom { length_m } = inp.pump_arrangement
        && !(length_m.is_finite() && length_m > 0.0)
    {
        return Err(value_error(
            "the pump room length must be a positive number".into(),
        ));
    }
    if !(inp.zone_aft_m.is_finite()
        && inp.zone_fwd_m.is_finite()
        && inp.zone_aft_m >= 0.0
        && inp.zone_fwd_m > inp.zone_aft_m
        && inp.zone_fwd_m <= inp.loa_m)
    {
        return Err(value_error(
            "the cargo zone must lie inside the hull, with a positive length".into(),
        ));
    }
    if inp.double_side_m >= 0.5 * inp.beam_m {
        return Err(value_error(
            "the double side must be less than half the beam".into(),
        ));
    }
    if inp.double_bottom_m >= inp.depth_m {
        return Err(value_error(
            "the double bottom must be lower than the depth".into(),
        ));
    }
    Ok(())
}

/// The Spaces, volumes, masses, free surfaces and lightship of a liquid bulk hull.
///
/// Errors: an input out of range, or a cargo zone too short for its end spaces and one tank.
pub fn compute(inp: &LiquidBulkInputs) -> Result<LiquidBulkResult, PyError> {
    check_inputs(inp)?;
    let spaces = layout::spaces(inp)?;
    let cargo: Vec<&Space> = spaces
        .iter()
        .filter(|s| s.kind == SpaceKind::CargoTank)
        .collect();
    let slop: Vec<&Space> = spaces
        .iter()
        .filter(|s| s.kind == SpaceKind::Slop)
        .collect();
    let cargo_gross = cargo.iter().fold(0.0, |a, s| a + s.volume_m3());
    let slop_gross = slop.iter().fold(0.0, |a, s| a + s.volume_m3());
    let cargo_volume = cargo_gross * inp.fill_ratio;
    let (mut len_min, mut len_max, mut breadth_max) = (f64::INFINITY, 0.0_f64, 0.0_f64);
    for s in &cargo {
        len_min = len_min.min(s.length());
        len_max = len_max.max(s.length());
        breadth_max = breadth_max.max(free_surface::breadth_m(s));
    }
    let tanks_in_line = inp.tanks_in_line;
    let (lightship_t, lightship_items) = lightship::compute(inp, cargo_gross + slop_gross);
    Ok(LiquidBulkResult {
        free_surface: free_surface::of_tanks(&spaces),
        tanks_in_line,
        loa_m: inp.loa_m,
        beam_m: inp.beam_m,
        double_side_width_m: inp.double_side_m,
        double_bottom_height_m: inp.double_bottom_m,
        lbh_m3: inp.loa_m * inp.beam_m * inp.depth_m,
        cargo_gross_m3: cargo_gross,
        slop_gross_m3: slop_gross,
        cargo_volume_m3: cargo_volume,
        cargo_mass_t: cargo_volume * inp.cargo_density_t_m3,
        slop_share_actual: if cargo_gross + slop_gross > 0.0 {
            slop_gross / (cargo_gross + slop_gross)
        } else {
            0.0
        },
        cargo_tank_length_min_m: len_min,
        cargo_tank_length_max_m: len_max,
        cargo_tank_breadth_max_m: breadth_max,
        lightship_t,
        lightship_items,
        spaces,
    })
}
