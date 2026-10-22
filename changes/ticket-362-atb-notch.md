<!-- unreleased -->
- Formations (issue #362, spec 0007, part 1 of the ATB: the Notch and the tug sizing; the Equivalent Hull and the ATB powering come with #359): the barge of an `atb` Formation has a Notch. `ovcd-core::atb` sizes the ATB tug from the barge unless typed (length 0.28 of the barge length, beam 0.53 of its beam, draught beam over 2.33) and the Notch from the tug unless typed (0.50 of the tug length deep, the tug's beam and 0.30 m wide), and `Hull::build` cuts it into the Sections Table after the Stern Form (`HullShape.notch`; `apply_notch`, #465). The Notch takes its volume, LCB and waterplane from the sections, so the capacity, the draught and the stability of an ATB barge see it. A Notch of zero depth or width gives the plain barge bit for bit, and every other Formation is unchanged. Only `atb` Options move; the tug and Notch inputs were already in the file.

<!-- design-impact -->
### ticket/362-atb-notch (2026-10-22)

Between `main c4dd7c9` and `this branch 40d82b0`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep atb notch 100x20 | `checks[10].message` | GM 21.76 m (fluid), max GZ 6.01 m at 34° | GM 21.59 m (fluid), max GZ 6.10 m at 34° | changed |
| sweep atb notch 100x20 | `checks[11].message` | GM 42.71 m (fluid), max GZ 5.99 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | GM 42.79 m (fluid), max GZ 6.04 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | changed |
| sweep atb notch 100x20 | `checks[5].message` | intact GM 5.23 m >= 0.15 m at the reported cargo 9514 t, draught 5.70 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 5.63 m >= 0.15 m at the reported cargo 8644 t, draught 5.70 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| sweep atb notch 100x20 | `checks[7].message` | GM 5.23 m (fluid), max GZ 0.26 m at 6° — fails: Area 0–40° 0.060 < 0.09, Area 30–40° -0.017 < 0.03, GZ at ≥ 30° -0.025 < 0.2, Angle of max GZ 6.000 < 25.0 | GM -1.00 m (fluid), max GZ 0.00 m at 0° — fails: Trim within the solver's range 10.000 < 10.0 | changed |
| sweep atb notch 100x20 | `checks[8].message` | GM 5.23 m (fluid), max GZ 0.26 m at 6° — fails: Area 0–40° 0.060 < 0.09, Area 30–40° -0.017 < 0.03, GZ at ≥ 30° -0.025 < 0.2, Angle of max GZ 6.000 < 25.0 | GM -1.00 m (fluid), max GZ 0.00 m at 0° — fails: Trim within the solver's range 10.000 < 10.0 | changed |
| sweep atb notch 100x20 | `checks[9].message` | GM 21.76 m (fluid), max GZ 6.01 m at 34° | GM 21.59 m (fluid), max GZ 6.10 m at 34° | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[0].name` | Area 0–30° | Trim within the solver's range | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[0].ok` | True | False | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[0].source` | IMO IS Code 2008, Part A 2.2.1 | the hull would trim more than 10 degrees | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[1].limit` | 0.09 | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[1].name` | Area 0–40° | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[1].ok` | False | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[1].source` | IMO IS Code 2008, Part A 2.2.1 | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[1].value` | 0.0595769450525 | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[2].limit` | 0.03 | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[2].name` | Area 30–40° | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[2].ok` | False | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[2].source` | IMO IS Code 2008, Part A 2.2.1 | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[2].value` | -0.0174270075878 | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[3].limit` | 0.2 | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[3].name` | GZ at ≥ 30° | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[3].ok` | False | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[3].source` | IMO IS Code 2008, Part A 2.2.2 | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[3].value` | -0.0245749165226 | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[4].limit` | 25 | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[4].name` | Angle of max GZ | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[4].ok` | False | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[4].source` | IMO IS Code 2008, Part A 2.2.3 | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[4].value` | 6 | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[5].limit` | 0.15 | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[5].name` | Initial GM (fluid) | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[5].ok` | True | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[5].source` | IMO IS Code 2008, Part A 2.2.4 | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].criteria[5].value` | 5.23385487843 | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].deck_edge_deg` | 1.72303841447 | none | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].gz[0][0]` | 0 | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].gz[0][1]` | 1.03852450522e-17 | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].gz[10][0]` | 10 | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].gz[10][1]` | 0.236966966582 | absent | changed |
| sweep atb notch 100x20 | `loading_conditions.conditions[0].gz[11][0]` | 11 | absent | changed |
| ... | 1249 more changed quantities | | | |
