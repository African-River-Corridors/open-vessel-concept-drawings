# Spec 0004: The rules engine

Status: **draft, deepened 2026-10-20, for the maintainer's approval.** Decision records: ADR 0013 (rules as data), ADR 0014
(formal proofs). Issue: #176.

## Problem Statement

The engine checks a Design against engineering rules, but the rules are not visible as rules. They are
constants, `match` branches on a ship-type string, and citations inside message text, spread through
ten modules of `ovcd-core` (about 40 checks, plus the six stability criteria). A designer cannot see which
rule sets a result was checked against, which paragraph each limit comes from, or by how much a check
passed. A reviewer must read Rust to check a number against a page. An agent that adds a rule set must
edit code in several places, and nothing stops it from leaving a rule half built: that is how `ADN_G`
looked like compliance until #175 removed it.

A code audit against the source texts (205 rule-like items in the core and the format crate) found:

- 23 items cite a paragraph and match its text.
- 11 cite a paragraph but the value or the measure differs. Seven of them are already fixed (#237, the
  stepped d(Vc); #215, the inland type G floors). Four remain.
- 128 are design defaults, tool limits or physical feasibility checks: OVCD's own choices, not rules.
- 43 have no source that can be checked yet: pressure-vessel formulas from a paywalled code, material
  strengths, and the validity ranges and coefficients of published methods (squat, resistance, waves).

The evaluation harness (#184) needs one row per rule to say which rule a disagreement is about. `fit`
(#264) needs a list of the rules a design must meet. Neither exists.

## Solution

Rules become **data**: named, versioned **Rule Sets** in files that a reviewer can read and an agent can
edit. Physics stays **code**: **Measures** in `ovcd-core` compute the quantities the rules compare.

1. A Rule Set file format (JSON, with a published schema) and a loader in `ovcd-format`. The built-in
   sets are files in the repo, built into every binary and the WebAssembly build. Any set can also be
   loaded at run time.
2. A rule evaluator and a **Rule Trace** in `ovcd-core`: every result lists every Rule of every selected
   Rule Set, with citation, inputs, value, limit, margin, outcome and verification status.
3. Today's rules move into four built-in sets, with **no change to any result or results hash**:
   `is2008-a` (IS Code 2008 Part A), `igc-2g` (IGC Code as amended, type 2G), `lr-inland-gas` (the gas
   separation clauses of Lloyd's Register's inland waterways rules, draft) and `ovcd` (OVCD's own checks).
4. A **Service Area** input (river only, or coastal and river) sets the default Rule Sets of an Option.
   The Option stores the sets it used, with version and hash.
5. `fit`, the evaluation harness and the app read the Rule Trace.
6. One `.vdesign` format version (3) carries the new inputs, shared with the hull geometry block of
   ADR 0012.

## User Stories

### Designer
1. As a designer, I want to choose a Service Area for each Option, so that the Option is checked against the rules for where it will operate.
2. As a designer, I want the Service Area to pick the Rule Sets for me, so that I do not have to know the rule books to start.
3. As a designer, I want to see which Rule Sets and versions an Option was checked against, so that I can state the basis of the design.
4. As a designer, I want to add or remove an optional Rule Set, so that I can check an owner's or a class requirement.
5. As a designer, I want every rule check to show its value, its limit and its margin, so that I see how close the design is to failing.
6. As a designer, I want to see which rule governs when two Rule Sets cover the same requirement, so that I know which one to design to.
7. As a designer, I want rules the engine cannot evaluate yet listed as "not evaluated", so that a missing check is never mistaken for a pass.
8. As a designer, I want unverified rules and OVCD's own checks marked as such, so that I never present them as published rules.
9. As a designer, I want `fit` to return only designs that meet every blocking rule, so that the answer I get is a design that passes.
10. As a designer, I want a saved file to give the same results when I open it a year later, so that an engine update does not change my study silently.
11. As a designer, I want a warning when a Rule Set in my file has a newer version, so that I can choose to re-check against it.

### Reviewer or expert
12. As a reviewer, I want each rule to cite its document, edition and paragraph, so that I can check it against the text in minutes.
13. As a reviewer, I want each rule's limits and tables in a readable file, so that I can check numbers without reading Rust.
14. As a reviewer, I want each rule to say who verified it against which edition, and when, so that I know what has been checked.
15. As a reviewer, I want the origin of every check (published rule, class rule, method validity, physical, tool limit), so that I can tell compliance from engineering judgement.
16. As a reviewer, I want each rule whose code value differs from its source linked to a finding, so that known gaps are listed, not hidden.
17. As a reviewer, I want to propose a rule change as a small data diff, so that I can contribute without writing Rust.
18. As a reviewer, I want the harness to show me only the disagreements it cannot sort, so that my time goes where it is needed.

### Agent
19. As an agent, I want a schema and a loader that refuse a malformed Rule Set with a clear message, so that I can edit rule files safely.
20. As an agent, I want to load a Rule Set file at run time (`--rule-set`), so that I can test a rule change without rebuilding.
21. As an agent, I want a new rule set to need only a data file and harness cases, unless it needs a new quantity, so that adding ADN or a class rule set later is small.
22. As an agent, I want the Rule Trace as JSON from `ovcd run --json`, so that I can read the governing rule and its margin.
23. As an agent, I want CI to fail when a released Rule Set version changes, so that old files always reproduce.
24. As an agent, I want the harness to name the first rule whose verdict differs, so that I fix the right thing.
25. As an agent, I want the harness to move each rule limit and see if a case notices, so that I know when the corpus is too weak.

### Maintainer
26. As the maintainer, I want every migration step to change no result and no results hash, so that the move to data is safe.
27. As the maintainer, I want the four rules whose cited value differs filed as findings, so that I decide each fix one at a time.
28. As the maintainer, I want one format version bump for the rules and the hull block, so that users migrate once.

## Implementation Decisions

### Terms

The glossary holds the terms: **Rule**, **Criterion**, **Requirement**, **Applicability**, **Blocking
Rule**, **Verified**, **Rule Set**, **Measure**, **Rule Trace**, **Margin**, **Service Area**,
**Governing Rule**, **OVCD Check**. The IS Code calls its requirements "criteria"; in OVCD a Criterion is
only the test part of a Rule (comparison and limit). The `Criterion` struct of `gas_stability` becomes a
trace row in code.

### Domain model (precise)

This section is the reference for the build tickets. Where a later section repeats a point, this one
wins.

**Objects.** A Measure is computed for one object of one **object type**: `option`, `hull`, `tank`
(a Type C gas cargo tank), `loading-condition`, and one type per liquid bulk tank role:
`cargo-tank`, `ballast-tank`, `slop-tank` (spec 0006 §5.2, #336). A Rule on cargo tanks so never
reaches a ballast or slop tank. An Option has one `option` and one `hull`, zero or more `tank`s, and zero or more
`loading-condition`s. Objects of one type are listed in a fixed order (tank 1, 2, …; loading conditions
in the order the engine builds them), so a trace is the same on every run and platform.

**Measure.** A Rust function with a stable id, a unit (or `bool`), an object type, named arguments
(`gz.area` takes `from_deg` and `to_deg`), a **method** citation and a **method status**
(`verified` / `unverified`). A **Measure instance** is a Measure id with its arguments bound
(`gz.area(0, 30)`). Two kinds:

- *Computed*: the engine works it out (`gz.area`, `tank.side_gap`).
- *Input*: the value of a design input the engine uses (`input.weld_factor`,
  `input.tank_allowable_stress`), whether typed or default. A Rule that bounds an input is a normal Rule
  on an input Measure. This replaces the `parameter` kind of the first draft.

A Measure either gives a finite value or fails with a reason. It never gives NaN or an infinity to a
Rule: the evaluator turns a non-finite value into a failure.

**Criterion.** `{compare, limit, tolerance}`.

- `compare`: `ge`, `gt` (a *lower* bound) or `le`, `lt` (an *upper* bound). `eq` is not offered: an
  equality in a rule book is two Criteria, or a design input.
- `limit`: a constant with a unit; or a **table** row picked by a Measure value or an input (steps or
  piecewise linear); or a **formula shape** coded in Rust with coefficients in the file. A table or
  formula limit is worked out per object before the comparison.
- `tolerance`: an absolute value in the limit's unit, default 0. It keeps the code's own comparison:
  `gas_separation` compares `value >= limit - 1e-9`, `gas_stability` compares `value >= limit` with no
  tolerance. The evaluator uses exactly the form `value >= limit - tolerance` (and the mirror forms), so
  results stay bit-identical. The tolerance is in the content hash.

The Criterion **holds** for a value v when: `ge`: v ≥ limit − tol; `gt`: v > limit − tol; `le`:
v ≤ limit + tol; `lt`: v < limit + tol. The **Margin** is v − limit for a lower bound and limit − v for
an upper bound, in the Measure's unit. So a Criterion that holds has Margin ≥ −tol (and > −tol for a
strict one); with a zero tolerance, `ge` and `le` pass at Margin 0 and `gt` and `lt` fail there.

**Strictness.** Of two Criteria in the same direction, the stricter is the one whose effective bound
(limit − tol for a lower bound, limit + tol for an upper bound) lets fewer values pass: the larger
effective bound for a lower bound, the smaller for an upper bound; at an equal effective bound, a strict
comparison (`gt`, `lt`) is stricter than a non-strict one. This is a total preorder on the Criteria of
one direction, for one object, once table and formula limits are worked out. So "the strictest" always
exists for a non-empty Requirement, and a value meets every Criterion of the Requirement exactly when it
meets the strictest one. (ADR 0014 records the proof of this property.)

**Rule.** One requirement about one Measure instance. It has: an id; a title; an origin; a source
citation; a verification record; an object type (the Measure's); an **applicability** (below); a Measure
instance; a Criterion (none for a `check` Rule, whose Measure is `bool` and which holds when the Measure
is true); a **consequence** (`block`, `warn` or `note`); a `sizes` flag; an `evaluated` flag with its gap
text; an optional finding; and `since`. A `note` Rule has no Criterion and only reports a value with its
citation.

**Applicability.** A conjunction of conditions `input ∈ {values}` over the Option's inputs: `cargo`,
`cargo_containment`, `service_area`, `lpg_carriage_condition`, `tank_head_type`, and any other enumerated
input the schema names. It is the same form on a Rule Set (`applies_when`) and on a Rule
(`applies_when`). Four **derived inputs** are read-only: the engine works them out and a user never
types them (spec 0006 §5.2, #336):

- `service_area`: `river-only` or `coastal-and-river`. Since format version 3 (#274) it is an input of
  the Option, not derived; a file saved before reads `river-only`.
- `cargo_class`: `gas`, `dry_bulk` or `liquid_bulk`, from the Cargo, so a new Cargo of a class needs no
  Rule Set edit. An oil category for liquid bulk Cargos comes with the Crude Grade data (spec 0006,
  ticket 2).
- `deadweight_band`: `under-600-t`, `600-to-5000-t`, `5000-to-20000-t`, `20000-t-and-above`, from the
  result's deadweight, at the thresholds of MARPOL Annex I Reg. 19 ("x tonnes deadweight and above" is
  in the band that starts at x). So the 5,000 t switch of the MARPOL construction rules stays
  `input ∈ {values}`.
- `length_band`: `under-45-m`, `45-to-110-m`, `over-110-m`, from the length overall of the Option, at
  the thresholds of SOLAS V/22.1 ("not less than 45 m": 45 m is in the band that starts at 45) and
  ES-TRIN 28.02 ("exceeds 110 m": 110 m is in the band below). The length overall stands in for the
  length the Regulations define; the two differ by a few per cent (#543).

The loader refuses an Applicability that names an input it cannot name (a number input, an unknown
name) or a value the input does not have, so a typo never makes a Rule silently not apply. There is no expression language: no `or` inside a condition list across inputs, no
arithmetic, no thresholds. A rule-book condition on a number is a band of a derived input (`deadweight_band`, `length_band`) when
the thresholds are known, else a `check` Rule.

**Requirement.** The Rules of all selected Rule Sets with the same Measure instance, the same object and
the same direction. `check` Rules on one Measure form one Requirement. `note` Rules form none.

**Outcome of one Rule on one object** (a trace row):

| Condition | Outcome |
|---|---|
| The Rule's set does not apply to the Option | (one `not applicable` row for the set) |
| The Rule does not apply, or the Option has no object of its type | `not applicable` |
| `evaluated: false` | `not evaluated` |
| The Measure failed (error or non-finite value) | `fail` for a `block` Rule, `warn` for a `warn` Rule, with the reason as the gap |
| The Criterion holds | `pass` |
| The Criterion does not hold | `fail` (`block`), `warn` (`warn`) |
| `note` Rule | `note` |

A Measure that cannot be computed never passes. A Rule the engine lacks is `not evaluated`, which is
never a pass either.

**Governing Rule.** In each Requirement, the trace marks one row `governing`: the stricter of the
`block` Rules, or of all Rules if none blocks; on a tie of strictness, the first in selection order
(below), then by rule id. It is chosen from the Criteria, not from the value, so it does not change when
the design changes. Because one Measure value meets every Criterion exactly when it meets the strictest
one, the governing row has the smallest Margin of its Requirement whenever every row has the same
tolerance. `check` Requirements have no governing row.

**Option verdict.** The Option **passes** when no row has outcome `fail`. Rows with `not evaluated` do
not stop it from passing, but every result that says "passes" or `feasible: true` also says how many
Blocking Rules were not evaluated and names them ("meets every evaluated blocking Rule; 2 blocking Rules
not evaluated: IGC 2.7 damage survival, IS Code A 2.3 weather criterion"). No output says "meets every
rule" while a selected Blocking Rule is not evaluated. (Maintainer decision, 2026-10-20.)

**Sizing.** A Rule with `sizes: true` lends its limit to the arrangement search: the search builds
candidates with the governing limit of that Requirement (for example the clear side gap of each tank).
The same Rule is still checked on the result, so a typed override that breaks it fails. Sizing is a
property of how the search uses a Rule, not an outcome; the first draft's `size` outcome is gone.

**Rule Set.** A named, versioned group of Rules from one source edition, or from OVCD itself for the
`ovcd` set. It has an applicability, and it may `include` other sets at a pinned version.

**Selection.** The Option's selected sets, closed under `includes`. The closure is refused when: an
include cycle exists; two versions of one set id are in it (a typed `is2008-a@1.1.0` beside an
`igc-2g@1.0.0` that includes `is2008-a@1.0.0` is refused, with a message naming both); or one rule id
appears in two sets with different result-changing content. A rule id that appears in two sets with the
same content is evaluated once, and its row names every set that holds it. **Selection order** is the
order of the stored list, with each included set placed just after the set that includes it.

**Coherence of one set.** A set must not hold two Rules that no value can meet together without saying
so. For constant Criteria on the same Measure instance and object, the loader refuses a lower bound that
is stricter than an upper bound can allow (lower effective bound > upper effective bound, or equal with
a strict side) unless the set declares the pair in `known_conflicts` with a reason. Two selected sets
that conflict in this way are not refused: the user chose them, and the trace shows both failing; the
loader warns once. Conflicts through table or formula limits, or across different Measures, cannot be
found from the data alone and are left to the harness.

**Defaults coherence.** The defaults of the Defaults Profile must meet every Rule of the default stack
that bounds an input Measure. A test runs every default combination of the profile tables through those
Rules. A default that breaks one is a finding, not a silent pass (see the allowable stress finding under
#292).

### Measure (code)

- A named Rust function in `ovcd-core` that computes one quantity for one object. It has a stable id
  (`gz.area`, `gz.at_angle`, `gz.angle_of_max`, `gm0`, `tank.side_gap`, `tank.bottom_gap`,
  `tank.vc`, `tank.wall_thickness`, `squat.froude_depth`, ...), a unit, the object type it applies to,
  its arguments, and its own tests. Input Measures (`input.<name>`) echo the value of an input.
- It cites a **method**, not a rule ("trapezoid area under the GZ curve on a 1° grid"). A method from a
  source not yet in the library is `unverified` (the ASME pressure-vessel formulas, the squat and
  resistance methods), and the trace says so in `method_status`.
- The core keeps a registry of Measure ids with unit, object type, arguments and method status.
  `ovcd rules --measures` prints it.

### Rule (data)

| Field | Meaning | Example |
|---|---|---|
| `id` | `<family>/<paragraph>/<slug>` (grammar below), stable, never reused for another requirement | `is2008/A.2.2.1/area-0-30` |
| `title` | OVCD's own words, at most 15 words | "Area under the GZ curve, 0° to 30°" |
| `origin` | `regulation`, `class`, `method-validity`, `physical`, `tool-limit`; and, added in schema 1.x for outside users, `guideline`, `national-practice` (#463) | `regulation` |
| `source` | document, edition, paragraph (`locator`), page, `access` | MSC.267(85), IS Code 2008, Part A 2.2.1, p12 |
| `codified` | `{by, date, review}`: who wrote the record (a role) and the pull request | `{by: "agent", date: 2026-10-21, review: "#n"}` |
| `verified` | `{by, date, edition_held, locator_read, review}` or `false` | `{by: "maintainer", date: 2026-10-22, edition_held: "MSC.267(85)", locator_read: "Part A 2.2.1, p12", review: "#n"}` |
| `applies_when` | the Applicability; empty means always | `{tank_head_type: [hemispherical, torispherical]}` |
| `kind` | `limit`, `table`, `formula`, `check`, `note` | `limit` |
| `measure` | Measure id and its arguments | `gz.area` from 0° to 30° |
| `compare` | `ge`, `gt`, `le`, `lt` (none for `check` and `note`) | `ge` |
| `limit` | a constant with unit; a table id; or a formula shape id with coefficients | 0.055 m·rad |
| `tolerance` | absolute, in the limit's unit; default 0 | 0 |
| `consequence` | `block`, `warn`, `note` | `block` |
| `sizes` | `true` when the arrangement search uses the limit | `false` |
| `evaluated` | `true`, or `false` with a `gap` text naming what the engine lacks | `false`, "needs the wind heeling lever" |
| `finding` | issue number, when the engine's value or measure differs from the source | `#282` |
| `since` | the set version that added or last changed the Rule | `1.0.0` |

- **Rule id grammar.** `^[a-z][a-z0-9-]*/[A-Za-z0-9][A-Za-z0-9.()-]*/[a-z0-9][a-z0-9-]*$`. The family is
  the set's `family`. For a `regulation` or `class` Rule the paragraph segment is the source locator
  with spaces removed (`A.2.2.1`, `9.3.1.11.1(b)`), and the loader checks that they agree. For an OVCD
  Check it is a topic (`gas-arrangement`). The slug says which requirement of the paragraph. Ids are
  unique within a set. An id keeps its Measure and direction in every later version of its set; a
  change of either is a new id, and the old id is never used again.
- **Kinds.** `limit`: a Measure against a constant. `table`: the limit is a row of a table of steps or
  piecewise rows. `formula`: the limit comes from a formula shape coded in Rust with coefficients in the
  file (d = a + Vc × b / c). The shapes are `less-input` (`of − less`), `clamp`
  (`max(lo, min(hi, a + x × b / c))`, "a + b·x, the lesser of that and hi, not less than lo", as MARPOL
  Annex I Reg. 19.3.1 and 19.3.2 write the double-hull widths) and `at-least` (`max(lo, a + x × b / c)`,
  Reg. 19.6). The operand `x` of `clamp` and `at-least` may be in any unit: the coefficients convert it. `check`: a yes/no Measure. `note`: a cited value shown in the trace, with
  no test.
- **Formula shapes keep the code's arithmetic.** A shape evaluates in the same order of operations as
  the code it replaces (`a + v * b / c`, not a rewritten interpolation), so results stay bit-identical.
  Rewriting a shape is a later, declared design change.
- **Who is named in `codified.by` and `verified.by`.** A role (`maintainer`, `expert-reviewer`,
  `agent`), never a person's name: the repo is public. The `review` link (the pull request) shows who.
- **Who may verify.** Any second reader, human or agent, who did not codify the Rule and who reads the
  cited paragraph in the cited edition (maintainer decision, 2026-10-20). The role in `verified.by`
  shows which kind of reader it was. The codification process is in `docs/rules/codification.md`.
- **Paywalled texts.** `source.access` is `free`, `paywalled` or `sign-up`. For a paywalled text the file
  holds numbers, tables and the paragraph, never the prose.
- **Sourced** means `origin` is a published one (`regulation`, `class`, `guideline` or
  `national-practice`) **and** `verified` is filled. Nothing else is ever shown as a published rule.
  A `guideline` is a design guideline that says it is not a standard; `national-practice` is a
  national standard known through another text (a guideline's survey, say). OVCD's sets use neither. A sourced Rule whose Measure method is `unverified` is shown as
  sourced with an unverified method: both statuses are in the trace.

### Rule Set (data)

| Field | Meaning |
|---|---|
| `id`, `family`, `title` | `igc-2g`, `igc`, "IGC Code as amended, ship type 2G" |
| `version` | semver of OVCD's encoding (`1.0.0`), rules below |
| `edition` | the source edition: document, edition, amendments included, in-force date |
| `sources` | each text read: document, edition, URL when free, access, SHA-256 of the file read |
| `status` | `draft` or `stable` (stable needs full harness coverage, below) |
| `applies_when` | the Applicability of the whole set |
| `includes` | other sets this one requires, with version (IGC 2.2.2 requires the IS Code: `igc-2g` includes `is2008-a`) |
| `known_conflicts` | pairs of Rule ids that cannot both be met, each with a reason (normally empty) |
| `tables` | the step and piecewise tables its Rules name |
| `rules` | the Rules, in display order |

- **Content hash.** SHA-256 of the canonical JSON of the fields that change a result: rule ids,
  `applies_when` (set and Rule), `kind`, `measure`, `compare`, `limit`, `tolerance`, `consequence`,
  `sizes`, `evaluated`, `tables`, `includes`. Titles, citations, `codified` and `verified` are left out,
  so recording a verification does not trip the warning, and changing a limit does.
- **Version numbers.** A new version with the same content hash is a **patch** (citations,
  verification, titles). One that only adds Rules is a **minor** version. One that changes the
  result-changing fields of an existing Rule, or removes one, is a **major** version. CI checks the
  class against the hashes (#273). Two versions with the same content hash are equivalent: opening a
  file stored with the older one gives no "newer version" warning.
- **Files.** One file per set version: `crates/ovcd-format/rules/<set-id>/<version>.json`, with a JSON
  Schema beside them (`rules/rule-set.schema.json`). The built-in sets are embedded with `include_str!`,
  as `builtin_profile.json` is, so the WebAssembly build has them.
- **Every released version is kept.** `rules/RELEASED.json` lists each released set version and its
  content hash. CI fails if a listed file changes. A change is a new file with a new version. A set
  version is released when a tagged release ships it.

### The shared kernel (`arc-rules`, #463, ADR 0019)

The model, the schema, the loader, the content hash, the library, Applicability and the evaluator are
a domain-free crate, `arc-rules`, that a second tool shares. OVCD registers its side with it:

- **Object types are string keys** (`option`, `hull`, `tank`, `loading-condition`, `cargo-tank`,
  `ballast-tank`, `slop-tank`). The caller declares them; `option` is OVCD's design object.
- **The caller passes in** the Measure registry, the applicability vocabulary and the formula shapes
  (`less-input`, `clamp`, `at-least`, `max-capped-less`, `lesser-ratio` are OVCD's). The table row
  shapes `const`, `a+v*b/c` and `a+v/c` are the kernel's.
- **Object-scoped inputs.** The vocabulary marks each input `design` or `object`. A Rule's
  Applicability condition on an object input is read for each object of the Rule's type
  (`MeasureSource::object_input`, by default the design's value); an object that does not meet it has
  a `not applicable` row. A set's own Applicability names only design inputs. Every OVCD input is a
  design input, so no OVCD result changes.
- **Rule Set schema 1.x is frozen for outside users.** A change is additive only: a new optional field,
  a new enumeration value, a new shape the caller registers. A breaking change needs schema 2.0 and a
  maintainer decision. The schema file is the kernel's (`crates/arc-rules/rule-set.schema.json`); the
  copy beside OVCD's rule files must stay the same file (a test checks it).

### Loading at run time

- The loader (in the `arc-rules` kernel; OVCD gives it its vocabulary from `ovcd-format`) reads a set file, checks it against the schema, and refuses it with a clear
  message when: a field is missing; a rule id is malformed, repeated, or its paragraph segment does not
  match its locator; a Measure id is unknown, its arguments are wrong, or its unit does not match the
  limit; a table or formula shape is unknown; a limit or coefficient is not finite; a `check` Rule names
  a non-`bool` Measure; an `includes` target is missing; a Rule of a published origin has no
  paragraph; two constant Criteria of the set conflict and are not declared (Coherence, above). It warns
  (does not refuse) when a blocking Rule is `unverified`.
- After loading, the **selection** is checked as a whole (Selection, above): include cycles, two versions
  of one set, and one rule id with two meanings are refused; conflicts across sets are warned.
- The core gets plain typed structs, never JSON. The core stays `no_std` and parses nothing.
- **Order of loading.** The built-in sets first, then each `--rule-set <file>` in the order given. A
  loaded set with the id and version of a built-in set and the same hash is the same set; with a
  different hash it is refused (a changed set needs a new version). A loaded set with a new version of a
  built-in id is a separate version: it is used only where a Design or a flag selects that version.
  Nothing loaded at run time replaces a built-in version silently.
- `ovcd run`, `ovcd fill` and `ovcd validate` take `--rule-set <file>` (repeatable), as they take
  `--profile`. A loaded set is added to the stack of the run only (it is not written to the file);
  `--rule-set` with the id of a set in the stack and a new version replaces that set for the run, and the
  output says so. A loaded file also supplies a stored version the engine lacks.
- `ovcd rules` lists the built-in sets and versions; `ovcd rules show <id>[@version]` prints one set
  with its Rules and verification status; `ovcd rules --measures` prints the Measure registry.
- The Design session API (used by the app and, later, the web app) takes the same loaded sets.

### Composition: strictest governs

- Every Rule of every selected set is evaluated, sets pulled in by `includes` too, in selection order.
- Rules in one Requirement all apply. The Requirement is met only if every Rule in it is met. The
  stricter Criterion governs (Domain model, above). No Rule loosens another. There is no override field.
- A set whose `applies_when` does not match the Option gives one trace row, `not applicable`, for the
  set as a whole.

### Rule Trace (in every result)

One row per Rule per object, in selection order, then the set's rule order, then object order:

| Field | Example |
|---|---|
| `rule` | `is2008/A.2.2.1/area-0-30` |
| `rule_sets` | `["is2008-a@1.0.0"]` (more than one when sets share the Rule) |
| `citation` | IS Code 2008, MSC.267(85), Part A 2.2.1 |
| `origin` | `regulation` |
| `rule_status` | `verified` or `unverified` (the record is in the set) |
| `method_status` | the Measure's method status |
| `object` | loading condition "Laden" |
| `inputs` | each input used: name, value, unit, and where it came from (`typed`, `default`, `default-estimate`, `class`, `computed`) |
| `value` | 0.071 m·rad |
| `limit`, `compare`, `tolerance` | 0.055 m·rad, `ge`, 0 |
| `margin` | +0.016 m·rad (Domain model, above) |
| `outcome` | `pass`, `fail`, `warn`, `note`, `not applicable`, `not evaluated` |
| `governing` | `true` for the governing row of its Requirement |
| `sizes` | `true` when the search used its limit |
| `gap` | for `not evaluated`, and for a Measure that failed: the reason |
| `finding` | the finding issue, when one is linked |

- **Every Rule of a selected set appears**: exactly one row per object for each Rule of an applicable
  set (one `not applicable` row when the Option has no object of its type), and one row for each set
  that does not apply. The `limitations` text of `StabilityAssessment` ("no wind heel, no damage
  stability") becomes `not evaluated` rows.
- **The trace is outside the results hash**, as the `drawing` object is (ADR 0007). Every result number
  is already hashed in its own field, and the stored Rule Set hashes cover the rules. So adding the trace
  changes no hash, and a rule change is reported as a rule change, not as "results differ".
- The existing `checks` list stays for now, so nothing that reads it breaks. Each check reports one
  Rule. The `checks` list is in the results hash, so the Rule id is not written into each check (that
  would move every saved hash): the table `CHECK_RULES` in `crates/ovcd-format/src/trace.rs` holds the
  link, and a test over the fixture Designs keeps it complete (#272). Removing the duplicate list is a
  later change.
- A blocking failure is a row with outcome `fail`. The Option's `ok` is unchanged in meaning: no
  blocking Rule fails.
- **A row added in the migration may not change `ok`.** Where a new Rule would fail a design that passes
  today (a Rule the code never checked), it enters version `1.0.0` with consequence `warn` and a linked
  finding. Making it `block` is a design change, decided on the finding.

### Service Area and default stacks

- New input `service_area`: `river-only` or `coastal-and-river`. `ocean-only` is not offered (ADR 0011;
  a new ADR is needed). Stored as used, marked typed or default. The Defaults Profile default is
  `river-only` (confirmed by the maintainer, 2026-10-16).
- New input `rule_sets`: the list of selected sets, each `{id, version, hash, source}` where `source` is
  `default` (from the Service Area) or `typed` (added by the user). The `ovcd` set is always selected and
  cannot be removed. A set required by `includes` cannot be removed while the set that includes it is
  selected.
- **Since format version 3** (#274, with the ADR 0012 hull block) an Option stores `service_area` and
  `rule_sets`, and the run checks the stored sets at the stored versions.
- **Where the stacks live (#275).** In the Defaults Profile, input `rule_sets`, key `default_stacks`:
  per Service Area, a `gas` list (a Type C tank Cargo Containment), an `other` list and, since #339, a
  `liquid_bulk` list (the `integral_tanks` Cargo Containment; a profile without it takes `other` with
  `adn-tank`, and `marpol-i` on coastal-and-river, put in before `ovcd`) of set ids. A new
  Option takes each set at its highest built-in version. A profile copy may change them; the `ovcd` set
  must be in each. Changing the Service Area or the Cargo resets the `default` entries of `rule_sets` and
  keeps the `typed` ones (a typed entry takes the place of the default one with its id).
- **Default stacks built in this spec.** The sets that exist after this spec are the same for both
  Service Areas. The input still goes in now, so files record where the vessel operates, and the stacks
  can diverge as sets land, with no further format change.

| Cargo | `river-only` and `coastal-and-river` (this spec) |
|---|---|
| Gas | `igc-2g`, which includes `is2008-a`; `lr-inland-gas` (draft, unverified, until the IGC chapter 3 rules replace it); `ovcd` |
| Dry bulk and others | `is2008-a` (its rows are `not evaluated` until intact stability covers every Cargo, roadmap phase 1); `ovcd` |
| Liquid bulk (crude oil, #339) | the dry bulk stack with `adn-tank` (any Service Area), and on `coastal-and-river` also `marpol-i` and the freeboard sets (`icll-freeboard`, `solas-v22`); `normam-202` is never in a default stack; `ovcd` |

- **Target stacks (later specs, not built here).** River only adds an inland class stability set (BV
  NR217 Pt B Ch 2 Sec 2 [4.1], which has the same three numbers as ADN 9.3.1.14.2) and ES-TRIN 2025/1
  Chapter 4 freeboard and safety clearance as a reference. Coastal and river adds IS Code 2008 Part B
  2.2 (pontoons) for barges, which class points cargo barges of 24 m or more to (ABS Steel Barges
  3-6-1/3.1), and ICLL freeboard. ADN 2025 becomes an optional set for European inland service; when it
  is selected with IGC, both apply and the strictest governs.
- **Scope wording.** Every result names its Service Area and its sets, and the count of Blocking Rules
  not evaluated. A river-only result says that no statutory technical code was checked, only the named
  Rule Sets (ADR 0011).

### How `fit` uses it (#264)

- #264 has landed with its own check list (one list in `gas_arrangement.rs`). #277 replaces that list:
  `fit`, `smallest_viable_loa` and the design sweep evaluate each candidate's Rule Trace and keep only
  candidates with no `fail` row. Among passing candidates the most cargo wins; on equal cargo, the first
  in the search order wins (as today). If none passes, the result is `feasible: false` with the best
  candidate's `fail` rows.
- **The selected list.** `fit` uses the Rules of the Option's selection: the default stack, plus any set
  loaded with `--rule-set`. `warn` and `note` Rules never remove a candidate. (Whether the wheelhouse
  sightline should block is a separate maintainer question on #264.)
- **Sizing Rules** give the search its clearances: the governing limit of each `sizes` Requirement, per
  candidate (d(Vc) depends on the candidate's tank volume).
- **Guarantees** (property tests, and ADR 0014): (1) `fit` never returns `feasible: true` for a
  candidate with a `fail` row; (2) for a fixed list of candidates, adding a Rule never raises the cargo
  of the answer and never turns `feasible: false` into `true`. When a new `sizes` Rule changes the
  candidates themselves, (2) holds only through (1): the answer still meets every Rule.
- Cheap Rules (dimensions, clearances) are evaluated before expensive ones (stability) to prune. The
  order of evaluation never changes the answer: a test runs the search with and without pruning.
- `feasible: true` carries the not-evaluated count, as every passing verdict does.

### How the evaluation harness uses it (#184, roadmap T5)

- **Keyed by rule id.** A case's `expected` entries are keyed by rule id (today's keys `area_0_30` and
  so on map to `is2008/A.2.2.1/area-0-30` and so on). The runner reads the Rule Trace from the engine and
  compares row by row, keyed by rule id and object. The first row whose outcome differs names the rule.
- **What counts as a false pass.** A case expects `fail` on a Blocking Rule and the engine's row says
  `pass`. A `not evaluated` row against an expected `fail` is not a pass: it goes to `model-gap`. A
  Measure that failed gives `fail`, never `pass`, so it cannot be a false pass.
- **Boundary cases per Rule.** Each new evaluated Rule comes with harness cases just inside and just
  outside its limit (the codification process, `docs/rules/codification.md`), written by the oracle, not
  by hand.
- **Automatic triage.** Each disagreement is put in one bucket: `model-gap` (the expected Rule is `not
  evaluated`, or the case needs an input the engine lacks); `code-bug` (the independent oracle and the
  engine disagree); `rule-suspect` (oracle and engine agree with each other but not with a `published`
  verdict); `known-limitation` (the Rule links a finding); `needs-expert` (the rest). `results.md` lists
  the buckets. A false pass on a blocking Rule still fails the build.
- **Coverage by rule id.** `results.md` lists, per Rule Set, each Rule with its number of passing and
  failing cases. A set can be `stable` only when every blocking Rule has at least one of each and none is
  `unverified`.
- **Rule mutation (T5.6).** For each numeric limit and coefficient of a selected set, the harness writes
  a mutated copy of the set (the value moved by a step to each side), loads it at run time, and runs the
  corpus. A value that moves without any case changing its verdict is reported as a weak corpus. Warnings
  only. It runs in the pull-request gate while it stays inside the 5-minute budget, and at the release
  gate otherwise.

### Migration of today's rules, with no result change

Every step is one pull request with an empty design-impact table and identical results hashes. A step
that cannot keep them is split or stopped. The audit dispositions decide where each item goes:

| Audit disposition | Where it goes |
|---|---|
| Cited, and the value matches the text | A Rule in its set, `verified`, with the locator read |
| Cited, but the value or measure differs | A Rule that keeps today's value, `unverified`, linked to a finding. The finding is decided later, one at a time, as a design change |
| Design default or tool limit that is an input | Stays an input with its basis in the Defaults Profile. Not a Rule |
| Physical feasibility check or tool limit that is a check | A Rule in the `ovcd` set with origin `physical` or `tool-limit`. Shown as an OVCD check, never as sourced |
| No checkable source | `unverified`: a method validity range is a Rule in `ovcd` with origin `method-validity`; a formula or coefficient is a Measure whose method is unverified; a material strength stays an input with an unverified basis |

The regulation and class items, by set (paragraphs as read in the free texts; see Appendix A):

- **`is2008-a`** (MSC.267(85), Part A): 2.2.1 area 0–30° ≥ 0.055 m·rad, 0–40° ≥ 0.090, 30–40° ≥ 0.030;
  2.2.2 GZ ≥ 0.20 m at 30° or more; 2.2.3 angle of maximum GZ ≥ 25°; 2.2.4 GM0 ≥ 0.15 m. The 0–40° and
  30–40° Rules are verified, but the Measure runs to 40° always, while the text says "or the angle of
  down-flooding if less": linked to a finding. Weather criterion 2.3: `not evaluated` (needs the wind
  heeling lever).
- **`igc-2g`** (MSC.370(93) as amended to MSC.566(109); no amendment changes 2.3 or 2.4): d(Vc) of
  2.4.1.1 per tank as the set's table `igc-d-vc` (four branches; a formula, not a Rule: for type 2G the
  requirement is stated in 2.4.1.2); 2.4.1.2 side clearance ≥ d, a `sizes` Rule on the clear
  gap measured as in 2.4.2 (plate to plate, insulation inside d); 2.3.1.2.3 with 2.4.1.2 bottom clearance
  max(min(B/15, 2 m), d); 4.23.2.1.2 minimum wall thickness after forming (5 mm for carbon-manganese and
  nickel steels, 3 mm for austenitic steels, 7 mm for aluminium alloys), a `table` limit on
  `tank.wall_thickness` keyed by the steel class; 4.23.2.1.3 weld factor, an upper bound 0.95 (`warn`:
  the text lets it rise to 1 on other considerations) and an upper bound 1 (`block`) on
  `tank.weld_factor` (the factor the wall formula used; it is not a design input); 4.23.3.1 allowable
  stress, an upper bound on `tank.allowable_stress` (the f the wall formula used) by the formula shape
  "lesser of Rm/A and Re/B" (`lesser-ratio`) with A and B per steel
  class as data (A 3, B 1.5 for nickel and carbon-manganese steels; A 3.5, B 1.5 for austenitic steels;
  A 4, B 1.5 for aluminium alloys). The material strengths Rm and Re are now cited in the material data
  (#313; the Rule still marks them unverified until a second reader checks them). The S690QL1 default was
  285 MPa (Rm 770 / 2.7), above Rm/3 = 256.7 MPa (the finding of #292); since #313 a new Design takes
  256.7 MPa, and only a file saved before keeps 285 MPa (`tank_allowable_stress_basis` =
  `published_design`), so this Rule enters `1.0.0` as `warn` for those files. 304L, 316L and 9Ni meet it.
  Damage survival 2.7: `not evaluated`. The filling limit of 15.3 stays the Defaults Profile basis of the
  `fill_ratio` input. Since #315 the engine loads an LPG Cargo to the loading limit of 15.5.1 (LL = FL ×
  ρR / ρL) with the cargo densities by carriage condition; `igc/15.5.1/loading-limit` as a Rule that
  checks it is later.
- **`lr-inland-gas`** (Lloyd's Register, Rules and Regulations for the Classification of Inland
  Waterways Ships, Pt 4 Ch 4 Sec 3, July 2022 edition as cited by the code; draft): opening setback 2.00 m (3.3.7, 3.3.11; same value in ADN 9.3.1.17.4), cited and
  matching; cofferdam 0.60 m (3.6.13 is a type C clause, not a gas tank clause) and end-bulkhead space
  0.50 m / 0.20 m (the text's condition is plane or domed tank ends, not "standard" or "pressure"):
  kept as they are, `unverified`, each linked to a finding. These three Rules keep the code's tolerance
  of 1e-9 m. The set stays in the default gas stack, marked unverified, until the IGC chapter 3 rules on
  segregation of the cargo area replace it (maintainer, 2026-10-16).
- **`ovcd`**: the tank length-over-diameter limit 7.0 (a tool limit; its origin is ADN 9.3.1.11.1(b),
  which applies only if an ADN set is selected later), physical feasibility checks (draught below depth,
  keel clears the bed, air draft), method validity ranges (squat Froude depth number below 0.7, blockage
  at most 0.35), the hopper GM warning, and the other checks of the ten modules.

The four remaining "value differs" items are filed as findings under #121: the cofferdam clause (#280),
the end-bulkhead condition (#281), the 0–40° area without a down-flooding angle (#282), and the 1.0 m
"standard IGC-2G spacing" of `design_grid` (#283; no such value in IGC 2.4.1.2).

### File format (ADR 0007): version 3

- An Option gains `service_area` and `rule_sets` (above). The `ruleset` input is removed.
- Reading version 1 or 2: `ruleset: IGC_2G` (and the names #175 and #237 already map to it) becomes the
  default stack for the Option's Cargo, each set at its highest built-in version, marked `default`;
  `service_area` becomes `river-only`, marked `default`. One warning per Option. Results cannot change,
  because that is the stack the engine checked every Option against. (The first draft said version
  `1.0.0`; `ovcd` and `lr-inland-gas` have since moved past it, and a stack at `1.0.0` would not be today's
  rules.) Both new inputs join the one "fixed value the engine used before" warning of the Option.
- Opening a file whose stored set version is built in: that version is used. A newer version exists: a
  warning, never a silent swap. A stored hash that differs from the built-in hash of the same version: a
  warning that the rules differ from the file. A stored version the engine does not have and no
  `--rule-set` supplies: the Option is refused with a message naming the set.
- **Shared with the ADR 0012 hull block.** Version 3 holds both. Whichever lands first opens version 3;
  the rules-engine fields wait for the hull block (maintainer decision 3), so version 3 ships once,
  with both.
- `ovcd schema` lists the new inputs and the Rule Set schema.

### App

- **Rule check calculations tab** (`app/src/lib/views/RuleChecks.svelte`): one row per trace row, with
  columns Result, Rule (title and id), Object, Value, Limit, Margin, Citation, Verified. `not evaluated`,
  `not applicable` and `warn` rows have their own tags (word and shape, not colour only). The governing
  Rule is marked. Failures first, then warnings, then passes, then not evaluated. The app computes
  nothing: every field comes from the trace.
- **Inputs panel.** A Service Area dropdown and a read-only list of the selected sets with version,
  status and the count of unverified Rules. Adding, removing or loading a set in the app is later; the
  CLI and the session API do it in this spec.
- The app's lanes (restyle #248, web app #244) touch the same files. The app ticket starts after #252.

### Testing Decisions

- **A good test checks behaviour through a public seam:** `ovcd run --json`, the Design session, the
  loader, the harness runner. Not the private layout of the evaluator.
- **No result change is the main test of the migration.** Each migration ticket shows an empty
  design-impact table and unchanged fixture hashes (`tools/check_fixture_hashes.py`). No snapshot is
  re-pinned in a migration ticket.
- **Loader:** each refusal reason has a test with a minimal bad file. The schema and the loader agree
  (every schema example loads; every refused example fails the schema or a named loader check).
- **Evaluator:** sourced tests for every Rule from its paragraph (the limit and comparison at both sides
  of the boundary, worked by hand and cited). Property tests (`proptest`, in the fast gate, bounded
  cases): the Margin and the outcome agree (Domain model); every Rule of a selected set appears exactly
  once per object; the trace is the same on every platform and in WebAssembly; a Measure failure never
  gives `pass`; the governing row does not change when only the Measure value changes.
- **Strictest governs:** a test with two run-time-loaded fixture sets that cover one requirement with
  different limits: the stricter governs, both rows appear, and loosening is impossible. Property tests:
  a value meets every Criterion of a Requirement exactly when it meets the governing one; adding a Rule
  never turns a failing Option into a passing one.
- **Selection and coherence:** tests for an include cycle, two versions of one set, one rule id with two
  meanings, an undeclared in-set conflict (refused) and a cross-set conflict (warned). The defaults
  coherence test runs every default combination of the profile through the input-bounding Rules.
- **Formal model (ADR 0014):** the Lean model of the composition kernel is checked in a separate CI job,
  never in the fast gate. A differential test feeds the same random cases to the Rust evaluator and to
  the Lean oracle and compares the verdicts and the governing rows. It runs with the Lean job.
- **Released versions:** CI fails if a file listed in `RELEASED.json` changes, or if a new version's
  number does not match its class (patch, minor, major) by the content hashes.
- **Provenance gate:** CI fails if a `regulation` or `class` Rule has no paragraph, and prints the count
  of unverified blocking Rules per set.
- **Harness:** the runner tests of `tests/test_evaluation_runner.py` extend to rule-id keys, triage
  buckets and rule mutation.
- **Prior art:** the design-impact check and fixture hashes (spec 0001), the Defaults Profile loader and
  `--profile`, `tools/gen_vessel_classes.py` (data reviewed in the repo), the harness (#184).

## Out of Scope

- New rules or rule sets beyond today's (BV NR217, ES-TRIN, IS Code Part B, ICLL, ADN, IGC 1G and 3G):
  each comes as its own data file and harness cases after this spec.
- Applicability Rules beyond the derived bands (who is in or out of SOLAS or ICLL by tonnage or
  propulsion, a free numeric threshold).
- Any fix of a finding. Each is decided on its own issue, as a design change.
- Damage stability, the weather criterion and the down-flooding angle as Measures (roadmap phases 1
  and 2). Here they are only `not evaluated` rows.
- Giving Defaults Profile ranges rule ids. They stay input validation in `ovcd-format` (ADR 0007).
- Adding, removing or loading Rule Sets in the app.
- An expression language, and Rule overrides (ADR 0013).
- The Ocean only Service Area.

## Decisions taken and open questions

Answered by the maintainer:

1. **The gas separation clauses** (2026-10-16): they move unchanged into the draft set `lr-inland-gas`,
   marked unverified; a follow-up replaces them with the IGC chapter 3 rules on segregation of the cargo
   area, cited against the text.
2. **Default Service Area** (2026-10-16): `river-only`.
3. **Format version** (2026-10-16): one shared `.vdesign` bump (version 3) carries the rules-engine fields
   and the ADR 0012 hull block; the rules-engine format change waits for the hull block (#274). The
   application stays on 0.2.x.
4. **Who verifies** (2026-10-20): any second reader, human or agent, who did not codify the Rule.
5. **Not-evaluated Blocking Rules** (2026-10-20): the Option and `fit` still pass, and every passing
   verdict states the count and names of Blocking Rules not evaluated.

Open: see "Decisions for the maintainer" in the pull request that deepened this spec, and the
recommendations on #280 to #283 and #292.

## Further Notes

**Tickets, in order** (sub-issues of #267, "Spec 0004: Rules engine", with native blocked-by links):

1. #268 Rule Set file format, schema and loader. No engine change.
2. #269 Measure registry, evaluator and Rule Trace in the results (outside the hash). Needs 1.
3. #270 `is2008-a` 1.0.0 as data. Needs 2.
4. #271 `igc-2g` 1.0.0 as data. Needs 2.
5. #272 `ovcd` and `lr-inland-gas` 1.0.0: the remaining checks, with honest origins; uncited Rules marked
   unverified; findings filed. Needs 2.
6. #273 Released versions kept and the provenance gate in CI. Needs 3, 4, 5.
7. #274 `.vdesign` version 3: `service_area` and `rule_sets`, migration from `ruleset`. Needs 3, 4, 5.
8. #275 Service Area input and default stacks; scope wording. Needs 7.
9. #276 The Rule check calculations tab shows the trace. Needs 3, 4, 5 and #252.
10. #277 `fit` keeps only candidates that meet every blocking Rule. Needs 4, 5 and #264.
11. #278 Harness keyed by rule id, with automatic triage. Needs 3.
12. #279 Harness rule mutation. Needs 11 and 1.

Tickets 1 to 6 and 11 change no result. Tickets 3, 4 and 5 can run in any order after 2. Ticket 10
moves results only through #264. Tickets 7 and 8 wait for the hull geometry block (ADR 0012).

**Related documents.** The rule codification process and the rule record template:
`docs/rules/codification.md`. The decision on formal proofs: ADR 0014.

**Design review, 2026-10-20.** The first draft left these points open or in conflict; the Domain model
section now settles them:

1. The comparison had no tolerance, but `gas_separation` compares with 1e-9 and `gas_stability` with
   none. A Rule now has a `tolerance`, in the hash, so results stay bit-identical.
2. "The same requirement" did not say whether `gz.area(0, 30)` and `gz.area(0, 40)` are one
   requirement. A Requirement is now keyed by Measure instance (with its arguments), object and
   direction.
3. "Smallest margin governs" depends on the value, so the governing Rule could change as the design
   changes, and ties had no rule. Governing is now the strictest Criterion, with a fixed tie order.
4. `parameter` Rules (the weld factor) had no meaning under "strictest governs". They are now Rules on
   input Measures with a normal Criterion.
5. `size` was both an outcome and a use. It is now a `sizes` flag; the outcome is pass or fail.
6. Applicability existed only on sets, but #281 needs a Rule keyed to the tank end shape. Rules now have
   `applies_when`, in the same restricted form as sets.
7. A Measure that cannot be computed had no outcome. It is now `fail` (or `warn`), never `pass` and
   never `not evaluated`.
8. The rule id form had no grammar, no link to the locator and no rule for one id in two sets (an IGC
   Rule shared by 1G and 2G sets). All three are now defined.
9. Selecting two versions of one set, an include cycle, and a conflict between two Rules had no rule.
   They are now refused or warned (Selection, Coherence).
10. Recording a verification needs a new file (released files never change), which would have raised
    "newer version" warnings for no change. Versions with the same content hash are now equivalent, and
    semver classes are defined by the hash.
11. A Rule could be verified while its Measure's method is not. The trace now shows both statuses.
12. The glossary said a Rule comes from a published source, while OVCD Checks are Rules in the `ovcd`
    set. The glossary now says an OVCD Check is a Rule with OVCD's own origin.
13. "`fit` must meet every selected rule" could not hold for `not evaluated` Rules. The verdict now
    names them (maintainer decision 5).
14. Defaults were never checked against the Rules that bound them. A defaults coherence test now does,
    and it finds the S690QL1 allowable stress (see #292; the default is fixed by #313).
15. The first draft named a Rule `igc/2.4.1.1/d-vc`. d(Vc) is a formula, not a requirement; for type 2G
    the requirement is in 2.4.1.2. d(Vc) is now the set's table `igc-d-vc`, cited to 2.4.1.1.

## Appendix A: regulation and class items in today's code

Item names are the code's; paragraph numbers are from the free texts (IMO resolutions MSC.267(85),
MSC.370(93) and its amendments; ADN 2025; the inland class rule book's 2015 edition, which has the same
clauses). "Fixed" means a merged change already did what the disposition asks.

| Code item | Value | Target Rule | Disposition |
|---|---|---|---|
| `protective_location::d_vc_exact` | 0.8; 0.75 + V·0.2/4000; 0.8 + V/25000; 2.0 m (V of one tank) | table `igc-d-vc` (2.4.1.1), used by `igc/2.4.1.2/side-clearance` | Cited, matches |
| stepped `distance_d_vc` (total volume) | 0.8 / 1.0 / 2.0 m | — | Value differed. Fixed: removed (#237) |
| side clearance, 2G | d − insulation | `igc/2.4.1.2/side-clearance` | Cited, matches (2.4.2 datum) |
| side clearance, 1G and IGF | max(min(B/5, 11.5), d) | later set `igc-1g` | Cited, matches; not reachable from a `.vdesign` today |
| side clearance, 3G | 0.8 m | later set `igc-3g` | Cited, matches; not reachable today |
| bottom clearance, all types | max(min(B/15, 2.0), d) | `igc/2.4.1.2/bottom-clearance` (2.4.1.2 states the 2G requirement; 2.3.1.2.3 gives the extent) | Cited, matches |
| Vc includes the dome | per-tank gross volume | Measure `tank.vc` | Cited, matches. Fixed: dome added (#237) |
| `pressure_vessel` minimum thickness | 3 / 3 / 5 mm by material | `igc/4.23.2.1.2/min-thickness` | Cited, matches |
| `DEFAULT_WELD_EFFICIENCY` | 0.95 | `igc/4.23.2.1.3/weld-factor` | Cited, matches |
| allowable stress f | lesser of Rm/A, Re/B | `igc/4.23.3.1/allowable-stress` | Formula verified; material strengths cited (#313), unverified by a second reader. Fixed: S690QL1 default 256.7 MPa (#313) |
| shell and head thickness formulas | t = PR/(fE − 0.6P) and two head forms | Measure `tank.wall_thickness` | No checkable source (paywalled pressure-vessel code): unverified |
| `gas_stability` area 0–30° | ≥ 0.055 m·rad | `is2008/A.2.2.1/area-0-30` | Cited, matches |
| `gas_stability` area 0–40° | ≥ 0.090 m·rad, always to 40° | `is2008/A.2.2.1/area-0-40` | Cited, measure differs (no down-flooding angle): finding |
| `gas_stability` area 30–40° | ≥ 0.030 m·rad | `is2008/A.2.2.1/area-30-40` | Cited, matches (same down-flooding note) |
| `gas_stability` GZ at 30° or more | ≥ 0.20 m | `is2008/A.2.2.2/gz-30` | Cited, matches |
| `gas_stability` angle of max GZ | ≥ 25° | `is2008/A.2.2.3/angle-max-gz` | Cited, matches |
| `gas_stability` GM0 | ≥ 0.15 m | `is2008/A.2.2.4/gm0` | Cited, matches |
| `gas_stability` limitations text | no wind heel, no damage | `is2008/A.2.3/weather`, `igc/2.7/damage-survival` | `not evaluated` rows |
| `OPENING_SETBACK_M` | 2.00 m | `lr-inland/3.3.7/opening-setback` | Cited, matches (also ADN 9.3.1.17.4) |
| `COFFERDAM_MIN_M` | 0.60 m | `lr-inland/3.6.13/cofferdam` | Cited, value differs (a type C clause): finding |
| `END_BULKHEAD_SPACE_M` | 0.50 m | `lr-inland/3.5.3/end-space-plane` | Cited, condition differs (plane ends, not "standard"): finding |
| `END_BULKHEAD_SPACE_PRESSURE_M` | 0.20 m | `lr-inland/3.5.3/end-space-domed` | Cited, condition differs (same finding) |
| inland type G side and double-bottom floors | 0.80 m, 0.60 m | — | Measure differed. Fixed: removed (#215) |
| `design_grid::SIDE_CLEARANCE_M` | 1.0 m "standard IGC-2G spacing" | input default, not a Rule | Cited, value differs (IGC has no 1.0 m standard): finding |
| `MAX_TANK_LENGTH_OVER_OD` | 7.0 | `ovcd/gas-arrangement/tank-l-over-d` | Tool limit, origin recorded (ADN 9.3.1.11.1(b)) |
| `fill_ratio` default | 0.98 | input with basis IGC 15.3 | Cited, matches |
