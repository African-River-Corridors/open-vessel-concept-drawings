# Evaluation harness

A growing set of designs whose compliance is already known, each with its expected verdict per rule.
The runner (added in the second part of phase 0, issue #184) runs every case through the engine and
compares verdicts, so that a disagreement shows whether the rule or the code is wrong. See
`ROADMAP.md`, "Evaluation harness".

This folder holds:

- `cases/`: the corpus (format below).
- `mutants.yaml`: the boundary mutants, generated from the cases by `oracle/mutants.py`.
- `run.py`: the runner. `results.md`: its generated report.
- `oracle/box_oracle.py`: an independent calculation of the IS Code criteria for a box section. It
  never calls the engine and shares no code with it.
- `oracle/write_cases.py`: writes the base case files from the oracle. Expected values of a derived
  case are printed by the oracle, never typed by hand. `--check` fails if a file differs.

## Run it

    python3 evaluation/run.py            # run every case and mutant; write evaluation/results.md
    python3 evaluation/run.py --check    # what CI runs

CI runs `--check` on every pull request, in the `test` job. The runner reads the engine's Rule Trace
of each case (`box_trace`, the built-in set `is2008-a` 1.0.0) and compares each row with the case's
expected verdict, keyed by rule id (issue #278). **A false pass fails the build**: the engine passes a
blocking Rule a case expects to fail. A false fail (the engine fails a Rule expected to pass) and a
coverage gap (an evaluated blocking Rule with no passing or no failing case) are warning annotations, as
the Calibration Bands are (issue #119). A `results.md` that is not what the run writes also fails.

### Triage

Each disagreement outside the grid error goes in one bucket, in this order:

| Bucket | When |
|---|---|
| `model-gap` | the Rule is `not evaluated` or has no row, or the case has an input the box trace does not read. A `not evaluated` row against an expected `fail` is a model gap, never a pass |
| `known-limitation` | the Rule links a finding (`finding` in its row) |
| `code-bug` | the oracle and the engine disagree. A derived case's verdict is the oracle's |
| `rule-suspect` | the oracle and the engine agree with each other, but not with a published verdict |
| `needs-expert` | the rest: for example a published verdict with no oracle verdict to compare |

A false pass on a blocking Rule fails the build whatever its bucket. `results.md` lists, per Rule Set
and per Rule by id, the passing and failing case counts, the coverage, and the buckets.

### Rule mutation

`rule_mutation.py` (issue #279) tests the corpus, not the engine. For each numeric limit and coefficient
of each evaluated Rule of a selected set, it writes a mutated copy of the set with that one number moved
by a step to each side. The copy keeps the family, so its rule ids are the built-in ones, and takes the
set id `<id>-mutant-<n>`. It is loaded at run time next to the built-in sets, as a `.vdesign` run loads
one, and selected alone. The corpus runs on it. When no case changes its verdict on the mutated Rule,
the corpus is **weak** there: no case sits close enough to the limit to notice. That is a warning
annotation (`Evaluation weak corpus`), never a failure. `results.md` lists every mutant and the cases
that notice it.

The steps: a constant limit moves by the big step of its unit, above the engine's grid error (m rad
0.002, m 0.01, degrees 2; GM, in metres, takes the GZ step, above its own 0.005 m). A formula
coefficient, a table value, or a constant in another unit moves by 1 % of its value. A table's `when`
bounds say where a row applies; they do not move.

The whole harness, rule mutation included, runs in about ten seconds (a debug build of the engine), so it
stays in the pull-request gate (`--check`, in the `test` job).

## Boundary mutants

`oracle/mutants.py` makes the mutants; none is written by hand. For each base case and each of the six
criteria it sets the criterion's governing value to its limit plus and minus a step (so each criterion
is tested from both sides), with a big step and a small one:

- A **big step** is above the engine's grid error (the trapezoid rule on a 1 degree grid, and a
  whole-degree angle). The engine must get its verdict right.
- A **small step** is inside the grid error on purpose and has `grid: true`. Grid criteria are run and
  listed, but never counted as a false fail or a false pass.

The steps: areas 0.002 m rad (small 0.00002), GZ 0.01 m (0.0001), angle 2 degrees (0.2), GM 0.005 m (0.001;
the engine's GM is exact). A criterion whose expected margin is inside its grid tolerance (areas 0.0005
m rad, GZ 0.002 m, angle 1 degree, GM 1e-6 m) is marked `grid: true` for that mutant, whichever criterion
the mutant targets. The tolerances are above the largest engine error seen on the corpus
(`tests/test_evaluation_runner.py` checks it).

Which input moves: **KG** for GM, the three areas and GZ at 30 degrees or more (at fixed displacement GZ
changes by -dKG x sin(phi), so GM moves by -dKG and each area by -dKG x (cos a0 - cos a1); the oracle
finds the KG, by bisection for GZ at 30 degrees); **depth** for the angle of maximum GZ (a lower depth
brings the deck edge under sooner, and the oracle finds the depth by bisection). The expected verdict of
all six criteria is recorded for every mutant, from the oracle. When no KG on the hull (between keel and
deck) crosses a limit, as for box A (whose margins are large), the mutant is not made and `results.md`
lists it.

## Liquid bulk cases (`kind: tank`, issue #338)

The liquid bulk Rule Sets (`adn-tank`, `marpol-i`) are tested the same way, with their own oracle. A `kind: tank`
case gives a crude oil layout: `loa_m`, `beam_m`, `depth_m`, the cargo zone (`zone_aft_m`, `zone_fwd_m`), the
layout (`double_side_m`, `double_bottom_m`, `tanks_in_line`, `centreline_bulkhead`, `cofferdam_m`, `pump_room_m`,
`slop_share`), the particulars `deadweight_t`, `service_area` and `adn_vessel_type`, and the sets it selects
(`rule_sets`). The runner calls the engine's `liquid_trace` (`engine.py`): the layout is worked out by the core,
the Rule Trace is the one a run writes. `expected` is keyed by rule id, as for the box cases: a Rule with a row per
cargo tank has the verdict of its worst tank.

- `oracle/tank_oracle.py` is independent (standard library only). It lays the tanks out from the case's inputs and
  works out each limit from the text, not from the Rule Set data: the ADN capacity table as printed (180 + (L x B x
  H - 600) x 0.0635), the tank length limit, the cofferdam, the double side and bottom of type C (1.00 m, 0.70 m
  and 0.60 m) and type N (0.60 m, 0.50 m), MARPOL Reg. 19.6.1 and 19.6.2 for tankers of 600 to under 5,000 tonnes
  deadweight on coastal service, and Reg. 29 (3 per cent). A Rule whose limit is worked out (a table row, a
  formula) has a tolerance of 1e-9 in the unit of the limit, in the Rule Set and in the oracle.
- `oracle/write_cases.py` writes the base cases (`TANK_BASES`: a hull, a layout and the particulars; none is from a
  publication) and `oracle/mutants.py` writes `tank_mutants.yaml`: for each evaluated Rule of each base case, the
  input that moves the Rule's governing value (the double side, the double bottom, the cofferdam, the slop share,
  or the cargo zone for a tank's volume and length) set one step inside and one step across the limit, with a big
  and a small step. A limit that no cargo zone inside the hull reaches has no boundary case (`skipped`).
- The rule-data mutation runs over these cases for `adn-tank` and `marpol-i`. The base cases are chosen so that every
  row of the capacity and length tables, and each of the 0.76 m minimum and the formula of Reg. 19.6, governs
  somewhere: no mutant goes unnoticed.
- The Rules of these sets enter as `warn`, so a disagreement is a false fail, a warning, never a false pass.
- Not evaluated: ADN 9.3.x.14.2 (stability), 9.3.x.17.6 (pump room), MARPOL Reg. 19.3, 19.7, 23, 27
  and 28 (above 5,000 tonnes, collision bulkhead, outflow, stability) and NORMAM-202 5.23.3, 5.23.8 and 5.23.9; each
  says what is missing in its `gap`.
- ADN 9.3.x.15.1 (damage stability) is evaluated in a run (issue #341), where the damages come from the laden
  departure condition. A `tank` case has no loading condition, so its row is `not evaluated` with that reason, and
  the harness has no case for it; `crates/ovcd-core/tests/damage_stability.rs` checks the calculation against closed forms.

## Why phase 0 uses particulars

The core checks the IMO Intact Stability Code 2008, Part A 2.2 only inside the gas `assess()`, and
there the KG comes out of the gas arrangement, so a `.vdesign` cannot set a known KG. Phase 0 therefore
takes box particulars (beam, depth, draught, KG, free surface) and calls `box_condition`, the
per-condition work `assess` does, made callable on its own (`ovcd_core::gas_stability::box_condition`;
`evaluation/engine.py`, the harness's one call into the engine, outside the package's public API).
`box_trace` evaluates the selected Rule Sets on that one loading condition, named "Box", and gives
the Rule Trace a run writes. A later phase runs whole `.vdesign` files; the case format already has a
`vdesign` field for that.

## Case format

One YAML file per case in `cases/`:

| Field | Meaning |
|---|---|
| `id`, `title`, `kind` | the file stem, a name, and `box` (the only kind in phase 0) |
| `inputs.form` | `particulars` in phase 0 |
| `inputs.vdesign` | path to a `.vdesign` for later phases; `null` for particulars |
| `inputs.length_m`, `beam_m`, `depth_m`, `draught_m`, `kg_m`, `water_density_t_m3`, `free_surface_m` | the box. The draught is given, so the length and the water density do not enter the criteria |
| `expected.<rule id>` | for each of the six Rules below: `rule` (citation), `what`, `unit`, `verdict` (`pass` or `fail`), `value` (the governing value), `limit`, `margin` (value minus limit); optionally `oracle` (the oracle's verdict, for a case whose verdict is `published`) |
| `provenance.verdict_basis` | `published` (the verdict is in the source) or `derived` (worked out by the oracle) |
| `provenance.source`, `edition`, `locator`, `licence`, `confidence`, `inputs_basis` | where it comes from: the document, the edition, the page or paragraph, the licence, how far to trust it, and where the inputs came from |
| `mutant` (mutants only) | `parent` (case id), `criterion` (the rule id), `side` (`pass` or `fail`), and `grid: true` when the step is inside the engine's grid error |

The six Rules (keys of `expected`, the ids of `is2008-a`; the oracle's own names in brackets):
`is2008/A.2.2.1/area-0-30`, `is2008/A.2.2.1/area-0-40`, `is2008/A.2.2.1/area-30-40` (`area_0_30`,
`area_0_40`, `area_30_40`; Part A 2.2.1, limits 0.055, 0.090 and 0.030 m rad), `is2008/A.2.2.2/gz-30`
(`gz_30_plus`; 2.2.2, GZ at least 0.20 m at 30 degrees or more), `is2008/A.2.2.3/angle-max-gz`
(`angle_max_gz`; 2.2.3, at least 25 degrees) and `is2008/A.2.2.4/gm0` (`gm`; 2.2.4, at least 0.15 m).
`oracle/write_cases.py` holds the map (`RULE_IDS`); the oracle keeps its own names, so it shares nothing
with the engine.

## Sources, and what was read

Every source is opened and read before a number from it is used. Cite or skip.

1. **The limits.** IMO resolution MSC.267(85), annex, *International Code on Intact Stability, 2008*,
   Part A 2.2.1 to 2.2.4. Read in the certified true copy at
   <https://law.resource.org/pub/us/cfr/ibr/inc/imo/imo.msc.267.2008.pdf> (text read with `pdftotext`;
   2.2.1 and 2.2.2 are on printed page 11, 2.2.3 and 2.2.4 on page 12): not less than 0.055 m rad up to
   30 degrees and 0.09 m rad up to 40 degrees (or the downflooding angle if less), 0.03 m rad between 30 and
   40 degrees; GZ at least 0.2 m at 30 degrees or more; the maximum GZ at not less than 25 degrees;
   initial GM not less than 0.15 m. A box has no downflooding openings, so the 40 degree limit applies.
   Licence: IMO text, quoted for the limits only.
2. **Hull particulars of the cases.** Box A and box B are chosen for this corpus. Their verdicts are
   `derived`: worked out by the oracle, which is checked against closed forms (GM = KB + BM - KG and
   the wall-sided GZ formula) in `tests/test_evaluation_oracle.py`.

2a. **The crude oil tank barge cases** (`cases/liquid_*.yaml`, issue #332, spec 0006). A tank barge with three
   rectangular cargo tanks, worked from closed forms in `oracle/write_cases.py` (`liquid_particulars`): the
   draught is the displacement over L x B x rho, the KG the mean of the lightship and the cargo, and the free
   surface correction rho x n x l x b^3 / 12 over the displacement for a slack tank (a tank at 98 % or more is
   nominally full and has none, IS Code 2008 Part B 3.1.2). `liquid_slack` has no centreline bulkhead and fails
   the angle of maximum GZ; `liquid_slack_centreline` is the same hull with one (a quarter of the free
   surface moment) and passes. They use the `kind: box` form with
   `free_surface_m`, so the box oracle and the engine's `box_condition` are the same as for the other cases. The
   verdicts are derived by the oracle and the inputs are particulars chosen for the corpus, not a publication.

3. **The generic barge** (`cases/generic_barge.yaml`). Marlantes, K. E.; Kim, S.; Hurt, L. A. (authors as
   printed on the paper), "Implementation of the IMO Second Generation Intact Stability Guidelines",
   *J. Mar. Sci. Eng.* 2022, 10(1), 41, published 30 December 2021, <https://doi.org/10.3390/jmse10010041>,
   licence CC BY 4.0 (<https://creativecommons.org/licenses/by/4.0/>; attribution to these authors).
   Section 5.1, "Barge": "a simple uniform rectangular cross section". Table 3 (principal dimensions):
   L 100.00 m, B 20.00 m, D 10.00 m, full draught 5.00 m, CB 1.00, CM 1.00. Table 4 (loading condition):
   draughts aft and fore 5 m, KG 7 m, **GM 2.167 m**, volumetric displacement 10 000 m3, and the angle at
   which openings immerse, 40 degrees (so the 2.2.1 areas run to 40 degrees unchanged). Table 5 is the
   second generation dead-ship level 1 check, not an IS Code 2.2 check, and is not used. *How it was read:*
   in a browser by the maintainer's build coordinator on 2026-10-16, because the publisher blocks scripted
   downloads (HTTP 403). The inputs are `published`; the paper gives no IS Code 2.2 verdict, so the verdicts
   are `derived` by the oracle. The published GM is a cited intermediate value: KB 2.5 + BM 6.667 - KG 7 =
   2.167, and the oracle and the engine both reproduce it to the printed three decimals
   (`tests/test_evaluation_oracle.py`).

4. **The Navy LCU** (`cases/lcu_empty.yaml`, `cases/lcu_loaded.yaml`). US Naval Academy, Naval
   Architecture and Ocean Engineering Department, *EN400 Principles of Ship Performance, Course Notes*,
   Version: Summer 2020,
   <https://www.usna.edu/NAOE/_files/documents/Courses/EN400/EN400_Course_Notes_Summer_2020.pdf>.
   Chapter 4 (Stability), end-of-chapter problem 33, printed page 4-51 (PDF page 190 of 484). *Licence:*
   not stated; course notes of a US federal academy, likely a US Government work (17 U.S.C. 105), not
   verified. Only numbers are used; the PDF is not committed. *How it was read:* downloaded and read with
   `pdftotext` on 2026-10-16. Printed: "A Navy LCU can be modeled as a rectangular, box-shaped barge", length
   135 ft, beam 35 ft, depth 7 ft, deck 7 ft above the keel, gunwales 4 ft above the deck; draft 4, 5, 6 ft
   with KM 27.5, 22.9, 20.0 ft and TPI 11.25 LT/in. Empty: draft 4 ft, KG 1.9 ft. Loaded to maximum capacity
   with 3 M-60 tanks, 60 LT each, centre of gravity 4 ft above the deck.
   - `lcu_empty`: draft 4 ft, KG 1.9 ft. Cited value: KM 27.5 ft (a box gives 27.52), GM 25.6 ft.
   - `lcu_loaded`: the draft and the KG are worked from the printed numbers (the arithmetic is in the case
     file, `worked_from_printed`): the notes derive TPI = Awp/420 from 64 lb/ft3 (so 35 ft3 per long ton);
     180 LT / 11.25 LT per inch = 16 in, so the draft is 5.333 ft; the empty displacement is 540 LT (the box
     at 4 ft); KG = (540 x 1.9 + 180 x 11) / 720 = 4.175 ft. The printed KM at 5 ft (22.9) and 6 ft (20.0)
     bracket the box KM at 5.333 ft (21.81); linear interpolation of the printed table gives 21.93, a little
     higher, because the curve is convex.
   - Feet are converted to metres with 0.3048. The deck edge is at the depth, 7 ft: the gunwales that
     extend 4 ft above the deck are not watertight hull and are not modelled. The source gives no
     downflooding angle, so none is set. Parts (d) to (g) of the problem (water on deck) are not used.
   - The notes give no IS Code 2.2 verdict. The verdicts are `derived` by the oracle and recorded as it
     gives them; no input is adjusted to make a verdict pass. The loaded LCU fails 2.2.3 (the angle of
     maximum GZ is 23.4 degrees: a wide shallow box loses its righting lever when the deck edge goes under).

**Cases with an external published source for the inputs: three** (`generic_barge`, `lcu_empty`,
`lcu_loaded`), with verdicts **derived** by the oracle in every case, because no free source with a
published IS Code 2.2 verdict for a box-like hull was found. **Two** cases (box A, box B) are derived
entirely. A published case is added only when its source has been opened and its page or section
recorded.

## Assumptions of the oracle

- The hull is a prismatic box of constant displaced area `B x T`, as in the engine.
- GZ is evaluated from 0 to 60 degrees, the engine's range; 2.2.2 is checked on 30 to 60 degrees.
- No downflooding angle: the area limit applies up to 40 degrees.
- Areas are integrated with Simpson's rule on a 0.01 degree step; the angle of maximum GZ is the
  continuous one. The engine uses the trapezoid rule on a 1 degree grid and a whole-degree angle, so
  its values differ by up to the grid error (areas about 1e-4 m rad on box B, the angle by up to 1 degree).
