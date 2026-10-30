//! The layout: the end spaces, the tanks in line, the port and starboard split, the slop tanks and the
//! double hull, as typed Spaces.
//!
//! From aft to fore the cargo zone holds: the pump room (when there is one), a cofferdam, the tank
//! block, a cofferdam. The tank block holds the slop tanks at its aft end, then the cargo tanks, all
//! across the inner hull. The double bottom and the double side run the length of the tank block as
//! ballast. The cofferdams and the pump room take the full beam and depth. Together they fill the zone.
//!
//! The inner hull is a private seam with one adapter today: the parallel body, a box. The sections
//! table (ADR 0012) gives it a second adapter later.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use super::{LiquidBulkInputs, PumpArrangement, value_error};
use crate::compartments::{Side, Space, SpaceKind, SpaceShape, poly, rect};
use crate::pressure_vessel::PyError;

/// The inner hull of the parallel body: the box the cargo tanks sit in.
struct InnerHull {
    /// Half the beam: the outer side.
    half_beam: f64,
    /// Half the breadth inside the double side.
    inner_half: f64,
    /// The height of the inner bottom above the keel.
    bottom: f64,
    /// The deck.
    deck: f64,
}

impl InnerHull {
    fn of(inp: &LiquidBulkInputs) -> InnerHull {
        InnerHull {
            half_beam: 0.5 * inp.beam_m,
            inner_half: 0.5 * inp.beam_m - inp.double_side_m,
            bottom: inp.double_bottom_m,
            deck: inp.depth_m,
        }
    }

    /// The section of a tank: one side of the centre line, or across it.
    fn tank_section(&self, side: Side) -> Vec<(f64, f64)> {
        match side {
            Side::Centre => poly(
                Side::Starboard,
                &[
                    (-self.inner_half, self.bottom),
                    (self.inner_half, self.bottom),
                    (self.inner_half, self.deck),
                    (-self.inner_half, self.deck),
                ],
            ),
            s => rect(s, 0.0, self.inner_half, self.bottom, self.deck),
        }
    }

    /// A full-beam, full-depth section: a cofferdam or the pump room.
    fn end_section(&self) -> Vec<(f64, f64)> {
        poly(
            Side::Starboard,
            &[
                (-self.half_beam, 0.0),
                (self.half_beam, 0.0),
                (self.half_beam, self.deck),
                (-self.half_beam, self.deck),
            ],
        )
    }
}

fn space(
    name: String,
    kind: SpaceKind,
    side: Side,
    (x_aft, x_fwd): (f64, f64),
    section: Vec<(f64, f64)>,
) -> Space {
    Space {
        name,
        kind,
        side,
        x_aft,
        x_fwd,
        shape: SpaceShape::Prism { section },
    }
}

/// One tank position along the hull: a port and a starboard tank with a centreline bulkhead, else one.
fn tank_position(
    prefix: &str,
    kind: SpaceKind,
    number: u32,
    x: (f64, f64),
    hull: &InnerHull,
    centreline: bool,
) -> Vec<Space> {
    let sides: &[Side] = if centreline {
        &[Side::Port, Side::Starboard]
    } else {
        &[Side::Centre]
    };
    sides
        .iter()
        .map(|&side| {
            space(
                format!("{prefix}{number}{}", side.letter()),
                kind,
                side,
                x,
                hull.tank_section(side),
            )
        })
        .collect()
}

/// Every Space of the cargo zone, in the order the result documents.
pub(super) fn spaces(inp: &LiquidBulkInputs) -> Result<Vec<Space>, PyError> {
    let hull = InnerHull::of(inp);
    let pump_room = match inp.pump_arrangement {
        PumpArrangement::Deepwell => 0.0,
        PumpArrangement::PumpRoom { length_m } => length_m,
    };
    let zone = inp.zone_fwd_m - inp.zone_aft_m;
    let block = zone - pump_room - 2.0 * inp.cofferdam_m;
    // At least a metre for each of the tank positions, so none is a sliver.
    let n = f64::from(inp.tanks_in_line);
    if block < n {
        return Err(value_error(format!(
            "the cargo zone ({zone:.2} m) is too short for {} cofferdams of {:.2} m{} and {} tank(s) of at least 1 m",
            2,
            inp.cofferdam_m,
            if pump_room > 0.0 {
                format!(", a pump room of {pump_room:.2} m")
            } else {
                String::new()
            },
            inp.tanks_in_line
        )));
    }
    let centreline = inp.centreline_bulkhead;

    // x positions, aft to fore.
    let x_pump = (inp.zone_aft_m, inp.zone_aft_m + pump_room);
    let x_cof_aft = (x_pump.1, x_pump.1 + inp.cofferdam_m);
    let block_aft = x_cof_aft.1;
    let block_fwd = block_aft + block;
    let x_cof_fwd = (block_fwd, block_fwd + inp.cofferdam_m);

    let slop_len = inp.slop_share * block;
    let cargo_len = (block - slop_len) / n;
    if cargo_len < 1.0 {
        return Err(value_error(format!(
            "with a slop share of {:.3} the cargo tanks would be shorter than 1 m",
            inp.slop_share
        )));
    }

    let mut out: Vec<Space> = Vec::new();
    // Cargo tanks, from aft.
    let first = block_aft + slop_len;
    for i in 0..inp.tanks_in_line {
        let x0 = first + f64::from(i) * cargo_len;
        // The last tank ends exactly at the block's fore end.
        let x1 = if i + 1 == inp.tanks_in_line {
            block_fwd
        } else {
            x0 + cargo_len
        };
        out.extend(tank_position(
            "",
            SpaceKind::CargoTank,
            i + 1,
            (x0, x1),
            &hull,
            centreline,
        ));
    }
    if slop_len > 0.0 {
        out.extend(tank_position(
            "SL",
            SpaceKind::Slop,
            1,
            (block_aft, first),
            &hull,
            centreline,
        ));
    }
    // The double hull, as ballast, along the tank block.
    let xb = (block_aft, block_fwd);
    for side in [Side::Port, Side::Starboard] {
        out.push(space(
            format!("DB1{}", side.letter()),
            SpaceKind::DoubleBottomBallast,
            side,
            xb,
            rect(side, 0.0, hull.half_beam, 0.0, hull.bottom),
        ));
    }
    if inp.double_side_m > 0.0 {
        for side in [Side::Port, Side::Starboard] {
            out.push(space(
                format!("WB1{}", side.letter()),
                SpaceKind::WingBallast,
                side,
                xb,
                rect(
                    side,
                    hull.inner_half,
                    hull.half_beam,
                    hull.bottom,
                    hull.deck,
                ),
            ));
        }
    }
    out.push(space(
        "CD1".into(),
        SpaceKind::Cofferdam,
        Side::Centre,
        x_cof_aft,
        hull.end_section(),
    ));
    out.push(space(
        "CD2".into(),
        SpaceKind::Cofferdam,
        Side::Centre,
        x_cof_fwd,
        hull.end_section(),
    ));
    if pump_room > 0.0 {
        out.push(space(
            "PR1".into(),
            SpaceKind::PumpRoom,
            Side::Centre,
            x_pump,
            hull.end_section(),
        ));
    }
    // A Space with no volume (a double bottom of height 0) is not a Space.
    out.retain(|s| s.volume_m3() > 0.0);
    Ok(out)
}
