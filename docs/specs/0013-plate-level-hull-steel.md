# Spec 0013: Plate-level hull steel, and a 3D view of the steel

Status: **agreed with the maintainer, 2026-10-31** (D1–D22, three grill rounds). Parent issue #726. Builds on #722 (the panel model, PR #732), spec 0008
(the Vessel Model; this spec is its phases 3 and 4 and the structural export of phase 6) and spec 0009
(the 3D view). ADR 0011 holds: concept and preliminary design, never approval.

## Problem Statement

#722 replaced a steel rate per m³ with one record per structural member: bottom, inner bottom, side,
inner side, wing tank top, deck, bulkheads, girders, floors and web frames, each with ABS barge minimum
scantlings. That fixed the depth response of the gas lightship. It left five gaps:

- **A panel is not a plate.** A yard prices, and a reviewer checks, plates by thickness and size and
  stiffeners by section and length. A panel has one thickness and an area.
- **Box-only and gas-only.** The ends are a box. The rakes, bilges and Stern Forms in the sections table
  carry no steel. Dry bulk, liquid bulk and deck barges still use older estimates.
- **Old rules, outside the rules engine.** The scantlings cite the ABS barge rules of 2021, a superseded
  edition, as strings in `hull_panels.rs`, not as versioned Rules (spec 0004, ADR 0013). The rule length
  is the LOA (an Estimate).
- **Nobody can see it.** The 3D view has a disabled "Structure" toggle. The midship section drawing
  shows the arrangement, not the scantlings.
- **Nothing leaves OVD.** A structural engineer must re-model the hull to run finite elements.

## Solution

1. **A structure layer in the Vessel Model** (ADR 0018 layer 3; spec 0008 phase 3): a `structure` block
   on the Hull holds frame spacing, longitudinal spacing, stock plate size, thickness step, steel grade
   and the structural wing deck switch. Frames become named reference planes.
2. **Scantlings as Rules** (spec 0008 phase 4): ABS Steel Barges (current edition) for coastal and
   unrestricted service; BV NR217 for inland. One versioned rule set each.
3. **Plates, stiffeners and primary members from the panels.** Each panel is split into plates by seams
   and butts laid out on the stock plate size. Each stiffener and each primary member is its own item.
   Brackets, collars and welds stay the 5 % allowance. No new scantling: the plate model re-cuts the panel
   model, rounds thickness to the step, and adds geometry.
4. **Shell plates follow the sections table**, so rakes, bilges and Stern Forms carry real plate.
5. **One steel authority.** The plate model's ordered weight drives the lightship and the cost; the
   panel table is its roll-up.
6. **Outputs:** a steel list (CSV and JSON) with plate counts by thickness and stock size and a counted
   scrap; weight and centre per member group; a structural midship section drawing; the 3D Structure
   layer.
7. **One export for FE and class: OCX** (Open Class 3D Exchange). A companion project meshes it.

## User Stories

### Designer
1. As a designer, I want a steel list by plate thickness and stock size, and by stiffener section and
   length, so that I can get a budget price from a yard.
2. As a designer, I want the rule-net and the ordered weight, so that I can see what rounding costs.
3. As a designer, I want the weight and centre of each member group, so that I can see what drives the
   lightship.
4. As a designer, I want the steel of the rakes and the stern, so that a short hull is not under-weighed.
5. As a designer, I want to switch the wing deck into the hull girder, so that I can test a gas hull
   whose tanks stand through the deck.
6. As a designer, I want to see the plates and stiffeners in 3D, coloured by thickness, so that I can
   check the structure the numbers describe.

### Reviewer
7. As a reviewer, I want a midship section with each strake's thickness, each longitudinal's section,
   the web frame outline and the hull girder table, so that I can check the scantlings against the rule.
8. As a reviewer, I want every item to name its rule set, edition and paragraph, or say "estimate".
9. As a reviewer, I want openings that are not deducted to be stated, so that I know the list errs heavy.

### Structural engineer
10. As a structural engineer, I want an OCX file of the structure, so that I can mesh it for FE without
    re-modelling.

### Maintainer
11. As a maintainer, I want the plates to sum to the panels, so that the re-cut cannot change the steel
    silently.
12. As a maintainer, I want the scantlings as versioned Rules, so that an edition change is one reviewed
    diff.

## Implementation Decisions

### Terms (OCX vocabulary; see "Research")
- **Panel:** a bounded plane or ruled surface of plating with one stiffening system; today's
  `PanelMember`. Named by OCX `functionType` (AP218 / IACS Rec 82), e.g. "DECK: Strength deck".
- **Plate:** a piece of a Panel between Seams and Butts, with one thickness and grade.
- **Seam / Butt:** a weld line along / across the length that bounds Plates.
- **Stiffener:** a profile on a Panel along a Trace Line, with a Bar Section from the Section Catalogue.
- **Primary Member:** a web frame, floor, girder or transverse that supports Stiffeners; a plated part.
- **Structure Layer:** the `structure` block of a Hull and the frames it sets.
- **Rule-net thickness / ordered thickness:** the rule result / the result rounded up to the step.
- **Stock plate:** the plate size a yard buys (an input; not a mill limit).

### Format (`.vdesign`)
A `structure` block on the Hull (one format version step; old files get the defaults with a note):

| Input | Default | Basis |
|---|---|---|
| `frame_spacing_m` (web frames and floors) | 3.66 | Rule maximum, ABS 5-1-2/5.7.1 (2021; re-check in the current edition) |
| `longitudinal_spacing_mm` | 700 | Estimate inside ABS 5-1-2/5.17's 610–915 mm |
| `stock_plate_m` (width × length) | 3.0 × 12.0 | **Estimate**; checked against mill limits (about 3.95 × 18.7 m, voestalpine; 28 m long, Dillinger) |
| `thickness_step_mm` | 0.5 | **Estimate** |
| `steel_grade` | ordinary strength (Q = 1) | as today |
| `structural_wing_deck` | off | #722 recommendation; conservative default |

### Rules (spec 0004, ADR 0013)
- **Rule set `abs-steel-barges`**, the current edition: Part 3 (hull) January 2026, Part 5 (barge types)
  July 2025. The ticket moves every formula of `hull_panels.rs` into Rules and **re-checks every
  paragraph** against the new edition. Edition differences move results; one design-impact table.
- **Rule length** per the rule's definition, from the hydrostatics, not the LOA.
- **Rule set `bv-nr217`** (June 2025) for inland service, in phase 3.
- Every item carries `rule_set`, `edition`, `paragraph` and `basis` (`rule` or `estimate`).
- **Structural wing deck** (when on): counted in the hull girder where continuous over 0.4L amidships,
  without openings over 2.5 m × 1.2 m, and carried on the inner side and side shell (IACS UR S5;
  ABS 3-2-1/3.3); deck modulus to the S5 y_t rule; scantled as a strength deck. The weather shield stays
  non-structural; the results say it needs expansion joints (ABS 3-2-8/1).

### Model (`ovd-core`, `no_std`)

| Module | Change |
|---|---|
| `hull_panels` | builds from a `HullStructureSpec` assembled from the Vessel Model (sections table, decks, bulkheads, Spaces, `structure`), not from `gas_hull_spec`; reads scantlings from the rule set; member names become `functionType` values |
| `hull_plates` (new) | `plates_of(panel, structure)`, `stiffeners_of(..)`, `primaries_of(..)`. Plates: seams across the girth at the stock width, kept clear of longitudinals; butts along the length at the stock length, on or offset from a frame. Stiffeners: one trace line per spacing. Primary members: one plated item per frame. Each item: id, `PartSource`, geometry, grade, rule-net and ordered thickness or section, weight, centre, citation, basis |
| `vessel_model` | `frames` filled as named reference planes (FR0, FR1, …) from `frame_spacing_m` |
| `sections` / `mesh` | shell strips per station interval (flat or single-curved), the same quads the hull mesh builds |
| `weights`, `cost` | read the plate model's **ordered** weight; the panel table is a roll-up of it |

**Ends.** Shell plates follow the sections table, one strip per station interval: exact on every
generator Hull Form (box, raked box, bilge radius, sheer, camber, Stern Forms). Internal end structure
(peaks, collision and aft peak bulkheads) from the panel rules.

**Openings.** As today: the large openings the model holds (deck tank cut-outs) are deducted.
Lightening holes are not deducted; the list says so and errs heavy. Saddles, foundations, machinery
seats and superstructure steel are later data tickets.

**Scrap** is counted, not factored: stock plates bought (by thickness and stock size) less net plate area.

### Results
- `structure.steel_list`: one row per plate, stiffener and primary member.
- `structure.groups`: weight and centre per member group; rule-net and ordered totals; allowance; scrap.
- `structure.purchase`: stock plates by thickness and size; stiffener metres by section.
- Weights and centres are in the results hash (they drive the lightship). Geometry is outside it, like
  meshes (ADR 0018).

### Outputs
- **Steel list:** `ovd export --format steel-csv` and the JSON above.
- **Midship section drawing** (phase 2): plate thickness per strake, each longitudinal with its section,
  the web frame and floor outline, the hull girder table (SMR, SM to deck and bottom, neutral axis), the
  rule edition and the ADR 0011 scope note. Shell expansion later.
- **3D Structure layer** (enables spec 0008's toggle): plates as thin solids at true (ordered) thickness,
  coloured by thickness band, with a legend that isolates a band; stiffeners and webs as extruded
  sections; pick shows the steel-list row; a Structure view preset. Behaviour after the open
  `ocx-render` viewer (see "Research").
- **OCX export** (phase 2): OVD keeps its own parametric model, named and structured on OCX concepts;
  one writer, pinned to OCX **v3.2.0** (the latest released tag; a new release moves by a reviewed
  ticket). Rule citation and basis as `CustomProperty`. A CI check in `tools/` runs the open
  `ocx-platform` (MIT, Python) to validate against the XSD and read the file back to a bill of materials
  equal to OVD's steel list. Nothing OCX is linked into OVD.
- **OCX oracles:** a vendor one-hold sample model checks OVD's weight arithmetic; one interoperability
  test case checks export conformance. Both are fetched by URL and SHA-256 in the test, never copied
  into the repo, and never used as steel calibration (a seagoing tanker is outside ADR 0011).

### Calibration
- Identity checks only at plate level (below).
- Today's lightship residuals (15 inland tank barges, the bunker vessel concept) re-reported after the
  switch, **never tuned**.
- A data ticket to find a published midship section or steel list as a plate-level benchmark.

## Testing Decisions
- **Identity:** plate areas sum to their panel's area to 0.1 %; stiffener count = girth / spacing, as the
  panel model rounds; plate model rule-net weight = panel weight.
- **Rounding:** ordered thickness ≥ rule-net, < rule-net + step, on every plate.
- **Scrap:** stock area bought − net area = scrap, exactly.
- **Geometry oracle:** shell plate area on the raked box = its closed-form surface area.
- **Rules:** each scantling Rule has a worked example from the rule edition; the rule trace names it.
- **Wing deck:** off reproduces today's hull girder; on, the depth sweep stops thickening the deck strips
  below about D = 10 m on the 140 × 29 m example (#722).
- **OCX:** every export validates against the pinned XSD; reading it back gives the steel list.
- **3D:** spec 0009 source test and triangle budget cover the Structure layer.
- **Design impact:** one table when the lightship switches; one when the ABS edition moves.
- `ovd-core` stays `no_std`; `bash tools/gates.sh` passes.

## Out of Scope
FE analysis, fatigue, buckling past the rule minimum, approval drawings, nesting and cutting files, weld
details, bevels, production sequence. Longitudinal strength (spec 0008 phase 5). Saddles, foundations,
machinery seats, superstructure steel, lightening holes (data tickets).

## Phases
- **P1:** structure inputs; ABS current-edition rule set; plate model on the gas hull (midbody and ends);
  steel list; lightship switch; 3D Structure layer; structural wing deck input.
- **P2:** midship section drawing; OCX export; other vessel types under ABS.
- **P3:** BV NR217 inland rule set.

## Research

Searched: open structural modelling and exchange formats, open FE and nesting tools, concept-design
steel methods, class scantling sources, mill plate sizes, and OVD's own panel model, 3D view and
calibration data.

- **OCX (Open Class 3D Exchange)**, [OCXStandard/OCX_Schema](https://github.com/OCXStandard/OCX_Schema),
  Apache-2.0, released v3.2.0, working draft 3.3.0. Built for class approval; design tools export it. A
  Panel has a moulded surface, the parts that bound it, its Plates, its Stiffeners and its Seams; a
  Stiffener is a trace line on the panel with a section reference, a web direction and a material;
  frames are named reference planes; sections and materials are catalogues by id; `functionType`
  follows STEP AP218 and IACS Rec 82. It holds finished geometry (NURBS), one element per part, and no
  parameters, rule citations or estimate labels. So it fits OVD as an export and a vocabulary, not as
  the internal model (ADR 0018 keeps parameters as master). The schema changes between versions
  (2.8.6 to 3.3.0 dropped the frame tables), so the export pins one release.
- **OCX tools:** `ocx-platform` (MIT, Python ≥ 3.12): parse, XSD validation, bill of materials, a
  three.js renderer coloured by thickness. `ocx-schema-parser`, `ocx-reader`, `ocxtools` (MIT, Python),
  `OCXViewer` (Apache-2.0, Java), `ocx-validator` (EUPL-1.2, docker). None can be linked into the
  `no_std` core or the WebAssembly app; `ocx-platform` can run as a CI check.
- **OCX sample models:** a vendor one-hold tanker model (523 plates, 3,963 stiffeners; every plate
  weight equals net area × thickness × 7,850 kg/m³) and interoperability test rounds exported by
  several design tools. Test oracles, not steel calibration: none is a barge, and a seagoing tanker is
  outside ADR 0011.
- **IFC 4.3** (`IfcPlate`, `IfcMember`): no frame system, no stiffener trace, no class data. Rejected.
  **STEP AP218** (ISO 10303-218:2004): paid text, little open tooling. Rejected.
- **FE and nesting:** Gmsh (GPL), CalculiX (GPL), Deepnest (MIT): companion or production tools,
  outside ADR 0011. Not linked.
- **Rules:** ABS Steel Barges: the 2025 edition was a major revision; Part 3 is January 2026 and Part 5
  July 2025 (ABS notices, January 2026). `hull_panels.rs` cites 2021. BV NR217 (June 2025) and the ABS
  River Rules (January 2025) are free to read. IACS UR S5 and S11 set the effective hull girder members.
- **Mill limits:** shipbuilding heavy plate up to about 3.95 m × 18.7 m (voestalpine) and 28 m long
  (Dillinger). A yard's stock size is its own choice; no open source gives a typical one, so the stock
  size is an input with an Estimate default.
- **Calibration:** no open plate-level steel list of a barge was found. The library's 15 inland tank
  barges give implied steel only.
- **No open tool** builds plates and stiffeners from class rules for a parametric concept hull. That
  generator is OVD's own work.

## Decisions taken and open questions

**Round 1, 2026-10-30**:
1. **D1** This spec is spec 0008 phases 3 and 4 plus the structural export of phase 6; phase 5 stays
   separate.
2. **D2** Plates by seams and butts; each stiffener and primary member an item; brackets, collars and
   welds the 5 % allowance (Estimate).
3. **D3** Stop at a steel list good for a yard budget quote; no nesting, cutting files, welds or bevels.
4. **D4** ABS Steel Barges coastal and unrestricted; BV NR217 inland; one versioned rule set each.
5. **D5** ABS moves to the current edition when its rule set is built; every paragraph re-checked.
6. **D6** Shell plates follow the sections table; internal end structure from the panel rules.
7. **D7** 3D: thin solids at true thickness, coloured by band; extruded stiffeners and webs; pick;
   Structure preset.
8. **D8** OCX export; a companion project meshes it for FE.
9. **D9** The plate model is the one steel authority; panel table its roll-up.

**Round 2, 2026-10-31**:
10. **D10** Thickness rounds up to a step (input, default 0.5 mm, Estimate); rule-net and ordered weight
    reported; ordered drives lightship and cost.
11. **D11** Stock plate size an input, default 3.0 × 12.0 m (Estimate), checked against mill limits.
12. **D12** Scrap counted from stock plates bought; no factor.
13. **D13** Openings as today; lightening holes not deducted (stated); saddles, foundations, seats and
    superstructure steel are data tickets.
14. **D14** A `structure` block on the Hull in `.vdesign`; one format version step.
15. **D15** Structural wing deck: an input, default off; scantled as a strength deck when on.
16. **D16** Calibration: identities plus re-reported lightship residuals, never tuned; a benchmark data
    ticket.
17. **D17** The midship section drawing content above; shell expansion later.
18. **D18** Phases P1, P2, P3 as above.

**Round 3, 2026-10-31** (the OCX question):
19. **D19** OCX is the export and the vocabulary, not the internal model.
20. **D20** The export pins OCX v3.2.0; a new release moves by a reviewed ticket.
21. **D21** CI checks the export with the open `ocx-platform` (validate, read back, compare to the steel
    list); nothing OCX is linked into OVD.
22. **D22** Vendor sample models are test oracles only (weight arithmetic, export conformance), fetched by
    URL and SHA-256; never calibration.

## Build tickets (proposed; filed after review)

| # | Ticket | Phase | Blocked by |
|---|---|---|---|
| T1 | `structure` block on the Hull in `.vdesign`; format version step; defaults with basis; frames as named reference planes in the Vessel Model | P1 | — |
| T2 | Rule set `abs-steel-barges`, current edition: move `hull_panels` formulas into Rules; rule length per the rule; re-check every paragraph; design-impact table | P1 | — |
| T3 | `HullStructureSpec` from the Vessel Model; `functionType` names; gas hull first | P1 | T1, T2 |
| T4 | `hull_plates`: plates by seams and butts on stock size, stiffener and primary items, thickness rounding, counted scrap, steel list and groups, identity tests | P1 | T3 |
| T5 | Ends: shell strips from the sections table; peak and collision bulkhead structure | P1 | T4 |
| T6 | Structural wing deck input (IACS S5, ABS 3-2-1/3.3) | P1 | T2, T3 |
| T7 | Lightship and cost read the plate model's ordered weight; panel roll-up; design-impact table; residuals re-reported | P1 | T4, T5 |
| T8 | 3D Structure layer: meshes, thickness legend and isolate, pick, Structure preset, triangle budget | P1 | T4 |
| T9 | Structural midship section drawing | P2 | T4 |
| T10 | OCX export pinned to v3.2.0; CustomProperty provenance; `tools/` CI check with ocx-platform (validate, read back, compare); VLCC arithmetic and one interoperability test case as oracles, fetched by URL and SHA-256 | P2 | T4 |
| T11 | Other vessel types under ABS: dry bulk and hopper, liquid bulk, deck barge | P2 | T4, T7 |
| T12 | Rule set `bv-nr217` and inland service selection | P3 | T2, T11 |
| T13 | Data: a published midship section or steel list as a plate-level benchmark | data | — |
| T14 | Data: tank saddles and foundations | data | T4 |
| T15 | Data: machinery seats, engine room and casing, superstructure and wheelhouse steel | data | T4 |
| T16 | Data: lightening holes and openings | data | T4 |
