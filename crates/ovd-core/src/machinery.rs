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
    /// Aft peak with the steering gear (m), estimate: the low end of the built shaft-driven inland
    /// ships (2.5 to 5.0 m); azimuth tugs have 1.5 to 2.0 m. Fitted to built ships (issue #365,
    /// `calibration/stern_block_results.md`); it was 1.5.
    pub const AFT_PEAK_M: f64 = 2.5;
    /// The factor on the drive room (or shaft run) plus the engine room, for the rooms the layout does
    /// not draw (switchboard room, workshop, stores, auxiliaries and the access between them),
    /// estimate: least-squares fit 1.42 to built ships (issue #365). The shaft mass, the engine
    /// widths and the gas rooms do not take it.
    pub const ROOM_ALLOWANCE_FACTOR: f64 = 1.4;
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
    /// The battery energy in force (kWh): a battery hybrid's room holds it. `None`: not sized yet.
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
    // The room drawn is the drive room or shaft run times the allowance; the shaft itself is `drive`.
    let drive_room = ROOM_ALLOWANCE_FACTOR * drive;
    let label = if inp.shaft_drive() {
        "SHAFT RUN"
    } else {
        "DRIVE ROOM"
    };
    zones.push((label.to_string(), x, x + drive_room));
    x += drive_room;
    let drive_room_x1_m = x;
    let mut shaft_length_m = if inp.shaft_drive() { drive } else { 0.0 };

    let (mut engine, mut n_engines) = (None, 0);
    let (mut er0, mut er1, mut br0, mut br1, mut br_m3) = (0.0, 0.0, 0.0, 0.0, 0.0);
    let power = inp.installed_power_kw.filter(|p| *p > 0.0);
    let engine_room = |e: &Engine, extra: f64, x: f64| {
        (
            x,
            x + ROOM_ALLOWANCE_FACTOR * (ENGINE_ROOM_A_M + e.length_m + extra + ENGINE_ROOM_C_M),
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
    let drive = drive_unit_positions(inp.propulsor, drive_room_centre(m), d);
    let drive_lcg = drive.lcg_m;
    let shaft_axis = axis_height(d);
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
            push(
                "propulsors",
                n * AZIMUTH_T_PER_D3 * d * d * d,
                drive.lcg_m,
                drive.kcg_m,
            );
        }
        PropulsorType::VoithSchneider => {
            push("propulsors", VSP_T_PER_KW * power, drive.lcg_m, drive.kcg_m)
        }
        PropulsorType::PumpJet => {
            let p = power / n;
            let kg = 0.003 * p * p + 6.0829 * p - 138.17;
            push(
                "propulsors",
                n * kg.max(0.0) / 1000.0,
                drive.lcg_m,
                drive.kcg_m,
            );
        }
        PropulsorType::ShaftOpen | PropulsorType::ShaftNozzle => {
            let shaft = shaft_positions(m, d);
            push(
                "shaft lines",
                n * SHAFT_T_PER_M * m.shaft_length_m,
                shaft.centre().lcg_m,
                shaft.centre().kcg_m,
            );
            let rudder = rudder_positions(d);
            push(
                "rudders and steering gear",
                n * (RUDDERS_PER_SHAFT * RUDDER_T_PER_M_D * d + STEERING_GEAR_T),
                rudder.lcg_m,
                rudder.kcg_m,
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
        let at = bow_thruster_position(inp);
        push(
            "bow thruster",
            BOW_THRUSTER_T_PER_KW * kw,
            at.lcg_m,
            at.kcg_m,
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

// --- Spec 0009 "Propulsors" (issue #691): the Propulsor Layout. ------------------------------------
//
// One set of position functions places every propulsor, shaft line, rudder and the bow thruster.
// [`mass_items`] (and the stern block of an `unspecified` Propulsor,
// [`crate::aft_arrangement::stern_block_mass`]) take their centres from them, and
// [`propulsor_layout`] gives the same places, with sizes, to the drawings and the 3D view. The
// layout adds geometry only: no mass and no result moves.

/// The sizes of the Propulsor Layout that no published proportion gives (spec 0009 decision D2):
/// each is an estimate, with a row in `docs/basis/11-drawings.md`. They size the drawn parts only;
/// no mass, centre or result reads them.
pub mod layout_rates {
    /// Nozzle length over the propeller diameter (estimate).
    pub const NOZZLE_LENGTH_OVER_D: f64 = 0.5;
    /// Azimuth pod length and diameter over the propeller diameter (estimates).
    pub const POD_LENGTH_OVER_D: f64 = 0.8;
    pub const POD_DIAMETER_OVER_D: f64 = 0.35;
    /// Azimuth leg: its top above the base, its chord and thickness, over the propeller diameter
    /// (estimates).
    pub const LEG_TOP_OVER_D: f64 = 1.5;
    pub const LEG_CHORD_OVER_D: f64 = 0.4;
    pub const LEG_THICKNESS_OVER_D: f64 = 0.15;
    /// Rudder span and chord over the propeller diameter, thickness over the chord, and the
    /// athwartships distance between the two rudders of a shaft over the diameter (estimates).
    pub const RUDDER_SPAN_OVER_D: f64 = 1.0;
    pub const RUDDER_CHORD_OVER_D: f64 = 0.7;
    pub const RUDDER_THICKNESS_OVER_CHORD: f64 = 0.15;
    pub const RUDDER_SPACING_OVER_D: f64 = 0.6;
    /// Bow thruster tunnel diameter per square root of its power (m per kW^0.5, estimate).
    pub const BOW_TUNNEL_D_PER_SQRT_KW: f64 = 0.07;
    /// The thickness of the propeller disc the 3D view draws, over the diameter (estimate; the
    /// Propulsor model is an actuator disc, `propulsor.rs`).
    pub const DISC_THICKNESS_OVER_D: f64 = 0.05;
}

/// A mass centre: LCG from the aft end and KCG above the base (m).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Place {
    pub lcg_m: f64,
    pub kcg_m: f64,
}

/// The shaft axis above the base: half the propeller diameter (the tip on the base, as the Stern
/// Form's D_max and the `powering.shaft_axis_height_m` result have it).
pub fn axis_height(d: f64) -> f64 {
    d / 2.0
}

/// The centre of the drive room (or shaft run), the second zone below deck; 0 with none.
pub fn drive_room_centre(m: &Machinery) -> f64 {
    match m.zones.get(1) {
        Some((_, a, f)) => (a + f) / 2.0,
        None => 0.0,
    }
}

/// The mass centre of the drive units (azimuth, Voith Schneider, pump jet) at a drive room centred
/// at `drive_lcg_m`: an azimuth or Voith Schneider unit at the base + D (spec 0007), a pump jet on
/// the shaft axis.
pub fn drive_unit_positions(propulsor: PropulsorType, drive_lcg_m: f64, d: f64) -> Place {
    let kcg_m = match propulsor {
        PropulsorType::PumpJet | PropulsorType::ShaftOpen | PropulsorType::ShaftNozzle => {
            axis_height(d)
        }
        _ => d,
    };
    Place {
        lcg_m: drive_lcg_m,
        kcg_m,
    }
}

/// A shaft line from the aft peak forward, on the shaft axis.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ShaftPlace {
    pub x_aft_m: f64,
    pub length_m: f64,
    pub axis_z_m: f64,
}

impl ShaftPlace {
    /// The shaft line's mass centre: its middle.
    pub fn centre(&self) -> Place {
        Place {
            lcg_m: self.x_aft_m + self.length_m / 2.0,
            kcg_m: self.axis_z_m,
        }
    }
}

/// The shaft lines of a shaft drive: from the aft peak's forward end, the shaft length forward. The
/// propeller sits at the aft end.
pub fn shaft_positions(m: &Machinery, d: f64) -> ShaftPlace {
    ShaftPlace {
        x_aft_m: rates::AFT_PEAK_M,
        length_m: m.shaft_length_m,
        axis_z_m: axis_height(d),
    }
}

/// The rudders and steering gear of a shaft drive: in the aft peak, aft of the propellers, at the
/// base + D (spec 0007).
pub fn rudder_positions(d: f64) -> Place {
    Place {
        lcg_m: rates::AFT_PEAK_M / 2.0,
        kcg_m: d,
    }
}

/// The bow thruster: at its LCG, 1.0 m above the base (spec 0007).
pub fn bow_thruster_position(inp: &MachineryInputs) -> Place {
    Place {
        lcg_m: inp.bow_thruster_lcg_m,
        kcg_m: 1.0,
    }
}

/// A nozzle round a propeller disc.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Nozzle {
    pub inner_diameter_m: f64,
    pub outer_diameter_m: f64,
    pub length_m: f64,
}

/// The leg and pod of an azimuth unit: the leg from the pod up to `leg_top_z_m`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AzimuthLeg {
    pub leg_top_z_m: f64,
    pub leg_chord_m: f64,
    pub leg_thickness_m: f64,
    pub pod_length_m: f64,
    pub pod_diameter_m: f64,
}

/// One propulsor: its disc centre (x from the aft end, y + port, z the axis), the mass centre of its
/// mass item, and its parts.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PropulsorPart {
    pub x_m: f64,
    pub y_m: f64,
    pub z_m: f64,
    /// The mass item this unit belongs to, and its centre.
    pub mass_item: &'static str,
    pub lcg_m: f64,
    pub kcg_m: f64,
    pub nozzle: Option<Nozzle>,
    pub azimuth: Option<AzimuthLeg>,
    /// A shaft drive's shaft line.
    pub shaft: Option<ShaftPlace>,
}

/// One rudder blade: its centre, span (vertical), chord (fore and aft) and thickness.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RudderPart {
    pub x_m: f64,
    pub y_m: f64,
    pub z_m: f64,
    pub span_m: f64,
    pub chord_m: f64,
    pub thickness_m: f64,
    pub lcg_m: f64,
    pub kcg_m: f64,
}

/// The bow thruster: a tunnel across the hull on its axis.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct BowThrusterPart {
    pub x_m: f64,
    pub z_m: f64,
    pub tunnel_diameter_m: f64,
    /// Across the hull: the moulded beam.
    pub tunnel_length_m: f64,
    pub power_kw: f64,
    pub lcg_m: f64,
    pub kcg_m: f64,
}

/// The Propulsor Layout (spec 0009): the place and size of each propulsor, nozzle, rudder and bow
/// thruster of a Hull.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PropulsorLayout {
    /// The Propulsor type key, or `unspecified` for the default stern block's thrusters.
    pub propulsor_type: &'static str,
    pub diameter_m: f64,
    pub axis_z_m: f64,
    pub ducted: bool,
    /// The Installed Power of one unit (kW), when it is sized or typed; `None` for the default
    /// stern block.
    pub unit_power_kw: Option<f64>,
    pub propulsors: Vec<PropulsorPart>,
    pub rudders: Vec<RudderPart>,
    pub bow_thruster: Option<BowThrusterPart>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub warnings: Vec<Text>,
}

/// The stern block's units of a specified Propulsor away from a tunnel stern (issue #702):
/// `propulsor_count` units, each of the Propulsor's own envelope (1.35 D), so the block's
/// `thruster_offsets_m` and the Propulsor Layout agree. `None` under a tunnel stern, where the
/// tunnels place the units.
pub fn stern_block_units(
    m: &MachineryInputs,
    stern: Option<&crate::stern_form::SternGeometry>,
) -> Option<(i64, f64)> {
    if stern.is_some_and(|g| g.form == crate::stern_form::SternForm::Tunnel) {
        return None;
    }
    Some((
        i64::from(m.propulsor_count.max(1)),
        crate::aft_arrangement::stern_mass::ENVELOPE_OVER_DIAMETER * m.propeller_diameter_m,
    ))
}

/// The athwartships centres of `count` units: the stern's offsets when they hold `count`, else
/// `count` units spread at `pitch` about the centre line, with a warning. `offsets` are the tunnel
/// centres under a tunnel stern, else the stern block's thruster offsets, which hold `count` units
/// (issue #702) unless the beam takes fewer.
fn unit_offsets(offsets: &[f64], count: usize, pitch: f64, warnings: &mut Vec<Text>) -> Vec<f64> {
    if offsets.len() == count {
        return offsets.to_vec();
    }
    warnings.push(text!(
        "warn-propulsor-layout-offsets",
        "The stern holds {held} propulsor places for {count} propulsors; the layout spreads the {count} at {pitch} m.",
        held = offsets.len() as i64,
        count = count as i64,
        pitch = Arg::fixed(pitch, 2),
    ));
    let span = (count as f64 - 1.0) * pitch;
    (0..count).map(|i| -span / 2.0 + i as f64 * pitch).collect()
}

/// The Propulsor Layout of a specified Propulsor. `offsets` are the tunnel centres under a tunnel
/// stern (every type), else the stern block's thruster offsets; `pitch` is the stern block's
/// thruster pitch, used when `offsets` does not hold `propulsor_count` units (the beam takes
/// fewer); `beam` the tunnel length of the bow thruster.
pub fn propulsor_layout(
    inp: &MachineryInputs,
    m: &Machinery,
    offsets: &[f64],
    pitch: f64,
    beam: f64,
) -> PropulsorLayout {
    use layout_rates::*;
    let d = inp.propeller_diameter_m;
    let axis = axis_height(d);
    let mut warnings = Vec::new();
    let ys = unit_offsets(
        offsets,
        inp.propulsor_count.max(1) as usize,
        pitch,
        &mut warnings,
    );
    let nozzle = inp.propulsor.ducted().then_some(Nozzle {
        inner_diameter_m: d,
        outer_diameter_m: crate::aft_arrangement::stern_mass::ENVELOPE_OVER_DIAMETER * d,
        length_m: NOZZLE_LENGTH_OVER_D * d,
    });
    let mut propulsors = Vec::new();
    let mut rudders = Vec::new();
    for &y in &ys {
        if inp.shaft_drive() {
            let shaft = shaft_positions(m, d);
            let centre = shaft.centre();
            propulsors.push(PropulsorPart {
                x_m: shaft.x_aft_m,
                y_m: y,
                z_m: axis,
                mass_item: "shaft lines",
                lcg_m: centre.lcg_m,
                kcg_m: centre.kcg_m,
                nozzle,
                azimuth: None,
                shaft: Some(shaft),
            });
            let r = rudder_positions(d);
            for side in [-1.0, 1.0] {
                rudders.push(RudderPart {
                    x_m: r.lcg_m,
                    y_m: y + side * RUDDER_SPACING_OVER_D * d / 2.0,
                    z_m: r.kcg_m,
                    span_m: RUDDER_SPAN_OVER_D * d,
                    chord_m: RUDDER_CHORD_OVER_D * d,
                    thickness_m: RUDDER_THICKNESS_OVER_CHORD * RUDDER_CHORD_OVER_D * d,
                    lcg_m: r.lcg_m,
                    kcg_m: r.kcg_m,
                });
            }
        } else {
            let at = drive_unit_positions(inp.propulsor, drive_room_centre(m), d);
            let azimuth = matches!(
                inp.propulsor,
                PropulsorType::AzimuthOpen | PropulsorType::AzimuthDucted
            )
            .then(|| azimuth_leg(d));
            propulsors.push(PropulsorPart {
                x_m: at.lcg_m,
                y_m: y,
                z_m: axis,
                mass_item: "propulsors",
                lcg_m: at.lcg_m,
                kcg_m: at.kcg_m,
                nozzle,
                azimuth,
                shaft: None,
            });
        }
    }
    let bow_thruster = inp.bow_thruster_kw.map(|kw| {
        let at = bow_thruster_position(inp);
        BowThrusterPart {
            x_m: at.lcg_m,
            z_m: at.kcg_m,
            tunnel_diameter_m: BOW_TUNNEL_D_PER_SQRT_KW * libm::sqrt(kw),
            tunnel_length_m: beam,
            power_kw: kw,
            lcg_m: at.lcg_m,
            kcg_m: at.kcg_m,
        }
    });
    PropulsorLayout {
        propulsor_type: inp.propulsor.key(),
        diameter_m: d,
        axis_z_m: axis,
        ducted: inp.propulsor.ducted(),
        unit_power_kw: inp
            .installed_power_kw
            .map(|p| p / f64::from(inp.propulsor_count.max(1))),
        propulsors,
        rudders,
        bow_thruster,
        warnings,
    }
}

fn azimuth_leg(d: f64) -> AzimuthLeg {
    use layout_rates::*;
    AzimuthLeg {
        leg_top_z_m: LEG_TOP_OVER_D * d,
        leg_chord_m: LEG_CHORD_OVER_D * d,
        leg_thickness_m: LEG_THICKNESS_OVER_D * d,
        pod_length_m: POD_LENGTH_OVER_D * d,
        pod_diameter_m: POD_DIAMETER_OVER_D * d,
    }
}

/// The Propulsor Layout of the default stern block (an `unspecified` Propulsor): its ducted azimuth
/// thrusters, D = the thruster envelope / 1.35, at the thruster room's centre and the stern block's
/// offsets (the count is the offsets', as the thrusters' mass item takes it).
pub fn default_thruster_layout(
    thruster_room_length_m: f64,
    thruster_envelope_m: f64,
    offsets: &[f64],
) -> Option<PropulsorLayout> {
    if offsets.is_empty() {
        return None;
    }
    let d = thruster_envelope_m / crate::aft_arrangement::stern_mass::ENVELOPE_OVER_DIAMETER;
    let at = drive_unit_positions(
        PropulsorType::AzimuthDucted,
        thruster_room_length_m / 2.0,
        d,
    );
    let propulsors = offsets
        .iter()
        .map(|&y| PropulsorPart {
            x_m: at.lcg_m,
            y_m: y,
            z_m: axis_height(d),
            mass_item: "thrusters",
            lcg_m: at.lcg_m,
            kcg_m: at.kcg_m,
            nozzle: Some(Nozzle {
                inner_diameter_m: d,
                outer_diameter_m: thruster_envelope_m,
                length_m: layout_rates::NOZZLE_LENGTH_OVER_D * d,
            }),
            azimuth: Some(azimuth_leg(d)),
            shaft: None,
        })
        .collect();
    Some(PropulsorLayout {
        propulsor_type: "unspecified",
        diameter_m: d,
        axis_z_m: axis_height(d),
        ducted: true,
        unit_power_kw: None,
        propulsors,
        rudders: Vec::new(),
        bow_thruster: None,
        warnings: Vec::new(),
    })
}

/// The Propulsor Layout of a hull's stern block, from the capacity inputs and result the run worked
/// with: a specified Propulsor from its machinery, else the default stern block's thrusters; `None`
/// with no stern block. Under a tunnel stern every type sits on its tunnel's axis (spec 0009).
pub fn layout_of(
    inp: &crate::barge_capacity::BargeCapacityInputs,
    r: &crate::barge_capacity::BargeCapacityResult,
) -> Option<PropulsorLayout> {
    let aft = r.aft_arrangement.as_ref()?;
    let (envelope, clear) = match &inp.aft {
        Some(a) => (a.thruster_envelope_m, a.thruster_clear_m),
        None => (
            inp.aft_design.thruster_envelope_m,
            inp.aft_design.thruster_clear_m,
        ),
    };
    match (&inp.machinery, &aft.machinery) {
        (Some(mi), Some(m)) => {
            // Issue #702: the stern block's pitch is the Propulsor's own envelope.
            let envelope =
                stern_block_units(mi, inp.stern_geometry.as_ref()).map_or(envelope, |(_, e)| e);
            let offsets = match &inp.stern_geometry {
                Some(g) if g.form == crate::stern_form::SternForm::Tunnel => {
                    g.tunnel_offsets_m.as_slice()
                }
                _ => aft.thruster_offsets_m.as_slice(),
            };
            Some(propulsor_layout(mi, m, offsets, envelope + clear, inp.beam))
        }
        _ => default_thruster_layout(
            aft.thruster_room_length_m,
            envelope,
            &aft.thruster_offsets_m,
        ),
    }
}
