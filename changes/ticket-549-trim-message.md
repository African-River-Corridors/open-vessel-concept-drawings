<!-- unreleased -->
- Design change (issue #549): when `trim_in_range` fails, its message says "no trim balances the load", names the condition, and gives its LCG and the furthest LCB the hull reaches (for example "Laden: LCG 53.78 m is forward of the furthest LCB the hull reaches (50.24 m)"), not the 10 degree limit, because the cause is that the deck goes under before any trim balances the load. The verdict does not change. Six dry bulk sweep Options (self-propelled, 90 x 14 m) show the new text; the message when the check passes is the same.

<!-- design-impact -->
### ticket/549-trim-message (2026-10-22)

Between `main fc7e931` and `this branch c6ef045`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep box self-propelled 90x14 | `checks[13].message` | the hull floats within 10 degrees of trim (Laden) | no trim balances the load: Laden: LCG 53.78 m is forward of the furthest LCB the hull reaches (50.24 m) | changed |
| sweep default depth dry bulk 90x14 | `checks[13].message` | the hull floats within 10 degrees of trim (Laden) | no trim balances the load: Laden: LCG 53.73 m is forward of the furthest LCB the hull reaches (50.24 m) | changed |
| sweep hull midship section in a channel 80x11.4 | `checks[13].message` | the hull floats within 10 degrees of trim (Laden) | no trim balances the load: Laden: LCG 45.77 m is forward of the furthest LCB the hull reaches (43.23 m) | changed |
| sweep optional aft_block_all_dry | `checks[13].message` | the hull floats within 10 degrees of trim (Laden) | no trim balances the load: Laden: LCG 53.78 m is forward of the furthest LCB the hull reaches (50.24 m) | changed |
| sweep optional propulsive_efficiency | `checks[13].message` | the hull floats within 10 degrees of trim (Laden) | no trim balances the load: Laden: LCG 53.80 m is forward of the furthest LCB the hull reaches (50.25 m) | changed |
| sweep optional vessel_class | `checks[14].message` | the hull floats within 10 degrees of trim (Laden) | no trim balances the load: Laden: LCG 59.80 m is forward of the furthest LCB the hull reaches (54.41 m) | changed |
| sweep optional vessel_scheme | `checks[14].message` | the hull floats within 10 degrees of trim (Laden) | no trim balances the load: Laden: LCG 61.10 m is forward of the furthest LCB the hull reaches (55.21 m) | changed |
