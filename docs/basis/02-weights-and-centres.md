# Weights and centres

> Concept and preliminary design only. Not for approval. Draft: needs expert review.
>
> Code state: written against `main` at commit `f6d446c` (2026-10-20).

**For:** naval architect, weight engineer  ·  **Code:** `crates/ovd-core/src/weights.rs`, `barge_capacity.rs`,
`gas_stability.rs` (weight split), `hydrostatics.rs` (legacy centres), `models.rs`, `aft_arrangement.rs`;
`crates/ovd-format/src/simulate.rs`, `builtin_profile.json`; `calibration/`  ·  **Rule Sets:** none. No Rule
in this area. Every weight and centre is a method, an estimate or a value fixed in code.

## What OVD computes here

- **Lightship** (`lightship_t`), in tonnes, for three Hulls: the gas barge with IMO Type C tanks, the box-hold dry
  bulk barge and the sloped-hopper dry bulk barge.
- **Weight breakdown** of the gas barge (`WeightBreakdown`): top-down hull steel, bottom-up hull steel and its
  parts, tank steel, insulation, piping and machinery, outfitting, and the steel delta (top-down less bottom-up).
- **Displacement** (`displacement_t`) and **deadweight** (`deadweight_t`). The deadweight is the cargo only.
- **Vertical centre of gravity (KG)** of each loading condition: three conditions for the gas barge (Laden, Half
  laden, Light) and one full-load condition for the hopper barge. Page 03 uses these KG values.
- **Calibration factors** for dry barges: the lightship factor k (t/m³ of L·B·D) per hold type and the box-barge
  block coefficient, fitted from Reference Vessels.

OVD gives no longitudinal or transverse centre of gravity in the main result. See "Limits".

## How it works today

### 1. Gas barge lightship

The gas branch (`gas_volumetric`, `crates/ovd-core/src/barge_capacity.rs:474`) builds a `BargeInputs` from the
dataclass defaults (`models.rs:166`), sets the tank and arrangement inputs, and calls `calculate_weights`
(`barge_capacity.rs:571`) on a box hull with no rakes (`barge_capacity.rs:570`).

`calculate_weights` (`crates/ovd-core/src/weights.rs:44`) does two steel estimates. Only the first goes into the
lightship.

1. **Hull steel: the panel model** (Method, issue #722; maintainer decision 2026-10-30), `hull_panels.rs`. The gas
   hull's lightship, piping, outfitting and cost take their hull steel from a build-up of the structural members:
   bottom shell, side shell and strength deck (midship 0.4L and ends), inner bottom, inner side, wing tank top,
   transverse bulkheads (one per hold end, a collision and an aft peak bulkhead), centre and side girders, solid
   floors, wing web frames, side web frames and deck transverses. Each member is a record with its extent (x, y and z
   bounds), plate thickness, stiffener direction, spacing and section, its supporting primary members and its rule
   paragraphs (`cost_masses.hull_panels` in the results). The scantlings are the class minimums of the ABS Rules for
   Building and Classing Steel Barges at the January 2026 Rules (Part 3 January 2026, Part 5 July 2025; the Rule Set
   `abs-steel-barges`, issue #740), with the rule length L of 3-1-1/3.1, which ABS 5-5/11.1 applies to a liquefied gas tank barge. The hull
   girder check of ABS 3-2-1/3.1 (SMR = C1 C2 L^2 B (Cb + 0.7), no depth term, K = 1) thickens the deck or bottom
   over 0.4L amidships when the midship section falls short. Only continuous longitudinal structure counts in that
   section: the deck strips outside the tank cut-outs, not the covering above D (see 6). Cb is the
   `block_coefficient` input (0.85, an Estimate: the laden value would need the lightship first). The weight is the
   sum over the records (plate area x thickness x 7.85 t/m3, plus stiffeners and primary members) plus **an allowance
   of 5 % for brackets, collars, welds and small items (an Estimate: no primary source gives one figure)**. The
   structure factor of 2.0 (#401, step 2) is retired. Concept and preliminary design only (ADR 0011): class minimum
   scantlings, not a structural approval.
   - **In the panel weight:** shell, deck, inner bottom, inner side and wing tank top plating; their longitudinals;
     transverse bulkhead plating with its stiffeners and vertical webs; double-bottom girders and floors with the
     floor stiffeners; wing web frames (plate diaphragms); side web frames; deck transverses.
   - **Not in it:** brackets, collars and welds (only through the 5 % allowance); openings and lightening holes (not
     deducted); a centreline bulkhead; tank saddles and their foundations; machinery seats, the engine room and
     casing; the superstructure and wheelhouse steel; buckling stiffening; the covering above D (its own items, 6).
   - The **top-down steel** (0.10 t/m3 x LOA x B x D) is still reported as `steel_top_down`, for comparison only.
   - **Bottom-up steel** = the panel steel + the covering's parts above D (wing plate, side plating to it, weather
     shield). The cost prices it, so no member below D is counted twice. The #404 disagreement warning no longer
     applies to a gas hull: its lightship and cost take the same steel.
   - Calibration: `calibration/hull_steel_cases.yaml` (no factor tuned).
3. **Tank weight** (`tank_total`, `insulation`): see page 04. This page uses the two values as they come.
4. **Piping and machinery** = 0.04 × (steel_top_down + tank_total), `weights.rs:170` (Fixed in code).
5. **Outfitting** = 0.035 × (steel_top_down + tank_total), `weights.rs:173` (Fixed in code).
6. **Covering above D** (Method, `weights::covering_items`, issue #410). By default everything above D is a
   non-structural weather covering, not hull girder. Its parts above D are lightship items, each with its own
   centre (see section 5): the wing plate, the side plating from D up to the wing plate, and the weather shield
   (6 mm, top at the tank top + 1.5 m). The plates take the bottom-up thicknesses. The covering is structural only
   by the user's choice of D: when D is at or above the wing plate and the shield top, the hull steel holds it and
   the list is empty.
7. **Lightship** = steel_top_down + tank_total + insulation + piping + outfitting + the covering items (Method).
   Piping and outfitting stay factors on the steel to D and the tanks.

Stores are an explicit zero (`weights.rs:194`). The `stores_weight` input (200 t, `models.rs:218`) is not read.

Example from the parity test (`crates/ovd-core/tests/weights.rs:38`), 90 × 14.8 × 6.5 m, four tanks:
top-down steel 865.8 t, bottom-up steel 568.7 t (delta 297.1 t), tank total 446.8 t, insulation 43.0 t, piping
52.5 t, outfitting 45.9 t, lightship 1,454.0 t.

### 2. Dry bulk barge lightship

`bulk_volumetric` (`barge_capacity.rs:610`) has two methods, chosen by hold type.

- **Box hold** (`box`, `container`), `barge_capacity.rs:652-660` (Method):
  lightship = k × LOA × B × D. k comes from the Defaults Profile (`lightship_factor_t_per_m3`, box hold 0.1086
  t/m³, `builtin_profile.json:80`). The app has no container hold, so only the box k is used.
- **Sloped hopper** (`sloped_hopper`), `barge_capacity.rs:634-651` (Method, carried over from the original engine):
  - hull steel = 0.10 t/m³ × LOA × B × D.
  - hold steel = (inner bottom + two hopper slopes + two wing bulkheads + (n_holds + 1) transverse bulkheads +
    two coamings) plate volume × 7.85 t/m³ × 1.40. Inner bottom = l_cargo × B × t_ib; slopes = 2 × h_hold ×
    √(s² + 1) × l_cargo × t_hop; wing bulkheads = 2 × h_hold × l_cargo × t_bhd; transverse bulkheads = (n + 1) ×
    hold section area × t_bhd; coamings = 2 × h_coaming × l_cargo × t_coam.
  - lightship = (hull steel + hold steel) × (1 + 0.04 + 0.035).

The hold geometry (h_hold, widths, l_cargo) is on page 06.

### 3. The stern block and machinery

- **Self-propelled Hull.** The stern block (thrusters, battery or genset units, accommodation units, lifting
  wheelhouse) sets the stern void length (`barge_capacity.rs:782-786`). Its mass is **not** in the lightship of
  either branch. The gas stability result says so (`gas_stability.rs:798-803`). The dry branch gives no warning.
- **Gensets.** For a gas-fuelled stern block, the engine dry mass of the chosen sets is reported
  (`genset_engine_mass_t`, `aft_arrangement.rs:1025`) from a published maker table (`aft_arrangement.rs:136`).
  The value is not added to the lightship.
- **Pushed barge.** The crew, battery and propulsion are on the tug and are not in the lightship
  (`barge_capacity.rs:810-812`).
- **"Piping and machinery"** on the gas barge is the 4 % factor in step 1.4. It is not a machinery estimate.

### 4. Displacement and deadweight

- **Volume-bound load** (`compute`, `barge_capacity.rs:892-905`): total weight = lightship + cargo capacity. The
  equilibrium draught T comes from `find_equilibrium_draft` (`hydrostatics.rs:21`), clamped to D. Displacement
  = Cb × V(T) × ρ. For the box hull V(T) = LOA × B × T. Deadweight = displacement − lightship
  (`barge_capacity.rs:1032`).
- **Governing load** (`simulate`, `crates/ovd-format/src/simulate.rs:740-751`, `797`): the cargo is the smaller of
  the volume-bound cargo and the weight-bound cargo, Cb × LOA × B × T_max × ρ − lightship. Displacement =
  lightship + cargo. Deadweight = displacement − lightship = cargo. The gas branch reports the payload as the
  deadweight (`simulate.rs:964`, `1081`).
- No fuel, fresh water, stores, crew or ballast is in the deadweight. `ballast_capacity_t` (0 t,
  `barge_capacity.rs:212`) only sets a ballast draught for the air draft.
- Cb comes from the Defaults Profile, 0.85 (`builtin_profile.json:52`), for every Hull. ρ is 1.0 t/m³
  (`builtin_profile.json:633`).

### 5. Vertical centres of gravity

**Gas barge** (`assess_num`, `crates/ovd-core/src/gas_stability.rs:690`). Four lightship items, each at one
height above the keel (`gas_stability.rs:714-719`):

| Item | Mass | KG |
|---|---|---|
| Hull steel | steel_top_down | 0.5 × D |
| Tanks and insulation | tank_total + insulation | tank centre = DB + h_support + OD / 2 |
| Piping and machinery | piping_machinery | D |
| Outfitting | outfitting | 0.75 × D |
| Wing plate (above D only) | plate over the beam, less the tank cut-outs | wing plate level |
| Side plating D to the wing plate | 2 × height × LOA × side plate | mid-height |
| Weather shield top | B × tank zone × 6 mm | tank top + clearance |
| Weather shield sides and ends | sides and ends above max(wing plate, D) | mid-height |

D is the hull depth. The covering items also carry an LCG for the trim: the wing plate's tank-zone part at the
tank zone centre and its full-beam parts in the bow and stern zones, the side plating at mid-length, the shield
at the tank zone centre (issue #410). The cargo is at the centroid of a filled circular segment of the tank inside
diameter (`gas_stability.rs:733-734`). KG = Σ(m × z) / Σm. The code adds a limitation if the four items differ from
the lightship by more than 1 t (`gas_stability.rs:805`).

**Hopper barge** (`bulk_gm`, `barge_capacity.rs:685-712`). Lightship KG = 0.5 × D. Cargo KG = DB_bulk + centroid
of the full trapezoidal hold, h/3 × (2 w_top + w_bot)/(w_top + w_bot). KG is the mass-weighted mean. The fill
ratio does not lower the cargo centroid.

**Legacy chain** (`calculate_stability`, `hydrostatics.rs:61`, and `calculate_trim`, `hydrostatics.rs:163`).
Reached only through the parity interface (`crates/ovd-format/src/eval.rs:1544`, `1570`), not by `simulate`.
KG: bottom-up hull steel at D/2, tank shells and insulation at the tank centre, cargo at the tank centre, piping at
D, outfitting at 0.75 D, stores at DB/2 (or 0.25 D with no double bottom). LCG from the aft end: hull and piping at
LOA/2, tanks and cargo at the centre of the tank zone, outfitting and stores at half the stern non-tank length.
Trim = (LCG − LCB) × L_wp / BM_L.

### 6. Calibration factors

`src/ovd/calibration.py` (`fit_factors`, line 107) fits the factors from `calibration/cases.yaml` and
writes `calibration/factors.yaml`. Held-out cases are left out of the fit.

- **k per hold type** = mean of observed lightship / (LOA × B × D). Box 0.1086 t/m³ (n = 2, range 0.095–0.1223);
  container 0.1711 t/m³ (n = 1).
- **Box-barge Cb on LOA** = mean of (observed DWT + lightship) / (LOA × B × T × ρ). 0.928 (n = 7, range
  0.883–0.995, 2 cases excluded). Where a case has no observed lightship, the fit uses k × L·B·D
  (`calibration.py:132`).
- **Deck barges and pontoons** have no factor. The harness runs them as box barges (`run_as: box`) with the box k
  and box Cb.
- The fitted Cb is used by the calibration harness (`calibration.py:161`). The app default is 0.85.

## Assumptions, rules and estimates

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| WC-A01 | Gas hull steel | the panel model: members with ABS Steel Barges minimum scantlings (the Rule Set `abs-steel-barges`, January 2026 Rules, issue #740; the rule length of 3-1-1/3.1), plus 5 % for brackets, welds and small items | Method; the allowance is an Estimate | Issue #722 (maintainer decision 2026-10-30); the top-down 0.10 t/m³ is reported only | `hull_panels.rs`; `weights.rs` | Lightship, draught, stability and cost |
| WC-A02 | Hull depth for top-down steel; covering above D | Steel to D; wing plate, side plating and weather shield above D are lightship items at their own centres | Method | Maintainer decision (issue #410): the covering above D is non-structural by default; structural only where the user sets D over it | `weights::covering_items` | Moves cargo: the light KG rises, and some designs fail the IS Code angle of max GZ |
| WC-A03 | Wing tank top | DB + h_support + 0.66 × OD | Estimate | Profile `wing_tank_height_ratio`, carried over from the original engine | `weights.rs:155-156`; `barge_capacity.rs:70` | Unknown |
| WC-A04 | Gas piping and machinery | 0.04 × (panel hull steel + tank total) | Fixed in code | Original engine; no source | `models.rs:217`; `weights.rs:170` | Unknown |
| WC-A05 | Gas outfitting | 0.035 × (panel hull steel + tank total) | Fixed in code | Original engine; no source | `models.rs:216`; `weights.rs:173` | Unknown |
| WC-A06 | Gas lightship sum | panel hull steel + tank total + insulation + piping + outfitting | Method | Original engine | `weights.rs:176` | Unknown |
| WC-A07 | Tank steel, internals, insulation | From the tank calculation | See page 04 | Page 04 | `weights.rs:75-79` | See page 04 |
| WC-A08 | Hull steel density | 7.85 t/m³ | Fixed in code | Carbon steel | `models.rs:211`; `weights.rs:49` | Small effect; bottom-up only |
| WC-A09 | Gas plate thicknesses | per member from the ABS Steel Barges formulas (Rule Set `abs-steel-barges` 1.0.0, issue #740) (the 12, 10 and 10 mm defaults are the legacy API only) | Method | Issue #722 | `hull_panels.rs` | Lightship and cost |
| WC-A10 | Gas structure factor | retired (issue #722): the members are weighed one by one; the legacy API keeps 1.4 | Method | Issue #722 | `hull_panels.rs` | None |
| WC-A11 | Rake plating thickness | mean of bottom and side thickness | Fixed in code | Original engine | `weights.rs:62-63` | None in the gas branch: the hull has no rakes (`barge_capacity.rs:570`) |
| WC-A12 | Wing plate | deck thickness × (B − n_wide × chord) × tank zone + B × end zones, plus side extension above D | Method | Original engine | `weights.rs:81-122` | Bottom-up and cost only |
| WC-A13 | Weather shield | 6 mm; top 1.5 m above tank top; top, sides and ends of the tank zone | Fixed in code | Original engine | `models.rs:185-186`; `weights.rs:124-149` | Bottom-up and cost only |
| WC-A14 | Steel delta | top-down − bottom-up, reported | Method | Diagnostic | `weights.rs:167` | None on results |
| WC-A15 | Stores in lightship | 0 t; `stores_weight` (200 t) is not read | Fixed in code | Comment: consumables are deadweight | `weights.rs:193-194`; `models.rs:218` | Non-conservative on cargo (see WC-A35) |
| WC-A16 | Hopper hull steel rate | 0.10 t/m³ of LOA × B × D | Fixed in code | Original engine; API field only | `barge_capacity.rs:252`, `649` | Unknown; no hopper case |
| WC-A17 | Hopper plate thicknesses | inner bottom 14, hopper 14, bulkheads 11, coaming 10 mm | Estimate | Profile, "carried over from the original engine" | `barge_capacity.rs:246-249`, `635-638`; `builtin_profile.json:376` | Unknown |
| WC-A18 | Hopper hold steel areas | inner bottom, slopes, wing bulkheads, (n + 1) transverse bulkheads, coamings | Method | Original engine | `barge_capacity.rs:639-646` | Unknown |
| WC-A19 | Hopper structure factor | 1.40 | Fixed in code | Original engine; API field only | `barge_capacity.rs:251`, `647` | Unknown |
| WC-A20 | Hopper steel density | 7.85 t/m³ | Fixed in code | Carbon steel | `barge_capacity.rs:250` | Small effect |
| WC-A21 | Hopper piping and outfitting | (1 + 0.04 + 0.035) × steel | Fixed in code | Original engine; API fields only | `barge_capacity.rs:253-254`, `651` | Unknown |
| WC-A22 | Number of holds | 3; sets n + 1 transverse bulkheads | Estimate | Profile, original engine | `barge_capacity.rs:242`, `643` | Small effect |
| WC-A23 | Hold geometry in the hopper steel | wing void, double bottom, slope, coaming | See page 06 | Page 06 | `barge_capacity.rs:615-631` | See page 06 |
| WC-A24 | Box-hold lightship | k × LOA × B × D | Method | Fit to Reference Vessels (`calibration/results.md`) | `barge_capacity.rs:652-660` | See WC-A25 |
| WC-A25 | k, box hold | 0.1086 t/m³ | Estimate | Fitted: 2 published lightships, in-sample, range 0.095–0.1223 (`calibration/factors.yaml`) | `builtin_profile.json:80-81`; `simulate.rs:716-719` | Unknown; out of sample, deck barges −6 %, pontoons −27 % |
| WC-A26 | k, container hold | 0.1711 t/m³ | Estimate | One case, in-sample (`calibration/factors.yaml`) | Calibration only; no app path | Unknown |
| WC-A27 | Deck barge and pontoon | No factor; run as box | Method | `calibration/README.md` | Calibration only | Pontoon lightship low (non-conservative on cargo) |
| WC-A28 | Sloped-hopper lightship | Build-up of WC-A16 to WC-A21 | Method | Original engine; no hopper case. On box barges it gave 31–111 % high (`calibration/README.md`, pass 1) | `barge_capacity.rs:634-651` | Unknown; likely high (conservative on cargo) |
| WC-A29 | Stern block mass | 0 t in lightship | Fixed in code | Not modelled | `barge_capacity.rs:571-572`, `651-660`; `gas_stability.rs:798-803` | Non-conservative on cargo and draught |
| WC-A30 | Genset engine mass | 9.8 to 21.0 t per set, from a maker table; reported only | Fixed in code | Maker product presentation, cited at `aft_arrangement.rs:132-135` | `aft_arrangement.rs:136-221`, `1025` | Not in lightship (see WC-A29) |
| WC-A31 | Pushed barge | crew, battery, propulsion on the tug, not in lightship | Method | Scope of the Formation | `barge_capacity.rs:810-812` | None for the barge; the tug is not weighed |
| WC-A32 | Displacement | Cb × LOA × B × T × ρ on a box hull | Method | Box hull, see page 01 | `barge_capacity.rs:905`; `simulate.rs:740-751` | See WC-A33 |
| WC-A33 | Block coefficient | 0.85 for every Hull | Estimate | Profile; basis cites calibrated box barges 0.883–0.995 | `builtin_profile.json:52`; `simulate.rs:667` | Box barge: displacement about 8 % low (conservative on cargo) |
| WC-A34 | Water density | 1.0 t/m³ | Estimate | Profile, fresh water as the original engine | `builtin_profile.json:633` | Small; sea water is about 1.025 |
| WC-A35 | Deadweight items | cargo only; no fuel, water, stores, crew or ballast | Method | Not modelled | `simulate.rs:797`, `1081`; `barge_capacity.rs:1032` | Non-conservative on cargo |
| WC-A36 | Ballast capacity | 0 t; ballast draught only | Fixed in code | API field only | `barge_capacity.rs:212`, `914-926` | None on deadweight |
| WC-A37 | Box-barge Cb (fitted) | 0.928 on LOA | Estimate | Fitted: 7 cases, 6 with a k-derived lightship (`calibration.py:132`) | Calibration only (`calibration.py:161`) | Unknown |
| WC-A38 | Gas hull steel KG | 0.5 × D | Fixed in code | First-order, original engine. Since issue #410 the steel is weighed to D, so the centre matches | `gas_stability.rs` | Unknown |
| WC-A39 | Gas tank and insulation KG | DB + h_support + OD / 2 | Method | Tank centre | `gas_stability.rs:708-716` | Small |
| WC-A40 | Gas piping and machinery KG | D | Fixed in code | First-order | `gas_stability.rs:717` | Unknown |
| WC-A41 | Gas outfitting KG | 0.75 × D | Fixed in code | First-order | `gas_stability.rs:718` | Unknown |
| WC-A42 | Gas cargo KG | inner bottom of tank + segment centroid at the fill fraction | Method | Geometry of a circular segment | `gas_stability.rs:710`, `733-734` | Small |
| WC-A43 | Gas weight split check | limitation if Σ items differs from lightship by more than 1 t | Method | Consistency check | `gas_stability.rs:805-811` | None |
| WC-A44 | Hopper lightship KG | 0.5 × D | Fixed in code | First-order (`BULK_GM_WARNING`) | `barge_capacity.rs:704` | Non-conservative against IS Code Part B 8.6 (main deck) |
| WC-A45 | Hopper cargo KG | DB_bulk + centroid of the full trapezoid hold | Method | Geometry; fill ratio ignored | `barge_capacity.rs:697-703` | Conservative (cargo shown high) |
| WC-A46 | Legacy KG items | bottom-up steel D/2; tank shells (no internals) and insulation at tank centre; piping D; outfitting 0.75 D; stores DB/2 or 0.25 D | Method | Original engine; parity interface only | `hydrostatics.rs:80-124` | Items do not sum to the lightship |
| WC-A47 | Legacy LCG items | hull, piping LOA/2; tanks, cargo at tank-zone centre; outfitting, stores at half the stern zone | Method | Original engine; parity interface only | `hydrostatics.rs:177-217` | Unknown |
| WC-A48 | LCG and trim in results | none; even keel | Fixed in code | Not modelled | `barge_capacity.rs:903-905`; `simulate.rs:751` | Unknown |
| WC-A49 | TCG | none; symmetric | Fixed in code | Not modelled | All centres | Small for symmetric Hulls |

## Inputs the user can change

From the Defaults Profile (`crates/ovd-format/src/builtin_profile.json`). Names are the glossary names.

| Input | Default | Unit | Where the default comes from |
|---|---|---|---|
| `block_coefficient` | 0.85 | – | Profile; basis cites calibrated box barges 0.883–0.995 |
| `lightship_factor_t_per_m3` | 0.1086 (box hold) | t/m³ | `calibration/factors.yaml`, box fit |
| `water_density_t_per_m3` | 1.0 | t/m³ | Fresh water, original engine |
| `wing_tank_height_ratio` | 0.66 | – | Original engine; sets the wing plate level and so the covering items |
| `tank_support_height_m` | 0.5 | m | Original engine; sets the wing tank top and the tank KG |
| `n_holds` | 3 | holds | Original engine; hopper steel |
| `inner_bottom_plate_mm`, `hopper_plate_mm`, `hold_bulkhead_plate_mm`, `hatch_coaming_plate_mm` | 14, 14, 11, 10 | mm | Original engine; hopper steel |
| Tank design inputs | see page 04 | | Page 04 |
| Formation (self-propelled or pushed) | – | – | Sets the stern void, not the lightship |

The gas factors (WC-A01, A04, A05, A08 to A13) and the hopper factors (WC-A16, A19 to A21) cannot be changed in
the app. Some are fields of `BargeCapacityInputs` or `BargeInputs` and can be set through the API.

## Limits and known findings

- The stern block of a self-propelled Hull has no mass in the lightship (WC-A29). The dry result gives no warning.
- Deadweight is the cargo only (WC-A35). No loading condition has fuel, stores or crew.
- Since issue #722 the gas lightship and the cost take the same hull steel (the panel model).
  In the parity example the two differ by 297 t.
- The gas lightship factors (WC-A01, A04, A05) have no calibration case. Gas and tank barge cases are in
  `not_yet_runnable_cases` (`calibration/cases.yaml:1321`).
- The sloped-hopper build-up has no calibration case (WC-A28).
- The result labels the lightship (issue #434). A sloped-hopper hull and a gas barge show
  `calibration.lightship = "uncalibrated"`; a box-hold barge shows `"calibrated"`. The label is an output kept
  out of the results hash (`UNHASHED_KEYS`).
- The app default Cb (0.85) is not the fitted box-barge Cb (0.928) (WC-A33, WC-A37).
- The main result has no LCG, trim or TCG (WC-A48, WC-A49). The legacy LCG is reachable only through the parity
  interface.
- No weight margin or growth allowance is added.
- Findings go to the register (#121) after triage. See the findings register (#121) and the engineering basis triage tickets.

## Evidence

- **Parity tests:** `crates/ovd-core/tests/weights.rs` checks every `WeightBreakdown` field against the original
  Python, for the 90 × 14.8 × 6.5 m case and for rakes, no shield, no wing tank and a full-height wing tank. This is
  parity, not validation.
- **Calibration** (`calibration/results.md`, bands in `calibration/bands.yaml`: lightship ±15/20/30 %, deadweight
  ±7.5/10/15 % at high/medium/low confidence):
  - Box-barge lightship, in-sample: 3 of 3 inside band, mean error −3.9 %.
  - Box-barge deadweight at the stated draught, in-sample: 8 of 8 inside, mean −0.4 %; out of sample: 30 of 30
    inside, mean −2.1 %. These runs use Cb 0.928, not the app default.
  - Deck barges run as box, out of sample: lightship 14 of 15 inside, mean −6.3 %; deadweight 8 of 13 inside,
    mean +7.6 %.
  - Pontoons run as box, out of sample: lightship 12 of 22 inside, mean −27.0 %; deadweight 3 of 7 inside, mean
    +25.8 %.
  - Container hold: one case, in-sample.
  - No case for the gas barge or the sloped-hopper barge.
- **Gas stability tests:** `crates/ovd-core/tests/gas_stability.rs` (parity with the original engine).

## Questions for reviewers

1. Is 0.10 t/m³ of LOA × B × D a fair hull steel rate for a Type C gas barge with a wing plate and a weather
   shield? Which built gas barges publish a lightship we can use as a case?
2. Should the lightship use the bottom-up steel (with the plating build-up) or the top-down rate? They differ by
   about a third in the parity example.
3. Are 4 % piping and machinery and 3.5 % outfitting, both on hull steel plus tank steel, fair for a gas barge?
   Should the tank steel be in the base?
4. What mass and centre should the stern block carry: thrusters, battery units, genset units, accommodation units
   and the lifting wheelhouse? Is a mass per 40 ft unit a fair start?
5. Should the default Cb be 0.928 for a box-hold barge and stay 0.85 only for a gas barge?
6. For a deck barge or pontoon, should lightship KG be at the main deck, as IS Code 2008 Part B 8.6 allows as a
   conservative value? Should OVD use the main deck for every barge at concept stage?
7. Is a 6 mm plate a fair weight for a non-structural weather shield, and should its cost rate differ from the
   hull steel rate (issue #410 left this open)?
8. Which consumables (fuel, fresh water, stores, crew) should the laden and light conditions carry for a
   self-propelled inland vessel?
9. Is a weight margin on the lightship normal at concept stage for these Hulls? What size?
10. Does the sloped-hopper build-up (14 mm inner bottom and slopes, factor 1.4) match built hopper barges?
