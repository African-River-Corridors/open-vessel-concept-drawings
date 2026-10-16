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

**Cases with an external published source for the inputs: none yet.** A generic barge in a 2022 open
access paper on the second generation intact stability guidelines (Marlantes, Kim and Hurt,
*J. Mar. Sci. Eng.* 10(1), 41, <https://doi.org/10.3390/jmse10010041>, CC BY 4.0) was planned as a
published case. The publisher refused the page and the PDF (HTTP 403), so the numbers could not be
checked and none is recorded here. A search for another free worked example of a box-like hull with a
published IS Code 2.2 verdict found none. The corpus therefore counts two derived cases and no
published ones. A published case is added only when its source has been opened and its page recorded.

## Assumptions of the oracle

- The hull is a prismatic box of constant displaced area `B x T`, as in the engine.
- GZ is evaluated from 0 to 60 degrees, the engine's range; 2.2.2 is checked on 30 to 60 degrees.
- No downflooding angle: the area limit applies up to 40 degrees.
- Areas are integrated with Simpson's rule on a 0.01 degree step; the angle of maximum GZ is the
  continuous one. The engine uses the trapezoid rule on a 1 degree grid and a whole-degree angle, so
  its values differ by up to the grid error (areas about 1e-4 m rad on box B, the angle by up to 1 degree).
