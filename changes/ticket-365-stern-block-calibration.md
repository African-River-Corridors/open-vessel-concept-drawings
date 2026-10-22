<!-- unreleased -->
- Calibration (issue #365): the stern block of a self-propelled Design with a specified Propulsor is calibrated against built ships. `calibration/propulsion_cases.yaml` gains `stern_block_case`: 28 ships and published standard designs (tugs, pushers, tankers, bulk carriers, gas tankers) whose aft peak and stern block are measured from the general arrangement on the maker's or designer's sheet (derived lengths with the source, its SHA-256, the scale and the reading tolerance; 23 are fitted, 5 are kept out with the reason). `python3 tools/stern_block_calibration.py <ovcd> --write` runs the layout against them and writes `calibration/stern_block_results.md`. The layout was 4.7 m (28 %) too short on average (RMS 5.8 m). Two fitted Estimates fix it: the aft peak is 2.5 m (was 1.5 m) and the drive room or shaft run plus the engine room are drawn 1.4 times as long, for the switchboard room, workshop, stores, auxiliaries and access. The mean error is now -0.3 m (RMS 3.3 m; inland ships only -0.7 m, RMS 2.8 m); 9 of the 28 ships are outside the new band (`stern_block_m` in `bands.yaml`, +-25/30/35 %, an estimate). The shaft mass, the gas rooms and the battery room do not take the factor. A Design with an unspecified Propulsor keeps its stern block, so no old file moves; the stern block of a Design with a specified Propulsor is longer, and the cargo zone shorter by as much.

<!-- design-impact -->
### ticket/365-stern-block-calibration (2026-10-22)

Between `main 54e0b1f` and `this branch 1a85aea`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived bulk 1 (88x16.3) | `checks[12].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 1.65 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 1.45 m of 6.00 m | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[16].message` | GM 4.82 m (fluid), max GZ 1.31 m at 25° | GM 4.82 m (fluid), max GZ 1.30 m at 25° | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[17].message` | GM 4.83 m (fluid), max GZ 1.30 m at 25° | GM 4.83 m (fluid), max GZ 1.29 m at 25° | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[19].message` | GM 16.04 m (fluid), max GZ 4.18 m at 30° | GM 16.03 m (fluid), max GZ 4.18 m at 30° | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[12].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 2.40 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 2.26 m of 6.00 m | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[16].message` | GM 3.79 m (fluid), max GZ 0.84 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | GM 3.79 m (fluid), max GZ 0.83 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `checks[12].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 2.35 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 2.21 m of 6.00 m | changed |
| sweep CN-derived bulk 3 (130x17.2) | `checks[18].message` | GM 18.96 m (fluid), max GZ 4.52 m at 30° | GM 18.96 m (fluid), max GZ 4.52 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.60 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.54 m of 6.00 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[21].message` | tank length 28.49 m / inner diameter 6.80 m = 4.19 <= 7.0 | tank length 27.29 m / inner diameter 6.80 m = 4.02 <= 7.0 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[22].message` | freeboard 2.93 m at the carried draught 2.77 m >= required 0.3 m | freeboard 3.01 m at the carried draught 2.69 m >= required 0.3 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[23].message` | GM 5.89 m (fluid), max GZ 2.51 m at 29° | GM 6.09 m (fluid), max GZ 2.57 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[24].message` | GM 5.90 m (fluid), max GZ 2.51 m at 29° | GM 6.10 m (fluid), max GZ 2.56 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[25].message` | GM 12.48 m (fluid), max GZ 3.47 m at 28° | GM 12.70 m (fluid), max GZ 3.49 m at 28° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[26].message` | GM 12.56 m (fluid), max GZ 3.46 m at 28° | GM 12.79 m (fluid), max GZ 3.48 m at 28° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[3].message` | freeboard 2.93 m >= required 0.3 m | freeboard 3.01 m >= required 0.3 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[4].message` | cargo zone 58.99 m = LOA 88.0 - bow void 9.32 m - stern void 19.69 m | cargo zone 56.59 m = LOA 88.0 - bow void 9.32 m - stern void 22.09 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].message` | blind distance ahead of the bow 2751.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 6.00 m of 6.00 m | blind distance ahead of the bow 2800.2 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 6.00 m of 6.00 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[22].message` | tank length 49.49 m / inner diameter 10.04 m = 4.93 <= 7.0 | tank length 48.29 m / inner diameter 10.05 m = 4.80 <= 7.0 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[23].message` | freeboard 2.51 m at the carried draught 3.19 m >= required 0.3 m | freeboard 2.56 m at the carried draught 3.14 m >= required 0.3 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[24].message` | GM 3.74 m (fluid), max GZ 1.51 m at 25° | GM 3.82 m (fluid), max GZ 1.55 m at 26° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[25].message` | GM 3.74 m (fluid), max GZ 1.52 m at 25° | GM 3.82 m (fluid), max GZ 1.55 m at 26° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[26].message` | GM 10.31 m (fluid), max GZ 2.90 m at 25° | GM 10.44 m (fluid), max GZ 2.92 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[27].message` | GM 10.36 m (fluid), max GZ 2.90 m at 25° | GM 10.49 m (fluid), max GZ 2.91 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[4].message` | freeboard 2.51 m >= required 0.3 m | freeboard 2.56 m >= required 0.3 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[5].message` | cargo zone 100.99 m = LOA 130.0 - bow void 9.32 m - stern void 19.69 m | cargo zone 98.59 m = LOA 130.0 - bow void 9.32 m - stern void 22.09 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `warnings[1]` | The blind distance needs 10.10 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | The blind distance needs 10.02 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 5.52 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 5.46 m of 6.00 m | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[21].message` | tank length 49.49 m / inner diameter 7.20 m = 6.88 <= 7.0 | tank length 48.29 m / inner diameter 7.20 m = 6.71 <= 7.0 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[22].message` | freeboard 2.60 m at the carried draught 3.10 m >= required 0.3 m | freeboard 2.65 m at the carried draught 3.05 m >= required 0.3 m | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[23].message` | GM 5.67 m (fluid), max GZ 2.26 m at 27° | GM 5.78 m (fluid), max GZ 2.30 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[24].message` | GM 5.68 m (fluid), max GZ 2.26 m at 27° | GM 5.79 m (fluid), max GZ 2.30 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[25].message` | GM 13.43 m (fluid), max GZ 3.64 m at 27° | GM 13.58 m (fluid), max GZ 3.66 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[26].message` | GM 13.50 m (fluid), max GZ 3.64 m at 27° | GM 13.65 m (fluid), max GZ 3.65 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[3].message` | freeboard 2.60 m >= required 0.3 m | freeboard 2.65 m >= required 0.3 m | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[4].message` | cargo zone 100.99 m = LOA 130.0 - bow void 9.32 m - stern void 19.69 m | cargo zone 98.59 m = LOA 130.0 - bow void 9.32 m - stern void 22.09 m | changed |
| sweep box self-propelled 60x10 | `checks[12].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 0.17 m of 6.00 m | blind distance ahead of the bow 245.1 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 0.00 m of 6.00 m | changed |
| sweep box self-propelled 60x10 | `checks[14].message` | intact GM 1.98 m >= 0.15 m at the reported cargo 878 t, draught 2.68 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 2.28 m >= 0.15 m at the reported cargo 728 t, draught 2.41 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| sweep box self-propelled 60x10 | `checks[16].message` | GM 2.03 m (fluid), max GZ 0.49 m at 25° | GM 2.37 m (fluid), max GZ 0.68 m at 28° | changed |
| sweep box self-propelled 60x10 | `checks[17].message` | GM 2.03 m (fluid), max GZ 0.46 m at 25° | GM 2.39 m (fluid), max GZ 0.65 m at 27° | changed |
| ... | 106439 more changed quantities | | | |
