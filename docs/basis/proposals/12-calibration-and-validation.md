# Proposals: 12 Calibration and validation

Not published. Companion to `docs/basis/12-calibration-and-validation.md`. Facts are on the page; this file holds
gaps, possible errors and proposed changes. Each error needs maintainer triage before it becomes a finding in the
register (#121).

## Gaps in evidence, by area

| # | Area | Gap | Why it matters |
|---|---|---|---|
| 01 | Hull form and hydrostatics | No case with a shaped hull or a published hydrostatic table | Every calibration and harness case is a box; the sections-table hull (ADR 0012) has no external check |
| 02 | Weights and centres | No self-propelled lightship; no published KG or LCG; pontoon lightship 10 of 22 outside band | Lightship and KG drive deadweight and stability; only box barges are checked |
| 03 | Intact stability | Harness is box-only, free surface always 0, no `published` verdict, no weather criterion; the gas `assess` path (KG from the arrangement) is untested by the harness | The criterion maths is checked, the inputs a real design feeds it are not |
| 04 | Type C tanks | 2 runnable cases, one a design study; structure factor from 1 tank | Tank steel feeds lightship and cost of every gas design |
| 05 | Gas cargo arrangement | No published gas barge arrangement runs; no harness case for IGC 2.4 | Protective location is a blocking check with no known-verdict case |
| 06 | Liquid, dry and container cargo | 41 tank-barge records cannot run; container k from 1 case | Tank barges are a common inland type with good published data already in the repo |
| 07 | Resistance and powering | No built-vessel trial point; pusher records unused | Power is unchecked against any built vessel (calibration README, pass 1 finding 4) |
| 08 | Waterway and route | No measured squat or journey data compared | Draught limits and journey derating rest on formulas checked only by hand calculation |
| 09 | Rules and Rule Sets | No Verified Rule; no Rule Trace; no rule-data mutation | A reviewer cannot see which harness case tests which Rule |
| 10 | Cost | `cost` outside design impact, results hash and web parity; no published cost case | A change to cost logic or rates moves budgets without any record |
| 11 | Drawings | Checks agreement between two implementations, not engineering content | A wrong arrangement drawn identically by both passes |
| 12 | This area | No freshness check on `calibration/results.md`; no written criteria for `confidence`; no V&V report | The page's numbers can go stale; band width depends on unstated judgement |

## Errors (code or data against its cited basis)

1. **Web sources have no checksum.** ADR 0001 says git keeps for each source the URL, retrieval date, checksum,
   licence and particulars. The 107 entries under `web_sources` in `calibration/sources.yaml` have `id`, `title`,
   `url`, `accessed` and (104 of them) `note`. None has a checksum, a store key or a licence. Either store them
   per ADR 0001 or amend the ADR for web records.
2. **Out-of-date statement in `calibration/README.md`.** "Corrections applied" says the barge database is
   unreachable. Sources `K1` to `K39` cite Wayback captures of it, and their notes say so. Update the README.
3. **Cb default against the fit (to check).** `crates/ovcd-format/src/builtin_profile.json` gives
   `block_coefficient` default 0.85, with a basis that quotes the calibrated range 0.883 to 0.995. The fit is 0.928.
   Pass 1 finding 2 says 0.85 gives box-barge DWT 19 to 33 % low. Check which Cb a box-barge Design gets when
   the user leaves the input out. If it is 0.85, a default sits below every calibrated value.
4. **Tank structure factor does not reproduce its own case (to check).** The profile basis for 1.12 uses
   hemispherical heads at the published thicknesses. The calibration run of the same tank uses `ellipsoidal_2to1`
   heads and the cylinder thickness, and gives +5.2 % steel on an in-sample figure. One of the two head
   assumptions is wrong for this tank, or the factor needs a note.
5. **In-sample lightship rests partly on forum statements.** One of the three box-k points
   (`us-hopper-195x35-generic`, confidence low) cites practitioner answers found by search. A fitted default in the
   Defaults Profile should not rest on an unsourced point. Drop it from the fit or replace it.
6. **Glossary.** `GLOSSARY.md` defines **Moulded Depth** twice, with different wording.

## Glossary terms missing

Calibration Case (ADR 0010 uses it), held-out case, in-sample, evaluation harness, boundary mutant, false pass,
false fail, oracle, design-impact check, results hash, grid error.

## Changes

Effect on results: **none** (evidence only), **small**, or **large** (moves designs).

| Order | Change | Effect | Notes |
|---|---|---|---|
| 1 | Add a freshness check for `calibration/results.md` and `factors.yaml` in CI, as for `evaluation/results.md` | None | Small test; stops this page citing stale numbers |
| 2 | Write criteria for case `confidence` (high, medium, low) in `calibration/README.md` | None | Bands depend on it |
| 3 | Fix errors 1, 2 and 6; triage errors 3, 4 and 5 into the findings register | None to small | Error 3 may be large if confirmed |
| 4 | Add a label for project acceptance thresholds (bands, tolerances, gate rules) to `docs/basis/README.md`, or agree to use **Fixed in code** for them | None | None of the six labels fits a maintainer threshold well |
| 5 | New calibration family: double-skin tank barges. Add a tank-hull run path to the calibration runner for the 41 records already in `cases.yaml` | None to engine; new evidence | Largest data set waiting; tests area 06 and 02 |
| 6 | Pontoon factor: fit a pontoon k from a split of the pontoon cases, keep the rest held out | Small | Today pontoons run as box and come out 27 % light |
| 7 | Expert-reviewed harness cases: add stability booklets or textbook examples that print an IS Code 2.2 verdict (`verdict_basis: published`), with a reviewer named in the page history | None | Gives the first verdicts not derived by the oracle |
| 8 | Harness: add free surface and a non-box section to the oracle and the corpus | None | Closes the two widest gaps in area 03 |
| 9 | Harness phase 1: run whole `.vdesign` files so the KG comes from the arrangement | None | Tests the path real designs use |
| 10 | New calibration family: built pushers and motor vessels with trial speed and power; run the 10 pusher records once a tug model exists | None; may lead to large | Separates margin, shallow water and model error (pass 1 finding 4) |
| 11 | New calibration family: Type C tanks with published steel weights, insulated and refrigerated | None; may move 1.30 factor (large for gas) | Today the insulated factor has no source |
| 12 | Put `cost` in the design-impact comparison as a separate, reported section (not the results hash) | None | A cost move then shows in the pull request |
| 13 | Rule-data mutation and rule-id keys in the harness (spec 0004, T5.6) | None | Needs the rules engine first |
| 14 | Label the 21 `RUST_UNLABELLED` test files | None | Lets a reviewer find the sourced numbers |
| 15 | Decide when a band blocks a release (ADR 0010 says "wait for more cases"); write the case count that triggers it | None | Turns the warning into a gate at a stated point |
| 16 | A verification and validation report per release (ADR 0011), built from `calibration/results.md`, `evaluation/results.md` and the coverage table on this page | None | ADR 0011 says reviewers rely on it; it does not exist |
