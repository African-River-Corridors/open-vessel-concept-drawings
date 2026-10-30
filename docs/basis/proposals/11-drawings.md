# Proposals: 11 Drawings

Not published. Gaps, errors and proposed changes for `docs/basis/11-drawings.md`. Errors are places where
the drawn geometry disagrees with the computed result. They become findings for the register (#121) after
maintainer triage.

## Errors

### E1. The hopper slope in the section is not the slope the volume model uses

- **Code:** `crates/ovd-core/src/barge_capacity.rs`, `bulk_volumetric`: the hold is a trapezoid of height
  `h_hold = (depth − double bottom) + coaming`. The top width `w_top = beam − 2 × wing void` is at the
  coaming top. `w_bot = w_top − 2 × slope × h_hold`. Area = `0.5 × (w_top + w_bot) × h_hold`.
- **Drawing:** `crates/ovd-format/src/ga.rs`, `midship_section`: the slope runs from `(w_top / 2, depth)`
  to `(w_bot / 2, inner bottom)`, and the coaming stands vertical at `w_top` above the deck.
- **Example** (fixture `dry_bulk_sloped_hopper.vdesign`: 76.5 × 11.4 × 3.5 m, defaults): `w_top` 9.4 m,
  `h_hold` 3.3 m, `w_bot` 2.8 m. Computed section area 20.13 m². Drawn slope 1.43 H:V, not the input 1.0.
  Drawn hold area 14.03 m² below the deck plus 9.40 m² inside the coaming = 23.43 m².
- **Effect:** the drawing shows about 16 % more hold section than the capacity uses. A reviewer who scales
  the section gets a different capacity.
- **Question:** which is right? See reviewer question 5 on the page. If the volume model is wrong, the
  finding belongs to area 06 too.

### E2. The wing tank top is cut at the deck, but the weights build the hull up to it

- **Code:** `crates/ovd-core/src/weights.rs` (wing plate block): wing tank top = tank bottom +
  `wing_tank_height_ratio` × OD. When it is above the deck, the weights add side plating from the deck up
  to it over the full LOA, a full-beam wing plate at that level, and take the moulded depth for the
  top-down steel as `max(depth, wing tank top)`. The weather shield (6 mm, 1.5 m above the tank top, over
  the tank zone) is also weighed and costed (`crates/ovd-format/src/cost.rs`).
- **Drawing:** `ga.rs`, `Hull::wing_tank`: `top = (tank bottom + ratio × OD).min(depth)`. The hull side
  stops at the depth. No wing plate, side extension or weather shield is drawn.
- **Example** (website example `lrg-gas-89.vdesign`): depth 5.70 m, tank OD 9.55 m, tank bottom 0.50 m.
  Wing tank top = 0.50 + 0.66 × 9.55 = 6.80 m. The weights carry 1.10 m of side extension each side over
  the LOA; the drawing shows the hull side at 5.70 m.
- **Effect:** the drawing hides steel that the lightship and the cost include. Non-conservative for a
  reader who checks the arrangement against the weights.

### E3. The wing tank width is always 0.4 m (core), so the drawn void does not match the clearance

- **Code:** `crates/ovd-core/src/barge_capacity.rs`: `base.post_init()` runs before the clearance loop.
  At that time `side_clearance` is `None`, so `models.rs` `post_init` sets it to 1.0 m and
  `wing_tank_width = 1.0 − wing_tank_gap = 0.4 m`. Inside the loop `bi.post_init()` runs again, but
  `wing_tank_width` is already set, so it never follows the real clearance.
- **Drawing:** `ga.rs`, `Hull::wing_tank` draws width 0.4 m and cuts the gap to `side clearance − 0.4`.
  When the clearance is more than 1.0 m, an unnamed space sits between the void and the tank. When it is
  less (0.95 m in `lrg-gas-89`), the drawn void (0.55 m) is narrower than the input gap (0.6 m).
- **Effect:** the drawing shows a core value faithfully, but the core value is wrong. Small effect on
  weights through the wing plate cut-outs. Owner: area 05 (and area 02 for the weights).

### E4. The dome is drawn wider than the dome the volume model uses, and the views disagree

- **Code:** `crates/ovd-core/src/barge_capacity.rs`: the dome is a cylinder of `tank_dome_diameter_m`
  (default 2.0 m, no source) and `tank_dome_height_m` (default 2.0 m). Its volume counts in Vc.
- **Drawing:** `ga.rs`: the dome is a curve with rise = dome height. Its base is 0.30 × OD in the section
  (`DOME_HALF_BASE_OF_OD`) and 2 × max(0.06 × tank length, 1.2 m) in the elevation
  (`DOME_HALF_BASE_OF_LENGTH`, `DOME_HALF_BASE_MIN_M`). The dome diameter is not in the `drawing` object.
- **Example** (`lrg-gas-89`): section base 2.87 m; elevation base at least 2.4 m; model 2.0 m.
- **Effect:** none on results. The drawing misstates the dome and the two views give two widths.

### E5. `ga_adapter.rs` gives a tank length that disagrees with `tanks.rs` (no effect on drawings)

- **Code:** `crates/ovd-core/src/ga_adapter.rs`, `tank_result`: `cylinder = volume per tank / (π r²)`,
  then `overall length = cylinder + 2 × head`. The cylinder already holds the whole volume, so the head
  volume counts twice and the length is too long. `crates/ovd-core/src/tanks.rs` takes
  `overall length = (cargo zone − gaps) / n_long` and `cylinder = overall − 2 × head`.
- **Effect:** none on the drawings: the view builder does not call `ga_adapter`. It feeds the parity
  harness (`crates/ovd-format/src/eval.rs`) and `crates/ovd-core/tests/design_grid.rs`. Risk: a reader
  takes it for the drawing path.

## Gaps

- **G1. Weather shield not drawn** although weighed and costed (see E2).
- **G2. Forward separation and service space not drawn.** `drawing.separation` carries
  `fwd_separation_m` and `service_space_m`; only the aft separation and cofferdam are drawn.
- **G3. Stern block machinery not drawn.** `aft_arrangement` gives the engine room, gas valve unit room and
  generator sets (`engine_room_x0_m`, `gvu_room_x0_m`, `genset_offsets_m` and sizes). None is drawn.
- **G4. Raised wheelhouse not drawn.** The code says the only chain line type stands for the light
  waterline. A different layer or a note could carry it.
- **G5. No air draft or ballast waterline.** The results give the air draft (laden and light) and
  `ballast_draft_m`; the drawings show neither a top-of-structure height nor the ballast waterline.
- **G6. One hold only.** `n_holds` is an input; the drawing shows one hold with no hold bulkheads.
- **G7. Container hold drawn as an empty box hold.** No container stowage is drawn.
- **G8. Bulk wing voids not named.** The space between the hull side and the hold is not labelled.
- **G9. No title block, drawing number, revision or Option name** on the sheet. Spec 0003 defers this.
  ADR 0011 asks every result to say what it is: the status line says "concept", but not the Rule Sets or
  the calibrated range.
- **G10. No common scale.** Each view picks its own scale. A draughtsperson cannot scale between views.
- **G11. The "midship section" is not cut at amidships.** It is a typical section through a tank or the
  hold.
- **G12. Saddles appear only in the section.** No saddle positions along the tank.
- **G13. Duplicated constants.** The tank gaps (2.0 m along, 1.0 m across) come from `BargeInputs::new`
  defaults in `ga.rs`, and the heads are always hemispherical. The core reads the same defaults today. If
  either becomes an input, or a head type other than `hemisphere` is used, the drawing drifts with no
  error.
- **G14. Tests do not check drawn tank, hold or dome sizes** against the core's `TankResult` or hold
  geometry. They check the five main dimensions and the positions read from the result.
- **G15. Dead stern rake path.** `ga.rs` reads `geometry.stern_rake_length_m`, which `simulate` never
  writes, so the stern is always square.
- **G16. Missing glossary terms** (`GLOSSARY.md`): Side Elevation, Plan, Midship Section, Cargo Zone,
  Cofferdam, Cradle (and whether Saddle is the same), Dome, Wing Ballast Tank, Stern Block (the code says
  "aft block" and "aft arrangement"), Light Waterline. Also: `GLOSSARY.md` defines Moulded Depth twice.

## Changes

| # | Change | Fixes | Effect on results | Order |
|---|---|---|---|---|
| C1 | Add the glossary terms of G16; remove the duplicate Moulded Depth entry. | G16 | None | 1 |
| C2 | Put the core's overall tank length, cylinder length, head type and the two tank gaps in `drawing`; draw from them; add a test that drawn tank length equals `TankResult.overall_tank_length`. | G13, G14 | None | 1 |
| C3 | Decide E1 (reviewer question 5). Then draw the hopper to the volume model (slope from the inner bottom to `w_top` at the coaming top), or change the volume model. Add a test: drawn hold area = computed area. | E1 | None (drawing fix) or small to large on bulk capacity (model fix) | 2 |
| C4 | Draw the wing tank top unclamped, with the side extension and wing plate, and draw the weather shield outline (dashed) when its thickness is above zero. | E2, G1 | None | 2 |
| C5 | Put `tank_dome_diameter_m` in `drawing`; draw the dome as a rectangle of that diameter and height in every view; remove the `DOME_*` constants. | E4 | None | 2 |
| C6 | Set `wing_tank_width` from the final side clearance in the core (re-derive it inside the clearance loop). Coordinate with area 05. | E3 | Small (wing plate weight, ballast) | 3 |
| C7 | Mark `ga_adapter.rs` as parity-only in its doc comment, or fix `tank_result` to take the length from the cargo zone as `tanks.rs` does. | E5 | None | 3 |
| C8 | Draw the forward separation, service space, engine room, gas valve unit room and generator sets from the results. | G2, G3 | None | 4 |
| C9 | Draw the air draft (a top-of-structure dimension) and the ballast waterline; draw hold bulkheads for `n_holds`. | G5, G6 | None | 4 |
| C10 | A GA sheet with a title block, Option name, Rule Sets and scope statement, and the three views at one scale (spec 0003 "later"). | G9, G10 | None | 5 |
| C11 | Cut the section at amidships, or label it "TYPICAL SECTION"; show saddle positions in elevation. | G11, G12 | None | 5 |
| C12 | Remove the stern rake read from `ga.rs`, or add a stern rake to the core and write it to `geometry`. | G15 | None (remove) or small (core) | 5 |

**Order of work:** C1 and C2 first: they are cheap and guard against silent drift. Then the three
drawing errors (C3 after the decision, C4, C5). Then the core fix C6 with area 05, and C7. Then the
missing items (C8, C9). The sheet and section changes (C10 to C12) come last, with the GA sheet work that
spec 0003 defers.
