//! Bottom-up ATB unit cost (cargo barge + electric pusher tug). Port of `core/barge_cost.py`.
//!
//! Rates are data, not code. The core does no I/O: a user's rates file is read by the caller and
//! passed in as text (`parse_flat_yaml` then `BargeCostRates::from_mapping`). With no rates given,
//! `compute` uses the example rates file, compiled in.
//!
//! Python traps copied here: `sum()` of floats is compensated (Neumaier), `round()` is half to
//! even and returns an `int` of any size (and raises for inf and nan), and `min` returns the first
//! of equal items.

use crate::checks::Check;
use crate::pyint::{self, PyInt, RoundError};
use crate::pystr;
use crate::text;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// `barge_cost_rates.example.yaml`, the illustrative rates (not market quotes).
pub const EXAMPLE_RATES_TEXT: &str =
    include_str!("../../../examples/barge_cost_rates.example.yaml");

/// Mass-model inputs (t, kWh, kW). Defaults are the Python dataclass defaults.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct BargeCostInputs {
    pub hull_steel_t: f64,
    pub wing_plate_t: f64,
    pub weather_shield_t: f64,
    pub tank_shell_t: f64,
    pub tank_supports_t: f64,
    pub insulation_t: f64,
    pub piping_machinery_t: f64,
    pub outfitting_t: f64,
    pub tug_hull_steel_t: f64,
    pub battery_kwh: f64,
    pub propulsion_kw: f64,
}

impl Default for BargeCostInputs {
    fn default() -> Self {
        Self {
            hull_steel_t: 253.2,
            wing_plate_t: 40.3,
            weather_shield_t: 50.1,
            tank_shell_t: 215.6,
            tank_supports_t: 64.7,
            insulation_t: 25.9,
            piping_machinery_t: 32.9,
            outfitting_t: 28.8,
            tug_hull_steel_t: 115.0,
            battery_kwh: 620.0,
            propulsion_kw: 230.0,
        }
    }
}

/// Fabrication and supply rates. No defaults: every rate comes from data you supply.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct BargeCostRates {
    pub hull_steel_rate: f64,
    pub tank_shell_rate: f64,
    pub tank_supports_rate: f64,
    pub insulation_rate: f64,
    pub piping_rate: f64,
    pub outfitting_rate: f64,
    pub cargo_system_lump: f64,
    pub paint_anodes_misc: f64,
    pub battery_rate_per_kwh: f64,
    pub propulsion_rate_per_kw: f64,
    pub wheelhouse_accom_lump: f64,
    pub tug_outfitting_lump: f64,
    pub yard_margin_fraction: f64,
    pub delivery_lump: f64,
    pub currency: String,
}

/// Why `from_mapping` failed: a Python exception class name and its `str()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappingError {
    pub type_name: &'static str,
    pub message: String,
}

impl MappingError {
    fn value(message: String) -> Self {
        Self {
            type_name: "ValueError",
            message,
        }
    }
}

/// A value in a rates mapping: what Python's `float(v)` and `str(v)` are applied to.
#[derive(Debug, Clone, PartialEq)]
pub enum MappingValue {
    Int(i64),
    /// A Python `int` too big for `i64`: its exact decimal digits, as `str(v)` prints them.
    BigInt(String),
    Float(f64),
    Text(String),
}

impl MappingValue {
    /// `float(v)`. The error is the exception Python raises: `OverflowError` for an int too big
    /// for a float, `ValueError` for text that is not a number.
    fn to_float(&self) -> Result<f64, MappingError> {
        match self {
            MappingValue::Int(i) => Ok(*i as f64),
            MappingValue::BigInt(d) => match d.parse::<f64>() {
                Ok(f) if f.is_finite() => Ok(f),
                _ => Err(MappingError {
                    type_name: "OverflowError",
                    message: "int too large to convert to float".to_string(),
                }),
            },
            MappingValue::Float(f) => Ok(*f),
            MappingValue::Text(s) => pystr::parse_float(s).ok_or_else(|| {
                MappingError::value(format!(
                    "could not convert string to float: {}",
                    pystr::repr_str(s)
                ))
            }),
        }
    }

    /// `str(v)`.
    fn to_text(&self) -> String {
        match self {
            MappingValue::Int(i) => i.to_string(),
            MappingValue::BigInt(d) => d.clone(),
            MappingValue::Float(f) => crate::pyfmt::py_str(*f),
            MappingValue::Text(s) => s.clone(),
        }
    }
}

/// The 14 numeric rate names, in the dataclass field order, then `currency`.
const RATE_NAMES: [&str; 14] = [
    "hull_steel_rate",
    "tank_shell_rate",
    "tank_supports_rate",
    "insulation_rate",
    "piping_rate",
    "outfitting_rate",
    "cargo_system_lump",
    "paint_anodes_misc",
    "battery_rate_per_kwh",
    "propulsion_rate_per_kw",
    "wheelhouse_accom_lump",
    "tug_outfitting_lump",
    "yard_margin_fraction",
    "delivery_lump",
];

impl BargeCostRates {
    /// `BargeCostRates.from_mapping`. Errors are the exceptions Python raises.
    pub fn from_mapping(data: &[(String, MappingValue)]) -> Result<Self, MappingError> {
        let is_name = |k: &str| k == "currency" || RATE_NAMES.contains(&k);
        let mut unknown: Vec<&str> = data
            .iter()
            .map(|(k, _)| k.as_str())
            .filter(|k| !is_name(k))
            .collect();
        unknown.sort_unstable();
        unknown.dedup();
        if !unknown.is_empty() {
            return Err(MappingError::value(format!(
                "unknown barge cost rate key(s): {}",
                pystr::repr_str_list(&unknown)
            )));
        }
        let mut missing: Vec<&str> = RATE_NAMES
            .iter()
            .copied()
            .filter(|n| !data.iter().any(|(k, _)| k == n))
            .collect();
        missing.sort_unstable();
        if !missing.is_empty() {
            return Err(MappingError::value(format!(
                "missing barge cost rate key(s): {}",
                pystr::repr_str_list(&missing)
            )));
        }
        // Convert in mapping order, so the first bad value is the one reported.
        let mut rates = Self {
            hull_steel_rate: 0.0,
            tank_shell_rate: 0.0,
            tank_supports_rate: 0.0,
            insulation_rate: 0.0,
            piping_rate: 0.0,
            outfitting_rate: 0.0,
            cargo_system_lump: 0.0,
            paint_anodes_misc: 0.0,
            battery_rate_per_kwh: 0.0,
            propulsion_rate_per_kw: 0.0,
            wheelhouse_accom_lump: 0.0,
            tug_outfitting_lump: 0.0,
            yard_margin_fraction: 0.0,
            delivery_lump: 0.0,
            currency: "USD".to_string(),
        };
        for (key, value) in data {
            if key == "currency" {
                rates.currency = value.to_text();
                continue;
            }
            let x = value.to_float()?;
            match key.as_str() {
                "hull_steel_rate" => rates.hull_steel_rate = x,
                "tank_shell_rate" => rates.tank_shell_rate = x,
                "tank_supports_rate" => rates.tank_supports_rate = x,
                "insulation_rate" => rates.insulation_rate = x,
                "piping_rate" => rates.piping_rate = x,
                "outfitting_rate" => rates.outfitting_rate = x,
                "cargo_system_lump" => rates.cargo_system_lump = x,
                "paint_anodes_misc" => rates.paint_anodes_misc = x,
                "battery_rate_per_kwh" => rates.battery_rate_per_kwh = x,
                "propulsion_rate_per_kw" => rates.propulsion_rate_per_kw = x,
                "wheelhouse_accom_lump" => rates.wheelhouse_accom_lump = x,
                "tug_outfitting_lump" => rates.tug_outfitting_lump = x,
                "yard_margin_fraction" => rates.yard_margin_fraction = x,
                _ => rates.delivery_lump = x,
            }
        }
        Ok(rates)
    }
}

/// `parse_flat_yaml`: `key: value` lines, `#` comments and blank lines; anything else is an
/// error. Values stay text. Pairs come back in file order.
pub fn parse_flat_yaml(text: &str) -> Result<Vec<(String, String)>, String> {
    let mut out: Vec<(String, String)> = Vec::new();
    for (index, raw) in pystr::splitlines(text).into_iter().enumerate() {
        let n = index + 1;
        let before_hash = raw.split('#').next().unwrap_or("");
        let line = pystr::rstrip(before_hash);
        if pystr::strip(line).is_empty() {
            continue;
        }
        let (key, sep, value) = match line.split_once(':') {
            Some((k, v)) => (k, true, v),
            None => (line, false, ""),
        };
        let key = pystr::strip(key);
        let value = pystr::strip(value).trim_matches(['\'', '"']);
        let starts_with_space = raw.chars().next().is_some_and(pystr::is_space);
        if !sep
            || key.is_empty()
            || value.is_empty()
            || starts_with_space
            || !pystr::is_identifier(key)
        {
            return Err(format!(
                "rates file line {n}: expected flat 'key: value', got {}",
                pystr::repr_str(raw)
            ));
        }
        if out.iter().any(|(k, _)| k == key) {
            return Err(format!(
                "rates file line {n}: duplicate key {}",
                pystr::repr_str(key)
            ));
        }
        out.push((key.to_string(), value.to_string()));
    }
    Ok(out)
}

/// `load_rates` for the built-in case (`rates=None`): the compiled-in example rates.
pub fn example_rates() -> Result<BargeCostRates, String> {
    let pairs = parse_flat_yaml(EXAMPLE_RATES_TEXT)?;
    let mapping: Vec<(String, MappingValue)> = pairs
        .into_iter()
        .map(|(k, v)| (k, MappingValue::Text(v)))
        .collect();
    BargeCostRates::from_mapping(&mapping).map_err(|e| e.message)
}

/// Unit cost and its parts. Rounded money is whole currency units: a Python `int`, so [`PyInt`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct BargeCostResult {
    pub barge_breakdown: Vec<(&'static str, PyInt)>,
    pub tug_breakdown: Vec<(&'static str, PyInt)>,
    pub barge_subtotal: PyInt,
    pub tug_subtotal: PyInt,
    pub yard_margin: PyInt,
    pub delivery: f64,
    pub unit_total: PyInt,
    pub checks: Vec<Check>,
    pub rates: BargeCostRates,
}

impl BargeCostResult {
    pub fn ok(&self) -> bool {
        crate::checks::all_ok(&self.checks)
    }
}

/// Python 3.12 `sum()` over floats: Neumaier compensated summation.
fn py_sum(values: &[f64]) -> f64 {
    let mut total = 0.0_f64;
    let mut compensation = 0.0_f64;
    for &x in values {
        let t = total + x;
        if libm::fabs(total) >= libm::fabs(x) {
            compensation += (total - t) + x;
        } else {
            compensation += (x - t) + total;
        }
        total = t;
    }
    if compensation != 0.0 && compensation.is_finite() {
        total += compensation;
    }
    total
}

/// Why `compute` failed: the exception Python's `round()` raises on a result that is nan or infinite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComputeError(pub RoundError);

impl ComputeError {
    /// The Python exception class name (`ValueError` or `OverflowError`).
    pub fn type_name(self) -> &'static str {
        self.0.type_name()
    }

    /// `str(exception)`.
    pub fn message(self) -> &'static str {
        self.0.message()
    }
}

/// Every priced mass is there (issue #404; it tested only the tank shell and the hull steel): the
/// masses every gas barge has are positive, and the ones a design may lack (a wing plate, a weather
/// shield, a battery, propulsion, a tug hull) are zero or more. A NaN fails.
fn masses_present(inp: &BargeCostInputs) -> bool {
    let required = [
        inp.hull_steel_t,
        inp.tank_shell_t,
        inp.tank_supports_t,
        inp.insulation_t,
        inp.piping_machinery_t,
        inp.outfitting_t,
    ];
    let optional = [
        inp.wing_plate_t,
        inp.weather_shield_t,
        inp.tug_hull_steel_t,
        inp.battery_kwh,
        inp.propulsion_kw,
    ];
    required.iter().all(|m| *m > 0.0) && optional.iter().all(|m| *m >= 0.0)
}

fn py_round(x: f64) -> Result<PyInt, ComputeError> {
    pyint::round(x).map_err(ComputeError)
}

/// `compute`: unit cost from the masses in `inp` and the `rates`. Fails where Python's `round()`
/// raises: a nan or infinite figure (a rate of `nan` or `inf`, or a mass so large the cost overflows).
pub fn compute(
    inp: &BargeCostInputs,
    rt: &BargeCostRates,
) -> Result<BargeCostResult, ComputeError> {
    compute_for(inp, rt, true)
}

/// [`compute`] for a Formation with a tug (`tug`) or without one (issue #403). With no tug there is
/// no tug breakdown: the tug's hull steel and outfitting lump are not costed, and the vessel's own
/// battery, propulsion and wheelhouse and accommodation are barge items.
pub fn compute_for(
    inp: &BargeCostInputs,
    rt: &BargeCostRates,
    tug: bool,
) -> Result<BargeCostResult, ComputeError> {
    let mut barge: Vec<(&'static str, f64)> = alloc::vec![
        (
            "hull_steel",
            (inp.hull_steel_t + inp.wing_plate_t + inp.weather_shield_t) * rt.hull_steel_rate,
        ),
        ("type_c_tanks", inp.tank_shell_t * rt.tank_shell_rate),
        ("tank_supports", inp.tank_supports_t * rt.tank_supports_rate),
        ("insulation", inp.insulation_t * rt.insulation_rate),
        ("cargo_system", rt.cargo_system_lump),
        ("piping_machinery", inp.piping_machinery_t * rt.piping_rate),
        ("outfitting", inp.outfitting_t * rt.outfitting_rate),
        ("paint_anodes_misc", rt.paint_anodes_misc),
    ];
    let powering = [
        ("battery", inp.battery_kwh * rt.battery_rate_per_kwh),
        ("propulsion", inp.propulsion_kw * rt.propulsion_rate_per_kw),
        ("wheelhouse_accommodation", rt.wheelhouse_accom_lump),
    ];
    let tug: Vec<(&'static str, f64)> = if tug {
        let mut rows = alloc::vec![("hull_steel", inp.tug_hull_steel_t * rt.hull_steel_rate)];
        rows.extend(powering);
        rows.push(("outfitting", rt.tug_outfitting_lump));
        rows
    } else {
        barge.extend(powering);
        Vec::new()
    };
    let barge_values: Vec<f64> = barge.iter().map(|(_, v)| *v).collect();
    let tug_values: Vec<f64> = tug.iter().map(|(_, v)| *v).collect();
    let b_sub = py_sum(&barge_values);
    let t_sub = py_sum(&tug_values);
    let margin = rt.yard_margin_fraction * (b_sub + t_sub);
    let total = b_sub + t_sub + margin + rt.delivery_lump;

    let checks = alloc::vec![
        Check::new(
            "masses_positive",
            masses_present(inp),
            text!("check-masses-positive", "mass-model inputs present")
        ),
        Check::with_severity(
            "no_reliquefaction_in_scope",
            true,
            text!(
                "check-no-reliquefaction-in-scope",
                "cargo system excludes reliquefaction and nitrogen plant per the \
                 pressure-accumulation operating concept"
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
    // Python builds these in this order, so the first figure that cannot round is the error.
    let round_all =
        |rows: &[(&'static str, f64)]| -> Result<Vec<(&'static str, PyInt)>, ComputeError> {
            rows.iter().map(|(k, v)| Ok((*k, py_round(*v)?))).collect()
        };
    Ok(BargeCostResult {
        barge_breakdown: round_all(&barge)?,
        tug_breakdown: round_all(&tug)?,
        barge_subtotal: py_round(b_sub)?,
        tug_subtotal: py_round(t_sub)?,
        yard_margin: py_round(margin)?,
        delivery: rt.delivery_lump,
        unit_total: py_round(total)?,
        checks,
        rates: rt.clone(),
    })
}
