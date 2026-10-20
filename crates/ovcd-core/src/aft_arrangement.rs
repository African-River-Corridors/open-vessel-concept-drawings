//! Aft arrangement: the self-propelled vessel's stern block, derived from its equipment.
//! Port of `core/aft_arrangement.py`. The reasoning behind the arrangement lives in the Python file.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::checks::{Check, all_ok};
use crate::pyfmt::fixed;
use crate::text;
use crate::text::{Arg, Text};

// ISO 40 ft container envelope (ISO 668 series 1, 1AAA high-cube).
pub const ISO40_LENGTH_M: f64 = 12.192;
pub const ISO40_WIDTH_M: f64 = 2.438;
pub const ISO40_HEIGHT_HC_M: f64 = 2.896;
pub const ISO40_HEIGHT_STD_M: f64 = 2.591;

// The aft block's design values before they were inputs (issue #4). `AftArrangementInputs::new`
// and `AftDesign::default` both read them, and so does the `.vdesign` table of fixed values for
// files that lack the inputs, through a test that holds them equal.
/// Most thrusters the block asks for; the beam may take fewer.
pub const DEFAULT_N_THRUSTERS: i64 = 3;
pub const DEFAULT_THRUSTER_ENVELOPE_M: f64 = 2.60;
pub const DEFAULT_THRUSTER_CLEAR_M: f64 = 0.60;
pub const DEFAULT_THRUSTER_ROOM_LENGTH_M: f64 = 3.50;
/// 40 ft units by Cargo phase: dry bulk carries a second battery, gas a second accommodation unit.
pub const DEFAULT_BATTERY_UNITS_GAS: i64 = 1;
pub const DEFAULT_BATTERY_UNITS_DRY: i64 = 2;
pub const DEFAULT_ACCOM_UNITS_GAS: i64 = 2;
pub const DEFAULT_ACCOM_UNITS_DRY: i64 = 1;
pub const DEFAULT_CONTAINER_SIDE_GAP_M: f64 = 0.90;
pub const DEFAULT_CONTAINER_GAP_M: f64 = 0.80;
pub const DEFAULT_CONTAINER_FOUNDATION_M: f64 = 0.30;
pub const DEFAULT_WHEELHOUSE_LENGTH_M: f64 = 3.60;
pub const DEFAULT_WHEELHOUSE_WIDTH_M: f64 = 6.40;
pub const DEFAULT_WHEELHOUSE_HEIGHT_M: f64 = 3.40;
pub const DEFAULT_WHEELHOUSE_GAP_M: f64 = 0.30;
pub const DEFAULT_WHEELHOUSE_EYE_ABOVE_FLOOR_M: f64 = 1.50;
pub const DEFAULT_SIGHTLINE_MARGIN_M: f64 = 0.50;
pub const DEFAULT_WHEELHOUSE_MAX_LIFT_M: f64 = 6.00;

/// The design values of the stern block that a design chooses (not ISO dimensions, not genset
/// data). `Default` is the values the engine used before they were inputs. A vessel built with
/// these needs no `AftArrangementInputs`: `barge_capacity` makes the block from them.
#[derive(Debug, Clone, PartialEq)]
pub struct AftDesign {
    /// A maximum: `compute` fits what the beam takes.
    pub n_thrusters: i64,
    pub thruster_envelope_m: f64,
    pub thruster_clear_m: f64,
    pub thruster_room_length_m: f64,
    /// `None`: the default of the Cargo phase (gas 1, dry 2).
    pub n_battery_units: Option<i64>,
    /// `None`: the default of the Cargo phase (gas 2, dry 1).
    pub n_accom_units: Option<i64>,
    pub container_height_m: f64,
    pub container_side_gap_m: f64,
    pub container_gap_m: f64,
    pub container_foundation_m: f64,
    pub wheelhouse_length_m: f64,
    pub wheelhouse_width_m: f64,
    pub wheelhouse_height_m: f64,
    pub wheelhouse_gap_m: f64,
    pub wheelhouse_eye_above_floor_m: f64,
    pub sightline_margin_m: f64,
    pub wheelhouse_max_lift_m: f64,
}

impl Default for AftDesign {
    fn default() -> Self {
        Self {
            n_thrusters: DEFAULT_N_THRUSTERS,
            thruster_envelope_m: DEFAULT_THRUSTER_ENVELOPE_M,
            thruster_clear_m: DEFAULT_THRUSTER_CLEAR_M,
            thruster_room_length_m: DEFAULT_THRUSTER_ROOM_LENGTH_M,
            n_battery_units: None,
            n_accom_units: None,
            container_height_m: ISO40_HEIGHT_HC_M,
            container_side_gap_m: DEFAULT_CONTAINER_SIDE_GAP_M,
            container_gap_m: DEFAULT_CONTAINER_GAP_M,
            container_foundation_m: DEFAULT_CONTAINER_FOUNDATION_M,
            wheelhouse_length_m: DEFAULT_WHEELHOUSE_LENGTH_M,
            wheelhouse_width_m: DEFAULT_WHEELHOUSE_WIDTH_M,
            wheelhouse_height_m: DEFAULT_WHEELHOUSE_HEIGHT_M,
            wheelhouse_gap_m: DEFAULT_WHEELHOUSE_GAP_M,
            wheelhouse_eye_above_floor_m: DEFAULT_WHEELHOUSE_EYE_ABOVE_FLOOR_M,
            sightline_margin_m: DEFAULT_SIGHTLINE_MARGIN_M,
            wheelhouse_max_lift_m: DEFAULT_WHEELHOUSE_MAX_LIFT_M,
        }
    }
}

/// The two propulsion arrangements. `battery_electric` is the default and its numbers are those of
/// 0.1.0; `gas_fuelled` is issue #180 (see the Python file for the layout).
pub const PROPULSION_OPTIONS: [&str; 2] = ["battery_electric", "gas_fuelled"];

/// One marine generating set: engine and alternator on a common base frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Genset {
    pub genset_model: &'static str,
    pub engine_model: &'static str,
    /// Continuous electrical output (kWe).
    pub kwe: f64,
    pub rpm: f64,
    /// Engine only, as published (mm converted to m; kg to t).
    pub engine_length_m: f64,
    pub engine_width_m: f64,
    pub engine_height_m: f64,
    pub engine_mass_t: f64,
}

// Envelope of a set beyond the published engine: the alternator on the free end and the common base
// frame. ESTIMATES until a vendor GA exists for the chosen set.
pub const ALTERNATOR_LENGTH_M: f64 = 1.90; // ESTIMATE
pub const ALTERNATOR_WIDTH_M: f64 = 1.90; // ESTIMATE
pub const BASE_FRAME_HEIGHT_M: f64 = 0.25; // ESTIMATE

impl Genset {
    pub fn length_m(&self) -> f64 {
        self.engine_length_m + ALTERNATOR_LENGTH_M
    }
    pub fn width_m(&self) -> f64 {
        py_max(self.engine_width_m, ALTERNATOR_WIDTH_M)
    }
    pub fn height_m(&self) -> f64 {
        self.engine_height_m + BASE_FRAME_HEIGHT_M
    }
}

/// Weichai medium-speed marine generating sets of the WH20 and WH25 families, whose engines are
/// offered for MDO/MGO/HFO and dual fuel (DF), IMO Tier II/III (Tier III with SCR), CCS/BV classed.
/// Source: Weichai Power, "Weichai Medium Speed Marine Engine", product presentation, April 2023:
/// WH20 and WH25 basic parameters (engine dimensions and mass, slides 12 and 15), the WH20 marine
/// auxiliary table (6WH20 genset power), and the marine genset model list, 440 V / 50 Hz (CCFJ
/// models, slides 30 and 31).
/// Ascending by output. Confirm the dual-fuel build of a chosen set with the maker.
pub const GAS_GENSETS: [Genset; 8] = [
    Genset {
        genset_model: "6WH20LCD1000-1",
        engine_model: "6WH20LCD1000-1",
        kwe: 900.0,
        rpm: 1000.0,
        engine_length_m: 3.532,
        engine_width_m: 1.365,
        engine_height_m: 2.642,
        engine_mass_t: 9.8,
    },
    Genset {
        genset_model: "6WH20LCD1200-1",
        engine_model: "6WH20LCD1200-1",
        kwe: 1050.0,
        rpm: 1000.0,
        engine_length_m: 3.532,
        engine_width_m: 1.365,
        engine_height_m: 2.642,
        engine_mass_t: 9.8,
    },
    Genset {
        genset_model: "CCFJ1200J-WMW",
        engine_model: "8WH20LCD1440-1",
        kwe: 1200.0,
        rpm: 1000.0,
        engine_length_m: 4.132,
        engine_width_m: 1.365,
        engine_height_m: 2.642,
        engine_mass_t: 12.5,
    },
    Genset {
        genset_model: "CCFJ1400J-WMW",
        engine_model: "8WH20LCD1600-1",
        kwe: 1400.0,
        rpm: 1000.0,
        engine_length_m: 4.132,
        engine_width_m: 1.365,
        engine_height_m: 2.642,
        engine_mass_t: 12.5,
    },
    Genset {
        genset_model: "CCFJ1600J-WMW",
        engine_model: "8WH25LCD1800-7.5",
        kwe: 1600.0,
        rpm: 750.0,
        engine_length_m: 5.136,
        engine_width_m: 1.710,
        engine_height_m: 2.829,
        engine_mass_t: 21.0,
    },
    Genset {
        genset_model: "CCFJ1800J-WMW",
        engine_model: "8WH25LCD2000-1",
        kwe: 1800.0,
        rpm: 1000.0,
        engine_length_m: 5.136,
        engine_width_m: 1.710,
        engine_height_m: 2.829,
        engine_mass_t: 21.0,
    },
    Genset {
        genset_model: "CCFJ2000J-WMW",
        engine_model: "8WH25LCD2200-1",
        kwe: 2000.0,
        rpm: 1000.0,
        engine_length_m: 5.136,
        engine_width_m: 1.710,
        engine_height_m: 2.829,
        engine_mass_t: 21.0,
    },
    Genset {
        genset_model: "CCFJ2200J-WMW",
        engine_model: "8WH25LCD2450-1",
        kwe: 2200.0,
        rpm: 1000.0,
        engine_length_m: 5.136,
        engine_width_m: 1.710,
        engine_height_m: 2.829,
        engine_mass_t: 21.0,
    },
];

/// The default set when no electrical load is given: the smallest 8-cylinder WH20 set.
pub const DEFAULT_GENSET: Genset = GAS_GENSETS[2];
pub const MAX_GENSETS: i64 = 6;

/// The set and count for an electrical load, with one set in reserve (N+1): the fewest kWe installed
/// such that N-1 sets carry the load, N >= 2; equal installed power goes to fewer sets. `None` when
/// even six of the largest set cannot carry it, or the load is not a positive finite number.
pub fn select_gensets(load_kwe: f64) -> Option<(Genset, i64)> {
    if !(load_kwe.is_finite() && load_kwe > 0.0) {
        return None;
    }
    let mut best: Option<(Genset, i64)> = None;
    for n in 2..=MAX_GENSETS {
        for g in GAS_GENSETS.iter() {
            if (n - 1) as f64 * g.kwe + 1e-9 >= load_kwe {
                let better = match best {
                    None => true,
                    Some((b, bn)) => (n as f64 * g.kwe) < (bn as f64 * b.kwe) - 1e-9,
                };
                if better {
                    best = Some((*g, n));
                }
                break; // the smallest set for this n; larger ones only add kWe
            }
        }
    }
    best
}

/// What has to fit in the stern block, and how much room each item needs.
/// `AftArrangementInputs::new(beam, depth)` gives the Python dataclass defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct AftArrangementInputs {
    /// Moulded beam (m).
    pub beam: f64,
    /// Moulded depth: main deck above keel (m).
    pub depth: f64,

    /// Requested number of thrusters. A maximum, not a promise: `compute` fits what the beam takes.
    pub n_thrusters: i64,
    pub thruster_envelope_m: f64,
    pub thruster_clear_m: f64,
    pub thruster_room_length_m: f64,

    pub n_battery_units: i64,
    pub n_accom_units: i64,
    pub container_length_m: f64,
    pub container_width_m: f64,
    pub container_height_m: f64,
    pub container_side_gap_m: f64,
    pub container_gap_m: f64,
    pub container_foundation_m: f64,

    pub wheelhouse_length_m: f64,
    pub wheelhouse_width_m: f64,
    pub wheelhouse_height_m: f64,
    pub wheelhouse_gap_m: f64,
    pub wheelhouse_eye_above_floor_m: f64,
    /// Height above keel the helmsman's eye must clear when raised. `None`: nothing to see over.
    pub sightline_obstruction_m: Option<f64>,
    pub sightline_margin_m: f64,
    pub wheelhouse_max_lift_m: f64,

    /// `"battery_electric"` (default) or `"gas_fuelled"` (issue #180). See `PROPULSION_OPTIONS`.
    pub propulsion: String,
    /// Electrical load the gas gensets must carry (kWe). `Some`: `compute` picks the set and count
    /// from `GAS_GENSETS` (N+1). `None`: the genset fields below are used as given.
    pub electrical_load_kwe: Option<f64>,
    pub n_gensets: i64,
    pub genset_model: String,
    pub genset_kwe: f64,
    pub genset_length_m: f64,
    pub genset_width_m: f64,
    pub genset_height_m: f64,
    pub genset_engine_mass_t: f64,
    /// Walkway between sets and to the shell (m). ESTIMATE.
    pub genset_clear_m: f64,
    /// Maintenance space fore and aft of the sets (m). ESTIMATE.
    pub engine_room_end_clear_m: f64,
    /// Gas valve unit room forward of the engine room (IGF Code: its own space). ESTIMATE length.
    pub gvu_room_length_m: f64,
}

impl AftArrangementInputs {
    pub fn new(beam: f64, depth: f64) -> Self {
        Self {
            beam,
            depth,
            n_thrusters: DEFAULT_N_THRUSTERS,
            thruster_envelope_m: DEFAULT_THRUSTER_ENVELOPE_M,
            thruster_clear_m: DEFAULT_THRUSTER_CLEAR_M,
            thruster_room_length_m: DEFAULT_THRUSTER_ROOM_LENGTH_M,
            n_battery_units: DEFAULT_BATTERY_UNITS_GAS,
            n_accom_units: DEFAULT_ACCOM_UNITS_GAS,
            container_length_m: ISO40_LENGTH_M,
            container_width_m: ISO40_WIDTH_M,
            container_height_m: ISO40_HEIGHT_HC_M,
            container_side_gap_m: DEFAULT_CONTAINER_SIDE_GAP_M,
            container_gap_m: DEFAULT_CONTAINER_GAP_M,
            container_foundation_m: DEFAULT_CONTAINER_FOUNDATION_M,
            wheelhouse_length_m: DEFAULT_WHEELHOUSE_LENGTH_M,
            wheelhouse_width_m: DEFAULT_WHEELHOUSE_WIDTH_M,
            wheelhouse_height_m: DEFAULT_WHEELHOUSE_HEIGHT_M,
            wheelhouse_gap_m: DEFAULT_WHEELHOUSE_GAP_M,
            wheelhouse_eye_above_floor_m: DEFAULT_WHEELHOUSE_EYE_ABOVE_FLOOR_M,
            sightline_obstruction_m: None,
            sightline_margin_m: DEFAULT_SIGHTLINE_MARGIN_M,
            wheelhouse_max_lift_m: DEFAULT_WHEELHOUSE_MAX_LIFT_M,
            propulsion: "battery_electric".to_string(),
            electrical_load_kwe: None,
            n_gensets: 2,
            genset_model: DEFAULT_GENSET.genset_model.to_string(),
            genset_kwe: DEFAULT_GENSET.kwe,
            genset_length_m: DEFAULT_GENSET.length_m(),
            genset_width_m: DEFAULT_GENSET.width_m(),
            genset_height_m: DEFAULT_GENSET.height_m(),
            genset_engine_mass_t: DEFAULT_GENSET.engine_mass_t,
            genset_clear_m: 1.0,
            engine_room_end_clear_m: 1.2,
            gvu_room_length_m: 2.0,
        }
    }

    pub fn is_gas_fuelled(&self) -> bool {
        self.propulsion == "gas_fuelled"
    }

    /// `__post_init__`: the `ValueError` message on failure.
    pub fn validate(&self) -> Result<(), String> {
        if self.beam <= 0.0 || self.depth <= 0.0 {
            return Err("beam and depth must be positive".to_string());
        }
        if self.n_thrusters < 1 {
            return Err("n_thrusters must be >= 1".to_string());
        }
        if !PROPULSION_OPTIONS.contains(&self.propulsion.as_str()) {
            return Err(format!(
                "propulsion must be one of ('battery_electric', 'gas_fuelled'), got '{}'",
                self.propulsion
            ));
        }
        if self.is_gas_fuelled() {
            if self.n_battery_units != 0 {
                return Err(format!(
                    "a gas_fuelled block carries no battery units (n_battery_units={})",
                    self.n_battery_units
                ));
            }
            if self.electrical_load_kwe.is_none() && self.n_gensets < 2 {
                return Err("a gas_fuelled block needs n_gensets >= 2 (one in reserve)".to_string());
            }
            if let Some(l) = self.electrical_load_kwe
                && select_gensets(l).is_none()
            {
                return Err(format!(
                    "no set in GAS_GENSETS carries an electrical load of {} kWe with one in reserve",
                    fixed(l, 1)
                ));
            }
        }
        Ok(())
    }
}

/// The stern block, dimensioned. All x are metres forward of the stern (x = 0).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AftArrangementResult {
    pub length_m: f64,
    /// Longitudinal zones: (label, x_aft, x_fwd).
    pub zones: Vec<(String, f64, f64)>,
    pub thruster_offsets_m: Vec<f64>,
    pub thruster_room_length_m: f64,
    pub container_x0_m: f64,
    pub container_x1_m: f64,
    pub container_total_width_m: f64,
    pub container_top_above_keel_m: f64,
    pub container_labels: Vec<String>,
    pub container_unit_width_m: f64,
    pub container_side_gap_m: f64,
    pub container_rows: i64,
    pub container_per_row: i64,
    pub container_row_length_m: f64,
    pub container_row_gap_m: f64,
    pub wheelhouse_x0_m: f64,
    pub wheelhouse_x1_m: f64,
    pub wheelhouse_floor_lowered_m: f64,
    pub wheelhouse_top_lowered_m: f64,
    pub wheelhouse_floor_raised_m: f64,
    pub wheelhouse_top_raised_m: f64,
    pub wheelhouse_lift_m: f64,
    pub wheelhouse_width_m: f64,
    pub wheelhouse_height_m: f64,
    pub block_top_lowered_m: f64,
    pub checks: Vec<Check>,
    pub warnings: Vec<Text>,

    // --- Issue #180: gas-fuelled stern (zeros, empty or "rows" for battery_electric) ---
    pub propulsion: String,
    /// "rows" (abreast, in rows aft of the cab) or "beside_wheelhouse".
    pub container_layout: String,
    /// Every unit's footprint: (label, x_aft, x_fwd, y_centre), y + = port.
    pub container_units: Vec<(String, f64, f64, f64)>,
    pub engine_room_x0_m: f64,
    pub engine_room_x1_m: f64,
    pub gvu_room_x0_m: f64,
    pub gvu_room_x1_m: f64,
    pub n_gensets: i64,
    pub genset_model: String,
    pub genset_engine_model: String,
    pub genset_kwe: f64,
    pub installed_kwe: f64,
    pub genset_offsets_m: Vec<f64>,
    pub genset_x0_m: f64,
    pub genset_length_m: f64,
    pub genset_width_m: f64,
    pub genset_height_m: f64,
    /// Published engine dry mass of all sets (t); alternators not included.
    pub genset_engine_mass_t: f64,
    pub below_deck_length_m: f64,
    pub deck_length_m: f64,
}

impl AftArrangementResult {
    /// The Python `ok` property.
    pub fn ok(&self) -> bool {
        all_ok(&self.checks)
    }
}

/// An exception the original Python raises out of `compute`. The type name and message match
/// CPython 3.12 (the version CI runs), because both are compared. CPython 3.14 words the zero-divisor message "division by zero"; 3.12 and 3.13
/// say "float floor division by zero".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PyException {
    ValueError(String),
    ZeroDivisionError(String),
    OverflowError(String),
}

impl PyException {
    /// The Python class name.
    pub fn type_name(&self) -> &'static str {
        match self {
            PyException::ValueError(_) => "ValueError",
            PyException::ZeroDivisionError(_) => "ZeroDivisionError",
            PyException::OverflowError(_) => "OverflowError",
        }
    }

    pub fn message(&self) -> &str {
        match self {
            PyException::ValueError(m)
            | PyException::ZeroDivisionError(m)
            | PyException::OverflowError(m) => m,
        }
    }
}

/// Python `int(x)` on a float, for a value that is only ever compared with small integers:
/// a finite result saturates at the `i64` limits, which cannot change a comparison with them.
/// NaN raises `ValueError` and infinity `OverflowError`, with CPython's messages.
fn py_int(x: f64) -> Result<i64, PyException> {
    if x.is_nan() {
        Err(PyException::ValueError(
            "cannot convert float NaN to integer".to_string(),
        ))
    } else if x.is_infinite() {
        Err(PyException::OverflowError(
            "cannot convert float infinity to integer".to_string(),
        ))
    } else {
        Ok(x as i64)
    }
}

/// Python `int(a // b)`: `ZeroDivisionError` for a zero divisor, then `py_int`.
fn py_int_floor_div(a: f64, b: f64) -> Result<i64, PyException> {
    if b == 0.0 {
        return Err(PyException::ZeroDivisionError(
            "float floor division by zero".to_string(),
        ));
    }
    py_int(py_floor_div(a, b))
}

/// The decimal digits of a whole-valued finite float, exactly as Python prints the `int`.
/// Python ints do not saturate, so `round(3e18)` prints in full.
fn whole_float_digits(x: f64) -> String {
    let bits = x.to_bits();
    let negative = bits >> 63 == 1;
    let exp_bits = ((bits >> 52) & 0x7ff) as i32;
    let frac = bits & ((1u64 << 52) - 1);
    let (mantissa, exp) = if exp_bits == 0 {
        (frac, -1074)
    } else {
        (frac | (1u64 << 52), exp_bits - 1075)
    };
    // A whole value has `exp >= 0` or a mantissa with enough trailing zeros to shift out.
    let (mantissa, exp) = if exp < 0 {
        (mantissa >> (-exp).min(63) as u32, 0)
    } else {
        (mantissa, exp)
    };
    // Little-endian base-10^9 limbs of `mantissa * 2^exp`.
    let mut limbs: Vec<u32> = Vec::new();
    let mut m = mantissa;
    while m > 0 {
        limbs.push((m % 1_000_000_000) as u32);
        m /= 1_000_000_000;
    }
    for _ in 0..exp {
        let mut carry = 0u32;
        for limb in limbs.iter_mut() {
            let v = *limb * 2 + carry;
            *limb = v % 1_000_000_000;
            carry = v / 1_000_000_000;
        }
        if carry > 0 {
            limbs.push(carry);
        }
    }
    let mut out = String::new();
    if negative && !limbs.is_empty() {
        out.push('-');
    }
    match limbs.split_last() {
        None => out.push('0'),
        Some((top, rest)) => {
            out.push_str(&top.to_string());
            for limb in rest.iter().rev() {
                out.push_str(&format!("{limb:09}"));
            }
        }
    }
    out
}

/// The one message for an integer that no 64-bit field can hold (the integer range rule:
/// Python holds a count of any size, a 64-bit field cannot).
pub const INT_OVERFLOW_MESSAGE: &str = "integer does not fit in 64 bits";

fn int_overflow() -> PyException {
    PyException::OverflowError(INT_OVERFLOW_MESSAGE.to_string())
}

/// Python `-(-n // d)` for `d >= 1`: the ceiling of `n / d`, without negating `n`.
fn ceil_div(n: i64, d: i64) -> i64 {
    debug_assert!(d >= 1);
    let q = n.div_euclid(d);
    if n.rem_euclid(d) > 0 {
        // The remainder is positive, so `q < n / d <= i64::MAX`: this cannot overflow.
        q.saturating_add(1)
    } else {
        q
    }
}

/// Python `float(n + delta)` for an integer that feeds only a float product. Python's integer is
/// exact, so the sum is taken in `i128` (it cannot overflow there) and rounded once, to nearest
/// even, as Python's int-to-float conversion does.
fn wide_f64(n: i64, delta: i64) -> f64 {
    (i128::from(n).saturating_add(i128::from(delta))) as f64
}

/// Python `max(a, b)`: the first of two equal items.
fn py_max(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// Python `min(a, b)`: the first of two equal items.
fn py_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// Python `a // b` on floats (CPython `float_floor_div`). Not `floor(a / b)`: it works from
/// `fmod`, so `1.0 // 0.1` is 9.0 although `1.0 / 0.1` rounds to 10.0.
fn py_floor_div(a: f64, b: f64) -> f64 {
    let modulo = libm::fmod(a, b);
    let mut div = (a - modulo) / b;
    // A remainder of the wrong sign moves the quotient down one. A zero remainder needs nothing.
    if modulo != 0.0 && (b < 0.0) != (modulo < 0.0) {
        div -= 1.0;
    }
    if div != 0.0 {
        let floored = libm::floor(div);
        if div - floored > 0.5 {
            floored + 1.0
        } else {
            floored
        }
    } else {
        libm::copysign(0.0, a / b)
    }
}

/// Build the stern block from its equipment. `Err` is a `ValueError` message.
pub fn compute(inp: &AftArrangementInputs) -> Result<AftArrangementResult, PyException> {
    inp.validate().map_err(PyException::ValueError)?;
    let gas = inp.is_gas_fuelled();
    let n_units = inp
        .n_battery_units
        .checked_add(inp.n_accom_units)
        .ok_or_else(int_overflow)?;
    let uw = inp.container_width_m;
    let gap = inp.container_side_gap_m;

    // Gas-fuelled: the genset set and count, from the load when one is given.
    let (g_model, g_engine, g_kwe, g_len, g_wid, g_hgt, g_mass, n_gen) = if gas {
        match inp.electrical_load_kwe.and_then(select_gensets) {
            Some((g, n)) => (
                g.genset_model.to_string(),
                g.engine_model.to_string(),
                g.kwe,
                g.length_m(),
                g.width_m(),
                g.height_m(),
                g.engine_mass_t,
                n,
            ),
            None => (
                inp.genset_model.clone(),
                String::new(),
                inp.genset_kwe,
                inp.genset_length_m,
                inp.genset_width_m,
                inp.genset_height_m,
                inp.genset_engine_mass_t,
                inp.n_gensets,
            ),
        }
    } else {
        (String::new(), String::new(), 0.0, 0.0, 0.0, 0.0, 0.0, 0)
    };
    let g_engine = if gas && g_engine.is_empty() {
        GAS_GENSETS
            .iter()
            .find(|g| g.genset_model == g_model)
            .map(|g| g.engine_model.to_string())
            .unwrap_or_default()
    } else {
        g_engine
    };

    // Gas-fuelled: can the units stand BESIDE the wheelhouse? One row across the beam, the cab in
    // the middle: n units + the cab = n + 1 items, so n + 2 walkways.
    let beside_w = n_units as f64 * uw + inp.wheelhouse_width_m + wide_f64(n_units, 2) * gap;
    let beside = gas && n_units > 0 && beside_w <= inp.beam + 1e-9;

    // How many units go abreast, and how many rows that needs.
    let per_row_fit = py_int_floor_div(inp.beam - gap, uw + gap)?;
    let mut n_per_row = core::cmp::max(1, core::cmp::min(n_units, per_row_fit));
    // `-(-n_units // n_per_row)`: ceiling division, written without negating (`-i64::MIN`
    // overflows). `n_per_row` is at least 1, so `div_euclid` is floor division here.
    let mut n_rows = ceil_div(n_units, n_per_row);
    if beside {
        n_per_row = n_units;
        n_rows = 1;
    }

    // Longitudinal build-up, stern (x = 0) forward.
    let mut zones: Vec<(String, f64, f64)> = Vec::new();
    let thr_x0 = 0.0;
    let thr_x1 = thr_x0 + inp.thruster_room_length_m;
    zones.push(("THRUSTER ROOM".to_string(), thr_x0, thr_x1));
    let x = thr_x1;

    // Below deck, gas-fuelled only: engine room (sets abreast, maintenance space fore and aft), then
    // the gas valve unit room. They sit under the deck items and cost length only where they run
    // further forward.
    let (mut er_x0, mut er_x1, mut gvu_x0, mut gvu_x1) = (0.0, 0.0, 0.0, 0.0);
    let mut below_len = thr_x1;
    if gas {
        er_x0 = thr_x1;
        er_x1 = er_x0 + g_len + 2.0 * inp.engine_room_end_clear_m;
        gvu_x0 = er_x1;
        gvu_x1 = gvu_x0 + inp.gvu_room_length_m;
        zones.push(("ENGINE ROOM (GAS GENSETS)".to_string(), er_x0, er_x1));
        zones.push(("GVU ROOM".to_string(), gvu_x0, gvu_x1));
        below_len = gvu_x1;
    }

    let cont_x0 = x + inp.container_foundation_m;
    // `int(round(...))`: round half to even. The integer is unbounded in Python, so print it whole.
    let feet_rounded = libm::rint(inp.container_length_m / 0.3048);
    if !feet_rounded.is_finite() {
        py_int(feet_rounded)?;
    }
    let feet = whole_float_digits(feet_rounded);
    let (cont_x1, wh_x0, wh_x1);
    if beside {
        // The cab stands at the forward end of the units, between them.
        cont_x1 = cont_x0 + inp.container_length_m;
        zones.push((
            format!("ACCOM {feet} FT BESIDE WHEELHOUSE ({n_units} UNITS)"),
            cont_x0,
            cont_x1,
        ));
        wh_x1 = cont_x1;
        wh_x0 = wh_x1 - inp.wheelhouse_length_m;
        zones.push(("WHEELHOUSE (LIFTING)".to_string(), wh_x0, wh_x1));
    } else {
        cont_x1 = cont_x0
            + n_rows as f64 * inp.container_length_m
            + wide_f64(n_rows, -1) * inp.container_gap_m;
        zones.push((
            format!("{feet} FT UNITS ({n_per_row} ABREAST x {n_rows})"),
            cont_x0,
            cont_x1,
        ));
        wh_x0 = cont_x1 + inp.wheelhouse_gap_m;
        wh_x1 = wh_x0 + inp.wheelhouse_length_m;
        zones.push(("WHEELHOUSE (LIFTING)".to_string(), wh_x0, wh_x1));
    }
    let deck_len = wh_x1;
    // Battery-electric: the deck items set the length (0.1.0). Gas-fuelled: the longer layer.
    let length = if gas {
        py_max(deck_len, below_len)
    } else {
        wh_x1
    };

    // Transverse: thrusters equally spaced, containers side by side.
    let pitch = inp.thruster_envelope_m + inp.thruster_clear_m;
    let usable = inp.beam - 2.0 * inp.thruster_clear_m;
    let n_fit = if pitch > 0.0 {
        py_int_floor_div(usable + inp.thruster_clear_m, pitch)?
    } else {
        0
    };
    let n_thr = core::cmp::max(1, core::cmp::min(inp.n_thrusters, n_fit));
    let span = wide_f64(n_thr, -1) * pitch;
    let offsets: Vec<f64> = (0..n_thr).map(|i| -span / 2.0 + i as f64 * pitch).collect();
    let thruster_beam_needed = span + inp.thruster_envelope_m + 2.0 * inp.thruster_clear_m;
    let requested_beam_needed = wide_f64(inp.n_thrusters, -1) * pitch
        + inp.thruster_envelope_m
        + 2.0 * inp.thruster_clear_m;

    let cont_w = if beside {
        beside_w
    } else {
        n_per_row as f64 * uw + wide_f64(n_per_row, 1) * gap
    };
    let mut labels: Vec<String> = Vec::new();
    for _ in 0..inp.n_battery_units.max(0) {
        labels.push("BATTERY 40 FT".to_string());
    }
    for _ in 0..inp.n_accom_units.max(0) {
        labels.push("ACCOM 40 FT".to_string());
    }

    // Every unit's footprint, so a drawing places what the model counted.
    let mut units: Vec<(String, f64, f64, f64)> = Vec::new();
    if beside {
        let n_port = ceil_div(n_units, 2);
        let hw = inp.wheelhouse_width_m / 2.0;
        for (k, lab) in labels.iter().enumerate() {
            let k = k as i64;
            let (side, j) = if k < n_port {
                (1.0, k)
            } else {
                (-1.0, k - n_port)
            };
            let yc = side * (hw + gap + uw / 2.0 + j as f64 * (uw + gap));
            units.push((lab.clone(), cont_x0, cont_x1, yc));
        }
    } else {
        let y0 = cont_w / 2.0 - gap;
        for (k, lab) in labels.iter().enumerate() {
            let k = k as i64;
            let (row, col) = (k.div_euclid(n_per_row), k.rem_euclid(n_per_row));
            let ux0 = cont_x0 + row as f64 * (inp.container_length_m + inp.container_gap_m);
            units.push((
                lab.clone(),
                ux0,
                ux0 + inp.container_length_m,
                y0 - col as f64 * (uw + gap) - uw / 2.0,
            ));
        }
    }

    // Gensets abreast across the engine room.
    let mut g_off: Vec<f64> = Vec::new();
    let mut g_beam_needed = 0.0;
    if gas {
        let g_pitch = g_wid + inp.genset_clear_m;
        let g_span = wide_f64(n_gen, -1) * g_pitch;
        g_off = (0..n_gen)
            .map(|i| -g_span / 2.0 + i as f64 * g_pitch)
            .collect();
        g_beam_needed = n_gen as f64 * g_wid + wide_f64(n_gen, 1) * inp.genset_clear_m;
    }

    // Vertical: containers on deck; the wheelhouse stows at deck level.
    let deck = inp.depth;
    let cont_top = deck + inp.container_foundation_m + inp.container_height_m;

    let wh_floor_lo = deck;
    let wh_top_lo = wh_floor_lo + inp.wheelhouse_height_m;

    let mut lift = 0.0;
    if let Some(obstruction) = inp.sightline_obstruction_m {
        // The eye must sit `margin` above the tallest thing forward of it.
        let eye_lo = wh_floor_lo + inp.wheelhouse_eye_above_floor_m;
        let needed = obstruction + inp.sightline_margin_m;
        lift = py_max(0.0, needed - eye_lo);
    }
    let lift_used = py_min(lift, inp.wheelhouse_max_lift_m);
    let wh_floor_hi = wh_floor_lo + lift_used;
    let wh_top_hi = wh_floor_hi + inp.wheelhouse_height_m;

    let mut checks = alloc::vec![
        Check::new(
            "thrusters_fit_beam",
            thruster_beam_needed <= inp.beam,
            text!(
                "check-thrusters-fit-beam",
                "{count} x {envelope} m thrusters need {needed} m across a {beam} m beam",
                count = n_thr,
                envelope = Arg::fixed(inp.thruster_envelope_m, 2),
                needed = Arg::fixed(thruster_beam_needed, 2),
                beam = Arg::fixed(inp.beam, 2),
            ),
        ),
        Check::with_severity(
            "requested_thruster_count_fits",
            n_thr >= inp.n_thrusters,
            text!(
                "check-requested-thruster-count-fits",
                "{fit} of {requested} requested thrusters fit; {requested} would need {needed} m of {beam} m beam",
                fit = n_thr,
                requested = inp.n_thrusters,
                needed = Arg::fixed(requested_beam_needed, 2),
                beam = Arg::fixed(inp.beam, 2),
            ),
            "warning",
        ),
        Check::new(
            "containers_fit_beam",
            cont_w <= inp.beam,
            text!(
                "check-containers-fit-beam",
                "{count} x 40 ft units abreast need {needed} m across a {beam} m beam",
                count = n_per_row,
                needed = Arg::fixed(cont_w, 2),
                beam = Arg::fixed(inp.beam, 2),
            ),
        ),
        Check::with_severity(
            "units_fit_in_one_row",
            n_rows == 1,
            text!(
                "check-units-fit-in-one-row",
                "{units} units in {rows} row(s) of {per_row}; a second row adds {added} m to the stern block and \
                 comes straight off the cargo zone",
                units = n_units,
                rows = n_rows,
                per_row = n_per_row,
                added = Arg::fixed(inp.container_length_m + inp.container_gap_m, 2),
            ),
            "warning",
        ),
        Check::new(
            "wheelhouse_fits_beam",
            inp.wheelhouse_width_m <= inp.beam,
            text!(
                "check-wheelhouse-fits-beam",
                "wheelhouse {width} m wide on a {beam} m beam",
                width = Arg::fixed(inp.wheelhouse_width_m, 2),
                beam = Arg::fixed(inp.beam, 2),
            ),
        ),
    ];
    let mut warnings: Vec<Text> = Vec::new();
    if gas {
        let msg = if beside {
            text!(
                "check-accom-beside-wheelhouse",
                "{units} x 40 ft units beside a {wheelhouse} m wheelhouse need {needed} m of {beam} m beam",
                units = n_units,
                wheelhouse = Arg::fixed(inp.wheelhouse_width_m, 2),
                needed = Arg::fixed(beside_w, 2),
                beam = Arg::fixed(inp.beam, 2),
            )
        } else {
            text!(
                "check-accom-beside-wheelhouse-rows",
                "{units} x 40 ft units beside a {wheelhouse} m wheelhouse need {needed} m of {beam} m beam\
                 ; they go in a row aft of the cab instead, which lengthens the block",
                units = n_units,
                wheelhouse = Arg::fixed(inp.wheelhouse_width_m, 2),
                needed = Arg::fixed(beside_w, 2),
                beam = Arg::fixed(inp.beam, 2),
            )
        };
        checks.push(Check::with_severity(
            "accom_beside_wheelhouse",
            beside,
            msg,
            "warning",
        ));
        checks.push(Check::new(
            "gensets_fit_beam",
            g_beam_needed <= inp.beam + 1e-9,
            text!(
                "check-gensets-fit-beam",
                "{count} x {width} m gensets abreast need {needed} m across a {beam} m beam",
                count = n_gen,
                width = Arg::fixed(g_wid, 2),
                needed = Arg::fixed(g_beam_needed, 2),
                beam = Arg::fixed(inp.beam, 2),
            ),
        ));
        checks.push(Check::with_severity(
            "gensets_fit_under_deck",
            g_hgt + 1.0 <= inp.depth,
            text!(
                "check-gensets-fit-under-deck",
                "genset {height} m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth {depth} m",
                height = Arg::fixed(g_hgt, 2),
                depth = Arg::fixed(inp.depth, 2),
            ),
            "warning",
        ));
        if let Some(l) = inp.electrical_load_kwe {
            checks.push(Check::new(
                "gensets_carry_load_with_one_in_reserve",
                wide_f64(n_gen, -1) * g_kwe + 1e-9 >= l,
                text!(
                    "check-gensets-carry-load-with-one-in-reserve",
                    "{running} of {count} x {set_kwe} kWe sets carry {load} kWe",
                    running = n_gen - 1,
                    count = n_gen,
                    set_kwe = Arg::fixed(g_kwe, 0),
                    load = Arg::fixed(l, 1),
                ),
            ));
        }
        warnings.push(Text::fixed(
            "warn-aft-gas-stern-estimates",
            "Gas-fuelled stern block: the genset envelopes beyond the published engine (alternator, \
             base frame) and the engine-room and GVU-room clearances are ESTIMATES (no vendor GA). \
             The exhaust uptake and the fuel vent mast are not in the air draft (assumed lowerable). \
             Fuel is taken from the cargo boil-off (IGC Code chapter 16); no separate fuel tank is in \
             the block.",
        ));
    }
    if let Some(obstruction) = inp.sightline_obstruction_m {
        let cleared = (wh_floor_hi + inp.wheelhouse_eye_above_floor_m)
            >= (obstruction + inp.sightline_margin_m - 1e-9);
        // A warning, not a gate: see the Python file.
        checks.push(Check::with_severity(
            "wheelhouse_sees_over_cargo",
            cleared,
            text!(
                "check-wheelhouse-sees-over-cargo",
                "raised eye {eye} m >= obstruction {obstruction} m + {margin} m margin (lift used {lift} m of {max_lift} m)",
                eye = Arg::fixed(wh_floor_hi + inp.wheelhouse_eye_above_floor_m, 2),
                obstruction = Arg::fixed(obstruction, 2),
                margin = Arg::fixed(inp.sightline_margin_m, 2),
                lift = Arg::fixed(lift_used, 2),
                max_lift = Arg::fixed(inp.wheelhouse_max_lift_m, 2),
            ),
            "warning",
        ));
        if lift > inp.wheelhouse_max_lift_m + 1e-9 {
            warnings.push(text!(
                "warn-aft-sightline-lift",
                "Sightline needs {lift} m of lift; the column gives {max_lift} m. Raise the column or \
                 lower the cargo top.",
                lift = Arg::fixed(lift, 2),
                max_lift = Arg::fixed(inp.wheelhouse_max_lift_m, 2),
            ));
        }
    }

    Ok(AftArrangementResult {
        length_m: length,
        zones,
        thruster_offsets_m: offsets,
        thruster_room_length_m: inp.thruster_room_length_m,
        container_x0_m: cont_x0,
        container_x1_m: cont_x1,
        container_total_width_m: cont_w,
        container_top_above_keel_m: cont_top,
        container_labels: labels,
        container_unit_width_m: inp.container_width_m,
        container_side_gap_m: inp.container_side_gap_m,
        container_rows: n_rows,
        container_per_row: n_per_row,
        container_row_length_m: inp.container_length_m,
        container_row_gap_m: inp.container_gap_m,
        wheelhouse_x0_m: wh_x0,
        wheelhouse_x1_m: wh_x1,
        wheelhouse_floor_lowered_m: wh_floor_lo,
        wheelhouse_top_lowered_m: wh_top_lo,
        wheelhouse_floor_raised_m: wh_floor_hi,
        wheelhouse_top_raised_m: wh_top_hi,
        wheelhouse_lift_m: lift_used,
        wheelhouse_width_m: inp.wheelhouse_width_m,
        wheelhouse_height_m: inp.wheelhouse_height_m,
        block_top_lowered_m: py_max(cont_top, wh_top_lo),
        checks,
        warnings,
        propulsion: inp.propulsion.clone(),
        container_layout: if beside { "beside_wheelhouse" } else { "rows" }.to_string(),
        container_units: units,
        engine_room_x0_m: er_x0,
        engine_room_x1_m: er_x1,
        gvu_room_x0_m: gvu_x0,
        gvu_room_x1_m: gvu_x1,
        n_gensets: n_gen,
        genset_model: g_model,
        genset_engine_model: g_engine,
        genset_kwe: g_kwe,
        installed_kwe: n_gen as f64 * g_kwe,
        genset_offsets_m: g_off,
        genset_x0_m: if gas {
            er_x0 + inp.engine_room_end_clear_m
        } else {
            0.0
        },
        genset_length_m: g_len,
        genset_width_m: g_wid,
        genset_height_m: g_hgt,
        genset_engine_mass_t: n_gen as f64 * g_mass,
        below_deck_length_m: below_len,
        deck_length_m: deck_len,
    })
}

#[cfg(test)]
mod tests {
    use super::py_floor_div;

    /// Expected values are printed by CPython 3.14 (`a // b`). The sign of a zero is part of
    /// the answer, so the comparison is on the bits.
    #[track_caller]
    fn check(a: f64, b: f64, expected: f64) {
        let got = py_floor_div(a, b);
        assert_eq!(
            got.to_bits(),
            expected.to_bits(),
            "{a} // {b}: got {got}, Python gives {expected}"
        );
    }

    #[test]
    fn positive_operands() {
        check(1.0, 0.1, 9.0);
        check(7.0, 2.0, 3.0);
        check(6.0, 2.0, 3.0);
        check(0.0, 2.0, 0.0);
    }

    #[test]
    fn a_remainder_of_the_wrong_sign_moves_the_quotient_down() {
        check(-7.0, 2.0, -4.0);
        check(7.0, -2.0, -4.0);
        check(-0.5, 2.0, -1.0);
        check(-16.2, 3.93, -5.0);
        check(16.2, -3.93, -5.0);
    }

    #[test]
    fn a_remainder_of_the_right_sign_does_not() {
        check(-7.0, -2.0, 3.0);
        check(-6.0, 2.0, -3.0);
        check(6.0, -2.0, -3.0);
        check(1.5, -0.5, -3.0);
        check(-1.5, 0.5, -3.0);
    }

    /// When `(a - mod) / b` lands a hair under a whole number, Python rounds it up.
    #[test]
    fn a_quotient_just_under_a_whole_number_is_rounded_up() {
        check(17.567, -2.7, -7.0);
        check(-17.567, 2.7, -7.0);
    }

    #[test]
    fn a_zero_quotient_takes_the_sign_of_a_over_b() {
        check(1.66, 1.86, 0.0);
        check(0.0, 2.0, 0.0);
        check(0.0, -2.0, -0.0);
        check(-0.0, 2.0, -0.0);
        check(-0.0, -2.0, 0.0);
    }
}
