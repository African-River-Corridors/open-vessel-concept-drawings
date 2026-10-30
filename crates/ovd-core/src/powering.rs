//! Power and transmission (issue #353, spec 0007 "Power and transmission"): from the resistance
//! curve of each environment to the delivered, shaft and installed power, the service speed per
//! environment, the bollard pull and the diesel fuel.
//!
//! P_D at each speed comes from the Propulsor curve ([`crate::propulsor::performance`]). The shaft
//! power is P_S = P_D / eta_S, eta_S 0.99 for a shaft line (sourced, MAN) and 0.95 for an azimuth
//! unit (estimate: two gear sets). The Power Source loss then gives the power the engines, gensets or
//! battery must give, P_needed = P_S / eta_source:
//! - diesel-mechanical: gearbox 0.97 (estimate);
//! - diesel-electric and gas-fuelled gensets: the electric chain loss, 15 % at 1 MW or less, 9.4 % at
//!   5 MW, interpolated on log power and held flat outside (sourced, Wartsila, a vendor article);
//! - battery-electric: 0.92, today's drivetrain value (estimate);
//! - battery hybrid: diesel-mechanical in transit (the battery leg at 0.92).
//!
//! The Installed Power is P_B = P_needed at the Design Speed x the margin, unless typed. It is
//! sized on the highest P_needed over the environments (deep water and the design channel), each at
//! the Design Speed or the fastest speed it allows, if lower (the triage note on #353), and rounded
//! up to the engine table where engines drive. The service speed of an environment is the highest
//! speed where P_needed <= P_B / margin, within today's squat and channel caps.

use alloc::string::String;
use alloc::vec::Vec;

use libm::log10;

use crate::machinery::PowerSource;
use crate::propulsor::{PropulsorInputs, PropulsorType, bollard_pull, performance};

/// Shaft efficiency of a shaft line (sourced, MAN) and of an azimuth unit (estimate).
pub const ETA_SHAFT_LINE: f64 = 0.99;
pub const ETA_AZIMUTH: f64 = 0.95;
/// The diesel-mechanical gearbox (estimate).
pub const ETA_GEARBOX: f64 = 0.97;
/// The battery drivetrain, today's value (estimate).
pub const ETA_BATTERY: f64 = 0.92;
/// The electric chain loss at 1 MW or less and at 5 MW (Wartsila, a vendor article).
pub const CHAIN_LOSS_1MW: f64 = 0.15;
pub const CHAIN_LOSS_5MW: f64 = 0.094;
/// Diesel fuel on the engine load (g/kWh), today's value.
pub const DIESEL_G_PER_KWH: f64 = 195.0;

/// The least shaft axis submergence over the propeller diameter (Beveridge 1971, a guide): open and
/// ducted (issue #355).
pub const IMMERSION_OPEN: f64 = 0.76;
pub const IMMERSION_DUCTED: f64 = 0.71;

/// The shaft efficiency of a Propulsor type: a shaft line or a pump jet's shaft 0.99, an azimuth
/// unit or a Voith Schneider rotor (gears to the rotor) 0.95.
pub fn shaft_efficiency(t: PropulsorType) -> f64 {
    match t {
        PropulsorType::ShaftOpen | PropulsorType::ShaftNozzle | PropulsorType::PumpJet => {
            ETA_SHAFT_LINE
        }
        _ => ETA_AZIMUTH,
    }
}

/// The electric chain loss at the power `kw`: 15 % at 1 MW or less, 9.4 % at 5 MW, linear in log
/// power between, flat outside.
pub fn electric_chain_loss(kw: f64) -> f64 {
    if kw <= 1000.0 {
        return CHAIN_LOSS_1MW;
    }
    if kw >= 5000.0 {
        return CHAIN_LOSS_5MW;
    }
    let f = (log10(kw) - 3.0) / (log10(5000.0) - 3.0);
    CHAIN_LOSS_1MW + f * (CHAIN_LOSS_5MW - CHAIN_LOSS_1MW)
}

/// The efficiency from the shaft to the engines, gensets or battery, at the shaft power `kw`.
pub fn source_efficiency(source: PowerSource, kw: f64) -> f64 {
    match source {
        PowerSource::DieselMechanical | PowerSource::BatteryHybrid => ETA_GEARBOX,
        PowerSource::DieselElectric | PowerSource::GasFuelled => 1.0 - electric_chain_loss(kw),
        PowerSource::BatteryElectric => ETA_BATTERY,
    }
}

/// The rating ranges of the engine table (kW per engine, machinery), lowest to highest: an engine is
/// offered across its range, so only a power in a gap between ranges rounds up.
const ENGINE_RANGES: [(f64, f64); 6] = [
    (955.0, 1903.0),
    (1491.0, 2013.0),
    (1492.0, 2525.0),
    (1740.0, 2580.0),
    (2560.0, 3200.0),
    (3400.0, 4400.0),
];

/// A power per engine rounded up to the engine table: unchanged inside a rating range or below the
/// table (the small-engine rule), the start of the next range in a gap, unchanged above the table.
pub fn round_up_to_engine_table(kw: f64) -> f64 {
    if kw < ENGINE_RANGES[0].0 || ENGINE_RANGES.iter().any(|(lo, hi)| kw >= *lo && kw <= *hi) {
        return kw;
    }
    ENGINE_RANGES
        .iter()
        .map(|(lo, _)| *lo)
        .find(|lo| *lo >= kw)
        .unwrap_or(kw)
}

/// One speed of an environment's resistance curve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurvePoint {
    /// Speed over ground, and through the water (kn).
    pub speed_kn: f64,
    pub speed_through_water_kn: f64,
    pub resistance_kn: f64,
    /// Squat, not power, forbids this speed.
    pub squat_limited: bool,
}

/// An environment's resistance curve, capped as today's resistance method caps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EnvironmentCurve {
    pub name: String,
    pub points: Vec<CurvePoint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoweringInputs {
    pub propulsor: PropulsorInputs,
    pub power_source: PowerSource,
    pub design_speed_kn: f64,
    pub margin: f64,
    /// A typed Installed Power (kW); `None`: sized.
    pub installed_power_kw: Option<f64>,
    /// The length of one trip on one charge (km), for a battery Power Source; `None`: no route.
    pub trip_length_km: Option<f64>,
}

/// The battery energy over the energy per trip: a 1.25 reserve at 80 % usable (estimates, spec 0007
/// and the `battery_energy_kwh` basis).
pub const BATTERY_RESERVE: f64 = 1.25;
pub const BATTERY_USABLE: f64 = 0.8;

/// The power at one speed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowerPoint {
    pub speed_kn: f64,
    pub delivered_kw: f64,
    pub shaft_kw: f64,
    /// The power the engines, gensets or battery give.
    pub needed_kw: f64,
    pub eta_d: f64,
    pub squat_limited: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnvironmentPower {
    pub name: String,
    pub points: Vec<PowerPoint>,
    /// The highest speed where P_needed <= P_B / margin within the caps (kn), and P_needed there.
    pub service_speed_kn: f64,
    pub needed_at_service_kw: f64,
    /// Diesel fuel at the service speed (kg/h), for a Power Source that burns diesel in transit.
    pub fuel_kg_per_h: f64,
    /// The battery energy of one trip at the service speed (kWh), with a battery Power Source and a
    /// trip length.
    pub energy_per_trip_kwh: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Powering {
    pub installed_power_kw: f64,
    /// The Installed Power was sized here (not typed).
    pub sized: bool,
    pub environments: Vec<EnvironmentPower>,
    /// The bollard pull of all units at the Installed Power (t).
    pub bollard_pull_t: f64,
    /// The battery energy the trip asks for (kWh): the largest energy per trip of the environments
    /// x the reserve / the usable share. `None` without a battery Power Source or a trip length.
    pub battery_energy_kwh: Option<f64>,
    /// The blade load of one unit at the Installed Power (kW per m^2 of disc), and the limit of its
    /// type (issue #355).
    pub blade_load_kw_m2: f64,
    pub blade_load_limit_kw_m2: f64,
}

/// The power points of a curve.
pub fn power_points(inp: &PoweringInputs, curve: &EnvironmentCurve) -> Vec<PowerPoint> {
    let speeds: Vec<f64> = curve
        .points
        .iter()
        .map(|p| p.speed_through_water_kn)
        .collect();
    let r: Vec<f64> = curve.points.iter().map(|p| p.resistance_kn).collect();
    let pc = performance(&inp.propulsor, &speeds, &r);
    let eta_s = shaft_efficiency(inp.propulsor.propulsor_type);
    curve
        .points
        .iter()
        .zip(&pc.points)
        .map(|(c, p)| {
            let shaft = p.delivered_power_kw / eta_s;
            PowerPoint {
                speed_kn: c.speed_kn,
                delivered_kw: p.delivered_power_kw,
                shaft_kw: shaft,
                needed_kw: shaft / source_efficiency(inp.power_source, shaft),
                eta_d: p.eta_d,
                squat_limited: c.squat_limited,
            }
        })
        .collect()
}

/// The fastest speed of the points where P_needed <= `available_kw` and squat allows, interpolated
/// between the last such point and the next one; (speed, P_needed there).
fn fastest(points: &[PowerPoint], available_kw: f64) -> (f64, f64) {
    let mut best = (0.0, 0.0);
    for (i, p) in points.iter().enumerate() {
        if p.squat_limited || p.needed_kw > available_kw {
            if i > 0 && !p.squat_limited {
                let a = &points[i - 1];
                if a.needed_kw <= available_kw && p.needed_kw > a.needed_kw {
                    let f = (available_kw - a.needed_kw) / (p.needed_kw - a.needed_kw);
                    best = (a.speed_kn + f * (p.speed_kn - a.speed_kn), available_kw);
                }
            }
            break;
        }
        best = (p.speed_kn, p.needed_kw);
    }
    best
}

/// P_needed at the speed `v` of the points, interpolated; at the last allowed point if `v` is
/// beyond the points the caps allow.
pub fn needed_at(points: &[PowerPoint], v: f64) -> f64 {
    let allowed: Vec<&PowerPoint> = points.iter().take_while(|p| !p.squat_limited).collect();
    let Some(last) = allowed.last() else {
        return 0.0;
    };
    if v >= last.speed_kn {
        return last.needed_kw;
    }
    for w in allowed.windows(2) {
        if v <= w[1].speed_kn {
            let f = (v - w[0].speed_kn) / (w[1].speed_kn - w[0].speed_kn);
            return w[0].needed_kw + f * (w[1].needed_kw - w[0].needed_kw);
        }
    }
    allowed[0].needed_kw
}

/// Size or take the Installed Power, then the service speed of each environment.
pub fn compute(inp: &PoweringInputs, curves: &[EnvironmentCurve]) -> Powering {
    let envs: Vec<(String, Vec<PowerPoint>)> = curves
        .iter()
        .map(|c| (c.name.clone(), power_points(inp, c)))
        .collect();
    let n = f64::from(inp.propulsor.count.max(1));
    let (installed, sized) = match inp.installed_power_kw.filter(|p| *p > 0.0) {
        Some(p) => (p, false),
        None => {
            let need = envs
                .iter()
                .map(|(_, pts)| needed_at(pts, inp.design_speed_kn))
                .fold(0.0, f64::max);
            let p = need * inp.margin;
            let p = match inp.power_source {
                PowerSource::DieselMechanical
                | PowerSource::BatteryHybrid
                | PowerSource::DieselElectric => n * round_up_to_engine_table(p / n),
                _ => p,
            };
            (p, true)
        }
    };
    let available = installed / inp.margin;
    let burns_diesel = matches!(
        inp.power_source,
        PowerSource::DieselMechanical | PowerSource::DieselElectric | PowerSource::BatteryHybrid
    );
    let on_battery = matches!(
        inp.power_source,
        PowerSource::BatteryElectric | PowerSource::BatteryHybrid
    );
    let trip = inp.trip_length_km.filter(|l| *l > 0.0 && on_battery);
    let environments: Vec<EnvironmentPower> = envs
        .into_iter()
        .map(|(name, points)| {
            let (v, need) = fastest(&points, available);
            // A hybrid's battery leg runs at the battery drivetrain, not the gearbox.
            let battery_kw = match inp.power_source {
                PowerSource::BatteryHybrid => need * ETA_GEARBOX / ETA_BATTERY,
                _ => need,
            };
            let energy_per_trip_kwh = trip
                .filter(|_| v > 0.0)
                .map(|l| battery_kw * l / (v * 1.852));
            EnvironmentPower {
                name,
                points,
                service_speed_kn: v,
                needed_at_service_kw: need,
                fuel_kg_per_h: if burns_diesel {
                    DIESEL_G_PER_KWH * need / 1000.0
                } else {
                    0.0
                },
                energy_per_trip_kwh,
            }
        })
        .collect();
    let battery_energy_kwh = environments
        .iter()
        .filter_map(|e| e.energy_per_trip_kwh)
        .reduce(f64::max)
        .map(|e| e * BATTERY_RESERVE / BATTERY_USABLE);
    // At the bollard each unit takes its share of the Installed Power through the transmission.
    let eta_s = shaft_efficiency(inp.propulsor.propulsor_type);
    let pd_unit = installed / n * source_efficiency(inp.power_source, installed) * eta_s;
    let (blade_load_kw_m2, blade_load_limit_kw_m2) =
        crate::propulsor::blade_load(&inp.propulsor, pd_unit);
    Powering {
        installed_power_kw: installed,
        sized,
        environments,
        bollard_pull_t: bollard_pull(&inp.propulsor, pd_unit),
        battery_energy_kwh,
        blade_load_kw_m2,
        blade_load_limit_kw_m2,
    }
}
