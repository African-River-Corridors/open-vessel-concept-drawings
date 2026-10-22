<!-- unreleased -->
- Propeller Checks (issue #355): three new Rules in the unreleased `ovcd` 2.1.0, each a warning. `ovcd/propulsion/propeller-fits-stern`: the propeller diameter is at most the largest the Stern Form allows. `ovcd/propulsion/immersion-light`: the shaft axis is under the water by at least 0.76 diameters (0.71 ducted) at the lightest aft draught of the loading conditions; not for a tunnel stern; the axis is taken at half a diameter above the base. `ovcd/propulsion/blade-load`: the delivered power of one unit at the Installed Power is at most 400 kW per m2 of disc (500 ducted). The `powering` block of a self-propelled result gets five new keys that these read: `propeller_diameter_m`, `shaft_axis_height_m`, `blade_load_kw_m2`, `blade_load_limit_kw_m2`, `propeller_ducted`. Every self-propelled Option with a specified Propulsor gets a new results hash; no existing value or verdict changes. A default hull shows an immersion warning: its propeller is sized to the design draught, so at the light draught the axis is above the water.
- Evaluation harness (issue #355): `kind: stern` cases keyed by rule id for the Blind Distance, speed, redundancy and propeller Rules of `es-trin-2025`, `solas-v22`, `rheinschpv` and `ovcd`, with an independent oracle (`evaluation/oracle/stern_oracle.py`) and generated boundary cases (the Blind Distance 249 and 251 m, the speed 12.9 and 13.1 km/h, a 110.1 m craft with and without a bow thruster). The engine has a new evaluation entry point, `gas_stability.stern_trace`. 0 false passes and 0 false fails.

<!-- design-impact -->
### ticket/355-propulsor-checks (2026-10-22)

Between `main 263faba` and `this branch 67fb8f4`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived bulk 1 (88x16.3) | `powering.blade_load_kw_m2` | absent | 45.9246945142 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `powering.blade_load_limit_kw_m2` | absent | 500 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `powering.propeller_diameter_m` | absent | 3.44 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `powering.propeller_ducted` | absent | True | changed |
| sweep CN-derived bulk 1 (88x16.3) | `powering.shaft_axis_height_m` | absent | 1.72 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `powering.blade_load_kw_m2` | absent | 64.1155601136 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `powering.blade_load_limit_kw_m2` | absent | 500 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `powering.propeller_diameter_m` | absent | 3.44 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `powering.propeller_ducted` | absent | True | changed |
| sweep CN-derived bulk 2 (130x16.3) | `powering.shaft_axis_height_m` | absent | 1.72 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `powering.blade_load_kw_m2` | absent | 66.727371405 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `powering.blade_load_limit_kw_m2` | absent | 500 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `powering.propeller_diameter_m` | absent | 3.44 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `powering.propeller_ducted` | absent | True | changed |
| sweep CN-derived bulk 3 (130x17.2) | `powering.shaft_axis_height_m` | absent | 1.72 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `powering.blade_load_kw_m2` | absent | 39.633945209 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `powering.blade_load_limit_kw_m2` | absent | 500 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `powering.propeller_diameter_m` | absent | 2.73 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `powering.propeller_ducted` | absent | True | changed |
| sweep CN-derived liquid 1 (88x16.3) | `powering.shaft_axis_height_m` | absent | 1.365 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `powering.blade_load_kw_m2` | absent | 56.4768249549 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `powering.blade_load_limit_kw_m2` | absent | 500 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `powering.propeller_diameter_m` | absent | 2.73 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `powering.propeller_ducted` | absent | True | changed |
| sweep CN-derived liquid 2 (130x16.3) | `powering.shaft_axis_height_m` | absent | 1.365 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `powering.blade_load_kw_m2` | absent | 58.8017949233 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `powering.blade_load_limit_kw_m2` | absent | 500 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `powering.propeller_diameter_m` | absent | 2.73 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `powering.propeller_ducted` | absent | True | changed |
| sweep CN-derived liquid 3 (130x17.2) | `powering.shaft_axis_height_m` | absent | 1.365 | changed |
| sweep box self-propelled 60x10 | `powering.blade_load_kw_m2` | absent | 63.5242556774 | changed |
| sweep box self-propelled 60x10 | `powering.blade_load_limit_kw_m2` | absent | 500 | changed |
| sweep box self-propelled 60x10 | `powering.propeller_diameter_m` | absent | 2.26133508433 | changed |
| sweep box self-propelled 60x10 | `powering.propeller_ducted` | absent | True | changed |
| sweep box self-propelled 60x10 | `powering.shaft_axis_height_m` | absent | 1.13066754217 | changed |
| sweep box self-propelled 90x14 | `powering.blade_load_kw_m2` | absent | 53.8774664691 | changed |
| sweep box self-propelled 90x14 | `powering.blade_load_limit_kw_m2` | absent | 500 | changed |
| sweep box self-propelled 90x14 | `powering.propeller_diameter_m` | absent | 3.16586911807 | changed |
| sweep box self-propelled 90x14 | `powering.propeller_ducted` | absent | True | changed |
| sweep box self-propelled 90x14 | `powering.shaft_axis_height_m` | absent | 1.58293455903 | changed |
| ... | 705 more changed quantities | | | |
