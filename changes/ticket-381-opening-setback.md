<!-- unreleased -->
- Gas opening setback (issue #381): the 2.00 m setback of an entrance or openable window is now measured from the end of the cargo area, so the aft end space counts as part of the aft separation, not toward the setback. New stored choice input `opening_setback_basis` (`tank_end` for a file saved before, `cargo_area` for a new Option). The aft separation grows by 0.20 m, which takes back about two thirds of the cargo gain of the end space change (CN II cargo 2,033.4 t to 2,025.3 t). `lr-inland-gas` 2.0.0 (unreleased) drops its setback Rule (`lr-inland/3.3.7/opening-setback`). The Python API default of the gas separation inputs changes with it.

<!-- design-impact -->
### ticket/381-opening-setback (2026-10-22)

Between `main 33e1cb2` and `this branch 6adfac4`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `checks[15].message` | aft separation 2.00 m >= 2.00 m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11) | distance 2.00 m from the cargo area (end space included) to any entrance or openable window >= 2.00 m (cl. 3.3.7 / 3.3.11) | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[21].message` | tank length 28.59 m / inner diameter 6.80 m = 4.21 <= 7.0 | tank length 28.49 m / inner diameter 6.80 m = 4.19 <= 7.0 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[22].message` | freeboard 2.92 m at the carried draught 2.78 m >= required 0.3 m | freeboard 2.93 m at the carried draught 2.77 m >= required 0.3 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[23].message` | GM 5.87 m (fluid), max GZ 2.51 m at 29° | GM 5.89 m (fluid), max GZ 2.51 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[24].message` | GM 5.89 m (fluid), max GZ 2.51 m at 29° | GM 5.90 m (fluid), max GZ 2.51 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[25].message` | GM 12.46 m (fluid), max GZ 3.46 m at 28° | GM 12.48 m (fluid), max GZ 3.47 m at 28° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[26].message` | GM 12.54 m (fluid), max GZ 3.46 m at 28° | GM 12.56 m (fluid), max GZ 3.46 m at 28° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[3].message` | freeboard 2.92 m >= required 0.3 m | freeboard 2.93 m >= required 0.3 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[4].message` | cargo zone 59.19 m = LOA 88.0 - bow void 9.32 m - stern void 19.49 m | cargo zone 58.99 m = LOA 88.0 - bow void 9.32 m - stern void 19.69 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].message` | blind distance ahead of the bow 2741.8 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 6.00 m of 6.00 m | blind distance ahead of the bow 2751.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 6.00 m of 6.00 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[16].message` | aft separation 2.00 m >= 2.00 m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11) | distance 2.00 m from the cargo area (end space included) to any entrance or openable window >= 2.00 m (cl. 3.3.7 / 3.3.11) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[22].message` | tank length 49.59 m / inner diameter 10.04 m = 4.94 <= 7.0 | tank length 49.49 m / inner diameter 10.04 m = 4.93 <= 7.0 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[23].message` | freeboard 2.50 m at the carried draught 3.20 m >= required 0.3 m | freeboard 2.51 m at the carried draught 3.19 m >= required 0.3 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[24].message` | GM 3.73 m (fluid), max GZ 1.51 m at 25° | GM 3.74 m (fluid), max GZ 1.51 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[25].message` | GM 3.73 m (fluid), max GZ 1.51 m at 25° | GM 3.74 m (fluid), max GZ 1.52 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[26].message` | GM 10.30 m (fluid), max GZ 2.90 m at 25° | GM 10.31 m (fluid), max GZ 2.90 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[27].message` | GM 10.35 m (fluid), max GZ 2.90 m at 25° | GM 10.36 m (fluid), max GZ 2.90 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[4].message` | freeboard 2.50 m >= required 0.3 m | freeboard 2.51 m >= required 0.3 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[5].message` | cargo zone 101.19 m = LOA 130.0 - bow void 9.32 m - stern void 19.49 m | cargo zone 100.99 m = LOA 130.0 - bow void 9.32 m - stern void 19.69 m | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[15].message` | aft separation 2.00 m >= 2.00 m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11) | distance 2.00 m from the cargo area (end space included) to any entrance or openable window >= 2.00 m (cl. 3.3.7 / 3.3.11) | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[21].message` | tank length 49.59 m / inner diameter 7.20 m = 6.89 <= 7.0 | tank length 49.49 m / inner diameter 7.20 m = 6.88 <= 7.0 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[22].message` | freeboard 2.59 m at the carried draught 3.11 m >= required 0.3 m | freeboard 2.60 m at the carried draught 3.10 m >= required 0.3 m | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[23].message` | GM 5.66 m (fluid), max GZ 2.25 m at 27° | GM 5.67 m (fluid), max GZ 2.26 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[24].message` | GM 5.67 m (fluid), max GZ 2.26 m at 27° | GM 5.68 m (fluid), max GZ 2.26 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[25].message` | GM 13.42 m (fluid), max GZ 3.64 m at 27° | GM 13.43 m (fluid), max GZ 3.64 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[26].message` | GM 13.49 m (fluid), max GZ 3.64 m at 27° | GM 13.50 m (fluid), max GZ 3.64 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[3].message` | freeboard 2.59 m >= required 0.3 m | freeboard 2.60 m >= required 0.3 m | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[4].message` | cargo zone 101.19 m = LOA 130.0 - bow void 9.32 m - stern void 19.49 m | cargo zone 100.99 m = LOA 130.0 - bow void 9.32 m - stern void 19.69 m | changed |
| sweep default depth gas 90x14 | `checks[15].message` | aft separation 2.00 m >= 2.00 m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11) | distance 2.00 m from the cargo area (end space included) to any entrance or openable window >= 2.00 m (cl. 3.3.7 / 3.3.11) | changed |
| sweep default depth gas 90x14 | `checks[21].message` | tank length 26.04 m / inner diameter 9.23 m = 2.82 <= 7.0 | tank length 25.94 m / inner diameter 9.23 m = 2.81 <= 7.0 | changed |
| sweep default depth gas 90x14 | `checks[23].message` | GM 3.00 m (fluid), max GZ 1.89 m at 40° | GM 3.01 m (fluid), max GZ 1.89 m at 40° | changed |
| sweep default depth gas 90x14 | `checks[24].message` | GM 3.00 m (fluid), max GZ 1.88 m at 40° | GM 3.01 m (fluid), max GZ 1.88 m at 40° | changed |
| sweep default depth gas 90x14 | `checks[26].message` | GM 6.99 m (fluid), max GZ 2.11 m at 27° | GM 7.00 m (fluid), max GZ 2.11 m at 27° | changed |
| sweep default depth gas 90x14 | `checks[4].message` | cargo zone 54.09 m = LOA 90.0 - bow void 12.52 m - stern void 23.39 m | cargo zone 53.89 m = LOA 90.0 - bow void 12.52 m - stern void 23.59 m | changed |
| sweep gas IGC_2G 60x10 | `checks[15].message` | aft separation 2.00 m >= 2.00 m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11) | distance 2.00 m from the cargo area (end space included) to any entrance or openable window >= 2.00 m (cl. 3.3.7 / 3.3.11) | changed |
| sweep gas IGC_2G 60x10 | `checks[21].message` | tank length 12.80 m / inner diameter 8.34 m = 1.54 <= 7.0 | tank length 12.70 m / inner diameter 8.34 m = 1.52 <= 7.0 | changed |
| sweep gas IGC_2G 60x10 | `checks[22].message` | freeboard 2.65 m at the carried draught 2.85 m >= required 0.3 m | freeboard 2.66 m at the carried draught 2.84 m >= required 0.3 m | changed |
| sweep gas IGC_2G 60x10 | `checks[23].message` | GM 0.57 m (fluid), max GZ 0.58 m at 35° | GM 0.57 m (fluid), max GZ 0.59 m at 35° | changed |
| sweep gas IGC_2G 60x10 | `checks[24].message` | GM 0.54 m (fluid), max GZ 0.57 m at 35° | GM 0.55 m (fluid), max GZ 0.57 m at 35° | changed |
| sweep gas IGC_2G 60x10 | `checks[25].message` | GM 1.73 m (fluid), max GZ 0.86 m at 33° | GM 1.74 m (fluid), max GZ 0.87 m at 33° | changed |
| ... | 79448 more changed quantities | | | |
