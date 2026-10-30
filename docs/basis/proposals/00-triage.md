# Triage: engineering basis proposals 01 to 12

Not published. This file merges the Errors, Gaps and Changes of `proposals/01-…` to `proposals/12-…` into one
ranked list. A maintainer can accept it in one pass. Builder lanes can turn each item into a ticket.

OVD is for concept and preliminary design only, never for approval (ADR 0011).

## How to read this

- **Code state.** Checked against `origin/main` at `b2e943f` (2026-10-21, after #389). Line numbers marked
  "main" were read at that commit. Line numbers marked "draft" come from the proposals (worktree, 2026-10-20) and
  were not read again.
- **Area tags.** `01 E4` means proposals file 01, error E4. `07 C2` means file 07, change C2. `05 G9` means file
  05, gap 9. Page row ids (for example `WC-A29`) refer to the page tables.
- **Type.**
  - **Finding**: the code disagrees with a cited source, with its own basis, or with physics.
  - **Change**: a code improvement.
  - **Docs**: a docs-only fix.
  - **Expert question**: needs a domain reviewer before any change.
- **Severity** (findings only): **non-conservative** (unsafe side), **conservative**, **mixed**, **unknown**, or
  **none** (no number moves).
- **Verified.**
  - **Verified (re-read)**: the triage read the source text again for this file.
  - **Verified (agent)**: the drafting agent says it read the source text. A second reader has not.
  - **Verified (geometry)** or **Verified (code)**: the proof is arithmetic or code against code. No external text
    is needed.
  - **Needs source**: rests on a derivation or on memory. Get the source before the finding goes to #121.
- **Coverage.** **Covered** (ticket number), **partly covered** (what is missing), or **new**.
- **Effect.** None, small (under about 5 % on the output), or large. The effect sizes are the proposals'
  estimates. Nothing was run for this triage.
- **Lanes.** Rules builder, tank and stern builder, UI builder, crude oil, or a new **basis fixes** builder.

## Counts

| Group | Count |
|---|---|
| Ranked items (new or partly covered) | 50 |
| Findings | 22 |
| Findings, non-conservative, verified (one of them mixed) | 9 |
| Findings, non-conservative, needs source | 4 |
| Findings, unknown direction | 4 |
| Findings, conservative (one of them mixed, design grid only) | 3 |
| Findings, no number moves | 2 |
| Changes | 22 |
| Docs | 2, plus the per-page list in "Docs before publishing" |
| Expert questions (stand-alone) | 4. Six more items carry an expert question inside them |
| Coverage: new | 29 |
| Coverage: partly covered | 21 |
| Already fixed or fully covered (not ranked; see the last table) | 31 rows |

## Ranked list

Order: (1) non-conservative findings, verified; (2) non-conservative findings that need a source; (3) changes,
and one finding, that unblock the most; (4) the rest, by effect.

### Tier 1: non-conservative findings, verified

#### 1. Midship area uses the block coefficient

- **Areas:** 01 E4, 01 G4, 01 C7; 07 E3, 07 C2; 08 E2, 08 C2.
- **Type:** Finding. **Severity:** non-conservative. **Verified (re-read):** ES-TRIN 2025/1, Art. 1.01, 4.9
  defines the block coefficient on displaced volume. The midship area is B·T·Cm by definition.
- **Evidence (main):** `crates/ovd-core/src/resistance.rs:367-369`, `midship_section_area = B × T × Cb`. Used in
  the limiting speed (`:378`) and in the blockage term (`:412-418`). `crates/ovd-core/src/squat.rs:143` uses B × T.
- **Effect:** large in confined water. With Cb 0.85 the blockage ratio n is 15 % low. The limiting speed is too
  high and the blockage resistance too low. One hull gets two blockage ratios in one run. Outputs:
  `limiting_speed_kn`, channel speed curves, "channel" governing label, service speed caps.
- **Coverage:** new.
- **Ticket scope:** step 1, one midship-area function for squat and resistance, Am = B·T (no new input). Step 2
  (in batch 6), a midship coefficient input, an Estimate near 1 for barges. Test: n is the same in both modules.
- **Dependencies:** none for step 1. Land with item 10 as one design change (both proposals ask for this).
- **Lane:** basis fixes (batch 1).

#### 2. Dry bulk GM check is missing for heavy cargo and runs at the wrong load

- **Areas:** 06 E1, 06 C1; 03 E7; 09 E10, 09 C9.
- **Type:** Finding. **Severity:** non-conservative (a stability check that does not appear). **Verified
  (agent):** IS Code 2008 (MSC.267(85)) Part A 2.2.4, GM0 not less than 0.15 m.
- **Evidence (main):** `crates/ovd-core/src/barge_capacity.rs:1270-1290`. The check runs only when
  `!overloaded`, and at the volume-bound load. The Design reports `min(volume, weight)` cargo. For a dense cargo
  the volume-bound load sinks the hull, so no GM figure appears at all. The name `gm_positive` tests GM ≥ 0.15 m.
- **Effect:** none on capacity. Large on what the user is told: a dry bulk Design can show no stability figure.
- **Coverage:** partly covered. #272 encoded it as `ovd/capacity/hopper-gm` (warn). Missing: run it at the cargo
  the Design reports, always; rename; cite 2.2.4 as the origin of the value.
- **Ticket scope:** compute `bulk_gm` at the reported cargo in every dry bulk Option. Rename the check. Keep it
  `warn` until a dry bulk Rule Set exists.
- **Dependencies:** none. Touches `barge_capacity.rs` (stern lane, #350 and #352) and `simulate.rs` (rules lane).
- **Lane:** rules builder (batch 5), sequenced after #350.

#### 3. Loading conditions are not the IS Code set; consumables have no mass

- **Areas:** 03 E2, 03 G3, 03 G4, 03 C6; 02 WC-E3, 02 WC-G5, 02 WC-C6; 06 G6, 06 C8; 09 G9; 07 gap 10.
- **Type:** Finding. **Severity:** non-conservative (likely for arrival and ballast conditions). **Verified:**
  IS Code 2008 Introduction 2.23 (re-read: lightship is without consumables and stores). Part A 2.1.1, 2.1.2 and
  Part B 3.4.1.2, 3.1.4 (agent).
- **Evidence (main):** `crates/ovd-core/src/models.rs:147`, `:227` (`stores_weight` 200 t, documented as diesel,
  water, provisions); `crates/ovd-core/src/weights.rs:269` (`stores: 0.0`). No loading condition carries stores,
  fuel or water. The gas stability has three cargo conditions only.
- **Effect:** small to large. Cargo is high by the consumables mass. Arrival (10 % consumables) and ballast
  conditions are not checked. Consumables free surface is not in GM.
- **Coverage:** partly covered. #332 adds standard loading conditions for liquid bulk only.
- **Ticket scope:** a stores and fuel allowance input (Defaults Profile, with basis); departure, arrival and
  ballast conditions for gas and dry bulk; consumables free surface. Remove the inert `stores_weight` or wire it.
  Before and after table.
- **Dependencies:** the allowance input needs format v3 (batch 6). Reuse the #332 condition shape.
- **Lane:** basis fixes, after #332 sets the shape. **Expert question:** the default allowance for inland service.

#### 4. Gas hull steel KG uses D; the steel mass uses the deeper D_m

- **Areas:** 02 WC-E4, 02 WC-C7.
- **Type:** Finding. **Severity:** non-conservative, small. **Verified (code):** `weights.rs` takes the moulded
  depth as the wing plate level; the KG item does not follow.
- **Evidence (main):** `crates/ovd-core/src/weights.rs:232-238` (moulded depth = max(D, wing plate level) when a
  wing tank exists); `crates/ovd-core/src/gas_stability.rs:254` (`steel_top_down` at 0.5 × D).
- **Effect:** small. KG is low when the wing plate is above D, which is common with the default wing tank height
  ratio 0.66. GM is high.
- **Coverage:** new.
- **Ticket scope:** put the top-down steel KG at 0.5 × D_m. Design-impact table.
- **Dependencies:** none. `gas_stability.rs` was last edited by the stern lane (#348, #349).
- **Lane:** tank and stern builder (batch 4).

#### 5. 2:1 head length counts half the insulation (now live for LPG)

- **Areas:** 04 E3, 04 change order 2 (second row).
- **Type:** Finding. **Severity:** non-conservative on tank volume, small. **Verified (geometry).**
- **Evidence (main):** `crates/ovd-core/src/tanks.rs:206` (`head_length_outer = outer_radius / 2.0`, with the
  insulated radius); `:227` (cylinder = overall − 2 × head length). The steel head is R_s/2 deep plus the
  insulation t_ins, so the true length is R_s/2 + t_ins. The hemisphere branch (`:202`) is right.
- **Effect:** small. The cylinder is t_ins too long per head. Before #375 this reached no Design. Since #383 the
  LPG default is 2:1 heads, so it reaches semi-pressurised and refrigerated LPG Designs (fully pressurised LPG has
  no insulation). Outputs: tank volume, cargo.
- **Coverage:** new.
- **Ticket scope:** head length R_s/2 + t_ins for 2:1 heads. Test against a hand case. Design-impact table.
- **Dependencies:** none.
- **Lane:** tank and stern builder (batch 4).

#### 6. Tank weight leaves out the dome and the forming allowance

- **Areas:** 04 E5; 03 G11 (dome part), 03 C8; page 04 step 8 ("The dome has no weight").
- **Type:** Finding. **Severity:** non-conservative, small (light tanks; a missing high mass lowers KG).
  **Verified (agent):** IGC 4.23.2.1.1, the calculated thickness is a minimum after forming, without negative
  tolerance. **Verified (code):** no dome item in `weights.rs` on main.
- **Evidence (main):** `crates/ovd-core/src/weights.rs` has no dome mass. Tank steel is weighed at the required
  thickness (`tanks.rs`, draft line 216-219).
- **Effect:** small. Tank weight low; lightship and KG low.
- **Coverage:** partly covered. #317 adds tank weight calibration records, but no runnable case and no part model.
- **Ticket scope:** add a dome mass item (diameter and height from #316, wall an Estimate) at its height; weigh
  plate at required thickness plus a forming allowance (Estimate input). Fold into item 39 if that lands first.
- **Dependencies:** none for the dome item.
- **Lane:** tank and stern builder (batch 4).

#### 7. Legacy API chain gives wrong stability and geometry

- **Areas:** 03 E5, 03 E6, 03 E8, 03 C3; 04 E6, 04 change order 6; 06 E3, 06 C3; 01 E5, 01 C5; 02 WC-E5;
  05 E11; 08 E4, 08 C6; 11 E5, 11 C7.
- **Type:** Finding. **Severity:** non-conservative, large, but API only (no Design uses it). **Verified
  (geometry, code).**
- **Evidence (main):**
  - `crates/ovd-core/src/tanks.rs:307-315`: `h = fill_ratio × 2r`, a volume ratio read as a height ratio. At 0.98
    fill the free surface moment is about a quarter of the true value. Used by legacy
    `hydrostatics::calculate_stability`, which the Python API exposes.
  - `crates/ovd-core/src/hydrostatics.rs:70`, `:198`: Cb applied to a raked-box volume (counts the form twice).
  - `crates/ovd-core/src/ga_adapter.rs:166` (cylinder holds the whole volume, so the heads count twice) and
    `:247-251` (`min_freeboard_ok`, `air_draft_ok`, `stability_ok` always `true`).
  - `crates/ovd-core/src/models.rs:411`, `:418`: doc comments promise checks no code makes.
  - Legacy KG items do not sum to the lightship (02 WC-E5, draft lines). API clearance check ignores insulation
    (05 E11, conservative, draft lines).
- **Effect:** none on Designs. Large on API answers (GM_fluid too high).
- **Coverage:** new.
- **Ticket scope:** deprecate `calculate_stability`, `calculate_trim` and `to_ga` in the Python API, or route them
  through `gas_stability`, `hydrostatics::trim` and `tanks.rs`. Until then fix the free surface (reuse
  `segment_fill`), drop the always-true flags, fix the doc comments. Change both twins, or the parity test fails.
- **Dependencies:** touches `tanks.rs` (tank lane) and `hydrostatics.rs` (#350).
- **Lane:** tank and stern builder (batch 4), as one small ticket between #350 and #351.

#### 8. Default block coefficient 0.85 sits below its own basis

- **Areas:** 01 E3, 01 C6, 01 G2; 02 WC-E1, 02 WC-C1; 06 E4, 06 C4; 12 E3. The label part is 01 E2, 01 C1.
- **Type:** Finding with an expert question. **Severity:** mixed. Conservative on cargo and laden draught;
  non-conservative on the light air draft (a deeper light draught gives a smaller air draft). **Verified (code,
  data):** `calibration/factors.yaml` fits 0.928 on LOA (n = 7). `calibration/README.md:89` lists it as applied.
  The harness uses it. The app does not. ES-TRIN 4.9 (re-read) defines Cb on L_WL, not LOA.
- **Evidence (main):** `crates/ovd-format/src/builtin_profile.json:52-58` (default 0.85, no keyed default,
  basis "calibrated box barges span 0.883 to 0.995").
- **Effect:** large. Box-barge displacement at a given draught is 8.4 % low; weight-bound cargo falls more. The
  calibration bands describe 0.928 runs, not the default Design. Outputs: draught, cargo, deadweight, air draft.
- **Coverage:** partly covered. #350 derives Cb from the raked-box volume for non-legacy Stern Forms (new
  self-propelled Options). Pushed box barges keep 0.85. #4 names Cb as an example only.
- **Ticket scope:** keyed default by Cargo Containment: `box_hold` and `sloped_hopper_hold` 0.928 (sourced to the
  fit); `type_c_tanks` an Estimate with a stated basis. Say "on the length overall" in the basis and the label.
  Old files keep their stored value. Design-impact table.
- **Dependencies:** maintainer decision. Profile edit in the format v3 window (batch 6). Item 15 later makes Cb a
  result.
- **Lane:** basis fixes (batch 6). **Expert question:** a Cb for shaped gas hulls with no case.

#### 9. Lightship and cost use two different steel masses

- **Areas:** 02 WC-E2, 02 WC-C2, 02 WC-C3; 10 E1, 10 C6; 11 E2 (the steel the drawing hides).
- **Type:** Finding. **Severity:** non-conservative on cost in the parity case (cost 34 % below the floated
  steel); sign varies by design. **Verified (code).**
- **Evidence (main):** `crates/ovd-core/src/weights.rs:238-251` (lightship, piping and outfitting on
  `steel_top_down`; `steel_delta` computed, not shown); `crates/ovd-format/src/cost.rs:72-78` (cost on
  `hull_steel_total + wing_plate + weather_shield`, the bottom-up figure).
- **Effect:** large on the capital budget. None on lightship if top-down stays.
- **Coverage:** new.
- **Ticket scope:** step 1, show `steel_delta` beside the hull steel line and fix the `calculate_weights` and
  `piping_factor` doc comments. Step 2, after an expert decision, price one steel mass.
- **Dependencies:** step 2 waits for the expert answer and a gas case.
- **Lane:** basis fixes (batch 2). **Expert question:** which steel figure is the better concept estimate.

### Tier 2: non-conservative findings that need a source

#### 10. Limiting speed formula does not tend to the open-water limit

- **Areas:** 07 E1, 07 C1; 08 E1, 08 C1, 08 C13; 09 E8, 09 C11 (two margins on one limit).
- **Type:** Finding. **Severity:** non-conservative, large. **Needs source.** Schijf (1949) and PIANC guidance are
  not in the source library. 07 derives the one-dimensional solution; 08 quotes a closed form from memory,
  F = [2 sin(arcsin(1 − m)/3)]^1.5. The triage checked that the two agree: 1.000 at m = 0 and 0.475 at m = 0.2.
- **Evidence (main):** `crates/ovd-core/src/resistance.rs:382`, F = √(2/3)·(1 − n)^(1/3), on the full depth. It
  gives 0.816 at n = 0, where the open shallow-water limit is 1.0. Above n ≈ 0.03 it gives a higher speed than
  the 1-D solution (about 60 % higher at n = 0.2). Margins: `resistance.rs:582`, `:634` (0.95) against
  `crates/ovd-core/src/electric_energy.rs:297` (1.0); neither is cited.
- **Effect:** large on channel speed curves, the 0.95 sweep cap, the 0.9 electric cap and the "channel" label.
  None on cargo or draught. Results hashes move for Options with a channel.
- **Coverage:** new.
- **Ticket scope:** get Schijf (1949) or PIANC text into the library; replace the formula (depth basis per the
  source); one margin, cited or labelled an OVD choice; test F = 1 at m = 0.
- **Dependencies:** the source. Land with item 1.
- **Lane:** basis fixes (batch 1).

#### 11. Electric installed power comes from the slowest environment

- **Areas:** 07 E4, 07 C5.
- **Type:** Finding. **Severity:** non-conservative, latent. It shows once item 10 lowers the channel cap.
  **Needs source** (depends on item 10).
- **Evidence (main):** `crates/ovd-core/src/electric_energy.rs:136` (binding = lowest service speed), `:82`
  (margin 1.20).
- **Effect:** latent today; large for some cases after item 10.
- **Coverage:** partly covered. #353 sizes installed power at the Design Speed in deep water on the new powering
  path. The legacy electric path keeps the rule.
- **Ticket scope:** size installed power on the highest battery power over all environments. Add to #353 as an
  acceptance criterion, and fix the legacy path in batch 1.
- **Dependencies:** item 10.
- **Lane:** basis fixes (legacy path, batch 1) and tank and stern builder (#353).

#### 12. Tank length over diameter uses the insulated diameter

- **Areas:** 09 E6, 09 C2; page 09 Q2.
- **Type:** Expert question. **Severity if read the other way:** non-conservative. A thick-insulated tank can pass
  at a steel ratio above 7.
- **Evidence (draft):** `gas_arrangement.rs` `length_over_diameter_check`, ratio on the insulated OD. Inland rule
  book 3.5.1 and ADN 2025 9.3.1.11.1(b) say "diameter" of the tank.
- **Coverage:** new.
- **Ticket scope:** reviewer settles the datum. If steel diameter, change the Measure (design change).
- **Dependencies:** none. Feeds item 35.
- **Lane:** rules builder.

#### 13. Tank internals are priced at the supports rate

- **Areas:** 10 E4, 10 C7; 04 G3 (note).
- **Type:** Finding. **Severity:** non-conservative on cost, small to medium. **Needs source** (stiffeners and
  ring frames of tank steel is practice, not a cited text).
- **Evidence (draft):** `weights.rs` lines 76-77 (structure factor surplus holds stiffeners, ring frames and
  saddles); `barge_cost.rs` prices it all at the supports rate (2,000 per t against 10,000 per t).
- **Coverage:** new.
- **Ticket scope:** split the surplus: internals at the tank shell rate, saddles at the supports rate, with a
  split factor (Estimate). Fold into item 39 if that lands first.
- **Dependencies:** expert answer.
- **Lane:** basis fixes (batch 2).

#### 14. Freeboard default has no rule basis; no freeboard rule exists

- **Areas:** 08 E5, 08 gap 2, 08 C8.
- **Type:** Finding with an expert question. **Severity:** none today; non-conservative for coastal hulls over
  about 36 m once the coastal stack exists. **Verified (re-read):** ES-TRIN 2025/1 Art. 4.01(1), safety clearance
  at least 300 mm; 4.01(2), 500 mm for open holds. **Verified (agent):** ICLL Reg. 28, Table 28.2.
- **Evidence (main):** `builtin_profile.json:59-62`, `min_freeboard_m` 0.3 m. OVD measures it to the deck at
  side (the ES-TRIN freeboard measure, Art. 4.02), but the value is the ES-TRIN safety-clearance number.
- **Coverage:** partly covered. #338 adds `icll-type-a` for liquid bulk only. #355 adds `es-trin-2025` with no
  freeboard Rule. #275 adds the Service Area.
- **Ticket scope:** ES-TRIN 4.01 and 4.02 Rules in the river stack; ICLL Reg. 27(14) and 28 in the coastal stack
  for all Cargoes; relabel `min_freeboard_m`.
- **Dependencies:** #275.
- **Lane:** rules builder.

### Tier 3: changes (and one finding) that unblock the most

#### 15. Build the hull sections table (ADR 0012)

- **Areas:** 01 G1, 01 G2, 01 G3, 01 G5, 01 G6, 01 G7, 01 C8, 01 C9, 01 C10; 03 G6, 03 C10; 05 G8 (part).
- **Type:** Change. **Effect:** large; it changes the hull every module sees.
- **Why it unblocks:** Cb becomes a result (closes item 8 for good), KB, BM, LCB and LCF follow the real hull,
  the drawn hull and the hydrostatic hull become one, and GZ, damage stability and a hydrostatic table become
  possible. An interim step (01 C8) can make BM consistent with the Cb volume first.
- **Coverage:** partly covered. #274 reserves the hull block in format v3. #350 leaves a hook. No ticket builds
  the generator.
- **Ticket scope:** ADR 0012 generator with the raked box and the #350 Stern Forms as the first generators;
  integrate Cb, KB, BM, LCB, LCF; hydrostatic table output.
- **Dependencies:** #350 (same files: `hull.rs`, `hydrostatics.rs`).
- **Lane:** tank and stern builder, after #350; or a new hull lane.

#### 16. Open the ticket for the IGC chapter 3 replacement of `lr-inland-gas`

- **Areas:** 05 E3, 05 G1, 05 G6, 05 G7, 05 C6; 09 G7, 09 C12; findings #280, #381.
- **Type:** Change (decided on #267 and #280, 2026-10-20). **Effect:** large, unknown until codified.
- **Why it unblocks:** closes #280 and #381 in one design change, adds IGC 3.1.1, 3.1.2, 3.2.1, 3.2.4.1 (openings
  at 4 % of L, 3 m to 5 m) with a deckhouse-opening Measure, and the inland 3.3.8 and 3.4.1 setbacks.
- **Coverage:** partly covered. Decided, but no ticket exists.
- **Ticket scope:** as decided on #280: remove the 0.60 m cofferdam Rule; codify the chapter 3 Rules with a second
  reader; Measures that measure the distance to each opening.
- **Dependencies:** #277.
- **Lane:** rules builder.

#### 17. A hidden 0.3 m freeboard gate removes `fit` candidates

- **Areas:** 05 E8, 05 C4; 08 E3, 08 C3.
- **Type:** Finding. **Severity:** unknown, mostly conservative (it removes de-rated candidates the user would
  accept; the user's own value is applied at the carried draught). **Verified (code, against code intent).**
- **Evidence (main):** `crates/ovd-core/src/gas_arrangement.rs:309-332`, `cheap_checks` takes every capacity
  check, so `freeboard_ok` gates. `barge_capacity.rs:270` (`required_freeboard: 0.3`) and `:1202`.
  `gas_arrangement.rs:466` `base_inputs` never passes the user's freeboard. `crates/ovd-format/src/simulate.rs:374`
  hides `freeboard_ok`, so the user does not see the check that removed a candidate.
- **Effect:** unknown, possibly large for deep-loaded gas hulls with a tight draught cap. A user can see "No
  arrangement passes" with no visible reason.
- **Why it unblocks:** #277 rebuilds `fit` on the Rule Trace. It should start from a clean check list.
- **Coverage:** partly covered. #277 replaces the check list, but the decision (does the full-volume freeboard
  gate a de-rated hull?) is not on any ticket.
- **Ticket scope:** maintainer decides; pass the user's freeboard, or drop `freeboard_ok` from `cheap_checks`;
  show every gating check. Design-impact table.
- **Dependencies:** land before or with #277.
- **Lane:** rules builder (batch 5).

#### 18. Show severity and gating in every output

- **Areas:** 09 G5, 09 C5; 08 E3 (part); 10 E2 (always-failing note).
- **Type:** Change. **Effect:** none on numbers; text and display change.
- **Evidence:** `app/src/lib/views/RuleChecks.svelte` on main has no severity or warn handling. Warning checks
  that fail (hopper GM, thruster count, sightline, `fabrication_rates_are_estimates`) read as FAIL. Five
  `simulate.rs` checks have no severity (draft).
- **Coverage:** partly covered. #276 shows the Rule Trace with `warn` tags in the app. The CLI summary and the
  `simulate.rs` checks without severity are not covered.
- **Ticket scope:** severity on the five checks; `warn` in the CLI summary; add to #276 acceptance.
- **Dependencies:** #276.
- **Lane:** rules builder (batch 5), UI part with #276.

#### 19. Method-validity checks reach the Design result

- **Areas:** 07 C6, 07 gap 5; 08 C5, 08 C4, 08 gaps 4 and 5.
- **Type:** Change. **Effect:** none on values; new warnings.
- **Coverage:** partly covered. #272 put the squat and waves checks in the `ovd` set (`ovd/squat/*`,
  `ovd/waves/*`). The resistance `build_checks` and the squat checks are not carried into `simulate` output, so
  those Rules can show `not applicable`. No warning says a channel with no bottom width skips the squat check.
- **Ticket scope:** carry the checks into `simulate`; add Fn, Fnh, n and Cb validity per fit; warn on no bottom
  width.
- **Dependencies:** none.
- **Lane:** rules builder (batch 5).

#### 20. One glossary pass for all twelve areas

- **Areas:** 01 G11; 02 WC-G11, WC-C13; 03 glossary; 04 G9; 05 G13, C14; 06 G10; 07 gap 13; 08 gap 13, C14;
  09 G10; 10 G13; 11 G16, C1; 12 E6 and glossary list.
- **Type:** Docs. **Effect:** none.
- **Evidence (main):** `GLOSSARY.md:106` and `:118` both define **Moulded Depth**, with different text.
- **Why it unblocks:** the pages cannot be published with undefined terms.
- **Coverage:** partly covered. #373 added Filling Limit, Loading Limit, Stern Block and others.
- **Ticket scope:** merge the two Moulded Depth entries; add the missing terms (Block Coefficient with its length
  basis, Lightship, Deadweight, Loading Condition, Free Surface Correction, Down-flooding Angle, Protective
  Location, Clear Gap, Air Draft, Squat, Under-Keel Clearance, Limiting Speed, Blockage Ratio, Capital Budget, BOQ,
  Yard Margin, Calibration Case, Origin, Consequence, Not Evaluated, and the rest in the area lists). Pick one term
  for side clearance and clear gap. Stop using "governing" for the lowest-GM condition and the light air draft.
- **Dependencies:** none.
- **Lane:** basis fixes (batch 7).

#### 21. Draw tanks and domes from the core's values, with a test

- **Areas:** 11 C2, 11 G13, 11 G14; 11 C5 (rest).
- **Type:** Change. **Effect:** none; it guards against silent drift.
- **Evidence (main):** `crates/ovd-format/src/ga.rs:115-119`, `DOME_*` constants stay as fallbacks after #369.
  The tank gaps come from `BargeInputs::new` defaults in `ga.rs`.
- **Coverage:** new.
- **Ticket scope:** put overall tank length, cylinder length, head type and the two gaps in `drawing`; draw from
  them; remove the fallback constants; test drawn length against `TankResult.overall_tank_length`.
- **Dependencies:** `drawing` keys are written in `simulate.rs` (rules lane): unhashed, small; coordinate.
- **Lane:** basis fixes (batch 3).

#### 22. Calibration hygiene

- **Areas:** 12 C1, C2, C4, C14, C15; 12 E1, E2, E5; 07 E9.
- **Type:** Change and docs. **Effect:** none to small.
- **Evidence (main):** `calibration/README.md:94` still says the barge database is unreachable (sources K1 to K39
  cite archived copies). `calibration/results.md:55` uses Lwl = 0.97·LOA; the app uses Lwl = LOA
  (`simulate.rs:329`). Web sources carry no checksum (12 E1, draft count 107).
- **Coverage:** new.
- **Ticket scope:** freshness check for `calibration/results.md` and `factors.yaml` in CI; written criteria for
  case `confidence`; fix the README; align the Lwl note; checksums for web sources or an ADR 0001 amendment; drop
  or replace the forum-sourced box-k point (12 E5); label the 21 unlabelled test files; state when a band blocks a
  release.
- **Dependencies:** none.
- **Lane:** basis fixes (batch 7).

### Tier 4: the rest, by effect

| # | Title | Areas | Type, severity, verified | Evidence | Effect | Coverage | Ticket scope | Deps | Lane |
|---|---|---|---|---|---|---|---|---|---|
| 23 | Resistance method: residuary step now, sourced method later | 07 E2, C7, C8, C10; 01 G7 | Finding, unknown, verified (code) for the step; expert question for the method | `resistance.rs:348-358` (main): 0.179 just below Fn 0.20, 0.150 at 0.20; Cr as a multiple of Cf; cited Guldhammer-Harvald not used | Small (local step) now; large with a new method | New | Make the fit continuous now; reviewer picks a sourced residuary method, (1+k), ΔCF; Lwl as an input | Expert answer | Basis fixes (batch 1) |
| 24 | Duplicate air draft check | 09 E11, C8 | Finding, unknown, verified (code) | `simulate.rs:385` (main) ignores ballast; `barge_capacity.rs` air-draft check uses it; both are `ovd` Rules | Small: one ballasted Design can get two verdicts | New | Keep the ballast-aware check; design change | None | Rules builder (batch 5) |
| 25 | Tug added to every gas budget | 10 E6, C5 | Finding, conservative on cost, verified (code) | `cost.rs:150-160` (main) no Formation test; `barge_cost.rs:360-370` always adds the tug | Large for self-propelled cost (1,322,500 at example rates) | New | Add the tug only for Formations that have one | Coordinate with #357 | Basis fixes (batch 2) |
| 26 | Cost labels: basis line, currency, mass check | 10 E2, E3, E5, C2, C3, C13 | Change | `vdesign.rs:314` (main) fixed "Example rates" text; currency fixed to USD; `barge_cost.rs:382` checks two masses only | None | New | Basis line follows the rates; `currency` and `price_base_date` inputs; check all masses | Currency input needs format v3 | Basis fixes (batch 2; inputs in batch 6) |
| 27 | Drawing hides steel the weights carry | 11 E2, G1, C4 | Finding, none on results, verified (code) | `ga.rs:457` (main) clamps the wing tank top at the depth; no wing plate or weather shield drawn | None on numbers; reader checks the wrong arrangement | New | Draw the wing tank top unclamped, the side extension, and the weather shield (dashed) | None | Basis fixes (batch 3) |
| 28 | Hopper section: drawing or volume model? | 11 E1, C3; page 11 Q5 | Expert question | Drawn slope 1.43 H:V on the hopper fixture against input 1.0; drawn hold section about 16 % larger | None (drawing fix) or small to large (model fix) | New | Reviewer decides; then fix one; test drawn area = computed area | Expert answer | Basis fixes (batch 3) |
| 29 | Stern rake LCB is mirrored | 01 E1, C4 | Finding, unknown (latent), verified (geometry) | `hull.rs:204-210` (main), stern rake integrated with `toward_bow = true` from x = 0. `hydrostatics.rs:164-174` (main) new `trim` (#349) reads `hull.LCB` | None today (no stern rake on the main path); live once #350 adds Stern Forms | Partly covered: #350 tests new forms against a numerical integral, not the legacy rake | Add the independent prism-and-wedge test and the fix to #350 | #350 | Tank and stern builder (batch 4) |
| 30 | `fit` rounding and the cap tank gap | 05 E9, E10, C11 | Finding, conservative, small, verified (code) | `gas_arrangement.rs:891` (main) rounds the viable length down untested; `:505` cap ignores the gap between tanks | Small | New | Round up or test the rounded value; count the gaps | With #277 | Rules builder (batch 5) |
| 31 | Convoy beam check cannot fail | 01 E6, C3 | Finding, none, verified (code) | `vessel_standards.rs:92` sets unit beam = beam / n; `:221-222` tests the same identity | None; a catalogue error is not caught | New | Check against an independent unit beam, or drop it | None | Basis fixes (batch 7) |
| 32 | Code text and citation fixes | 03 E3, E4, E8, C1; 04 E4, change 0; 05 E6, E7, C1; 06 E5; 07 E8, E10, C3; 01 E7, C1, C2; 09 E5 (label) | Docs (code comments and messages) | Main: `gas_stability.rs:361` cites ADN 9.3.1.15 for wind heel; `:321`, `:541` `area_0_27` reported, no Rule; `gas_arrangement.rs:298` claims a protective location check; `:47` L/D provenance; `pressure_vessel.rs:230`, `:246` cite UG-27 for heads; `resistance.rs:7` "Nothing is fixed"; `resistance.rs:36-37` against `squat.rs`/`waves.rs` g and knot; `src/ovd/core/cargo.py:65-66` says butane 601 | None (check the results hash for `area_0_27` and the limitation text) | New | One text-only PR; messages through Message Ids | None | Basis fixes (batch 7); `gas_arrangement.rs` and `gas_stability.rs` lines to their lanes |
| 33 | Speed governing labels | 07 E5, E6, C4 | Change | `resistance.rs:627-640` (main): "power" at the grid top; a failing speed reported as service speed | None | New | Add a "speed grid" label; flag a service speed that fails squat | None | Basis fixes (batch 1) |
| 34 | Design grid: 0.95 fill, no clearance check | 05 E5, G11; 09 E4 (rest) | Finding, mixed (conservative under 13,750 m³ per tank, non-conservative above), verified (code) | `design_grid.rs:26`, `:103` (main) 1.0 m override; `barge_capacity.rs:290` fill 0.95 | Medium on the design grid only | Partly covered: #283 step 2 in #277 sets the spacing; fill not covered | Grid from the side-clearance Rule; fill from the tank tables | #277 | Rules builder (batch 5) |
| 35 | `lr-inland-gas` 1.1: L/D as a class Rule, inland rows not evaluated | 05 E6, G4, G5, C9, C10; 09 E5, C3, G7 (part) | Change (rule data) | `ovd/gas-arrangement/tank-l-over-d` (main) is a tool limit; inland rule book 3.5.1 gives the same limit (agent); 3.2.1 380 m³ cap and 3.5.2 hull clauses absent | None now | New | `lr-inland/3.5.1/tank-l-over-d` (origin class); `not evaluated` rows for 3.2.1 and 3.5.2 | Item 12 | Rules builder (batch 5) |
| 36 | Weather criterion | 03 G2, C9; 08 gap 7, C12 | Change, with expert question (windage model) | `is2008-a` 2.3 is `not evaluated` (#270) | Large for light, high-sided hulls | New | Windage area and lever from the GA; Part A 2.3 Measure | Lateral area output | Basis fixes (later) |
| 37 | Pontoon and inland intact sets; length applicability; deck-level KG | 03 G7, G8, C11; 09 G8; 02 WC-G7, WC-C10 | Expert question | Part A 1.1.1 (24 m and over); Part B 2.2 pontoons; ABS barges 3-6-1/3.1 (agent) | Large: verdicts can flip for wide, shallow barges | New | Reviewer picks the default for river-only barges; then a selectable set | Expert answer | Rules builder (later) |
| 38 | Slack tank and head liquid in free surface | 03 G5, G10, C7 | Change | All tanks share one fill; heads left out (page 03 IS-A18, A19, A22) | Small to medium | New | One-tank-slack condition; head liquid in the inertia | Item 3 | Basis fixes (later) |
| 39 | Tank weight from named parts; supports; BOQ split | 04 G2, G3, change order 3; 10 G7, G8, C9, C10 | Change | Tank steel ~0.05 t/m³ against 0.096 to 0.115 published (#345) | Large: tank weight may double | Partly covered: #317 has records, no runnable case | Shell, heads, rings, saddles, wood, chocks (IGC 4.15 loads), dome, fittings; matching BOQ lines; rates by material; insulation per m² | A runnable tank weight case | Tank and stern builder (later) |
| 40 | More tank and arrangement Rules | 04 G4, G5, change order 5; 05 G2, G3, G8, C8, C13 | Change (rules) | IGC 4.13.2.2 (45 °C vapour pressure), 4.23.2.3 (external pressure), 3.5.3.5 (inspection clearances), 2.4.4 (collision bulkhead), bilge corner (the #271 second reader's remark) | None at `warn` or `not evaluated`; medium if `block` | Partly covered: #317 holds the minimum pressure and steel-by-temperature Rules; #315 offers the 45 °C basis as a choice | Add as `warn` or `not evaluated` with gaps; buckling needs a method source | #317 | Tank and stern builder (with #317) |
| 41 | Tank options | 04 G1 (rest), G6, G7, G8, change order 4 and 7 | Change | Insulation 60 kg/m³ for every type; weld efficiency and corrosion allowance fixed; no torispherical or bilobe tank | Small to large (bilobe) | New (weld factor already a Measure in #271) | Insulation density by type; vacuum double wall; weld and corrosion inputs; torispherical head; bilobe geometry | Format v3 for inputs | Tank and stern builder (later) |
| 42 | Route and environment | 07 gaps 8, 9, 11, C14; 08 gaps 3, 6, 8, 9, 10, 11, 12, C9, C10, C11, C12 | Change | One channel section; current always opposing; no waves on a Design; deep-water wavelength; squat not in the draught cap; deck cargo not in the top point | Small to large | New | Route legs; downstream leg; water temperature; waves per leg; air-draft margin and water level; optional squat in the cap | #275 | Basis fixes (roadmap) |
| 43 | Cost model maturity | 10 G1 to G6, G9, G10, C1, C3, C4, C8, C11, C14 to C17 | Change, with expert question (accuracy class source) | Example rates undated; lump sums flat; three copies of the rates | Medium | Partly covered: #334 adds liquid cost lines | Accuracy class statement (verify AACE text first); contingency line; price base date; quantity, unit and rate per line; one rate source | Item 9 | Basis fixes (batch 2, later parts) |
| 44 | Drawings completeness | 11 G2, G5, G6, G8 to G12, C8 to C11 | Change | Forward separation, service space, air draft, ballast waterline, hold bulkheads, title block, common scale not drawn | None | Partly covered: #356 (machinery, raised wheelhouse, stern), #333 (liquid tanks) | Draw the rest; label the section "TYPICAL SECTION" or cut it at amidships | Item 21 | Basis fixes (batch 3) |
| 45 | Cargo coverage | 06 G2, G3, G5, G7, C7, C9, C10; 02 WC-G6, WC-G9, WC-C12; 11 G7; 12 C6 | Change (roadmap) | No container hold, one dry bulk cargo, no deck or pontoon containment | New results only | New (liquid bulk is #329 to #342) | Container hold with ES-TRIN ch. 27; dry bulk cargo table and density input; optional ADN dry sets; pontoon k | Sources per value | Basis fixes (roadmap) |
| 46 | Lightship evidence | 02 WC-G3, G4, G8, G10, WC-C9, C11, C14; 12 E5 | Change, with expert question (margin) | Hopper and gas lightship uncalibrated; Cb fit uses k on 6 of 7 points | Unknown | Partly covered: #3 | Label hopper and gas lightship "uncalibrated"; fit k and Cb jointly; decide a weight margin | Item 8 | Basis fixes (batch 7) |
| 47 | Harness and V&V evidence | 03 G12, C13; 05 G12, C12; 07 C15; 08 C15; 12 C7, C8, C9, C12, C16 | Change | No published stability verdict; oracle shares the box idealisation; no V&V report | None (evidence) | Partly covered: #278, #279 | Gas arrangement harness cases; published verdicts; free surface and non-box oracle; whole-file harness; cost in design impact; V&V report per release | #278 | Basis fixes (later) |
| 48 | Generate basis evidence from the rules engine | 09 "What the docs need" 1 to 7 | Change | Page 09 was written by reading Rust | None | New | Per-rule and per-set pages from `ovd rules show --json`; Measure registry page; coverage by rule id | #278 | Rules builder (later) |
| 49 | Electric and gas-fuelled propulsion completeness | 07 gaps 6, 7, C11, C12 | Change | Fixed battery unit count; cost uses 620 kWh; no gas consumption or boil-off | Large for electric Options | Partly covered: #352, #353 | Battery kWh from energy per trip; gas consumption; boil-off check | #353 | Tank and stern builder (with #353) |
| 50 | Vessel Class maxima used as design dimensions | 01 G8 | Expert question | Some schemes allow LOA up to 10 % under the maximum | Unknown | New | Reviewer: offer "up to" sizing or keep maxima | None | UI builder (later) |

## Build batches

Each batch avoids the files that an active lane edits at the same time. Active lanes today:

- **Rules builder:** rules data, `simulate.rs`, `gas_arrangement.rs` (#273 to #279).
- **Tank and stern builder:** `tanks.rs`, `pressure_vessel.rs`, `aft_arrangement.rs`, `vdesign.rs` (format v3);
  next `hull.rs`, `hydrostatics.rs`, `barge_capacity.rs` (#350), `weights.rs` (#352), `electric_energy.rs`
  (#353), `ga.rs` (#356).
- **Crude oil:** new `liquid_bulk/`; later `ga.rs` (#333) and cost (#334).

| Batch | Lane | Files | Items | When |
|---|---|---|---|---|
| 1. Confined water | Basis fixes (new) | `resistance.rs`, `squat.rs`, `electric_energy.rs` (legacy path only) | 1 (step 1), 10, 11 (legacy), 23 (step), 33 | Now. Land before #353 and #364 reach these files. Items 1 and 10 as one design change with one table, once the Schijf source is in the library |
| 2. Cost | Basis fixes | `cost.rs`, `barge_cost.rs`, `CapitalBudget.svelte` | 9 (step 1), 13, 25, 26 (not the inputs), 43 (text parts) | Now. Land before #334 |
| 3. Drawings | Basis fixes | `ga.rs`; unhashed `drawing` keys in `simulate.rs` (tell the rules lane) | 21, 27, 28 (after the answer), 44 (parts not in #356) | Now. Land before #333 and #356 |
| 4. Tank and stern add-ons | Tank and stern builder | `tanks.rs`, `hull.rs`, `hydrostatics.rs`, `gas_stability.rs`, `ga_adapter.rs`, `models.rs` | 4, 5, 6, 7, 29 (in #350), 11 (criterion in #353), 40 (with #317), 49 (with #353) | Between tickets of that lane. Item 29 inside #350. Item 7 as one ticket after #350 |
| 5. Rules add-ons | Rules builder | `simulate.rs`, `gas_arrangement.rs`, rules data; `barge_capacity.rs` check only (after #350) | 2, 12, 16 (new ticket), 17, 18, 19, 24, 30, 34, 35 | Item 17 before or with #277. Items 30 and 34 with #277. Item 16 after #277 |
| 6. Decisions on shared data | Basis fixes, in the format v3 window | `builtin_profile.json`, `vdesign.rs` (inputs), `simulate.rs` (wiring) | 1 (step 2, Cm input), 3, 8, 14, 26 (inputs) | One at a time, each with its own design-impact table, after a maintainer decision. Agree the window with the tank and stern lane, which owns format v3 |
| 7. Docs and hygiene | Basis fixes | `docs/basis/*.md`, `GLOSSARY.md`, `calibration/README.md`, comment-only code lines | 20, 22, 31, 32, 46 (labels) | Now. Comment lines in `gas_arrangement.rs` and `gas_stability.rs` go to their lanes |

Not batched now (roadmap, expert answer or source first): 15, 36, 37, 38, 39, 41, 42, 45, 47, 48, 50.

## Already fixed or fully covered

These need no new ticket. A page that still describes the old state must be updated (see the next section).

| Proposal items | State on main | Ticket or PR |
|---|---|---|
| Stern block has no mass (02 WC-G1, WC-C5; 06 G4, C5; 03 G11 stern part) | Fixed for new Options; old files keep the switch off | #348 (PR #388) |
| No LCG or trim in the main result (02 WC-G2, WC-C8; 01 G6 part) | Fixed when the stern mass is in | #349 (PR #389) |
| No design liquid pressure in the wall (04 E1, change order 1) | Fixed: Peq = P0 + Pgd, static default | #374 (PR #378) |
| Head type not a Design input (04 G1, change order 2; 05 G9; 11 G13 part) | Fixed: default by Cargo and carriage condition | #375 (PR #383) |
| 2:1 head area; why two reviews differed (04 E2) | Fixed and explained | #314 (PR #325) |
| S690QL1 stress; stress basis text (04 E7; 09 G3) | Fixed; allowable stress Rule `warn` | #313 (PR #320), #271 |
| LPG density by carriage condition; loading limit (06 E2, C2; 06 G8) | Fixed | #315 (PR #327) |
| Dome default and dome drawing (11 E4, C5; page 04 dome rows) | Fixed; fallback constants remain (item 21) | #316 (PR #369) |
| End space keyed to "pressure tank" (05 E4, C5; 09 E2) | Fixed | #281 (PR #384) |
| Gas cofferdam 0.60 m (05 E3; 09 E1) | Decided: fix with item 16 | #280 |
| Opening setback measure | Decided path: item 16 | #381 |
| Areas to 40° ignore down-flooding (03 E1, G1, C5; 09 E3) | Decided: known limitation now; typed φf in format v3; Measure in phase 1 | #282, #274 |
| 1.0 m "IGC" wording (05 E5 wording; 09 E4 wording) | Fixed | #283 step 1 (PR #380) |
| Wing tank width ignores the clearance (05 E2, C3; 11 E3, C6) | Decided: derive from the side-clearance Rule | #283 step 2, #277 |
| Sightline does not block `fit` (05 E1, C2) | Decided; ticketed | #264, #354 |
| Side and bottom clearance not checked on the result (09 G1; 05 E7 check part, C7) | Rows in the trace; `fit` reads them next | #271 (PR #380), #277 |
| Typed wall skips the IGC minimum (09 G2) | Rule on `tank.wall_thickness` | #271 |
| Weld factor upper bound reading (09 E9, C2) | Settled: warn above 0.95, block above 1.0 | #271 |
| Not-evaluated rows (09 G4, C6; 03 C4 part) | `is2008-a` 2.3 and `igc-2g` 2.7 rows exist | #270, #271 |
| Count of not-evaluated Rules; scope wording (09 G6, C7) | Ticketed | #275 |
| IS Code criteria as a Rule Set (03 C4) | Built, verified by a second reader | #270 (PR #379) |
| OVD checks with honest origins (09 C10 part) | Built (`ovd` 1.0.0) | #272 (PR #382) |
| Service Area input and stacks (08 gap 1, C7) | Ticketed | #274, #275 |
| Propulsor model and bollard pull (07 gap 2) | Ticketed | #351 |
| Installed power, transmission losses, SFC on brake power (07 E7, C9, gap 3) | Ticketed | #353 |
| Convoy and tug powering (07 gap 1, C13; 01 G9; 12 C10) | Ticketed | #358, #359, #361, #362 |
| Stern machinery and raised wheelhouse not drawn; dead stern rake read (11 G3, G4, G15, C8 part, C12) | Ticketed | #356, #350 |
| Liquid cargo other than gas; tank barge calibration (06 G1, C11; 12 C5) | Ticketed | #329 to #342, #335 |
| Tank weight calibration records (04 G2 part; 12 C11) | Records added, held out | #317 part 1 (PR #345) |
| Gas and hopper factors as inputs (02 WC-C4; 06 C6) | Ticketed as slice 7 | #4 |
| Rule-data mutation and rule-id harness (12 C13) | Ticketed | #278, #279 |

Note for the register: #121 lists #65 and #280 to #283. It does not list #381. Add it.

## Docs before publishing

### Every page

1. Remove each "see the proposals file" pointer. The `proposals/` folder is never published. Link the ticket or
   the #121 finding instead.
2. Pin the code state: name the commit in the page header, or replace line numbers with function names. Many
   line numbers moved with #347 to #389.
3. Keep the first-line notice. Publish with the banner below as well.
4. Add the label **OVD Check** to the README label table (09 G11), and decide a label for maintainer thresholds
   (12 C4). Relabel the rows that used "Fixed in code" for a check.
5. Use only glossary terms (item 20 first).
6. Re-check the public-repo rule. The pages hold no client, project or person names. Keep it so: name example
   files by purpose (for example "the gas website example"), not by vessel.

### Per page

| Page | Must change before publishing |
|---|---|
| 01 Hull form | Say Cb is on LOA (ES-TRIN uses L_WL). Link items 1, 8, 15, 29. HF-A42: note that the main-path `trim` (#349) now reads `hull.LCB`. Fix the D/B basis ranges text (01 E7) |
| 02 Weights | Rewrite section 3 and WC-A29 for #348 (stern block mass) and WC-A48 for #349 (trim). Link items 3, 4, 9. Keep the deck-KG note (item 37) |
| 03 Intact stability | IS-A14 (stern block mass) and IS-A47 (trim) for #348 and #349. Criteria are now the Rule Set `is2008-a` 1.0.0, verified by a second reader (#270); the 40° rows link #282 as a known limitation. Correct the limitation-text citation (item 32) |
| 04 Type C tanks | Liquid pressure is in the wall (#374, input `tank_liquid_pressure`). Head type is an input (#375; LPG 2:1, LNG hemispherical). Dome from the dome rule (#316). Allowable stresses from Rm and Re (#313). Remove "pending" for these from the header. Link items 5, 6, 39, 40 |
| 05 Gas arrangement | End space by tank end shape, not "pressure tank" (#281). Rule Sets `igc-2g` and `lr-inland-gas` are built (#271, #272). Add #381. Link items 16, 17, 34, 35 |
| 06 Liquid, dry, container | LPG density by carriage condition and loading limit (#315): rewrite CG-A03, A05, A07 and the limits list. Stern block mass (#348). Link items 2, 8, 45 |
| 07 Resistance | Link items 1, 10, 11, 23, 33 and #351, #353. State that the limiting-speed source is not held |
| 08 Waterway | Label the Schijf row "probable error, source not held" with item 10. Freeboard row: cite ES-TRIN 4.01(1) as read, and say the measure differs (item 14). Link item 17 |
| 09 Rules | The largest rewrite. "Designed, not built" is out of date: the loader (#268), evaluator and Rule Trace (#269), `is2008-a` (#270), `igc-2g` (#271), `ovd` and `lr-inland-gas` (#272) are merged. `ovd rules` and `--rule-set` exist. Update the inventory to the rule ids. Remove the "open tickets #268 to #279" statement |
| 10 Cost | Verify the AACE accuracy-class ranges against the AACE text before any statement goes out (the draft read a secondary summary). Link items 9, 13, 25, 26, 43 |
| 11 Drawings | DR-A38 (dome): drawn at its diameter since #369. DR-A31 (heads): 2:1 ends drawn since #383. Link items 21, 27, 28, 44 |
| 12 Calibration | Add the tank weight records of #345. Fix the calibration README statement (item 22). Link item 47 |
| README (basis) | Keep as the authoring guide; do not publish it as is. Add the OVD Check label. Point to this triage |

### How to publish

1. **Section.** Add a nav entry after "Specs" in `website/mkdocs.yml`: `- Engineering basis: basis/`. The
   `on_config` hook in `website/hooks.py` already expands any nav value that ends in "/" into every page of that
   folder, so no hook change is needed.
2. **Assembly.** In `website/build.py`, add a loop beside the `docs/adr` and `docs/specs` loop:
   `for md in sorted((REPO / "docs/basis").glob("[0-9][0-9]-*.md")): PAGES[f"docs/basis/{md.name}"] =
   f"basis/{md.name}"`. The pattern takes only the twelve area pages. It skips `README.md` and never reaches
   `proposals/` (the glob is not recursive). If an index page is wanted, write a short `docs/basis/index.md`
   for readers; do not publish the authoring README.
3. **Proposals never published.** Also check `rewrite_link`: a relative link to `proposals/…` becomes a GitHub
   link. Step 1 of "Every page" removes those links.
4. **Banner.** Inject it in `build.py` for every basis page, so a page cannot ship without it:
   `!!! warning "Draft: needs expert review"` (the `admonition` extension is on). Text: "Concept and preliminary
   design only. Not for approval. A named discipline reviewer has not signed this page off." Remove it per page
   only when the page history records a discipline sign-off (README "Publishing").
5. **Gate.** Run the docs build with the nav and link validation in `mkdocs.yml` (warnings must be zero for the
   basis pages), the release guard (tree and history) and the embargo check before the first publish.

## What this triage could not check

- No build and no run. Every effect size is the proposals' estimate.
- Sources not held or not read: Schijf (1949), PIANC channel guidance, Barrass (2004), Lackenby (1963), the
  Guldhammer-Harvald charts, AACE RP 18R-97 and 17R-97, ASME VIII-1, IMO MSC.1/Circ.1281. Items 10, 11, 13, 23
  and 43 rest on these.
- Re-read by the triage: ES-TRIN 2025/1 Art. 1.01 4.9 and Art. 4.01; IS Code 2008 Introduction 2.23. Every other
  "Verified (agent)" item needs a second reader before it goes to #121.
- Lines marked "draft" were not read again on main: the legacy KG items (02 WC-E5), the API clearance datum
  (05 E11), the five `simulate.rs` checks without severity, the web-source count in `calibration/sources.yaml`.
- Issues read in full: #3, #4 (skimmed), #121, #264, #267 to #283, #292 (skimmed), #313 to #317, #329 to #343,
  #346 to #365, #374, #375, #381. Other issues were matched by title only.
- Whether the `ovd/squat/*` and `ovd/waves/*` Rules evaluate on the Design path or show `not applicable`
  (item 19) was inferred from the code, not from a run.
- The draft pages may lag main in places not listed above. A page-by-page diff against `b2e943f` is part of the
  docs batch.
