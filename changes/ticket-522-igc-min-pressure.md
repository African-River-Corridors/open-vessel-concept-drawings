<!-- unreleased -->
- Design change (issue #522, maintainer decision of 2026-10-22): the default design pressure of fully refrigerated LPG (propane, butane and the mix) is 0.55 MPa, not 0.4 MPa. IGC Code, MSC.370(93), 4.23.1.2 asks for Po = 0.2 + A C (rho_r)^1.5 MPa; the minimum depends on the tank length (0.33 MPa for a 60 m hull, 0.44 to 0.48 MPa for 110 m, 0.54 MPa for butane on 120 m), so one constant cannot be the minimum, and 0.55 MPa covers tanks up to about 45 m (conservative for shorter tanks; the profile basis says so). A saved file that stores a design pressure keeps it. A per-design default is issue #577. New fully refrigerated LPG Options get a slightly thicker wall; the three fully refrigerated sweep cases move.

<!-- design-impact -->
### ticket/522-igc-min-pressure (2026-10-22)

Between `main fc7e931` and `this branch abbf71f`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep gas carriage LPG_butane fully_refrigerated | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.97 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.95 m of 6.00 m | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[21].message` | tank length 37.25 m / inner diameter 7.63 m = 4.88 <= 7.0 | tank length 37.25 m / inner diameter 7.62 m = 4.89 <= 7.0 | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[22].message` | freeboard 3.09 m at the carried draught 2.91 m >= required 0.3 m | freeboard 3.05 m at the carried draught 2.95 m >= required 0.3 m | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[23].message` | GM 1.50 m (fluid), max GZ 1.13 m at 36° | GM 1.45 m (fluid), max GZ 1.10 m at 36° | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[24].message` | GM 1.49 m (fluid), max GZ 1.12 m at 36° | GM 1.45 m (fluid), max GZ 1.09 m at 36° | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[25].message` | GM 5.77 m (fluid), max GZ 1.69 m at 26° | GM 5.43 m (fluid), max GZ 1.64 m at 26° | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[26].message` | GM 5.82 m (fluid), max GZ 1.68 m at 26° | GM 5.47 m (fluid), max GZ 1.64 m at 26° | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[3].message` | freeboard 3.09 m >= required 0.3 m | freeboard 3.05 m >= required 0.3 m | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.97 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.95 m of 6.00 m | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[21].message` | tank length 37.25 m / inner diameter 7.63 m = 4.88 <= 7.0 | tank length 37.25 m / inner diameter 7.62 m = 4.89 <= 7.0 | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[22].message` | freeboard 3.11 m at the carried draught 2.89 m >= required 0.3 m | freeboard 3.07 m at the carried draught 2.93 m >= required 0.3 m | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[23].message` | GM 1.51 m (fluid), max GZ 1.13 m at 36° | GM 1.46 m (fluid), max GZ 1.10 m at 36° | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[24].message` | GM 1.50 m (fluid), max GZ 1.13 m at 36° | GM 1.46 m (fluid), max GZ 1.10 m at 36° | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[25].message` | GM 5.77 m (fluid), max GZ 1.69 m at 26° | GM 5.43 m (fluid), max GZ 1.64 m at 26° | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[26].message` | GM 5.82 m (fluid), max GZ 1.68 m at 26° | GM 5.47 m (fluid), max GZ 1.64 m at 26° | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[3].message` | freeboard 3.11 m >= required 0.3 m | freeboard 3.07 m >= required 0.3 m | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.97 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.95 m of 6.00 m | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[21].message` | tank length 37.25 m / inner diameter 7.63 m = 4.88 <= 7.0 | tank length 37.25 m / inner diameter 7.62 m = 4.89 <= 7.0 | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[22].message` | freeboard 3.15 m at the carried draught 2.85 m >= required 0.3 m | freeboard 3.11 m at the carried draught 2.89 m >= required 0.3 m | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[23].message` | GM 1.54 m (fluid), max GZ 1.15 m at 36° | GM 1.49 m (fluid), max GZ 1.12 m at 36° | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[24].message` | GM 1.54 m (fluid), max GZ 1.15 m at 36° | GM 1.49 m (fluid), max GZ 1.12 m at 36° | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[25].message` | GM 5.77 m (fluid), max GZ 1.69 m at 26° | GM 5.43 m (fluid), max GZ 1.64 m at 26° | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[26].message` | GM 5.82 m (fluid), max GZ 1.68 m at 26° | GM 5.47 m (fluid), max GZ 1.64 m at 26° | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[3].message` | freeboard 3.15 m >= required 0.3 m | freeboard 3.11 m >= required 0.3 m | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `loading_conditions.conditions[2].draught_fwd_m` | 0.582449890263 | 0.653952017041 | +12.3 % |
| sweep gas carriage LPG_butane fully_refrigerated | `stability.conditions[2].draught_fwd_m` | 0.582449890263 | 0.653952017041 | +12.3 % |
| sweep gas carriage LPG_mix fully_refrigerated | `loading_conditions.conditions[2].draught_fwd_m` | 0.582449890263 | 0.653952017041 | +12.3 % |
| sweep gas carriage LPG_mix fully_refrigerated | `stability.conditions[2].draught_fwd_m` | 0.582449890263 | 0.653952017041 | +12.3 % |
| sweep gas carriage LPG_propane fully_refrigerated | `loading_conditions.conditions[2].draught_fwd_m` | 0.582449890263 | 0.653952017041 | +12.3 % |
| sweep gas carriage LPG_propane fully_refrigerated | `stability.conditions[2].draught_fwd_m` | 0.582449890263 | 0.653952017041 | +12.3 % |
| sweep gas carriage LPG_butane fully_refrigerated | `loading_conditions.conditions[3].draught_fwd_m` | 0.622107621598 | 0.693418765958 | +11.5 % |
| sweep gas carriage LPG_butane fully_refrigerated | `stability.conditions[3].draught_fwd_m` | 0.622107621598 | 0.693418765958 | +11.5 % |
| sweep gas carriage LPG_mix fully_refrigerated | `loading_conditions.conditions[3].draught_fwd_m` | 0.622107621598 | 0.693418765958 | +11.5 % |
| sweep gas carriage LPG_mix fully_refrigerated | `stability.conditions[3].draught_fwd_m` | 0.622107621598 | 0.693418765958 | +11.5 % |
| sweep gas carriage LPG_propane fully_refrigerated | `loading_conditions.conditions[3].draught_fwd_m` | 0.622107621598 | 0.693418765958 | +11.5 % |
| sweep gas carriage LPG_propane fully_refrigerated | `stability.conditions[3].draught_fwd_m` | 0.622107621598 | 0.693418765958 | +11.5 % |
| sweep gas carriage LPG_propane fully_refrigerated | `loading_conditions.conditions[0].trim_m` | 0.485692462593 | 0.515445476217 | +6.13 % |
| sweep gas carriage LPG_propane fully_refrigerated | `stability.conditions[0].trim_m` | 0.485692462593 | 0.515445476217 | +6.13 % |
| sweep gas carriage LPG_butane fully_refrigerated | `loading_conditions.conditions[3].criteria[5].value` | 5.81786874375 | 5.46585415334 | -6.05 % |
| sweep gas carriage LPG_butane fully_refrigerated | `loading_conditions.conditions[3].gm_fluid_m` | 5.81786874375 | 5.46585415334 | -6.05 % |
| ... | 2156 more changed quantities | | | |
