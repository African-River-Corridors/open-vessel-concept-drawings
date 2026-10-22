# Liquid, dry bulk and container cargo

> Concept and preliminary design only. Not for approval. Draft: needs expert review.
>
> Code state: written against `main` at commit `f6d446c` (2026-10-20).

**For:** Barge designer  ·  **Code:** `crates/ovcd-core/src/cargo.rs`, `barge_capacity.rs`, `tanks.rs` (volumes and free
surface), `journey_derating.rs`, `checks.rs`, `models.rs`; `crates/ovcd-format/src/simulate.rs`, `vdesign.rs`, `run.rs`,
`builtin_profile.json`  ·  **Rule Sets:** none. No Rule Set is in the code yet (spec 0004 is a design). Every limit on
this page is a constant or a Defaults Profile value.

This page describes the code on 2026-10-20. "Liquid cargo" in OVCD today means liquefied gas in IMO Type C tanks only.
The tank design, the fit of the tanks and the gas clauses are on pages 04 and 05. This page covers what is common to all
Cargo: the cargo data, the cargo volume, the cargo mass, the capacity and deadweight, the dry bulk hold, the double
bottom and wing spaces, and the cargo zone checks.

## What OVCD computes here

For each Option, `ovcd run` gives these results (`simulate.rs`, `simulate` for dry bulk, `simulate_gas` for gas):

- `cargo_t`: the cargo mass the Option carries, the lesser of the volume-bound and the weight-bound cargo.
- `cargo_volume_bound_t` and `cargo_weight_bound_t` (dry bulk), or `payload_t` and `volume_capacity_t` (gas).
- `governs`: "hold volume" or "tank volume", or "draught — <limit name>".
- `cargo_volume_m3`, `lightship_t`, `displacement_t`, `deadweight_t`, `draught_m`, `draught_max_m`, `freeboard_m`.
- `light_draught_m`, `air_draft_light_m`, `air_draft_loaded_m`.
- Geometry: `cargo_zone_length_m`, `bow_void_length_m`, `stern_void_length_m`, `bow_rake_length_m`, and for a dry
  hold `hold_height_m`, `hold_top_width_m`, `hold_bottom_width_m`.
- Checks: "Cargo > 0", "feasible geometry", "cargo zone positive", "gm positive" (dry bulk, warning only), the light
  air draft check, and the stern block and gas checks rolled up from other modules.

OVCD does **not** compute today:

- **Liquid cargo other than liquefied gas.** No oil, chemical or other liquid Cargo exists. There is no integral or
  independent tank for these cargoes, no ADN tank vessel type, no tank capacity limit and no cofferdam rule for them.
  The Defaults Profile has a `liquid` depth group with no Cargo in it (`builtin_profile.json` line 25).
- **Container stowage.** There is no container Cargo, no TEU count, no tier count, no stack weight and no
  container stability. The core has a `"container"` hold type (`barge_capacity.rs` line 621), but no Design can select
  it (see "How it works today", step 1). The catalogue holds TEU figures for Vessel Classes, but the generated core
  table drops them (`crates/ovcd-core/src/vessel_classes.rs` lines 25 to 46 have no `teu` field).
- **Dry bulk holds are work in development.** The method is a first estimate, with one Cargo (bauxite), no cargo
  shift and a first-order GM only.

## How it works today

### 1. Cargo, phase and Cargo Containment

1. The Cargo is a key in `CARGO_DATABASE` (`cargo.rs` lines 27 to 94). Each entry has a density, a storage
   temperature, a storage pressure and a phase: `liquid_gas` or `dry_bulk`. Six Cargos exist: `LNG`, `LNG_ethane`,
   `LPG_propane`, `LPG_butane`, `LPG_mix` (all `liquid_gas`) and `bauxite` (`dry_bulk`). `get_cargo` (line 97) refuses
   any other key. Label: **Fixed in code**.
2. The storage temperature and pressure feed no calculation. Only the API echoes them (`crates/ovcd-format/src/eval.rs`
   lines 223 to 224).
3. The Cargo Containment is one of `box_hold`, `sloped_hopper_hold` or `type_c_tanks` (`vdesign.rs` lines 46 to 50).
   A dry bulk Cargo goes only in a hold; a gas Cargo goes only in Type C tanks. A file that breaks this is refused
   (`vdesign.rs` lines 2697 to 2720). Left out, the containment of a dry bulk Cargo is `box_hold`
   (`builtin_profile.json` line 50). Label: **Estimate** (a default with no source).
4. `run.rs` (`simulate_params_of`, lines 172 to 177) maps the containment to the engine: `box_hold` to `box_barge`,
   `sloped_hopper_hold` to `hopper_barge`, `type_c_tanks` to `gas_barge`. `simulate.rs` (`vessel`, lines 192 to 203)
   then sets the core hold type: `box` or `sloped_hopper`. No path sets `container`.
5. Left out, the moulded depth is a ratio times the beam: 0.33 for dry bulk, 0.55 for gas (`builtin_profile.json`
   lines 24 to 30; `vdesign.rs`, `depth_group`, line 1707). Label: **Estimate** (midpoint of reference vessels).

### 2. Cargo zone (both phases)

The cargo zone is the hull length that is left after the two end spaces (`barge_capacity.rs`, `arrangement`, lines
727 to 808, and `compute`, line 833).

- Bow rake: `rake = 1.6 × D` unless given (lines 788 to 791). Label: **Estimate**.
- Stern block: a self-propelled hull takes the length of the stern block (thrusters, 40 ft battery and accommodation
  units, wheelhouse; page 02 and `aft_arrangement.rs`). A pushed barge takes `pushed_stern_void_m = 3.0 m`
  (lines 782 to 786). Label: **Estimate**.
- Separation at each end: dry bulk takes `bulk_end_bulkhead_m = 0.5 m` at each end (line 798). Gas takes the
  cofferdam and end spaces of `gas_separation.rs` (line 795; page 05).
- `bow void = rake + forward separation`; `stern void = stern block + aft separation` (lines 802 to 803).
- `cargo zone = LOA − bow void − stern void` (line 833). The check `cargo_zone_positive` needs it above zero
  (lines 949 to 959). Label: **Method** (OVCD Check).

### 3. Dry bulk hold volume (work in development)

`barge_capacity.rs`, `bulk_volumetric`, lines 610 to 679. All lengths in m.

1. Hold height: `h = (D − DB) + coaming` (line 617). DB is `hold_double_bottom_m` (default 0.8; 1.2 in a file saved before #597), coaming is
   `hatch_coaming_height_m` (default 1.0).
2. Hold top width: `w_top = B − 2 × wing void` (line 618). Wing void is `hold_wing_void_width_m` (default 1.0).
3. Hold bottom width (line 619 to 628):
   - Sloped hopper: `w_bot = max(1.0, w_top − 2 × s × h)`, with `s` the hopper slope (H:V, default 1.0). The slope runs
     over the full hold height, coaming included. The 1.0 m floor is fixed in code.
   - Box: `w_bot = w_top` (vertical inner sides).
4. Section area: `A = 0.5 × (w_top + w_bot) × h` (line 629). The coaming part of the hold has the full hold width.
5. Hold length: `L_c = max(0, LOA − bow void − stern void)` (line 630). No length is taken off for transverse
   bulkheads between holds.
6. Hold volume `V_hold = A × L_c`; cargo volume `V_c = V_hold × hold fill` (lines 631 to 632). Hold fill is
   `hold_fill_ratio` (default 0.95).

Labels: the geometry is a **Method** with no reference (an OVCD estimate). Each default is an **Estimate**.

### 4. Dry bulk lightship (work in development)

- **Box hold:** `lightship = k × LOA × B × D`, with `k = lightship_factor_t_per_m3` (default 0.1093 t/m³)
  (`barge_capacity.rs` lines 652 to 660; `builtin_profile.json` lines 80 to 86). `k` is the mean of three published
  pushed box barges (`calibration/factors.yaml`). Label: **Estimate** (in-sample fit, n = 3).
- **Sloped hopper hold:** a plating-area build-up (lines 635 to 651):
  - plate volumes (m³): inner bottom `L_c × B × t_ib`; hopper slopes `2 × h × √(s² + 1) × L_c × t_hop`; wing
    bulkheads `2 × h × L_c × t_bhd`; transverse bulkheads `(n_holds + 1) × A × t_bhd`; coaming
    `2 × coaming × L_c × t_coam`;
  - `hopper steel = Σ plate volumes × 7.85 × 1.40`;
  - `hull steel = 0.10 × LOA × B × D`;
  - `lightship = (hull steel + hopper steel) × (1 + 0.04 + 0.035)`.
  Plate thicknesses are inputs (14, 14, 11, 10 mm). The steel density 7.85 t/m³, structure factor 1.40, hull steel
  rate 0.10 t/m³, piping 0.04 and outfitting 0.035 are core inputs that no Design can set (`simulate.rs` lines 693 to
  715 do not pass them). Label: **Method** with no reference; constants **Fixed in code**.
- **Both holds:** the stern block of a self-propelled hull adds no mass. `aft_arrangement.rs` returns no weight, and
  `compute` takes the lightship from the hold branch only (line 861 to 867).

### 5. Liquid (gas) cargo volume

`tanks.rs`, `calculate_tank_arrangement_wide`, lines 205 to 211: `V_tank = π r_i² L_cyl + 2 V_head`;
`V_total = n_tanks × V_tank`; `V_c = V_total × fill`. The fill is `fill_ratio`, 0.98 by default
(`builtin_profile.json` lines 95 to 112; `simulate.rs`, `fill_by_ruleset`, line 814). The source is IGC Code
(MSC.370(93)) paragraph 15.3: the default filling limit is 98 % at the reference temperature. Label: **Sourced
default**. OVCD applies the filling limit to the volume. It does not apply the loading limit of IGC 15.5.1
(`LL = FL × ρR / ρL`). The core default is 0.95 (`barge_capacity.rs` line 230; `models.rs` line 206), but a Design
always passes the profile value. The dome counts in Vc only, not in the cargo volume (line 523; page 04).

### 6. Cargo mass

`cargo mass = V_c × ρ_cargo / 1000` (t), with `ρ_cargo` from `CARGO_DATABASE` (`barge_capacity.rs` lines 828 and
886). The LPG density does not change with the LPG Carriage Condition. Label: **Method**; densities **Fixed in code**.

### 7. Volume-bound load and hydrostatics

`barge_capacity.rs`, `compute`, lines 886 to 927.

1. The hull is a box `LOA × B × D` with no rakes (line 889). Displacement `Δ(T) = Cb × LOA × B × min(T, D) × ρ_w`
   (line 905; `hull.rs`, `displaced_volume`, lines 124 to 132). Cb default 0.85; ρ_w default 1.0 t/m³.
2. `overloaded` if `(lightship + cargo) / (Cb ρ_w) > LOA × B × D` (lines 895 to 901).
3. The draught is a bisection between 0 and 1.5 D, tolerance `0.001 × B` on volume, 100 rounds at most
   (`hydrostatics.rs`, `find_equilibrium_draft`, lines 21 to 56). The result is clamped to D (line 904).
4. `freeboard = D − T`; `air draft = top above keel − T` (lines 906 to 907). The light draught uses the lightship only
   (lines 910 to 912). Ballast is 0 t in every Design (line 212; no input).
5. The core deadweight is `Δ − lightship` (line 1032).

Label: **Method**.

### 8. Weight-bound cargo, capacity and deadweight (Design path)

Dry bulk (`simulate.rs`, `simulate`, lines 730 to 751):

1. `T_max` = the least of `D − min freeboard`, `draught_limit_m` and channel depth less under-keel clearance
   (lines 730 to 739). Min freeboard default 0.3 m (line 682; `builtin_profile.json` line 59).
2. `weight-bound cargo = max(0, Cb × LOA × B × T_max × ρ_w − lightship)` (lines 741 to 742).
3. `cargo = min(volume-bound cargo, weight-bound cargo)` (line 744). `governs` names the lesser (line 745).
4. `displacement = lightship + cargo`; `T = displacement / (Cb LOA B ρ_w)`; `deadweight = displacement − lightship`
   (line 751 and the `results` block). So the reported deadweight equals the cargo: no stores, fuel or crew.

Gas: `ga::fit` (page 05) runs `journey_derating::compute` (`journey_derating.rs` lines 105 to 135). If the full-load
draught is below `T_max`, the volume governs. Otherwise the cargo is `min(volume capacity, Δ(T_max) − lightship)`.
A warning `utilisation_reasonable` fires below 50 % volume utilisation (lines 160 to 167; default 0.5, line 51).

There is no utilisation against a Vessel Class deadweight. The app only shows the class reference deadweight beside the
cargo (`crates/ovcd-format/src/session.rs` line 153).

### 9. Double bottom and wing spaces

- **Dry bulk holds:** the "double hull" is the wing void (1.0 m each side) and the hold double bottom (0.8 m). They set
  the hold size and nothing else. No check compares them with a rule. No structure or weight is given to them in the
  box hold (the k factor covers all steel). In the sloped hopper the inner bottom plate is in the build-up.
- **Gas tanks:** the side and bottom clearances come from IGC 2.4 and set a double bottom of `max(0, cb − saddle
  height)` (`barge_capacity.rs` line 541). See page 05.
- A file whose wing voids or double bottom leave a negative hold width or height is refused (`vdesign.rs`,
  `impossible_results`, lines 2806 to 2860). Label: **Method** (OVCD Check, physical).

### 10. Dry bulk stability (first order, warning only)

`barge_capacity.rs`, `bulk_gm`, lines 685 to 712, called at lines 1001 to 1015.

- `GM = KB + BM_t − KG`, with KB and BM of the box hull at the **volume-bound** draught (`hull.rs`, `KB` and
  `BM_transverse`; no Cb in the volume).
- `KG_light = 0.5 × D`. `KG_cargo = DB + z_c`, with `z_c = h/3 × (2 w_top + w_bot) / (w_top + w_bot)`, the centroid
  of the full hold section (the hold taken as full to the coaming top).
- The check `gm_positive` is `GM ≥ 0.15 m`, severity warning (line 1006). The value equals IS Code 2008 Part A 2.2.4
  ("not less than 0.15 m"), but the code cites no rule.
- The check runs only if the volume-bound load is not overloaded and the draught is above zero (line 1002). Since #415 a Design's results carry the check `intact gm at reported cargo` instead: the same first-order GM at the cargo and draught the Design reports, in every dry bulk Option (Rule `ovcd/capacity/intact-gm`, `ovcd` 2.0.0).
- A fixed warning says cargo shift and angle of repose are not modelled (lines 814 to 815).

### 11. Free surface of a cylindrical tank (API only)

`tanks.rs`, `free_surface_moment_cylindrical`, lines 272 to 295. It takes the liquid height as `fill × 2r`
(line 280), and `I = L_cyl × chord³ / 12` (line 292); heads are left out. Only `hydrostatics::calculate_stability`
calls it (line 132), and only the Python API (`eval.rs` line 1544) reaches that. A Design does not use it.

### 12. Checks

`checks.rs` lines 12 to 49: a check has a name, a pass flag, a message and a severity. Only severity `"error"` gates
the result; any other text is a warning. In the dry bulk Design path, `simulate.rs` (`capacity_checks`, lines 368 to
381) drops `not_overloaded` and `freeboard_ok`, because the weight-bound cargo replaces them. It keeps
`feasible_geometry` (cargo volume above zero), `cargo_zone_positive`, `gm_positive` and the stern block checks, and
adds "Cargo > 0" (line 755) and the light air draft check (lines 383 to 392).

## Assumptions, rules and estimates

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| CG-A01 | LNG density | 450 kg/m³ at −162 °C | Fixed in code | None in code | `cargo.rs` line 32 | Unknown |
| CG-A02 | LNG/ethane density | 500 kg/m³ at −162 °C | Fixed in code | None in code | `cargo.rs` line 43 | Unknown |
| CG-A03 | LPG propane density | 581 kg/m³ at −42.1 °C (fully refrigerated) | Fixed in code | Python comment names reference tables; not read in this review | `cargo.rs` line 54 | Non-conservative (cargo mass high) for a fully pressurised Option |
| CG-A04 | LPG butane density | 600 kg/m³ at −1.0 °C | Fixed in code | None in code | `cargo.rs` line 65 | Unknown |
| CG-A05 | LPG mix density | 588.4 kg/m³: harmonic blend, 60 % propane at 581, 40 % butane at 600, by mass | Fixed in code | Arithmetic from A03 and A04 (gives 588.45) | `cargo.rs` line 76 | As A03 |
| CG-A06 | Bauxite density | 1500 kg/m³ (stowage factor about 0.67 m³/t) | Fixed in code | None in code | `cargo.rs` line 87 | Unknown: real bauxite stowage factors vary with the ore |
| CG-A07 | LPG density by carriage condition | One density for all three conditions | Fixed in code | — | `barge_capacity.rs` line 828 | Non-conservative for fully pressurised (the default condition) |
| CG-A08 | Custom cargo density | `cargo_density_override` exists and is never read | Fixed in code | — | `models.rs` lines 110, 205 | No effect |
| CG-A09 | Cargo storage temperature and pressure | Stored; feed no calculation | Fixed in code | — | `eval.rs` lines 223 to 224 | No effect |
| CG-A10 | Containment and Cargo pairs | Dry bulk in box or sloped hopper hold; gas in Type C tanks | Fixed in code | — | `vdesign.rs` lines 46 to 50, 2697 to 2720 | — |
| CG-A11 | Default containment, dry bulk | `box_hold` | Estimate | No source | `builtin_profile.json` line 50 | Unknown |
| CG-A12 | Default depth over beam | Dry bulk 0.33, liquid 0.30 (no Cargo), gas 0.55 | Estimate | Midpoint of reference vessels (`catalogue/depth_rule.md`) | `builtin_profile.json` lines 24 to 30 | Unknown |
| CG-A13 | Cargo mass | `V_c × ρ / 1000` | Method | — | `barge_capacity.rs` line 886 | — |
| CG-A14 | Bow rake length | 1.6 × D | Estimate | Carried over from the original engine | `barge_capacity.rs` lines 788 to 791; `builtin_profile.json` line 66 | Unknown |
| CG-A15 | Pushed barge stern void | 3.0 m | Estimate | Carried over from the original engine | `barge_capacity.rs` line 785; `builtin_profile.json` line 73 | Unknown |
| CG-A16 | Dry bulk end bulkhead allowance | 0.5 m at each end | Estimate | Carried over from the original engine | `barge_capacity.rs` line 798; `builtin_profile.json` line 362 | Unknown; small |
| CG-A17 | 40 ft units in the stern block | Dry: 2 battery, 1 accommodation; gas: 1 and 2 | Estimate | Carried over from the original engine | `builtin_profile.json` lines 542 to 555; `barge_capacity.rs` lines 764 to 774 | Sets cargo zone length |
| CG-A18 | Cargo zone | `LOA − bow void − stern void` | Method | — | `barge_capacity.rs` line 833 | — |
| CG-A19 | Cargo zone check | Cargo zone > 0, error | Method (OVCD Check) | Physical | `barge_capacity.rs` lines 949 to 959 | — |
| CG-A20 | Hold height | `(D − DB) + coaming` | Method | No reference | `barge_capacity.rs` line 617 | Non-conservative (volume high): the coaming part takes the full hold width |
| CG-A21 | Hold top width | `B − 2 × wing void` | Method | No reference | `barge_capacity.rs` line 618 | — |
| CG-A22 | Hopper bottom width | `max(1.0, w_top − 2 s h)`, slope over the full height | Method | No reference | `barge_capacity.rs` line 620 | Conservative (volume low) if real hoppers are only at the lower corners |
| CG-A23 | Hopper bottom width floor | 1.0 m | Fixed in code | None | `barge_capacity.rs` line 620 | Unknown |
| CG-A24 | Box hold sides | Vertical, `w_bot = w_top` | Method | — | `barge_capacity.rs` line 622 | — |
| CG-A25 | Hold length | Full cargo zone, no transverse bulkhead length taken off | Method | — | `barge_capacity.rs` line 630 | Non-conservative; small |
| CG-A26 | Hold fill ratio | 0.95 | Estimate | Carried over from the original engine | `barge_capacity.rs` line 632; `builtin_profile.json` line 369 | Unknown |
| CG-A27 | Wing void width | 1.0 m per side | Estimate | Carried over from the original engine | `barge_capacity.rs` line 618; `builtin_profile.json` line 327 | Unknown; ADN 9.1.0.91.2 asks 0.80 m for a double-hull dry cargo vessel (not checked) |
| CG-A28 | Hold double bottom | 0.8 m | Estimate | Two published inland dry bulk barge designs (0.8 m); built barges range 0.38 to 1.0 m. 1.2 m before #597 | `barge_capacity.rs` line 617; `builtin_profile.json` line 334 | Unknown; ADN 9.1.0.91.3 asks at least 0.50 m (not checked) |
| CG-A29 | Hopper slope | 1.0 H:V | Estimate | Carried over from the original engine | `barge_capacity.rs` line 620; `builtin_profile.json` line 341 | Unknown |
| CG-A30 | Hatch coaming height | 1.0 m | Estimate | Carried over from the original engine | `barge_capacity.rs` lines 617, 662; `builtin_profile.json` line 348 | Unknown |
| CG-A31 | Number of holds | 3 (steel weight only) | Estimate | Carried over from the original engine | `barge_capacity.rs` line 643; `builtin_profile.json` line 355 | Small |
| CG-A32 | Plate thicknesses, sloped hopper | Inner bottom 14, hopper 14, bulkheads 11, coaming 10 mm | Estimate | Carried over from the original engine | `barge_capacity.rs` lines 635 to 638; `builtin_profile.json` lines 376 to 403 | Unknown |
| CG-A33 | Hopper steel method | Plating-area build-up (step 4) | Method | No reference | `barge_capacity.rs` lines 639 to 647 | Conservative (lightship high): see Evidence |
| CG-A34 | Steel density, hopper | 7.85 t/m³ | Fixed in code | Standard steel value | `barge_capacity.rs` line 250 | — |
| CG-A35 | Hopper structure factor | 1.40 | Fixed in code | None | `barge_capacity.rs` line 251 | Unknown |
| CG-A36 | Top-down hull steel rate, hopper | 0.10 t/m³ of LOA × B × D | Fixed in code | None | `barge_capacity.rs` lines 252, 649 | Unknown |
| CG-A37 | Piping and outfitting, hopper | 0.04 and 0.035 of steel | Fixed in code | None | `barge_capacity.rs` lines 253 to 254, 651 | Unknown |
| CG-A38 | Box hold lightship factor | k = 0.1093 t/m³ × LOA × B × D | Estimate | Fit to 3 published pushed box barges, range 0.095 to 0.1223 (`calibration/factors.yaml`) | `barge_capacity.rs` line 659; `builtin_profile.json` line 81 | Unknown; in-sample |
| CG-A39 | Container hold lightship factor | k = 0.1711 t/m³ | Estimate | One calibration case; not reachable from a Design | `calibration/factors.yaml`; `barge_capacity.rs` lines 652 to 660 | No effect today |
| CG-A40 | Stern block mass | 0 t in the lightship | Fixed in code | — | `barge_capacity.rs` lines 861 to 867 | Non-conservative for a self-propelled dry hull (cargo and deadweight high) |
| CG-A41 | Bow and stern void fractions | 0.10 and 0.15; not read in the capacity path | Fixed in code | — | `barge_capacity.rs` lines 244 to 245 | No effect |
| CG-A42 | Gas filling limit | 0.98 of tank volume | Sourced default | IGC Code, MSC.370(93), 15.3 | `builtin_profile.json` lines 95 to 112; `simulate.rs` line 814 | Non-conservative if the loading limit (15.5.1) is lower |
| CG-A43 | Gas loading limit | Not applied | Fixed in code | IGC 15.5.1 not coded | — | Non-conservative |
| CG-A44 | Gas cargo volume | Inner tank volume × fill; dome not in cargo | Method | — | `tanks.rs` lines 205 to 211 | — |
| CG-A45 | Capacity hull | Box `LOA × B × D`, no rakes, × Cb | Method | — | `barge_capacity.rs` lines 889, 905; `hull.rs` lines 124 to 132 | — |
| CG-A46 | Block coefficient | 0.85 | Estimate | Profile basis: calibrated box barges 0.883 to 0.995, widened down for gas hulls | `builtin_profile.json` line 53 | Conservative for capacity of a box barge (fit is 0.928); non-conservative for light air draft |
| CG-A47 | Water density | 1.0 t/m³ | Estimate | Fresh water, as the original engine | `builtin_profile.json` line 634 | Conservative in salt water |
| CG-A48 | Draught solver | Bisection 0 to 1.5 D, tolerance 0.001 × B on volume, 100 rounds | Fixed in code | — | `hydrostatics.rs` lines 21 to 56 | Small |
| CG-A49 | Draught clamp | `T ≤ D` | Method | Physical | `barge_capacity.rs` line 904 | — |
| CG-A50 | Overloaded | Required volume > box volume at D | Method (OVCD Check) | Physical | `barge_capacity.rs` lines 895 to 901 | — |
| CG-A51 | Minimum freeboard | 0.3 m | Estimate | No source in the profile | `simulate.rs` lines 682, 732; `builtin_profile.json` line 60 | Unknown; see page 08 |
| CG-A52 | Maximum draught | Least of `D − freeboard`, draught limit, channel depth − UKC | Method | — | `simulate.rs` lines 730 to 739 | — |
| CG-A53 | Weight-bound cargo | `max(0, Cb LOA B T_max ρ_w − lightship)` | Method | — | `simulate.rs` lines 741 to 742 | — |
| CG-A54 | Cargo carried | `min(volume-bound, weight-bound)` | Method | — | `simulate.rs` line 744 | — |
| CG-A55 | Deadweight | `displacement − lightship` = cargo; no stores, fuel or crew | Method | — | `simulate.rs` `results` block; `barge_capacity.rs` line 1032 | Non-conservative for cargo (cargo equals deadweight) |
| CG-A56 | Gas payload | Journey derating: volume, or `Δ(T_max) − lightship` | Method | — | `journey_derating.rs` lines 115 to 135 | — |
| CG-A57 | Volume utilisation floor | 0.5, warning | Fixed in code | None | `journey_derating.rs` lines 51, 160 to 167 | Advisory only |
| CG-A58 | Dry cargo top | `D + coaming` | Method | Cargo not above the coaming | `barge_capacity.rs` line 662 | Non-conservative for a heaped cargo |
| CG-A59 | Air draft check | Light (or ballasted light) condition governs, not laden | Method (OVCD Check) | — | `barge_capacity.rs` lines 971 to 993; `simulate.rs` lines 383 to 392 | — |
| CG-A60 | Ballast | 0 t; no Design input | Fixed in code | — | `barge_capacity.rs` line 212 | Conservative for the air draft check |
| CG-A61 | Bulk GM method | `KB + BM_t − KG`, box hull, volume-bound draught | Method | No reference | `barge_capacity.rs` lines 685 to 712 | Unknown |
| CG-A62 | Lightship KG | 0.5 × D | Fixed in code | None | `barge_capacity.rs` line 704 | Unknown |
| CG-A63 | Cargo KG | `DB + z_c` of the full hold section | Method | Hold taken as full | `barge_capacity.rs` lines 698 to 703 | Conservative when the hold is not full |
| CG-A64 | Bulk GM limit | 0.15 m, warning only | Fixed in code | Equals IS Code 2008 Part A 2.2.4 (MSC.267(85)); not cited in code | `barge_capacity.rs` line 1006 | Non-conservative: it never blocks |
| CG-A65 | When bulk GM runs | Only if not overloaded at the volume-bound load | Fixed in code | — | `barge_capacity.rs` line 1002 | Non-conservative: dense cargo gets no GM check |
| CG-A66 | Cargo shift | Not modelled; fixed warning | Fixed in code | — | `barge_capacity.rs` lines 814 to 815 | Non-conservative |
| CG-A67 | Free surface liquid height | `fill × 2r` (volume fraction used as height fraction) | Method | No reference | `tanks.rs` line 280 | Non-conservative (API only) |
| CG-A68 | Free surface shape | Rectangle `chord × L_cyl`; heads left out | Method | No reference | `tanks.rs` line 292 | Non-conservative; small (API only) |
| CG-A69 | Check severity | Only `"error"` gates | Fixed in code | — | `checks.rs` lines 41 to 49 | — |
| CG-A70 | Feasible geometry | Cargo volume > 0, error | Method (OVCD Check) | Physical | `barge_capacity.rs` line 934 | — |
| CG-A71 | Impossible hold | Hold width or height < 0 refused | Method (OVCD Check) | Physical | `vdesign.rs` lines 2806 to 2860 | — |
| CG-A72 | Dry checks dropped in Design path | `not_overloaded` and `freeboard_ok` removed | Fixed in code | Weight-bound cargo replaces them | `simulate.rs` lines 368 to 381 | — |
| CG-A73 | Liquid cargo other than gas | Not modelled | — | — | — | Gap |
| CG-A74 | Container stowage (TEU, tiers, weights) | Not modelled; TEU dropped from the core table | — | — | `vessel_classes.rs` lines 25 to 46 | Gap |

## Inputs the user can change

All in `builtin_profile.json`. "Carried over" means the value of the original engine, with no published source.

| Input | Default | Unit | Applies to | Where the default comes from |
|---|---|---|---|---|
| `cargo` | (required) | — | all | One of six keys in `cargo.rs` |
| `cargo_containment` | dry bulk: `box_hold`; gas: `type_c_tanks` | — | all | Estimate (line 50) |
| `moulded_depth_m` | ratio × beam (0.33 dry, 0.55 gas) | m | all | `depth_to_beam_ratio`, lines 24 to 30 |
| `block_coefficient` | 0.85 | — | all | Lines 52 to 58 |
| `min_freeboard_m` | 0.3 | m | all | Lines 59 to 65, no source |
| `water_density_t_per_m3` | 1.0 | t/m³ | all | Lines 633 to 639 |
| `bow_rake_length_to_depth_ratio` | 1.6 | — | all | Carried over |
| `pushed_stern_void_m` | 3.0 | m | pushed | Carried over |
| `draught_limit_m`, `air_draft_limit_m` | none | m | all | — |
| `lightship_factor_t_per_m3` | 0.1093 | t/m³ | box hold | `calibration/factors.yaml` |
| `hold_wing_void_width_m` | 1.0 | m | both holds | Carried over |
| `hold_double_bottom_m` | 0.8 | m | both holds | Estimate from built barges |
| `hatch_coaming_height_m` | 1.0 | m | both holds | Carried over |
| `bulk_end_bulkhead_m` | 0.5 | m | both holds | Carried over |
| `hold_fill_ratio` | 0.95 | — | both holds | Carried over |
| `hopper_slope_h_per_v` | 1.0 | H:V | sloped hopper | Carried over |
| `n_holds` | 3 | holds | sloped hopper | Carried over |
| `inner_bottom_plate_mm`, `hopper_plate_mm`, `hold_bulkhead_plate_mm`, `hatch_coaming_plate_mm` | 14, 14, 11, 10 | mm | sloped hopper | Carried over |
| `fill_ratio` | 0.98 | — | Type C tanks | IGC 15.3 |
| `battery_unit_count`, `accommodation_unit_count` | by phase (dry 2/1, gas 1/2) | units | self-propelled | Carried over |

The `vdesign.rs` tables `LEGACY_FIXED_INPUTS` (lines 158 to 184 and 244 onward) hold the same dry hold values for old
files, so an old file keeps its results.

## Limits and known findings

- **No liquid cargo other than liquefied gas, and no container cargo.** See "What OVCD computes here". The catalogue
  carries `cargo_modes` such as `liquid` and `container` and TEU figures, but no calculation uses them.
- **The dense dry bulk case gets no stability check.** For bauxite in a box hold, the volume-bound load is often heavier
  than the hull can float. Hand calculation for the default box hold on LOA 100 m, B 16 m, D 6 m, self-propelled
  (cargo zone 69.5 m as in the golden test): `V_c = 14 × 5.8 × 69.5 × 0.95 ≈ 5,360 m³`, cargo `≈ 8,040 t`, lightship
  `≈ 1,050 t`, total `≈ 9,090 t`, against `0.85 × 9,600 = 8,160 t` at D. The load is overloaded, so `gm_positive` does
  not run (CG-A65). The Design path then carries the weight-bound cargo (`≈ 6,700 t`) with no GM figure.
- **The bulk GM, when it runs, is for the volume-bound load,** not for the cargo the Design reports. (Fixed in the Design results by #415.)
- **The LPG density is the fully refrigerated value for every carriage condition** (CG-A03, CG-A07). The default
  carriage condition is fully pressurised (`builtin_profile.json` line 92). The code's own comment puts propane at
  ambient temperature at about 489 to 493 kg/m³, so a pressurised LPG cargo mass reads about 16 % high. That comment's
  sources were not read in this review.
- **Self-propelled dry hulls carry no stern block mass** (CG-A40). The box k factor is fitted to pushed barges only.
- **The reported deadweight is the cargo** (CG-A55). Stores, fuel, water and crew are not in it.
- **The sloped hopper section is a V over the full hold depth,** coaming included. With the defaults on B = 16 m the
  hold bottom is 2.4 m wide.
- **No double hull rule is checked.** The wing void and double bottom are inputs only. ADN 2025 has double-hull
  minimum sizes for dry cargo vessels (9.1.0.91.2, 9.1.0.91.3) and tank vessels (9.3.2.11.7, 9.3.2.11.8). None
  applies by default, and no ADN Rule Set exists.
- **The free surface formula of `tanks.rs` uses the volume fill as a height fraction** (CG-A67). At 98 % fill it gives
  about a quarter of the true moment for a circular section. It reaches only the Python API, not a Design.
- Findings register (#121): none filed for this area yet. Proposed findings are in the engineering basis triage tickets.

## Evidence

- **Calibration (`calibration/results.md`):**
  - Box hold lightship (k × LOA × B × D): 3 cases inside their bands, in-sample (section 1).
  - Deadweight at a stated draught, box barges: 9 of 10 in-sample cases inside ±10 % (section 2). These use the
    fitted Cb 0.928, not the profile default 0.85.
  - Held-out box barge deadweight cases: most inside their bands (section 4). Deck and pontoon barges run as box
    barges miss the lightship band often (section 4).
  - The sloped hopper plating build-up ("Ported" column, section 1) gives 84 % to 111 % more lightship than the
    published box barges. These are box barges, so this shows the build-up's level, not its error on a hopper.
  - Seven double-skin tank barges cannot run: "no tank hull model in the calibration path" (section 4, last table).
- **Unit and golden tests:** `tests/core/test_barge_capacity.py`: `test_bauxite_golden_example` (sloped hopper,
  4,711 t, draught 4.60 m, snapshot), `test_cargo_volume_is_inner_volume_times_fill`, `test_wing_void_widens_cuts_bulk_volume`,
  `test_bulk_gm_computed_and_positive`, `test_air_draft_check_gates_on_light_not_laden` and others.
  `crates/ovcd-format/tests/dry_hold_inputs.rs`: each dry hold input reaches the core, and the impossible hold check.
  `crates/ovcd-core/tests/barge_capacity.rs`: parity with the Python reference.
- **Evaluation harness (`evaluation/results.md`):** covers IS Code Part A 2.2 on box sections (page 03). It does not
  cover the bulk GM warning.
- No case checks bauxite density, hold fill or the hopper geometry against a built hopper barge.

## Questions for reviewers

1. Is 1,500 kg/m³ a fair single bauxite stowage density for inland barges? Which other dry bulk Cargos (coal, grain,
   aggregates, ore) should come first, and with which stowage factors?
2. Is a 0.95 hold fill a fair default for a box hold loaded by grab or conveyor, with or without trimming?
3. The sloped hopper runs from the coaming top to a 1.0 m minimum bottom. What hopper geometry (hopper height, slope)
   does a typical inland hopper barge have?
4. Should the hold double bottom default be 1.2 m and the wing void 1.0 m for a box barge, or nearer the ADN
   9.1.0.91 minimums (0.50 m and 0.80 m)?
5. Is `k = 0.1093 t/m³ × LOA × B × D` fair for a self-propelled dry bulk hull if the stern block is added on top?
   What mass should the stern block carry?
6. For a box barge, should the default Cb be the calibrated 0.928 rather than 0.85?
7. Should the bulk GM warning become blocking at 0.15 m (IS Code 2008 Part A 2.2.4), and should it run at the
   cargo the Design reports?
8. For container barges: which method would you trust for TEU at concept stage (bays × rows × tiers from the hold
   size, or stack weight limits)? Which stability rule should govern (ES-TRIN 2025/1 chapter 27, IS Code Part B 2.2
   for pontoons)?
9. For liquid tank barges: should the first liquid Cargo be a product (density about 800 to 900 kg/m³) in integral
   tanks, with the ADN 9.3.2.11.1 tank capacity limit and the 9.3.2.11.7 double hull?
10. Should the deadweight carry a stores and consumables allowance, so cargo and deadweight differ?
