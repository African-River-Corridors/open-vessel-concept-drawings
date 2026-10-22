<!-- unreleased -->
- Hold inner bottom default (issue #597): a new dry bulk Option takes an inner bottom 0.80 m above the keel, not 1.20 m. Built barges and published designs show 0.38 to 1.0 m (0.80 m in two published inland designs). The hold is 0.40 m taller, so a 14 m beam, 4.5 m depth box hold gains 9 % cargo volume. A file saved before keeps 1.20 m. A new dry bulk Option already defaulted to the box hold (vertical inner sides); the 1.0 H:V slope applies only to a hold typed as sloped hopper, and it is unchanged.

<!-- design-impact -->
### ticket/597-box-hold-default (2026-10-22)

Between `main 5d5ec18` and `this branch 612dd5a`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| calibration amaggi-box-2000 | `checks[10].message` | GM 6.49 m (fluid), max GZ 1.84 m at 25° | GM 6.40 m (fluid), max GZ 1.85 m at 25° | changed |
| calibration amaggi-box-2000 | `checks[5].message` | intact GM 1.53 m >= 0.15 m at the reported cargo 1861 t, draught 3.56 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 1.70 m >= 0.15 m at the reported cargo 1861 t, draught 3.56 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| calibration amaggi-box-2000 | `checks[7].message` | GM 1.53 m (fluid), max GZ 0.26 m at 13° — fails: Area 30–40° 0.002 < 0.03, GZ at ≥ 30° 0.091 < 0.2, Angle of max GZ 13.000 < 25.0 | GM 1.70 m (fluid), max GZ 0.31 m at 15° — fails: Area 30–40° 0.019 < 0.03, GZ at ≥ 30° 0.177 < 0.2, Angle of max GZ 15.000 < 25.0 | changed |
| calibration amaggi-box-2000 | `checks[8].message` | GM 1.53 m (fluid), max GZ 0.26 m at 13° — fails: Area 30–40° 0.002 < 0.03, GZ at ≥ 30° 0.091 < 0.2, Angle of max GZ 13.000 < 25.0 | GM 1.70 m (fluid), max GZ 0.31 m at 15° — fails: Area 30–40° 0.019 < 0.03, GZ at ≥ 30° 0.177 < 0.2, Angle of max GZ 15.000 < 25.0 | changed |
| calibration amaggi-box-2000 | `checks[9].message` | GM 6.49 m (fluid), max GZ 1.84 m at 25° | GM 6.40 m (fluid), max GZ 1.85 m at 25° | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[10].message` | GM 3.86 m (fluid), max GZ 1.23 m at 26° | GM 3.94 m (fluid), max GZ 1.22 m at 26° | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[2].ok` | True | False | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[3].message` | freeboard 0.04 m >= required 0.3 m | freeboard 0.00 m >= required 0.3 m | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[5].message` | intact GM 0.46 m >= 0.15 m at the reported cargo 1012 t, draught 2.74 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 0.63 m >= 0.15 m at the reported cargo 1012 t, draught 2.74 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[7].message` | GM 0.46 m (fluid), max GZ 0.14 m at 18° — fails: Area 0–30° 0.047 < 0.055, Area 0–40° 0.045 < 0.09, Area 30–40° -0.001 < 0.03, GZ at ≥ 30° 0.054 < 0.2, Angle of max GZ 18.000 < 25.0 | GM 0.63 m (fluid), max GZ 0.19 m at 19° — fails: Area 0–40° 0.085 < 0.09, Area 30–40° 0.015 < 0.03, GZ at ≥ 30° 0.138 < 0.2, Angle of max GZ 19.000 < 25.0 | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[8].message` | GM 0.46 m (fluid), max GZ 0.14 m at 18° — fails: Area 0–30° 0.047 < 0.055, Area 0–40° 0.045 < 0.09, Area 30–40° -0.001 < 0.03, GZ at ≥ 30° 0.054 < 0.2, Angle of max GZ 18.000 < 25.0 | GM 0.63 m (fluid), max GZ 0.19 m at 19° — fails: Area 0–40° 0.085 < 0.09, Area 30–40° 0.015 < 0.03, GZ at ≥ 30° 0.138 < 0.2, Angle of max GZ 19.000 < 25.0 | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[9].message` | GM 3.86 m (fluid), max GZ 1.23 m at 26° | GM 3.94 m (fluid), max GZ 1.22 m at 26° | changed |
| calibration canal-box-200x25x12-pb2230 | `loading_conditions.conditions[0].criteria[0].ok` | False | True | changed |
| calibration canal-box-200x25x12-pb2230 | `loading_conditions.conditions[1].criteria[0].ok` | False | True | changed |
| calibration canal-box-200x25x12-pb2230 | `stability.conditions[0].criteria[0].ok` | False | True | changed |
| calibration canal-box-200x25x12-pb2230 | `stability.conditions[1].criteria[0].ok` | False | True | changed |
| calibration canal-box-200x35x12-cby222 | `checks[5].message` | intact GM 2.22 m >= 0.15 m at the reported cargo 1417 t, draught 2.74 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 2.39 m >= 0.15 m at the reported cargo 1417 t, draught 2.74 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| calibration canal-box-200x35x12-cby222 | `checks[7].message` | GM 2.22 m (fluid), max GZ 0.53 m at 19° — fails: Angle of max GZ 19.000 < 25.0 | GM 2.39 m (fluid), max GZ 0.58 m at 21° — fails: Angle of max GZ 21.000 < 25.0 | changed |
| calibration canal-box-200x35x12-cby222 | `checks[8].message` | GM 2.22 m (fluid), max GZ 0.53 m at 19° — fails: Angle of max GZ 19.000 < 25.0 | GM 2.39 m (fluid), max GZ 0.58 m at 21° — fails: Angle of max GZ 21.000 < 25.0 | changed |
| calibration canal-box-200x35x12-imt201 | `checks[5].message` | intact GM 2.22 m >= 0.15 m at the reported cargo 1417 t, draught 2.74 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 2.39 m >= 0.15 m at the reported cargo 1417 t, draught 2.74 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| calibration canal-box-200x35x12-imt201 | `checks[7].message` | GM 2.22 m (fluid), max GZ 0.53 m at 19° — fails: Angle of max GZ 19.000 < 25.0 | GM 2.39 m (fluid), max GZ 0.58 m at 21° — fails: Angle of max GZ 21.000 < 25.0 | changed |
| calibration canal-box-200x35x12-imt201 | `checks[8].message` | GM 2.22 m (fluid), max GZ 0.53 m at 19° — fails: Angle of max GZ 19.000 < 25.0 | GM 2.39 m (fluid), max GZ 0.58 m at 21° — fails: Angle of max GZ 21.000 < 25.0 | changed |
| calibration canal-box-200x35x12-imt205 | `checks[5].message` | intact GM 2.22 m >= 0.15 m at the reported cargo 1417 t, draught 2.74 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 2.39 m >= 0.15 m at the reported cargo 1417 t, draught 2.74 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| calibration canal-box-200x35x12-imt205 | `checks[7].message` | GM 2.22 m (fluid), max GZ 0.53 m at 19° — fails: Angle of max GZ 19.000 < 25.0 | GM 2.39 m (fluid), max GZ 0.58 m at 21° — fails: Angle of max GZ 21.000 < 25.0 | changed |
| calibration canal-box-200x35x12-imt205 | `checks[8].message` | GM 2.22 m (fluid), max GZ 0.53 m at 19° — fails: Angle of max GZ 19.000 < 25.0 | GM 2.39 m (fluid), max GZ 0.58 m at 21° — fails: Angle of max GZ 21.000 < 25.0 | changed |
| calibration canal-box-200x35x12-pb2012 | `checks[5].message` | intact GM 2.22 m >= 0.15 m at the reported cargo 1417 t, draught 2.74 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 2.39 m >= 0.15 m at the reported cargo 1417 t, draught 2.74 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| calibration canal-box-200x35x12-pb2012 | `checks[7].message` | GM 2.22 m (fluid), max GZ 0.53 m at 19° — fails: Angle of max GZ 19.000 < 25.0 | GM 2.39 m (fluid), max GZ 0.58 m at 21° — fails: Angle of max GZ 21.000 < 25.0 | changed |
| calibration canal-box-200x35x12-pb2012 | `checks[8].message` | GM 2.22 m (fluid), max GZ 0.53 m at 19° — fails: Angle of max GZ 19.000 < 25.0 | GM 2.39 m (fluid), max GZ 0.58 m at 21° — fails: Angle of max GZ 21.000 < 25.0 | changed |
| calibration canal-box-200x35x12-rci250 | `checks[5].message` | intact GM 2.22 m >= 0.15 m at the reported cargo 1417 t, draught 2.74 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 2.39 m >= 0.15 m at the reported cargo 1417 t, draught 2.74 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| calibration canal-box-200x35x12-rci250 | `checks[7].message` | GM 2.22 m (fluid), max GZ 0.53 m at 19° — fails: Angle of max GZ 19.000 < 25.0 | GM 2.39 m (fluid), max GZ 0.58 m at 21° — fails: Angle of max GZ 21.000 < 25.0 | changed |
| calibration canal-box-200x35x12-rci250 | `checks[8].message` | GM 2.22 m (fluid), max GZ 0.53 m at 19° — fails: Angle of max GZ 19.000 < 25.0 | GM 2.39 m (fluid), max GZ 0.58 m at 21° — fails: Angle of max GZ 21.000 < 25.0 | changed |
| calibration canal-box-200x35x13-cby252 | `checks[10].message` | GM 6.58 m (fluid), max GZ 1.90 m at 26° | GM 6.79 m (fluid), max GZ 1.89 m at 25° | changed |
| calibration canal-box-200x35x13-cby252 | `checks[5].message` | intact GM 2.09 m >= 0.15 m at the reported cargo 1381 t, draught 2.74 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 2.25 m >= 0.15 m at the reported cargo 1381 t, draught 2.74 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| calibration canal-box-200x35x13-cby252 | `checks[7].message` | GM 2.08 m (fluid), max GZ 0.67 m at 25° | GM 2.25 m (fluid), max GZ 0.74 m at 27° | changed |
| calibration canal-box-200x35x13-cby252 | `checks[8].message` | GM 2.08 m (fluid), max GZ 0.67 m at 25° | GM 2.25 m (fluid), max GZ 0.74 m at 27° | changed |
| calibration canal-box-200x35x13-cby252 | `checks[9].message` | GM 6.58 m (fluid), max GZ 1.90 m at 26° | GM 6.79 m (fluid), max GZ 1.89 m at 25° | changed |
| calibration canal-box-200x35x13-cby400 | `checks[10].message` | GM 6.58 m (fluid), max GZ 1.90 m at 26° | GM 6.79 m (fluid), max GZ 1.89 m at 25° | changed |
| calibration canal-box-200x35x13-cby400 | `checks[5].message` | intact GM 2.09 m >= 0.15 m at the reported cargo 1381 t, draught 2.74 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 2.25 m >= 0.15 m at the reported cargo 1381 t, draught 2.74 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| calibration canal-box-200x35x13-cby400 | `checks[7].message` | GM 2.08 m (fluid), max GZ 0.67 m at 25° | GM 2.25 m (fluid), max GZ 0.74 m at 27° | changed |
| calibration canal-box-200x35x13-cby400 | `checks[8].message` | GM 2.08 m (fluid), max GZ 0.67 m at 25° | GM 2.25 m (fluid), max GZ 0.74 m at 27° | changed |
| ... | 70154 more changed quantities | | | |
