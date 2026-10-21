# Calibration and validation

> Concept and preliminary design only. Not for approval. Draft: needs expert review.
>
> Code state: written against `main` at commit `f6d446c` (2026-10-20).

**For:** Any reviewer  ·  **Code:** `src/vessel_designer/calibration.py`, `calibration/`, `evaluation/run.py`, `evaluation/oracle/`, `tools/design_impact.py`, `tools/web_parity.mjs`, `tools/check_fixture_hashes.py`, `tools/draw_crosscheck.py`, `.github/workflows/ci.yml`  ·  **Rule Sets:** none (the harness checks the IS Code 2008 Part A 2.2 criteria)

OVCD is a concept and preliminary design tool (ADR 0011). This page answers one question: "why should I trust
this number?" It lists the evidence that exists today, how strong it is, and where there is none. A result
inside a Calibration Band is not a validation. No part of OVCD has a verification and validation report yet.

## What OVCD computes here

Four checks run on the engine. Each answers a different question.

- **Calibration** (`python -m vessel_designer.calibration`): how far is the model from published vessels and
  tanks? It writes `calibration/results.md` and `calibration/factors.yaml`. CI runs `--check`, which prints
  one warning annotation per figure outside its Calibration Band.
- **Evaluation harness** (`python3 evaluation/run.py`): does the engine give the known verdict on designs
  whose compliance is known? It writes `evaluation/results.md` with the false-pass count, the false-fail count
  and the coverage of each criterion.
- **Design-impact check** (`python3 tools/design_impact.py`): did this change move any design? It writes a
  before/after table for each pull request.
- **Parity checks**: does the same design give the same result on every system? Native against WebAssembly
  (`tools/web_parity.mjs`), the fixture results hashes on four systems and WASI at release
  (`tools/check_fixture_hashes.py`), and the drawing crate against a second implementation of the drawing
  standard (`tools/draw_crosscheck.py`).

Calibration and the harness test whether the answer is **right**. The design-impact and parity checks test only
whether the answer is **the same** as before or elsewhere.

## How it works today

### 1. Calibration cases and their sources

The cases are data files in `calibration/`. A case holds the inputs of a published vessel or tank, the
published figures, a source and a `confidence` (high, medium or low). A figure the source does not give is
absent. Nothing is estimated or filled in. ADR 0001 keeps the source files in a private store; git holds a
record of each one.

| Set | File | Records | Run | What is compared |
|---|---|---|---|---|
| Dry barges, in-sample | `cases.yaml`, `dry_barge` | 12 | 12 | Lightship, deadweight at a stated draught |
| Dry barges, held out | `cases.yaml`, `dry_barge` | 78 (box 32, deck 21, pontoon 25) | 78 (3 are cross-checks, no band) | Lightship, deadweight |
| Held out, not runnable | `cases.yaml`, `not_yet_runnable_cases` | 48 (41 tank barges, 6 gas barges, 1 motor container vessel) | 0 | Listed with their figures and the reason |
| Inland motor vessels | `cn_yangtze_gb38030_draft_2018.yaml` | 20 design points (7 bulk, 7 liquid, 6 container) | 14 (bulk and liquid) | Installed power against computed delivered power; no band |
| Type C tanks | `tank_cases.yaml`, `type_c_tank` | 2 | 2 | Gross volume, cylinder wall thickness, tank steel weight |
| Type C tanks, not runnable | `tank_cases.yaml`, `not_yet_runnable_tank_cases` | 3 | 0 | Listed with the reason (two bilobe tanks, one record with no length) |
| Pushers and pushboats | `pushers.yaml` | 10 | 0 | Nothing. No code reads this file |

Sources (`sources.yaml`): 8 stored documents (`sources`: Chinese inland standard ship-type series and their
consultation-draft notes), each with a checksum, size, store key and licence; and 107 web records
(`web_sources`, ids `S`, `T`, `M`, `K`, `W`, `C`, `D`, `J`, `P`), each with a title, URL, access date and,
for most, a note. The web records carry no checksum. The 12 in-sample cases cite free-text sources (a thesis,
journal papers, trade press, designer pages, and for one case, practitioner statements found by search). The
126 held-out records cite a `web_sources` id. Label: **Sourced default** for each published figure, at the
confidence the case states.

### 2. Held-out against in-sample

`fit_factors` (`calibration.py`, line 107) fits two factors from the cases. It drops every case with a
`held_out` marker first, so a held-out case can never move a factor. Label: **Method**.

- Lightship factor k per hold type: the mean of lightship ÷ (L × B × D) over cases with a published lightship.
  Box: 0.1093 t/m³ (n = 3, range 0.095 to 0.1223). Container: 0.1711 t/m³ (n = 1).
- Box-barge block coefficient on LOA: the mean of (DWT + lightship) ÷ (L × B × T × ρ) over box cases with a
  draught and a DWT, less the cases marked `exclude_from_fit_cb`. Cb = 0.928 (n = 7, range 0.883 to 0.995;
  2 excluded because they imply Cb above 1).

The 12 first cases are **in-sample**: their errors come from the same cases the factors were fitted to. All
126 later records are **held out** (62 from a research note, 64 from follow-up research; tests check the
split). Their errors are **out of sample**. Deck and pontoon barges have no model of their own. They run as box
barges (`run_as: box`), with the box k and the box Cb. `run_barge` (line 155) runs each case twice: as ported
(sloped-hopper build-up, Cb 0.85) and as calibrated (k and Cb from `factors.yaml`). The report also shows two
published lightship formulas for comparison.

The 2 tank cases are held out. One exception: the tank structure factor 1.12 of fully pressurised LPG in the
Defaults Profile was worked out from the first tank's published steel weight, so that one figure is in-sample.

### 3. Calibration Bands (warnings only)

A band is the accepted error of the model against a published figure, in percent either way, per figure and
per confidence of the case. `BandCheck.inside` (line 226) is |error| ≤ band. The bands are starting values. The
maintainer set the barge bands (issue #119); the tank bands were proposed with the first tank cases (issue
#292) and wait for the maintainer. No published source or statistical fit sets any band. `main_check` (line 448)
prints one `::warning` per miss and exits 0. A crash (a missing file, a bad case) still fails. Tests
check both sides (`tests/test_calibration_runs.py`: a moved figure fires the warning; the check still exits
0; a crash fails).

Results today (`calibration/results.md`, as committed):

- In-sample: 13 of 14 calibrated figures inside their band.
- Held out: 67 of 87 calibrated figures inside their band.
- Held-out deadweight, box barges: 30 of 30 inside, mean absolute error 4.1 %.
- Held-out lightship, deck barges run as box: 14 of 15 inside, mean absolute error 11.2 %.
- Held-out pontoons run as box: lightship 12 of 22 inside, mean error −27.0 % (the box k is too light for
  pontoons); deadweight 3 of 7 inside, mean error +25.8 %.
- Tanks: 5 of 5 figures inside. Volume −0.1 % and +0.2 %; wall +6.9 % and +12.0 %; steel +5.2 % (in-sample).
- Motor vessels: installed power is 1.72 to 2.11 times the computed deep-water delivered power. The points are
  design-study output, not built ships, so this is not a calibration against real vessels.

### 4. The evaluation harness

The harness (`evaluation/`, issue #184, phase 0) tests one thing: the six IS Code 2008 Part A 2.2 criteria of
a box section with a known KG. `evaluation/engine.py` calls the engine's `box_condition` (in
`crates/ovcd-core/src/gas_stability.rs`), the per-condition step of the gas `assess`. It does not run a
`.vdesign`.

- **Known designs.** 5 base cases in `evaluation/cases/`. Two are chosen boxes. Three take their inputs from
  published sources: a uniform box barge from an open-access journal paper (CC BY 4.0), and an empty and a
  loaded landing craft modelled as a box in naval academy course notes. No source gives an IS Code 2.2
  verdict, so every verdict is **derived** by the oracle.
- **The oracle** (`evaluation/oracle/box_oracle.py`) is a second, independent calculation. It shares no code
  with the engine, and a test checks that it never imports it. It integrates with Simpson's rule on a 0.01°
  step from 0° to 60°. Tests check it against closed forms (GM = KB + BM − KG, the wall-sided GZ formula) and
  against the published GM and KM of the source cases. Label: **Method**.
- **Boundary mutants.** `oracle/mutants.py` moves one input (KG, or the depth for the angle of maximum GZ) so
  that one criterion sits just inside or just outside its limit. Each criterion gets a big step (above the
  engine's grid error) and a small step (inside it, marked `grid: true`). 60 mutants are made. 60 more are
  skipped because no KG between keel and deck crosses the limit.
- **Verdicts.** 361 criterion verdicts are compared outside the grid error: 253 expected pass, 108 expected
  fail. **False passes: 0. False fails: 0.** Every criterion has at least one passing and one failing case.
  Nine small-step verdicts inside the grid error differ; they are listed and not counted.
- **Gate.** A false pass fails the build. A false fail and a coverage gap are warnings. A `results.md` that is
  not what the run writes fails. A test checks that `mutants.yaml` is what the generator writes.

### 5. The design-impact check

`tools/design_impact.py` (ADR 0010, ticket 28) builds `ovcd` from the merge base and from the branch. It runs
the same 172 designs through both: 43 box barges from the calibration cases (in-sample and held out), 6
points from the motor-vessel design note, 76 cases that each set one optional input, and 47 sweep cases
(sizes, formations, channels, every gas Cargo, every LPG Carriage Condition, designs that pass every check,
designs that carry nothing). It compares every number at a relative and absolute tolerance of 1e-9 (`same`,
line 443). A change of type (integer to float) is a change. The keys `drawing`, `stability_marks`,
`cost_masses` and `cost` are left out of the comparison, as they are left out of the results hash
(`crates/ovcd-format/src/run.rs`, line 244).

The check has no stored answers. It says only whether a design moved, not whether the new value is right. The
fast gate (`tools/gates.sh`, step 6) fails when a design moved and `CHANGELOG.md` has no table for it, when
either build fails, when the branch has fewer cases than the base, or when every design stops computing. A
move needs no sign-off.

### 6. Parity checks

- **Native against WebAssembly** (CI job `web-parity`, issue #245): every Option of the 6 fixture `.vdesign`
  files must give the same SHA-256 results hash from the WebAssembly Design session as from native
  `ovcd run --json`. The job also checks that SVG, PDF and DXF exports come out of the WebAssembly build. Exact
  equality; no tolerance.
- **Fixture hashes on every system** (release gate): the fixtures store their results hashes.
  `check_fixture_hashes.py` checks them with the release binary on Windows, Intel macOS and Apple-silicon macOS,
  and with the CLI built for WASI under `wasmtime`. At least 6 Options must match. `cargo test` checks them on
  Linux.
- **Drawings** (CI job `drawing-crosscheck`, spec 0003): each fixture's drawing documents go through the Rust
  crate and through the Python implementation of the same drawing standard, pinned to one commit. The
  normalised content must be equal (numbers compared as numbers), over at least 15 documents. The DXF files are
  read back by the standard's own reader and by `ezdxf` with its audit.
- **Lean model** (`proofs/lean`, ADR 0014): proofs of four properties of a model of the future rules
  composition kernel. A separate weekly job. It does not test the Rust engine yet: the Rust rule evaluator does
  not exist (spec 0004 is a draft).

These checks show that two builds or two implementations agree. They do not show that either is right.

### 7. Fixed-number tests

Each test that pins a number carries a label (`tests/test_test_labels.py`): `sourced` (from a standard, a
published example or a hand calculation), `snapshot` (what code printed) or `property` (no fixed number). Today
the Python and Rust tests hold about 23 sourced tests, 55 snapshot labels and 248 property labels. 21 older Rust
test files are exempt from the label rule (`RUST_UNLABELLED`). A snapshot shows only that the Rust port agrees
with the original code.

## Assumptions, rules and estimates

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| CV-A01 | Band, barge lightship | ±15 / 20 / 30 % at high / medium / low confidence | Fixed in code | Maintainer starting value, issue #119; no published basis (`calibration/bands.yaml`) | `calibration.py` `_band_check` | Unknown: too wide hides model error; too narrow gives noise warnings |
| CV-A02 | Band, barge deadweight at the stated draught | ±7.5 / 10 / 15 % | Fixed in code | Maintainer starting value, issue #119 | `_band_check` | Unknown |
| CV-A03 | Band, gross tank volume | ±3 / 5 / 10 % | Fixed in code | Proposed with issue #292; maintainer has not set it (`tank_bands.yaml`) | `_assess_tanks` | Unknown |
| CV-A04 | Band, cylinder wall thickness | ±10 / 15 / 25 % | Fixed in code | Proposed with issue #292 | `_assess_tanks` | Unknown |
| CV-A05 | Band, tank steel weight | ±15 / 20 / 30 % | Fixed in code | Proposed with issue #292 | `_assess_tanks` | Unknown |
| CV-A06 | Band acceptance | Inside when abs(error) ≤ band; a miss is a warning, never a failure; a crash fails | Fixed in code | ADR 0010; issue #119 | `BandCheck.inside`, `main_check` | Non-conservative: a large error never stops a merge |
| CV-A07 | Confidence of a case | high / medium / low, set by the person who adds the case | Estimate | Judgement recorded in each case; no written criteria for the levels | Every band | Unknown: the band depends on it |
| CV-A08 | Lightship factor k, box hold | 0.1093 t/m³, mean of 3 cases (0.095 to 0.1223) | Estimate | `factors.yaml`, in-sample fit; one of the 3 points rests on practitioner statements (low confidence) | Calibrated runs; Defaults Profile `lightship_factor_t_per_m3` for a box hold | Unknown; held-out deck barges agree (mean error −6.3 %), pontoons do not (−27.0 %) |
| CV-A09 | Lightship factor k, container hold | 0.1711 t/m³, 1 case | Estimate | `factors.yaml`; one point, in-sample, not validated | Calibrated runs | Unknown |
| CV-A10 | Box-barge Cb on LOA | 0.928, mean of 7 cases (0.883 to 0.995) | Estimate | `factors.yaml`; 2 cases excluded (imply Cb above 1) | Calibrated runs only | Unknown; held-out box DWT agrees (30 of 30 inside) |
| CV-A11 | Ported baseline | Sloped-hopper build-up and Cb 0.85 | Fixed in code | The original engine, kept for comparison | `run_barge` | Not a result; for comparison only |
| CV-A12 | Water density for predicted DWT | 1.000 t/m³; 1.025 where the source gives a sea-going or saltwater figure (22 cases) | Fixed in code | The case's `water_density_t_m3` | `_rho_of` | Small: 2.5 % on DWT |
| CV-A13 | Deck and pontoon barges | Run as box (box k, box Cb) | Fixed in code | No deck or pontoon model exists | `_run_hold_type` | Pontoons: lightship low, DWT high (non-conservative on capacity) |
| CV-A14 | Motor-vessel run | Deep water 50 m, Lwl = 0.97 × LOA, Cb = implied Cb ÷ 0.97 capped at 0.99 | Fixed in code | Runner choice; the design note gives no waterline length or water depth | `run_motor` | Unknown: the power ratio mixes margins, shallow water and model error |
| CV-A15 | Tank case datum | Diameter and length brought to the outside of the insulation from the basis each case states | Method | GLOSSARY "Tank Outer Diameter"; the case's `diameter_basis`, `length_basis` | `_tank_geometry` | Volume error if a source's basis is misread |
| CV-A16 | Tank head and steel stand-ins | Nearest head (`run_as_head`) and nearest steel (`run_as_material`) the engine has | Fixed in code | Engine limits; the gap is part of the result | `_run_tank` | Second tank: wall high (9Ni stress below the real steel's), so conservative |
| CV-A17 | Harness limits | 0.055, 0.090, 0.030 m rad; GZ 0.20 m at 30° or more; max GZ at 25° or more; GM 0.15 m | Rule (unverified) | IMO MSC.267(85), IS Code 2008 Part A 2.2.1 to 2.2.4, read in a certified copy; no second reader recorded | Expected verdicts of every case | A misread limit makes both oracle and engine wrong in the same way |
| CV-A18 | Oracle integration | Simpson's rule, 0.01° step, 0° to 60°, continuous angle of max GZ, no downflooding angle | Method | `evaluation/README.md`; checked against closed forms | `box_oracle.py` | Small; box only |
| CV-A19 | Grid tolerances | Areas 0.0005 m rad, GZ 0.002 m, angle 1°, GM 1e-6 m | Fixed in code | Above the largest engine error seen on the corpus (a test checks it) | Mutants marked `grid: true` | A real engine error smaller than the tolerance is not seen |
| CV-A20 | Mutant steps | Big: areas 0.002 m rad, GZ 0.01 m, angle 2°, GM 0.005 m. Small: 0.00002, 0.0001, 0.2°, 0.001 m | Fixed in code | `oracle/mutants.py` | `mutants.yaml` | Unknown |
| CV-A21 | Harness acceptance | False pass = 0 or the build fails; false fail and coverage gap warn; stale `results.md` fails | Fixed in code | Issue #184; ROADMAP "Evaluation harness" | `evaluation/run.py` `main` | False fails do not stop a merge |
| CV-A22 | Design-impact tolerance | Relative and absolute 1e-9; a type change counts | Fixed in code | `tools/design_impact.py` `same` | Every compared number | None in practice |
| CV-A23 | Design-impact acceptance | Fails only on a move with no `CHANGELOG.md` table, a failed build, fewer cases, or all cases failing | Fixed in code | ADR 0010; `docs/agents/build-process.md` step 4 | `gates.sh` step 6 | A wrong move passes if it is logged |
| CV-A24 | Keys outside the comparison | `drawing`, `stability_marks`, `cost_masses`, `cost` | Fixed in code | `UNHASHED_KEYS`, `run.rs` line 244 and `design_impact.py` (a test keeps them equal) | Design impact, results hash, web parity | A cost change is never reported |
| CV-A25 | Stand-in depth, motor-vessel sweep | Draught + 1.5 m | Fixed in code | The design note gives no depth | `design_impact.py` `CN_DEPTH_ABOVE_DRAUGHT_M` | None: sweep points only |
| CV-A26 | Parity acceptance | Exact hash equality; at least 1 hash compared (web), 6 Options (fixtures), 15 drawing documents | Fixed in code | `web_parity.mjs`, `check_fixture_hashes.py`, `draw_crosscheck.py` | CI and release gate | None: equality is strict |

## Inputs the user can change

None in the app. The bands, cases and factors are repository data, not Design inputs. A contributor changes them
by a pull request:

- `calibration/cases.yaml`, `tank_cases.yaml`: add a case. It must have a `confidence`; a new case is held out.
- `calibration/bands.yaml`, `tank_bands.yaml`: the bands (maintainer).
- `OVCD_CALIBRATION_DIR` (environment): the calibration folder when the package runs from an installed wheel.

The fitted k for a box hold reaches users as the Defaults Profile default of `lightship_factor_t_per_m3`
(0.1093 t/m³, soft range 0.08 to 0.15), which the user can change.

## Limits and known findings

- **Bands are warnings.** No calibration error can stop a merge or a release (ADR 0010, issue #119).
- **Calibration covers only dry barges and one tank shape.** No calibration case runs a self-propelled vessel, a
  tank barge, a gas barge, a pusher or a convoy. 48 held-out records and all 10 pusher records cannot run.
- **No built-vessel speed and power point.** The motor-vessel points are design-study output, and every one
  implies the same Cb (0.835), so they test OVCD against another model.
- **The in-sample lightship rests on 3 points**, one of them practitioner statements. The container factor rests
  on 1.
- **The harness covers 6 criteria on a box section.** It does not run a `.vdesign`, a real hull form, free
  surface (every case has 0), the weather criterion, or any IGC, inland or OVCD Check rule.
- **No verdict in the harness is published.** All are derived by the oracle. Oracle and engine could share a
  misreading of the IS Code text; nothing outside them checks it.
- **`calibration/results.md` has no freshness check.** CI runs `--check` but does not compare the committed file
  with a new run, as it does for `evaluation/results.md`. The figures on this page come from the committed file.
- **Cost is outside every check.** `cost` is out of the results hash, the design-impact comparison and web parity.
- **Web sources have no checksum.** ADR 0001 asks for a checksum for each source; the 107 `web_sources` have none.
- **A source note is out of date.** `calibration/README.md` says one operator's barge database is unreachable; 39 `K`
  sources now cite Wayback captures of it.
- **21 Rust test files are unlabelled**, so a reader cannot tell sourced numbers from snapshots there.
- **Mutation testing is not a gate** (`docs/agents/build-process.md`). Rule-data mutation (moving a limit to see
  that a case notices) is planned in spec 0004 and does not exist yet.

## Evidence

### Evidence by area

| # | Area | Evidence today | Gap |
|---|---|---|---|
| 01 | Hull form and hydrostatics | Box-barge Cb fit (7 cases) and 30 held-out box DWT figures inside band; harness checks KB, BM, GM of a box against closed forms and two published GM/KM values; sourced Vessel Class tests (PIANC WG 141 tables) | No case for a shaped hull (V-bottom, chine, raked ends); no published hydrostatic table compared; Cb default in the profile is not the fit |
| 02 | Weights and centres | Lightship k fit (3 + 1 in-sample); 37 held-out lightship figures (26 inside band); tank steel 1 case (in-sample) | No self-propelled lightship case; no centre of gravity case; pontoons 10 of 22 outside band; Pusher lightship never published in the records |
| 03 | Intact stability | Harness: 5 cases, 60 mutants, 361 verdicts, 0 false pass, 0 false fail; sourced closed-form tests | Box only; no free surface; no published verdict; no weather criterion; gas `assess` KG path not tested by the harness |
| 04 | Type C tanks | 2 tank cases (volume, wall, steel: 5 of 5 inside band); sourced wall-formula test (one paper) | 2 cases, one a design study; no bilobe; no insulated tank steel weight; structure factor rests on 1 tank |
| 05 | Gas cargo arrangement | Sourced tests for IGC 2.4.1.1 d(Vc) and inland gas separation clauses; design-impact sweep of every gas Cargo | No published gas barge arrangement run (6 gas records not runnable); no harness case for any IGC rule |
| 06 | Liquid, dry and container cargo | 90 dry-barge records; box DWT 30 of 30 inside band | 41 tank barges not runnable; container rests on 1 case; no container stowage case |
| 07 | Resistance and powering | 14 design-note points (ratio 1.72 to 2.11); sourced wave and squat tests | No built-vessel trial point; 10 pusher records not run; no push-tow or tug-barge case |
| 08 | Waterway and route | Sourced squat-formula and wave (ITTC) tests; design-impact channel cases | No published squat or journey measurement compared |
| 09 | Rules and Rule Sets | Spec 0004 audit of 205 rule-like items (23 match their source, 11 differ, 4 of those still open); Lean proofs of a kernel model | No Rule is Verified; the Rust rule evaluator and Rule Trace do not exist; no rule-data mutation |
| 10 | Cost, BOQ and capital budget | One sourced hand-calculation test of the cost sum | No published cost case; `cost` outside design impact and every hash |
| 11 | Drawings | Rust against Python drawing standard on 6 fixtures; DXF read back by two readers; app screenshots | Checks only agreement and format, not that a drawing shows the right arrangement |

### Calibration (as committed in `calibration/results.md`)

In-sample 13 of 14 inside; held out 67 of 87 inside; tanks 5 of 5 inside. Detail in sections 1 to 5 of the
report.

### Evaluation harness (`evaluation/results.md`)

5 base cases, 60 mutants, 361 verdicts, 0 false pass, 0 false fail, full criterion coverage.

### Tests

`tests/test_calibration_runs.py`, `test_calibration_held_out.py`, `test_calibration_tanks.py`,
`test_calibration_folder.py` (the calibration machinery); `test_evaluation_oracle.py`,
`test_evaluation_runner.py` (the harness); `test_design_impact*.py` (the design-impact check);
`test_gates_fail_closed.py` (the fast gate cannot skip a step).

## Questions for reviewers

1. Are lightship ±15 / 20 / 30 % and deadweight ±7.5 / 10 / 15 % fair accepted errors for concept design of
   inland barges? Should they depend on hull type?
2. Is a mean of 3 lightship ratios a sound basis for the box-hold k? Would you weight by confidence?
3. Pontoon barges come out about 27 % light when run as box barges. Should pontoons get their own factor, or is
   the box model wrong for them?
4. What criteria should set a case's confidence (high, medium, low)? Today it is judgement.
5. The motor-vessel installed power is 1.7 to 2.1 times computed deep-water delivered power. What ratio would
   you expect from a normal sea margin and engine margin on an inland vessel?
6. Should a calibration band ever block a release (not a merge)? At what case count?
7. Which published stability booklets or worked examples with an IS Code 2.2 verdict could join the harness as
   `published` cases?
8. Are the tank bands (volume ±3 / 5 / 10 %, wall ±10 / 15 / 25 %, steel ±15 / 20 / 30 %) fair for Type C tanks?
9. Is "derived by an independent oracle" enough for a known verdict, or must each harness verdict also have an
   expert's sign-off?
