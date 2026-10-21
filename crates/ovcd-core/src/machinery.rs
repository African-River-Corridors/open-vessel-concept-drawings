//! The machinery space and its mass (issue #352, spec 0007 "Machinery space and mass"): the
//! below-deck run of the stern block by Power Source and Propulsor, the engines, and the mass items.
//!
//! The layout follows Hekkenberg 2013 sec. 4.2.4 (sourced as a method): below deck, aft to forward,
//! the aft peak with the steering gear, the drive room (an azimuth unit's room, or the shaft run
//! inside the stern), the engine room (A + engine + gearbox + C) when the Power Source has engines on
//! the shafts, the genset or battery room when it has one, and the gas valve unit room of a
//! gas-fuelled block. Engines stand abreast, one per propulsor; gensets abreast, N + 1. The stern
//! block is the longest of this run, the deck items and the stern run where the propulsors sit
//! ([`crate::aft_arrangement`]).
//!
//! A Design with an `unspecified` Propulsor never comes here: its stern block is today's.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::checks::Check;
use crate::propulsor::PropulsorType;
use crate::text;
use crate::text::{Arg, Text};
use crate::weights::MassItem;

/// The Power Sources of spec 0007.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum PowerSource {
    BatteryElectric,
    GasFuelled,
    DieselMechanical,
    DieselElectric,
    BatteryHybrid,
}

impl PowerSource {
    pub fn from_key(key: &str) -> Option<Self> {
        Some(match key {
            "battery_electric" => Self::BatteryElectric,
            "gas_fuelled" => Self::GasFuelled,
            "diesel_mechanical" => Self::DieselMechanical,
            "diesel_electric" => Self::DieselElectric,
            "battery_hybrid" => Self::BatteryHybrid,
            _ => return None,
        })
    }

    /// Engines on the shafts, through a gearbox.
    pub fn engines_on_shafts(self) -> bool {
        matches!(self, Self::DieselMechanical | Self::BatteryHybrid)
    }

    /// Electric motors drive the propulsors.
    pub fn electric_drive(self) -> bool {
        matches!(
            self,
            Self::BatteryElectric | Self::GasFuelled | Self::DieselElectric
        )
    }
}

/// The rates and lengths of spec 0007 "Machinery space and mass".
pub mod rates {
    /// Aft peak with the steering gear (m), estimate.
    pub const AFT_PEAK_M: f64 = 1.5;
    /// Engine room space aft of the engines (A) and the clearance to the forward bulkhead (C) (m),
    /// estimates.
    pub const ENGINE_ROOM_A_M: f64 = 1.0;
    pub const ENGINE_ROOM_C_M: f64 = 0.8;
    /// Gearbox length, and with a PTI motor (m): sourced range, Reintjes WAF-RHS.
    pub const GEARBOX_M: f64 = 1.2;
    pub const GEARBOX_PTI_M: f64 = 2.8;
    /// The shaft run inside the stern of a shaft drive, over the propeller diameter (estimate: the
    /// stern tube and the thrust bearing).
    pub const SHAFT_RUN_OVER_D: f64 = 2.0;
    /// Battery room energy density (kWh per m3 of room), estimate (research note sec. 4.3: 70 to 85
    /// for NMC, from 2.5 to 3 times the string volume); full beam less this at each side (m).
    pub const BATTERY_ROOM_KWH_M3: f64 = 70.0;
    pub const BATTERY_ROOM_SIDE_M: f64 = 1.0;
    /// Double bottom and deck structure under an engine (m), estimate, as the gas genset check.
    pub const UNDER_DECK_ALLOWANCE_M: f64 = 1.0;
    /// Walkway between engines and to the shell (m), estimate, as the gas gensets.
    pub const ENGINE_CLEAR_M: f64 = 1.0;
    /// The alternator of a diesel genset (m), estimate, as the gas gensets.
    pub const ALTERNATOR_M: f64 = 1.9;

    /// Masses (Hekkenberg 2013, Appendix B pp. 221-225, sourced unless marked).
    pub const DIESEL_T_PER_KW: f64 = 0.0049;
    pub const GEARBOX_T_PER_KW: f64 = 0.002;
    pub const GENSET_T_PER_KVA: f64 = 0.009;
    /// kVA = kW / 0.8 (estimate).
    pub const POWER_FACTOR: f64 = 0.8;
    pub const SWITCHBOARD_T_PER_KVA: f64 = 0.001;
    pub const SWITCHBOARD_MIN_T: f64 = 0.3;
    pub const CONVERTER_T: f64 = 0.9;
    pub const CONVERTER_T_PER_KVA: f64 = 0.002;
    pub const MOTOR_T_PER_KW: f64 = 0.006;
    /// PTI motor and converter (derived, Reintjes, 7.5 to 8.1 kg/kWe).
    pub const PTI_T_PER_KWE: f64 = 0.008;
    /// The largest Reintjes hybrid unit (kWe), the PTI motor of a hybrid shaft (estimate: one unit
    /// per shaft, at most the shaft's power).
    pub const PTI_MAX_KWE: f64 = 630.0;
    pub const AZIMUTH_T_PER_D3: f64 = 1.5;
    /// Voith Schneider (derived, Voith release).
    pub const VSP_T_PER_KW: f64 = 0.022;
    /// Shaft line (estimate).
    pub const SHAFT_T_PER_M: f64 = 1.0;
    pub const RUDDER_T_PER_M_D: f64 = 1.0;
    pub const RUDDERS_PER_SHAFT: f64 = 2.0;
    pub const STEERING_GEAR_T: f64 = 0.5;
    pub const BOW_THRUSTER_T_PER_KW: f64 = 0.0054;
    /// The default bow thruster power, kW per m3 of L x B x T (Hekkenberg 2013 p. 222).
    pub const BOW_THRUSTER_KW_PER_M3: f64 = 0.11;
    pub const SMALL_ITEMS_T_PER_KW: f64 = 0.001;
    pub const PIPING_T: f64 = 3.0;
    pub const PIPING_T_PER_KW: f64 = 0.0025;
    pub const BATTERY_T_PER_KWH: f64 = 0.006;
}

/// An engine of the sourced table (research note sec. 4.2, maker sheets).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Engine {
    pub model: String,
    /// The highest rating of the model (kW).
    pub max_kw: f64,
    pub length_m: f64,
    pub width_m: f64,
    pub height_m: f64,
}

/// (model, highest rating kW, length, width, height m), in the order of spec 0007: Cat 3512C
/// 955-1,903 bkW and Cat 3516C 1,492-2,525 bkW (Cat dealer pages, the longest length and the
/// largest width and height of each), Cummins QSK60 1,491-2,013 kW (Cummins sheet), Everllence
/// 12V/16V/20V 175D up to 2,580 / 3,200 / 4,400 kW (fact sheet, length L3).
const ENGINE_TABLE: [(&str, f64, f64, f64, f64); 6] = [
    ("Cat 3512C", 1903.0, 2.590, 2.037, 1.904),
    ("Cat 3516C", 2525.0, 3.761, 2.142, 2.150),
    ("Cummins QSK60", 2013.0, 3.290, 1.757, 2.415),
    ("Everllence 12V175D", 2580.0, 2.901, 1.661, 2.295),
    ("Everllence 16V175D", 3200.0, 3.421, 1.661, 2.316),
    ("Everllence 20V175D", 4400.0, 3.941, 1.647, 2.297),
];
/// Below this rating per engine (kW) the size follows the small-engine rule.
pub const SMALL_ENGINE_KW: f64 = 955.0;

/// The engine for a power per engine: the first of the table whose rating covers it; below 955 kW,
/// the Cat 3512C scaled by (P / 955)^(1/3) (estimate). `None` above the largest engine.
pub fn engine_for(kw: f64) -> Option<Engine> {
    let row = |(m, max, l, w, h): (&str, f64, f64, f64, f64), s: f64| Engine {
        model: m.to_string(),
        max_kw: max,
        length_m: l * s,
        width_m: w * s,
        height_m: h * s,
    };
    if kw < SMALL_ENGINE_KW {
        let s = libm::cbrt(kw.max(0.0) / SMALL_ENGINE_KW);
        let mut e = row(ENGINE_TABLE[0], s);
        e.model = "small engine (scaled Cat 3512C)".to_string();
        e.max_kw = kw;
        return Some(e);
    }
    ENGINE_TABLE
        .iter()
        .find(|r| r.1 + 1e-9 >= kw)
        .map(|&r| row(r, 1.0))
}

/// What the machinery needs.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MachineryInputs {
    pub power_source: PowerSource,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub propulsor: PropulsorType,
    pub propulsor_count: u32,
    /// The propeller diameter in force (m).
    pub propeller_diameter_m: f64,
    /// The installed power P_B of all engines or motors (kW). `None`: not sized yet (#353).
    pub installed_power_kw: Option<f64>,
    /// The battery energy of a battery hybrid's room (kWh). `None`: not sized yet.
    pub battery_energy_kwh: Option<f64>,
    /// The bow thruster's power (kW) when one is fitted: typed, or the default of L x B x T.
    pub bow_thruster_kw: Option<f64>,
    /// The LCG of the bow thruster from the aft end (m).
    pub bow_thruster_lcg_m: f64,
    /// The stern run where the propulsors sit (m).
    pub stern_run_m: f64,
}

impl MachineryInputs {
    /// A shaft line drives the propulsors (with rudders); every other type has its own drive room.
    pub fn shaft_drive(&self) -> bool {
        matches!(
            self.propulsor,
            PropulsorType::ShaftOpen | PropulsorType::ShaftNozzle
        )
    }
}

/// The default bow thruster power (kW): 0.11 kW per m3 of L x B x T.
pub fn default_bow_thruster_kw(loa: f64, beam: f64, draught: f64) -> f64 {
    rates::BOW_THRUSTER_KW_PER_M3 * loa * beam * draught
}

/// The machinery of one stern block, laid out.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Machinery {
    /// Below-deck zones aft to forward: (label, x_aft, x_fwd), x from the aft end.
    pub zones: Vec<(String, f64, f64)>,
    pub below_deck_length_m: f64,
    /// Where the drive room ends: the deck items start forward of it.
    pub drive_room_x1_m: f64,
    pub shaft_length_m: f64,
    /// The engines on the shafts, or the diesel gensets, with their count.
    pub engine: Option<Engine>,
    pub n_engines: u32,
    pub engine_room_x0_m: f64,
    pub engine_room_x1_m: f64,
    pub battery_room_x0_m: f64,
    pub battery_room_x1_m: f64,
    pub battery_room_m3: f64,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub checks: Vec<Check>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub warnings: Vec<Text>,
}

/// The rooms of the stern block below deck. `drive_room_m` is an azimuth unit's room (today's
/// thruster room); `gas_rooms_m` the gas genset room and the GVU room of a gas-fuelled block, sized
/// by `aft_arrangement` from its gensets.
pub fn arrange(
    inp: &MachineryInputs,
    beam: f64,
    depth: f64,
    drive_room_m: f64,
    gas_rooms_m: Option<(f64, f64)>,
) -> Machinery {
    use rates::*;
    let mut zones: Vec<(String, f64, f64)> = Vec::new();
    let mut checks = Vec::new();
    let mut warnings = Vec::new();
    let d = inp.propeller_diameter_m;
    let n = inp.propulsor_count.max(1);
    let mut x = AFT_PEAK_M;
    zones.push(("AFT PEAK, STEERING GEAR".to_string(), 0.0, x));
    let drive = if inp.shaft_drive() {
        SHAFT_RUN_OVER_D * d
    } else {
        drive_room_m
    };
    let label = if inp.shaft_drive() {
        "SHAFT RUN"
    } else {
        "DRIVE ROOM"
    };
    zones.push((label.to_string(), x, x + drive));
    x += drive;
    let drive_room_x1_m = x;
    let mut shaft_length_m = if inp.shaft_drive() { drive } else { 0.0 };

    let (mut engine, mut n_engines) = (None, 0);
    let (mut er0, mut er1, mut br0, mut br1, mut br_m3) = (0.0, 0.0, 0.0, 0.0, 0.0);
    let power = inp.installed_power_kw.filter(|p| *p > 0.0);
    let engine_room = |e: &Engine, extra: f64, x: f64| {
        (
            x,
            x + ENGINE_ROOM_A_M + e.length_m + extra + ENGINE_ROOM_C_M,
        )
    };
    match inp.power_source {
        PowerSource::DieselMechanical | PowerSource::BatteryHybrid => match power {
            Some(p) => match engine_for(p / f64::from(n)) {
                Some(e) => {
                    let gearbox = if inp.power_source == PowerSource::BatteryHybrid {
                        GEARBOX_PTI_M
                    } else {
                        GEARBOX_M
                    };
                    (er0, er1) = engine_room(&e, gearbox, x);
                    zones.push(("ENGINE ROOM".to_string(), er0, er1));
                    if inp.shaft_drive() {
                        shaft_length_m += ENGINE_ROOM_A_M;
                    }
                    x = er1;
                    engine = Some(e);
                    n_engines = n;
                }
                None => warnings.push(no_engine(p / f64::from(n))),
            },
            None => warnings.push(power_not_sized()),
        },
        PowerSource::DieselElectric => match power {
            // N running sets, one per propulsor, and one in reserve.
            Some(p) => match engine_for(p / f64::from(n)) {
                Some(e) => {
                    (er0, er1) = engine_room(&e, ALTERNATOR_M, x);
                    zones.push(("GENSET ROOM".to_string(), er0, er1));
                    x = er1;
                    engine = Some(e);
                    n_engines = n + 1;
                }
                None => warnings.push(no_engine(p / f64::from(n))),
            },
            None => warnings.push(power_not_sized()),
        },
        PowerSource::GasFuelled => {
            if let Some((genset_room, gvu)) = gas_rooms_m {
                zones.push(("ENGINE ROOM (GAS GENSETS)".to_string(), x, x + genset_room));
                (er0, er1) = (x, x + genset_room);
                x += genset_room;
                zones.push(("GVU ROOM".to_string(), x, x + gvu));
                x += gvu;
            }
        }
        PowerSource::BatteryElectric => {}
    }
    if inp.power_source == PowerSource::BatteryHybrid {
        match inp.battery_energy_kwh.filter(|e| *e > 0.0) {
            Some(e) => {
                br_m3 = e / BATTERY_ROOM_KWH_M3;
                let section = (depth - UNDER_DECK_ALLOWANCE_M) * (beam - 2.0 * BATTERY_ROOM_SIDE_M);
                let len = if section > 0.0 { br_m3 / section } else { 0.0 };
                (br0, br1) = (x, x + len);
                zones.push(("BATTERY ROOM".to_string(), br0, br1));
                x = br1;
            }
            None => warnings.push(Text::fixed(
                "warn-machinery-battery-not-sized",
                "The battery energy of the hybrid is not sized yet, so the battery room is not in \
                 the stern block.",
            )),
        }
    }
    if let Some(e) = &engine {
        let needed = f64::from(n_engines) * e.width_m + f64::from(n_engines + 1) * ENGINE_CLEAR_M;
        checks.push(Check::new(
            "machinery_fits_beam",
            needed <= beam + 1e-9,
            text!(
                "check-machinery-fits-beam",
                "{count} x {width} m engines abreast need {needed} m across a {beam} m beam",
                count = i64::from(n_engines),
                width = Arg::fixed(e.width_m, 2),
                needed = Arg::fixed(needed, 2),
                beam = Arg::fixed(beam, 2),
            ),
        ));
        checks.push(Check::with_severity(
            "machinery_fits_under_deck",
            e.height_m + UNDER_DECK_ALLOWANCE_M <= depth,
            text!(
                "check-machinery-fits-under-deck",
                "engine {height} m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth {depth} m",
                height = Arg::fixed(e.height_m, 2),
                depth = Arg::fixed(depth, 2),
            ),
            "warning",
        ));
    }
    Machinery {
        zones,
        below_deck_length_m: x,
        drive_room_x1_m,
        shaft_length_m,
        engine,
        n_engines,
        engine_room_x0_m: er0,
        engine_room_x1_m: er1,
        battery_room_x0_m: br0,
        battery_room_x1_m: br1,
        battery_room_m3: br_m3,
        checks,
        warnings,
    }
}

fn power_not_sized() -> Text {
    Text::fixed(
        "warn-machinery-power-not-sized",
        "The installed power is not sized yet, so the engines and their room are not in the stern \
         block; type an installed power to place them.",
    )
}

fn no_engine(kw: f64) -> Text {
    text!(
        "warn-machinery-no-engine",
        "No engine of the table covers {kw} kW per engine (the largest is 4,400 kW), so the engine room is not in the stern block.",
        kw = Arg::fixed(kw, 0),
    )
}

/// The mass items of the machinery (spec 0007 table), each at its zone centre: the propulsors, the
/// rudders of a shaft drive, the engines, gearboxes, gensets, electrical plant, the bow thruster, the
/// small items and the piping, and a hybrid's battery room. `gas_gensets` is the gas genset item of
/// `aft_arrangement` (a gas-fuelled block); `power_fallback_kw` the power the per-kW items take when
/// none is sized (#348: the installed genset power, else none).
pub fn mass_items(
    inp: &MachineryInputs,
    m: &Machinery,
    depth: f64,
    double_bottom_m: f64,
    power_fallback_kw: f64,
) -> Vec<MassItem> {
    use rates::*;
    let mut items = Vec::new();
    let d = inp.propeller_diameter_m;
    let n = f64::from(inp.propulsor_count.max(1));
    let power = inp
        .installed_power_kw
        .filter(|p| *p > 0.0)
        .unwrap_or(power_fallback_kw);
    let drive_lcg = match m.zones.get(1) {
        Some((_, a, f)) => (a + f) / 2.0,
        None => 0.0,
    };
    let shaft_axis = d / 2.0;
    let mut push = |label: &'static str, mass_t: f64, lcg_m: f64, kcg_m: f64| {
        if mass_t > 0.0 {
            items.push(MassItem {
                label,
                mass_t,
                lcg_m,
                kcg_m,
            });
        }
    };
    match inp.propulsor {
        PropulsorType::AzimuthDucted | PropulsorType::AzimuthOpen => {
            push("propulsors", n * AZIMUTH_T_PER_D3 * d * d * d, drive_lcg, d);
        }
        PropulsorType::VoithSchneider => push("propulsors", VSP_T_PER_KW * power, drive_lcg, d),
        PropulsorType::PumpJet => {
            let p = power / n;
            let kg = 0.003 * p * p + 6.0829 * p - 138.17;
            push(
                "propulsors",
                n * kg.max(0.0) / 1000.0,
                drive_lcg,
                shaft_axis,
            );
        }
        PropulsorType::ShaftOpen | PropulsorType::ShaftNozzle => {
            push(
                "shaft lines",
                n * SHAFT_T_PER_M * m.shaft_length_m,
                AFT_PEAK_M + m.shaft_length_m / 2.0,
                shaft_axis,
            );
            push(
                "rudders and steering gear",
                n * (RUDDERS_PER_SHAFT * RUDDER_T_PER_M_D * d + STEERING_GEAR_T),
                AFT_PEAK_M / 2.0,
                d,
            );
        }
    }
    let er_lcg = (m.engine_room_x0_m + m.engine_room_x1_m) / 2.0;
    if let Some(e) = &m.engine {
        let p = inp.installed_power_kw.unwrap_or(0.0);
        let engine_kcg = double_bottom_m + e.height_m / 2.0;
        match inp.power_source {
            PowerSource::DieselMechanical | PowerSource::BatteryHybrid => {
                push("engines", DIESEL_T_PER_KW * p, er_lcg, engine_kcg);
                push("gearboxes", GEARBOX_T_PER_KW * p, er_lcg, shaft_axis);
                if inp.power_source == PowerSource::BatteryHybrid {
                    let pti = n * (p / n).min(PTI_MAX_KWE);
                    push(
                        "PTI motors and converters",
                        PTI_T_PER_KWE * pti,
                        er_lcg,
                        shaft_axis,
                    );
                }
            }
            PowerSource::DieselElectric => {
                // The N + 1 sets, each of P / N.
                let kva = f64::from(m.n_engines) * p / n / POWER_FACTOR;
                push("gensets", GENSET_T_PER_KVA * kva, er_lcg, engine_kcg);
            }
            _ => {}
        }
    }
    if inp.power_source.electric_drive() {
        let kva = power / POWER_FACTOR;
        let deck_lcg = m.drive_room_x1_m / 2.0;
        push(
            "switchboards",
            (SWITCHBOARD_T_PER_KVA * kva).max(SWITCHBOARD_MIN_T),
            deck_lcg,
            depth - 1.0,
        );
        push(
            "frequency converters",
            CONVERTER_T + CONVERTER_T_PER_KVA * kva,
            deck_lcg,
            depth - 1.0,
        );
        push(
            "electric motors",
            MOTOR_T_PER_KW * power,
            drive_lcg,
            shaft_axis,
        );
    }
    if let Some(kw) = inp.bow_thruster_kw {
        push(
            "bow thruster",
            BOW_THRUSTER_T_PER_KW * kw,
            inp.bow_thruster_lcg_m,
            1.0,
        );
    }
    if m.battery_room_m3 > 0.0 {
        push(
            "battery room",
            BATTERY_T_PER_KWH * inp.battery_energy_kwh.unwrap_or(0.0),
            (m.battery_room_x0_m + m.battery_room_x1_m) / 2.0,
            double_bottom_m + (depth - double_bottom_m) / 2.0,
        );
    }
    let block_lcg = m.below_deck_length_m / 2.0;
    push(
        "small items",
        SMALL_ITEMS_T_PER_KW * power,
        block_lcg,
        double_bottom_m,
    );
    push(
        "piping and cabling",
        PIPING_T + PIPING_T_PER_KW * power,
        block_lcg,
        double_bottom_m,
    );
    items
}
