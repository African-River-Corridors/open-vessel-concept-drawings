# Roadmap

Planned work. Each item links to its issue — comment there to help or to discuss.

## Where this is going

Open Vessel Concept Design starts as a concept design tool. The aim is a **trusted, open preliminary-design
resource for the shipbuilding community**: intact and damage stability, hull strength in waves, basic
construction drawings, a bill of quantities, a costing engine, voyage simulation, better equipment and
accommodation arrangements, tug-barge systems (pushed, towed and articulated tug-barge) with standard tug
designs, and CFD through open solvers. Free to use, open formats, documented methods, no gatekeeping.

**Scope** ([ADR 0011](docs/adr/0011-scope-concept-and-preliminary-design-never-approval.md)): concept and
preliminary design for inland and coastal vessels and tug-barge systems. **Never class approval.** Seagoing
ships under the IACS Common Structural Rules wait until after version 1.0.

**How it earns trust:**
- Every result names its rule set, edition and paragraph, and every coefficient its source.
- Rules are versioned data, not code ([#176](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/176)).
- An **evaluation harness** tests the engine against known designs with known verdicts (below).
- New analyses get a second, independent implementation, as the drawing standard has (ADR 0009).
- Heavy physics runs on proven open solvers (for example OpenFOAM for CFD, Capytaine for wave loads),
  not new solvers written here.
- The maintainer leads. **Expert contributors are welcome as reviewers**: the harness sorts most problems
  by machine, so expert time goes to the cases it cannot sort.

## Evaluation harness

A growing set of **designs whose compliance is already known**, each with its expected verdict per rule,
runs against the engine on every pull request ([#184](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/184), phase 0).

- **Known-good** cases: worked examples from the rules and from textbooks, public approved designs, and
  built reference vessels. **Known-bad** cases: published non-compliant designs, and **boundary mutants** —
  a known-good design moved just across one rule's limit, so each rule is tested from both sides.
- Each result carries a **trace** of every rule checked: rule, citation, inputs, value, limit, margin.
- When the engine disagrees with a known verdict, the harness sorts it: a gap in what the engine can model;
  a **code bug** (the two independent implementations disagree); a **suspect rule** (they agree with each
  other but not with the known verdict); or a known limitation
  ([#121](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/121)). Only what is
  left goes to an expert, and each settled case joins the set.
- Each release reports the false-fail rate, the false-pass rate (held at zero for blocking rules) and
  rule coverage (every rule has at least one passing and one failing case). A new rule ships with its own
  cases.

## Toward version 1.0

In order. Each phase builds on the one before; dates depend on contributors.

| Phase | Version | Theme | Main work | What users get |
|---|---|---|---|---|
| Now | 0.2 | Concept tool | Capacity, gas tank arrangement, hydrostatics, journey de-rating, resistance and power; the desktop app with live results and GA drawings | **Released** as unsigned test builds |
| 0 | 0.3 | Trust the base | Post-port fixes, test gaps (#123, #168), findings register (#121), more calibration cases (#3), PyPI (#1), recover the earlier GA arrangement detail (double bottom, voids, cofferdams, tank cradles), rules structure design (#176), evaluation harness phase 0 (#184), scope statement | `pip install`; a fuller GA; a stated scope |
| 1 | 0.4 | Rules and geometry | Rules engine with versioned rule sets; hull geometry as a sections table ([ADR 0012](docs/adr/0012-hull-geometry-as-a-sections-table.md)) with more Hull Forms (V-bottom, chine); intact stability for every Cargo, with the weather criterion; harness rule trace and automatic sorting | Stability for any cargo and hull form, every criterion cited |
| 2 | 0.5 | Spaces and damage | Compartments, tanks and loading conditions; deterministic damage stability; better equipment and accommodation arrangement; GA drawings from the compartment model; checkable reports | Damage stability, tank tables, loading conditions |
| 3 | 0.6 | Structure and strength | Structural model; still-water and wave bending moment and shear; section modulus; first inland scantling rule set; lightship from the steel weight | Longitudinal strength and scantlings |
| 4 | 0.7 | Build outputs | Midship section, shell expansion, plate and stiffener lists; bill of quantities; signed installers (#122); the web version | Drawings and quantities a yard can price |
| 5 | 0.8 | Systems and voyages | Convoys, towing and articulated tug-barges with a library of standard tugs; resistance, propeller and power; voyage simulation on a real river | Barge and tug designed together; trip time, fuel and cargo |
| 6 | 0.9 | Cost | Build cost, running cost and lifecycle cost from the same design file, with an open rate set | Cost per tonne-km over the vessel's life |
| 7 | 1.0 | Trusted analysis tool | Every claimed rule set at full harness coverage; expert review of what the harness cannot sort; a verification and validation report; translations | A tool a yard, a lender's engineer or a class reviewer can check and cite |
| Later | 1.x | Beyond | CFD and seakeeping through open solvers (research starts earlier, in parallel); probabilistic damage stability; production export; a wider vessel envelope only by a new ADR | High-fidelity physics on free solvers |

## Near term

- **Publish to PyPI** ([#1](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/1)) so it installs with `pip install vessel-designer`.
- **Public example river on an open DEM** ([#2](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/2)) — a worked case on a freely downloadable elevation model
  whose licence allows commercial reuse.
- **Calibrate against more real barges** ([#3](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/3)) — more built vessels with published lightship, capacity
  and draught.
- **Turn remaining baked-in defaults into inputs** ([#4](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/4)) — e.g. block coefficient 0.85 and the stern-block
  size. (Fabrication rates are already data: see `examples/barge_cost_rates.example.yaml`. The tank
  outer diameter cap is already an input, `max_tank_od_m`, and there is no tank-length cap.)
- **Define an Option by a standard Vessel Class** ([#94](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/94)) — pick a class
  (e.g. CEMT Va) instead of typing length and beam. **Done**: length and beam decide the class; picking
  one sets them, the Formation and the draught and air-draft limits.
- **Close the mutation test gaps from the Rust port** ([#123](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/123)) — tests
  only; no result changes.

### After the Rust port

Spec 0001 (`docs/specs/0001-parity-harness-and-rust-core.md`, the Rust port) is done. The plan from
there, in order:

1. Calibration Bands and the calibration check (high priority,
   [#119](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/119)).
   **Done** as a warning only, while the model is in development: a case outside its band is marked in
   `calibration/results.md` and raises a CI warning annotation, and never fails the build.
2. Default moulded-depth rule by Cargo (high priority): fewer inputs for non-experts. **Done**: a left-out
   depth is the beam times a ratio by Cargo (`catalogue/depth_rule.md` lists the reference vessels against it)
   ([#120](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/120)).
3. Review of the known engineering findings and general improvements to the model. The findings are
   listed in [#121](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/121);
   the post-build review of the port ([#113](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/113))
   adds to it. Nothing is agreed yet. Each agreed change comes one at a time, with a before/after table.
4. Formations: convoy totals, then convoy resistance and power.
5. More Hull Forms (V-bottom and others), with configurable parameters.
6. The Tauri desktop app (in parallel with the CLI). **First version done** (spec 0002, version 0.2.1).
   Still to come: US units, translations, the PDF report.
7. The Drawing Format Standard, then GA drawings and DXF in the app. **Done** (spec 0003, version 0.2.1):
   side elevation, plan and midship section, written as SVG, PDF and DXF.
8. Later: construction drawings and plate lists; CFD runs. Now planned in phases 4 and later, above.
9. A web version on the WebAssembly core.
10. The first signed public installer, once the known engineering findings (#121) are reviewed and each is
    fixed or shown in the app as a known limitation. It also needs signing and notarisation
    ([#122](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/122)). Until then,
    unsigned pre-releases (from 0.2.1) are published for trial use, with a first-run notice that results
    are unverified.

Want something else? Open an issue with the `roadmap` label.
