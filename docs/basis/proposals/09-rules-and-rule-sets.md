# Proposals: Rules and Rule Sets

Not published. Companion to `docs/basis/09-rules-and-rule-sets.md`. Row ids (RS-Ann) refer to the
inventory on that page. Errors become findings for #121 after maintainer triage. "Effect on results" is
an estimate from reading the code, not a run.

## Gaps

What a classification or regulatory specialist would expect that OVCD does not do today.

1. **No check of the side and bottom clearances on the result.** IGC 2.4.1.2 is applied only as sizing
   (RS-A60, RS-A61). Two paths replace the computed gap with no check: `side_clearance_override_m`
   (`crates/ovcd-core/src/barge_capacity.rs`, `gas_volumetric`, "if let Some(over)") and
   `design_grid` (`crates/ovcd-core/src/design_grid.rs`, line 102, 1.0 m). Spec 0004 already says a
   `sizes` Rule is "still checked on the result". Until #271 and #277, nothing does it.
2. **A typed tank wall thickness skips the IGC minimum** (RS-A62). `tanks.rs` uses the typed value as
   is; the floor of IGC 4.23.2.1.2 runs only on the formula path (`pressure_vessel.rs`,
   `calculate_wall_thicknesses_with`). The profile's hard range is only `> 0`.
3. **No check of the allowable stress against IGC 4.23.3.1** (RS-A64). Already planned as a `warn` Rule
   with a finding (spec 0004, #292 follow-up). The defaults coherence test is not built.
4. **Not-evaluated requirements are silent.** The weather criterion (IS Code Part A 2.3) and damage
   survival (IGC 2.7) appear only in `stability.limitations` text. A Design result has no count of
   them. Spec 0004 plans `not evaluated` rows and the count in every passing verdict.
5. **Severity is lost in the outputs.** `RuleChecks.svelte` and the `run.rs` summary show any
   `ok: false` as FAIL. A warning-severity check that fails (hopper GM, thruster count, sightline,
   always-failing `fabrication_rates_are_estimates`) reads like a blocking failure, while `fit` and
   `all_ok` ignore it. Five checks added in `simulate.rs` have no `severity` field at all.
6. **No statement of what was not checked.** A river-only gas result does not say that no statutory
   technical code (freeboard, inland stability, ADN) was checked. ADR 0011 and spec 0004 ask for it.
7. **Inland type G clauses not in the designed `lr-inland-gas` set.** The same section that holds 3.3.7
   and 3.5.3 also holds 3.5.1 (L/D ≤ 7), 3.5.2 (0.80 m side, 0.60 m double bottom, saddle extent),
   3.3.8 (engine ventilation inlets, 2.00 m) and 3.4.1 (funnel outlets, 2.00 m). The set as designed
   takes only three clauses. #215 removed the 3.5.2 floors because their Measure differed; nothing
   replaces them. An expert who selects an inland gas set will expect them.
8. **No length applicability.** IS Code Part A 1.1.1 applies to ships of 24 m in length and over. The
   Applicability form (enumerated inputs only) cannot express it. Spec 0004 puts Applicability Rules out
   of scope; the result should at least say the criteria were applied regardless of length.
9. **Free surface of consumables** (IGC 2.2.3) is not in the GM. Only cargo free surface is.
10. **Glossary terms missing or in conflict.**
    - `GLOSSARY.md` defines **Moulded Depth** twice, with different wording.
    - No term for **Origin** (`regulation`, `class`, `physical`, `tool-limit`, `method-validity`),
      **Consequence** (`block`, `warn`, `note`), **Content hash**, **Set version** versus **Edition**,
      **Selection order**, **Sizing Rule** (`sizes`), **Method status**, **Not evaluated**.
    - "Governing" has two meanings in today's results (lowest-GM loading condition; light air draft)
      and a third in the glossary (Governing Rule). The result keys should stop using the word for the
      first two.
11. **The page labels lack an OVCD Check label.** `docs/basis/README.md` has no label for a check whose
    origin is OVCD (physical, tool limit, method validity). This page had to use "Fixed in code" for
    them, which mixes a constant with a check. Proposal: add **OVCD Check** as a label, matching the
    glossary.

## Errors (code against the cited source)

| # | Code location | Code says | Source says | Effect | Status |
|---|---|---|---|---|---|
| E1 | `crates/ovcd-core/src/gas_separation.rs` line 12 and 96–104, `COFFERDAM_MIN_M`, message "cl. 3.6.13" | 0.60 m cofferdam for gas | Inland rule book Pt 4 Ch 4 Sec 3, 3.6.13 is in 3.6 "Special requirements for Type C tankers" (2015 edition, `LR-IWW-Rules-Nov2015.txt` line 15218; July 2022 mirror same). Type G has 3.5.3 | No geometry moves with defaults (setback 2.00 m governs aft) | Filed: #280 |
| E2 | `gas_separation.rs` lines 89–93 and 115–127, "pressure tank" / "standard", "cl. 3.5.3 / 3.6.3" | 0.20 m if pressure tank, else 0.50 m | 3.5.3(a): 0.20 m; 0.50 m "where the cargo tanks have plane end bulkheads". The pressure-tank wording is 3.6.3 (type C). ADN 2025 9.3.1.11.3(a) as 3.5.3 | None with defaults | Filed: #281 |
| E3 | `gas_stability.rs` `criteria`, areas to `CRITERIA_ANGLES_DEG[1]` = 40 | Areas 0–40° and 30–40° always to 40° | IS Code Part A 2.2.1: to 40° "or the angle of down-flooding φf if this angle is less than 40°" (`MSC.267-85.txt` lines 552–556) | Non-conservative for hulls with openings that immerse before 40° | Filed: #282 |
| E4 | `design_grid.rs` lines 24–25 and 176 | 1.0 m "standard IGC-2G protective-location spacing" | IGC 2.4.1.2 with 2.4.1.1: d(Vc), 0.8 m to 2.0 m; no 1.0 m standard (`MSC.370-93.txt` lines 1141–1185) | Either way by tank size | Filed: #283 |
| E5 | `gas_arrangement.rs` lines 42–47, `MAX_TANK_LENGTH_OVER_OD` doc comment | "no IGC Code paragraph is cited for it"; origin given only as ADN 9.3.1.11.1(b) of a removed set; spec 0004 makes it a `tool-limit` in `ovcd` | Inland rule book 3.5.1 (type G): "ratio of the cargo tank length over the diameter is not to exceed seven" (`LR-IWW-Rules-Nov2015.txt` line 15122; July 2022 mirror same). ADN 2025 9.3.1.11.1(b): pressure tanks over 7 "are prohibited" | None on numbers. The origin label is wrong: it is a class and regulation value, so the Rule would show as an OVCD Check when a cited class Rule exists | New. Proposed finding |
| E6 | `gas_arrangement.rs` `length_over_diameter_check` | ratio = overall length / **insulated** outer diameter | 3.5.1 and 9.3.1.11.1(b) say "diameter" of the tank, not of its insulation. Reading not settled | Non-conservative if the steel diameter is meant: a tank with thick insulation can pass at a steel ratio above 7 | New. Question for reviewers (Q2 on the page) |
| E7 | `gas_arrangement.rs` comment block lines 276–293 (text at 283–284) | "the gas separation (protective location clearances)" | `gas_separation.rs` gives longitudinal end spaces; protective location (`protective_location.rs`) emits no check | None on numbers; misleads a reviewer | New. Comment fix |
| E8 | `electric_energy.rs` lines 291–296 against `resistance.rs` lines 684–698 | `service_below_schijf`: speed ≤ Schijf limit | `service_speed_below_limiting`: speed ≤ 0.95 × Schijf limit. Same physical limit, two margins. Neither factor is cited | Electric path allows 5 % more speed | New. Internal inconsistency, not a source error |
| E9 | Spec 0004 Appendix A and "Migration" `igc-2g` bullet, weld factor | Upper bound 0.95 on `input.weld_factor` | IGC 4.23.2.1.3: 0.95 with the stated NDT, "may be increased up to 1" with other considerations; not more than 0.85 for partial NDT on process vessels (`MSC.370-93.txt` lines 3327–3340) | A `block` bound at 0.95 would refuse a value the text allows | New. Spec reading to confirm before #271 |
| E10 | `crates/ovcd-core/src/barge_capacity.rs` lines 1004–1012, `gm_positive` | Name says GM positive; test is GM ≥ 0.15 m | The 0.15 m is IS Code Part A 2.2.4's GM0 value, uncited, applied to a first-order hopper GM | None on numbers; the name and the missing citation mislead | New. Naming |
| E11 | `simulate.rs` `air_draft_check` against `barge_capacity.rs` lines 970–995 | Two air draft checks; the `simulate.rs` one ignores ballast | `air_draft_within_limit` uses the ballasted light condition when ballast is set | The two can disagree on one Design | New. Duplicate |

Note on E5: the same paragraph family (3.6.11 and 3.7.3) also exempts tanks with L/D ≤ 7 from a
tank-length cap for types C and N. Those are not gas clauses and need no Rule here.

## Changes

Each change says its effect on results (none, small, large). Order of work follows the ticket chain
of spec 0004.

| Order | Change | Ticket | Effect on results |
|---|---|---|---|
| 1 | Record E5 to E11 as findings or comment fixes, after maintainer triage. E7 is a comment fix with no result change | #121 | None |
| 2 | Settle the weld factor reading (E9) before encoding it: either `le 1.0` block with 0.95 as the default, or 0.95 as a `note`. Settle the L/D diameter datum (E6) | #271, #272 | None (data choice before the first release) |
| 3 | In `lr-inland-gas` 1.0.0, take the L/D limit as `lr-inland/3.5.1/tank-l-over-d` (origin `class`), with `ovcd/gas-arrangement/tank-l-over-d` only if a tool limit beyond it is wanted. Keep the code's measure (insulated diameter) and link E6 | #272 | None (same value, same comparison) |
| 4 | Add `warn` Rules (spec 0004 rule for new rows) for the result checks of the sizing values: side and bottom clearance on the built design, IGC minimum thickness on a typed wall, allowable stress against Rm/A and Re/B | #271 | None to `ok`; new warning rows on override paths and typed values |
| 5 | Show `severity` (later the `warn` outcome) in the app and the CLI summary; give the five `simulate.rs` checks a severity | #276, or a small fix before it | None to numbers; checks text and display change; results hash may move through `checks[]` |
| 6 | Add `not evaluated` rows for IS Code Part A 2.3 and IGC 2.7, and the count in every passing verdict | #269, #270, #271 | None to numbers (trace is outside the hash) |
| 7 | Add the scope wording: "no statutory technical code checked, only the named Rule Sets" for river-only | #275 | None to numbers |
| 8 | Remove the duplicate air draft check (E11), keeping the ballast-aware one | #272 (as a separate design change) | Small: one check row less; a ballasted Design may change verdict |
| 9 | Rename `gm_positive`, cite 2.2.4 as origin of the value, keep it `warn` in `ovcd` | #272 | None to numbers; check name changes |
| 10 | Make the scope notes (RS-A38, -A39, -A43, -A46) `note` Rules or move them to `warnings` | #272 | None to numbers; list changes |
| 11 | Align E8: one Schijf margin, cited or labelled as an OVCD choice | Own design change | Small, electric path only |
| 12 | Plan a later `lr-inland-gas` version (or its IGC chapter 3 replacement) with 3.3.8, 3.4.1 and the 3.5.2 hull clauses, each with a Measure that measures what the text measures | After #280 decision | Large for inland gas Designs if `block` |

## What the docs for experts need from the rules engine

The page above had to read Rust to answer simple questions. The rules engine should expose these, so
these docs can be generated, not hand-written:

1. **A per-rule page generated from the rule data.** For each Rule: id, title, origin, Rule Sets and
   versions that hold it, edition, locator and page, access, the Criterion in plain notation, the
   Measure with its method citation and method status, Applicability, consequence, `sizes`,
   `evaluated` with gap, linked finding, `codified` and `verified` records, and the sourced tests and
   harness cases that cover it (with counts of passing and failing cases). `build.py` could render
   `ovcd rules show <id> --json` into the docs site.
2. **A per-set page**: edition with every amendment checked and whether it changes any Rule, sources
   with SHA-256, content hash, status, includes, known conflicts, and the count of unverified Blocking
   Rules.
3. **A Measure registry page** (`ovcd rules --measures`): unit, object type, arguments, method citation,
   method status, and the code location. This replaces the "How it works" walk through the code.
4. **A machine list of OVCD Checks** with origin, so the inventory on this page is produced, not
   counted by hand. Today a script cannot do it: names are free text and some checks are made in
   `ovcd-format` (`simulate.rs`), outside the core.
5. **A "where applied" field**: whether a Rule gates `fit`, shows in a Design result, or exists only in
   a module call. Today that depends on which function builds the list.
6. **Finding links both ways**: each finding issue should list its Rule ids, and the per-rule page
   should show the finding's decision (fix, known limitation).
7. **A coverage page from the harness** keyed by rule id (#278), so the Evidence section of each basis
   page is generated.
