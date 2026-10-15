# Roadmap

Planned work. Each item links to its issue — comment there to help or to discuss.

- **Publish to PyPI** ([#1](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/1)) so it installs with `pip install vessel-designer`.
- **Public example river on an open DEM** ([#2](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/2)) — a worked case on a freely downloadable elevation model
  whose licence allows commercial reuse.
- **Calibrate against more real barges** ([#3](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/3)) — more built vessels with published lightship, capacity
  and draught.
- **Turn remaining baked-in defaults into inputs** ([#4](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/4)) — e.g. block coefficient 0.85 and the stern-block
  size. (Fabrication rates are already data: see `examples/barge_cost_rates.example.yaml`. The tank
  outer diameter cap is already an input, `max_tank_od_m`, and there is no tank-length cap.)
- **Define an Option by a standard Vessel Class** ([#94](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/94)) — pick a class
  (e.g. CEMT Va) instead of typing length and beam. Waits for the maintainer's answers to its open questions.
- **Close the mutation test gaps from the Rust port** ([#123](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/123)) — tests
  only; no result changes.

## After the Rust port

Spec 0001 (`docs/specs/0001-parity-harness-and-rust-core.md`, the Rust port) is done. The plan from
there, in order:

1. Calibration Bands and the calibration check (high priority,
   [#119](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/119)).
   **Done** as a warning only, while the model is in development: a case outside its band is marked in
   `calibration/results.md` and raises a CI warning annotation, and never fails the build.
2. Default moulded-depth rule by Cargo (high priority): fewer inputs for non-experts
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
8. Later: construction drawings and plate lists; CFD runs.
9. A web version on the WebAssembly core.
10. The first signed public installer, once the known engineering findings (#121) are reviewed and each is
    fixed or shown in the app as a known limitation. It also needs signing and notarisation
    ([#122](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/122)). Until then,
    unsigned pre-releases (from 0.2.1) are published for trial use, with a first-run notice that results
    are unverified.

Want something else? Open an issue with the `roadmap` label.
