# Open Vessel Concept Design (OVCD)

An open instrument for the concept and preliminary design of inland and coastal vessels and
tug-barge systems. Give it a hull and a cargo; it gives you capacity, draught, weights, tank
arrangement, stability, resistance and power, and general-arrangement drawings — each result with
its checks, checked against real vessels.

**Website and docs:** [africanrivercorridors.com/ovcd](https://www.africanrivercorridors.com/ovcd) ·
**Roadmap:** [ROADMAP.md](ROADMAP.md) · **Decisions:** [architecture decision records](docs/adr/0011-scope-concept-and-preliminary-design-never-approval.md)

## Ways to use it

- **Desktop app** (Windows and macOS): live results as you type, drawings to the Drawing Format
  Standard (SVG, PDF and DXF), stability curves, rule checks and a capital budget. See
  [Run the desktop app](#run-the-desktop-app).
- **Web version — coming.** The same Rust core compiled to WebAssembly, running in your browser
  with nothing to install; your designs stay on your computer. Planned on the [roadmap](ROADMAP.md).
- **Command line** (`ovcd`): create, check, run and draw `.vdesign` files.
- **Python library** (`vessel_designer`): the calculation core from Python.

All of them use one compiled Rust calculation core (`ovcd-core`: pure calculation, no I/O), so they
give the same numbers.

## What it is, and what it is not

**It is** a concept and preliminary design tool for inland and coastal vessels and tug-barge
systems; a calculation core you can read, where results name their checks and sources; calibrated
against real barges and published design points, with the cases in the open; free, with open
formats and documented methods.

**It is not** class approval, a stability booklet or construction approval, and it never will be
([ADR 0011](docs/adr/0011-scope-concept-and-preliminary-design-never-approval.md)). It is not a
substitute for a naval architect, a yard design office or a class surveyor. It is not yet verified:
known engineering findings are under review in the open
([#121](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/121)).

## What it does

- **Standard vessel classes.** A catalogue of standard inland classes (CEMT, the Dutch 2011 update,
  Chinese GB 38030, US and Paraná sizes) in `catalogue/`, and a typed lookup of the common ones in
  `vessel_designer.core.vessel_standards`.
- **Capacity.** Volume-bound cargo for dry-bulk holds (sloped hopper, box, container) and for
  liquefied-gas barges with IMO Type C tanks (`core.barge_capacity`).
- **Gas tank arrangement.** Protective-location clearances under IGC type 2G (the distance d(Vc) of
  IGC 2.4.1.1 per tank, dome included in Vc, measured plate to plate per 2.4.2), a tank outer diameter cap (an input, 16 m by default) and tank length at most 7
  diameters, and a search over depth and tanks abreast for the most cargo at a draught limit
  (`core.gas_arrangement`). Only IGC rules apply to gas tanks.
- **Hydrostatics and stability.** Equilibrium draught, freeboard, air draught, and intact stability
  for gas barges (`core.hydrostatics`, `core.gas_stability`).
- **Journey de-rating.** Cargo per trip when the channel, not the hull, limits the draught
  (`core.journey_derating`).
- **Resistance, squat, waves and power** in deep and confined shallow water (`core.resistance`,
  `core.squat`, `core.waves`, `core.electric_energy`).
- **Calibration** against real barges and published design points (`calibration/`).

## Install

Python 3.12 or later.

```
pip install git+https://github.com/African-River-Corridors/open-vessel-concept-drawings
```

## Run the desktop app

The desktop app is a concept design tool with live results and drawings. It needs no Python.
The first start shows a short notice: results are not verified and there is no warranty.

**Download an installer.** Open the newest release on the
[releases page](https://github.com/African-River-Corridors/open-vessel-concept-drawings/releases)
and take the file for your computer. A release named `v<version>` is a numbered release; one named
`test-...` is a test build of work in progress.

- Windows: `OVCD_<version>_x64-setup.exe`
- macOS, Apple silicon: `OVCD_<version>_aarch64.dmg`
- macOS, Intel: `OVCD_<version>_x64.dmg`

`SHA256SUMS.txt` on the same page lets you check the download. The installers are not signed, so
your system warns you once:

- Windows SmartScreen says "Windows protected your PC". Click **More info**, then **Run anyway**.
- macOS says the app "can't be verified" (the download is not signed). Open the `.dmg`, drag OVCD to
  Applications, then open it once from **System Settings → Privacy & Security → Open Anyway**. If
  that does not work, run `xattr -dr com.apple.quarantine /Applications/OVCD.app` in Terminal and
  open OVCD again.

**Build from source.** You need Git, Node 24 and Rust (install with [rustup](https://rustup.rs)).
On Windows, rustup also asks for the Microsoft C++ build tools (MSVC); accept that. Then:

```
git clone https://github.com/African-River-Corridors/open-vessel-concept-drawings
cd open-vessel-concept-drawings/app
npm ci
npx tauri dev       # run the app
npx tauri build     # make the installer (target/release/bundle)
```

## Worked example

Size a liquefied-propane barge on a standard Chinese Class II hull (`examples/quickstart.py`):

```python
from vessel_designer.core.barge_capacity import BargeCapacityInputs, compute
from vessel_designer.core.gas_arrangement import fit
from vessel_designer.core.vessel_standards import get_standard

cls = get_standard("CN_II_motor")
loa, beam = cls.loa_m[1], cls.beam_m

# One capacity run: the core picks a tank arrangement for this hull.
auto = compute(BargeCapacityInputs(loa=loa, beam=beam, depth=6.5, cargo_type="LPG_propane"))

# Search the arrangement for the most cargo per journey at a 4.4 m draught cap.
best = fit(loa, beam, "LPG_propane", 4.4, ruleset="IGC_2G", depths=(6.5,))
```

Output:

```
Class CN-GB50139 CN-II-MOTOR: 90 m x 14.8 m, 2,000 t class deadweight
Single run : 1 tanks abreast, OD 13.70 m, 3,278 t, laden draught 4.29 m
Searched   : 1 tank(s) abreast, OD 10.70 m, 2,021 t (volume-bound), draught 3.20 m
```

The single run fills the tank volume and does not look at the draught cap or the checks. The search
respects the cap and keeps only arrangements that pass every blocking Rule, stability and the
wheelhouse sightline included. The larger tanks are too tender when laden or hide the view from the
raised wheelhouse, so the search carries 2,021 t in a 10.70 m tank. If no arrangement passed, `feasible` would be false and
`failed_checks` would list the checks the closest one fails. Tank side clearance follows IGC Code
2.4.1.1 (the distance d(Vc), per tank, dome included in Vc), measured plate to plate with the
insulation inside d (2.4.2). A check that is only advisory, a warning, does not stop an arrangement
passing; it stays in the results as a failed check.

## Layout

| Path | Holds |
|---|---|
| `src/vessel_designer/core/` | the calculation core |
| `src/vessel_designer/simulate.py` | one-call JSON-in, JSON-out simulation |
| `catalogue/` | standard vessel classes and reference vessels, with sources |
| `calibration/` | calibration cases, fitted factors and results |
| `crates/` | the Rust core (`ovcd-core`), file format and Design session (`ovcd-format`), drawings (`ovcd-draw`) and CLI |
| `app/` | the desktop app (Tauri) |
| `website/` | the website and docs ([africanrivercorridors.com/ovcd](https://www.africanrivercorridors.com/ovcd)) |
| `docs/adr/` | architecture decisions |
| `GLOSSARY.md` | the domain language |

## Contributing

Issues and pull requests are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) and the
[roadmap](ROADMAP.md). Please follow the [code of conduct](CODE_OF_CONDUCT.md). Report security
issues privately — see [SECURITY.md](SECURITY.md).

## Licence

Apache License 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
Copyright 2026 Mobile Clipboard Limited (trading as African River Corridors).
