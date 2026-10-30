# Gas cargo arrangement

> Concept and preliminary design only. Not for approval. Draft: needs expert review.
>
> Code state: written against `main` at commit `f6d446c` (2026-10-20).

**For:** gas carrier designer  ·  **Code:** `crates/ovd-core/src/gas_arrangement.rs`, `barge_capacity.rs`
(gas branch), `protective_location.rs`, `gas_separation.rs`, `design_grid.rs`, `aft_arrangement.rs`, `tanks.rs`,
`checks.rs`; the Design path in `crates/ovd-format/src/simulate.rs`  ·  **Rule Sets:** none built yet. Spec 0004
plans `igc-2g` (with `is2008-a`), `lr-inland-gas` (draft, unverified) and `ovd`. Today the rules are constants and
checks in code, so no value on this page is **Rule (verified)**.

This page covers where the Type C cargo tanks go in the hull: the protective location of IGC 2.4, the tank
fit and the `fit` search, and the length the gas code takes off each end of the cargo area. The tank itself (wall,
insulation, dome, fill) is on page 04. The stability criteria are on page 03.

## What OVD computes here

For a gas Option, `ovd run` calls `fit` once (`simulate.rs`, `simulate_gas`, line 950). The user sees:

- **Tank arrangement:** tanks across (`n_tanks_wide`), tanks along (`n_tanks_long`), total tanks, tank outer
  diameter (to the outside of the insulation), overall tank length.
- **Clearances:** side clearance per side (`side_clearance_m`) and bottom clearance (`bottom_clearance_m`), both
  measured to the outer face of the insulation; the double bottom height that follows.
- **d(Vc):** used inside the engine to size the side and bottom clearances. It is not a separate output.
- **End lengths:** bow void, stern void, cargo zone length; the gas separation result (aft separation, forward
  separation, cofferdam, service space) and its three checks.
- **Search result:** payload per journey, what binds it (`volume`, `draft` or `no cargo area`), `feasible`, the
  failed checks of the shown candidate, and the number of candidates tried.
- **"Feasible arrangement" check:** shown only when no candidate passes (`simulate.rs`, lines 978 to 994).

## How it works today

### 1. Protective location of each tank (IGC 2.4)

1. **d(Vc), per tank.** `protective_location.rs`, `d_vc_exact` (lines 126 to 138). Label: **Rule (unverified)**,
   IGC Code 2.4.1.1 (MSC.370(93)).
   - Vc ≤ 1,000 m³: d = 0.8 m
   - 1,000 < Vc < 5,000 m³: d = 0.75 + Vc × 0.2 / 4,000 m
   - 5,000 ≤ Vc < 30,000 m³: d = 0.8 + Vc / 25,000 m
   - Vc ≥ 30,000 m³: d = 2.0 m
2. **Vc of one tank.** Vc = (inner volume of all tanks / number of tanks) + volume of one dome.
   `barge_capacity.rs`, `gas_volumetric`, lines 559 to 561. The inner volume is inside the steel shell (cylinder
   plus two heads, `tanks.rs`, `calculate_tank_arrangement_wide`). The dome is a cylinder:
   V = π/4 × D² × H (`barge_capacity.rs`, `dome_volume_m3`, line 467). Label: **Rule (unverified)** for "per tank,
   domes included" (IGC 2.4.1.1); **Method** for the cylinder dome. The dome counts in Vc only. It is not in the
   cargo volume, the fill or the weights. Appendages (nozzles, manways) add no volume (line 521).
3. **Datum.** IGC 2.4.2 measures the distances plate to plate, moulded line to moulded line, excluding
   insulation. The engine measures its clearances to the outer face of the insulation. So the clear gap it needs
   is d less the insulation thickness. `gas_volumetric`, line 518; `gas_arrangement.rs`,
   `required_side_clearance`, line 252. Label: **Rule (unverified)**, IGC 2.4.2.
4. **Side clearance, type 2G.** cs = max(d − t_ins, 0), per side, in m. `gas_volumetric`, line 530. Label:
   **Rule (unverified)**, IGC 2.4.1.2 ("nowhere less than d").
5. **Bottom clearance, type 2G.** cb = max(min(B/15, 2.0 m), d) − t_ins, with B the moulded beam.
   `gas_volumetric`, line 531. Label: **Rule (unverified)**, IGC 2.4.1.2 with the vertical extent of bottom
   damage of 2.3.1.2.3.
6. **Double bottom.** db = max(0, cb − h_support), with h_support the saddle height (default 0.5 m).
   `gas_volumetric`, line 541. Label: **Method**. With small tanks db is often zero: the saddles stand on the
   bottom shell. Example: B = 11.4 m, Vc = 500 m³, t_ins = 330 mm gives d = 0.80 m, cb = 0.47 m, db = 0.
7. **Circular dependency.** d depends on Vc, Vc depends on the tank size, and the tank size depends on the
   clearance. The engine starts at d = 1.0 m and repeats steps 4 to 6 and the tank fit up to 40 times. It stops
   when d changes by less than 1e-9 m (`gas_volumetric`, lines 509 and 528 to 567). Label: **Fixed in code**.
8. **Other ship types.** `required_side_protection` (lines 142 to 153) also holds type 1G and IGF
   (max(min(B/5, 11.5 m), d)) and type 3G (0.8 m). A Design cannot reach them: `fit` accepts only `IGC_2G`
   (`gas_arrangement.rs`, `RULESETS`, line 42).

### 2. Tank fit in the section and along the hull

1. **Tank outer diameter.** OD = (B − 2·cs − (n_wide − 1)·g_t) / n_wide, with g_t = 1.0 m between tanks
   (`tanks.rs`, lines 99 to 102; `models.rs`, line 189). Label: **Method**; the gap is **Fixed in code**.
2. **Tank length.** L_tank = (L_cargo zone − (n_long − 1)·g_l) / n_long, with g_l = 2.0 m (`tanks.rs`, lines 182
   to 190; `models.rs`, line 190). Heads are hemispherical in this path (`models.rs`, line 194), so each head is
   OD/2 long. Label: **Method**; the gap and the head type are **Fixed in code**.
3. **Tank top.** Cargo top above keel = db + h_support + OD + dome height (`barge_capacity.rs`, line 573). This
   is the obstruction for the wheelhouse sightline and the top for the air draft.
4. **Tanks across, automatic.** Outside `fit` (the design grid, a direct capacity call), the engine takes the
   fewest tanks across that keep OD at or below the cap: n ≥ (B − 2·cs + g_t) / (OD_cap + g_t)
   (`barge_capacity.rs`, `tanks_wide_for_beam`, lines 397 to 432). `fit` always names the count, so it does not
   use this. Label: **Method**.
5. **Tool limits on the tank.** OD ≤ 16.0 m (an input, `max_tank_od_m`) and overall tank length over OD ≤ 7.0
   (`gas_arrangement.rs`, `MAX_TANK_LENGTH_OVER_OD`, line 47; check `length_over_diameter_check`, lines 257 to
   274). Label: **Fixed in code** (the 7.0) and **Estimate** (the 16 m cap, a tool limit with no rule).

### 3. Separation at the ends of the cargo area

`gas_separation.rs`, `compute` (lines 83 to 140). The inputs are the defaults of `GasSeparationInputs`
(lines 34 to 45). A Design cannot change them (`barge_capacity.rs`, line 794).

1. **Aft separation** = max(cofferdam, opening setback) = max(0.60, 2.00) = 2.00 m (line 85). Label: **Method**.
2. **Forward separation** = `fwd_end_bulkhead_m` = 0.50 m (line 86). Label: **Fixed in code**.
3. **Service space** = max(0, aft separation − cofferdam) = 1.40 m. Reported only (line 87).
4. **Checks** (error severity, tolerance 1e-9 m), lines 95 to 129. Label: **Rule (unverified)** for each. Every
   result also carries a warning to confirm the clauses against the text (lines 17 to 18).
   - `cofferdam_min_width`: cofferdam ≥ 0.60 m, cited as "cl. 3.6.13". Finding #280: that clause is in the
     type C (liquid) part of the inland rule book.
   - `opening_setback`: aft separation ≥ 2.00 m, cited as "cl. 3.3.7 / 3.3.11". The text matches: entrances of
     engine rooms, and openable entrances and windows of superstructures and accommodation, not less than 2.00 m
     from the cargo area.
   - `fwd_end_bulkhead_space`: forward separation ≥ 0.20 m (`is_pressure_tank: true`, the default) or 0.50 m,
     cited as "cl. 3.5.3 / 3.6.3". Finding #281: in 3.5.3 (type G) the condition is the tank end shape (0.50 m
     for plane ends), not "pressure" or "standard"; 3.6.3 is a type C clause.
5. **End lengths.** `barge_capacity.rs`, `arrangement` (lines 727 to 808):
   - Bow void = bow rake + forward separation; bow rake = 1.6 × moulded depth (an input).
   - Stern void = stern block length (self-propelled) or 3.0 m (pushed barge, an input) + aft separation.
   - Cargo zone = LOA − bow void − stern void. Check `cargo_zone_positive` (lines 949 to 959).
   - The stern block length comes from the equipment (`aft_arrangement.rs`, `compute`). The tank zone starts
     2.00 m forward of the wheelhouse front.

### 4. Wing tanks

The wing ballast tank width is max(0, 1.0 m − wing tank gap) = 0.4 m with the defaults. It does not follow the
side clearance. `models.rs`, `post_init` (lines 245 to 254) sets it from the 1.0 m default before the clearance
is known; `gas_volumetric` (lines 507 and 553 to 557) keeps it. The width acts only as an on/off switch for the
wing plate, the weather shield and the raised moulded depth in the weights (`weights.rs`, lines 84, 127, 157), and
it sets the drawing. The engine has no wing ballast capacity. The drawing cuts the gap so that no line enters a
tank (`ovd-format/src/ga.rs`, `wing_tank`, lines 410 to 431). Label: **Fixed in code** (a known defect, #283).

### 5. The `fit` search (feasible first, then most cargo)

`gas_arrangement.rs`, `fit` (lines 655 to 782) and `search` (lines 535 to 626).

1. **Candidates.** For each depth, then for each tanks-across count in {1, 2, 3}:
   - A base candidate with the rule clearances of section 1 (`build`, lines 464 to 520).
   - If its OD is zero or above the cap (tolerance 1e-6 m), a cap candidate: cs = (B − n_wide × OD_cap) / 2. It is
     kept only if cs ≥ d(Vc) − t_ins of the cap tank, less 1e-9 m (lines 471 to 517).
   - For each kept candidate, four more with the side clearance raised by 0.5, 1.0, 1.5 and 2.0 m
     (lines 555 to 576). They give narrower, lower tanks.
   - In a Design the depth list is the typed depth only. With `optimise_depth`, it is 3.5 to 7.0 m in 0.25 m steps,
     less the depths whose depth less freeboard is below the draught cap (`simulate.rs`, lines 856 to 881).
   - Tanks along the hull stay at the input (default 2). The search does not vary them.
2. **Payload.** Each candidate is de-rated to the draught cap t_max with no under-keel clearance
   (`derate_candidate`, lines 407 to 422). Page 08 covers the de-rating.
3. **Cheap checks** (lines 294 to 315): every check of the capacity result (geometry, overload, freeboard at the
   volume-bound load, cargo zone, stern block, the three separation checks, the light air draft when a limit is
   set), the journey checks, `tank_length_over_diameter`, and `freeboard_at_payload` (depth less carried draught ≥
   the required freeboard, tolerance 1e-9 m).
4. **Costly check:** stability, IS Code 2008 Part A 2.2, each loading condition (`stability_checks`, lines 350 to
   378). It runs only on candidates that pass the cheap checks, from the most cargo down, and stops at the first
   that passes (lines 600 to 624).
5. **Winner:** the candidate with the most payload that passes every error-severity check. Within 1e-6 t, the
   first built wins. Label: **Method**.
6. **Gate.** Only error-severity checks gate (`checks.rs`, `all_ok`, lines 47 to 49). Warnings never do. The
   wheelhouse sightline check `wheelhouse_sees_over_cargo` is a **warning** (`aft_arrangement.rs`, lines 949 to
   965). So today a failed sightline does **not** remove a candidate. The maintainer decided on 2026-10-20 (#264)
   that it must block `fit`; the code does not do this yet.
7. **No pass.** `fit` returns the candidate with the fewest failed error checks, then the most cargo, with
   `feasible: false` and its failed checks (`closest_to_passing`, lines 630 to 646).
8. **Required side clearance is built in, not checked.** The base candidate gets the rule clearance by
   construction, the raised candidates are wider, and the cap candidate is tested against the rule. No check on
   the result compares the side or bottom clearance with d(Vc). The comment at lines 282 to 284 says the gas
   separation checks carry the protective location; they do not.

### 6. Shortest viable length

`smallest_viable_loa` (lines 819 to 849) scans up from the lower bound in 2 m steps to the first length with a
passing arrangement. It then halves the last step to 0.1 m and rounds to one decimal. Viability is not monotone in
length, so it does not bisect from the start.

### 7. Design grid

`design_grid.rs`, `sweep` (lines 165 to 186) runs the capacity chain for every length and beam. It does not run
`fit`. Each cell has two tanks along, automatic tanks across, and a fixed side clearance of 1.0 m per side
(line 25, line 102). The output names this "IGC-2G protective location" (line 176). Finding #283: IGC has no such
value. A cell is feasible if the capacity checks and the length over diameter check pass. The grid does not check
the clearance against d(Vc), and it does not check stability. It uses the capacity default fill of 0.95
(`barge_capacity.rs`, line 230), not the 0.98 of a Design.

## Assumptions, rules and estimates

Source for all IGC items: Res. MSC.370(93), text as read in the rule-source library. Spec 0004 records that no
later amendment changes 2.3 or 2.4; this page did not re-check that. "Inland rule book" means the Lloyd's Register
Rules for the Classification of Inland Waterways Ships, Pt 4 Ch 4 Sec 3, July 2022, read in an unofficial mirror.

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| GA-A01 | d(Vc) | 0.8; 0.75 + Vc·0.2/4,000; 0.8 + Vc/25,000; 2.0 m | Rule (unverified) | IGC 2.4.1.1. Matches the text as read; no recorded second reader | `protective_location.rs`, `d_vc_exact`, L126–138 | Unknown |
| GA-A02 | Vc basis | Inner volume of one tank + one dome | Rule (unverified) | IGC 2.4.1.1: gross design volume of the individual tank, domes and appendages included | `barge_capacity.rs`, `gas_volumetric`, L559–561; `gas_arrangement.rs`, `build`, L509–511 | Unknown |
| GA-A03 | Appendages in Vc | None added | Fixed in code | Not modelled | `gas_volumetric`, L521 | Non-conservative (Vc low, d low; effect very small) |
| GA-A04 | Dome volume | π/4 × D² × H (cylinder) | Method | Geometry | `barge_capacity.rs`, `dome_volume_m3`, L467 | Unknown |
| GA-A05 | Dome diameter | 2.0 m | Estimate | "Nominal, as wide as high"; no source | `barge_capacity.rs`, L58; profile `tank_dome_diameter_m` | Unknown (small effect on d) |
| GA-A06 | Dome height | 2.0 m | Estimate | Carried over from the original engine | `barge_capacity.rs`, L55; profile `tank_dome_height_m` | Unknown (sets cargo top, air draft, sightline) |
| GA-A07 | Clearance datum | Clear gap = d − insulation thickness | Rule (unverified) | IGC 2.4.2: plate to plate, excluding insulation | `gas_volumetric`, L518, L530–531; `required_side_clearance`, L252 | Unknown |
| GA-A08 | Side clearance, 2G | cs = max(d − t_ins, 0) | Rule (unverified) | IGC 2.4.1.2 | `gas_volumetric`, L530 | Unknown |
| GA-A09 | Bottom clearance, 2G | cb = max(min(B/15, 2.0), d) − t_ins | Rule (unverified) | IGC 2.4.1.2 with 2.3.1.2.3 | `gas_volumetric`, L531 | Unknown |
| GA-A10 | Double bottom height | db = max(0, cb − h_support) | Method | Saddles fill the rest of the bottom clearance | `gas_volumetric`, L541 | Unknown |
| GA-A11 | Saddle height | 0.5 m | Estimate | Carried over from the original engine | `barge_capacity.rs`, L53; profile `tank_support_height_m` | Unknown |
| GA-A12 | d iteration | Start 1.0 m, at most 40 rounds, stop at 1e-9 m | Fixed in code | Contraction argument in the comment | `gas_volumetric`, L509, L528, L563 | None expected |
| GA-A13 | Side clearance, 1G and IGF | max(min(B/5, 11.5), d) | Rule (unverified) | IGC 2.4.1.1 (1G). Not reachable from a Design | `protective_location.rs`, L148 | Not used |
| GA-A14 | Side clearance, 3G | 0.8 m | Rule (unverified) | IGC 2.4.1.3. Not reachable from a Design | `protective_location.rs`, L150 | Not used |
| GA-A15 | Ship types offered | `IGC_2G` only | Fixed in code | Issue #175 | `gas_arrangement.rs`, `RULESETS`, L42 | Not a value |
| GA-A16 | API clearance check | Compares given clearances with d, no insulation term | Method | Reached only through the API, not a Design | `protective_location.rs`, `check_protective_location`, L177–202 | Conservative by the insulation |
| GA-A17 | Insulation thickness | LNG 330 mm (profile); core default 350 mm | Estimate | Page 04 | profile `tank_insulation_thickness_mm`; `barge_capacity.rs`, L65 | Unknown (thicker gives a smaller clear gap) |
| GA-A18 | Tank outer diameter | (B − 2cs − (n−1)·g_t) / n | Method | Geometry | `tanks.rs`, L99–102 | None |
| GA-A19 | Gap between tanks across | 1.0 m | Fixed in code | Carried over; no source | `models.rs`, L189 | Unknown |
| GA-A20 | Gap between tanks along | 2.0 m | Fixed in code | Carried over; no source | `models.rs`, L190 | Unknown |
| GA-A21 | Head type in this path | Hemispherical, head length OD/2 | Fixed in code | Carried over | `models.rs`, L194; `tanks.rs` head geometry | Unknown |
| GA-A22 | Tank OD cap | 16.0 m | Estimate | Tool limit, not a rule | `barge_capacity.rs`, L46; profile `max_tank_od_m` | Unknown |
| GA-A23 | Tank length over OD | ≤ 7.0 | Fixed in code | OVD Check. Same value in ADN 2025 9.3.1.11.1(b) and inland rule book 3.5.1 (type G) | `gas_arrangement.rs`, L47, L257–274 | Unknown |
| GA-A24 | Tanks along the hull | 2 | Estimate | Carried over; an input | `barge_capacity.rs`, L51; profile `tanks_in_line` | Unknown |
| GA-A25 | Automatic tanks across | Fewest n with OD ≤ cap | Method | Inverts GA-A18 | `barge_capacity.rs`, `tanks_wide_for_beam`, L397–432 | None |
| GA-A26 | Tanks across searched | {1, 2, 3}, or the one number of the input `tanks_across` (issue #675) | Fixed in code; input | Python default | `gas_arrangement.rs`, `DEFAULT_TANKS_WIDE`; `simulate.rs` | May miss a better arrangement on a wide hull unless the user fixes it |
| GA-A26a | Least side space | The larger of `min_side_space_m` (default 0) and the IGC 2.4 minimum; the wing tank is it less the wing tank gap. The app warns when the wing tank is 0 | Input | Issue #675 | `gas_arrangement.rs`, `build_or_refuse`; `simulate.rs`, `no_wing_tank_note` | None: a user value |
| GA-A27 | Extra side clearance searched | {0.5, 1.0, 1.5, 2.0} m | Fixed in code | Python default | `gas_arrangement.rs`, L54, L555–576 | May miss a better arrangement |
| GA-A28 | Depth grid | 3.5 to 7.0 m, 0.25 m steps | Fixed in code | Used with `optimise_depth` and the API | `gas_arrangement.rs`, L50; `simulate.rs`, L856–881 | May miss a better depth |
| GA-A29 | Cap candidate clearance | cs = (B − n·OD_cap)/2 | Method | Puts the tank on the cap; ignores the gap between tanks | `gas_arrangement.rs`, `build`, L471 | Conservative (less cargo when n > 1) |
| GA-A30 | Search tolerances | OD over cap 1e-6 m; cap clearance 1e-9 m; tie 1e-6 t | Fixed in code | Python parity | `gas_arrangement.rs`, L461, L515, L608 | None expected |
| GA-A31 | Winner rule | Most payload among passing candidates; first built on a tie | Method | Issue #264, maintainer decision 2026-10-16 | `search`, L600–624 | Not a value |
| GA-A32 | No-pass answer | Fewest failed error checks, then most payload | Method | Issue #264 | `closest_to_passing`, L630–646 | Not a value |
| GA-A33 | De-rating inside `fit` | Under-keel clearance 0; design depth = t_max | Fixed in code | Page 08 | `derate_candidate`, L416–421 | Unknown |
| GA-A34 | Freeboard at payload | ≥ required freeboard (default 0.3 m), tol 1e-9 m | Estimate | Profile `min_freeboard_m` | `cheap_checks`, L303–313; L56 | Unknown |
| GA-A35 | Freeboard at volume-bound load | ≥ 0.3 m always (user value not passed) | Fixed in code | `BargeCapacityInputs::new`; `base_inputs` does not set it | `barge_capacity.rs`, L210, L940–948; `gas_arrangement.rs`, L434–457 | Conservative or non-conservative, by the user value |
| GA-A36 | Only error checks gate | Warnings never gate | Method | `all_ok` | `checks.rs`, L47–49 | Non-conservative where a warning is a real limit (sightline) |
| GA-A37 | Wheelhouse sightline | Warning, not a gate | Fixed in code | Against the maintainer decision on #264 (2026-10-20) | `aft_arrangement.rs`, L949–965 | Non-conservative |
| GA-A38 | Sightline obstruction | Cargo top incl. dome | Method | — | `barge_capacity.rs`, L573, L871–874 | Unknown |
| GA-A39 | Sightline values | Margin 0.5 m, eye 1.5 m above floor | Estimate | Carried over; inputs | `aft_arrangement.rs`, L37–39, L820–826 | Unknown |
| GA-A39a | Wheelhouse lift | ≤ 6.0 m above the lowered floor on the deck (= the light air draft of a gas Design) | Estimate | Carried over; inside the published range of lifting-shaft wheelhouse lifts, 2.5–8.9 m (BÜTER Hebetechnik; 8.8 and 8.9 m rated 12 t, 7.0 m rated 6 t; issue #675). No source gives one value for a gas Design | Defaults Profile `wheelhouse_max_lift_m` | Often decides tanks across. The stroke that fits depends on the hull depth below the deck (the shaft goes into the hull); neither it nor stability with the wheelhouse raised (ES-TRIN 7.12) is checked |
| GA-A40 | Fill ratio in `fit` | Design 0.98; `FitOptions` default 0.95 | Sourced default (Design) | IGC 15.3 (page 04) | `simulate.rs`, `fill_by_ruleset`, L814–816; `gas_arrangement.rs`, L106 | Page 04 |
| GA-A41 | Cofferdam | ≥ 0.60 m | Rule (unverified) | Inland rule book 3.6.13 is a type C clause; finding #280 | `gas_separation.rs`, L12, L96–104 | Unknown (no gas source) |
| GA-A42 | Opening setback | ≥ 2.00 m from the cargo area | Rule (unverified) | Inland rule book 3.3.7, 3.3.11; same in ADN 9.3.1.17.4 | `gas_separation.rs`, L13, L105–114 | Unknown (IGC 3.2.4.1 not checked) |
| GA-A43 | Aft separation | max(cofferdam, setback) | Method | — | `gas_separation.rs`, L85 | None |
| GA-A44 | Service space | max(0, aft − cofferdam) | Method | Reported only | `gas_separation.rs`, L87 | None |
| GA-A45 | Forward separation used | 0.50 m | Fixed in code | `fwd_end_bulkhead_m` default | `gas_separation.rs`, L14, L39, L86 | Conservative against the 0.20 m floor |
| GA-A46 | Forward end space floor | 0.20 m (pressure tank) or 0.50 m | Rule (unverified) | Inland rule book 3.5.3 keys it to plane ends; finding #281 | `gas_separation.rs`, L89–93, L115–128 | Unknown |
| GA-A47 | Pressure tank flag | `true` | Fixed in code | Default | `gas_separation.rs`, L40 | None today |
| GA-A48 | Separation tolerance | 1e-9 m | Fixed in code | Python parity | `gas_separation.rs`, L98, L107, L117 | None |
| GA-A49 | Separation inputs in a Design | Defaults only | Fixed in code | Not in the Defaults Profile | `barge_capacity.rs`, L794 | Not a value |
| GA-A50 | Bow rake | 1.6 × moulded depth | Estimate | Carried over; input | `barge_capacity.rs`, L49, L788–791 | Unknown |
| GA-A51 | Pushed barge stern void | 3.0 m (+ aft separation) | Estimate | Carried over; input | `barge_capacity.rs`, L72, L782–786 | Unknown |
| GA-A52 | Cargo zone | LOA − bow void − stern void > 0 | Method | OVD Check | `barge_capacity.rs`, L833, L949–959 | None |
| GA-A53 | Wing tank width | max(0, 1.0 − gap), not from cs | Fixed in code | Known defect (#283) | `models.rs`, L245–254; `barge_capacity.rs`, L507, L553–557 | Unknown (switch and drawing only) |
| GA-A54 | Wing tank gap | 0.6 m | Estimate | Carried over; input | `barge_capacity.rs`, L68 | Unknown |
| GA-A55 | Wing tank height | 0.66 × OD | Estimate | Carried over; input | `barge_capacity.rs`, L70 | Unknown |
| GA-A56 | Design grid side clearance | 1.0 m per side | Fixed in code | Called "standard IGC-2G spacing"; no such value (#283) | `design_grid.rs`, L25, L102, L176 | Conservative up to about 13,750 m³ per tank (350 mm insulation); non-conservative above |
| GA-A57 | Design grid tanks along | 2 | Fixed in code | — | `design_grid.rs`, L23 | Unknown |
| GA-A58 | Design grid fill | 0.95 | Fixed in code | Capacity default, not the profile | `barge_capacity.rs`, L230 | Conservative (cargo low by about 3 %) |
| GA-A59 | Design grid tug length | 16.0 m | Fixed in code | Not used by any calculation | `design_grid.rs`, L27 | None |
| GA-A60 | Shortest viable length | 2 m scan, halve to 0.1 m, round to 0.1 m | Method | Viability not monotone | `gas_arrangement.rs`, L809, L819–849 | Can round 0.05 m below the last tested viable length |

## Inputs the user can change

Defaults come from the Defaults Profile (`crates/ovd-format/src/builtin_profile.json`) unless noted.

| Input | Default | Unit | Where the default comes from |
|---|---|---|---|
| `ruleset` | `IGC_2G` (only value) | — | IGC 2.4.1.1 per tank, 2.4.2 datum |
| `max_tank_od_m` | 16.0 | m | Tool limit, not a rule |
| `tanks_across` | none (the search tries 1, 2 and 3) | tanks | Issue #675 |
| `tanks_in_line` | 2 | tanks | Original engine |
| `tank_support_height_m` | 0.5 | m | Original engine |
| `tank_dome_height_m` | 2.0 | m | Original engine |
| `tank_dome_diameter_m` | 2.0 | m | Nominal; no source |
| `tank_insulation_thickness_mm` | by Cargo (LNG 330) | mm | Page 04 |
| `wing_tank_gap_m` | 0.6 | m | Original engine |
| `wing_tank_height_ratio` | 0.66 | — | Original engine |
| `min_side_space_m` | 0 (the IGC 2.4 minimum alone) | m | Issue #675; the wing tank is this less `wing_tank_gap_m` |
| `bow_rake_length_to_depth_ratio` | 1.6 | — | Original engine |
| `pushed_stern_void_m` | 3.0 | m | Original engine |
| `min_freeboard_m` | 0.3 | m | Profile (reaches `freeboard_at_payload` only) |
| `optimise_depth` | false | — | Profile |
| `sightline_margin_m`, `wheelhouse_eye_height_m` | 0.5, 1.65 | m | Original engine; ES-TRIN 7.02 eye height (#347) |
| `wheelhouse_max_lift_m` | 6.0 | m | Estimate inside the published range 2.5–8.9 m (BÜTER Hebetechnik); above the lowered floor on the deck (GA-A39a, #675) |

Not changeable in a Design: cofferdam, opening setback, forward end space, the pressure-tank flag, the extra
clearance the search adds, the gaps between tanks, the head type.

The result lists the arrangements the search tried (`fit_tried`, outside the results hash; #675): one row per depth
and tanks across, with its outcome (chosen, less cargo, the failing checks, or why no tank was built). The app shows
it as "Arrangements tried". A gas answer with no wing ballast tank gets the warning `warn-fit-no-wing-tank`. The JSON evaluation API accepts the
search sets as `fit` arguments (`ovd-format/src/gas_arrangement.rs`, `FIT_OPTION_KEYS`).

## Limits and known findings

- **Wheelhouse sightline does not gate `fit`.** The maintainer decided on #264 that it must. Not built.
- **#280:** the 0.60 m gas cofferdam cites a type C clause. Decided: fix with the IGC chapter 3 replacement of
  `lr-inland-gas`.
- **#281:** the forward end space is keyed to "pressure tank", not to the tank end shape. Decided: fix right
  after #272.
- **#283:** the 1.0 m "standard IGC-2G spacing" of the design grid, and the wing tank width taken from a 1.0 m
  default. Decided: remove the IGC wording in #271; derive the spacing from the side-clearance Rule in #277.
- **No check on the clearance of the result.** The side and bottom clearances meet IGC 2.4.1.2 by construction
  in `fit`. The design grid and a direct capacity call with a side clearance override can go below d(Vc) − t_ins
  with no failed check.
- **Not modelled:** IGC 3.2.4.1 (entrances at 4 % of L, at least 3 m, from the deckhouse front); IGC 2.4.4 (no
  cargo tank forward of the collision bulkhead: the hull has no collision bulkhead); IGC 3.5.3.5 inspection
  clearances (450 mm from a curved tank to a shell with structure); the inland 380 m³ tank capacity limit
  (inland rule book 3.2.1, ADN 2025 9.3.1.11.1(a)); the inland hull design of 3.5.2 (removed in #215).
- **Geometry limits:** box section with vertical sides; d is checked at the side and the bottom only, not at the
  turn of bilge or the deck; one head type; tanks of one size.
- **Two freeboard checks** with different limits in `fit` (GA-A34, GA-A35).
- **The design grid** has no stability check and no protective location check, so its "feasible" is weaker than
  `fit`'s.
- Register of findings: #121.

## Evidence

- **Unit tests** (`crates/ovd-core/tests/`):
  - `protective_location.rs`: d(Vc) on both sides of 1,000, 5,000 and 30,000 m³; the clear gap is d less the
    insulation; side and bottom per ship type.
  - `dome_in_vc.rs`: the default dome holds 2π m³; the side clearance is d of the tank with its dome less the
    insulation; the dome is not in the cargo volume.
  - `tank_datum.rs`: the clearances are measured to the steel whatever the insulation.
  - `gas_separation.rs`: the constants, the golden defaults, each check failing alone, the 1e-9 m tolerance.
  - `gas_arrangement_feasible.rs`: the most-cargo arrangement fails stability and a smaller passing one wins;
    nothing passes gives `feasible: false`; no feasible result has a failed check; the air draft limit and the
    payload freeboard remove candidates; the shortest viable length has a passing arrangement.
  - `gas_arrangement.rs`: a 90 × 14.8 m LPG benchmark at 6.5 m depth (2 tanks, 1 across, 15 candidates, payload
    pinned); the side-clearance rule is never below 0.45 m with 350 mm insulation; no feasible fit has a tank over
    7 diameters.
  - `design_grid.rs`: two tanks along, no tank over 7 diameters in a feasible cell.
  - `fit_choices.rs` (#675): a fixed tanks across is the only one tried, or the answer is infeasible with its
    failing checks; a least side space is kept by every candidate and gives a wing tank; each arrangement tried has
    one outcome; a failing blind distance names the lift needed.
- **Format tests** (`crates/ovd-format/tests/fit_choices_inputs.rs`, #675): the two inputs leave a file saved
  before as it was; they reach the search through a Design; the no-wing-tank warning; the list is outside the
  hash; the lift default and its datum.
- **Design impact of #264:** 47 of 135 designs moved (all gas). 42 now pass with a smaller arrangement, 5 stay
  infeasible, none that passed before changed (issue #264, build notes).
- **Evaluation harness** (`evaluation/results.md`): covers IS Code Part A 2.2 on box sections only. It has no
  case for d(Vc), the clearances, the separation or `fit`.
- **Calibration** (`calibration/results.md`): the catalogue holds gas barges with published capacity, but the
  calibration path has no gas hull model. No arrangement is calibrated against a built gas vessel.

These tests check that the code does what it says. None checks the arrangement against a built vessel or an
independent oracle.

## Questions for reviewers

1. Is "d less the insulation thickness" the right reading of IGC 2.4.2 for a Type C tank with external
   insulation, for both the side and the bottom?
2. Should the bottom clearance be measured to the lowest point of the tank (as now), or does the 2.3.1.2.3 value
   at the ship's centreline need a separate check when tanks sit off the centreline (two or three across)?
3. With small tanks the double bottom often comes out as zero (saddles on the bottom shell). Is that a credible
   concept for an inland or coastal Type C gas barge, or should a least double bottom be an input?
4. Is a nominal cylindrical dome of 2.0 × 2.0 m fair for Vc and the cargo top on tanks of 200 to 2,000 m³?
5. Are 1.0 m between tanks across and 2.0 m between tanks along fair concept values? Should IGC 3.5.3.5 (450 mm
   and 380 mm) set a lower bound on the side and bottom clear gaps?
6. Should IGC 3.2.4.1 (entrances at least 4 % of L, not less than 3 m, from the deckhouse front) replace or join
   the 2.00 m opening setback for a coastal Service Area?
7. Is the 2.00 m aft separation right for a pushed gas barge that carries no accommodation or engine room?
8. For European inland service, should the 380 m³ tank capacity limit of ADN 9.3.1.11.1(a) apply, with the 9.3.4
   alternative as an input?
9. Should `fit` also vary the number of tanks along the hull (1 to 4), not only the count across?
10. Is it right that `fit` also requires the volume-bound (tanks full) load to keep 0.3 m freeboard when the
    payload is limited by draught?
11. What wheelhouse lift should a new Design assume (GA-A39a)? Published lifting-shaft lifts have strokes of 2.5
    to 8.9 m, and the stroke that fits depends on the hull depth below the deck. Should the default stay 6.0 m,
    take a published stroke (for example 8.8 m, rated 12 t), or follow the moulded depth by a rule (#675)?
