//! The capital budget of an Option: the core's `barge_cost::compute`, run where the results are
//! assembled. The mass inputs come from the Option's own results where the engine has them (the
//! weight breakdown of a gas barge); the rest keep the core's defaults, or are zero where the item
//! does not exist on the vessel (a dry-bulk barge has no tanks). The rates are the Option's cost-rate
//! inputs. Nothing here feeds `simulate`: the budget is an output, kept out of the results hash
//! (`UNHASHED_KEYS`), so saved Designs keep their hashes.

use crate::vdesign::{COST_RATES_BASIS, DesignOption};
use ovcd_core::barge_cost::{BargeCostInputs, BargeCostRates, BargeCostResult, compute};
use ovcd_core::models::WeightBreakdown;
use serde_json::{Value, json};

/// The 14 numeric rates, as `(input name, how to set it on the core's rates)`. The currency is not an
/// input: it comes from the example rates file.
type SetRate = fn(&mut BargeCostRates, f64);
pub const RATE_INPUTS: [(&str, SetRate); 14] = [
    ("hull_steel_rate", |r, v| r.hull_steel_rate = v),
    ("tank_shell_rate", |r, v| r.tank_shell_rate = v),
    ("tank_supports_rate", |r, v| r.tank_supports_rate = v),
    ("insulation_rate", |r, v| r.insulation_rate = v),
    ("piping_rate", |r, v| r.piping_rate = v),
    ("outfitting_rate", |r, v| r.outfitting_rate = v),
    ("cargo_system_lump", |r, v| r.cargo_system_lump = v),
    ("paint_anodes_misc", |r, v| r.paint_anodes_misc = v),
    ("battery_rate_per_kwh", |r, v| r.battery_rate_per_kwh = v),
    ("propulsion_rate_per_kw", |r, v| {
        r.propulsion_rate_per_kw = v
    }),
    ("wheelhouse_accom_lump", |r, v| r.wheelhouse_accom_lump = v),
    ("tug_outfitting_lump", |r, v| r.tug_outfitting_lump = v),
    ("yard_margin_fraction", |r, v| r.yard_margin_fraction = v),
    ("delivery_lump", |r, v| r.delivery_lump = v),
];

/// The rates of an Option: its cost-rate inputs, with the currency of the example rates. `None` if
/// an input is missing or not a number.
pub fn rates_of(option: &DesignOption) -> Option<BargeCostRates> {
    let mut rates = ovcd_core::barge_cost::example_rates().ok()?;
    for (name, set) in RATE_INPUTS {
        set(&mut rates, option.input(name)?.value.as_f64()?);
    }
    Some(rates)
}

/// Which mass inputs of the core come from where, as `simulate` reports them next to the masses.
fn masses_json(m: &BargeCostInputs, from_design: &[&str], zero: &[&str]) -> Value {
    let all = [
        ("hull_steel_t", m.hull_steel_t),
        ("wing_plate_t", m.wing_plate_t),
        ("weather_shield_t", m.weather_shield_t),
        ("tank_shell_t", m.tank_shell_t),
        ("tank_supports_t", m.tank_supports_t),
        ("insulation_t", m.insulation_t),
        ("piping_machinery_t", m.piping_machinery_t),
        ("outfitting_t", m.outfitting_t),
        ("tug_hull_steel_t", m.tug_hull_steel_t),
        ("battery_kwh", m.battery_kwh),
        ("propulsion_kw", m.propulsion_kw),
    ];
    let core_default: Vec<&str> = all
        .iter()
        .map(|(n, _)| *n)
        .filter(|n| !from_design.contains(n) && !zero.contains(n))
        .collect();
    json!({
        "masses": all.iter().map(|(n, v)| ((*n).to_string(), json!(v))).collect::<serde_json::Map<_, _>>(),
        "from_design": from_design,
        "zero": zero,
        "core_default": core_default,
    })
}

/// The masses of a gas barge: its weight breakdown. The tug (hull steel, battery, propulsion) is a
/// separate vessel that the engine does not size, so those three keep the core's defaults.
pub fn gas_masses(w: &WeightBreakdown) -> Value {
    let m = BargeCostInputs {
        hull_steel_t: w.hull_steel_total,
        wing_plate_t: w.wing_plate,
        weather_shield_t: w.weather_shield,
        tank_shell_t: w.tank_shells,
        tank_supports_t: w.tank_internals_supports,
        insulation_t: w.insulation,
        piping_machinery_t: w.piping_machinery,
        outfitting_t: w.outfitting,
        ..BargeCostInputs::default()
    };
    masses_json(
        &m,
        &[
            "hull_steel_t",
            "wing_plate_t",
            "weather_shield_t",
            "tank_shell_t",
            "tank_supports_t",
            "insulation_t",
            "piping_machinery_t",
            "outfitting_t",
        ],
        &[],
    )
}

/// The masses of a dry-bulk barge. The engine gives no weight breakdown for a hold, so the hull steel,
/// piping and outfitting keep the core's defaults; the items of a gas barge (tanks, supports,
/// insulation, wing plate, weather shield) do not exist on it and are zero.
pub fn dry_bulk_masses() -> Value {
    let m = BargeCostInputs {
        wing_plate_t: 0.0,
        weather_shield_t: 0.0,
        tank_shell_t: 0.0,
        tank_supports_t: 0.0,
        insulation_t: 0.0,
        ..BargeCostInputs::default()
    };
    masses_json(
        &m,
        &[],
        &[
            "wing_plate_t",
            "weather_shield_t",
            "tank_shell_t",
            "tank_supports_t",
            "insulation_t",
        ],
    )
}

fn masses_of(m: &Value) -> Option<BargeCostInputs> {
    let g = |k: &str| m["masses"][k].as_f64();
    Some(BargeCostInputs {
        hull_steel_t: g("hull_steel_t")?,
        wing_plate_t: g("wing_plate_t")?,
        weather_shield_t: g("weather_shield_t")?,
        tank_shell_t: g("tank_shell_t")?,
        tank_supports_t: g("tank_supports_t")?,
        insulation_t: g("insulation_t")?,
        piping_machinery_t: g("piping_machinery_t")?,
        outfitting_t: g("outfitting_t")?,
        tug_hull_steel_t: g("tug_hull_steel_t")?,
        battery_kwh: g("battery_kwh")?,
        propulsion_kw: g("propulsion_kw")?,
    })
}

fn money(i: &ovcd_core::pyint::PyInt) -> Value {
    serde_json::from_str(i.as_str()).expect("a PyInt is valid JSON integer text")
}

fn rows(list: &[(&str, ovcd_core::pyint::PyInt)]) -> Value {
    Value::Array(
        list.iter()
            .map(|(item, amount)| json!({ "item": item, "amount": money(amount) }))
            .collect(),
    )
}

fn result_json(r: &BargeCostResult, masses: &Value) -> Value {
    json!({
        "currency": r.rates.currency,
        "basis": COST_RATES_BASIS,
        "barge": rows(&r.barge_breakdown),
        "tug": rows(&r.tug_breakdown),
        "barge_subtotal": money(&r.barge_subtotal),
        "tug_subtotal": money(&r.tug_subtotal),
        "yard_margin_fraction": r.rates.yard_margin_fraction,
        "yard_margin": money(&r.yard_margin),
        "delivery": r.delivery,
        "unit_total": money(&r.unit_total),
        "checks": crate::eval::checks_json(&r.checks),
        "masses": masses["masses"],
        "from_design": masses["from_design"],
        "zero": masses["zero"],
        "core_default": masses["core_default"],
    })
}

/// The `cost` of an Option: the budget, or `null` when the results have no masses (a gas hull that
/// holds no arrangement) or the figures cannot be rounded.
pub fn cost_of(option: &DesignOption, results: &Value) -> Value {
    let masses = &results["cost_masses"];
    let (Some(inputs), Some(rates)) = (masses_of(masses), rates_of(option)) else {
        return Value::Null;
    };
    match compute(&inputs, &rates) {
        Ok(r) => result_json(&r, masses),
        Err(_) => Value::Null,
    }
}
