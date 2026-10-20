# A Lean model of the rule composition kernel, as a separate pass and a test oracle

Spec 0004 says how Rules combine: a Requirement is met only if every Rule in it is met, the strictest
governs, no Rule loosens another, and `fit` returns only designs that pass. These claims are easy to
state and easy to break in code: a wrong tie rule, a strict and a non-strict bound at the same limit, a
failing Measure read as a pass, or a search that ranks before it filters (#264 found that one). We asked
whether Lean 4 proofs can show that the engine is coherent, and how such proofs would say anything
about the shipped Rust engine.

A prototype is in `proofs/lean/` (about 640 lines, core Lean 4.22, no Mathlib). It proves four
properties of a model of the kernel and builds an executable oracle from the same definitions. A cold
build takes about 2 s once the toolchain is installed; the toolchain is about 1.8 GB on disk.

## Decision

Accepted by the maintainer on 2026-10-20, with the narrow scope below.


- **Lean models the composition kernel only**: Criteria and their strictness order, the outcome of a
  Rule on a value, the Option verdict, the governing Rule, the constant-bound conflict check, and the
  `fit` selection over a list of candidates. Nothing else. Measures, physics, geometry, tables,
  formula shapes and the loader stay in tested Rust.
- **What Lean proves** (theorem names in `proofs/lean/README.md`):
  1. Strictest governs is well defined: the strictness order on Criteria of one direction is total and
     transitive, and a value meets the strictest Criterion exactly when it meets every Criterion of the
     Requirement.
  2. The conflict check is sound: when it reports a conflict, no value meets both Criteria; when it
     reports none for two non-strict Criteria, a value exists.
  3. Adding a Rule never turns a failing design into a passing one.
  4. `fit` never returns a candidate that fails, it returns the most cargo among those that pass, and
     for a fixed candidate list adding Rules never raises the cargo and never turns "none passes" into
     an answer.
- **Where each candidate property is checked:**

| Property | Lean | Rust property test | Load-time or CI check |
|---|---|---|---|
| Strictest governs is well defined | proved (1) | the same statement on the Rust evaluator | — |
| No two Rules that cannot both be met, unless declared | proves the check sound (2) | — | the loader refuses an undeclared constant conflict in one set; warns across sets |
| Rule ids unique, of the form `<family>/<paragraph>/<slug>` | not modelled: nothing to prove | — | the loader (grammar, uniqueness, locator match) and the provenance gate (#273) |
| Adding a Rule never makes a failing design pass | proved (3) | random Rule sets on the Rust evaluator | — |
| `fit` never labels a failing design as passing | proved (4) | random designs and tightened sets on the real `fit` | the harness: 0 false passes |

- **The tie to Rust is a differential test, not trust.** The oracle (`lake exe ovcd-rules-oracle`) reads
  cases as JSON (Rules with key, direction, limit, strictness, consequence; Measure values, `null` for
  not evaluated) and prints the verdict, each row's outcome and each Requirement's governing Rule. A
  Rust test generates random cases with a fixed seed, writes the same JSON from the evaluator's own
  input structs, runs the oracle, and compares every field. Values pass as exact `f64` bit patterns,
  so no decimal parsing stands between the two. The Rust side folds each Rule's tolerance into its
  limit with the same `f64` operation its own comparison uses, so both sides compare the same numbers.
  The spec's Domain model section names the Lean definition of each term.
- **Never in the normal build.** The proofs and the differential test run in their own CI job
  (`.github/workflows/lean-proofs.yml`): on changes to `proofs/lean/**` (and to the evaluator's files,
  once they exist), once a week, and by hand. The job is not a required check and is not in `tools/gates.sh`. Like
  mutation testing, a failure is an issue for the next builder, not a blocked merge. A builder never
  needs Lean installed to land a ticket.
- **Core Lean only.** No Mathlib: it would add minutes of build and gigabytes of cache for theorems
  this kernel does not need. The model defines its own small total-preorder class.

## Considered

- **Reject: property tests only.** Property tests on the Rust evaluator catch most kernel bugs, and they
  are in the plan anyway. Not chosen as the whole answer: a property test samples, so it shows the
  property holds on the cases it drew; the proofs show it for every case of the model. The oracle also
  gives the differential test a second, independent implementation written from the spec's words,
  which is what found five gaps in the spec (below). The cost is small and kept out of the build.
- **Lean over the whole engine** (Measures and physics too). Rejected: the physics is floating-point
  arithmetic ported bit for bit from the original engine; proofs over reals would not hold for the
  shipped `f64` results, and the effort is out of proportion. The risky errors there are wrong data and
  wrong Measures, which sourced tests, Calibration and the harness catch.
- **Verifying the Rust code directly** (a bounded model checker such as Kani, or a deductive verifier).
  Attractive, because there is no model to drift. Not chosen now: the evaluator does not exist yet, and
  a bounded check only covers small sizes. Worth trying on the evaluator once #269 lands; if it works,
  it can replace the differential test.
- **Mathlib.** Gives `LinearOrder`, real numbers and dense orders. Rejected for build time and size (see
  Decision).
- **Alloy or TLA+.** Suited to state machines and protocols. The kernel is a pure function, so they add
  nothing over Lean or property tests.

## Consequences

- The proofs speak about the model. They speak about the engine only as far as the differential test
  passes, and only for a lawful order: `Float` has none (NaN). The loader refuses non-finite limits and
  the evaluator turns a non-finite Measure into a failure, so NaN never reaches a comparison; that
  claim is itself tested in Rust, not proved.
- The cargo property of `fit` holds for a fixed candidate list. A `sizes` Rule changes the candidates
  the search builds, so for those the guarantee is soundness only (every answer meets every Rule).
- A change to the composition semantics (a new comparison, a new consequence, an override, which ADR
  0013 rules out) needs the model and its proofs updated in the same pull request. A change to rule
  data or to a Measure never touches Lean.
- Building the prototype forced five decisions the spec had not made: a tie rule for the governing
  Rule; whether an advisory Rule may govern; that the Requirement key includes the direction; that a
  strict bound is stricter than a non-strict one at the same limit; and a per-Rule tolerance (one
  module compares with 1e-9, another with none). Spec 0004 now settles all five.
- Contributors who never open `proofs/lean/` are not affected. The maintainer may drop the job at any
  time without touching the engine.
- Follow-up work, once the evaluator exists (#269): the Rust side of the differential test, and the
  path filter on the evaluator's files.

## References

- `proofs/lean/README.md`: the model, the theorems, how to run, what is left out.
- Spec 0004, "Domain model (precise)".
- ADR 0013 (rules as data), ADR 0010 (test strategy), `docs/agents/build-process.md` (hardening pass).
