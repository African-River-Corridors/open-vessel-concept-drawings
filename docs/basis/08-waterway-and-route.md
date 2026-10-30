# Waterway and route

> Concept and preliminary design only. Not for approval. Draft: needs expert review.
>
> Code state: written against `main` at commit `f6d446c` (2026-10-20).

**For:** Inland waterway engineer  ·  **Code:** `crates/ovcd-core/src/squat.rs`, `waves.rs`,
`journey_derating.rs`, `barge_capacity.rs`, `gas_arrangement.rs`, `resistance.rs` (channel parts),
`vessel_classes.rs`; `crates/ovcd-format/src/simulate.rs`, `run.rs`, `class_input.rs`, `builtin_profile.json`
·  **Rule Sets:** none yet. Every check on this page is an OVCD Check or an unverified method.

This page tells you how OVCD handles the water a vessel operates in: where it operates (Service Area), how deep
it may float, how high it may stand, how much it squats, what waves add, and how much freeboard it keeps. Every
statement points to the code that does it. Proposed changes are in a separate file, not here.

## What OVCD computes here

For one Option (the `simulate` path, `crates/ovcd-format/src/simulate.rs`):

- `draught_max_m`: the draught cap, the least of the hull limit, the typed draught limit and the channel limit.
- `governs`: what limits the cargo, the hold or tank volume or the draught cap (with the name of the limit).
- `cargo_t`, `cargo_volume_bound_t`, `cargo_weight_bound_t`: the cargo per journey, and the two bounds.
- `draught_m`, `freeboard_m`: the draught and the freeboard at the carried cargo.
- `light_draught_m`, `air_draft_light_m`, `air_draft_loaded_m`: the light draught and the air draft, light and loaded.
- `h_over_T`: channel depth over draught, when a channel depth is given.
- Checks: `Cargo > 0`, the capacity checks (with the air draft check), `Air draft (empty, governing)`, and for gas,
  `Feasible arrangement` and the arrangement search checks (with `freeboard_at_payload`).
- `curves`: speed against power for "Deep water" and, when the channel has a depth and a bottom width, "Channel".
  Each point shows `squat_m` and `squat_limited`. Each curve shows `limiting_speed_kn` (Schijf) and `blockage`.

Through the evaluation API only (`crates/ovcd-format/src/eval.rs`, not from a Design):

- `squat.compute`: squat by two methods, static and dynamic under-keel clearance, and the method checks.
- `squat.max_speed_for_ukc`: the highest speed that keeps a margin under the keel.
- `waves.compute`: added resistance in head waves (STAwave-1) and its validity checks.
- `resistance.compute` with wave height, wave heading and wave period per environment.

## How it works today

### 1. Service Area

1. **There is no Service Area input in the code yet.** No file, function or field named `service_area` exists in
   `crates/`. Spec 0004 (section "Service Area and default stacks") defines it: `river-only` (the Defaults
   Profile default) or `coastal-and-river`. `ocean-only` is not offered (ADR 0011). Label: **Fixed in code**
   (absent).
2. Today every Option runs the same checks, whatever its waters. Spec 0004 says the engine uses the `river-only`
   stack for every Option until `.vdesign` format version 3 (#274). Both stacks hold the same sets in that spec.
3. So the Service Area selects nothing today. The target stacks in spec 0004 add ES-TRIN 2025/1 Chapter 4
   (river only, as a reference) and ICLL freeboard (coastal and river) in later specs.

### 2. The river or route model

1. A route is **one channel section** with one depth, one bottom width, one bank slope, one current and one water
   density. There is no list of legs, no water level series, no locks and no bends. Inputs:
   `channel_depth_m`, `channel_under_keel_clearance_m`, `channel_bottom_width_m`, `channel_bank_slope`,
   `channel_current_kn`, `water_density_t_per_m3` (`crates/ovcd-format/src/run.rs`, `simulate_params_of`).
2. The channel is a trapezoid. Surface width b_s = b + 2 · n · h, with b the bottom width, n the bank slope
   (H:V) and h the depth (`simulate.rs`, `env`, line 211). Section area A_c = (b + b_s) / 2 · h
   (`resistance.rs`, `channel_cross_section`, line 360). Label: **Method**.
3. The channel goes to the engine only when it has a depth (`run.rs`, `simulate_params_of`). It gets a resistance
   and squat curve only when it also has a bottom width (`simulate.rs`, `curves`, line 275). With a depth and no
   width, the draught cap applies but no squat or speed check runs.
4. A second environment, "Deep water", is always added: depth 50 m, width 1000 m, bank slope 3, no current,
   no squat (`simulate.rs`, `curves`, line 291). Label: **Fixed in code**.
5. The current is always opposing. Speed through water = speed over ground + current
   (`resistance.rs`, `compute_speed_point`, line 446). Label: **Fixed in code**.
6. One water density applies to the draught, the stability and the resistance (`simulate.rs`, `simulate`; profile
   entry `water_density_t_per_m3`).

### 3. Draught limits and the draught cap

1. The simulator builds a list of limits and takes the least (`simulate.rs`, `min_item`, line 135):
   - Hull: moulded depth D − minimum freeboard (line 730 dry, line 848 gas).
   - Draught limit: `draught_limit_m`, when given.
   - Channel: channel depth − under-keel clearance (`channel_limit`, line 252). The clearance falls back to 0.5 m.
   T_max = min of these. Label: **Method**.
2. A Vessel Class fills `draught_limit_m` with the **top** of its draught range, unless the user typed a limit
   (`vessel_classes.rs`, `draught_limit_m`, line 78; `class_input.rs`). Label: **Fixed in code**.
3. **Dry cargo** (`simulate.rs`, `simulate`, lines 728 to 751):
   - Weight-bound cargo = max(0, C_b · L · B · T_max · ρ − lightship).
   - Cargo = min(volume-bound cargo, weight-bound cargo). The smaller one `governs`.
   - Draught T = (lightship + cargo) / (C_b · L · B · ρ). Label: **Method**.
4. **Gas cargo** (`simulate.rs`, `simulate_gas`, lines 845 to 950): T_max goes to the arrangement search
   (`gas_arrangement.rs`, `fit`). Each candidate is de-rated by `journey_derating::compute` with design depth
   = T_max and UKC = 0 (`gas_arrangement.rs`, `derate_candidate`, line 410).
5. **Journey de-rating** (`journey_derating.rs`, `compute`, line 105):
   - T_max = max(0, design depth − UKC).
   - If T_max ≥ full-load draught, the binding limit is "volume" and the full cargo goes.
   - If not, the binding limit is "draft". Payload = max(0, min(volume capacity,
     C_b · V_box(min(T_max, D)) · ρ − lightship)). The draught at payload is T_max.
   - V_box is the box-hull volume of `HullGeometry` with zero rakes. Label: **Method**.
6. **Depth search** (gas, `optimise_depth` true, `simulate.rs` line 856): the search keeps the default depths
   3.5 m to 7.0 m in 0.25 m steps (`gas_arrangement.rs`, `DEFAULT_DEPTHS_M`) where depth − freeboard ≥ cap. If none
   is left, it uses 7.0 m. With no waterway limit, the cap is 7.0 m − freeboard. Label: **Fixed in code**.

### 4. Squat and under-keel clearance

1. **Static under-keel clearance** = water depth − draught (`squat.rs`, `compute`, line 170). Dynamic = static −
   squat. Label: **Method**.
2. **Barrass II**: S = C_b · s^0.81 · V_k^2.08 / 20 [m], with s the blockage and V_k the speed through water in
   knots (`squat.rs`, `barrass`, line 107). It returns 0 when the blockage is 0 (open water) or the speed is 0.
   Label: **Method**, unverified (no source in the library).
3. **ICORELS**: S = 2.4 · (∇ / L_pp²) · F_nh² / √(1 − F_nh²), with F_nh = V / √(g · h) (`squat.rs`, `icorels`,
   line 115). Not evaluated at F_nh ≥ 0.99. Label: **Method**, unverified.
4. Blockage for squat s = B · T / A_c (`squat.rs`, line 142). ∇ = C_b · L_pp · B · T when no displacement is
   given (line 137). On the Design path L_pp is the length overall (`simulate.rs`, `curves`: `lwl: l`).
5. The adopted squat is the larger of the two; Barrass wins a tie (`squat.rs`, line 162).
6. Squat runs only in an environment that is not deep water and has a depth above 0 (`resistance.rs`,
   `squat_applies`, line 294). It runs at the loaded draught of the Option only.
7. A speed is "squat limited" when the dynamic clearance is less than the squat safety margin, 0.15 m by default
   (`resistance.rs`, line 526). A squat-limited speed is not a service speed (`compute_environment`).
8. The squat-limited speed comes from a bisection between 0 and 20 kn, to 0.01 kn (`squat.rs`,
   `max_speed_for_ukc`, line 274; called at `resistance.rs` line 608). It returns 0 when the static clearance is
   at or below the margin.
9. Squat does **not** change the draught cap. The cap uses the static clearance only (step 3.1). Squat limits the
   speed.
10. The squat module makes five checks (`squat.rs`, lines 178 to 238): `keel_clears_bed` (error),
    `keel_clears_bed_with_margin`, `froude_depth_subcritical` (F_nh < 0.70), `blockage_within_barrass_range`
    (s ≤ 0.35) and `methods_agree_within_50pct` (warnings). On the Design path these checks are dropped: the
    resistance code keeps only the squat value and the dynamic clearance (`resistance.rs`, line 523).

### 5. Speed limit in a channel

1. **Limiting speed**: F_nh,crit = √(2/3) · (1 − m)^(1/3), V_L = F_nh,crit · √(g · h), with m = A_m / A_c
   (`resistance.rs`, `schijf_limiting_speed`, line 370). The code names it Schijf (1949). Label: **Method**,
   unverified. see the triage tickets: this form does not tend to F_nh = 1 when m tends to 0.
2. Midship area here A_m = B · T · C_b (`resistance.rs`, `midship_section_area`, line 365). This differs from the
   squat blockage (B · T).
3. The speed sweep stops at 0.95 · V_L (`resistance.rs`, line 580). "channel" governs the service speed when the
   service speed ≥ 0.95 · V_L − 0.05 kn (line 632).
4. The shallow-water resistance factor (Lackenby term and a blockage term) is on page 07. In short:
   1 + 1.4 · (T/h)^1.5 · F_nh^2.5 + 2 · m · F_nh², with the Lackenby term set to 5.0 at F_nh ≥ 0.95
   (`resistance.rs`, `shallow_water_factor`, line 386).

### 6. Waves and wind

1. **STAwave-1**: R_AW = (1/16) · ρ · g · H_s² · B · √(B / L_BWL) [kN after ÷ 1000] (`waves.rs`, `compute`,
   line 119). Label: **Method**, unverified.
2. L_BWL defaults to 0.2 · L_pp (line 108). Heading factor = cos²(μ), set to 0 when cos μ ≤ 1e-12 (line 125).
3. The peak period defaults to T_p = 4.0 · √H_s (`peak_period`, line 94). The wavelength is the deep-water value
   λ = g · T_p² / (2π) (line 130). These set only the validity check, not the resistance.
4. Checks (all warnings): λ/L_pp < 0.5, H_s ≤ 3.0 m, bow length given. A warning for heading > 60°.
5. Added power = R_AW · V (effective power, `waves.rs` line 141).
6. **Waves do not reach a Design.** The simulator never sets a wave height. Every environment on the Design path
   has H_s = 0 (`resistance.rs`, `EnvironmentInputs::default`). Waves work only through `eval`.
7. **There is no wind model.** No wind load, no wind heel, no windage area. `gas_stability.rs` (line 795) states
   that there is no wind heel.

### 7. Air draft and bridge clearance

1. Air draft = height of the top point above the keel − draught (`barge_capacity.rs`, lines 907, 912, 926).
2. Top point:
   - Gas: double bottom + tank support height + tank outer diameter + dome height (`barge_capacity.rs`, line 573).
   - Dry: moulded depth + hatch coaming height (line 662).
   - Self-propelled: the higher of the cargo top and the stern block with the wheelhouse lowered (line 879).
3. The navigating air draft (wheelhouse raised) is reported as `air_draft_navigating_light_m` (line 927). No check
   uses it.
4. The bridge check is on the **light** condition, or the ballasted light condition when ballast is given:
   `air_draft_within_limit` (error, `barge_capacity.rs`, line 970). The simulator adds a second row,
   `Air draft (empty, governing)`, on the light condition (`simulate.rs`, `air_draft_check`, line 383).
5. The limit is compared as it is: no clearance margin, no water level allowance.
6. A Vessel Class fills `air_draft_limit_m` with the **bottom** of its air-draft range, the lowest bridge
   clearance (`vessel_classes.rs`, `air_draft_limit_m`, line 84; `class_input.rs`).
7. The gas-fuelled stern block leaves the exhaust uptake and the vent mast out of the air draft, as lowerable
   (`aft_arrangement.rs`, warning text near line 940).

### 8. Freeboard

1. Freeboard = moulded depth − draught, at the side, with no sheer, no camber and no trim (`barge_capacity.rs`,
   line 906; `simulate.rs`, `freeboard_m`).
2. **No rule sets the freeboard.** The minimum freeboard is one input, `min_freeboard_m`, default 0.3 m, with no
   published source (`builtin_profile.json`, `min_freeboard_m`; `barge_capacity.rs` line 210;
   `gas_arrangement.rs` `DEFAULT_REQUIRED_FREEBOARD_M`, line 56; `simulate.rs` lines 682 and 845). Label:
   **Estimate**.
3. It acts in three places:
   - As a draught limit: D − freeboard (step 3.1).
   - `freeboard_ok` (error) in the capacity result, at the volume-bound full-load draught
     (`barge_capacity.rs`, line 941). The simulator hides this row (`capacity_checks`, line 368), but the gas
     search still gates on it, with a fixed 0.3 m (see the triage tickets).
   - `freeboard_at_payload` (error) in the gas search, at the carried draught, with a 1e-9 m tolerance
     (`gas_arrangement.rs`, `cheap_checks`, line 297).
4. Input checks: minimum freeboard < moulded depth (hard); with `optimise_depth`, minimum freeboard ≤ 7.0 m
   (hard) (`vdesign.rs`, near line 2721 and 2768).
5. On a coastal Service Area, the Rule Set `icll-freeboard` 2.0.0 checks the freeboard at the laden draught
   (Measure `freeboard.laden`, `D − T` of the result, ICLL regulation 3(8)) against the type A table 28.1 at the
   length overall (issue #659). It does not read `min_freeboard_m`. Version 1.0.0 compared `min_freeboard_m`, the
   safety clearance, with the table, so every coastal Option at the 0.3 m default failed whatever its real
   freeboard; it is kept unchanged for files that store it.

## Assumptions, rules and estimates

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| WR-A01 | Service Area | No input; every Option gets the same checks | Fixed in code | Spec 0004 plans `river-only` / `coastal-and-river` | Not in `crates/` | Non-conservative for coastal service (no coastal rules) |
| WR-A02 | Water density | 1.0 t/m³ | Estimate | Profile: "fresh water, as the original engine" | `builtin_profile.json`; `gas_stability.rs` `DEFAULT_WATER_DENSITY` l.581 | Non-conservative in salt water (draught under-estimated by about 2.5 %) |
| WR-A03 | One density for draught, stability, resistance | One value per run | Fixed in code | Design choice | `simulate.rs` `simulate` | Unknown for estuary routes |
| WR-A04 | Channel section | Trapezoid, A_c = (b + b_s)/2 · h, b_s = b + 2nh | Method | Geometry | `simulate.rs` `env` l.211; `resistance.rs` l.360 | Unknown; real sections are not trapezoids |
| WR-A05 | Bank slope | 3.0 H:V | Estimate | No basis given | Profile `channel_bank_slope`; `simulate.rs` l.309 | Unknown |
| WR-A06 | Current | 0 kn, always opposing; STW = SOG + current | Fixed in code | Design choice | Profile `channel_current_kn`; `resistance.rs` l.446 | Conservative for squat; downstream leg not modelled |
| WR-A07 | Channel under-keel clearance | 0.5 m | Estimate | No basis given | Profile `channel_under_keel_clearance_m`; `simulate.rs` `channel_limit` l.256 | Unknown; depends on bed type and authority |
| WR-A08 | Draught cap | T_max = min(D − freeboard, draught limit, channel depth − UKC) | Method | Design choice | `simulate.rs` l.730–739, l.848–887 | Correct form; values are inputs |
| WR-A09 | Class draught limit | Top of the class draught range | Fixed in code | Catalogue `vessel_classes.yaml` | `vessel_classes.rs` l.78; `class_input.rs` | Non-conservative where the waterway has less depth than the class top |
| WR-A10 | Class air-draft limit | Bottom of the class air-draft range | Fixed in code | Catalogue | `vessel_classes.rs` l.84; `class_input.rs` | Conservative |
| WR-A11 | Deep-water environment on the Design path | h 50 m, b 1000 m, n 3, no current, no squat | Fixed in code | Design choice | `simulate.rs` `curves` l.291 | Small for inland hulls |
| WR-A12 | Default environments (API only) | Open sea 50 m / 1000 m / ρ 1.025; river 6.5 m / 40–60 m / 1.0 kn; creek 8 m / 60–100 m / 0.5 kn | Fixed in code | Examples from the original engine | `resistance.rs` `default_environments` l.108 | Not used by a Design |
| WR-A13 | Barrass II squat | S = C_b · s^0.81 · V_k^2.08 / 20 | Method | Barrass, per code notes; not in the library | `squat.rs` `barrass` l.107 | Unknown until verified |
| WR-A14 | Barrass in open water | 0 when s = 0 | Fixed in code | Form needs a blockage | `squat.rs` l.108 | Non-conservative with no channel area (ICORELS only) |
| WR-A15 | ICORELS squat | S = 2.4 · ∇/L_pp² · F_nh²/√(1 − F_nh²) | Method | PIANC, per code notes; not in the library | `squat.rs` `icorels` l.115 | Unknown until verified |
| WR-A16 | ICORELS cut-off | None at F_nh ≥ 0.99 | Fixed in code | Singular at 1 | `squat.rs` l.119 | Small; flow is critical there |
| WR-A17 | Adopted squat | Larger of the two; Barrass on a tie | Fixed in code | Design choice | `squat.rs` l.162 | Conservative |
| WR-A18 | Squat blockage area | A_m = B · T (no midship coefficient) | Fixed in code | Design choice | `squat.rs` l.142 | Slightly conservative |
| WR-A19 | Squat volume and length | ∇ = C_b · L · B · T; L = length overall on the Design path | Fixed in code | `simulate.rs` `curves` sets `lwl: l` | `squat.rs` l.137 | Small |
| WR-A20 | Squat C_b range | 0.3 to 1.0 (hard); default 0.85 | Fixed in code | Profile: "squat method's validity floor" | `squat.rs` `validate` l.75 | Unknown |
| WR-A21 | Froude depth validity | F_nh < 0.70 (warning) | Fixed in code | Code notes | `squat.rs` `FNH_VALID_MAX` l.26 | Unknown; warning lost on the Design path |
| WR-A22 | Barrass blockage validity | s ≤ 0.35 (warning) | Fixed in code | Code notes | `squat.rs` l.215 | Unknown; warning lost on the Design path |
| WR-A23 | Method spread | ≤ 50 % (warning) | Fixed in code | Design choice | `squat.rs` l.229 | None on results |
| WR-A24 | Squat safety margin | 0.15 m below the squatted keel | Estimate | Profile: "carried over from the original engine" | Profile `squat_safety_margin_m`; `squat.rs` l.23; `resistance.rs` l.101 | Unknown |
| WR-A25 | Gravity and knot | 9.80665 m/s², 1852/3600 m/s (resistance, squat, waves) | Fixed in code | One definition (issue #526) | `constants.rs` | Negligible |
| WR-A26 | Squat-limited speed search | Bisection, 0 to 20 kn, tolerance 0.01 kn | Fixed in code | Design choice | `resistance.rs` l.608–617 | Small; above 20 kn not searched |
| WR-A27 | Where squat applies | Not deep water, depth > 0 | Fixed in code | Design choice | `resistance.rs` `squat_applies` l.294 | None |
| WR-A28 | Draught for squat | Loaded draught of the Option only | Fixed in code | Design choice | `simulate.rs` `curves` | Non-conservative for trim by the bow |
| WR-A29 | Limiting speed | F_nh,crit = √(2/3) · (1 − m)^(1/3) | Method | Named Schijf (1949) in code; not in the library | `resistance.rs` `schijf_limiting_speed` l.370 | Non-conservative for m above about 0.03 (see the triage tickets) |
| WR-A30 | Limiting-speed midship area | A_m = B · T · C_b | Fixed in code | Design choice | `resistance.rs` l.365 | Non-conservative (A_m too small for a barge) |
| WR-A31 | Speed cap | 0.95 · V_L | Fixed in code | Design choice | `resistance.rs` l.580 | Unknown |
| WR-A32 | "channel" governs | Service ≥ 0.95 · V_L − 0.05 kn | Fixed in code | Design choice | `resistance.rs` l.632 | Label only |
| WR-A33 | Shallow-water factor | 1 + 1.4 (T/h)^1.5 F_nh^2.5 + 2 m F_nh²; 5.0 at F_nh ≥ 0.95 | Method | Code: "uncalibrated first-order" | `resistance.rs` l.386; page 07 | Unknown |
| WR-A34 | STAwave-1 | R_AW = ρ g H_s² B √(B/L_BWL) / 16 | Method | ITTC, per code notes; not in the library | `waves.rs` l.119 | Non-conservative for λ/L_pp ≥ 0.5 (code says so) |
| WR-A35 | Bow length | L_BWL = 0.2 · L_pp | Estimate | Code: blunt barge bow | `waves.rs` l.108 | Conservative (per code text) |
| WR-A36 | Heading factor | cos² μ; 0 abeam | Fixed in code | OVCD's own screening treatment | `waves.rs` l.122–126 | Non-conservative off the bow (code calls it a floor) |
| WR-A37 | Peak period | T_p = 4.0 · √H_s | Estimate | Code: fetch-limited sea | `waves.rs` `peak_period` l.94 | Validity check only |
| WR-A38 | Wavelength | Deep water, λ = g T²/(2π) | Fixed in code | Linear wave theory | `waves.rs` l.130 | Validity check passes too easily in shallow water |
| WR-A39 | STAwave-1 validity | λ/L_pp < 0.5 (warning) | Method | ITTC, per code notes | `waves.rs` `LAMBDA_OVER_LPP_MAX` l.20 | Unknown |
| WR-A40 | Wave height screening | H_s ≤ 3.0 m (warning) | Fixed in code | Design choice | `waves.rs` l.161 | None on results |
| WR-A41 | Beam-sea warning | Heading > 60° | Fixed in code | Design choice | `waves.rs` l.186 | None on results |
| WR-A42 | Wave water density | 1.025 t/m³ default | Fixed in code | Sea water | `waves.rs` l.17 | Small |
| WR-A43 | Waves on the Design path | H_s = 0 always | Fixed in code | Simulator sets no wave height | `resistance.rs` l.488; `simulate.rs` | Non-conservative for coastal or estuary legs |
| WR-A44 | Wind | No wind load, no wind heel | Fixed in code | Not modelled | `gas_stability.rs` l.795 | Non-conservative for high-sided or light vessels |
| WR-A45 | Air draft | Top point above keel − draught | Method | Geometry | `barge_capacity.rs` l.907, 912, 926 | Correct form |
| WR-A46 | Top point, gas | Double bottom + support + tank OD + dome | Method | Geometry; see page 04 | `barge_capacity.rs` l.573 | Non-conservative if masts or rails stand higher |
| WR-A47 | Top point, dry | D + coaming height | Method | Geometry | `barge_capacity.rs` l.662 | Non-conservative for deck cargo or container tiers |
| WR-A48 | Top point, self-propelled | Max of cargo top and stern block, wheelhouse lowered | Method | Design choice | `barge_capacity.rs` l.879 | Correct for transit under a bridge |
| WR-A49 | Navigating air draft | Wheelhouse raised; reported, not checked | Fixed in code | Design choice | `barge_capacity.rs` l.927 | Unknown |
| WR-A50 | Governing bridge condition | Light, or ballasted light when ballast given | Fixed in code | Design choice | `barge_capacity.rs` l.970; `simulate.rs` l.383 | Non-conservative if a laden case stands higher (deck cargo) |
| WR-A51 | Masts and exhaust | Left out of the air draft, as lowerable | Estimate | Code warning text | `aft_arrangement.rs` near l.940 | Non-conservative if they cannot lower |
| WR-A52 | Bridge margin and water level | None: air draft ≤ limit as given | Fixed in code | Design choice | `barge_capacity.rs` l.984 | Non-conservative unless the limit already has a margin |
| WR-A53 | Freeboard definition | D − T at the side; no sheer, camber, trim | Method | Box hull | `barge_capacity.rs` l.906 | Unknown; real rules measure differently |
| WR-A54 | Minimum freeboard | 0.3 m | Estimate | No source; profile basis only "less than depth" | Profile `min_freeboard_m`; `barge_capacity.rs` l.210; `gas_arrangement.rs` l.56; `simulate.rs` l.682, 845 | Conservative against ES-TRIN 4.02(1) (150 mm); non-conservative against ICLL Table 28.2 for L > 36 m |
| WR-A55 | `freeboard_ok` | Freeboard ≥ required at volume full load (error) | Fixed in code | Design choice | `barge_capacity.rs` l.941 | Over-strict in the gas search (fixed 0.3 m) |
| WR-A56 | `freeboard_at_payload` | D − T_carried + 1e-9 ≥ required (error) | Fixed in code | OVCD Check | `gas_arrangement.rs` l.297–313 | Correct form |
| WR-A57 | Freeboard input limits | < D (hard); ≤ 7.0 m with depth search (hard) | Fixed in code | Physical limit | `vdesign.rs` near l.2721, 2768 | None |
| WR-A58 | De-rating cap | T_max = max(0, design depth − UKC) | Method | Design choice | `journey_derating.rs` l.112 | Correct form |
| WR-A59 | De-rated payload | min(capacity, C_b V_box(min(T_max, D)) ρ − LS), ≥ 0 | Method | Same box hydrostatics as capacity | `journey_derating.rs` l.122–131 | Unknown; inherits page 01 |
| WR-A60 | Binding limit | "volume" if T_max ≥ full-load draught, else "draft" | Fixed in code | Design choice | `journey_derating.rs` l.117 | None |
| WR-A61 | Utilisation floor | 0.5 (warning) | Estimate | Code: hull over-built below it | `journey_derating.rs` l.51 | None on results |
| WR-A62 | De-rating defaults | C_b 0.85, ρ 1.0, design depth 0, UKC 0; search passes UKC 0 with T_max already reduced | Fixed in code | Design choice | `journey_derating.rs` l.47–51; `gas_arrangement.rs` l.419–420 | None (clearance is in T_max) |
| WR-A63 | Dry weight-bound cargo | C_b · L · B · T_max · ρ − LS; T = Δ/(C_b L B ρ) | Method | Box hull | `simulate.rs` l.741, 751 | Unknown; inherits page 01 |
| WR-A64 | Depth search filter | Depth − freeboard ≥ cap; else 7.0 m | Fixed in code | Design choice | `simulate.rs` l.856–880; `vdesign.rs` `DEPTH_SEARCH_MAX_M` l.148 | None |
| WR-A65 | h/T output | Channel depth / loaded draught | Method | Ratio | `simulate.rs` `h_over_t` l.263 | None |
| WR-A66 | Old result fields | `min_freeboard_ok`, `air_draft_ok` always true; `total_depth_required` = D; `underkeel_clearance` 0.5 and `vertical_clearance` unused | Fixed in code | Port of the original engine | `ga_adapter.rs` l.241–248; `models.rs` l.30, 134, 215, 399–411 | Misleading labels only; no gate |

## Inputs the user can change

All from `crates/ovcd-format/src/builtin_profile.json` unless stated.

| Input | Default | Unit | Range (hard / soft) | Where the default comes from |
|---|---|---|---|---|
| `min_freeboard_m` | 0.3 | m | ≥ 0 / 0.15–1.5 | No source given |
| `draught_limit_m` | none | m | > 0, ≤ 100 / 0.8–12 | Vessel Class top of range, or typed |
| `air_draft_limit_m` | none | m | > 0, ≤ 1000 / 3–40 | Vessel Class bottom of range, or typed |
| `channel_depth_m` | none | m | 0.1–1000 / 1–20 | Typed only |
| `channel_under_keel_clearance_m` | 0.5 | m | 0–100 / 0.2–2 | No source given |
| `channel_bottom_width_m` | none | m | > 0 / 10–1000 | Typed only; needed for squat |
| `channel_bank_slope` | 3.0 | H:V | 0–100 / 1–10 | No source given |
| `channel_current_kn` | 0.0 | kn | 0–50 / 0–5 | No source given |
| `water_density_t_per_m3` | 1.0 | t/m³ | 0.9–1.1 / 0.99–1.03 | Fresh water |
| `squat_safety_margin_m` | 0.15 | m | 0–5 / 0.05–0.5 | Carried over from the original engine |
| `max_speed_kn` | 10.0 | kn | 1–100 / 2–20 | Top of the speed-power grid |
| `optimise_depth` | false | – | – | `vdesign.rs` |
| `vessel_scheme`, `vessel_class` | none | – | – | Fill the two limits (`class_input.rs`) |
| Wave height, heading, period (API only) | 0, 0°, 0 s | m, °, s | heading 0–180° | `resistance.rs` `EnvironmentInputs::default` |

## Limits and known findings

- No Service Area in the code. The spec defines it; the engine does not read it (#275, #274).
- One channel section per Option. No route legs, no water levels, no locks, no bends, no tidal windows.
- Squat: no trim split (bow and stern), no bank effect, no passing ships, no heel in bends, no squat in the light
  condition. The Python notes say so (`src/vessel_designer/core/squat.py`).
- Squat does not lower the draught cap. The static clearance sets the cap; squat only limits the speed.
- With a channel depth and no bottom width, no squat runs at all, and no warning says so.
- The squat and resistance validity checks (F_nh 0.70, blockage 0.35, Schijf 0.95, blockage 0 to 1) do not reach
  the Design result. The `simulate` curves show values only.
- Waves and wind do not reach a Design. STAwave-1 is for head seas and short waves only.
- Freeboard has no rule. 0.3 m is an OVCD default. ES-TRIN Chapter 4 and ICLL are target sets in spec 0004.
- Air draft has no clearance margin and no water level allowance. Deck cargo and container tiers are not in the
  top point.
- Old result fields in the GA payload say "ok" without a check (WR-A66).
- Spec 0004 lists the squat and wave methods among 43 items with no checkable source. Findings go to #121 after
  triage. See the findings register (#121) and the engineering basis triage tickets.

## Evidence

- `crates/ovcd-core/tests/squat.rs`: 10 tests. Expected values are what the original Python printed (parity),
  for example a CN II motor vessel, 87.3 × 14.8 × 4.4 m, in a 4.8 m channel of 209.5 m²: Barrass 0.69 m and
  ICORELS 0.34 m at 6 kn. They prove the port, not the method.
- `crates/ovcd-core/tests/waves.rs`: 4 tests, Python parity (for example 8.56 kN at H_s 1 m, B 14.8 m,
  L_pp 87.3 m).
- `crates/ovcd-core/tests/journey_derating.rs`: 10 tests (draft-bound, volume-bound, floats nothing, input
  refusals, water density).
- `crates/ovcd-core/tests/resistance.rs`: 28 tests, among them channel governs, blockage checks, wave heading,
  squat margin edge, current.
- `crates/ovcd-format/tests/environment_inputs.rs` (7) and `vessel_class.rs` (23): input mapping and class limits.
- `calibration/results.md`, section 4: deadweight at the stated draught. Box barges out of sample: 30 of 30
  inside the band, mean absolute error 4.1 %. This backs the draught-to-deadweight step that the draught cap uses.
- No squat, wave, limiting-speed or air-draft result is checked against measured data or an independent
  calculation.

## Questions for reviewers

1. Is 0.5 m a fair default static under-keel clearance for a concept study on a sand or silt river bed? Should it
   depend on draught or bed type?
2. Is 0.15 m a fair margin below the squatted keel, on top of the static clearance?
3. Is the larger of Barrass II and ICORELS the right choice for a full barge (C_b about 0.85) in a canalised river?
   Is ICORELS valid in a restricted channel at all?
4. Is s ≤ 0.35 the right upper validity limit for Barrass II, and F_nh < 0.70 for ICORELS?
5. Is √(2/3) · (1 − m)^(1/3) a form of the Schijf limit you recognise? We expected the limit to tend to
   F_nh = 1 as m tends to 0.
6. Should the limiting speed use the midship area B · T · C_m, not B · T · C_b?
7. Should squat reduce the draught cap (cap = depth − UKC − squat at the service speed), or is a speed limit enough
   for concept design?
8. For a Vessel Class, is the top of the CEMT draught range a fair draught limit, or should OVCD take the bottom?
9. Do the CEMT air-draft values already hold a safety clearance under the bridge? If not, what margin should OVCD
   add, and at which water level?
10. For river-only service, should the minimum freeboard follow ES-TRIN 2025/1 Art. 4.02 (150 mm for a flush deck)
    and the safety clearance Art. 4.01 (300 mm, 500 mm for open holds)?
11. For coastal-and-river service, is ICLL Reg. 28 (Table 28.2 for type B, with Reg. 27(14) for barges) the right
    freeboard basis, or a class rule for restricted service?
12. Should the bridge check also cover the navigating air draft with the wheelhouse raised?
13. Is STAwave-1 a fair screening method for an inland barge in estuary waves up to H_s 2.0 m (BV NR217, June
    2025, Pt A Ch 1 Sec 3 [12.2.3], range of navigation IN(x ≤ 2)), or is another method better for blunt bows?
