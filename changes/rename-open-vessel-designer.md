<!-- unreleased -->
- OVCD is now Open Vessel Designer (OVD). The repository, crates, the `ovd` command, the Python package (`open-vessel-designer`, `import ovd`) and the app use the new name. The old names no longer work. Design files keep the `.vdesign` extension, but files saved by 0.2.x do not open in 0.3.0.
- Rename (spec 0010, ADR 0022): the crates are `ovd-core`, `ovd-format`, `ovd-cli`, `ovd-py`, `ovd-shell`, `ovd-web` and `ovd-app`; the command is `ovd` (`ovd.exe`, `ovd.wasm`); the app is OVD, with the identifier `org.africanrivercorridors.ovd` and the file type "OVD Design"; environment variables start `OVD_` and browser storage keys `ovd.`, and the old ones are not read, so the app shows the first-run notice again and starts with no stored Design or settings. The project Rule Set is `ovd` 3.0.0, with the Rules of the old 2.1.0 and ids that start `ovd/`; its earlier versions are gone, and `RELEASED.json` no longer lists the old id. A file that stores the old Rule Set id fails with one error that names the rename. Every `.vdesign` in the repository stores `ovd` 3.0.0 with its hash. No result and no results hash moves: every fixture and example gives the same results before and after, apart from the Rule Set id, version and hash. The website moves from `/ovcd` to `/ovd`, with no redirect. `tools/old_name_gate.py` (in the fast gate) fails on the old name outside the history. The design-impact table has one row, the case that types the Rule Set id: the engine before the rename cannot read `ovd`, so that case failed there and computes here; the other 252 cases do not move. `tools/design_impact.py` reads the CLI package and binary names from the tree it builds, so it can build the base of this change.

<!-- design-impact -->
### rename/open-vessel-designer (2026-10-30)

Between `main 26d5710` and `this branch 124893fb`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep optional rule_sets | `result` | fill: warning: Option 'sweep optional rule_sets': `price_base_date` was not stored in the file; wrote in the Defaults Profile value none, marked default \| warning: Option 'sweep optional rule_sets': `service_area` was not stored in the file; wrote in the Defaults Profile value river-only, marked default \| ovcd fill: 1 error(s); nothing written | computed | changed |
