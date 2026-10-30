<!-- unreleased -->
- Gas hull steel from structural panels (issue #722, maintainer decision): `ovd_core::hull_panels` weighs the hull member by member (bottom, inner bottom, side, inner side, wing tank top, deck, transverse bulkheads, double-bottom girders and floors, web frames, deck transverses, with their stiffeners) with the minimum scantlings of the ABS Steel Barge Rules (2021) cited per member, and a hull girder check (3-2-1/3.1) that thickens the deck or bottom of a shallow hull. Only continuous longitudinal structure up to D counts in the hull girder. The weight is the sum over the members plus 5 % for brackets, welds and small items (an Estimate). **This steel now drives the gas-hull lightship, piping, outfitting and cost**, so depth follows the plating, not L x B x D. The top-down rate is reported for comparison only. The cost prices the panel steel plus the covering's parts above D. The structure factor of 2.0 is retired, and the #404 steel-disagreement warning no longer applies to a gas hull. The result carries the panel table as `cost_masses.hull_panels`. On the 140 x 29 m coastal LNG example (Option A) the lightship falls from 6,620 t to 5,243 t, the laden draught from 3.96 m to 3.56 m, and the barge cost rises 19 %; the cargo does not move. Calibration record: `calibration/hull_steel_cases.yaml`. Concept and preliminary design only; not a structural approval.

<!-- design-impact -->
### fix/722-gas-hull-steel (2026-10-30)

Between `main 0422419` and `this branch 9c20b6b9` (the table of CI's design-impact run on the pull request, run 41125670189).

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.55 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.53 m of 6.00 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[22].message` | freeboard 3.05 m at the carried draught 2.65 m >= required 0.3 m | freeboard 3.01 m at the carried draught 2.69 m >= required 0.3 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[23].message` | GM 6.17 m (fluid), max GZ 2.57 m at 29° | GM 6.07 m (fluid), max GZ 2.53 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[24].message` | GM 6.18 m (fluid), max GZ 2.56 m at 29° | GM 6.08 m (fluid), max GZ 2.53 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[25].message` | GM 13.07 m (fluid), max GZ 3.50 m at 27° | GM 12.60 m (fluid), max GZ 3.48 m at 28° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[26].message` | GM 13.16 m (fluid), max GZ 3.49 m at 27° | GM 12.69 m (fluid), max GZ 3.48 m at 28° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[3].message` | freeboard 3.05 m >= required 0.3 m | freeboard 3.01 m >= required 0.3 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[5].message` | 2 x 1.35 m thrusters need 4.50 m across a 16.30 m beam | 2 x 1.42 m thrusters need 4.64 m across a 16.30 m beam | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[6].message` | 2 of 2 requested thrusters fit; 2 would need 4.50 m of 16.30 m beam | 2 of 2 requested thrusters fit; 2 would need 4.64 m of 16.30 m beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].message` | blind distance ahead of the bow over 250 m, light (ES-TRIN 7.02(2)): lift needed 10.03 m of 6.00 m | blind distance ahead of the bow over 250 m, light (ES-TRIN 7.02(2)): lift needed 9.88 m of 6.00 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[23].message` | freeboard 2.58 m at the carried draught 3.12 m >= required 0.3 m | freeboard 2.21 m at the carried draught 3.49 m >= required 0.3 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[24].message` | GM 3.85 m (fluid), max GZ 1.56 m at 26° | GM 3.52 m (fluid), max GZ 1.28 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[25].message` | GM 3.85 m (fluid), max GZ 1.56 m at 26° | GM 3.52 m (fluid), max GZ 1.29 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[26].message` | GM 10.62 m (fluid), max GZ 2.92 m at 25° | GM 8.30 m (fluid), max GZ 2.86 m at 29° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[27].message` | GM 10.67 m (fluid), max GZ 2.92 m at 25° | GM 8.33 m (fluid), max GZ 2.86 m at 29° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[4].message` | freeboard 2.58 m >= required 0.3 m | freeboard 2.21 m >= required 0.3 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[6].message` | 2 x 1.55 m thrusters need 4.91 m across a 16.30 m beam | 2 x 2.03 m thrusters need 5.85 m across a 16.30 m beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[7].message` | 2 of 2 requested thrusters fit; 2 would need 4.91 m of 16.30 m beam | 2 of 2 requested thrusters fit; 2 would need 5.85 m of 16.30 m beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `warnings[1]` | The blind distance needs 10.03 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | The blind distance needs 9.88 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 5.47 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 5.28 m of 6.00 m | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[22].message` | freeboard 2.68 m at the carried draught 3.02 m >= required 0.3 m | freeboard 2.21 m at the carried draught 3.49 m >= required 0.3 m | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[23].message` | GM 5.82 m (fluid), max GZ 2.31 m at 27° | GM 5.14 m (fluid), max GZ 1.87 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[24].message` | GM 5.82 m (fluid), max GZ 2.31 m at 27° | GM 5.14 m (fluid), max GZ 1.87 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[25].message` | GM 13.81 m (fluid), max GZ 3.66 m at 27° | GM 10.13 m (fluid), max GZ 3.50 m at 29° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[26].message` | GM 13.88 m (fluid), max GZ 3.66 m at 27° | GM 10.16 m (fluid), max GZ 3.50 m at 29° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[3].message` | freeboard 2.68 m >= required 0.3 m | freeboard 2.21 m >= required 0.3 m | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[5].message` | 2 x 1.55 m thrusters need 4.91 m across a 17.20 m beam | 2 x 2.09 m thrusters need 5.99 m across a 17.20 m beam | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[6].message` | 2 of 2 requested thrusters fit; 2 would need 4.91 m of 17.20 m beam | 2 of 2 requested thrusters fit; 2 would need 5.99 m of 17.20 m beam | changed |
| sweep default depth gas 90x14 | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 5.05 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 5.07 m of 6.00 m | changed |
| sweep default depth gas 90x14 | `checks[22].message` | freeboard 5.14 m at the carried draught 2.56 m >= required 0.3 m | freeboard 5.22 m at the carried draught 2.48 m >= required 0.3 m | changed |
| sweep default depth gas 90x14 | `checks[23].message` | GM 3.39 m (fluid), max GZ 1.89 m at 39° | GM 3.55 m (fluid), max GZ 1.90 m at 39° | changed |
| sweep default depth gas 90x14 | `checks[24].message` | GM 3.40 m (fluid), max GZ 1.88 m at 39° | GM 3.56 m (fluid), max GZ 1.88 m at 38° | changed |
| sweep default depth gas 90x14 | `checks[25].message` | GM 7.96 m (fluid), max GZ 2.36 m at 26° | GM 8.63 m (fluid), max GZ 2.41 m at 25° | changed |
| sweep default depth gas 90x14 | `checks[26].message` | GM 8.03 m (fluid), max GZ 2.35 m at 25° | GM 6.12 m (fluid), max GZ 1.87 m at 25° | changed |
| sweep default depth gas 90x14 | `checks[29].message` | boil-off 176 kg/h > gas use 86 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | boil-off 176 kg/h > gas use 123 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | changed |
| sweep default depth gas 90x14 | `checks[3].message` | freeboard 5.14 m >= required 0.3 m | freeboard 5.22 m >= required 0.3 m | changed |
| sweep default depth gas 90x14 | `checks[5].message` | 2 x 1.62 m thrusters need 5.04 m across a 14.00 m beam | 2 x 0.81 m thrusters need 3.42 m across a 14.00 m beam | changed |
| sweep default depth gas 90x14 | `checks[6].message` | 2 of 2 requested thrusters fit; 2 would need 5.04 m of 14.00 m beam | 2 of 2 requested thrusters fit; 2 would need 3.42 m of 14.00 m beam | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[3].ballast[2].fill` | absent | 0.45 | changed |
| sweep default depth gas 90x14 | `loading_conditions.conditions[3].ballast[2].free_surface_moment_tm` | absent | 1564.98905356 | changed |
| ... | 82464 more changed quantities | | | |
