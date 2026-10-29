# 140 x 29 m coastal LNG gas barge: review run

**Concept and preliminary design only. Not verified. Never for approval.** This folder is a worked example run on `main` (`ovcd` 0.2.4, commit f9a4ef7e, 2026-10-29) for an expert review of the tool. The numbers are what the tool prints; nothing here is a design.

## The design

Self-propelled gas barge, IGC Type C tanks, LNG, 140.0 m x 29.0 m, raked box hull, Service Area `coastal-and-river`, tanks at most 12.0 m outer diameter, two tanks along the hull, maximum speed 12 kn. Every other input is the Defaults Profile value (`source: default` in the file). Three Options:

- **A basis**: moulded depth 12.0 m; end spaces 0.60 m aft and 0.50 m forward (the former core defaults, kept as typed inputs: maintainer triage on #280, 2026-10-29).
- **B profile**: as A with the Defaults Profile end spaces (0.20 m, the type G rule value).
- **C deep**: as A at the default gas depth (0.55 x beam = 15.95 m).

`min_freeboard_m` is typed at 1.81 m in every Option. Without it `fit` fails the ICLL tabular-freeboard Rule for every coastal Design, because that Rule reads the safety clearance as the freeboard (finding filed on the register, #121). The river-only stack gives the same results hash as A (sensitivity M).

## Results

| Quantity | A basis | B profile | C deep |
|---|---:|---:|---:|
| Moulded depth (m) | 12.00 | 12.00 | 15.95 |
| Tanks across x along | 2 x 2 | 2 x 2 | 2 x 2 |
| Tank outer diameter, to insulation (m) | 11.00 | 11.00 | 12.00 |
| Side clearance per side (m) | 3.00 | 3.00 | 2.00 |
| Cargo zone (m) | 97.8 | 98.5 | 91.5 |
| Bow void (m) | 19.7 | 19.4 | 26.0 |
| Stern void (m) | 22.5 | 22.1 | 22.5 |
| Cargo volume at 0.98 fill (m3) | 14,355 | 14,469 | 15,890 |
| Cargo (t) | 6,460 | 6,511 | 7,150 |
| Lightship (t) | 6,620 | 6,628 | 8,268 |
| Displacement, laden (t) | 13,115 | 13,174 | 15,457 |
| Draught, laden (m) | 3.96 | 3.97 | 5.06 |
| Draught cap (depth less freeboard) (m) | 10.19 | 10.19 | 14.14 |
| Freeboard, laden (m) | 8.04 | 8.03 | 10.89 |
| Air draft, laden (m) | 11.44 | 11.43 | 14.29 |
| Air draft, light (m) | 13.36 | 13.35 | 16.56 |
| GM fluid, full load departure (m) | 14.28 | 14.21 | 10.00 |
| Max GZ, full load departure (m) | 6.06 | 6.05 | 5.45 |
| GM fluid, ballast departure (m) | 29.51 | 29.46 | 19.12 |
| Installed power (design speed 9.7 kn, margin 1.2) (kW) | 1,717 | 1,717 | 2,022 |
| Service speed, deep water (kn) | 11.3 | 11.3 | 11.9 |
| Power at the service speed (kW) | 1,431 | 1,431 | 1,685 |
| Boil-off (IGC 7.2 ambient) (kg/h) | 731 | 737 | 747 |
| Gas use at the service speed (kg/h) | 261 | 261 | 300 |
| Checks | boil-off within gas use | boil-off within gas use | boil-off within gas use |
| Results hash | afc4b81c0170 | c8b01d222dfc | 74baa7a8d347 |

Power curve, deep water (kW at kn): 8 kn 661; 9 kn 935; 10 kn 1,279; 11 kn 1,701; 12 kn 2,213 (A basis).

Every Option meets every evaluated blocking Rule. Four blocking Rules are not evaluated by the tool: IGC 2.7 damage survival, the IS Code weather criterion, the inland tank-capacity table and the hull-design clause. One warning check is not met in every Option: the boil-off exceeds the gas use at the service speed, so the excess needs reliquefaction, pressure accumulation or a gas combustion unit (IGC 7.1).

## What moves the number (sensitivities file)

| Option | Change from A | Tank OD (m) | Cargo (m3) | Cargo (t) | Draught (m) |
|---|---|---:|---:|---:|---:|
| I lift 9 m | wheelhouse lift cap 6 to 9 m | 12.00 | 17,135 | 7,711 | 4.36 |
| J bow 12 m | bow rake typed 12 m (default 1.6 x depth = 19.2 m) | 11.00 | 15,535 | 6,991 | 3.94 |
| K both | both of the above | 12.00 | 18,554 | 8,349 | 4.36 |
| L depth 14 | as K at depth 14 m | 12.00 | 18,554 | 8,349 | 4.68 |
| M river | Service Area river-only (same hash as A) | 11.00 | 14,355 | 6,460 | 3.96 |

The river blind-distance Rule (ES-TRIN 7.02(2), 250 m) with the 6 m lift cap is what keeps the tank at 11 m instead of the 12 m cap on a 12 m hull. The bow rake default takes 19.2 m of the 140 m. Together they are the difference between 14,355 and 18,554 m3.

## Reproduce

```
ovcd run examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign
ovcd run --json examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign > results.json
ovcd draw --out ga examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign
```

`trace.md` lists every Rule of Option A with its outcome, value, limit, status and citation (from `rule_trace` in `results.json`). `ga/` holds the three GA views of each Option as PDF and SVG.

## Findings from this run

Filed on the register (#121): the ICLL freeboard Measure; `ovcd draw` and Option names over 15 characters; the bow rake default and the river sightline Rule on a coastal hull. See the register for the ticket numbers.
