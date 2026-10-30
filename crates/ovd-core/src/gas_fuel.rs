//! The gas a gas-fuelled Power Source burns, and the boil-off of the Type C tanks that could feed it
//! (issue #464, spec 0007 "Powering").
//!
//! Gas consumption is the fuel-gas consumption of a published dual-fuel genset of the same size class
//! as the catalogue's sets: the Wartsila 20DF Product Guide, issue 1/2018 (17 September 2018), Section
//! 3, printed p. 3-1, 6L20DF in gas mode at 160 kW per cylinder and 1000 rpm: 8048, 8219, 8326 and
//! 8862 kJ/kWh of shaft output at 100, 85, 75 and 50 % load. Note 4 (p. 3-3): ISO 15550 conditions,
//! gas lower calorific value 49,620 kJ/kg, tolerance 5 %. It stands in for the Weichai sets of
//! `aft_arrangement`, whose source gives no gas consumption. The guide is not in the rule-source
//! library (its notice forbids copying); the values and the hash of the copy read are in
//! `gas-genset/NOTES.md` there. It is an Estimate until a maker's sheet for the chosen set is found.
//!
//! The boil-off is the heat that leaks through the insulation at the upper ambient design air
//! temperature of IGC Code 7.2 (45 C), over the latent heat of the cargo at its storage temperature.
//! The latent heat of methane is from the NIST saturation table (`type-c/cargo/NIST_saturation_
//! methane.tsv`). It is worked out for LNG only: IGC 16.1 makes methane (LNG) the one cargo whose
//! vapour may fuel an engine in a machinery space of category A (16.9.1 admits other gases if the
//! Administration accepts them).

use libm::log;

/// The lower calorific value of the gas fuel (kJ/kg): Wartsila 20DF Product Guide 1/2018, p. 3-3,
/// Note 4.
pub const GAS_LCV_KJ_PER_KG: f64 = 49_620.0;

/// The efficiency of the alternator (an Estimate): the middle of the 0.95 to 0.96 the Wartsila 20DF
/// leaflet of 09.2017 gives (p. 2).
pub const ALTERNATOR_EFFICIENCY: f64 = 0.955;

/// Fuel-gas consumption (kJ per kWh of shaft output) at a load (share of the rating), ascending by
/// load: Wartsila 20DF Product Guide 1/2018, p. 3-1, 6L20DF, gas mode, 160 kW per cylinder.
const FUEL_GAS_KJ_PER_KWH: [(f64, f64); 4] = [
    (0.50, 8862.0),
    (0.75, 8326.0),
    (0.85, 8219.0),
    (1.00, 8048.0),
];

/// The fuel-gas consumption at `load` (kJ/kWh of shaft output), interpolated between the four loads
/// of the guide and held at its ends.
pub fn fuel_gas_kj_per_kwh(load: f64) -> f64 {
    let t = &FUEL_GAS_KJ_PER_KWH;
    if load <= t[0].0 {
        return t[0].1;
    }
    for w in t.windows(2) {
        if load <= w[1].0 {
            let f = (load - w[0].0) / (w[1].0 - w[0].0);
            return w[0].1 + f * (w[1].1 - w[0].1);
        }
    }
    t[t.len() - 1].1
}

/// The gas the gensets burn (kg/h) to give `needed_kwe` of the `installed_kwe`: the shaft output is
/// the electric output over the alternator efficiency, at the load of that share of the rating.
pub fn gas_use_kg_per_h(needed_kwe: f64, installed_kwe: f64) -> f64 {
    if needed_kwe <= 0.0 || installed_kwe <= 0.0 {
        return 0.0;
    }
    let shaft_kw = needed_kwe / ALTERNATOR_EFFICIENCY;
    let load = (needed_kwe / installed_kwe).min(1.0);
    shaft_kw * fuel_gas_kj_per_kwh(load) / GAS_LCV_KJ_PER_KG
}

/// The upper ambient design air temperature for normal service (C): IGC Code 7.2.
pub const UPPER_AMBIENT_AIR_C: f64 = 45.0;

/// The thermal conductivity of the insulation (W/m.K), an Estimate: polyurethane foam of 96 kg/m3
/// at the mean of the cargo and the ambient temperature (about 215 K), 0.0266 W/m.K at 200 K and
/// 0.0248 at 230 K (Lee et al., J. Mar. Sci. Eng. 2021, 9, 36, Table 3; the table prints the unit as
/// mW/m.K, which its values and the stainless steel rows show to be W/m.K).
pub const INSULATION_CONDUCTIVITY_W_PER_MK: f64 = 0.026;

/// The latent heat of methane (kJ/kg) at a saturation temperature (C), from the NIST saturation
/// table: enthalpy of the vapour less that of the liquid, interpolated between its 1 K rows.
pub fn methane_latent_heat_kj_per_kg(t_c: f64) -> f64 {
    // (T in C, kJ/kg): -165, -163, -162, -161, -160 of the table.
    const T: [(f64, f64); 5] = [
        (-165.0, 517.09),
        (-163.0, 513.57),
        (-162.0, 511.77),
        (-161.0, 509.94),
        (-160.0, 508.09),
    ];
    if t_c <= T[0].0 {
        return T[0].1;
    }
    for w in T.windows(2) {
        if t_c <= w[1].0 {
            let f = (t_c - w[0].0) / (w[1].0 - w[0].0);
            return w[0].1 + f * (w[1].1 - w[0].1);
        }
    }
    T[T.len() - 1].1
}

/// The heat that leaks into one tank (kW): conduction through a cylinder shell of the insulation
/// (steel radius `r_steel_m`, outer radius `r_outer_m`, length `cylinder_m`) and through two heads
/// taken as one sphere of the same radii, for a temperature difference of `delta_k`. The domes, the
/// saddles and the pipes are not in it.
pub fn tank_heat_leak_kw(
    r_steel_m: f64,
    r_outer_m: f64,
    cylinder_m: f64,
    delta_k: f64,
    conductivity_w_per_mk: f64,
) -> f64 {
    if r_outer_m <= r_steel_m || r_steel_m <= 0.0 {
        return 0.0;
    }
    let pi = core::f64::consts::PI;
    let cylinder = 2.0 * pi * conductivity_w_per_mk * cylinder_m.max(0.0) * delta_k
        / log(r_outer_m / r_steel_m);
    let heads = 4.0 * pi * conductivity_w_per_mk * delta_k / (1.0 / r_steel_m - 1.0 / r_outer_m);
    (cylinder + heads) / 1000.0
}

/// The boil-off (kg/h) of a heat leak `kw` at the latent heat `latent_kj_per_kg`.
pub fn boil_off_kg_per_h(kw: f64, latent_kj_per_kg: f64) -> f64 {
    kw * 3600.0 / latent_kj_per_kg
}
