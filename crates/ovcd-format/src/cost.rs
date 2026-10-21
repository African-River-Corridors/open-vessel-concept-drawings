//! The capital budget of an Option: the core's `barge_cost::compute`, run where the results are
//! assembled, for a gas barge only: the core models a barge with Type C tanks and an LNG cargo system,
//! so any other Cargo has no budget (`cost` is null). The mass inputs come from the Option's own
//! weight breakdown; the tug, which the engine does not size, keeps the core's defaults. The rates are
//! the Option's cost-rate inputs. Nothing here feeds `simulate`: the budget is an output, kept out of the results hash
//! (`UNHASHED_KEYS`), so saved Designs keep their hashes.

use crate::vdesign::{COST_RATES_BASIS, COST_RATES_BASIS_USER, DesignOption, Source};
use ovcd_core::barge_cost::{BargeCostInputs, BargeCostRates, BargeCostResult, compute_for};
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

/// The masses and which of them come from the design, as `simulate` reports them.
fn masses_json(m: &BargeCostInputs, from_design: &[&str]) -> Value {
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
        .filter(|n| !from_design.contains(n))
        .collect();
    json!({
        "masses": all.iter().map(|(n, v)| ((*n).to_string(), json!(v))).collect::<serde_json::Map<_, _>>(),
        "from_design": from_design,
        "core_default": core_default,
    })
}

/// The masses of a self-propelled gas Option with its own powering (issue #353): the weight
/// breakdown, and its own battery energy and Installed Power in place of a tug's. The tug's hull
/// steel keeps the core's default.
pub fn self_propelled_gas_masses(
    w: &WeightBreakdown,
    battery_kwh: f64,
    propulsion_kw: f64,
) -> Value {
    let mut v = gas_masses(w);
    v["masses"]["battery_kwh"] = json!(battery_kwh);
    v["masses"]["propulsion_kw"] = json!(propulsion_kw);
    for name in ["battery_kwh", "propulsion_kw"] {
        if let Some(list) = v["from_design"].as_array_mut() {
            list.push(json!(name));
        }
        if let Some(list) = v["core_default"].as_array_mut() {
            list.retain(|n| n != name);
        }
    }
    v
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
    let mut v = masses_json(
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
    );
    v["steel"] = steel_json(w);
    v
}

/// The two hull steel figures of a gas barge (issue #401): the lightship uses the top-down one (a
/// rate per cubic metre of moulded volume); the budget prices the bottom-up one (hull plating and
/// structure, wing plate, weather shield). `delta_t` is top-down less bottom-up.
fn steel_json(w: &WeightBreakdown) -> Value {
    json!({
        "top_down_t": w.steel_top_down,
        "bottom_up_t": w.steel_bottom_up,
        "delta_t": w.steel_delta,
    })
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

/// Which rates the budget uses (issue #404): `example` while every rate is the Defaults Profile's
/// (the example file's), `user` once any rate is typed.
fn rates_kind(option: &DesignOption) -> &'static str {
    let typed = RATE_INPUTS.iter().any(|(name, _)| {
        option
            .input(name)
            .is_some_and(|i| i.source != Source::Default)
    });
    if typed { "user" } else { "example" }
}

fn result_json(r: &BargeCostResult, masses: &Value, tug: bool, rates: &str) -> Value {
    // With no tug its hull steel is not costed, so it is not listed (issue #403).
    let mut masses = masses.clone();
    if !tug {
        if let Some(m) = masses["masses"].as_object_mut() {
            m.remove("tug_hull_steel_t");
        }
        if let Some(list) = masses["core_default"].as_array_mut() {
            list.retain(|n| n != "tug_hull_steel_t");
        }
    }
    json!({
        "currency": r.rates.currency,
        "rates": rates,
        "basis": if rates == "user" { COST_RATES_BASIS_USER } else { COST_RATES_BASIS },
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
        "core_default": masses["core_default"],
        "steel": masses["steel"],
    })
}

/// The `cost` of an Option: the budget, or `null` when the results have no masses (a Cargo that is not
/// gas, or a gas hull that holds no arrangement) or the figures cannot be rounded.
pub fn cost_of(option: &DesignOption, results: &Value) -> Value {
    let masses = &results["cost_masses"];
    let (Some(inputs), Some(rates)) = (masses_of(masses), rates_of(option)) else {
        return Value::Null;
    };
    // Only a Formation with a tug costs one (issue #403): a self-propelled vessel has none.
    let tug = has_tug(option);
    match compute_for(&inputs, &rates, tug) {
        Ok(r) => result_json(&r, masses, tug, rates_kind(option)),
        Err(_) => Value::Null,
    }
}

/// Whether the Option's Formation has a tug: every one but `self_propelled`.
fn has_tug(option: &DesignOption) -> bool {
    let kind = option
        .input("formation")
        .and_then(|i| i.value.get("kind"))
        .and_then(Value::as_str);
    kind != Some("self_propelled")
}
