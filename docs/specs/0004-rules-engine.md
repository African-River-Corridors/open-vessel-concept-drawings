# Spec 0004: The rules engine

Status: **draft, for the maintainer's approval.** Decision record: ADR 0013. Issue: #176.

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

The glossary holds the terms: **Rule**, **Rule Set**, **Measure**, **Rule Trace**, **Margin**, **Service
Area**, **Governing Rule**, **OVCD Check**. The IS Code calls its requirements "criteria"; titles may say
criterion, but the term is Rule. The `Criterion` struct of `gas_stability` becomes a trace row in code.

### Measure (code)

- A named Rust function in `ovcd-core` that computes one quantity for one object. It has a stable id
  (`gz.area`, `gz.at_angle`, `gz.angle_of_max`, `gm0`, `tank.side_gap`, `tank.bottom_gap`,
  `tank.vc`, `tank.wall_thickness`, `squat.froude_depth`, ...), a unit, the object it applies to
  (Option, tank, loading condition, Hull), and its own tests.
- It cites a **method**, not a rule ("trapezoid area under the GZ curve on a 1° grid"). A method from a
  source not yet in the library is `unverified` (the ASME pressure-vessel formulas, the squat and
  resistance methods), and the trace says so.
- The core keeps a registry of Measure ids with unit and object. `ovcd rules --measures` prints it.

### Rule (data)

| Field | Meaning | Example |
|---|---|---|
| `id` | `<family>/<paragraph>/<slug>`, stable, never reused | `is2008/A.2.2.1/area-0-30` |
| `title` | OVCD's own words, at most 15 words | "Area under the GZ curve, 0° to 30°" |
| `origin` | `regulation`, `class`, `method-validity`, `physical`, `tool-limit` | `regulation` |
| `source` | document, edition, paragraph (`locator`), page | MSC.267(85), IS Code 2008, Part A 2.2.1 |
| `verified` | `{by, date, edition_held}` or `false` | `{by: "rule-source review", date: 2026-10-16, edition_held: "MSC.267(85)"}` |
| `applies_to` | object and condition | each loading condition, gas Cargo |
| `kind` | `limit`, `table`, `formula`, `check`, `parameter` | `limit` |
| `measure` | Measure id and its arguments | `gz.area` from 0° to 30° |
| `compare` | `ge`, `gt`, `le`, `lt` (none for `check` and `parameter`) | `ge` |
| `limit` | a constant with unit; a table id; or a formula shape id with coefficients | 0.055 m·rad |
| `outcome` | `fail` (blocking), `warn`, `size`, `note` | `fail` |
| `evaluated` | `true`, or `false` with a `gap` text naming what the engine lacks | `false`, "needs the wind heeling lever" |
| `finding` | issue number, when the engine's value or measure differs from the source | `#n` |
| `since` | the set version that added or last changed the Rule | `1.0.0` |

- **Kinds.** `limit`: a Measure against a constant. `table`: the limit is looked up in a table of steps
  or piecewise rows. `formula`: the limit comes from a formula shape coded in Rust with coefficients in
  the file (d = a + Vc × b / c). `check`: a yes/no condition with no number. `parameter`: a cited number a
  Measure or a sizing step uses (the weld factor E = 0.95 of IGC 4.23.2.1.3).
- **Formula shapes keep the code's arithmetic.** A shape evaluates in the same order of operations as
  the code it replaces (`a + v * b / c`, not a rewritten interpolation), so results stay bit-identical.
  Rewriting a shape is a later, declared design change.
- **Who is named in `verified.by`.** A role ("maintainer", "rule-source review"), never a person's name:
  the repo is public.
- **Paywalled texts.** `source.access` is `free`, `paywalled` or `sign-up`. For a paywalled text the file
  holds numbers, tables and the paragraph, never the prose.
- **Sourced** means `origin` is `regulation` or `class` **and** `verified` is filled. Nothing else is
  ever shown as a published rule.

### Rule Set (data)

| Field | Meaning |
|---|---|
| `id`, `family`, `title` | `igc-2g`, `igc`, "IGC Code as amended, ship type 2G" |
| `version` | semver of OVCD's encoding (`1.0.0`) |
| `edition` | the source edition: document, edition, amendments included, in-force date |
| `sources` | each text read: document, edition, URL when free, access, SHA-256 of the file read |
| `status` | `draft` or `stable` (stable needs full harness coverage, below) |
| `applies_when` | the Options it can apply to (Cargo, Cargo Containment) |
| `includes` | other sets this one requires, with version (IGC 2.2.2 requires the IS Code: `igc-2g` includes `is2008-a`) |
| `tables` | the step and piecewise tables its Rules name |
| `rules` | the Rules |

- **Content hash.** SHA-256 of the canonical JSON of the fields that change a result: rule ids,
  `applies_to`, `kind`, `measure`, `compare`, `limit`, `outcome`, `evaluated`, `tables`, `includes`,
  `applies_when`. Titles, citations and verification are left out, so recording a verification does not
  trip the warning, and changing a limit does.
- **Files.** One file per set version: `crates/ovcd-format/rules/<set-id>/<version>.json`, with a JSON
  Schema beside them (`rules/rule-set.schema.json`). The built-in sets are embedded with `include_str!`,
  as `builtin_profile.json` is, so the WebAssembly build has them.
- **Every released version is kept.** `rules/RELEASED.json` lists each released set version and its
  content hash. CI fails if a listed file changes. A change is a new file with a new version. A set
  version is released when a tagged release ships it.

### Loading at run time

- The loader in `ovcd-format` reads a set file, checks it against the schema, and refuses it with a clear
  message when: a field is missing; a rule id is malformed or repeated; a Measure id is unknown or its
  unit does not match the limit; a table or formula shape is unknown; an `includes` target is missing;
  a `regulation` or `class` Rule has no paragraph. It warns (does not refuse) when a blocking Rule is
  `unverified`.
- The core gets plain typed structs, never JSON. The core stays `no_std` and parses nothing.
- `ovcd run`, `ovcd fill` and `ovcd validate` take `--rule-set <file>` (repeatable), as they take
  `--profile`. A loaded set with the id and version of a built-in set but a different hash is refused:
  a changed set needs a new version.
- `ovcd rules` lists the built-in sets and versions; `ovcd rules show <id>[@version]` prints one set
  with its Rules and verification status; `ovcd rules --measures` prints the Measure registry.
- The Design session API (used by the app and, later, the web app) takes the same loaded sets.

### Composition: strictest governs

- Every Rule of every selected set is evaluated. Sets pulled in by `includes` count as selected.
- Two Rules cover **one requirement** when they compare the same Measure, on the same object, in the
  same direction. All of them apply. The requirement is met only if every one is met. The Rule with the
  smallest margin is marked `governing` in the trace. For `size` Rules (a required clearance), the
  largest required value governs.
- No Rule loosens another. There is no override field.
- A set whose `applies_when` does not match the Option gives one trace row, `not applicable`, for the
  set as a whole.

### Rule Trace (in every result)

One row per Rule per object:

| Field | Example |
|---|---|
| `rule` | `is2008/A.2.2.1/area-0-30` |
| `rule_set` | `is2008-a@1.0.0` |
| `citation` | IS Code 2008, MSC.267(85), Part A 2.2.1 |
| `origin` | `regulation` |
| `verified` | `{by, date, edition_held}` or `false` |
| `object` | loading condition "Laden" |
| `inputs` | each input used: name, value, unit, and where it came from (`typed`, `default`, `class`, `computed`) |
| `value` | 0.071 m·rad |
| `limit`, `compare` | 0.055 m·rad, `ge` |
| `margin` | +0.016 m·rad (signed: positive always passes; for `le`, limit minus value) |
| `outcome` | `pass`, `fail`, `warn`, `size`, `note`, `not applicable`, `not evaluated` |
| `governing` | `true` when it governs its requirement |
| `gap` | for `not evaluated`: what the engine lacks |
| `finding` | the finding issue, when one is linked |

- **Every Rule of a selected set appears**, including those not evaluated. The `limitations` text of
  `StabilityAssessment` ("no wind heel, no damage stability") becomes `not evaluated` rows.
- **The trace is outside the results hash**, as the `drawing` object is (ADR 0007). Every result number
  is already hashed in its own field, and the stored Rule Set hashes cover the rules. So adding the trace
  changes no hash, and a rule change is reported as a rule change, not as "results differ".
- The existing `checks` list stays for now, so nothing that reads it breaks. Each check gains the id of
  the Rule it reports. Removing the duplicate list is a later change.
- A blocking failure is a row with outcome `fail`. The Option's `ok` is unchanged in meaning: no
  blocking Rule fails.

### Service Area and default stacks

- New input `service_area`: `river-only` or `coastal-and-river`. `ocean-only` is not offered (ADR 0011;
  a new ADR is needed). Stored as used, marked typed or default. The Defaults Profile default is
  `river-only`.
- New input `rule_sets`: the list of selected sets, each `{id, version, hash, source}` where `source` is
  `default` (from the Service Area) or `typed` (added by the user). The `ovcd` set is always selected and
  cannot be removed. A set required by `includes` cannot be removed while the set that includes it is
  selected.
- **Default stacks built in this spec.** The sets that exist after this spec are the same for both
  Service Areas. The input still goes in now, so files record where the vessel operates, and the stacks
  can diverge as sets land, with no further format change.

| Cargo | `river-only` and `coastal-and-river` (this spec) |
|---|---|
| Gas | `igc-2g`, which includes `is2008-a`; `lr-inland-gas` (draft, see open question 1); `ovcd` |
| Dry bulk and others | `is2008-a` (its rows are `not evaluated` until intact stability covers every Cargo, roadmap phase 1); `ovcd` |

- **Target stacks (later specs, not built here).** River only adds an inland class stability set (BV
  NR217 Pt B Ch 2 Sec 2 [4.1], which has the same three numbers as ADN 9.3.1.14.2) and ES-TRIN 2025/1
  Chapter 4 freeboard and safety clearance as a reference. Coastal and river adds IS Code 2008 Part B
  2.2 (pontoons) for barges, which class points cargo barges of 24 m or more to (ABS Steel Barges
  3-6-1/3.1), and ICLL freeboard. ADN 2025 becomes an optional set for European inland service; when it
  is selected with IGC, both apply and the strictest governs.
- **Scope wording.** Every result names its Service Area and its sets. A river-only result says that no
  statutory technical code was checked, only the named Rule Sets (ADR 0011).

### How `fit` uses it (#264)

- #264 lands first, with its own list of checks. Then `fit`, `smallest_viable_loa` and the design sweep
  evaluate each candidate's Rule Trace and keep only candidates with no blocking `fail`. `size` Rules give
  the clearances the search uses. Among passing candidates the most cargo wins. If none passes, the
  result is `feasible: false` with the best candidate's failing rows.
- Cheap Rules (dimensions, clearances) are evaluated before expensive ones (stability) to prune.

### How the evaluation harness uses it (#184, roadmap T5)

- **Keyed by rule id.** A case's `expected` entries are keyed by rule id (today's keys `area_0_30` and
  so on map to `is2008/A.2.2.1/area-0-30` and so on). The runner reads the Rule Trace from the engine and
  compares row by row. The first row whose outcome differs names the rule.
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
- **`igc-2g`** (MSC.370(93) as amended to MSC.566(109); no amendment changes 2.3 or 2.4): 2.4.1.1 d(Vc)
  per tank, a `formula` table of four branches; 2.4.1.2 side clearance ≥ d, a `size` Rule on the clear gap
  measured as in 2.4.2 (plate to plate, insulation inside d); 2.3.1.2.3 with 2.4.1.2 bottom clearance
  max(min(B/15, 2 m), d); 4.23.2.1.2 minimum wall thickness by material; 4.23.2.1.3 weld factor 0.95
  (`parameter`); 4.23.3.1 allowable stress factors A and B (`parameter`). Damage survival 2.7: `not
  evaluated`. The filling limit of 15.3 stays the Defaults Profile basis of the `fill_ratio` input.
- **`lr-inland-gas`** (Lloyd's Register, Rules and Regulations for the Classification of Inland
  Waterways Ships, Pt 4 Ch 4 Sec 3, July 2022 edition as cited by the code; draft): opening setback 2.00 m (3.3.7, 3.3.11; same value in ADN 9.3.1.17.4), cited and
  matching; cofferdam 0.60 m (3.6.13 is a type C clause, not a gas tank clause) and end-bulkhead space
  0.50 m / 0.20 m (the text's condition is plane or domed tank ends, not "standard" or "pressure"):
  kept as they are, `unverified`, each linked to a finding.
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
  default stack for the Option's Cargo at version `1.0.0`, marked `default`; `service_area` becomes
  `river-only`, marked `default`. One warning per Option. Results cannot change, because the `1.0.0` sets
  hold today's values.
- Opening a file whose stored set version is built in: that version is used. A newer version exists: a
  warning, never a silent swap. A stored hash that differs from the built-in hash of the same version: a
  warning that the rules differ from the file. A stored version the engine does not have and no
  `--rule-set` supplies: the Option is refused with a message naming the set.
- **Shared with the ADR 0012 hull block.** Version 3 holds both. Whichever lands first opens version 3;
  the second adds to it if no release has shipped version 3 yet. If one has, the maintainer decides
  (see open question 3).
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
  of the boundary, worked by hand and cited). Property tests: margin sign matches the outcome; every Rule
  of a selected set appears exactly once per object; the trace is the same on every platform and in
  WebAssembly.
- **Strictest governs:** a test with two run-time-loaded fixture sets that cover one requirement with
  different limits: the stricter governs, both rows appear, and loosening is impossible.
- **Released versions:** CI fails if a file listed in `RELEASED.json` changes.
- **Provenance gate:** CI fails if a `regulation` or `class` Rule has no paragraph, and prints the count
  of unverified blocking Rules per set.
- **Harness:** the runner tests of `tests/test_evaluation_runner.py` extend to rule-id keys, triage
  buckets and rule mutation.
- **Prior art:** the design-impact check and fixture hashes (spec 0001), the Defaults Profile loader and
  `--profile`, `tools/gen_vessel_classes.py` (data reviewed in the repo), the harness (#184).

## Out of Scope

- New rules or rule sets beyond today's (BV NR217, ES-TRIN, IS Code Part B, ICLL, ADN, IGC 1G and 3G):
  each comes as its own data file and harness cases after this spec.
- Applicability Rules (who is in or out of SOLAS or ICLL by length, tonnage or propulsion).
- Any fix of a finding. Each is decided on its own issue, as a design change.
- Damage stability, the weather criterion and the down-flooding angle as Measures (roadmap phases 1
  and 2). Here they are only `not evaluated` rows.
- Giving Defaults Profile ranges rule ids. They stay input validation in `ovcd-format` (ADR 0007).
- Adding, removing or loading Rule Sets in the app.
- An expression language, and Rule overrides (ADR 0013).
- The Ocean only Service Area.

## Open questions for the maintainer

1. **The gas separation clauses.** They come from an inland class rule book whose current official text
   needs an account, and two of them apply type C (liquid) clauses to gas tanks. For the migration they
   move unchanged into the draft set `lr-inland-gas`, which stays in the default gas stack so no result
   moves. Keep that set in the default stack, replace it with the IGC chapter 3 rules on segregation of
   the cargo area, or remove it?
2. **Default Service Area.** The spec defaults to `river-only`. Confirm.
3. **Format version 3 timing.** If a release must ship version 3 before both blocks are ready, does the
   second block take version 4, or wait?

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
moves results only through #264.

## Appendix A: regulation and class items in today's code

Item names are the code's; paragraph numbers are from the free texts (IMO resolutions MSC.267(85),
MSC.370(93) and its amendments; ADN 2025; the inland class rule book's 2015 edition, which has the same
clauses). "Fixed" means a merged change already did what the disposition asks.

| Code item | Value | Target Rule | Disposition |
|---|---|---|---|
| `protective_location::d_vc_exact` | 0.8; 0.75 + V·0.2/4000; 0.8 + V/25000; 2.0 m (V of one tank) | `igc/2.4.1.1/d-vc` | Cited, matches |
| stepped `distance_d_vc` (total volume) | 0.8 / 1.0 / 2.0 m | — | Value differed. Fixed: removed (#237) |
| side clearance, 2G | d − insulation | `igc/2.4.1.2/side-clearance` | Cited, matches (2.4.2 datum) |
| side clearance, 1G and IGF | max(min(B/5, 11.5), d) | later set `igc-1g` | Cited, matches; not reachable from a `.vdesign` today |
| side clearance, 3G | 0.8 m | later set `igc-3g` | Cited, matches; not reachable today |
| bottom clearance, all types | max(min(B/15, 2.0), d) | `igc/2.3.1.2.3/bottom-clearance` | Cited, matches |
| Vc includes the dome | per-tank gross volume | Measure `tank.vc` | Cited, matches. Fixed: dome added (#237) |
| `pressure_vessel` minimum thickness | 3 / 3 / 5 mm by material | `igc/4.23.2.1.2/min-thickness` | Cited, matches |
| `DEFAULT_WELD_EFFICIENCY` | 0.95 | `igc/4.23.2.1.3/weld-factor` | Cited, matches |
| allowable stress f | lesser of Rm/A, Re/B | `igc/4.23.3.1/allowable-stress` | Formula verified; material strengths unverified |
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
