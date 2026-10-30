# Cost, BOQ and capital budget

> Concept and preliminary design only. Not for approval. Draft: needs expert review.
>
> Code state: written against `main` at commit `f6d446c` (2026-10-20).

**For:** Cost engineer  ·  **Code:** `crates/ovd-core/src/barge_cost.rs`, `crates/ovd-format/src/cost.rs`, `crates/ovd-core/src/weights.rs`, `app/src/lib/views/CapitalBudget.svelte`  ·  **Rule Sets:** none

OVD is a concept and preliminary design tool (ADR 0011). The capital budget on this page is a bottom-up
order-of-magnitude figure for one unit. Its rates are examples, not market prices. Do not use it for a budget
approval, a bid or a contract.

## What OVD computes here

For a gas Option only (a Hull with Type C tanks), each run gives a `cost` results key. The app shows it in
**BOQ & Finance → Capital budget**:

- **Barge** lines: Hull steel, Type C tanks, Tank supports, Insulation, Cargo system, Piping and machinery,
  Outfitting, Paint, anodes and miscellaneous. Then the **Barge subtotal**.
- **Tug** lines: Hull steel, Battery, Propulsion, Wheelhouse and accommodation, Outfitting. Then the
  **Tug subtotal**.
- **Unit** table: Barge subtotal, Tug subtotal, **Yard margin (n %)**, **Delivery**, **Unit total**.
- The currency, a basis line ("Example rates — replace with yours."), and a list of which masses come from
  this design and which are core defaults.
- Notes from three checks (see CB-A44 to CB-A46).

What OVD does **not** compute today:

- **No bill of quantities.** The "BOQ" view is a placeholder: "The bill of quantities comes in a later phase"
  (`app/src/lib/MainPanel.svelte`, lines 160-163).
- **No budget for other Cargos.** For a dry bulk Option, `cost` is null and the app shows "Work in
  development" (`CapitalBudget.svelte`, lines 58-62; `crates/ovd-format/src/simulate.rs`, line 809).
- **No contingency, escalation, owner's costs, financing or running cost.**

## How it works today

1. **Masses (quantities).** The gas capacity calculation runs `calculate_weights`
   (`crates/ovd-core/src/barge_capacity.rs`, line 571) on a plain box hull (no rakes, line 570).
   `cost::gas_masses` (`crates/ovd-format/src/cost.rs`, lines 74-99) copies eight masses from the
   `WeightBreakdown` into the cost inputs. The three tug quantities keep the core's defaults, because the
   engine does not size the tug. `simulate` writes the result as `cost_masses`
   (`crates/ovd-format/src/simulate.rs`, line 1095). Label: **Method** (masses), **Fixed in code** (tug).
2. **How each mass is derived** (`crates/ovd-core/src/weights.rs`, `calculate_weights`). Label: **Method**,
   with the factors below as **Fixed in code** or **Estimate**:
   - Hull steel (bottom-up) = plating × structure factor (2.0 on a gas barge, an Estimate; see page 02). Plating = Σ (area × thickness × 7.85 t/m³) for
     bottom, side, deck and rakes (lines 51-72).
   - Wing plate = deck-thickness plate over the beam less the tank chord cut-outs in the tank zone, full beam
     outside it, plus any side extension (lines 81-122).
   - Weather shield = (top + two sides + two ends) area × 6 mm × 7.85 t/m³ (lines 124-149).
   - Tank shells = shell volume × tank material density (`crates/ovd-core/src/tanks.rs`, lines 213-222).
   - Tank supports = tank shells × (tank structure factor − 1) (`weights.rs`, line 77). This mass also holds
     the tank stiffeners and ring frames (comment, line 76). The budget prices it in two parts (issue #402):
     the share `tank_internals_fraction` (default 0.5) at the tank shell rate, as stiffeners and ring frames in
     tank steel, and the rest at the supports rate, as saddles. **The share is an assumption, an Estimate with
     no source read:** the rule-source library has no component breakdown of a marine Type C tank. The results
     say so (`cost.tank_internals`), and so does the app. The lightship does not change.
   - Insulation = outer surface area × insulation thickness × 60 kg/m³ (`tanks.rs`, lines 225-234).
   - Piping and machinery = 0.04 × (top-down steel + tank total) (`weights.rs`, line 170).
   - Outfitting = 0.035 × (top-down steel + tank total) (`weights.rs`, line 173).
   - Top-down steel = 0.10 t/m³ × L × B × moulded depth (`weights.rs`, lines 162-163). The lightship uses
     this top-down steel, not the bottom-up steel (line 176). The cost uses the bottom-up steel.
3. **Rates.** `cost::rates_of` (`cost.rs`, lines 37-43) starts from the compiled-in example rates file
   (`examples/barge_cost_rates.example.yaml`) and sets the 14 numeric rates from the Option's "Cost rates"
   inputs. The currency comes from the example file and is not an input. Label: **Estimate** (each rate).
4. **Line amounts** (`crates/ovd-core/src/barge_cost.rs`, `compute`, lines 346-365). Each line is
   mass × rate, or a lump sum. Label: **Method**.
5. **Subtotals, margin, total** (`barge_cost.rs`, lines 368-371):
   - Barge subtotal = Σ barge lines; Tug subtotal = Σ tug lines (compensated sum, as Python does).
   - Yard margin = yard margin fraction × (Barge subtotal + Tug subtotal).
   - Unit total = Barge subtotal + Tug subtotal + Yard margin + Delivery.
   Delivery is outside the margin base. Label: **Method**.
6. **Rounding.** Each line, subtotal, the margin and the total round to whole currency units, half to even
   (`barge_cost.rs`, lines 401-412). The total is rounded from the unrounded sum, so the shown lines can differ
   from the total by up to one unit per line. Delivery is passed through unrounded. Label: **Method**.
7. **Output.** `cost::cost_of` (`cost.rs`, lines 151-160) puts the result in `cost`. `cost` and
   `cost_masses` are out of the results hash (`crates/ovd-format/src/run.rs`, line 244), and the rates never
   reach `simulate` (lines 269-274). A rate change moves only the budget.

## Assumptions, rules and estimates

Id prefix `CB` (cost and budget).

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| CB-A01 | Currency | USD | Fixed in code | Example rates file; not an input | `examples/barge_cost_rates.example.yaml` line 13; `cost.rs` lines 13-14, 38 | Unknown (wrong unit label if the user enters rates in another currency) |
| CB-A02 | Price base date | None stated | Fixed in code | No date in the rates file or the profile | — | Unknown |
| CB-A03 | Hull steel rate | 2,500 per t fabricated | Estimate | "Not a market quote" (profile basis) | `builtin_profile.json` lines 647-652; `barge_cost.rs` lines 349, 360 | Unknown |
| CB-A04 | Tank shell rate | 10,000 per t | Estimate | Example; same for every tank material | profile lines 653-658; `barge_cost.rs` line 351 | Unknown; non-conservative for 9 % Ni if it is low, conservative for carbon-manganese steel if high |
| CB-A05 | Tank supports rate | 2,000 per t | Estimate | Example: "saddles, anti-flotation and internals" | profile lines 659-664; `barge_cost.rs` line 352 | Smaller effect since #402: the stiffeners and ring frames part is priced at the tank shell rate |
| CB-A06 | Insulation rate | 4,000 per t installed | Estimate | Example, "per tonne equivalent" | profile lines 665-670; `barge_cost.rs` line 353 | Unknown; at 60 kg/m³ this is 240 per m³ of insulation |
| CB-A07 | Piping and machinery rate | 4,500 per t | Estimate | Example | profile lines 671-676; `barge_cost.rs` line 355 | Unknown |
| CB-A08 | Outfitting rate (barge) | 4,000 per t | Estimate | Example | profile lines 677-682; `barge_cost.rs` line 356 | Unknown |
| CB-A09 | Cargo system | 500,000 per unit (lump) | Estimate | Example: pumps, manifold, ESD, relief, gauging | profile lines 683-688; `barge_cost.rs` line 354 | Unknown; does not scale with size or Cargo |
| CB-A10 | Paint, anodes and miscellaneous | 100,000 per unit (lump) | Estimate | Example | profile lines 689-694; `barge_cost.rs` line 357 | Unknown; does not scale with surface area |
| CB-A11 | Battery rate | 250 per kWh installed | Estimate | Example | profile lines 695-700; `barge_cost.rs` line 361 | Unknown |
| CB-A12 | Propulsion rate | 1,000 per kW installed | Estimate | Example: thrusters, drives, switchboard | profile lines 701-706; `barge_cost.rs` line 362 | Unknown |
| CB-A13 | Wheelhouse and accommodation | 450,000 per unit (lump) | Estimate | Example | profile lines 707-712; `barge_cost.rs` line 363 | Unknown |
| CB-A14 | Tug outfitting | 200,000 per unit (lump) | Estimate | Example | profile lines 713-718; `barge_cost.rs` line 364 | Unknown |
| CB-A15 | Yard margin fraction | 0.10 of barge + tug subtotal | Estimate | Example: "engineering, class, trials and margin" | profile lines 719-724; `barge_cost.rs` line 370 | Unknown; it mixes overhead, profit and contingency |
| CB-A16 | Delivery | 200,000 per unit (lump) | Estimate | Example: delivery voyage or heavy lift | profile lines 725-730; `barge_cost.rs` line 371 | Unknown |
| CB-A17 | Hard range of each money rate | 0 to 1 × 10⁹ | Fixed in code | Profile hard range | profile lines 647-731 | None (input check only) |
| CB-A18 | Hard range of yard margin fraction | 0 to 1 | Fixed in code | Profile hard range | profile line 722 | None |
| CB-A19 | Tug hull steel mass | 115.0 t | Fixed in code | Python dataclass default; engine does not size the tug | `barge_cost.rs` line 50; `cost.rs` line 84 | Unknown; does not follow the barge size |
| CB-A20 | Tug battery capacity | 620 kWh | Fixed in code | As CB-A19 | `barge_cost.rs` line 51 | Unknown |
| CB-A21 | Tug propulsion power | 230 kW | Fixed in code | As CB-A19; not linked to the powering result | `barge_cost.rs` line 52 | Unknown |
| CB-A22 | Core default barge masses (API only) | Hull 253.2, wing plate 40.3, weather shield 50.1, tank shell 215.6, supports 64.7, insulation 25.9, piping 32.9, outfitting 28.8 t | Fixed in code | Python dataclass defaults; used only when the core is called with no masses. A gas Option replaces them | `barge_cost.rs` lines 42-49 | None for app results |
| CB-A23 | Hull steel line | (hull steel + wing plate + weather shield) × hull steel rate | Method | Bottom-up steel | `barge_cost.rs` lines 347-350 | Unknown (see CB-A33) |
| CB-A24 | Type C tanks line | tank shell mass × tank shell rate | Method | — | `barge_cost.rs` line 351 | — |
| CB-A25 | Tank supports line | supports mass × supports rate | Method | — | `barge_cost.rs` line 352 | — |
| CB-A26 | Insulation line | insulation mass × insulation rate | Method | Priced per tonne, not per m² | `barge_cost.rs` line 353 | Unknown; mass is a weak driver for low-density insulation |
| CB-A27 | Piping and outfitting lines | mass × rate | Method | Masses are fractions of steel (CB-A37, CB-A38) | `barge_cost.rs` lines 355-356 | Unknown |
| CB-A28 | Tug lines | tug hull steel × hull steel rate; kWh × battery rate; kW × propulsion rate; two lumps | Method | — | `barge_cost.rs` lines 359-365 | Unknown |
| CB-A29 | Yard margin base | Barge + tug subtotal; delivery excluded | Method | — | `barge_cost.rs` line 370 | — |
| CB-A30 | Unit total | Subtotals + margin + delivery | Method | — | `barge_cost.rs` line 371 | — |
| CB-A31 | Rounding | Whole currency units, half to even, each figure rounded alone | Method | Python `round()` parity | `barge_cost.rs` lines 401-412 | None (≤ 1 unit per line) |
| CB-A32 | Contingency, escalation, owner's costs | None | Fixed in code | Not modelled | — | Non-conservative |
| CB-A33 | Hull steel mass basis | Bottom-up plating × structure factor; the lightship uses top-down steel instead | Method | `weights.rs` lines 166 and 176 | `cost.rs` line 76 | Unknown; the two steel masses differ by `steel_delta` |
| CB-A34 | Hull structure factor | 2.0 × plating on a gas barge (issue #401, step 2) | Estimate | `GAS_HULL_STRUCTURE_FACTOR`, set to about 0.10 t/m³ of L × B × D, the top-down rate; the reference vessel library (designer estimates) gives 0.145; `BargeInputs::new` keeps 1.4 for the legacy API; not an input on the gas path | `models.rs` line 210; `weights.rs` line 69 | Unknown |
| CB-A35 | Plate thicknesses | Bottom 12 mm, side 10 mm, deck 10 mm; rakes the mean of bottom and side | Fixed in code | `BargeInputs::new` default | `models.rs` lines 207-209; `weights.rs` lines 52-63 | Unknown |
| CB-A36 | Hull steel density | 7.85 t/m³ | Fixed in code | Carbon steel | `models.rs` line 211 | None of note |
| CB-A37 | Piping and machinery mass | 0.04 × (top-down steel + tank total) | Fixed in code | `BargeInputs::new` default | `models.rs` line 217; `weights.rs` line 170 | Unknown |
| CB-A38 | Outfitting mass | 0.035 × (top-down steel + tank total) | Fixed in code | `BargeInputs::new` default | `models.rs` line 216; `weights.rs` line 173 | Unknown |
| CB-A39 | Top-down steel rate | 0.10 t/m³ of L × B × moulded depth | Fixed in code | `BargeInputs::new` default | `models.rs` line 212; `weights.rs` line 163 | Unknown; drives CB-A37 and CB-A38 |
| CB-A40 | Weather shield | 6 mm plate; top 1.5 m above the tank top | Fixed in code | `BargeInputs::new` default | `models.rs` lines 185-186; `weights.rs` lines 124-149 | Unknown |
| CB-A41 | Tank supports mass | tank shells × (tank structure factor − 1); default factor 1.3, 1.12 for fully pressurised LPG | Estimate | Profile table (one published tank for 1.12; 1.3 carried over) | `builtin_profile.json` lines 303-320; `weights.rs` line 77 | Unknown |
| CB-A41a | Tank internals share | `tank_internals_fraction` 0.5 of the structure surplus priced at the tank shell rate, the rest at the supports rate | Estimate | An assumption: no source read gives the split (issue #402) | `cost.rs` `cost_of`; `builtin_profile.json` | Unknown; 0 gives the earlier pricing |
| CB-A42 | Insulation density | 60 kg/m³ | Fixed in code | `BargeInputs::new` default | `models.rs` line 203; `tanks.rs` line 233 | Unknown |
| CB-A43 | Hull shape for the masses | Box, no rakes | Fixed in code | `HullGeometry::new(..., 0, 0, 0, 0)` | `barge_capacity.rs` line 570 | Unknown |
| CB-A44 | Check `masses_positive` | Error if min(tank shell, hull steel) ≤ 0 | Method | Other masses are not checked | `barge_cost.rs` lines 374-384 | — |
| CB-A45 | Note `no_reliquefaction_in_scope` | Always passes; says the cargo system has no reliquefaction or nitrogen plant | Fixed in code | Pressure-accumulation operating concept | `barge_cost.rs` lines 385-391 | Non-conservative if the Option needs that plant |
| CB-A46 | Note `fabrication_rates_are_estimates` | Always fails as a warning, so it always shows | Fixed in code | — | `barge_cost.rs` lines 392-398 | — |
| CB-A47 | Basis line | "Example rates — replace with yours." shown always, also after the user edits rates | Fixed in code | `COST_RATES_BASIS` | `crates/ovd-format/src/vdesign.rs` line 241; `CapitalBudget.svelte` line 66 | — |
| CB-A48 | Scope of the budget | Gas Options only; one barge plus one electric pusher tug as one unit | Fixed in code | Core models a gas ATB unit | `cost.rs` lines 1-6; `simulate.rs` lines 809, 1095 | Unknown for other Formations |

## Inputs the user can change

All 14 inputs are in the "Cost rates" input group (`crates/ovd-format/src/vdesign.rs`, lines 1426-1516). Each
default equals the example rates file (a test keeps them equal: `crates/ovd-format/tests/cost_rates.rs`).
A file saved before these inputs existed opens with the example values, marked `default`, with a warning
(`CHANGELOG.md`).

| Input | Label in the app | Default | Unit |
|---|---|---|---|
| `hull_steel_rate` | Hull steel rate | 2,500 | per t |
| `tank_shell_rate` | Tank shell rate | 10,000 | per t |
| `tank_supports_rate` | Tank supports rate | 2,000 | per t |
| `insulation_rate` | Insulation rate | 4,000 | per t |
| `piping_rate` | Piping and machinery rate | 4,500 | per t |
| `outfitting_rate` | Outfitting rate | 4,000 | per t |
| `cargo_system_lump` | Cargo system | 500,000 | per unit |
| `paint_anodes_misc` | Paint, anodes and miscellaneous | 100,000 | per unit |
| `battery_rate_per_kwh` | Battery rate | 250 | per kWh |
| `propulsion_rate_per_kw` | Propulsion rate | 1,000 | per kW |
| `wheelhouse_accom_lump` | Wheelhouse and accommodation | 450,000 | per unit |
| `tug_outfitting_lump` | Tug outfitting | 200,000 | per unit |
| `yard_margin_fraction` | Yard margin | 0.10 | fraction |
| `delivery_lump` | Delivery | 200,000 | per unit |

The masses are not cost inputs. They follow the design inputs of other areas (see pages 02 and 04). Of the
mass factors, only the tank structure factor is an input. The currency and the tug quantities are not inputs.

## Limits and known findings

- The figure is a unit cost (barge plus tug), not a project capital budget. It has no contingency, escalation,
  owner's costs, financing or spares.
- The hull steel line uses the bottom-up steel mass. The lightship and the piping and outfitting masses use the
  top-down steel mass. One Option so carries two steel masses.
- The tug is not sized. Its three quantities are constants, whatever the barge or the powering result.
- Lump sums do not scale with size, tank count or Cargo.
- The tank shell rate is the same for every tank material.
- No price base date and no currency input.
- The view shows amounts only. It does not show quantity, unit and rate per line, so a reviewer cannot audit a
  line in the app. The quantities are in `cost.masses` in the results file.
- No findings in the register (#121) for this area yet. Proposed findings are in the engineering basis triage tickets.

## Evidence

- `crates/ovd-core/tests/barge_cost.rs`, `the_example_rates_price_the_default_barge`: the core default masses
  and example rates give a unit total of 6,177,125. A hand check gives the same: barge 4,111,250, tug
  1,322,500, margin 543,375, delivery 200,000. This is an arithmetic check, not a price check.
- `crates/ovd-core/tests/barge_cost.rs`: rates-file parsing, bad values, compensated sum, zero-mass check.
- `crates/ovd-format/tests/cost_rates.rs`: property tests. Each default equals the example file; a budget is
  the sum of its parts; a rate edit moves only the budget; the budget is out of the results hash; a dry bulk
  Option has no budget; gas masses come from the design.
- `app/e2e/smoke.test.mjs`, lines 509-545: the view shows the basis, the three tables, the mass sources, a rate
  edit moves the total, and dry bulk shows "Work in development".
- **No calibration against real prices.** No Reference Vessel has a published build cost in `calibration/`.

## Questions for reviewers

1. Which accuracy class should the page state for this figure? Is AACE Class 5 (concept screening) right?
2. Should the hull steel line use the top-down steel (the lightship basis) or the bottom-up steel?
3. Is one tank shell rate per tonne fair across 9 % Ni, 304L, 316L and S690QL1 steels, or should the rate key on
   the tank material?
4. The budget now prices an assumed half (`tank_internals_fraction` 0.5) of the tank structure surplus at the tank
   shell rate (stiffeners and ring frames) and the rest at the supports rate (saddles). What share of a marine
   Type C tank's structure surplus is stiffeners and ring frames, and where is it published?
5. Should insulation be priced per m² of insulated surface at a given thickness, not per tonne?
6. Is a yard margin of 10 % on barge plus tug, with delivery outside it, a fair split? Should engineering,
   class, trials, profit and contingency be separate lines?
7. Which lump sums should scale (cargo system with tank count or volume; paint with wetted and painted area)?
8. Which price index and base date should the example rates carry, so a user can escalate them?
