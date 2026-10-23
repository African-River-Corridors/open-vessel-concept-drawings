<!-- unreleased -->
- Propeller Checks (issue #355): three new Rules in the unreleased `ovcd` 2.1.0, each a warning. `ovcd/propulsion/propeller-fits-stern`: the propeller diameter is at most the largest the Stern Form allows. `ovcd/propulsion/immersion-light`: the shaft axis is under the water by at least 0.76 diameters (0.71 ducted) at the lightest aft draught of the loading conditions; not for a tunnel stern; the axis is taken at half a diameter above the base. `ovcd/propulsion/blade-load`: the delivered power of one unit at the Installed Power is at most 400 kW per m2 of disc (500 ducted). The `powering` block of a self-propelled result gets five new keys that these read: `propeller_diameter_m`, `shaft_axis_height_m`, `blade_load_kw_m2`, `blade_load_limit_kw_m2`, `propeller_ducted`. Every self-propelled Option with a specified Propulsor gets a new results hash.
- Design change (issue #355, the maintainer's decision): a default propeller at a transom or pram stern is sized to the light draught, so a new self-propelled hull passes `ovcd/propulsion/immersion-light`. The diameter is the Stern Form's D_max, but no larger than keeps the shaft axis 0.76 D (0.71 D ducted) under the lightest aft draught of the loading conditions, rounded down to 0.05 m. The run is done again with that diameter until it fits (at most four runs), because a smaller propeller makes the stern block lighter and the light draught smaller. A new stored input, `propeller_diameter_basis` (`light_draught`, the default, an estimate; `stern_form`, D_max), keeps D_max for a file saved before. A typed diameter, a tunnel stern and an unspecified Propulsor do not change. A Design that a user changes can still warn. Every new gas Option moves (a transom stern is the gas default; dry and liquid bulk take a tunnel stern and do not move). On a 110 x 11.4 x 6.0 m propane hull: D 3.71 m to 1.05 m, stern block 251 t to 107 t, lightship 1,653 t to 1,508 t, installed power 621 kW to 941 kW (a smaller disc is less efficient), immersion ratio 0.01 (warn) to 1.07 (pass), blade load 0.88 of the limit (pass). A few gas cases still warn, where the light draught falls as fast as the diameter (an LNG hull, the tank head and tank liquid pressure sweep cases, and the gas down-flooding opening case). The Python API reads a left-out key as `light_draught`.
- Evaluation harness (issue #355): `kind: stern` cases keyed by rule id for the Blind Distance, speed, redundancy and propeller Rules of `es-trin-2025`, `solas-v22`, `rheinschpv` and `ovcd`, with an independent oracle (`evaluation/oracle/stern_oracle.py`) and generated boundary cases (the Blind Distance 249 and 251 m, the speed 12.9 and 13.1 km/h, a 110.1 m craft with and without a bow thruster). The engine has a new evaluation entry point, `gas_stability.stern_trace`. 0 false passes and 0 false fails.

<!-- design-impact -->
### ticket/355-propulsor-checks (2026-10-23)

Between `main 9fc7e39` and `this branch 19406c9e`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived bulk 1 (88x16.3) | `powering.blade_load_kw_m2` | absent | 45.9347097366 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `powering.blade_load_limit_kw_m2` | absent | 500 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `powering.propeller_diameter_m` | absent | 3.44 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `powering.propeller_ducted` | absent | True | changed |
| sweep CN-derived bulk 1 (88x16.3) | `powering.shaft_axis_height_m` | absent | 1.72 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `powering.blade_load_kw_m2` | absent | 64.1288506566 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `powering.blade_load_limit_kw_m2` | absent | 500 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `powering.propeller_diameter_m` | absent | 3.44 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `powering.propeller_ducted` | absent | True | changed |
| sweep CN-derived bulk 2 (130x16.3) | `powering.shaft_axis_height_m` | absent | 1.72 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `powering.blade_load_kw_m2` | absent | 66.7412389576 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `powering.blade_load_limit_kw_m2` | absent | 500 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `powering.propeller_diameter_m` | absent | 3.44 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `powering.propeller_ducted` | absent | True | changed |
| sweep CN-derived bulk 3 (130x17.2) | `powering.shaft_axis_height_m` | absent | 1.72 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.54 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.55 m of 6.00 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[22].message` | freeboard 3.01 m at the carried draught 2.69 m >= required 0.3 m | freeboard 3.05 m at the carried draught 2.65 m >= required 0.3 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[23].message` | GM 6.08 m (fluid), max GZ 2.56 m at 29° | GM 6.17 m (fluid), max GZ 2.57 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[24].message` | GM 6.09 m (fluid), max GZ 2.56 m at 29° | GM 6.18 m (fluid), max GZ 2.56 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[25].message` | GM 12.69 m (fluid), max GZ 3.48 m at 28° | GM 13.07 m (fluid), max GZ 3.50 m at 27° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[26].message` | GM 12.78 m (fluid), max GZ 3.48 m at 27° | GM 13.16 m (fluid), max GZ 3.49 m at 27° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[3].message` | freeboard 3.01 m >= required 0.3 m | freeboard 3.05 m >= required 0.3 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `powering.blade_load_kw_m2` | absent | 427.881368069 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `powering.blade_load_limit_kw_m2` | absent | 500 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `powering.propeller_diameter_m` | absent | 1 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `powering.propeller_ducted` | absent | True | changed |
| sweep CN-derived liquid 1 (88x16.3) | `powering.shaft_axis_height_m` | absent | 0.5 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].message` | blind distance ahead of the bow 2800.2 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 6.00 m of 6.00 m | blind distance ahead of the bow 2806.8 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 6.00 m of 6.00 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[23].message` | freeboard 2.56 m at the carried draught 3.14 m >= required 0.3 m | freeboard 2.58 m at the carried draught 3.12 m >= required 0.3 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[24].message` | GM 3.82 m (fluid), max GZ 1.55 m at 26° | GM 3.85 m (fluid), max GZ 1.56 m at 26° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[25].message` | GM 3.82 m (fluid), max GZ 1.55 m at 26° | GM 3.85 m (fluid), max GZ 1.56 m at 26° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[26].message` | GM 10.43 m (fluid), max GZ 2.92 m at 25° | GM 10.62 m (fluid), max GZ 2.92 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[27].message` | GM 10.48 m (fluid), max GZ 2.91 m at 25° | GM 10.67 m (fluid), max GZ 2.92 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[4].message` | freeboard 2.56 m >= required 0.3 m | freeboard 2.58 m >= required 0.3 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `powering.blade_load_kw_m2` | absent | 452.868361704 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `powering.blade_load_limit_kw_m2` | absent | 500 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `powering.propeller_diameter_m` | absent | 1.15 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `powering.propeller_ducted` | absent | True | changed |
| sweep CN-derived liquid 2 (130x16.3) | `powering.shaft_axis_height_m` | absent | 0.575 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `warnings[1]` | The blind distance needs 10.02 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | The blind distance needs 10.03 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| ... | 73601 more changed quantities | | | |
