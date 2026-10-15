## What this changes

## Why

Closes #

## Design impact

Filled in by CI (the `Design impact` section): "no design change", or a before/after table and the
`design-change` label. If a table appears, run `python3 tools/design_impact.py changelog` and commit
`CHANGELOG.md`; the fast gate fails until the table is there. No sign-off is needed.

## How it was checked

- [ ] `bash tools/gates.sh` passes (fast gate: Rust, design impact, pytest)
- [ ] `python tools/release_guard.py` passes
- [ ] New calculations have golden, property and feasibility tests, with sources cited
- Sourced tests changed: none / list each (a `Label: sourced` test changes only with its source named)
