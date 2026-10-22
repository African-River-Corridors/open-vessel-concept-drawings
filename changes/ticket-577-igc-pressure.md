<!-- unreleased -->
- Design change (issue #577): the default design pressure of fully refrigerated LPG is the IGC 4.23.1.2 minimum of each design's own tank (Po = 0.2 + A C (rho_r)^1.5 MPa), not the single 0.55 MPa of #522. The Defaults Profile row holds `none`, as the dome rules do, and the core works the value out from the tank's outer diameter and length. It needs no iteration, because neither depends on the wall. A 60 m hull takes about 0.35 MPa, an 85 m hull 0.47 MPa and a 110 m hull 0.54 MPa (9 % nickel steel), so short tanks get a thinner wall and more cargo. The input `tank_design_pressure_mpa` now accepts `none`. A file that stores a pressure keeps it; a file saved before keeps its own.

<!-- design-impact -->
### ticket/577-igc-pressure (2026-10-22)

Between `main 2bd64c3` and `this branch cc0d426`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep gas carriage LPG_butane fully_refrigerated | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.96 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.97 m of 6.00 m | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[22].message` | freeboard 3.05 m at the carried draught 2.95 m >= required 0.3 m | freeboard 3.07 m at the carried draught 2.93 m >= required 0.3 m | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[23].message` | GM 1.45 m (fluid), max GZ 1.09 m at 36° | GM 1.47 m (fluid), max GZ 1.11 m at 36° | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[24].message` | GM 1.44 m (fluid), max GZ 1.09 m at 36° | GM 1.46 m (fluid), max GZ 1.10 m at 36° | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[25].message` | GM 5.42 m (fluid), max GZ 1.65 m at 26° | GM 5.56 m (fluid), max GZ 1.66 m at 26° | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[26].message` | GM 5.46 m (fluid), max GZ 1.64 m at 26° | GM 5.60 m (fluid), max GZ 1.66 m at 26° | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[3].message` | freeboard 3.05 m >= required 0.3 m | freeboard 3.07 m >= required 0.3 m | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.96 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.97 m of 6.00 m | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[22].message` | freeboard 3.06 m at the carried draught 2.94 m >= required 0.3 m | freeboard 3.09 m at the carried draught 2.91 m >= required 0.3 m | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[23].message` | GM 1.46 m (fluid), max GZ 1.10 m at 36° | GM 1.49 m (fluid), max GZ 1.12 m at 36° | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[24].message` | GM 1.45 m (fluid), max GZ 1.10 m at 36° | GM 1.48 m (fluid), max GZ 1.11 m at 36° | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[25].message` | GM 5.42 m (fluid), max GZ 1.65 m at 26° | GM 5.63 m (fluid), max GZ 1.67 m at 26° | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[26].message` | GM 5.46 m (fluid), max GZ 1.64 m at 26° | GM 5.67 m (fluid), max GZ 1.67 m at 26° | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[3].message` | freeboard 3.06 m >= required 0.3 m | freeboard 3.09 m >= required 0.3 m | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.96 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.97 m of 6.00 m | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[22].message` | freeboard 3.10 m at the carried draught 2.90 m >= required 0.3 m | freeboard 3.13 m at the carried draught 2.87 m >= required 0.3 m | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[23].message` | GM 1.49 m (fluid), max GZ 1.12 m at 36° | GM 1.52 m (fluid), max GZ 1.14 m at 36° | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[24].message` | GM 1.48 m (fluid), max GZ 1.12 m at 36° | GM 1.51 m (fluid), max GZ 1.13 m at 36° | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[25].message` | GM 5.42 m (fluid), max GZ 1.65 m at 26° | GM 5.63 m (fluid), max GZ 1.67 m at 26° | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[26].message` | GM 5.46 m (fluid), max GZ 1.64 m at 26° | GM 5.67 m (fluid), max GZ 1.67 m at 26° | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[3].message` | freeboard 3.10 m >= required 0.3 m | freeboard 3.13 m >= required 0.3 m | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `loading_conditions.conditions[2].draught_fwd_m` | 0.659937267646 | 0.616548668958 | -6.57 % |
| sweep gas carriage LPG_mix fully_refrigerated | `stability.conditions[2].draught_fwd_m` | 0.659937267646 | 0.616548668958 | -6.57 % |
| sweep gas carriage LPG_propane fully_refrigerated | `loading_conditions.conditions[2].draught_fwd_m` | 0.659937267646 | 0.616548668958 | -6.57 % |
| sweep gas carriage LPG_propane fully_refrigerated | `stability.conditions[2].draught_fwd_m` | 0.659937267646 | 0.616548668958 | -6.57 % |
| sweep gas carriage LPG_mix fully_refrigerated | `loading_conditions.conditions[3].draught_fwd_m` | 0.699399778504 | 0.656126019518 | -6.19 % |
| sweep gas carriage LPG_mix fully_refrigerated | `stability.conditions[3].draught_fwd_m` | 0.699399778504 | 0.656126019518 | -6.19 % |
| sweep gas carriage LPG_propane fully_refrigerated | `loading_conditions.conditions[3].draught_fwd_m` | 0.699399778504 | 0.656126019518 | -6.19 % |
| sweep gas carriage LPG_propane fully_refrigerated | `stability.conditions[3].draught_fwd_m` | 0.699399778504 | 0.656126019518 | -6.19 % |
| sweep gas carriage LPG_butane fully_refrigerated | `loading_conditions.conditions[2].draught_fwd_m` | 0.659937267646 | 0.631027442165 | -4.38 % |
| sweep gas carriage LPG_butane fully_refrigerated | `stability.conditions[2].draught_fwd_m` | 0.659937267646 | 0.631027442165 | -4.38 % |
| sweep gas carriage LPG_butane fully_refrigerated | `loading_conditions.conditions[3].draught_fwd_m` | 0.699399778504 | 0.670566461121 | -4.12 % |
| sweep gas carriage LPG_butane fully_refrigerated | `stability.conditions[3].draught_fwd_m` | 0.699399778504 | 0.670566461121 | -4.12 % |
| sweep gas carriage LPG_mix fully_refrigerated | `loading_conditions.conditions[3].criteria[5].value` | 5.46166547222 | 5.66938141668 | +3.8 % |
| sweep gas carriage LPG_mix fully_refrigerated | `loading_conditions.conditions[3].gm_fluid_m` | 5.46166547222 | 5.66938141668 | +3.8 % |
| sweep gas carriage LPG_mix fully_refrigerated | `stability.conditions[3].criteria[5].value` | 5.46166547222 | 5.66938141668 | +3.8 % |
| sweep gas carriage LPG_mix fully_refrigerated | `stability.conditions[3].gm_fluid_m` | 5.46166547222 | 5.66938141668 | +3.8 % |
| sweep gas carriage LPG_propane fully_refrigerated | `loading_conditions.conditions[3].criteria[5].value` | 5.46166547222 | 5.66938141668 | +3.8 % |
| sweep gas carriage LPG_propane fully_refrigerated | `loading_conditions.conditions[3].gm_fluid_m` | 5.46166547222 | 5.66938141668 | +3.8 % |
| sweep gas carriage LPG_propane fully_refrigerated | `stability.conditions[3].criteria[5].value` | 5.46166547222 | 5.66938141668 | +3.8 % |
| ... | 2153 more changed quantities | | | |
