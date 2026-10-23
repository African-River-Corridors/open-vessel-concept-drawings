<!-- unreleased -->
- Consumables tanks inside the hull (issue #631): under a raked stern bottom the consumables pair lay partly outside the hull. The pair is now clipped by the hull and made long enough that its volume inside the hull holds the stores, so the stores mass is kept. GM and GZ move a little in the self-propelled cases (148 of 250 design-impact cases); no check changes its result.

<!-- design-impact -->
### ticket/631-consumables-inside (2026-10-23)

Between `main 584d862` and `this branch e651eb1e`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived bulk 1 (88x16.3) | `checks[16].message` | GM 4.82 m (fluid), max GZ 1.30 m at 25° | GM 4.81 m (fluid), max GZ 1.30 m at 25° | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[17].message` | GM 4.83 m (fluid), max GZ 1.29 m at 25° | GM 4.81 m (fluid), max GZ 1.28 m at 25° | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[18].message` | GM 15.90 m (fluid), max GZ 4.19 m at 31° | GM 15.85 m (fluid), max GZ 4.16 m at 31° | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[19].message` | GM 16.03 m (fluid), max GZ 4.18 m at 30° | GM 15.99 m (fluid), max GZ 4.16 m at 30° | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[16].message` | GM 3.79 m (fluid), max GZ 0.83 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | GM 3.78 m (fluid), max GZ 0.83 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[17].message` | GM 3.79 m (fluid), max GZ 0.83 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | GM 3.78 m (fluid), max GZ 0.83 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[18].message` | GM 16.43 m (fluid), max GZ 4.14 m at 30° | GM 16.40 m (fluid), max GZ 4.13 m at 29° | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[19].message` | GM 16.56 m (fluid), max GZ 4.14 m at 29° | GM 16.53 m (fluid), max GZ 4.12 m at 29° | changed |
| sweep CN-derived bulk 3 (130x17.2) | `checks[16].message` | GM 4.48 m (fluid), max GZ 0.93 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | GM 4.46 m (fluid), max GZ 0.92 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `checks[17].message` | GM 4.48 m (fluid), max GZ 0.93 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | GM 4.46 m (fluid), max GZ 0.92 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `checks[18].message` | GM 18.96 m (fluid), max GZ 4.52 m at 29° | GM 18.91 m (fluid), max GZ 4.50 m at 29° | changed |
| sweep CN-derived bulk 3 (130x17.2) | `checks[19].message` | GM 19.11 m (fluid), max GZ 4.52 m at 29° | GM 19.06 m (fluid), max GZ 4.50 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[23].message` | GM 6.09 m (fluid), max GZ 2.57 m at 29° | GM 6.08 m (fluid), max GZ 2.56 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[24].message` | GM 6.10 m (fluid), max GZ 2.56 m at 29° | GM 6.09 m (fluid), max GZ 2.56 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[25].message` | GM 12.70 m (fluid), max GZ 3.49 m at 28° | GM 12.69 m (fluid), max GZ 3.48 m at 28° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[26].message` | GM 12.79 m (fluid), max GZ 3.48 m at 28° | GM 12.78 m (fluid), max GZ 3.48 m at 27° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[26].message` | GM 10.44 m (fluid), max GZ 2.92 m at 25° | GM 10.43 m (fluid), max GZ 2.92 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[27].message` | GM 10.49 m (fluid), max GZ 2.91 m at 25° | GM 10.48 m (fluid), max GZ 2.91 m at 25° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[23].message` | GM 5.78 m (fluid), max GZ 2.30 m at 27° | GM 5.77 m (fluid), max GZ 2.30 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[24].message` | GM 5.79 m (fluid), max GZ 2.30 m at 27° | GM 5.78 m (fluid), max GZ 2.30 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[25].message` | GM 13.58 m (fluid), max GZ 3.66 m at 27° | GM 13.57 m (fluid), max GZ 3.65 m at 27° | changed |
| sweep box self-propelled 60x10 | `checks[16].message` | GM 2.37 m (fluid), max GZ 0.68 m at 28° | GM 2.35 m (fluid), max GZ 0.67 m at 28° | changed |
| sweep box self-propelled 60x10 | `checks[17].message` | GM 2.39 m (fluid), max GZ 0.65 m at 27° | GM 2.37 m (fluid), max GZ 0.64 m at 27° | changed |
| sweep box self-propelled 60x10 | `checks[18].message` | GM 5.92 m (fluid), max GZ 1.78 m at 25° | GM 5.87 m (fluid), max GZ 1.76 m at 25° | changed |
| sweep box self-propelled 60x10 | `checks[19].message` | GM 5.95 m (fluid), max GZ 1.77 m at 25° | GM 5.86 m (fluid), max GZ 1.75 m at 25° | changed |
| sweep box self-propelled 90x14 | `checks[19].message` | GM 18.47 m (fluid), max GZ 3.84 m at 26° | GM 18.46 m (fluid), max GZ 3.84 m at 26° | changed |
| sweep crude coastal service 90x14 | `checks[19].message` | GM 5.47 m (fluid), max GZ 1.78 m at 26° | GM 5.47 m (fluid), max GZ 1.77 m at 26° | changed |
| sweep crude self-propelled 60x10 | `checks[15].message` | GM 3.62 m (fluid), max GZ 1.53 m at 33° | GM 3.60 m (fluid), max GZ 1.51 m at 33° | changed |
| sweep crude self-propelled 60x10 | `checks[16].message` | GM 3.66 m (fluid), max GZ 1.50 m at 33° | GM 3.64 m (fluid), max GZ 1.49 m at 33° | changed |
| sweep crude self-propelled 60x10 | `checks[17].message` | GM 6.85 m (fluid), max GZ 1.95 m at 25° | GM 6.80 m (fluid), max GZ 1.93 m at 25° | changed |
| sweep crude self-propelled 60x10 | `checks[18].message` | GM 5.78 m (fluid), max GZ 1.74 m at 25° | GM 5.74 m (fluid), max GZ 1.72 m at 25° | changed |
| sweep crude self-propelled 60x10 | `checks[19].message` | GM 3.69 m (fluid), max GZ 1.56 m at 33° | GM 3.67 m (fluid), max GZ 1.54 m at 33° | changed |
| sweep crude self-propelled 90x14 | `checks[15].message` | GM 5.22 m (fluid), max GZ 1.66 m at 26° | GM 5.22 m (fluid), max GZ 1.65 m at 26° | changed |
| sweep default depth dry bulk 90x14 | `checks[19].message` | GM 18.05 m (fluid), max GZ 3.80 m at 26° | GM 18.04 m (fluid), max GZ 3.79 m at 26° | changed |
| sweep default depth gas 90x14 | `checks[25].message` | GM 7.02 m (fluid), max GZ 2.14 m at 28° | GM 7.01 m (fluid), max GZ 2.14 m at 28° | changed |
| sweep default depth liquid 90x14 | `checks[15].message` | GM 6.30 m (fluid), max GZ 1.98 m at 27° | GM 6.29 m (fluid), max GZ 1.97 m at 27° | changed |
| sweep default depth liquid 90x14 | `checks[17].message` | GM 18.03 m (fluid), max GZ 3.86 m at 27° | GM 18.02 m (fluid), max GZ 3.85 m at 27° | changed |
| sweep default depth liquid 90x14 | `checks[18].message` | GM 18.37 m (fluid), max GZ 3.86 m at 26° | GM 18.35 m (fluid), max GZ 3.85 m at 26° | changed |
| sweep default depth liquid 90x14 | `checks[19].message` | GM 5.82 m (fluid), max GZ 1.79 m at 25° | GM 5.81 m (fluid), max GZ 1.79 m at 25° | changed |
| sweep diesel coastal service 90x14 | `checks[18].message` | GM 17.12 m (fluid), max GZ 3.75 m at 26° | GM 17.11 m (fluid), max GZ 3.75 m at 26° | changed |
| ... | 90828 more changed quantities | | | |
