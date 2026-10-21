# Engineering basis: docs for domain experts (design)

Status: draft design. The pages in this folder are first drafts and need expert review.

## Purpose

These pages tell a domain expert what OVCD assumes, which rules it applies, and how it estimates values that no
rule gives. A naval architect, a gas containment engineer or a cost engineer reads the page for their area and can
answer three questions:

1. What does OVCD compute here, and by which method?
2. Which values come from a rule, which come from a published source, and which are estimates?
3. Where is OVCD wrong, too simple or not yet checked?

The pages describe what the code does **today**, and each statement points to the code or data that does it. The
pages do not say what the code should do. Proposed changes go in a separate file per area (see "Proposals" below),
so a reviewer can tell fact from proposal.

OVCD is for concept and preliminary design, never for approval (ADR 0011). Every page says so in its first lines.

## Labels

Every value and method on a page gets exactly one label. These are the same labels the Defaults Profile and the rules
engine use (spec 0004).

| Label | Meaning |
|---|---|
| **Rule (verified)** | A Rule in a Rule Set, with a paragraph citation, checked against the text by a second reader. |
| **Rule (unverified)** | A Rule with a citation that no second reader has checked yet. |
| **Sourced default** | A default value with a published source in the Defaults Profile. The user can change it. |
| **Estimate** | A value with a stated basis but no source that gives it directly. The user can change it. |
| **Method** | A calculation method (for example a resistance series), with its reference. |
| **Fixed in code** | A constant the user cannot change yet. Each one is a candidate to become an input (#4). |

## Areas

One page per area. Each area has the reviewer discipline it is written for.

| # | Page | For | Covers |
|---|---|---|---|
| 01 | `01-hull-form-and-hydrostatics.md` | Naval architect | Principal dimensions, Vessel Classes as envelopes, hull form, sections, hydrostatics, coefficients |
| 02 | `02-weights-and-centres.md` | Naval architect, weight engineer | Lightship (steel, outfit, machinery), deadweight, centres of gravity, calibration factors |
| 03 | `03-intact-stability.md` | Stability specialist | Loading conditions, KG, free surface, GZ, IS Code 2008 Part A criteria, known limits |
| 04 | `04-type-c-tanks.md` | Gas containment engineer | Tank geometry, heads, wall thickness, materials, design pressure and temperature, insulation, fill and loading limit, dome, tank weight |
| 05 | `05-gas-cargo-arrangement.md` | Gas carrier designer | Protective location (IGC 2.4), tank fit and `fit`, separation, cofferdams, end spaces, inland gas clauses |
| 06 | `06-liquid-dry-and-container-cargo.md` | Barge designer | Liquid tanks, dry bulk holds, container stowage, capacity, double hull and double bottom |
| 07 | `07-resistance-and-powering.md` | Hydrodynamicist, marine engineer | Resistance, propulsion, power, push-tow and tug-barge systems, gas-fuelled and electric options |
| 08 | `08-waterway-and-route.md` | Inland waterway engineer | Service Area, draught and squat, air draft, waves, freeboard, journey derating |
| 09 | `09-rules-and-rule-sets.md` | Classification and regulatory specialist | Rule Sets, how a rule is interpreted and applied, the strictest rule governs, verification status, codification |
| 10 | `10-cost-boq-and-capital-budget.md` | Cost engineer | BOQ lines, quantities, cost rates as inputs, the capital budget |
| 11 | `11-drawings.md` | Naval architect, draughtsperson | What each drawing shows, conventions, what it does not show |
| 12 | `12-calibration-and-validation.md` | Any reviewer | Calibration cases and bands, the evaluation harness, what evidence backs each area |

## Page template

Each page follows this structure. Keep the headings, so the pages read the same.

```markdown
# <Area>

> Concept and preliminary design only. Not for approval. Draft: needs expert review.

**For:** <discipline>  ·  **Code:** <main modules>  ·  **Rule Sets:** <ids, if any>

## What OVCD computes here
<Outputs, in a short list, with the result names a user sees.>

## How it works today
<The method, step by step. Formulas in plain notation with units. Each step names its code location
(`crates/ovcd-core/src/<file>.rs`) and its label.>

## Assumptions, rules and estimates
| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
<One row per value. Id form: <area>-A<nn>, for example `TC-A03`. "If it is wrong" says the direction of the
error: conservative, non-conservative or unknown.>

## Inputs the user can change
<Input name, default, unit, where the default comes from.>

## Limits and known findings
<What the method does not cover. Links to findings in the register (#121) and to open issues.>

## Evidence
<Calibration cases, evaluation harness cases and tests that back this area, with their results.>

## Questions for reviewers
<Numbered, specific questions an expert can answer. For example: "Is 0.17 × D a fair dome diameter for
tanks under 500 m³?">
```

## Proposals

Each area has a companion file `proposals/<nn>-<slug>.md`. It is not published. It lists:

- **Gaps:** what an expert would expect that OVCD does not do.
- **Errors:** places where the code and its cited source disagree. Each one needs the code location and the
  source paragraph. These become findings for the register (#121) after maintainer triage.
- **Changes:** proposed improvements, each with its effect on results (none, small, large) and its order of work.

## Rules for writing

- Describe the code as it is. Read the code; do not trust comments or older docs without a check.
- Cite only text you have read. For a rule, give the document, edition and paragraph.
- Simplified Technical English: short sentences, active voice, one meaning per word.
- Use the glossary terms (`GLOSSARY.md`). If a term is missing, list it in the proposals file.
- The repo is public: no client or project names, no people's names.

## Publishing

The twelve area pages are published as draft pages in the "Engineering basis" section of the docs site (maintainer
decision, 2026-10-21). `website/build.py` assembles `docs/basis/[0-9][0-9]-*.md` only: this README and `proposals/`
are never published. `build.py` puts a "Draft: needs expert review" banner on every page, so no page can ship
without it. A page loses the banner only when a named discipline reviewer signs it off in the page history.

The triage of all proposals is `proposals/00-triage.md`; its items are GitHub issues titled "Basis <n>: …".
