<!-- design-impact -->
### ticket/358-convoy-totals-fix (2026-10-22)

Between `main 54e0b1f` and `this branch 1efe004`.

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
