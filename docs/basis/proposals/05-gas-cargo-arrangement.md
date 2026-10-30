# Proposals: 05 Gas cargo arrangement

Not published. Companion to `docs/basis/05-gas-cargo-arrangement.md`. Facts about the code are on that page;
this file holds gaps, errors and proposed changes. Each error gives the code location and the source location.
Errors go to the findings register (#121) only after maintainer triage.

## Errors

### E1. The wheelhouse sightline does not block `fit`

- **Code:** `crates/ovd-core/src/aft_arrangement.rs`, `compute`, lines 949 to 965: `wheelhouse_sees_over_cargo`
  has severity `"warning"`. `crates/ovd-core/src/checks.rs`, `all_ok`, lines 47 to 49, ignores warnings.
  `gas_arrangement.rs`, `search`, line 588 and line 611, gate on `all_ok`.
- **Source:** maintainer decision on #264 (comment, 2026-10-20): the sightline blocks `fit`; a candidate that
  fails it is not feasible. Spec 0004 still lists it as an open question ("How `fit` uses it").
- **Effect on results:** large. The #264 build notes say the sightline fails on many designs. Gating it lowers
  cargo or makes more hulls infeasible. The `extra_clearance_m` candidates (lower tanks) will win more often.

### E2. The wing tank width ignores the side clearance

- **Code:** `crates/ovd-core/src/models.rs`, `post_init`, lines 245 to 254 set `side_clearance = 1.0` and
  `wing_tank_width = 1.0 − wing_tank_gap` when both are empty. `barge_capacity.rs`, `gas_volumetric`, line 507 runs
  `post_init` on the base inputs first, so each round (lines 553 to 557) copies the 0.4 m width and only changes
  the clearance.
- **Source:** the code's own comment (`models.rs`, line 53: "auto (side_clearance - gap)") and the Profile basis
  of `wing_tank_gap_m` ("the wing tank width is the side clearance less this gap"). Finding #283 notes it.
- **Effect on results:** small today. The width is only a switch (> 0) in `weights.rs` (lines 84, 127, 157) and a
  value for the drawing. With cs = 0.47 m, width 0.4 m plus gap 0.6 m is 1.0 m: the wing tank overlaps the
  cargo tank, and `ga.rs` (`wing_tank`, lines 410 to 431) cuts the drawn gap to hide it. If the width followed cs,
  it would be max(0, 0.47 − 0.6) = 0: the wing plate and the weather shield would leave the lightship for most
  small-tank designs. That is a weight change (page 02), not a ballast change.
- **Note on #283's impact estimate:** it says ballast capacity moves. The engine has no wing ballast capacity, so
  only the weights switch and the drawing move.

### E3. Gas cofferdam 0.60 m cites a type C clause (#280)

- **Code:** `gas_separation.rs`, line 12 and lines 96 to 104.
- **Source:** inland rule book Pt 4 Ch 4 Sec 3, 3.6.13 (in 3.6, "Special requirements for Type C tankers").
  The type G part, 3.5.3, asks for an A-60 bulkhead and an end space instead. ADN 2025 9.3.1.11.3(a) is the same.
- **Effect:** none on geometry with the defaults (the 2.00 m setback governs aft). Decided: fix with the IGC
  chapter 3 replacement of `lr-inland-gas`.

### E4. Forward end space keyed to "pressure tank" (#281), and the length used is 0.50 m

- **Code:** `gas_separation.rs`, lines 14 to 15, 39 to 40, 86, 89 to 93, 115 to 128.
- **Source:** inland rule book 3.5.3: at least 0.20 m, 0.50 m where the tanks have plane end bulkheads. The
  "pressure tank" wording comes from 3.6.3, a type C clause.
- **Extra point:** the length taken off the cargo area is `fwd_end_bulkhead_m` = 0.50 m (line 86), not the
  0.20 m floor. The #281 recommendation says the default "already applies 0.20 m"; that is the check floor only.
  Its conclusion (no length moves) still holds, because the 0.50 m length does not change.
- **Effect:** none on geometry today; every gas results hash moves through the message text when fixed.

### E5. Design grid: 1.0 m "standard IGC-2G spacing" (#283), no clearance check, 0.95 fill

- **Code:** `design_grid.rs`, line 25 (`SIDE_CLEARANCE_M`), line 102 (override), line 176 (`clearance_basis`).
  The cell uses `BargeCapacityInputs::new` (fill 0.95, `barge_capacity.rs`, line 230; insulation 350 mm).
- **Source:** IGC 2.4.1.2 sets the side distance to d(Vc) of 2.4.1.1; there is no 1.0 m value.
- **Effect:** with 350 mm insulation, 1.0 m meets d − t_ins up to d = 1.35 m, that is Vc ≤ 13,750 m³ per tank.
  Below that the grid is conservative (narrower tanks, less cargo); above it the grid is non-conservative and no
  check fails. The 0.95 fill puts grid cargo about 3 % below a Design at 0.98. Medium.

### E6. Provenance of the length-over-diameter limit

- **Code:** `gas_arrangement.rs`, lines 43 to 47, says the 7.0 comes from the removed `ADN_G` ruleset and that no
  IGC paragraph is cited. Spec 0004 files it in `ovd` as a tool limit.
- **Source:** the inland rule book that `lr-inland-gas` already cites gives the same limit for type G: 3.5.1,
  ratio of cargo tank length over diameter not to exceed seven. ADN 2025 9.3.1.11.1(b) applies it to pressure tanks.
- **Effect:** none on numbers. It changes the label: the limit can be a cited Rule in `lr-inland-gas` (unverified
  until a second reader checks it), not an OVD tool limit.

### E7. The check list comment claims a protective location check that does not exist

- **Code:** `gas_arrangement.rs`, lines 282 to 284: "the gas separation (protective location clearances)".
  `gas_separation.rs` has three checks (cofferdam, opening setback, forward end space); none is a clearance check.
  `barge_capacity.rs`, `compute`, adds no clearance check either.
- **Source:** spec 0004 plans `igc/2.4.1.2/side-clearance` as a `sizes` Rule that "is still checked on the result".
- **Effect:** none in `fit` today (the clearance is built in). It hides that a side clearance override (design
  grid, direct API) is never checked.

### E8. Two freeboard checks with different limits inside `fit`

- **Code:** `gas_arrangement.rs`, `cheap_checks`, line 300 takes every check of the capacity result, including
  `freeboard_ok` at the volume-bound load (`barge_capacity.rs`, lines 940 to 948) with `required_freeboard` = 0.3 m.
  `base_inputs` (lines 434 to 457) never passes the user's freeboard to the capacity inputs. The user's value
  reaches only `freeboard_at_payload` (lines 303 to 313). `simulate.rs`, `capacity_checks` (lines 368 to 381)
  hides `freeboard_ok` from the result, so the user does not see the check that removed a candidate.
- **Source:** code intent: `FitOptions::required_freeboard_m` (line 93 to 95) says a candidate fails
  `freeboard_at_payload`; nothing says the full-tank condition also gates.
- **Effect:** unknown, possibly large for deep-loaded hulls. A draught-bound candidate whose full tanks would
  leave less than 0.3 m freeboard is removed. A user freeboard above 0.3 m is not applied to the full-tank load.

### E9. `smallest_viable_loa` can return an untested length

- **Code:** `gas_arrangement.rs`, line 848, `round_to(hi, 1)`. `hi` is viable; the rounded value can be up to
  0.05 m shorter and is not tested.
- **Source:** the doc comment (lines 816 to 817): "The length it returns always has a passing arrangement."
- **Effect:** small.

### E10. The cap candidate ignores the gap between tanks

- **Code:** `gas_arrangement.rs`, `build`, line 471: cs = (B − n_wide × OD_cap) / 2. With n_wide > 1 the tanks
  also need (n_wide − 1) × 1.0 m of gap, so OD comes out at OD_cap − (n_wide − 1)/n_wide m, not on the cap.
- **Source:** the comment on line 469 ("put the tank ON the cap").
- **Effect:** small, conservative (less cargo on wide hulls with tanks at the cap).

### E11. The API clearance check uses a different datum

- **Code:** `protective_location.rs`, `check_protective_location`, lines 177 to 202, compares the given side
  clearance and `double_bottom + support` with d, with no insulation term. `barge_capacity.rs` measures to the
  insulation face and needs d − t_ins.
- **Source:** IGC 2.4.2 (plate to plate, excluding insulation).
- **Effect:** small, conservative. Reached only through the JSON API, not a Design.

## Gaps

- **G1. IGC 3.2.4.1, entrances and openings.** Entrances, air inlets and openings to accommodation, service and
  machinery spaces must not face the cargo area. On the side of the deckhouse they must be at least 4 % of L, not
  less than 3 m (and need not exceed 5 m), from the end facing the cargo area. OVD models only the 2.00 m inland
  setback, measured from the cargo area. The stern block layout (`aft_arrangement.rs`) has no openings.
- **G2. IGC 2.4.4, collision bulkhead.** Cargo tanks shall not be forward of the collision bulkhead. The hull
  has no collision bulkhead; the bow void is the rake plus 0.50 m.
- **G3. IGC 3.5.3.5, inspection clearances.** 450 mm between a curved tank surface and a shell or bulkhead with
  structure; 380 mm to the free edge of structure; 150 mm from a dome to deck structure. With d = 0.8 m and
  insulation over 350 mm the clear side gap is below 450 mm, and frames reduce it further. Not checked.
- **G4. Inland tank capacity limit.** Inland rule book 3.2.1 (Table 4.3.1) and ADN 2025 9.3.1.11.1(a): at most
  380 m³ per cargo tank when L × B × H > 3,750 m³ (less for smaller hulls), unless an alternative construction is
  proven. `lr-inland-gas` holds only the separation clauses, so a European inland design can carry tanks of
  several thousand m³ with no flag.
- **G5. Inland hull design (3.5.2, ADN 9.3.1.11.2).** Double hull 0.80 m and double bottom 0.60 m, or a
  stiffened single hull with 0.80 m and 0.60 m to the tanks. Removed in #215 (the measure differed). If
  `lr-inland-gas` stays in the default gas stack, these should at least be `not evaluated` rows.
- **G6. Inland rule book 3.3.2 and 3.3.3.** Cargo pumps and compressors on deck at least 6.00 m from openings
  (3.00 m with a transverse bulkhead). The cargo manifold and compressor positions are not modelled; this may
  set the aft separation.
- **G7. Aft end space.** The tank-to-hold-end-bulkhead space (3.5.3) is checked only forward. Aft it is covered
  by the 2.00 m aft separation. If the aft separation is reduced, nothing checks it.
- **G8. Geometry of the protective location.** d is checked at the vertical side and the flat bottom only. No
  turn of bilge, no flare, no check at the deck or for tanks above the deck.
- **G9. One head type.** The arrangement path always uses hemispherical heads (`models.rs`, line 194). The
  #281 fix keys the end space to `tank_head_type`, which a Design cannot set today.
- **G10. Search space.** `fit` does not vary the tanks along the hull, and searches only 1 to 3 tanks across and
  four fixed extra clearances.
- **G11. Design grid is weaker than `fit`.** No stability, no clearance check, different fill. Spec 0004 (#277)
  plans to put the sweep on the Rule Trace.
- **G12. No evidence for the arrangement.** No harness case for d(Vc), clearances, separation or `fit`; no
  calibration against a built gas barge (the catalogue has pressure barges with capacity only).
- **G13. Glossary terms missing.** Protective location; clear gap (side gap, bottom gap); cargo zone; bow void;
  stern void; stern block; end space; opening setback; cofferdam; wing tank; candidate (of `fit`). The spec uses
  `tank.side_gap`; the code uses "side clearance". One term is needed.
- **G14. Other ship types.** IGC 1G, 3G and IGF clearances exist in code but no Design reaches them.

## Changes

| # | Change | Fixes | Effect on results | When |
|---|---|---|---|---|
| C1 | Correct the comments: the check list (E7), the L/D provenance (E6), the cap comment (E10). No code change | E6, E7 | None | Now |
| C2 | Make the sightline block `fit`: error severity in `fit`'s list, or a named gate. Keep it a warning in a plain capacity run if wanted | E1 | Large | Next; it is a decided change |
| C3 | Set the wing tank width from the converged cs (clear the width before each round, or compute it in `gas_volumetric`) | E2 | Small to medium (weights) | With #277 and #283 |
| C4 | Pass the user's freeboard to the capacity inputs in `base_inputs`, and decide whether the full-tank condition gates. Show the check that removed a candidate | E8 | Unknown; may be large | Before #277, with a design-impact table |
| C5 | Key the end space to the tank end shape; make `tank_head_type` reachable from a Design; drop the 3.6.3 citation; use the floor as the length or keep 0.50 m as a declared input | E4, G9 | None on geometry; hashes move | Right after #272 (decided) |
| C6 | Replace `lr-inland-gas` separation clauses with IGC chapter 3 Rules: 3.1.1, 3.1.2, 3.2.1, 3.2.4.1 (with a deckhouse opening Measure), and remove the 0.60 m cofferdam Rule | E3, G1 | Large, unknown until codified | With #280 (decided) |
| C7 | Add a `sizes` Rule `igc/2.4.1.2/side-clearance` and a bottom Rule, checked on the result; drive the design grid from them | E5, E7, G11 | Medium on the design grid; none on `fit` | #271 (wording), #277 (values) |
| C8 | Add IGC 3.5.3.5 inspection clearances as Rules, `warn` first | G3 | None at `warn`; medium if `block` | After C7 |
| C9 | Move the L/D 7 limit into `lr-inland-gas` as a cited Rule (3.5.1), keep the `ovd` tool limit if wanted | E6 | None | With #272 |
| C10 | Add the inland 380 m³ capacity limit and the 3.5.2 hull clauses as `not evaluated` rows now, and as Rules in an optional ADN set later | G4, G5 | None now; large when ADN is selected | With #272 (rows), ADN set later |
| C11 | Round `smallest_viable_loa` up, or test the rounded value; put the cap tank on the cap with the gaps counted | E9, E10 | Small | Any time, with a design-impact table |
| C12 | Harness cases: d(Vc) at 1,000, 5,000, 30,000 m³ on each side; clear gap with and without insulation; each separation check at its limit; one `fit` case with a known answer | G12 | None (tests) | With #278 |
| C13 | Collision bulkhead position and its IGC 2.4.4 check | G2 | Small to medium (bow void) | With the bow arrangement work |
| C14 | Add the glossary terms of G13 | G13 | None | Now |

**Order of work.** C1 and C14 now (no result change). C2 next: it is decided and moves many results, so it
should land on its own with one design-impact table. Then C4, so that the later rule tickets start from a clean
check list. Then the spec 0004 tickets: C9 and C10 with #272, C5 right after #272, C7 and C3 with #271 and #277,
C12 with #278. C6 and C8 follow as the IGC chapter 3 work. C11 and C13 when their areas are next touched.
