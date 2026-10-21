# Changelog

## Unreleased

- Rules engine (issue #336, the spec 0004 additions for liquid cargo Rules): the formula shapes `clamp` and `at-least` (the MARPOL Annex I Reg. 19 double-hull widths), the derived inputs `cargo_class` and `deadweight_band` that an Applicability can name, and one object type per liquid bulk tank role (`cargo-tank`, `ballast-tank`, `slop-tank`). The loader now refuses an Applicability that names an unknown input or value. No result moves.
- Type C tanks, summary (issues #313 to #317, from the maintainer's decisions on #292): the allowable stress of each tank steel is the IGC 4.23.3.1 limit, with cited strengths (S690QL1 285 to 256.7 MPa); the 2:1 elliptical head area is half an oblate spheroid (it was 21.5 % low); an LPG Cargo is loaded at its loading temperature, at the saturated density there, to the IGC 15.5.1 loading limit (fully pressurised propane 569 to 432 kg per m3 of tank); the dome default is the dome rule, not a fixed 2.0 m; two published tank weights per m3 are held-out calibration records (0.096 to 0.115 t per m3). A saved file keeps its results in every case. Two Rules wait for the rules engine: IGC 4.23.1.2 minimum design pressure and steel by design temperature (#317), and the IGC 4.28.1.2 dome volume check. New glossary terms: Allowable Stress Basis, Filling Limit, Loading Limit, Dome Rule. Each change has its own entry below.
- Rules engine (issue #269): every result has a Rule Trace, `rule_trace`, outside the results hash (ADR 0007). It lists each Rule of each selected Rule Set per object, with value, limit, Margin, outcome, verification status and the governing Rule of each Requirement; its verdict names any blocking Rule that is not evaluated. A Measure registry (`ovcd rules --measures`) and the loader now refuse an unknown Measure, wrong arguments, a limit in another unit and an unknown table or formula shape. `ovcd run --rule-set` evaluates the loaded sets. There are no built-in sets yet, so no result moves and no check changes.
- Design change (issue #316): the tank dome default is the dome rule, not a fixed 2.0 m. Dome diameter = the larger of 1.2 m and 0.17 x the steel outer diameter of each tank; dome height above the tank = the dome diameter in force. It is an estimate on two published points (a 2,500 mm dome, 2,500 mm high, on a 14.6 m lobe, Heo, Kang and Park 2017, Fig. 19; the 1,200 mm valve dome of a 4.73 m river LPG tank, Senjanovic et al., DESIGN 2006), shown as an `estimate` row in the Defaults Profile. `tank_dome_diameter_m` and `tank_dome_height_m` stay as optional overrides (none: the rule). The same dome sets the tank top and the air draft, counts in Vc (IGC 2.4.1.1), and is drawn at its diameter in the section and the elevation; `drawing.tank` shows the dome in force and where it came from. On a 110 x 11.4 m hull (tank about 9.6 to 10.3 m outside) the dome falls from 2.0 to 1.64 m and the loaded air draft by 0.36 m; on 90 x 14 m (12.2 to 12.9 m tanks) it rises to 2.08 m. Cargo moves by under 0.01 %. A saved file keeps its stored dome and its results; a file that lacks the inputs keeps 2.0 m. The IGC 4.28.1.2 dome volume check waits for the rules engine. Sourced test added: `crates/ovcd-core/tests/dome_rule.rs`. The before/after table is under "Design changes" below.
- Rules engine (issue #268): Rule Set files, their JSON Schema (`crates/ovcd-format/rules/rule-set.schema.json`) and a loader. `ovcd rules` lists the Rule Sets and `ovcd rules show <id>[@<version>]` prints one; `ovcd run`, `fill` and `validate` take `--rule-set <file>` (repeatable). A file is refused with a message that names the field when it is malformed. Nothing reads the sets yet, so no result moves. There are no built-in sets yet; an example set for the tests is in `crates/ovcd-format/rules/examples/`.
- Calibration (issue #317, part 1): two published empty weights of marine LNG Type C tanks are tank weight records in `calibration/tank_cases.yaml` (`tank_weight_records`), with their sources (`sources.yaml` P6 to P8): 155 t for a 1,350 m3 bunker barge tank (0.115 t per m3) and 240 t for a 2,500 m3 bilobe bunker vessel tank (0.096 t per m3). They are held out, and neither is run: one gives only a transport envelope with no datum, the other is a bilobe tank with no diameter. The report lists them with the published range, 0.096 to 0.115 t per m3, as the check on the tank structure factor before it is tuned. No result changes. The two Rules of the issue (IGC 4.23.1.2 minimum design pressure, steel by design temperature) wait for the rules engine (#268, #269).
- Design change (issue #315): an LPG Cargo is loaded at its loading temperature, at the saturated liquid density there (NIST Chemistry WebBook tables of propane and n-butane, `catalogue/lpg_saturation/`; the 60/40 mix as an ideal solution of the two, an estimate), to the loading limit of IGC 15.5.1, LL = FL x rhoR / rhoL. `fill_ratio` is the filling limit FL (98 %). The reference temperature follows IGC 15.1.3: a fully pressurised tank (no temperature control) takes the temperature at which the vapour pressure is the MARVS (15.1.3.1, the maintainer's choice); semi-pressurised and fully refrigerated tanks (with temperature control) take the loading temperature (15.1.3.2), so LL = FL. Two new LPG inputs: `lpg_loading_limit_basis` (`igc_15_1_3` default; `igc_15_5_2`, a 45 C reference for a pressurised tank, which IGC 15.5.2 lets an Administration allow; `refrigerated_density`, the old way) and `lpg_loading_temperature_c` (by carriage condition: 15 C pressurised and -10 C semi-pressurised, both estimates; the boiling point at one atmosphere when fully refrigerated). Cargo per m3 of tank: propane fully pressurised 569 to 432 kg (LL 85.1 %, reference 54.1 C), semi-pressurised 569 to 531 kg, fully refrigerated 569 kg (no change); butane fully pressurised 588 to 437 kg (LL 74.9 %, reference 110.7 C), semi-pressurised 588 to 599 kg, fully refrigerated 588 to 589 kg; mix fully pressurised 577 to 440 kg, semi-pressurised 556 kg, fully refrigerated 584 kg. On a 110 x 11.4 m hull, fully pressurised propane carries 2,166 t (was 2,316 t); the search now takes a wider tank because a lighter cargo frees displacement. LNG does not move. A file saved before keeps `refrigerated_density`, so its results do not change. Loading warmer than the MARVS allows is a soft warning. No sourced test changed; one new sourced test, `crates/ovcd-core/tests/lpg_saturation.rs` (the IGC worked case on the NIST rows). The before/after table is under "Design changes" below.
- Engine fix (issue #314): the surface area of a 2:1 semi-ellipsoidal tank head is the closed-form area of half an oblate spheroid, 4.336 R^2 = 1.084 D^2 (MathWorld, "Oblate Spheroid", eq. 11). The engine took 1.084 pi R^2, the 1.084 D^2 rule with pi R^2 in place of D^2, so the head steel and head insulation were 21.5 % low (the true area is 27.3 % above the old one: the 21 % and 27 % of the two reviews are the same error, measured from different bases). The steel area is at the steel outer surface and the insulation area at the tank outer diameter, as before. Hemispherical heads (2 pi R^2) were right and do not move; the engine has no torispherical head. No design of the fixed set moves: every Design uses hemispherical heads. The Python API and the tank calibration move: the river LPG tank (run with 2:1 heads) has 37.4 t of steel against 36.2 t (34.4 t published, inside its band). Snapshot re-pinned: `crates/ovcd-core/tests/tanks.rs::an_ellipsoidal_head_is_half_as_long_and_uses_the_oblate_spheroid_area` (shell 105.55 to 108.52 t, insulation 43.36 to 44.72 t). No sourced test changed; one new sourced test, `crates/ovcd-core/tests/head_area.rs`.
- Design change (issue #313): the allowable stress of each tank steel is the IGC 4.23.3.1 limit, the lesser of Rm/A and Re/B, with cited Rm and Re. S690QL1, the fully pressurised LPG steel, moves from 285 MPa (Rm/2.7, an inland basis) to 256.7 MPa (Rm/3). 304L, 316L (113 MPa) and 9Ni (230 MPa) do not move. Fully pressurised LPG walls get about 10 % thicker (29.0 to 32.0 mm on a 110 x 11.4 m hull, 40.5 to 45.0 mm on 90 x 14 m), tank steel and lightship rise (+52 t and +69 t), and cargo falls a little. A new input, `tank_allowable_stress_basis`, keeps 285 MPa for a file saved before, so its results do not change; a stress above the IGC limit is a soft warning. The river LPG tank calibration wall is now outside its band (15.5 mm against 13.1 mm printed), recorded as an accepted miss: the paper designs to the inland basis. Sourced tests changed: `tank_wall_formula.rs` (the paper's 285 MPa is now typed; new hand calculation at 256.7 MPa) and `test_the_published_river_tank_gives_the_published_wall_by_the_formula`. The before/after table is under "Design changes" below.
- App and web app (issue #300): dropdowns draw their own arrow and look the same on Linux as on macOS and Windows. The 30° and 40° labels on the stability chart have a ground, so a curve does not cross them. The bundled fonts are woff2 (0.95 MB, was 2.56 MB), with no change to the glyphs.

## 0.2.3 (2026-10-16)

Highlights, since 0.2.2:

- The web app. OVCD now also runs in the browser, at `/ovcd/app`, with no install. The engine is compiled to WebAssembly and runs fully in the page. Its results hashes equal the native ones.
- Gas tanks follow the IGC Code only. One rule set, `IGC_2G`, sets the side clearance per tank. The inland type-G checks are gone.
- A new calculations table lists each rule check with its working.
- A capital budget is worked out from 14 cost rates, which are now inputs.
- 38 more design values are inputs, for the dry-bulk hold, the hull ends, the gas tank arrangement, the waterway and the aft block (thrusters, battery and accommodation units).
- The search for the best gas arrangement now returns a design that meets every check, stability included.
- A gas-fuelled self-propelled stern, with gas gensets and the accommodation beside the wheelhouse.
- The general arrangement drawings show much more of the hull and tanks, and their labels and leaders are easier to read.
- The Views tabs fit to the panel, zoom and pan, and the side panels can be hidden.
- An evaluation harness tests the checks against a corpus, and the calibration now has held-out cases.
- The app has a new look: style guide, tokens, fonts, icon, splash, app bar and tabs, input and output panels, charts and the drawing canvas. It has light and dark themes and follows the system setting by default.

Notes:

- 17 aft block values are now inputs, in a new "Aft block" group (issue #4, fourth slice), for a self-propelled Formation only: `thruster_count` (3, the most the beam takes), `thruster_envelope_m` (2.6 m), `thruster_clearance_m` (0.6 m), `thruster_room_length_m` (3.5 m), `battery_unit_count` and `accommodation_unit_count` (default by Cargo phase: gas 1 and 2, dry bulk 2 and 1), `unit_height_m` (2.896 m, a high-cube 40 ft unit), `unit_side_gap_m` (0.9 m), `unit_row_gap_m` (0.8 m), `unit_foundation_m` (0.3 m), `wheelhouse_length_m` (3.6 m), `wheelhouse_width_m` (6.4 m), `wheelhouse_height_m` (3.4 m), `wheelhouse_gap_m` (0.3 m), `wheelhouse_eye_height_m` (1.5 m), `sightline_margin_m` (0.5 m) and `wheelhouse_max_lift_m` (6 m). Each default is the value the engine used before. A self-propelled file that lacks them opens with the same values, marked `default`, with one warning; a pushed file has none of them. The gas search carries the values to every candidate. The ISO container length and width, the genset data and the gas-fuelled stern allowances stay in the engine (propulsion is not yet an input). Not a design change: no calculation result moves. The evaluation entry point `fit` accepts the new values under the core's names.
- Design change (issue #264): `fit` returns the arrangement with the most cargo among those that meet every check, stability included (IS Code 2008 Part A 2.2, each loading condition). Before, it ranked by cargo and ran stability afterwards, so it could return a design that failed it. If no arrangement passes, `feasible` is false, the closest candidate (fewest failed checks, then most cargo) comes back, and `failed_checks` lists what it fails. The before/after table is under "Design changes" below.
  - The check list of the search is one place, `gas_arrangement.rs` ("THE CHECKS THE SEARCH USES"; the rules engine, #176, will replace it): the capacity, journey and gas-separation checks the engine already ran, tank length over diameter, the air draft limit (new `FitOptions::air_draft_limit_m`, passed from `air_draft_limit_m`), freeboard at the carried draught (new `required_freeboard_m`, passed from `freeboard_min_m`), and stability. Cheap checks run first; stability runs only for the candidates that pass them, from the most cargo down. A check of warning severity (the wheelhouse sightline, for one) is advisory and does not gate, as before.
  - `failed_checks` lists failed error checks only (it also listed failed warnings). `smallest_viable_loa` asks for a length with a passing arrangement and scans up from `lo` in 2 m steps before halving to 0.1 m, because viability is not monotone in length. The design sweep (`design_grid`) does not search arrangements (one fixed arrangement per cell) and is unchanged. `ovcd run`: the "Feasible arrangement" check now says no arrangement passes and names the failed checks of the closest one.
  - Tests re-pinned to what the code prints now (no tolerance loosened; old values in the history). Rust: `the_cn_ii_benchmark_is_reproduced` (2,832.6643735601965 t, binding "volume"), `the_smallest_viable_loa_is_found_to_a_tenth_of_a_metre` (39.9 m), `a_hull_too_short_for_a_tank_is_reported_infeasible_not_raised` (2 failed checks), `json_results_are_what_simulate_answers_for_a_self_propelled_gas_hull` (cargo 2,832.6643735601965 t, lightship 1,893.6869533634328 t, governs "tank volume"), the stored results hashes of the two gas fixtures. Python: `test_cn_ii_benchmark_is_reproduced`, the two regression snapshots `(90, 12.9)` (12 m cap) and `(90, 14.8)` (16 m cap), the quickstart example (README). Tests changed in meaning, not re-pinned: `the_default_cap_lets_a_tank_pass_the_old_12_m`, `max_tank_od_m_reaches_the_arrangement_search`, the failed-checks note test (now a hull where nothing passes), the section drawing test (also draws a 6 m deep hull), `test_tender_gas_barge_shows_failed_stability_checks` and the two stability tests that need a failing design (now a 60 x 10 x 6 m hull). No sourced test changed.
- Design change (issue #237): one gas rule set, `IGC_2G`, the per-tank distance d(Vc) of IGC Code 2.4.1.1. The stepped reading (d from the total tank volume: 0.8 m up to 1,000 m3, 1.0 m up to 5,000 m3, 2.0 m above) is removed: it is not in the IGC or IGF text. The text was checked against the Code as adopted by MSC.370(93) and every amendment in force; none changes 2.4. The before/after table is under "Design changes" below.
  - The rule. d is read per individual tank: up to 1,000 m3 0.8 m; 1,000 to 5,000 m3 0.75 + Vc x 0.2 / 4,000 m; 5,000 to 30,000 m3 0.8 + Vc / 25,000 m; 30,000 m3 and over 2 m. The datum is 2.4.2: d runs "plate to plate, from the moulded line to the moulded line, excluding insulation", so the clear gap from the outer face of the insulation is d less the insulation thickness (as `IGC_2G_EXACT` already did). Every gas search now takes the per-tank loop.
  - The dome counts in Vc (2.4.1.1: "100 % of the gross design volume of the individual cargo tank at 20 C, including domes and appendages"). New input `tank_dome_diameter_m` in the Gas tanks group, 2.0 m by default: a nominal cylindrical dome as wide as it is high; there is no published source for the size yet, and the maintainer is to confirm it. The dome volume, pi/4 x diameter^2 x dome height per tank, goes into Vc only, not into the cargo volume, the fill ratio, the weights or `containment_inner_volume_m3`. Appendages (nozzles, manways and the like) are not modelled: no volume is added for them. At 2 m by 2 m the dome adds 6.3 m3 per tank, which moves d by 0.3 mm.
  - Names. `IGC_2G_EXACT` is no longer offered; the Defaults Profile ruleset default is `IGC_2G` (fill ratio 0.98). A file of any format version that names `IGC_2G_EXACT` opens as `IGC_2G` with the warning "IGC_2G_EXACT is now IGC_2G: the same per-tank rule of IGC 2.4.1.1". A version 1 file that names `IGC_2G` meant the stepped reading: it opens as `IGC_2G` with a warning that the per-tank rule applies now and results may change. (A version 2 file gets no such warning: version 2 is unreleased and this build wrote it.) The version 1 `ADN_G` mapping to `IGC_2G` still works. A file that lacks `tank_dome_diameter_m` gets 2.0 m, marked `default`, with a warning that results may change, because the engine had no dome in Vc before. ADR 0007 is updated.
  - Python, known changes from 0.1.0: removed names `distance_d_vc` (`core.protective_location`, and the copies `core.barge_capacity` and `core.gas_arrangement` imported) and the enum member `VesselType.IGC_2G_EXACT`; `RULESETS` is `("IGC_2G",)`. `check_protective_location` reads the volume of one tank (its argument keeps the name `total_tank_volume`). `required_side_clearance` ignores `total_m3` and takes the per-tank volume. `BargeCapacityInputs` has the new field `dome_diameter` (2.0 m). The scenarios of `tests/data/api_ce96e66.json` that hold a gas stability result, the side clearance rule or the protective location are listed as known differences in `tests/test_api_oracle.py`.
  - Stability. A design that carries more because it has less side clearance can be less stable. In the fixed design set, `sweep gas passes all checks IGC_2G 110x11.4` and `sweep gas tank longer than 7 diameters IGC_2G 200x12` now fail the laden stability check. The CN II hull (90 x 14.8 m, propane, 4.4 m draught) with the default 16 m tank cap carries 2,943 t in one 13.7 m tank and fails the laden angle of maximum GZ (21 degrees, limit 25); the search does not gate on stability. The arrangement search itself is unchanged.
  - Tests re-pinned to what the engine prints now (never by hand, no tolerance loosened; the old values are in the history). Rust: `the_cn_ii_benchmark_is_reproduced` (2,943.4407266060916 t, binding "draft"), `the_smallest_viable_loa_is_found_to_a_tenth_of_a_metre` (36.1 m), `the_quickstart_propane_barge_matches_python`, `json_results_are_what_simulate_answers_for_a_self_propelled_gas_hull`, the stored results hashes of the two gas fixtures. Python: the CN II and 16 m cap snapshots, the gas stability values of `tests/test_api_passed_objects.py` and `tests/test_api_reused_inputs.py`, `tests/test_api_ints_numpy.py`, `tests/test_simulate.py`, `tests/test_api_small_changes.py` and the quickstart example (README). The gas fixture `gas_igc_2g_exact.vdesign` keeps its name and now names `IGC_2G`. New tests: the d(Vc) boundaries at 1,000, 5,000 and 30,000 m3 and either side (sourced, IGC Code 2.4.1.1), the 2.4.2 datum, the dome in Vc, and the opening of files that name the old rule sets.
- Design change (issue #215): the two inland "type G" checks are removed. Gas tanks are sized under IGC only (#175), and these floors came from the inland ADN rules. The before/after table is under "Design changes" below. ADN may return later as a selectable rule set, not as fixed checks (#176).
  - Removed checks: `side_clearance_type_g_floor` (side clearance at least 0.80 m) and `double_bottom_type_g_floor` (double bottom at least 0.60 m). A gas result lists two checks fewer. A design that failed only these checks now passes its feasibility, and the arrangement search can pick an arrangement it used to reject (for example 120 x 18 m, LNG, `IGC_2G_EXACT`: two 8 m tanks abreast, 2,922 t, where it picked one 11 m tank, 2,813 t). The IGC side clearance d(Vc) is not changed here (issue #237).
  - Python, known changes from 0.1.0: removed names `SIDE_MIN_TYPE_G_M` and `DOUBLE_BOTTOM_MIN_TYPE_G_M` (`core.gas_separation`). The inputs `side_clearance_m` and `double_bottom_m` of `GasSeparationInputs` stay, so older code still runs, but no check reads them. The scenarios of `tests/data/api_ce96e66.json` that hold a gas check list or fit are listed as known differences in `tests/test_api_oracle.py`.
  - The second run of the gas separation in `barge_capacity.compute` is removed. It existed only to pass the as-designed clearances to the type-G checks; the separation lengths, checks and warnings do not depend on them, so no other output changes.
  - Tests: the stored results hashes of the two gas fixtures (`gas_igc_2g_exact`, `gas_igc_2g_self_propelled`) are re-pinned to the values the engine prints now, and 120 x 18 m LNG (`IGC_2G_EXACT`) is pinned at 2,922 t (two 8 m tanks) again, the value of 0.1.0. The failed-check note test of the self-propelled LNG case now needs 3 failed checks, not 6 (two of the six were the removed floors).
- The 14 cost rates of the capital budget are now inputs, in a new "Cost rates" group, and each Option has a capital budget (issue #233). The inputs are `hull_steel_rate`, `tank_shell_rate`, `tank_supports_rate`, `insulation_rate`, `piping_rate`, `outfitting_rate`, `cargo_system_lump`, `paint_anodes_misc`, `battery_rate_per_kwh`, `propulsion_rate_per_kw`, `wheelhouse_accom_lump`, `tug_outfitting_lump`, `yard_margin_fraction` and `delivery_lump`. Each default is the value in `examples/barge_cost_rates.example.yaml` (illustrative rates, not market quotes: the group says "Example rates — replace with yours"; a test keeps the two equal). The currency stays that of the example file. `ovcd run` runs the core's `barge_cost` for each gas Option and puts the budget in a new `cost` results key, with the masses it used in `cost_masses` (a gas barge takes its masses from its weight breakdown; the tug masses keep the core's defaults). The core models a gas barge, so a Cargo that is not gas has `cost` and `cost_masses` null, and the app says the budget is work in development. Both keys are left out of the results hash, as `drawing` and `stability_marks` are. The rates do not reach `simulate`. A file that lacks the rates opens with the example values, marked `default`, with a warning. Not a design change: no calculation result moves.
- Two environment values are now inputs, in the Waterway Limits group (issue #4, sixth slice): `water_density_t_per_m3` (1.0 t/m3, fresh water; sea water is about 1.025) and `squat_safety_margin_m` (0.15 m). The water density was held in four places (the `simulate` constant, the capacity inputs, the stability default and the journey de-rating); on the `ovcd run` path all four, and the resistance curves, now read the one input, so a changed value moves the draught, the stability and the speed curves together. A file that lacks them opens with the old values, marked `default`, with a warning. Not a design change: no calculation result moves. The utilisation floor (advisory, 0.5) is not an input yet: it only sets a warning check that no `ovcd run` result shows.
- Six gas tank arrangement values are now inputs, in the Gas tanks group (issue #4, third slice): `tanks_in_line` (2), `tank_support_height_m` (0.5 m), `tank_dome_height_m` (2.0 m), `wing_tank_gap_m` (0.6 m), `wing_tank_height_ratio` (0.66) and `tank_wall_thickness_mm` (25 mm). Each default is the value the engine used before. A gas file that lacks them opens with the same values, marked `default`, with a warning. A different `tanks_in_line` changes the tanks along the hull for every candidate: fewer tanks are longer and may break the length over diameter limit of 7, more tanks are shorter and may leave no room for their heads (the hull is then infeasible). The drawing follows each input through the results. Not a design change: no calculation result moves. The evaluation entry points `barge_capacity` and `gas_arrangement` accept the new fields (and `bow_rake_to_depth_ratio` from the second slice).
- Three hull end values are now inputs (issue #4, second slice): `bow_rake_length_to_depth_ratio` (1.6, the bow rake as a multiple of the moulded depth, every hull), `pushed_stern_void_m` (3.0 m, the aft void of a pushed barge or convoy) and `bulk_end_bulkhead_m` (0.5 m, the end bulkhead allowance at each end of a dry-bulk hold). Each default is the value the engine used before. An input can now apply to some Formations only: `ovcd schema` gives `applies_to_formation` for every input, and the app shows the stern void for a pushed Formation only. A file that lacks them opens with the same values, marked `default`, with a warning. Also: in the app and the Design session, an input that a reset, a new Option or a changed Cargo Containment or Formation brings back takes the Defaults Profile value (slice 1 gave it the fixed value of an old file, so an edited profile entry for a dry-bulk hold input was ignored there). Not a design change: no calculation result moves.
- Ten dry-bulk hold values are now inputs, in a new "Dry-bulk hold" group (issue #4, first slice): `hold_wing_void_width_m` (1.0 m), `hold_double_bottom_m` (1.2 m), `hatch_coaming_height_m` (1.0 m) and `hold_fill_ratio` (0.95) for box and sloped hopper holds, and `hopper_slope_h_per_v` (1.0), `n_holds` (3) and the four plate thicknesses `inner_bottom_plate_mm` (14), `hopper_plate_mm` (14), `hold_bulkhead_plate_mm` (11) and `hatch_coaming_plate_mm` (10) for the sloped hopper hold. Each default is the value the engine used before, with a basis in the Defaults Profile, `ovcd schema` and the app. A file that lacks them (version 1, or a version 2 file from a test build) opens with those same values marked `default` and a warning, and an older Defaults Profile copy takes the built-in entries. Not a design change: no calculation result moves.
- Fix: `vessel_designer.calibration` imports from an installed wheel (issue #217). Importing it reads no file: the bands are read from `CAL` when first used, and the new environment variable `OVCD_CALIBRATION_DIR` sets `CAL` (default: the checkout's `calibration/`, as before). The release gate's wheel check runs the held-out calibration tests and the public API snapshot test again. No public name changes and no calculation result changes.
- Default moulded depth (issue #120). Moulded depth is keel to main deck at the side. It is no longer a required input: left out, it is the beam times a depth-to-beam ratio by Cargo, a rule in the Defaults Profile (`depth_to_beam_ratio`: dry bulk and container 0.33, liquid 0.30, gas 0.55), rounded to the millimetre, stored as used and marked `default`. A class-defined Option (issue #94) picks it up like any other. A depth on its default follows a change of beam or Cargo; a typed depth is never changed.
  - Which Cargo is in which group: a dry-bulk Cargo (`bauxite`) is dry bulk, and every liquefied-gas Cargo (`LNG`, `LNG_ethane`, `LPG_propane`, `LPG_butane`, `LPG_mix`) is gas. Container shares the dry-bulk ratio. The liquid group has a ratio in the profile but no Cargo in it yet.
  - The rule and its basis are in `ovcd schema` (`moulded_depth_m.default_rule`) and on hover over Moulded depth in the app. The profile's basis text "the catalogue holds no depths" is replaced. `catalogue/depth_rule.md` is generated by `tools/gen_depth_rule.py` (`--check`, tested in the fast gate) and lists every Reference Vessel with a published depth with the rule's value beside it.
  - New soft warning: length over depth above about 25 (`length_to_depth_ratio` in the profile). Note that a default depth gives length over depth above 25 for a hull whose length is more than 8.25 times its beam.
  - Moulded depth moves from the Required group to Hull and containment. `ovcd validate` of a file that leaves it out now says it is not stored (like any default) instead of "required for now"; a null depth is "must be a number". `ovcd new` still writes an example typed depth, and `ovcd new --class` leaves it to the rule. No calculation result changes for a design that types its depth; `.vdesign` stays at version 2.
- Vessel Class (issue #94): an Option is defined by typed dimensions or by a standard Vessel Class.
  - New inputs `vessel_scheme` (the Class Scheme) and `vessel_class`, held only by an Option defined by a class. **Length and beam decide the class**: the vessel sits in the smallest class of the scheme (and Formation kind) whose length and beam maxima hold it, moves up or down as they change, and past the largest class of the scheme there is a warning, "beyond the largest <scheme> class". The scheme never changes by itself. Motor and convoy classes are separate ladders; the `CEMT` `-W` and `-E` classes share one.
  - Picking a class sets length and beam to its maxima and the Formation (a convoy class also its abreast and in-line counts), and fills the draught limit (top of the class draught range) and the air-draft limit (bottom of its air-draft range, the lowest bridge clearance), unless a limit was typed; a typed limit above the class's is a warning. A class that gives no value fills nothing. Moulded depth stays typed. These inputs carry the new source marker `class` in the file and the left panel. The class's reference deadweight is shown beside the computed cargo and is never an input. The `.vdesign` format stays at version 2 (ADR 0007); version 1 files open unchanged. No calculation result changes for a given set of dimensions.
  - The same in the app (left panel: "Typed dimensions" or "Standard class"), `ovcd new --class <id>`, `ovcd fill` (a class named in a hand-written file gives the dimensions it leaves out), the Design session and `ovcd schema`.
  - One catalogue: `catalogue/vessel_classes.yaml` is the only source of the classes. The core's list is generated from it by `tools/gen_vessel_classes.py` (`--check`; a test in the fast gate fails on drift), and the Python `VESSEL_STANDARDS` is read from the core. Not every class can define a vessel: one with no length or beam, or a towed barge, ATB, pusher, tug or towboat (no Formation kind), is not offered. Cargo does not filter the list; `cargo_modes` stay catalogue data.
  - Python `vessel_standards`, known changes from 0.1.0: the list is the catalogue's 292 classes, with the catalogue's ids (`CEMT-Va`, `CN-II-MOTOR`). The 12 old ids (`CEMT_Va`, `CN_II_motor`, `CEMT_VIa_motor_NL2011`, ...) still work in `get_standard` (the NL2011 class is `RWS-M12`) but are not keys of `VESSEL_STANDARDS`. `standards_for` returns every class for either cargo mode (it still rejects other modes), because Cargo is not a class property. The catalogue's figures replace the hand-typed ones: CEMT Va draught is 2.5 to 2.8 m (was 2.5 to 4.5), the NL2011 class is 135 m and 5601 t (was 110 to 135 m and 6000 t) with no bridge clearance (the 7.0 m had no source), `CN_II_motor` has no moulded depth (the 6.5 m had no source), and `family`, `class_name`, `source`, `notes` and `cargo_modes` are the catalogue's. `loa_m`, `beam_m`, `draught_m` and `dwt_t` can be `None` when a class gives no figure, `beam_m` is the largest beam, `min_bridge_clearance_m` is the bottom of the air-draft range, and `VesselStandard` has four new fields (`air_draft_m`, `region`, `waters`, `confidence`). The `vessel_standards` oracle scenario is a known difference.
- Calibration Bands (issue #119): `calibration/bands.yaml` gives each figure a band per confidence (lightship ±15 / 20 / 30 %, deadweight ±7.5 / 10 / 15 % at high / medium / low). `calibration/results.md` shows each calibrated result against its band and marks cases outside it, and says the errors are in-sample. `python -m vessel_designer.calibration --check` prints a `::warning` annotation per case outside its band; the `test` CI job runs it (about a second). A band miss is a warning only and never fails the build; a crash fails. No calculation result changes.
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
- Web app (issues #245 and #266): `ovcd-web` is the Design session compiled to WebAssembly, with a platform adapter and a web build of the app. The release gate checks that the fixture results hashes match on WebAssembly under wasmtime. No calculation result changes.
- App restyle, so far (issues #248, #250, #251, #255, #257): a style guide, tokens, bundled fonts, a new icon, splash and first-run notice, a new app bar, panel toggles, theme control and two-level tabs, and new input fields with source tags. The theme setting `Auto` shows light until the restyle is complete. No calculation result changes.
- Views (issues #234, #235): tabs, fit to the panel with zoom and pan, and hide the side panels; a click on a point of the power and speed curves reads its values. Intact stability shows GZ curves with marks from the engine (#236), and a rule check calculations table was added with two Intact stability fixes (#232).
- GA views (issues #192 to #198, #213, #214): the drawing is an object in `simulate` results; tank, double bottom, wing tanks, voids, cradles, domes, cofferdam, zone limits, the dry-bulk hold and the aft block are drawn; a failed-checks note sits in the strip; leaders in a column do not cross and labels keep clear of each other and the frame.
- Evaluation harness (issue #184): a box-condition check, an oracle, a corpus format, a mutant set, a runner and a CI gate. The corpus includes the Navy LCU, empty and loaded, from published inputs.
- Calibration (issue #3): 126 cited held-out cases (62 and 64), with an out-of-sample section in `calibration/results.md`.

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

### ticket/316-dome-rule (2026-10-20)

Between `main 2800268` and `this branch 56ae7a8`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `checks[8].message` | raised eye 13.20 m >= obstruction 14.52 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.20 m >= obstruction 14.47 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep CN-derived liquid 1 (88x16.3) | `warnings[0]` | Sightline needs 7.82 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 7.77 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[8].message` | raised eye 13.20 m >= obstruction 13.25 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.20 m >= obstruction 12.97 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `warnings[0]` | Sightline needs 6.55 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.27 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[8].message` | raised eye 13.20 m >= obstruction 14.14 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.20 m >= obstruction 14.01 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep CN-derived liquid 3 (130x17.2) | `warnings[0]` | Sightline needs 7.44 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 7.31 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep default depth gas 90x14 | `checks[9].message` | raised eye 15.20 m >= obstruction 15.51 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 15.20 m >= obstruction 15.59 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep default depth gas 90x14 | `warnings[0]` | Sightline needs 6.81 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.89 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep gas IGC_2G 60x10 | `checks[9].message` | raised eye 7.00 m >= obstruction 6.50 m + 0.50 m margin (lift used 0.00 m of 6.00 m) | raised eye 7.00 m >= obstruction 5.70 m + 0.50 m margin (lift used 0.00 m of 6.00 m) | changed |
| sweep gas IGC_2G 90x14 | `checks[8].message` | raised eye 13.00 m >= obstruction 13.15 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.00 m >= obstruction 12.89 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep gas IGC_2G 90x14 | `warnings[0]` | Sightline needs 6.65 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.39 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep gas IGC_2G_EXACT 60x10 | `checks[9].message` | raised eye 7.00 m >= obstruction 6.50 m + 0.50 m margin (lift used 0.00 m of 6.00 m) | raised eye 7.00 m >= obstruction 5.70 m + 0.50 m margin (lift used 0.00 m of 6.00 m) | changed |
| sweep gas IGC_2G_EXACT 90x14 | `checks[8].message` | raised eye 13.00 m >= obstruction 13.15 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.00 m >= obstruction 12.89 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep gas IGC_2G_EXACT 90x14 | `warnings[0]` | Sightline needs 6.65 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.39 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep gas cargo LNG | `checks[8].message` | raised eye 13.00 m >= obstruction 14.49 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.00 m >= obstruction 14.39 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep gas cargo LNG | `warnings[0]` | Sightline needs 7.99 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 7.89 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep gas cargo LNG_ethane | `checks[8].message` | raised eye 13.00 m >= obstruction 14.49 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.00 m >= obstruction 14.39 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep gas cargo LNG_ethane | `warnings[0]` | Sightline needs 7.99 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 7.89 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep gas cargo LPG_butane | `checks[8].message` | raised eye 13.00 m >= obstruction 13.15 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.00 m >= obstruction 12.89 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep gas cargo LPG_butane | `warnings[0]` | Sightline needs 6.65 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.39 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep gas cargo LPG_mix | `checks[8].message` | raised eye 13.00 m >= obstruction 13.15 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.00 m >= obstruction 12.89 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep gas cargo LPG_mix | `warnings[0]` | Sightline needs 6.65 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.39 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep gas cargo LPG_propane | `checks[8].message` | raised eye 13.00 m >= obstruction 13.15 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.00 m >= obstruction 12.89 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep gas cargo LPG_propane | `warnings[0]` | Sightline needs 6.65 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.39 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep gas carriage LPG_butane fully_pressurised | `checks[8].message` | raised eye 13.02 m >= obstruction 12.52 m + 0.50 m margin (lift used 5.52 m of 6.00 m) | raised eye 12.66 m >= obstruction 12.16 m + 0.50 m margin (lift used 5.16 m of 6.00 m) | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[8].message` | raised eye 13.22 m >= obstruction 12.72 m + 0.50 m margin (lift used 5.72 m of 6.00 m) | raised eye 12.86 m >= obstruction 12.36 m + 0.50 m margin (lift used 5.36 m of 6.00 m) | changed |
| sweep gas carriage LPG_butane semi_pressurised | `checks[14].message` | GM 5.51 m (fluid), max GZ 1.71 m at 25° | GM 5.50 m (fluid), max GZ 1.71 m at 25° | changed |
| sweep gas carriage LPG_butane semi_pressurised | `checks[8].message` | raised eye 12.20 m >= obstruction 11.70 m + 0.50 m margin (lift used 4.70 m of 6.00 m) | raised eye 11.67 m >= obstruction 11.17 m + 0.50 m margin (lift used 4.17 m of 6.00 m) | changed |
| sweep gas carriage LPG_mix fully_pressurised | `checks[8].message` | raised eye 13.02 m >= obstruction 12.52 m + 0.50 m margin (lift used 5.52 m of 6.00 m) | raised eye 12.66 m >= obstruction 12.16 m + 0.50 m margin (lift used 5.16 m of 6.00 m) | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[8].message` | raised eye 13.22 m >= obstruction 12.72 m + 0.50 m margin (lift used 5.72 m of 6.00 m) | raised eye 12.86 m >= obstruction 12.36 m + 0.50 m margin (lift used 5.36 m of 6.00 m) | changed |
| sweep gas carriage LPG_mix semi_pressurised | `checks[14].message` | GM 5.51 m (fluid), max GZ 1.71 m at 25° | GM 5.50 m (fluid), max GZ 1.71 m at 25° | changed |
| sweep gas carriage LPG_mix semi_pressurised | `checks[8].message` | raised eye 12.20 m >= obstruction 11.70 m + 0.50 m margin (lift used 4.70 m of 6.00 m) | raised eye 11.67 m >= obstruction 11.17 m + 0.50 m margin (lift used 4.17 m of 6.00 m) | changed |
| sweep gas carriage LPG_propane fully_pressurised | `checks[8].message` | raised eye 13.02 m >= obstruction 12.52 m + 0.50 m margin (lift used 5.52 m of 6.00 m) | raised eye 12.66 m >= obstruction 12.16 m + 0.50 m margin (lift used 5.16 m of 6.00 m) | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[8].message` | raised eye 13.22 m >= obstruction 12.72 m + 0.50 m margin (lift used 5.72 m of 6.00 m) | raised eye 12.86 m >= obstruction 12.36 m + 0.50 m margin (lift used 5.36 m of 6.00 m) | changed |
| sweep gas carriage LPG_propane semi_pressurised | `checks[8].message` | raised eye 13.23 m >= obstruction 12.73 m + 0.50 m margin (lift used 5.73 m of 6.00 m) | raised eye 12.87 m >= obstruction 12.37 m + 0.50 m margin (lift used 5.37 m of 6.00 m) | changed |
| sweep gas carries nothing (draught limit) | `checks[9].message` | raised eye 13.00 m >= obstruction 15.15 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.00 m >= obstruction 15.23 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep gas carries nothing (draught limit) | `warnings[0]` | Sightline needs 8.65 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 8.73 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep gas passes all checks IGC_2G 110x11.4 | `checks[8].message` | raised eye 13.02 m >= obstruction 12.52 m + 0.50 m margin (lift used 5.52 m of 6.00 m) | raised eye 12.66 m >= obstruction 12.16 m + 0.50 m margin (lift used 5.16 m of 6.00 m) | changed |
| sweep gas passes all checks IGC_2G_EXACT 80x14 | `checks[8].message` | raised eye 12.00 m >= obstruction 12.20 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 12.00 m >= obstruction 11.77 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep gas passes all checks IGC_2G_EXACT 80x14 | `warnings[0]` | Sightline needs 6.70 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.27 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| ... | 22853 more changed quantities | | | |

### ticket/315-lpg-density-loading-limit (2026-10-20)

Between `main 32f0184` and `this branch e585a6b`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `checks[12].message` | GM 3.85 m (fluid), max GZ 1.55 m at 26° | GM 3.62 m (fluid), max GZ 1.46 m at 26° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[13].message` | GM 5.88 m (fluid), max GZ 2.39 m at 27° | GM 5.53 m (fluid), max GZ 2.27 m at 27° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[14].message` | GM 12.13 m (fluid), max GZ 3.35 m at 27° | GM 10.49 m (fluid), max GZ 3.15 m at 28° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[8].message` | raised eye 13.20 m >= obstruction 13.52 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.20 m >= obstruction 14.52 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep CN-derived liquid 1 (88x16.3) | `warnings[0]` | Sightline needs 6.82 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 7.82 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep CN-derived liquid 2 (130x16.3) | `arrangement.feasible` | False | True | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[10].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft separation 2.00 m >= 2.00 m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[10].name` | cofferdam min width | opening setback | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[11].message` | aft separation 2.00 m >= 2.00 m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11) | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[11].name` | opening setback | fwd end bulkhead space | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | GM 3.82 m (fluid), max GZ 1.56 m at 26° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].name` | fwd end bulkhead space | Stability — laden | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].severity` | error | absent | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[13].message` | GM 2.40 m (fluid), max GZ 0.56 m at 16° — fails: Area 30–40° 0.028 < 0.03, Angle of max GZ 16.000 < 25.0 | GM 5.77 m (fluid), max GZ 2.34 m at 27° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[13].name` | Stability — laden | Stability — half laden | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[13].ok` | False | True | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].message` | GM 3.83 m (fluid), max GZ 1.63 m at 25° | GM 11.17 m (fluid), max GZ 3.23 m at 28° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].name` | Stability — half laden | Stability — light | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[15].message` | GM 9.45 m (fluid), max GZ 3.00 m at 29° | absent | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[15].name` | Stability — light | absent | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[15].ok` | True | absent | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[1].message` | No arrangement passes every check on this hull. The closest one is shown. It fails: Stability — laden. | cargo volume must be positive (tank/hold fits) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[1].name` | Feasible arrangement | feasible geometry | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[1].ok` | False | True | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[1].severity` | absent | error | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[2].message` | cargo volume must be positive (tank/hold fits) | cargo zone 98.49 m = LOA 130.0 - bow void 9.62 m - stern void 21.89 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[2].name` | feasible geometry | cargo zone positive | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[3].message` | cargo zone 98.49 m = LOA 130.0 - bow void 9.62 m - stern void 21.89 m | 3 x 2.60 m thrusters need 10.20 m across a 16.30 m beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[3].name` | cargo zone positive | thrusters fit beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[4].message` | 3 x 2.60 m thrusters need 10.20 m across a 16.30 m beam | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 16.30 m beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[4].name` | thrusters fit beam | requested thruster count fits | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[4].severity` | error | warning | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[5].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 16.30 m beam | 3 x 40 ft units abreast need 10.91 m across a 16.30 m beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[5].name` | requested thruster count fits | containers fit beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[5].severity` | warning | error | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[6].message` | 3 x 40 ft units abreast need 10.91 m across a 16.30 m beam | 3 units in 1 row(s) of 3; a second row adds 12.99 m to the stern block and comes straight off the cargo zone | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[6].name` | containers fit beam | units fit in one row | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[6].severity` | error | warning | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[7].message` | 3 units in 1 row(s) of 3; a second row adds 12.99 m to the stern block and comes straight off the cargo zone | wheelhouse 6.40 m wide on a 16.30 m beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[7].name` | units fit in one row | wheelhouse fits beam | changed |
| ... | 20729 more changed quantities | | | |

### ticket/313-s690-allowable-stress (2026-10-20)

Between `main f6d446c` and `this branch 1e49b1a`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `checks[12].message` | GM 3.93 m (fluid), max GZ 1.60 m at 26° | GM 3.85 m (fluid), max GZ 1.55 m at 26° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[13].message` | GM 6.04 m (fluid), max GZ 2.43 m at 27° | GM 5.88 m (fluid), max GZ 2.39 m at 27° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[14].message` | GM 12.62 m (fluid), max GZ 3.41 m at 27° | GM 12.13 m (fluid), max GZ 3.35 m at 27° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `arrangement.feasible` | True | False | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[10].message` | aft separation 2.00 m >= 2.00 m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11) | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[10].name` | opening setback | cofferdam min width | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | aft separation 2.00 m >= 2.00 m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[11].name` | fwd end bulkhead space | opening setback | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].message` | GM 3.20 m (fluid), max GZ 1.05 m at 25° | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].name` | Stability — laden | fwd end bulkhead space | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].severity` | absent | error | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[13].message` | GM 4.97 m (fluid), max GZ 2.10 m at 27° | GM 2.40 m (fluid), max GZ 0.56 m at 16° — fails: Area 30–40° 0.028 < 0.03, Angle of max GZ 16.000 < 25.0 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[13].name` | Stability — half laden | Stability — laden | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[13].ok` | True | False | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].message` | GM 11.70 m (fluid), max GZ 3.30 m at 28° | GM 3.83 m (fluid), max GZ 1.63 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].name` | Stability — light | Stability — half laden | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[15].message` | absent | GM 9.45 m (fluid), max GZ 3.00 m at 29° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[15].name` | absent | Stability — light | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[15].ok` | absent | True | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[1].message` | cargo volume must be positive (tank/hold fits) | No arrangement passes every check on this hull. The closest one is shown. It fails: Stability — laden. | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[1].name` | feasible geometry | Feasible arrangement | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[1].ok` | True | False | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[1].severity` | error | absent | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[2].message` | cargo zone 98.49 m = LOA 130.0 - bow void 9.62 m - stern void 21.89 m | cargo volume must be positive (tank/hold fits) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[2].name` | cargo zone positive | feasible geometry | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[3].message` | 3 x 2.60 m thrusters need 10.20 m across a 16.30 m beam | cargo zone 98.49 m = LOA 130.0 - bow void 9.62 m - stern void 21.89 m | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[3].name` | thrusters fit beam | cargo zone positive | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[4].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 16.30 m beam | 3 x 2.60 m thrusters need 10.20 m across a 16.30 m beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[4].name` | requested thruster count fits | thrusters fit beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[4].severity` | warning | error | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[5].message` | 3 x 40 ft units abreast need 10.91 m across a 16.30 m beam | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 16.30 m beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[5].name` | containers fit beam | requested thruster count fits | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[5].severity` | error | warning | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[6].message` | 3 units in 1 row(s) of 3; a second row adds 12.99 m to the stern block and comes straight off the cargo zone | 3 x 40 ft units abreast need 10.91 m across a 16.30 m beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[6].name` | units fit in one row | containers fit beam | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[6].severity` | warning | error | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[7].message` | wheelhouse 6.40 m wide on a 16.30 m beam | 3 units in 1 row(s) of 3; a second row adds 12.99 m to the stern block and comes straight off the cargo zone | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[7].name` | wheelhouse fits beam | units fit in one row | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[7].severity` | error | warning | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[8].message` | raised eye 13.20 m >= obstruction 13.24 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | wheelhouse 6.40 m wide on a 16.30 m beam | changed |
| ... | 17531 more changed quantities | | | |

### ticket/292-typec-defaults (2026-10-16)

Between `main 5b0fa58` and `this branch 9d42348`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `checks[13].message` | GM 5.96 m (fluid), max GZ 2.41 m at 27° | GM 6.04 m (fluid), max GZ 2.43 m at 27° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[14].message` | GM 12.07 m (fluid), max GZ 3.35 m at 28° | GM 12.62 m (fluid), max GZ 3.41 m at 27° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[8].message` | raised eye 13.20 m >= obstruction 13.88 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.20 m >= obstruction 13.52 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep CN-derived liquid 1 (88x16.3) | `warnings[0]` | Sightline needs 7.18 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.82 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].message` | GM 3.21 m (fluid), max GZ 1.06 m at 25° | GM 3.20 m (fluid), max GZ 1.05 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[13].message` | GM 4.96 m (fluid), max GZ 2.09 m at 27° | GM 4.97 m (fluid), max GZ 2.10 m at 27° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].message` | GM 11.48 m (fluid), max GZ 3.28 m at 28° | GM 11.70 m (fluid), max GZ 3.30 m at 28° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[8].message` | raised eye 13.20 m >= obstruction 13.60 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.20 m >= obstruction 13.24 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `warnings[0]` | Sightline needs 6.90 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.54 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[12].message` | GM 5.18 m (fluid), max GZ 1.99 m at 28° | GM 4.95 m (fluid), max GZ 1.85 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[13].message` | GM 7.48 m (fluid), max GZ 2.93 m at 28° | GM 7.46 m (fluid), max GZ 2.91 m at 28° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[14].message` | GM 14.77 m (fluid), max GZ 3.83 m at 28° | GM 16.70 m (fluid), max GZ 3.96 m at 26° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[8].message` | raised eye 10.91 m >= obstruction 10.41 m + 0.50 m margin (lift used 3.71 m of 6.00 m) | raised eye 10.90 m >= obstruction 10.40 m + 0.50 m margin (lift used 3.70 m of 6.00 m) | changed |
| sweep default depth gas 90x14 | `arrangement.feasible` | True | False | changed |
| sweep default depth gas 90x14 | `checks[10].message` | aft separation 2.00 m >= 2.00 m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11) | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | changed |
| sweep default depth gas 90x14 | `checks[10].name` | opening setback | cofferdam min width | changed |
| sweep default depth gas 90x14 | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | aft separation 2.00 m >= 2.00 m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11) | changed |
| sweep default depth gas 90x14 | `checks[11].name` | fwd end bulkhead space | opening setback | changed |
| sweep default depth gas 90x14 | `checks[12].message` | GM 1.02 m (fluid), max GZ 0.96 m at 36° | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | changed |
| sweep default depth gas 90x14 | `checks[12].name` | Stability — laden | fwd end bulkhead space | changed |
| sweep default depth gas 90x14 | `checks[12].severity` | absent | error | changed |
| sweep default depth gas 90x14 | `checks[13].message` | GM 1.88 m (fluid), max GZ 1.16 m at 36° | GM 1.42 m (fluid), max GZ 1.19 m at 37° | changed |
| sweep default depth gas 90x14 | `checks[13].name` | Stability — half laden | Stability — laden | changed |
| sweep default depth gas 90x14 | `checks[14].message` | GM 5.40 m (fluid), max GZ 1.83 m at 26° | GM 2.70 m (fluid), max GZ 1.37 m at 33° | changed |
| sweep default depth gas 90x14 | `checks[14].name` | Stability — light | Stability — half laden | changed |
| sweep default depth gas 90x14 | `checks[15].message` | absent | GM 7.92 m (fluid), max GZ 2.26 m at 24° — fails: Angle of max GZ 24.000 < 25.0 | changed |
| sweep default depth gas 90x14 | `checks[15].name` | absent | Stability — light | changed |
| sweep default depth gas 90x14 | `checks[15].ok` | absent | False | changed |
| sweep default depth gas 90x14 | `checks[1].message` | cargo volume must be positive (tank/hold fits) | No arrangement passes every check on this hull. The closest one is shown. It fails: Stability — light. | changed |
| sweep default depth gas 90x14 | `checks[1].name` | feasible geometry | Feasible arrangement | changed |
| sweep default depth gas 90x14 | `checks[1].ok` | True | False | changed |
| sweep default depth gas 90x14 | `checks[1].severity` | error | absent | changed |
| sweep default depth gas 90x14 | `checks[2].message` | cargo zone 55.29 m = LOA 90.0 - bow void 12.82 m - stern void 21.89 m | cargo volume must be positive (tank/hold fits) | changed |
| sweep default depth gas 90x14 | `checks[2].name` | cargo zone positive | feasible geometry | changed |
| sweep default depth gas 90x14 | `checks[3].message` | 3 x 2.60 m thrusters need 10.20 m across a 14.00 m beam | cargo zone 55.29 m = LOA 90.0 - bow void 12.82 m - stern void 21.89 m | changed |
| sweep default depth gas 90x14 | `checks[3].name` | thrusters fit beam | cargo zone positive | changed |
| sweep default depth gas 90x14 | `checks[4].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 14.00 m beam | 3 x 2.60 m thrusters need 10.20 m across a 14.00 m beam | changed |
| sweep default depth gas 90x14 | `checks[4].name` | requested thruster count fits | thrusters fit beam | changed |
| sweep default depth gas 90x14 | `checks[4].severity` | warning | error | changed |
| sweep default depth gas 90x14 | `checks[5].message` | 3 x 40 ft units abreast need 10.91 m across a 14.00 m beam | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 14.00 m beam | changed |
| ... | 23567 more changed quantities | | | |

### ticket/264-feasible-fit (2026-10-16)

Between `main 06d10da` and `this branch 5a78519`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `checks[12].message` | GM 2.36 m (fluid), max GZ 0.55 m at 16° — fails: Area 30–40° 0.024 < 0.03, Angle of max GZ 16.000 < 25.0 | GM 3.93 m (fluid), max GZ 1.60 m at 26° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[12].ok` | False | True | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[13].message` | GM 3.86 m (fluid), max GZ 1.65 m at 25° | GM 5.96 m (fluid), max GZ 2.41 m at 27° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[14].message` | GM 9.66 m (fluid), max GZ 3.05 m at 29° | GM 12.07 m (fluid), max GZ 3.35 m at 28° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[8].message` | raised eye 13.20 m >= obstruction 15.88 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.20 m >= obstruction 13.88 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep CN-derived liquid 1 (88x16.3) | `results.governs` | draught — Draught limit | tank volume | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stability.conditions[0].criteria[2].ok` | False | True | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stability.conditions[0].criteria[4].ok` | False | True | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stability.ok` | False | True | changed |
| sweep CN-derived liquid 1 (88x16.3) | `warnings[0]` | Sightline needs 9.18 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 7.18 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].message` | GM 2.47 m (fluid), max GZ 0.58 m at 17° — fails: Angle of max GZ 17.000 < 25.0 | GM 3.21 m (fluid), max GZ 1.06 m at 25° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].ok` | False | True | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[13].message` | GM 3.95 m (fluid), max GZ 1.69 m at 26° | GM 4.96 m (fluid), max GZ 2.09 m at 27° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].message` | GM 10.18 m (fluid), max GZ 3.11 m at 29° | GM 11.48 m (fluid), max GZ 3.28 m at 28° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[8].message` | raised eye 13.20 m >= obstruction 14.60 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.20 m >= obstruction 13.60 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `results.governs` | draught — Draught limit | tank volume | changed |
| sweep CN-derived liquid 2 (130x16.3) | `stability.conditions[0].criteria[4].ok` | False | True | changed |
| sweep CN-derived liquid 2 (130x16.3) | `stability.ok` | False | True | changed |
| sweep CN-derived liquid 2 (130x16.3) | `warnings[0]` | Sightline needs 7.90 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.90 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[12].message` | GM 2.75 m (fluid), max GZ 0.61 m at 16° — fails: Area 30–40° 0.023 < 0.03, Angle of max GZ 16.000 < 25.0 | GM 5.18 m (fluid), max GZ 1.99 m at 28° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[12].ok` | False | True | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[13].message` | GM 4.54 m (fluid), max GZ 1.82 m at 25° | GM 7.48 m (fluid), max GZ 2.93 m at 28° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[14].message` | GM 10.89 m (fluid), max GZ 3.35 m at 29° | GM 14.77 m (fluid), max GZ 3.83 m at 28° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[8].message` | raised eye 13.20 m >= obstruction 15.50 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 10.91 m >= obstruction 10.41 m + 0.50 m margin (lift used 3.71 m of 6.00 m) | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[8].ok` | False | True | changed |
| sweep CN-derived liquid 3 (130x17.2) | `results.governs` | draught — Draught limit | tank volume | changed |
| sweep CN-derived liquid 3 (130x17.2) | `stability.conditions[0].criteria[2].ok` | False | True | changed |
| sweep CN-derived liquid 3 (130x17.2) | `stability.conditions[0].criteria[4].ok` | False | True | changed |
| sweep CN-derived liquid 3 (130x17.2) | `stability.ok` | False | True | changed |
| sweep CN-derived liquid 3 (130x17.2) | `warnings[0]` | Sightline needs 8.80 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Separation clauses are cited from the classification rules for inland-waterways ships (Pt 4 Ch 4 Sec 3, Jul 2022). Confirm against the ADN text before release. | changed |
| sweep CN-derived liquid 3 (130x17.2) | `warnings[1]` | Separation clauses are cited from the classification rules for inland-waterways ships (Pt 4 Ch 4 Sec 3, Jul 2022). Confirm against the ADN text before release. | absent | changed |
| sweep gas IGC_2G 60x10 | `arrangement.feasible` | True | False | changed |
| sweep gas IGC_2G 60x10 | `checks[10].message` | aft separation 2.00 m >= 2.00 m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11) | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | changed |
| sweep gas IGC_2G 60x10 | `checks[10].name` | opening setback | cofferdam min width | changed |
| sweep gas IGC_2G 60x10 | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | aft separation 2.00 m >= 2.00 m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11) | changed |
| sweep gas IGC_2G 60x10 | `checks[11].name` | fwd end bulkhead space | opening setback | changed |
| sweep gas IGC_2G 60x10 | `checks[12].message` | GM 5.81 m (fluid), max GZ 1.82 m at 29° | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | changed |
| sweep gas IGC_2G 60x10 | `checks[12].name` | Stability — laden | fwd end bulkhead space | changed |
| sweep gas IGC_2G 60x10 | `checks[12].severity` | absent | error | changed |
| sweep gas IGC_2G 60x10 | `checks[13].message` | GM 6.48 m (fluid), max GZ 1.85 m at 26° | GM 5.81 m (fluid), max GZ 1.82 m at 29° | changed |
| ... | 12817 more changed quantities | | | |

### Per-tank gas tank clearance, dome in Vc (#237) (2026-10-16)

Between `main d7dbbbe` and `this branch e7477e5`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `arrangement.ruleset` | IGC_2G_EXACT | IGC_2G | changed |
| sweep CN-derived liquid 2 (130x16.3) | `arrangement.ruleset` | IGC_2G_EXACT | IGC_2G | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].message` | GM 2.46 m (fluid), max GZ 0.58 m at 17° — fails: Angle of max GZ 17.000 < 25.0 | GM 2.47 m (fluid), max GZ 0.58 m at 17° — fails: Angle of max GZ 17.000 < 25.0 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `arrangement.ruleset` | IGC_2G_EXACT | IGC_2G | changed |
| sweep default depth gas 90x14 | `arrangement.ruleset` | IGC_2G_EXACT | IGC_2G | changed |
| sweep gas IGC_2G 60x10 | `checks[12].message` | GM 6.12 m (fluid), max GZ 1.82 m at 27° | GM 5.81 m (fluid), max GZ 1.82 m at 29° | changed |
| sweep gas IGC_2G 60x10 | `checks[13].message` | GM 6.74 m (fluid), max GZ 1.85 m at 26° | GM 6.48 m (fluid), max GZ 1.85 m at 26° | changed |
| sweep gas IGC_2G 60x10 | `checks[14].message` | GM 7.53 m (fluid), max GZ 1.90 m at 24° — fails: Angle of max GZ 24.000 < 25.0 | GM 7.37 m (fluid), max GZ 1.90 m at 24° — fails: Angle of max GZ 24.000 < 25.0 | changed |
| sweep gas IGC_2G 60x10 | `checks[8].message` | raised eye 7.00 m >= obstruction 6.50 m + 0.50 m margin (lift used 0.00 m of 6.00 m) | raised eye 7.05 m >= obstruction 6.55 m + 0.50 m margin (lift used 0.05 m of 6.00 m) | changed |
| sweep gas IGC_2G 90x14 | `checks[12].message` | GM 1.04 m (fluid), max GZ 0.23 m at 14° — fails: Area 0–40° 0.007 < 0.09, Area 30–40° -0.050 < 0.03, GZ at ≥ 30° -0.120 < 0.2, Angle of max GZ 14.000 < 25.0 | GM 0.91 m (fluid), max GZ 0.12 m at 8° — fails: Area 0–30° -0.035 < 0.055, Area 0–40° -0.148 < 0.09, Area 30–40° -0.113 < 0.03, GZ at ≥ 30° -0.466 < 0.2, Angle of max GZ 8.000 < 25.0 | changed |
| sweep gas IGC_2G 90x14 | `checks[13].message` | GM 1.96 m (fluid), max GZ 0.98 m at 26° | GM 1.59 m (fluid), max GZ 0.73 m at 26° | changed |
| sweep gas IGC_2G 90x14 | `checks[14].message` | GM 6.41 m (fluid), max GZ 2.23 m at 29° | GM 5.99 m (fluid), max GZ 2.17 m at 30° | changed |
| sweep gas IGC_2G 90x14 | `checks[8].message` | raised eye 13.00 m >= obstruction 15.00 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.00 m >= obstruction 15.51 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep gas IGC_2G 90x14 | `stability.conditions[0].criteria[0].ok` | True | False | changed |
| sweep gas IGC_2G 90x14 | `warnings[0]` | Sightline needs 8.50 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 9.01 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep gas IGC_2G_EXACT 60x10 | `arrangement.ruleset` | IGC_2G_EXACT | IGC_2G | changed |
| sweep gas IGC_2G_EXACT 90x14 | `arrangement.ruleset` | IGC_2G_EXACT | IGC_2G | changed |
| sweep gas IGC_2G_EXACT 90x14 | `checks[12].message` | GM 0.91 m (fluid), max GZ 0.12 m at 8° — fails: Area 0–30° -0.036 < 0.055, Area 0–40° -0.148 < 0.09, Area 30–40° -0.113 < 0.03, GZ at ≥ 30° -0.467 < 0.2, Angle of max GZ 8.000 < 25.0 | GM 0.91 m (fluid), max GZ 0.12 m at 8° — fails: Area 0–30° -0.035 < 0.055, Area 0–40° -0.148 < 0.09, Area 30–40° -0.113 < 0.03, GZ at ≥ 30° -0.466 < 0.2, Angle of max GZ 8.000 < 25.0 | changed |
| sweep gas cargo LNG | `arrangement.ruleset` | IGC_2G_EXACT | IGC_2G | changed |
| sweep gas cargo LNG | `checks[12].message` | GM 1.22 m (fluid), max GZ 0.32 m at 16° — fails: Area 30–40° -0.011 < 0.03, GZ at ≥ 30° 0.083 < 0.2, Angle of max GZ 16.000 < 25.0 | GM 1.22 m (fluid), max GZ 0.32 m at 16° — fails: Area 30–40° -0.011 < 0.03, GZ at ≥ 30° 0.084 < 0.2, Angle of max GZ 16.000 < 25.0 | changed |
| sweep gas cargo LNG pushed | `arrangement.ruleset` | IGC_2G_EXACT | IGC_2G | changed |
| sweep gas cargo LNG_ethane | `arrangement.ruleset` | IGC_2G_EXACT | IGC_2G | changed |
| sweep gas cargo LNG_ethane | `checks[12].message` | GM 1.08 m (fluid), max GZ 0.23 m at 13° — fails: Area 0–30° 0.054 < 0.055, Area 0–40° 0.002 < 0.09, Area 30–40° -0.052 < 0.03, GZ at ≥ 30° -0.136 < 0.2, Angle of max GZ 13.000 < 25.0 | GM 1.08 m (fluid), max GZ 0.23 m at 13° — fails: Area 0–30° 0.054 < 0.055, Area 0–40° 0.002 < 0.09, Area 30–40° -0.052 < 0.03, GZ at ≥ 30° -0.135 < 0.2, Angle of max GZ 13.000 < 25.0 | changed |
| sweep gas cargo LNG_ethane pushed | `arrangement.ruleset` | IGC_2G_EXACT | IGC_2G | changed |
| sweep gas cargo LNG_ethane pushed | `checks[6].message` | GM 0.61 m (fluid), max GZ 0.07 m at 7° — fails: Area 0–30° -0.072 < 0.055, Area 0–40° -0.202 < 0.09, Area 30–40° -0.130 < 0.03, GZ at ≥ 30° -0.565 < 0.2, Angle of max GZ 7.000 < 25.0 | GM 0.61 m (fluid), max GZ 0.07 m at 7° — fails: Area 0–30° -0.072 < 0.055, Area 0–40° -0.201 < 0.09, Area 30–40° -0.130 < 0.03, GZ at ≥ 30° -0.564 < 0.2, Angle of max GZ 7.000 < 25.0 | changed |
| sweep gas cargo LPG_butane | `arrangement.ruleset` | IGC_2G_EXACT | IGC_2G | changed |
| sweep gas cargo LPG_butane | `checks[12].message` | GM 0.88 m (fluid), max GZ 0.10 m at 7° — fails: Area 0–30° -0.058 < 0.055, Area 0–40° -0.185 < 0.09, Area 30–40° -0.126 < 0.03, GZ at ≥ 30° -0.542 < 0.2, Angle of max GZ 7.000 < 25.0 | GM 0.88 m (fluid), max GZ 0.10 m at 7° — fails: Area 0–30° -0.058 < 0.055, Area 0–40° -0.184 < 0.09, Area 30–40° -0.126 < 0.03, GZ at ≥ 30° -0.541 < 0.2, Angle of max GZ 7.000 < 25.0 | changed |
| sweep gas cargo LPG_butane pushed | `arrangement.ruleset` | IGC_2G_EXACT | IGC_2G | changed |
| sweep gas cargo LPG_mix | `arrangement.ruleset` | IGC_2G_EXACT | IGC_2G | changed |
| sweep gas cargo LPG_mix pushed | `arrangement.ruleset` | IGC_2G_EXACT | IGC_2G | changed |
| sweep gas cargo LPG_propane | `arrangement.ruleset` | IGC_2G_EXACT | IGC_2G | changed |
| sweep gas cargo LPG_propane | `checks[12].message` | GM 0.91 m (fluid), max GZ 0.12 m at 8° — fails: Area 0–30° -0.036 < 0.055, Area 0–40° -0.148 < 0.09, Area 30–40° -0.113 < 0.03, GZ at ≥ 30° -0.467 < 0.2, Angle of max GZ 8.000 < 25.0 | GM 0.91 m (fluid), max GZ 0.12 m at 8° — fails: Area 0–30° -0.035 < 0.055, Area 0–40° -0.148 < 0.09, Area 30–40° -0.113 < 0.03, GZ at ≥ 30° -0.466 < 0.2, Angle of max GZ 8.000 < 25.0 | changed |
| sweep gas cargo LPG_propane pushed | `arrangement.ruleset` | IGC_2G_EXACT | IGC_2G | changed |
| sweep gas carries nothing (draught limit) | `arrangement.ruleset` | IGC_2G_EXACT | IGC_2G | changed |
| sweep gas carries nothing (hull too short) | `arrangement.ruleset` | IGC_2G_EXACT | IGC_2G | changed |
| sweep gas channel 60x10 | `arrangement.ruleset` | IGC_2G_EXACT | IGC_2G | changed |
| sweep gas channel 90x14 | `arrangement.ruleset` | IGC_2G_EXACT | IGC_2G | changed |
| sweep gas passes all checks IGC_2G 110x11.4 | `checks[12].message` | GM 0.36 m (fluid), max GZ 0.31 m at 31° | GM 0.30 m (fluid), max GZ 0.15 m at 21° — fails: Area 0–30° 0.045 < 0.055, Area 0–40° 0.048 < 0.09, Area 30–40° 0.002 < 0.03, GZ at ≥ 30° 0.088 < 0.2, Angle of max GZ 21.000 < 25.0 | changed |
| sweep gas passes all checks IGC_2G 110x11.4 | `checks[12].ok` | True | False | changed |
| sweep gas passes all checks IGC_2G 110x11.4 | `checks[13].message` | GM 0.86 m (fluid), max GZ 0.71 m at 34° | GM 0.51 m (fluid), max GZ 0.56 m at 33° | changed |
| ... | 12711 more changed quantities | | | |

### Remove the inland type-G checks (#215) (2026-10-16)

Between `main 84deae4` and `this branch fd101cd`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `checks[12].message` | side clearance 1.58 m >= 0.80 m inland type-G floor (cl. 3.5.2); IGC d(Vc) may require more | GM 2.36 m (fluid), max GZ 0.55 m at 16° — fails: Area 30–40° 0.024 < 0.03, Angle of max GZ 16.000 < 25.0 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[12].name` | side clearance type g floor | Stability — laden | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[12].ok` | True | False | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[12].severity` | error | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[13].message` | double bottom 0.74 m >= 0.60 m inland type-G floor (cl. 3.5.2) | GM 3.86 m (fluid), max GZ 1.65 m at 25° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[13].name` | double bottom type g floor | Stability — half laden | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[13].severity` | error | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[14].message` | GM 2.36 m (fluid), max GZ 0.55 m at 16° — fails: Area 30–40° 0.024 < 0.03, Angle of max GZ 16.000 < 25.0 | GM 9.66 m (fluid), max GZ 3.05 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[14].name` | Stability — laden | Stability — light | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[14].ok` | False | True | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[15].message` | GM 3.86 m (fluid), max GZ 1.65 m at 25° | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[15].name` | Stability — half laden | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[15].ok` | True | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[16].message` | GM 9.66 m (fluid), max GZ 3.05 m at 29° | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[16].name` | Stability — light | absent | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[16].ok` | True | absent | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].message` | side clearance 2.22 m >= 0.80 m inland type-G floor (cl. 3.5.2); IGC d(Vc) may require more | GM 2.46 m (fluid), max GZ 0.58 m at 17° — fails: Angle of max GZ 17.000 < 25.0 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].name` | side clearance type g floor | Stability — laden | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].ok` | True | False | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].severity` | error | absent | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[13].message` | double bottom 0.74 m >= 0.60 m inland type-G floor (cl. 3.5.2) | GM 3.95 m (fluid), max GZ 1.69 m at 26° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[13].name` | double bottom type g floor | Stability — half laden | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[13].severity` | error | absent | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].message` | GM 2.46 m (fluid), max GZ 0.58 m at 17° — fails: Angle of max GZ 17.000 < 25.0 | GM 10.18 m (fluid), max GZ 3.11 m at 29° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].name` | Stability — laden | Stability — light | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].ok` | False | True | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[15].message` | GM 3.95 m (fluid), max GZ 1.69 m at 26° | absent | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[15].name` | Stability — half laden | absent | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[15].ok` | True | absent | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[16].message` | GM 10.18 m (fluid), max GZ 3.11 m at 29° | absent | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[16].name` | Stability — light | absent | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[16].ok` | True | absent | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[12].message` | side clearance 2.25 m >= 0.80 m inland type-G floor (cl. 3.5.2); IGC d(Vc) may require more | GM 2.75 m (fluid), max GZ 0.61 m at 16° — fails: Area 30–40° 0.023 < 0.03, Angle of max GZ 16.000 < 25.0 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[12].name` | side clearance type g floor | Stability — laden | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[12].ok` | True | False | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[12].severity` | error | absent | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[13].message` | double bottom 0.80 m >= 0.60 m inland type-G floor (cl. 3.5.2) | GM 4.54 m (fluid), max GZ 1.82 m at 25° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[13].name` | double bottom type g floor | Stability — half laden | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[13].severity` | error | absent | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[14].message` | GM 2.75 m (fluid), max GZ 0.61 m at 16° — fails: Area 30–40° 0.023 < 0.03, Angle of max GZ 16.000 < 25.0 | GM 10.89 m (fluid), max GZ 3.35 m at 29° | changed |
| ... | 2649 more changed quantities | | | |

### Default moulded depth (#120) (2026-10-15)

Between `main 222d307` and `this branch d5ca0f9`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep default depth dry bulk 90x14 | `result` | fill: warning: Option 'sweep default depth dry bulk 90x14': `channel_bank_slope` was not stored in the file; wrote in the Defaults Profile value 3, marked default \| warning: Option 'sweep default depth dry bulk 90x14': `channel_current_kn` was not stored in the file; wrote in the Defaults Profile value 0, marked default \| ovcd fill: 1 error(s); nothing written | computed | changed |
| sweep default depth gas 90x14 | `result` | fill: warning: Option 'sweep default depth gas 90x14': `channel_bank_slope` was not stored in the file; wrote in the Defaults Profile value 3, marked default \| warning: Option 'sweep default depth gas 90x14': `channel_current_kn` was not stored in the file; wrote in the Defaults Profile value 0, marked default \| ovcd fill: 1 error(s); nothing written | computed | changed |

### Vessel Class (#94) (2026-10-15)

Between `main 882b30b` and `this branch c78f2d5`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep optional vessel_class | `result` | fill: warning: Option 'sweep optional vessel_class': `channel_bank_slope` was not stored in the file; wrote in the Defaults Profile value 3, marked default \| warning: Option 'sweep optional vessel_class': `channel_current_kn` was not stored in the file; wrote in the Defaults Profile value 0, marked default \| ovcd fill: 1 error(s); nothing written | computed | changed |
| sweep optional vessel_scheme | `result` | fill: warning: Option 'sweep optional vessel_scheme': `channel_bank_slope` was not stored in the file; wrote in the Defaults Profile value 3, marked default \| warning: Option 'sweep optional vessel_scheme': `channel_current_kn` was not stored in the file; wrote in the Defaults Profile value 0, marked default \| ovcd fill: 1 error(s); nothing written | computed | changed |

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
