<!-- unreleased -->
- Feature (issue #358): a pushed convoy has results. The Option's inputs describe one barge; `simulate` runs it as a pushed barge and the result gains a `convoy` block: the barge, and the totals of every barge (cargo, lightship, deadweight, displacement, cargo volume, overall length = barges in line x length, overall beam = barges abreast x beam). The pusher is not in the totals (#359). "convoy results not yet available" is gone for a convoy; its drawings say "no drawings of a pushed convoy yet". `ovcd run` prints a Convoy line. Two new Waterway Limits inputs, `lock_length_m` and `lock_width_m` (default none), add the checks "within lock length" and "within lock width" on the overall size of a self-propelled hull, a pushed barge or a convoy; a file saved before keeps none. The fixture `pushed_convoy` now stores its results hash. No other design moves.

<!-- design-impact -->
### ticket/358-convoy-totals (2026-10-22)

Between `main fc7e931` and `this branch 799d0a2`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep box convoy 2x3 | `result` | convoy results not yet available | computed | changed |
| sweep box convoy 2x3 in a lock | `result` | convoy results not yet available | computed | changed |
| sweep optional lock_length_m | `checks[20].message` | absent | overall length 90.00 m <= lock length 85.0 m | changed |
| sweep optional lock_length_m | `checks[20].name` | absent | within lock length | changed |
| sweep optional lock_length_m | `checks[20].ok` | absent | False | changed |
| sweep optional lock_length_m | `checks[20].severity` | absent | error | changed |
| sweep optional lock_width_m | `checks[12].message` | absent | overall beam 14.00 m <= lock width 13.0 m | changed |
| sweep optional lock_width_m | `checks[12].name` | absent | within lock width | changed |
| sweep optional lock_width_m | `checks[12].ok` | absent | False | changed |
| sweep optional lock_width_m | `checks[12].severity` | absent | error | changed |
