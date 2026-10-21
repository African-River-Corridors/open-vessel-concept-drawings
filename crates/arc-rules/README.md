# arc-rules

A domain-free rules engine. It reads versioned, content-hashed **Rule Sets** (JSON), evaluates
them on a design, and gives a **Rule Trace**: one row per Rule per object, the governing row of each
requirement, and the verdict. Of the Rules on one Measure, one object and one direction, the
strictest governs. No Rule loosens another.

The crate knows no domain. The caller declares its object types, its Measures, the inputs a Rule
may depend on, and any formula shapes. Open Vessel Concept Drawings (OVCD) is the first user; its
vessel rules live in OVCD's own crates, not here.

- `no_std` with `alloc`. No file, clock, environment or randomness access.
- With no features the crate has no dependency. The `load` feature (on by default) adds the Rule
  Set loader, the content hash and the library, with `serde_json` and `sha2` (both without `std`).
- Design records: ADR 0013 (rules are versioned Rule Sets), ADR 0014 (the Lean model) and
  ADR 0019 (this crate). The full model is spec 0004 of the OVCD repository.

## Schema 1.x is frozen

The Rule Set file format is schema 1.x (`rule-set.schema.json`). For outside users it is frozen:
a change is **additive only** (a new optional field, a new value of an enumeration, a new shape a
caller registers). A breaking change needs schema 2.0 and a maintainer decision. The loader
refuses a file the schema refuses.

## Extension points

| What | How the caller gives it | Used by |
|---|---|---|
| Object types | String keys, such as `building` or `stair`. `Domain::design_object` names the design itself. | loader, evaluator |
| Measures | `Domain::measure(id)` and `MeasureSource::def(id)` give a `MeasureInfo`: unit, object type, value kind, arguments, method status. | loader, evaluator |
| Applicability vocabulary | `Domain::input(name)` gives an input's values and its scope: `Design` (one value) or `Object` (one value per object). | loader |
| Input values | `MeasureSource::input(name)` for the design; `MeasureSource::object_input(object, index, name)` for one object (by default the design's value). | evaluator |
| Formula shapes | A list of `FormulaShape`: id, operands, coefficients, the operands in a free unit, and a function. | loader, evaluator |
| Measure values | `MeasureSource::value(measure, index)`: the only way the evaluator reads a value. | evaluator |

The table row shapes `const`, `a+v*b/c` and `a+v/c` are built in: they are linear and generic.

A Rule's `origin` says where it comes from. `regulation`, `class`, `guideline` and
`national-practice` are published texts: such a Rule cites its paragraph, and it is *sourced*
only when `verified` is filled. `method-validity`, `physical` and `tool-limit` are the tool's own
checks, never shown as sourced.

## A minimal example

A toy domain of stairs in a building (invented numbers; it is in `tests/toy_domain.rs` and runs in
the crate tests):

```rust
use arc_rules::domain::*;
use arc_rules::eval::{MeasureSource, MeasureValue, evaluate};
use arc_rules::load::load;
use arc_rules::model::MeasureRef;

struct Stairs; // the vocabulary, for the loader

impl Domain for Stairs {
    fn measure(&self, id: &str) -> Option<MeasureInfo> {
        (id == "stair.riser").then(|| MeasureInfo {
            unit: "mm".into(),
            object: "stair".into(),
            value: ValueKind::Number,
            args: Vec::new(),
            method_status: MethodStatus::Verified,
        })
    }
    fn input(&self, name: &str) -> Option<InputDef> {
        (name == "location").then(|| InputDef {
            values: vec!["inside".into(), "outside".into()],
            scope: InputScope::Object,
        })
    }
    fn shapes(&self) -> &[FormulaShape] {
        &[]
    }
    fn design_object(&self) -> &str {
        "building"
    }
}

struct Building; // one design, for the evaluator

impl MeasureSource for Building {
    fn def(&self, id: &str) -> Option<MeasureInfo> {
        Stairs.measure(id)
    }
    fn objects(&self, object: &str) -> Vec<String> {
        if object == "stair" { vec!["Main stair".into(), "Garden steps".into()] } else { vec![] }
    }
    fn value(&self, _: &MeasureRef, index: usize) -> Result<MeasureValue, String> {
        Ok(MeasureValue::Number([175.0, 160.0][index]))
    }
    fn input(&self, _: &str) -> Option<String> {
        None
    }
    fn object_input(&self, _: &str, index: usize, _: &str) -> Option<String> {
        Some(["inside", "outside"][index].into())
    }
}

let set = load(RULE_SET_JSON, &Stairs)?.set;
let trace = evaluate(&[&set], Stairs.shapes(), &Building);
for row in &trace.rows {
    println!("{:?} {:?} {}", row.rule, row.object, row.outcome.key());
}
```

## Tests

- `tests/toy_domain.rs`: the example above, with a formula shape and both input scopes.
- `tests/independence.rs`: the crate depends on no crate of a tool that uses it.
- `tests/lean_oracle.rs`: the differential test against the Lean model of the composition kernel.
  It runs when `ARC_RULES_LEAN_ORACLE` names the built oracle (`proofs/lean`, weekly job).

Licence: Apache-2.0.
