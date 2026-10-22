# Calibration

`cases.yaml` holds real vessels (and published design points) with their sources.
`python -m vessel_designer.calibration` runs them through the core and writes `results.md`.
The pass **measures**; it does not tune. Each correction is a separate change with its evidence.

## Calibration Bands

`bands.yaml` gives each figure a band per confidence level (the `confidence` of the case):
lightship ±15 / 20 / 30 % and deadweight ±7.5 / 10 / 15 % at high / medium / low. A case's band and its
basis are therefore its figure plus its confidence. `results.md` shows each calibrated result against
its band and marks cases outside it. `python -m vessel_designer.calibration --check` prints one
`::warning` annotation per case outside its band; CI runs it. **A band miss is a warning, never a
failure**, while the model is in development; a crash still fails. The errors of the first 12 cases
are **in-sample**: the factors in `factors.yaml` were fitted to the same cases (the container factor to
one case). The errors of held-out cases are **out of sample** (next section).
To add a case, give it a `confidence`; it is banded from then on.

## Case confidence (draft, for review)

The `confidence` of a case sets its band. The criteria below are a **draft** written from how the 90 dry-barge
cases are marked today (1 high, 50 medium, 39 low). They are not yet agreed. Nothing in the harness reads them.

- **high**: the source is the owner's or builder's own specification of this one vessel, and the figures agree
  with each other (a draught, a lightship and a deadweight that give a Cb a barge can have).
- **medium**: the source is a builder, owner or operator sheet or table, or a thesis table, with the vessel's
  dimensions and either a lightship or a deadweight, and nothing in the figures disagrees.
- **low**: the source is a broker listing with blank fields, a practitioner statement, a table value read at one
  draught, or figures that cannot all be true together (the case notes say which).

A case whose figures disagree is `low` or is excluded from a fit with its reason written on the case.

## Web source checksums

A checksum of each web source must be **attempted**. It need not succeed. Each entry of `web_sources` in
`sources.yaml` records which happened:

- **achieved:** `retrieved` (the date the page was fetched) and `sha256` (of the bytes served). A page that has
  changed since gives a different hash, so the entry shows it.
- **not achieved:** `checksum_attempted` (the date) and `checksum_failed` (the reason).

A missing checksum raises no warning and fails no gate. On 2026-10-22, 119 of 122 pages were fetched; the other
three (S14, K10, P1) have the second form.

## Held-out cases (issue #3)

A case with a `held_out` marker (a one-line reason) is run and banded but **never used to fit** k or Cb:
`fit_factors` drops it, so `factors.yaml` does not move when held-out cases are added. The 62 cases
added for issue #3 from the research note are all held out, and so are 64 more found by follow-up
research the same day (`found_by`, no `note_suggests`). Each carries the source text as printed (`published`), a `source`
that starts with an id in `sources.yaml` (`web_sources`), and, for the note's 62, what the
research note suggested (`note_suggests`).

- **Deck barges** have `hold_type: deck` and `run_as: box`. The core has no deck model, so they run
  with the box k and box Cb. This tests whether the box model carries over. There is no deck factor.
- **Pontoons** (`hold_type: pontoon`, `run_as: box`) are flat-top pontoon barges from the follow-up batch.
  They run as box barges and get their own summary group, "pontoon (run as box)". A published DWT with
  no draught is kept as `observed.dwt_t_draught_not_stated` and in `published`; it is never banded.
- **`water_density_t_m3`** (1.025 for sea-going or saltwater figures) sets the density of the predicted
  DWT. Left out, it is 1.000.
- **`not_yet_runnable_cases`** (tank, gas and the motor container point) have no hull model in the
  calibration path. `results.md` lists their figures and the reason; they are never run.
- **`cross_check_only`** and **`exclude_from_fit_cb`** cases are shown in section 4 of `results.md`
  but left out of its summary. A cross-check has no band.
- A figure the source does not give is left out of the case. Nothing is estimated or filled in.

## Type C tank cases (issue #292)

`tank_cases.yaml` holds published cargo tanks, run through the tank maths only (one tank, no hull), with
their own bands in `tank_bands.yaml` (starting values in the shape of the barge bands: volume ±3 / 5 / 10 %,
wall thickness ±10 / 15 / 25 %, steel weight ±15 / 20 / 30 % at high / medium / low confidence; the maintainer
sets them). They are kept apart from `cases.yaml` because they do not run a hull. The conventions are the
same: every case is held out (run, banded, never used to fit), it cites a listed source (`web_sources`,
ids `P1` and up), `published` keeps the source text, and a figure the source does not give is absent.
A band miss is a warning, never a failure; `--check` prints one annotation per tank miss too.

- **The datum is stated, not assumed.** The engine's tank outer diameter is the outside of the insulation.
  A source prints the diameter on another face, so each case gives `diameter_basis` and `length_basis`, and
  the runner brings them to the engine's datum (a case without them cannot run). The report shows the steel
  and the insulated diameters it used.
- **What is run.** Volume and steel weight at the published shell thickness (the geometry and the datum are
  what is tested); the wall thickness from the pressure-vessel formula at the steel diameter, with the steel
  the case says it is run as. `run_as_head` and `run_as_material` are the nearest the engine has; the gap is
  part of the result.
- **In-sample warning.** The structure factor of fully pressurised LPG in the Defaults Profile (1.12) was worked
  out from the one published tank steel weight (the first case). That case's steel error is in-sample.
- **Not yet runnable.** Bilobe tanks, a record with no length, and a double-shell fuel tank are listed in
  `not_yet_runnable_tank_cases` with the reason.

## Tank barges (issue #335)

`python -m vessel_designer.calibration` also runs every tank barge: the 41 `hold_type: tank` records of
`cases.yaml` and the 12 reference vessels of `tank_barge_cases.yaml` (the vessels of
`docs/research/crude-oil/03-design-practice.md` section 3 that have full data and were not yet in
`cases.yaml`; the other four, ROB20, 004ROB05, ROB07 and 1635T, were). The model is the one of
`tank_lightship.yaml`: lightship = k × L × B × D + outfit, and the deadweight at the stated draught is the
box-barge Cb × L × B × T × ρ less that lightship. Results are in section 5a of `results.md`; the bands are
in `tank_barge_bands.yaml`, starting values for the maintainer, equal to the dry-barge bands.

- **In-sample and held out.** The 15 cases of the k-fit subset (`tank_lightship_inputs.yaml`) are
  **in-sample**: k was fitted on them from the same deadweights, so their errors say little. Every other
  case is held out: run, banded, never used to fit. The report labels each row and gives both summaries.
- **No band where the model does not apply.** A self-propelled vessel has no band: the model has no
  machinery, accommodation or fuel, so its deadweight comes out too high by about that weight (the
  error is shown). A cargo capacity is not a deadweight: it is shown against the predicted deadweight and
  has no band. `not_banded` on a case says why.
- **Cite or skip.** Each new vessel was read on its source page (`web_sources` M15 to M25 and R1, with the
  checksum of the page as served). A figure the page does not give is absent. The estimated lightships of
  the research note are derived, not published, so no record has an observed lightship; the lightship band
  is set and unused.
- A band miss is a warning, never a failure; `--check` prints one annotation per miss.

## Pass 1 findings (core as first ported)

1. **Barge lightship is overestimated by 31–111 %.** The bulk branch adds the ported sloped-hopper
   structure (14 mm inner bottom and hopper plates, × structure factor 1.4) on top of 0.10 t/m³ × L·B·D.
   Real US, Paraná and Danube barges are box holds and much lighter. Observed LS/(L·B·D):
   Pará Mississippi 0.095, Danube 0.122, Victor container barge 0.171 (shallow 2.8 m depth with a
   20 mm sheer strake, so a high ratio). Padovezi's 0.12 is closest on average (−30 % to +26 %).
   The CN draft formula overestimates barges (+28 % to +76 %); it was fitted to motor vessels.
2. **Default Cb = 0.85 is too low for barges.** Implied Cb is 0.90–1.0 on LOA; DWT comes out
   19–33 % low. Two cases imply Cb > 1 (Pará 1.035, LHG 1.013), which is impossible on LOA —
   so their DWT, lightship or units do not belong to the same condition. Flagged, not used to fit.
3. **The Yangtze design points are themselves model output.** Every row implies exactly
   Cb = 0.835 on LOA, so the draft note computed displacement from a fixed Cb. They are a check on
   *our* model against *theirs*, not against built ships.
4. **Installed power is 1.7–2.1 × our deep-water delivered power.** A normal margin
   (85 % MCR, 15 % sea margin) gives about 1.4 ×. The remaining gap is either shallow-water
   resistance on the Yangtze (h/T not stated in the note) or the core under-predicting resistance
   for full inland hulls. Needs a built vessel with a trial speed–power point to separate the two.

## Corrections applied

- **Barge lightship by hold type.** `BargeCapacityInputs.hold_type` = `box` | `container` uses
  k × L·B·D; `sloped_hopper` (the default) keeps the ported build-up unchanged. k is fitted by the
  harness into `factors.yaml` — box 0.109 t/m³ (n = 3, range 0.095–0.122), container 0.171 (n = 1,
  **one point: in-sample, not validated**).
- **Box-barge Cb = 0.928 on LOA** (n = 7, range 0.883–0.995), fitted the same way. Two cases
  excluded with reasons recorded.
- Result on the cases: box-barge lightship within −11 % to +15 %; deadweight within −8 % to +6 %
  on the fitted cases. These are **in-sample** errors. The next real lightship is the first
  out-of-sample test.
- Kirby's barge database (draft/tonnage tables from 0 t): the live site did not answer, so sources
  K1 to K39 in `sources.yaml` cite archived copies of its pages. Still the best source of more
  lightships.

## Proposed corrections — original list

- Barge lightship: replace the hopper-structure term for box barges with a calibrated
  k × L·B·D, k fitted per hold type (box hopper ≈ 0.095–0.12; container ≈ 0.17), and keep the
  structural build-up only for real sloped-hopper holds. Needs ≥ 3 more observed lightships.
- Cb: make it a required Brief input; default per Hull family from the implied values
  (box barge ≈ 0.93–0.97).
- Power: hold until a built-vessel speed–power point exists; report the margin explicitly.

## Tank-barge lightship (issue #330)

`tank_lightship_inputs.yaml` declares the subset of the 41 tank-barge records that k is fitted on, the
records left out of it with the reason, and the outfit coefficients (all Estimates). `python3
tools/fit_tank_lightship.py` writes `tank_lightship.yaml`: k, the implied lightship and the model error of
every record, in-sample and held out. The other records stay held out: run, compared, never fitted. A
record gives a deadweight, not a lightship, so each lightship is implied with the box-barge block
coefficient (an assumption), and k is an Estimate. The held-out error is large; it is reported, not
tuned away. The Rust constants in `crates/ovcd-core/src/liquid_bulk/lightship.rs` equal the generated file,
and a test keeps them equal.
