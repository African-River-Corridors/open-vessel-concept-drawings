<!-- unreleased -->
- Resistance (issue #735, ADR 0024): a fourth `resistance_method`, `barge_form_factor`: ITTC-1957 friction times (1 + k1 + dk + w), with the Holtrop 1984 form factor, the Millward (1989) shallow-water increase dk = 0.644 (T/h)^1.72 and the Froude-number rise of the bluff-barge fit. Checked against 22 barge towing-tank points (`calibration/resistance_cases.csv`, `calibration/resistance_results.md`): mean absolute error 17 %, against 26 % for Holtrop-Mennen and 32 % for the barge fit, which reads low on every point. Its declared range is that of the data (Fn up to 0.137, Cp 0.768 to 0.877, L/B 7.59 to 16.29, B/T 4.05 to 8.12, h/T from 1.2); Rule `ovd/resistance/method-in-range` warns outside it, now including a water depth below 1.2 times the draught.
- **Default resistance method** (issue #735): `barge_form_factor` is the default for new Designs; it is 17 % mean error against 22 published barge points; outside its range the speeds are indicative. `barge_fit` stays selectable. Saved Designs do not move: a file without `resistance_method` reads `barge_fit` (its legacy value), and a file that stores a method keeps it. A new Design's resistance is higher than under the barge fit, so its service speed falls or its sized power rises (the 140 x 29 m LNG hull of Cb 0.96: 8.71 kn at 1,729 kW instead of 11.19 kn). The design-impact case designs are filled as new Designs, so they move: 253 of 254, by a higher sized tug power and a new `resistance method in range` check (table below).
- Example `gas-140x29-coastal-lng`: the Cb sweep again as Options P1 to P9, with `barge_form_factor`. Every P Option is outside the method's range (L/B and Fn).

<!-- design-impact -->
### feat/resistance-barge-calibration (2026-10-30)

Between `main 06f4a35` and `this branch bb0b606`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| calibration amaggi-box-2000 | `checks[10].name` | Stability — ballast, arrival | Stability — ballast, departure | changed |
| calibration amaggi-box-2000 | `checks[11].message` | GM 15.93 m (fluid), max GZ 2.81 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | GM 6.42 m (fluid), max GZ 1.85 m at 25° | changed |
| calibration amaggi-box-2000 | `checks[11].name` | Stability — lightship, no ballast | Stability — ballast, arrival | changed |
| calibration amaggi-box-2000 | `checks[11].ok` | False | True | changed |
| calibration amaggi-box-2000 | `checks[12].message` | absent | GM 15.93 m (fluid), max GZ 2.81 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | changed |
| calibration amaggi-box-2000 | `checks[12].name` | absent | Stability — lightship, no ballast | changed |
| calibration amaggi-box-2000 | `checks[12].ok` | absent | False | changed |
| calibration amaggi-box-2000 | `checks[12].severity` | absent | warning | changed |
| calibration amaggi-box-2000 | `checks[6].message` | Residuary Cr/Cf fit, Lackenby shallow-water, blockage and 1.02 form factor are uncalibrated first-order estimates (RS1) — ITTC-1957 friction is standard. | barge form factor at the service speed: Fn 0.212, Cp 0.952, L/B 5.61, B/T 3.00; outside its range: Fn 0.21 (range 0.00 to 0.14), Cp 0.95 (range 0.77 to 0.88), L/B 5.61 (range 7.59 to 16.29), B/T 3.00 (range 4.05 to 8.12) | changed |
| calibration amaggi-box-2000 | `checks[6].name` | coefficients are first order | resistance method in range[Deep water] | changed |
| calibration amaggi-box-2000 | `checks[6].ok` | True | False | changed |
| calibration amaggi-box-2000 | `checks[7].message` | GM 1.70 m (fluid), max GZ 0.31 m at 14° — fails: Area 30–40° 0.019 < 0.03, GZ at ≥ 30° 0.176 < 0.2, Angle of max GZ 14.000 < 25.0 | Residuary Cr/Cf fit, Lackenby shallow-water, blockage and 1.02 form factor are uncalibrated first-order estimates (RS1) — ITTC-1957 friction is standard. | changed |
| calibration amaggi-box-2000 | `checks[7].name` | Stability — full load, departure | coefficients are first order | changed |
| calibration amaggi-box-2000 | `checks[7].ok` | False | True | changed |
| calibration amaggi-box-2000 | `checks[8].name` | Stability — full load, arrival | Stability — full load, departure | changed |
| calibration amaggi-box-2000 | `checks[9].message` | GM 6.42 m (fluid), max GZ 1.85 m at 25° | GM 1.70 m (fluid), max GZ 0.31 m at 14° — fails: Area 30–40° 0.019 < 0.03, GZ at ≥ 30° 0.176 < 0.2, Angle of max GZ 14.000 < 25.0 | changed |
| calibration amaggi-box-2000 | `checks[9].name` | Stability — ballast, departure | Stability — full load, arrival | changed |
| calibration amaggi-box-2000 | `checks[9].ok` | True | False | changed |
| calibration amaggi-box-2000 | `formation.notes[1]` | Coupling force between the pusher and the pushed craft: 208 kN (ES-TRIN 2025/1, ESI-III-6 2.1: 270 x P_B x L_S / B_S, with P_B 476 kW, L_S 9.5 m taken as the pusher length, B_S 5.8 m). | Coupling force between the pusher and the pushed craft: 260 kN (ES-TRIN 2025/1, ESI-III-6 2.1: 270 x P_B x L_S / B_S, with P_B 571 kW, L_S 10.7 m taken as the pusher length, B_S 6.3 m). | changed |
| calibration amaggi-box-2000 | `formation.notes[2]` | Service margins, shown and not used to size the tug: the still-water power in deep water at the design speed, 397 kW, x the sea margin 1.15 x the engine margin 1/0.90 (MAN B&W, Basic Principles of Ship Propulsion, p. 27 and Fig. 17) = 507 kW, for an installed power of 476 kW. | Service margins, shown and not used to size the tug: the still-water power in deep water at the design speed, 476 kW, x the sea margin 1.15 x the engine margin 1/0.90 (MAN B&W, Basic Principles of Ship Propulsion, p. 27 and Fig. 17) = 608 kW, for an installed power of 571 kW. | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[10].name` | Stability — ballast, arrival | Stability — ballast, departure | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[11].message` | GM 9.26 m (fluid), max GZ 1.81 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | GM 3.96 m (fluid), max GZ 1.22 m at 26° | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[11].name` | Stability — lightship, no ballast | Stability — ballast, arrival | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[11].ok` | False | True | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[12].message` | absent | GM 9.26 m (fluid), max GZ 1.81 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[12].name` | absent | Stability — lightship, no ballast | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[12].ok` | absent | False | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[12].severity` | absent | warning | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[6].message` | Residuary Cr/Cf fit, Lackenby shallow-water, blockage and 1.02 form factor are uncalibrated first-order estimates (RS1) — ITTC-1957 friction is standard. | barge form factor at the service speed: Fn 0.213, Cp 0.963, L/B 7.81, B/T 2.78; outside its range: Fn 0.21 (range 0.00 to 0.14), Cp 0.96 (range 0.77 to 0.88), B/T 2.78 (range 4.05 to 8.12) | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[6].name` | coefficients are first order | resistance method in range[Deep water] | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[6].ok` | True | False | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[7].message` | GM 0.63 m (fluid), max GZ 0.19 m at 19° — fails: Area 0–40° 0.085 < 0.09, Area 30–40° 0.015 < 0.03, GZ at ≥ 30° 0.138 < 0.2, Angle of max GZ 19.000 < 25.0 | Residuary Cr/Cf fit, Lackenby shallow-water, blockage and 1.02 form factor are uncalibrated first-order estimates (RS1) — ITTC-1957 friction is standard. | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[7].name` | Stability — full load, departure | coefficients are first order | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[7].ok` | False | True | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[8].name` | Stability — full load, arrival | Stability — full load, departure | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[9].message` | GM 3.96 m (fluid), max GZ 1.22 m at 26° | GM 0.63 m (fluid), max GZ 0.19 m at 19° — fails: Area 0–40° 0.085 < 0.09, Area 30–40° 0.015 < 0.03, GZ at ≥ 30° 0.138 < 0.2, Angle of max GZ 19.000 < 25.0 | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[9].name` | Stability — ballast, departure | Stability — full load, arrival | changed |
| calibration canal-box-200x25x12-pb2230 | `checks[9].ok` | True | False | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.notes[1]` | Coupling force between the pusher and the pushed craft: 128 kN (ES-TRIN 2025/1, ESI-III-6 2.1: 270 x P_B x L_S / B_S, with P_B 319 kW, L_S 7.3 m taken as the pusher length, B_S 4.9 m). | Coupling force between the pusher and the pushed craft: 153 kN (ES-TRIN 2025/1, ESI-III-6 2.1: 270 x P_B x L_S / B_S, with P_B 370 kW, L_S 8.0 m taken as the pusher length, B_S 5.2 m). | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.notes[2]` | Service margins, shown and not used to size the tug: the still-water power in deep water at the design speed, 266 kW, x the sea margin 1.15 x the engine margin 1/0.90 (MAN B&W, Basic Principles of Ship Propulsion, p. 27 and Fig. 17) = 340 kW, for an installed power of 319 kW. | Service margins, shown and not used to size the tug: the still-water power in deep water at the design speed, 308 kW, x the sea margin 1.15 x the engine margin 1/0.90 (MAN B&W, Basic Principles of Ship Propulsion, p. 27 and Fig. 17) = 394 kW, for an installed power of 370 kW. | changed |
| ... | 119067 more changed quantities | | | |
