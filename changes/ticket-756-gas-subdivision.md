<!-- unreleased -->
- Gas hull subdivision (issue #756, spec 0012 D6): one hold per tank in line (`H1C`, `H2C`, ...), with a watertight bulkhead midway between the heads of two tanks in line, placed from the Tank Layout. The double bottom and the wing tanks are cut at the same bulkheads (`DB1P`, `DB2P`, ...), and the strip above each wing tank, outboard of the hold, is a void Space (`WV1P`, ...). The cargo capacity does not move. The gas loading conditions list the cut ballast tanks; the ballast plan fills each kind together, so its mass and centre are the same, with last-digit changes in the stability numbers.

<!-- design-impact -->
### ticket/756-gas-subdivision (2026-10-31)

Between `main 0bab67f` and `this branch b3ce955b`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[2].kind` | wing_ballast | double_bottom_ballast | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[2].name` | WB1P | DB2P | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[3].kind` | wing_ballast | double_bottom_ballast | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[3].name` | WB1S | DB2S | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[4].kind` | peak_ballast | wing_ballast | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[4].name` | FP1C | WB1P | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[4].side` | centre | port | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[5].capacity_m3` | absent | 28.4128348 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[5].capacity_t` | absent | 28.4128348 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[5].kind` | absent | wing_ballast | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[5].lcg_m` | absent | 36.239 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[5].name` | absent | WB1S | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[5].side` | absent | starboard | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[5].tcg_m` | absent | 8.05 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[5].vcg_m` | absent | 3.09716666667 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[6].capacity_m3` | absent | 28.4128348 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[6].capacity_t` | absent | 28.4128348 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[6].kind` | absent | wing_ballast | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[6].lcg_m` | absent | 64.533 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[6].name` | absent | WB2P | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[6].side` | absent | port | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[6].tcg_m` | absent | -8.05 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[6].vcg_m` | absent | 3.09716666667 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[7].capacity_m3` | absent | 28.4128348 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[7].capacity_t` | absent | 28.4128348 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[7].kind` | absent | wing_ballast | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[7].lcg_m` | absent | 64.533 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[7].name` | absent | WB2S | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[7].side` | absent | starboard | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[7].tcg_m` | absent | 8.05 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[7].vcg_m` | absent | 3.09716666667 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[8].capacity_m3` | absent | 442.2516 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[8].capacity_t` | absent | 442.2516 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[8].kind` | absent | peak_ballast | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[8].lcg_m` | absent | 81.7880672269 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[8].name` | absent | FP1C | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[8].side` | absent | centre | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[8].tcg_m` | absent | 1.38814404221e-16 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `loading_conditions.ballast_tanks[8].vcg_m` | absent | 3.76008403361 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `loading_conditions.ballast_tanks[2].kind` | wing_ballast | double_bottom_ballast | changed |
| ... | 7133 more changed quantities | | | |
