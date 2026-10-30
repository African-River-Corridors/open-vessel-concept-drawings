# Resistance calibration: full-form barges (issue #735)

`resistance_cases.csv` holds 22 towing-tank points of full-form barge hulls, each with its source.
`cargo test -p ovd-core --test resistance_calibration -- --nocapture` runs the methods of issue #723 and
the new `barge_form_factor` against them and prints the tables below. The pass compares at
**model scale**: the test water's viscosity and no correlation allowance, so the methods differ only in
their form, wave and shallow-water terms. Every method is run with OVD's own shallow and confined water
factor (Lackenby plus blockage) on top, as the core runs it.

## Research line

- **Searched** (2026-10-30): open towing-tank data for barges and pushed convoys (DST Duisburg, HSVA,
  MARIN, VBD, US Army Corps, Polish inland-vessel work), US 5,488,918, Tabaczek et al. 2007, the
  Strathclyde recalibration of Holtrop (Nikolopoulos and Boulougouris 2019), Hollenbach 1998,
  shallow-water corrections (Karpov, Zeng, Millward, Lackenby; OpenTNSim `energy`, MIT, commit cb6bdda).
- **Exists, with numbers, used here:**

  | Source | What | Cb | L/B | B/T | Fn | h/T |
  |---|---|---|---|---|---|---|
  | Zentari 2023, Dr.-Ing. thesis, Duisburg-Essen (DuEPublico), Figs 5.9, 5.10, Table 4.1 | Model tests (1:16) of a pusher with 1, 2 and 2 x 2 Europa-type barges (barge 76.45 x 11.36 x 2.80 m, Cb 0.92), HSVA deep, MARIN shallow | 0.77-0.83 (convoy) | 8.1-16.3 | 4.1-8.1 | 0.052-0.136 | 1.2, 1.5, 2.0, > 30 |
  | Tabaczek and Zawislak 2018, Polish Maritime Research 25(S1), Table 1, Fig. 4 (CC BY-NC-ND 4.0) | Model tests (1:16) of an inland motor cargo vessel, fit CTM = 2.134 CF0M + 0.907 Fr^2.847; 1+k = 1.33 in deep water | 0.88 | 7.6 | 5.6 | 0.054-0.118 | 1.56 |

- **Exists, not usable as numbers:** Tabaczek, Kulczyk and Zawislak 2007 (CFD of 11 barge bows,
  Cb 0.94-0.97, bar charts only); US 5,488,918 (70 river-barge models, Cb 0.936-0.988, graphs only);
  Kulczyk 2018 (shallow-water friction ratios, no CT); Zentari et al. 2022 (Applied Ocean Research,
  paywalled; the thesis carries the same tests); Mucha et al. 2017/2018 inland ship (in Fig. 5.9, but
  its particulars are in a paywalled paper); Nikolopoulos and Boulougouris 2019 (seven bulk carriers
  and tankers with bulbs, Cb 0.80-0.86, no licence); Hollenbach 1998 (its window closes near Fn 0.17 at
  Cb 0.96); a KRISO comparison of barge methods (not reachable).
- **Not found:** any open test of a wide, short, full barge like the coastal LNG concept (L/B 3-4.5,
  B/T about 6, Cb 0.84-0.97, Fn 0.17-0.21). No source covers it.
- **Reusing:** the Holtrop 1984 form factor (already in `holtrop_mennen.rs`); Millward 1989
  (International Shipbuilding Progress 36(407)), dk = 0.644 (T/h)^1.72, as reproduced by Zawislak 2007
  (Polish Maritime Research) and judged the best of the classic shallow-water methods by Raven 2012
  (29th Symposium on Naval Hydrodynamics). Zeng's friction line (OpenTNSim) changes Cf by a few per cent
  on these cases (Kulczyk 2018: 0.95 to 1.05), far less than the measured increase, so it is not added.
- **Building because:** no published barge method with numbers covers these hulls. The new method uses
  only published terms and fits no constant.

## Error per case (CT, method over test minus 1)

| case | Fn | Cb | L/B | B/T | h/T | CT x1000 | barge_fit | holtrop_mennen | barge_form_factor | 1+k1 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| z23-1x1-deep-a | 0.068 | 0.769 | 9.55 | 4.06 | deep | 5.960 | -33.5 % | -30.5 % | -30.5 % | 1.150 |
| z23-1x1-deep-b | 0.102 | 0.769 | 9.55 | 4.06 | deep | 5.752 | -36.1 % | -32.8 % | -33.2 % | 1.150 |
| z23-1x1-deep-c | 0.136 | 0.769 | 9.55 | 4.06 | deep | 5.801 | -38.9 % | -34.7 % | -36.2 % | 1.150 |
| z23-2x1-deep-a | 0.052 | 0.833 | 16.28 | 4.06 | deep | 4.436 | -19.1 % | -16.9 % | -18.9 % | 1.103 |
| z23-2x1-deep-b | 0.078 | 0.833 | 16.28 | 4.06 | deep | 4.160 | -19.7 % | -2.2 % | -19.5 % | 1.103 |
| z23-2x1-deep-c | 0.104 | 0.833 | 16.28 | 4.06 | deep | 4.234 | -24.8 % | +18.3 % | -24.7 % | 1.103 |
| z23-2x2-deep-a | 0.052 | 0.798 | 8.14 | 8.11 | deep | 3.879 | -7.5 % | -0.5 % | -0.6 % | 1.182 |
| z23-2x2-deep-b | 0.078 | 0.798 | 8.14 | 8.11 | deep | 3.575 | -6.6 % | +1.6 % | +0.4 % | 1.182 |
| z23-2x2-deep-c | 0.104 | 0.798 | 8.14 | 8.11 | deep | 3.612 | -11.9 % | +1.3 % | -5.3 % | 1.182 |
| z23-1x1-h2.0 | 0.068 | 0.769 | 9.55 | 4.06 | 2.00 | 6.275 | -35.1 % | -32.1 % | -20.6 % | 1.150 |
| z23-1x1-h1.5 | 0.068 | 0.769 | 9.55 | 4.06 | 1.50 | 7.094 | -40.7 % | -38.0 % | -20.8 % | 1.150 |
| z23-1x1-h1.2 | 0.068 | 0.769 | 9.55 | 4.06 | 1.20 | 8.968 | -50.9 % | -48.7 % | -27.7 % | 1.150 |
| z23-2x1-h2.0 | 0.052 | 0.833 | 16.28 | 4.06 | 2.00 | 4.838 | -23.7 % | -21.7 % | -10.0 % | 1.103 |
| z23-2x1-h1.5 | 0.052 | 0.833 | 16.28 | 4.06 | 1.50 | 5.388 | -29.3 % | -27.4 % | -8.5 % | 1.103 |
| z23-2x1-h1.2 | 0.052 | 0.833 | 16.28 | 4.06 | 1.20 | 5.819 | -31.5 % | -29.6 % | -2.0 % | 1.103 |
| z23-2x2-h2.0 | 0.052 | 0.798 | 8.14 | 8.11 | 2.00 | 4.847 | -23.6 % | -17.8 % | -4.2 % | 1.182 |
| z23-2x2-h1.5 | 0.052 | 0.798 | 8.14 | 8.11 | 1.50 | 6.016 | -36.3 % | -31.5 % | -12.9 % | 1.182 |
| z23-2x2-h1.2 | 0.052 | 0.798 | 8.14 | 8.11 | 1.20 | 6.744 | -40.3 % | -35.8 % | -10.3 % | 1.182 |
| t18-obm-h1.56-fr0.060 | 0.060 | 0.876 | 7.60 | 5.57 | 1.56 | 9.541 | -47.6 % | -38.2 % | -23.9 % | 1.298 |
| t18-obm-h1.56-fr0.080 | 0.080 | 0.876 | 7.60 | 5.57 | 1.56 | 9.393 | -47.5 % | -37.9 % | -23.8 % | 1.298 |
| t18-obm-h1.56-fr0.100 | 0.100 | 0.876 | 7.60 | 5.57 | 1.56 | 9.619 | -47.9 % | -37.2 % | -24.3 % | 1.298 |
| t18-obm-h1.56-fr0.118 | 0.118 | 0.876 | 7.60 | 5.57 | 1.56 | 10.129 | -48.4 % | -35.1 % | -25.2 % | 1.298 |

## Error per range

`by_block_coefficient` is Holtrop-Mennen on every Zentari case (Cp up to 0.833) and the barge fit on
the motor cargo vessel (Cp 0.876): -49 to +18 %, mean absolute 27.9 % over all 22 cases.

| group | cases | barge_fit | holtrop_mennen | barge_form_factor |
|---|---:|---|---|---|
| deep, pusher + 1 barge | 3 | -39 to -34 % (mean abs 36 %) | -35 to -30 % (mean abs 33 %) | -36 to -31 % (mean abs 33 %) |
| deep, pusher + 2 or 4 barges | 6 | -25 to -7 % (mean abs 15 %) | -17 to +18 % (mean abs 7 %) | -25 to +0 % (mean abs 12 %) |
| shallow, pusher + 1 barge | 3 | -51 to -35 % (mean abs 42 %) | -49 to -32 % (mean abs 40 %) | -28 to -21 % (mean abs 23 %) |
| shallow, other | 10 | -48 to -24 % (mean abs 38 %) | -38 to -18 % (mean abs 31 %) | -25 to -2 % (mean abs 15 %) |
| all | 22 | mean abs 31.9 % | mean abs 25.9 % | mean abs 17.4 % |

Deep-water form factor of the motor cargo vessel (Tabaczek and Zawislak 2018): tests 1.33; Holtrop 1984
1 + k1 = 1.298 (-2 %); the bluff-barge fit's low-speed equivalent 1.10 (-17 %).

## What the data say

1. **The bluff-barge fit under-predicts every case**, by 7 to 51 %. Its low-speed Cr/Cf of 0.10 (an
   effective form factor of 1.10) is below every measured value (1.19 to 1.66 in deep water). It is
   optimistic, and it has no form term, so it cannot rank hulls.
2. **Holtrop-Mennen holds in deep water on the single-row and wide convoys** (pusher + 2 x 2 barges:
   -0.5 to +1.6 %), though the cases are outside its L/B and B/T range. Its wave term is erratic on the
   long, narrow convoy (L/B 16: -17 to +18 % over Fn 0.05 to 0.10). Its shallow-water result is as poor
   as the barge fit's, because both carry the same Lackenby term.
3. **The shallow-water term is the largest error.** At h/T 1.2 the tests rise 1.31 to 1.74 times over
   deep water; OVD's Lackenby plus blockage factor gives about 1.12. Millward's increase of k closes
   most of the gap.
4. **`barge_form_factor`** (Holtrop 1 + k1, Millward dk, the barge fit's Froude-number rise) has the
   smallest error: mean absolute 17 % against 26 % (Holtrop) and 32 % (barge fit); in shallow water
   -2 to -28 % against -18 to -51 %. It still reads low, by about 17 % on average.
5. **The pusher with one barge** is low by 31 to 36 % under every method. The thesis names its gap
   between barge stern and pusher bow as the worst case for flow; no method models that gap.

## Limits

- The convoy cases enter the methods as one hull (summed length, volume and wetted surface, barge
  draught). The thesis does not state the wetted surface it divides by; the sum is assumed.
- CT values are read from vector figures (about 0.1 %); the shallow-water viscosity is not stated and is
  taken as the deep-water one.
- The data cover Cb 0.77 to 0.88 as single hulls, L/B 7.6 to 16.3, B/T 4.1 to 8.1, Fn up to 0.136.
  The coastal LNG concept (L/B 3-4.5, Fn about 0.2, Cb up to 0.97) is outside every one of them.
- Model scale only. No full-scale barge trial was found; the correlation allowance (Holtrop's adds
  about 30 % of the friction on these hulls) is not tested.
