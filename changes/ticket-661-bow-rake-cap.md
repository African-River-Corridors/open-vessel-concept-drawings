<!-- unreleased -->
- Design change (issue #661, the maintainer's choice of 2026-10-30): a new input, `bow_rake_max_length_m` (Hull and containment, default 10 m, an estimate), caps the bow rake that `bow_rake_length_to_depth_ratio` x moulded depth gives. The rake is now `min(1.6 x D, 10 m)`, so it changes only hulls deeper than 6.25 m. A typed `bow_rake_length_m` is not capped. The rake rise is unchanged (the moulded depth), so a capped rake is steeper. A file saved before keeps no cap and its results. On the worked 140 x 29 m gas example (D = 12 m) a 10 m cap gives a 10 m rake instead of 19.2 m, 9.2 m more cargo zone and 15,862 m3 of cargo instead of 14,355 m3.

<!-- design-impact -->
### ticket/661-bow-rake-cap (2026-10-30)

Between `main 278d0a1` and `this branch 0870f445`.

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
| sweep default depth gas 90x14 | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.90 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 5.01 m of 6.00 m | changed |
| sweep default depth gas 90x14 | `checks[21].message` | tank length 24.74 m / inner diameter 9.24 m = 2.68 <= 7.0 | tank length 25.90 m / inner diameter 9.23 m = 2.81 <= 7.0 | changed |
| sweep default depth gas 90x14 | `checks[22].message` | freeboard 4.95 m at the carried draught 2.75 m >= required 0.3 m | freeboard 4.94 m at the carried draught 2.76 m >= required 0.3 m | changed |
| sweep default depth gas 90x14 | `checks[23].message` | GM 3.13 m (fluid), max GZ 1.91 m at 41° | GM 3.07 m (fluid), max GZ 1.88 m at 40° | changed |
| sweep default depth gas 90x14 | `checks[24].message` | GM 3.13 m (fluid), max GZ 1.90 m at 41° | GM 3.07 m (fluid), max GZ 1.87 m at 40° | changed |
| sweep default depth gas 90x14 | `checks[25].message` | GM 7.01 m (fluid), max GZ 2.14 m at 28° | GM 7.06 m (fluid), max GZ 2.14 m at 27° | changed |
| sweep default depth gas 90x14 | `checks[26].message` | GM 7.05 m (fluid), max GZ 2.13 m at 27° | GM 7.11 m (fluid), max GZ 2.13 m at 27° | changed |
| sweep default depth gas 90x14 | `checks[29].message` | boil-off 169 kg/h > gas use 66 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | boil-off 176 kg/h > gas use 67 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | changed |
| sweep default depth gas 90x14 | `checks[3].message` | freeboard 4.95 m >= required 0.3 m | freeboard 4.94 m >= required 0.3 m | changed |
| sweep default depth gas 90x14 | `checks[4].message` | cargo zone 51.49 m = LOA 90.0 - bow void 12.52 m - stern void 25.99 m | cargo zone 53.81 m = LOA 90.0 - bow void 10.20 m - stern void 25.99 m | changed |
| sweep default depth gas 90x14 | `stability.lowest_gm_condition` | Full load, arrival | Full load, departure | changed |
| sweep default depth gas 90x14 | `warnings[2]` | The lightship uses 970.2 t of hull steel (top-down); the cost prices 777.7 t (bottom-up). They differ by 192.5 t (24.7 %), more than the 20 % limit (an Estimate). The cost keeps the bottom-up mass. | The lightship uses 970.2 t of hull steel (top-down); the cost prices 778.6 t (bottom-up). They differ by 191.6 t (24.6 %), more than the 20 % limit (an Estimate). The cost keeps the bottom-up mass. | changed |
| sweep gas wide hull IGC_2G 110x24 | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.90 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.92 m of 6.00 m | changed |
| sweep gas wide hull IGC_2G 110x24 | `checks[21].message` | tank length 37.65 m / inner diameter 6.34 m = 5.94 <= 7.0 | tank length 37.85 m / inner diameter 6.34 m = 5.97 <= 7.0 | changed |
| sweep gas wide hull IGC_2G 110x24 | `checks[23].message` | GM 18.48 m (fluid), max GZ 5.65 m at 27° | GM 18.46 m (fluid), max GZ 5.64 m at 27° | changed |
| sweep gas wide hull IGC_2G 110x24 | `checks[24].message` | GM 18.53 m (fluid), max GZ 5.65 m at 27° | GM 18.52 m (fluid), max GZ 5.64 m at 27° | changed |
| sweep gas wide hull IGC_2G 110x24 | `checks[25].message` | GM 27.07 m (fluid), max GZ 6.94 m at 31° | GM 27.09 m (fluid), max GZ 6.94 m at 31° | changed |
| sweep gas wide hull IGC_2G 110x24 | `checks[26].message` | GM 27.19 m (fluid), max GZ 6.94 m at 31° | GM 27.21 m (fluid), max GZ 6.94 m at 31° | changed |
| sweep gas wide hull IGC_2G 110x24 | `checks[29].message` | boil-off 539 kg/h > gas use 127 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | boil-off 542 kg/h > gas use 128 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | changed |
| sweep gas wide hull IGC_2G 110x24 | `checks[4].message` | cargo zone 77.31 m = LOA 110.0 - bow void 10.60 m - stern void 22.09 m | cargo zone 77.71 m = LOA 110.0 - bow void 10.20 m - stern void 22.09 m | changed |
| sweep gas wide hull IGC_2G_EXACT 110x24 | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.90 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.92 m of 6.00 m | changed |
| sweep gas wide hull IGC_2G_EXACT 110x24 | `checks[21].message` | tank length 37.65 m / inner diameter 6.34 m = 5.94 <= 7.0 | tank length 37.85 m / inner diameter 6.34 m = 5.97 <= 7.0 | changed |
| sweep gas wide hull IGC_2G_EXACT 110x24 | `checks[23].message` | GM 18.48 m (fluid), max GZ 5.65 m at 27° | GM 18.46 m (fluid), max GZ 5.64 m at 27° | changed |
| sweep gas wide hull IGC_2G_EXACT 110x24 | `checks[24].message` | GM 18.53 m (fluid), max GZ 5.65 m at 27° | GM 18.52 m (fluid), max GZ 5.64 m at 27° | changed |
| sweep gas wide hull IGC_2G_EXACT 110x24 | `checks[25].message` | GM 27.07 m (fluid), max GZ 6.94 m at 31° | GM 27.09 m (fluid), max GZ 6.94 m at 31° | changed |
| sweep gas wide hull IGC_2G_EXACT 110x24 | `checks[26].message` | GM 27.19 m (fluid), max GZ 6.94 m at 31° | GM 27.21 m (fluid), max GZ 6.94 m at 31° | changed |
| sweep gas wide hull IGC_2G_EXACT 110x24 | `checks[29].message` | boil-off 539 kg/h > gas use 127 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | boil-off 542 kg/h > gas use 128 kg/h at the service speed: the excess needs reliquefaction, pressure accumulation or thermal oxidation, as venting is not acceptable (IGC Code 7.1.1, 7.1.3) | changed |
| sweep gas wide hull IGC_2G_EXACT 110x24 | `checks[4].message` | cargo zone 77.31 m = LOA 110.0 - bow void 10.60 m - stern void 22.09 m | cargo zone 77.71 m = LOA 110.0 - bow void 10.20 m - stern void 22.09 m | changed |
| sweep hull shipd bulbs 80 m | `checks[4].message` | cargo zone 63.15 m = LOA 80.0 - bow void 13.35 m - stern void 3.50 m | cargo zone 66.00 m = LOA 80.0 - bow void 10.50 m - stern void 3.50 m | changed |
| calibration meb-dcb23-ptich | `loading_conditions.conditions[0].trim_m` | 0.259294177615 | 0.362963980286 | +40 % |
| calibration meb-dcb23-ptich | `loading_conditions.conditions[1].trim_m` | 0.259294177615 | 0.362963980286 | +40 % |
| calibration meb-dcb23-ptich | `stability.conditions[0].trim_m` | 0.259294177615 | 0.362963980286 | +40 % |
| ... | 3806 more changed quantities | | | |
