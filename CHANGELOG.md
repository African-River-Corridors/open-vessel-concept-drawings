# Changelog

## Unreleased

- Behaviour back to 0.1.0 (issue #128, the rows of the ticket):
  - `simulate` reports a `ruleset` that is not text (`None`, a number) as `{"error": "Unknown ruleset None."}`, and reads numbers written as text (`"loa_m": "80"`) as `float()` does; text that is not a number raises `ValueError`.
  - `protective_location.required_side_protection` raises `ValueError` for plain text such as `"IGC_2G"`; `required_bottom_protection` treats plain text `"ADN_G"` as not ADN, as 0.1.0 did.
  - `cargo.get_cargo(5)` raises `ValueError`; `smallest_viable_loa(..., depth=6.5)` raises the same `TypeError` message as 0.1.0 (`fit() got an unexpected keyword argument 'depth'`).
  - `BargeCapacityInputs(loa=float("nan"))` and `inf` compute again, with the same answers and errors as 0.1.0. (`loa=True` is accepted again by the fix for issue #127.)
- Known changes from 0.1.0 (decided, not matched; issue #128):
  - A whole number given for a float field comes back as a float in echoed fields (`90` in, `90.0` out) in `gas_arrangement.fit`, `barge_cost`, `aft_arrangement`, `resistance`, `tanks`, `gas_separation` and `protective_location`. The values are equal (`90 == 90.0`). Matching would mean the core tracking int against float for every echo.
  - A zero container pitch raises `ZeroDivisionError` with the text "float floor division by zero". 0.1.0 said "integer division or modulo by zero" when the inputs were ints. The type is the same.
  - A numpy `float32` input is computed in float64 and returns a Python float. 0.1.0 did the sums in float32 and returned a numpy float32, so its answer differed in the last digits.
  - A float that is not whole in an int field (`n_holds=4.5`) raises `TypeError`. 0.1.0 let it through and often ignored it.
- Fix: functions use the object they are given again, as 0.1.0 did. `hydrostatics.calculate_stability` and `calculate_trim` use the hull, tanks and weights passed in; `weights.calculate_weights` uses the hull and tanks; `ga_adapter.to_ga`, `journey_derating.compute` and `gas_stability.assess` use the capacity result passed in. A result rebuilt with `dataclasses.replace` or built by hand works (issue #126). No calculation result changes.
- Fix: whole numbers (`90` for `90.0`) and numpy numbers are accepted again wherever a float is taken, on every public path (`HullGeometry` methods, `JourneyDeratingInputs` and the dataclass inputs). A whole float, a numpy integer or a `bool` is accepted for an int field such as `n_holds` (issue #127). No calculation result changes.
- Fix: a capacity result now keeps a copy of its inputs, so editing or reusing the inputs object after `barge_capacity.compute` no longer changes `journey_derating`, `gas_stability.assess` or `ga_adapter.is_gas` (issue #125). No calculation result changes.
- Fix: publishing the app installers now needs `test` and `release-guard` to pass on a tagged commit that is on `main`, and runs the release gate (wheels, binaries, pytest on each system) first. No calculation result changes.

## 0.2.2 (2026-10-15)

- Fix: the macOS app is now signed as a whole bundle (ad-hoc), so macOS no longer reports it as damaged. No calculation result changes.

## 0.2.1 (2026-10-14)

- Windows and macOS installers: the `app-build` workflow builds an NSIS `-setup.exe` and `.dmg` files, and a pushed `v*` tag publishes them as a GitHub pre-release (unsigned) with a SHA-256 checksum file. The README says how to run the app.
- A first-run notice in the app (concept design tool, unverified results, no warranty), shown once and reachable from About.
- The desktop app: a Design with one Option at a time, live results, a compare table, and the side elevation, plan and midship section of the active Option.
- Messages in the app use the plain input labels. When an input has a hard error, the app says why the results are out of date and links to the input.
- Fix: switching Formation back from a pushed convoy no longer leaves a hard error; the convoy counts come back when the convoy does.
- The drawing crate follows the Drawing Format Standard at commit 4936998: the status is a small line of text, not a diagonal watermark. DXF files are written with every drawing.
- No calculation result changes.
- Add the Design session to `ovcd-format` and the `ovcd schema` command, which prints the input schema (no result changes).
- Add the `ovcd-draw` crate (the Drawing Format Standard in Rust, first part) with the standard's conformance suite vendored at a pinned commit, and `ovcd draw run` (no result changes).
- Finish the standard in `ovcd-draw`: hatches, dimensions and leaders, symbols, title block, register and strips. All 194 conformance cases pass (no result changes).
- Add the SVG and PDF writers of `ovcd-draw` (from normalised content only; the PDF embeds the standard's font) and `ovcd draw build` (no result changes).
- Add the GA view builder to `ovcd-format` (side elevation, plan and midship section of an Option, as drawing documents) and `ovcd draw <file.vdesign>` to write them as SVG, PDF and normalised content, with a CI check that the Rust and Python implementations of the standard agree on every fixture view (no result changes).
- Show the side elevation, plan and midship section of the active Option in the app, drawn in Rust to the Drawing Format Standard (no result changes).
- Add the DXF writer of `ovcd-draw` (model space in metres, a paper-space layout `SHEET` with viewports at their scales) and write a DXF with every drawing from `ovcd draw` (no result changes).
- Update the vendored Drawing Format Standard to the version with a small status line instead of the diagonal watermark; the GA views show it as one line in their notes strip, and the strip is 3 mm higher to hold it. All 200 conformance cases pass (no result changes).

## Design changes

Each entry is a before/after table written by `python3 tools/design_impact.py changelog` when a
branch moved the results of the fixed design set. No sign-off is needed; the entries are kept so
a change can be traced later. Newest first.
