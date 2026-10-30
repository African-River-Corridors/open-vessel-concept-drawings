//! The capital budget of a tank barge: the cargo tank items, as lump sums (issue #334, spec 0006).
//!
//! The quantities come from the [`LiquidBulkResult`] and nothing else: the hull steel is the lightship's
//! hull item, the coating area is the inside surface of the cargo and slop tanks, and the pumps, vents and
//! gauging are counted by tank. The rates are inputs (example rates, not market quotes). Lump sums come
//! first; bill of quantities lines belong to a later phase. The yard margin, delivery and the powering
//! items are the ones every containment shares ([`barge_cost::assemble`]).

use alloc::vec::Vec;

use super::{LiquidBulkResult, lightship};
use crate::barge_cost::{self, BargeCostInputs, BargeCostRates, BargeCostResult, ComputeError};
use crate::checks::Check;
use crate::compartments::SpaceKind;
use crate::text;

/// What the cargo tank items are priced on.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CostQuantities {
    /// The hull steel (t): the hull item of the lightship. The tank boundaries are the hull's, so the
    /// tank steel is in it.
    pub hull_steel_t: f64,
    /// The inside surface of the cargo and slop tanks (m²): the coating area.
    pub coating_area_m2: f64,
    /// The cargo tank Spaces (a port and a starboard tank are two).
    pub cargo_tank_count: u32,
    /// The installed heating (kW).
    pub heating_kw: f64,
}

/// The rates of the cargo tank items. Each is an example rate that the user replaces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidCostRates {
    /// Coating, per m² of tank surface.
    pub tank_coating_rate: f64,
    /// Cargo pumps, per pump (one for each cargo tank).
    pub cargo_pump_lump: f64,
    /// The cargo piping system, per vessel.
    pub cargo_piping_lump: f64,
    /// Cargo heating, per kW installed.
    pub heating_rate_per_kw: f64,
    /// Vents and gauging, per cargo tank.
    pub vents_gauging_lump: f64,
}

/// The inside surface of a prism Space: its section's perimeter along its length, and its two ends.
fn surface_m2(s: &crate::compartments::Space) -> f64 {
    let n = s.section().len();
    let perimeter = (0..n).fold(0.0, |a, i| {
        let (y0, z0) = s.section()[i];
        let (y1, z1) = s.section()[(i + 1) % n];
        a + libm::sqrt((y1 - y0) * (y1 - y0) + (z1 - z0) * (z1 - z0))
    });
    let length = s.length();
    let area = if length > 0.0 {
        s.volume_m3() / length
    } else {
        0.0
    };
    perimeter * length + 2.0 * area
}

/// The quantities of a hull's result.
pub fn quantities(r: &LiquidBulkResult) -> CostQuantities {
    let item_t = |name: &str| {
        r.lightship_items
            .iter()
            .find(|(n, _)| n == name)
            .map_or(0.0, |(_, i)| i.mass_t)
    };
    CostQuantities {
        hull_steel_t: item_t("hull"),
        coating_area_m2: r
            .spaces
            .iter()
            .filter(|s| matches!(s.kind, SpaceKind::CargoTank | SpaceKind::Slop))
            .fold(0.0, |a, s| a + surface_m2(s)),
        cargo_tank_count: r.of_kind(SpaceKind::CargoTank).count() as u32,
        heating_kw: item_t("heating") / lightship::HEATING_T_PER_KW,
    }
}

/// The budget of a tank barge: the cargo tank items from `q` and `lr`, then the items every containment
/// shares, from `inp` (the tug and the powering) and `rt` (the hull steel rate, the paint and anodes
/// lump, the yard margin and the delivery).
pub fn compute(
    q: &CostQuantities,
    lr: &LiquidCostRates,
    inp: &BargeCostInputs,
    rt: &BargeCostRates,
    tug: bool,
) -> Result<BargeCostResult, ComputeError> {
    let tanks = f64::from(q.cargo_tank_count);
    let barge: Vec<(&'static str, f64)> = alloc::vec![
        ("hull_steel", q.hull_steel_t * rt.hull_steel_rate),
        ("tank_coating", q.coating_area_m2 * lr.tank_coating_rate),
        ("cargo_pumps", tanks * lr.cargo_pump_lump),
        ("cargo_piping", lr.cargo_piping_lump),
        ("cargo_heating", q.heating_kw * lr.heating_rate_per_kw),
        ("vents_gauging", tanks * lr.vents_gauging_lump),
        ("paint_anodes_misc", rt.paint_anodes_misc),
    ];
    let present = q.hull_steel_t > 0.0
        && q.coating_area_m2 > 0.0
        && q.cargo_tank_count > 0
        && q.heating_kw >= 0.0
        && inp.tug_hull_steel_t >= 0.0
        && inp.battery_kwh >= 0.0
        && inp.propulsion_kw >= 0.0;
    let checks = alloc::vec![
        Check::new(
            "quantities_present",
            present,
            text!(
                "check-tank-quantities-present",
                "cargo tank quantities present"
            ),
        ),
        Check::with_severity(
            "tank_items_are_lump_sums",
            false,
            text!(
                "check-tank-items-are-lump-sums",
                "the cargo tank items are lump sums from the tank layout, not a bill of quantities"
            ),
            "warning",
        ),
        Check::with_severity(
            "fabrication_rates_are_estimates",
            false,
            text!(
                "check-fabrication-rates-are-estimates",
                "fabrication/supply rates are user-supplied estimates; replace them with \
                 yard quotes before relying on the total"
            ),
            "warning",
        ),
    ];
    barge_cost::assemble(barge, inp, rt, tug, checks)
}
