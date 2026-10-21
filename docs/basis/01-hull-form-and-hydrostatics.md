# Hull form and hydrostatics

> Concept and preliminary design only. Not for approval. Draft: needs expert review.
>
> Code state: written against `main` at commit `f6d446c` (2026-10-20).

**For:** naval architect  ·  **Code:** `crates/ovcd-core/src/hull.rs`, `hydrostatics.rs`, `barge_capacity.rs`,
`vessel_classes.rs`, `vessel_standards.rs`, `gas_stability.rs` (box hydrostatics only), `squat.rs` (hull-form terms
only); `crates/ovcd-format/src/class_input.rs`, `vdesign.rs`, `simulate.rs`; Defaults Profile
`crates/ovcd-format/src/builtin_profile.json`  ·  **Rule Sets:** none. This area applies no published Rule. Its
feasibility tests (the hull floats within its depth) are OVCD Checks.

## What OVCD computes here

- **Principal dimensions:** length overall, beam and Moulded Depth of one Hull, typed or taken from a Vessel Class.
  A left-out depth comes from a depth-to-beam ratio per Cargo group.
- **Vessel Class placement:** the class the vessel sits in, the draught limit and the air-draft limit that the class
  gives, and a warning past the largest class of the scheme.
- **Hull Form:** the bow rake length, which sets the bow void. The hydrostatics use a box; the drawing shows the rake.
- **Hydrostatics:** laden draught (`draught_m`, `full_load_draft_m`), light draught (`light_draught_m`), ballast
  draught, displacement, freeboard (`freeboard_m`), the `not_overloaded` and `freeboard_ok` checks, and the draught
  limit that sets the weight-bound cargo (`draught_max_m`).
- **Form values for stability:** KB and transverse BM at a draught. Area 03 uses them for GM and GZ.
- **Legacy path only** (the Python API and the evaluation harness): longitudinal BM, LCB, trim and the draughts fore
  and aft, from the raked box.

## How it works today

There are two hull models. Know which one you read.

1. **Main path** (`.vdesign` files, the app and the CLI, through `simulate.rs` and `barge_capacity::compute`). The
   hull is a **rectangular box** `L × B × D` with no rakes: `HullGeometry::new(loa, beam, depth, 0, 0, 0, 0)` at
   `barge_capacity.rs:570` and `:889`, and `journey_derating.rs:113`. The block coefficient is the only form
   reduction. Label: Fixed in code.
2. **Legacy path** (`BargeInputs`, `hydrostatics::calculate_stability` and `calculate_trim`, reached from
   `crates/ovcd-format/src/eval.rs`). The hull is the **raked box** of `hull.rs` (`hull_form::RakedBox`), with
   flat-bottom rakes at bow and stern. Label: Method.

ADR 0012 decides that the master geometry will become a **table of sections** (offsets at stations). **No sections
table exists in the code today.** The raked box stays as a Hull Form generator and as an exact test case. The format
change waits for `.vdesign` version 3 (spec 0004, issue #274).

### Step 1. Principal dimensions

- `L` is the length overall (`length_overall_m`). OVCD uses it as the length for every hydrostatic quantity: the
  displacement, the block coefficient basis, the squat `Lpp` and the resistance `Lwl` (`simulate.rs:666`, `:741`,
  `:751`; `ResistanceInputs { lwl: l }` in `simulate.rs` `curves`). Label: Fixed in code.
- `B` is the moulded beam (`beam_m`). The section is a full rectangle of width `B`. Label: Fixed in code.
- `D` is the Moulded Depth (`moulded_depth_m`). Left out, `D = round(r × B, 0.001 m)`, where `r` is the
  depth-to-beam ratio of the Cargo group (`vdesign.rs` `depth_default`, line 2478; group from `depth_group`, line
  1707). Dry-bulk Cargo is group `dry_bulk`; every liquefied gas is group `gas`; group `liquid` has no Cargo yet.
  Label: Estimate.
- A hull with `L / D` above the soft limit gets a warning (`vdesign.rs` `cross_checks`, line 2683). Label: Estimate.

### Step 2. Vessel Classes as maximum envelopes

- The catalogue is `catalogue/vessel_classes.yaml`, compiled to `vessel_classes_generated.rs` (292 classes). Each
  row has a scheme (`family`), a Formation word, length, beam, draught, air-draft and deadweight ranges, a
  confidence (204 high, 76 medium, 12 low) and a source.
- A class's length and beam are **maxima**: the upper end of the range (`ClassRow::loa_max_m`, `beam_max_m`,
  `vessel_classes.rs:68-75`).
- A class is *pickable* when it has a length, a beam and a Formation kind the format knows (`pickable`, line 94).
  Towed barges, ATBs, pushers, tugs and towboats are not pickable.
- The *ladder* is the pickable classes of one scheme and one Formation kind, sorted by length maximum, then beam
  maximum, then id (`ladder`, lines 120-138).
- The vessel sits in the **first class on the ladder whose length and beam maxima both hold it** (`place`, lines
  152-180). Past the largest class it stays in the largest and `beyond_largest` is true; the app warns
  (`class_input.rs` `resolve`, line 185).
- **Picking a class sets `L` and `B` to the class maxima** and sets the Formation (`class_input.rs` `class_values`,
  line 48). Depth stays typed or comes from Step 1.
- The class fills the **draught limit with the top of its draught range** and the **air-draft limit with the bottom
  of its air-draft range**, unless the designer typed a limit (`class_limits`, line 59; `vessel_classes.rs:78-86`).
- The class's deadweight is a reference value. It is shown beside the result, never used as an input
  (`class_input.rs` `summary`).
- A pushed-convoy Formation gives no results today: `run.rs:91` returns `CONVOY_NOT_AVAILABLE`.

### Step 3. Hull form

- **Main path.** The bow rake length is `1.6 × D` unless typed (`barge_capacity.rs` `arrangement`, lines 788-791;
  constant `DEFAULT_BOW_RAKE_TO_DEPTH_RATIO`, line 49). It is part of the bow void: `bow void = rake + forward
  separation` (line 802). A pushed barge has a stern void of 3.0 m (line 72); a self-propelled Hull has its stern
  block. The rake does **not** enter the hydrostatics or the hull steel (the box at line 570 has zero rakes). The
  drawing does show the rake (`ga_adapter.rs:204`), with a rise equal to the depth (`models.rs:232-237`).
- **Legacy path.** The raked box has, per rake, a horizontal length `l_r` and a vertical rise `h_r` (default `h_r =
  D`, `models.rs` `post_init`). For draught `T` (`hull.rs:64-132`):
  - parallel midbody length `L_p = L − l_bow − l_stern`;
  - midbody volume `B × min(T, D) × L_p`;
  - rake volume `B × T² × l_r / (2 h_r)` when `T ≤ h_r`, else `B × l_r × (T − h_r / 2)`;
  - waterplane length `L_wp = L_p + Σ min(l_r, T × l_r / h_r)`; waterplane area `B × L_wp`.

### Step 4. Displacement, draught and freeboard (main path)

- Displaced volume `∇ = Cb × L × B × T`, in m³. Displacement `Δ = ρ × ∇`, in t (`barge_capacity.rs:905`,
  `simulate.rs:741`, `:751`). `Cb` is an input (`block_coefficient`, default 0.85) and is on the length overall.
  Label: Method; the value of `Cb` is an Estimate.
- The laden draught is found by bisection (`hydrostatics.rs` `find_equilibrium_draft`, lines 21-56): required volume
  `W / (ρ × Cb)`, search from 0 to `1.5 × D`, at most 100 steps, stop when the volume error is below `0.001 × B` m³.
  The draught is then clamped to `D` (`barge_capacity.rs:904`). Label: Method.
- `overloaded` is true when `W / (ρ Cb) > L × B × D` (line 901).
- Freeboard `F = D − T` (line 906). The check `freeboard_ok` is `F ≥ min_freeboard_m` (default 0.3 m).
- The light draught uses the lightship alone; the ballast draught adds `ballast_capacity_t` (lines 910-921).
- For a box or hopper hold, `simulate.rs` then takes the draught limit `T_max = min(D − F_min, draught_limit_m,
  channel depth − under-keel clearance)` (lines 726-738). The weight-bound cargo is `Cb × L × B × T_max × ρ −
  lightship`. The cargo is the smaller of the weight bound and the volume bound. The reported draught is `T = Δ / (Cb
  × L × B × ρ)` (line 751).
- For Type C tanks, `gas_arrangement::fit` and `journey_derating::compute` do the same with the box hull
  (`journey_derating.rs:113-130`). With `optimise_depth` on, the depth is picked from 3.5 to 7.0 m in 0.25 m steps
  (`gas_arrangement.rs:50`).

### Step 5. Form values for stability

- **Main path, gas:** `KB = T / 2`, `BM = B² / (12 T)` (`gas_stability.rs` `box_gm_solid`, line 586). The draught is
  `T = Δ / (Cb × L × B × ρ)` (line 743).
- **Main path, dry bulk:** `hull.KB(T) + hull.BM_transverse(T)` of the box (`barge_capacity.rs` `bulk_gm`, line
  711), which gives the same `T / 2` and `B² / (12 T)`.
- In both, `BM` divides the full box waterplane inertia `L B³ / 12` by the **box** volume `L B T`, not by the
  Cb-reduced volume. Label: Fixed in code.
- **Legacy path:** `KB` by midpoint integration of the waterplane area over 100 strips (`hull.rs:171-188`).
  `I_T = L_wp × B³ / 12`, `I_L = B × L_wp³ / 12` (lines 158-167). `BM_T = I_T / ∇`, `BM_L = I_L / ∇`, with the raked
  volume (lines 224-240). `LCB` from the stern by first moments of the midbody and the two rakes (lines 192-220).
- **Legacy trim:** `trim = (LCG − LCB) × L_wp / BM_L`, with `BM_L` in place of `GM_L`; positive is bow down. The
  draughts fore and aft are `T ± trim / 2`, which puts the centre of flotation at mid-length (`hydrostatics.rs`
  lines 222-231). The displacement in the legacy stability multiplies the raked volume by `Cb` again (line 71).

### Step 6. Squat, where the hull form drives it

The squat method belongs to area 08. The hull form enters it through four terms (`squat.rs` `compute`, lines
129-160):

- `Lpp` is the length overall (passed from the resistance `lwl`, `resistance.rs:612`).
- `∇ = Cb × Lpp × B × T`, unless a volume is given.
- The midship area is `B × T` (a midship coefficient of 1). The blockage is `B T / A_channel`.
- `Cb` enters Barrass II directly: `S = Cb × S_b^0.81 × V^2.08 / 20` (V in knots). ICORELS uses
  `2.4 × ∇ / Lpp² × Fnh² / √(1 − Fnh²)`. The larger is adopted.

The resistance module takes a different midship area, `B × T × Cb` (`resistance.rs:365-367`), for its blockage and
its Schijf speed limit.

## Assumptions, rules and estimates

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| HF-A01 | Length for hydrostatics | Length overall `L`, for `∇`, `Cb`, squat `Lpp` and resistance `Lwl` | Fixed in code | None; the input is the only length | `simulate.rs:666`, `:741`, `:751`, `curves`; `barge_capacity.rs:889` | Unknown. With a `Cb` taken on `Lwl`, the draught is too small: non-conservative |
| HF-A02 | Section shape | Full rectangle of width `B`, vertical sides, flat bottom | Fixed in code | Concept barge form | `hull.rs` all; `gas_stability.rs:242`, `:586` | Small for box barges; unknown for shaped hulls |
| HF-A03 | Default Moulded Depth | `D = round(r × B, 1 mm)`, `r` by Cargo group | Estimate | `depth_to_beam_ratio` basis; `catalogue/depth_rule.md` | `vdesign.rs` `depth_default` (2478) | Unknown. Rule minus published depth runs from −50 % to +47 % on the reference vessels |
| HF-A04 | D/B, dry bulk and container | 0.33 | Estimate | Midpoint of reference vessels, stated as 0.22 to 0.44 | profile `depth_to_beam_ratio.default_by_cargo_group` | Unknown; 20 inland vessels span 0.22 to 0.49 |
| HF-A05 | D/B, liquid | 0.30 | Estimate | Stated as 0.22 to 0.38; no Cargo uses it | profile | None today |
| HF-A06 | D/B, gas | 0.55 | Estimate | 8 gas vessels, 7 sea-going; range 0.44 to 0.66 | profile | Deeper than the one inland gas vessel (0.50): conservative for freeboard, heavier hull |
| HF-A07 | Slenderness warning | `L / D` above 25 warns | Estimate | Reference vessels with a depth are below 25 | `vdesign.rs:2683` | None on results; warning only |
| HF-A08 | Length range | Hard 1 to 1000 m; soft 20 to 250 m | Estimate | Catalogue single-hull classes 32 to 225.5 m (tugs and towboats from 23.8 m) | profile `length_overall_m` | Warning only |
| HF-A09 | Beam range | Hard 1 to 200 m; soft 4 to 35 m | Estimate | Catalogue 4.7 to 28.8 m | profile `beam_m` | Warning only |
| HF-A10 | Depth range | Hard 0.1 to 100 m; soft 1 to 15 m | Estimate | 41 reference vessels 2.2 to 12.5 m | profile `moulded_depth_m` | Warning only |
| HF-A11 | D/B range | Hard above 0 to 2; soft 0.2 to 0.7 | Estimate | Reference vessels | profile `depth_to_beam_ratio` | Warning only |
| HF-A12 | Class length and beam | Upper end of the class range | Method | Catalogue rule (issue #94) | `vessel_classes.rs:68-75` | Unknown |
| HF-A13 | Picking a class | Sets `L` and `B` to the class maxima | Method | Catalogue rule (issue #94) | `class_input.rs` `class_values` (48) | Hull at the envelope limit. Some schemes allow LOA up to 10 % under (GB 38030 rows): capacity on the high side |
| HF-A14 | Class placement | First class on the ladder (length max, beam max, id) that holds both | Method | Issue #94 | `vessel_classes.rs` `ladder`, `place` (120-180) | None on results |
| HF-A15 | Beyond the largest class | Stays in the largest class, warning | Method | Issue #94 | `vessel_classes.rs:164-169`; `class_input.rs:185` | Warning only |
| HF-A16 | Draught limit from a class | Top of the class draught range | Method | Catalogue | `vessel_classes.rs:78`; `class_input.rs:59` | Non-conservative if the waterway is shallower than the class top |
| HF-A17 | Air-draft limit from a class | Bottom of the class air-draft range | Method | Catalogue | `vessel_classes.rs:84` | Conservative |
| HF-A18 | Class data | Length, beam, draught, air draft, deadweight per class, each with a source and a confidence | Sourced default | Per row `source` in `catalogue/vessel_classes.yaml` | `vessel_classes_generated.rs` | Depends on the row; 12 rows are low confidence |
| HF-A19 | Convoy unit beam | `B / n_abreast` | Fixed in code | None | `vessel_standards.rs:92-94` | Unknown; see the triage tickets |
| HF-A20 | Pushed convoy | No results | Fixed in code | Not built | `run.rs:91` | Not applicable |
| HF-A21 | Hydrostatic hull, main path | Box `L × B × D`, no rakes | Fixed in code | Carried over from the original engine | `barge_capacity.rs:570`, `:889`; `journey_derating.rs:113` | See HF-A26 |
| HF-A22 | Bow rake length | `1.6 × D` | Estimate | Carried over from the original engine | `barge_capacity.rs:49`, `:788-791` | Sets the bow void and so the cargo length: unknown |
| HF-A23 | Bow rake rise | `= D` | Fixed in code | Carried over | `models.rs:232-234`; drawing only on the main path | Drawing only |
| HF-A24 | Stern rake, main path | None. Pushed barge: 3.0 m stern void | Estimate | Carried over | `barge_capacity.rs:72`, `:785` | Unknown |
| HF-A25 | Stern rake rise, legacy | `= D` | Fixed in code | Carried over | `models.rs:235-237` | Legacy path only |
| HF-A26 | Block coefficient | 0.85, on `L × B × T` with `L` the length overall | Estimate | Basis cites calibrated box barges 0.883 to 0.995; fitted value 0.928 (`calibration/factors.yaml`) | profile `block_coefficient`; `barge_capacity.rs:208` | For box barges, 9 % below the fitted value: draught too deep, weight-bound cargo too low (conservative for draught, low on capacity) |
| HF-A27 | Cb range | Hard 0.3 to 1.0; soft 0.7 to 1.0 | Estimate | 0.3 is called the squat method's validity floor; no source read | profile; `squat.rs:75` | Unknown |
| HF-A28 | Use of Cb | One uniform factor on the box volume: `∇ = Cb L B T` | Method | Definition of Cb | `simulate.rs:741`, `:751`; `barge_capacity.rs:905`; `hydrostatics.rs:36` | Exact for the definition; the draught is linear in `1 / Cb` |
| HF-A29 | Water density | 1.0 t/m³ (fresh). Hard 0.9 to 1.1; soft 0.99 to 1.03 | Estimate | Fresh water; sea water about 1.025 | profile `water_density_t_per_m3`; `simulate.rs:669` | In sea water, draught 2.5 % too deep: conservative |
| HF-A30 | Draught solver | Bisection, 0 to `1.5 D`, at most 100 steps, stop at volume error `< 0.001 × B` m³ | Fixed in code | None | `hydrostatics.rs:21-56`; `barge_capacity.rs:903` | Negligible |
| HF-A31 | Draught clamp | `T ≤ D`; `overloaded` when `W / (ρ Cb) > L B D` | Fixed in code (OVCD Check) | Physical | `barge_capacity.rs:895-904` | None |
| HF-A32 | Freeboard | `F = D − T`, at the side, at mid-length, with no sheer or camber | Method | Plane of maximum draught to the deck at side | `barge_capacity.rs:906`; `simulate.rs` results | Unknown; no trim on the main path |
| HF-A33 | Minimum freeboard | 0.3 m; must be less than `D` | Estimate | Carried over; no source given | profile `min_freeboard_m` | Area 08 |
| HF-A34 | Draught limit | `min(D − F_min, draught_limit_m, channel depth − UKC)` | Method | None | `simulate.rs:726-738`, `:846-886` | Area 08 |
| HF-A35 | Light and ballast draught | Same solver with lightship, or lightship + ballast | Method | None | `barge_capacity.rs:910-921` | As HF-A26 |
| HF-A36 | KB, main path | `T / 2` | Fixed in code | Box section | `gas_stability.rs:586`; `barge_capacity.rs:711` | For a shaped hull KB is usually above `T / 2`: conservative for GM |
| HF-A37 | BM, main path | `B² / (12 T)`: box inertia over the box volume, not the Cb volume | Fixed in code | Box section | `gas_stability.rs:586`; `barge_capacity.rs:711` | With a full waterplane and `Cb < 1`, BM is low by the factor `Cb`: conservative for GM. Unknown for a fine waterplane |
| HF-A38 | Waterplane coefficient | Full rectangle, `Cwp = 1` on the main path | Fixed in code | Box | implicit in HF-A37 | As HF-A37 |
| HF-A39 | KB, legacy | 100-strip midpoint integration of the waterplane area | Method | Numerical integration | `hull.rs:171-188` | Negligible |
| HF-A40 | Waterplane inertias, legacy | `I_T = L_wp B³ / 12`, `I_L = B L_wp³ / 12` | Method | Rectangle | `hull.rs:158-167` | Exact for the raked box |
| HF-A41 | Rake volume, legacy | Triangle below the rise, trapezium above; not clipped at `D` | Method | Geometry | `hull.rs:64-76`, `:128-131` | Exact below `D`; above `D` only when overloaded |
| HF-A42 | LCB, legacy | First moments of midbody and rakes, from the stern | Method | Geometry | `hull.rs:192-220` | Stern rake moment is mirrored: LCB too far aft (see the triage tickets). Legacy path only |
| HF-A43 | Trim, legacy | `(LCG − LCB) × L_wp / BM_L`, `GM_L ≈ BM_L` | Method | Moment to change trim with `GM_L ≈ BM_L` | `hydrostatics.rs:220-227` | Small |
| HF-A44 | Draughts fore and aft, legacy | `T ∓ trim / 2`: centre of flotation at mid-length | Fixed in code | None | `hydrostatics.rs:230-231` | Unknown for a hull with unequal rakes |
| HF-A45 | Displacement, legacy stability | Raked volume × `ρ` × `Cb` | Fixed in code | None | `hydrostatics.rs:71`, `:172` | With rakes, the form reduction counts twice: displacement too low |
| HF-A46 | Depth search | 3.5 to 7.0 m, 0.25 m steps, when `optimise_depth` is on (default off) | Fixed in code | None | `gas_arrangement.rs:50`; `simulate.rs:856-878` | Hulls outside the list are never tried |
| HF-A47 | Hull plating areas, legacy | Bottom `L_p B`; sides `2 L D`; deck `L B`; rakes slope length × `B` | Method | Geometry | `hull.rs:243-283`; used by `weights.rs:63` | Area 02. Main path has zero rake plating |
| HF-A48 | Squat length | `Lpp = L` (length overall) | Fixed in code | None | `resistance.rs:612`; `simulate.rs` `curves` | ICORELS squat goes as `Cb B T / Lpp`, so it is low by the ratio `Lpp / L`: non-conservative |
| HF-A49 | Squat volume | `∇ = Cb Lpp B T` | Method | Definition of Cb | `squat.rs:137-141` | As HF-A26 |
| HF-A50 | Squat midship area | `B × T` (`Cm = 1`) | Fixed in code | None | `squat.rs:142` | Conservative (larger blockage) |
| HF-A51 | Barrass II | `Cb × S_b^0.81 × V^2.08 / 20` | Method | Named "Barrass II" in code; source text not read | `squat.rs:107-112` | Area 08 |
| HF-A52 | ICORELS | `2.4 ∇ / Lpp² × Fnh² / √(1 − Fnh²)` | Method | Named "ICORELS / PIANC" in code; source text not read | `squat.rs:115-127` | Area 08 |
| HF-A53 | Squat validity | `Fnh < 0.70` and blockage `≤ 0.35` warn; `Cb` 0.3 to 1.0 refused outside | Fixed in code (OVCD Check) | No source read | `squat.rs:26`, `:75`, `:202`, `:214` | Warning only |
| HF-A54 | Resistance midship area | `B × T × Cb` | Fixed in code | None | `resistance.rs:365-367` | Blockage low by `Cb`; Schijf speed limit too high: non-conservative (area 07) |
| HF-A55 | Wetted surface | `(L B + 2 L T) × 1.02` | Fixed in code | "1.02 rake form factor"; no source | `resistance.rs:309-313` | Area 07 |

## Inputs the user can change

| Input | Default | Unit | Where the default comes from |
|---|---|---|---|
| `length_overall_m` | none (required, or the class maximum) | m | Typed, or `class_input.rs` `class_values` |
| `beam_m` | none (required, or the class maximum) | m | Typed, or the class |
| `moulded_depth_m` | `depth_to_beam_ratio × beam_m` | m | Profile rule, `vdesign.rs` `depth_default` |
| `depth_to_beam_ratio` (profile only) | dry bulk 0.33, liquid 0.30, gas 0.55 | - | Profile `default_by_cargo_group` |
| `vessel_scheme`, `vessel_class` | none (typed dimensions) | - | `class_input.rs` |
| `formation` | from the class, or typed | - | Pushed convoy has no results yet |
| `block_coefficient` | 0.85 | - | Profile |
| `water_density_t_per_m3` | 1.0 | t/m³ | Profile |
| `bow_rake_length_to_depth_ratio` | 1.6 | - | Profile, carried over |
| `pushed_stern_void_m` | 3.0 | m | Profile, carried over |
| `min_freeboard_m` | 0.3 | m | Profile |
| `draught_limit_m` | none, or the top of the class draught range | m | Profile or class |
| `air_draft_limit_m` | none, or the bottom of the class air-draft range | m | Profile or class |
| `optimise_depth` | false | - | Profile |

The stern rake, the rake rises, the midship coefficient, the waterplane coefficient and the length basis of `Cb`
are not inputs.

## Limits and known findings

- No sections table. Every hull is a box for hydrostatics, with one `Cb` factor (ADR 0012, issue #274).
- No midship, prismatic or waterplane coefficient. The waterplane is a full rectangle on the main path.
- No LCB, LCF, trim or draughts fore and aft on the main path. The hull floats level.
- No hydrostatic table (TPC, MCT, KM against draught).
- `Cb` is on the length overall. ES-TRIN 2025/1 Article 1.01, 4.9 defines the block coefficient on `Lwl × Bwl × T`.
  A `Cb` taken from a source on that basis gives OVCD too small a draught.
- The drawn bow rake is not in the hydrostatics, and `Cb` is not derived from it.
- The default `Cb` (0.85) is below the range its own basis cites (0.883 to 0.995).
- Pushed convoys give no results.
- Errors and proposed changes: see the findings register (#121) and the engineering basis triage tickets. After triage they go to the findings
  register (#121).

## Evidence

- `crates/ovcd-core/tests/hull.rs` and `tests/hydrostatics.rs`: parity with the values the original Python engine
  printed (volumes, waterplane, KB, BM, LCB, trim, the solver edge cases). These prove the port, not the physics. No
  test checks the LCB of a hull with a stern rake against an independent calculation.
- `crates/ovcd-core/tests/vessel_classes.rs`, `crates/ovcd-format/tests/vessel_class.rs`: the ladder, placement,
  limits, and that picking any class and placing its maxima gives the same class back, for every class.
- `crates/ovcd-format/tests/default_depth.rs`: the default depth rule.
- `catalogue/depth_rule.md`: the depth rule against 41 reference vessels with a published depth.
- `calibration/results.md`: deadweight at the stated draught, 30 held-out box barges, mean error −2.1 %, 30 of 30
  inside the band. **This uses the fitted `Cb` of 0.928, not the default 0.85.** Deck barges (13) and pontoons (7)
  run as boxes miss more often (8 of 13 and 3 of 7 inside).

## Questions for reviewers

1. Is `Cb` on the length overall a fair basis for raked inland barges at concept stage, or should OVCD use `Lwl`?
2. Is 0.85 a fair default `Cb` for a Type C tank barge? Box barges fit 0.928 on the length overall.
3. Is a bow rake of `1.6 × D` with a rise of `D` typical of inland barges? What stern rake should a pushed barge get?
4. The gas D/B ratio (0.55) comes mostly from sea-going gas carriers. What ratio fits inland gas barges?
5. For GM at concept stage, is `KB = T / 2` and `BM = B² / (12 T)` acceptable for a hull with `Cb` 0.85, or should BM
   use the Cb volume?
6. Should picking a Vessel Class set length and beam to the class maxima, or to a margin below them?
7. For squat, should `Lpp` be the length overall less the rakes?
8. Is a midship coefficient of 1 fair for inland barges in the blockage ratio?
