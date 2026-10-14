## What this changes

## Why

Closes #

## Design impact

Filled in by CI (the `Design impact` section): "no design change", or a before/after table and the
`design-change` label. If a table appears, run `python3 tools/design_impact.py changelog` and commit
`CHANGELOG.md`; the fast gate fails until the table is there. No sign-off is needed.

## How it was checked

- [ ] `bash tools/gates.sh` passes (fast gate: Rust, parity, oracle guard, pytest)
- [ ] `python tools/release_guard.py` passes
- [ ] New calculations have golden, property and feasibility tests, with sources cited
