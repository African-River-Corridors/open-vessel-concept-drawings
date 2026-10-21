# Proposals: 08 Waterway and route

Not published. Gaps, errors and proposed changes for `docs/basis/08-waterway-and-route.md`. Errors become findings
for the register (#121) after maintainer triage. Assumption ids (WR-Ann) refer to the page.

## Errors

### E1. The limiting speed formula does not match the Schijf limit (probable error, large, non-conservative)

- **Code:** `crates/ovcd-core/src/resistance.rs`, `schijf_limiting_speed`, line 370:
  F_nh,crit = √(2/3) · (1 − m)^(1/3). The doc comment calls it "Schijf (1949) critical speed".
- **Problem:** with no blockage (m → 0) the critical speed in shallow water is √(g · h), so F_nh,crit must tend to
  1.0. The code gives 0.816. The closed form often quoted for Schijf is
  F_nh,crit = [2 · sin(arcsin(1 − m) / 3)]^1.5, which gives 1.0 at m = 0. Values:

  | m | Code | Closed form |
  |---|---|---|
  | 0.05 | 0.803 | 0.731 |
  | 0.10 | 0.788 | 0.623 |
  | 0.20 | 0.758 | 0.475 |
  | 0.30 | 0.725 | 0.365 |

  Above m ≈ 0.03 the code allows a higher speed than the closed form: about 60 % higher at m = 0.2.
- **Source:** Schijf (1949) is not in `~/Dev/ovcd-rule-sources/`. I have not read it. The closed form above is
  from memory and needs a check against the source or PIANC guidance before it becomes a finding.
- **Effect:** large on the channel service speed, the "channel" governs label and the 0.95 cap, where squat does
  not govern first (deep, narrow canals). None on cargo or draught.

### E2. The limiting speed uses C_b as the midship coefficient (small to medium, non-conservative)

- **Code:** `resistance.rs`, `midship_section_area`, line 365: A_m = B · T · C_b. The squat module uses
  A_m = B · T (`squat.rs`, line 142).
- **Problem:** the midship section of a barge has C_m near 1. With C_b 0.85, m is about 15 % low, so V_L is high
  (worse with E1 fixed). The two modules also report two different blockage values for one hull and channel.
- **Source:** no rule; geometry. The midship coefficient is the section coefficient, not the block coefficient.
- **Effect:** small to medium on the limiting speed and the shallow-water blockage term (page 07).

### E3. The gas search gates on `freeboard_ok` with a fixed 0.3 m (medium)

- **Code:** `gas_arrangement.rs`, `Search::base_inputs` (near line 432) does not copy
  `FitOptions::required_freeboard_m` into `BargeCapacityInputs`, so `required_freeboard` stays 0.3 m
  (`barge_capacity.rs`, line 210). `cheap_checks` (line 297) adds every capacity check, so `freeboard_ok`
  (`barge_capacity.rs`, line 941) gates each candidate at the **volume-bound full-load** draught.
- **Problem:** (a) the user's `min_freeboard_m` does not reach this check; a smaller typed value does not relax it.
  (b) A candidate that the draught cap de-rates is still refused when its full-volume draught would leave less than
  0.3 m. The simulator hides the row as "superseded" (`simulate.rs`, `capacity_checks`, line 368), so the user can
  see "No arrangement passes" with no visible reason, or a smaller result than the inputs allow.
- **Source:** spec 0004 ("How `fit` uses it") and the comment block in `gas_arrangement.rs` (line 277) list
  "freeboard" among the capacity checks without saying at which draught.
- **Effect:** medium on gas results with a tight draught cap: fewer candidates, possibly less cargo. Needs a
  decision: is full-volume freeboard intended for a de-rated hull?

### E4. Old result fields claim checks that no code makes (none on results, misleading)

- **Code:** `models.rs` doc comments: `min_freeboard_ok` "True if freeboard >= 0.3 m" (line 400),
  `air_draft_ok` "True if air_draft <= vertical_clearance" (line 404), `total_depth_required` "draft + underkeel
  clearance" (line 410). `ga_adapter.rs`, `to_ga` (lines 241 to 248) sets both flags to `true` and
  `total_depth_required` to the moulded depth. `underkeel_clearance` (0.5 m) and `vertical_clearance` are read by
  no calculation.
- **Effect:** none on results. The GA JSON (`eval.rs`, `ga_results_json`, line 2494) shows "ok" flags that were
  never tested. A reader can take them as checks.

### E5. The page cannot cite the freeboard default to any rule (labelling, not code)

- **Code:** `builtin_profile.json`, `min_freeboard_m` 0.3 m; basis text gives only the hard limit.
- **Source:** ES-TRIN 2025/1, Art. 4.02(1): freeboard 150 mm for a flush deck vessel; Art. 4.01(1): safety
  clearance at least 300 mm; Art. 4.01(2): 500 mm for open holds. ICLL (Annex I as amended by MSC.143(77)),
  Reg. 28, Table 28.2: type B tabular freeboard 200 mm at 24 m, 300 mm at 36 m, 721 mm at 70 m, 1690 mm at 120 m.
- **Problem:** 0.3 m matches the ES-TRIN safety clearance number, but OVCD measures it to the deck at side, which
  is the ES-TRIN freeboard measure. Against ICLL it is far too small above 36 m.
- **Effect:** none today; it decides whether the value is a "Sourced default" or an "Estimate" and what the
  coastal stack must change.

## Gaps

1. **Service Area is not in the code.** No input, no default stack, no scope wording (#274, #275).
2. **No freeboard rule for either Service Area.** River: ES-TRIN Chapter 4 (Art. 4.01, 4.02; Art. 4.05 for zone 4).
   Coastal: ICLL Reg. 27 and 28 with Reg. 27(14) for barges. Neither is coded.
3. **Squat does not feed the draught cap.** The cap uses the static clearance; squat limits speed only. An expert
   may expect cap = depth − squat(V_service) − net clearance.
4. **No squat on a channel with no bottom width.** The draught cap applies, the squat check is skipped, and no
   warning says so (`simulate.rs`, `curves`, line 302).
5. **Validity checks are dropped on the Design path.** The squat checks (`squat.rs` lines 178 to 238) and the
   resistance checks (`resistance.rs`, `build_checks`, line 680) do not reach `simulate` output.
6. **One channel section only.** No route legs, no water level series, no locks (length and width as Waterway
   Limits), no bends, no downstream leg (current is always opposing).
7. **No waves or wind on a Design.** `simulate` never sets a wave height. No wind load or wind heel anywhere.
   Coastal-and-river needs both, and spec 0004 targets IS Code Part B 2.2 for pontoons, which has a wind heel
   criterion (540 Pa).
8. **No bridge margin and no water level** in the air draft check. The navigating air draft (wheelhouse raised)
   is not checked.
9. **Top point misses deck cargo and container tiers** (`barge_capacity.rs`, line 662), so a laden case that stands
   higher than the light case is never checked.
10. **Barrass returns 0 in open water.** With no channel area the squat is ICORELS only; Barrass has an open-water
    use in the literature (not verified here).
11. **Squat only at the loaded draught.** Not in the light or ballast condition, not with trim.
12. **Wavelength is deep water** (`waves.rs`, line 130). In shallow water λ is shorter, so the λ/L_pp < 0.5 check
    passes too easily.
13. **Glossary terms missing:** Air Draft, Under-Keel Clearance (static, dynamic), Squat, Freeboard, Minimum
    Freeboard, Draught Cap, Channel (the route model), Safety Clearance, Journey De-rating, Binding Limit.
    `draught_limit_m` and `air_draft_limit_m` sit in the input group "Waterway Limits"; the glossary's Waterway
    Limit names only lock length and width. Glossary also defines Moulded Depth twice.
14. **No independent evidence.** Squat, wave and limiting-speed tests are Python parity only.

## Changes

| # | Change | Effect on results | Order |
|---|---|---|---|
| C1 | Check the limiting-speed formula against Schijf (1949) or PIANC; fix E1 if confirmed; add a test at m = 0 (expect F_nh = 1). | Large on channel speed curves; none on cargo | 1 |
| C2 | Use B · T · C_m (C_m input, default near 1) for the limiting speed and the blockage term (E2); one blockage function for squat and resistance. | Small to medium on speed | 2 (with C1) |
| C3 | Pass `required_freeboard_m` into the capacity inputs of the gas search, and decide whether `freeboard_ok` at full volume gates a de-rated candidate (E3). Show every gating check. | Medium on gas results with a tight cap | 3 |
| C4 | Warn when a channel has a depth but no bottom width, and say no squat check ran (gap 4). | None; adds a warning | 4 |
| C5 | Carry the squat and resistance validity checks into the `simulate` result and the Rule Trace as `ovcd` method-validity Rules (spec 0004 lists F_nh 0.7 and blockage 0.35) (gap 5). | None on values; new warnings | 5 (with #272) |
| C6 | Remove or compute the old fields in E4 (`min_freeboard_ok`, `air_draft_ok`, `total_depth_required`), and drop the unused `underkeel_clearance` and `vertical_clearance`. | None | 6 |
| C7 | Add the Service Area input and default stacks (#274, #275). | None until stacks diverge | 7 |
| C8 | River stack: ES-TRIN 2025/1 Art. 4.01 and 4.02 as Rules (freeboard to deck, safety clearance to the lowest opening, 500 mm for open holds). Coastal stack: ICLL Reg. 28 Table 28.2 with Reg. 27(14). Then relabel `min_freeboard_m` (E5, gap 2). | Large for coastal hulls over about 36 m; small for river | 8 (after C7) |
| C9 | Add an air-draft margin input and a design water level; optionally check the raised wheelhouse; add deck cargo height to the top point (gaps 8, 9). | Small to medium where a bridge governs | 9 |
| C10 | Offer squat in the draught cap as an option: cap = depth − squat(V_service) − net clearance (gap 3). | Medium in shallow channels | 10 |
| C11 | Route legs: a list of channel sections with their own depth, width, current direction and water density; the governing leg sets the cap and the speed (gap 6). | Large for mixed routes | 11 |
| C12 | Waves and wind on a Design for coastal-and-river: wave height per leg to resistance; a wind heel check with the IS Code Part B set (gap 7). | Large for coastal service | 12 (after C7, C11) |
| C13 | Get Barrass (2004) and PIANC (2014) channel guidance into the library and verify WR-A13, A15, A21, A22; get ITTC 7.5-04-01-01.2 for STAwave-1 (WR-A34, A39) and the heading factor. | None until a value differs | Parallel to all |
| C14 | Add the missing glossary terms (gap 13). | None | Any time |
| C15 | Add independent cases: a published squat measurement or worked example, and a published STAwave-1 example, to the evaluation harness. | None; evidence | After C13 |
