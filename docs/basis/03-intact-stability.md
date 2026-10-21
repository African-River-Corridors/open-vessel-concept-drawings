# Intact stability

> Concept and preliminary design only. Not for approval. Draft: needs expert review.
>
> Code state: written against `main` at commit `f6d446c` (2026-10-20).

**For:** stability specialist  ·  **Code:** `crates/ovcd-core/src/gas_stability.rs`, `barge_capacity.rs` (`bulk_gm`),
`gas_arrangement.rs` (stability checks), `hydrostatics.rs` (legacy API only)  ·  **Rule Sets:** none loaded yet. The
six IS Code 2008 Part A 2.2 criteria are constants in code with citations. Spec 0004 plans them as the Rule Set
`is2008-a`.

OVCD is a concept tool (ADR 0011). This page says what the stability calculation does today. It is not a stability
booklet and it gives no verdict that an Administration or class can use.

## What OVCD computes here

For a **gas Cargo in Type C tanks** (`results.stability` in a `simulate` result, and the "Intact stability" view):

- Three loading conditions: `Laden`, `Half laden`, `Light`.
- For each condition: cargo mass, fill fraction, displacement, draught, KG, GM solid, free surface correction,
  GM fluid, the GZ curve from 0° to 60° on a 1° grid, maximum GZ and its angle, the areas 0–27°, 0–30°, 0–40°
  and 30–40°, the upright deck-edge immersion angle, and six criteria with value, limit, pass or fail and source.
- `governing`: the condition with the lowest fluid GM.
- `limitations`: plain-text statements of what the method leaves out.
- One check per condition (`Stability — laden`, and so on) in the `checks` list. These checks gate the
  arrangement search `fit` (#264): `fit` returns only an arrangement that passes all of them, or says that none does.

For a **dry bulk Cargo** (work in development):

- One number, `gm_m`: a first-order transverse GM at full load, and one advisory check `gm_positive`.
- No GZ curve, no criteria and no `results.stability` object (`crates/ovcd-format/tests/stability_curves.rs`).

## How it works today

### Gas Cargo (`gas_stability::assess_num`, `gas_stability.rs:690`)

1. **Inputs.** The converged gas arrangement (`GasBasis`, `gas_stability.rs:510`): LOA, beam B, depth D, double
   bottom height, tank support height, Tank Outer Diameter OD, inner diameter, total inner tank volume V, number
   of tanks n, cylinder length, and the lightship weight split. Also the payload, the block coefficient Cb and
   the water density ρ. The payload is the journey-derated tonnage (`gas_arrangement.rs:355`, `simulate.rs:1009`).
   *Method.*
2. **Lightship and its KG** (`gas_stability.rs:714`). Four items, each with a fixed height above the keel:
   hull steel (top-down) at 0.5 D; tanks and insulation at the tank centre (double bottom + support + OD/2);
   piping and machinery at D; outfitting at 0.75 D. *Fixed in code.* No stores, fuel, fresh water or ballast
   are in any condition. The weight of the self-propelled stern block is not in the split.
3. **Loading conditions** (`gas_stability.rs:723`). Laden = payload; Half laden = 0.5 × payload; Light = no cargo.
   The cargo is spread equally over all tanks. *Fixed in code.*
4. **Fill and cargo KG** (`gas_stability.rs:728`, `segment_fill` at `gas_stability.rs:278`). Fill fraction
   f = cargo / (ρ_cargo × V). OVCD treats f as the fraction of the circular cross-section area of the inner
   shell. It finds the segment angle θ from f = (θ − sin θ) / (2π) by an 80-step bisection. The liquid height is
   h = r (1 − cos(θ/2)), and the liquid centroid is r − 4 r sin³(θ/2) / (3 (θ − sin θ)) above the inner bottom.
   The inner bottom is the outer tank bottom plus (OD/2 − r_in). *Method* (circular-segment geometry), with the
   fill applied to the heads as if they were cylinder: *Estimate*.
5. **Displacement, KG, draught** (`gas_stability.rs:740`). Δ = Σ masses. KG = Σ (m × z) / Δ.
   T = Δ / (Cb × L × B × ρ). *Method.* If T ≥ D, the condition gets one failed check "Floats with freeboard" and no
   curve (`gas_stability.rs:744`). *OVCD Check, Fixed in code.*
6. **GM solid** (`box_gm_solid`, `gas_stability.rs:586`). GM = KB + BM − KG with KB = T/2 and BM = B²/(12 T), the
   values for a box. *Method.*
7. **Free surface** (`gas_stability.rs:776`). The free surface of each tank is a rectangle: the chord
   c = 2 √(r² − (r − h)²) wide, the cylinder length long. i = n × L_cyl × c³ / 12. FSC = ρ_cargo × i / Δ, in metres
   of GM. No free surface at f = 0 or f ≥ 1. GM fluid = GM solid − FSC. *Method* (IS Code Part B 3.1.8 form).
8. **GZ curve** (`box_gz`, `gas_stability.rs:242`; `box_condition_at`, `gas_stability.rs:596`). For each heel angle
   0° to 60° in 1° steps: rotate the rectangle B × D, find by an 80-step bisection the waterline that keeps the
   submerged area at B × T, clip the polygon (Sutherland–Hodgman) and take its centroid (shoelace). GZ_solid =
   y_G − y_B with y_G = −KG sin φ. GZ = GZ_solid − FSC × sin φ. *Method.* The section is a prismatic box over the
   full length: no rakes, sheer, camber, coaming, bulwark or tank dome buoyancy, and no trim.
9. **Curve properties** (`gas_stability.rs:611`). Maximum GZ and its angle on the grid (first of equal values).
   Areas by the trapezoid rule on the 1° grid, in m·rad (`area`, `gas_stability.rs:408`). Upright deck-edge angle
   atan(2 (D − T)/B) (`gas_stability.rs:612`), reported only. *Method.*
10. **Criteria** (`criteria`, `gas_stability.rs:430`). Six comparisons `value ≥ limit`, no tolerance, cited to
    "IMO IS Code 2008, Part A 2.2.n". See rows IS-A30 to IS-A35. *Rule (unverified).*
11. **Use.** `simulate` runs `assess` only when the payload is above zero and the result has a gas basis
    (`crates/ovcd-format/src/simulate.rs:1009`). `fit` makes one error-severity check per condition
    (`stability_check`, `gas_arrangement.rs:318`); if the assessment raises an error, it makes one failed check
    "Stability — not assessed" (`gas_arrangement.rs:373`). The criteria set is always `IS_CODE_2008`, whatever
    ruleset the Option names (`criteria_set_for`, `gas_stability.rs:504`).

### Dry bulk Cargo (`bulk_gm`, `barge_capacity.rs:685`): work in development

1. Full-load draught from the hull geometry (area 01). KB and BM_T come from `HullGeometry::KB` and
   `HullGeometry::BM_transverse` (`hull.rs:171`, `hull.rs:224`). *Method.*
2. KG: lightship at 0.5 D; cargo at the double bottom plus the centroid of the trapezoidal hold section over the
   full hold height (depth − double bottom + coaming), h/3 × (2 w_top + w_bot)/(w_top + w_bot). *Estimate.*
3. GM = KB + BM − KG. No free surface, no GZ, no criteria, full load only.
4. Check `gm_positive`: GM ≥ 0.15 m, severity "warning" (does not gate), plus a warning that the method is first
   order and does not model cargo shift (`barge_capacity.rs:1004`, `barge_capacity.rs:814`). It runs only when the
   draught is above zero and the hull is not overloaded. *Fixed in code.*
   Since #415 a Design's results carry the check `intact gm at reported cargo` instead: the same first-order GM at the cargo and draught the Design reports, in every dry bulk Option (Rule `ovcd/capacity/intact-gm`, `ovcd` 2.0.0).

### Legacy API (`hydrostatics::calculate_stability`, `hydrostatics.rs:61`)

A port of the original engine's GM function. Only the Python API `hydrostatics.calculate_stability` and
`ovcd_format::eval` call it; `simulate` and `fit` do not. It differs from `gas_stability` (cargo KG at the tank
centre, a stores item, the fill read as a height fraction). see the triage tickets.

## Assumptions, rules and estimates

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| IS-A01 | Heel angle grid | 0° to 60°, 1° step | Fixed in code | Port of the original engine | `gas_stability.rs:28` `ANGLES_DEG` | Unknown: a curve with its peak or vanishing angle above 60° is cut |
| IS-A02 | Section for GZ | Prismatic box B × D, full length, even keel | Estimate | Concept-stage idealisation; same idealisation as the harness oracle | `box_gz`, `gas_stability.rs:242` | Unknown: no reserve buoyancy from sheer, coaming or domes (conservative); no rakes or trim (either way) |
| IS-A03 | Heeled waterline | Bisection, 80 steps, constant submerged area B × T | Method | Constant-displacement free-heel calculation | `gas_stability.rs:262` | Small (numerical) |
| IS-A04 | Submerged shape | Sutherland–Hodgman clip, shoelace area and centroid | Method | Standard polygon geometry | `gas_stability.rs:206`, `:224` | Small (numerical) |
| IS-A05 | Draught | T = Δ / (Cb L B ρ) | Method | Block-coefficient draught | `gas_stability.rs:743` | Unknown: follows the Cb input |
| IS-A06 | KB | T/2 | Method | Box value | `box_gm_solid`, `gas_stability.rs:586` | Likely conservative: a shaped hull has KB above T/2 |
| IS-A07 | BM | B²/(12 T) | Method | Box value (waterplane coefficient 1, divided by the box volume) | `gas_stability.rs:586` | Unknown: with Cb < 1 both the true inertia and the true volume are smaller |
| IS-A08 | GM solid | KB + BM − KG | Method | Initial stability | `gas_stability.rs:588` | — |
| IS-A09 | Hull steel KG | 0.5 × D (input depth) | Fixed in code | Port; no source | `gas_stability.rs:715` | Unknown; the mass uses the moulded depth, which can exceed D with wing tanks |
| IS-A10 | Tanks and insulation KG | Tank centre: double bottom + support + OD/2 | Estimate | Geometry of the tank | `gas_stability.rs:709`, `:716` | Small |
| IS-A11 | Piping and machinery KG | D (deck level) | Fixed in code | Port; no source | `gas_stability.rs:717` | Likely conservative for deck piping; unknown for machinery |
| IS-A12 | Outfitting KG | 0.75 × D | Fixed in code | Port; no source | `gas_stability.rs:718` | Unknown |
| IS-A13 | Tank dome mass | Not in the weight split | Fixed in code | Domes count in Vc only (`barge_capacity.rs:465`) | `gas_stability.rs:714` | Non-conservative, small (high mass left out) |
| IS-A14 | Stern block mass | Not in KG or Δ; a limitation line is added | Fixed in code | Gas lightship has no aft block | `gas_stability.rs:798` | Unknown; the code says GM could be lower |
| IS-A15 | Stores, fuel, fresh water, ballast | None in any condition (stores = 0 t) | Fixed in code | `weights.rs:194` | `gas_stability.rs:714` | Unknown for KG; non-conservative for free surface |
| IS-A16 | Weight-split check | Limitation line if split differs from lightship by more than 1 t | Fixed in code | Consistency check | `gas_stability.rs:805` | — |
| IS-A17 | Loading conditions | Laden (payload), Half laden (0.5 × payload), Light (no cargo) | Fixed in code | Port; not the IS Code Part B 3.4.1.2 standard set | `gas_stability.rs:723` | Unknown: arrival conditions, ballast and loading stages are not checked |
| IS-A18 | Cargo distribution | Same fill in every tank | Fixed in code | Homogeneous loading | `gas_stability.rs:728` | Non-conservative for slack-tank cases (one tank part full) |
| IS-A19 | Fill fraction | f = cargo / (ρ_cargo × V); V is the inner volume without domes; f applied to the circular section | Estimate | Heads treated as cylinder for the level | `gas_stability.rs:728`, `:733` | Small |
| IS-A20 | Cargo KG | Inner bottom + circular-segment centroid height | Method | Circular-segment geometry | `segment_fill`, `gas_stability.rs:278`; `:734` | Small |
| IS-A21 | Segment angle | Bisection, 80 steps, on (θ − sin θ)/(2π) = f | Method | Circular-segment area | `gas_stability.rs:308` | Small (numerical) |
| IS-A22 | Free surface inertia | i = n × L_cyl × c³/12; heads left out | Method | Rectangle of chord width; no source cited in code | `gas_stability.rs:776` | Non-conservative, small (head liquid left out) |
| IS-A23 | Free surface correction to GM | FSC = ρ_cargo × i / Δ | Method | IS Code 2008 Part B 3.1.8 (inertia at 0°) | `gas_stability.rs:782` | — |
| IS-A24 | Free surface correction to GZ | FSC × sin φ at every angle | Method | Inertia at 0° held constant (Part B 3.1.9 lets the Administration agree the method) | `box_condition_at`, `gas_stability.rs:609` | Small for the cylindrical part (see question 3) |
| IS-A25 | Free surface limits | None at f = 0 or f ≥ 1 | Fixed in code | Part B 3.1.2 asks for a correction at 98 % for nominally full cargo tanks; laden f ≤ the fill ratio (0.98 default), so the correction applies | `gas_stability.rs:776` | Non-conservative only if a typed fill ratio reaches 1.0 |
| IS-A26 | Free surface of consumables and ballast | None | Fixed in code | Not modelled | — | Non-conservative |
| IS-A27 | Water density | 1.0 t/m³ | Estimate | Defaults Profile `water_density_t_per_m3`, basis "fresh water"; `DEFAULT_WATER_DENSITY`, `gas_stability.rs:581` | Draught only | Small; sea water gives a smaller draught |
| IS-A28 | Block coefficient | 0.85 | Estimate | Defaults Profile `block_coefficient`, basis calibrated box barges; `DEFAULT_BLOCK_COEFFICIENT`, `gas_stability.rs:580` | Draught | Unknown |
| IS-A29 | Area under GZ | Trapezoid rule on 1° grid, m·rad | Method | Grid error about 0.0005 m·rad (`evaluation/README.md`) | `area`, `gas_stability.rs:408` | Small |
| IS-A30 | Area 0–30° | ≥ 0.055 m·rad | Rule (unverified) | IMO MSC.267(85), 2008 IS Code, Part A 2.2.1 | `gas_stability.rs:441` | Matches the text |
| IS-A31 | Area 0–40° | ≥ 0.090 m·rad, always to 40° | Rule (unverified) | Part A 2.2.1 ("40° or the angle of down-flooding if less"); finding #282 | `gas_stability.rs:448` | Non-conservative when θf < 40° |
| IS-A32 | Area 30–40° | ≥ 0.030 m·rad, always to 40° | Rule (unverified) | Part A 2.2.1; finding #282 | `gas_stability.rs:455` | Non-conservative when θf < 40° |
| IS-A33 | GZ at 30° or more | Max GZ over 30°–60° ≥ 0.20 m | Rule (unverified) | Part A 2.2.2 | `gas_stability.rs:433`, `:462` | Matches the text, inside the 60° grid |
| IS-A34 | Angle of max GZ | ≥ 25°, integer degree of the global maximum on 0°–60° | Rule (unverified) | Part A 2.2.3 (the alternative criteria clause is not modelled) | `gas_stability.rs:468` | Grid error 1° |
| IS-A35 | Initial GM | GM fluid ≥ 0.15 m | Rule (unverified) | Part A 2.2.4 | `gas_stability.rs:476` | Matches the text |
| IS-A36 | Comparison | value ≥ limit, no tolerance | Fixed in code | Spec 0004 keeps it as tolerance 0 | `gas_stability.rs:444`–`:479` | — |
| IS-A37 | Down-flooding angle | None; areas always run to 40° | Fixed in code | Known limitation, finding #282; limitation line "no downflooding openings" | `gas_stability.rs:627`–`:629`, `:793` | Non-conservative |
| IS-A38 | Weather criterion (Part A 2.3) | Not evaluated; limitation line "No wind heel" | Fixed in code | Needs the wind heeling lever | `gas_stability.rs:795` | Non-conservative |
| IS-A39 | Deck-edge angle | atan(2 (D − T)/B), reported only | Method | Upright box geometry | `gas_stability.rs:612` | — |
| IS-A40 | Area 0–27° | Computed and reported; no criterion | Fixed in code | Left from the removed ADN set (#175) | `gas_stability.rs:626` | None (no use) |
| IS-A41 | Governing condition | Lowest GM fluid, first of equal | Fixed in code | Display choice; not the smallest Margin | `governing`, `gas_stability.rs:396` | None on pass or fail |
| IS-A42 | Condition that does not float | T ≥ D: one failed check "Floats with freeboard", no curve | Fixed in code | OVCD Check (physical feasibility) | `gas_stability.rs:744` | — |
| IS-A43 | Criteria set | Always `IS_CODE_2008` | Fixed in code | Both IGC rulesets use the IS Code intact criteria | `criteria_set_for`, `gas_stability.rs:504` | — |
| IS-A44 | When stability runs | Payload > 0 and gas basis present | Fixed in code | — | `simulate.rs:1009`, `gas_arrangement.rs:355` | No stability for an empty result |
| IS-A45 | Gating | Each condition check has severity "error": it gates `fit`; an assessment error is one failed check | Fixed in code | Design change #264 | `gas_arrangement.rs:318`, `:369` | — |
| IS-A46 | Applicability by length | Criteria applied to every length | Fixed in code | Part A 1.1.1 applies to ships of 24 m and over | `gas_stability.rs:430` | Conservative below 24 m in principle; no other set is offered |
| IS-A47 | Trim | Not in the GZ; even keel | Fixed in code | Prismatic section | `box_gz` | Unknown |
| IS-A48 | Bulk GM | KB + BM_T − KG at full-load draught, from hull geometry | Method | Initial stability | `bulk_gm`, `barge_capacity.rs:685` | — |
| IS-A49 | Bulk lightship KG | 0.5 × D | Fixed in code | Port; no source | `barge_capacity.rs:704` | Unknown |
| IS-A50 | Bulk cargo KG | Double bottom + trapezoid centroid over the full hold height, coaming included | Estimate | Hold fill (0.95) not used for the level | `barge_capacity.rs:697`–`:703` | Conservative (cargo placed high) |
| IS-A51 | Bulk GM limit | GM ≥ 0.15 m, warning only | Fixed in code | Same number as Part A 2.2.4; no citation in the check | `barge_capacity.rs:1004` | Does not gate |
| IS-A52 | Bulk scope | No free surface, no cargo shift, no GZ, full load only | Fixed in code | Warning text `BULK_GM_WARNING` | `barge_capacity.rs:814` | Unknown |
| IS-A53 | Bulk check guard | Runs only if draught > 0 and not overloaded | Fixed in code | — | `barge_capacity.rs:1002` | — |
| IS-A54 | Legacy cargo KG | Tank centre for any fill | Fixed in code | Port | `hydrostatics.rs:94` | Conservative below full |
| IS-A55 | Legacy stores KG | Double bottom/2, or 0.25 D if no double bottom | Fixed in code | Port | `hydrostatics.rs:106` | Unknown (stores mass is 0) |
| IS-A56 | Legacy free surface fill | `fill_ratio` read as a fraction of the diameter | Fixed in code | Port | `tanks.rs:280` | Unknown; differs from IS-A19 |

## Inputs the user can change

| Input | Default | Unit | Where the default comes from |
|---|---|---|---|
| `block_coefficient` | 0.85 | – | Defaults Profile (`crates/ovcd-format/src/builtin_profile.json`); basis: calibrated box barges 0.883 to 0.995, widened for shaped gas hulls |
| `water_density_t_per_m3` | 1.0 | t/m³ | Defaults Profile; basis: fresh water. One value for draught, stability and resistance |
| `fill_ratio` | 0.98 | – | Defaults Profile; source IGC Code MSC.370(93) 15.3. It sets the cargo volume, so the laden fill |
| `hold_fill_ratio` (bulk) | 0.95 | – | `barge_capacity.rs:243`; not used for the bulk cargo KG |

The stability method itself has no inputs. The KG heights, the loading conditions, the angle grid and the criteria
are fixed in code. Other inputs (beam, depth, double bottom, tank support height, Tank Outer Diameter, cargo
density) reach stability through the arrangement; see areas 01, 02, 04 and 05.

## Limits and known findings

- **Down-flooding angle (#282).** OVCD has no openings, so the 0–40° and 30–40° areas always run to 40°.
  Part A 2.2.1 runs them to the down-flooding angle if that is less. Spec 0004 links both Rules to this finding.
- **Weather criterion.** Part A 2.3 is not evaluated. Spec 0004 plans it as a `not evaluated` row until the wind
  heeling lever exists (roadmap phase 1).
- **Damage stability.** Not evaluated (limitation line; roadmap phase 2).
- **Loading conditions.** Three conditions with no consumables or ballast. Part A 2.1.1 points to the standard
  conditions of Part B 3.4; IGC 2.2.2 adds loading and unloading stages.
- **Box section.** The GZ comes from a box B × D at even keel. The hull form (area 01) does not reach the GZ.
- **Free surface.** Cargo tanks only, cylinder part only.
- **Dry bulk.** Work in development: one first-order GM at full load, advisory only. Spec 0004 lists the
  `is2008-a` rows as `not evaluated` for dry bulk until roadmap phase 1.
- **Pontoon criteria.** IS Code Part B 2.2 (pontoons) is not offered. Spec 0004 names it as a later target for
  barges.
- **Rule status.** No IS Code criterion is a Rule in a loaded Rule Set yet. The spec 0004 audit records all six
  as "Cited, matches" except the down-flooding note. This draft re-read Part A 2.2.1 to 2.2.4 in MSC.267(85) and
  agrees with the values. A second-reader record does not exist yet, so the label stays "Rule (unverified)".

## Evidence

- **Evaluation harness, phase 0 (#184)** (`evaluation/results.md`): 5 base cases and 60 boundary mutants of box
  sections with a known KG, through `box_condition`. 361 criterion verdicts compared: 0 false fails, 0 false
  passes. Three base cases have published inputs; all verdicts are derived by the oracle. The oracle
  (`evaluation/oracle/box_oracle.py`) solves the heeled waterline exactly and integrates with Simpson's rule on a
  0.01° step. It uses the same box idealisation and the same polygon clip as the engine, so it does not test
  IS-A02.
- **`tests/test_evaluation_oracle.py`**: the oracle against closed forms (GM, wall-sided GZ) and
  `box_condition` against `assess`.
- **Unit and seam tests**: `crates/ovcd-core/tests/gas_stability.rs` (31 tests, values from the original Python),
  `crates/ovcd-core/tests/box_condition.rs` (4), `crates/ovcd-core/tests/gas_arrangement_feasible.rs` (7, the
  search keeps only arrangements that pass), `crates/ovcd-format/tests/stability_curves.rs` (5), and
  `every_is_code_criterion_passes_at_exactly_its_limit` in `gas_stability.rs`.
- **Calibration**: no calibration case holds a published KG, GM or GZ. The stability results have no check
  against a real vessel.

## Questions for reviewers

1. Is a prismatic box section B × D at even keel fair for the GZ of a raked, flat-bottom gas barge at concept
   stage, up to 60°? Which correction would you apply first: rakes, trim or reserve buoyancy of coamings and domes?
2. Are the lightship heights fair for a concept: hull steel 0.5 D, piping and machinery at D, outfitting 0.75 D?
3. For a horizontal circular tank, the liquid surface keeps its width at any heel. Do you agree that
   FSC × sin φ with the 0° inertia (IS-A24) is then close for the cylinder part, and is leaving out the heads
   acceptable at concept stage?
4. Which loading conditions should a concept gas barge check: the Part B 3.4.1.2 departure and arrival set, the
   IGC 2.2.2 loading stages, a one-tank-slack case? Is Half laden at equal fill a fair worst case for free surface?
5. For wide, shallow barges the angle of maximum GZ is often below 25° (Part A 2.2.3). Should OVCD offer the
   Part B 2.2 pontoon criteria, or an inland set, as the default for river-only barges?
6. Without openings data, what default down-flooding angle, if any, should a concept assume for a gas barge
   (for example the deck-edge angle, or a vent height above deck)?
7. For dry bulk, is a full-load GM with the cargo at the full hold height enough at concept stage, or is a GZ
   with a grain-type heeling moment needed first?
