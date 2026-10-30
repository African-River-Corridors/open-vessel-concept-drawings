# Hull form and hydrostatics

> Concept and preliminary design only. Not for approval. Draft: needs expert review.
>
> Code state: written against `main` at commit `f6d446c` (2026-10-20). The hull and hydrostatics text follows
> issue #448 (hydrostatics and GZ from the Sections Table); its line references are to that change.

**For:** naval architect  ·  **Code:** `crates/ovcd-core/src/sections.rs`, `hydrostatics.rs` (`Hull`), `hull.rs`,
`stern_form.rs`, `barge_capacity.rs`, `vessel_classes.rs`, `vessel_standards.rs`, `squat.rs` (hull-form terms
only); `crates/ovcd-format/src/class_input.rs`, `vdesign.rs`, `simulate.rs`; Defaults Profile
`crates/ovcd-format/src/builtin_profile.json`  ·  **Rule Sets:** none. This area applies no published Rule. Its
feasibility tests (the hull floats within its depth, the hull finds a trim within ±10°) are OVCD Checks.

## What OVCD computes here

- **Principal dimensions:** length overall, beam and Moulded Depth of one Hull, typed or taken from a Vessel Class.
  A left-out depth comes from a depth-to-beam ratio per Cargo group.
- **Vessel Class placement:** the class the vessel sits in, the draught limit and the air-draft limit that the class
  gives, and a warning past the largest class of the scheme.
- **Hull Form:** the bow rake (length and rise) and the Stern Form. They make the hull's Sections Table. The
  hydrostatics and the drawing use the same hull.
- **Hydrostatics:** laden draught (`draught_m`, `full_load_draft_m`), light draught (`light_draught_m`), ballast
  draught, displacement, freeboard (`freeboard_m`), the `not_overloaded` and `freeboard_ok` checks, and the draught
  limit that sets the weight-bound cargo (`draught_max_m`).
- **Hull values in the results:** `hydrostatics` (the upright values at the laden draught) and `hydrostatic_table`
  (10 draughts, 0.1 D to D). The block coefficient `Cb` (`dims.cb`) is a result, not an input.
- **Trim:** the free-trim floating position, where OVCD has the mass items along the hull. The check
  `trim_in_range` says when no trim within ±10° balances the hull.
- **Form values for stability:** KM_T of the upright cut at a draught. Area 03 uses it for GM, and the sections for GZ.
- **Legacy path only** (the Python API and the evaluation harness): KB, BM, LCB, a linear trim and the draughts fore
  and aft, from the sections of the raked box.

## How it works today

There is one hull model: the **Sections Table** (ADR 0012, spec 0008). Every hydrostatic value comes from it.

1. **Main path** (`.vdesign` files, the app and the CLI, through `simulate.rs` and `barge_capacity::compute`). The
   hull is the raked box of the Option (bow rake, no stern rake), with the Stern Form cut into it:
   `Hull::build` (`hydrostatics.rs:328-354`), called at `barge_capacity.rs:1208`, `simulate.rs:1625`
   (`option_hull`) and `journey_derating.rs:113`. Label: Method.
2. **Legacy path** (`BargeInputs`, `hydrostatics::calculate_stability` and `calculate_trim`, reached from
   `crates/ovcd-format/src/eval.rs`). The hull is the **raked box** of `hull.rs`, with flat-bottom rakes at bow and
   stern. It is turned into sections (`RakedBox::sections`, `hull.rs:433`) before any integration. Label: Method.

The Sections Table holds offsets at stations. The bilge radius (16 chords per quarter circle), sheer (a parabola, 0 at
mid-length, with 8 more stations on each sheered half) and camber (a parabola, 16 chords on each half) shape it
(#449); all three default to 0. The other Hull Form inputs of spec 0008 (deadrise, keel half-width, chine) are inputs
only. No hull uses them yet (#450).

Two parts of the engine still use a box `L × B × D` with no rakes: the weights (`barge_capacity.rs:776`, area 02)
and the resistance wetted surface (Step 6). They can disagree a little with the sections.

### Step 1. Principal dimensions

- `L` is the length overall (`length_overall_m`). It is the length of the hull and of its sections. The lengths
  the other methods read come from the sections at the draught: the waterline length `L_WL` for `Cb`, the
  resistance `Lwl` and the squat `Lpp` (`simulate.rs:1651-1668`, `:365`). Label: Method.
- `B` is the moulded beam (`beam_m`). Every section is a rectangle of width `B`, less the rake and the Stern Form
  cuts. Label: Fixed in code.
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

- **Main path.** The bow rake length is `1.6 × D`, at most `bow_rake_max_length_m` (10 m; issue #661), unless `bow_rake_length_m` is typed (`barge_capacity.rs:1079-1082`;
  constant `DEFAULT_BOW_RAKE_TO_DEPTH_RATIO`, line 50; `simulate.rs` `bow_rake_of`, line 1613). The keel rises in a
  straight line to `bow_rake_rise_m` at the bow, or to `D` when it is not typed. `Hull::build` cuts a rake longer
  than the hull to `L`, and a rise above `D` to `D` (`hydrostatics.rs:328-334`).
- The rake is part of the bow void: `bow void = rake + forward separation` (`barge_capacity.rs:1093`). A pushed
  barge has a square stern and a stern void of 3.0 m (line 100). A self-propelled Hull has its Stern Form: its run
  and tunnels are cut into the sections (`stern_form::apply`, `stern_form.rs:412`).
- The rake and the Stern Form **do** enter the hydrostatics now. The drawing shows the same rake
  (`ga_adapter.rs:232`).
- The hull steel does not see them: the weights still take the box `L × B × D` with zero rakes
  (`barge_capacity.rs:776`; area 02).
- **Legacy path.** The raked box has, per rake, a horizontal length `l_r` and a vertical rise `h_r` (default `h_r =
  D`, `models.rs` `post_init`). Its sections come from `RakedBox::sections` (`hull.rs:433-445`). The closed forms of
  `hull.rs:64-132` stay as an exact test case for the sections (`tests/hull_hydrostatics.rs`). For draught `T`:
  - parallel midbody length `L_p = L − l_bow − l_stern`;
  - midbody volume `B × min(T, D) × L_p`;
  - rake volume `B × T² × l_r / (2 h_r)` when `T ≤ h_r`, else `B × l_r × (T − h_r / 2)`;
  - waterplane length `L_wp = L_p + Σ min(l_r, T × l_r / h_r)`; waterplane area `B × L_wp`.

### Step 4. Displacement, draught and freeboard (main path)

- **Volume.** `∇` is the volume of the sections below the waterplane, in m³ (`sections::immersed`,
  `sections.rs:645`). Each section is clipped by the waterplane (Sutherland–Hodgman). The shoelace formula gives
  its area and moments. Along the length, OVCD splits each interval where an edge crosses the waterplane, then
  uses Simpson's rule. This is exact (to rounding) for a prismatic interval and for the upright raked box.
  Displacement `Δ = ρ × ∇`, in t. Label: Method.
- **Level draught.** `draught_for_volume` (`sections.rs:814`) finds the upright, level draught for a volume. It
  uses the Illinois secant method, at most 16 steps, stopping when the volume is within 1e-13 of the target (`VOLUME_STEPS`, `VOLUME_TOLERANCE`). The bracket runs from
  the keel to the top of the hull. Label: Method.
- A load that needs the whole hull volume or more has no draught. The draught is then set to `D`.
  `overloaded` is true when `W / ρ` is more than the volume to the deck (`barge_capacity.rs:1223-1235`).
- **Free trim.** A dry bulk hull with its stern block in the lightship has mass items along the hull. Its Laden and
  Light conditions float at free trim (`Hull::float`, `hydrostatics.rs:372`; `sections::equilibrium`,
  `sections.rs:891`). The inner solve finds the volume (at most 16 steps). The outer solve finds the trim that puts the
  centre of buoyancy under G (at most 12 steps, `TRIM_STEPS`; it stops when B - G is within 1e-11 m). The trim stays within ±10° (`TRIM_LIMIT_DEG`,
  line 762). The draught is the one at mid-length. The draughts aft and forward are at `x = 0` and `x = L`,
  square to the base line. Label: Method.
- If no trim within ±10° balances the hull, the condition keeps its level draught. The check `trim_in_range` then
  fails (`barge_capacity.rs:1274-1283`, `:1461-1476`). This happens on a deep-laden hull with the stern block aft
  and G well forward: the bow deck would go under. An Option with no mass items along the hull floats level.
- Freeboard `F = D − T`, with `T` at mid-length (`barge_capacity.rs:1298`). The check `freeboard_ok` is
  `F ≥ min_freeboard_m` (default 0.3 m).
- The light draught uses the lightship alone; the ballast draught adds `ballast_capacity_t`. The ballast draught is
  always level (`barge_capacity.rs:1302-1305`).
- For a box or hopper hold, `simulate.rs` then takes the draught limit `T_max = min(D − F_min, draught_limit_m,
  channel depth − under-keel clearance)` (lines 1053-1063). The weight-bound cargo is `ρ × ∇(min(T_max, D)) −
  lightship`, with `∇` from the sections (line 1066). The cargo is the smaller of the weight bound and the volume
  bound. The reported draught `draught_m` is the level draught of the reported displacement (line 1103). With a
  stern block, the `trim` list gives the free-trim draughts as well.
- For Type C tanks, `gas_arrangement::fit` and `journey_derating::compute` work on each candidate's own hull
  (`journey_derating.rs:113-136`; `gas_arrangement.rs:613-634`). With `optimise_depth` on, the depth is picked from
  3.5 to 7.0 m in 0.25 m steps (`gas_arrangement.rs:50`). Each candidate depth gets its own rake, unless the rake is
  typed.
- **Hull values in the results** (`simulate.rs:1671-1712`). `hydrostatics` holds the upright, level values at the
  laden draught: volume, displacement, KB, LCB, BM_T, BM_L, KM_T, KM_L, waterplane area, LCF, `L_WL`, `B_WL`,
  midship area, wetted surface, TPC, MCT 1 cm, and the coefficients Cb, Cm, Cp and Cwp. It is null with no
  draught. `hydrostatic_table` holds the same values at 10 draughts, 0.1 D to D (`hydrostatics.rs:322`, `:408`).
  At D the waterplane is the deck.
- **Cb is a result**: `Cb = ∇ / (L_WL × B_WL × T)` at the laden level draught (`sections.rs:736`; `dims.cb`,
  `simulate.rs:1232`, `:2085-2090`). This is the ES-TRIN 2025/1 Article 1.01, 4.9 definition. With no draught,
  `dims.cb` gives Cb at the design draught.
- The IGC 4.28.2 vertical acceleration of a gas tank reads Cb at the design draught (`simulate.rs:972`, `:997`,
  `:1987`).

### Step 5. Form values for stability

- **Main path.** `sections::upright` (`sections.rs:724-754`) cuts the hull upright and level at one draught. KB is
  the height of the centre of buoyancy of that cut. `I_T` and `I_L` are the waterplane inertias about its
  centroid. `BM_T = I_T / ∇`, `BM_L = I_L / ∇`, `KM_T = KB + BM_T`. TPC is `ρ A_WP / 100`. MCT 1 cm is
  `Δ BM_L / (100 L_WL)`, with KG taken as KB. Label: Method.
- GM is `KM_T − KG`, with KM_T of the upright cut at the condition's draught. Dry bulk: `bulk_gm`
  (`barge_capacity.rs:947-961`). Gas: area 03. The box values `T / 2` and `B² / (12 T)` are no longer used by any
  Design.
- **Legacy path.** `calculate_stability` and `calculate_trim` take the upright cut of the raked box's sections
  (`hydrostatics.rs:57-91`, `:96`, `:202`). KB, BM_T, BM_L and LCB come from that cut. The displacement is `ρ ∇`:
  Cb no longer counts twice.
- **Legacy trim:** `trim = (LCG − LCB) × L_WL / BM_L`, with `BM_L` in place of `GM_L`; positive is bow down. The
  draughts fore and aft are `T ± trim / 2`, which puts the centre of flotation at mid-length (`hydrostatics.rs`
  lines 262-271). This trim is still linear; it is not the free-trim float of Step 4.
- **Legacy draught.** `find_equilibrium_draft` (`hydrostatics.rs:30-53`) now returns the level draught of the
  sections. It still takes a block coefficient and a tolerance, and uses neither. A weight the whole hull cannot
  float gives `1.5 × max_draft`.

### Step 6. Squat, where the hull form drives it

The squat method belongs to area 08. The hull form enters it through four terms (`squat.rs` `compute`, lines
138-279):

- `Lpp` is the hull's waterline length `L_WL` at the draught (passed from the resistance `lwl`,
  `resistance.rs:587`, `:604`; set at `simulate.rs:365`).
- `∇ = Cb × Lpp × B × T`, with the hull's Cb, unless a volume is given. With `B_WL = B`, this is the volume of the
  sections.
- The midship area is the hull's own: the immersed area of the section at mid-length of the waterline
  (`squat.rs:147-152`; `sections.rs` `immersed`). The blockage is `A_M / A_channel`.
- `Cb` enters Barrass II directly: `S = Cb × S_b^0.81 × V^2.08 / 20` (V in knots). ICORELS uses
  `2.4 × ∇ / Lpp² × Fnh² / √(1 − Fnh²)`. The larger is adopted.

The resistance module takes the same midship area for its blockage and its Schijf speed limit
(`resistance.rs:395-402`). The Python API has no sections and keeps `B × T` (`eval.rs` `resistance_inputs`).

The resistance wetted surface stays box-based: `(L_WL B + 2 L_WL T) × 1.02`, plus the area the Stern Form adds
(`resistance.rs:345-349`). It does not use the wetted surface of the sections.

## Assumptions, rules and estimates

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| HF-A01 | Length for hydrostatics | The sections run over the length overall `L`. `Cb`, squat `Lpp` and resistance `Lwl` take the waterline length `L_WL` at the draught | Method | Spec 0008; ES-TRIN 2025/1 Art. 1.01, 4.9 for `Cb` | `hydrostatics.rs:328-354`; `simulate.rs:365`, `:1651-1668` | Small. The weights still use `L` (area 02) |
| HF-A02 | Section shape | Rectangle of width `B`, vertical sides, flat bottom, square bilge; less the bow rake and the Stern Form cuts | Fixed in code | Concept barge form; spec 0008 | `hydrostatics.rs:328-354`; `hull.rs` `RakedBox::sections` (433) | Small for box barges. A bilge radius, sheer and camber are inputs (#449), 0 by default: unknown for a shaped hull left at 0 |
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
| HF-A21 | Hydrostatic hull, main path | Sections Table of the raked box (bow rake, no stern rake), with the Stern Form cut in | Method | ADR 0012, spec 0008 | `barge_capacity.rs:1208-1216`; `simulate.rs:1625-1643`; `journey_derating.rs:113-124` | Exact to rounding for this shape (`tests/hull_hydrostatics.rs`). A real hull with a bilge radius has less volume |
| HF-A22 | Bow rake length | `min(1.6 × D, bow_rake_max_length_m)` (cap 10 m; none in a file saved before #661), unless `bow_rake_length_m` is typed; cut to `L` | Estimate | Ratio carried over from the original engine; cap a working value set by the maintainer (issue #661). No published source for either. For scale, a 195 x 35 x 12 ft jumbo hopper barge has a 27 ft 6 in (8.4 m) rake (Jeffboat, Maritime Reporter, 15 October 1977, p. 9) | `barge_capacity.rs:50`, `:1079-1082`; `simulate.rs:1613-1623`; `hydrostatics.rs:329` | Sets the bow void, the bow volume and `L_WL`: unknown. On a deep hull it binds the cargo: the worked 140 x 29 m gas example at `D` = 12 m has a 19.2 m rake; a 12 m rake gives 7.2 m more cargo zone and 8.2 % more cargo volume |
| HF-A23 | Bow rake rise | `= D`, unless `bow_rake_rise_m` is typed; cut to `D` | Estimate | Carried over; the rake the drawings show | `simulate.rs:1621`; `barge_capacity.rs:1213`; `hydrostatics.rs:333` | Sets the bow volume below the rise and the waterplane length at low draughts |
| HF-A24 | Stern rake, main path | None. Pushed barge: 3.0 m stern void, square stern. Self-propelled: the Stern Form | Estimate | Carried over; spec 0007 for the Stern Form | `barge_capacity.rs:100`; `stern_form.rs:412` | Unknown for a pushed barge with a raked stern: volume too large, non-conservative for draught |
| HF-A25 | Stern rake rise, legacy | `= D` | Fixed in code | Carried over | `models.rs:235-237` | Legacy path only |
| HF-A26 | Block coefficient | Removed (#448). `Cb` is no longer an input; it is a result of the sections (HF-A60) | — | — | — | — |
| HF-A27 | Cb range | Removed (#448). There is no `Cb` input to range. The squat still refuses `Cb` outside 0.3 to 1.0 (HF-A53) | — | — | — | — |
| HF-A28 | Use of Cb | Removed (#448). The volume comes from the sections, not from `Cb L B T` | — | — | — | — |
| HF-A29 | Water density | 1.0 t/m³ (fresh). Hard 0.9 to 1.1; soft 0.99 to 1.03 | Estimate | Fresh water; sea water about 1.025 | profile `water_density_t_per_m3`; `simulate.rs:669` | In sea water, draught 2.5 % too deep: conservative |
| HF-A30 | Draught solver | Illinois secant method, at most 16 steps, stop at 1e-13 of the target volume; bracket from the keel to the top of the hull | Fixed in code | Step cap and tolerance are fixed, so native and WebAssembly stop at the same step and give the same bits | `sections.rs:757`, `:782-826` | Negligible |
| HF-A31 | Draught clamp | `T ≤ D`; `overloaded` when `W / ρ` is more than the volume of the sections to the deck | Fixed in code (OVCD Check) | Physical | `barge_capacity.rs:1223-1235` | None |
| HF-A32 | Freeboard | `F = D − T`, at the side, at mid-length, with no sheer or camber | Method | Plane of maximum draught to the deck at side | `barge_capacity.rs:1298`; `simulate.rs:1248` | With trim, the freeboard at the deeper end is less: non-conservative |
| HF-A33 | Minimum freeboard | 0.3 m; must be less than `D` | Estimate | Carried over; no source given | profile `min_freeboard_m` | Area 08 |
| HF-A34 | Draught limit | `min(D − F_min, draught_limit_m, channel depth − UKC)` | Method | None | `simulate.rs:1053-1066` | Area 08 |
| HF-A35 | Light and ballast draught | Level draught of the sections for the lightship, or lightship + ballast. With a stern block, Light floats at free trim | Method | None | `barge_capacity.rs:1226-1305` | Small |
| HF-A36 | KB, main path | Centre of buoyancy of the upright cut of the sections | Method | Exact polygon integration | `sections.rs:645-700`, `:724`; `barge_capacity.rs:947-961` | Negligible for the modelled shape |
| HF-A37 | BM, main path | `BM_T = I_T / ∇` of the upright cut; GM uses `KM_T = KB + BM_T` | Method | Initial stability | `sections.rs:724-754` | Negligible for the modelled shape |
| HF-A38 | Waterplane coefficient | A result (`cwp`): `A_WP / (L_WL B_WL)` of the cut | Method | Definition | `sections.rs:751` | None; below 1 only where the bow rake is in the waterplane |
| HF-A39 | KB, legacy | Centre of buoyancy of the upright cut of the raked box's sections | Method | Exact polygon integration | `hydrostatics.rs:57-91`, `:110` | Negligible |
| HF-A40 | Waterplane inertias, legacy | `I_T`, `I_L` of the waterplane of the sections | Method | Exact polygon integration | `hydrostatics.rs:113-114`; `sections.rs:645` | Exact for the raked box |
| HF-A41 | Rake volume, legacy | Volume of the sections; the hull is closed at the deck, so the volume stops at `D` | Method | Geometry | `hull.rs:433-445`; `sections.rs:645` | Exact |
| HF-A42 | LCB, legacy | Centre of buoyancy of the sections, from the aft end | Method | Exact polygon integration | `hydrostatics.rs:215` | Small. The mirrored stern-rake moment of the closed form is no longer used |
| HF-A43 | Trim, legacy | `(LCG − LCB) × L_WL / BM_L`, `GM_L ≈ BM_L`, with the values of the sections | Method | Moment to change trim with `GM_L ≈ BM_L` | `hydrostatics.rs:262-268` | Small at small trim. Not the free-trim float (HF-A57) |
| HF-A44 | Draughts fore and aft, legacy | `T ∓ trim / 2`: centre of flotation at mid-length | Fixed in code | None | `hydrostatics.rs:270-271` | Unknown for a hull with unequal rakes |
| HF-A45 | Displacement, legacy stability | Removed (#448). The displacement is `ρ ∇` of the sections, so the form reduction no longer counts twice | — | — | — | — |
| HF-A46 | Depth search | 3.5 to 7.0 m, 0.25 m steps, when `optimise_depth` is on (default off). Each candidate has its own sections | Fixed in code | None | `gas_arrangement.rs:50`, `:883-885` | Hulls outside the list are never tried |
| HF-A47 | Hull plating areas | Legacy: bottom `L_p B`; sides `2 L D`; deck `L B`; rakes slope length × `B`. Main path: the box `L × B × D` with zero rakes, not the sections | Method | Geometry | `hull.rs:243-283`; `weights.rs:63`; `barge_capacity.rs:776` | Area 02. The main-path steel has no rake plating and ignores the Stern Form |
| HF-A48 | Squat length | `Lpp = L_WL`, the hull's waterline length at the draught | Method | Waterline of the sections | `resistance.rs:587`, `:604`; `simulate.rs:365` | Small; `L_WL` is near `Lpp` for a raked barge |
| HF-A49 | Squat volume | `∇ = Cb Lpp B T`, with the hull's `Cb` | Method | Definition of Cb | `squat.rs:141-145` | None while `B_WL = B`: it is then the volume of the sections |
| HF-A50 | Squat midship area | The hull's own: the immersed section at mid-length of the waterline | Method | Cut of the sections | `squat.rs:147-152`; `sections.rs:674-680` | Small. The Python API keeps `B × T` |
| HF-A51 | Barrass II | `Cb × S_b^0.81 × V^2.08 / 20` | Method | Named "Barrass II" in code; source text not read | `squat.rs:107-112` | Area 08 |
| HF-A52 | ICORELS | `2.4 ∇ / Lpp² × Fnh² / √(1 − Fnh²)` | Method | Named "ICORELS / PIANC" in code; source text not read | `squat.rs:115-127` | Area 08 |
| HF-A53 | Squat validity | `Fnh < 0.70` and blockage `≤ 0.35` warn; `Cb` 0.3 to 1.0 refused outside | Fixed in code (OVCD Check) | No source read | `squat.rs:26`, `:75`, `:202`, `:214` | Warning only |
| HF-A54 | Resistance midship area | The hull's own, as HF-A50 | Method | Cut of the sections | `resistance.rs:395-402`; `simulate.rs:370` | Small (area 07) |
| HF-A55 | Wetted surface | `(L_WL B + 2 L_WL T) × 1.02`, plus the area the Stern Form adds. Still box-based; it does not use the sections | Fixed in code | "1.02 rake form factor"; no source | `resistance.rs:345-349` | Area 07. Out of scope of #448 |
| HF-A56 | Volume integration | Each section clipped by the waterplane (Sutherland–Hodgman), shoelace area and moments; Simpson's rule along the length, split where an edge crosses the waterplane | Method | Standard polygon geometry (spec 0008) | `sections.rs:645` `immersed` | Exact to rounding for prismatic intervals and the upright raked box. Small for the Stern Form's curved run |
| HF-A57 | Free-trim float | Inner solve on the volume (at most 16 steps), outer solve on the trim (at most 12 steps), Illinois secant method, each stopping at a tolerance | Method | Free-floating equilibrium; G and B on one vertical | `sections.rs:757-762`, `:891-951`; `hydrostatics.rs:372-386` | Negligible inside the bracket |
| HF-A58 | Trim limit | The trim is searched within ±10°. No balance there: the condition keeps its level draught and fails `trim_in_range` (dry bulk) or the criterion "Trim within the solver's range" (area 03) | Fixed in code (OVCD Check) | An end would go under before 10° on these hulls | `sections.rs:762`, `:937-941`; `barge_capacity.rs:1274-1283`, `:1461-1476` | Conservative: the condition fails |
| HF-A59 | Where the hull floats at free trim | Dry bulk with a stern block (Laden, Light); the IS Code conditions; the legacy gas conditions with a stern block. Every other condition floats level | Fixed in code | Free trim needs the mass items along the hull | `barge_capacity.rs:1241-1296`; `loading_conditions.rs:448-506`; `gas_stability.rs:329-385` | Unknown where a level hull would trim |
| HF-A60 | Cb, a result | `∇ / (L_WL B_WL T)` at the laden level draught; at the design draught when there is no draught | Method | ES-TRIN 2025/1 Art. 1.01, 4.9 | `sections.rs:736`; `simulate.rs:1232`, `:2085-2090` | None; it is reported, and read by the squat and the IGC acceleration |
| HF-A61 | Reported draught | `draught_m` is the level draught of the reported displacement. Draughts aft and forward are at `x = 0` and `x = L`, square to the base line | Fixed in code | Issue #448 definitions | `simulate.rs:1103`, `:2074-2075`; `hydrostatics.rs:372-386` | Small; with trim, see the `trim` list |
| HF-A62 | Hydrostatic table | 10 upright, level rows, 0.1 D to D; at D the waterplane is the deck | Fixed in code | Issue #448 | `hydrostatics.rs:322`, `:408-417`; `simulate.rs:1700-1712` | None; reported only |
| HF-A63 | MCT 1 cm and TPC | `Δ BM_L / (100 L_WL)` with KG = KB; `ρ A_WP / 100` | Method | Usual first estimate | `sections.rs:746-747` | MCT a little high: KG is above KB in practice |
| HF-A64 | Cb for the IGC 4.28.2 acceleration | The hull's Cb at the design draught | Method | Issue #448 | `simulate.rs:972`, `:997`, `:1987` | Small |
| HF-A65 | Old files with `block_coefficient` | The stored value is dropped, with one warning per Option | Fixed in code | Format version 3 has no such input | `vdesign.rs:3258`, `:3346-3360` | None |
| HF-A66 | Legacy draught function | `find_equilibrium_draft` takes Cb and a tolerance and uses neither. A weight the hull cannot float gives `1.5 × max_draft` | Fixed in code | Kept for the Python API | `hydrostatics.rs:30-53` | Legacy path only |

## Inputs the user can change

| Input | Default | Unit | Where the default comes from |
|---|---|---|---|
| `length_overall_m` | none (required, or the class maximum) | m | Typed, or `class_input.rs` `class_values` |
| `beam_m` | none (required, or the class maximum) | m | Typed, or the class |
| `moulded_depth_m` | `depth_to_beam_ratio × beam_m` | m | Profile rule, `vdesign.rs` `depth_default` |
| `depth_to_beam_ratio` (profile only) | dry bulk 0.33, liquid 0.30, gas 0.55 | - | Profile `default_by_cargo_group` |
| `vessel_scheme`, `vessel_class` | none (typed dimensions) | - | `class_input.rs` |
| `formation` | from the class, or typed | - | Pushed convoy has no results yet |
| `water_density_t_per_m3` | 1.0 | t/m³ | Profile |
| `bow_rake_length_m` | none: `bow_rake_length_to_depth_ratio × D` | m | Profile |
| `bow_rake_length_to_depth_ratio` | 1.6 | - | Profile, carried over |
| `bow_rake_rise_m` | none: `D` | m | Profile |
| `stern_form` (self-propelled) | dry bulk tunnel, liquid gas transom | - | Profile; spec 0007 |
| `pushed_stern_void_m` | 3.0 | m | Profile, carried over |
| `min_freeboard_m` | 0.3 | m | Profile |
| `draught_limit_m` | none, or the top of the class draught range | m | Profile or class |
| `air_draft_limit_m` | none, or the bottom of the class air-draft range | m | Profile or class |
| `optimise_depth` | false | - | Profile |

`block_coefficient` is no longer an input (#448). A file that stores it is read with the value dropped, and one
warning per Option. The coefficients Cb, Cm, Cp and Cwp are results. The stern rake of a pushed barge is not an
input. The V-bottom and chine values are inputs, but no hull uses them yet.

## Limits and known findings

- The Sections Table holds the raked box, the Stern Form, and a bilge radius, sheer and camber (#449, all 0 by
  default). It has no V-bottom or chine (#450). The freeboard is read at mid-length, where the sheer is 0.
- The weights and the resistance wetted surface still use a box (`barge_capacity.rs:776`, `resistance.rs:345-349`).
  They can disagree a little with the sections until a later ticket.
- Free trim needs the mass items along the hull. A dry bulk hull without a stern block floats level. The gas
  capacity floats level too; only its stability conditions float at free trim (area 03). The reported `draught_m` is the
  mid-length draught at free trim for a dry bulk hull with its stern block mass, and the level draught otherwise.
- The free-trim search is limited to ±10°. A deep-laden dry bulk hull with its stern block aft and G well forward
  finds no balance there. It keeps its level draught and fails `trim_in_range`.
- The freeboard is taken at mid-length. With trim, the deeper end has less.
- The hydrostatic table is upright and level. There are no cross curves (KN against heel) in the results.
- A GZ curve on the sections costs time: about 200 ms per curve natively on a tunnel stern. OVCD builds a curve
  only where a criterion needs it (area 03).
- Settled by #448: the hull is no longer a box with one `Cb` factor; `Cb` is on `L_WL × B_WL × T`, the ES-TRIN
  2025/1 Article 1.01, 4.9 basis; the drawn bow rake is in the hydrostatics; there is no default `Cb` to sit below
  its basis.
- The midship coefficient Cm is a result in a Design (`hydrostatics.cm`, from the hull's own midship section); the
  input left format v3 (#535), and a file that stores it opens with one warning per Option. Where no hull section is
  given (the Python API) the midship section is still B × T × Cm, with Cm 1.0 by default (an Estimate).
- The convoy beam check of the Vessel Class catalogue (`convoy_beam_consistent`) is an identity for the built-in catalogue: its unit beam is the convoy beam over the number abreast. It can fail only for a catalogue that supplies its own unit beam (#432).
- Pushed convoys give no results.
- Errors and proposed changes: see the findings register (#121) and the engineering basis triage tickets. After triage they go to the findings
  register (#121).

## Evidence

- `crates/ovcd-core/tests/hull_hydrostatics.rs`: the `Hull` of the sections against closed forms. A box gives
  `∇ = L B T`, `KB = T / 2`, `BM_T = B² / (12 T)` and `Cb = 1`. A raked box matches the closed forms of `hull.rs`. A
  Stern Form removes the volume `SternGeometry::removed` gives. The free-trim float is level when G is over B and
  trims by the bow when G is forward. The table has 10 rows from 0.1 D to D.
- `crates/ovcd-core/tests/sections_equilibrium.rs`: the free-trim float and GZ against the box GZ, the wall-sided
  formula and the first-order trim; a trim beyond 10° is an error.
- `crates/ovcd-core/tests/hull.rs` and `tests/hydrostatics.rs`: the legacy entry points. These prove the port of
  the solver edge cases, not the physics.
- `crates/ovcd-core/tests/vessel_classes.rs`, `crates/ovcd-format/tests/vessel_class.rs`: the ladder, placement,
  limits, and that picking any class and placing its maxima gives the same class back, for every class.
- `crates/ovcd-format/tests/default_depth.rs`: the default depth rule.
- `catalogue/depth_rule.md`: the depth rule against 41 reference vessels with a published depth.
- `calibration/results.md`: deadweight at the stated draught, 30 held-out box barges, mean error −2.1 %, 30 of 30
  inside the band. **This run used a box hull with the fitted `Cb` of 0.928, before #448.** It is not yet re-run
  on the sections. Deck barges (13) and pontoons (7) run as boxes miss more often (8 of 13 and 3 of 7 inside).

## Questions for reviewers

1. Settled (#448): `Cb` is now a result on `L_WL × B_WL × T`. Is the laden level draught the right one to report it at?
2. Settled (#448): there is no default `Cb`. The hull shape sets it.
3. Is a bow rake of `1.6 × D` with a rise of `D` typical of inland barges? It now sets the bow volume. What stern
   rake should a pushed barge get?
4. The gas D/B ratio (0.55) comes mostly from sea-going gas carriers. What ratio fits inland gas barges?
5. Settled (#448): KB and BM come from the sections, not from `T / 2` and `B² / (12 T)`.
6. Should picking a Vessel Class set length and beam to the class maxima, or to a margin below them?
7. For squat, `Lpp` is now `L_WL` at the draught. Is that fair, or should it be the length between perpendiculars?
8. Settled (#448): the blockage takes the hull's own midship area.
9. Is ±10° a fair limit for the free-trim search? A hull that needs more fails the condition.
10. Should the reported `draught_m` be the free-trim draught at mid-length where OVCD has the mass items, not the
    level draught?
