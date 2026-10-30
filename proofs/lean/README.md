# Lean model of the rules engine's composition kernel

This folder holds a small Lean 4 model of how the rules engine (spec 0004) combines Rules. It
proves six properties of that model. It also builds an oracle program that runs the same
definitions on real numbers, so the Rust engine can be compared with it.

The model is a prototype for the Lean decision record (ADR 0014). It is **not part of the
build**. The normal gates (`tools/gates.sh`, the `test` job) never run it. A separate CI job,
`.github/workflows/lean-proofs.yml`, runs it when this folder changes, once a week, and by hand.

## What the model holds

- **Bound.** A direction (`lower` for `ge`/`gt`, `upper` for `le`/`lt`), a limit and a `strict`
  flag. `holds b v` says whether the value `v` meets the bound.
- **At least as strict.** For two bounds of one direction: a lower bound is stricter when its
  limit is higher, or equal and strict. An upper bound mirrors this.
- **Rule.** An id, a requirement key (the Measure id, its arguments and the object), a bound and
  a `blocking` flag.
- **Design.** A value for each requirement key, or `none` when the engine cannot evaluate the
  Measure. A row is `pass`, `fail` or `notEvaluated`.
- **Verdict (`ok`).** No blocking Rule fails. A `notEvaluated` row does not fail the design; the
  result names it.
- **Governing Rule.** Of the Rules on one key and one direction, the one with the strictest
  bound. On a tie the earlier Rule governs.
- **`fit`.** Of a list of candidates, the one with the most cargo among those whose verdict is
  `ok`. On a tie the earlier candidate wins. `none` when no candidate passes.

## What is proved

All proofs are complete. No proof uses `sorry`. `AxiomCheck.lean` prints the axioms each theorem
uses; they are core Lean's `propext` and `Quot.sound` only.

| Property | Theorems | File |
|---|---|---|
| 1. Strictest governs is well defined | `atLeastAsStrict_refl`, `atLeastAsStrict_total`, `atLeastAsStrict_trans`, `holds_of_atLeastAsStrict`, `holds_strictestBy_iff` | `Bound.lean`, `Strictest.lean` |
| 2. The conflict check is sound | `no_value_of_conflict`, `witness_of_not_conflict` | `Conflict.lean` |
| 3. Adding a Rule never makes a failing design pass | `ok_of_ok_append`, `not_ok_append` | `Rule.lean` |
| 4. `fit` never returns a failing design | `fit_sound`, `fit_max`, `fit_cargo_antitone`, `fit_none_append` | `Fit.lean` |
| 5. The sized value meets its Requirement | `holds_of_sizedAt`, `sized_meets_requirement` | `Sizing.lean` |
| 6. Adding a Rule never lowers (lower bound) or raises (upper bound) the sized value | `strictestBy_mono`, `sized_limit_mono_lower`, `sized_limit_mono_upper` | `Sizing.lean` |

In words:

1. Of bounds of one direction, one is always at least as strict as the other, and the relation
   is transitive. A value meets the strictest bound exactly when it meets every bound. So "the
   strictest governs" and "every Rule applies" give the same verdict.
2. When the check reports a conflict between a lower and an upper bound, no value meets both.
   When it reports none and both bounds are non-strict, the lower limit meets both.
3. A design that passes a list of Rules plus more Rules passes the list alone.
4. `fit` returns a candidate from the list that meets every blocking Rule, with the most cargo
   of those that pass. Adding Rules never raises the cargo of the answer, and never turns "no
   answer" into an answer.

5. A Rule with `sizes: true` lends its limit to a sizing step. For a non-empty Requirement of bounds
   of one direction, the value of the governing bound meets every bound of the Requirement. The
   value is the limit for a non-strict bound; for a strict bound (`gt`, `lt`) it is any value
   strictly beyond the limit (`Bound.sizedAt`). The limit is the effective bound: the Rust
   evaluator folds the tolerance into it before it compares.
6. Adding bounds to a Requirement never makes the governing effective bound smaller (lower bounds)
   or larger (upper bounds), so the sized value never falls (or rises). The general form: the
   governing bound of a larger Requirement is at least as strict as that of the smaller one.

## The order the proofs use

Core Lean has no `LinearOrder` class; that class is in Mathlib, which this model does not use.
The proofs need less: a **total preorder** on the limit type (reflexive, transitive, total). The
class `TotalPreorder` in `Order.lean` states it. `Int` and `Nat` have instances. The rationals
and the reals also satisfy it. The definitions use only `≤`, so they also run on `Float`.

## The oracle and `Float`

`Float` has no lawful order: `NaN ≤ NaN` is false, so reflexivity and totality fail. The proofs
therefore hold only for a lawful order. The oracle runs the same definitions at `Float`. Its
answers match the proved behaviour only when every limit and every value is finite. The Rule Set
loader must refuse a non-finite limit, and the evaluator must turn a non-finite Measure value
into a failed row before it compares. Both are load-time or run-time checks in Rust, not proofs.

The oracle reads decimal numbers from JSON. For the example limits, Lean's parse and Rust's
parse give the same IEEE 754 bits. A differential test must still check this, or pass numbers as
bit patterns.

## How to run

Install elan (the Lean version manager). The file `lean-toolchain` pins Lean 4.22.0.

```bash
cd proofs/lean
lake build
lake env lean AxiomCheck.lean
./.lake/build/bin/ovd-rules-oracle examples/cases.json
```

A cold build takes about 2 s on an Apple silicon laptop once the toolchain is installed. The
toolchain itself is about 1.8 GB on disk.

### Oracle input

```json
{"cases": [{
  "name": "...",
  "rules": [{"id": "is2008/A.2.2.1/area-0-30", "key": "gz.area(0,30)@laden",
             "dir": "lower", "limit": 0.055, "strict": false, "blocking": true}],
  "values": {"gz.area(0,30)@laden": 0.071}
}]}
```

A missing key or `null` value means "not evaluated". A Rule may carry `"sizes": true` (default
false). The output gives, per case, the verdict `ok`, each Rule's outcome, the governing Rule id per
`key|direction`, and `sized`: for each requirement that has a `sizes` Rule, the governing Rule, its
effective bound (`limit`) and its `strict` flag. `examples/expected.json`
holds the expected output of `examples/cases.json`; CI compares the two.

## The tie to the Rust engine

The proofs say nothing about the shipped engine on their own. The tie is a **differential test**
against the domain-free rules kernel, `arc-rules` (`crates/arc-rules/tests/lean_oracle.rs`, ADR
0019). It generates random groups of constant-limit Rules and random values, writes them in the
oracle's JSON form, runs the oracle, and compares the verdict, each outcome, each governing
Rule and the sized bound of each requirement that has a `sizes` Rule with the kernel's evaluator. A
key with no value is written as Rules that are not evaluated; the model's governing Rule and sized
bound of such a key are not compared, as the evaluator marks governing rows
only among worked-out limits. It runs in the same separate pass as this job, never in the fast
gate:

```bash
cd proofs/lean && lake build && cd ../..
ARC_RULES_LEAN_ORACLE=$PWD/proofs/lean/.lake/build/bin/ovd-rules-oracle \
  cargo test -p arc-rules --test lean_oracle -- --nocapture
```

Without `ARC_RULES_LEAN_ORACLE` the test passes with a note, so `cargo test` needs no Lean.
`ARC_RULES_LEAN_CASES` (default 400) and `ARC_RULES_LEAN_SEED` set the number of cases and the
seed.

## What the model leaves out

- **Measures.** The model takes each value as given. It proves nothing about the physics or the
  geometry.
- **Tables and formula shapes.** A limit is one number per row, already worked out.
- **A comparison tolerance.** Some checks today compare against `limit - 1e-9`. The model has no
  tolerance; the Rust evaluator must fold it into the limit before it compares.
- **`size` Rules that shape the candidates.** In the engine, the governing clearance also sets
  the candidates `fit` builds. Here the candidate list is fixed. Property 4 still holds for any
  list, but the cargo property compares two runs over the same list only.
- **Cargo as a real number.** Cargo is a `Nat` here; in the engine it is a volume in m³.
- **Applicability, `includes` and rule-id rules.** These are load-time checks in Rust.
- **Advisory Rules and governing.** The proved `governing` picks the strictest of the Rules it is
  given. The oracle gives it the blocking Rules of a requirement, or all of them if none blocks, as
  spec 0004 says, and reports a failing advisory Rule as `warn`.
- **`size` naming.** Spec 0004 now calls these Rules `sizes` Rules; sizing is a use, not an
  outcome.
