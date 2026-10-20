# Rules are versioned Rule Sets: cited data over coded Measures

Engineering rules are spread through the core as constants and branches (#176). A stability limit is a
`const`, a tank clearance is a `match` on a ship-type string, and the citation of a class rule sits inside
a message. Nothing lists which rules a Design was checked against. A rule set that was only half built
(`ADN_G`, removed in #175) looked like compliance. A rule that cited the wrong tank type (#215) and a
reading of IGC that the text does not support (#237) were found only by reading the code against the
source texts. ADR 0011 says every result names its rule set, edition and each rule checked with its
citation. The evaluation harness (#184) needs one row per rule to pin a disagreement on one rule.

## Decision

- **A Rule is data.** It is one requirement from one source about one Measure: what is compared, how,
  against which limit, and what happens if it is not met. Limits, coefficients, tables and citations are
  data in a reviewable file. A reviewer with the rule book open checks a number against a page without
  reading Rust.
- **A Measure is code.** It is one quantity the engine computes for one object (the GZ area from 0° to
  30° of a loading condition, the clear side gap of a tank). Physics and geometry stay in typed, tested
  Rust in `ovcd-core`. Many Rules share one Measure.
- **No expression language.** A limit is a constant, a table, or a formula **shape** coded in Rust whose
  coefficients are data (for example d = a + Vc × b / c, with a, b and c in the file). The shapes keep
  the order of operations the code uses today, so moving a number into data changes no bit of a result.
- **A Rule Set is a named, versioned group of Rules** from one source edition, for example the IGC Code
  as amended for ship type 2G, or the IS Code 2008 Part A. It has two version numbers: the **source
  edition** (what IMO or the class society published) and the **set version** (OVCD's encoding of it).
  A typo fix in our table is a new set version of the same edition.
- **Rule ids are `<family>/<paragraph>/<slug>`**, stable and never reused (for example
  `is2008/A.2.2.1/area-0-30`).
- **A Design selects its Rule Sets.** A new **Service Area** input (river only, or coastal and river)
  sets the default selection. Ocean only is not in the first release. Each Option stores the Rule Sets it
  used, with version and content hash, marked typed or default, like every input.
- **Strictest governs.** A Requirement is the set of Rules, from any selected Rule Set, that test the
  same Measure (with the same arguments) on the same object in the same direction. All of them apply.
  The Option meets the Requirement only if it meets every one. Criteria of one direction are totally
  ordered by strictness, so the strictest always exists, and a value meets every Rule of the
  Requirement exactly when it meets the strictest one. That Rule is marked as governing; it is chosen
  from the Criteria, not from the value, with a fixed order for ties. No Rule can loosen another.
- **A Rule is a Criterion on a Measure, with an Applicability.** A Criterion is a comparison, a limit and
  a tolerance that keeps the code's own comparison. A design input that a rule book bounds (the weld
  factor, the allowable stress) is an input Measure with a normal Criterion; there is no separate
  "parameter" Rule. An Applicability is a list of conditions `input ∈ {values}` on a Rule or a Rule
  Set, and never grows into an expression language.
- **Every released version of every Rule Set is kept**, unchanged, so an old file reproduces exactly. A
  change makes a new version. The engine never moves a saved file to a newer version without a warning.
- **Rule Sets load at run time.** The built-in sets are data files in the repo, built into every binary
  and the WebAssembly build. A user or an agent can also load a Rule Set file at run time, as a Defaults
  Profile is loaded today. Agents change rules by editing these files in a pull request.
- **Only text-checked rules count as sourced.** A Rule is `verified` only when a second reader, human or
  agent, who did not write the Rule, has read its paragraph in the actual text, with the document,
  edition and locator recorded. The process is `docs/rules/codification.md`. Every other Rule is
  `unverified`, and every output says so. OVCD's own checks (physical feasibility, method validity
  ranges, tool limits) carry an honest origin and are never shown as a published rule.
- **Paywalled texts:** store the numbers and tables with the paragraph cited. Never copy the prose. A
  title is OVCD's own words.
- **Every result carries a Rule Trace**: one row per Rule of every selected Rule Set, with the rule id,
  the Rule Set and version, the citation, the inputs, the value, the limit, the margin, the outcome and
  the verification status. A Rule the engine cannot evaluate yet is a row with the outcome `not
  evaluated`, never left out.
- **One `.vdesign` format version** carries both the Rule Set selection and the hull geometry block of
  ADR 0012.

## Considered

- **A. Rules as declarative data with a small expression language.** Lowest barrier for an expert who
  does not read Rust, and all rule content in one place. Rejected as the whole answer: an expression
  language is a second programming language with no types, units or tests, and it grows. Rewriting a
  formula in it can change the last bit of a result, which breaks the results hash of ADR 0007. Physics
  cannot live in it anyway, so new rules still need Rust. Decision-table engines that share one standard
  still differ in what they support; a home-made language would drift further.
- **B. Rules as typed Rust code with a registry and metadata attributes.** Type safety, unit safety and
  bit-identical results come for free. Rejected as the whole answer: limits and citations in attributes
  are not data a reviewer can read; a new rule set needs a Rust change and a release; a CI check that
  every rule has a citation must parse source code; and rule-data mutation (harness T5.6) needs numbers
  it can move.
- **Build-time generation** of Rust tables from rule files (as `tools/gen_vessel_classes.py` does for
  Vessel Classes). Rejected because Rule Sets must load at run time. The built-in sets take the same
  loading path as a user's file, so there is one path to test.
- **Interpretation as a separate Rule Set** (`IGC_2G` and `IGC_2G_EXACT` side by side). Rejected: they
  were two readings of one paragraph. #237 kept only the reading the text supports. Where a real
  ambiguity remains, it is recorded on the Rule, not hidden as a second set.
- **Declared overrides** (a Rule that loosens another, with its own citation). Not adopted: strictest
  governs, always. A real case that needs an exception (for example an equivalent-safety route) gets its
  own ADR.

The synthesis takes A's data and B's code: numbers and citations are data, arithmetic is code.

## Consequences

- Adding a Rule Set is a data file plus its harness cases. Only a quantity the engine does not compute
  needs Rust: a new Measure with its own sourced test. Never a new `if ruleset == ...` branch.
- The migration of today's rules is done in steps that change no result and no results hash. The
  Rule Trace is not part of the results hash (as the `drawing` object is not); the Rule Set hashes stored
  in the file cover the rules. A rule whose cited value differs from its source keeps its current value,
  is marked `unverified`, and becomes a finding (#121), decided one at a time.
- The results show which Rule Sets were checked and which were not. A river-only result says that no
  statutory technical code was checked, only the named Rule Sets.
- `fit` (#264) keeps only arrangements that meet every evaluated blocking Rule of the selected Rule
  Sets. A Rule the engine cannot evaluate yet never counts as met: every passing verdict names the
  blocking Rules not evaluated.
- The coherence properties of the composition (strictest governs is well defined; adding a Rule never
  turns a failing design into a passing one; `fit` never labels a failing design as passing) are stated
  in the spec, tested as properties, and modelled in Lean in a separate job (ADR 0014).
- The harness keys its expected verdicts by rule id, reads the Rule Trace row by row, and mutates rule
  data loaded at run time.
- ADR 0007 gains format version 3 and the note that the Rule Trace is outside the results hash.
- The design and the build plan are in `docs/specs/0004-rules-engine.md`.

## References

- OpenHull, `AGENTS.md` §5 formula whitelist (register first, location checked against the text).
- OpenFisca legislation parameters (law as YAML parameters with a reference per value; formulas in code).
- buildingSMART Information Delivery Specification 1.0 (applicability and requirement blocks).
- OMG DMN decision tables (the "collect, max" hit policy is the model for strictest governs).
- Catala (exceptions declared, never implied by order).
- Class society rule editions: one edition a year with an entry-into-force date; the edition that applies
  is fixed by the contract date.
