# Rules and Rule Sets

> Concept and preliminary design only. Not for approval. Draft: needs expert review.
>
> Code state: written against `main` at commit `f6d446c` (2026-10-20). **Out of date:** since then the Rule Set loader (#268), the evaluator and Rule Trace (#269), `is2008-a` (#270), `igc-2g` (#271) and `ovcd` / `lr-inland-gas` (#272) have merged. A rewrite is ticketed.

**For:** Classification and regulatory specialist  ·  **Code:** `crates/ovcd-core/src/checks.rs`,
`gas_stability.rs`, `gas_separation.rs`, `protective_location.rs`, `gas_arrangement.rs`,
`barge_capacity.rs`, `crates/ovcd-format/src/simulate.rs`  ·  **Rule Sets:** none in code today.
Designed: `is2008-a`, `igc-2g`, `lr-inland-gas`, `ovcd` (spec 0004, tickets #268 to #279).

This page has two parts. The first part says how OVCD means to interpret and apply rules: the terms,
codification, rule ids, editions, "the strictest rule governs", the Service Area, outcomes and the Rule
Trace. Most of that is **designed, not built**. The second part is the inventory: every check the engine
runs **today**, with its citation and its status. Each section says which part exists in code.

## What OVCD computes here

Today (in code):

- A list `checks` in every Design result. Each item has `name`, `ok`, `message` and, for most items,
  `severity` (`"error"` or `"warning"`). The app shows it in the **Rule checks** tab
  (`app/src/lib/views/RuleChecks.svelte`). The CLI summary prints "Checks: all N pass" or the names of
  the failing checks (`crates/ovcd-format/src/run.rs`).
- For a gas Design, a `stability` object: the criteria set ("IMO IS Code 2008, Part A 2.2"), three
  loading conditions, and per condition six criteria with value, limit, `ok` and a source text.
- For a gas Design, `fit` selects the arrangement. It keeps a candidate only when every
  error-severity check of that candidate passes (`gas_arrangement.rs`, "THE CHECKS THE SEARCH USES").
- A `capital budget` object with its own three checks (`crates/ovcd-format/src/cost.rs`).

Designed, not built (spec 0004):

- A **Rule Trace** in every result: one row per Rule, per object, of every selected Rule Set.
- The Rule Sets and versions each Option used, with a content hash.
- A count, in every passing verdict, of the Blocking Rules that the engine did not evaluate.

## How it works today

### 1. The terms (glossary)

These terms come from `GLOSSARY.md` and spec 0004. They define the target design. Today the code has only
the `Check` struct and, for stability, a `Criterion` struct.

- **Rule.** One requirement about one Measure: its Criterion, its Applicability, its consequence and its
  source. An **OVCD Check** is a Rule whose origin is OVCD itself (physical feasibility, the validity
  range of a method, or a limit of the tool). It is never shown as a published rule.
- **Measure.** A quantity the engine computes for one object (an Option, the hull, a tank or a loading
  condition), for example the GZ area from 0° to 30° of one loading condition. A Measure is Rust code with
  its own tests. It cites a **method**, not a rule. A design input that a rule bounds (for example the
  weld factor) is an *input Measure*.
- **Criterion.** The test part of a Rule: a comparison (`ge`, `gt`, `le`, `lt`), a limit (a constant, a
  table row, or a formula shape coded in Rust with its coefficients in data) and a tolerance (default 0).
  The Criterion holds for value v when v ≥ limit − tolerance (`ge`), and so on for the other forms.
- **Margin.** v − limit for a lower bound, limit − v for an upper bound. A positive Margin passes.
- **Requirement.** The Rules, from all selected Rule Sets, on the same Measure instance (same arguments),
  the same object and the same direction.
- **Rule Set.** A named, versioned group of Rules from one source edition. It has two versions: the source
  **edition** (what the rule maker published) and the **set version** (OVCD's encoding, semver).
- **Applicability.** A list of conditions `input ∈ {values}` on the Option's enumerated inputs (Cargo,
  Cargo Containment, Service Area, LPG Carriage Condition, tank head type). There is no expression
  language. A condition on a number (for example "24 m in length and over") cannot be written as
  Applicability.

**In code today:** `Check` (`checks.rs`) has a name, a yes/no result, a message and a severity text. Only
the exact text `"error"` gates a result (`all_ok`); every other text acts as a warning. A check has no
value field, no limit field, no Margin, no citation field and no verification status. Where a citation
exists, it is inside the message text. The stability `Criterion` (`gas_stability.rs`) has a name, a
value, a limit, `ok` and a source text, but no comparison field and no Margin.

### 2. How a rule text becomes a Rule (codification) — designed

The process is in `docs/rules/codification.md`. A **codifier** (a person or an agent):

1. Gets the official text and fixes the edition. "As amended" means: list each amending resolution
   checked, and say which ones change the paragraph. The file goes into the rule-source library with its
   SHA-256.
2. Splits the paragraph into one Rule per requirement, and gives each an id.
3. Picks the Measure and checks that it measures what the text measures (object, datum, range, units).
   If no Measure fits, the work stops and a new Measure is a Rust change.
4. Records the comparison from the words ("not less than" is `ge`), the limit as printed, the units,
   the Applicability and the citation (document, edition, paragraph, page, access).
5. Sets `codified`, and `verified: false`.
6. Adds sourced tests at both sides of the limit, and harness boundary cases from the oracle.

A **second reader**, human or agent, who did not codify the Rule, opens the cited edition at the cited
page and works through a checklist. Only then is `verified` filled, with the role (never a person's name),
the date, the edition held, the locator read and the review link. Spec 0004 calls a Rule **sourced** only
when its origin is `regulation` or `class` **and** `verified` is filled.

**In code today:** no Rule records exist. No check has been verified by this process. The code audit
behind spec 0004 (205 rule-like items) is one reading, not a recorded second reading.

### 3. Rule ids and citations — designed

A rule id is `<family>/<paragraph>/<slug>`, for example `is2008/A.2.2.1/area-0-30`. For a `regulation`
or `class` Rule, the paragraph segment is the source locator with spaces removed, and the loader checks
that the two agree. For an OVCD Check, the paragraph segment is a topic (`ovcd/gas-arrangement/...`). An
id keeps its Measure and direction in every later version. A change of either gives a new id. An id is
never used again.

A formula that a requirement uses, but that is not itself a requirement, is a table of the set, not a
Rule. Example: d(Vc) of IGC 2.4.1.1 is the table `igc-d-vc`. The type 2G requirement that uses it is
`igc/2.4.1.2/side-clearance`.

**In code today:** a check is known by its `name` (for example `opening_setback`). The citation, where
one exists, is free text in the message ("cl. 3.3.7 / 3.3.11") or the stability source text
("IMO IS Code 2008, Part A 2.2.1"). The text names no edition, page or resolution.

### 4. Editions and amendments — designed

A Rule Set records its edition: the base document, each amendment, and the in-force date. Example
(spec 0004): `igc-2g` is the IGC Code as adopted by MSC.370(93), as amended by MSC.411(97), MSC.441(99),
MSC.476(102), MSC.492(104), MSC.523(106) and MSC.566(109). The rule-source notes
(`ovcd-rule-sources/igc/NOTES.md`) record that no amending resolution changes 2.3 or 2.4; only
MSC.492(104) changes chapter 2, at 2.7.1.1.

Every released set version is kept unchanged. A change of a limit is a new version. A verification that
changes no result-changing field is a **patch** version with the same content hash. A saved file keeps
the set version it used. The engine never moves a file to a newer version without a warning.

**In code today:** no edition is recorded anywhere a result shows. The only rule-set input is `ruleset`,
with one accepted value, `IGC_2G`. A file that names `IGC_2G_EXACT` or `ADN_G` is read as `IGC_2G`, with
a warning (`crates/ovcd-format/src/vdesign.rs`, `migrate_ruleset`).

### 5. The strictest rule governs — designed

All Rules in a Requirement apply. The Option meets the Requirement only if it meets every one. Of two
Criteria in one direction, the stricter one lets fewer values pass (larger effective bound for a lower
bound, smaller for an upper bound; at an equal bound, a strict comparison is stricter). This order is
total, so the strictest Rule always exists. It is marked **governing**. It is chosen from the Criteria,
not from the design value, so it does not change when the design changes. Ties go to the first set in
selection order, then to the rule id. There is no override field: no Rule can loosen another (ADR 0013).
A Lean model proves these properties for a model of the kernel (ADR 0014, `proofs/lean/`). It is tied to
Rust by a differential test that waits for the evaluator (#269).

**In code today:** there is never more than one limit per quantity, so nothing chooses a governing rule.
The word "governing" in today's results means something else: the stability loading condition with the
lowest fluid GM (`StabilityAssessment::governing`), and the light air draft
("Air draft (empty, governing)").

### 6. How the Service Area selects Rule Sets — designed

A new input `service_area` (`river-only` or `coastal-and-river`; default `river-only`) sets the default
stack. In the first release both Service Areas give the same stack:

| Cargo | Default stack (spec 0004) |
|---|---|
| Gas | `igc-2g` (which includes `is2008-a`), `lr-inland-gas` (draft, unverified), `ovcd` |
| Dry bulk and others | `is2008-a` (all rows `not evaluated`), `ovcd` |

The `ovcd` set is always selected. A river-only result must say that no statutory technical code was
checked, only the named Rule Sets. **In code today:** there is no Service Area input and no stack. Gas
Designs get the IS Code 2008 Part A 2.2 criteria and the gas checks. Dry bulk Designs get no stability
criteria at all, only a first-order hopper GM warning.

### 7. Pass, fail, warn and "not evaluated"

Designed (spec 0004, "Outcome of one Rule on one object"):

| Condition | Outcome |
|---|---|
| The Rule's set does not apply | one `not applicable` row for the set |
| The Rule does not apply, or no object of its type | `not applicable` |
| The engine lacks the Measure (`evaluated: false`) | `not evaluated`, with a gap text |
| The Measure failed, or gave NaN or infinity | `fail` (block) or `warn` (warn), never `pass` |
| The Criterion holds | `pass` |
| The Criterion does not hold | `fail` (block) or `warn` (warn) |

An Option passes when no row is `fail`. A `not evaluated` row does not stop a pass, but every passing
verdict names the Blocking Rules not evaluated (maintainer decision, 2026-10-20).

In code today:

- An `"error"` check with `ok: false` makes the module result not `ok`, and makes a `fit` candidate
  fail. A `"warning"` check never gates.
- The app and the CLI summary ignore `severity`. A failed warning shows as **FAIL**, the same as a failed
  error (`RuleChecks.svelte`; `run.rs`).
- Five checks that `simulate.rs` adds ("Cargo > 0", "Feasible arrangement",
  "Air draft (empty, governing)", "tank length over diameter", and the `Stability — <condition>` rows)
  carry no `severity` field.
- "Not evaluated" does not exist as an outcome. Two requirements the engine lacks are only plain text in
  `stability.limitations`: "No wind heel and no damage stability".
- When the stability assessment cannot run, `fit` adds one failed check, "Stability — not assessed".
  So a candidate the engine cannot judge never passes (`gas_arrangement.rs`, `stability_checks`).

### 8. What the Rule Trace shows — designed

One row per Rule, per object, in selection order: rule id, Rule Sets that hold it, citation, origin,
rule status and method status (both shown), object, inputs with their source (typed, default,
default-estimate, class, computed), value, limit, comparison, tolerance, Margin, outcome, governing flag,
`sizes` flag, gap text and linked finding. The trace is outside the results hash (as the `drawing` object
is). The existing `checks` list stays, and each check gains the id of the Rule it reports.

**In code today:** no trace exists. The nearest things are the `checks` list and the `stability`
conditions, which show value and limit but no Margin, no rule status and no edition.

### 9. Rules that size the design, not checks

Some rule values never appear as a check. The search builds the arrangement with them, so the result
meets them by construction. Spec 0004 calls these `sizes` Rules and says the same Rule must also be
checked on the result. **In code today** they are only sizing: nothing checks the result against them.
They are rows RS-A60 to RS-A66 in the inventory below.

## Assumptions, rules and estimates

This table is the **check inventory**. It lists every check the engine emits today. "Where applied"
says where the check shows:

- **Design**: in the `checks` list (or the `stability` or `capital budget` object) of `ovcd run` and the app.
- **fit**: it gates a `fit` candidate. A fit-only check shows in a Design result only through the message
  of "Feasible arrangement" when no candidate passes.
- **API**: emitted by a core module, but only through the module call (`ovcd eval`, the Python bridge),
  not in a Design result.

Every row is today a **hard-coded check, not yet a Rule**. The target Rule Set is the one spec 0004
names. "Matches the text" means the reading done for this page; it is not a recorded second reading. No
check is verified in the spec 0004 sense. Labels follow `docs/basis/README.md`: a cited check is
**Rule (unverified)**; an uncited hard-coded limit is **Fixed in code**; a limit from an input is
**Estimate** or **Sourced default** as its input is.

### Published-rule checks (cited)

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| RS-A01 | Area under GZ, 0° to 30° | ≥ 0.055 m·rad, trapezoid on a 1° grid, per loading condition | Rule (unverified) | IS Code 2008, MSC.267(85), Part A 2.2.1 (PDF p. 12). Matches the text. | `gas_stability.rs` `criteria`; Design, fit. Target `is2008/A.2.2.1/area-0-30` in `is2008-a` (#270) | Unknown (the Measure is a prismatic box section) |
| RS-A02 | Area under GZ, 0° to 40° | ≥ 0.090 m·rad, always to 40° | Rule (unverified) | Part A 2.2.1. Value matches. The text says "or the angle of down-flooding φf if this angle is less than 40°"; the engine has no φf. Finding #282 | As RS-A01. Target `is2008/A.2.2.1/area-0-40` | Non-conservative where openings immerse before 40° |
| RS-A03 | Area under GZ, 30° to 40° | ≥ 0.030 m·rad, always 30° to 40° | Rule (unverified) | Part A 2.2.1. Value matches; same φf gap (#282) | As RS-A01. Target `is2008/A.2.2.1/area-30-40` | Non-conservative, as RS-A02 |
| RS-A04 | GZ at 30° or more | max GZ on the 30° to 60° grid ≥ 0.20 m | Rule (unverified) | Part A 2.2.2. Matches the text | As RS-A01. Target `is2008/A.2.2.2/gz-30` | Unknown |
| RS-A05 | Angle of maximum GZ | ≥ 25°, on the 0° to 60° grid (integer degrees) | Rule (unverified) | Part A 2.2.3. Matches. The text also allows alternative criteria with the Administration's approval; OVCD does not model that | As RS-A01. Target `is2008/A.2.2.3/angle-max-gz` | Conservative (no alternative route) |
| RS-A06 | Initial GM (fluid) | GM0 less free surface of the cargo ≥ 0.15 m | Rule (unverified) | Part A 2.2.4. Matches. Free surface counts the cargo tanks only, not consumables (IGC 2.2.3 asks for those) | As RS-A01. Target `is2008/A.2.2.4/gm0` | Non-conservative by the consumables' free surface |
| RS-A07 | Opening setback, aft | aft separation = max(cofferdam, setback) ≥ 2.00 m, tolerance 1e-9 m | Rule (unverified) | Inland class rule book (LR Inland Waterways Ships) Pt 4 Ch 4 Sec 3, 3.3.7 and 3.3.11; same 2.00 m in ADN 2025 9.3.1.17.4. Matches (2015 edition and the July 2022 mirror). Measure is a proxy: a length, not the position of each opening | `gas_separation.rs`; Design (always passes: a Design cannot change the inputs), fit; API can change them. Target `lr-inland/3.3.7/opening-setback` in `lr-inland-gas` (#272) | Unknown (proxy) |
| RS-A08 | Cofferdam width | ≥ 0.60 m, tolerance 1e-9 m | Rule (unverified) | Cites 3.6.13. That clause is in the **type C** (liquid) part. No gas-tank source. Finding #280 (fix with the IGC chapter 3 replacement) | As RS-A07. Target `lr-inland/3.6.13/cofferdam` | No gas basis; with defaults no geometry moves |
| RS-A09 | Forward end-bulkhead space | ≥ 0.20 m if `is_pressure_tank`, else ≥ 0.50 m; tolerance 1e-9 m | Rule (unverified) | Cites 3.5.3 / 3.6.3. 3.5.3 (type G) keys the 0.50 m to **plane tank ends**; the "pressure tank" wording is from 3.6.3 (type C). Finding #281 | As RS-A07. Target `lr-inland/3.5.3/end-space-domed` and `-plane` | With defaults (domed heads, 0.20 m) correct; wrong only if `is_pressure_tank: false` |

### OVCD checks with a known origin but no citation in the check

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| RS-A10 | Tank length over diameter | overall tank length (heads in) / Tank Outer Diameter (insulated) ≤ 7.0 | Fixed in code | Code comment: the figure of the removed `ADN_G` set, ADN 2025 9.3.1.11.1(b) ("ratio of length to diameter exceeds 7 are prohibited"). The same rule book family as RS-A07 also has it for type G: 3.5.1. The check message cites nothing | `gas_arrangement.rs` `length_over_diameter_check`; fit; Design (only when it fails). Target `ovcd/gas-arrangement/tank-l-over-d` (tool limit) | Unknown: the insulated diameter gives a smaller ratio than the steel diameter |
| RS-A11 | Hopper intact GM | GM ≥ 0.15 m, warning, first-order (KG light at 0.5 D) | Fixed in code | Same number as IS Code Part A 2.2.4, not cited. Name `gm_positive` says "> 0" but the limit is 0.15 | `barge_capacity.rs`; Design (dry bulk). Target `ovcd` | Unknown |

### OVCD Checks: physical feasibility and tool limits (gas and dry bulk)

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| RS-A12 | Cargo > 0 | carried cargo > 0 t | Fixed in code | Physical | `simulate.rs`; Design. No severity field | — |
| RS-A13 | Feasible arrangement | added only when no `fit` candidate passes; lists the failed check names | Fixed in code | Tool | `simulate.rs`; Design (gas) | — |
| RS-A14 | `feasible_geometry` | cargo volume > 0 | Fixed in code | Physical | `barge_capacity.rs`; Design, fit | — |
| RS-A15 | `cargo_zone_positive` | LOA − bow void − stern void > 0 | Fixed in code | Physical | `barge_capacity.rs`; Design, fit | — |
| RS-A16 | `not_overloaded` | volume-bound load floats within the moulded depth | Fixed in code | Physical | `barge_capacity.rs`; fit. Removed from the Design list (`simulate.rs` `capacity_checks`) | — |
| RS-A17 | `freeboard_ok` | freeboard at volume-bound load ≥ required freeboard | Estimate (`min_freeboard_m`, 0.3 m, no source) | No freeboard rule (ICLL, ES-TRIN) is applied | `barge_capacity.rs`; fit. Removed from the Design list; a dry bulk Design caps the draught at depth − freeboard instead | Unknown: not a rule value |
| RS-A18 | `freeboard_at_payload` | depth − carried draught + 1e-9 ≥ required freeboard | Estimate (0.3 m) | As RS-A17 | `gas_arrangement.rs`; fit | As RS-A17 |
| RS-A19 | `air_draft_within_limit` | light (or ballasted light) air draft ≤ limit | Estimate (user input `air_draft_limit_m`) | Waterway limit typed by the user | `barge_capacity.rs`; Design, fit, when a limit is set | — |
| RS-A20 | Air draft (empty, governing) | light air draft ≤ limit (ignores ballast) | Estimate (as RS-A19) | Duplicate of RS-A19 | `simulate.rs`; Design. No severity field | Differs from RS-A19 when ballast is set |
| RS-A21 | `capacity_is_volume_bound` | capacity input not overloaded | Fixed in code | Tool | `journey_derating.rs`; fit | — |
| RS-A22 | `floats` | the channel draught cap leaves a cargo > 0 | Fixed in code | Physical | `journey_derating.rs`; fit | — |
| RS-A23 | `ukc_consistent` | design depth ≥ UKC | Fixed in code | Tool (input consistency) | `journey_derating.rs`; fit | — |
| RS-A24 | `utilisation_reasonable` | volume utilisation ≥ 0.5, warning | Fixed in code | Tool | `journey_derating.rs`; fit (never gates) | — |
| RS-A25 | Floats with freeboard | draught < depth, else this one failed criterion replaces the six | Fixed in code | Physical; source text "draught ≥ depth" | `gas_stability.rs`; Design, fit | — |
| RS-A26 | Stability — not assessed | one failed check when the assessment raises an error | Fixed in code | Tool; a candidate the engine cannot judge never passes | `gas_arrangement.rs`; fit | — |

### OVCD Checks: stern block (self-propelled hulls)

All in `aft_arrangement.rs`; Design and fit. Target `ovcd` (#272). None is cited.

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| RS-A27 | `thrusters_fit_beam` | thrusters abreast fit the beam | Fixed in code | Physical | error | — |
| RS-A28 | `requested_thruster_count_fits` | all requested thrusters fit | Fixed in code | Tool | warning | — |
| RS-A29 | `containers_fit_beam` | 40 ft units abreast fit the beam | Fixed in code | Physical | error | — |
| RS-A30 | `units_fit_in_one_row` | one row of units | Fixed in code | Tool | warning | — |
| RS-A31 | `wheelhouse_fits_beam` | wheelhouse width ≤ beam | Fixed in code | Physical | error | — |
| RS-A32 | `accom_beside_wheelhouse` | units fit beside the wheelhouse (gas) | Fixed in code | Tool | warning | — |
| RS-A33 | `gensets_fit_beam` | gensets abreast fit the beam, tolerance 1e-9 m | Fixed in code | Physical | error | — |
| RS-A34 | `gensets_fit_under_deck` | genset height + 1.00 m ≤ moulded depth | Estimate (1.00 m, marked ESTIMATE in the message) | Tool | warning | Unknown |
| RS-A35 | `gensets_carry_load_with_one_in_reserve` | (n − 1) × set rating ≥ electrical load | Fixed in code | No rule cited; an N−1 redundancy criterion | error, when a load is given | Unknown |
| RS-A36 | `wheelhouse_sees_over_cargo` | raised eye ≥ obstruction + margin | Estimate (margin input) | No visibility rule cited | warning (an open maintainer question on #264 is whether it should block) | Unknown |

### OVCD Checks: capital budget

`barge_cost.rs`; Design (`capital budget` object). Target `ovcd`.

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| RS-A37 | `masses_positive` | lightest mass input > 0 | Fixed in code | Tool | error | — |
| RS-A38 | `no_reliquefaction_in_scope` | always `ok: true` | Fixed in code | A scope note, not a test | warning | — |
| RS-A39 | `fabrication_rates_are_estimates` | always `ok: false` | Fixed in code | A scope note; it always reads as failed | warning | — |

### OVCD Checks: method validity and tool limits, module API only

These are not in a Design result. A Design run uses the squat limit only to cut the speed curve
(`squat_limited`), not as a check.

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| RS-A40 | `service_speed_below_limiting[env]` | service speed ≤ 0.95 × Schijf limiting speed + 1e-6 | Fixed in code | Method validity; 0.95 not cited | `resistance.rs`; API | Unknown |
| RS-A41 | `power_within_installed[env]` | power at service ≤ installed + 1e-6 | Fixed in code | Physical | `resistance.rs`; API | — |
| RS-A42 | `blockage_in_range[env]` | 0 < blockage < 1 | Fixed in code | Method validity | `resistance.rs`; API | — |
| RS-A43 | `coefficients_are_first_order` | always `ok: true`, warning | Fixed in code | A notice | `resistance.rs`; API | — |
| RS-A44 | `service_below_schijf` | service speed ≤ Schijf limit + 1e-6 (no 0.95 factor) | Fixed in code | Method validity; differs from RS-A40 | `electric_energy.rs`; API | Non-conservative against RS-A40 |
| RS-A45 | `energy_positive` | energy per km > 0 | Fixed in code | Physical | `electric_energy.rs`; API | — |
| RS-A46 | `resistance_coeffs_first_order` | always `ok: true`, warning | Fixed in code | A notice | `electric_energy.rs`; API | — |
| RS-A47 | `keel_clears_bed` | dynamic UKC > 0 | Fixed in code | Physical | `squat.rs`; API | — |
| RS-A48 | `keel_clears_bed_with_margin` | dynamic UKC ≥ margin (default 0.15 m), warning | Estimate (`squat_safety_margin_m`) | Carried over from the original engine | `squat.rs`; API | Unknown |
| RS-A49 | `froude_depth_subcritical` | depth Froude number < 0.70, warning | Fixed in code | Method validity, not cited | `squat.rs`; API | Unknown |
| RS-A50 | `blockage_within_barrass_range` | blockage ≤ 0.35, warning | Fixed in code | Method validity, not cited | `squat.rs`; API | Unknown |
| RS-A51 | `methods_agree_within_50pct` | two squat methods within 50 %, warning | Fixed in code | Tool | `squat.rs`; API | — |
| RS-A52 | `stawave1_short_wave_validity` | 0 < λ/Lpp < 0.5, warning | Fixed in code | Method validity, not cited | `waves.rs`; API | Non-conservative outside the range (the message says so) |
| RS-A53 | `wave_height_within_screening_range` | Hs ≤ 3.0 m, warning | Fixed in code | Tool | `waves.rs`; API | — |
| RS-A54 | `bow_length_supplied` | bow waterline length given, warning | Fixed in code | Tool | `waves.rs`; API | — |

### Catalogue data checks (not design checks)

`vessel_standards.rs`; API. They test the Vessel Class data, not a Design. Not planned as Rules.

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| RS-A55 | `unique_ids` | each key matches its standard id | Fixed in code | Data integrity | API | — |
| RS-A56 | `ranges_ordered` | every (min, max) ordered | Fixed in code | Data integrity | API | — |
| RS-A57 | `positive_dimensions` | beams and lengths > 0 | Fixed in code | Data integrity | API | — |
| RS-A58 | `convoy_beam_consistent` | convoy beam = unit beam × abreast | Fixed in code | Data integrity | API | — |
| RS-A59 | `source_cited` | every class cites its table, warning | Fixed in code | Data integrity | API | — |

### Rule values that size the design but are not checked

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| RS-A60 | Tank side clearance, type 2G | clear gap = max(d(Vc) − insulation, 0); d per tank, dome in Vc, four branches | Rule (unverified) | IGC MSC.370(93) 2.4.1.2 with 2.4.1.1 and 2.4.2 (PDF pp. 21–22). Matches the text. Sourced tests at each branch boundary (`tests/protective_location.rs`) | `barge_capacity.rs` `gas_volumetric` (sizing); no check. `side_clearance_override_m` replaces it with no check, and `design_grid` sets it to 1.0 m (RS-A66). Target `igc/2.4.1.2/side-clearance` (`sizes`) | Non-conservative on the override paths |
| RS-A61 | Tank bottom clearance, type 2G | max(min(B/15, 2.0 m), d) − insulation | Rule (unverified) | IGC 2.4.1.2 with 2.3.1.2.3 and 2.4.2. Matches | As RS-A60; no check | As RS-A60 |
| RS-A62 | Minimum wall thickness after forming | 3 mm (304L, 316L), 5 mm (9Ni, S690QL1), as a floor | Rule (unverified) | IGC 4.23.2.1.2 (5 mm C-Mn and nickel steels, 3 mm austenitic). Matches | `pressure_vessel.rs` (formula path only). A typed fixed thickness skips the floor, and no check reports it | Non-conservative for a typed thickness below the floor |
| RS-A63 | Weld factor | 0.95 default (`DEFAULT_WELD_EFFICIENCY`) | Rule (unverified) | IGC 4.23.2.1.3: 0.95 with the stated NDT; "may be increased up to 1". No bound is checked | `pressure_vessel.rs`; input of the eval API | Unknown |
| RS-A64 | Allowable stress | material table (113, 113, 230, 285 MPa) or a typed value; no check | Estimate | IGC 4.23.3.1: f = lesser of Rm/A, Re/B. S690QL1 285 MPa is above Rm/3 = 256.7 MPa (spec 0004, #292) | `pressure_vessel.rs`; no check | Non-conservative for S690QL1 |
| RS-A65 | Filling limit | `fill_ratio` 0.98 | Sourced default | IGC 15.3 (Defaults Profile) | input, not a check | — |
| RS-A66 | `design_grid` side clearance | 1.0 m "standard IGC-2G spacing" | Fixed in code | No such value in IGC 2.4.1.2. Finding #283 | `design_grid.rs` (override, no check) | Either way, by tank size |

### Requirements the engine does not evaluate (text only today)

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| RS-A67 | Weather criterion | not computed | — | IS Code Part A 2.3. Only the `limitations` text | `gas_stability.rs`. Target `not evaluated` row | Non-conservative (silent today) |
| RS-A68 | Damage survival | not computed | — | IGC 2.7 (and ADN 9.3.1.15 in the message). Only the `limitations` text | `gas_stability.rs`. Target `not evaluated` row | Non-conservative (silent today) |
| RS-A69 | IS Code applicability by length | criteria applied to every gas Design | — | IS Code Part A 1.1.1: ships of 24 m in length and over. Not expressible as Applicability | `gas_stability.rs` | Conservative for hulls under 24 m |

**Counts.** 59 emitted checks (RS-A01 to RS-A59; the six stability criteria counted one by one). 9 cite a
paragraph in the check itself (RS-A01 to RS-A09). Of these, 5 match the text in value and Measure
(RS-A01, -A04, -A05, -A06, -A07) and 4 differ (RS-A02, -A03 through #282; RS-A08 #280; RS-A09 #281).
0 are verified. 50 carry no citation. A further 7 rule values size the design or bound an input with no
check (RS-A60 to RS-A66), and 3 requirements are not evaluated (RS-A67 to RS-A69).

## Inputs the user can change

| Input | Default | Unit | Where the default comes from |
|---|---|---|---|
| `ruleset` | `IGC_2G` (the only value) | — | Defaults Profile; basis text cites IGC 2.4.1.1 and 2.4.2 |
| `min_freeboard_m` | 0.3 | m | Defaults Profile; no rule source |
| `air_draft_limit_m` | none | m | Typed by the user (waterway) |
| `draught_limit_m`, channel depth and UKC | none | m | Typed by the user (waterway) |
| `squat_safety_margin_m` | 0.15 | m | Carried over from the original engine |
| `fill_ratio` | 0.98 | — | IGC 15.3 (sourced) |
| `tank_wall_thickness_mm` | none (formula) | mm | A typed value skips the IGC minimum floor |
| `tank_allowable_stress_mpa` | none (material table) | MPa | No check against IGC 4.23.3.1 |

A Design cannot change the gas separation inputs (cofferdam, opening setback, forward end space,
`is_pressure_tank`), so RS-A07 to RS-A09 always pass in a Design run. A user cannot select, add or
remove a rule set, and cannot load one (`--rule-set` is designed, not built).

## Limits and known findings

- **No rules engine exists in code.** No Rule data, loader, evaluator, Measure registry, Rule Trace,
  Service Area, `ovcd rules` command or `crates/ovcd-format/rules/` folder. Tickets #268 to #279 are
  all open. The Lean model (`proofs/lean/`) exists; its tie to Rust waits for #269.
- **Findings register (#121):** #280 (cofferdam cites a type C clause), #281 (end space keyed to tank
  type, not end shape), #282 (areas to 40° ignore φf; decided: known limitation now, fix in phase 1),
  #283 (1.0 m "standard IGC-2G spacing"). All open; decisions recorded 2026-10-20. #65 is a tank-size
  limitation outside this page.
- Side and bottom clearances are sizing only. An override path (`side_clearance_override_m`,
  `design_grid`) can give a gap below d(Vc) − insulation with no failed check.
- Warnings show as FAIL in the app and the CLI summary; `severity` is not shown.
- The gas separation checks always pass in a Design run, because a Design cannot change their inputs.
- No freeboard rule (ICLL, ES-TRIN), no inland stability rule (BV NR217, ADN 9.3.1.13) and no
  ES-TRIN visibility rule is checked. A river-only result does not say this today.
- The type G hull clauses of the inland rule book (3.5.2: 0.80 m side, 0.60 m double bottom; ADN
  9.3.1.11.2(a)) were removed in #215 and are not in the designed `lr-inland-gas` set.
- The comment block in `gas_arrangement.rs` calls the gas separation checks "protective location
  clearances". They are longitudinal end spaces; protective location gives no check.

## Evidence

- `crates/ovcd-core/tests/protective_location.rs`: sourced tests of d(Vc) at 1,000, 5,000 and
  30,000 m³ and either side, and of the clear gap d less the insulation (labelled
  `sourced - IGC Code 2.4.1.1`).
- `crates/ovcd-core/tests/gas_separation.rs`: the constants, the 1e-9 m slack, and failing cases for a
  thin cofferdam and a short setback.
- `crates/ovcd-core/src/gas_stability.rs` unit test: each of the six criteria passes exactly at its limit
  and fails one floating-point step below.
- `crates/ovcd-core/tests/tank_datum.rs`, `dome_in_vc.rs`, `tank_wall_formula.rs`: the clearance datum,
  the dome in Vc, and the wall formula on the steel diameter.
- Evaluation harness, phase 0 (`evaluation/results.md`, #184): IS Code Part A 2.2.1 to 2.2.4 on box
  sections with a known KG. 5 base cases (3 with published inputs), 60 boundary mutants, 361 criterion
  verdicts, 0 false passes, 0 false fails. Verdicts come from the independent oracle; no source gives a
  published verdict. It covers RS-A01 to RS-A06 only.
- No test or harness case covers RS-A10 against a cited source, or any check of the stern block,
  capital budget or method validity against a source.

## Questions for reviewers

1. IGC 4.23.2.1.3 says the weld factor "shall be 0.95" with the stated NDT and "may be increased up to
   1". Spec 0004 plans an upper bound of 0.95 (`igc/4.23.2.1.3/weld-factor`). Is 0.95 a bound, or is
   1.0 the bound with 0.95 the default?
2. For the length-over-diameter limit of 7 (ADN 9.3.1.11.1(b); inland rule book 3.5.1), is the diameter
   the steel shell or the outside of the insulation? OVCD uses the insulated diameter.
3. Should the limit of 7 be a class or regulation Rule in `lr-inland-gas` (3.5.1, type G), rather than an
   OVCD tool limit, given that the set already holds 3.3.7 and 3.5.3 of the same section?
4. Is the aft separation length a fair Measure for 3.3.7 and 3.3.11, which set 2.00 m from the cargo area
   to each entrance and opening? Should 3.3.8 (engine room ventilation inlets) and 3.4.1 (funnel outlets),
   also 2.00 m, be checked too?
5. Should the IS Code Part A criteria apply to a gas barge under 24 m (Part A 1.1.1), and to a river-only
   unmanned barge at all? Which inland stability set should replace them for river-only service?
6. Is N−1 genset redundancy (RS-A35) a class requirement you would cite, and from which rule book?
7. The hopper GM warning uses 0.15 m. Is that the right first check for a dry bulk barge, or should
   dry bulk wait for the full `is2008-a` or an inland stability set?
8. Should a failed `warn` Rule show differently from a failed `block` Rule in every output, as spec 0004
   plans (`warn` outcome), and should the scope notes (RS-A38, -A39, -A43, -A46) be `note` Rules or
   leave the check list?
