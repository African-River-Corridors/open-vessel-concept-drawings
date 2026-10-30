<!-- unreleased -->
- Design change (issue #661, the maintainer's choice of 2026-10-30): a new input, `bow_rake_max_length_m` (Hull and containment, default 10 m, an estimate), caps the bow rake that `bow_rake_length_to_depth_ratio` x moulded depth gives. The rake is now `min(1.6 x D, 10 m)`, so it changes only hulls deeper than 6.25 m. A typed `bow_rake_length_m` is not capped. The rake rise is unchanged (the moulded depth), so a capped rake is steeper. A file saved before keeps no cap and its results. On the worked 140 x 29 m gas example (D = 12 m) a 10 m cap gives a 10 m rake instead of 19.2 m, 9.2 m more cargo zone and 15,862 m3 of cargo instead of 14,355 m3.

<!-- design-impact -->
### ticket/661-bow-rake-cap (2026-10-30)

Between `main 6f8c7e7` and `this branch a4943a6d`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| calibration meb-dcb23-ptich | `checks[10].message` | GM 11.77 m (fluid), max GZ 4.28 m at 40° | GM 12.02 m (fluid), max GZ 4.28 m at 40° | changed |
| calibration meb-dcb23-ptich | `checks[11].message` | GM 19.19 m (fluid), max GZ 3.93 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | GM 20.24 m (fluid), max GZ 3.96 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | changed |
| calibration meb-dcb23-ptich | `checks[4].message` | cargo zone 83.18 m = LOA 100.0 - bow void 13.30 m - stern void 3.50 m | cargo zone 85.98 m = LOA 100.0 - bow void 10.50 m - stern void 3.50 m | changed |
| calibration meb-dcb23-ptich | `checks[5].message` | intact GM 2.67 m >= 0.15 m at the reported cargo 6167 t, draught 5.00 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 2.61 m >= 0.15 m at the reported cargo 6327 t, draught 5.00 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| calibration meb-dcb23-ptich | `checks[7].message` | GM 2.66 m (fluid), max GZ 1.44 m at 35° | GM 2.60 m (fluid), max GZ 1.41 m at 35° | changed |
| calibration meb-dcb23-ptich | `checks[8].message` | GM 2.66 m (fluid), max GZ 1.44 m at 35° | GM 2.60 m (fluid), max GZ 1.41 m at 35° | changed |
| calibration meb-dcb23-ptich | `checks[9].message` | GM 11.77 m (fluid), max GZ 4.28 m at 40° | GM 12.02 m (fluid), max GZ 4.28 m at 40° | changed |
| calibration meb-dcb23-ptich | `formation.notes[3]` | The tug's view over the barges needs 9.02 m of wheelhouse lift; the tug gives 6.00 m. The Blind Distance light is 1800.4 m, more than 250 m (ES-TRIN 7.02(2)). | The tug's view over the barges needs 9.06 m of wheelhouse lift; the tug gives 6.00 m. The Blind Distance light is 1918.8 m, more than 250 m (ES-TRIN 7.02(2)). | changed |
| sweep default depth gas 90x14 | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.95 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 5.05 m of 6.00 m | changed |
| sweep default depth gas 90x14 | `checks[21].message` | tank length 24.74 m / inner diameter 9.24 m = 2.68 <= 7.0 | tank length 25.90 m / inner diameter 9.23 m = 2.81 <= 7.0 | changed |
| sweep default depth gas 90x14 | `checks[23].message` | GM 3.43 m (fluid), max GZ 1.93 m at 39° | GM 3.39 m (fluid), max GZ 1.89 m at 39° | changed |
| sweep default depth gas 90x14 | `checks[24].message` | GM 3.44 m (fluid), max GZ 1.91 m at 39° | GM 3.40 m (fluid), max GZ 1.88 m at 39° | changed |
| sweep default depth gas 90x14 | `checks[25].message` | GM 7.80 m (fluid), max GZ 2.35 m at 26° | GM 7.96 m (fluid), max GZ 2.36 m at 26° | changed |
| sweep default depth gas 90x14 | `checks[26].message` | GM 7.86 m (fluid), max GZ 2.35 m at 26° | GM 8.03 m (fluid), max GZ 2.35 m at 25° | changed |
| sweep default depth gas 90x14 | `checks[29].message` | boil-off 169 kg/h > gas use 86 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | boil-off 176 kg/h > gas use 86 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | changed |
| sweep default depth gas 90x14 | `checks[4].message` | cargo zone 51.49 m = LOA 90.0 - bow void 12.52 m - stern void 25.99 m | cargo zone 53.81 m = LOA 90.0 - bow void 10.20 m - stern void 25.99 m | changed |
| sweep default depth gas 90x14 | `warnings[2]` | The lightship uses 970.2 t of hull steel (top-down); the cost prices 777.7 t (bottom-up). They differ by 192.5 t (24.7 %), more than the 20 % limit (an Estimate). The cost keeps the bottom-up mass. | The lightship uses 970.2 t of hull steel (top-down); the cost prices 778.6 t (bottom-up). They differ by 191.6 t (24.6 %), more than the 20 % limit (an Estimate). The cost keeps the bottom-up mass. | changed |
| sweep gas wide hull IGC_2G 110x24 | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.91 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.93 m of 6.00 m | changed |
| sweep gas wide hull IGC_2G 110x24 | `checks[21].message` | tank length 37.65 m / inner diameter 6.34 m = 5.94 <= 7.0 | tank length 37.85 m / inner diameter 6.34 m = 5.97 <= 7.0 | changed |
| sweep gas wide hull IGC_2G 110x24 | `checks[23].message` | GM 18.68 m (fluid), max GZ 5.66 m at 27° | GM 18.67 m (fluid), max GZ 5.65 m at 27° | changed |
| sweep gas wide hull IGC_2G 110x24 | `checks[24].message` | GM 18.73 m (fluid), max GZ 5.66 m at 27° | GM 18.72 m (fluid), max GZ 5.65 m at 27° | changed |
| sweep gas wide hull IGC_2G 110x24 | `checks[25].message` | GM 27.48 m (fluid), max GZ 6.96 m at 31° | GM 27.51 m (fluid), max GZ 6.96 m at 31° | changed |
| sweep gas wide hull IGC_2G 110x24 | `checks[26].message` | GM 27.60 m (fluid), max GZ 6.96 m at 31° | GM 27.63 m (fluid), max GZ 6.96 m at 31° | changed |
| sweep gas wide hull IGC_2G 110x24 | `checks[29].message` | boil-off 539 kg/h > gas use 175 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | boil-off 542 kg/h > gas use 176 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | changed |
| sweep gas wide hull IGC_2G 110x24 | `checks[4].message` | cargo zone 77.31 m = LOA 110.0 - bow void 10.60 m - stern void 22.09 m | cargo zone 77.71 m = LOA 110.0 - bow void 10.20 m - stern void 22.09 m | changed |
| sweep gas wide hull IGC_2G_EXACT 110x24 | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.91 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.93 m of 6.00 m | changed |
| sweep gas wide hull IGC_2G_EXACT 110x24 | `checks[21].message` | tank length 37.65 m / inner diameter 6.34 m = 5.94 <= 7.0 | tank length 37.85 m / inner diameter 6.34 m = 5.97 <= 7.0 | changed |
| sweep gas wide hull IGC_2G_EXACT 110x24 | `checks[23].message` | GM 18.68 m (fluid), max GZ 5.66 m at 27° | GM 18.67 m (fluid), max GZ 5.65 m at 27° | changed |
| sweep gas wide hull IGC_2G_EXACT 110x24 | `checks[24].message` | GM 18.73 m (fluid), max GZ 5.66 m at 27° | GM 18.72 m (fluid), max GZ 5.65 m at 27° | changed |
| sweep gas wide hull IGC_2G_EXACT 110x24 | `checks[25].message` | GM 27.48 m (fluid), max GZ 6.96 m at 31° | GM 27.51 m (fluid), max GZ 6.96 m at 31° | changed |
| sweep gas wide hull IGC_2G_EXACT 110x24 | `checks[26].message` | GM 27.60 m (fluid), max GZ 6.96 m at 31° | GM 27.63 m (fluid), max GZ 6.96 m at 31° | changed |
| sweep gas wide hull IGC_2G_EXACT 110x24 | `checks[29].message` | boil-off 539 kg/h > gas use 175 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | boil-off 542 kg/h > gas use 176 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | changed |
| sweep gas wide hull IGC_2G_EXACT 110x24 | `checks[4].message` | cargo zone 77.31 m = LOA 110.0 - bow void 10.60 m - stern void 22.09 m | cargo zone 77.71 m = LOA 110.0 - bow void 10.20 m - stern void 22.09 m | changed |
| sweep hull shipd bulbs 80 m | `checks[4].message` | cargo zone 63.15 m = LOA 80.0 - bow void 13.35 m - stern void 3.50 m | cargo zone 66.00 m = LOA 80.0 - bow void 10.50 m - stern void 3.50 m | changed |
| sweep optional bow_rake_max_length_m | `checks[12].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 1.44 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 1.53 m of 6.00 m | changed |
| sweep optional bow_rake_max_length_m | `checks[14].message` | intact GM 4.08 m >= 0.15 m at the reported cargo 2512 t, draught 3.39 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 4.01 m >= 0.15 m at the reported cargo 2570 t, draught 3.41 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| sweep optional bow_rake_max_length_m | `checks[16].message` | GM 4.08 m (fluid), max GZ 0.79 m at 21° — fails: Angle of max GZ 21.000 < 25.0 | GM 4.01 m (fluid), max GZ 0.73 m at 21° — fails: Angle of max GZ 21.000 < 25.0 | changed |
| sweep optional bow_rake_max_length_m | `checks[17].message` | GM 4.09 m (fluid), max GZ 0.78 m at 21° — fails: Angle of max GZ 21.000 < 25.0 | GM 4.02 m (fluid), max GZ 0.73 m at 21° — fails: Angle of max GZ 21.000 < 25.0 | changed |
| sweep optional bow_rake_max_length_m | `checks[18].message` | GM 13.40 m (fluid), max GZ 3.63 m at 29° | GM 13.34 m (fluid), max GZ 3.63 m at 29° | changed |
| sweep optional bow_rake_max_length_m | `checks[19].message` | GM 13.56 m (fluid), max GZ 3.63 m at 29° | GM 13.49 m (fluid), max GZ 3.63 m at 29° | changed |
| ... | 4752 more changed quantities | | | |
