# Drawings

> Concept and preliminary design only. Not for approval. Draft: needs expert review.

**For:** naval architect, draughtsperson  ·  **Code:** `crates/ovcd-format/src/ga.rs` (view builder),
`crates/ovcd-draw/` (Drawing Format Standard, Rust), `crates/ovcd-format/src/simulate.rs` (the `geometry`
and `drawing` result objects), `crates/ovcd-cli/src/draw.rs`  ·  **Rule Sets:** none

## What OVCD computes here

OVCD computes no engineering value for a drawing. It draws the values that the calculation already gives.
For each Option that runs, it makes three drawings (General Arrangement views, spec 0003):

- **Side elevation** (`side-elevation`): hull profile, deck, waterlines, cargo containment, zones, the stern
  block of a self-propelled hull, labels for draught and freeboard.
- **Plan** (`plan`): hull outline, hold opening or tank outlines, zones, the stern block, centre line.
- **Midship section** (`midship-section`): hull section, the hold shape or the tank circles with the double
  bottom, wing ballast tanks, voids, cradles and domes, waterlines, clearances.

Each drawing is one drawing document (JSON) of the Drawing Format Standard. `ovcd-draw` turns it into
normalised content, then SVG, PDF and DXF.

A convoy, or an Option that cannot be computed, gives no drawings. The message is the same as the outputs
panel (`ga_views_of`, `ga.rs`).

## How it works today

1. **Read the results.** `read_hull` (`ga.rs`) reads the `simulate` result object: `dims` (LOA, beam,
   depth), `results` (draught, light draught), `geometry` (voids, cargo zone, bow rake, tank or hold size,
   clearances), `arrangement` (tanks across and along), `drawing` (double bottom, support height, dome,
   wing tank, coaming, gas separation, stern block) and `checks`. LOA, beam, depth and draught must be
   positive, or the drawing stops with an error. A gas value that is `null` is left out of the drawing.
   Label: Method.
2. **Build the geometry in model metres.** Each view puts lines, arcs, circles, polylines, a hatch,
   dimensions and leaders on the standard layers. Origin: x = 0 at the stern, along the hull; y = 0 at the
   keel (elevation, section) or at the centre line (plan). Label: Method.
3. **Choose the scale.** `document` (`ga.rs`) tries the ratios 1:1 to 1:100000 in order (the preferred
   ratios of standard clause 7.3.1 and the house ratios 1:25, 1:250, 1:1250, 1:2500 of clause 7.3.4). It
   takes the first ratio at which the window fits the frame. The window is the extent of all geometry,
   labels and dimensions, with 3 mm of room round it and 26 mm for the scale bar. The document then states
   `scale: fit`, and the standard resolves it again (clause 7.5.2). Each view gets its own scale. Label:
   Method.
4. **Dimensions.** A dimension is drawn only if its line is at least 8 mm long on paper. If not, the value
   goes in a leader label. The standard computes the dimension value from the geometry (clause 12), so the
   figure equals the result. Label: Method.
5. **Labels.** Labels stand in one column: to the right of the bow (elevation, plan) or to the left of the
   hull (section). `label_column` spreads the labels 6 mm apart, as near their targets as possible, and
   swaps two labels whose leaders cross. Label: Method.
6. **Notes and status.** The bottom strip holds: the view name and "DIMENSIONS IN METRES"; the vessel and
   Cargo, when they are plain ASCII and at most 100 characters; and, if checks fail, one note "DESIGN FAILS
   n CHECKS: ..." (`failed_notes`). The status is `concept`, so the standard adds the status line
   "CONCEPT – NOT FOR CONSTRUCTION" (clause 14.9). Label: Method.
7. **Outputs.** `ovcd draw <file.vdesign> --out <dir> [--option <name>]` writes per Option and view: the
   drawing document, the normalised content, DXF, SVG and PDF (`draw.rs`). `ovcd draw --document` prints
   the drawing documents only. The app shows the SVG (`crates/ovcd-shell/src/views.rs`); the web build
   exports SVG, PDF and DXF (`crates/ovcd-web/src/export.rs`). Label: Method.

### Output formats

| Format | What it holds | Source |
|---|---|---|
| SVG | One user unit = 1 mm of paper. All in black. Font embedded by the CLI and web export; not in the app panel. | standard clause 15.2, `render/svg.rs` |
| PDF | One page, the sheet size, in black. Font (Barlow Semi Condensed, OFL 1.1) embedded. Same bytes for the same document. | clause 15.3, `render/pdf.rs` |
| DXF | Model space in metres, +y up. Paper-space layout `SHEET` in mm with the viewport at 1:N. Layers with standard colours and line weights. Extended data `TDFA` on each entity. | clause 15.4, `render/dxf.rs` |
| Drawing document, normalised content | JSON, for agents and the Python implementation of the standard. | clauses 4 and 16 |

### Conventions

- **Sheet:** A3 landscape, one viewport, notes strip at the bottom, no title block, no revision register.
- **Line types and layers** (standard clause 8.5.2): `OUTLINE` thick continuous (hull, tank circles in
  section); `OBJECT` thin continuous (parts in view, wheelhouse); `HIDDEN` thin dashed (parts under the
  deck); `CENTRE` thin chain; `WATER` thin continuous (design waterline) or chain double-dashed (light
  waterline); `ZONE` thin dashed (zone limits, voids: not physical); `EQUIP` thick continuous (container
  units, thrusters); `HATCH` (double bottom); `DIMENSIONS`, `ANNO`, `TEXT`. Line group 0.35 (A3).
- **Units:** metres in model space; labels give two decimals and "m".
- **Hidden and visible:** a tank part above the deck is drawn in view (`OBJECT`), the part below it hidden.
  In plan, the part above the deck is the chord of the tank at deck height.

## Assumptions, rules and estimates

Each row is one drawn element. "Source or basis" starts with the geometry source: **computed result**,
**input** (with its default) or **fixed proportion** (a drawing constant in `ga.rs`).

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| DR-A01 | Sheet | A3 landscape, drawing space 20, 10, 390 × 277 mm; one viewport; no title block | Fixed in code | Fixed proportion. Spec 0003; standard clause 6.3 | `ga.rs` `SHEET`, `document` | Unknown (presentation only) |
| DR-A02 | Scale | Largest ratio of the preferred and house lists at which the window fits; each view separately | Method | Standard clause 7.5.2 | `ga.rs` `document`, `RATIOS` | Unknown (presentation only) |
| DR-A03 | Scale bar | Under the drawing, 26 mm room kept | Fixed in code | Fixed proportion. Standard clause 7.9 | `ga.rs` `BAR_ROOM_MM` | Unknown |
| DR-A04 | Notes strip | View name and units; vessel and Cargo; failed-checks note | Method | Computed result (`vessel`, `cargo`, `checks`) | `ga.rs` `document`, `failed_notes` | Unknown |
| DR-A05 | Failed-checks note | "DESIGN FAILS n CHECKS: A, B AND k MORE", one line, at most 120 characters | Method | Computed result: each entry of `checks` with `ok` false | `ga.rs` `failed_checks`, `failed_notes` | Non-conservative if a failing check is not in `checks` |
| DR-A06 | Status line | "CONCEPT – NOT FOR CONSTRUCTION" | Fixed in code | Fixed. Standard clause 14.9 | `ga.rs` `document` (`status: concept`) | Unknown |
| DR-A07 | Hull length and depth (elevation) | Profile from stern to LOA, keel to deck | Method | Computed result: `dims.loa_m`, `dims.depth_m` | `ga.rs` `profile_points` | Unknown |
| DR-A08 | Bow rake | Straight line from keel at LOA − rake to deck at LOA; plan line at LOA − rake | Estimate | Input: `bow_rake_length_to_depth_ratio`, default 1.6 × depth, no published source; read as `geometry.bow_rake_length_m` | `ga.rs` `profile_points`, `x_at_height`, `plan` | Unknown. The hydrostatics use the block coefficient, not this shape |
| DR-A09 | Stern | Always square: the drawing reads `geometry.stern_rake_length_m`, which `simulate` never writes | Fixed in code | Fixed proportion (zero) | `ga.rs` `read_hull`, `stern_x_at_height` | Unknown |
| DR-A10 | Deck line | Straight, at moulded depth; no sheer, no camber | Fixed in code | Computed result: `dims.depth_m` | `ga.rs` `side_elevation`, `section_deck_line` | Unknown |
| DR-A11 | Midship hull section | Rectangle, beam × depth; no bilge radius | Fixed in code | Computed result: `dims`. Only `raked_box` Hull Form exists | `ga.rs` `midship_section` | Unknown |
| DR-A12 | Design waterline | Line at the design draught, between the stern and the bow rake | Method | Computed result: `results.draught_m` | `ga.rs` `side_elevation`, `midship_section` | Unknown |
| DR-A13 | Light waterline | Chain double-dashed line, only if 0 < light draught < depth | Method | Computed result: `results.light_draught_m` | `ga.rs` `read_hull` | Unknown |
| DR-A14 | Draught, freeboard, light draught labels | Values in metres, 2 decimals; freeboard = depth − draught | Method | Computed result | `ga.rs` `side_elevation`, `midship_section` | Unknown |
| DR-A15 | Dimensions LOA, beam, depth, draught, freeboard | Linear dimensions, 12 mm (or 24 mm) off the hull; label if shorter than 8 mm on paper | Method | Computed result; value from geometry (standard clause 12) | `ga.rs` `try_dim_*`, `MIN_DIM_MM` | Unknown |
| DR-A16 | Cargo zone limits | `ZONE` lines at the stern void end and at LOA − bow void | Method | Computed result: `geometry.stern_void_length_m`, `bow_void_length_m` | `ga.rs` `zones` | Unknown |
| DR-A17 | Forward equipment zone | Label "FWD EQUIP." at the middle of the bow void | Method | Computed result: `geometry.bow_void_length_m` | `ga.rs` `zones` | Unknown |
| DR-A18 | Cofferdam (gas) | Two hidden bulkheads: at stern void − aft separation, and that + cofferdam length | Method | Computed result: `drawing.separation.aft_separation_m`, `cofferdam_m` | `ga.rs` `zones` | Unknown |
| DR-A19 | Stern block zone bulkheads | Hidden line at the forward end of each zone | Method | Computed result: `drawing.aft_arrangement.zones` | `ga.rs` `aft_block` | Unknown |
| DR-A20 | Deck container units | Elevation: one box per row, deck to unit top. Plan: one box per unit | Method | Computed result: `container_x0_m`, rows, per row, unit width, side gap, row length and gap, top | `ga.rs` `aft_block` | Unknown |
| DR-A21 | Wheelhouse | Box in the lowered position (elevation); footprint (plan). Raised position not drawn | Method | Computed result: `wheelhouse_x0_m`, `x1_m`, `floor_lowered_m`, `top_lowered_m`, `width_m` | `ga.rs` `aft_block` | Unknown |
| DR-A22 | Block top label | "BLOCK TOP x m" on the part that sets it | Method | Computed result: `block_top_lowered_m` | `ga.rs` `aft_block` | Unknown |
| DR-A23 | Thrusters | Circles. Count and plan offsets from the result. Radius: elevation 0.30 × room length (max 1.1 m), 0.2 m above the keel; plan 0.32 × room length (max 1.3 m) | Fixed in code | Positions: computed result (`thruster_offsets_m`, `thruster_room_length_m`). Size: fixed proportion | `ga.rs` `THRUSTER_*` | Unknown (symbol only) |
| DR-A24 | Hold ends and inner bottom (dry) | Hidden hold ends from inner bottom to deck; hidden inner bottom along the full hull | Method | Input: `hold_double_bottom_m`, default 1.2 m (`drawing.double_bottom_m`) | `ga.rs` `side_elevation` | Unknown |
| DR-A25 | Hatch coaming (dry) | Above the deck over the hold length (elevation); two vertical lines at the hold top width (section) | Method | Input: `hatch_coaming_height_m`, default 1.0 m | `ga.rs` `side_elevation`, `midship_section` | Unknown |
| DR-A26 | Hold opening (plan) | Rectangle of the hold top width over the cargo zone; bottom width hidden if different | Method | Computed result: `geometry.hold_top_width_m`, `hold_bottom_width_m` | `ga.rs` `plan` | Unknown |
| DR-A27 | Hopper slope (section) | Straight line from (top width / 2, deck) to (bottom width / 2, inner bottom) | Method | Computed result: top and bottom width. The drawn slope is not the input slope: see proposals E1 | `ga.rs` `midship_section` | Unknown: drawn hold area differs from the computed one |
| DR-A28 | Tank outer diameter | Circle (section) and stadium (elevation, plan) at the Tank Outer Diameter (insulated face) | Method | Computed result: `geometry.tank_outer_diameter_m` | `ga.rs` `read_hull` | Unknown |
| DR-A29 | Tanks across and along | Count from the arrangement | Method | Computed result: `arrangement.n_wide`, `n_long` | `ga.rs` `tank_layout` | Unknown |
| DR-A30 | Tank length and spacing | Length = (cargo zone − (n_long − 1) × 2.0 m) / n_long; first tank at the stern void end; across: from −(beam − 2 × side clearance) / 2, pitch OD + 1.0 m | Fixed in code | Computed result (cargo zone, OD, side clearance) and fixed gaps 2.0 m and 1.0 m read from `BargeInputs` defaults, the same as the core | `ga.rs` `tank_layout`, `read_hull` | Unknown |
| DR-A31 | Tank heads | Hemispherical (half circle of radius OD / 2) | Fixed in code | Fixed proportion. The core default head type is also `hemisphere` | `ga.rs` `stadium` | Unknown |
| DR-A32 | Tank height | Tank bottom = double bottom + support height; centre = bottom + OD / 2 | Method | Computed result: `drawing.double_bottom_m` (= max(0, bottom clearance − support)); input `tank_support_height_m`, default 0.5 m | `ga.rs` `tank_bottom`, `tank_centre_y` | Unknown |
| DR-A33 | Tank above and below the deck | Elevation: stadium split at deck height. Plan: hidden full outline, plus the chord outline at deck height | Method | Computed result | `ga.rs` `stadium_split`, `chord_half` | Unknown |
| DR-A34 | Double bottom (gas) | Elevation: hidden line along the hull. Section: line across and a `lines` hatch under it. Only if > 0 | Method | Computed result: `drawing.double_bottom_m` | `ga.rs` `elevation_gas_detail`, `section_gas_detail` | Unknown |
| DR-A35 | Wing ballast tank | Inboard wall and top per side; top = tank bottom + ratio × OD, cut at the deck. Not drawn if its width ≥ side clearance | Method | Computed result: `drawing.wing_tank_width_m` (in practice 0.4 m, see proposals E3); input `wing_tank_height_ratio`, default 0.66 | `ga.rs` `wing_tank` | Non-conservative: the cut at the deck hides the side extension the weights count (proposals E2) |
| DR-A36 | Void beside the wing tank | `ZONE` line from the wing tank wall, width = min(gap, side clearance − wing width) | Method | Input: `wing_tank_gap_m`, default 0.6 m | `ga.rs` `wing_tank` | Unknown |
| DR-A37 | Cradle (saddle) | Section only: arc of 120° under each tank, 0.05 m outside it, two legs down to the double bottom | Fixed in code | Fixed proportion (`CRADLE_WRAP_DEG`, `CRADLE_CLEARANCE_M`). Saddle height is the support height (DR-A32) | `ga.rs` `section_gas_detail` | Unknown (symbol only) |
| DR-A38 | Dome | Curve on each tank top. Rise = dome height. Base: section 0.30 × OD; elevation 2 × max(0.06 × tank length, 1.2 m), at most tank length − OD | Fixed in code | Rise: input `tank_dome_height_m`, default 2.0 m. Base and shape: fixed proportion; the dome diameter input is not read (proposals E4) | `ga.rs` `DOME_*`, `Canvas::dome` | Unknown |
| DR-A39 | Side clearance label | Value at the gap between the hull side and the first tank | Method | Computed result: `geometry.side_clearance_m` | `ga.rs` `midship_section` | Unknown |
| DR-A40 | Bottom clearance label | "BOTTOM CLEARANCE x m", or "TANK BOTTOM x m (CLEARANCE y m)" when they differ | Method | Computed result: `geometry.bottom_clearance_m` and DR-A32 | `ga.rs` `midship_section` | Unknown |
| DR-A41 | Section deck line | Stops at each tank circle that passes through the deck | Method | Computed result | `ga.rs` `section_deck_line` | Unknown |
| DR-A42 | Centre line | Plan and section, 4 mm past the hull | Fixed in code | Fixed proportion | `ga.rs` `plan`, `midship_section` | Unknown |
| DR-A43 | Label layout | Column 6 mm past the bow (elevation, plan) or 24 mm left of the hull (section); 6 mm pitch; text 3.5 mm | Fixed in code | Fixed proportion; standard clause 12.6 | `ga.rs` `label_column`, `LABEL_*` | Unknown (presentation only) |

## Inputs the user can change

No input belongs to the drawing alone. These inputs move drawn geometry. Defaults are from
`crates/ovcd-core/src/barge_capacity.rs` and `models.rs`.

| Input | Default | Unit | Drawn as |
|---|---|---|---|
| `length_overall_m`, `beam_m`, `moulded_depth_m` | none (typed) | m | Hull outline, dimensions |
| `bow_rake_length_to_depth_ratio` | 1.6 | – | Bow rake (DR-A08) |
| `pushed_stern_void_m` | 3.0 (pushed barge) | m | Cargo zone start (DR-A16) |
| `hold_wing_void_width_m` | 1.0 | m | Hold top width (DR-A26) |
| `hold_double_bottom_m` | 1.2 | m | Inner bottom (DR-A24) |
| `hopper_slope_h_per_v` | 1.0 | – | Hold bottom width only (DR-A27) |
| `hatch_coaming_height_m` | 1.0 | m | Coaming (DR-A25) |
| `tanks_in_line` | 2 | – | Tanks along (DR-A29) |
| `tank_support_height_m` | 0.5 | m | Tank bottom, cradle legs (DR-A32, DR-A37) |
| `tank_dome_height_m` | 2.0 | m | Dome rise (DR-A38) |
| `wing_tank_gap_m` | 0.6 | m | Void (DR-A36) |
| `wing_tank_height_ratio` | 0.66 | – | Wing tank top (DR-A35) |
| Stern block inputs (`wheelhouse_*`, container units, thrusters) | `aft_arrangement.rs` defaults | various | DR-A19 to DR-A23 |

The drawing constants in DR-A23, DR-A30, DR-A31, DR-A37, DR-A38 and DR-A43 are not inputs.

## Limits and known findings

- The drawings are schematic. They show the hull envelope and the main spaces. They do not show frames,
  scantlings, plating, stiffeners, deck equipment, mooring, pipework, hatch covers, nozzles or construction
  detail (spec 0003, out of scope).
- The drawings do not show: the weather shield that the weights count; the wing plate and side extension
  when the wing tank top is above the deck; the forward separation and service space; the engine room, gas
  valve unit room and generator sets of the stern block; the raised wheelhouse; the ballast waterline;
  the air draft; separate holds (`n_holds`); containers in a container hold; the bulk wing voids as
  labelled spaces. See the proposals file.
- The "midship section" is a typical section through the tanks or the hold. It is not cut at
  amidships: with two tanks in line it can show a tank where amidships falls in the gap between tanks.
- The three views can have different scales. A view is not at a fixed scale for a given hull.
- The hull form is a raked box only. The drawn bow rake is not the shape that the hydrostatics use (they
  use the block coefficient).
- There is no title block, no drawing number and no revision register. The status is always `concept`.
- `crates/ovcd-core/src/ga_adapter.rs` is a port of the earlier Python drawing adapter. The view builder
  does not use it. It serves the parity harness (`eval.rs`) and one core test only.
- Findings for the register (#121): see `proposals/11-drawings.md`, errors E1 to E4.

## Evidence

- **Standard conformance:** `crates/ovcd-draw/tests/conformance.rs` runs every case of the vendored suite
  (`crates/ovcd-draw/standard/conformance/suite.json`). `tests/vendored.rs` checks the copy against
  `standard/PIN.json` (pinned commit of the standard's repository, with a hash).
- **Cross-implementation:** a CI job (`.github/workflows/ci.yml`) installs the Python tool at the pinned
  commit and compares normalised content for every fixture Design.
- **Outputs:** `crates/ovcd-draw/tests/outputs.rs` and `tests/dxf.rs` (DXF read back with a second reader,
  `tests/support/dxf_reader.rs`).
- **View builder:** `crates/ovcd-format/tests/ga.rs` (every document validates; LOA, beam, depth, draught
  and freeboard on the drawings equal the results; dry bulk and gas show their containment);
  `ga_recovery.rs` (each drawn item at the place the results give, `null` items left out);
  `ga_leaders.rs` (labels inside the frame, no crossing leaders, no overlapping label boxes);
  `ga_failed_note.rs`.
- **Render check for this page:** the built CLI (`ovcd 0.2.2`, older than the workspace 0.2.3) drew a
  sloped-hopper fixture and the website gas example. They confirmed the layout above. That build refused
  two newer fixtures (unknown inputs), so the current code was not rendered.
- No test checks drawn geometry against a computed result other than the five main dimensions and the
  positions read from the result.

## Questions for reviewers

1. Is a 120° cradle at 0.05 m outside the tank a fair concept symbol for a Type C saddle? Should the
   elevation and plan show saddle positions?
2. Should the dome be drawn as a cylinder of the dome diameter input (2.0 m default), as the volume model
   has it, rather than as a curve of 0.30 × OD?
3. Should the midship section be cut at amidships, at the largest tank section, or both?
4. When the wing tank top is above the deck, should the section show the raised side and the wing plate
   that the weights count, with the deck at the wing plate level?
5. For a sloped hopper, does the hopper slope run from the inner bottom to the coaming top (as the volume
   model has it) or to the deck (as drawn)?
6. Should all three views share one scale, so a draughtsperson can scale between them?
7. Is a square stern acceptable for every pushed barge and self-propelled hull at concept stage?
8. Which items must a concept GA show for an inland gas carrier that these views lack (for example tank
   connection space, gas detection zones, hazardous areas)?
