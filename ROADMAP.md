# Roadmap

Planned work. Each item links to its issue — comment there to help or to discuss.

- **Publish to PyPI** ([#1](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/1)) so it installs with `pip install vessel_designer`.
- **Public example river on an open DEM** ([#2](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/2)) — a worked case on a freely downloadable elevation model
  whose licence allows commercial reuse.
- **Calibrate against more real barges** ([#3](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/3)) — more built vessels with published lightship, capacity
  and draught.
- **Turn remaining baked-in defaults into inputs** ([#4](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/4)) — e.g. block coefficient 0.85, the 28 m tank-length
  cap and the stern-block size. (Fabrication rates are already data: see
  `examples/barge_cost_rates.example.yaml`.)

## After the Rust port

The plan once spec 0001 (`docs/specs/0001-parity-harness-and-rust-core.md`) is done, in order:

1. Calibration Bands and the calibration check (high priority).
2. Default moulded-depth rule by Cargo (high priority): fewer inputs for non-experts.
3. Review of the known engineering findings and general improvements to the model. Nothing is agreed yet. Each agreed change comes one at a time, with a before/after table.
4. Formations: convoy totals, then convoy resistance and power.
5. More Hull Forms (V-bottom and others), with configurable parameters.
6. The Tauri desktop app (in parallel with the CLI), US units, translations, the PDF report.
7. The Drawing Format Standard, then GA drawings and DXF in the app.
8. Later: construction drawings and plate lists; CFD runs.
9. A web version on the WebAssembly core.
10. The first public installer, once the known engineering findings are reviewed and each is fixed or shown in the app as a known limitation.

Want something else? Open an issue with the `roadmap` label.
