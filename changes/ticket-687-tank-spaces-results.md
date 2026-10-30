<!-- unreleased -->
- Results Spaces table (issue #687, spec 0009 D1 b, ADR 0021): a gas Design's results gain `spaces`, one row per Type C tank (name, kind, side, volume = the capacity's `volume_per_tank`, centroid = the middle of its axis). Added rows only: no existing value moves. The results hash of every gas Option changes, and the gas fixtures and examples are re-stamped.

<!-- design-impact -->
### ticket/687-tank-spaces-results (2026-10-30)

Between `main 6f8c7e7` and `this branch d0c40c25`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `spaces[0].kind` | absent | cargo_tank | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[0].lcg_m` | absent | 35.739 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[0].name` | absent | CT1P | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[0].side` | absent | port | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[0].tcg_m` | absent | -3.925 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[0].vcg_m` | absent | 4.51166666667 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[0].volume_m3` | absent | 948.581622 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[1].kind` | absent | cargo_tank | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[1].lcg_m` | absent | 35.739 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[1].name` | absent | CT1S | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[1].side` | absent | starboard | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[1].tcg_m` | absent | 3.925 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[1].vcg_m` | absent | 4.51166666667 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[1].volume_m3` | absent | 948.581622 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[2].kind` | absent | cargo_tank | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[2].lcg_m` | absent | 65.033 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[2].name` | absent | CT2P | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[2].side` | absent | port | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[2].tcg_m` | absent | -3.925 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[2].vcg_m` | absent | 4.51166666667 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[2].volume_m3` | absent | 948.581622 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[3].kind` | absent | cargo_tank | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[3].lcg_m` | absent | 65.033 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[3].name` | absent | CT2S | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[3].side` | absent | starboard | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[3].tcg_m` | absent | 3.925 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[3].vcg_m` | absent | 4.51166666667 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `spaces[3].volume_m3` | absent | 948.581622 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `spaces[0].kind` | absent | cargo_tank | changed |
| sweep CN-derived liquid 2 (130x16.3) | `spaces[0].lcg_m` | absent | 46.239 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `spaces[0].name` | absent | CT1C | changed |
| sweep CN-derived liquid 2 (130x16.3) | `spaces[0].side` | absent | centre | changed |
| sweep CN-derived liquid 2 (130x16.3) | `spaces[0].tcg_m` | absent | 0 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `spaces[0].vcg_m` | absent | 6.15261830238 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `spaces[0].volume_m3` | absent | 3698.64119127 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `spaces[1].kind` | absent | cargo_tank | changed |
| sweep CN-derived liquid 2 (130x16.3) | `spaces[1].lcg_m` | absent | 96.533 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `spaces[1].name` | absent | CT2C | changed |
| sweep CN-derived liquid 2 (130x16.3) | `spaces[1].side` | absent | centre | changed |
| sweep CN-derived liquid 2 (130x16.3) | `spaces[1].tcg_m` | absent | 0 | changed |
| ... | 1626 more changed quantities | | | |
