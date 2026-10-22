<!-- unreleased -->
- Feature (issue #464): a gas-fuelled Option (power source `gas_fuelled`) gets the gas its gensets burn and a boil-off check. Each environment of `powering` gains `fuel_gas_kg_per_h` and `fuel_gas_kj_per_kwh`: the shaft output (the electric need over an alternator efficiency of 0.955) at the load of the sets that run, at the fuel-gas consumption of the Wartsila 20DF Product Guide 1/2018 (p. 3-1, 6L20DF, gas mode: 8048, 8219, 8326 and 8862 kJ/kWh at 100, 85, 75 and 50 % load; gas lower calorific value 49,620 kJ/kg, Note 4 p. 3-3). **This is a stand-in Estimate:** the Weichai source of the genset table gives no gas consumption, and no maker's sheet for those sets was found. A new `boil_off` block holds the heat leak into the LNG tanks (conduction through the insulation at 45 C air, IGC Code 7.2; the conductivity is an Estimate from a polyurethane foam table) over the latent heat of methane (NIST saturation table, 511.77 kJ/kg at -162 C). Two warning checks join the result: "cargo as fuel" (IGC 16.1: methane only; 16.9.1: another cargo gas needs the Administration's acceptance) and "boil-off within gas use" (IGC 7.1.1 and 7.1.3: venting is not acceptable, so boil-off above the gas used needs reliquefaction, pressure accumulation or thermal oxidation). The boil-off is worked out for LNG only. The Weichai genset claim "dual-fuel" is corrected in the code and in the entry of the genset catalogue. A file that does not use a gas-fuelled power source does not move.

<!-- design-impact -->
### ticket/464-gas-consumption (2026-10-22)

Between `main 599e821` and `this branch 494d7f6`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `boil_off.evaluated` | absent | False | changed |
| sweep CN-derived liquid 1 (88x16.3) | `boil_off.reason` | absent | the boil-off is worked out for LNG only (the latent heat of methane is the data held) | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[28].message` | absent | only methane (LNG) vapour may fuel an engine in a machinery space of category A (IGC Code 16.1); another cargo gas needs the Administration's acceptance (16.9.1) | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[28].name` | absent | cargo as fuel | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[28].ok` | absent | False | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[28].severity` | absent | warning | changed |
| sweep CN-derived liquid 1 (88x16.3) | `powering.environments[0].fuel_gas_kg_per_h` | absent | 89.548911849 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `powering.environments[0].fuel_gas_kj_per_kwh` | absent | 8862 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `powering.fuel_gas.alternator_efficiency` | absent | 0.955 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `powering.fuel_gas.basis` | absent | Wartsila 20DF Product Guide 1/2018 p. 3-1 (6L20DF, gas mode, 160 kW per cylinder), a published dual-fuel genset of the same size class: a stand-in Estimate until a maker's sheet for the chosen genset is found | changed |
| sweep CN-derived liquid 1 (88x16.3) | `powering.fuel_gas.lower_calorific_value_kj_per_kg` | absent | 49620 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `boil_off.evaluated` | absent | False | changed |
| sweep CN-derived liquid 2 (130x16.3) | `boil_off.reason` | absent | the boil-off is worked out for LNG only (the latent heat of methane is the data held) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[29].message` | absent | only methane (LNG) vapour may fuel an engine in a machinery space of category A (IGC Code 16.1); another cargo gas needs the Administration's acceptance (16.9.1) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[29].name` | absent | cargo as fuel | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[29].ok` | absent | False | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[29].severity` | absent | warning | changed |
| sweep CN-derived liquid 2 (130x16.3) | `powering.environments[0].fuel_gas_kg_per_h` | absent | 125.485782034 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `powering.environments[0].fuel_gas_kj_per_kwh` | absent | 8714.91159013 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `powering.fuel_gas.alternator_efficiency` | absent | 0.955 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `powering.fuel_gas.basis` | absent | Wartsila 20DF Product Guide 1/2018 p. 3-1 (6L20DF, gas mode, 160 kW per cylinder), a published dual-fuel genset of the same size class: a stand-in Estimate until a maker's sheet for the chosen genset is found | changed |
| sweep CN-derived liquid 2 (130x16.3) | `powering.fuel_gas.lower_calorific_value_kj_per_kg` | absent | 49620 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `boil_off.evaluated` | absent | False | changed |
| sweep CN-derived liquid 3 (130x17.2) | `boil_off.reason` | absent | the boil-off is worked out for LNG only (the latent heat of methane is the data held) | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[28].message` | absent | only methane (LNG) vapour may fuel an engine in a machinery space of category A (IGC Code 16.1); another cargo gas needs the Administration's acceptance (16.9.1) | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[28].name` | absent | cargo as fuel | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[28].ok` | absent | False | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[28].severity` | absent | warning | changed |
| sweep CN-derived liquid 3 (130x17.2) | `powering.environments[0].fuel_gas_kg_per_h` | absent | 129.899255111 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `powering.environments[0].fuel_gas_kj_per_kwh` | absent | 8664.72562546 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `powering.fuel_gas.alternator_efficiency` | absent | 0.955 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `powering.fuel_gas.basis` | absent | Wartsila 20DF Product Guide 1/2018 p. 3-1 (6L20DF, gas mode, 160 kW per cylinder), a published dual-fuel genset of the same size class: a stand-in Estimate until a maker's sheet for the chosen genset is found | changed |
| sweep CN-derived liquid 3 (130x17.2) | `powering.fuel_gas.lower_calorific_value_kj_per_kg` | absent | 49620 | changed |
| sweep default depth gas 90x14 | `boil_off.ambient_air_c` | absent | 45 | changed |
| sweep default depth gas 90x14 | `boil_off.basis` | absent | Conduction through the insulation at the upper ambient design air temperature (IGC Code 7.2), over the latent heat of methane (NIST saturation table); domes, saddles and pipes are not in the heat leak. The conductivity is an Estimate. | changed |
| sweep default depth gas 90x14 | `boil_off.boil_off_kg_per_h` | absent | 176.765923467 | changed |
| sweep default depth gas 90x14 | `boil_off.cargo_temperature_c` | absent | -162 | changed |
| sweep default depth gas 90x14 | `boil_off.conductivity_w_per_mk` | absent | 0.026 | changed |
| sweep default depth gas 90x14 | `boil_off.evaluated` | absent | True | changed |
| sweep default depth gas 90x14 | `boil_off.gas_use_kg_per_h` | absent | 66.6649223077 | changed |
| ... | 1159 more changed quantities | | | |
