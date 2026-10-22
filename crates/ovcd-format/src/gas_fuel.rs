//! The gas consumption and the boil-off check of a gas-fuelled Option (issue #464).
//!
//! For a Power Source `gas_fuelled` the `powering` block gains the gas the gensets burn at the service
//! speed of each environment, the result gains a `boil_off` block (the heat leak into the Type C
//! tanks at the upper ambient design temperature of IGC 7.2, over the latent heat of the cargo), and
//! the checks gain two warnings: the boil-off against the gas used (IGC 7.1.3 allows no venting), and
//! the cargo as a fuel (IGC 16.1: methane only, other gases if the Administration accepts them, 16.9.1).
//! See `ovcd_core::gas_fuel` for the data and its sources.

use crate::text_ids::TextIds;
use crate::vdesign::DesignOption;
use ovcd_core::checks::Check;
use ovcd_core::gas_fuel::{
    ALTERNATOR_EFFICIENCY, GAS_LCV_KJ_PER_KG, INSULATION_CONDUCTIVITY_W_PER_MK,
    UPPER_AMBIENT_AIR_C, boil_off_kg_per_h, fuel_gas_kj_per_kwh, gas_use_kg_per_h,
    methane_latent_heat_kj_per_kg, tank_heat_leak_kw,
};
use ovcd_core::text;
use ovcd_core::text::Arg;
use serde_json::{Value, json};

fn str_input<'a>(option: &'a DesignOption, name: &str) -> Option<&'a str> {
    option.input(name).and_then(|i| i.value.as_str())
}

/// Add the gas consumption, the `boil_off` block and the two checks to the result of a gas-fuelled
/// Option. Nothing is added for any other Power Source.
pub(crate) fn add_gas_fuel(option: &DesignOption, v: &mut Value) {
    if str_input(option, "power_source") != Some("gas_fuelled") || !v["powering"].is_object() {
        return;
    }
    // The gensets of the stern block: the electric power they give (kWe each) and their number. The
    // load of a genset is the need over the sets that run (as many as the need asks for, at least
    // one). Without a genset table the Installed Power stands for one set.
    let aft = &v["drawing"]["aft_arrangement"];
    let (set_kwe, n_sets) = match (aft["genset_kwe"].as_f64(), aft["n_gensets"].as_f64()) {
        (Some(k), Some(n)) if k > 0.0 && n >= 1.0 => (k, n),
        _ => (
            v["powering"]["installed_power_kw"].as_f64().unwrap_or(0.0),
            1.0,
        ),
    };
    // The gas the gensets burn at the service speed of each environment.
    let mut use_by_environment: Vec<(String, f64)> = Vec::new();
    if let Some(list) = v["powering"]["environments"].as_array_mut() {
        for e in list {
            let needed = e["power_at_service_kw"].as_f64().unwrap_or(0.0);
            let running = if set_kwe > 0.0 {
                (needed / set_kwe).ceil().clamp(1.0, n_sets) * set_kwe
            } else {
                0.0
            };
            let kg_h = gas_use_kg_per_h(needed, running);
            let load = if running > 0.0 {
                (needed / running).min(1.0)
            } else {
                0.0
            };
            e["fuel_gas_kg_per_h"] = crate::eval::num(kg_h);
            e["fuel_gas_kj_per_kwh"] = crate::eval::num(fuel_gas_kj_per_kwh(load));
            use_by_environment.push((e["name"].as_str().unwrap_or("?").to_string(), kg_h));
        }
    }
    v["powering"]["fuel_gas"] = json!({
        "lower_calorific_value_kj_per_kg": crate::eval::num(GAS_LCV_KJ_PER_KG),
        "alternator_efficiency": crate::eval::num(ALTERNATOR_EFFICIENCY),
        "basis": "Wartsila 20DF Product Guide 1/2018 p. 3-1 (6L20DF, gas mode, 160 kW per cylinder), a published dual-fuel genset of the same size class: a stand-in Estimate until a maker's sheet for the chosen genset is found",
    });

    let cargo = str_input(option, "cargo").unwrap_or("");
    let mut checks: Vec<Check> = Vec::new();
    // IGC 16.1 and 16.9.1: the cargo as a fuel.
    checks.push(Check::with_severity(
        "cargo as fuel",
        cargo == "LNG",
        if cargo == "LNG" {
            text!(
                "check-cargo-as-fuel-lng",
                "methane (LNG) vapour may fuel the gensets (IGC Code 16.1)"
            )
        } else {
            text!(
                "check-cargo-as-fuel-other",
                "only methane (LNG) vapour may fuel an engine in a machinery space of category A (IGC Code 16.1); another cargo gas needs the Administration's acceptance (16.9.1)"
            )
        },
        "warning",
    ));

    // The boil-off: for LNG in insulated tanks.
    let tank = &v["drawing"]["tank"];
    let n_tanks = v["geometry"]["n_tanks_total"].as_f64();
    let dims = (
        tank["tank_steel_outer_diameter_m"].as_f64(),
        tank["tank_outer_diameter_m"].as_f64(),
        tank["cylinder_length_m"].as_f64(),
        tank["insulation_thickness_mm"].as_f64(),
        n_tanks,
    );
    let boil = match dims {
        (Some(ds), Some(d_o), Some(len), Some(t_mm), Some(n)) if cargo == "LNG" && t_mm > 0.0 => {
            let cargo_c = ovcd_core::cargo::get_cargo(cargo).map_or(-162.0, |c| c.temperature);
            let delta = UPPER_AMBIENT_AIR_C - cargo_c;
            let per_tank_kw = tank_heat_leak_kw(
                ds / 2.0,
                d_o / 2.0,
                len,
                delta,
                INSULATION_CONDUCTIVITY_W_PER_MK,
            );
            let latent = methane_latent_heat_kj_per_kg(cargo_c);
            let kw = n * per_tank_kw;
            Some((kw, boil_off_kg_per_h(kw, latent), latent, cargo_c))
        }
        _ => None,
    };
    match boil {
        Some((kw, boil_kg_h, latent, cargo_c)) => {
            // The service condition that burns the least gas: the boil-off continues in it.
            let least = use_by_environment
                .iter()
                .cloned()
                .reduce(|a, b| if b.1 < a.1 { b } else { a });
            let use_kg_h = least.as_ref().map_or(0.0, |l| l.1);
            v["boil_off"] = json!({
                "evaluated": true,
                "heat_leak_kw": crate::eval::num(kw),
                "boil_off_kg_per_h": crate::eval::num(boil_kg_h),
                "latent_heat_kj_per_kg": crate::eval::num(latent),
                "cargo_temperature_c": crate::eval::num(cargo_c),
                "ambient_air_c": crate::eval::num(UPPER_AMBIENT_AIR_C),
                "conductivity_w_per_mk": crate::eval::num(INSULATION_CONDUCTIVITY_W_PER_MK),
                "gas_use_kg_per_h": crate::eval::num(use_kg_h),
                "basis": "Conduction through the insulation at the upper ambient design air temperature (IGC Code 7.2), over the latent heat of methane (NIST saturation table); domes, saddles and pipes are not in the heat leak. The conductivity is an Estimate.",
            });
            checks.push(Check::with_severity(
                "boil-off within gas use",
                boil_kg_h <= use_kg_h,
                if boil_kg_h <= use_kg_h {
                    text!(
                        "check-boil-off-within-gas-use",
                        "boil-off {boil} kg/h <= gas use {used} kg/h at the service speed",
                        boil = Arg::fixed(boil_kg_h, 0),
                        used = Arg::fixed(use_kg_h, 0),
                    )
                } else {
                    text!(
                        "check-boil-off-exceeds-gas-use",
                        "boil-off {boil} kg/h > gas use {used} kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3)",
                        boil = Arg::fixed(boil_kg_h, 0),
                        used = Arg::fixed(use_kg_h, 0),
                    )
                },
                "warning",
            ));
        }
        None => {
            let reason = if cargo != "LNG" {
                "the boil-off is worked out for LNG only (the latent heat of methane is the data held)"
            } else {
                "the tank has no insulation, or the result has no tank dimensions"
            };
            v["boil_off"] = json!({ "evaluated": false, "reason": reason });
        }
    }

    let mut ids = TextIds::default();
    ids.checks(&checks);
    let listed: Vec<Value> = checks
        .iter()
        .map(
            |c| json!({ "name": c.name, "ok": c.ok, "message": c.message, "severity": c.severity }),
        )
        .collect();
    let new_ids = ids.finish(&Value::Array(listed.clone()));
    if let Some(o) = v.as_object_mut() {
        if let Some(Value::Array(all)) = o.get_mut("checks") {
            all.extend(listed);
        }
        if let (Some(Value::Object(have)), Value::Object(add)) = (o.get_mut("text_ids"), new_ids) {
            for (k, val) in add {
                have.entry(k).or_insert(val);
            }
        }
    }
}
