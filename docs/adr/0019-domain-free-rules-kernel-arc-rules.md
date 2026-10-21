# A domain-free rules kernel, `arc-rules`, that two tools share

A second open-source tool, a waterway designer, needs the same rules engine as OVCD: versioned,
content-hashed Rule Sets, the strictest Rule governs, and a Rule Trace on every result (spec 0004,
ADR 0013). Its rules are channel rules, not vessel rules. The evaluator was already domain-free, but
its object types were a closed vessel enum, and the Measure registry, the applicability vocabulary
and the formula shapes were OVCD's. The maintainer decided on 2026-10-21 to carve out one shared
kernel before more OVCD Rule Set work (issue #463).

## Decision

- A new crate, **`arc-rules`**, in the OVCD workspace, holds the domain-free kernel only: the Rule
  Set model, schema and loader; the content hash; the library and the selection checks; the
  version classes and the released-versions check; Applicability; the evaluator (`holds`,
  `margin`, `stricter`, strictest governs, the Rule Trace); and the formula-shape interface.
- **Object types are string keys.** The caller declares its object types; a Measure names the key
  of the type it is computed for.
- **The caller passes in** its Measure registry, its applicability vocabulary and its formula
  shapes, through the `Domain` trait for the loader and a list of `FormulaShape` for the evaluator.
  The kernel holds no vessel or channel shapes. Only the linear table row shapes (`const`,
  `a+v*b/c`, `a+v/c`) stay in the kernel: they are generic.
- **The evaluator reads values only through `MeasureSource`.**
- `arc-rules` is `no_std` with `alloc`, like `ovcd-core`. With no features it has no dependency at
  all; the `load` feature (on by default) adds `serde_json` and `sha2`, both without `std`. It holds
  only generic content: no project, site or client names. The release guard covers it.
- OVCD keeps its side in its own crates: `ovcd-core::rule_trace` declares the object types, the
  computed Measures and the formula shapes, and wraps `evaluate` and `failures` with its shapes;
  `ovcd-format::rule_set` declares the vocabulary to the loader (`Ovcd`), holds the built-in sets and
  the provenance gate. Every path OVCD code used before still works (`ovcd_core::rules` re-exports
  the model).
- **Rule Set schema 1.x is frozen for outside users.** A change is additive only: a new optional
  field, a new value of an enumeration, a new shape the caller registers. A breaking change needs
  schema 2.0 and a maintainer decision.
- Two additive requests of the other tool, approved by the maintainer on 2026-10-21, join schema 1.x
  now:
  1. **Object-scoped inputs.** The vocabulary marks each input as design or object scope.
     `MeasureSource::object_input(object, index, name)` reads an object's value; by default it
     returns the design-wide value. An Applicability condition on an object-scoped input is read for
     each object of the Rule's type: an object that does not meet it gets a `not applicable` row. A
     Rule Set's own Applicability may not name an object-scoped input. The file format does not
     change, and the content hash leaves the scope out (the file does not say it).
  2. **Two more origins**, `guideline` and `national-practice`. Like `regulation` and `class`, a Rule
     of these origins cites its paragraph, and it is sourced once `verified` is filled.
- The **Lean differential test** (ADR 0014) runs against `arc-rules`: random constant-limit Rule
  groups and values go to both the Lean oracle and the kernel's evaluator, and the verdict, each
  outcome and each governing Rule must agree. It runs in the weekly Lean job, never in the fast gate.

## Considered options

- **Copy the engine into the second tool.** Rejected: two engines drift, and the Lean tie and the
  harness would cover only one.
- **A separate repository for the kernel now.** Rejected for now: OVCD is the only user that ships,
  and its tests are the kernel's best tests. The other tool pins an OVCD release tag that holds the
  crate. The crate can move out later without an API change.
- **One object type per reach kind** (as OVCD has one per tank role) instead of object-scoped
  inputs. Rejected by the other tool's design: it would repeat every Measure and Rule.

## Consequences

- No OVCD result changes: every results hash, fixture hash, Rule Set content hash and Rule Trace
  stays the same. A new OVCD Measure, object type or shape is registered on the OVCD side; the
  kernel does not change for it.
- A change to the kernel's behaviour is a change for both tools. It goes through OVCD's gates (the
  design-impact check, the evaluation harness, WebAssembly parity) and is released by an OVCD tag.
- The schema file lives in the kernel (`crates/arc-rules/rule-set.schema.json`). OVCD keeps a copy
  beside its rule files, because their `$schema` points there; a test keeps the two the same.
