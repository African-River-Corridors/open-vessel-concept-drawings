//! The Propulsor model (issue #351, spec 0007 "Propulsor model"): an actuator disc per unit.
//!
//! At ship speed V each of the n units gives the thrust T = R / (n (1 - t)), at the advance speed
//! V_A = V (1 - w), on the disc area A = pi D^2 / 4. The thrust loading is C_T = T / (rho A V_A^2 / 2)
//! and the ideal efficiency of the open disc eta_i = 2 / (1 + sqrt(1 + C_T)). In a duct the duct
//! carries the share 1 - tau of the thrust, so the propeller sees tau C_T and eta_i = 2 / (1 +
//! sqrt(1 + tau C_T)), tau 0.75 (estimate: duct lift "can be 30 %" of the propeller's thrust,
//! Rotteveel 2019 sec. 2.2.2). The propulsive efficiency is eta_D = kappa x eta_i x eta_R x eta_H, with
//! eta_R 0.98 and eta_H = (1 - t) / (1 - w) (MAN, "Basic principles of ship propulsion"); the
//! spec takes t = w, so eta_H = 1. kappa is the factor of the Propulsor type.
//!
//! At the bollard (V = 0) the disc's momentum limit is T_0 = (2 rho A)^(1/3) P_D^(2/3) per unit; the
//! factor kappa_b of the type takes in the duct's extra thrust and the losses. With rho 1,025 kg/m^3
//! and kappa_b 0.77 this is the Wartsila open-propeller estimate 0.9 x (D P_D)^(2/3) kN, D in m and
//! P_D in kW (derived).

use alloc::vec::Vec;

use libm::{cbrt, sqrt};

/// The Propulsor types other than `unspecified` (spec 0007).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropulsorType {
    AzimuthOpen,
    AzimuthDucted,
    ShaftOpen,
    ShaftNozzle,
    VoithSchneider,
    PumpJet,
}

impl PropulsorType {
    pub const ALL: [Self; 6] = [
        Self::AzimuthOpen,
        Self::AzimuthDucted,
        Self::ShaftOpen,
        Self::ShaftNozzle,
        Self::VoithSchneider,
        Self::PumpJet,
    ];

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.key() == key)
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::AzimuthOpen => "azimuth_open",
            Self::AzimuthDucted => "azimuth_ducted",
            Self::ShaftOpen => "shaft_open",
            Self::ShaftNozzle => "shaft_nozzle",
            Self::VoithSchneider => "voith_schneider",
            Self::PumpJet => "pump_jet",
        }
    }

    /// A propeller in a duct or nozzle: the ducted disc, and the ducted blade-load limit.
    pub fn ducted(self) -> bool {
        matches!(self, Self::AzimuthDucted | Self::ShaftNozzle)
    }
}

/// The propeller's share of the thrust in a duct (estimate, spec 0007).
pub const DUCT_PROPELLER_SHARE: f64 = 0.75;
/// The relative rotative efficiency, twin screw (sourced: MAN, about 0.98).
pub const ETA_R: f64 = 0.98;
/// The blade-load limits of the Veth rating basis, kW per m^2 of disc (sourced: Veth azimuth
/// thrusters p.83): open and ducted.
pub const BLADE_LOAD_OPEN_KW_M2: f64 = 400.0;
pub const BLADE_LOAD_DUCTED_KW_M2: f64 = 500.0;
/// The gravity the spec turns kN into tonnes with.
pub const G: f64 = 9.81;

/// The largest azimuth unit of the maker range (m): the Schottel SRP 800, 4,200 mm (SCHOTTEL Product
/// Guide 1125, the SRP table). It caps the default propeller diameter (issue #352).
pub const AZIMUTH_MAX_DIAMETER_M: f64 = 4.2;

impl PropulsorType {
    /// The largest unit of the type's maker range (m), where a range is known: azimuth units only.
    pub fn max_unit_diameter_m(self) -> Option<f64> {
        match self {
            Self::AzimuthOpen | Self::AzimuthDucted => Some(AZIMUTH_MAX_DIAMETER_M),
            _ => None,
        }
    }
}

/// One set of identical propulsors.
#[derive(Debug, Clone, PartialEq)]
pub struct PropulsorInputs {
    pub propulsor_type: PropulsorType,
    pub count: u32,
    /// The propeller diameter, or a pump jet's inlet diameter (m).
    pub diameter_m: f64,
    /// The wake fraction w; the thrust deduction t is taken equal to it (spec 0007).
    pub wake: f64,
    /// kappa_type, the efficiency factor of the type.
    pub kappa: f64,
    /// kappa_b,type, the bollard pull factor of the type.
    pub kappa_b: f64,
    /// Water density (kg/m^3).
    pub rho_kg_m3: f64,
}

impl PropulsorInputs {
    /// The disc area of one unit (m^2).
    pub fn disc_area_m2(&self) -> f64 {
        core::f64::consts::PI * self.diameter_m * self.diameter_m / 4.0
    }
}

/// The propulsor at one ship speed.
#[derive(Debug, Clone, PartialEq)]
pub struct PropulsorPoint {
    pub speed_kn: f64,
    /// The thrust per unit (kN) and the advance speed (m/s).
    pub thrust_per_unit_kn: f64,
    pub advance_speed_m_s: f64,
    pub thrust_loading: f64,
    pub eta_i: f64,
    pub eta_d: f64,
    /// The delivered power of all units together (kW).
    pub delivered_power_kw: f64,
}

/// The propulsor over a speed range.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PropulsorCurve {
    pub points: Vec<PropulsorPoint>,
}

/// The ideal efficiency of the disc at thrust loading `c_t`: the open disc, or the propeller of a
/// ducted disc, which carries the share tau of the thrust.
pub fn ideal_efficiency(c_t: f64, ducted: bool) -> f64 {
    let load = if ducted {
        DUCT_PROPELLER_SHARE * c_t
    } else {
        c_t
    };
    2.0 / (1.0 + sqrt(1.0 + load.max(0.0)))
}

const MS_PER_KN: f64 = 1852.0 / 3600.0;

/// The propulsor at each speed (kn), for the total resistance there (kN). A speed of 0 or less, or no
/// resistance, gives no thrust and no power.
pub fn performance(
    inp: &PropulsorInputs,
    speeds_kn: &[f64],
    resistance_kn: &[f64],
) -> PropulsorCurve {
    let n = f64::from(inp.count.max(1));
    let area = inp.disc_area_m2();
    // The thrust deduction equals the wake (spec 0007), so eta_H = (1 - t) / (1 - w) = 1.
    let t = inp.wake;
    let eta_h = 1.0;
    let points = speeds_kn
        .iter()
        .zip(resistance_kn)
        .map(|(&v_kn, &r_kn)| {
            let v = v_kn * MS_PER_KN;
            let thrust_kn = r_kn / (n * (1.0 - t));
            let va = v * (1.0 - inp.wake);
            let dynamic = 0.5 * inp.rho_kg_m3 * area * va * va;
            let (c_t, eta_i, eta_d, pd) = if v > 0.0 && r_kn > 0.0 && dynamic > 0.0 {
                let c_t = thrust_kn * 1000.0 / dynamic;
                let eta_i = ideal_efficiency(c_t, inp.propulsor_type.ducted());
                let eta_d = inp.kappa * eta_i * ETA_R * eta_h;
                (c_t, eta_i, eta_d, r_kn * v / eta_d)
            } else {
                (0.0, 1.0, inp.kappa * ETA_R * eta_h, 0.0)
            };
            PropulsorPoint {
                speed_kn: v_kn,
                thrust_per_unit_kn: thrust_kn.max(0.0),
                advance_speed_m_s: va,
                thrust_loading: c_t,
                eta_i,
                eta_d,
                delivered_power_kw: pd,
            }
        })
        .collect();
    PropulsorCurve { points }
}

/// The bollard pull of all units (t) at the delivered power `pd_kw` of each unit: kappa_b x the
/// disc's momentum limit (2 rho A)^(1/3) P_D^(2/3), over g.
pub fn bollard_pull(inp: &PropulsorInputs, pd_kw: f64) -> f64 {
    let per_unit_n = inp.kappa_b
        * cbrt(2.0 * inp.rho_kg_m3 * inp.disc_area_m2())
        * cbrt(pd_kw.max(0.0) * 1000.0 * pd_kw.max(0.0) * 1000.0);
    f64::from(inp.count) * per_unit_n / 1000.0 / G
}

/// The blade load of one unit (kW per m^2 of disc), and the limit of its type.
pub fn blade_load(inp: &PropulsorInputs, pd_kw: f64) -> (f64, f64) {
    let limit = if inp.propulsor_type.ducted() {
        BLADE_LOAD_DUCTED_KW_M2
    } else {
        BLADE_LOAD_OPEN_KW_M2
    };
    (pd_kw / inp.disc_area_m2(), limit)
}
