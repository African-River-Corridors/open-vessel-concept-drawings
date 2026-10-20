# 04 — Crude oil in OVCD: codebase fit

> Read-only review of `open-vessel-concept-drawings` at `main` = `fbfe2e8` (2026-10-20), plus the draft expert pages on
> `origin/docs/engineering-basis` (pages 02, 03, 05, 06, 09 and their `proposals/`). No code was changed, nothing was
> built. Line numbers are from that commit. Codebase-design terms (module, interface, depth, seam, adapter, leverage,
> locality) are used as the `codebase-design` skill defines them.

## 1. Summary

- OVCD has two Cargo "phases" today: `liquid_gas` (five gases, Type C tanks) and `dry_bulk` (bauxite, box or sloped
  hopper hold). The phase is a **string** on `CargoProperties`. Code branches on it in at least 14 places, in four
  different styles (`== "dry_bulk"`, `!= "dry_bulk"`, `== "liquid_gas"`, `separation.is_some()`).
- **Two of those branches route any new phase to the gas path, and one routes it to the dry path.** A crude Cargo with
  a new phase, added only to the database, would be sized as Type C tanks in `barge_capacity::compute` (`is_gas =
  phase != "dry_bulk"`, line 829), and drawn as a dry hold in `ga.rs` (`read_hull`, line 298). This is the main code
  risk. The first ticket must make the phase an enum and every branch an exhaustive `match`, with no result change.
- **Much is reusable.** The dry path already does what a tank barge needs at concept level: cargo zone from end spaces,
  a volume-bound cargo, a weight-bound cargo from the draught limit, `cargo = min(both)`, and a box hydrostatic model.
  The gas stability path already has a generic box GZ curve with a free surface correction. The Defaults Profile
  already has an empty `liquid` depth group (ratio 0.30). The calibration file already lists 41 tank barges that
  cannot run "because there is no tank hull model".
- **What is new:** a containment module for integral (and later independent) cargo tanks bounded by the inner hull,
  with a centreline bulkhead and transverse bulkheads; rectangular-tank free surface; standard loading conditions
  with ballast; a tank-barge lightship method; GA views of the tanks; cost lines; and two Rule Sets (`marpol-i`,
  `adn-n`) with about eight new Measures.
- **Spec 0004 carries most of the rules work as data**, but four things are missing for liquid cargo rules: numeric
  applicability (deadweight bands), per-object applicability (a cargo tank versus a ballast tank), a clamp formula
  shape, and a Cargo group that Applicability can name. See section 6.
- **Touch points: 58** (section 3), in 7 crates or folders.

## 2. How the code models Cargo today

### 2.1 The Cargo

| What | Where | Notes |
|---|---|---|
| `CargoProperties { name, density, temperature, pressure, description, phase }` | `crates/ovcd-core/src/cargo.rs` 10–24 | `phase` is `&'static str`: `"liquid_gas"` or `"dry_bulk"`. Temperature and pressure feed no calculation. |
| `CARGO_DATABASE: [(&str, CargoProperties); 6]` | `cargo.rs` 27–94 | Fixed-size static array. Order is the old Python dict order. |
| `get_cargo(key)` | `cargo.rs` 97–109 | The error text lists every valid key, so a new key changes a pinned message. |
| Python copy of the data | `src/vessel_designer/core/cargo.py` | The Python package has its **own** `CARGO_DATABASE` (0.1.0 API); `tests/data/public_api.json` snapshots it. |
| `cargo_density_override` | `crates/ovcd-core/src/models.rs` 110, 205 | Dead field: nothing reads it (basis page 06, G9). |
| Density by carriage condition | open ticket #315 | Will move density from one number per Cargo to a value by (Cargo, condition) with a cited source. Crude needs the same shape (crude density varies about 790 to 990 kg/m³ by grade; a typed density is needed). |

**How a Cargo is added today:** one entry in `cargo.rs`, the same in `cargo.py`, then the profile rows that
`check_profile_defaults` and `table_combinations` demand, then tests that pin the list. A new **phase** has no single
place: every branch in section 3 must be found by hand.

### 2.2 The Cargo Containment

| Containment | Phase | Engine vessel id (`run.rs` 172–177) | Core path |
|---|---|---|---|
| `box_hold` | `dry_bulk` | `box_barge` | `barge_capacity::bulk_volumetric`, `hold_type = "box"` |
| `sloped_hopper_hold` | `dry_bulk` | `hopper_barge` | `bulk_volumetric`, `hold_type = "sloped_hopper"` |
| `type_c_tanks` | `liquid_gas` | `gas_barge` | `gas_volumetric` → `tanks.rs` → `weights.rs`; then `gas_arrangement::fit` and `gas_stability::assess` |

- The pairing lives in `vdesign.rs` `CARGO_CONTAINMENTS` (46–50). `cross_checks` (2697–2720) refuses a bad pair.
- The core has a `"container"` hold type (`barge_capacity.rs` 178, 621) that no Design can reach.
- The `BargeCapacityInputs` struct (`barge_capacity.rs` ~120–257) is one flat struct with a gas branch and a bulk
  branch. `BargeInputs` (`models.rs` 19–160) is the gas barge's input set ("ATB LNG/LPG barge").

### 2.3 Double bottom and wing spaces today

- **Dry holds:** `hold_double_bottom_m` (1.2 m) and `hold_wing_void_width_m` (1.0 m) set the hold size only. No
  structure, no weight (box k factor), no rule.
- **Gas tanks:** the double bottom is `max(0, bottom clearance − saddle height)` from IGC 2.4; "wing tanks" are a
  ballast shelf beside a cylinder (`wing_tank_width`, `wing_tank_gap`, `wing_tank_height_ratio`), with a wing plate
  and weather shield in `weights.rs`. This geometry is specific to cylinders and does not carry over.

### 2.4 Capacity, weights and stability

| Step | Dry bulk | Gas |
|---|---|---|
| Cargo zone | `arrangement()` `barge_capacity.rs` 727–808: rake + end bulkhead allowance; stern block or pushed void | Same, with `gas_separation` cofferdam and end spaces (793–797) |
| Cargo volume | Hold section × length × `hold_fill_ratio` | Tanks × inner volume × `fill_ratio` |
| Cargo mass | `V × ρ` (line 886) | Same |
| Lightship | `k × L × B × D` (box) or plating build-up (hopper) | `weights::calculate_weights`: hull plating + tank shells + insulation + wing plate + shield, factors |
| Weight-bound cargo | `simulate.rs` 730–751: `Cb·L·B·Tmax·ρw − lightship` | `journey_derating::compute` |
| Stability | `bulk_gm`: one first-order GM, warning only (lines 685–712, 1001–1015) | `gas_stability::assess_num` (690–800): three conditions (Laden, Half laden, Light), segment fill of a circle, free surface `n·L·c³/12`, box GZ 0–60°, IS Code A 2.2 criteria |
| Free surface | none | Cylinder chord only. `tanks::free_surface_moment_cylindrical` (272–295) has a known bug (basis 06 E3) and reaches only the Python API. |

## 3. Map of touch points

Legend for "Change": **R** = refactor with no result change; **A** = add a branch or entry; **N** = new code; **D**
= data (profile, rule file, locale). Every row lists the function and what changes.

### 3.1 Core (`crates/ovcd-core/src`)

| # | File | Function or item | Change |
|---|---|---|---|
| 1 | `cargo.rs` | `CargoProperties.phase` | **R**: replace `&str` with `enum CargoClass { LiquefiedGas, DryBulk, LiquidBulk }`; keep the string for serialisation. Add fields a liquid needs: density basis (temperature), heating need, ADN UN number or MARPOL "oil" flag (each value sourced or marked estimate). |
| 2 | `cargo.rs` | `CARGO_DATABASE` (array size 6) | **A**: add `crude_oil` (and later product, chemicals, vegetable oil). Array length changes. |
| 3 | `cargo.rs` | `get_cargo` error text | **A**: the "Valid types" list grows; update pinned messages. |
| 4 | `barge_capacity.rs` | `compute`, line 829 `is_gas = phase != "dry_bulk"` | **R** then **A**: the main trap. Becomes `match class`, with a `LiquidBulk` arm to the new module. |
| 5 | `barge_capacity.rs` | `compute`, line 836 `if phase == "dry_bulk"` | **R/A**: same `match`. |
| 6 | `barge_capacity.rs` | `compute`, line 1002 bulk GM gate | **R**: only `DryBulk`. |
| 7 | `barge_capacity.rs` | `arrangement(inp, is_gas)` 727–808 | **R/A**: `is_gas: bool` becomes the containment; the end separation (aft and forward) for cargo tanks is cofferdams and an optional pump room, not `gas_separation`. Unit-count defaults by phase (764–774) gain a third arm. |
| 8 | `barge_capacity.rs` | `BargeCapacityResult` | **A**: `phase` string gains a value; add an optional `liquid_tanks` detail (or keep the detail inside the new module's own result, see section 5). |
| 9 | `barge_capacity.rs` | `bulk_volumetric` `hold_type` | none for crude; note the dead `"container"` arm. |
| 10 | `ga_adapter.rs` | `is_gas` (100–102), `to_ga` (226) | **A**: Python-API GA; add a tank arm or refuse liquid bulk clearly. |
| 11 | `gas_stability.rs` | `box_gz` (242), `box_condition_at` (596), `criteria` (430) | **R**: move to a shared `box_stability` module so the liquid path can call them. No result change. |
| 12 | `gas_stability.rs` | `assess_num` (690) | none; stays gas. Its condition loop is the template for the liquid conditions. |
| 13 | `weights.rs` | `calculate_weights` hull plating part (lines ~49–72) | **R**: split the hull plating and structure factor into a function the tank path can reuse. |
| 14 | `hydrostatics.rs` | `find_equilibrium_draft` (21) | reuse as is. |
| 15 | `journey_derating.rs` | `compute` (105) | reuse for the weight-bound cargo, or reuse the dry `simulate` form. |
| 16 | `tanks.rs` | `free_surface_moment_cylindrical` (272) | do not reuse (bug E3). New rectangular free surface goes in the new module. |
| 17 | `vessel_standards.rs` | `standards_for` (138–147) | keep: frozen 0.1.0 API accepts only `dry_bulk` / `liquid_gas`. Document; do not widen. |
| 18 | `barge_cost.rs` | `BargeCostInputs`, `compute` (342–351) | **A**: fixed 8-line barge array has `type_c_tanks`, `tank_supports`, `insulation`. Liquid needs cargo tank steel (in hull steel), coating, pumps, cargo piping, heating coils, and no insulation. |
| 19 | `design_grid.rs` | `sweep` | none: gas only (`TANKS_LONG`, `SIDE_CLEARANCE_M`). A liquid sweep is later. |
| 20 | `gas_arrangement.rs` | `fit`, `smallest_viable_loa` | none: gas only. A liquid `fit` (number of tanks in line, double-hull width) is a later ticket after #277. |
| 21 | new `liquid_bulk/` | see section 5 | **N** |

### 3.2 Format (`crates/ovcd-format/src`)

| # | File | Function or item | Change |
|---|---|---|---|
| 22 | `vdesign.rs` | `CARGO_CONTAINMENTS` (46–50) | **A**: `("integral_tanks", "liquid_bulk")`, later `("independent_tanks", "liquid_bulk")`. |
| 23 | `vdesign.rs` | `CARGO_PHASES` (53) | **A**: `"liquid_bulk"`. Feeds every `KEYED_DEFAULTS` by `cargo_phase`. |
| 24 | `vdesign.rs` | `KEYED_DEFAULTS` (57–66) | **D**: the profile must give `cargo_containment`, `battery_unit_count`, `accommodation_unit_count` for the new phase (checked by `check_profile_defaults`). |
| 25 | `vdesign.rs` | `table_combinations` (115–143) | **R/A**: today it skips every containment except `type_c_tanks`. If a liquid input is keyed by (containment, Cargo), widen it; else leave it. |
| 26 | `vdesign.rs` | `TABLE_KEYS` (107) | none for crude. A grade or temperature key, if any, follows #315's shape. |
| 27 | `vdesign.rs` | `Group` enum (756–805), `ALL` | **A**: `Group::CargoTanks` ("Cargo tanks"), with a note like `GAS_TANKS_NOTE`. |
| 28 | `vdesign.rs` | `CHOICE_LABELS` (808–828) | **A**: `integral_tanks`, `independent_tanks`, pump arrangement choices. |
| 29 | `vdesign.rs` | `INPUTS` spec list (~973–1270), `cargo_containment` choice (1021–1026) | **A**: containment choice list; new inputs `only("integral_tanks")`: double side width, cargo double bottom, tanks in line, centreline bulkhead, cargo fill ratio, slop tank share, cofferdam length, pump arrangement, pump room length, heating, cargo density (typed), cargo temperature. |
| 30 | `vdesign.rs` | `depth_group` (1707–1714) | **A**: `LiquidBulk → "liquid"`. The profile ratio 0.30 already exists. |
| 31 | `vdesign.rs` | `cross_checks` (2697–2720) | automatic from row 22; add tank-geometry cross checks (double side < B/2, tanks in line ≥ 1). |
| 32 | `vdesign.rs` | `impossible_results` (2806–2860) | **A**: refuse a tank breadth or height ≤ 0, as for holds today. |
| 33 | `vdesign.rs` | `FORMAT_VERSION` (36) | see row 58 and section 4. |
| 34 | `run.rs` | `simulate_params_of` (86–200) | **A**: new input names in the `put` list; containment → `"tank_barge"`. |
| 35 | `run.rs` | `UNHASHED_KEYS` (244) | none. Put drawing-only values in `drawing` (unhashed). |
| 36 | `simulate.rs` | `vessel()` (193–204) | **A**: `"tank_barge" => ("Tank barge, integral cargo tanks", None, "liquid_bulk")`. |
| 37 | `simulate.rs` | `simulate` dispatch (647–680) | **A**: `if vessel_id == "tank_barge" { simulate_liquid(...) }`. |
| 38 | `simulate.rs` | new `simulate_liquid` | **N**: mirrors the dry path (730–811): limits, weight-bound, `min`, `governs` = "tank volume" or "draught — …", checks, curves, plus `stability` like `simulate_gas`. |
| 39 | `simulate.rs` | `structure_json` (413–484) | **A**: `is_gas = separation.is_some()` stays correct; add cargo-tank drawing values under `drawing`. |
| 40 | `simulate.rs` | `geometry` object (765–779) | **Risk**: this object is hashed. Add new keys **only in the liquid path**, never to the dry or gas objects, or every saved hash moves. |
| 41 | `simulate.rs` | `cargo_options` (176–190), `simulate_condition` (1101) | `cargo_options` grows automatically; reuse `simulate_condition` for the liquid conditions. |
| 42 | `session.rs` | `input_entry` cargo choices (112–125), `applies_to_cargo` (89–94) | **A**: emit a `status: "development"` per Cargo so the app can mark crude. |
| 43 | `session.rs` | `depth_rule` `cargo_group` (260–285) | automatic via row 30. |
| 44 | `session.rs` | keyed-default chain `cargo` → `cargo_phase` (936–965) | automatic; test that changing Cargo from bauxite to crude resets the containment default. |
| 45 | `profile.rs` | `from_json` backfill (lines ~266–316) | **A**: an older user copy has no `liquid_bulk` key in `default_by_cargo_phase`. Backfill it from the built-in, as `LEGACY_FIXED_INPUTS` entries are backfilled, or the copy is refused. |
| 46 | `builtin_profile.json` | `cargo_containment.default_by_cargo_phase` (line 50), unit counts, new input entries | **D**: each value `sourced` or `estimate` with a basis. |
| 47 | `objects.rs` | `phase` parse (337) | none (string). |
| 48 | `eval.rs` | `cargo_options` / `get_cargo` (226, 237), `standard_from_fields` (906) | `cargo_options` grows; keep 906 as frozen API. |
| 49 | `ga.rs` | `Containment` enum, `read_hull` (292–330) | **R/A**: `else` makes every non-gas phase a `Hold` and then fails on `hold_top_width_m`. Add `Containment::CargoTanks { n_long, centreline, double_side, double_bottom, tank_x: Vec<(f64,f64)>, cofferdams, pump_room }`, and match on phase exhaustively. |
| 50 | `ga.rs` | `zones` (874), `side_elevation` (1059), `plan` (1158), `midship_section` (1361) | **N**: tank outlines, centreline and transverse bulkheads, ballast spaces hatched, cofferdams, pump room in the aft zones, trunk or deck line, heating note. |
| 51 | `cost.rs` | `gas_masses` (74), `cost_of` (151) | **A**: a `liquid_masses` for `cost_masses`. Until then `cost_masses` is null and the app shows "Work in development". |

### 3.3 App, Python, tools, tests

| # | File | Item | Change |
|---|---|---|---|
| 52 | `app/src/lib/views/CapitalBudget.svelte` | `phase !== "liquid_gas"` → WIP (line 60) | Crude gets "Work in development" for free. **Risk**: when crude gains cost, this test becomes wrong; switch to `cost == null`. |
| 53 | `app/src/lib/views/Stability.svelte` | `!stability` → WIP (310–314) | Works once `simulate_liquid` writes `stability`. |
| 54 | `app/src/lib/InputField.svelte` | cargo picker (134), `GROUP_IDS.liquid` (45) | Picker list comes from the schema (row 42). Show the `development` tag beside crude. `app-cargo-group-liquid` already exists. |
| 55 | `app/src/lib/MainPanel.svelte` | BOQ placeholder (161–164) | none until the BOQ phase; crude BOQ lines wait with it. |
| 56 | `locales/en/app.ftl`; after #309 `engine.ftl`, `inputs.ftl` | new labels, groups, choices, checks, GA labels | **D**: English first; draft translations per #311. |
| 57 | `src/vessel_designer/core/cargo.py`; `tests/data/public_api.json`; `tests/api_scenarios.py` | Python copy and API snapshot | **A**: keep the Python `CARGO_DATABASE` equal to Rust; re-pin the snapshot in its own commit. |
| 58 | Tests that pin the Cargo list or phase | `crates/ovcd-format/tests/default_depth.rs` 101 (`…liquid_has_none_yet`), `type_c_defaults.rs`, `simulate_inputs.rs`, `vdesign.rs`, `eval_refusals.rs`; `tools/design_impact.py` corpus (line 77 `GAS_CARGOES`, 85–99); `calibration/cases.yaml` 1321+ (41 `hold_type: tank` records); `evaluation/` | **A**: update pins; add crude fixtures (`crates/ovcd-format/tests/fixtures/crude_integral_tanks.vdesign`, pushed and self-propelled), design-impact cases, calibration runner arm, harness cases. |

**Count: 58 touch points.** Of these, 14 are phase branches that the enum refactor (row 1) turns into compile errors,
which is the point of doing it first.

## 4. The Design, the format and the Defaults Profile

- **No format bump is needed to add crude.** A new Cargo key, a new containment value and new inputs are additive.
  ADR 0007 lets an unreleased change reuse a version while it is unreleased; a new input with a profile default fills
  in for older files (they have no crude Options anyway).
- **But crude should not ship before format version 3 (#274) if it needs `service_area` and `rule_sets`.** The
  `marpol-i` versus `adn-n` choice is driven by the Service Area. Until #274/#275, every Option uses the
  `river-only` default stack, so a coastal crude tanker would be checked against no oil Rule Set. Ship crude as
  "work in development" before #274, with its Rule rows `not evaluated`, or wait for #275.
- **Hull sections table (ADR 0012).** Version 3 also carries the hull block. Integral tanks are bounded by the inner
  hull, so their volume is an integral of the inner-hull offsets. Today the hull is a raked box and the cargo zone sits
  in the parallel body, so a rectangular tank is exact there. Design the tank geometry so that it asks the hull for an
  inner breadth and height at a station (section 5, internal seam). When the sections table lands, a second adapter
  plugs in and the seam becomes real.
- **Defaults Profile rows needed** (each with `status` and `basis`):
  - `cargo_containment.default_by_cargo_phase.liquid_bulk = "integral_tanks"`.
  - `battery_unit_count` / `accommodation_unit_count` by phase: new `liquid_bulk` values.
  - New inputs: double side width, cargo double bottom height, tanks in line, centreline bulkhead, fill ratio, slop
    share, cofferdam length, pump room length, heating, density, temperature. Defaults that a rule book bounds (the
    double hull widths, tank volume) must pass the **defaults coherence test** of spec 0004 once the `adn-n` or
    `marpol-i` Rules exist.
  - `default_by_table` keyed by (containment, Cargo, …) is the right home for a value that differs by oil type (fill
    ratio, heating, density). The table machinery (#292) already supports it; `table_combinations` must stop skipping
    non-Type-C containments for those inputs.
  - Cost rate entries for the new BOQ lines (example rates, "not a market quote", as today).
- **User profile copies** (row 45): add backfill for the new keyed key, or older copies are refused.

## 5. Proposed module structure: liquid bulk / integral tank containment

### 5.1 Design goals

- One **deep module** that turns "a hull, a cargo and a tank layout" into "tanks, volumes, masses, free surfaces and
  spaces". Callers (`simulate_liquid`, the GA, the cost, the rules Measures, a later `fit`) learn one small interface.
- It serves crude first and other oils later with **no new branch per oil**: an oil differs by data (density, heating,
  fill ratio, ADN type or MARPOL category), not by code.
- It is the first cut of the roadmap phase 2 **compartment model** (`ROADMAP.md` line 56), not a parallel throwaway.
  The cargo tanks, wing ballast tanks, double bottom, cofferdams, slop tanks and pump room are spaces with a box, a
  role and a volume.

### 5.2 Layout

```
crates/ovcd-core/src/
  cargo.rs                     CargoClass enum; CargoProperties gains liquid fields (data only)
  box_stability.rs   (moved)   box_gz, box_condition_at, criteria  ← from gas_stability.rs, no result change
  liquid_bulk/
    mod.rs                     INTERFACE: compute(&LiquidBulkInputs) -> Result<LiquidBulkResult, PyError>
                                          conditions(&LiquidBulkResult, &ConditionInputs) -> Vec<Condition>
    layout.rs                  internal: end spaces (cofferdams, pump room), tanks in line, P/S split,
                               slop tanks; gives a list of Spaces
    geometry.rs                internal seam: InnerHull (breadth, bottom, deck at a station);
                               adapter 1 = raked-box parallel body; adapter 2 = sections table (ADR 0012, later)
    spaces.rs                  Space { id, role: CargoTank|Slop|WingBallast|DoubleBottom|Cofferdam|PumpRoom,
                                       side: P|S|C, x0, x1, y0, y1, z0, z1, gross_volume, centroid }
    free_surface.rs            rectangular tank i = l·b³/12 per slack tank; worst-case per condition
    lightship.rs               tank-barge lightship: k-factor fit (41 calibration records) first;
                               plating build-up (reuses weights.rs hull plating) second
    tests (inline + crates/ovcd-core/tests/liquid_bulk.rs)
crates/ovcd-format/src/
  liquid.rs                    simulate_liquid, liquid results JSON, liquid_masses for cost
  ga.rs                        Containment::CargoTanks arm; draws Spaces by role
```

### 5.3 The interface

```rust
pub struct LiquidBulkInputs {
    pub hull: HullDims,                 // loa, beam, depth, cb, rakes (or a sections table later)
    pub cargo: &'static CargoProperties,// or a resolved density + flags after #315
    pub containment: TankContainment,   // Integral | Independent { clearance_m }
    pub layout: TankLayout,             // double_side_m, double_bottom_m, tanks_in_line, centreline: bool,
                                        // cofferdam_m, pump_room_m, slop_share
    pub fill_ratio: f64,
    pub aft: Option<AftArrangementResult>,
}

pub struct LiquidBulkResult {
    pub spaces: Vec<Space>,             // fixed order: 1P, 1S, 2P, … then ballast, then others
    pub cargo_volume_m3: f64,           // Σ cargo tank gross × fill
    pub cargo_mass_t: f64,              // volume-bound
    pub lightship_t: f64,
    pub weight_items: Vec<(f64, f64)>,  // (t, KG) for stability
    pub checks: Vec<Check>,
    pub warnings: Vec<Text>,            // `Text` after #309
}
```

- **Depth.** The caller gives seven layout numbers and a Cargo, and gets every space, every volume and every mass.
  Bulkhead placement, P/S split, slop allocation, end spaces, inner-hull clipping and free surface stay inside.
- **Two adapters at the `TankContainment` seam** (integral, independent) make it a real seam: independent tanks only
  add a clearance to the inner hull and their own shell weight.
- **`InnerHull` is an internal seam with one adapter today.** Keep it private until the sections table gives a second
  adapter; then lift it to the interface. (One adapter means a hypothetical seam.)
- **Rules read Measures from `LiquidBulkResult`**, never from the module's inside: `hull.double_side_width`,
  `hull.double_bottom_height`, `tank.volume`, `tank.length`, `option.cargo_capacity_m3`, `option.deadweight`. That keeps
  the module testable through one interface, and the Measure registry (#269) is the second caller of the same seam.
- **Deletion test.** Delete the module and the layout, clipping and free-surface logic reappears in `simulate_liquid`,
  `ga.rs`, `cost.rs` and the Measures: four callers. It earns its keep.

### 5.4 What is reusable

| Reuse | From | For |
|---|---|---|
| Cargo zone and end spaces pattern | `barge_capacity::arrangement` | Bow rake, stern block or pushed void; add cofferdams and pump room as a third arm. |
| Volume-bound vs weight-bound cargo, `governs` | `simulate.rs` 730–751 | Same formula, "tank volume" in place of "hold volume". |
| Draught solver, box hydrostatics | `hydrostatics::find_equilibrium_draft`, `hull::RakedBox` | Unchanged. |
| Box GZ with free surface correction, IS Code A 2.2 criteria | `gas_stability::box_gz`, `box_condition_at`, `criteria` | Liquid loading conditions (after the move to `box_stability.rs`). |
| Condition JSON and the app stability view | `simulate_condition`, `Stability.svelte` | Unchanged shape. |
| Hull plating weights | `weights.rs` (plating part) | Tank-barge plating build-up. |
| Stern block, thrusters, wheelhouse | `aft_arrangement.rs` | Self-propelled small coastal tanker. |
| Resistance, squat, waves curves | `simulate::curves` | Unchanged. |
| Keyed and table defaults | `KEYED_DEFAULTS`, `default_by_table` (#292) | Oil-type defaults. |
| Depth rule group `liquid` (0.30) | `builtin_profile.json` 24–30 | Default moulded depth for crude. |
| Calibration records | `calibration/cases.yaml` 1321+ (41 tank barges with DWT and tank capacity) | Lightship k fit and capacity checks, held out. |
| Evaluation case form `kind: box` with `free_surface_m` | `evaluation/cases/*.yaml`, `oracle/box_oracle.py` | Stability boundary cases for liquid conditions. |

**Not reusable:** cylinder tanks, segment fill, cylinder free surface, wing shelf and weather shield (`tanks.rs`,
`weights.rs` wing parts), `gas_separation`, `protective_location` (IGC d(Vc)), `design_grid`, gas `fit`.

### 5.5 What a crude path needs that nothing gives today

1. Rectangular cargo tanks between the inner bottom, the inner sides, the deck (or trunk), the centreline bulkhead and
   transverse bulkheads.
2. Ballast spaces in the double hull, with volume and centroid (a ballast condition needs them; MARPOL segregated
   ballast rules need them).
3. Standard loading conditions: departure laden, arrival laden (10 % consumables), ballast, and a worst slack-tank
   condition for liquid transfer. Today: Laden, Half laden, Light only, no consumables (basis 03, G3, G4).
4. Free surface for rectangular slack tanks, split by the centreline bulkhead (the bulkhead cuts `b³` by four).
5. A tank-barge lightship method with a calibration basis.
6. Heating: coil mass and its cost line; a heating-power estimate is later.
7. Pumps: a pump room length in the end spaces, or deepwell pumps per tank (mass and cost only).
8. Cofferdams and slop tanks as spaces.

## 6. Rules engine (spec 0004): new Rule Sets and Measures

### 6.1 What the spec already supports

- A new Rule Set is a data file: `crates/ovcd-format/rules/<set-id>/<version>.json`, with `applies_when`,
  `includes`, tables and Rules (spec "Rule Set (data)"). `marpol-i` and `adn-n` fit this, family `marpol` and `adn`.
- Rule ids by paragraph (`adn/9.3.3.11.7/double-side`, `marpol/I.19.6.1/double-bottom-height` style); the loader
  checks the paragraph against the locator. Each must be read in the text first (`docs/rules/codification.md`
  step 1); this review read neither code.
- A **table** limit keyed by a Measure value: the ADN tank capacity limit by L × B × H (basis 06 proposal G1) fits,
  with a Measure `hull.lbh` and a `tank.volume` Rule.
- A **formula shape** with coefficients as data: a double-hull width that grows with deadweight fits, if the shape
  exists in Rust.
- Object types `option`, `hull`, `tank`, `loading-condition`: cargo tanks are `tank` objects; the double-hull widths
  are `hull` Measures; intact stability Rules are `loading-condition` Rules.
- `sizes: true` lets the double-side and double-bottom Rules feed a later liquid `fit`.
- `not evaluated` rows let oil outflow and damage stability appear before their Measures exist.
- Service Area default stacks: add a row "Liquid bulk" to the spec's stack table.

### 6.2 What is missing for liquid cargo rules

| Gap | Why it matters | Proposal |
|---|---|---|
| **Numeric applicability.** Applicability is `input ∈ {values}` only (spec "Applicability"). | MARPOL Annex I construction rules switch on deadweight bands (for example below and above 5,000 t); ADN on length or capacity. | Either the out-of-scope "Applicability Rules", or a table limit whose out-of-band rows give a no-op bound, or split by an enumerated derived input (`deadweight_band`). Needs a maintainer decision and probably an ADR 0013 note. |
| **Per-object applicability.** `applies_when` reads Option inputs, not object attributes. | A Rule on `tank.volume` must apply to cargo tanks, not to ballast tanks or slop tanks. | Either one object type per role (`cargo-tank`, `ballast-tank`), or an object attribute condition. One object type per role is the smaller change and keeps the no-expression rule. |
| **Cargo group for Applicability.** Rules name `cargo` values one by one. | Every oil added later would need every Rule Set edited. | Make the Cargo class (or an oil category) an enumerated input the schema names (derived, read-only), so `applies_when: {cargo_class: [liquid_bulk]}`. |
| **Clamp formula shape.** | Double-hull widths are "a + b·DW, not less than x, need not exceed y". | Add a shape `max(lo, min(hi, a + b·x))` in Rust, coefficients as data, with its own sourced test. |
| **Gas sets must not apply to crude.** | `igc-2g`, `lr-inland-gas` and `ovcd/gas-arrangement/*` apply today by default stack, not by applicability. | Give each of them `applies_when: {cargo_containment: [type_c_tanks]}` in #271/#272, so a crude Option gets `not applicable` rows, not false passes or failures. |
| **Loading conditions beyond three.** | MARPOL Annex I intact stability asks for the worst case in liquid transfer; IS Code Part B lists departure and arrival conditions. | `loading-condition` objects exist; the condition generator (section 5.5 item 3) is the missing Measure input. |
| **Oil outflow.** | A probabilistic Measure over the compartment layout and damage distributions. | Large new Measure; ship it first as a `not evaluated` row with `gap: "needs oil outflow Measure"`. |
| **`ovcd` checks for liquid.** | Tank breadth > 0, double side < B/2, cargo zone > 0. | New Rules in the `ovcd` set with origin `physical`. |

### 6.3 New Measures (Rust, with registry entries)

| Measure id | Object | Unit | Source of value |
|---|---|---|---|
| `hull.double_side_width` | hull | m | `LiquidBulkResult` (min over cargo length) |
| `hull.double_bottom_height` | hull | m | same |
| `hull.lbh` | hull | m³ | L × B × H, as the ADN table needs it (definition to be read in the text) |
| `tank.volume` | cargo tank | m³ | gross volume of each cargo tank |
| `tank.length` | cargo tank | m | each cargo tank |
| `option.deadweight` | option | t | weight-bound result |
| `option.cargo_capacity_m3` | option | m³ | Σ cargo tanks |
| `option.slop_capacity_share` | option | – | slop / cargo volume |
| `loading-condition.*` | condition | – | existing `gz.*`, `gm0` Measures, fed by the liquid conditions |
| `option.oil_outflow_mean` | option | – | later; `not evaluated` until built |

## 7. Drawings, BOQ, cost and app UI

- **GA (`ga.rs`):** a `CargoTanks` containment arm. Plan: cargo tanks P/S with numbers, centreline bulkhead,
  transverse bulkheads, wing ballast hatched, cofferdams, pump room or deepwell pump symbols. Elevation: tank
  boundaries, double bottom, cofferdams, trunk if any. Midship section: inner hull, double side, double bottom,
  centreline bulkhead, heating coil note. Labels go through the Text Library (#310). No pipe routing: concept only
  (ADR 0011).
- **Draw crate (`ovcd-draw`):** no change to the format standard; a pump symbol needs a symbol library entry.
- **BOQ:** the app has no BOQ yet (`MainPanel.svelte` 161–164, "comes in a later phase"). Crude BOQ lines (cargo
  tank steel, coating area, heating coil length, pumps, cargo piping, valves, gauging, vents) wait for that phase;
  the cost model (row 18) can carry lump sums first.
- **Cost:** `barge_cost::compute` has a fixed barge item list. Make the list per containment (Type C list, cargo
  tank list) behind one `compute`. App `CapitalBudget.svelte` item ids gain the new lines and locale ids.
- **App cargo picker:** the list is the schema's `choices`. Today nothing marks a Cargo as in development. Add a
  `status: "development"` field per Cargo in `session.rs` and show a tag in `InputField.svelte`; reuse
  `app-work-in-development`. The Capital budget and Stability views already show "Work in development" when cost or
  stability is null, so crude is marked there with no app change, until row 52 is fixed.

## 8. Tests and harness

- **Unit and golden tests** (`crates/ovcd-core/tests/liquid_bulk.rs`): closed-form box tanks (volume, centroid, free
  surface `l·b³/12`) labelled `Label: sourced` or `derived` per `docs/agents/build-process.md`.
- **Fixtures** (`crates/ovcd-format/tests/fixtures/`): `crude_integral_tanks_pushed.vdesign` and
  `…_self_propelled.vdesign`, with pinned results hashes (`tools/check_fixture_hashes.py`) and the web-parity job.
- **Design impact** (`tools/design_impact.py`): add crude cases to the corpus so later changes show their effect.
- **Calibration** (`calibration/cases.yaml`): the 41 `hold_type: tank` records become runnable through a `tank`
  arm in `src/vessel_designer/calibration.py` `run_barge`. Keep them held out; fit k on a declared subset only.
  Add `tank_bands.yaml`-style bands for tank barge lightship and deadweight.
- **Evaluation harness** (`evaluation/`): stability cases use the existing `kind: box` form with `free_surface_m`, so
  `oracle/box_oracle.py` works as is. Double-hull and tank-volume Rules need a new independent oracle
  (`oracle/tank_oracle.py`) that writes boundary cases, never typed by hand (codification step 6). After #278 the cases
  are keyed by rule id; after #279 the `adn-n` and `marpol-i` limits are mutated.

## 9. Order of work and dependencies

| Step | Work | Result change | Depends on |
|---|---|---|---|
| 0 | Expert input: which oil first (crude vs product), ADN tank vessel type and tank design for crude from ADN Table C, MARPOL Annex I applicability for the size band, fill ratio basis. Read the texts. | none | Basis page 06 proposals Q9; codification step 1 |
| 1 | **Refactor:** `CargoClass` enum; every phase branch a `match`; move box stability to `box_stability.rs`; split hull plating out of `weights.rs`. | none (hashes identical) | none. Do before #309 touches the same files, or rebase after it. |
| 2 | Cargo data: `crude_oil` entry (Rust and Python), following #315's density shape; Cargo `status: development`. | new Cargo only | step 1; **#315** (same files: `cargo.rs`, profile, `TABLE_KEYS`) |
| 3 | `liquid_bulk` module: layout, spaces, volumes, free surface, k-factor lightship. | new results only | step 1 |
| 4 | Format: containment `integral_tanks`, inputs, profile rows and backfill, `simulate_liquid`, run mapping, fixtures, design-impact cases. | new results only | steps 2, 3 |
| 5 | Loading conditions (departure, arrival, ballast, slack) and the stability output. | new results only | step 3; reuse after step 1 |
| 6 | GA arm and drawing labels. | none to hashes (`drawing` is unhashed) | step 4; **#310** for labels |
| 7 | App: development tag, cost WIP test fix, locale ids. | none | step 4; **#309**, **#311** |
| 8 | Calibration tank arm; held-out runs. | none | step 4 |
| 9 | Spec 0004 additions: numeric applicability decision, per-role object types, cargo class input, clamp shape; gas sets get `applies_when` on containment. | none | **#268, #269**; gas `applies_when` inside **#271, #272** |
| 10 | Measures of section 6.3 in the registry. | none | **#269**, step 3 |
| 11 | Rule Sets `adn-n` and `marpol-i` 1.0.0, `unverified` until a second reader; harness boundary cases. | warn first (spec: a new Rule enters as `warn`) | step 10; **#273**; **#278** for rule-id cases |
| 12 | Default stacks by Service Area for liquid bulk. | default stack change | **#274** (format v3, with the ADR 0012 hull block), **#275** |
| 13 | Rule check tab shows liquid rows; `fit` for liquid (tanks in line, double hull). | `fit` results | **#276**, **#277** |
| 14 | Rule mutation over the oil sets. | none | **#279** |
| 15 | Cost lines, then BOQ in the BOQ phase. | cost only (unhashed) | step 4 |
| 16 | Second oil (product, chemicals, vegetable oil): data rows only, plus ADN type C or chemical rules as new sets. | new Cargo only | steps 2–11 |

**Tank fix tickets #313, #314, #316, #317** touch Type C tanks and the `igc-2g` set only. They do not block crude.
**#315** shares files and the density model; land it first or agree the shape. **#313**'s defaults coherence finding
shows the same test will run over the new liquid defaults.

**Language tickets:** every new check text, warning, input label and GA label after **#309** uses `text!` with a
Message Id; before #309 it is plain English that #309 then converts. Land step 1 and the new messages after #309 to
avoid doing the work twice. **#310** owns drawing labels; **#311** adds draft translations for the new ids.

## 10. Main risks

1. **Silent wrong routing.** `is_gas = phase != "dry_bulk"` (core) and the `else → Hold` in `ga.rs` would send
   crude down the wrong path with no error. Mitigation: step 1 enum refactor first.
2. **Hash drift.** Adding keys to the shared `geometry` or `results` objects moves every saved results hash. Keep new
   keys in the liquid path only, and drawing-only values in `drawing`.
3. **A coastal crude tanker checked against nothing.** Until #274/#275 every Option uses the river-only stack, and
   gas Rule Sets apply by stack, not by applicability. Mitigation: mark crude "work in development"; give gas sets an
   `applies_when` on containment; list `marpol-i` Rules as `not evaluated`.
4. **Spec 0004 cannot express deadweight bands or cargo-tank-only Rules.** Needs a maintainer decision before
   `marpol-i` is codified.
5. **Unsourced values.** Crude density, fill ratio, double-hull widths and tank limits must be read in the texts.
   This review read none of the MARPOL or ADN text. Every default starts `estimate` until sourced.
6. **Two Cargo databases.** The Python `cargo.py` copy and the Rust array can diverge; a test should keep them equal.
7. **Parallel compartment model.** If `liquid_bulk` is built as a one-off, roadmap phase 2 builds a second one. Build
   `Space` as the seed of the compartment model.
8. **Older user profiles refused** unless the new `liquid_bulk` keyed default is backfilled.
9. **Scope.** MARPOL Annex I and ADN tank vessels sit near the edge of ADR 0011 ("below the size where SOLAS applies in
   full"). Small coastal tankers are inside it; confirm the size band with the maintainer.
