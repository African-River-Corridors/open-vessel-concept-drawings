# vessel_designer

Concept design of inland and coastal cargo vessels for feasibility studies. Give it a hull and a
cargo; it gives you capacity, draught, weights, tank arrangement, stability, resistance and power —
checked against real vessels.

It is a Python library over a compiled Rust calculation core (pure calculation, no I/O): the Python
imports, names and result types are unchanged, and the numbers come from `ovcd-core`. A desktop app
is planned; see the [roadmap](ROADMAP.md).

## What it does

- **Standard vessel classes.** A catalogue of standard inland classes (CEMT, the Dutch 2011 update,
  Chinese GB 38030, US and Paraná sizes) in `catalogue/`, and a typed lookup of the common ones in
  `vessel_designer.core.vessel_standards`.
- **Capacity.** Volume-bound cargo for dry-bulk holds (sloped hopper, box, container) and for
  liquefied-gas barges with IMO Type C tanks (`core.barge_capacity`).
- **Gas tank arrangement.** Protective-location clearances under IGC type 2G and ADN type G, and a
  search over depth and tanks abreast for the most cargo at a draught limit (`core.gas_arrangement`).
- **Hydrostatics and stability.** Equilibrium draught, freeboard, air draught, and intact stability
  for gas barges (`core.hydrostatics`, `core.gas_stability`).
- **Journey de-rating.** Cargo per trip when the channel, not the hull, limits the draught
  (`core.journey_derating`).
- **Resistance, squat, waves and power** in deep and confined shallow water (`core.resistance`,
  `core.squat`, `core.waves`, `core.electric_energy`).
- **Calibration** against real barges and published design points (`calibration/`).

Concept stage only. It does not replace class approval, a stability booklet or a yard design.

## Install

Python 3.12 or later.

```
pip install git+https://github.com/African-River-Corridors/open-vessel-concept-drawings
```

## Run the desktop app

The desktop app is a concept design tool with live results and drawings. It needs no Python.
The first start shows a short notice: results are not verified and there is no warranty.

**Download an installer.** Open the [latest build](https://github.com/African-River-Corridors/open-vessel-concept-drawings/releases/tag/latest)
(rebuilt after every change to the code) or a numbered release from the
[releases page](https://github.com/African-River-Corridors/open-vessel-concept-drawings/releases),
and take the file for your computer:

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
Class CN II: 90 m x 14.8 m, 2,000 t class deadweight
Single run : 2 tanks abreast, OD 5.90 m, 1,156 t, laden draught 2.31 m
Searched   : 1 tank(s) abreast, OD 10.80 m, 2,058 t (volume-bound), draught 3.42 m
  check failed: wheelhouse_sees_over_cargo: raised eye 14.00 m >= obstruction 14.80 m + 0.50 m margin (lift used 6.00 m of 6.00 m)
```

The single run settles on two small tanks; the search finds one large tank carries 78 % more. Every
result carries its checks. Here the wheelhouse cannot see over the large tank, so a real design must
raise the eye or accept a smaller tank.

## Layout

| Path | Holds |
|---|---|
| `src/vessel_designer/core/` | the calculation core |
| `src/vessel_designer/simulate.py` | one-call JSON-in, JSON-out simulation |
| `catalogue/` | standard vessel classes and reference vessels, with sources |
| `calibration/` | calibration cases, fitted factors and results |
| `docs/adr/` | architecture decisions |
| `GLOSSARY.md` | the domain language |

## Contributing

Issues and pull requests are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) and the
[roadmap](ROADMAP.md). Please follow the [code of conduct](CODE_OF_CONDUCT.md). Report security
issues privately — see [SECURITY.md](SECURITY.md).

## Licence

Apache License 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
Copyright 2026 Mobile Clipboard Limited (trading as African River Corridors).
