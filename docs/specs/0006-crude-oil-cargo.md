# Spec 0006: Crude oil cargo (tank barges and small coastal tankers)

Status: **agreed with the maintainer, 2026-10-20** (D1–D7 accepted as recommended). Decision record: ADR 0016.
Research: `docs/research/crude-oil/` (01 sea-going rules, 02 inland rules, 03 design practice, 04 codebase fit).

Concept and preliminary design only, never approval (ADR 0011).

## 1. Goal

A user can design a **crude oil tank barge** (pushed, towed or self-propelled, inland or coastal) or a **small
coastal tanker**. OVD sizes the cargo tanks inside a double hull, computes capacity, lightship, deadweight and
stability for the standard loading conditions, draws the tank arrangement, prices it, and checks it against the rules
that apply to its Service Area. Other oils (products, vegetable oil, chemicals) come later **as data rows**, not as new
code paths.

**Size band, first release:** up to 5,000 dwt (MARPOL Annex I Reg. 19.6 "small tanker" rules) and inland ADN
type N and C tank vessels, with the ES-TRIN rules above 110 m (28.03–28.04) listed. Above 5,000 dwt, the larger-tanker rules (Reg. 19.3, 23, 27, 28) are listed as
`not evaluated` rows until their Measures exist.

## 2. What drives a crude oil design

These are the requirements that most change a concept design, in order. Each one cites its research note.

| # | Driver | Effect on the design | Source |
|---|---|---|---|
| 1 | **ADN vessel type from cargo data.** Crude that is CMR needs at least closed type N with tank walls apart from the hull. More than 10 % benzene, H₂S or strong aquatic toxicity needs type C. UN 3494 (sour crude) is always type C. | Sets the whole cargo-area layout and the hull widths | 02 §1.2, ADN 2025 3.2.3.3 and Table C |
| 2 | **Double side and double bottom.** ADN C: 1.00 m side (0.80 m reinforced), 0.70 m mean bottom. ADN N: 0.60 m and 0.50 m. MARPOL below 5,000 dwt: bottom B/15 ≥ 0.76 m; side ≥ 0.76 m unless every tank ≤ 700 m³. NORMAM-202: 1.00 m and max(B/15, 0.76 m). | Inner-hull size, so cargo volume, ballast volume and steel weight | 01 §H.1, 02 §7 |
| 3 | **Cargo tank size.** ADN: ≤ 380 m³ per tank (from an L·B·H table) and ≤ 0.20 L long. MARPOL < 5,000 dwt without wings: ≤ 700 m³. | Number of tanks and transverse bulkheads, steel weight | 02 §8.3 |
| 4 | **Intact stability with free surfaces** at every loading stage and every density. MARPOL Reg. 27 from 5,000 dwt includes liquid transfer. ADN asks for explicit criteria when a tank is wider than 0.70 B. | One tank across the beam or two (centreline bulkhead) | 01 §H.2, 02 §8.5, 03 §5 |
| 5 | **Slack loaded tanks.** Crude (SG 0.77–0.97) often fills the draught before the tanks. Inland barges rated about 0.69 t/m³ load crude slack. | Free surface in the laden condition, not only in partial ones | 03 §1.1, §5.1 |
| 6 | **Damage stability.** ADN damage reaches only the ballast spaces (12° final heel). MARPOL-type extents reach the cargo tanks. | Wing compartment lengths; can set the draught | 02 §3, §7 |
| 7 | **Cargo-area zoning.** Cofferdams (0.60 m) at both ends; 2 m and 6 m setbacks to accommodation and openings; no oil forward of the collision bulkhead. | Usable cargo length inside L | 01 §H.6–7, 02 §2.2 |
| 8 | **Freeboard.** Coastal: ICLL Type A (unmanned barges may get 25 % less). Inland: ES-TRIN 4.01–4.05. | Depth for a target draught and deadweight | 01 §H.3, 02 §4 |
| 9 | **Slop tanks** 3 % of cargo volume (2 % with segregated ballast or crude oil washing) on sea-going tankers. | Net cargo capacity | 01 §H.8 |
| 10 | **Pumps and heating.** Pump room below deck is allowed for UN 1267 with conditions, not for UN 3494. Waxy crude needs heating: built vessels have 0.15–0.65 kW/m³. Above 80 °C cargo, ADN asks for independent tanks. | End-space length, machinery weight and power, cost | 02 §8.8–9, 03 §1.2 |
| 11 | **Tank pressure.** Closed tanks vent at 10–50 kPa; volatile crude may need 400 kPa pressure tanks. | Scantlings and weight (pressure tanks out of scope, first release) | 02 §8.6, 03 §1.3 |
| 12 | **Inert gas** from 8,000 dwt (SOLAS II-2/4.5.5). | Out of the first size band | 01 §H.10 |

## 3. Domain model

### 3.1 Cargo: crude oil with a grade

`crude_oil` is one Cargo. A **Crude Grade** gives its properties as data:

| Property | Unit | Why | Default (first release) |
|---|---|---|---|
| Density at 15 °C | kg/m³ | Mass, draught, structure | 870, `estimate` (mid-range of SG 0.77–0.97) |
| Design density for structure | kg/m³ | Scantlings and tank test head | 1,025, `estimate` (03 §1.1) |
| Sour (H₂S, UN 3494) | yes / no | ADN type C, no pump room below deck | no |
| CMR / benzene > 10 % / aquatic toxicity | flags | ADN vessel type (3.2.3.3) | **unknown → type C** (see D2) |
| Pour point | °C | Heating needed or not | none (no heating) |
| Carriage temperature | °C | Heating power, independent tanks above 80 °C | ambient |
| Vapour pressure class | low / volatile | Closed vs pressure tanks | low |

The grade gives a derived **ADN Tank Vessel Type** (N open, N closed, C) by the ADN 3.2.3.3 scheme. The user can
override it. With unknown flags the default is **type C** (the conservative choice, 02 §8.1).

### 3.2 Cargo Containment: integral cargo tanks

A new Cargo Containment, **Integral Tanks**: rectangular cargo tanks bounded by the inner bottom, the inner sides,
the deck (or a trunk), an optional centreline bulkhead and transverse bulkheads. **Independent Tanks** (an adapter on
the same seam: a clearance plus the tank's own shell) comes second, for heated cargo above 80 °C.

Layout inputs, each with a Defaults Profile row. `tanks_in_line` and `fill_ratio` already mean the Type C tanks, with
their own group and note, so the liquid inputs are `liquid_tanks_in_line` and `liquid_fill_ratio` (issue #331). A
`pump_room_length_m` input (default 4 m, used with a pump room) joins them:

| Input | Default | Basis |
|---|---|---|
| `double_side_m` | the governing Rule's minimum for the Option (`sizes: true`) | Rule Sets in §5 |
| `double_bottom_m` | same | same |
| `liquid_tanks_in_line` | the smallest count that meets the tank volume and length limits (the profile default is 4; `fit` works out the count for a hull, issue #340) | same |
| `centreline_bulkhead` | on when a tank would be wider than 0.70 B | ADN 9.3.x.13, BV NR217 (02 §8.5) |
| `cofferdam_m` | 0.60 | ADN 9.3.x.11.3 |
| `pump_arrangement` | `deepwell` (one pump per tank) | 03 §2.3; works for UN 1267 and 3494 |
| `slop_share` | 0.03 sea-going, 0 inland | MARPOL Reg. 29 |
| `heating_kw_per_m3` | 0 (none) | 03 §1.2, range 0.15–0.65 when used |
| `liquid_fill_ratio` | 0.98, `estimate` | the ADN degree-of-filling text must be read before it is `sourced` |

### 3.3 Spaces

The containment returns typed **Spaces**: cargo tank, slop tank, wing ballast, double bottom ballast, cofferdam,
pump room. Each has a box, a side (P, S, C), a gross volume and a centroid. Spaces are the **seed of the roadmap
phase 2 compartment model** (ADR 0016), not a one-off.

## 4. Effects on the engine

| Area | Change | Page in the engineering basis |
|---|---|---|
| Capacity | Cargo volume = Σ cargo tanks × fill. Volume-bound vs weight-bound as for dry bulk (`governs`). | 06 |
| Weights | Tank-barge lightship: a k-factor fit on the 41 tank-barge calibration records first; a plating build-up from the inner and outer hull second. Pumps, piping, heating coils and vents as outfit items (`estimate`). | 02 |
| Stability | Standard loading conditions: departure laden, arrival laden (10 % consumables), ballast, and the worst slack-tank condition. Free surface for rectangular slack tanks, split by the centreline bulkhead. Reuses the box GZ and the IS Code 2008 Part A criteria after they move out of the gas module. | 03 |
| Damage stability | The ADN 9.3.2.15 and 9.3.3.15 damages (issue #341): a side damage 0.79 m (type C) or 0.59 m (type N) inboard of the side and a bottom damage 3.00 m wide and 0.59 m or 0.49 m high, each 0.10 L but not under 5.00 m long, at every position along the hull that floods a different set of compartments. Ballast spaces are cut at the transverse bulkheads of the cargo tanks beside them, so a damage floods a compartment and not a whole tank block; cargo and slop tanks are full (0 % permeability), ballast spaces, cofferdams and the pump room are empty (95 %). Lost buoyancy method on the Sections Table: the mass and its centre of gravity do not change, the water in a flooded compartment is not displaced, and the floating position (waterplane height, trim, heel) is solved. The `stability.damage` Measure of each damage is true when the heel at equilibrium is at most 12 degrees and, beyond it, the righting lever reaches 0.05 m with an area of 0.0065 m.rad up to 27 degrees. Final stage of flooding only: the intermediate stages (9.3.x.15.2), the non-watertight openings (27 degrees stands in for the first one) and the flooding of undamaged compartments (9.3.x.15.3, 15.4) are not modelled, nor are the end spaces beyond the cargo zone. NORMAM-202 5.23.8 and 5.23.9 stay `not evaluated`. MARPOL Annex I Reg. 28 has its own damages and criteria (the Measure `stability.damage_marpol`, object `marpol-damage-case`, worked out when the Rule Set `marpol-i` is in the stack): side damage `(1/3) L^(2/3)` (at most 14.5 m) long and `B/5` (at most 11.5 m) deep, bottom damage `(1/3) L^(2/3)` (at most 14.5 m forward of 0.7 L, at most 5 m elsewhere) long, `B/6` (at most 10 m forward, 5 m elsewhere) wide and `B/15` (at most 6 m) high; above 150 m the damage is anywhere in the length, at 150 m or less between adjacent transverse bulkheads (a bulkhead is damaged only beside a bay shorter than the damage). A damaged cargo or slop tank is flooded at 95 % and the oil keeps its mass. The criteria are 28.3.2 (heel at most 25 degrees; the 30 degrees allowance needs no deck edge immersion, which is not checked) and 28.3.3 (range of 20 degrees beyond equilibrium, lever 0.1 m, area 0.0175 m.rad within it). Not modelled: openings (28.3.1), intermediate stages (28.3.4), the machinery space of 28.1.2, steps and recesses (28.2.5.2), the 20,000 t bottom raking damage of 28.6, the oil kept in a damaged tank, free surface of the partly full tanks. The validation against a published damage case with righting lever figures is open: none was found. | 03, 09 |
| Accidental oil outflow | MARPOL Annex I Reg. 23 (issue #341). From 5,000 t deadweight, the Measure `oil_outflow.ratio` (object `oil-outflow`) is OM over its limit (23.3.1: 0.015 up to 200,000 m3, 0.012 from 400,000 m3), OM by the damaged tank method of 23.4 to 23.9: the cargo at 98 % filling and a permeability of 0.99, the nominal density `1000 DW / C`, the probabilities of breaching each tank by a side damage (23.8: Xa, Xf, Zl, Zu, y) and a bottom damage (23.9: Xa, Xf, Yp, Ys, z), the side outflow as the whole tank, the bottom outflow by pressure balance at tides of 0 and -2.5 m (7.3.2, with at least 1 % for a tank on the bottom shell) times 0.6 for a tank over a non-oil space, `OM = (0.4 OMS + 0.6 OMB) / C`. The port and the starboard damage are averaged (explanatory notes 6.2, 6.3). The method reproduces the tank barge worked in the explanatory notes, Res. MEPC.122(52) part C 1, to the rounding of its probabilities (0.5 %). Under 5,000 t, `cargo-tank.marpol_length_ratio` is the tank length over the longest tank of 23.3.2.1 to 23.3.2.3 (10 m or the formula by the bulkheads abreast and `bi/B`). The model takes the hull as a prism (BS, BB the beam; the bottom shell flat at the base line), L as the overall length, dS as the draught of the full load, and an inert gas overpressure of 5 kPa (the text allows 0 without an inert gas system, which would leave more oil in the tank, so 5 kPa does not understate the outflow). C3 is 1.0 (the two continuous longitudinal bulkheads of 0.77 are not looked for) and fuel tanks in the cargo block (23.4.2) are not modelled. The results have an `oil_outflow` block outside the results hash. | 03 |
| Freeboard | ICLL Type A (coastal) and ES-TRIN (inland) as Rules; the depth rule group `liquid` (D/B 0.30) gives the default depth. | 01, 08 |
| Drawings | A cargo-tank arm in `ga.rs`: tanks P/S with numbers, centreline and transverse bulkheads, wing ballast hatched, cofferdams, the pump room (a deepwell pump has no symbol: the model has no size for one), midship section with inner hull. No pipe routing. | 11 |
| Cost | A cost item list per containment. Cargo tank list: tank steel, coating area, pumps, piping, heating, vents, gauging. Lump sums first, BOQ lines in the BOQ phase. | 10 |
| App | Crude oil shows "work in development" until damage stability and the oil Rule Sets are in. | — |

## 5. Rules engine

### 5.1 New Rule Sets

| Rule Set | Applies when | Contents (first release) |
|---|---|---|
| `adn-tank` | Service Area includes river; Cargo class liquid bulk | Vessel type from cargo data; double side and bottom by type; tank volume and length caps; cofferdams; setbacks; pump room; intact stability for tanks wider than 0.70 B; damage stability (evaluated for the ADN extents, issue #341) |
| `marpol-i` | Service Area includes coastal; Cargo class oil | Reg. 19.6 small-tanker double bottom and side or 700 m³ cap; Reg. 19.7 nothing forward of the collision bulkhead; Reg. 29 slop tanks; Reg. 28 evaluated for every size and Reg. 23 evaluated (issue #341; see the Accidental oil outflow row of section 5); Reg. 19.3 and 27 as `not evaluated` above 5,000 dwt |
| `icll-freeboard` | Service Area includes coastal | Type A freeboard, table 28.1, and the barge reduction (27.14), in the set `icll-freeboard` 1.0.0 (issue #428); the minimum bow height (39) is not codified: the model has no bow height Measure. A separate `icll-type-a` set is not made |
| `normam-202` | optional, chosen by the user | Brazilian inland tank barge widths and setbacks; an example of a national Rule Set |
| `ovd` (existing) | always | Physical checks: tank breadth > 0, double side < B/2, cargo zone length > 0 |

"The strictest rule governs" applies Measure by Measure. On coastal-and-river service, below 5,000 dwt:
double side = max(ADN type, MARPOL 0.76 m), double bottom = max(ADN type, B/15, 0.76 m). For damage stability no rule
is stricter on every criterion (ADN: heel 12°; MARPOL: range and area), so both run (02 §7).

All Rules start `unverified` and enter as `warn` (spec 0004). Rule ids follow `<family>/<paragraph>/<slug>`, as
listed in the research notes.

### 5.2 What spec 0004 must add

The codebase review (04 §6.2) found four gaps. Each needs a maintainer decision (D5):

1. **Numeric applicability.** MARPOL construction rules switch on deadweight bands (below or from 5,000 t). Proposal:
   an enumerated, derived, read-only input `deadweight_band`, so applicability stays `input ∈ {values}`.
2. **Per-object applicability.** A tank volume Rule must apply to cargo tanks, not to ballast or slop tanks. Proposal:
   one object type per role (`cargo-tank`, `ballast-tank`).
3. **Cargo class as an input.** Proposal: a derived input `cargo_class` (`gas`, `dry_bulk`, `liquid_bulk`, and an oil
   category), so a new oil needs no Rule Set edits.
4. **Clamp formula shape.** Double-hull widths are "a + b·DW, not less than x, need not exceed y". Proposal: a
   `max(lo, min(hi, a + b·x))` shape in Rust, coefficients as data, with a sourced test.

And one fix that crude needs before any oil Rule runs: **the gas Rule Sets get `applies_when` on containment**
(`type_c_tanks`), in #271 and #272. Without it a crude Option gets gas rows, as false passes or false failures.

### 5.3 New Measures

`hull.double_side_width`, `hull.double_bottom_height`, `hull.lbh`, `cargo-tank.volume`, `cargo-tank.length`,
`option.deadweight`, `option.cargo_capacity_m3`, `option.slop_capacity_share`, and later `option.oil_outflow_mean`.
The liquid-condition stability Measures reuse the existing `gz.*` and `gm0` Measures.

## 6. Code structure

From the codebase review (04 §5). Summary:

- **Refactor first, with no result change.** The cargo phase becomes a Rust enum (`CargoClass`), so the compiler finds
  every branch. Today an unknown phase goes down the gas path in `barge_capacity.rs` and is drawn as a dry hold in
  `ga.rs`, with no error. The box stability code moves to `box_stability.rs`. Hull plating moves out of `weights.rs`.
- **New deep module `ovd-core/src/liquid_bulk/`.** One interface: `compute(&LiquidBulkInputs) -> LiquidBulkResult`
  and `conditions(...)`. Layout, spaces, inner-hull clipping and free surface stay inside. The `TankContainment` seam
  has two adapters (integral, independent). The `InnerHull` seam stays private until the sections table (ADR 0012)
  gives it a second adapter.
- **Rules read Measures from the result**, never from the inside of the module.
- **Results hash:** new keys stay in the liquid path. Drawing-only values go in the unhashed `drawing` object.

## 7. Calibration and evaluation

- The 41 tank-barge records already in `calibration/cases.yaml` become runnable through a `tank` arm, held out.
- The 16 reference vessels with full data in 03 §3 become new cases (river barges, Danube barges, river-sea tankers).
  The 23 with partial data stay listed, not run. The cite-or-skip rule applies.
- Bands for tank-barge lightship and deadweight, as for `tank_bands.yaml`.
- Evaluation harness: liquid stability cases in the existing `kind: box` form with free surface. A new independent
  oracle for the double-hull and tank-volume Rules writes boundary cases (codification step 6).

Known gaps: no as-built double-hull widths, no steel, outfit and machinery split, no West African vessel data, cost
data only from two US contracts (03 §7).

## 8. Order of work

Filed as #328 to #342, in the order of this table (ticket n = #327 + n). "None" in the result column means the results hash does not move.

| # | Ticket | Result change | Depends on |
|---|---|---|---|
| 1 | Refactor: `CargoClass` enum, every phase branch a `match`; box stability to `box_stability.rs`; hull plating split | none | — (land before or rebase after #309) |
| 2 | Crude oil Cargo and Crude Grade data (Rust and Python, kept equal by a test); "work in development" status per Cargo | new Cargo only | 1; #315 (density shape) |
| 3 | `liquid_bulk` module: layout, Spaces, volumes, free surface, k-factor lightship | new results only | 1 |
| 4 | Format: containment `integral_tanks`, inputs, profile rows, `simulate_liquid`, fixtures, design-impact cases | new results only | 2, 3 |
| 5 | Loading conditions (departure, arrival, ballast, slack) and the stability output | new results only | 3 |
| 6 | Drawings: cargo-tank arm and labels | none | 4; #310 |
| 7 | App: cargo picker status tag, locale ids, cost lines | none | 4; #309, #311 |
| 8 | Calibration: tank arm, 41 existing records plus the 16 new cases, bands | none | 4 |
| 9 | Spec 0004 additions (§5.2) and gas sets `applies_when` on containment | none | #268, #269; inside #271, #272 |
| 10 | Measures of §5.3 | none | #269, 3 |
| 11 | Rule Sets `adn-tank`, `marpol-i`, `icll-type-a` 1.0.0, `unverified`, as `warn`; harness boundary cases | warnings only | 10; #273, #278 |
| 12 | Default Rule Set stacks for liquid bulk by Service Area | default stack | #274, #275 |
| 13 | `fit` for liquid bulk (tanks in line, double hull from the governing Rule): `liquid_fit` in `ovd-format`, and the `liquid_fit` entry of the evaluation API | `fit` results only; no `run` calls it | #276, #277 |
| 14 | Damage stability Measure (ADN extents first), then oil outflow | new checks | 5, 11 |
| 15 | Second oil (product, vegetable oil): data rows only. Done for diesel fuel to EN 590 (`diesel_en590`, issue #342): a Cargo row, a grade row, a fixture; no new branch on Cargo in Rust code (a test keeps it so) | new Cargo only | 2–11 |

Tank fix tickets #313, #314, #316 and #317 do not block this. #315 shares files and the density model: land it first
or agree its shape.

## 9. Decisions (accepted 2026-10-20, all as recommended)

- **D1 Scope.** First release up to 5,000 dwt and ADN type N and C tank vessels; larger-tanker rules as
  `not evaluated`. *Recommendation: yes.* Small coastal tankers sit inside ADR 0011; above 5,000 dwt they near the
  size where SOLAS applies in full.
- **D2 Default vessel type.** With unknown crude data, default to ADN **type C**. *Recommendation: yes*
  (conservative; the user sets the flags to get type N).
- **D3 Containment first.** Integral tanks first, independent tanks second, pressure tanks out of scope.
  *Recommendation: yes.*
- **D4 Pumps.** Default deepwell pumps, with a pump room as a choice. *Recommendation: yes* (allowed for both UN 1267
  and 3494).
- **D5 Spec 0004 additions** in §5.2: derived `deadweight_band` and `cargo_class` inputs, one object type per tank
  role, a clamp shape. *Recommendation: accept all four;* they keep the no-expression rule.
- **D6 National rules.** Ship `normam-202` as an optional Rule Set, as the model for other national inland rules (CCS,
  West African authorities when a text exists). *Recommendation: yes, optional, never in a default stack.*
- **D7 The ADN remark 38 conflict** for some high-benzene crude (02 §9). *Recommendation: treat as type C until an
  expert or the ADN Safety Committee answers; record it as a reviewer question.*
