<!-- unreleased -->
- Stability (issue #592, spec 0004, IS Code 2008 Part A 2.2.1): the angle of down-flooding can be worked out from the Design's deck openings. The new optional Option input `downflooding_openings` (Loading conditions group) lists them, one row each: `x_m` from the aft end, `y_m` across from the centre line (either side), `height_above_deck_m` above the deck at the side; a row may carry a `name`. With no angle typed, the angle of each loading condition is the heel, from the hull's sections at that condition's free-trim equilibrium, at which the lowest opening is immersed (linear between whole degrees); the areas 0-40 and 30-40 then end there where it is below 40 degrees, as with a typed angle (#282), which still wins. The worked angle shows in each condition's results as `downflooding_deg` where there is one (a typed angle is the Option's input and is not repeated). The derived input `downflooding_angle` has a third value, `computed`, and the note Rule `is2008/A.2.2.1/downflooding-angle-not-given` shows only where nothing is typed or listed. The default is none listed, so no existing file moves and no hash changes; a Design that lists openings gets shorter areas and may fail. Openings outside the hull (x, y) or below the deck are named. The app shows the count; the openings are typed in the file. `is2008-a` 2.0.0 (unreleased) gets the new gap text in place.

<!-- design-impact -->
### ticket/592-downflooding-openings (2026-10-22)

Between `main 8b47128` and `this branch c2f1bbc`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep downflooding opening gas 90x11.4 | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 5.67 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 1.06 m of 6.00 m | changed |
| sweep downflooding opening gas 90x11.4 | `checks[21].message` | tank length 28.10 m / inner diameter 7.70 m = 3.65 <= 7.0 | tank length 28.10 m / inner diameter 4.06 m = 6.92 <= 7.0 | changed |
| sweep downflooding opening gas 90x11.4 | `checks[22].message` | freeboard 2.78 m at the carried draught 2.22 m >= required 0.3 m | freeboard 3.40 m at the carried draught 1.60 m >= required 0.3 m | changed |
| sweep downflooding opening gas 90x11.4 | `checks[23].message` | GM 2.45 m (fluid), max GZ 1.34 m at 32° | GM 5.01 m (fluid), max GZ 2.20 m at 37° | changed |
| sweep downflooding opening gas 90x11.4 | `checks[24].message` | GM 2.46 m (fluid), max GZ 1.33 m at 32° | GM 5.04 m (fluid), max GZ 2.19 m at 37° | changed |
| sweep downflooding opening gas 90x11.4 | `checks[25].message` | GM 6.76 m (fluid), max GZ 1.87 m at 25° | GM 9.17 m (fluid), max GZ 2.33 m at 28° | changed |
| sweep downflooding opening gas 90x11.4 | `checks[26].message` | GM 6.84 m (fluid), max GZ 1.87 m at 25° | GM 9.30 m (fluid), max GZ 2.33 m at 27° | changed |
| sweep downflooding opening gas 90x11.4 | `checks[3].message` | freeboard 2.78 m >= required 0.3 m | freeboard 3.40 m >= required 0.3 m | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[0].kind` | wing_ballast | peak_ballast | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[0].name` | WB1P | FP1C | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[0].side` | port | centre | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[1].capacity_m3` | 319.341818054 | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[1].capacity_t` | 319.341818054 | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[1].kind` | wing_ballast | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[1].lcg_m` | 52.696 | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[1].name` | WB1S | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[1].side` | starboard | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[1].tcg_m` | 5.24518729454 | absent | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.ballast_tanks[1].vcg_m` | 3.01564722879 | absent | changed |
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
| sweep downflooding opening gas 90x11.4 | `loading_conditions.conditions[0].downflooding_deg` | absent | 34.84653041 | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.conditions[1].criteria[1].name` | Area 0–40° | Area 0–35.1° | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.conditions[1].criteria[2].name` | Area 30–40° | Area 30–35.1° | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.conditions[1].downflooding_deg` | absent | 35.1142534465 | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.conditions[2].downflooding_deg` | absent | 50.291477078 | changed |
| sweep downflooding opening gas 90x11.4 | `loading_conditions.conditions[3].downflooding_deg` | absent | 50.6898479268 | changed |
| sweep downflooding opening gas 90x11.4 | `stability.conditions[0].criteria[1].name` | Area 0–40° | Area 0–34.8° | changed |
| sweep downflooding opening gas 90x11.4 | `stability.conditions[0].criteria[2].name` | Area 30–40° | Area 30–34.8° | changed |
| sweep downflooding opening gas 90x11.4 | `stability.conditions[0].downflooding_deg` | absent | 34.84653041 | changed |
| sweep downflooding opening gas 90x11.4 | `stability.conditions[1].criteria[1].name` | Area 0–40° | Area 0–35.1° | changed |
| sweep downflooding opening gas 90x11.4 | `stability.conditions[1].criteria[2].name` | Area 30–40° | Area 30–35.1° | changed |
| ... | 747 more changed quantities | | | |
