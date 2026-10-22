# Changelog

## Unreleased

- Design change (issue #448, spec 0008): hydrostatics and stability come from the hull's Sections Table (the raked bow and the Stern Form), not from a box with a block coefficient. Draught: the level draught of the sections, or at free trim (Illinois secant, 16/12 steps, trim within 10 degrees) where the hull has LCG data; the reported `draught_m` is then the mid-length draught. KM_T is the upright cut's; GZ is the sections' free-trim curve. The main path, the IS Code and legacy loading conditions, `gas_stability::assess`, journey de-rating, the gas arrangement search, resistance and squat take the hull's displacement, Cb, L_WL and midship area. **Cb is a result** (V / (L_WL B_WL T) at the laden level draught, ES-TRIN 2025/1 Article 1.01, 4.9): `block_coefficient` leaves format v3, and a file that stores it opens with one warning per Option. New result keys `hydrostatics` (the upright values at the laden level draught) and `hydrostatic_table` (10 draughts, 0.1 D to D), shown in the app's results. A condition with no balance within 10 degrees of trim keeps its level draught and fails `trim_in_range`. The Design session no longer runs the engine on each edit (it stamped a hash it never read). The shaft-nozzle propulsor efficiency factor is refitted, 0.447 to 0.437 (the calibration harness writes it). Weights and the resistance wetted surface stay box-based. In a Design the midship section is the hull's own, so the `midship_coefficient` input of #425 has no effect there (it applies in the Python API); #535 takes it out of format v3. Python API: the legacy functions take the sections (Cb no longer counts twice); a NaN or infinite length is refused ("length, beam and moulded depth must be positive numbers"; 0.1.0 gave 0 t and inf t). Design impact: 224 of 224 cases move (`main e53e611`). Cb reads 0.79 to 0.99 (median 0.96). Dry bulk: 72 of 107 carry more cargo, median +13.3 %, up to +16.9 % (a draught-bound hold had the volume of Cb 0.85); 4 self-propelled 90 x 14 m cases now fail `trim_in_range` in the Laden condition, which they passed before. Gas: 91 of 99 feasible before and after; 7 carry more cargo (up to +27 %, the draught-bound channel cases). Crude oil: 17 of 18 feasible before and after; no case carries more cargo. Many more hulls now fail IS Code 2008 Part A 2.2.3 (angle of maximum GZ of at least 25 degrees) in the Light condition, narrow ones too (#498). Fixture and legacy hashes re-stamped from `ovcd run`.
- Design change (issue #448, spec 0008): hydrostatics and stability come from the hull's Sections Table (the raked bow and the Stern Form), not from a box with a block coefficient. Draught: the level draught of the sections, or at free trim (Illinois secant, 16/12 steps, trim within 10 degrees) where the hull has LCG data; the reported `draught_m` is then the mid-length draught. KM_T is the upright cut's; GZ is the sections' free-trim curve. The main path, the IS Code and legacy loading conditions, `gas_stability::assess`, journey de-rating, the gas arrangement search, resistance and squat take the hull's displacement, Cb, L_WL and midship area. **Cb is a result** (V / (L_WL B_WL T) at the laden level draught, ES-TRIN 2025/1 Article 1.01, 4.9): `block_coefficient` leaves format v3, and a file that stores it opens with one warning per Option. New result keys `hydrostatics` (the upright values at the laden level draught) and `hydrostatic_table` (10 draughts, 0.1 D to D), shown in the app's results. A condition with no balance within 10 degrees of trim keeps its level draught and fails `trim_in_range`. The Design session no longer runs the engine on each edit (it stamped a hash it never read). The shaft-nozzle propulsor efficiency factor is refitted, 0.447 to 0.437 (the calibration harness writes it). Weights and the resistance wetted surface stay box-based. In a Design the midship section is the hull's own, so the `midship_coefficient` input of #425 has no effect there (it applies in the Python API); #535 takes it out of format v3. Python API: the legacy functions take the sections (Cb no longer counts twice); a NaN or infinite length is refused ("length, beam and moulded depth must be positive numbers"; 0.1.0 gave 0 t and inf t). Design impact: 201 of 201 cases move (`main 6d46b83`). Cb reads 0.80 to 0.99 (median 0.96). Dry bulk: 71 of 104 carry more cargo, median +11.8 %, up to +16.9 % (a draught-bound hold had the volume of Cb 0.85); no feasibility verdict changes. Gas: 86 of 94 feasible before and after; 7 carry more cargo (up to +27 %, the draught-bound channel cases); the draught at the same displacement is up to 0.30 m less or 0.75 m more (median +0.11 m), as the raked bow and Stern Form hold less volume per metre of length overall than Cb 0.85 did. Many more hulls now fail IS Code 2008 Part A 2.2.3 (angle of maximum GZ of at least 25 degrees) in the Light condition, narrow ones too (#498). Fixture and legacy hashes re-stamped from `ovcd run`.
- Rules (issue #337, spec 0006 section 5.3): the liquid bulk Measures, in the registry and read from the `LiquidBulkResult` only (`Measured::liquid`). `hull.double_side_width` (m), `hull.double_bottom_height` (m) and `hull.lbh` (m3; L x B x H as ADN 2025 9.3.2.11.1 (a) defines it for the tank capacity table: the overall length of the hull, the extreme breadth and the moulded depth) are on the hull object; `cargo-tank.volume` (m3) and `cargo-tank.length` (m) are on the cargo tank object, one per cargo tank (a port and a starboard tank beside a centreline bulkhead are two); `option.deadweight` (t), `option.cargo_capacity_m3` (the gross volume of the cargo tanks, slop excluded) and `option.slop_capacity_share` (the slop tanks over the cargo and slop tanks) are on the Option. The hull, cargo tank, ballast tank and slop tank objects now list the Spaces of the layout. A Rule on a liquid Measure is `not applicable` for a Design with no liquid layout. All are `unverified`. No Rule Set uses them yet, so no result and no Rule Trace moves; the loading condition Measures (`gz.*`, `gm0`) are unchanged.
- Stability (issue #332, spec 0006 sections 2 and 4): a crude oil run has the IS Code loading conditions with its cargo tanks, and a worst slack-tank condition. `loading_conditions` and `stability` are in the results of an `integral_tanks` Option (they were absent). The four conditions of IS Code 2008 Part B 3.4.1.2 are those of the dry bulk path: departure and arrival laden (10 % stores and fuel on arrival), ballast departure and arrival with the double bottom, peak and wing ballast Spaces, and "Lightship, no ballast" where the vessel may sail that way. The cargo is spread alike over the tanks of the layout (`loading_conditions::LiquidCargo`): a tank at 98 % or more is nominally full and has no free surface; a slack tank has the free surface of its section, rho x l x b^3 / 12, a quarter as much with a centreline bulkhead. The worst slack-tank condition (`assess_slack`, the new role `slack_cargo`, named "Slack cargo tanks, worst case") is the departure condition with the tanks filled alike to the part load, from 5 % to 95 % of the gross volume in steps of 5 %, whose fluid GM is the lowest. Every liquid condition lists its weights and where they are (`weights`: the lightship, the cargo, the stores and fuel, each ballast tank; their sum is the displacement and their mean KG is the condition's KG). Gas and dry bulk results are unchanged: the `weights` key is for liquid conditions only. The checks are warnings, and the liquid conditions stay out of the Rule Trace until the liquid Rule Sets exist (the angle of maximum GZ of a low-freeboard barge at light and slack loads misses 25 degrees, as for dry bulk, issue #498). The evaluation harness has two liquid cases in the `box` form with free surface (`liquid_slack`, `liquid_slack_centreline`), written by the oracle from closed forms: 0 false passes, 0 false fails. A warning that stability is computed with the IS Code conditions only replaces the one of the first crude oil ticket (`warn-liquid-legacy-conditions`). The design-impact cases of crude oil move (the table below); no other case moves.
- OVCD Checks of the stern, second slice (issue #355, part): three Rules folded into the unreleased `ovcd` 2.1.0 (minor, Rules added; no Measure is dropped or changed): `ovcd/powering/installed-covers-design-speed` (the lowest service speed of any environment over the Design Speed is at least 1, `block`), `ovcd/wheelhouse/lift-enough` (the lift the Blind Distance needs is within the maximum lift, a check, `block`) and `ovcd/powering/shallow-water-validity` (h/T of the design channel at least 2.0, `warn`, method-validity). New Measures `powering.speed_over_design`, `wheelhouse.lift_enough` and `powering.depth_over_draught`, read from the result's `powering` and `sightline` blocks and the channel input, so a candidate of the arrangement search has none and the fit is unchanged. The other OVCD Checks of the spec table wait for outputs the results do not carry: propeller fits the stern, immersion at the light draught and blade load (propulsor outputs), machinery fits and tug proportions (machinery and tug outputs). The stored hash of `ovcd` 2.1.0 changed (it is a draft), so the fixtures store the new one. No result moves.
- Stern and wheelhouse Rules, first slice (issue #355; the OVCD Checks of spec 0007 and the evaluation-harness cases are not in this slice, so the ticket stays open): `es-trin-2025` 1.0.0 (unreleased, so the Rules are folded in) gains `es-trin/7.02(2)/blind-zone-unladen` (Blind Distance, light, at most 250 m, `block`), `es-trin/5.06(1)/speed-13kmh` (speed through the water at the installed power, laden, deep water, at least 13 km/h, `block`), `es-trin/28.04(1)(a)/propulsion-redundancy` (a check Rule, `block`: a craft over 110 m has a bow thruster with one or more propulsors; equal power, independence and a thruster that works unladen are assumed), `es-trin/7.02(3)/field-of-view` and `es-trin/7.12(2)/elevating-wheelhouse-stability` (`not evaluated`, with their gaps, self-propelled Options) and `es-trin/ESI-III-6.2.1/coupling-force` (a note, `not evaluated` until the Formation powering is built, pushed Options). New `solas-v22` 1.0.0 (`solas/V.22.1.1/blind-zone-light` and `-laden`: at most min(2 L, 500 m), a `clamp` formula; `warn` for every length until the rule data can say "45 m and more", #543; the 10 degree side check is a gap) and `rheinschpv` 1.0.0 (`rheinschpv/1.07(2)/blind-zone-laden`, at most 350 m, `block`, in no default stack: a Designer adds it). All unverified until a second reader checks them. The default stacks: `es-trin-2025` in both Service Areas (as before), `solas-v22` in `coastal-and-river`. New Measures `wheelhouse.blind_distance(condition)`, `powering.speed_through_water_kmh(environment)`, `propulsion.redundancy_28_04`, and the ones named by the `not evaluated` Rules; the Blind Distance and the speed come from the result's `sightline` and `powering` blocks, so a candidate of the arrangement search does not evaluate them (the fit is unchanged, and the 250 m Blind Distance stays sized by `ovcd/aft-block/wheelhouse-sightline`). The Applicability gains the derived input `formation_kind`. A saved file keeps its stored stack, so no old result or trace changes; the new Rules show on a new Option. No result moves.
- Format (issue #331, spec 0006 sections 3.2 and 4): the Cargo Containment `integral_tanks` for a liquid bulk Cargo (crude oil), and a crude oil run. `ovcd-core` `barge_capacity::compute` has a `LiquidBulk` volume branch that calls `liquid_bulk::compute` (so the stern block, trim, sightline, draught and freeboard are those of the dry hull); `takes_gas_path` is gone, and a liquid Cargo with no tank layout is infeasible with a reason. New Hull and containment inputs for `integral_tanks`, in the group `Liquid cargo tanks`: `double_side_m` (1.0), `double_bottom_m` (0.7), `liquid_tanks_in_line` (4), `centreline_bulkhead` (true), `cofferdam_m` (0.6, ADN 2025 9.3.2.11.3), `pump_arrangement` (`deepwell`, or `pump_room` with `pump_room_length_m`, 4.0), `slop_share` (0 river only, 0.03 coastal and river, MARPOL Annex I Reg. 29), `heating_kw_per_m3` (0) and `liquid_fill_ratio` (0.98). The names differ from the spec's table where an input of the same name already exists for Type C tanks (`tanks_in_line`, `fill_ratio`), which has its own group and note. `lightship_factor_t_per_m3` now also applies to `integral_tanks` (0.1141, the fitted tank-barge k). Every keyed default has a `liquid_bulk` row. Results of a crude oil run (`phase` `liquid_bulk`) carry a `liquid` block (Spaces, volumes, free surface of each tank, lightship items, the ADN Tank Vessel Type of the Cargo) and `drawing.liquid_tanks`; the stability and loading conditions, the cost and the drawing come with their own tickets, so the Stability and Capital budget views show Work in development. Gas and dry bulk results have no `liquid` key. Two fixtures, `crude_integral_tanks_pushed` and `crude_integral_tanks_self_propelled`, pin the results hash, and the design-impact set has 18 crude oil cases (a pushed and a self-propelled hull, a channel, coastal service, a hull that carries nothing, the default depth and one case for each new input). Every earlier case gives the results it gave before (the design-impact table below).
- Design change (issue #280): the check "cofferdam min width" (0.60 m, inland class rules Pt 4 Ch 4 3.6.13) is replaced, for every gas Design, by "aft end bulkhead space": the space between the tanks and the aft end bulkhead of the hold is at least 0.20 m, or 0.50 m for plane tank ends (3.5.3, the type G clause; ADN 2025 9.3.1.11.3(a)), as the forward space already was. 3.6.13 is in the type C (liquid cargo) part and asks 0.60 m of a cofferdam; ADN 9.3.2.11.3(a) has the same split, so no gas source asked for 0.60 m. The value checked is the same input as before (`cofferdam_m`, 0.60 m in every Design, which cannot type it), so every verdict stays and every gas results hash moves (the check's name and message): 99 of 224 case designs, each by those two quantities only (the table below). The A-60 class of the bulkhead to the spaces outside the cargo area (3.5.3) is not checked. Rule Sets: `lr-inland-gas` 2.0.0 (unreleased) gains `lr-inland/3.5.3/aft-end-space` on the same Measure, `unverified`; the dropped cofferdam Rule is in 1.0.0 and 1.1.0 only. Python API 0.1.0: `gas_separation.compute` and every result that holds its checks (`simulate`, `barge_capacity.compute`, `gas_arrangement.fit`, the GA adapter) list `aft_end_bulkhead_space` where they listed `cofferdam_min_width`, and a cofferdam between 0.20 m and 0.60 m no longer fails; the 13 scenarios of the API oracle that hold it are in `KNOWN_DIFFERENCES` with the issue. Fixture hashes of the gas fixtures and of the three version 2 legacy gas files were re-stamped from `ovcd run`; the version 1 file keeps its old hash on purpose. #381 (the distance to each opening) is covered by the typed openings of #417 and its review, not by this change.
- Freeboard Rules (issue #428, the maintainer's option A): two new Rule Sets, `es-trin-2025` 1.0.0 and `icll-freeboard` 1.0.0, on the input `min_freeboard_m`. `es-trin-2025` holds ES-TRIN 2025/1 Article 4.01(1) (safety clearance at least 300 mm), 4.01(2) (at least 500 mm for a vessel that sails with its holds uncovered) and 4.02(1) (freeboard at least 150 mm); `icll-freeboard` holds the ICLL type A tabular freeboard of Regulation 28(1), Table 28.1 (24 m to 365 m, linear interpolation between the printed lengths, as the text says), for barges by Regulation 27(14), at the overall length; corrections of Regulations 29 to 40, type B (Table 28.2) and the 25 % reduction for unmanned barges are not applied, so for a self-propelled dry-bulk vessel the limit is a lower bound, not the full requirement. All four Rules are `block` and unverified until a second reader has read ICLL 27(14) and 28 and ES-TRIN 4.01 and 4.02. The default stacks in the Defaults Profile: `river-only` adds `es-trin-2025`; `coastal-and-river` adds `es-trin-2025` and `icll-freeboard` (a coastal vessel also sails on rivers, and the strictest governs). A new dry-bulk input, `hold_covering` (`covered` or `open`, default `covered`, an estimate; a file saved before reads `covered`), makes the 500 mm Rule apply. `min_freeboard_m` is relabelled "Minimum freeboard (safety clearance)": its basis now says it is the ES-TRIN number, not an ICLL freeboard. A file saved before keeps its stored stack, so none of its results change; the engine's `rule_sets` default of a file with no stored stack is the stack the engine checked before. A new coastal Option longer than about 36 m, left at the 0.30 m default, now fails the ICLL Rule and shows the table value (0.793 m at 76.5 m); that is the intended verdict change, and it shows only where a Design chooses `coastal-and-river`. The case set of `tools/design_impact.py` has no coastal Design, so its table is empty.
- Service Area and default stacks (issue #275, spec 0004 Rules engine 8): the Defaults Profile holds the default Rule Set stack by Service Area and Cargo (`rule_sets.default_stacks`; gas: `igc-2g`, `is2008-a`, `lr-inland-gas`, `ovcd`; others: `is2008-a`, `ovcd`; the same for `river-only` and `coastal-and-river` for now). A copy of the profile that lacks them takes the built-in ones; one that leaves out the `ovcd` set is refused. Changing the Service Area or the Cargo in a session resets the `default` sets to the new stack and keeps the `typed` ones. The session API and `ovcd fill` add and remove sets (`add_rule_set`, `remove_rule_set`; `--add-rule-set <id>@<version>`, `--remove-rule-set <id>`); the `ovcd` set cannot be removed, and a set that a selected set includes stays while that set is selected. Every result states its scope (`rule_trace.scope`, in the `ovcd run` summary and in the app's results): the Service Area, the sets, that a river-only result checked no statutory technical code, only the named Rule Sets (ADR 0011), and the count and names of the blocking Rules not evaluated. The app shows the Service Area as a choice and the sets as a list. No result moves.
- Typed openings and their Rules (issue #417, a trial: the maintainer chose option 1 and keeps the ticket open for a review of the openings model; this is part of it, not all). Five new inputs in a new group "Openings", for a gas Option: `deckhouse_front_opening` (false), `deckhouse_side_opening_distance_m` (5 m), `entrance_distance_m`, `engine_room_vent_inlet_distance_m` and `funnel_outlet_distance_m` (2 m each), all estimates; the engine does not place openings, the Designer types where they are. A file saved before reads the same values, marked default, with the one existing note, and keeps its results hash. `igc-2g` 1.1.0 (not yet released, so the Rules are folded in): `igc/3.2.4.1/front-openings` (no entrance, air inlet or opening faces the cargo area) and `igc/3.2.4.1/side-opening-distance` (at least 4 % of L, not less than 3 m, not more than 5 m, from the deckhouse end facing the cargo area; L taken as the overall length), both `block`; and `igc/3.1.1`, `igc/3.1.2` and `igc/3.2.1`, `not evaluated` with their gaps (the model has no machinery, accommodation or store spaces and no division to class). `lr-inland-gas` 2.0.0, a major version: the 0.60 m cofferdam Rule `lr-inland/3.6.13/cofferdam` is dropped (its clause is in the type C, liquid cargo, part; the type G part, 3.5.3, asks for an A-60 bulkhead and 0.20 m, which `lr-inland/3.5.3/end-space` already checks); new `lr-inland/3.3.7/entrance-setback`, `lr-inland/3.3.8/vent-inlet-setback`, `lr-inland/3.3.11/opening-setback` and `lr-inland/3.4.1/funnel-setback` (each at least 2.00 m from the cargo area, on the typed distances), `block`. All new Rules are unverified until a second reader checks them. A deep cofferdam with a door on its aft bulkhead no longer passes: the door's typed distance fails the opening Rule (#381). A new Option takes these versions; a saved file keeps the versions it stores. No result moves: the engine's cofferdam check stays in the check list, because the Python API of 0.1.0 has it and a Design cannot type the cofferdam (it is 0.60 m), so no Design could fail it; the stored hashes are unchanged.
- Core (issue #330, spec 0006 sections 3.2 to 3.3): the `liquid_bulk` module, `ovcd-core/src/liquid_bulk/`, with one interface, `compute(&LiquidBulkInputs) -> LiquidBulkResult`. From a hull, a cargo zone and a layout it gives typed Spaces (cargo tanks, slop tanks, double bottom and wing ballast, cofferdams, pump room) as `compartments::Space`, with volumes, the cargo volume and mass at the fill ratio, the free surface of each slack tank (rectangular, `i = l b^3 / 12`, split by an optional centreline bulkhead) and a lightship. `SpaceKind` has four new kinds. The lightship is `k x LOA x B x D` plus outfit items for pumps, piping, vents and gauging, and heating. k = 0.1141 t/m3 is fitted by `tools/fit_tank_lightship.py` on a declared subset of 15 of the 41 tank-barge records (`calibration/tank_lightship_inputs.yaml`), and written to `calibration/tank_lightship.yaml`; the other records are held out, with a mean error of 41 % against their implied lightship (reported, not tuned). k and the outfit coefficients are Estimates: a record gives a deadweight, not a lightship, and no published outfit weight was found. Nothing calls `compute` yet (the `integral_tanks` ticket does), so no Design and no result moves.
- Cargo (issue #329, spec 0006 section 3.1): crude oil is a seventh Cargo, `crude_oil`, with Cargo Class `liquid_bulk` and a new `status` field (`released` or `development`; crude oil is `development`, every other Cargo `released`). The app cargo list and the cargo picker data carry `status` so the picker can tag it. The default Crude Grade (`ovcd-core/src/liquid_bulk/crude_grade.rs`, kept equal by `tests/test_crude_grade.py` in Python `CRUDE_GRADES`) gives density at 15 C 870 kg/m3 and design density 1,025 kg/m3 (both estimates, with their basis), the sour flag, the CMR, benzene, aquatic toxicity, flash point and vapour pressure flags, pour point, carriage temperature and vapour pressure class. Each value says `sourced` or `estimate` and why. `liquid_bulk::adn_vessel_type` derives the ADN Tank Vessel Type (N open, N closed, C) from the ADN 2025 3.2.3.3 flowchart, with UN 3494 as type C, an unknown flag as type C (decision D2) and a user override. No Design can use crude oil yet: it has no Cargo Containment until the `integral_tanks` ticket, and a Design with it is refused. The six existing Cargos and every saved file keep their results (the design-impact check gives no design change). API changes (Python): `CargoProperties.status`, `Graded`, `CrudeGrade` and `CRUDE_GRADES` are new, and an unknown-cargo error lists `crude_oil`, so the `cargo.get_cargo` and `simulate.cargo_options` scenarios of the API oracle are known differences.
- Text (issue #432, basis item 31): the `convoy_beam_consistent` check of the Vessel Class catalogue says that for the built-in catalogue it is an identity, because the built-in unit beam is the convoy beam over the number abreast. A catalogue given through the API with its own unit beam is still checked independently. The passing message changes, so the two catalogue scenarios of the API oracle are known differences. No Design result moves. An independent unit beam per convoy row (a sourced value) is a separate ticket.
- Input (issue #429, basis item 26, part 2): two new Cost rates inputs, `currency` (a three-letter code in capitals, default `USD`) and `price_base_date` (`YYYY`, `YYYY-MM` or `YYYY-MM-DD`, default none). The capital budget shows both: its basis line now ends "Current estimates." with no date, or "Prices as of <date>." with one. A typed currency makes the rates "Your rates.", as the example rates are in USD. Nothing converts a currency. No figure moves and the results hash is unchanged: the budget is kept out of the hash (the design-impact check gives no design change). A file saved before takes USD and no date, with one note.
- Result (issue #434, basis item 46, labels): every result now carries `calibration`, with `lightship` set to `uncalibrated` for a sloped-hopper hull and a gas barge (no calibration case covers their lightship) and `calibrated` for a box-hold barge, with its basis text. The key is an output, kept out of the results hash (`UNHASHED_KEYS`): no number and no saved hash moves (the design-impact check gives no design change). Not in this change: fitting k and Cb together, and a weight margin.
- Proofs (issue #482): the Lean model proves two properties of sizing, for the waterway designer's use of `arc-rules`. The value sized from the governing Criterion of a Requirement meets every Criterion of it (a strict Criterion: any value strictly beyond the limit), and adding Criteria never lowers the sized value of a lower bound or raises that of an upper bound (`proofs/lean/OvcdRules/Sizing.lean`; core Lean, no Mathlib; names in `proofs/lean/README.md`). The oracle now reports the governing effective bound of each `sizes` Requirement, and the differential test `crates/arc-rules/tests/lean_oracle.rs` compares it with the evaluator. The Lean job stays outside the required checks. No Design moves.
- Rules (issue #317, items 1 and 2): `igc-2g` 1.1.0, a minor version (Rules added; 1.0.0 is kept). `igc/4.23.1.2/min-design-pressure`: the design vapour pressure is not less than Po = 0.2 + A C (rho_r)^1.5 MPa, with A = 0.00185 (sigma_m / 55)^2, C the greatest of the tank diameter, 0.75 times it and 0.45 times the tank length, rho_r the liquid density at the design temperature, and sigma_m taken as the allowable stress the wall formula used. `igc/6.4.1.1/steel-design-temperature`: 9 % Ni, 304L and 316L are steels IGC Table 6.3 permits down to -165 C, so the design temperature is not lower. `igc/6.4.1.1/steel-by-approval`: S690QL1 has a yield stress above the 410 N/mm2 of the carbon-manganese tables, so no permitted temperature can be read; `not evaluated`, with the gap. All three are `warn` and unverified until a second reader checks them. New Measures `tank.design_pressure`, `tank.design_temperature` and `tank.dynamic_pressure_term`. No result moves (the Rule Trace is not hashed). Finding: the default design pressure of fully refrigerated LPG tanks, 0.4 MPa (an estimate), is below the IGC minimum for a large tank (0.444 MPa for propane in the 110 m case), so those Options show a `warn` row. A new Option takes `igc-2g` 1.1.0; a saved file keeps the version it stores.
- Input (issue #425, basis item 1 step 2): a new Hull and containment input, `midship_coefficient` (Cm, default 1.0, an Estimate), sets the midship section area to B x T x Cm for the blockage ratio in squat, the shallow-water resistance and the Schijf limiting speed. At Cm = 1 the section is B x T, as before: no Design moves, and a file saved before takes Cm = 1 with one note. A shaped hull may type a lower value. No source held gives a value of Cm. The Python API has no Cm.
- Rules inputs (issue #274, spec 0004 Rules engine 7): format version 3 gains the rules block, in a new group "Rules". `service_area` (`river-only` or `coastal-and-river`, default `river-only`) and `rule_sets`, the list of Rule Sets an Option is checked against, each `{id, version, hash, source}` with `source` `default` or `typed`. A new Option takes the default stack of its Cargo Containment (gas: `igc-2g`, `is2008-a`, `lr-inland-gas`, `ovcd`; others: `is2008-a`, `ovcd`), each set at its highest built-in version. A run checks the stored sets at the stored versions: a stored version the engine lacks (and no `--rule-set` file supplies) refuses the Option, naming the set; a stored hash that differs from the built-in hash of that version, and a newer built-in version with other content, each give a warning in the Rule Trace, never a swap. The `ruleset` input is removed (its one value, `IGC_2G`, is the rule the gas search always used): a file that names it is read with the name dropped, and a value other than `IGC_2G` is still refused. A file saved before gets `river-only` and the default stack with one warning per Option, and keeps its results hash (the first draft of the spec said version `1.0.0` for each set; `ovcd` and `lr-inland-gas` are past it, so the stack is the highest built-in version, which is what the engine checked). The six fixtures were refilled with `ovcd fill`. Inputs only: no result moves.
- Design change (issue #420, part b): the resistance method's validity checks reach the Design result. The checks part a keeps at each service speed (squat, waves, the resistance fits, the Schijf limit) are now in the run's `checks`, named `<check>[<environment>]` in a channel, with their own severity. `power within installed` is left out: the run gives the resistance model a stand-in installed power, so it would test nothing. The Rule Trace reads each check that comes once per environment under its plain name too, passing only when every environment passes, so the `ovcd/squat/*` Rules now evaluate in a channel (they were all `not applicable`). A channel with a depth but no bottom width gets a warning that its squat, Schijf limit and blockage are not checked (new Message Id `warn-channel-no-bottom-width`, draft `es`, `pt` and `zh-Hans`). `ovcd` 2.1.0, a minor version: six Rules added for the new checks; 2.0.0 is kept. Of the 196 design-impact cases, all move: each gets `coefficients are first order`, and the 10 with a channel get 8 more. No error check fails; 5 warnings fail (`methods agree within 50pct` in 4 cases, `froude depth subcritical` in 1). Fixture and legacy hashes re-stamped from `ovcd run`.
- Input (issue #410, part 1b): two new Gas tanks inputs set the weather shield over the tanks: `weather_shield_thickness_mm` (default 6) and `weather_shield_clearance_m` (the shield top over the tank top, default 1.5). Part 1a fixed both values in the engine. A file saved before format v3 keeps them, with one note. At the defaults no Design moves (the design-impact check gives no design change).
- Design change (issue #411): an insulated 2:1 semi-ellipsoidal tank head is R_s/2 + t_ins long (R_s the shell outer radius, t_ins the insulation), not (R_s + t_ins)/2: the insulation adds its full thickness at the crown. The cylinder is t_ins shorter per tank, so the tank volume and the cargo fall a little. Only insulated LPG Options with 2:1 heads move (semi-pressurised and fully refrigerated LPG; fully pressurised LPG has no insulation; hemispheres were already right): cargo -0.5 % on the LPG carriage sweeps, -0.9 % on the `tank_head_type` sweep. No saved file moves.
- Design change (issue #410, part 1a, the maintainer decision on #410): the covering above the moulded depth D is a non-structural weather covering by default. The hull steel is weighed to D only (`steel_rate_per_cbm` x LOA x B x D; the wing plate no longer raises the moulded depth). The wing plate, the side plating from D up to it, and the weather shield (6 mm, top at the tank top + 1.5 m) are lightship items, each with its own VCG and LCG (`weights::covering_items`); only the parts above D count, so a D set over the tanks makes the covering structural. The gas stability and the IS Code loading conditions take the items at their own heights. The bottom-up steel and the cost do not change. Effect, on top of the IS Code loading conditions of #426 part 3: the light KG rises by about 0.5 to 0.6 m on gas designs with a wing tank. 86 gas cases move (lightship, KG, GM); the cargo moves on three: -21 % on the `tank_material` sweep (1,754 t to 1,378 t), and the two gas channel cases (+6 % and +25 %, both now infeasible). Four sweeps turn infeasible: LNG and LNG/ethane pushed, and the two gas channel cases. The new failures are the IS Code 2008 Part A 2.2.3 angle of maximum GZ of at least 25 degrees, in the Light condition. That criterion on wide, shallow hulls is #498. In the `gas_arrangement::fit` API (the legacy gas stability) the CN II benchmark falls from 2,021 t to 1,406 t: one tank abreast fails that criterion, so the search takes two. Part 1b (the shield thickness and clearance as inputs) follows.
- Text fix (issue #433, item 1): the limitation `stab-limit-no-wind-or-damage` cited ADN 9.3.1.15 for wind heel, but ADN 9.3.1.15 is the damaged-condition stability only. It now cites ADN 9.3.1.15 and IGC 2.7 for damage stability and the IS Code 2008 weather criterion (Part A 2.3) for wind heel. Every result with a stability block moves by this text (107 design-impact cases move by it alone).
- Change (issue #420, part a, the resistance side): `resistance::compute` now keeps the method checks at the service speed of each environment. Where squat applies: the `squat.rs` checks (`keel_clears_bed`, `keel_clears_bed_with_margin`, `froude_depth_subcritical`, and `blockage_within_barrass_range` and `methods_agree_within_50pct` when they apply); where there are waves: the STAwave-1 checks (`stawave1_short_wave_validity`, `wave_height_within_screening_range`, `bow_length_supplied`); in shallow water: a new warning `lackenby_fit_subcritical` (the shallow-water fit holds below a depth Froude number of 0.95, `LACKENBY_FNH_MAX`; above it a fixed factor stands in, an OVCD choice). Each is named for its environment, as `keel_clears_bed[<environment>]`. They are in `ResistanceResults.checks` (and the Python API); the Design run does not copy them yet (part b, the rules lane), so no result or hash moves. New message `check-lackenby-fit-subcritical`.
- Design change (issue #421): one air draft verdict per Design. With an air draft limit, a run listed two checks of the hull's air draft: the capacity result's `air draft within limit`, which takes the ballasted light condition when the hull carries ballast, and the run's own `Air draft (empty, governing)`, which always took the empty light condition. The second is removed (its Message Ids `check-air-draft-limit` and `name-check-air-draft` too). The IS Code loading conditions keep their own air draft per condition. `ovcd` 2.0.0, still unreleased since #415, drops the Rule `ovcd/capacity/air-draft-empty`, so 2.0.0 is the one major version for both. Of the 194 design-impact cases, the 3 with an air draft limit move: one check fewer, and the checks after it move up one place; no value and no verdict moves.
- Change (issue #419): severity on every check of a run, and a failed warning reads as a warning. Four checks of `simulate.rs` had no `severity` and were read as errors: `Cargo > 0` (dry and gas), `Feasible arrangement`, and the gas stability check of each legacy loading condition; each now says `error` (the fifth, `Air draft (empty, governing)`, goes in #421). No verdict changes: a check with no severity was already an error. The app's check list (`BottomPanel.svelte`) shows a failed warning check with the warning tag, not FAIL, after the failures that gate; the CLI summary already listed failed warnings apart (#418) and now has a test for it. Every results hash moves (194 of 194 design-impact cases, only in `checks[n].severity`, absent to `error`); fixture and legacy hashes re-stamped from `ovcd run`.
- Docs (issue #431, part 1): `calibration/README.md` no longer says the barge database is unreachable; sources K1 to K39 cite archived copies of its pages. The Yangtze motor vessel note in `calibration/results.md` (and the harness text that writes it) says the harness takes Lwl = 0.97 x LOA while the app uses Lwl = LOA. A new test fails when the committed `calibration/results.md` or `factors.yaml` is not what the harness writes (`python -m vessel_designer.calibration` writes them). No result moves.
- Drawings (issue #407, part 1): the GA draws the wing tank top at the wing plate level the weights carry (tank bottom + wing tank height ratio x tank outer diameter), no longer clamped at the deck. Where it stands above the Moulded Depth, the midship section draws the side shell up to it, and the side elevation shows the wing plate in view (hidden below the deck). Both gas fixtures have the wing plate above the deck (6.05 m and 5.35 m on a 4.5 m depth), so their drawings change. The weather shield outline waits for its values in the `drawing` block. No result moves.
- API change (issue #413), Python API only; no Design moves (the design-impact check gives no design change). The legacy chain `hydrostatics.calculate_stability` / `tanks.free_surface_moment_cylindrical` took the fill as a height ratio (h = fill x 2r); it now takes it as a volume ratio and finds the liquid height of the circular segment (`gas_stability.segment_fill`), as the gas stability does. At 0.95 to 0.98 fill the free surface moment is about 12 times larger, so the legacy GM_fluid is much lower. `to_ga` no longer sets three flags to a constant true: `min_freeboard_ok` is the drawn freeboard against the required freeboard, `air_draft_ok` the air draft against its limit (true with no limit), `stability_ok` the gas stability criteria (a dry result: GM at least 0.15 m; false with no GM). Its drawn tank cylinder holds the volume the two heads do not (the heads counted twice). The doc comments say what the legacy chain leaves out (Cb applied to the raked hull volume; KG items that do not sum to the lightship).
- Design change (issue #415): the dry bulk hold GM is checked at the cargo and the draught the Design reports, in every dry bulk Option. It ran only at the volume-bound load, and only when that load floated, so a dense cargo that a draught limit cuts (bauxite in the default box hold) got no GM figure at all. The check is renamed `intact gm at reported cargo` (it tests GM >= 0.15 m; "gm positive" said less) and stays a warning. The first-order method is unchanged (`barge_capacity::bulk_gm_at`; the cargo KG of the full hold, on the safe side for a smaller load). The core `barge_capacity` result keeps its own `gm_positive` check (Python API unchanged). The `ovcd` Rule Set is 2.0.0, a major version: `ovcd/capacity/hopper-gm` is replaced by `ovcd/capacity/intact-gm`, which cites IS Code 2008 Part A 2.2.4 as the origin of the 0.15 m value (read in MSC.267(85), annex p. 12); it is a tool limit, not a sourced Rule. `ovcd` 1.0.0 is kept. New Message Ids `check-intact-gm-reported-cargo` and `check-name-intact-gm-at-reported-cargo` (draft `es`, `pt` and `zh-Hans`). Of the 194 design-impact cases, the 104 dry bulk ones move: only `checks` (the new check, and the checks after it move one place) and `warnings` (the first-order GM note, now in all 104); no cargo, draught or feasibility value moves, and no new check fails. Dry bulk fixture and legacy hashes re-stamped from `ovcd run`.
- Drawings (issue #499): the drawing crate moved to its own repository, [open-technical-drawer](https://github.com/African-River-Corridors/open-technical-drawer), as `otd` 0.1.0, so other tools can draw through it. OVCD pins it by tag and keeps the name `ovcd_draw` in its code. The code is the same and the standard is still 1.0, so no drawing and no result changes. `tools/otd.py` says where cargo keeps the pinned crate; `tools/vendor_standard.py` moved there.
- Docs (issue #430): one glossary pass for the engineering basis. The two Moulded Depth entries are one. New terms in new groups (Hull and stability; Gas cargo arrangement; Waterway and powering; Cost; Drawings; Evidence), tank and cargo terms in Design, and Origin, Consequence, Sizing Rule, Not Evaluated, Set Version, Edition and Selection Order in Rules. Side Clearance is the one term for the clear distance from the moulded side to the insulation ("clear gap" is to be avoided). "Governing" is kept for the Governing Rule. Waterway Limit now names the draught and air draft limits too. No result moves.
- Drawings (issue #406, part 1): the GA draws a dome only at the dome diameter the results give (`drawing.tank.dome_diameter_m`, the core's rule or the typed size, #316). The three `DOME_*` proportion constants that stood in when it was missing are removed; results with a dome and no diameter are refused with a message, as for any other missing value. The engine always gives the diameter with a tank, so no drawing moves. No result moves.
- Rules (issue #424): `lr-inland-gas` 1.1.0, a minor version (Rules added; 1.0.0 is kept). The tank length over diameter limit of Pt 4 Ch 4 3.5.1 is now a class Rule, `lr-inland/3.5.1/tank-l-over-d`, on the engine check "tank length over diameter" (the inner shell diameter since #416); the `ovcd` tool limit stays. 3.2.1 (tank capacity, Table 4.3.1) and 3.5.2 (the hull in the cargo area) are listed as not evaluated, each with what the engine lacks. The three are codified and wait for a second reader. Gas Options now select 1.1.0. In `fit`, the Rules on one engine check now count once when the search ranks candidates that all fail (`failing_groups`; the kernel `Failure` carries its Measure), so a check that two sets report does not count twice. No case of the design-impact set moves.
- Design change (issue #426, part 3): the IS Code 2008 loading conditions are the default, and the gas arrangement search checks its candidates in them. A new Option gets `loading_conditions = is_code_2008` (the four conditions of Part B 3.4.1.2, with stores and fuel and water ballast, and "Lightship, no ballast" where the vessel may sail that way); a file saved before keeps `legacy`, and every fixture and legacy results hash still matches. With the IS Code conditions, `fit` judges each candidate by them (stability, ballast plan and air draft of each condition, by the same core functions the results use, `gas_arrangement::is_code_conditions` and `is_code_cargo`) and ranks it by the cargo it carries with its stores and fuel aboard. The Python API keeps the legacy search. The stores and fuel are now worked out before the search, at the design draught (the draught the installed power is sized at); part 2 used the answer's draught. Design impact (194 cases, `main b8e3059`): 190 move, most only by the new `loading_conditions` and `stability` blocks. Gas: 87 of 90 keep their arrangement and cargo. On main, run with `is_code_2008`, none of the 86 feasible gas answers fails an IS Code condition, so the 43 of 89 failures PR #440 found no longer occur (`fit` has changed since: #418, #422, #277). Three gas cases move: the two 110 x 24 m wide hulls become feasible (a 3-wide arrangement, 4,161 t to 3,011 t; before, no arrangement passed and the closest failed the sightline), and the 200 x 12 m hull, infeasible both before and after, shows a different closest candidate (4,333 t to 2,111 t). Every feasible gas answer meets every IS Code condition (86 of 86). Dry bulk: 7 of 103 cases carry up to 0.75 % less cargo (a self-propelled, draught-bound hold now carries its stores and fuel); their condition checks stay warnings. Tests: `trim_results` now sets `legacy`, as its point is the legacy trim fields.
- Docs (issue #433, the parts outside the rules lane's files): the head thickness formulas no longer cite ASME UG-27, which is the cylindrical shell paragraph; the head paragraph is not in the rule-source library, so none is cited (Rust and Python). The `resistance.rs` header no longer says nothing is fixed in the port, and states that it keeps g = 9.81 m/s^2 and 1 kn = 0.5144 m/s while `squat.rs` and `waves.rs` use 9.80665 and 0.514444 (making them one would move results; not done). The LPG mix density comment uses butane at 600, the value the blend is worked out with (it said 601). Comments only: no result moves.

## 0.2.4 (2026-10-21)

Highlights, since 0.2.3:

- Rule Sets. Checks are now Rules in versioned, content-hashed Rule Sets: `is2008-a`, `igc-2g`, `lr-inland-gas` and `ovcd` (1.0.0), and `lr-inland-gas` 1.1.0. Every applicable Rule applies and the strictest governs. Every result carries a Rule Trace, and the app shows it. This release is the first to ship Rule Set versions, so their files are now frozen (`RELEASED.json`).
- `arc-rules`: the domain-free part of the rules engine is its own crate, so other tools can share it.
- Type C tanks follow the IGC Code more closely: cited allowable stresses, the design liquid pressure on the wall, the tank head type as an input, the dome rule, and LPG at its loading temperature.
- Stern and propulsion for self-propelled hulls: stern form, propulsor, machinery, installed power, trim per loading condition, and the wheelhouse Blind Distance.
- Tug inputs and two new Formations: ATB and towed barge.
- Confined water: one midship section, the Schijf limiting speed, and installed power.
- The IS Code 2008 loading conditions are the default for new files, and the gas arrangement search checks its candidates in them.
- The first Vessel Model steps: the Sections Table, equilibrium with free trim, and the GZ curve on the sections.

Read this before you use this build:

- Format version 3 is not final. Files saved as version 3 may need changes before 0.3. Version 2 files still open.
- Results move for most gas designs. See the design-impact entries below.
- Known open finding: the gas hull steel KG is low when the wing plate is above the moulded depth D (issue #410), so GM is high on those designs.
- Concept and preliminary design only. Results are not verified.
- This build also carries the changes listed under "Unreleased" at the `v0.2.4` tag. They were merged after these notes were cut.

- Change (issue #401, step 1): the capital budget shows its two hull steel figures. The lightship, piping and outfitting use the top-down steel (`steel_rate_per_cbm` x moulded volume); the budget prices the bottom-up steel (hull plating and structure, wing plate, weather shield). The `cost` block has a new `steel` field (`top_down_t`, `bottom_up_t`, `delta_t` = top-down less bottom-up), and the app shows it under the hull steel line (new message `app-cost-steel-delta`, English). The doc comments of `calculate_weights` and `piping_factor` now say which steel each uses. The budget is outside the results hash, so no result or hash moves.
- Change (issue #403): the capital budget adds a tug only for a Formation that has one. A self-propelled Option has no tug breakdown: the tug's hull steel (the core default, 115 t) and its outfitting lump are no longer costed, and the vessel's own battery, propulsion and wheelhouse and accommodation are barge items. At the example rates that is 487,500 less in the subtotal (115 t x 2,500 + 200,000), 536,250 less in the unit total with the 10 % yard margin, for a self-propelled gas Option. Every other Formation keeps the tug line. New core function `barge_cost::compute_for(inputs, rates, tug)`; `compute` is unchanged, so the Python API does not move. The app hides an empty tug table. The budget is outside the results hash, so no result or hash moves.
- Change (issue #404, part 1, reading A): cost labels and the mass check. The capital budget's basis line follows the rates: "Example rates — replace with yours." only while every cost rate is the Defaults Profile's; once any rate is typed it reads "Your rates." The `cost` block has a new `rates` field (`example` or `user`), and the app shows the line from it (new messages `app-cost-basis-example`, `app-cost-basis-user`). The `masses_positive` check now covers every priced mass: the hull steel, tank shell, tank supports, insulation, piping and outfitting must be positive; the wing plate, weather shield, tug hull, battery and propulsion must be zero or more (it tested only the tank shell and the hull steel). No case of the Python API oracle changes. The budget is outside the results hash, so no result or hash moves.
- Design change (issue #399): the residuary resistance fit is continuous at Fn 0.20. Cr/Cf rose to about 0.179 just below Fn 0.20 and stepped down to 0.150 at Fn 0.20; the part above Fn 0.20 now starts at 0.179 (0.10 + 2.5 x 0.10^1.5), the higher and so conservative of the two ways to close the step. The fit is an OVCD Estimate, not from a source: the Python module no longer names the Guldhammer-Harvald charts, which the code does not use. At Fn 0.20 and above the residuary resistance rises by 0.029 x the frictional resistance (19.4 % of the residuary part at Fn 0.20, less above), and power and fuel follow; points below Fn 0.20 do not move. Tests re-pinned to what the code prints. No sourced test changed.
- Change (issue #400): speed governing labels. A service speed at the top of the speed grid is governed by `speed grid` (it was `power`). A sweep that stops at the channel cap is `channel` even when the grid step stops it more than 0.05 kn below the cap (it was `power`). A new warning check, `service_speed_clears_squat[<environment>]`, flags a service speed whose point fails the under-keel clearance (no speed point clears it, so the first point stands in); new message `check-service-speed-fails-squat`. No number moves.
- Rules engine (issue #463): the domain-free part of the engine is a new crate, `arc-rules` (ADR 0019), so another tool can share it. It holds the Rule Set model, the schema and the loader, the content hash, the library and selection checks, the version classes, Applicability and the evaluator (strictest governs, the Rule Trace). Object types are string keys; the caller passes in its Measure registry, its applicability vocabulary and its formula shapes, and the evaluator reads values only through `MeasureSource`. OVCD registers its side in `ovcd-core::rule_trace` (object types, Measures, the five formula shapes) and `ovcd-format::rule_set` (the vocabulary, the built-in sets, the provenance gate). The crate is `no_std` with `alloc` and depends on no OVCD crate. Rule Set schema 1.x is now frozen for outside users (additive changes only); two additive changes join it: object-scoped inputs for Applicability (`MeasureSource::object_input`, by default the design's value) and the origins `guideline` and `national-practice` (published, like `regulation` and `class`; OVCD uses neither). The Lean differential test now exists and runs against `arc-rules` in the weekly Lean job. No result moves: every results hash, fixture hash, Rule Set content hash and Rule Trace is the same (194 design-impact cases, 11 fixture and legacy files, compared in full).
- Design change (issues #396, #397, #398): confined water. One midship section, B x T, for squat and resistance (resistance used B x T x Cb, so its blockage ratio was low by Cb). The Schijf limiting speed is the first critical speed proved by Delefortrie, Verwilligen, Vantorre and Lataire (35th PIANC World Congress, 2024, Paper 8): Fr_h = [2 sin(arcsin(1 - m) / 3)]^(3/2) on the water depth, which is 1 in open water (the old sqrt(2/3)(1 - m)^(1/3) gave 0.816). One margin, 0.95 of the limit (an OVCD choice), in the resistance sweep, its check and the electric check (which used 1.0). The electric installed power follows the environment with the highest battery power (it followed the lowest service speed). The open-water environment reports a blockage of 0 (maintainer decision on #396), so every results hash moves once; numbers move only with a channel. In the 10 design-impact cases with a channel the limit falls by 9 % to 53 % (box 90 x 14 m: 10.60 to 7.24 kn; the narrowest sweep channel: 7.73 to 3.61 kn) and the speed curves stop lower. Python, known differences from 0.1.0: the `resistance`, `electric_energy.compute`, `simulate.box` and `simulate.hopper_channel` scenarios. Tests re-pinned to what the code prints; three knife-edge inputs solved again; fixture and legacy hashes re-stamped from `ovcd run`. No sourced test changed.
- Feature (issue #426, part 2): the IS Code 2008 loading conditions, opt-in by the new format v3 input `loading_conditions` (`legacy` stays the default until the gas arrangement search uses the new conditions; a file saved before reads `legacy`). With `is_code_2008`, gas and dry bulk Options are checked in the four standard conditions of Part B 3.4.1.2 (full load and ballast, departure and arrival), and in "Lightship, no ballast" where `may_sail_without_ballast` (default: barges yes, self-propelled no). Stores and fuel (self-propelled only: `fixed_stores_t` plus fuel for `endurance_h` at the service speed; none for battery-electric) are 100 % at departure and 10 % at arrival, with their largest free surface (3.1.3.2, 3.1.4); the full load carries them, so a draught-bound cargo is that much less and the deadweight holds them. Each ballast condition has a ballast plan, the least ballast that meets every criterion or user fills, stated per tank (3.4.2.3), with the free surface of slack tanks. Dry bulk condition checks are warnings and stay out of the Rule Trace until a dry bulk Rule Set exists. New results blocks `loading_conditions` and (for dry bulk too) `stability`, only when opted in. Five design-impact sweep cases opt in; their table is under "Design changes".
- Design change (issue #423), design grid only: each cell fills its tanks to the Filling Limit, 98 % (IGC 15.3; `tanks::DEFAULT_FILLING_LIMIT`, which `simulate` now also uses), not the 0.95 default of the capacity inputs. The grid has no loading-limit inputs, so it fills as a `.vdesign` run does when none is typed. A cell's cargo rises by 0.98 / 0.95 (60 x 11 m propane: 692.6 t to 714.4 t). The spacing part of the issue was already built: #391 (#277) took the 1.0 m override out, so the side clearance is IGC 2.4.1.2 per tank. Single-Design results do not move (177 designs, no change).
- Design change (issue #422): two small fixes in the gas arrangement search. (1) A tank put on the diameter cap now leaves the n - 1 transverse gaps (1.0 m each, `DEFAULT_TANK_GAP_TRANSVERSE_M`) between n tanks across; the side clearance took all the spare beam before, so two or more capped tanks came out below the cap (for example 7.5 m on an 8 m cap, 22 m beam, two across). (2) `smallest_viable_loa` returns only a length it has tested: the rounded length when it passes, else the length rounded up to 0.1 m when that passes, else the tested length itself. Rounding to nearest could return an untested length below the one the search found. No case of the design-impact set moves (177 designs); results move only for capped hulls with two or more tanks across.
- Design change (issue #416, maintainer decision on the reviewer's question): the tank length over diameter limit (at most 7; ADN 2025 9.3.1.11.1 (b), inland class rules Pt 4 Ch 4 3.5.1) takes the inner diameter of the steel shell, as ADN 2025 1.2.1 defines the diameter of a tank shell. It took the outer diameter over the insulation, the most lenient datum. The length stays the overall tank length, heads included. Stricter: a long, thin tank can now fail. The check message names the inner diameter (new Message Id `check-tank-length-over-inner-diameter`; draft `es`, `pt` and `zh-Hans`), and the design grid's `slenderness` uses the same datum. Of the 194 design-impact cases, the 89 gas ones move in that message only; no verdict, cargo or feasibility changes. Gas fixture and legacy hashes re-stamped from `ovcd run`. A run-time Rule test moved to a 90 m hull, where a narrowed tank still meets the limit.
- Design change (issue #418, two maintainer decisions): no hidden check gates `fit`. The two capacity checks that test the full tank volume, `not overloaded` and `freeboard ok`, no longer remove candidates: a candidate is de-rated to its payload, and the carried draught is gated by `freeboard at payload` (with the user's freeboard) and the journey `floats` check. `freeboard ok` now reads the user's freeboard, not a hidden 0.3 m. Every result lists the two as warnings (they were hidden), and a gas result now also lists the search's own checks of the answer (`capacity is volume bound`, `floats`, `ukc consistent`, `utilisation reasonable` as a warning, `tank length over diameter`, `freeboard at payload`), so every check that can remove a candidate is visible. The `ovcd` Rules `ovcd/capacity/freeboard` and `ovcd/capacity/not-overloaded` are `warn`. The CLI summary and the drawing's failed-checks note count failed errors only; the CLI lists failed warnings on their own line. Every results hash moves (177 of 177 design-impact cases; no cargo, draught or feasibility value moves); the fixtures and the legacy test files are re-stamped from `ovcd run` (the v1 ADN file keeps its old hash, as its test expects a difference). API: `simulate.box` and `simulate.hopper_channel` are new known differences from 0.1.0. New core function `gas_arrangement::answer_search_checks`.
- Trademark policy: new `TRADEMARKS.md` says how the OVCD and ARC names and logos may be used. The Apache licence covers the code; this policy covers the marks. `NOTICE` names the copyright holder by its new name, African River Corridors Limited (company no. 541064, formerly Mobile Clipboard Limited). No result moves.
- Known limitation (issue #282, as the maintainer decided): the 0 to 40 degree and 30 to 40 degree areas of IS Code 2008 Part A 2.2.1 always run to 40 degrees, because the model has no down-flooding angle yet. The README says so, and the app's Stability view lists it under Limitations for every gas design (new message `app-stab-limit-downflooding`; draft `es`, `pt` and `zh-Hans`). The Rule Trace rows of the two areas already link the finding. The engine's own limitation list is part of the results and is not changed, so no result or hash moves.
- Evaluation harness (issue #279): rule data mutation. For each numeric limit and coefficient of each evaluated Rule of the selected sets, the harness writes a mutated copy of the set with that number moved one step to each side, loads it at run time, and runs the corpus. A mutant that no case notices is a weak-corpus warning; `results.md` lists every mutant and the cases that notice it. The steps are the big steps of the boundary mutants per unit (m rad 0.002, m 0.01, degrees 2; 1 % for coefficients and table values). On `is2008-a` all 12 mutants are noticed. The harness runs in about a second, so it stays in the pull-request gate. No result moves.
- Evaluation harness (issue #278): the harness reads the engine's Rule Trace and compares by rule id. A case's `expected` entries are keyed by the `is2008-a` rule ids (`area_0_30` is now `is2008/A.2.2.1/area-0-30`, and so on); `oracle/write_cases.py` and `oracle/mutants.py` write the new keys, and no value changed. A new eval function, `gas_stability.box_trace`, gives the Rule Trace of one box loading condition under the selected Rule Sets (built-in, or loaded at run time); it is the harness's, not part of the Python API. Each disagreement goes in one triage bucket: `model-gap`, `known-limitation`, `code-bug`, `rule-suspect` or `needs-expert`. `results.md` lists coverage per Rule Set and per Rule by id, and the buckets. A false pass on a blocking Rule still fails the build; a `not evaluated` row against an expected `fail` is a model gap, never a pass. The corpus result is unchanged: 253 known-good and 108 known-bad verdicts, 0 false fails, 0 false passes, no disagreement. No result moves.
- Design change (issue #277): the gas arrangement search (`fit`) reads the Rule Trace of each candidate. With Rule Sets selected (every `.vdesign` run), a candidate wins only if no blocking Rule fails; the answer's trace is the one shown, and the failing Rules of an infeasible answer are named in "Feasible arrangement". The `ovcd` set gains Rules for the search-only checks (overload, freeboard, journey, utilisation as a warning, freeboard at payload, stability not assessed). A Rule Set loaded at run time now constrains `fit`. Two maintainer decisions are built here. (1) Issue #264: the wheelhouse sightline is an error check, so it blocks `fit`, the design grid and `smallest_viable_loa`; `ovcd/aft-block/wheelhouse-sightline` is `block`. (2) Issue #283, step 2: the design grid sizes each cell's side clearance with the IGC 2.4.1.2 rule (d(Vc) per tank less the insulation), not the 1.0 m default, and the wing ballast tank width is that clearance less the gap. Of the 177 designs of the design-impact set, 107 move: 92 only by the sightline check's severity (`warning` to `error`); 11 gas designs carry 14 % to 31 % less cargo, because a taller tank hid the view from the raised wheelhouse (the CN II benchmark, 90 x 14.8 x 6.5 m propane at a 4.4 m draught cap: 2,832.66 t to 2,021.25 t, a 10.70 m tank); 2 sweep designs have no feasible arrangement now (130 x 16.3 m: the eye cannot clear the tanks at any depth; the optional tank head type case: each arrangement that clears the view fails the light-condition angle of maximum GZ); the other 2 move in other quantities. The before/after table is under "Design changes" below. API: `aft_arrangement.compute`, `barge_capacity.compute` (dry bulk) and `ga_adapter.to_ga` (dry bulk) list the sightline with severity `error` (known differences from 0.1.0); the 0.1.0 constant `design_grid.SIDE_CLEARANCE_M` is kept but not read. Tests re-pinned to what the code prints (the CN II benchmark, the gas snapshots, the README example and the design grid cells, which now pick cells the sightline does not block). Fixture hashes re-stamped from `ovcd run`. No sourced test changed.
- Rules engine (issue #273): every released Rule Set version is kept. `crates/ovcd-format/rules/RELEASED.json` lists the versions a tagged release has shipped, with their content hashes (none yet: the next version bump adds the four built-in sets). A new fast-gate step, `ovcd rules --provenance`, fails on a changed released file, a version number below its class (patch, minor or major by content hash), a rule id that changes its Measure or direction or comes back after it was dropped, and a regulation or class Rule with no paragraph; it prints the unverified blocking Rules of each set. A `v*` release fails if a built-in set version is not listed. No result moves.
- Design change (issue #281): the space between the gas cargo tanks and the hold end bulkhead is keyed to the shape of the tank ends, as the inland class rules (Pt 4 Ch 4 3.5.3) and ADN 9.3.1.11.3(a) say: 0.50 m for plane tank ends, 0.20 m for any other end. It was keyed to "pressure tank" (0.20 m) or "standard" (0.50 m), citing 3.6.3, a type C (liquid cargo) clause. Every tank OVCD models has domed heads, so no length, volume or verdict moves. The check message of every gas design changes ("domed tank ends, cl. 3.5.3" for "pressure tank, cl. 3.5.3 / 3.6.3"), so every gas results hash moves. The before/after table is under "Design changes" below. In the Python API, `GasSeparationInputs.is_pressure_tank` is kept but no longer read, and the new Rust field `plane_tank_ends` (default false) sets plane ends. The Rule `lr-inland/3.5.3/end-space` of `lr-inland-gas` follows the tank end shape and no longer links a finding. Fixture hashes re-stamped from `ovcd run` (two gas fixtures and three gas files of `tests/legacy`; the v1 file keeps its old hash, as its test expects a difference). No sourced test changed.
- App and web app (issue #276): the Rule check calculations tab shows the Rule Trace: one row per Rule per object, with Result, Rule (title, id, sets), Object, Value, Limit, Margin, Citation (with any finding or gap) and Status (verified, unverified, OVCD Check, method unverified). Failures come first, then warnings, passes, notes, not evaluated and not applicable. Tags differ by word and shape, not colour only; the governing Rule and the Rules that size the arrangement are marked. The verdict line names any blocking Rule that is not evaluated. The app computes nothing: every field comes from the trace.
- Rules engine (issue #272): two more built-in Rule Sets. `lr-inland-gas` 1.0.0 (draft) holds the three gas separation checks of the inland class rules as Rules, with the code's 1e-9 m tolerance: the opening setback (3.3.7, 3.3.11, unverified, finding #381), the cofferdam (3.6.13, unverified, finding #280) and the end-bulkhead space (3.5.3, unverified, finding #281). `ovcd` 1.0.0 holds every other check as an OVCD Check with an honest origin (physical, tool limit or method validity), never shown as a published rule. `is2008-a` gains a note that shows each loading condition's stability check. Every check of a result reports exactly one Rule (the table `CHECK_RULES`; the check list itself does not change, so no results hash moves). No result moves.
- Rules engine (issue #271): the built-in Rule Set `igc-2g` 1.0.0, the IGC Code as amended for ship type 2G. It includes `is2008-a` and applies to Type C tanks only. Each gas Option's Rule Trace lists, per tank, the side and bottom distances (IGC 2.4.1.2 with d(Vc) of 2.4.1.1, less the insulation as 2.4.2 measures), the minimum wall thickness (4.23.2.1.2), the weld factor (4.23.2.1.3) and the allowable stress (4.23.3.1, `warn` for files that keep the old 285 MPa basis, #313), and damage survival (2.7) as not evaluated. Every Rule was verified by a second reader. Issue #283, step 1: the 1.0 m side clearance default of the design sweep no longer claims to be an IGC value; its basis now says it is a design default with no published source (the `clearance_basis` text of `design_grid`). No result moves.
- Rules engine (issue #270): the first built-in Rule Set, `is2008-a` 1.0.0, the IS Code 2008 Part A 2.2 criteria and the 2.3 weather criterion (not evaluated). Every gas Option's Rule Trace lists the six criteria per loading condition, with the same numbers and verdicts as the stability checks; a dry bulk Option lists them as not evaluated (its stability is first-order only). Six Rules and the weather Rule were verified by a second reader against MSC.267(85). The 0 to 40 and 30 to 40 degree areas link the down-flooding finding (#282). No result moves and no check changes.
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
- Design change (issue #374): the Type C tank wall takes the design liquid pressure on top of the design vapour pressure: Peq = Po + Pgd (IGC 4.13.2.4, 4.23.2.2), Pgd = a x Z x rho / 1.02e5 MPa (IGC 4.28.1.2), with Z the steel outer diameter and rho the largest cargo density at the design temperature. A new input, `tank_liquid_pressure`: `static` (a = 1, gravity only, the default, an estimate for inland and sheltered service: the maintainer's choice), `igc_4_28_2` (a = 1 + az, the IGC 4.28.2 guidance vertical acceleration for the North Atlantic, at the tank farthest from amidships) or `none` (the vapour pressure only, what the engine did before). On a 110 x 11.4 m hull the LNG wall goes from 9.5 to 10.5 mm (lightship +22 t), fully refrigerated propane 9.5 to 11.0 mm, fully pressurised propane 35.5 to 36.5 mm; cargo falls by under 0.1 %. A file saved before keeps `none` and its results. No sourced test changed; new sourced test `crates/ovcd-core/tests/liquid_pressure.rs`. The before/after table is under "Design changes" below.
- Design change (issue #375): the tank head type is an input, `tank_head_type` (`hemisphere`, `ellipsoidal_2to1`), with a Defaults Profile table by Cargo and carriage condition (the maintainer's choice): the LPG Cargos take 2:1 semi-ellipsoidal heads (estimates: the published river LPG tank has dished ends with a toroidal connection, a published 22,000 m3 LPG/NH3 carrier tank torispherical heads; 2:1 is the engine's nearest head) and LNG keeps hemispherical heads (estimate: the old value). The head type reaches the wall, the volume, the steel and the drawings (2:1 ends are drawn as half-ellipses, R/2 long), so the head-area fix of #314 now reaches Designs. Where stability does not govern, LPG cargo rises: fully refrigerated propane on a 110 x 11.4 m hull 2,850 to 2,992 t, fully pressurised propane on 90 x 14 m 2,260 to 2,450 t. Where it governs, cargo falls: fully pressurised propane on 110 x 11.4 m moves from a 9.65 m to an 8.64 m tank and from 2,165 t to 1,821 t, because the 2:1 heads put more cargo in a tank of the same length, the 9.65 m tank then fails laden stability, and the search takes the smaller one. LNG designs do not move. A file saved before keeps hemispherical heads and its results. No sourced test changed. The before/after table is under "Design changes" below.
- Format version 3 and the stern inputs (issue #347, spec 0007 stern 1): 18 inputs for the stern and propulsion of a self-propelled hull, in a new group "Stern and propulsion" (`stern_form`, `stern_run_length_m`, `stern_transom_height_m`, `tunnel_crown_height_m`, `propulsor_type`, `propulsor_count`, `propeller_diameter_m`, `power_source`, `installed_power_kw`, `design_speed_kn`, `installed_power_margin`, `bow_thruster`, `bow_thruster_power_kw`, `battery_energy_kwh`, `battery_unit_energy_kwh`, `wheelhouse_type`, `sightline_method`, `stern_block_mass_included`). Inputs only: the engine reads none of them yet; later spec 0007 tickets give them effect. A file of version 1 or 2 gets the values that reproduce today's stern block, with one warning per Option, and keeps its results; a new Option takes the Defaults Profile values (by Cargo phase: gas a transom stern, ducted azimuth thrusters and gas-fuelled gensets, dry bulk a tunnel stern, shafts in nozzles and diesel-mechanical), each `sourced` or `estimate` in the profile and the schema (`default_status`). `thruster_count` and `propulsive_efficiency` now apply only with an `unspecified` propulsor (`applies_when` in the schema). The one result that moves: a new Option's wheelhouse eye height is 1.65 m (ES-TRIN 7.02; was 1.50 m), so the sightline check of every new self-propelled Design reads 0.15 m higher (104 of 177 case designs, the sightline message only; no feasibility changes). Saving writes format version 3. The stern block API takes `power_source` as the new name of `propulsion`. No sourced test changed. The before/after table is under "Design changes" below.
- Design change (issue #348, spec 0007 stern 2): the stern block of a self-propelled hull has mass and centres, at the rates of spec 0007 (Hekkenberg 2013, Appendix B): ducted azimuth units 1.5 x D^3 t (D the thruster envelope / 1.35), batteries 6 kg/kWh, accommodation 0.17 t per m3 of unit, wheelhouse 8 t and its column 5 t, gas gensets (the published engine mass plus an alternator at 9 kg/kVA less the engine's 4.9 kg/kW), small items 1 kg/kW and piping and cabling 3 t + 2.5 kg/kW. Until the installed power is sized (#353) the per-kW items take a typed installed power, else the installed genset power, else none. The items enter the lightship and the KG of the gas and dry branches when `stern_block_mass_included` is true (a new Option), and the results show the breakdown (`stern_block_mass`); the stability limitation "the stern block's mass is not in the gas lightship" goes. On a 110 x 11.4 m hull today's battery-electric block weighs 107.4 t (three thrusters 32.1 t, one battery unit 30 t, two accommodation units 29.3 t, wheelhouse and column 13 t, piping 3 t): the lightship of every new self-propelled Design rises by about that much and its draught follows. One case changes feasibility: fully refrigerated propane on 110 x 11.4 m is now feasible with a 9.04 m tank (2,414 t), as the low block lowers the laden KG. A file saved before keeps the switch off and its results. No sourced test changed; new sourced test `crates/ovcd-core/tests/stern_block_mass.rs`. The before/after table is under "Design changes" below.
- Design change (issue #349, spec 0007 stern 3): trim for each loading condition. Every lightship item and the cargo get an LCG (the hull steel and the piping at the middle of the length, the tanks and the cargo at the middle of their zone, the outfitting in the stern zone, the stern block items at their zones). `hydrostatics::trim` gives the trim and the aft and forward draughts from the LCG, the LCB and the MTC, as the legacy `calculate_trim` does; positive trim is bow deeper. Each gas stability condition, and a dry bulk hull laden and light, gains `trim_m`, `draught_aft_m` and `draught_fwd_m`, only when the stern block mass is in the lightship (a new self-propelled Option). No other number moves. On a 110 x 11.4 m hull: LNG laden 0.79 m by the bow, light 0.53 m by the stern; bauxite on a 4.0 m depth laden 2.03 m by the bow. The hull is still the raked box with no Stern Form cut (#350), so the laden trims are by the bow. A file saved before gets no new field and keeps its results. No sourced test changed. The before/after table is under "Design changes" below.
- Design change (issue #350, spec 0007 stern 4): the Stern Form of a self-propelled hull has a shape. A new module, `stern_form`, gives in closed form the volume each form removes below a waterline and its moment, the waterplane cut, the extra wetted surface of the tunnels, the outlines for the drawings and the largest propeller D_max. Transom: the stern rake of the raked box, run 1.0 x depth up to the design draught (estimates). Pram: a parabolic bottom rise, run 1.6 x depth (estimate). Tunnel: a flat bottom with one tunnel per propulsor, a circular segment of radius D with its crown 0.10 D above the tip (estimate), tapering over the Heuser run (Rotteveel 2019 sec. 2.3.1), twin shafts 0.45 B apart (parallel) or 0.28 B (converging). D_max is 0.80 x the design draught under a tunnel (estimate) and 0.65 x at a transom or pram stern (sourced: MAN); under a tunnel it is also capped so the tunnels meet but do not overlap and stay inside the beam (the maintainer's decision on this issue; 2.58 m in place of 2.80 m on a 110 x 11.4 m hull at 3.5 m). A typed D is used as typed. The block coefficient default is now none, the rule: the raked box with its bow rake and Stern Form at the design draught over L x B x T, or 0.85 with no Stern Form. The trim takes the Stern Form's LCB, and the resistance its extra wetted surface. The results gain `geometry.stern_form`, `stern_run_length_m`, `stern_transom_height_m`, `propeller_diameter_m` and `propeller_d_max_m`, and `stern_rake_length_m` for a transom or pram stern, which the drawings read (finding G15); the drawing data gains the stern outlines. 106 of 177 case designs move, every new self-propelled one. On a 110 x 11.4 m hull: a gas hull (transom) takes Cb 0.927 (was 0.85); fully pressurised propane then carries 2,257 t (was 1,821 t) in a 9.64 m tank (was 8.64 m), which now passes stability; LNG keeps its cargo but fails light-condition stability (angle of max GZ 24 degrees, under 25), so no arrangement passes and the closest is shown; bauxite on a 4.0 m depth (tunnel) takes Cb 0.906 and carries 3,532 t (was 3,273 t), laden trim 0.77 m by the bow (was 2.03 m). In the case set two designs turn infeasible, both on light-condition stability: gas on 90 x 14 m and one tank liquid pressure sweep. A barge and a file saved before keep 0.85 and their results. No sourced test changed; new sourced test `crates/ovcd-core/tests/stern_form.rs`. The before/after table is under "Design changes" below.
- Propulsor model and calibration (issue #351, spec 0007 stern 5): a new module, `propulsor`, is the actuator disc of spec 0007 per unit: thrust, advance speed, thrust loading, the ideal efficiency of the open and the ducted disc (the propeller carries 0.75 of the thrust in a duct, an estimate), eta_D = kappa x eta_i x 0.98 x eta_H with eta_H = 1, delivered power per speed, the bollard pull kappa_b x (2 rho A)^(1/3) x P_D^(2/3), and the blade load against the Veth limits (400 kW/m2 open, 500 kW/m2 ducted). With kappa_b 0.77 the bollard pull is the Wartsila open-propeller estimate within 0.5 %. New calibration cases, `calibration/propulsion_cases.yaml`: 11 tugs and pushers with a published bollard pull and propeller or nozzle diameter, each with its source file and SHA-256, and 9 records that cannot run yet (no diameter printed, or a pod). The band `bollard_pull_t` (+-10/15/20 %) is in `bands.yaml`. Five in-sample tugs fit kappa_b: 0.951 for shafts in nozzles, 0.975 for ducted azimuth units; the other types take 0.77 (estimate). Every case is inside its band but one held-out shallow-draught pusher, CDS 4115 (+31 %), an accepted miss: its rudders and tunnel cut the nozzle thrust. The ASD 2312 pulls more than the ASD 2811 at the same power, as published, both within 4 %. Two new inputs, held only with a specified Propulsor type: `propulsor_efficiency_factor` (kappa, 0.80 for every type, an estimate until #353 fits it) and `bollard_pull_factor` (kappa_b, the fit), by Propulsor type in the Defaults Profile. The engine does not use the model yet (#353), so no result moves.
- Design change (issue #352, spec 0007 stern 6): the machinery space and its mass follow the Power Source and the Propulsor. A new module, `machinery`, lays out the stern block below deck for a specified Propulsor (Hekkenberg 2013 sec. 4.2.4 as the method): the aft peak with the steering gear (1.5 m, estimate), the drive room (an azimuth unit's room) or the shaft run (2 D, estimate), the engine room (A 1.0 m + engine + gearbox 1.2 m, or 2.8 m with a PTI motor, + C 0.8 m) for diesel-mechanical and battery hybrid, the diesel genset room (N + 1 sets), the hybrid's battery room (70 kWh per m3, estimate) and the gas genset and GVU rooms of a gas-fuelled block. The engine is the first of the maker table that covers the power per engine (Cat 3512C, Cat 3516C, Cummins QSK60, Everllence 12V/16V/20V 175D), below 955 kW the Cat 3512C scaled by (P / 955)^(1/3) (estimate). The stern block is the longest of the run below deck, the deck items and the stern run of the Stern Form. The mass items of spec 0007 replace the thrusters, small items and piping of #348 for a specified Propulsor: engines 4.9 kg/kW, gearboxes 2 kg/kW, diesel gensets 9 kg/kVA, switchboards and converters, electric motors 6 kg/kW, PTI motors 8 kg/kWe, ducted or open azimuth units 1.5 x D^3, Voith Schneider 22 kg/kW, pump jets by their formula, shaft lines 1.0 t/m, two rudders per shaft and the steering gear, the bow thruster 5.4 kg/kW (0.11 kW per m3 of L x B x T when none is typed), the battery room, small items and piping. The new Option's Power Source now takes effect: a gas hull is gas-fuelled, with no battery unit. Until the installed power is sized (#353), an Option with no typed installed power gets no engine room, with a warning. The default propeller diameter is capped at the largest unit of the maker range, 4.2 m for an azimuth unit (Schottel SRP), the maintainer's decision on this issue. Machinery checks: the engines fit the beam and fit under the deck. 106 of 177 case designs move, every new self-propelled one; an unspecified Propulsor and a file saved before keep today's stern block and their results. On 110 x 11.4 x 6 m, LNG: stern block 19.9 to 21.4 m, its mass 107 to 275 t (two 3.7 m ducted azimuth units, 152 t), cargo 2,193 to 2,147 t, and now feasible (the heavy low block lowers the KG). On 60 x 10 m, gas cargo doubles (259 to 529 t): the block loses its battery unit and shortens from 34.9 to 23.4 m. Bauxite on 110 x 11.4 x 4 m: stern block 19.9 to 24.8 m, set by the tunnel run; on 88 x 16.3 x 6 m 19.9 to 35.4 m and cargo 5,792 to 4,960 t. No sourced test changed; new sourced test `crates/ovcd-core/tests/machinery.rs`. The before/after table is under "Design changes" below.
- Design change (issue #353, spec 0007 stern 7): installed power, transmission and service speed. A new module, `powering`, takes the resistance of each environment through the Propulsor (#351) to the delivered power, the shaft power (shaft line 0.99, azimuth unit 0.95) and the power the engines, gensets or battery give (gearbox 0.97; the electric chain loss 15 % at 1 MW to 9.4 % at 5 MW on log power, Wartsila; battery drivetrain 0.92). A new self-propelled Option with a specified Propulsor sizes its Installed Power for the Design Speed (9.7 kn) x the margin (1.20), on the highest power over deep water and the design channel, at the design draught, and rounds it up to the engine table where engines drive; a typed Installed Power is used as typed. The sized power places the engines of #352 below deck. The results gain a `powering` block: the Installed Power, the bollard pull, and per environment the service speed (the highest speed where the power needed is at most P_B / margin, within today's squat and channel caps), the power there, the diesel fuel (195 g/kWh) and the battery energy per trip. A new input, `trip_length_km` (battery Power Sources, none by default): with a trip length the battery energy follows the energy per trip x 1.25 / 0.8, and the battery units or the hybrid's battery room follow the energy; the cost of a self-propelled gas Option takes the same battery energy and the Installed Power, not a tug's 620 kWh and 230 kW. Calibration: two inland tankers that print both their power and their design speed (CDS 110 and CDS 135 Tanker, 18 km/h) are installed power cases in `calibration/propulsion_cases.yaml`, band `installed_power_kw` (+-15/20/30 %); the CDS 110 fits the efficiency factor kappa of shafts in nozzles, 0.447 (it was 0.80, an estimate), and the held-out CDS 135 comes out at -13.9 %. The app shows the Installed Power, the bollard pull, the service speed in deep water and in the channel, and the battery energy. 106 of 177 case designs move, every new self-propelled one; an unspecified Propulsor and a file saved before get no block and keep their results. On 110 x 11.4 x 4 m, bauxite: 876 kW for 9.7 kn. On 110 x 11.4 x 6 m, LNG: 623 kW; its lighter electrical plant (sized on 623 kW, not on the gensets) raises the laden KG, so the search takes a 9.32 m tank (was 10.32 m) and cargo falls from 2,147 to 1,742 t. No sourced test changed; new sourced test `crates/ovcd-core/tests/powering.rs`. The before/after table is under "Design changes" below.
- Engine fix (issue #414): the LCB of the raked box's stern rake was mirrored. The rake was integrated from x = 0 forward, although its deep end is at its forward end and it rises aft. `HullGeometry::LCB` now takes it from its forward end, and a prism-and-wedge hand case checks it (`crates/ovcd-core/tests/stern_rake_lcb.rs`). No Design reaches this branch: every hull on the `.vdesign` path has no stern rake (the Stern Forms of #350 have their own LCB), so no result moves. In the Python API, `HullGeometry.LCB` of a hull with a stern rake moves (the `hull.methods` answer of the 0.1.0 oracle).
- Design change (issue #354, spec 0007 stern 8): the wheelhouse sightline as a Blind Distance. A new module, `wheelhouse`, works the view ahead as ES-TRIN 7.02(2) states the rule: the ray from the eye over each obstruction forward of it (the deck units, the cargo or tank top at the forward end of the cargo zone, the deck at the bow) meets the waterline, trimmed where the trim is known (#349); the Blind Distance is the farthest such point ahead of the bow (on an even keel, the similar triangles of research note sec. 9.2). The lift needed is the smallest that keeps the Blind Distance within 250 m in the light condition (ES-TRIN 7.02(2): unladen). A new Option (`sightline_method: blind_distance`) lifts its wheelhouse that far, up to its maximum lift; a fixed wheelhouse (`wheelhouse_type: fixed`) does not lift. Its check keeps the name "wheelhouse sees over cargo", so the blocking Rule of #277 keeps the fit (and the design grid and `smallest_viable_loa`) to arrangements that meet 250 m (the maintainer's decision on #264). The results gain a `sightline` block (the Blind Distance light and laden, the lift needed and used, the raised eye and floor), and the navigating air draft carries the raised wheelhouse. The margin method and every file saved before are unchanged. 109 of 182 case designs move, every new self-propelled one. Many gas designs lose cargo: the light draught is shallow, so a tall tank needs more than the 6 m column to keep 250 m, and the fit takes a lower tank. On 110 x 11.4 x 6 m, LNG 1,742 to 1,376 t (tank 9.32 to 8.32 m), fully pressurised propane 2,213 to 1,403 t (9.64 to 7.64 m); on 90 x 14 x 7.7 m, fully refrigerated propane 2,670 to 1,829 t. Two wide-hull sweeps (110 x 24 m) turn infeasible. No sourced test changed; new sourced test `crates/ovcd-core/tests/wheelhouse.rs`. The before/after table is under "Design changes" below.
- Tug inputs (issue #357, spec 0007 tug 1): every barge Formation has a tug as inputs, in a new group "Tug". Two new Formation kinds, `atb` (one barge with a Notch and its ATB tug) and `towed_barge` (barges on a hawser), so the ATB and towed-barge Vessel Classes can be picked; until the Formation powering (#359) they compute as the barge alone. The inputs: `tug_included` (true for a new Option), `tug_preset`, the tug's length, beam and draught (none: the sizing rules of #359), its stern and propulsion inputs prefixed `tug_` (by Formation kind: a pusher a tunnel stern and shafts in nozzles, an ATB tug and a towing tug a transom and ducted azimuth units; diesel-mechanical), `formation_resistance_allowance` (ATB 1.05 from Yanakiev 2025 Table 3, others 1.00, estimates) and the Notch of an ATB barge. 18 Reference Vessel pushers, tugs and ATB tugs of the research note are tug presets (`ovcd_core::tug_presets`), with their power, size, propulsor and source; picking one fills the tug inputs marked `reference`, a new source marker. A file saved before reads `tug_included: false` and keeps its results; the fixtures were refilled with `ovcd fill`. Inputs only: no result moves.
- Hull Form inputs (issue #446, spec 0008 Vessel Model 4): format version 3 gains the hull block, in a new group "Hull Form". `hull_form` picks the Hull Form (`box`, `raked_box`, `v_bottom`, `chine`, `shipd`, `offsets`); its parameters are the bow rake length and rise, the bilge radius, the sheer aft and forward, the camber, the deadrise, the keel half-width, the chine half-breadth and height, `shipd_parameters` (an object of named numbers) and `hull_offsets` (a table of points). Each applies only to the Hull Forms that use it, and the schema gives the shape of the two object-valued inputs. Every Defaults Profile value is an estimate with its basis; `shipd_parameters` is none until the ShipD ticket (#452) brings the dataset rows. A file saved before gets `raked_box`, its own bow rake, a square bilge, no sheer and no camber, with one warning per Option, and keeps its results. A Hull Form other than `raked_box` is refused with "not built yet" until its generator ticket. The app shows the Hull Form group. Inputs only: no result moves.

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

### ticket/448-sections-hydrostatics (2026-10-22)

Between `main e53e611` and `this branch 155333b`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| calibration amaggi-box-2000 | `checks[10].message` | GM 6.02 m (fluid), max GZ 1.76 m at 25° | GM 6.49 m (fluid), max GZ 1.84 m at 25° | changed |
| calibration amaggi-box-2000 | `checks[11].message` | GM 15.42 m (fluid), max GZ 2.76 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | GM 15.82 m (fluid), max GZ 2.80 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | changed |
| calibration amaggi-box-2000 | `checks[5].message` | intact GM 1.38 m >= 0.15 m at the reported cargo 1665 t, draught 3.56 m (first-order box; cargo KG of the full hold; KG_light @0.5D) | intact GM 1.53 m >= 0.15 m at the reported cargo 1861 t, draught 3.56 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| calibration amaggi-box-2000 | `checks[7].message` | GM 1.38 m (fluid), max GZ 0.23 m at 13° — fails: Area 0–40° 0.074 < 0.09, Area 30–40° -0.006 < 0.03, GZ at ≥ 30° 0.047 < 0.2, Angle of max GZ 13.000 < 25.0 | GM 1.53 m (fluid), max GZ 0.26 m at 13° — fails: Area 30–40° 0.002 < 0.03, GZ at ≥ 30° 0.091 < 0.2, Angle of max GZ 13.000 < 25.0 | changed |
| calibration amaggi-box-2000 | `checks[8].message` | GM 1.38 m (fluid), max GZ 0.23 m at 13° — fails: Area 0–40° 0.074 < 0.09, Area 30–40° -0.006 < 0.03, GZ at ≥ 30° 0.047 < 0.2, Angle of max GZ 13.000 < 25.0 | GM 1.53 m (fluid), max GZ 0.26 m at 13° — fails: Area 30–40° 0.002 < 0.03, GZ at ≥ 30° 0.091 < 0.2, Angle of max GZ 13.000 < 25.0 | changed |
| calibration amaggi-box-2000 | `checks[9].message` | GM 6.02 m (fluid), max GZ 1.76 m at 25° | GM 6.49 m (fluid), max GZ 1.84 m at 25° | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].b_wl_m` | absent | 10.67 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].bm_l_m` | absent | 589.989963648 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].bm_t_m` | absent | 22.3580973134 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].cb` | absent | 0.993767697113 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].cm` | absent | 1 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].cp` | absent | 0.993767697113 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].cwp` | absent | 1 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].displacement_t` | absent | 248.168399864 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].draught_m` | absent | 0.427 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].kb_m` | absent | 0.213946313785 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].km_l_m` | absent | 590.203909962 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].km_t_m` | absent | 22.5720436272 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].l_wl_m` | absent | 54.8112 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].lcb_m` | absent | 27.235157051 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].lcf_m` | absent | 27.4056 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].mct_1cm_tm` | absent | 26.7129464807 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].midship_area_m2` | absent | 4.55609 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].tpc_t` | absent | 5.84835504 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].volume_m3` | absent | 248.168399864 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].waterplane_area_m2` | absent | 584.835504 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[0].wetted_surface_m2` | absent | 637.215301819 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[1].b_wl_m` | absent | 10.67 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[1].bm_l_m` | absent | 304.255931468 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[1].bm_t_m` | absent | 11.2478514497 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[1].cb` | absent | 0.987688847884 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[1].cm` | absent | 1 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[1].cp` | absent | 0.987688847884 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[1].cwp` | absent | 1 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[1].displacement_t` | absent | 499.449520416 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[1].draught_m` | absent | 0.854 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[1].kb_m` | absent | 0.428774128888 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[1].km_l_m` | absent | 304.684705597 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[1].km_t_m` | absent | 11.6766255786 | changed |
| calibration amaggi-box-2000 | `hydrostatic_table[1].l_wl_m` | absent | 55.4944 | changed |
| ... | 212916 more changed quantities | | | |

### ticket/332-liquid-conditions (2026-10-22)

Between `main 251daf5` and `this branch a927401`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep crude carries nothing (draught limit) | `checks[10].message` | absent | GM 22.73 m (fluid), max GZ 3.84 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | changed |
| sweep crude carries nothing (draught limit) | `checks[10].name` | absent | Stability — lightship, no ballast | changed |
| sweep crude carries nothing (draught limit) | `checks[10].ok` | absent | False | changed |
| sweep crude carries nothing (draught limit) | `checks[10].severity` | absent | warning | changed |
| sweep crude carries nothing (draught limit) | `checks[11].message` | absent | GM 3.32 m (fluid), max GZ 0.74 m at 20° — fails: Angle of max GZ 20.000 < 25.0 | changed |
| sweep crude carries nothing (draught limit) | `checks[11].name` | absent | Stability — slack cargo tanks, worst case | changed |
| sweep crude carries nothing (draught limit) | `checks[11].ok` | absent | False | changed |
| sweep crude carries nothing (draught limit) | `checks[11].severity` | absent | warning | changed |
| sweep crude carries nothing (draught limit) | `checks[6].message` | absent | GM 22.73 m (fluid), max GZ 3.84 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | changed |
| sweep crude carries nothing (draught limit) | `checks[6].name` | absent | Stability — full load, departure | changed |
| sweep crude carries nothing (draught limit) | `checks[6].ok` | absent | False | changed |
| sweep crude carries nothing (draught limit) | `checks[6].severity` | absent | warning | changed |
| sweep crude carries nothing (draught limit) | `checks[7].message` | absent | GM 22.73 m (fluid), max GZ 3.84 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | changed |
| sweep crude carries nothing (draught limit) | `checks[7].name` | absent | Stability — full load, arrival | changed |
| sweep crude carries nothing (draught limit) | `checks[7].ok` | absent | False | changed |
| sweep crude carries nothing (draught limit) | `checks[7].severity` | absent | warning | changed |
| sweep crude carries nothing (draught limit) | `checks[8].message` | absent | GM 11.21 m (fluid), max GZ 3.85 m at 34° | changed |
| sweep crude carries nothing (draught limit) | `checks[8].name` | absent | Stability — ballast, departure | changed |
| sweep crude carries nothing (draught limit) | `checks[8].ok` | absent | True | changed |
| sweep crude carries nothing (draught limit) | `checks[8].severity` | absent | warning | changed |
| sweep crude carries nothing (draught limit) | `checks[9].message` | absent | GM 11.21 m (fluid), max GZ 3.85 m at 34° | changed |
| sweep crude carries nothing (draught limit) | `checks[9].name` | absent | Stability — ballast, arrival | changed |
| sweep crude carries nothing (draught limit) | `checks[9].ok` | absent | True | changed |
| sweep crude carries nothing (draught limit) | `checks[9].severity` | absent | warning | changed |
| sweep crude carries nothing (draught limit) | `loading_conditions.ballast_capacity_m3` | absent | 1783.44 | changed |
| sweep crude carries nothing (draught limit) | `loading_conditions.ballast_capacity_t` | absent | 1783.44 | changed |
| sweep crude carries nothing (draught limit) | `loading_conditions.ballast_tank_count` | absent | 6 | changed |
| sweep crude carries nothing (draught limit) | `loading_conditions.ballast_tanks[0].capacity_m3` | absent | 385.14 | changed |
| sweep crude carries nothing (draught limit) | `loading_conditions.ballast_tanks[0].capacity_t` | absent | 385.14 | changed |
| sweep crude carries nothing (draught limit) | `loading_conditions.ballast_tanks[0].kind` | absent | double_bottom_ballast | changed |
| sweep crude carries nothing (draught limit) | `loading_conditions.ballast_tanks[0].lcg_m` | absent | 42.9 | changed |
| sweep crude carries nothing (draught limit) | `loading_conditions.ballast_tanks[0].name` | absent | DB1P | changed |
| sweep crude carries nothing (draught limit) | `loading_conditions.ballast_tanks[0].side` | absent | port | changed |
| sweep crude carries nothing (draught limit) | `loading_conditions.ballast_tanks[0].tcg_m` | absent | -3.5 | changed |
| sweep crude carries nothing (draught limit) | `loading_conditions.ballast_tanks[0].vcg_m` | absent | 0.35 | changed |
| sweep crude carries nothing (draught limit) | `loading_conditions.ballast_tanks[1].capacity_m3` | absent | 385.14 | changed |
| sweep crude carries nothing (draught limit) | `loading_conditions.ballast_tanks[1].capacity_t` | absent | 385.14 | changed |
| sweep crude carries nothing (draught limit) | `loading_conditions.ballast_tanks[1].kind` | absent | double_bottom_ballast | changed |
| sweep crude carries nothing (draught limit) | `loading_conditions.ballast_tanks[1].lcg_m` | absent | 42.9 | changed |
| sweep crude carries nothing (draught limit) | `loading_conditions.ballast_tanks[1].name` | absent | DB1S | changed |
| ... | 40834 more changed quantities | | | |

### ticket/280-aft-end-space (2026-10-22)

Between `main 251daf5` and `this branch 052469e`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `checks[14].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[14].name` | cofferdam min width | aft end bulkhead space | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[15].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[15].name` | cofferdam min width | aft end bulkhead space | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[14].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[14].name` | cofferdam min width | aft end bulkhead space | changed |
| sweep default depth gas 90x14 | `checks[14].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep default depth gas 90x14 | `checks[14].name` | cofferdam min width | aft end bulkhead space | changed |
| sweep gas IGC_2G 60x10 | `checks[14].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas IGC_2G 60x10 | `checks[14].name` | cofferdam min width | aft end bulkhead space | changed |
| sweep gas IGC_2G 90x14 | `checks[14].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas IGC_2G 90x14 | `checks[14].name` | cofferdam min width | aft end bulkhead space | changed |
| sweep gas IGC_2G_EXACT 60x10 | `checks[14].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas IGC_2G_EXACT 60x10 | `checks[14].name` | cofferdam min width | aft end bulkhead space | changed |
| sweep gas IGC_2G_EXACT 90x14 | `checks[14].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas IGC_2G_EXACT 90x14 | `checks[14].name` | cofferdam min width | aft end bulkhead space | changed |
| sweep gas cargo LNG | `checks[14].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas cargo LNG | `checks[14].name` | cofferdam min width | aft end bulkhead space | changed |
| sweep gas cargo LNG pushed | `checks[6].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas cargo LNG pushed | `checks[6].name` | cofferdam min width | aft end bulkhead space | changed |
| sweep gas cargo LNG_ethane | `checks[14].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas cargo LNG_ethane | `checks[14].name` | cofferdam min width | aft end bulkhead space | changed |
| sweep gas cargo LNG_ethane pushed | `checks[6].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas cargo LNG_ethane pushed | `checks[6].name` | cofferdam min width | aft end bulkhead space | changed |
| sweep gas cargo LPG_butane | `checks[14].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas cargo LPG_butane | `checks[14].name` | cofferdam min width | aft end bulkhead space | changed |
| sweep gas cargo LPG_butane pushed | `checks[5].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas cargo LPG_butane pushed | `checks[5].name` | cofferdam min width | aft end bulkhead space | changed |
| sweep gas cargo LPG_mix | `checks[14].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas cargo LPG_mix | `checks[14].name` | cofferdam min width | aft end bulkhead space | changed |
| sweep gas cargo LPG_mix pushed | `checks[5].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas cargo LPG_mix pushed | `checks[5].name` | cofferdam min width | aft end bulkhead space | changed |
| sweep gas cargo LPG_propane | `checks[14].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas cargo LPG_propane | `checks[14].name` | cofferdam min width | aft end bulkhead space | changed |
| sweep gas cargo LPG_propane pushed | `checks[5].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas cargo LPG_propane pushed | `checks[5].name` | cofferdam min width | aft end bulkhead space | changed |
| sweep gas carriage LPG_butane fully_pressurised | `checks[14].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas carriage LPG_butane fully_pressurised | `checks[14].name` | cofferdam min width | aft end bulkhead space | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[14].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft end-bulkhead space 0.60 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[14].name` | cofferdam min width | aft end bulkhead space | changed |
| ... | 158 more changed quantities | | | |

### ticket/331-integral-tanks (2026-10-22)

Between `main 6d46b83` and `this branch 820f768`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep crude carries nothing (draught limit) | `result` | fill: warning: Option 'sweep crude carries nothing (draught limit)': `price_base_date` was not stored in the file; wrote in the Defaults Profile value none, marked default \| warning: Option 'sweep crude carries nothing (draught limit)': `service_area` was not stored in the file; wrote in the Defaults Profile value river-only, marked default \| ovcd fill: 1 error(s); nothing written | computed | changed |
| sweep crude channel 90x14 | `result` | fill: warning: Option 'sweep crude channel 90x14': `price_base_date` was not stored in the file; wrote in the Defaults Profile value none, marked default \| warning: Option 'sweep crude channel 90x14': `service_area` was not stored in the file; wrote in the Defaults Profile value river-only, marked default \| ovcd fill: 1 error(s); nothing written | computed | changed |
| sweep crude coastal service 90x14 | `result` | fill: warning: Option 'sweep crude coastal service 90x14': `currency` was not stored in the file; wrote in the Defaults Profile value USD, marked default \| warning: Option 'sweep crude coastal service 90x14': `price_base_date` was not stored in the file; wrote in the Defaults Profile value none, marked default \| ovcd fill: 6 error(s); nothing written | computed | changed |
| sweep crude pushed 60x10 | `result` | fill: warning: Option 'sweep crude pushed 60x10': `price_base_date` was not stored in the file; wrote in the Defaults Profile value none, marked default \| warning: Option 'sweep crude pushed 60x10': `service_area` was not stored in the file; wrote in the Defaults Profile value river-only, marked default \| ovcd fill: 1 error(s); nothing written | computed | changed |
| sweep crude pushed 90x14 | `result` | fill: warning: Option 'sweep crude pushed 90x14': `price_base_date` was not stored in the file; wrote in the Defaults Profile value none, marked default \| warning: Option 'sweep crude pushed 90x14': `service_area` was not stored in the file; wrote in the Defaults Profile value river-only, marked default \| ovcd fill: 1 error(s); nothing written | computed | changed |
| sweep crude self-propelled 60x10 | `result` | fill: warning: Option 'sweep crude self-propelled 60x10': `price_base_date` was not stored in the file; wrote in the Defaults Profile value none, marked default \| warning: Option 'sweep crude self-propelled 60x10': `service_area` was not stored in the file; wrote in the Defaults Profile value river-only, marked default \| ovcd fill: 6 error(s); nothing written | computed | changed |
| sweep crude self-propelled 90x14 | `result` | fill: warning: Option 'sweep crude self-propelled 90x14': `price_base_date` was not stored in the file; wrote in the Defaults Profile value none, marked default \| warning: Option 'sweep crude self-propelled 90x14': `service_area` was not stored in the file; wrote in the Defaults Profile value river-only, marked default \| ovcd fill: 6 error(s); nothing written | computed | changed |
| sweep default depth liquid 90x14 | `result` | fill: warning: Option 'sweep default depth liquid 90x14': `price_base_date` was not stored in the file; wrote in the Defaults Profile value none, marked default \| warning: Option 'sweep default depth liquid 90x14': `service_area` was not stored in the file; wrote in the Defaults Profile value river-only, marked default \| ovcd fill: 6 error(s); nothing written | computed | changed |
| sweep optional centreline_bulkhead | `result` | fill: warning: Option 'sweep optional centreline_bulkhead': `price_base_date` was not stored in the file; wrote in the Defaults Profile value none, marked default \| warning: Option 'sweep optional centreline_bulkhead': `service_area` was not stored in the file; wrote in the Defaults Profile value river-only, marked default \| ovcd fill: 1 error(s); nothing written | computed | changed |
| sweep optional cofferdam_m | `result` | fill: warning: Option 'sweep optional cofferdam_m': `price_base_date` was not stored in the file; wrote in the Defaults Profile value none, marked default \| warning: Option 'sweep optional cofferdam_m': `service_area` was not stored in the file; wrote in the Defaults Profile value river-only, marked default \| ovcd fill: 1 error(s); nothing written | computed | changed |
| sweep optional double_bottom_m | `result` | fill: warning: Option 'sweep optional double_bottom_m': `price_base_date` was not stored in the file; wrote in the Defaults Profile value none, marked default \| warning: Option 'sweep optional double_bottom_m': `service_area` was not stored in the file; wrote in the Defaults Profile value river-only, marked default \| ovcd fill: 1 error(s); nothing written | computed | changed |
| sweep optional double_side_m | `result` | fill: warning: Option 'sweep optional double_side_m': `price_base_date` was not stored in the file; wrote in the Defaults Profile value none, marked default \| warning: Option 'sweep optional double_side_m': `service_area` was not stored in the file; wrote in the Defaults Profile value river-only, marked default \| ovcd fill: 1 error(s); nothing written | computed | changed |
| sweep optional heating_kw_per_m3 | `result` | fill: warning: Option 'sweep optional heating_kw_per_m3': `price_base_date` was not stored in the file; wrote in the Defaults Profile value none, marked default \| warning: Option 'sweep optional heating_kw_per_m3': `service_area` was not stored in the file; wrote in the Defaults Profile value river-only, marked default \| ovcd fill: 1 error(s); nothing written | computed | changed |
| sweep optional liquid_fill_ratio | `result` | fill: warning: Option 'sweep optional liquid_fill_ratio': `price_base_date` was not stored in the file; wrote in the Defaults Profile value none, marked default \| warning: Option 'sweep optional liquid_fill_ratio': `service_area` was not stored in the file; wrote in the Defaults Profile value river-only, marked default \| ovcd fill: 1 error(s); nothing written | computed | changed |
| sweep optional liquid_tanks_in_line | `result` | fill: warning: Option 'sweep optional liquid_tanks_in_line': `price_base_date` was not stored in the file; wrote in the Defaults Profile value none, marked default \| warning: Option 'sweep optional liquid_tanks_in_line': `service_area` was not stored in the file; wrote in the Defaults Profile value river-only, marked default \| ovcd fill: 1 error(s); nothing written | computed | changed |
| sweep optional pump_arrangement | `result` | fill: warning: Option 'sweep optional pump_arrangement': `price_base_date` was not stored in the file; wrote in the Defaults Profile value none, marked default \| warning: Option 'sweep optional pump_arrangement': `service_area` was not stored in the file; wrote in the Defaults Profile value river-only, marked default \| ovcd fill: 1 error(s); nothing written | computed | changed |
| sweep optional pump_room_length_m | `result` | fill: warning: Option 'sweep optional pump_room_length_m': `price_base_date` was not stored in the file; wrote in the Defaults Profile value none, marked default \| warning: Option 'sweep optional pump_room_length_m': `service_area` was not stored in the file; wrote in the Defaults Profile value river-only, marked default \| ovcd fill: 1 error(s); nothing written | computed | changed |
| sweep optional slop_share | `result` | fill: warning: Option 'sweep optional slop_share': `price_base_date` was not stored in the file; wrote in the Defaults Profile value none, marked default \| warning: Option 'sweep optional slop_share': `service_area` was not stored in the file; wrote in the Defaults Profile value river-only, marked default \| ovcd fill: 1 error(s); nothing written | computed | changed |

### ticket/420-run-validity-checks-2 (2026-10-21)

Between `main 5eb6189` and `this branch a109891`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| calibration amaggi-box-2000 | `checks[10].message` | GM 15.42 m (fluid), max GZ 2.76 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | GM 6.02 m (fluid), max GZ 1.76 m at 25° | changed |
| calibration amaggi-box-2000 | `checks[10].name` | Stability — lightship, no ballast | Stability — ballast, arrival | changed |
| calibration amaggi-box-2000 | `checks[10].ok` | False | True | changed |
| calibration amaggi-box-2000 | `checks[11].message` | absent | GM 15.42 m (fluid), max GZ 2.76 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | changed |
| calibration amaggi-box-2000 | `checks[11].name` | absent | Stability — lightship, no ballast | changed |
| calibration amaggi-box-2000 | `checks[11].ok` | absent | False | changed |
| calibration amaggi-box-2000 | `checks[11].severity` | absent | warning | changed |
| calibration amaggi-box-2000 | `checks[6].message` | GM 1.38 m (fluid), max GZ 0.23 m at 13° — fails: Area 0–40° 0.074 < 0.09, Area 30–40° -0.006 < 0.03, GZ at ≥ 30° 0.047 < 0.2, Angle of max GZ 13.000 < 25.0 | Residuary Cr/Cf fit, Lackenby shallow-water, blockage and 1.02 form factor are uncalibrated first-order estimates (RS1) — ITTC-1957 friction is standard. | changed |
| calibration amaggi-box-2000 | `checks[6].name` | Stability — full load, departure | coefficients are first order | changed |
| calibration amaggi-box-2000 | `checks[6].ok` | False | True | changed |
| calibration amaggi-box-2000 | `checks[7].name` | Stability — full load, arrival | Stability — full load, departure | changed |
| calibration amaggi-box-2000 | `checks[8].message` | GM 6.02 m (fluid), max GZ 1.76 m at 25° | GM 1.38 m (fluid), max GZ 0.23 m at 13° — fails: Area 0–40° 0.074 < 0.09, Area 30–40° -0.006 < 0.03, GZ at ≥ 30° 0.047 < 0.2, Angle of max GZ 13.000 < 25.0 | changed |
| calibration amaggi-box-2000 | `checks[8].name` | Stability — ballast, departure | Stability — full load, arrival | changed |
| calibration amaggi-box-2000 | `checks[8].ok` | True | False | changed |
| calibration amaggi-box-2000 | `checks[9].name` | Stability — ballast, arrival | Stability — ballast, departure | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[10].message` | GM 8.69 m (fluid), max GZ 1.76 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | GM 3.71 m (fluid), max GZ 1.14 m at 25° | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[10].name` | Stability — lightship, no ballast | Stability — ballast, arrival | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[10].ok` | False | True | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[11].message` | absent | GM 8.69 m (fluid), max GZ 1.76 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[11].name` | absent | Stability — lightship, no ballast | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[11].ok` | absent | False | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[11].severity` | absent | warning | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[6].message` | GM 0.40 m (fluid), max GZ 0.12 m at 17° — fails: Area 0–30° 0.038 < 0.055, Area 0–40° 0.031 < 0.09, Area 30–40° -0.007 < 0.03, GZ at ≥ 30° 0.024 < 0.2, Angle of max GZ 17.000 < 25.0 | Residuary Cr/Cf fit, Lackenby shallow-water, blockage and 1.02 form factor are uncalibrated first-order estimates (RS1) — ITTC-1957 friction is standard. | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[6].name` | Stability — full load, departure | coefficients are first order | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[6].ok` | False | True | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[7].name` | Stability — full load, arrival | Stability — full load, departure | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[8].message` | GM 3.71 m (fluid), max GZ 1.14 m at 25° | GM 0.40 m (fluid), max GZ 0.12 m at 17° — fails: Area 0–30° 0.038 < 0.055, Area 0–40° 0.031 < 0.09, Area 30–40° -0.007 < 0.03, GZ at ≥ 30° 0.024 < 0.2, Angle of max GZ 17.000 < 25.0 | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[8].name` | Stability — ballast, departure | Stability — full load, arrival | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[8].ok` | True | False | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[9].name` | Stability — ballast, arrival | Stability — ballast, departure | changed |
| calibration canal-box-200x35x12-cby222 | `checks[10].message` | GM 18.57 m (fluid), max GZ 2.97 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | GM 6.28 m (fluid), max GZ 1.84 m at 26° | changed |
| calibration canal-box-200x35x12-cby222 | `checks[10].name` | Stability — lightship, no ballast | Stability — ballast, arrival | changed |
| calibration canal-box-200x35x12-cby222 | `checks[10].ok` | False | True | changed |
| calibration canal-box-200x35x12-cby222 | `checks[11].message` | absent | GM 18.57 m (fluid), max GZ 2.97 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | changed |
| calibration canal-box-200x35x12-cby222 | `checks[11].name` | absent | Stability — lightship, no ballast | changed |
| calibration canal-box-200x35x12-cby222 | `checks[11].ok` | absent | False | changed |
| calibration canal-box-200x35x12-cby222 | `checks[11].severity` | absent | warning | changed |
| calibration canal-box-200x35x12-cby222 | `checks[6].message` | GM 2.09 m (fluid), max GZ 0.49 m at 19° — fails: Angle of max GZ 19.000 < 25.0 | Residuary Cr/Cf fit, Lackenby shallow-water, blockage and 1.02 form factor are uncalibrated first-order estimates (RS1) — ITTC-1957 friction is standard. | changed |
| calibration canal-box-200x35x12-cby222 | `checks[6].name` | Stability — full load, departure | coefficients are first order | changed |
| calibration canal-box-200x35x12-cby222 | `checks[6].ok` | False | True | changed |
| ... | 2190 more changed quantities | | | |

### ticket/419-severity-everywhere (2026-10-21)

Between `main 66589b9` and `this branch 353bef4`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| calibration amaggi-box-2000 | `checks[0].severity` | absent | error | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[0].severity` | absent | error | changed |
| calibration canal-box-200x35x12-cby222 | `checks[0].severity` | absent | error | changed |
| calibration canal-box-200x35x12-imt201 | `checks[0].severity` | absent | error | changed |
| calibration canal-box-200x35x12-imt205 | `checks[0].severity` | absent | error | changed |
| calibration canal-box-200x35x12-pb2012 | `checks[0].severity` | absent | error | changed |
| calibration canal-box-200x35x12-rci250 | `checks[0].severity` | absent | error | changed |
| calibration canal-box-200x35x13-cby252 | `checks[0].severity` | absent | error | changed |
| calibration canal-box-200x35x13-cby400 | `checks[0].severity` | absent | error | changed |
| calibration canal-box-200x35x13-cby433 | `checks[0].severity` | absent | error | changed |
| calibration canal-box-200x35x13-pb2352 | `checks[0].severity` | absent | error | changed |
| calibration cargill-amazon-jumbo | `checks[0].severity` | absent | error | changed |
| calibration danube-barge-77 | `checks[0].severity` | absent | error | changed |
| calibration dsb-holdbarge-1997-76.5 | `checks[0].severity` | absent | error | changed |
| calibration europa-iib-1635ou | `checks[0].severity` | absent | error | changed |
| calibration europa-iib-rdb06 | `checks[0].severity` | absent | error | changed |
| calibration hidrovia-lhg-3200 | `checks[0].severity` | absent | error | changed |
| calibration jeffboat-am2100 | `checks[0].severity` | absent | error | changed |
| calibration mcdonough-195x35x12 | `checks[0].severity` | absent | error | changed |
| calibration meb-dcb23-ptich | `checks[0].severity` | absent | error | changed |
| calibration meb-dcb24-pola | `checks[0].severity` | absent | error | changed |
| calibration meb-rdb12-samarskaya | `checks[0].severity` | absent | error | changed |
| calibration meb-rdb20-belmax9 | `checks[0].severity` | absent | error | changed |
| calibration para-mississippi-2200 | `checks[0].severity` | absent | error | changed |
| calibration stretch-hopper-light | `checks[0].severity` | absent | error | changed |
| calibration taf-danube-2008 | `checks[0].severity` | absent | error | changed |
| calibration taf-mississippi-2011 | `checks[0].severity` | absent | error | changed |
| calibration taf-parana-2013 | `checks[0].severity` | absent | error | changed |
| calibration taf-rhin-1980-covered | `checks[0].severity` | absent | error | changed |
| calibration taf-rhin-2000 | `checks[0].severity` | absent | error | changed |
| calibration taf-rhin-2001 | `checks[0].severity` | absent | error | changed |
| calibration taf-rhin-2008-covered | `checks[0].severity` | absent | error | changed |
| calibration taf-rhin-2019-covered | `checks[0].severity` | absent | error | changed |
| calibration taf-seine-1973 | `checks[0].severity` | absent | error | changed |
| calibration taf-seine-1981 | `checks[0].severity` | absent | error | changed |
| calibration taf-seine-1993 | `checks[0].severity` | absent | error | changed |
| calibration taf-seine-1996 | `checks[0].severity` | absent | error | changed |
| calibration taf-seine-1997 | `checks[0].severity` | absent | error | changed |
| calibration taf-seine-2000 | `checks[0].severity` | absent | error | changed |
| calibration taf-seine-2008 | `checks[0].severity` | absent | error | changed |
| ... | 162 more changed quantities | | | |

### ticket/411-head-insulation (2026-10-21)

Between `main e8e5589` and `this branch a4a089f`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep gas carriage LPG_butane fully_refrigerated | `checks[21].message` | tank length 37.25 m / inner diameter 7.63 m = 4.89 <= 7.0 | tank length 37.25 m / inner diameter 7.63 m = 4.88 <= 7.0 | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[22].message` | freeboard 3.20 m at the carried draught 2.80 m >= required 0.3 m | freeboard 3.21 m at the carried draught 2.79 m >= required 0.3 m | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[23].message` | GM 1.43 m (fluid), max GZ 1.08 m at 36° | GM 1.44 m (fluid), max GZ 1.08 m at 36° | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[24].message` | GM 1.42 m (fluid), max GZ 1.08 m at 36° | GM 1.43 m (fluid), max GZ 1.08 m at 36° | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[26].message` | GM 5.73 m (fluid), max GZ 1.81 m at 27° | GM 5.74 m (fluid), max GZ 1.81 m at 27° | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[3].message` | freeboard 3.20 m >= required 0.3 m | freeboard 3.21 m >= required 0.3 m | changed |
| sweep gas carriage LPG_butane semi_pressurised | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.93 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.94 m of 6.00 m | changed |
| sweep gas carriage LPG_butane semi_pressurised | `checks[22].message` | freeboard 3.00 m at the carried draught 3.00 m >= required 0.3 m | freeboard 3.01 m at the carried draught 2.99 m >= required 0.3 m | changed |
| sweep gas carriage LPG_butane semi_pressurised | `checks[23].message` | GM 1.22 m (fluid), max GZ 0.95 m at 36° | GM 1.23 m (fluid), max GZ 0.96 m at 36° | changed |
| sweep gas carriage LPG_butane semi_pressurised | `checks[24].message` | GM 1.21 m (fluid), max GZ 0.95 m at 36° | GM 1.22 m (fluid), max GZ 0.95 m at 36° | changed |
| sweep gas carriage LPG_butane semi_pressurised | `checks[25].message` | GM 4.77 m (fluid), max GZ 1.57 m at 26° | GM 4.78 m (fluid), max GZ 1.57 m at 26° | changed |
| sweep gas carriage LPG_butane semi_pressurised | `checks[26].message` | GM 4.81 m (fluid), max GZ 1.57 m at 26° | GM 4.82 m (fluid), max GZ 1.57 m at 26° | changed |
| sweep gas carriage LPG_butane semi_pressurised | `checks[3].message` | freeboard 3.00 m >= required 0.3 m | freeboard 3.01 m >= required 0.3 m | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[21].message` | tank length 37.25 m / inner diameter 7.63 m = 4.89 <= 7.0 | tank length 37.25 m / inner diameter 7.63 m = 4.88 <= 7.0 | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[22].message` | freeboard 3.21 m at the carried draught 2.79 m >= required 0.3 m | freeboard 3.22 m at the carried draught 2.78 m >= required 0.3 m | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[23].message` | GM 1.44 m (fluid), max GZ 1.09 m at 36° | GM 1.45 m (fluid), max GZ 1.09 m at 36° | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[24].message` | GM 1.44 m (fluid), max GZ 1.08 m at 36° | GM 1.45 m (fluid), max GZ 1.09 m at 36° | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[26].message` | GM 5.73 m (fluid), max GZ 1.81 m at 27° | GM 5.74 m (fluid), max GZ 1.81 m at 27° | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[3].message` | freeboard 3.21 m >= required 0.3 m | freeboard 3.22 m >= required 0.3 m | changed |
| sweep gas carriage LPG_mix semi_pressurised | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.93 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.94 m of 6.00 m | changed |
| sweep gas carriage LPG_mix semi_pressurised | `checks[22].message` | freeboard 3.12 m at the carried draught 2.88 m >= required 0.3 m | freeboard 3.13 m at the carried draught 2.87 m >= required 0.3 m | changed |
| sweep gas carriage LPG_mix semi_pressurised | `checks[23].message` | GM 1.30 m (fluid), max GZ 1.01 m at 36° | GM 1.31 m (fluid), max GZ 1.01 m at 36° | changed |
| sweep gas carriage LPG_mix semi_pressurised | `checks[24].message` | GM 1.30 m (fluid), max GZ 1.00 m at 36° | GM 1.31 m (fluid), max GZ 1.01 m at 36° | changed |
| sweep gas carriage LPG_mix semi_pressurised | `checks[25].message` | GM 4.77 m (fluid), max GZ 1.57 m at 26° | GM 4.78 m (fluid), max GZ 1.57 m at 26° | changed |
| sweep gas carriage LPG_mix semi_pressurised | `checks[26].message` | GM 4.81 m (fluid), max GZ 1.57 m at 26° | GM 4.82 m (fluid), max GZ 1.57 m at 26° | changed |
| sweep gas carriage LPG_mix semi_pressurised | `checks[3].message` | freeboard 3.12 m >= required 0.3 m | freeboard 3.13 m >= required 0.3 m | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[21].message` | tank length 37.25 m / inner diameter 7.63 m = 4.89 <= 7.0 | tank length 37.25 m / inner diameter 7.63 m = 4.88 <= 7.0 | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[23].message` | GM 1.48 m (fluid), max GZ 1.10 m at 36° | GM 1.49 m (fluid), max GZ 1.11 m at 36° | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[26].message` | GM 5.73 m (fluid), max GZ 1.81 m at 27° | GM 5.74 m (fluid), max GZ 1.81 m at 27° | changed |
| sweep gas carriage LPG_propane semi_pressurised | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.93 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.94 m of 6.00 m | changed |
| sweep gas carriage LPG_propane semi_pressurised | `checks[22].message` | freeboard 3.19 m at the carried draught 2.81 m >= required 0.3 m | freeboard 3.20 m at the carried draught 2.80 m >= required 0.3 m | changed |
| sweep gas carriage LPG_propane semi_pressurised | `checks[23].message` | GM 1.36 m (fluid), max GZ 1.04 m at 36° | GM 1.37 m (fluid), max GZ 1.04 m at 36° | changed |
| sweep gas carriage LPG_propane semi_pressurised | `checks[25].message` | GM 4.77 m (fluid), max GZ 1.57 m at 26° | GM 4.78 m (fluid), max GZ 1.57 m at 26° | changed |
| sweep gas carriage LPG_propane semi_pressurised | `checks[26].message` | GM 4.81 m (fluid), max GZ 1.57 m at 26° | GM 4.82 m (fluid), max GZ 1.57 m at 26° | changed |
| sweep gas carriage LPG_propane semi_pressurised | `checks[3].message` | freeboard 3.19 m >= required 0.3 m | freeboard 3.20 m >= required 0.3 m | changed |
| sweep optional lpg_carriage_condition | `checks[13].message` | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.93 m of 6.00 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 4.94 m of 6.00 m | changed |
| sweep optional lpg_carriage_condition | `checks[22].message` | freeboard 3.19 m at the carried draught 2.81 m >= required 0.3 m | freeboard 3.20 m at the carried draught 2.80 m >= required 0.3 m | changed |
| sweep optional lpg_carriage_condition | `checks[23].message` | GM 1.36 m (fluid), max GZ 1.04 m at 36° | GM 1.37 m (fluid), max GZ 1.04 m at 36° | changed |
| sweep optional lpg_carriage_condition | `checks[24].message` | GM 1.35 m (fluid), max GZ 1.03 m at 36° | GM 1.36 m (fluid), max GZ 1.04 m at 36° | changed |
| sweep optional lpg_carriage_condition | `checks[25].message` | GM 4.75 m (fluid), max GZ 1.57 m at 26° | GM 4.76 m (fluid), max GZ 1.57 m at 26° | changed |
| ... | 6454 more changed quantities | | | |

### ticket/410-covering-lightship (2026-10-21)

Between `main c4c4d03` and `this branch cdf92bd`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| calibration amaggi-box-2000 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration canal-box-200x25x12-pb2230 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration canal-box-200x35x12-cby222 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration canal-box-200x35x12-imt201 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration canal-box-200x35x12-imt205 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration canal-box-200x35x12-pb2012 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration canal-box-200x35x12-rci250 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration canal-box-200x35x13-cby252 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration canal-box-200x35x13-cby400 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration canal-box-200x35x13-cby433 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration canal-box-200x35x13-pb2352 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration cargill-amazon-jumbo | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration danube-barge-77 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration dsb-holdbarge-1997-76.5 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration europa-iib-1635ou | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration europa-iib-rdb06 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration hidrovia-lhg-3200 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration jeffboat-am2100 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration mcdonough-195x35x12 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration meb-dcb23-ptich | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration meb-dcb24-pola | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration meb-rdb12-samarskaya | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration meb-rdb20-belmax9 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration para-mississippi-2200 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration stretch-hopper-light | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration taf-danube-2008 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration taf-mississippi-2011 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration taf-parana-2013 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration taf-rhin-1980-covered | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration taf-rhin-2000 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration taf-rhin-2001 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration taf-rhin-2008-covered | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration taf-rhin-2019-covered | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration taf-seine-1973 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration taf-seine-1981 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration taf-seine-1993 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration taf-seine-1996 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration taf-seine-1997 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration taf-seine-2000 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| calibration taf-seine-2008 | `stability.limitations[1]` | No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval). | No wind heel and no damage stability. Both are needed before approval: damage stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather criterion (Part A 2.3). | changed |
| ... | 61617 more changed quantities | | | |

### ticket/421-one-air-draft-check (2026-10-21)

Between `main aa5fe5a` and `this branch ff3cea1`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep optional air_draft_limit_m | `checks[10].name` | Stability — ballast, departure | Stability — ballast, arrival | changed |
| sweep optional air_draft_limit_m | `checks[11].message` | GM 8.30 m (fluid), max GZ 2.41 m at 26° | GM 26.27 m (fluid), max GZ 4.01 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | changed |
| sweep optional air_draft_limit_m | `checks[11].name` | Stability — ballast, arrival | Stability — lightship, no ballast | changed |
| sweep optional air_draft_limit_m | `checks[11].ok` | True | False | changed |
| sweep optional air_draft_limit_m | `checks[12].message` | GM 26.27 m (fluid), max GZ 4.01 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | absent | changed |
| sweep optional air_draft_limit_m | `checks[12].name` | Stability — lightship, no ballast | absent | changed |
| sweep optional air_draft_limit_m | `checks[12].ok` | False | absent | changed |
| sweep optional air_draft_limit_m | `checks[12].severity` | warning | absent | changed |
| sweep optional air_draft_limit_m | `checks[7].message` | 4.92 m vs limit 6.00 m | GM 2.79 m (fluid), max GZ 0.17 m at 6° — fails: Area 0–30° 0.021 < 0.055, Area 0–40° -0.024 < 0.09, Area 30–40° -0.046 < 0.03, GZ at ≥ 30° -0.174 < 0.2, Angle of max GZ 6.000 < 25.0 | changed |
| sweep optional air_draft_limit_m | `checks[7].name` | Air draft (empty, governing) | Stability — full load, departure | changed |
| sweep optional air_draft_limit_m | `checks[7].ok` | True | False | changed |
| sweep optional air_draft_limit_m | `checks[7].severity` | absent | warning | changed |
| sweep optional air_draft_limit_m | `checks[8].name` | Stability — full load, departure | Stability — full load, arrival | changed |
| sweep optional air_draft_limit_m | `checks[9].message` | GM 2.79 m (fluid), max GZ 0.17 m at 6° — fails: Area 0–30° 0.021 < 0.055, Area 0–40° -0.024 < 0.09, Area 30–40° -0.046 < 0.03, GZ at ≥ 30° -0.174 < 0.2, Angle of max GZ 6.000 < 25.0 | GM 8.30 m (fluid), max GZ 2.41 m at 26° | changed |
| sweep optional air_draft_limit_m | `checks[9].name` | Stability — full load, arrival | Stability — ballast, departure | changed |
| sweep optional air_draft_limit_m | `checks[9].ok` | False | True | changed |
| sweep optional vessel_class | `checks[15].message` | 7.23 m vs limit 5.25 m | GM 1.81 m (fluid), max GZ 0.79 m at 30° | changed |
| sweep optional vessel_class | `checks[15].name` | Air draft (empty, governing) | Stability — full load, departure | changed |
| sweep optional vessel_class | `checks[15].ok` | False | True | changed |
| sweep optional vessel_class | `checks[15].severity` | absent | warning | changed |
| sweep optional vessel_class | `checks[16].message` | GM 1.81 m (fluid), max GZ 0.79 m at 30° | GM 1.81 m (fluid), max GZ 0.80 m at 30° | changed |
| sweep optional vessel_class | `checks[16].name` | Stability — full load, departure | Stability — full load, arrival | changed |
| sweep optional vessel_class | `checks[17].message` | GM 1.81 m (fluid), max GZ 0.80 m at 30° | GM 3.70 m (fluid), max GZ 1.95 m at 37° | changed |
| sweep optional vessel_class | `checks[17].name` | Stability — full load, arrival | Stability — ballast, departure | changed |
| sweep optional vessel_class | `checks[18].message` | GM 3.70 m (fluid), max GZ 1.95 m at 37° | Ballast, departure: no ballast plan meets every criterion, even with every ballast tank full. | changed |
| sweep optional vessel_class | `checks[18].name` | Stability — ballast, departure | Ballast plan | changed |
| sweep optional vessel_class | `checks[18].ok` | True | False | changed |
| sweep optional vessel_class | `checks[19].message` | Ballast, departure: no ballast plan meets every criterion, even with every ballast tank full. | Ballast, departure: air draft 5.59 m is above the limit. | changed |
| sweep optional vessel_class | `checks[19].name` | Ballast plan | Air draft | changed |
| sweep optional vessel_class | `checks[20].message` | Ballast, departure: air draft 5.59 m is above the limit. | GM 3.71 m (fluid), max GZ 1.95 m at 36° | changed |
| sweep optional vessel_class | `checks[20].name` | Air draft | Stability — ballast, arrival | changed |
| sweep optional vessel_class | `checks[20].ok` | False | True | changed |
| sweep optional vessel_class | `checks[21].message` | GM 3.71 m (fluid), max GZ 1.95 m at 36° | Ballast, arrival: no ballast plan meets every criterion, even with every ballast tank full. | changed |
| sweep optional vessel_class | `checks[21].name` | Stability — ballast, arrival | Ballast plan | changed |
| sweep optional vessel_class | `checks[21].ok` | True | False | changed |
| sweep optional vessel_class | `checks[22].message` | Ballast, arrival: no ballast plan meets every criterion, even with every ballast tank full. | Ballast, arrival: air draft 5.61 m is above the limit. | changed |
| sweep optional vessel_class | `checks[22].name` | Ballast plan | Air draft | changed |
| sweep optional vessel_class | `checks[23].message` | Ballast, arrival: air draft 5.61 m is above the limit. | absent | changed |
| sweep optional vessel_class | `checks[23].name` | Air draft | absent | changed |
| sweep optional vessel_class | `checks[23].ok` | False | absent | changed |
| ... | 26 more changed quantities | | | |

### ticket/415-dry-bulk-gm (2026-10-21)

Between `main b1e9324` and `this branch dfc9420`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| calibration amaggi-box-2000 | `checks[10].message` | absent | GM 15.42 m (fluid), max GZ 2.76 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | changed |
| calibration amaggi-box-2000 | `checks[10].name` | absent | Stability — lightship, no ballast | changed |
| calibration amaggi-box-2000 | `checks[10].ok` | absent | False | changed |
| calibration amaggi-box-2000 | `checks[10].severity` | absent | warning | changed |
| calibration amaggi-box-2000 | `checks[5].message` | GM 1.38 m (fluid), max GZ 0.23 m at 13° — fails: Area 0–40° 0.074 < 0.09, Area 30–40° -0.006 < 0.03, GZ at ≥ 30° 0.047 < 0.2, Angle of max GZ 13.000 < 25.0 | intact GM 1.38 m >= 0.15 m at the reported cargo 1665 t, draught 3.56 m (first-order box; cargo KG of the full hold; KG_light @0.5D) | changed |
| calibration amaggi-box-2000 | `checks[5].name` | Stability — full load, departure | intact gm at reported cargo | changed |
| calibration amaggi-box-2000 | `checks[5].ok` | False | True | changed |
| calibration amaggi-box-2000 | `checks[6].name` | Stability — full load, arrival | Stability — full load, departure | changed |
| calibration amaggi-box-2000 | `checks[7].message` | GM 6.02 m (fluid), max GZ 1.76 m at 25° | GM 1.38 m (fluid), max GZ 0.23 m at 13° — fails: Area 0–40° 0.074 < 0.09, Area 30–40° -0.006 < 0.03, GZ at ≥ 30° 0.047 < 0.2, Angle of max GZ 13.000 < 25.0 | changed |
| calibration amaggi-box-2000 | `checks[7].name` | Stability — ballast, departure | Stability — full load, arrival | changed |
| calibration amaggi-box-2000 | `checks[7].ok` | True | False | changed |
| calibration amaggi-box-2000 | `checks[8].name` | Stability — ballast, arrival | Stability — ballast, departure | changed |
| calibration amaggi-box-2000 | `checks[9].message` | GM 15.42 m (fluid), max GZ 2.76 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | GM 6.02 m (fluid), max GZ 1.76 m at 25° | changed |
| calibration amaggi-box-2000 | `checks[9].name` | Stability — lightship, no ballast | Stability — ballast, arrival | changed |
| calibration amaggi-box-2000 | `checks[9].ok` | False | True | changed |
| calibration amaggi-box-2000 | `warnings[1]` | absent | Bulk GM is first-order (assumed lightship KG); cargo-shift/angle-of-repose not modelled — confirm with a full intact-stability booklet. | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[10].message` | absent | GM 8.69 m (fluid), max GZ 1.76 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[10].name` | absent | Stability — lightship, no ballast | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[10].ok` | absent | False | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[10].severity` | absent | warning | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[5].message` | GM 0.40 m (fluid), max GZ 0.12 m at 17° — fails: Area 0–30° 0.038 < 0.055, Area 0–40° 0.031 < 0.09, Area 30–40° -0.007 < 0.03, GZ at ≥ 30° 0.024 < 0.2, Angle of max GZ 17.000 < 25.0 | intact GM 0.40 m >= 0.15 m at the reported cargo 897 t, draught 2.74 m (first-order box; cargo KG of the full hold; KG_light @0.5D) | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[5].name` | Stability — full load, departure | intact gm at reported cargo | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[5].ok` | False | True | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[6].name` | Stability — full load, arrival | Stability — full load, departure | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[7].message` | GM 3.71 m (fluid), max GZ 1.14 m at 25° | GM 0.40 m (fluid), max GZ 0.12 m at 17° — fails: Area 0–30° 0.038 < 0.055, Area 0–40° 0.031 < 0.09, Area 30–40° -0.007 < 0.03, GZ at ≥ 30° 0.024 < 0.2, Angle of max GZ 17.000 < 25.0 | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[7].name` | Stability — ballast, departure | Stability — full load, arrival | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[7].ok` | True | False | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[8].name` | Stability — ballast, arrival | Stability — ballast, departure | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[9].message` | GM 8.69 m (fluid), max GZ 1.76 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | GM 3.71 m (fluid), max GZ 1.14 m at 25° | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[9].name` | Stability — lightship, no ballast | Stability — ballast, arrival | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[9].ok` | False | True | changed |
| calibration canal-box-200x25x12-pb2230 | `warnings[1]` | absent | Bulk GM is first-order (assumed lightship KG); cargo-shift/angle-of-repose not modelled — confirm with a full intact-stability booklet. | changed |
| calibration canal-box-200x35x12-cby222 | `checks[10].message` | absent | GM 18.57 m (fluid), max GZ 2.97 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | changed |
| calibration canal-box-200x35x12-cby222 | `checks[10].name` | absent | Stability — lightship, no ballast | changed |
| calibration canal-box-200x35x12-cby222 | `checks[10].ok` | absent | False | changed |
| calibration canal-box-200x35x12-cby222 | `checks[10].severity` | absent | warning | changed |
| calibration canal-box-200x35x12-cby222 | `checks[5].message` | GM 2.09 m (fluid), max GZ 0.49 m at 19° — fails: Angle of max GZ 19.000 < 25.0 | intact GM 2.09 m >= 0.15 m at the reported cargo 1256 t, draught 2.74 m (first-order box; cargo KG of the full hold; KG_light @0.5D) | changed |
| calibration canal-box-200x35x12-cby222 | `checks[5].name` | Stability — full load, departure | intact gm at reported cargo | changed |
| calibration canal-box-200x35x12-cby222 | `checks[5].ok` | False | True | changed |
| calibration canal-box-200x35x12-cby222 | `checks[6].name` | Stability — full load, arrival | Stability — full load, departure | changed |
| ... | 1102 more changed quantities | | | |

### ticket/426-part3-default-conditions (2026-10-21)

Between `main 01f2a10` and `this branch 7b56c5f`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| calibration amaggi-box-2000 | `checks[5].message` | absent | GM 1.38 m (fluid), max GZ 0.23 m at 13° — fails: Area 0–40° 0.074 < 0.09, Area 30–40° -0.006 < 0.03, GZ at ≥ 30° 0.047 < 0.2, Angle of max GZ 13.000 < 25.0 | changed |
| calibration amaggi-box-2000 | `checks[5].name` | absent | Stability — full load, departure | changed |
| calibration amaggi-box-2000 | `checks[5].ok` | absent | False | changed |
| calibration amaggi-box-2000 | `checks[5].severity` | absent | warning | changed |
| calibration amaggi-box-2000 | `checks[6].message` | absent | GM 1.38 m (fluid), max GZ 0.23 m at 13° — fails: Area 0–40° 0.074 < 0.09, Area 30–40° -0.006 < 0.03, GZ at ≥ 30° 0.047 < 0.2, Angle of max GZ 13.000 < 25.0 | changed |
| calibration amaggi-box-2000 | `checks[6].name` | absent | Stability — full load, arrival | changed |
| calibration amaggi-box-2000 | `checks[6].ok` | absent | False | changed |
| calibration amaggi-box-2000 | `checks[6].severity` | absent | warning | changed |
| calibration amaggi-box-2000 | `checks[7].message` | absent | GM 6.02 m (fluid), max GZ 1.76 m at 25° | changed |
| calibration amaggi-box-2000 | `checks[7].name` | absent | Stability — ballast, departure | changed |
| calibration amaggi-box-2000 | `checks[7].ok` | absent | True | changed |
| calibration amaggi-box-2000 | `checks[7].severity` | absent | warning | changed |
| calibration amaggi-box-2000 | `checks[8].message` | absent | GM 6.02 m (fluid), max GZ 1.76 m at 25° | changed |
| calibration amaggi-box-2000 | `checks[8].name` | absent | Stability — ballast, arrival | changed |
| calibration amaggi-box-2000 | `checks[8].ok` | absent | True | changed |
| calibration amaggi-box-2000 | `checks[8].severity` | absent | warning | changed |
| calibration amaggi-box-2000 | `checks[9].message` | absent | GM 15.42 m (fluid), max GZ 2.76 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | changed |
| calibration amaggi-box-2000 | `checks[9].name` | absent | Stability — lightship, no ballast | changed |
| calibration amaggi-box-2000 | `checks[9].ok` | absent | False | changed |
| calibration amaggi-box-2000 | `checks[9].severity` | absent | warning | changed |
| calibration amaggi-box-2000 | `loading_conditions.ballast_capacity_m3` | absent | 1287.5044664 | changed |
| calibration amaggi-box-2000 | `loading_conditions.ballast_capacity_t` | absent | 1287.5044664 | changed |
| calibration amaggi-box-2000 | `loading_conditions.ballast_tank_count` | absent | 6 | changed |
| calibration amaggi-box-2000 | `loading_conditions.ballast_tanks[0].capacity_m3` | absent | 320.919456 | changed |
| calibration amaggi-box-2000 | `loading_conditions.ballast_tanks[0].capacity_t` | absent | 320.919456 | changed |
| calibration amaggi-box-2000 | `loading_conditions.ballast_tanks[0].kind` | absent | double_bottom_ballast | changed |
| calibration amaggi-box-2000 | `loading_conditions.ballast_tanks[0].lcg_m` | absent | 28.564 | changed |
| calibration amaggi-box-2000 | `loading_conditions.ballast_tanks[0].name` | absent | DB1P | changed |
| calibration amaggi-box-2000 | `loading_conditions.ballast_tanks[0].side` | absent | port | changed |
| calibration amaggi-box-2000 | `loading_conditions.ballast_tanks[0].tcg_m` | absent | -2.6675 | changed |
| calibration amaggi-box-2000 | `loading_conditions.ballast_tanks[0].vcg_m` | absent | 0.6 | changed |
| calibration amaggi-box-2000 | `loading_conditions.ballast_tanks[1].capacity_m3` | absent | 320.919456 | changed |
| calibration amaggi-box-2000 | `loading_conditions.ballast_tanks[1].capacity_t` | absent | 320.919456 | changed |
| calibration amaggi-box-2000 | `loading_conditions.ballast_tanks[1].kind` | absent | double_bottom_ballast | changed |
| calibration amaggi-box-2000 | `loading_conditions.ballast_tanks[1].lcg_m` | absent | 28.564 | changed |
| calibration amaggi-box-2000 | `loading_conditions.ballast_tanks[1].name` | absent | DB1S | changed |
| calibration amaggi-box-2000 | `loading_conditions.ballast_tanks[1].side` | absent | starboard | changed |
| calibration amaggi-box-2000 | `loading_conditions.ballast_tanks[1].tcg_m` | absent | 2.6675 | changed |
| calibration amaggi-box-2000 | `loading_conditions.ballast_tanks[1].vcg_m` | absent | 0.6 | changed |
| calibration amaggi-box-2000 | `loading_conditions.ballast_tanks[2].capacity_m3` | absent | 153.89296 | changed |
| ... | 287251 more changed quantities | | | |

### ticket/416-ld-inner-diameter (2026-10-21)

Between `main 97f7d80` and `this branch f465131`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `checks[21].message` | tank length 28.44 m / outer diameter 6.85 m = 4.15 <= 7.0 | tank length 28.44 m / inner diameter 6.80 m = 4.18 <= 7.0 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[22].message` | tank length 49.44 m / outer diameter 10.12 m = 4.89 <= 7.0 | tank length 49.44 m / inner diameter 10.04 m = 4.92 <= 7.0 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[21].message` | tank length 49.44 m / outer diameter 7.25 m = 6.82 <= 7.0 | tank length 49.44 m / inner diameter 7.20 m = 6.87 <= 7.0 | changed |
| sweep default depth gas 90x14 | `checks[21].message` | tank length 25.89 m / outer diameter 9.91 m = 2.61 <= 7.0 | tank length 25.89 m / inner diameter 9.23 m = 2.81 <= 7.0 | changed |
| sweep gas IGC_2G 60x10 | `checks[21].message` | tank length 12.65 m / outer diameter 8.40 m = 1.51 <= 7.0 | tank length 12.65 m / inner diameter 8.34 m = 1.52 <= 7.0 | changed |
| sweep gas IGC_2G 90x14 | `checks[21].message` | tank length 27.65 m / outer diameter 8.20 m = 3.37 <= 7.0 | tank length 27.65 m / inner diameter 8.14 m = 3.40 <= 7.0 | changed |
| sweep gas IGC_2G_EXACT 60x10 | `checks[21].message` | tank length 12.65 m / outer diameter 8.40 m = 1.51 <= 7.0 | tank length 12.65 m / inner diameter 8.34 m = 1.52 <= 7.0 | changed |
| sweep gas IGC_2G_EXACT 90x14 | `checks[21].message` | tank length 27.65 m / outer diameter 8.20 m = 3.37 <= 7.0 | tank length 27.65 m / inner diameter 8.14 m = 3.40 <= 7.0 | changed |
| sweep gas cargo LNG | `checks[21].message` | tank length 27.65 m / outer diameter 8.89 m = 3.11 <= 7.0 | tank length 27.65 m / inner diameter 8.21 m = 3.37 <= 7.0 | changed |
| sweep gas cargo LNG pushed | `checks[12].message` | tank length 36.45 m / outer diameter 11.79 m = 3.09 <= 7.0 | tank length 36.45 m / inner diameter 11.11 m = 3.28 <= 7.0 | changed |
| sweep gas cargo LNG_ethane | `checks[21].message` | tank length 27.65 m / outer diameter 8.89 m = 3.11 <= 7.0 | tank length 27.65 m / inner diameter 8.21 m = 3.37 <= 7.0 | changed |
| sweep gas cargo LNG_ethane pushed | `checks[12].message` | tank length 36.45 m / outer diameter 10.79 m = 3.38 <= 7.0 | tank length 36.45 m / inner diameter 10.11 m = 3.60 <= 7.0 | changed |
| sweep gas cargo LPG_butane | `checks[21].message` | tank length 27.65 m / outer diameter 8.20 m = 3.37 <= 7.0 | tank length 27.65 m / inner diameter 8.14 m = 3.40 <= 7.0 | changed |
| sweep gas cargo LPG_butane pushed | `checks[12].message` | tank length 36.45 m / outer diameter 9.11 m = 4.00 <= 7.0 | tank length 36.45 m / inner diameter 9.04 m = 4.03 <= 7.0 | changed |
| sweep gas cargo LPG_mix | `checks[21].message` | tank length 27.65 m / outer diameter 8.20 m = 3.37 <= 7.0 | tank length 27.65 m / inner diameter 8.14 m = 3.40 <= 7.0 | changed |
| sweep gas cargo LPG_mix pushed | `checks[12].message` | tank length 36.45 m / outer diameter 9.11 m = 4.00 <= 7.0 | tank length 36.45 m / inner diameter 9.04 m = 4.03 <= 7.0 | changed |
| sweep gas cargo LPG_propane | `checks[21].message` | tank length 27.65 m / outer diameter 8.20 m = 3.37 <= 7.0 | tank length 27.65 m / inner diameter 8.14 m = 3.40 <= 7.0 | changed |
| sweep gas cargo LPG_propane pushed | `checks[12].message` | tank length 36.45 m / outer diameter 9.11 m = 4.00 <= 7.0 | tank length 36.45 m / inner diameter 9.04 m = 4.03 <= 7.0 | changed |
| sweep gas carriage LPG_butane fully_pressurised | `checks[21].message` | tank length 37.25 m / outer diameter 7.64 m = 4.87 <= 7.0 | tank length 37.25 m / inner diameter 7.59 m = 4.91 <= 7.0 | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[21].message` | tank length 37.25 m / outer diameter 8.04 m = 4.63 <= 7.0 | tank length 37.25 m / inner diameter 7.63 m = 4.89 <= 7.0 | changed |
| sweep gas carriage LPG_butane semi_pressurised | `checks[21].message` | tank length 37.25 m / outer diameter 8.04 m = 4.63 <= 7.0 | tank length 37.25 m / inner diameter 7.61 m = 4.90 <= 7.0 | changed |
| sweep gas carriage LPG_mix fully_pressurised | `checks[21].message` | tank length 37.25 m / outer diameter 7.64 m = 4.87 <= 7.0 | tank length 37.25 m / inner diameter 7.59 m = 4.91 <= 7.0 | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[21].message` | tank length 37.25 m / outer diameter 8.04 m = 4.63 <= 7.0 | tank length 37.25 m / inner diameter 7.63 m = 4.89 <= 7.0 | changed |
| sweep gas carriage LPG_mix semi_pressurised | `checks[21].message` | tank length 37.25 m / outer diameter 8.04 m = 4.63 <= 7.0 | tank length 37.25 m / inner diameter 7.61 m = 4.90 <= 7.0 | changed |
| sweep gas carriage LPG_propane fully_pressurised | `checks[21].message` | tank length 37.25 m / outer diameter 7.64 m = 4.87 <= 7.0 | tank length 37.25 m / inner diameter 7.59 m = 4.91 <= 7.0 | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[21].message` | tank length 37.25 m / outer diameter 8.04 m = 4.63 <= 7.0 | tank length 37.25 m / inner diameter 7.63 m = 4.89 <= 7.0 | changed |
| sweep gas carriage LPG_propane semi_pressurised | `checks[21].message` | tank length 37.25 m / outer diameter 8.04 m = 4.63 <= 7.0 | tank length 37.25 m / inner diameter 7.61 m = 4.90 <= 7.0 | changed |
| sweep gas carries nothing (draught limit) | `checks[22].message` | tank length 27.65 m / outer diameter 8.20 m = 3.37 <= 7.0 | tank length 27.65 m / inner diameter 8.14 m = 3.40 <= 7.0 | changed |
| sweep gas channel 60x10 | `checks[12].message` | tank length 23.05 m / outer diameter 8.05 m = 2.86 <= 7.0 | tank length 23.05 m / inner diameter 7.38 m = 3.13 <= 7.0 | changed |
| sweep gas channel 90x14 | `checks[12].message` | tank length 36.85 m / outer diameter 10.79 m = 3.42 <= 7.0 | tank length 36.85 m / inner diameter 10.11 m = 3.65 <= 7.0 | changed |
| sweep gas passes all checks IGC_2G 110x11.4 | `checks[21].message` | tank length 37.25 m / outer diameter 7.64 m = 4.87 <= 7.0 | tank length 37.25 m / inner diameter 7.59 m = 4.91 <= 7.0 | changed |
| sweep gas passes all checks IGC_2G_EXACT 80x14 | `checks[21].message` | tank length 23.45 m / outer diameter 5.70 m = 4.11 <= 7.0 | tank length 23.45 m / inner diameter 5.66 m = 4.15 <= 7.0 | changed |
| sweep gas tank longer than 7 diameters IGC_2G 200x12 | `checks[22].message` | tank length 82.65 m / outer diameter 9.58 m = 8.63 <= 7.0 | tank length 82.65 m / inner diameter 8.90 m = 9.29 <= 7.0 | changed |
| sweep gas wide hull IGC_2G 110x24 | `checks[22].message` | tank length 38.80 m / outer diameter 9.94 m = 3.91 <= 7.0 | tank length 38.80 m / inner diameter 9.26 m = 4.19 <= 7.0 | changed |
| sweep gas wide hull IGC_2G_EXACT 110x24 | `checks[22].message` | tank length 38.80 m / outer diameter 9.94 m = 3.91 <= 7.0 | tank length 38.80 m / inner diameter 9.26 m = 4.19 <= 7.0 | changed |
| sweep optional accommodation_unit_count | `checks[21].message` | tank length 37.25 m / outer diameter 7.64 m = 4.87 <= 7.0 | tank length 37.25 m / inner diameter 7.59 m = 4.91 <= 7.0 | changed |
| sweep optional battery_rate_per_kwh | `checks[21].message` | tank length 37.25 m / outer diameter 7.64 m = 4.87 <= 7.0 | tank length 37.25 m / inner diameter 7.59 m = 4.91 <= 7.0 | changed |
| sweep optional battery_unit_count | `checks[21].message` | tank length 37.25 m / outer diameter 7.64 m = 4.87 <= 7.0 | tank length 37.25 m / inner diameter 7.59 m = 4.91 <= 7.0 | changed |
| sweep optional cargo_system_lump | `checks[21].message` | tank length 37.25 m / outer diameter 7.64 m = 4.87 <= 7.0 | tank length 37.25 m / inner diameter 7.59 m = 4.91 <= 7.0 | changed |
| sweep optional delivery_lump | `checks[21].message` | tank length 37.25 m / outer diameter 7.64 m = 4.87 <= 7.0 | tank length 37.25 m / inner diameter 7.59 m = 4.91 <= 7.0 | changed |
| ... | 49 more changed quantities | | | |

### ticket/399-400-resistance-labels (2026-10-21)

Between `main 9980d3c` and `this branch 83ea31a`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep box self-propelled 60x10 | `checks[10].message` | 2 x 1.31 m engines abreast need 5.62 m across a 10.00 m beam | 2 x 1.32 m engines abreast need 5.65 m across a 10.00 m beam | changed |
| sweep box self-propelled 60x10 | `checks[11].message` | engine 1.23 m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth 4.50 m | engine 1.24 m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth 4.50 m | changed |
| sweep box self-propelled 60x10 | `checks[3].message` | freeboard 2.14 m >= required 0.3 m | freeboard 2.13 m >= required 0.3 m | changed |
| sweep hopper 60x10 | `checks[10].message` | 2 x 1.31 m engines abreast need 5.62 m across a 10.00 m beam | 2 x 1.32 m engines abreast need 5.65 m across a 10.00 m beam | changed |
| sweep hopper 60x10 | `checks[11].message` | engine 1.23 m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth 4.50 m | engine 1.24 m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth 4.50 m | changed |
| sweep box self-propelled 60x10 | `stern_block_mass.items[10].mass_t` | 0.508848855467 | 0.523200836184 | +2.82 % |
| sweep box self-propelled 60x10 | `stern_block_mass.items[8].mass_t` | 1.01769771093 | 1.04640167237 | +2.82 % |
| sweep hopper 60x10 | `stern_block_mass.items[10].mass_t` | 0.508848855467 | 0.523200836184 | +2.82 % |
| sweep hopper 60x10 | `stern_block_mass.items[8].mass_t` | 1.01769771093 | 1.04640167237 | +2.82 % |
| sweep box self-propelled 60x10 | `powering.installed_power_kw` | 508.848855467 | 523.200836184 | +2.82 % |
| sweep hopper 60x10 | `powering.installed_power_kw` | 508.848855467 | 523.200836184 | +2.82 % |
| sweep box self-propelled 60x10 | `stern_block_mass.items[7].mass_t` | 2.49335939179 | 2.5636840973 | +2.82 % |
| sweep hopper 60x10 | `stern_block_mass.items[7].mass_t` | 2.49335939179 | 2.5636840973 | +2.82 % |
| sweep box self-propelled 60x10 | `powering.environments[0].fuel_kg_per_h` | 71.5188254959 | 73.5086501428 | +2.78 % |
| sweep box self-propelled 60x10 | `powering.environments[0].power_at_service_kw` | 366.763207671 | 376.96743663 | +2.78 % |
| sweep hopper 60x10 | `powering.environments[0].power_at_service_kw` | 340.766331966 | 350.206876415 | +2.77 % |
| sweep hopper 60x10 | `powering.environments[0].fuel_kg_per_h` | 66.4494347334 | 68.2903409009 | +2.77 % |
| sweep gas IGC_2G 60x10 | `stern_block_mass.items[8].mass_t` | 1.97774040955 | 2.03008023023 | +2.65 % |
| sweep gas IGC_2G_EXACT 60x10 | `stern_block_mass.items[8].mass_t` | 1.97774040955 | 2.03008023023 | +2.65 % |
| sweep gas IGC_2G 60x10 | `stern_block_mass.items[6].mass_t` | 0.41202925199 | 0.422933381298 | +2.65 % |
| sweep gas IGC_2G_EXACT 60x10 | `stern_block_mass.items[6].mass_t` | 0.41202925199 | 0.422933381298 | +2.65 % |
| sweep gas IGC_2G 60x10 | `stern_block_mass.items[10].mass_t` | 0.329623401592 | 0.338346705038 | +2.65 % |
| sweep gas IGC_2G_EXACT 60x10 | `stern_block_mass.items[10].mass_t` | 0.329623401592 | 0.338346705038 | +2.65 % |
| sweep gas IGC_2G 60x10 | `powering.installed_power_kw` | 329.623401592 | 338.346705038 | +2.65 % |
| sweep gas IGC_2G_EXACT 60x10 | `powering.installed_power_kw` | 329.623401592 | 338.346705038 | +2.65 % |
| sweep gas IGC_2G 60x10 | `powering.environments[0].power_at_service_kw` | 218.405808977 | 224.127182331 | +2.62 % |
| sweep gas IGC_2G_EXACT 60x10 | `powering.environments[0].power_at_service_kw` | 218.405808977 | 224.127182331 | +2.62 % |
| sweep hopper 60x10 | `curves[0].points[17].kw` | 246.341894586 | 252.576008101 | +2.53 % |
| sweep hopper 60x10 | `curves[0].points[17].fuel_lph` | 56.8481295199 | 58.2867711003 | +2.53 % |
| sweep box self-propelled 60x10 | `curves[0].points[17].fuel_lph` | 60.7923600013 | 62.3306586724 | +2.53 % |
| sweep box self-propelled 60x10 | `curves[0].points[17].kw` | 263.433560006 | 270.099520914 | +2.53 % |
| sweep gas IGC_2G 60x10 | `curves[0].points[17].fuel_lph` | 58.8875771528 | 60.3772602403 | +2.53 % |
| sweep gas IGC_2G_EXACT 60x10 | `curves[0].points[17].fuel_lph` | 58.8875771528 | 60.3772602403 | +2.53 % |
| sweep gas IGC_2G 60x10 | `curves[0].points[17].kw` | 255.179500995 | 261.634794374 | +2.53 % |
| sweep gas IGC_2G_EXACT 60x10 | `curves[0].points[17].kw` | 255.179500995 | 261.634794374 | +2.53 % |
| sweep hopper 60x10 | `curves[0].points[18].kw` | 285.617550949 | 292.838451244 | +2.53 % |
| sweep hopper 60x10 | `curves[0].points[18].fuel_lph` | 65.9117425268 | 67.5781041333 | +2.53 % |
| sweep box self-propelled 60x10 | `curves[0].points[18].kw` | 305.434235509 | 313.155338786 | +2.53 % |
| sweep box self-propelled 60x10 | `curves[0].points[18].fuel_lph` | 70.4848235789 | 72.266616643 | +2.53 % |
| sweep gas IGC_2G 60x10 | `curves[0].points[18].kw` | 295.86418603 | 303.341273592 | +2.53 % |
| ... | 799 more changed quantities | | | |

### ticket/396-397-confined-water (2026-10-21)

Between `main 2a763b5` and `this branch ff452b9`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep box channel 60x10 | `curves[0].points[14].fuel_lph` | 58.2057232944 | absent | changed |
| sweep box channel 60x10 | `curves[0].points[14].kn` | 8 | absent | changed |
| sweep box channel 60x10 | `curves[0].points[14].kw` | 252.224800942 | absent | changed |
| sweep box channel 60x10 | `curves[0].points[14].squat_limited` | False | absent | changed |
| sweep box channel 60x10 | `curves[0].points[14].squat_m` | 0.609701277408 | absent | changed |
| sweep box channel 60x10 | `curves[0].points[15].fuel_lph` | 72.7498621271 | absent | changed |
| sweep box channel 60x10 | `curves[0].points[15].kn` | 8.5 | absent | changed |
| sweep box channel 60x10 | `curves[0].points[15].kw` | 315.249402551 | absent | changed |
| sweep box channel 60x10 | `curves[0].points[15].squat_limited` | True | absent | changed |
| sweep box channel 60x10 | `curves[0].points[15].squat_m` | 0.712973679425 | absent | changed |
| sweep box channel 60x10 | `curves[0].points[16].fuel_lph` | 90.2406511809 | absent | changed |
| sweep box channel 60x10 | `curves[0].points[16].kn` | 9 | absent | changed |
| sweep box channel 60x10 | `curves[0].points[16].kw` | 391.042821784 | absent | changed |
| sweep box channel 60x10 | `curves[0].points[16].squat_limited` | True | absent | changed |
| sweep box channel 60x10 | `curves[0].points[16].squat_m` | 0.832181363152 | absent | changed |
| sweep box channel 60x10 | `curves[0].points[17].fuel_lph` | 108.287106709 | absent | changed |
| sweep box channel 60x10 | `curves[0].points[17].kn` | 9.5 | absent | changed |
| sweep box channel 60x10 | `curves[0].points[17].kw` | 469.24412907 | absent | changed |
| sweep box channel 60x10 | `curves[0].points[17].squat_limited` | True | absent | changed |
| sweep box channel 60x10 | `curves[0].points[17].squat_m` | 0.971295867379 | absent | changed |
| sweep box channel 60x10 | `curves[0].points[18].fuel_lph` | 131.291188892 | absent | changed |
| sweep box channel 60x10 | `curves[0].points[18].kn` | 10 | absent | changed |
| sweep box channel 60x10 | `curves[0].points[18].kw` | 568.928485197 | absent | changed |
| sweep box channel 60x10 | `curves[0].points[18].squat_limited` | True | absent | changed |
| sweep box channel 60x10 | `curves[0].points[18].squat_m` | 1.13608981891 | absent | changed |
| sweep box channel 90x14 | `curves[0].points[12].fuel_lph` | 62.5910933648 | absent | changed |
| sweep box channel 90x14 | `curves[0].points[12].kn` | 7 | absent | changed |
| sweep box channel 90x14 | `curves[0].points[12].kw` | 271.228071247 | absent | changed |
| sweep box channel 90x14 | `curves[0].points[12].squat_limited` | False | absent | changed |
| sweep box channel 90x14 | `curves[0].points[12].squat_m` | 0.542529253467 | absent | changed |
| sweep box channel 90x14 | `curves[0].points[13].fuel_lph` | 79.5695628641 | absent | changed |
| sweep box channel 90x14 | `curves[0].points[13].kn` | 7.5 | absent | changed |
| sweep box channel 90x14 | `curves[0].points[13].kw` | 344.801439078 | absent | changed |
| sweep box channel 90x14 | `curves[0].points[13].squat_limited` | False | absent | changed |
| sweep box channel 90x14 | `curves[0].points[13].squat_m` | 0.626248451721 | absent | changed |
| sweep box channel 90x14 | `curves[0].points[14].fuel_lph` | 100.155490398 | absent | changed |
| sweep box channel 90x14 | `curves[0].points[14].kn` | 8 | absent | changed |
| sweep box channel 90x14 | `curves[0].points[14].kw` | 434.007125058 | absent | changed |
| sweep box channel 90x14 | `curves[0].points[14].squat_limited` | True | absent | changed |
| sweep box channel 90x14 | `curves[0].points[14].squat_m` | 0.716219943939 | absent | changed |
| ... | 649 more changed quantities | | | |

### ticket/354-blind-distance (2026-10-21)

Between `main c9584a2` and `this branch 34e1488`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived bulk 1 (88x16.3) | `checks[10].message` | raised eye 7.45 m >= obstruction 6.80 m + 0.50 m margin (lift used 0.00 m of 6.00 m) | 2 x 1.58 m engines abreast need 6.16 m across a 16.30 m beam | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[10].name` | wheelhouse sees over cargo | machinery fits beam | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[11].message` | 2 x 1.58 m engines abreast need 6.16 m across a 16.30 m beam | engine 1.48 m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth 5.80 m | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[11].name` | machinery fits beam | machinery fits under deck | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[11].severity` | error | warning | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[12].message` | engine 1.48 m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth 5.80 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 1.53 m of 6.00 m | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[12].name` | machinery fits under deck | wheelhouse sees over cargo | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[12].severity` | warning | error | changed |
| sweep CN-derived bulk 1 (88x16.3) | `sightline.blind_distance_laden_m` | absent | 35.5885251121 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `sightline.blind_distance_light_m` | absent | 250 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `sightline.eye_raised_m` | absent | 8.97752652034 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `sightline.floor_raised_m` | absent | 7.32752652034 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `sightline.lift_enough` | absent | True | changed |
| sweep CN-derived bulk 1 (88x16.3) | `sightline.lift_needed_m` | absent | 1.52752652034 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `sightline.lift_used_m` | absent | 1.52752652034 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `sightline.limit_m` | absent | 250 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `sightline.method` | absent | blind_distance | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[10].message` | raised eye 7.45 m >= obstruction 6.80 m + 0.50 m margin (lift used 0.00 m of 6.00 m) | 2 x 1.76 m engines abreast need 6.52 m across a 16.30 m beam | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[10].name` | wheelhouse sees over cargo | machinery fits beam | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[11].message` | 2 x 1.76 m engines abreast need 6.52 m across a 16.30 m beam | engine 1.65 m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth 5.80 m | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[11].name` | machinery fits beam | machinery fits under deck | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[11].severity` | error | warning | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[12].message` | engine 1.65 m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth 5.80 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 2.36 m of 6.00 m | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[12].name` | machinery fits under deck | wheelhouse sees over cargo | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[12].severity` | warning | error | changed |
| sweep CN-derived bulk 2 (130x16.3) | `sightline.blind_distance_laden_m` | absent | 32.4072853299 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `sightline.blind_distance_light_m` | absent | 250 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `sightline.eye_raised_m` | absent | 9.80606971828 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `sightline.floor_raised_m` | absent | 8.15606971828 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `sightline.lift_enough` | absent | True | changed |
| sweep CN-derived bulk 2 (130x16.3) | `sightline.lift_needed_m` | absent | 2.35606971828 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `sightline.lift_used_m` | absent | 2.35606971828 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `sightline.limit_m` | absent | 250 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `sightline.method` | absent | blind_distance | changed |
| sweep CN-derived bulk 3 (130x17.2) | `checks[10].message` | raised eye 7.45 m >= obstruction 6.80 m + 0.50 m margin (lift used 0.00 m of 6.00 m) | 2 x 1.78 m engines abreast need 6.57 m across a 17.20 m beam | changed |
| sweep CN-derived bulk 3 (130x17.2) | `checks[10].name` | wheelhouse sees over cargo | machinery fits beam | changed |
| sweep CN-derived bulk 3 (130x17.2) | `checks[11].message` | 2 x 1.78 m engines abreast need 6.57 m across a 17.20 m beam | engine 1.67 m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth 5.80 m | changed |
| sweep CN-derived bulk 3 (130x17.2) | `checks[11].name` | machinery fits beam | machinery fits under deck | changed |
| sweep CN-derived bulk 3 (130x17.2) | `checks[11].severity` | error | warning | changed |
| sweep CN-derived bulk 3 (130x17.2) | `checks[12].message` | engine 1.67 m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth 5.80 m | blind distance ahead of the bow 250.0 m <= 250 m, light (ES-TRIN 7.02(2)); lift used 2.32 m of 6.00 m | changed |
| ... | 23499 more changed quantities | | | |

### ticket/426-conditions (2026-10-21)

Between `main b0c1e32` and `this branch 2ef061a`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep optional ballast_plan | `checks[5].message` | absent | GM 2.79 m (fluid), max GZ 0.17 m at 6° — fails: Area 0–30° 0.021 < 0.055, Area 0–40° -0.024 < 0.09, Area 30–40° -0.046 < 0.03, GZ at ≥ 30° -0.174 < 0.2, Angle of max GZ 6.000 < 25.0 | changed |
| sweep optional ballast_plan | `checks[5].name` | absent | Stability — full load, departure | changed |
| sweep optional ballast_plan | `checks[5].ok` | absent | False | changed |
| sweep optional ballast_plan | `checks[5].severity` | absent | warning | changed |
| sweep optional ballast_plan | `checks[6].message` | absent | GM 2.79 m (fluid), max GZ 0.17 m at 6° — fails: Area 0–30° 0.021 < 0.055, Area 0–40° -0.024 < 0.09, Area 30–40° -0.046 < 0.03, GZ at ≥ 30° -0.174 < 0.2, Angle of max GZ 6.000 < 25.0 | changed |
| sweep optional ballast_plan | `checks[6].name` | absent | Stability — full load, arrival | changed |
| sweep optional ballast_plan | `checks[6].ok` | absent | False | changed |
| sweep optional ballast_plan | `checks[6].severity` | absent | warning | changed |
| sweep optional ballast_plan | `checks[7].message` | absent | GM 6.68 m (fluid), max GZ 2.27 m at 26° | changed |
| sweep optional ballast_plan | `checks[7].name` | absent | Stability — ballast, departure | changed |
| sweep optional ballast_plan | `checks[7].ok` | absent | True | changed |
| sweep optional ballast_plan | `checks[7].severity` | absent | warning | changed |
| sweep optional ballast_plan | `checks[8].message` | absent | GM 6.80 m (fluid), max GZ 2.87 m at 34° | changed |
| sweep optional ballast_plan | `checks[8].name` | absent | Stability — ballast, arrival | changed |
| sweep optional ballast_plan | `checks[8].ok` | absent | True | changed |
| sweep optional ballast_plan | `checks[8].severity` | absent | warning | changed |
| sweep optional ballast_plan | `checks[9].message` | absent | GM 26.27 m (fluid), max GZ 4.01 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | changed |
| sweep optional ballast_plan | `checks[9].name` | absent | Stability — lightship, no ballast | changed |
| sweep optional ballast_plan | `checks[9].ok` | absent | False | changed |
| sweep optional ballast_plan | `checks[9].severity` | absent | warning | changed |
| sweep optional ballast_plan | `loading_conditions.ballast_capacity_m3` | absent | 2322.72 | changed |
| sweep optional ballast_plan | `loading_conditions.ballast_capacity_t` | absent | 2322.72 | changed |
| sweep optional ballast_plan | `loading_conditions.ballast_tank_count` | absent | 6 | changed |
| sweep optional ballast_plan | `loading_conditions.ballast_tanks[0].capacity_m3` | absent | 661.92 | changed |
| sweep optional ballast_plan | `loading_conditions.ballast_tanks[0].capacity_t` | absent | 661.92 | changed |
| sweep optional ballast_plan | `loading_conditions.ballast_tanks[0].kind` | absent | double_bottom_ballast | changed |
| sweep optional ballast_plan | `loading_conditions.ballast_tanks[0].lcg_m` | absent | 42.9 | changed |
| sweep optional ballast_plan | `loading_conditions.ballast_tanks[0].name` | absent | DB1P | changed |
| sweep optional ballast_plan | `loading_conditions.ballast_tanks[0].side` | absent | port | changed |
| sweep optional ballast_plan | `loading_conditions.ballast_tanks[0].tcg_m` | absent | -3.5 | changed |
| sweep optional ballast_plan | `loading_conditions.ballast_tanks[0].vcg_m` | absent | 0.6 | changed |
| sweep optional ballast_plan | `loading_conditions.ballast_tanks[1].capacity_m3` | absent | 661.92 | changed |
| sweep optional ballast_plan | `loading_conditions.ballast_tanks[1].capacity_t` | absent | 661.92 | changed |
| sweep optional ballast_plan | `loading_conditions.ballast_tanks[1].kind` | absent | double_bottom_ballast | changed |
| sweep optional ballast_plan | `loading_conditions.ballast_tanks[1].lcg_m` | absent | 42.9 | changed |
| sweep optional ballast_plan | `loading_conditions.ballast_tanks[1].name` | absent | DB1S | changed |
| sweep optional ballast_plan | `loading_conditions.ballast_tanks[1].side` | absent | starboard | changed |
| sweep optional ballast_plan | `loading_conditions.ballast_tanks[1].tcg_m` | absent | 3.5 | changed |
| sweep optional ballast_plan | `loading_conditions.ballast_tanks[1].vcg_m` | absent | 0.6 | changed |
| sweep optional ballast_plan | `loading_conditions.ballast_tanks[2].capacity_m3` | absent | 260.04 | changed |
| ... | 7704 more changed quantities | | | |

### 418 (2026-10-21)

Between `main dc0ef3a` and `this branch fcaade0`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| calibration amaggi-box-2000 | `checks[2].message` | cargo zone 50.13 m = LOA 61.0 - bow void 7.33 m - stern void 3.50 m | volume-bound load must float within the moulded depth | changed |
| calibration amaggi-box-2000 | `checks[2].name` | cargo zone positive | not overloaded | changed |
| calibration amaggi-box-2000 | `checks[2].ok` | True | False | changed |
| calibration amaggi-box-2000 | `checks[2].severity` | error | warning | changed |
| calibration amaggi-box-2000 | `checks[3].message` | absent | freeboard 0.00 m >= required 0.3 m | changed |
| calibration amaggi-box-2000 | `checks[3].name` | absent | freeboard ok | changed |
| calibration amaggi-box-2000 | `checks[3].ok` | absent | False | changed |
| calibration amaggi-box-2000 | `checks[3].severity` | absent | warning | changed |
| calibration amaggi-box-2000 | `checks[4].message` | absent | cargo zone 50.13 m = LOA 61.0 - bow void 7.33 m - stern void 3.50 m | changed |
| calibration amaggi-box-2000 | `checks[4].name` | absent | cargo zone positive | changed |
| calibration amaggi-box-2000 | `checks[4].ok` | absent | True | changed |
| calibration amaggi-box-2000 | `checks[4].severity` | absent | error | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[2].message` | cargo zone 51.11 m = LOA 61.0 - bow void 6.35 m - stern void 3.50 m | volume-bound load must float within the moulded depth | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[2].name` | cargo zone positive | not overloaded | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[2].ok` | True | False | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[2].severity` | error | warning | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[3].message` | absent | freeboard 0.00 m >= required 0.3 m | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[3].name` | absent | freeboard ok | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[3].ok` | absent | False | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[3].severity` | absent | warning | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[4].message` | absent | cargo zone 51.11 m = LOA 61.0 - bow void 6.35 m - stern void 3.50 m | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[4].name` | absent | cargo zone positive | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[4].ok` | absent | True | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[4].severity` | absent | error | changed |
| calibration canal-box-200x35x12-cby222 | `checks[2].message` | cargo zone 51.11 m = LOA 61.0 - bow void 6.35 m - stern void 3.50 m | volume-bound load must float within the moulded depth | changed |
| calibration canal-box-200x35x12-cby222 | `checks[2].name` | cargo zone positive | not overloaded | changed |
| calibration canal-box-200x35x12-cby222 | `checks[2].ok` | True | False | changed |
| calibration canal-box-200x35x12-cby222 | `checks[2].severity` | error | warning | changed |
| calibration canal-box-200x35x12-cby222 | `checks[3].message` | absent | freeboard 0.00 m >= required 0.3 m | changed |
| calibration canal-box-200x35x12-cby222 | `checks[3].name` | absent | freeboard ok | changed |
| calibration canal-box-200x35x12-cby222 | `checks[3].ok` | absent | False | changed |
| calibration canal-box-200x35x12-cby222 | `checks[3].severity` | absent | warning | changed |
| calibration canal-box-200x35x12-cby222 | `checks[4].message` | absent | cargo zone 51.11 m = LOA 61.0 - bow void 6.35 m - stern void 3.50 m | changed |
| calibration canal-box-200x35x12-cby222 | `checks[4].name` | absent | cargo zone positive | changed |
| calibration canal-box-200x35x12-cby222 | `checks[4].ok` | absent | True | changed |
| calibration canal-box-200x35x12-cby222 | `checks[4].severity` | absent | error | changed |
| calibration canal-box-200x35x12-imt201 | `checks[2].message` | cargo zone 51.11 m = LOA 61.0 - bow void 6.35 m - stern void 3.50 m | volume-bound load must float within the moulded depth | changed |
| calibration canal-box-200x35x12-imt201 | `checks[2].name` | cargo zone positive | not overloaded | changed |
| calibration canal-box-200x35x12-imt201 | `checks[2].ok` | True | False | changed |
| calibration canal-box-200x35x12-imt201 | `checks[2].severity` | error | warning | changed |
| ... | 7628 more changed quantities | | | |

### ticket/353-powering (2026-10-21)

Between `main ead175a` and `this branch 416db8a`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived bulk 1 (88x16.3) | `checks[10].message` | absent | engine 1.48 m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth 5.80 m | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[10].name` | absent | machinery fits under deck | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[10].ok` | absent | True | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[10].severity` | absent | warning | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[11].message` | absent | hopper intact GM 3.67 m >= 0.15 m (first-order; KG_light @0.5D) | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[11].name` | absent | gm positive | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[11].ok` | absent | True | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[11].severity` | absent | warning | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[9].message` | hopper intact GM 3.67 m >= 0.15 m (first-order; KG_light @0.5D) | 2 x 1.58 m engines abreast need 6.16 m across a 16.30 m beam | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[9].name` | gm positive | machinery fits beam | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[9].severity` | warning | error | changed |
| sweep CN-derived bulk 1 (88x16.3) | `powering.battery_energy_kwh` | absent | none | changed |
| sweep CN-derived bulk 1 (88x16.3) | `powering.bollard_pull_t` | absent | 29.1057616709 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `powering.design_speed_kn` | absent | 9.7 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `powering.environments[0].energy_per_trip_kwh` | absent | none | changed |
| sweep CN-derived bulk 1 (88x16.3) | `powering.environments[0].fuel_kg_per_h` | absent | 137.431542742 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `powering.environments[0].name` | absent | Deep water | changed |
| sweep CN-derived bulk 1 (88x16.3) | `powering.environments[0].power_at_service_kw` | absent | 704.777142267 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `powering.environments[0].service_speed_kn` | absent | 9.5 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `powering.installed_power_kw` | absent | 888.507056199 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `powering.installed_power_sized` | absent | True | changed |
| sweep CN-derived bulk 1 (88x16.3) | `powering.margin` | absent | 1.2 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[10].kcg_m` | absent | 1.2 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[10].label` | absent | small items | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[10].lcg_m` | absent | 6.69341118947 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[10].mass_t` | absent | 0.888507056199 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[11].kcg_m` | absent | 1.2 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[11].label` | absent | piping and cabling | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[11].lcg_m` | absent | 6.69341118947 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[11].mass_t` | absent | 5.2212676405 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[7].label` | bow thruster | engines | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[8].label` | piping and cabling | gearboxes | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[9].kcg_m` | absent | 1 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[9].label` | absent | bow thruster | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[9].lcg_m` | absent | 78.72 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[9].mass_t` | absent | 3.66374448 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `warnings[0]` | The installed power is not sized yet, so the engines and their room are not in the stern block; type an installed power to place them. | Bulk GM is first-order (assumed lightship KG); cargo-shift/angle-of-repose not modelled — confirm with a full intact-stability booklet. | changed |
| sweep CN-derived bulk 1 (88x16.3) | `warnings[1]` | Bulk GM is first-order (assumed lightship KG); cargo-shift/angle-of-repose not modelled — confirm with a full intact-stability booklet. | absent | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[10].message` | absent | engine 1.65 m + 1.00 m double bottom and deck structure (ESTIMATE) <= moulded depth 5.80 m | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[10].name` | absent | machinery fits under deck | changed |
| ... | 26045 more changed quantities | | | |

### ticket/352-machinery (2026-10-21)

Between `main 5e36bd6` and `this branch 2042246`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived bulk 1 (88x16.3) | `checks[2].message` | cargo zone 57.83 m = LOA 88.0 - bow void 9.78 m - stern void 20.39 m | cargo zone 43.08 m = LOA 88.0 - bow void 9.78 m - stern void 35.14 m | changed |
| sweep CN-derived bulk 1 (88x16.3) | `checks[9].message` | hopper intact GM 2.86 m >= 0.15 m (first-order; KG_light @0.5D) | hopper intact GM 3.67 m >= 0.15 m (first-order; KG_light @0.5D) | changed |
| sweep CN-derived bulk 1 (88x16.3) | `results.governs` | draught — Draught limit | hold volume | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[0].label` | thrusters | battery unit | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[2].label` | battery unit | accommodation unit | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[3].label` | accommodation unit | wheelhouse | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[4].label` | wheelhouse | lift column | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[5].label` | lift column | shaft lines | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[6].label` | small items | rudders and steering gear | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[7].label` | piping and cabling | bow thruster | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[8].kcg_m` | absent | 1.2 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[8].label` | absent | piping and cabling | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[8].lcg_m` | absent | 4.19 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[8].mass_t` | absent | 3 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `warnings[0]` | Bulk GM is first-order (assumed lightship KG); cargo-shift/angle-of-repose not modelled — confirm with a full intact-stability booklet. | The installed power is not sized yet, so the engines and their room are not in the stern block; type an installed power to place them. | changed |
| sweep CN-derived bulk 1 (88x16.3) | `warnings[1]` | absent | Bulk GM is first-order (assumed lightship KG); cargo-shift/angle-of-repose not modelled — confirm with a full intact-stability booklet. | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[2].message` | cargo zone 99.83 m = LOA 130.0 - bow void 9.78 m - stern void 20.39 m | cargo zone 85.08 m = LOA 130.0 - bow void 9.78 m - stern void 35.14 m | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[9].message` | hopper intact GM 2.73 m >= 0.15 m (first-order; KG_light @0.5D) | hopper intact GM 3.09 m >= 0.15 m (first-order; KG_light @0.5D) | changed |
| sweep CN-derived bulk 2 (130x16.3) | `results.governs` | draught — Draught limit | hold volume | changed |
| sweep CN-derived bulk 2 (130x16.3) | `stern_block_mass.items[0].label` | thrusters | battery unit | changed |
| sweep CN-derived bulk 2 (130x16.3) | `stern_block_mass.items[2].label` | battery unit | accommodation unit | changed |
| sweep CN-derived bulk 2 (130x16.3) | `stern_block_mass.items[3].label` | accommodation unit | wheelhouse | changed |
| sweep CN-derived bulk 2 (130x16.3) | `stern_block_mass.items[4].label` | wheelhouse | lift column | changed |
| sweep CN-derived bulk 2 (130x16.3) | `stern_block_mass.items[5].label` | lift column | shaft lines | changed |
| sweep CN-derived bulk 2 (130x16.3) | `stern_block_mass.items[6].label` | small items | rudders and steering gear | changed |
| sweep CN-derived bulk 2 (130x16.3) | `stern_block_mass.items[7].label` | piping and cabling | bow thruster | changed |
| sweep CN-derived bulk 2 (130x16.3) | `stern_block_mass.items[8].kcg_m` | absent | 1.2 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `stern_block_mass.items[8].label` | absent | piping and cabling | changed |
| sweep CN-derived bulk 2 (130x16.3) | `stern_block_mass.items[8].lcg_m` | absent | 4.19 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `stern_block_mass.items[8].mass_t` | absent | 3 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `warnings[0]` | Bulk GM is first-order (assumed lightship KG); cargo-shift/angle-of-repose not modelled — confirm with a full intact-stability booklet. | The installed power is not sized yet, so the engines and their room are not in the stern block; type an installed power to place them. | changed |
| sweep CN-derived bulk 2 (130x16.3) | `warnings[1]` | absent | Bulk GM is first-order (assumed lightship KG); cargo-shift/angle-of-repose not modelled — confirm with a full intact-stability booklet. | changed |
| sweep CN-derived bulk 3 (130x17.2) | `checks[2].message` | cargo zone 99.83 m = LOA 130.0 - bow void 9.78 m - stern void 20.39 m | cargo zone 83.77 m = LOA 130.0 - bow void 9.78 m - stern void 36.45 m | changed |
| sweep CN-derived bulk 3 (130x17.2) | `checks[9].message` | hopper intact GM 3.20 m >= 0.15 m (first-order; KG_light @0.5D) | hopper intact GM 3.65 m >= 0.15 m (first-order; KG_light @0.5D) | changed |
| sweep CN-derived bulk 3 (130x17.2) | `stern_block_mass.items[0].label` | thrusters | battery unit | changed |
| sweep CN-derived bulk 3 (130x17.2) | `stern_block_mass.items[2].label` | battery unit | accommodation unit | changed |
| sweep CN-derived bulk 3 (130x17.2) | `stern_block_mass.items[3].label` | accommodation unit | wheelhouse | changed |
| sweep CN-derived bulk 3 (130x17.2) | `stern_block_mass.items[4].label` | wheelhouse | lift column | changed |
| sweep CN-derived bulk 3 (130x17.2) | `stern_block_mass.items[5].label` | lift column | shaft lines | changed |
| sweep CN-derived bulk 3 (130x17.2) | `stern_block_mass.items[6].label` | small items | rudders and steering gear | changed |
| ... | 32380 more changed quantities | | | |

### 277 (2026-10-21)

Between `main b781747` and `this branch 365fa94`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived bulk 1 (88x16.3) | `checks[8].severity` | warning | error | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[8].severity` | warning | error | changed |
| sweep CN-derived bulk 3 (130x17.2) | `checks[8].severity` | warning | error | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[12].message` | GM 3.08 m (fluid), max GZ 1.15 m at 25° | GM 6.33 m (fluid), max GZ 2.68 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[13].message` | GM 4.65 m (fluid), max GZ 1.96 m at 26° | GM 8.87 m (fluid), max GZ 3.10 m at 30° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[14].message` | GM 8.87 m (fluid), max GZ 2.89 m at 29° | GM 14.89 m (fluid), max GZ 3.61 m at 26° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[8].message` | raised eye 13.35 m >= obstruction 14.43 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 9.64 m >= obstruction 9.14 m + 0.50 m margin (lift used 2.29 m of 6.00 m) | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[8].ok` | False | True | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[8].severity` | warning | error | changed |
| sweep CN-derived liquid 1 (88x16.3) | `warnings[0]` | Sightline needs 7.58 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Separation clauses are cited from the classification rules for inland-waterways ships (Pt 4 Ch 4 Sec 3, Jul 2022). Confirm against the ADN text before release. | changed |
| sweep CN-derived liquid 1 (88x16.3) | `warnings[1]` | Separation clauses are cited from the classification rules for inland-waterways ships (Pt 4 Ch 4 Sec 3, Jul 2022). Confirm against the ADN text before release. | absent | changed |
| sweep CN-derived liquid 2 (130x16.3) | `arrangement.feasible` | True | False | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[10].message` | aft separation 2.00 m >= 2.00 m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11) | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[10].name` | opening setback | cofferdam min width | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | aft separation 2.00 m >= 2.00 m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[11].name` | fwd end bulkhead space | opening setback | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].message` | GM 3.87 m (fluid), max GZ 1.62 m at 26° | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].name` | Stability — laden | fwd end bulkhead space | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].severity` | absent | error | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[13].message` | GM 5.77 m (fluid), max GZ 2.32 m at 27° | GM 3.87 m (fluid), max GZ 1.62 m at 26° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[13].name` | Stability — half laden | Stability — laden | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].message` | GM 10.93 m (fluid), max GZ 3.15 m at 27° | GM 5.77 m (fluid), max GZ 2.32 m at 27° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].name` | Stability — light | Stability — half laden | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[15].message` | absent | GM 10.93 m (fluid), max GZ 3.15 m at 27° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[15].name` | absent | Stability — light | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[15].ok` | absent | True | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[1].message` | cargo volume must be positive (tank/hold fits) | No arrangement passes every check on this hull. The closest one is shown. It fails: wheelhouse_sees_over_cargo. | changed |
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
| ... | 4086 more changed quantities | | | |

### ticket/350-stern-form (2026-10-21)

Between `main b2e943f` and `this branch 3a9a9aa`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived bulk 1 (88x16.3) | `checks[9].message` | hopper intact GM 2.97 m >= 0.15 m (first-order; KG_light @0.5D) | hopper intact GM 2.86 m >= 0.15 m (first-order; KG_light @0.5D) | changed |
| sweep CN-derived bulk 1 (88x16.3) | `geometry.propeller_d_max_m` | absent | 3.44 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `geometry.propeller_diameter_m` | absent | 3.44 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `geometry.stern_form` | absent | tunnel | changed |
| sweep CN-derived bulk 1 (88x16.3) | `geometry.stern_run_length_m` | absent | 34.643 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `geometry.stern_transom_height_m` | absent | 0 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[9].message` | hopper intact GM 2.67 m >= 0.15 m (first-order; KG_light @0.5D) | hopper intact GM 2.73 m >= 0.15 m (first-order; KG_light @0.5D) | changed |
| sweep CN-derived bulk 2 (130x16.3) | `geometry.propeller_d_max_m` | absent | 3.44 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `geometry.propeller_diameter_m` | absent | 3.44 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `geometry.stern_form` | absent | tunnel | changed |
| sweep CN-derived bulk 2 (130x16.3) | `geometry.stern_run_length_m` | absent | 34.643 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `geometry.stern_transom_height_m` | absent | 0 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `checks[9].message` | hopper intact GM 3.13 m >= 0.15 m (first-order; KG_light @0.5D) | hopper intact GM 3.20 m >= 0.15 m (first-order; KG_light @0.5D) | changed |
| sweep CN-derived bulk 3 (130x17.2) | `geometry.propeller_d_max_m` | absent | 3.44 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `geometry.propeller_diameter_m` | absent | 3.44 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `geometry.stern_form` | absent | tunnel | changed |
| sweep CN-derived bulk 3 (130x17.2) | `geometry.stern_run_length_m` | absent | 35.948 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `geometry.stern_transom_height_m` | absent | 0 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[12].message` | GM 3.84 m (fluid), max GZ 1.59 m at 26° | GM 3.08 m (fluid), max GZ 1.15 m at 25° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[13].message` | GM 5.55 m (fluid), max GZ 2.27 m at 27° | GM 4.65 m (fluid), max GZ 1.96 m at 26° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[14].message` | GM 9.80 m (fluid), max GZ 3.06 m at 29° | GM 8.87 m (fluid), max GZ 2.89 m at 29° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[8].message` | raised eye 13.35 m >= obstruction 13.26 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.35 m >= obstruction 14.43 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep CN-derived liquid 1 (88x16.3) | `geometry.propeller_d_max_m` | absent | 2.73 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `geometry.propeller_diameter_m` | absent | 2.73 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `geometry.stern_form` | absent | transom | changed |
| sweep CN-derived liquid 1 (88x16.3) | `geometry.stern_rake_length_m` | absent | 5.7 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `geometry.stern_run_length_m` | absent | 5.7 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `geometry.stern_transom_height_m` | absent | 4.2 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `warnings[0]` | Sightline needs 6.41 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 7.58 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].message` | GM 3.40 m (fluid), max GZ 1.29 m at 26° | GM 3.87 m (fluid), max GZ 1.62 m at 26° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[13].message` | GM 5.04 m (fluid), max GZ 2.12 m at 27° | GM 5.77 m (fluid), max GZ 2.32 m at 27° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].message` | GM 9.71 m (fluid), max GZ 3.05 m at 29° | GM 10.93 m (fluid), max GZ 3.15 m at 27° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `geometry.propeller_d_max_m` | absent | 2.73 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `geometry.propeller_diameter_m` | absent | 2.73 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `geometry.stern_form` | absent | transom | changed |
| sweep CN-derived liquid 2 (130x16.3) | `geometry.stern_rake_length_m` | absent | 5.7 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `geometry.stern_run_length_m` | absent | 5.7 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `geometry.stern_transom_height_m` | absent | 4.2 | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[12].message` | GM 5.38 m (fluid), max GZ 2.15 m at 27° | GM 6.00 m (fluid), max GZ 2.46 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[13].message` | GM 7.71 m (fluid), max GZ 2.96 m at 28° | GM 8.67 m (fluid), max GZ 3.14 m at 28° | changed |
| ... | 25172 more changed quantities | | | |

### ticket/349-trim (2026-10-21)

Between `main f9475c8` and `this branch ba007ac`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived bulk 1 (88x16.3) | `trim[0].draught_aft_m` | absent | 3.53178909759 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `trim[0].draught_fwd_m` | absent | 5.42863608856 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `trim[0].draught_m` | absent | 4.48021259308 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `trim[0].name` | absent | Laden | changed |
| sweep CN-derived bulk 1 (88x16.3) | `trim[0].trim_m` | absent | 1.89684699097 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `trim[1].draught_aft_m` | absent | 1.43019756501 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `trim[1].draught_fwd_m` | absent | 0.944527013849 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `trim[1].draught_m` | absent | 1.18736228943 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `trim[1].name` | absent | Light | changed |
| sweep CN-derived bulk 1 (88x16.3) | `trim[1].trim_m` | absent | -0.48567055116 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `trim[0].draught_aft_m` | absent | 4.28707776911 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `trim[0].draught_fwd_m` | absent | 5.81703447454 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `trim[0].draught_m` | absent | 5.05205612183 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `trim[0].name` | absent | Laden | changed |
| sweep CN-derived bulk 2 (130x16.3) | `trim[0].trim_m` | absent | 1.52995670542 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `trim[1].draught_aft_m` | absent | 1.38150000601 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `trim[1].draught_fwd_m` | absent | 1.02681069345 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `trim[1].draught_m` | absent | 1.20415534973 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `trim[1].name` | absent | Light | changed |
| sweep CN-derived bulk 2 (130x16.3) | `trim[1].trim_m` | absent | -0.354689312554 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `trim[0].draught_aft_m` | absent | 4.3951874033 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `trim[0].draught_fwd_m` | absent | 6.02985150661 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `trim[0].draught_m` | absent | 5.21251945496 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `trim[0].name` | absent | Laden | changed |
| sweep CN-derived bulk 3 (130x17.2) | `trim[0].trim_m` | absent | 1.63466410331 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `trim[1].draught_aft_m` | absent | 1.35678742518 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `trim[1].draught_fwd_m` | absent | 1.02065855871 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `trim[1].draught_m` | absent | 1.18872299194 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `trim[1].name` | absent | Light | changed |
| sweep CN-derived bulk 3 (130x17.2) | `trim[1].trim_m` | absent | -0.336128866468 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stability.conditions[0].draught_aft_m` | absent | 2.65848498747 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stability.conditions[0].draught_fwd_m` | absent | 3.71594955805 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stability.conditions[0].trim_m` | absent | 1.05746457057 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stability.conditions[1].draught_aft_m` | absent | 2.21921463387 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stability.conditions[1].draught_fwd_m` | absent | 2.64475605118 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stability.conditions[1].trim_m` | absent | 0.425541417313 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stability.conditions[2].draught_aft_m` | absent | 1.77994428027 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stability.conditions[2].draught_fwd_m` | absent | 1.57356254432 | changed |
| sweep CN-derived liquid 1 (88x16.3) | `stability.conditions[2].trim_m` | absent | -0.206381735949 | changed |
| sweep CN-derived liquid 2 (130x16.3) | `stability.conditions[0].draught_aft_m` | absent | 3.07426959839 | changed |
| ... | 920 more changed quantities | | | |

### ticket/348-stern-block-mass (2026-10-21)

Between `main c5361a0` and `this branch 4a10aa4`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived bulk 1 (88x16.3) | `checks[9].message` | hopper intact GM 3.07 m >= 0.15 m (first-order; KG_light @0.5D) | hopper intact GM 2.97 m >= 0.15 m (first-order; KG_light @0.5D) | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[0].kcg_m` | absent | 1.92592592593 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[0].label` | absent | thrusters | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[0].lcg_m` | absent | 1.75 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[0].mass_t` | absent | 32.1463191587 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[1].kcg_m` | absent | 7.548 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[1].label` | absent | battery unit | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[1].lcg_m` | absent | 9.896 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[1].mass_t` | absent | 30 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[2].kcg_m` | absent | 7.548 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[2].label` | absent | battery unit | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[2].lcg_m` | absent | 9.896 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[2].mass_t` | absent | 30 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[3].kcg_m` | absent | 7.548 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[3].label` | absent | accommodation unit | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[3].lcg_m` | absent | 9.896 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[3].mass_t` | absent | 14.6337669427 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[4].kcg_m` | absent | 7.5 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[4].label` | absent | wheelhouse | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[4].lcg_m` | absent | 18.092 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[4].mass_t` | absent | 8 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[5].kcg_m` | absent | 5.8 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[5].label` | absent | lift column | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[5].lcg_m` | absent | 18.092 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[5].mass_t` | absent | 5 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[6].kcg_m` | absent | 1.2 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[6].label` | absent | small items | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[6].lcg_m` | absent | 9.946 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[6].mass_t` | absent | 0 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[7].kcg_m` | absent | 1.2 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[7].label` | absent | piping and cabling | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[7].lcg_m` | absent | 9.946 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.items[7].mass_t` | absent | 3 | changed |
| sweep CN-derived bulk 1 (88x16.3) | `stern_block_mass.total_t` | absent | 122.780086101 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[9].message` | hopper intact GM 2.72 m >= 0.15 m (first-order; KG_light @0.5D) | hopper intact GM 2.67 m >= 0.15 m (first-order; KG_light @0.5D) | changed |
| sweep CN-derived bulk 2 (130x16.3) | `stern_block_mass.items[0].kcg_m` | absent | 1.92592592593 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `stern_block_mass.items[0].label` | absent | thrusters | changed |
| sweep CN-derived bulk 2 (130x16.3) | `stern_block_mass.items[0].lcg_m` | absent | 1.75 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `stern_block_mass.items[0].mass_t` | absent | 32.1463191587 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `stern_block_mass.items[1].kcg_m` | absent | 7.548 | changed |
| ... | 26252 more changed quantities | | | |

### ticket/347-stern-inputs (2026-10-21)

Between `main 6bdb1d8` and `this branch c5a0143`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived bulk 1 (88x16.3) | `checks[8].message` | raised eye 7.30 m >= obstruction 6.80 m + 0.50 m margin (lift used 0.00 m of 6.00 m) | raised eye 7.45 m >= obstruction 6.80 m + 0.50 m margin (lift used 0.00 m of 6.00 m) | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[8].message` | raised eye 7.30 m >= obstruction 6.80 m + 0.50 m margin (lift used 0.00 m of 6.00 m) | raised eye 7.45 m >= obstruction 6.80 m + 0.50 m margin (lift used 0.00 m of 6.00 m) | changed |
| sweep CN-derived bulk 3 (130x17.2) | `checks[8].message` | raised eye 7.30 m >= obstruction 6.80 m + 0.50 m margin (lift used 0.00 m of 6.00 m) | raised eye 7.45 m >= obstruction 6.80 m + 0.50 m margin (lift used 0.00 m of 6.00 m) | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[8].message` | raised eye 13.20 m >= obstruction 13.26 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.35 m >= obstruction 13.26 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep CN-derived liquid 1 (88x16.3) | `warnings[0]` | Sightline needs 6.56 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.41 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[8].message` | raised eye 13.20 m >= obstruction 12.94 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.35 m >= obstruction 12.94 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `warnings[0]` | Sightline needs 6.24 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.09 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[8].message` | raised eye 10.13 m >= obstruction 9.63 m + 0.50 m margin (lift used 2.93 m of 6.00 m) | raised eye 10.13 m >= obstruction 9.63 m + 0.50 m margin (lift used 2.78 m of 6.00 m) | changed |
| sweep box self-propelled 60x10 | `checks[8].message` | raised eye 6.00 m >= obstruction 5.50 m + 0.50 m margin (lift used 0.00 m of 6.00 m) | raised eye 6.15 m >= obstruction 5.50 m + 0.50 m margin (lift used 0.00 m of 6.00 m) | changed |
| sweep box self-propelled 90x14 | `checks[8].message` | raised eye 6.00 m >= obstruction 5.50 m + 0.50 m margin (lift used 0.00 m of 6.00 m) | raised eye 6.15 m >= obstruction 5.50 m + 0.50 m margin (lift used 0.00 m of 6.00 m) | changed |
| sweep default depth dry bulk 90x14 | `checks[8].message` | raised eye 6.12 m >= obstruction 5.62 m + 0.50 m margin (lift used 0.00 m of 6.00 m) | raised eye 6.27 m >= obstruction 5.62 m + 0.50 m margin (lift used 0.00 m of 6.00 m) | changed |
| sweep default depth gas 90x14 | `checks[8].message` | raised eye 15.20 m >= obstruction 15.59 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 15.35 m >= obstruction 15.59 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep default depth gas 90x14 | `warnings[0]` | Sightline needs 6.89 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.74 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep gas IGC_2G 60x10 | `checks[8].message` | raised eye 11.13 m >= obstruction 10.63 m + 0.50 m margin (lift used 4.13 m of 6.00 m) | raised eye 11.13 m >= obstruction 10.63 m + 0.50 m margin (lift used 3.98 m of 6.00 m) | changed |
| sweep gas IGC_2G 90x14 | `checks[8].message` | raised eye 13.00 m >= obstruction 12.86 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.15 m >= obstruction 12.86 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep gas IGC_2G 90x14 | `warnings[0]` | Sightline needs 6.36 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.21 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep gas IGC_2G_EXACT 60x10 | `checks[8].message` | raised eye 11.13 m >= obstruction 10.63 m + 0.50 m margin (lift used 4.13 m of 6.00 m) | raised eye 11.13 m >= obstruction 10.63 m + 0.50 m margin (lift used 3.98 m of 6.00 m) | changed |
| sweep gas IGC_2G_EXACT 90x14 | `checks[8].message` | raised eye 13.00 m >= obstruction 12.86 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.15 m >= obstruction 12.86 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep gas IGC_2G_EXACT 90x14 | `warnings[0]` | Sightline needs 6.36 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.21 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep gas cargo LNG | `checks[8].message` | raised eye 13.00 m >= obstruction 14.39 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.15 m >= obstruction 14.39 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep gas cargo LNG | `warnings[0]` | Sightline needs 7.89 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 7.74 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep gas cargo LNG_ethane | `checks[8].message` | raised eye 13.00 m >= obstruction 14.39 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.15 m >= obstruction 14.39 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep gas cargo LNG_ethane | `warnings[0]` | Sightline needs 7.89 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 7.74 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep gas cargo LPG_butane | `checks[8].message` | raised eye 13.00 m >= obstruction 12.86 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.15 m >= obstruction 12.86 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep gas cargo LPG_butane | `warnings[0]` | Sightline needs 6.36 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.21 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep gas cargo LPG_mix | `checks[8].message` | raised eye 13.00 m >= obstruction 12.86 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.15 m >= obstruction 12.86 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep gas cargo LPG_mix | `warnings[0]` | Sightline needs 6.36 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.21 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep gas cargo LPG_propane | `checks[8].message` | raised eye 13.00 m >= obstruction 12.86 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.15 m >= obstruction 12.86 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep gas cargo LPG_propane | `warnings[0]` | Sightline needs 6.36 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.21 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep gas carriage LPG_butane fully_pressurised | `checks[8].message` | raised eye 11.46 m >= obstruction 10.96 m + 0.50 m margin (lift used 3.96 m of 6.00 m) | raised eye 11.46 m >= obstruction 10.96 m + 0.50 m margin (lift used 3.81 m of 6.00 m) | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[9].message` | raised eye 12.86 m >= obstruction 12.36 m + 0.50 m margin (lift used 5.36 m of 6.00 m) | raised eye 12.86 m >= obstruction 12.36 m + 0.50 m margin (lift used 5.21 m of 6.00 m) | changed |
| sweep gas carriage LPG_butane semi_pressurised | `checks[8].message` | raised eye 11.66 m >= obstruction 11.16 m + 0.50 m margin (lift used 4.16 m of 6.00 m) | raised eye 11.66 m >= obstruction 11.16 m + 0.50 m margin (lift used 4.01 m of 6.00 m) | changed |
| sweep gas carriage LPG_mix fully_pressurised | `checks[8].message` | raised eye 11.46 m >= obstruction 10.96 m + 0.50 m margin (lift used 3.96 m of 6.00 m) | raised eye 11.46 m >= obstruction 10.96 m + 0.50 m margin (lift used 3.81 m of 6.00 m) | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[9].message` | raised eye 12.86 m >= obstruction 12.36 m + 0.50 m margin (lift used 5.36 m of 6.00 m) | raised eye 12.86 m >= obstruction 12.36 m + 0.50 m margin (lift used 5.21 m of 6.00 m) | changed |
| sweep gas carriage LPG_mix semi_pressurised | `checks[8].message` | raised eye 11.66 m >= obstruction 11.16 m + 0.50 m margin (lift used 4.16 m of 6.00 m) | raised eye 11.66 m >= obstruction 11.16 m + 0.50 m margin (lift used 4.01 m of 6.00 m) | changed |
| sweep gas carriage LPG_propane fully_pressurised | `checks[8].message` | raised eye 11.46 m >= obstruction 10.96 m + 0.50 m margin (lift used 3.96 m of 6.00 m) | raised eye 11.46 m >= obstruction 10.96 m + 0.50 m margin (lift used 3.81 m of 6.00 m) | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[9].message` | raised eye 12.86 m >= obstruction 12.36 m + 0.50 m margin (lift used 5.36 m of 6.00 m) | raised eye 12.86 m >= obstruction 12.36 m + 0.50 m margin (lift used 5.21 m of 6.00 m) | changed |
| sweep gas carriage LPG_propane semi_pressurised | `checks[8].message` | raised eye 11.66 m >= obstruction 11.16 m + 0.50 m margin (lift used 4.16 m of 6.00 m) | raised eye 11.66 m >= obstruction 11.16 m + 0.50 m margin (lift used 4.01 m of 6.00 m) | changed |
| sweep gas carries nothing (draught limit) | `checks[9].message` | raised eye 13.00 m >= obstruction 15.20 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.15 m >= obstruction 15.20 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep gas carries nothing (draught limit) | `warnings[0]` | Sightline needs 8.70 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 8.55 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| ... | 109 more changed quantities | | | |

### 281 (2026-10-21)

Between `main 0edcb52` and `this branch 920c302`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep default depth gas 90x14 | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas IGC_2G 60x10 | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas IGC_2G 90x14 | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas IGC_2G_EXACT 60x10 | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas IGC_2G_EXACT 90x14 | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas cargo LNG | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas cargo LNG pushed | `checks[5].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas cargo LNG_ethane | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas cargo LNG_ethane pushed | `checks[5].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas cargo LPG_butane | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas cargo LPG_butane pushed | `checks[5].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas cargo LPG_mix | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas cargo LPG_mix pushed | `checks[5].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas cargo LPG_propane | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas cargo LPG_propane pushed | `checks[5].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas carriage LPG_butane fully_pressurised | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas carriage LPG_butane fully_refrigerated | `checks[12].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas carriage LPG_butane semi_pressurised | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas carriage LPG_mix fully_pressurised | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas carriage LPG_mix fully_refrigerated | `checks[12].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas carriage LPG_mix semi_pressurised | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas carriage LPG_propane fully_pressurised | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas carriage LPG_propane fully_refrigerated | `checks[12].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas carriage LPG_propane semi_pressurised | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas carries nothing (draught limit) | `checks[12].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas carries nothing (hull too short) | `checks[12].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas channel 60x10 | `checks[5].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas channel 90x14 | `checks[5].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas passes all checks IGC_2G 110x11.4 | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas passes all checks IGC_2G_EXACT 80x14 | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas tank longer than 7 diameters IGC_2G 200x12 | `checks[12].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas wide hull IGC_2G 110x24 | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep gas wide hull IGC_2G_EXACT 110x24 | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep optional accommodation_unit_count | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep optional battery_rate_per_kwh | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep optional battery_unit_count | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| sweep optional cargo_system_lump | `checks[11].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | forward end-bulkhead space 0.50 m >= 0.20 m (domed tank ends, cl. 3.5.3) | changed |
| ... | 49 more changed quantities | | | |

### ticket/375-head-type (2026-10-21)

Between `main db9e179` and `this branch ff8158d`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `checks[12].message` | GM 3.58 m (fluid), max GZ 1.44 m at 26° | GM 4.01 m (fluid), max GZ 1.69 m at 26° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[13].message` | GM 5.47 m (fluid), max GZ 2.25 m at 27° | GM 5.87 m (fluid), max GZ 2.36 m at 27° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[14].message` | GM 10.33 m (fluid), max GZ 3.13 m at 29° | GM 10.56 m (fluid), max GZ 3.15 m at 28° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[8].message` | raised eye 13.20 m >= obstruction 14.47 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.20 m >= obstruction 13.26 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep CN-derived liquid 1 (88x16.3) | `warnings[0]` | Sightline needs 7.77 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.56 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].message` | GM 3.79 m (fluid), max GZ 1.55 m at 26° | GM 3.49 m (fluid), max GZ 1.36 m at 26° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[13].message` | GM 5.71 m (fluid), max GZ 2.33 m at 27° | GM 5.23 m (fluid), max GZ 2.18 m at 27° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].message` | GM 11.02 m (fluid), max GZ 3.21 m at 28° | GM 10.21 m (fluid), max GZ 3.11 m at 29° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[8].message` | raised eye 13.20 m >= obstruction 12.97 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 13.20 m >= obstruction 12.94 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | changed |
| sweep CN-derived liquid 2 (130x16.3) | `warnings[0]` | Sightline needs 6.27 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Sightline needs 6.24 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[12].message` | GM 3.77 m (fluid), max GZ 1.28 m at 25° | GM 5.52 m (fluid), max GZ 2.22 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[13].message` | GM 5.72 m (fluid), max GZ 2.28 m at 26° | GM 7.99 m (fluid), max GZ 3.02 m at 28° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[14].message` | GM 11.24 m (fluid), max GZ 3.38 m at 29° | GM 14.91 m (fluid), max GZ 3.82 m at 27° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[8].message` | raised eye 13.20 m >= obstruction 14.01 m + 0.50 m margin (lift used 6.00 m of 6.00 m) | raised eye 10.13 m >= obstruction 9.63 m + 0.50 m margin (lift used 2.93 m of 6.00 m) | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[8].ok` | False | True | changed |
| sweep CN-derived liquid 3 (130x17.2) | `warnings[0]` | Sightline needs 7.31 m of lift; the column gives 6.00 m. Raise the column or lower the cargo top. | Separation clauses are cited from the classification rules for inland-waterways ships (Pt 4 Ch 4 Sec 3, Jul 2022). Confirm against the ADN text before release. | changed |
| sweep CN-derived liquid 3 (130x17.2) | `warnings[1]` | Separation clauses are cited from the classification rules for inland-waterways ships (Pt 4 Ch 4 Sec 3, Jul 2022). Confirm against the ADN text before release. | absent | changed |
| sweep gas IGC_2G 60x10 | `arrangement.feasible` | False | True | changed |
| sweep gas IGC_2G 60x10 | `checks[10].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft separation 2.00 m >= 2.00 m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11) | changed |
| sweep gas IGC_2G 60x10 | `checks[10].name` | cofferdam min width | opening setback | changed |
| sweep gas IGC_2G 60x10 | `checks[11].message` | aft separation 2.00 m >= 2.00 m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11) | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | changed |
| sweep gas IGC_2G 60x10 | `checks[11].name` | opening setback | fwd end bulkhead space | changed |
| sweep gas IGC_2G 60x10 | `checks[12].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | GM 2.55 m (fluid), max GZ 1.15 m at 32° | changed |
| sweep gas IGC_2G 60x10 | `checks[12].name` | fwd end bulkhead space | Stability — laden | changed |
| sweep gas IGC_2G 60x10 | `checks[12].severity` | error | absent | changed |
| sweep gas IGC_2G 60x10 | `checks[13].message` | GM 6.41 m (fluid), max GZ 1.86 m at 27° | GM 3.44 m (fluid), max GZ 1.27 m at 28° | changed |
| sweep gas IGC_2G 60x10 | `checks[13].name` | Stability — laden | Stability — half laden | changed |
| sweep gas IGC_2G 60x10 | `checks[14].message` | GM 7.29 m (fluid), max GZ 1.91 m at 25° | GM 4.92 m (fluid), max GZ 1.49 m at 25° | changed |
| sweep gas IGC_2G 60x10 | `checks[14].name` | Stability — half laden | Stability — light | changed |
| sweep gas IGC_2G 60x10 | `checks[15].message` | GM 8.46 m (fluid), max GZ 1.97 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | absent | changed |
| sweep gas IGC_2G 60x10 | `checks[15].name` | Stability — light | absent | changed |
| sweep gas IGC_2G 60x10 | `checks[15].ok` | False | absent | changed |
| sweep gas IGC_2G 60x10 | `checks[1].message` | No arrangement passes every check on this hull. The closest one is shown. It fails: Stability — light. | cargo volume must be positive (tank/hold fits) | changed |
| sweep gas IGC_2G 60x10 | `checks[1].name` | Feasible arrangement | feasible geometry | changed |
| sweep gas IGC_2G 60x10 | `checks[1].ok` | False | True | changed |
| sweep gas IGC_2G 60x10 | `checks[1].severity` | absent | error | changed |
| sweep gas IGC_2G 60x10 | `checks[2].message` | cargo volume must be positive (tank/hold fits) | cargo zone 15.82 m = LOA 60.0 - bow void 9.30 m - stern void 34.88 m | changed |
| sweep gas IGC_2G 60x10 | `checks[2].name` | feasible geometry | cargo zone positive | changed |
| sweep gas IGC_2G 60x10 | `checks[3].message` | cargo zone 15.82 m = LOA 60.0 - bow void 9.30 m - stern void 34.88 m | 2 x 2.60 m thrusters need 7.00 m across a 10.00 m beam | changed |
| sweep gas IGC_2G 60x10 | `checks[3].name` | cargo zone positive | thrusters fit beam | changed |
| ... | 21856 more changed quantities | | | |

### ticket/374-liquid-pressure (2026-10-21)

Between `main abf1300` and `this branch 73ad0f6`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived liquid 1 (88x16.3) | `checks[12].message` | GM 3.62 m (fluid), max GZ 1.46 m at 26° | GM 3.58 m (fluid), max GZ 1.44 m at 26° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[13].message` | GM 5.53 m (fluid), max GZ 2.27 m at 27° | GM 5.47 m (fluid), max GZ 2.25 m at 27° | changed |
| sweep CN-derived liquid 1 (88x16.3) | `checks[14].message` | GM 10.49 m (fluid), max GZ 3.15 m at 28° | GM 10.33 m (fluid), max GZ 3.13 m at 29° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[12].message` | GM 3.82 m (fluid), max GZ 1.56 m at 26° | GM 3.79 m (fluid), max GZ 1.55 m at 26° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[13].message` | GM 5.77 m (fluid), max GZ 2.34 m at 27° | GM 5.71 m (fluid), max GZ 2.33 m at 27° | changed |
| sweep CN-derived liquid 2 (130x16.3) | `checks[14].message` | GM 11.17 m (fluid), max GZ 3.23 m at 28° | GM 11.02 m (fluid), max GZ 3.21 m at 28° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[12].message` | GM 3.81 m (fluid), max GZ 1.31 m at 25° | GM 3.77 m (fluid), max GZ 1.28 m at 25° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[13].message` | GM 5.79 m (fluid), max GZ 2.31 m at 26° | GM 5.72 m (fluid), max GZ 2.28 m at 26° | changed |
| sweep CN-derived liquid 3 (130x17.2) | `checks[14].message` | GM 11.44 m (fluid), max GZ 3.40 m at 29° | GM 11.24 m (fluid), max GZ 3.38 m at 29° | changed |
| sweep default depth gas 90x14 | `arrangement.feasible` | False | True | changed |
| sweep default depth gas 90x14 | `checks[10].message` | cofferdam 0.60 m >= 0.60 m (cl. 3.6.13) | aft separation 2.00 m >= 2.00 m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11) | changed |
| sweep default depth gas 90x14 | `checks[10].name` | cofferdam min width | opening setback | changed |
| sweep default depth gas 90x14 | `checks[11].message` | aft separation 2.00 m >= 2.00 m from the cargo area to any entrance or openable window (cl. 3.3.7 / 3.3.11) | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | changed |
| sweep default depth gas 90x14 | `checks[11].name` | opening setback | fwd end bulkhead space | changed |
| sweep default depth gas 90x14 | `checks[12].message` | forward end-bulkhead space 0.50 m >= 0.20 m (pressure tank, cl. 3.5.3 / 3.6.3) | GM 1.39 m (fluid), max GZ 1.17 m at 37° | changed |
| sweep default depth gas 90x14 | `checks[12].name` | fwd end bulkhead space | Stability — laden | changed |
| sweep default depth gas 90x14 | `checks[12].severity` | error | absent | changed |
| sweep default depth gas 90x14 | `checks[13].message` | GM 1.42 m (fluid), max GZ 1.19 m at 37° | GM 2.63 m (fluid), max GZ 1.36 m at 33° | changed |
| sweep default depth gas 90x14 | `checks[13].name` | Stability — laden | Stability — half laden | changed |
| sweep default depth gas 90x14 | `checks[14].message` | GM 2.70 m (fluid), max GZ 1.37 m at 33° | GM 7.72 m (fluid), max GZ 2.23 m at 25° | changed |
| sweep default depth gas 90x14 | `checks[14].name` | Stability — half laden | Stability — light | changed |
| sweep default depth gas 90x14 | `checks[15].message` | GM 7.92 m (fluid), max GZ 2.26 m at 24° — fails: Angle of max GZ 24.000 < 25.0 | absent | changed |
| sweep default depth gas 90x14 | `checks[15].name` | Stability — light | absent | changed |
| sweep default depth gas 90x14 | `checks[15].ok` | False | absent | changed |
| sweep default depth gas 90x14 | `checks[1].message` | No arrangement passes every check on this hull. The closest one is shown. It fails: Stability — light. | cargo volume must be positive (tank/hold fits) | changed |
| sweep default depth gas 90x14 | `checks[1].name` | Feasible arrangement | feasible geometry | changed |
| sweep default depth gas 90x14 | `checks[1].ok` | False | True | changed |
| sweep default depth gas 90x14 | `checks[1].severity` | absent | error | changed |
| sweep default depth gas 90x14 | `checks[2].message` | cargo volume must be positive (tank/hold fits) | cargo zone 55.29 m = LOA 90.0 - bow void 12.82 m - stern void 21.89 m | changed |
| sweep default depth gas 90x14 | `checks[2].name` | feasible geometry | cargo zone positive | changed |
| sweep default depth gas 90x14 | `checks[3].message` | cargo zone 55.29 m = LOA 90.0 - bow void 12.82 m - stern void 21.89 m | 3 x 2.60 m thrusters need 10.20 m across a 14.00 m beam | changed |
| sweep default depth gas 90x14 | `checks[3].name` | cargo zone positive | thrusters fit beam | changed |
| sweep default depth gas 90x14 | `checks[4].message` | 3 x 2.60 m thrusters need 10.20 m across a 14.00 m beam | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 14.00 m beam | changed |
| sweep default depth gas 90x14 | `checks[4].name` | thrusters fit beam | requested thruster count fits | changed |
| sweep default depth gas 90x14 | `checks[4].severity` | error | warning | changed |
| sweep default depth gas 90x14 | `checks[5].message` | 3 of 3 requested thrusters fit; 3 would need 10.20 m of 14.00 m beam | 3 x 40 ft units abreast need 10.91 m across a 14.00 m beam | changed |
| sweep default depth gas 90x14 | `checks[5].name` | requested thruster count fits | containers fit beam | changed |
| sweep default depth gas 90x14 | `checks[5].severity` | warning | error | changed |
| sweep default depth gas 90x14 | `checks[6].message` | 3 x 40 ft units abreast need 10.91 m across a 14.00 m beam | 3 units in 1 row(s) of 3; a second row adds 12.99 m to the stern block and comes straight off the cargo zone | changed |
| sweep default depth gas 90x14 | `checks[6].name` | containers fit beam | units fit in one row | changed |
| ... | 23947 more changed quantities | | | |

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
