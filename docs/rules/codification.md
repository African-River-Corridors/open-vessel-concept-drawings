# Codifying a rule

How a person or an agent turns a paragraph of a rule book into a Rule that OVD can load. The design is
in `docs/specs/0004-rules-engine.md` (section "Domain model") and ADR 0013. The terms are in
`GLOSSARY.md`.

You do not need to read Rust to do this. You need the rule book open at the right paragraph, a text
editor, and this page.

## Who does what

| Role | Does | May not |
|---|---|---|
| **Codifier** (a person or an agent) | Finds the source, writes the rule record, adds the test cases, opens the pull request | Set `verified` on a Rule they wrote |
| **Second reader** (a person or an agent who did not write the Rule) | Reads the cited paragraph in the cited edition and checks the record field by field | Change the record in the same review without saying so |
| **Maintainer** | Merges; decides findings; decides any change to a released set | — |

The maintainer decided on 2026-10-20 that the second reader may be a person or an agent. The record
says which kind (`maintainer`, `expert-reviewer` or `agent`). It never names a person: the repository is
public. The pull request link shows who did the work.

## The steps

### 1. Find the source and fix its edition

- Use the official text. For IMO codes, that is the adopting resolution and every amending resolution
  (free on the IMO index of resolutions). For a class rule book, the edition the design will use.
- Record the **edition**: the base document, every amendment that is in force on the date you record,
  and the in-force date. "As amended" means: list the amendments you checked and say that none changes
  your paragraph, or say which one does and use its text.
- Put the file in the rule-source library with its SHA-256, and add it to the set's `sources`.
- **Cite only what you have read.** If you cannot get the text, stop: write the Rule as `unverified`
  with `source.access` set, and say in the pull request what is missing. Never write a value from
  memory, a summary, a secondary web page or a model's answer as if it were the text.
- **Paywalled texts.** Record the numbers, the tables and the paragraph. Never copy the prose. Write the
  title in your own words.

### 2. Extract the clause and give it a rule id

- Read the whole paragraph and the paragraphs it points to (definitions, figures, "as indicated in").
  A limit is often in one paragraph and its measuring convention in another (IGC 2.4.1 and 2.4.2).
- Split the paragraph into **one Rule per requirement**. "Not less than 0.055 … and not less than 0.09"
  is two Rules.
- Rule id: `<family>/<paragraph>/<slug>`.
  - `family`: the Rule Set's family (`igc`, `is2008`, `lr-inland`, `ovd`).
  - `paragraph`: the locator of the paragraph **that states the requirement**, spaces removed
    (`2.4.1.2`, `A.2.2.1`, `9.3.1.11.1(b)`). The loader checks it against `source.locator`.
  - `slug`: lowercase words with hyphens that say which requirement of the paragraph (`area-0-30`).
  - Grammar: `^[a-z][a-z0-9-]*/[A-Za-z0-9][A-Za-z0-9.()-]*/[a-z0-9][a-z0-9-]*$`.
- Check that the id is new. An id is never used again for another requirement, even after a Rule is
  removed. If the same requirement already exists in another set of the same family (an IGC paragraph
  that applies to 1G and 2G), reuse that id and that content exactly.

### 3. Write the Measure and the Criterion

- **Measure**: what quantity the paragraph tests, on which object. Pick it from `ovd rules --measures`.
  Check that the Measure's method measures what the text measures: the datum (plate to plate, excluding
  insulation), the range (up to 40° or the down-flooding angle), the object (each tank, each loading
  condition). If no Measure fits, stop and open an issue for a new Measure: that needs Rust, with its own
  sourced test. Do not bend the Rule to fit a Measure that measures something else.
- **Criterion**: the comparison (`ge`, `gt`, `le`, `lt`) and the limit.
  - "Not less than", "at least" → `ge`. "Not more than", "shall not exceed" → `le`. "Greater than" →
    `gt`. "Less than" → `lt`.
  - A constant: the number and its unit, as printed.
  - A table: each row as printed, with the side of each boundary (≤ or <) exactly as the text says.
  - A formula: the formula shape and its coefficients as printed. Keep the order of operations of the
    text (`0.75 + Vc × 0.2 / 4,000`), not a simplified form.
  - `tolerance`: 0, unless the Rule replaces code that compares with a tolerance (the migration keeps
    it).
- **Kind**: `limit` (a constant), `table`, `formula`, `check` (a yes/no condition) or `note` (a cited
  value shown with no test).
- **Consequence**: `block` if the text says "shall"; `warn` for "should", a recommendation, or a
  migration Rule that the code never checked (see the spec); `note` for information.

### 4. Record units, applicability and the citation

- Units of the Measure and of every number in the limit. SI units as in the text.
- **Applicability** (`applies_when`): the conditions in the text on the Option's inputs, each as
  `input: [values]`: Cargo, Cargo Containment, Service Area, LPG Carriage Condition, tank head type.
  The ship type (2G) is on the Rule Set, not on each Rule. A condition the inputs cannot express (a ship
  length of 24 m or more) is not written as applicability: say so in the pull request.
- **Citation** (`source`): document, edition, locator, the page in the file you read, access.
- **Origin**: `regulation` (a statutory code), `class` (a class rule book), or for OVD's own checks
  `physical`, `tool-limit`, `method-validity`.

### 5. Set the status

- `codified`: your role, the date, and the pull request.
- `verified`: `false`. It stays `false` until a second reader checks it (step 7).
- `evaluated`: `true` if the engine computes the Measure; else `false`, with a `gap` that says what the
  engine lacks ("needs the wind heeling lever").
- `finding`: if the engine's current value or Measure differs from the text, file a finding under #121
  and link it. Do not fix it in the same pull request.

### 6. Add the test cases

For each evaluated Rule:

- **A sourced test** at both sides of the limit, worked by hand from the text and labelled
  `Label: sourced - <citation>`. For a table or formula, test each branch and each boundary, on the side
  the text gives (for d(Vc): Vc = 1,000 m³ is in the first branch; 5,000 m³ in the third).
- **Harness cases** in `evaluation/`: one design just inside the limit and one just outside, written by
  the oracle (`evaluation/oracle/`), not typed by hand. These are the **boundary cases**. The rule-data
  mutation of the harness (#279) then moves the limit a step each way: a move no case notices is a weak
  corpus warning.
- If the Rule shares a Requirement with a Rule of another set, a test that the stricter one governs.

### 7. Second reading

The second reader opens the cited edition at the cited page and checks, field by field, using the
checklist below. Then they either set `verified` (`by`, `date`, `edition_held`, `locator_read`,
`review`) in a follow-up commit on the same pull request, or write what is wrong. A verification
recorded after a set version is released goes into a new **patch** version of the set: the content
hash does not change, so no design result changes and no warning appears.

**Second-reader checklist**

- [ ] The edition and amendments recorded are the ones I read, and none of the listed amendments
      changes this paragraph.
- [ ] The locator and page lead to the paragraph, and the rule id's paragraph segment matches it.
- [ ] One Rule per requirement; nothing in the paragraph was left out without a reason.
- [ ] The Measure measures what the text measures (object, datum, range, units).
- [ ] The comparison matches the words ("not less than" is `ge`).
- [ ] Every number and unit matches the text; every table boundary is on the side the text gives.
- [ ] The formula keeps the text's form and order.
- [ ] The applicability matches the text's conditions, and none is missing.
- [ ] The consequence matches "shall" or "should".
- [ ] For a paywalled text: no prose copied.
- [ ] The sourced test values follow from the text by hand.

### 8. Review and merge

- One pull request per Rule Set change. The body lists each Rule added or changed, its status, and any
  finding filed. A changed released set is a **new version file**; never edit a released file (#273).
- The usual gates apply (`docs/agents/build-process.md`). A Rule that changes a design result needs the
  design-impact table in `CHANGELOG.md`; a new `unverified` Rule in a default stack is a change the
  maintainer decides.
- Check the body and every comment for names and confidential text before posting (the embargo check in
  `docs/agents/issue-tracker.md`).

## Rule record template

Copy this into the `rules` list of the set file. Delete the comments (JSON has none; they are here to
explain). `rule-record.template.json` beside this page holds the same record with no comments.

```jsonc
{
  "id": "<family>/<paragraph>/<slug>",
  "title": "<your own words, at most 15 words>",
  "origin": "regulation",                  // regulation | class | physical | tool-limit | method-validity
  "source": {
    "document": "<document and adopting resolution>",
    "edition": "<base edition and amendments checked>",
    "locator": "<paragraph, as printed>",
    "page": "<file and page you read>",
    "access": "free"                       // free | paywalled | sign-up
  },
  "codified": { "by": "agent", "date": "YYYY-MM-DD", "review": "#<pull request>" },
  "verified": false,                       // or { by, date, edition_held, locator_read, review }
  "applies_when": {},                      // e.g. { "cargo_containment": ["type_c_tanks"] }
  "kind": "limit",                         // limit | table | formula | check | note
  "measure": { "id": "<measure id>", "args": {} },
  "compare": "ge",                         // ge | gt | le | lt
  "limit": { "value": 0.0, "unit": "<unit>" },
  "tolerance": 0,
  "consequence": "block",                  // block | warn | note
  "sizes": false,
  "evaluated": true,                       // or false, with "gap": "<what the engine lacks>"
  "finding": null,                         // or "#<issue>"
  "since": "<set version>"
}
```

## Worked example: IGC 2.4.1 tank side distance, type 2G

**Step 1. Source.** IGC Code as adopted by MSC.370(93) (adopted 22 May 2014, in force 1 January 2016),
amended by MSC.411(97), MSC.441(99), MSC.476(102), MSC.492(104), MSC.523(106) and MSC.566(109). Each
amending resolution was opened: none changes 2.3 or 2.4 (`ovd-rule-sources/igc/NOTES.md`). File
`MSC.370-93.pdf`, PDF pages 21–22. Free.

**Step 2. Clause and id.** 2.4.1.2 says that for types 2G/2PG cargo tanks are located "nowhere less than
'd' as indicated in 2.4.1.1". 2.4.1.1 gives d as a function of Vc in four branches and defines Vc: "100%
of the gross design volume of the individual cargo tank at 20°C, including domes and appendages".
2.4.2 says the distances "shall be measured plate to plate, from the moulded line to the moulded line,
excluding insulation".

So the requirement for a 2G ship is stated in **2.4.1.2**, and d is a formula from **2.4.1.1**. The Rule
is `igc/2.4.1.2/side-clearance`. The formula d(Vc) is a table of the set (`igc-d-vc`), cited to 2.4.1.1,
that other Rules can use too (the bottom distance). (The first draft of spec 0004 named a Rule
`igc/2.4.1.1/d-vc`; that is a formula, not a requirement, and is corrected in the spec.)

**Step 3. Measure and Criterion.**

- Measure: `tank.side_gap`, the clear gap from the moulded line of the outer shell to the outside of
  the tank's insulation, per tank. The engine's tank diameter is to the outside of the insulation
  (GLOSSARY, Tank Outer Diameter). Because 2.4.2 measures to the steel, excluding insulation, the
  insulation lies inside d: the clear gap must be at least d less the insulation thickness. The code
  compares exactly that, so the Rule keeps it.
- Criterion: `ge`; limit: formula shape `less-input` (d − insulation), with d from table `igc-d-vc` at
  Vc = `tank.vc` and the insulation from `tank.insulation`; tolerance 1e-9 m. The tolerance keeps the
  engine's own sizing: its clearance loop stops when d moves less than 1e-9 m, so the gap it builds can
  sit that far under the d of the final tank volume.
- Table `igc-d-vc`, boundaries as printed:

| Branch | Vc (m³) | d (m) |
|---|---|---|
| 2.4.1.1.1 | Vc ≤ 1,000 | 0.8 |
| 2.4.1.1.2 | 1,000 < Vc < 5,000 | 0.75 + Vc × 0.2 / 4,000 |
| 2.4.1.1.3 | 5,000 ≤ Vc < 30,000 | 0.8 + Vc / 25,000 |
| 2.4.1.1.4 | Vc ≥ 30,000 | 2 |

- Consequence `block`; `sizes: true` (the search builds tanks with this clearance).

**Step 4. Units, applicability, citation.** Metres and cubic metres. Applicability: on the Rule Set
`igc-2g` (ship type 2G): Cargo Containment `type_c_tanks`, so a membrane tank (which 2.4.2 measures
differently) or a crude oil tank never gets the Rule. Citation: IGC Code, MSC.370(93) as amended,
2.4.1.2 with 2.4.1.1 and 2.4.2, `MSC.370-93.pdf` annex pp. 20–21.

**Step 5. Status.** `codified` by `agent`; `verified: false` until the second reading; `evaluated: true`.
No finding: the code's `d_vc_exact` has the same four branches and the same boundaries, and since #237
it uses the volume of one tank including the dome.

**Step 6. Tests.** Sourced tests, by hand from the table:

| Vc (m³) | Branch | d (m) |
|---|---|---|
| 1,000 | .1 | 0.8 |
| 1,000.001 | .2 | 0.75 + 1,000.001 × 0.2 / 4,000 = 0.80000005 |
| 4,999 | .2 | 0.75 + 4,999 × 0.2 / 4,000 = 0.99995 |
| 5,000 | .3 | 0.8 + 5,000 / 25,000 = 1.0 |
| 29,999 | .3 | 0.8 + 29,999 / 25,000 = 1.99996 |
| 30,000 | .4 | 2.0 |

The table is continuous (0.8 at 1,000 m³ from both sides, 1.0 at 5,000, 2.0 at 30,000) and never
falls as Vc grows: a test checks both, because a typo in a coefficient breaks one or the other. Harness
cases: for one tank of each branch, a design whose clear gap is 1 mm above d less the insulation and
one 1 mm below, from the oracle.

**The record.**

```json
{
  "id": "igc/2.4.1.2/side-clearance",
  "title": "Cargo tank side distance from the outer shell, at least d(Vc)",
  "origin": "regulation",
  "source": {
    "document": "IGC Code, resolution MSC.370(93)",
    "edition": "MSC.370(93) as amended by MSC.411(97), MSC.441(99), MSC.476(102), MSC.492(104), MSC.523(106), MSC.566(109); none changes 2.4",
    "locator": "2.4.1.2",
    "also": ["2.4.1.1", "2.4.2"],
    "page": "MSC.370-93.pdf pp. 21-22",
    "access": "free"
  },
  "codified": { "by": "agent", "date": "2026-10-20", "review": "#<pull request>" },
  "verified": false,
  "applies_when": {},
  "kind": "formula",
  "measure": { "id": "tank.side_gap", "args": {} },
  "compare": "ge",
  "limit": {
    "shape": "less-input",
    "of": { "table": "igc-d-vc", "at": "tank.vc" },
    "less": "tank.insulation",
    "unit": "m"
  },
  "tolerance": 1e-9,
  "consequence": "block",
  "sizes": true,
  "evaluated": true,
  "finding": null,
  "since": "1.0.0"
}
```

And the table in the same set file:

```json
{
  "id": "igc-d-vc",
  "source": { "locator": "2.4.1.1", "page": "MSC.370-93.pdf pp. 21-22" },
  "input": { "name": "Vc", "unit": "m3" },
  "unit": "m",
  "rows": [
    { "when": { "le": 1000 },              "shape": "const",        "a": 0.8 },
    { "when": { "gt": 1000, "lt": 5000 },  "shape": "a+v*b/c",      "a": 0.75, "b": 0.2, "c": 4000 },
    { "when": { "ge": 5000, "lt": 30000 }, "shape": "a+v/c",        "a": 0.8, "c": 25000 },
    { "when": { "ge": 30000 },             "shape": "const",        "a": 2.0 }
  ]
}
```

**Step 7. Second reading.** The second reader opens `MSC.370-93.pdf` at pages 21–22 and works through
the checklist. On success they add, for example:

```json
"verified": { "by": "expert-reviewer", "date": "YYYY-MM-DD", "edition_held": "MSC.370(93) and amendments to MSC.566(109)", "locator_read": "2.4.1.1, 2.4.1.2, 2.4.2; pp. 21-22", "review": "#<pull request>" }
```
