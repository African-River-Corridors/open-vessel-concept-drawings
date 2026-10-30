# A hull-form resistance method beside the bluff-barge fit

Status: **proposed**, 2026-10-30. Issue #723. The default does not change in this decision; the
switch is a question for the maintainer (see "Open question").

## Context

The calm-water resistance is ITTC-1957 friction × (1 + Cr/Cf), with Cr/Cf a function of the Froude
number only (`resistance.rs::residuary_ratio`). That fit is an Estimate for bluff barges. It has no
hull-form term: bow rake, stern run, bilge radius and Cb change the power only through the wetted
surface and the waterline length. On a 140 × 29 m coastal LNG barge concept the finest variant tried
gained 0.18 kn at fixed power. The maintainer asked for a model that can test hulls with a lower
block coefficient.

## Research

Searched, 2026-10-30: open resistance methods for full forms and barges, open implementations, and
their licences.

| Source | What it gives | Use here |
|---|---|---|
| Holtrop and Mennen 1982, ISP 29(335) | Components R_F(1+k1), R_APP, R_W, R_B, R_TR, R_A, and a worked example (205 m ship, 25 kn) | **Implemented** (`Variant::Hm1982`); the worked example is the verification case |
| Holtrop 1984, ISP 31(363) | New form factor 1+k1 (with c14) and low-speed wave term R_W-A (m4), Fn ≤ 0.4 | **Implemented** (`Variant::Hm1984`), the method OVD uses |
| Ortigosa, López and García 2009, JMR VI(III), Table 1 | Data range by ship type: tankers and bulk carriers Fn ≤ 0.24, Cp 0.73–0.85, L/B 5.1–7.1, B/T 2.4–3.2; general cargo Fn ≤ 0.30, Cp 0.58–0.72, L/B 5.3–8.0, B/T 2.4–4.0 | **The declared envelope** of the Holtrop method |
| Nikolopoulos and Boulougouris 2019, JSPD 35(1) (Strathclyde) | A genetic-algorithm recalibration of the Holtrop constants for bulk carriers and tankers, Cb 0.80–0.86, Fn about 0.12–0.18. Original Holtrop errs by up to about 20 % on these hulls at low Fn | **Not reused**: calibrated on seven ships with bulbs, same Cb band as Holtrop, no licence on the manuscript. Cited as the accuracy band |
| Hollenbach 1998 (via Schneekluth, Ship Design for Efficiency and Economy, Table 6.14) | Single- and twin-screw regression; Fn_max = 0.614 − 0.717 Cb + 0.261 Cb² | **Not reused**: at Cb 0.96 the window closes (Fn_max ≈ 0.17); it adds nothing above Holtrop for barges |
| OpenTNSim `opentnsim/energy` (TU Delft, MIT), commit cb6bdda | Holtrop components with Karpov (speed) and Zeng (friction) shallow-water corrections; estimates C_WP = (1+2Cb)/3, lcb = −13.5 + 19.4 Cp | **Reused**: its C_WP and lcb estimates (cited in the code), and its form factor as an independent check. Not reused verbatim: it mixes the 1982 wave term with the 1984 form factor, takes c14 = 1 + 0.0011 C_stern (the paper has 0.011), rounds the exponents, and overwrites c7 for B/L < 0.11. Its Karpov and Zeng terms are a separate step (see "Not done") |
| Tabaczek et al. 2007 (CC BY-NC-ND) | CFD of barge bows, Cb 0.95–0.97: pressure resistance sets the ranking | **Link only**. No regression to reuse |
| US 5,488,918 (1996), optimised barge bow | Tank tests of 70 river-barge models, Cb 0.936–0.988 | **No numbers**: the results are only graphs |

No open method with numbers covers a barge with L/B about 4 and B/T about 6.5. So the bluff-barge fit
stays the only method above Cp 0.85.

## Decision

1. A new module `ovd-core/src/holtrop_mennen.rs` implements Holtrop–Mennen with both variants.
   OVD uses the 1984 variant. Bulb, transom and appendage terms are in the module, and the OVD hull
   passes none of them (no bulb, no immersed transom, no appendages, normal afterbody C_stern 0).
   The wetted surface is the one OVD already uses, so the only change between methods is the method.
2. A new input `resistance_method` (Speed and power group):
   - `barge_fit`: today's fit. **The default**, and the value a file saved before reads.
   - `holtrop_mennen`: Holtrop 1984 at any fullness. Where the formulas have no value (Cp near 1:
     1 − Cp − 0.0225 lcb ≤ 0), the run stops with a message that names `by_block_coefficient`.
   - `by_block_coefficient`: Holtrop 1984 up to Cp 0.85, the barge fit above.
3. Each method declares its range ([`Envelope`]):

   | Method | Fn | Cp | L/B | B/T | Status |
   |---|---|---|---|---|---|
   | Holtrop–Mennen 1984, full forms | ≤ 0.24 | 0.73–0.85 | 5.1–7.1 | 2.4–3.2 | sourced (Ortigosa 2009) |
   | Holtrop–Mennen 1984, general cargo (Cp < 0.725) | ≤ 0.30 | 0.58–0.72 | 5.3–8.0 | 2.4–4.0 | sourced (Ortigosa 2009) |
   | Bluff-barge fit | — | 0.85–1.0 | — | — | Estimate, no source |

4. A Design that chooses a method other than `barge_fit` gets a warning check per environment,
   `resistance method in range`, which names the method used and each parameter outside its range.
   Rule `ovd/resistance/method-in-range` (Rule Set `ovd` 3.1.0, applies when `resistance_method` is
   `holtrop_mennen` or `by_block_coefficient`) reports it. A Design on the default gets no new check,
   so its results and hash do not move.

## Evidence

- **Verification against the 1982 worked example** (`crates/ovd-core/tests/holtrop_mennen.rs`,
  labelled `sourced`). Every coefficient and component within 0.1 % of the printed value, except
  R_A: the paper prints R_A 221.98 kN, which is 0.6 % above its own printed C_A × ½ρV²S. Total
  1,792.10 kN against 1,793.26 kN (−0.065 %).
- **The 1984 form factor** agrees with OpenTNSim to 0.02 % (1.15980 against 1.16003; OpenTNSim
  rounds the exponents).
- **No stored result moves.** Every fixture, website example and worked example keeps its hash
  (`tools/check_fixture_hashes.py`); `tools/design_impact.py` finds no moved design.
- **Cb sweep, 140 × 29 m coastal LNG barge concept**, installed power typed at 1,729 kW, deep water,
  `by_block_coefficient` (Options `N1`–`N9` of the example's sensitivities file):

  | Option | Bow rake / stern run / bilge radius (m) | Cb | Cp | L_WL (m) | Cargo (m³) | Method | Outside its range | Speed (kn) |
  |---|---|---:|---:|---:|---:|---|---|---:|
  | N1 | 6 / rule / 0 | 0.972 | 0.972 | 129.3 | 19,737 | bluff-barge fit | none | 10.99 |
  | N2 (= K) | 12 / rule / 0 | 0.962 | 0.962 | 125.5 | 18,554 | bluff-barge fit | none | 11.08 |
  | N3 | 20 / 12 / 0 | 0.949 | 0.949 | 120.4 | 16,977 | bluff-barge fit | none | 11.21 |
  | N4 | 20 / 20 / 1.5 | 0.921 | 0.928 | 116.8 | 16,956 | bluff-barge fit | none | 11.26 |
  | N5 | 30 / 30 / 1.5 | 0.873 | 0.879 | 105.3 | 13,014 | bluff-barge fit | none | 11.59 |
  | N6 | 35 / 35 / 2 | 0.839 | 0.850 | 99.8 | 11,044 | bluff-barge fit (Cp just above 0.85) | none | 11.76 |
  | N7 | 40 / 40 / 2 | 0.808 | 0.818 | 94.1 | 9,073 | Holtrop–Mennen 1984 | L/B 3.25, B/T 6.17 | 9.75 |
  | N8 | 45 / 45 / 2 | 0.772 | 0.781 | 88.5 | 7,102 | Holtrop–Mennen 1984 | L/B 3.05, B/T 6.15 | 10.08 |
  | N9 | 48 / 48 / 2.5 | 0.741 | 0.756 | 85.4 | 5,919 | Holtrop–Mennen 1984 | L/B 2.94, B/T 6.10 | 10.30 |

  Within each method a finer hull is faster: +0.77 kn from N1 to N6 under the fit (wetted surface and
  length only), +0.55 kn from N7 to N9 under Holtrop. Across Cp 0.85 the speed falls by 2 kn: that
  step is the disagreement of the two methods, not a property of the hull. The finer hulls also lose
  cargo volume (the rakes shorten the tanks): 19,737 m³ at N1, 5,919 m³ at N9.

## Open question

**Should `by_block_coefficient` become the default for new Designs?** The two methods do not
meet at Cp 0.85. On the same hull (N6, Cp 0.850), Holtrop gives 9.4 kn and the barge fit 11.8 kn at
1,729 kW; at 8 to 10 kn Holtrop needs 1.7 to 2.0 times the power. Holtrop puts 1 + k1 near 1.5 and a correlation
allowance near 30 % of the friction on these short, beamy hulls; the barge fit has an effective form
factor of 1.1 and no allowance. Barge model data (Tabaczek et al. 2007) say the pressure
resistance of a bluff barge is large, so the barge fit is likely optimistic. But the hull is outside
Holtrop's L/B and B/T range at every Cb, so neither number is validated. Options:

- keep `barge_fit` as the default and label it plainly as unable to rank hull forms (today);
- make `by_block_coefficient` the default for new Designs, with the warning (saved Designs keep
  `barge_fit` through the legacy value);
- calibrate the barge fit against barge model data before any switch.

## Not done

- **Shallow water.** Holtrop is deep-water. OVD keeps its own Lackenby and blockage factor on top
  of either method. OpenTNSim's Karpov and Zeng terms are the candidate replacement (a follow-up).
- **The hull's own lcb, transom and wetted surface.** The method uses OpenTNSim's lcb estimate, no
  immersed transom, and OVD's box wetted surface. The sections model knows the true values; passing
  them is a follow-up.
- Translations of the new messages (English only).
