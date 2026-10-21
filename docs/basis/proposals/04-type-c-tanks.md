# Proposals: Type C tanks

Not published. Companion to `docs/basis/04-type-c-tanks.md`. Ids `TC-A..` refer to the assumptions table on that
page. Items already in hand are not repeated: #313 (S690QL1 stress), #314 (2:1 head area), #315 (LPG density and
loading limit), #316 (dome default), #317 (minimum design pressure, steel by temperature, tank weight calibration).
"Derived" means arithmetic on cited inputs, not a source statement. IGC = MSC.370(93), held at
`~/Dev/ovcd-rule-sources/igc/MSC.370-93.txt`.

## Errors

Places where the code and its cited source disagree, or the code disagrees with itself.

**E1. No design liquid pressure in the wall (TC-A14).** Code: `tanks.rs` line 133 passes `inputs.design_pressure`
alone to `pressure_vessel.rs::calculate_wall_thicknesses_with`; the profile basis says "The engine adds no liquid
head" (`builtin_profile.json` line 162). Source: IGC 4.23.2.2 ("The design liquid pressure defined in 4.13.2 shall be
taken into account"), 4.13.2.4 (Peq = P0 + (Pgd)max) and 4.28.1.2 (guidance for Pgd). The file header cites the IGC
Code for the wall. Effect: large on low-pressure tanks, non-conservative. Static head alone on an 8 m LNG tank is
about 0.035 MPa, about 9 % of 0.4 MPa; with dynamic acceleration more (derived). About 2 % to 3 % on 1.77 MPa LPG.

**E2. #314 reads one error two ways.** Code: `tanks.rs` line 173 gives 1.084·π·R² = 3.405·R² per 2:1 head. The
half 2:1 oblate spheroid is 4.336·R² (derived; equal to the 1.084·D² rule the PR #305 draft quotes). The code area is
21.5 % below the true one; the true area is 27.3 % above the code one. So the "21 %" and "27 %" of the two reviews
are the same error. #314 asks why they differ: this is the answer. Effect: none beyond #314.

**E3. 2:1 head length counts half the end insulation.** Code: `tanks.rs` line 171, `head_length_outer = outer_radius
/ 2.0`, so the insulated head length is (R_s + t_ins)/2. The steel head is R_s/2 deep and the insulation adds t_ins,
so the true insulated length is R_s/2 + t_ins. The cylinder comes out t_ins too long per tank (line 192). The
hemisphere branch (line 167) is right. Source: geometry; the code's own datum (TC-A01). Effect: small,
non-conservative on volume (about 1.5 % on a 9 m × 25 m LNG tank with 330 mm insulation, derived). API only today.

**E4. Head formulas cited to the shell paragraph.** Code: `pressure_vessel.rs` lines 170 and 186 name "ASME UG-27"
for the hemispherical and 2:1 head formulas. The secondary source held (CheCalc vessel guide,
`type-c/design/checalc_vessel_guide.html`) gives the head formulas apart from the UG-27 shell formula and names
UG-32(e) for torispherical heads; ASME VIII-1 itself is not held. Effect: none on numbers; wrong citation. Fix the
comments when the standard is held.

**E5. Thickness after forming without tolerance (TC-A19).** Code: `tanks.rs` lines 216–219 weigh the required
thickness. Source: IGC 4.23.2.1.1 says the calculated thickness is "a minimum thickness after forming, without any
negative tolerance". A built plate is thicker. Effect: small, non-conservative on tank weight.

**E6. Cylinder free surface takes the fill ratio as a height ratio (API only).** Code: `tanks.rs` line 280,
`h = fill_ratio * 2.0 * r`, used by `hydrostatics.rs` line 132. A fill ratio is a volume ratio (IGC 15.1.1). At
0.98 the true liquid height is 0.948·D and the chord 0.89·D; the code uses 0.56·D, so the free surface moment is
about 4 times low (derived). The Design path is right: `gas_stability.rs` line 733 uses `segment_fill`. Effect:
large, non-conservative, API only. Belongs with page 03; listed here because the function is in `tanks.rs`.

**E7. Profile text says "no source" for allowable stresses that match the IGC.** Profile lines 212 and 235 say
304L, 316L and 9Ni stresses carry "no source cited". They match IGC 4.23.3.1 with standard minimum strengths
(TC-A21 to TC-A23). The code holds the stress, not Rm and Re. #313 item 2 covers the check; this item is the text.
Effect: none on numbers.

## Gaps

What a gas containment engineer would expect that OVCD does not do.

**G1. Head type is not a Design input (TC-A06).** Every Design gets hemispherical heads. Hemispherical heads are
half the shell thickness and as long as the radius, so tanks are light at the ends and short in the cylinder. Built
small tanks often have torispherical or 2:1 heads. The 2:1 branch exists in code, but a Design cannot reach it, so
the #314 fix reaches no Design. No torispherical head exists.

**G2. Tank weight is one factor (TC-A63, TC-A64).** The model tank weighs about 0.05 t/m³ against 0.096 to 0.115
t/m³ for two published marine LNG tanks (derived; page Evidence). #317 adds calibration cases but keeps the single
factor. Not modelled: ring frames at saddles, saddles, support wood and resin, anti-flotation chocks, longitudinal
stops, dome steel and insert, sump, manholes, internal piping, ladders and platforms, nozzle reinforcement.

**G3. No supports or support loads (TC-A68).** No saddle count, embrace angle, wood or chocks. No collision load
(IGC 4.15.1: 0.5 g forward, 0.25 g aft), flooding uplift (4.15.2) or static heel up to 30° (4.13.9). Supports must
be designed to a recognised standard (4.23.2.5.2; IACS UI GC8 ring stress min(0.57 Rm, 0.85 Re)). The BOQ line
`tank_supports` is the structure factor surplus, priced at the supports rate. Page 10 proposals E4 finds that the
internals inside it are priced at the wrong rate.

**G4. No external pressure or buckling.** IGC 4.23.2.3 (Pe = P1 + P2 + P3 + P4, P1 not less than 0.025 MPa without
vacuum relief valves) and 4.23.3.2. A formula wall of 8.5 mm on a 9 m, 0.4 MPa LNG tank (derived) may be governed by
buckling, handling or a yard minimum. The method source (ASME VIII-1 UG-28, EN 13445-3 clause 8) is not held.

**G5. Ambient vapour pressure for fully pressurised LPG.** IGC 4.13.2.2: with no temperature control, P0 is not
less than the vapour pressure of the cargo at 45 °C. No check exists. It sits beside the #317 minimum-pressure Rule.

**G6. Design temperature and insulation type do nothing (TC-A31 to TC-A34, TC-A43 to TC-A46).** #317 adds the steel
check. Insulation density does not follow the type (60 kg/m³ for panel and PUF alike), and no vacuum-insulated
double-wall tank exists. Published figures: perlite 40 kg/m³ and Durolight 45–50 kg/m³ (Zagreb 2024 study), lab PUF
96 kg/m³ (JMSE 2021); no installed PUF density for a Type C tank was found.

**G7. Weld efficiency and corrosion allowance are not inputs (TC-A15, TC-A16).** IGC 4.23.2.1.3 allows 0.95 up to
1.0; the river LPG calibration case uses 1.0 through the API. The 0.5 mm allowance for 9Ni and S690QL1 has no source.

**G8. Bilobe tanks.** Two published bilobe records cannot run (`calibration/tank_cases.yaml`). Bilobe tanks use a
beam better and are common above about 2,000 m³.

**G9. Glossary terms missing.** Filling Limit, Loading Limit, Design Vapour Pressure (P0, MARVS), Tank Dome, Tank
Head (hemispherical, 2:1 ellipsoidal, torispherical), Tank Structure Factor, Steel Outer Diameter. `GLOSSARY.md`
has "Tank Outer Diameter" only.

## Changes

Each with its effect on results and its order. Order 0 is text only and can go now. Later orders wait for
#313–#317 to land, so the design-impact tables do not mix.

| Order | Change | Fixes | Effect on results |
|---|---|---|---|
| 0 | Fix the head formula citations in `pressure_vessel.rs` comments; record E2 on #314; rewrite the profile basis of 304L, 316L, 9Ni stresses to show min(Rm/A, Re/B) and the strengths, marked unverified | E2, E4, E7 | None |
| 0 | Add the glossary terms of G9 | G9 | None |
| 1 | Add the Rule `igc/4.23.2.2/liquid-head`: wall sized on Peq = P0 + (Pgd)max with IGC 4.28.1.2. Start with static head plus a vertical acceleration input; enter as `warn` in `igc-2g` so the migration moves no result, then as a sizing change | E1 | Large on LNG and refrigerated LPG walls (about +9 % or more); small on pressurised LPG |
| 2 | Make head type an input (`hemisphere`, `ellipsoidal_2to1`, later `torispherical`), with a sourced default; ship with or after #314 | G1 | Medium: head steel, tank length and volume move |
| 2 | Fix the 2:1 insulated head length to R_s/2 + t_ins | E3 | Small, 2:1 heads only |
| 3 | Component tank weight model, behind the #317 calibration cases: shell and heads at plate thickness plus a forming allowance; one ring frame per saddle; two saddles (fixed and sliding); wood (350 mm, one built ship) and resin (20 mm); chocks and stops sized from the IGC 4.15 loads; dome and insert (Ø, height, 20 mm wall); manholes Ø 600 mm; a fittings allowance. Keep the structure factor as a fallback | G2, G3, E5 | Large: tank weight about doubles towards published 0.10 t/m³; lightship up, cargo down on draught-bound designs |
| 3 | Split the BOQ lines to match: `tank_shell`, `tank_heads`, `tank_rings_internals` (tank shell rate), `tank_saddles` (hull steel rate), `support_wood_resin`, `chocks_stops`, `dome`, `insulation` (m² and m³), `fittings` set, `NDT_PWHT_test` (lump). With page 10 | G3 | Medium on cost; none on weight beyond the row above |
| 4 | Insulation density by type, with sources; add vacuum double-wall as a type with its own weight | G6 | Small to medium on LNG tank weight |
| 4 | Weld efficiency and corrosion allowance as inputs with bounds (IGC 4.23.2.1.3 upper bound 1.0 as a Rule) | G7 | None by default |
| 5 | Ambient vapour pressure Rule for fully pressurised LPG (IGC 4.13.2.2) with cited vapour pressures | G5 | None unless a design fails |
| 5 | External pressure and buckling check, `not evaluated` until a method source is held | G4 | None until evaluated; then possibly large on thin LNG walls |
| 6 | Fix `free_surface_moment_cylindrical` to take a volume ratio (reuse `segment_fill`), with page 03 | E6 | Large on API stability answers; none on Designs |
| 7 | Bilobe tank geometry | G8 | Large for wide hulls; new feature |

Dependencies: order 1 needs the rules engine (#268, #269) for the Rule form, but a sizing change can go first. Order
3 needs the tank weight calibration cases of #317, so the new parts are checked against held-out weights before any
default moves.
