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

## Held-out cases (issue #3)

A case with a `held_out` marker (a one-line reason) is run and banded but **never used to fit** k or Cb:
`fit_factors` drops it, so `factors.yaml` does not move when held-out cases are added. The 62 cases
added for issue #3 are all held out. Each carries the source text as printed (`published`), a `source`
that starts with an id in `sources.yaml` (`web_sources`), and what the research note suggested
(`note_suggests`).

- **Deck barges** have `hold_type: deck` and `run_as: box`. The core has no deck model, so they run
  with the box k and box Cb. This tests whether the box model carries over. There is no deck factor.
- **`water_density_t_m3`** (1.025 for sea-going or saltwater figures) sets the density of the predicted
  DWT. Left out, it is 1.000.
- **`not_yet_runnable_cases`** (tank, gas and the motor container point) have no hull model in the
  calibration path. `results.md` lists their figures and the reason; they are never run.
- **`cross_check_only`** and **`exclude_from_fit_cb`** cases are shown in section 4 of `results.md`
  but left out of its summary. A cross-check has no band.
- A figure the source does not give is left out of the case. Nothing is estimated or filled in.

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
- Kirby's barge database (draft/tonnage tables from 0 t) is unreachable from this machine
  (host times out; kirbycorp.com itself answers). Still the best source of more lightships.

## Proposed corrections — original list

- Barge lightship: replace the hopper-structure term for box barges with a calibrated
  k × L·B·D, k fitted per hold type (box hopper ≈ 0.095–0.12; container ≈ 0.17), and keep the
  structural build-up only for real sloped-hopper holds. Needs ≥ 3 more observed lightships.
- Cb: make it a required Brief input; default per Hull family from the implied values
  (box barge ≈ 0.93–0.97).
- Power: hold until a built-vessel speed–power point exists; report the margin explicitly.
