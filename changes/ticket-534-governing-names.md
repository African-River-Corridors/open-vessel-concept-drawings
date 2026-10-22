<!-- unreleased -->
- Design change (issue #534): the result key `stability.governing` is now `stability.lowest_gm_condition`; it names the Loading Condition with the lowest fluid GM. The app reads the new key. The other name the ticket listed, the check "Air draft (empty, governing)", no longer exists: #421 removed it, and the light air draft is checked by `air draft within limit`. The Python class `StabilityAssessment` keeps its `governing` method, because it is public API of 0.1.0. Every result that carries a stability block gets a new results hash; stored fixture hashes are re-stamped. No value or verdict changes.

<!-- design-impact -->
### ticket/534-governing-names (2026-10-22)

Between `the parent commit b7ae9d6` and `this branch 5d5ec18`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| calibration amaggi-box-2000 | `stability.governing` | Full load, departure | absent | changed |
| calibration amaggi-box-2000 | `stability.lowest_gm_condition` | absent | Full load, departure | changed |
| calibration canal-box-200x25x12-pb2230 | `stability.governing` | Full load, departure | absent | changed |
| calibration canal-box-200x25x12-pb2230 | `stability.lowest_gm_condition` | absent | Full load, departure | changed |
| calibration canal-box-200x35x12-cby222 | `stability.governing` | Full load, departure | absent | changed |
| calibration canal-box-200x35x12-cby222 | `stability.lowest_gm_condition` | absent | Full load, departure | changed |
| calibration canal-box-200x35x12-imt201 | `stability.governing` | Full load, departure | absent | changed |
| calibration canal-box-200x35x12-imt201 | `stability.lowest_gm_condition` | absent | Full load, departure | changed |
| calibration canal-box-200x35x12-imt205 | `stability.governing` | Full load, departure | absent | changed |
| calibration canal-box-200x35x12-imt205 | `stability.lowest_gm_condition` | absent | Full load, departure | changed |
| calibration canal-box-200x35x12-pb2012 | `stability.governing` | Full load, departure | absent | changed |
| calibration canal-box-200x35x12-pb2012 | `stability.lowest_gm_condition` | absent | Full load, departure | changed |
| calibration canal-box-200x35x12-rci250 | `stability.governing` | Full load, departure | absent | changed |
| calibration canal-box-200x35x12-rci250 | `stability.lowest_gm_condition` | absent | Full load, departure | changed |
| calibration canal-box-200x35x13-cby252 | `stability.governing` | Full load, departure | absent | changed |
| calibration canal-box-200x35x13-cby252 | `stability.lowest_gm_condition` | absent | Full load, departure | changed |
| calibration canal-box-200x35x13-cby400 | `stability.governing` | Full load, departure | absent | changed |
| calibration canal-box-200x35x13-cby400 | `stability.lowest_gm_condition` | absent | Full load, departure | changed |
| calibration canal-box-200x35x13-cby433 | `stability.governing` | Full load, departure | absent | changed |
| calibration canal-box-200x35x13-cby433 | `stability.lowest_gm_condition` | absent | Full load, departure | changed |
| calibration canal-box-200x35x13-pb2352 | `stability.governing` | Full load, departure | absent | changed |
| calibration canal-box-200x35x13-pb2352 | `stability.lowest_gm_condition` | absent | Full load, departure | changed |
| calibration cargill-amazon-jumbo | `stability.governing` | Full load, departure | absent | changed |
| calibration cargill-amazon-jumbo | `stability.lowest_gm_condition` | absent | Full load, departure | changed |
| calibration danube-barge-77 | `stability.governing` | Full load, departure | absent | changed |
| calibration danube-barge-77 | `stability.lowest_gm_condition` | absent | Full load, departure | changed |
| calibration dsb-holdbarge-1997-76.5 | `stability.governing` | Full load, departure | absent | changed |
| calibration dsb-holdbarge-1997-76.5 | `stability.lowest_gm_condition` | absent | Full load, departure | changed |
| calibration europa-iib-1635ou | `stability.governing` | Full load, departure | absent | changed |
| calibration europa-iib-1635ou | `stability.lowest_gm_condition` | absent | Full load, departure | changed |
| calibration europa-iib-rdb06 | `stability.governing` | Full load, departure | absent | changed |
| calibration europa-iib-rdb06 | `stability.lowest_gm_condition` | absent | Full load, departure | changed |
| calibration hidrovia-lhg-3200 | `stability.governing` | Full load, departure | absent | changed |
| calibration hidrovia-lhg-3200 | `stability.lowest_gm_condition` | absent | Full load, departure | changed |
| calibration jeffboat-am2100 | `stability.governing` | Full load, departure | absent | changed |
| calibration jeffboat-am2100 | `stability.lowest_gm_condition` | absent | Full load, departure | changed |
| calibration mcdonough-195x35x12 | `stability.governing` | Full load, departure | absent | changed |
| calibration mcdonough-195x35x12 | `stability.lowest_gm_condition` | absent | Full load, departure | changed |
| calibration meb-dcb23-ptich | `stability.governing` | Full load, departure | absent | changed |
| calibration meb-dcb23-ptich | `stability.lowest_gm_condition` | absent | Full load, departure | changed |
| ... | 433 more changed quantities | | | |
