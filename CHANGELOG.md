# Changelog

## Unreleased

- Add the Design session to `ovcd-format` and the `ovcd schema` command, which prints the input schema (no result changes).
- Add the `ovcd-draw` crate (the Drawing Format Standard in Rust, first part) with the standard's conformance suite vendored at a pinned commit, and `ovcd draw run` (no result changes).
- Finish the standard in `ovcd-draw`: hatches, dimensions and leaders, symbols, title block, register and strips. All 194 conformance cases pass (no result changes).

## Design changes

Each entry is a before/after table written by `python3 tools/design_impact.py changelog` when a
branch moved the results of the fixed design set. No sign-off is needed; the entries are kept so
a change can be traced later. Newest first.
