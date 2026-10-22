<!-- unreleased -->
- Design change (issue #280, maintainer decision of 2026-10-22): the end-bulkhead space aft and forward of the gas tanks follows the type G rule, 0.20 m for domed tank ends (0.50 m for plane ends; Lloyd's Register inland waterways rules Pt 4 Ch 4 3.5.3, ADN 2025 9.3.1.11.3(a)), in place of 0.60 m aft (the type C cofferdam) and 0.50 m forward. They are now two stored inputs, `aft_end_space_m` and `fwd_end_space_m` (Gas tanks group). A new Option takes 0.20 m from the Defaults Profile; a file that lacks them keeps 0.60 m and 0.50 m with one note, so no existing file moves. The aft separation stays 2.00 m (the opening setback governs), so the cargo gain comes from the forward end: 0.30 m more cargo zone (issue #381 later measures the setback from the cargo area). User-visible change in the Python API: the defaults of `GasSeparationInputs` (cofferdam and forward end space) move from 0.60 m and 0.50 m to 0.20 m, and the cargo of `fit`, `compute` and `simulate` for a gas Design rises accordingly. Examples of new Options: the CN II motor gas barge 2,021.2 t to 2,033.4 t (+0.6 %), a 60 x 11 m hull 714.4 t to 726.2 t.

<!-- design-impact -->
### ticket/280-type-g-default (2026-10-22)

Between `main fc7e931` and `this branch dc51879`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.59 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 3.60 m of 6.00 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[14].message` | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | aft end-bulkhead space 0.20 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[16].message` | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | forward end-bulkhead space 0.20 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[21].message` | tank length 28.44 m / inner diameter 6.80 m = 4.18 <= 7.0 | tank length 28.59 m / inner diameter 6.80 m = 4.21 <= 7.0 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[22].message` | freeboard 2.93 m at the carried draught 2.77 m >= required 0.3 m | freeboard 2.92 m at the carried draught 2.78 m >= required 0.3 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[23].message` | GM 5.90 m (fluid), max GZ 2.52 m at 29° | GM 5.87 m (fluid), max GZ 2.51 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[24].message` | GM 5.91 m (fluid), max GZ 2.52 m at 29° | GM 5.89 m (fluid), max GZ 2.51 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[25].message` | GM 12.49 m (fluid), max GZ 3.47 m at 28° | GM 12.46 m (fluid), max GZ 3.46 m at 28° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[26].message` | GM 12.58 m (fluid), max GZ 3.46 m at 28° | GM 12.54 m (fluid), max GZ 3.46 m at 28° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[3].message` | freeboard 2.93 m >= required 0.3 m | freeboard 2.92 m >= required 0.3 m | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[4].message` | cargo zone 58.89 m = LOA 88.0 - bow void 9.62 m - stern void 19.49 m | cargo zone 59.19 m = LOA 88.0 - bow void 9.32 m - stern void 19.49 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].message` | blind distance ahead of the bow 2747.3 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 6.00 m of 6.00 m | blind distance ahead of the bow 2741.8 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 6.00 m of 6.00 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[15].message` | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | aft end-bulkhead space 0.20 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[17].message` | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | forward end-bulkhead space 0.20 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[22].message` | tank length 49.44 m / inner diameter 10.04 m = 4.92 <= 7.0 | tank length 49.59 m / inner diameter 10.04 m = 4.94 <= 7.0 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[23].message` | freeboard 2.51 m at the carried draught 3.19 m >= required 0.3 m | freeboard 2.50 m at the carried draught 3.20 m >= required 0.3 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[24].message` | GM 3.74 m (fluid), max GZ 1.52 m at 25° | GM 3.73 m (fluid), max GZ 1.51 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[25].message` | GM 3.74 m (fluid), max GZ 1.52 m at 25° | GM 3.73 m (fluid), max GZ 1.51 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[26].message` | GM 10.32 m (fluid), max GZ 2.90 m at 25° | GM 10.30 m (fluid), max GZ 2.90 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[27].message` | GM 10.37 m (fluid), max GZ 2.90 m at 25° | GM 10.35 m (fluid), max GZ 2.90 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[4].message` | freeboard 2.51 m >= required 0.3 m | freeboard 2.50 m >= required 0.3 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[5].message` | cargo zone 100.89 m = LOA 130.0 - bow void 9.62 m - stern void 19.49 m | cargo zone 101.19 m = LOA 130.0 - bow void 9.32 m - stern void 19.49 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `warnings[1]` | The blind distance needs 10.08 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | The blind distance needs 10.10 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 5.51 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 5.52 m of 6.00 m | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[14].message` | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | aft end-bulkhead space 0.20 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[16].message` | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | forward end-bulkhead space 0.20 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[21].message` | tank length 49.44 m / inner diameter 7.20 m = 6.87 <= 7.0 | tank length 49.59 m / inner diameter 7.20 m = 6.89 <= 7.0 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[22].message` | freeboard 2.60 m at the carried draught 3.10 m >= required 0.3 m | freeboard 2.59 m at the carried draught 3.11 m >= required 0.3 m | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[23].message` | GM 5.67 m (fluid), max GZ 2.26 m at 27° | GM 5.66 m (fluid), max GZ 2.25 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[24].message` | GM 5.68 m (fluid), max GZ 2.27 m at 27° | GM 5.67 m (fluid), max GZ 2.26 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[25].message` | GM 13.44 m (fluid), max GZ 3.64 m at 27° | GM 13.42 m (fluid), max GZ 3.64 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[26].message` | GM 13.51 m (fluid), max GZ 3.64 m at 27° | GM 13.49 m (fluid), max GZ 3.64 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[3].message` | freeboard 2.60 m >= required 0.3 m | freeboard 2.59 m >= required 0.3 m | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[4].message` | cargo zone 100.89 m = LOA 130.0 - bow void 9.62 m - stern void 19.49 m | cargo zone 101.19 m = LOA 130.0 - bow void 9.32 m - stern void 19.49 m | changed |
| sweep default depth gas 90x14 | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.96 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.98 m of 6.00 m | changed |
| sweep default depth gas 90x14 | `checks[14].message` | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | aft end-bulkhead space 0.20 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep default depth gas 90x14 | `checks[16].message` | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | forward end-bulkhead space 0.20 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep default depth gas 90x14 | `checks[21].message` | tank length 25.89 m / inner diameter 9.23 m = 2.81 <= 7.0 | tank length 26.04 m / inner diameter 9.23 m = 2.82 <= 7.0 | changed |
| sweep default depth gas 90x14 | `checks[22].message` | freeboard 4.88 m at the carried draught 2.82 m >= required 0.3 m | freeboard 4.87 m at the carried draught 2.83 m >= required 0.3 m | changed |
| sweep default depth gas 90x14 | `checks[23].message` | GM 3.02 m (fluid), max GZ 1.89 m at 40° | GM 3.00 m (fluid), max GZ 1.89 m at 40° | changed |
| ... | 78661 more changed quantities | | | |
