<!-- unreleased -->
- Cost (issue #404): a gas barge result now warns when the top-down hull steel of the lightship and the bottom-up hull steel that the cost prices differ by more than 20 % of the bottom-up steel. The limit is an Estimate, to calibrate against the reference vessel library. The message gives both masses and the delta. The cost keeps pricing the bottom-up steel. Only the `warnings` list changes.

<!-- design-impact -->
### ticket/404-steel-warning (2026-10-23)

Between `main aff1173` and `this branch 47b942e1`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep default depth gas 90x14 | `warnings[2]` | absent | The lightship uses 970.2 t of hull steel (top-down); the cost prices 778.7 t (bottom-up). They differ by 191.5 t (24.6 %), more than the 20 % limit (an Estimate). The cost keeps the bottom-up mass. | changed |
| sweep gas wide hull IGC_2G 110x24 | `warnings[2]` | absent | The lightship uses 1716.0 t of hull steel (top-down); the cost prices 1136.4 t (bottom-up). They differ by 579.6 t (51.0 %), more than the 20 % limit (an Estimate). The cost keeps the bottom-up mass. | changed |
| sweep gas wide hull IGC_2G_EXACT 110x24 | `warnings[2]` | absent | The lightship uses 1716.0 t of hull steel (top-down); the cost prices 1136.4 t (bottom-up). They differ by 579.6 t (51.0 %), more than the 20 % limit (an Estimate). The cost keeps the bottom-up mass. | changed |
