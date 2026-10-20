//! Electric-barge energy and service speed from hull resistance in a confined channel. Port of
//! `core/electric_energy.py`.
//!
//! Couples [`crate::resistance::compute`] to the electric drivetrain: shaft power from
//! resistance, battery power from shaft power, energy per kilometre from battery power. The
//! Python file holds the engineering notes. Nothing is fixed in the port.
//!
//! The speed grid is the resistance module's (repeated addition, then `round(s, 1)`), so a step
//! of 0.25 gives 2.0, 2.2, 2.5, 2.8. The service speed is rounded with `round(v, 2)`.
//! `Err` carries the Python exception the same input raises: a resistance error comes out
//! unchanged, and a zero drivetrain efficiency raises `ZeroDivisionError`.

use crate::aft_arrangement::PyException;
use crate::checks::{Check, all_ok};
use crate::pyfmt::round_to;
use crate::resistance::{self, EnvironmentInputs, ResistanceInputs, TugInputs};
use crate::text;
use crate::text::Arg;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

pub const KN_TO_KMH: f64 = 1.852;

#[derive(Debug, Clone, PartialEq)]
pub struct EnergyInputs {
    // --- barge (beam, shape) ---
    pub beam: f64,
    pub loa: f64,
    pub lwl: f64,
    pub draft: f64,
    pub block_coefficient: f64,

    // --- confined design channel (width, depth) ---
    pub channel_depth_m: f64,
    pub channel_bottom_width_m: f64,
    pub channel_top_width_m: f64,
    pub current_knots: f64,
    pub fresh_water: bool,
    /// Add an open-sea environment for comparison.
    pub also_open_sea: bool,

    // --- electric drivetrain ---
    /// P_D = R x V / PC (hull and propeller).
    pub propulsive_coefficient: f64,
    /// Battery to shaft (motor, inverter and line).
    pub drivetrain_efficiency: f64,

    // --- service speed and route ---
    pub target_service_speed_knots: f64,
    /// Cap the service speed at this times the Schijf limit.
    pub service_speed_safety_fraction: f64,
    /// Installed power = battery power at service speed x margin.
    pub power_margin: f64,
    pub one_way_km: f64,
    pub trips_per_year: f64,

    // --- speed sweep for the curves ---
    pub speed_min_knots: f64,
    pub speed_max_knots: f64,
    pub speed_step_knots: f64,
}

impl Default for EnergyInputs {
    fn default() -> Self {
        Self {
            beam: 12.0,
            loa: 80.0,
            lwl: 78.0,
            draft: 3.0,
            block_coefficient: 0.85,
            channel_depth_m: 4.0,
            channel_bottom_width_m: 34.0,
            channel_top_width_m: 58.0,
            current_knots: 0.5,
            fresh_water: true,
            also_open_sea: true,
            propulsive_coefficient: 0.45,
            drivetrain_efficiency: 0.92,
            target_service_speed_knots: 6.0,
            service_speed_safety_fraction: 0.9,
            power_margin: 1.20,
            one_way_km: 102.0,
            trips_per_year: 300.0,
            speed_min_knots: 2.0,
            speed_max_knots: 10.0,
            speed_step_knots: 0.25,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnergyPoint {
    pub speed_knots: f64,
    pub speed_kmh: f64,
    pub froude_depth: f64,
    pub resistance_kn: f64,
    /// Delivered P_D.
    pub shaft_power_kw: f64,
    /// P_D / drivetrain efficiency.
    pub battery_power_kw: f64,
    pub energy_kwh_per_km: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnergyEnvironment {
    pub name: String,
    pub limiting_speed_knots: Option<f64>,
    pub service_speed_knots: f64,
    pub blockage_ratio: f64,
    pub curve: Vec<EnergyPoint>,
    // at service speed (interpolated on the curve):
    pub resistance_kn: f64,
    pub shaft_power_kw: f64,
    pub battery_power_kw: f64,
    pub energy_kwh_per_km: f64,
    /// One way over `one_way_km`.
    pub energy_per_trip_kwh: f64,
    /// Round trips x `trips_per_year`.
    pub energy_per_year_mwh: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnergyResult {
    pub inputs: EnergyInputs,
    pub installed_power_kw: f64,
    pub environments: Vec<EnergyEnvironment>,
    pub checks: Vec<Check>,
}

impl EnergyResult {
    pub fn ok(&self) -> bool {
        all_ok(&self.checks)
    }

    /// The most confined environment (lowest service speed); the first of equal ones.
    pub fn binding(&self) -> Option<&EnergyEnvironment> {
        binding_of(&self.environments)
    }
}

fn binding_of(envs: &[EnergyEnvironment]) -> Option<&EnergyEnvironment> {
    let mut best: Option<&EnergyEnvironment> = None;
    for e in envs {
        match best {
            Some(b) if e.service_speed_knots < b.service_speed_knots => best = Some(e),
            Some(_) => {}
            None => best = Some(e),
        }
    }
    best
}

/// Python `min(a, b)`: `a` unless `b` is strictly smaller.
fn py_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// Linear interpolation of a curve attribute at speed `v_knots`.
fn interp(curve: &[EnergyPoint], v_knots: f64, attr: fn(&EnergyPoint) -> f64) -> f64 {
    let (Some(first), Some(last)) = (curve.first(), curve.last()) else {
        return 0.0;
    };
    if v_knots <= first.speed_knots {
        return attr(first);
    }
    if v_knots >= last.speed_knots {
        return attr(last);
    }
    for w in curve.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        if a.speed_knots <= v_knots && v_knots <= b.speed_knots {
            // `(b - a or 1.0)`: 1.0 when the gap is zero.
            let gap = b.speed_knots - a.speed_knots;
            let t = (v_knots - a.speed_knots) / if gap != 0.0 { gap } else { 1.0 };
            return attr(a) + t * (attr(b) - attr(a));
        }
    }
    attr(last)
}

pub fn compute(inp: &EnergyInputs) -> Result<EnergyResult, PyException> {
    // Build the environments and run the resistance model. Installed engine power is set huge so
    // resistance does not cap the sweep: the service speed is chosen here.
    let mut envs = vec![EnvironmentInputs {
        name: "Design channel".to_string(),
        water_depth: inp.channel_depth_m,
        channel_width_bottom: inp.channel_bottom_width_m,
        channel_width_surface: inp.channel_top_width_m,
        current_speed_knots: inp.current_knots,
        water_density: if inp.fresh_water { 1.0 } else { 1.025 },
        ..EnvironmentInputs::default()
    }];
    if inp.also_open_sea {
        envs.push(EnvironmentInputs {
            name: "Open sea".to_string(),
            water_depth: 50.0,
            channel_width_bottom: 1000.0,
            channel_width_surface: 1000.0,
            is_deep_water: true,
            water_density: 1.025,
            ..EnvironmentInputs::default()
        });
    }

    let rin = ResistanceInputs {
        loa: inp.loa,
        lwl: inp.lwl,
        beam: inp.beam,
        draft: inp.draft,
        block_coefficient: inp.block_coefficient,
        environments: envs,
        speed_min_knots: inp.speed_min_knots,
        speed_max_knots: inp.speed_max_knots,
        speed_step_knots: inp.speed_step_knots,
        tug: TugInputs {
            engine_power_kw: 1e12,
            propulsive_efficiency: inp.propulsive_coefficient,
            ..TugInputs::default()
        },
        ..ResistanceInputs::default()
    };
    let rres = resistance::compute(&rin)?;

    let mut energy_envs: Vec<EnergyEnvironment> = Vec::new();
    for e in &rres.environments {
        let mut curve: Vec<EnergyPoint> = Vec::new();
        for p in &e.speed_curve {
            let v_kmh = p.speed_knots * KN_TO_KMH;
            if inp.drivetrain_efficiency == 0.0 {
                return Err(PyException::ZeroDivisionError(
                    "float division by zero".to_string(),
                ));
            }
            let p_bat = p.delivered_power_kw / inp.drivetrain_efficiency;
            let e_km = if v_kmh > 0.0 { p_bat / v_kmh } else { 0.0 };
            curve.push(EnergyPoint {
                speed_knots: p.speed_knots,
                speed_kmh: v_kmh,
                froude_depth: p.froude_depth,
                resistance_kn: p.resistance_total_kn,
                shaft_power_kw: p.delivered_power_kw,
                battery_power_kw: p_bat,
                energy_kwh_per_km: e_km,
            });
        }

        // Service speed: the target cruise, capped safely below the Schijf limit.
        let mut v_service = inp.target_service_speed_knots;
        // `if x and x > 0`: None and 0.0 are falsy, NaN is truthy but not > 0.
        if let Some(l) = e.limiting_speed_knots
            && l > 0.0
        {
            v_service = py_min(v_service, inp.service_speed_safety_fraction * l);
        }
        if let Some(last) = curve.last() {
            v_service = py_min(v_service, last.speed_knots);
        }

        let e_km = interp(&curve, v_service, |p| p.energy_kwh_per_km);
        let trip_kwh = e_km * inp.one_way_km;
        let year_mwh = trip_kwh * 2.0 * inp.trips_per_year / 1000.0;
        energy_envs.push(EnergyEnvironment {
            name: e.name.clone(),
            limiting_speed_knots: e.limiting_speed_knots,
            service_speed_knots: round_to(v_service, 2),
            blockage_ratio: e.blockage_ratio,
            resistance_kn: interp(&curve, v_service, |p| p.resistance_kn),
            shaft_power_kw: interp(&curve, v_service, |p| p.shaft_power_kw),
            battery_power_kw: interp(&curve, v_service, |p| p.battery_power_kw),
            curve,
            energy_kwh_per_km: e_km,
            energy_per_trip_kwh: trip_kwh,
            energy_per_year_mwh: year_mwh,
        });
    }

    let binding = binding_of(&energy_envs);
    let installed = binding.map_or(0.0, |b| b.battery_power_kw * inp.power_margin);

    let mut checks: Vec<Check> = Vec::new();
    if let Some(b) = binding {
        // `if limiting:` is false for None and for 0.0.
        let message = match b.limiting_speed_knots {
            Some(l) if l != 0.0 => text!(
                "check-service-below-schijf",
                "service {service} kn stays below the Schijf limit {limit} kn",
                service = Arg::fixed(b.service_speed_knots, 1),
                limit = Arg::fixed(l, 1),
            ),
            _ => text!(
                "check-service-below-schijf-open-water",
                "no confined limit (open water)"
            ),
        };
        checks.push(Check::new(
            "service_below_schijf",
            b.limiting_speed_knots
                .is_none_or(|l| b.service_speed_knots <= l + 1e-6),
            message,
        ));
        checks.push(Check::new(
            "energy_positive",
            b.energy_kwh_per_km > 0.0,
            text!(
                "check-energy-positive",
                "energy {energy} kWh/km at service speed",
                energy = Arg::fixed(b.energy_kwh_per_km, 1),
            ),
        ));
    }
    checks.push(Check::with_severity(
        "resistance_coeffs_first_order",
        true,
        text!(
            "check-resistance-coeffs-first-order",
            "confined-water resistance (residuary/Lackenby/blockage) is a first-order estimate \
             (RS1) \u{2014} ITTC-1957 friction is standard; anchor with a model test/CFD before class use."
        ),

        "warning",
    ));

    Ok(EnergyResult {
        inputs: inp.clone(),
        installed_power_kw: installed,
        environments: energy_envs,
        checks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(name: &str, service: f64) -> EnergyEnvironment {
        EnergyEnvironment {
            name: name.to_string(),
            limiting_speed_knots: None,
            service_speed_knots: service,
            blockage_ratio: 0.0,
            curve: vec![],
            resistance_kn: 0.0,
            shaft_power_kw: 0.0,
            battery_power_kw: 0.0,
            energy_kwh_per_km: 0.0,
            energy_per_trip_kwh: 0.0,
            energy_per_year_mwh: 0.0,
        }
    }

    #[test]
    fn binding_is_the_lowest_service_speed_and_the_first_of_equals() {
        assert!(binding_of(&[]).is_none());
        let envs = [env("a", 8.0), env("b", 6.0), env("c", 6.0), env("d", 9.0)];
        assert_eq!(binding_of(&envs).unwrap().name, "b");
    }
}
