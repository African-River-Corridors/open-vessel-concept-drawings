# Rules and Rule Sets

> Concept and preliminary design only. Not for approval. Draft: needs expert review.
>
> Code state: written against `main` at commit `d60e5ec` (2026-10-21). This page replaces the first draft, which
> was written before the rules engine existed.

**For:** Classification and regulatory specialist  ·  **Code:** `crates/arc-rules/` (the kernel: `model.rs`,
`load.rs`, `eval.rs`), `crates/ovcd-format/src/trace.rs` (the Measures, the default stack, the Rule Trace),
`run.rs`, `crates/ovcd-core/src/checks.rs`; **Data:** `crates/ovcd-format/rules/`  ·  **Rule Sets:** `is2008-a`,
`igc-2g`, `lr-inland-gas`, `ovcd` (all `draft`).

This page says how OVCD interprets and applies rules, then lists every Rule in the built-in Rule Sets with its
status. A reviewer can answer three questions here: which published rules does OVCD apply, which of them has a
second reader checked against the text, and which requirements does OVCD say that it does not evaluate.

## What OVCD computes here

- **Four built-in Rule Sets**, as data (JSON, one file per set version). `ovcd rules` lists them with version,
  status, number of Rules and content hash. The highest built-in version of each set is the one a run uses.

| Set | Highest version | Rules | Source |
|---|---|---|---|
| `is2008-a` | 1.0.0 | 8 | IS Code 2008, Part A 2.2 and 2.3 (MSC.267(85)) |
| `igc-2g` | 1.1.0 | 10 | IGC Code, ship type 2G: 2.4 tank location, 2.7, 4.23 Type C tank walls, 6.4 (MSC.370(93) as amended) |
| `lr-inland-gas` | 1.1.0 | 6 | Inland class rules (Lloyd's Register), Pt 4 Ch 4 Sec 3, gas tank clauses |
| `ovcd` | 2.1.0 | 39 | OVCD's own checks: physical feasibility, tool limits, method validity |

  Earlier versions are kept unchanged: `is2008-a` 1.0.0, `igc-2g` 1.0.0, `lr-inland-gas` 1.0.0 and 1.1.0, and
  `ovcd` 1.0.0 are in `rules/RELEASED.json` with their content hashes. `igc-2g` 1.1.0, `ovcd` 2.0.0 and `ovcd`
  2.1.0 are not released yet (`ovcd rules --provenance` is the gate).
- **A Rule Trace in every result** (`rule_trace`): one row per Rule per object, for every selected Rule Set. It is
  outside the results hash. The app shows it in the **Rule checks** tab (`RuleChecks.svelte`); `ovcd run --json`
  gives it.
- **A verdict.** The Option passes when no row fails. The verdict names every blocking Rule that the engine did
  not evaluate, so a pass never hides a gap.
- **The `checks` list** of the result still exists. Each check that a Rule reports is mapped to its Rule id
  (`CHECK_RULES` in `trace.rs`).
- **`fit` for gas Designs** keeps a tank arrangement only when it meets every blocking Rule (#277).

Not built: the **Service Area** input and the `rule_sets` input of format version 3 (#274, #275). A user selects
Rule Sets only with `--rule-set <file>`.

## How it works today

### 1. The terms (glossary)

These terms come from `GLOSSARY.md` and spec 0004. They are in the code (`crates/arc-rules/src/model.rs`).

- **Rule.** One requirement about one Measure: its Criterion, its Applicability, its consequence and its source.
  The **origin** of a Rule is one of `regulation`, `class`, `physical`, `tool-limit`, `method-validity`,
  `guideline` or `national-practice`. An **OVCD Check** is a Rule whose origin is OVCD itself (`physical`,
  `tool-limit` or `method-validity`). It is never shown as a published rule.
- **Kind.** A Rule is a `limit` (a constant), a `table` (a row of one of the set's tables), a `formula` (a shape
  that Rust codes, with its coefficients as data), a `check` (a yes/no Measure with no comparison) or a `note`
  (a cited value shown in the trace with no test).
- **Measure.** A quantity the engine computes for one object (an Option, the hull, a tank, a loading condition).
  A Measure is Rust code with its own tests. It cites a **method**, not a rule. `ovcd rules --measures` prints the
  registry, with the method status of each. A design input that a rule bounds is an *input Measure*
  (`input.<name>`).
- **Criterion.** The test part of a Rule: a comparison (`ge`, `gt`, `le`, `lt`), a limit and a tolerance (default
  0). `ge` holds when value ≥ limit − tolerance, and so on (`holds` in `eval.rs`).
- **Margin.** value − limit for a lower bound, limit − value for an upper bound. A positive Margin passes.
- **Requirement.** The Rules, from all selected Rule Sets, on the same Measure instance, the same object and the
  same direction.
- **Rule Set.** A named, versioned group of Rules from one source edition, with two versions: the source
  **edition** and the **set version** (OVCD's encoding, semver). Its `status` is `draft` or `stable`. All four
  built-in sets are `draft`.
- **Applicability.** A list of conditions `input ∈ {values}` on enumerated inputs (Cargo, Cargo Containment, tank
  material, derived inputs such as `service_area`). There is no expression language. A condition on a number (for
  example "24 m in length and over") cannot be written as Applicability.
- **Consequence.** `block` (a failure stops a pass), `warn` (a failure is shown, and does not stop a pass) or
  `note`.

### 2. How a rule text becomes a Rule (codification)

The process is in `docs/rules/codification.md`. A **codifier** (a person or an agent) gets the official text, fixes
the edition and puts the file in the rule-source library with its SHA-256. The codifier splits the paragraph into
one Rule per requirement, picks the Measure, records the comparison from the words ("not less than" is `ge`), the
limit as printed, the Applicability and the citation, and adds tests at both sides of the limit. The Rule gets a
`codified` record (role, date, review link).

A **second reader**, who did not codify the Rule, opens the cited edition at the cited page and works through a
checklist. Only then is `verified` filled, with the role (never a person's name), the date, the edition held, the
locator read and the review link. Spec 0004 calls a Rule **sourced** only when its origin is `regulation` or
`class` **and** `verified` is filled.

**In code today:** every Rule of the four sets has a `codified` record. Fifteen Rules have a `verified` record,
and every one of them is a published rule (`is2008-a`: 7 of 8, `igc-2g`: 7 of 10, `lr-inland-gas`: 1 of 6). None
of the 39 `ovcd` Rules is verified: an OVCD Check has no text to check against. The codifiers and second readers
recorded so far are all agents (`by: agent`). No Rule has a maintainer or expert-reviewer record yet.

### 3. Rule ids and citations

A rule id is `<family>/<paragraph>/<slug>`, for example `is2008/A.2.2.1/area-0-30`. For a `regulation` or `class`
Rule the paragraph segment is the source locator with spaces removed, and the loader refuses a Rule whose two
differ. For an OVCD Check the paragraph segment is a topic (`ovcd/gas-arrangement/...`). An id keeps its Measure
and direction in every later version, and an id is never used again once dropped (`ovcd rules --provenance`
checks both). A formula that a requirement uses but that is not itself a requirement is a table of the set, not a
Rule. Example: d(Vc) of IGC 2.4.1.1 is the table `igc-d-vc`; the requirement that uses it is
`igc/2.4.1.2/side-clearance`.

Every `regulation` or `class` Rule cites a paragraph (`source.locator`) and a page of the held copy. The loader
refuses one that does not.

### 4. Editions and amendments

A Rule Set records its edition: the base document, each amendment read, and the in-force date.

- `igc-2g`: IGC Code, MSC.370(93), as adopted 22 May 2014, in force 1 January 2016. MSC.411(97), MSC.441(99),
  MSC.476(102), MSC.492(104), MSC.523(106) and MSC.566(109) were read. None changes 2.3, 2.4 or 4.23.
  MSC.492(104) replaces 2.7.1.1.
- `is2008-a`: 2008 IS Code, MSC.267(85), as adopted 4 December 2008. MSC.443(99) and MSC.444(99) were read; neither
  changes Part A 2.2 or 2.3. MSC.413(97) and MSC.414(97) are not held.
- `lr-inland-gas`: Lloyd's Register inland rules, Pt 4 Ch 4 Sec 3, "July 2022, as the code cites it". The
  November 2015 edition was read (same clauses), and an unofficial copy of the July 2022 section.

Every released set version is kept unchanged. A change of a limit is a new version. A verification that changes no
result-changing field is a patch version with the same content hash. The Rule Trace names the set versions a run
used (`rule_sets`, for example `igc-2g@1.1.0`) and its `service_area`. A Design file does not store the set
versions yet: that is format version 3 (#274).

The `ruleset` input remains in the Design file with one accepted value, `IGC_2G`. A file that names `IGC_2G_EXACT`
or `ADN_G` is read as `IGC_2G`, with a warning (`migrate_ruleset` in `vdesign.rs`).

### 5. The strictest rule governs

All Rules in a Requirement apply. The Option meets the Requirement only if it meets every one. Of two Criteria in
one direction, the stricter lets fewer values pass (a larger effective bound for a lower bound, a smaller one for an
upper bound; at an equal bound, a strict comparison is stricter). The order is total, so the strictest Rule always
exists. It is marked **governing** in the trace (`mark_governing` in `eval.rs`), chosen from the Criteria and not
from the design value. Ties go to the first set in selection order, then to the rule id. There is no override
field: no Rule can loosen another (ADR 0013).

A Lean model proves properties of this composition (ADR 0014, `proofs/lean/`), including that the sized value
meets its Requirement and never falls when Rules are added (#482). A differential test compares the Rust
evaluator with the Lean oracle (`crates/arc-rules/tests/lean_oracle.rs`). It runs in the weekly Lean job, when
`ARC_RULES_LEAN_ORACLE` names the built oracle.

Do not confuse this with two older uses of the word "governing" in a result: the stability loading condition with
the lowest fluid GM (`stability.governing`), and the light air draft ("Air draft (empty, governing)"). Both name
a result, not a Rule. Renaming them moves results hashes and is not done (#430).

### 6. Which Rule Sets a Design is checked against

The default stack depends on the Cargo Containment (`default_stack` in `trace.rs`):

| Cargo Containment | Stack, in selection order |
|---|---|
| Type C tanks (gas) | `igc-2g`, `is2008-a`, `lr-inland-gas`, `ovcd` |
| Box hold, sloped hopper hold | `is2008-a`, `ovcd` |

`igc-2g` includes `is2008-a` 1.0.0; a set that is selected twice is checked once. A set loaded with `--rule-set`
is added after the stack, and a loaded set with the id of a stack set replaces it for the run. Every Option is
`river-only` until format version 3 stores a Service Area (#274, #275), and the first release gives both Service
Areas the same stack. A result does not yet say that no statutory technical code was checked, beyond naming the
Rule Sets it used.

### 7. Pass, fail, warn and "not evaluated"

One row of the trace has one outcome (`Outcome` in `eval.rs`):

| Condition | Outcome |
|---|---|
| The Rule's set does not apply | one `not applicable` row for the set |
| The Rule does not apply, or the design has no object of its type | `not applicable` |
| The Rule is marked `evaluated: false` (the engine lacks the Measure) | `not evaluated` |
| The design has no objects of the type because the engine did not compute them (no loading conditions, say) | `not evaluated`, with the reason as the gap text |
| The Measure could not be computed, or gave NaN or infinity | `fail` (block) or `warn` (warn), never `pass` |
| The Criterion holds | `pass` |
| The Criterion does not hold | `fail` (block) or `warn` (warn) |
| A `note` Rule with a value | `note` |

An Option passes when no row is `fail`. A `not evaluated` row does not stop a pass, and the verdict names every
blocking Rule not evaluated (a maintainer decision, 2026-10-20).

What a reader sees:

- The `checks` list of a result carries a severity on every check (#419). Only the exact text `"error"` gates
  (`all_ok` in `checks.rs`); any other text acts as a warning.
- The CLI summary lists a failed warning apart from a failed error ("Warning checks: ... not met"), and the Rule
  checks tab shows warnings as their own outcome (#418, #419).
- When the stability assessment cannot run, `fit` adds one failed check, "Stability — not assessed"
  (`ovcd/stability/assessed`): a candidate the engine cannot judge never passes.

**Seen in a run** (the fixtures of `crates/ovcd-format/tests/fixtures/` at the commit above, run with
`ovcd run --json`):

| Fixture | Rule Sets | Outcomes | Blocking Rules not evaluated |
|---|---|---|---|
| `dry_bulk_box_hold` | `is2008-a`, `ovcd` | 5 pass, 2 warn, 8 not evaluated, 32 not applicable | 7: every blocking IS Code row |
| `dry_bulk_sloped_hopper` | `is2008-a`, `ovcd` | 6 pass, 1 warn, 8 not evaluated, 32 not applicable | 7: every blocking IS Code row |
| `gas_igc_2g_exact` | all four | 48 pass, 3 note, 2 warn, 4 not evaluated, 28 not applicable | 4 (see below) |
| `gas_igc_2g_self_propelled` | all four | 54 pass, 3 note, 2 warn, 4 not evaluated, 22 not applicable | 4 (see below) |

So a dry bulk Design is checked against none of the IS Code criteria: the engine computes no loading conditions
with GZ curves for a dry bulk hull, so the Rules have no objects to test and the verdict says so. For the two gas Designs the four blocking Rules not evaluated are the damage
survival Rule (`igc/2.7/damage-survival`), the weather criterion (`is2008/A.2.3/weather`), the tank capacity
table (`lr-inland/3.2.1/tank-capacity`) and the cargo-area hull design (`lr-inland/3.5.2/hull-design`).

### 8. What the Rule Trace shows

One row per Rule per object, in selection order: the rule id, the Rule Sets that hold it, the citation, the origin,
the rule status (`verified` or `unverified`) and the method status of the Measure (both shown), the object, the
value, the limit, the comparison, the tolerance, the Margin, the inputs used, the consequence, the outcome, the
governing flag, the `sizes` flag, the gap text and the linked finding (`TraceRow` in `eval.rs`). Of the blocking
rows of a Requirement (all rows when none blocks), the strictest is the governing one. The app tab sorts
failures first, then warnings, passes, notes, not evaluated and not applicable. The trace is outside the results
hash, so a rule change is reported as a rule change, not as "results differ" (ADR 0007).

### 9. Rules that size the design, not only check it

Some rule values build the design: the search sizes the arrangement with them, so the result meets them by
construction. A Rule with `sizes: true` is both. The sized value is the limit of the governing Rule, and the same
Rule is evaluated on the result. Five Rules size today: `igc/2.4.1.2/side-clearance`,
`igc/2.4.1.2/bottom-clearance`, `lr-inland/3.6.13/cofferdam`, `lr-inland/3.3.7/opening-setback` and
`lr-inland/3.5.3/end-space`.

## Assumptions, rules and estimates

This table is the **inventory of the built-in Rules**, from the data files at the commit above. Labels follow
`docs/basis/README.md`: a verified published Rule is **Rule (verified)**, an unverified one **Rule (unverified)**.
An OVCD Check is not a published rule: its limit is **Fixed in code** or an **Estimate** as the page of its area says.

"Status" is `verified` (a second reader checked the text), `unverified` (codified, no second reader), or
`not evaluated` (the engine lacks the Measure: the Rule shows as `not evaluated` in every verdict). "If it fails"
is the Rule's consequence: `blocks` stops a pass, `warns` does not. A Finding is a ticket that records a known
difference between the Rule and its text, or a known limit of its Measure.

#### IS Code 2008, Part A (`is2008-a` 1.0.0)

| Rule id | What it tests | Origin | If it fails | Status | Finding |
|---|---|---|---|---|---|
| `is2008/A.2.2.1/area-0-30` | Area under the GZ curve from 0 to 30 degrees, at least 0.055 m.rad | regulation | blocks | verified | – |
| `is2008/A.2.2.1/area-0-40` | Area under the GZ curve from 0 to 40 degrees, at least 0.09 m.rad | regulation | blocks | verified | #282 |
| `is2008/A.2.2.1/area-30-40` | Area under the GZ curve from 30 to 40 degrees, at least 0.03 m.rad | regulation | blocks | verified | #282 |
| `is2008/A.2.2.2/gz-30` | Righting lever at least 0.2 m at a heel of 30 degrees or more | regulation | blocks | verified | – |
| `is2008/A.2.2.3/angle-max-gz` | Maximum righting lever at a heel of at least 25 degrees | regulation | blocks | verified | – |
| `is2008/A.2.2.4/gm0` | Initial metacentric height at least 0.15 m | regulation | blocks | verified | – |
| `is2008/A.2.2/condition` | Each loading condition's stability check; false also if it does not float | regulation | note | unverified | – |
| `is2008/A.2.3/weather` | Severe wind and rolling criterion (weather criterion) | regulation | blocks | not evaluated | – |

#### IGC Code, ship type 2G (`igc-2g` 1.1.0)

| Rule id | What it tests | Origin | If it fails | Status | Finding |
|---|---|---|---|---|---|
| `igc/2.4.1.2/side-clearance` | Cargo tank side distance from the outer shell, at least d(Vc) less the insulation | regulation | blocks | verified (sizes) | – |
| `igc/2.4.1.2/bottom-clearance` | Cargo tank bottom distance: the greater of B/15 (at most 2 m) and d(Vc), less the insulation | regulation | blocks | verified (sizes) | – |
| `igc/4.23.2.1.2/min-thickness` | Minimum shell and head thickness after forming, by steel class | regulation | blocks | verified | – |
| `igc/4.23.2.1.3/weld-factor` | Welded joint efficiency factor 0.95 (up to 1 on other considerations) | regulation | warns | verified | – |
| `igc/4.23.2.1.3/weld-factor-max` | Welded joint efficiency factor never above 1 | regulation | blocks | verified | – |
| `igc/4.23.3.1/allowable-stress` | Allowable stress at most the lesser of Rm/A and Re/B; warn: legacy 285 MPa files | regulation | warns | verified | #313 |
| `igc/2.7/damage-survival` | Survival after the assumed damage | regulation | blocks | not evaluated | – |
| `igc/4.23.1.2/min-design-pressure` | Design vapour pressure not less than Po = 0.2 + A C (rho_r)^1.5 MPa | regulation | warns | unverified | #317 |
| `igc/6.4.1.1/steel-design-temperature` | The tank steel is one Table 6.3 permits down to -165 C (9 % Ni, austenitic): the design temperature is not lower | regulation | warns | unverified | #317 |
| `igc/6.4.1.1/steel-by-approval` | A high-strength carbon-manganese steel (yield above 410 N/mm2) is by approval of the Administration, not by Table 6.1 to 6.3 | regulation | warns | not evaluated | #317 |

#### Inland class rules, gas tank clauses (`lr-inland-gas` 1.1.0)

| Rule id | What it tests | Origin | If it fails | Status | Finding |
|---|---|---|---|---|---|
| `lr-inland/3.6.13/cofferdam` | Cofferdam at the aft end of the cargo area at least 0.60 m wide | class | blocks | unverified (sizes) | #280 |
| `lr-inland/3.3.7/opening-setback` | Engine room and accommodation entrances at least 2.00 m from the cargo area | class | blocks | unverified (sizes) | #381 |
| `lr-inland/3.5.3/end-space` | Space between the tanks and the hold end bulkhead: 0.20 m, or 0.50 m for plane ends | class | blocks | verified (sizes) | – |
| `lr-inland/3.5.1/tank-l-over-d` | Cargo tank length at most seven diameters (Type G) | class | blocks | unverified | – |
| `lr-inland/3.2.1/tank-capacity` | Capacity of each cargo tank within Table 4.3.1 | class | blocks | not evaluated | – |
| `lr-inland/3.5.2/hull-design` | Hull in the cargo area: one of the three designs the clause allows (Type G) | class | blocks | not evaluated | – |

#### OVCD Checks (`ovcd` 2.1.0)

| Rule id | What it tests | Origin | If it fails | Status | Finding |
|---|---|---|---|---|---|
| `ovcd/capacity/cargo-positive` | The hull carries some cargo | physical | blocks | unverified | – |
| `ovcd/capacity/feasible-geometry` | The cargo space has a positive volume | physical | blocks | unverified | – |
| `ovcd/capacity/cargo-zone-positive` | The cargo zone is longer than zero | physical | blocks | unverified | – |
| `ovcd/capacity/air-draft-within-limit` | Light air draft (with ballast where carried) within the typed limit | physical | blocks | unverified | – |
| `ovcd/capacity/intact-gm` | First-order hold GM at the reported cargo at least 0.15 m (an estimate, not a stability check) | tool-limit | warns | unverified | – |
| `ovcd/gas-arrangement/feasible-arrangement` | Some tank arrangement passes every check on this hull | physical | blocks | unverified | – |
| `ovcd/gas-arrangement/tank-l-over-d` | Tank length at most 7 inner shell diameters | tool-limit | blocks | unverified | – |
| `ovcd/aft-block/thrusters-fit-beam` | The thrusters fit within the beam | physical | blocks | unverified | – |
| `ovcd/aft-block/thruster-count` | The requested number of thrusters fits | physical | warns | unverified | – |
| `ovcd/aft-block/containers-fit-beam` | The 40 ft units fit within the beam | physical | blocks | unverified | – |
| `ovcd/aft-block/units-in-one-row` | The 40 ft units fit in one row | tool-limit | warns | unverified | – |
| `ovcd/aft-block/wheelhouse-fits-beam` | The wheelhouse fits within the beam | physical | blocks | unverified | – |
| `ovcd/aft-block/accom-beside-wheelhouse` | The accommodation units fit beside the wheelhouse | physical | warns | unverified | – |
| `ovcd/aft-block/gensets-fit-beam` | The gas gensets fit within the beam | physical | blocks | unverified | – |
| `ovcd/aft-block/gensets-under-deck` | The gas gensets fit under the deck with 1.0 m allowance (an estimate) | physical | warns | unverified | – |
| `ovcd/aft-block/gensets-reserve` | The gensets carry the load with one set in reserve | physical | blocks | unverified | – |
| `ovcd/aft-block/wheelhouse-sightline` | The wheelhouse eye sees over the cargo with the margin | physical | blocks | unverified | – |
| `ovcd/squat/froude-depth` | Depth Froude number below the squat method's range limit 0.7 | method-validity | warns | unverified | – |
| `ovcd/squat/blockage` | Blockage at most 0.35, the squat method's range | method-validity | warns | unverified | – |
| `ovcd/squat/keel-clears-bed` | The keel clears the bed at speed | physical | blocks | unverified | – |
| `ovcd/squat/keel-margin` | The keel clears the bed at speed with the safety margin | physical | warns | unverified | – |
| `ovcd/squat/methods-agree` | The two squat methods agree within 50 % | method-validity | warns | unverified | – |
| `ovcd/waves/short-wave-validity` | Wave length within the added-resistance method's range | method-validity | warns | unverified | – |
| `ovcd/waves/wave-height` | Significant wave height at most 3.0 m, the screening range | method-validity | warns | unverified | – |
| `ovcd/energy/service-below-schijf` | Service speed within the Schijf limiting speed | method-validity | blocks | unverified | – |
| `ovcd/capacity/not-overloaded` | The full tank volume floats within the hull (information: it does not gate, #418) | physical | warns | unverified | – |
| `ovcd/capacity/freeboard` | Freeboard at the full-volume draught at least the required freeboard (information: it does not gate, #418) | physical | warns | unverified | – |
| `ovcd/journey/volume-bound` | The cargo is bound by the tank volume, not overloaded | physical | blocks | unverified | – |
| `ovcd/journey/floats` | The payload the draught limit leaves is above zero | physical | blocks | unverified | – |
| `ovcd/journey/ukc-consistent` | The draught limit leaves room for the under-keel clearance | physical | blocks | unverified | – |
| `ovcd/journey/utilisation` | The payload uses a reasonable share of the tank volume | tool-limit | warns | unverified | – |
| `ovcd/gas-arrangement/freeboard-at-payload` | Freeboard at the carried draught at least the required freeboard | physical | blocks | unverified | – |
| `ovcd/stability/assessed` | The engine could assess the stability of every loading condition | tool-limit | blocks | unverified | – |
| `ovcd/waves/bow-length-supplied` | The bow length the wave method needs is given, not estimated | method-validity | warns | unverified | – |
| `ovcd/resistance/service-below-limiting` | Service speed at most 0.95 times the Schijf limiting speed, in each channel | tool-limit | blocks | unverified | – |
| `ovcd/resistance/blockage-in-range` | Channel blockage ratio between 0 and 1, in each channel | physical | blocks | unverified | – |
| `ovcd/resistance/service-clears-squat` | At the service speed the squat leaves the under-keel clearance, in each channel | physical | warns | unverified | – |
| `ovcd/resistance/lackenby-fit` | Depth Froude number below 0.95, where the shallow-water correction is used | method-validity | warns | unverified | – |
| `ovcd/resistance/first-order-coefficients` | The resistance coefficients are uncalibrated first-order estimates | method-validity | warns | unverified | – |

### What the Rule Sets leave out

Checks that the engine still emits but that are not Rules of a Design:

- The three checks of the capital budget (`masses_positive`, `no_reliquefaction_in_scope` and
  `fabrication_rates_are_estimates`) are in the `cost` object, not in the Rule Trace. The last two are warnings
  that state a scope: `no_reliquefaction_in_scope` is always `ok`, and `fabrication_rates_are_estimates` is always
  not `ok` (`barge_cost.rs`).
- The checks of the Vessel Class catalogue (`vessel_standards.rs`: unique ids, ordered ranges, positive
  dimensions, convoy beam, source cited) test data, not a Design. They are reached through the module API only.
  See #432 on the convoy beam check.

## Inputs the user can change

| Input | Default | Unit | Where the default comes from |
|---|---|---|---|
| `ruleset` | `IGC_2G` (the only value) | – | Defaults Profile; basis text cites IGC 2.4.1.1 and 2.4.2 |
| `min_freeboard_m` | 0.3 | m | Defaults Profile; no rule source (#428) |
| `air_draft_limit_m` | none | m | Typed by the user (waterway) |
| `draught_limit_m`, channel depth and UKC | none | m | Typed by the user (waterway) |
| `squat_safety_margin_m` | 0.15 | m | Carried over from the original engine |
| `fill_ratio` | 0.98 | – | IGC 15.3 (sourced) |
| `tank_wall_thickness_mm` | none (formula) | mm | The wall of the result is checked against `igc/4.23.2.1.2/min-thickness`, typed or not |
| `tank_allowable_stress_mpa` | none (material table) | MPa | The stress of the result is checked against `igc/4.23.3.1/allowable-stress` (a warning) |

A user cannot yet select, add or remove a built-in Rule Set in a Design file. `--rule-set <file>` loads a set at
run time; a file with the id and version of a loaded set but other content is refused, because a changed set needs
a new version.

## Limits and known findings

- **Rule Sets are `draft`.** No set is `stable`. The 39 `ovcd` Rules, and 9 of the 24 published-rule Rules, have no
  second reader recorded (see the inventory).
- **Not evaluated, blocking:** the weather criterion (`is2008/A.2.3/weather`), damage survival
  (`igc/2.7/damage-survival`), the tank capacity table and the cargo-area hull design (`lr-inland-gas`), and, for a
  dry bulk Design, every IS Code row. The `stability.limitations` text says the same in words.
- **Findings** (#121): #280 (the cofferdam 0.60 m cites a type C clause; fix with #417), #282 (the areas to 40°
  ignore the down-flooding angle φf; a known limitation now), #381 (the opening setback is a length, not the
  position of each opening), #313 (the S690QL1 allowable stress, closed: `igc/4.23.3.1/allowable-stress` warns on the legacy 285 MPa value) and
  #317 (Type C tank checks; four Rules of `igc-2g` 1.1.0 are not verified). #281 (end space keyed to the tank end
  shape) and #283 (the 1.0 m "standard IGC-2G spacing") are fixed. #65 is a tank-size limitation outside this page.
- **No freeboard rule** (ICLL, ES-TRIN) is applied (#428). The freeboard checks are OVCD Checks on an estimated
  0.3 m. No inland stability rule (BV NR217, ADN 9.3.1.13) and no ES-TRIN visibility rule is checked. A river-only
  result names the Rule Sets used and does not say what no set covers.
- The type G hull clauses of the inland rule book (3.5.2) are in `lr-inland-gas` 1.1.0 as a Rule that is not
  evaluated. The numbers of the old clause (0.80 m side, 0.60 m double bottom) were removed in #215.
- The IS Code applies to ships of 24 m in length and over (Part A 1.1.1). OVCD applies the Rules to every gas
  Design: it cannot write a length condition as Applicability.
- The word "governing" names a Rule in the trace and a result in two places (section 5).

## Evidence

- `crates/ovcd-format/tests/rule_set.rs`: the loader refuses a `regulation` Rule that cites no paragraph and a rule
  id whose paragraph does not match its locator. `crates/arc-rules/tests/toy_domain.rs`: the kernel loads and
  evaluates a set in a toy domain that is not a vessel (the evaluator gives the outcomes, the Margin and the
  governing row).
- `crates/arc-rules/tests/lean_oracle.rs`: the differential test against the Lean model, in the weekly Lean job.
- `crates/ovcd-format/tests/is2008_a.rs`: the IS Code rows on box sections, including the verdict that names
  the blocking Rules not evaluated.
- `crates/ovcd-cli/tests/rules_command.rs`: `ovcd rules`, `ovcd rules show` and the trace in `ovcd run --json`.
- `ovcd rules --provenance` (CI): every released version is built in with its hash, a later version goes up by its
  class, a rule id keeps its Measure and direction, and every regulation or class Rule cites a paragraph.
- `crates/ovcd-core/tests/protective_location.rs`: sourced tests of d(Vc) at 1,000, 5,000 and 30,000 m³ and either
  side, and of the clear gap d less the insulation (labelled `sourced - IGC Code 2.4.1.1`).
- Evaluation harness (`evaluation/`, #184, #278, #279): IS Code Part A 2.2.1 to 2.2.4 on box sections with a known
  KG, 60 boundary mutants, 0 false passes; it also mutates rule data. Verdicts come from an independent oracle; no
  source gives a published verdict. It covers the IS Code rows only.
- No test or harness case checks an OVCD Check against a source: an OVCD Check has none.

## Questions for reviewers

1. IGC 4.23.2.1.3 says the weld factor "shall be 0.95" with the stated NDT and "may be increased up to 1".
   `igc-2g` holds two Rules: a warning above 0.95 and a block above 1. Is that the right reading?
2. For the length-over-diameter limit of 7 (ADN 9.3.1.11.1(b); inland rule book 3.5.1), is the diameter the steel
   shell or the outside of the insulation? OVCD uses the insulated diameter. The limit is both an `ovcd` tool limit
   and an unverified Rule in `lr-inland-gas` 1.1.0 (3.5.1). Should only one stay?
3. Is the aft separation length a fair Measure for 3.3.7 and 3.3.11, which set 2.00 m from the cargo area to each
   entrance and opening (#381)? Should 3.3.8 (engine room ventilation inlets) and 3.4.1 (funnel outlets), also
   2.00 m, be checked too?
4. Should the IS Code Part A criteria apply to a gas barge under 24 m (Part A 1.1.1), and to a river-only unmanned
   barge at all? Which inland stability set should replace them for river-only service?
5. Is N−1 genset redundancy (`ovcd/aft-block/gensets-reserve`) a class requirement you would cite, and from which
   rule book?
6. The hopper GM check (`ovcd/capacity/intact-gm`) warns at 0.15 m, the number of IS Code Part A 2.2.4. Is that the
   right first check for a dry bulk barge, or should dry bulk wait for the full `is2008-a` or an inland stability
   set?
7. Should the scope notes of the capital budget (`no_reliquefaction_in_scope`, `fabrication_rates_are_estimates`)
   be `note` Rules in the trace?
