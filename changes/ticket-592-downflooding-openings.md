<!-- unreleased -->
- Stability (issue #592, spec 0004, IS Code 2008 Part A 2.2.1): the angle of down-flooding can be worked out from the Design's deck openings. The new optional Option input `downflooding_openings` (Loading conditions group) lists them, one row each: `x_m` from the aft end, `y_m` across from the centre line (either side), `height_above_deck_m` above the deck at the side; a row may carry a `name`. With no angle typed, the angle of each loading condition is the heel, from the hull's sections at that condition's free-trim equilibrium, at which the lowest opening is immersed (linear between whole degrees); the areas 0-40 and 30-40 then end there where it is below 40 degrees, as with a typed angle (#282), which still wins. The angle shows in each condition's results as `downflooding_deg` where there is one. The derived input `downflooding_angle` has a third value, `computed`, and the note Rule `is2008/A.2.2.1/downflooding-angle-not-given` shows only where nothing is typed or listed. The default is none listed, so no existing file moves and no hash changes; a Design that lists openings gets shorter areas and may fail. Openings outside the hull (x, y) or below the deck are named. The app shows the count; the openings are typed in the file. `is2008-a` 2.0.0 (unreleased) gets the new gap text in place.

<!-- design-impact -->
### ticket/592-downflooding-openings (2026-10-22)

Between `main 5d5ec18` and `this branch 20625f7`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep downflooding opening gas 90x11.4 | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 5.67 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 1.06 m of 6.00 m | changed |
| sweep downflooding opening gas 90x11.4 | `checks[21].message` | tank length 28.20 m / inner diameter 7.70 m = 3.66 <= 7.0 | tank length 28.20 m / inner diameter 4.06 m = 6.95 <= 7.0 | changed |
| sweep downflooding opening gas 90x11.4 | `checks[22].message` | freeboard 2.78 m at the carried draught 2.22 m >= required 0.3 m | freeboard 3.39 m at the carried draught 1.61 m >= required 0.3 m | changed |
| sweep downflooding opening gas 90x11.4 | `checks[23].message` | GM 2.45 m (fluid), max GZ 1.34 m at 32° | GM 5.00 m (fluid), max GZ 2.20 m at 37° | changed |
| sweep downflooding opening gas 90x11.4 | `checks[24].message` | GM 2.45 m (fluid), max GZ 1.33 m at 32° | GM 5.03 m (fluid), max GZ 2.19 m at 37° | changed |
| sweep downflooding opening gas 90x11.4 | `checks[25].message` | GM 6.76 m (fluid), max GZ 1.87 m at 25° | GM 9.16 m (fluid), max GZ 2.33 m at 28° | changed |
| sweep downflooding opening gas 90x11.4 | `checks[26].message` | GM 6.83 m (fluid), max GZ 1.86 m at 25° | GM 9.29 m (fluid), max GZ 2.33 m at 27° | changed |
| sweep downflooding opening gas 90x11.4 | `checks[3].message` | freeboard 2.78 m >= required 0.3 m | freeboard 3.39 m >= required 0.3 m | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[0].kind` | wing_ballast | peak_ballast | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[0].name` | WB1P | FP1C | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[0].side` | port | centre | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[1].capacity_m3` | 320.539802048 | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[1].capacity_t` | 320.539802048 | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[1].kind` | wing_ballast | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[1].lcg_m` | 52.596 | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[1].name` | WB1S | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[1].side` | starboard | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[1].tcg_m` | 5.24500875146 | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[1].vcg_m` | 3.01541155193 | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[2].capacity_m3` | 239.4 | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[2].capacity_t` | 239.4 | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[2].kind` | peak_ballast | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[2].lcg_m` | 83.9 | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[2].name` | FP1C | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[2].side` | centre | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[2].tcg_m` | 0 | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[2].vcg_m` | 2.5 | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.conditions[0].criteria[1].name` | Area 0–40° | Area 0–34.8° | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.conditions[0].criteria[2].name` | Area 30–40° | Area 30–34.8° | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.conditions[0].downflooding_deg` | absent | 34.7988719551 | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.conditions[1].criteria[1].name` | Area 0–40° | Area 0–35.1° | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.conditions[1].criteria[2].name` | Area 30–40° | Area 30–35.1° | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.conditions[1].downflooding_deg` | absent | 35.065448435 | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.conditions[2].downflooding_deg` | absent | 50.2795980479 | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.conditions[3].downflooding_deg` | absent | 50.6777378335 | changed |
| sweep downflooding opening gas 90x11.4 | `stability.conditions[0].criteria[1].name` | Area 0–40° | Area 0–34.8° | changed |
| sweep downflooding opening gas 90x11.4 | `stability.conditions[0].criteria[2].name` | Area 30–40° | Area 30–34.8° | changed |
| sweep downflooding opening gas 90x11.4 | `stability.conditions[0].downflooding_deg` | absent | 34.7988719551 | changed |
| sweep downflooding opening gas 90x11.4 | `stability.conditions[1].criteria[1].name` | Area 0–40° | Area 0–35.1° | changed |
| sweep downflooding opening gas 90x11.4 | `stability.conditions[1].criteria[2].name` | Area 30–40° | Area 30–35.1° | changed |
| ... | 747 more changed quantities | | | |
