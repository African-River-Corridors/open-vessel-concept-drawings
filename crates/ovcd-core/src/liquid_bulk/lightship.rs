//! The tank-barge lightship: a k factor times L x B x D for the hull, and outfit items for the pumps,
//! the cargo piping, the vents and gauging, and the heating (spec 0006 section 4).
//!
//! k is fitted on a declared subset of the tank-barge records of `calibration/cases.yaml` by
//! `tools/fit_tank_lightship.py`; the rest are held out. The fit writes
//! `calibration/tank_lightship.yaml`, and a test keeps the constants below equal to it. The lightship
//! of a record is implied from its deadweight, so k is an Estimate. The outfit coefficients are
//! Estimates too: no published weight was found (`docs/research/crude-oil/03-design-practice.md` 4.1).

use alloc::string::String;
use alloc::vec::Vec;

use super::LiquidBulkInputs;
use crate::loading_conditions::Item;

/// k, t per m³ of L x B x D (the hull and the baseline outfit). Estimate; fitted by
/// `tools/fit_tank_lightship.py` on 15 records, see `calibration/tank_lightship.yaml`.
pub const TANK_LIGHTSHIP_K_T_PER_M3: f64 = 0.1141;
/// The cargo pumps, t per m³ of tank volume. Estimate: no published weight.
pub const PUMPS_T_PER_M3: f64 = 0.004;
/// Cargo piping, valves and manifolds, t per m³ of tank volume. Estimate: no published weight.
pub const PIPING_T_PER_M3: f64 = 0.010;
/// Vents and gauging, t per m³ of tank volume. Estimate: no published weight.
pub const VENTS_GAUGING_T_PER_M3: f64 = 0.002;
/// Heater and coils, t per kW installed. Estimate: no published weight.
pub const HEATING_T_PER_KW: f64 = 0.006;

/// The KG of the hull items: half the depth, as for the dry bulk hull.
const HULL_KG_RATIO: f64 = 0.5;

/// The outfit, in t, for `volume_m3` of tank volume and `heating_kw` of installed heating.
pub fn outfit_t(volume_m3: f64, heating_kw: f64) -> f64 {
    volume_m3 * (PUMPS_T_PER_M3 + PIPING_T_PER_M3 + VENTS_GAUGING_T_PER_M3)
        + heating_kw * HEATING_T_PER_KW
}

/// The lightship (t) and its items, for `tank_volume_m3` of cargo and slop tank volume (gross).
/// The outfit sits at the deck, at the middle of the cargo zone: the pumps and the piping are on deck.
pub(super) fn compute(inp: &LiquidBulkInputs, tank_volume_m3: f64) -> (f64, Vec<(String, Item)>) {
    let hull_t = TANK_LIGHTSHIP_K_T_PER_M3 * inp.loa_m * inp.beam_m * inp.depth_m;
    let heating_kw = inp.heating_kw_per_m3 * tank_volume_m3;
    let lcg_zone = 0.5 * (inp.zone_aft_m + inp.zone_fwd_m);
    let mut items: Vec<(String, Item)> = Vec::new();
    items.push((
        "hull".into(),
        Item {
            mass_t: hull_t,
            kcg_m: HULL_KG_RATIO * inp.depth_m,
            lcg_m: 0.5 * inp.loa_m,
        },
    ));
    let outfit = [
        ("pumps", tank_volume_m3 * PUMPS_T_PER_M3),
        ("piping", tank_volume_m3 * PIPING_T_PER_M3),
        ("vents and gauging", tank_volume_m3 * VENTS_GAUGING_T_PER_M3),
        ("heating", heating_kw * HEATING_T_PER_KW),
    ];
    for (name, mass_t) in outfit {
        if mass_t > 0.0 {
            items.push((
                name.into(),
                Item {
                    mass_t,
                    kcg_m: inp.depth_m,
                    lcg_m: lcg_zone,
                },
            ));
        }
    }
    (items.iter().map(|(_, i)| i.mass_t).sum(), items)
}
