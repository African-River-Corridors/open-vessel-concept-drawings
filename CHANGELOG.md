# Changelog

## Unreleased

- Addition (issue #180): a gas-fuelled self-propelled stern. The default stays battery-electric, and its results do not change.
  - New input `propulsion` on `AftArrangementInputs` and `BargeCapacityInputs`: `battery_electric` (default, as 0.1.0) or `gas_fuelled`. Any other value is a `ValueError`. `BargeCapacityInputs.propulsion` sets the default stern block when `aft` is not given; a given `aft` must agree.
  - Gas-fuelled: no battery units. Gas gensets in an engine room below deck burn the cargo boil-off, with a gas valve unit (GVU) room forward of it. The accommodation units stand beside the wheelhouse when the beam takes them (check `accom_beside_wheelhouse`, a warning; else they go in a row aft of the cab). The block is as long as the longer of the deck items and the rooms below. New checks `gensets_fit_beam`, `gensets_fit_under_deck` (warning) and `gensets_carry_load_with_one_in_reserve`.
  - Genset catalogue `aft_arrangement.GAS_GENSETS` (Rust and Python): eight Weichai WH20/WH25 dual-fuel sets, 900 to 2200 kWe, from Weichai Power, "Weichai Medium Speed Marine Engine" (April 2023), slides 12, 15, 30 and 31. Input `electrical_load_kwe` selects the set and count: the least installed power with one set in reserve, 2 to 6 sets, fewer sets on a tie. The alternator and base-frame allowances and the room clearances are ESTIMATES and the result says so.
  - New inputs `electrical_load_kwe`, `n_gensets`, `genset_model`, `genset_kwe`, `genset_length_m`, `genset_width_m`, `genset_height_m`, `genset_engine_mass_t`, `genset_clear_m`, `engine_room_end_clear_m`, `gvu_room_length_m`. New result fields `propulsion`, `container_layout`, `container_units`, `engine_room_x0_m`, `engine_room_x1_m`, `gvu_room_x0_m`, `gvu_room_x1_m`, `n_gensets`, `genset_model`, `genset_engine_model`, `genset_kwe`, `installed_kwe`, `genset_offsets_m`, `genset_x0_m`, `genset_length_m`, `genset_width_m`, `genset_height_m`, `genset_engine_mass_t`, `below_deck_length_m`, `deck_length_m`.
  - Python, known change from 0.1.0: `AftArrangementInputs.n_battery_units` is `Optional[int]`, default `None`, which means the propulsion's own default (1 battery-electric, 0 gas-fuelled). A given int is used as before.
  - Tests: the API oracle drops only these added keys before it compares, so every 0.1.0 key and value is still checked; `tests/data/public_api.json` lists the new names.
- Design change (issue #175): gas tanks follow IGC rules only. The before/after table is under "Design changes" below.
  - The `ADN_G` ruleset is removed: its per-tank volume cap, the tanks-along search, the wide-tank (0.70 B) check, the ADN stability criteria set and the ADN vessel type. The remaining rulesets are `IGC_2G` and `IGC_2G_EXACT`. The input `adn_tank_cap_m3` is removed.
  - New input `max_tank_od_m`, the cap on a tank's outer diameter, 16 m by default (it was the fixed constant `MAX_TANK_OD_M`, 12 m). It applies under both rulesets and is shown with its basis in `ovcd schema` and the app.
  - Length over diameter is at most 7 for every pressure tank under both rulesets (it was only under `ADN_G`). A candidate over it fails the check `tank_length_over_diameter`, so a hull with no compliant arrangement is reported infeasible. The check is listed in a `simulate` result only when it fails.
  - The 28 m tank-length cap of `design_grid` is removed. Every grid cell has two tanks along the hull, a cell is infeasible when its tank is longer than 7 diameters, and a cell's `tank_length_m` and `slenderness` are the engine's own figures (they were worked out from the hull length).
  - `.vdesign` `format_version` is 2 (ADR 0007). Version 1 files still open. A file that names `ADN_G` is read as `IGC_2G`, its `adn_tank_cap_m3` is dropped, and a gas Option with no `max_tank_od_m` gets the Defaults Profile value (16 m), marked `default` (version 1 used a fixed 12 m cap, so results may change). Each change is a warning, in `ovcd validate`, `ovcd fill`, `ovcd run`, the Design session and the app. Saving writes version 2.
  - Python, known changes from 0.1.0: removed names `MAX_TANK_OD_M` (now `barge_capacity.DEFAULT_MAX_TANK_OD_M`, 16.0), `MAX_TANK_LENGTH_M`, `n_tanks_for_length`, `ADN_G_MAX_L_OVER_D`, `ADN_G_TANK_CAP_9_3_4_M3`, `ADN_G_WIDE_TANK_FRACTION`, `ADN_LONGITUDINAL_GAP_M`, `ADN_G_BOTTOM_M`, `ADN_G_DOUBLE_BOTTOM_M`, `ADN_G_DOUBLE_HULL_M`, `ADN_G_INSPECTION_GAP_M`, `ADN_G_SADDLE_M`, `ADN_G_SIDE_M`, `ADN_G_TANK_CAP_M3`, and the field `DesignGrid.max_tank_length_m` (`max_tank_length_m`). `VesselType` has no `ADN_G`. `RULESETS` has two entries. `gas_arrangement.fit` has no `adn_tank_cap_m3`, and its `max_tank_od_m` defaults to 16.0. `BargeCapacityInputs` has a new last field `max_tank_od_m` (16.0), and `tanks_wide_for_beam` defaults `max_od_m` to 16.0. `criteria_set_for` gives `IS_CODE_2008` for every ruleset, and `assess` refuses `ADN_9_3_1_14_2`.
  - Tests: cases of 0.1.0 that depend on the tank cap now set `max_tank_od_m=12.0` (same numbers as before); `tests/data/api_ce96e66.json` was regenerated by `tests/data/make_api_ce96e66.py` from the frozen Python, and only the scenarios that used ADN or the tank-length cap changed. The `design_grid` scenario is a known difference.
- Fix: the release gate's wheel job skips the tests that read files it does not copy (the label guard, and two single tests that read `src/` and `CHANGELOG.md`), so the first `latest` build can publish (#171). No calculation result changes.
- Installers are built and published only on a version bump (a merge to `main` that changes the version publishes `v<version>`) or a pushed `test-<name>` tag (a test pre-release). Other merges publish nothing. The rolling `latest` build from #169 was reversed before it published (#173). No calculation result changes.
- Names importable in 0.1.0 are importable again (issue #143). Old modules re-exported names they imported, and scripts used them: `simulate.RULESETS`, `DEFAULT_DEPTHS_M`, `CARGO_DATABASE`, `gas_fit`, `capacity`, `BargeCapacityInputs`, `assess_stability`, `criteria_set_for`, `resistance` and the resistance input classes; `hydrostatics.free_surface_moment_cylindrical`; in `gas_arrangement` `ADN_G_MAX_L_OVER_D`, `ADN_G_WIDE_TANK_FRACTION`, `derate`, `capacity`, `JourneyDeratingInputs`, `all_ok` and the protective-location functions; in `barge_capacity` `HullGeometry`, `get_cargo`, `find_equilibrium_draft`, `calculate_tank_arrangement`, `calculate_weights`, `distance_d_vc`, `d_vc_exact`, `compute_required_clearances`, `compute_aft_arrangement` and `compute_gas_separation`; `design_grid.BargeCapacityInputs`, `electric_energy.R`, `ga_adapter.TankResult`, `journey_derating.HullGeometry`. Not kept (they came from the standard library or numpy, and the modules no longer need them): `math`, `np`, `replace`, `asdict`, `dataclass`, `field`, `Optional` and `Tuple` where a module no longer imports them.
- Known change from 0.1.0: the type hint of a result's list field (`checks`, `warnings`, `zones`, `gz`, ...) is `List[...]` where 0.1.0 said plain `list`. The bridge reads the item type to build the result objects. The values are lists as before (issue #143).
- Fix: `vessel_standards.validate_catalogue` checks any catalogue it is given again, as 0.1.0 did, including one without the built-in standards (it raised `NotImplementedError`).
- Fix: a capacity result no longer depends on the inputs object it came from, so editing or reusing that object after `barge_capacity.compute` no longer changes `journey_derating`, `gas_stability.assess` or `ga_adapter.is_gas` (issue #125). No calculation result changes.
- Fix: whole numbers (`90` for `90.0`) and numpy numbers are accepted again wherever a float is taken, on every public path (`HullGeometry` methods, `JourneyDeratingInputs` and the dataclass inputs). A whole float, a numpy integer or a `bool` is accepted for an int field such as `n_holds` (issue #127). No calculation result changes.
- Fix: functions use the object they are given again, as 0.1.0 did. `hydrostatics.calculate_stability` and `calculate_trim` use the hull, tanks and weights passed in; `weights.calculate_weights` uses the hull and tanks; `ga_adapter.to_ga`, `journey_derating.compute` and `gas_stability.assess` use the capacity result passed in. A result rebuilt with `dataclasses.replace` or built by hand works (issue #126). No calculation result changes.
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
- Fix: publishing the app installers now needs `test` and `release-guard` to pass on a tagged commit that is on `main`, and runs the release gate (wheels, binaries, pytest on each system) first. No calculation result changes.
- Tests: `tests/test_test_labels.py` requires a `Label: sourced|snapshot|property` line on every Python calculation test, and on new Rust test files that hold fixed numbers. The PR template asks whether a sourced test changed; build-process.md says how a snapshot is re-pinned. No result changes (#141).
- Docs: build notes go in a closing comment on the ticket and PR evidence must be reachable; squash merges use `--author-email`; CONTRIBUTING matches the real branch protection (#140, #136). No code or result changes.
- Tests: two new `gas_arrangement` tests kill two mutation survivors from PR #56 (values from the original Python at `ce96e66`); the other two are recorded as equivalent in the code. No calculation or result changes (#146).
- Docs: ADR 0007, spec 0001, the help text and the profile say a `.vdesign` file is SI except speeds, which are in knots (`max_speed_kn`, `channel_current_kn`). No format change; no calculation result changes (#134).
- Fix: the `ovcd run` summary keeps the case of the vessel label ("Gas barge, IMO Type C tanks"); a null `moulded_depth_m` gets the same "required for now; a default rule by Cargo is planned" text as a missing one; the roadmap names the package `vessel-designer`. No calculation result changes (#142).
- Fix: `ovcd fill` sets `engine_version` to this engine when it writes a results hash (and writes none beside a hash from another version). The hash warning now says "results differ from the hash saved in the file". A stored hash must be 64 hex digits, compared without regard to case; an empty `engine_version` is an error. No calculation result changes (#137).
- Change: `ovcd run` exits 0 for a valid file whose only gap is an Option that is not yet computable (a pushed convoy). The "convoy results not yet available" message stays in the summary and goes to stderr as a warning. In `--json` each Option has a `status`: `results`, `not_yet_supported` or `failed`. A hash stored on a convoy Option gets a note that it is not checked. Hard errors and failed computations still exit 1. No calculation result changes (#138).
- Fix: a `.vdesign` file with a duplicate JSON key (anywhere in the file) is now a hard validation error that names the key, instead of silently taking the last value. No calculation result changes (#139).

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

### Gas tanks: IGC rules only (#175) (2026-10-15)

Between `main e9854b5` and `this branch 405fc8a`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `checks[12].message` | side clearance 2.15 m >= 0.80 m inland type-G floor (cl. 3.5.2); IGC d(Vc) may require more | side clearance 1.58 m >= 0.80 m inland type-G floor (cl. 3.5.2); IGC d(Vc) may require more | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[14].message` | GM 3.17 m (fluid), max GZ 1.08 m at 25° | GM 2.36 m (fluid), max GZ 0.55 m at 16° — fails: Area 30–40° 0.024 < 0.03, Angle of max GZ 16.000 < 25.0 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[14].ok` | True | False | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[15].message` | GM 4.96 m (fluid), max GZ 2.09 m at 27° | GM 3.86 m (fluid), max GZ 1.65 m at 25° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[16].message` | GM 10.96 m (fluid), max GZ 3.22 m at 28° | GM 9.66 m (fluid), max GZ 3.05 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[8].message` | raised eye 13.20 m >= obstruction 14.74 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.20 m >= obstruction 15.88 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep CN-derived liquid 1 (88x16.3) | `results.governs` | tank volume | draught — Draught limit | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stability.conditions[0].criteria[2].ok` | True | False | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stability.conditions[0].criteria[4].ok` | True | False | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stability.ok` | True | False | changed |
| sweep CN-derived liquid 1 (88x16.3) | `warnings[0]` | Sightline needs 8.04 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 9.18 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].message` | side clearance 2.15 m >= 0.80 m inland type-G floor (cl. 3.5.2); IGC d(Vc) may require more | side clearance 2.22 m >= 0.80 m inland type-G floor (cl. 3.5.2); IGC d(Vc) may require more | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].message` | GM 2.41 m (fluid), max GZ 0.56 m at 16° — fails: Area 30–40° 0.029 < 0.03, Angle of max GZ 16.000 < 25.0 | GM 2.46 m (fluid), max GZ 0.58 m at 17° — fails: Angle of max GZ 17.000 < 25.0 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[15].message` | GM 3.91 m (fluid), max GZ 1.67 m at 25° | GM 3.95 m (fluid), max GZ 1.69 m at 26° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[16].message` | GM 10.02 m (fluid), max GZ 3.09 m at 29° | GM 10.18 m (fluid), max GZ 3.11 m at 29° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[8].message` | raised eye 13.20 m >= obstruction 14.74 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.20 m >= obstruction 14.60 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `stability.conditions[0].criteria[2].ok` | False | True | changed |
| sweep CN-derived liquid 2 (130x16.3) | `warnings[0]` | Sightline needs 8.04 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 7.90 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[12].message` | side clearance 2.60 m >= 0.80 m inland type-G floor (cl. 3.5.2); IGC d(Vc) may require more | side clearance 2.25 m >= 0.80 m inland type-G floor (cl. 3.5.2); IGC d(Vc) may require more | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[14].message` | GM 3.02 m (fluid), max GZ 0.68 m at 17° — fails: Angle of max GZ 17.000 < 25.0 | GM 2.75 m (fluid), max GZ 0.61 m at 16° — fails: Area 30–40° 0.023 < 0.03, Angle of max GZ 16.000 < 25.0 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[15].message` | GM 4.76 m (fluid), max GZ 1.91 m at 25° | GM 4.54 m (fluid), max GZ 1.82 m at 25° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[16].message` | GM 11.76 m (fluid), max GZ 3.46 m at 29° | GM 10.89 m (fluid), max GZ 3.35 m at 29° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[8].message` | raised eye 13.20 m >= obstruction 14.80 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.20 m >= obstruction 15.50 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep CN-derived liquid 3 (130x17.2) | `stability.conditions[0].criteria[2].ok` | True | False | changed |
| sweep CN-derived liquid 3 (130x17.2) | `warnings[0]` | Sightline needs 8.10 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 8.80 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep gas IGC_2G_EXACT 90x14 | `checks[13].message` | side clearance 1.00 m >= 0.80 m inland type-G floor (cl. 3.5.2); IGC d(Vc) may require more | side clearance 0.54 m >= 0.80 m inland type-G floor (cl. 3.5.2); IGC d(Vc) may require more | changed |
| sweep gas IGC_2G_EXACT 90x14 | `checks[13].ok` | True | False | changed |
| sweep gas IGC_2G_EXACT 90x14 | `checks[15].message` | GM 1.34 m (fluid), max GZ 0.32 m at 15° — fails: Area 30–40° -0.014 < 0.03, GZ at ≥ 30° 0.062 < 0.2, Angle of max GZ 15.000 < 25.0 | GM 0.91 m (fluid), max GZ 0.12 m at 8° — fails: Area 0–30° -0.036 < 0.055, Area 0–40° -0.148 < 0.09, Area 30–40° -0.113 < 0.03, GZ at ≥ 30° -0.467 < 0.2, Angle of max GZ 8.000 < 25.0 | changed |
| sweep gas IGC_2G_EXACT 90x14 | `checks[16].message` | GM 2.25 m (fluid), max GZ 1.12 m at 27° | GM 1.59 m (fluid), max GZ 0.73 m at 26° | changed |
| sweep gas IGC_2G_EXACT 90x14 | `checks[17].message` | GM 6.79 m (fluid), max GZ 2.31 m at 29° | GM 5.99 m (fluid), max GZ 2.17 m at 30° | changed |
| sweep gas IGC_2G_EXACT 90x14 | `checks[9].message` | raised eye 13.00 m >= obstruction 14.58 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.00 m >= obstruction 15.51 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep gas IGC_2G_EXACT 90x14 | `stability.conditions[0].criteria[0].ok` | True | False | changed |
| sweep gas IGC_2G_EXACT 90x14 | `stability.conditions[0].criteria[1].ok` | True | False | changed |
| sweep gas IGC_2G_EXACT 90x14 | `warnings[0]` | Sightline needs 8.08 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 9.01 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep gas cargo LNG | `checks[13].message` | side clearance 1.00 m >= 0.80 m inland type-G floor (cl. 3.5.2); IGC d(Vc) may require more | side clearance 0.54 m >= 0.80 m inland type-G floor (cl. 3.5.2); IGC d(Vc) may require more | changed |
| sweep gas cargo LNG | `checks[13].ok` | True | False | changed |
| sweep gas cargo LNG | `checks[15].message` | GM 1.74 m (fluid), max GZ 0.64 m at 24° — fails: Angle of max GZ 24.000 < 25.0 | GM 1.22 m (fluid), max GZ 0.32 m at 16° — fails: Area 30–40° -0.011 < 0.03, GZ at ≥ 30° 0.083 < 0.2, Angle of max GZ 16.000 < 25.0 | changed |
| sweep gas cargo LNG | `checks[16].message` | GM 2.84 m (fluid), max GZ 1.39 m at 28° | GM 2.13 m (fluid), max GZ 1.06 m at 27° | changed |
| sweep gas cargo LNG | `checks[17].message` | GM 6.79 m (fluid), max GZ 2.31 m at 29° | GM 5.99 m (fluid), max GZ 2.17 m at 30° | changed |
| sweep gas cargo LNG | `checks[9].message` | raised eye 13.00 m >= obstruction 14.58 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.00 m >= obstruction 15.51 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| ... | 5275 more changed quantities | | | |
