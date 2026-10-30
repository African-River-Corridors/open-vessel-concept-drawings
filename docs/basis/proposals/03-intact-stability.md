# Proposals: 03 Intact stability

Not published. Gaps, errors and proposed changes for `docs/basis/03-intact-stability.md`. Errors become findings
for the register (#121) only after maintainer triage. Row ids (IS-Ann) refer to the page's assumptions table.

## Gaps

What a stability specialist expects that OVD does not do.

| # | Gap | Page rows | Effect if closed |
|---|---|---|---|
| G1 | No down-flooding angle. No openings model, so areas always run to 40°. | IS-A31, IS-A32, IS-A37 | Large for a design with a low opening; none otherwise |
| G2 | No weather criterion (Part A 2.3). No lateral windage area or lever. The deck-edge angle needed by 2.3.1.2 is already computed but not used. | IS-A38, IS-A39 | Large for high-sided or light-draught hulls |
| G3 | No standard loading conditions (Part B 3.4.1.2): no departure or arrival, no stores, fuel or fresh water, no ballast condition. IGC 2.2.2 also asks for loading and unloading stages. | IS-A15, IS-A17 | Small to large; light-ship conditions change most |
| G4 | No free surface for consumables or ballast (Part B 3.1.4; IGC 2.2.3). | IS-A26 | Small to medium |
| G5 | No slack-tank case: all tanks always have the same fill. | IS-A18 | Medium for multi-tank designs |
| G6 | GZ from a prismatic box at even keel. The hull geometry of area 01 (rakes, `HullGeometry`) does not reach the GZ, KB or BM. No free trim. | IS-A02, IS-A06, IS-A07, IS-A47 | Unknown; likely medium at large angles |
| G7 | No pontoon criteria (Part B 2.2) and no inland intact set. Wide, shallow barges tend to fail Part A 2.2.3 (angle of max GZ ≥ 25°). Part A 2.2.3 allows alternative criteria; the ABS steel barge rules (2024, 3-6-1/3.1) point cargo barges of 24 m and over to Part B 2.2. | IS-A34, IS-A46 | Large for river barges: verdicts can flip |
| G8 | No applicability by length. Part A 1.1.1 covers ships of 24 m and over; OVD applies it to every length and says nothing. | IS-A46 | None on numbers; changes the wording of the verdict |
| G9 | Dry bulk: no GZ, no criteria, full load only, no cargo shift. | IS-A48 to IS-A53 | Large (the feature is missing) |
| G10 | Tank heads left out of the free surface inertia and treated as cylinder for the fill level. | IS-A19, IS-A22 | Small |
| G11 | Dome mass not in the weight split; stern block mass not in KG for self-propelled hulls. | IS-A13, IS-A14 | Small to medium (non-conservative) |
| G12 | No calibration case with a published KG, GM or GZ. The harness oracle shares the box idealisation and the polygon clip with the engine. | Evidence | None on numbers; it limits trust |

Glossary terms that are missing (`GLOSSARY.md`): **Loading Condition**, **Free Surface Correction**, **GZ curve**
(righting lever curve), **Down-flooding Angle**, **Lightship**. Spec 0004 already uses `loading-condition` as an
object type.

## Errors

Places where the code and its cited source, or two parts of the code, disagree.

| # | Error | Code | Source | Effect on results |
|---|---|---|---|---|
| E1 | Areas 0–40° and 30–40° always end at 40°. The text ends them at the down-flooding angle if less. (Known: finding #282.) | `gas_stability.rs:448`, `:455`, `:628`, `:629` | MSC.267(85), 2008 IS Code, Part A 2.2.1 | Non-conservative when θf < 40° |
| E2 | The code cites only Part A 2.2, but Part A 2.1.1 says the criteria apply to all loading conditions of Part B 3.3 and 3.4, and 2.1.2 says free surface applies to all of them. The three code conditions are not that set. | `gas_stability.rs:723`, `:430` | Part A 2.1.1, 2.1.2; Part B 3.4.1.2, 3.1.4 | Unknown; likely non-conservative for arrival and ballast conditions |
| E3 | The limitation line says "ADN 9.3.1.15 / IGC 2.7 need both" wind heel and damage stability. ADN 9.3.1.15 is damage stability only, the ADN set was removed (#175), and the wind heel requirement comes from IGC 2.2.2 through the IS Code (Part A 2.3), not from IGC 2.7. | `gas_stability.rs:795` | ADN 2025 9.3.1.15; IGC MSC.370(93) 2.2.2, 2.7 | None on numbers; the citation is wrong |
| E4 | `area_0_27` is computed and written to every condition but no Rule uses it. It is the ADN 9.3.1.14.2(b) measure (0.024 m·rad to 27°), left after #175. A reader can take it for a criterion. | `gas_stability.rs:626`; `crates/ovd-format/src/gas_stability.rs:196` | ADN 2025 9.3.1.14.2(b) | None on numbers |
| E5 | Two GM paths disagree. The legacy `calculate_stability` reads `fill_ratio` as a fraction of the diameter (`h = fill × 2r`), puts the cargo at the tank centre at any fill, and adds a stores item. `gas_stability` reads the fill as a fraction of the section area and puts the cargo at the liquid centroid. The Python API still exposes the legacy function. | `hydrostatics.rs:94`, `:106`, `:131`; `tanks.rs:280` vs `gas_stability.rs:733` | Internal (no rule) | Different GM for the same design through two public functions |
| E6 | `BargeResults.stability_ok` is documented "True if GM_fluid > 0.15 m" (strict), while Part A 2.2.4 says "not less than" 0.15 m. The GA adapter sets it to `true` always. | `models.rs:407`; `ga_adapter.rs:246` | Part A 2.2.4 | A reader of that field gets a pass with no check |
| E7 | The bulk check is named `gm_positive` but tests GM ≥ 0.15 m, uses the Part A 2.2.4 number with no citation, and is solid GM at full load only. | `barge_capacity.rs:1004`–`:1011` | Part A 2.2.4 | None on numbers; the name and the citation mislead |
| E8 | The Python comment says "gas uses hydrostatics.calculate_stability". Gas uses `gas_stability.assess`. | `src/ovd/core/barge_capacity.py:193` | Internal | None |

## Changes

Each change with its effect on results and its order of work. "Effect" is on today's verdicts.

| Order | Change | Closes | Effect |
|---|---|---|---|
| 1 | Correct the limitation text (E3): cite Part A 2.3 for the weather criterion and IGC 2.7 for damage survival; drop ADN. Remove `area_0_27` from the condition output, or label it "not a criterion" (E4). Fix the bulk check name and add its basis (E7). Fix the stale comment (E8). | E3, E4, E7, E8 | None (text and unhashed output only; check the results hash rule for `area_0_27`) |
| 2 | Add the glossary terms listed above. | Glossary | None |
| 3 | Mark the legacy `calculate_stability` as deprecated in the Python API, or route it through `gas_stability`; fix the `stability_ok` doc comment and stop the GA adapter from reporting `true` without a check (E5, E6). | E5, E6 | None for `simulate` and `fit`; changes the legacy API output |
| 4 | Encode the six criteria as the `is2008-a` Rule Set (spec 0004), with tolerance 0 and the second-reader record. Add `not evaluated` rows for Part A 2.3 and IGC 2.7, and an applicability note for length under 24 m (G8). | Rule status, G8 | None (spec 0004: migration may not change `ok`) |
| 5 | Add a down-flooding angle input (optional, default none) and run the 2.2.1 areas to min(40°, θf). Keep today's result when no angle is given, with the #282 finding shown in the trace. | E1, G1 | None by default; large when set |
| 6 | Add departure and arrival conditions with a stores and fuel allowance (input, default from the Defaults Profile), a ballast condition, and the consumables free surface of Part B 3.1.4 (E2, G3, G4). Make it a design change with a before and after table. | E2, G3, G4 | Small to large; `fit` may return smaller arrangements |
| 7 | Add a one-tank-slack condition and the head liquid in the free surface (G5, G10). | G5, G10 | Small to medium |
| 8 | Add the dome mass and, for self-propelled hulls, the stern block mass and its KG (G11). Coordinate with area 02. | G11 | Small to medium; GM goes down |
| 9 | Weather criterion (Part A 2.3) with a windage model from the GA (needs the lateral area and its centre). | G2 | Large for light conditions |
| 10 | GZ from the hull sections table (ADR 0012) with free trim, so KB, BM and GZ use the real hull (G6). | G6 | Unknown; possibly large at large angles |
| 11 | A selectable pontoon set (Part B 2.2) and an inland intact set; decide the default for river-only barges with the maintainer (G7). | G7 | Large: verdicts change for wide, shallow barges |
| 12 | Dry bulk: GZ and the `is2008-a` criteria for holds, then a cargo-shift model (G9). | G9 | Large (new results) |
| 13 | Add at least one calibration case with a published lightship KG or inclining result, and one harness case whose oracle does not use the box idealisation (G12). | G12 | None on numbers |

## Not verified in this draft

- BV NR217 Pt B Ch 2 Sec 2 [4.1] and its claimed match with ADN 9.3.1.14.2 (spec 0004): not read.
- IMO MSC.1/Circ.1281 (explanatory notes to the 2008 IS Code): not in the source library, not read. It may say how
  to read 2.2.2 and 2.2.3 for curves with two peaks.
- The claim that FSC × sin φ is close for a circular tank (question 3 on the page) is a geometric argument, not a
  cited source.
- No code was run. Line numbers are from the worktree as read on 2026-10-20.
