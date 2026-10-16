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

CI runs `--check` on every pull request, in the `test` job. **A false pass fails the build**: the
engine passes a criterion a case expects to fail. A false fail (the engine fails a criterion expected to
pass) and a coverage gap (a criterion with no passing or no failing case) are warning annotations, as
the Calibration Bands are (issue #119). A `results.md` that is not what the run writes also fails.
`results.md` reports the false-fail rate, the false-pass rate and the per-criterion coverage.

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

## Why phase 0 uses particulars

The core checks the IMO Intact Stability Code 2008, Part A 2.2 only inside the gas `assess()`, and
there the KG comes out of the gas arrangement, so a `.vdesign` cannot set a known KG. Phase 0 therefore
takes box particulars (beam, depth, draught, KG, free surface) and calls `box_condition`, the
per-condition work `assess` does, made callable on its own (`ovcd_core::gas_stability::box_condition`;
`evaluation/engine.py`, the harness's one call into the engine, outside the package's public API). A later phase runs whole `.vdesign` files; the case
format already has a `vdesign` field for that.

## Case format

One YAML file per case in `cases/`:

| Field | Meaning |
|---|---|
| `id`, `title`, `kind` | the file stem, a name, and `box` (the only kind in phase 0) |
| `inputs.form` | `particulars` in phase 0 |
| `inputs.vdesign` | path to a `.vdesign` for later phases; `null` for particulars |
| `inputs.length_m`, `beam_m`, `depth_m`, `draught_m`, `kg_m`, `water_density_t_m3`, `free_surface_m` | the box. The draught is given, so the length and the water density do not enter the criteria |
| `expected.<criterion>` | for each of the six criteria below: `rule` (citation), `what`, `unit`, `verdict` (`pass` or `fail`), `value` (the governing value), `limit`, `margin` (value minus limit) |
| `provenance.verdict_basis` | `published` (the verdict is in the source) or `derived` (worked out by the oracle) |
| `provenance.source`, `edition`, `locator`, `licence`, `confidence`, `inputs_basis` | where it comes from: the document, the edition, the page or paragraph, the licence, how far to trust it, and where the inputs came from |
| `mutant` (mutants only) | `parent` (case id), `criterion`, `side` (`pass` or `fail`), and `grid: true` when the step is inside the engine's grid error |

The six criteria (keys of `expected`): `area_0_30`, `area_0_40`, `area_30_40` (Part A 2.2.1, limits
0.055, 0.090 and 0.030 m rad), `gz_30_plus` (2.2.2, GZ at least 0.20 m at 30 degrees or more),
`angle_max_gz` (2.2.3, at least 25 degrees) and `gm` (2.2.4, at least 0.15 m).

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
