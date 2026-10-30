# Spec 0009: A 3D vessel you can read: cargo tanks, deck house, wheelhouse and propulsors

Status: **agreed with the maintainer, 2026-10-30** (D1 b, D2 a, D3 a, D4 a). Decision record: ADR 0021. Builds on spec 0008 (Vessel Model, phase 2, built) and
ADR 0018 (one parametric Vessel Model; meshes derived; three.js draws them). Uses the stern block and
propulsor model of spec 0007 and ADR 0017. Shares one data gap with the stern drawings (#555). ADR 0021 comes
with it. Parent issue #681.

## Problem Statement

Spec 0008 phase 2 is built. The 3D view draws the hull, the decks, the bulkheads, the prism Spaces and the
waterlines. A from-scratch run of a 140 m × 29 m self-propelled LNG barge with Type C tanks showed that
the view does not convey the vessel:

- **The hull is a ghost.** The deck and bulkhead layers start on, so the hull is drawn at 20 % opacity
  from the first frame (`Model3D.svelte`: `innerOn`, opacity 0.2).
- **The cargo is a box.** A gas Design shows one prism, `H1C`, 25,792 m³: the hold space inboard of the
  wing tanks. The Type C tanks, their heads and their domes are not drawn. A reviewer cannot see the tank
  arrangement that sets the capacity, the beam and the wheelhouse lift.
- **Nothing stands on the deck.** The accommodation and battery units, the wheelhouse (lowered or raised)
  and its lift column are not drawn. Spec 0008 left out every superstructure.
- **Nothing drives it.** No azimuth thruster, propeller, nozzle, rudder or bow thruster is drawn. The
  tunnels of a tunnel stern are in the hull; the units in them are not.
- **The picture is flat.** One ambient light (1.4) and one directional light (2.2), no edge lines, no
  water. The default camera is far and high.
- **The picture is mirrored.** The model axes (x forward, y to starboard, z up) are left-handed; three.js
  and glTF are right-handed. The view draws the raw positions, so the "profile from starboard" shows the
  bow on the left, and every port item would show to starboard. A symmetric hull hid this until now; the
  first unit, tank row or thruster off the centre line shows it.

Most of the data is already in the core. `AftArrangementResult` holds every deck unit's footprint, the
units' top, and the wheelhouse's lowered and raised floor and top. The tank diameter, length, head type,
count across and along, and the dome size are in `BargeCapacityResult` and the tank rule values. The core
places the tanks in four places that do not share one function: the per-tank centres of the IGC
accelerations (`barge_capacity.rs`), the cargo LCG and the tank axis KCG (`loading_conditions.rs`,
`gas_stability.rs`, `weights.rs`), and the GA drawer (`ga.rs`, `Hull::tank_layout`). The propulsor
**positions** are spread the same way: the mass items of `machinery.rs` place the drive units, the shafts
and the rudders; the Stern Form places the tunnels; the aft arrangement places the thruster envelopes; no
result says where a propeller is. #555 is blocked on that gap.

## Solution

Draw what the Design already holds, from the same functions that place it for the numbers, and make the
picture readable.

0. **Fix the picture first.** Mirror the scene into right-handed axes once, open with an opaque hull, a
   sea plane and the hull below the waterline in a second tone. This needs no new data and lands first.
1. **Tanks.** Each Type C tank is a cylinder with its Tank Head and its dome, at its Tank Outer Diameter,
   placed by one `tank_layout` function in the core. The capacity, the loading conditions and the GA
   drawer move onto the same function.
2. **Deck items.** Accommodation and battery units, the wheelhouse (lowered, raised, or both) and its lift
   column, from `AftArrangementResult`.
3. **Propulsors.** Named position functions in `machinery.rs` give the place of every propeller, nozzle,
   azimuth leg and pod, rudder and bow thruster. The mass items call them, so the lightship and the
   picture cannot disagree. The 3D view draws them; the stern drawings of #555 read the same layout.
4. **A readable picture.** Shading by part type, edge lines, a three-light rig, a default aft-quarter
   camera, three view presets (Exterior, Cargo, Arrangement), a half-model cut with a cut face, a deck cut,
   and theme colours that pass the contrast check in Dark and Light.

**Rule for every new mesh: no geometry without a source.** Each mesh group carries a typed source (which
tank, which unit, which propulsor). A test rebuilds each part's bounding box from its source values in the
results and compares it with the mesh. A part with no data in the model (a saddle, a hatch coaming, a mast,
a railing) is not drawn; it becomes a data ticket.

## User Stories

### Designer

1. As a designer, I want to see each cargo tank with its true diameter, length, heads and dome in its true
   place, so that I can judge the arrangement the fit chose.
2. As a designer, I want the tanks, the hold space and the wing tanks together in one cut, so that I can
   see the side space the IGC clearance leaves.
3. As a designer, I want to see the accommodation units and the wheelhouse on the deck, so that I can see
   what the cargo length pays for.
4. As a designer, I want to switch the wheelhouse between lowered, raised and both, so that I can see the
   lift the sightline needs against the air draft it costs.
5. As a designer, I want to see the thrusters or the propellers, nozzles and rudders under the stern, and
   the bow thruster, so that I can check that the propulsion fits the stern.
6. As a designer, I want the hull opaque on the water, with its draught visible, so that the first view
   reads as a vessel.
7. As a designer, I want one click to an exterior view, a cargo view and an arrangement view, so that I
   do not toggle twelve layers to show a point.
8. As a designer, I want to cut the model in half along the centre line and to lift the deck off, so that
   I can see inside without a translucent blur.
9. As a designer, I want to click a tank, a deck unit, the wheelhouse or a propulsor and see its source
   values, so that the picture leads me back to the numbers.

### Reviewer or expert

10. As a reviewer, I want the 3D view to show only what the Design holds, so that I can trust that a
    tank I see is a tank the capacity counts.
11. As a reviewer, I want port to be port, so that the view agrees with the plan drawing.

### Maintainer

12. As a maintainer, I want every new mesh to carry its source and a test that rebuilds it from that
    source, so that decorative geometry cannot creep in.
13. As a maintainer, I want a triangle budget with a test, so that a large Design stays smooth on the web.

## Implementation Decisions

### Terms

New glossary terms (added by the first ticket that needs each):

- **Outfit:** the parts of the Vessel Model that are not hull, compartments or structure and are not in
  the hydrostatics: the Deck Items and the propulsors (ADR 0021).
- **Deck Item:** an Outfit part that stands on the deck: an accommodation or battery unit, the
  wheelhouse, its lift column.
- **Propulsor Layout:** the position and size of each propulsor, nozzle, rudder and bow thruster of a
  Hull, from the core. The mass items, the drawings and the 3D view read it.
- **Tank Layout:** the position of each cargo tank (x range, y and z of its axis), from the core. The
  capacity, the loading conditions, the drawings and the 3D view read it.
- **View Preset:** a named set of layer toggles and a camera in the 3D view.

### ADR 0021: tank-shaped Spaces and an Outfit layer (new, with this spec)

ADR 0018 says "one compartment model … no second Space type" and names three layers: hull, arrangement,
structure. This spec keeps the first rule and adds a fourth layer:

- **A Type C tank is a `compartments::Space`** with a new shape: `SpaceShape::Prism { section }` (today's)
  or `SpaceShape::Tank { axis_x0, axis_x1, y, z, outer_diameter, head }`. `clip` passes a tank through
  unchanged (it never touches the hull; a test asserts it). Its volume and centroid come from the tank
  rule values, not the mesh. A tank is a compartment: IGC damage stability (#663) will need it as one. The tank Spaces join the results Spaces
  table (D1); their volume is the interior volume the capacity already counts.
- **Outfit is a fourth layer**, not in the hydrostatics, as spec 0008's "No superstructure" intends:
  `VesselModel` gains `outfit: Outfit { deck_items, propulsors }`. Its masses are already in the
  lightship (spec 0007, `machinery::mass_items`, `stern_block_mass`); the Outfit adds only geometry.
- Everything else in ADR 0018 holds: parameters stored, meshes derived, the core builds them, the mesh
  hash is outside the results hash.

### Axes and handedness

- The Vessel Model axes of spec 0008 hold: x forward from the aft end, **y to starboard**, z up from the
  base line. They are left-handed.
- **The view mirrors once.** `Model3D.svelte` puts every mesh under one root group with scale
  (1, −1, 1). three.js flips the front face for a negative determinant, so the winding stays correct. The
  fixed views and the clipping planes are set in model axes under that root.
- **glTF mirrors once.** The glTF root node gets the matrix that maps (x, y, z) model axes to glTF's
  right-handed, y-up axes with the mirror in it. The STL/OBJ panel mesh stays in hull axes (spec 0008,
  #461) and its file header says so.
- **Second hazard:** `AftArrangementResult.container_units` gives `y_centre` with **y + = port**. The Deck
  Item builder converts once.
- **Test the picture, not the sign.** An e2e check puts the camera in the starboard profile view, projects
  the bow point and a port unit's centre to the screen, and asserts the bow is on the right and the unit is
  behind the hull's centre plane (farther from the camera). The same check in the plan view asserts port is
  on the left with the bow up.

### Model changes (`ovcd-core`, all `no_std`)

| Module | Change | Holds |
|---|---|---|
| `tanks` | new `fn tank_layout(..) -> TankLayout` | each tank: name, axis x0..x1, y, z, outer diameter, head type and length; dome x, y, diameter, height |
| `barge_capacity`, `loading_conditions`, `gas_stability`, `weights` | call `tank_layout` where they place tanks today | no change of numbers (see Testing) |
| `compartments` | `SpaceShape`; `clip` passes a tank through | ADR 0021 |
| `machinery` | named position functions: `drive_unit_positions`, `shaft_positions`, `rudder_positions`, `bow_thruster_position`, returning `PropulsorLayout`; `mass_items` calls them | the one source of propulsor places |
| `vessel_model` | `spaces` gains the tanks (gas Designs); `outfit` gains Deck Items and the Propulsor Layout | assembled from results that already exist |
| `mesh` | `tank_mesh`, `deck_item_mesh`, `propulsor_mesh`, `sea_mesh`; `MeshGroup` gains `source: PartSource` (typed: `Tank(i)`, `Unit(label)`, `Wheelhouse(Lowered\|Raised)`, `LiftColumn`, `Propulsor(i)`, `Nozzle(i)`, `Rudder(i)`, `BowThruster`, and today's layers) | one group per part |

**Tanks.** The Tank Layout reproduces what the core already uses: tanks along the cargo zone from the
stern void with the longitudinal gap; across the beam less the side clearances with the transverse gap;
the axis at the bottom clearance plus half the outer diameter. Domes come from the dome rule of #316 (the
diameter and height the capacity uses for Vc). The weather shield (#410) is drawn when its inputs are set:
a thin shell at `weather_shield_clearance_m` over the tank tops. For a gas Design the hold space `H1C`
stays a Space, named in the view "Hold space (around tanks)" (IGC term).

**Deck Items** come from `AftArrangementResult`:
- Each `container_units` entry is a box: its footprint, `container_unit_width_m` wide, from the deck to
  `container_top_above_keel_m` (an input height, not the ISO constant).
- The wheelhouse: a box `wheelhouse_x0_m..x1_m` × `wheelhouse_width_m`, from `wheelhouse_floor_lowered_m`
  to `wheelhouse_top_lowered_m`, and a second box at the `_raised_m` values.
- The lift column: from the deck to the raised floor; its plan size is an estimate (D2).
- **The deck is not flat.** Hull Forms have camber and sheer. A unit stands on the highest deck point under
  its footprint, so it never cuts into the deck.
- The gensets stay below deck (they sit on the double bottom); they are machinery, inside the `M` Spaces.

**Propulsors** take their places from the functions the masses already use, and their sizes from the final
`powering.*` results (#643 iterates the diameter until the light-draught immersion holds):
- **D** = `powering.propeller_diameter_m`; **axis z** = `powering.shaft_axis_height_m` (D/2: the tip on
  the base, as `stern_form.rs` and #643 have it).
- **x:** shaft propellers at the aft end of the shaft line (`AFT_PEAK_M` from the aft end); azimuth units
  at the drive room as the mass items place them; rudders aft of each shaft propeller, where
  `rudders and steering gear` sits.
- **y:** under a tunnel stern, `SternGeometry.tunnel_offsets_m` for every propulsor type; otherwise
  `thruster_offsets_m`. **Count** = `propulsor_count`. Where `thruster_offsets_m` holds fewer (it is
  clamped by the envelopes that fit), the layout returns a warning, and the ticket files the disagreement
  as a finding.
- Nozzle, azimuth leg and pod, rudder span and chord, bow thruster tunnel: sizes from D or from the
  bow thruster power with `estimate` rows in the basis (D2).
- The layout goes into the results as `drawing.propulsors`, outside the results hash like the other
  `drawing.*` blocks, so #555 can draw the stern from it.

### Meshes and the parity hash

- Tessellation: cylinders and nozzles 48 segments round; heads 12 rings; domes 24 segments; constants in
  `mesh.rs`.
- Every new mesh is closed, welded and outward-facing, so the spec 0008 edge test applies unchanged.
- The mesh hash (`ovcd mesh --hash`) covers the new groups and stays outside the results hash.

### The 3D view (`app/src/lib/views/Model3D.svelte`, `model3d.ts`)

- **One geometry per group.** Groups are contiguous in the core's buffers; the view slices each into its
  own `BufferGeometry`. Today every group shares the whole buffer, so each has the whole model's bounding
  sphere: transparent parts sort at random and frustum culling does nothing.
- **Shading by part type:** `flatShading` for the hull, decks, boxes and bulkheads; smooth normals for
  cylinders, heads, domes and nozzles.
- **Lights:** a hemisphere light (sky and ground from theme tokens), a key directional light from the
  aft-starboard quarter and a weaker fill from port. No ambient flood.
- **Edges:** an edge-line overlay (`EdgesGeometry`, 30°) on the hull, tanks, Deck Items and propulsors,
  in the `--model-edge` token. The edge materials get the same clipping planes as the faces.
- **Water:** a sea plane at the shown loading condition's waterplane, translucent, from `--model-sea`;
  the hull below that waterplane in `--model-hull-wet`, above it in `--model-hull`.
- **Opaque by default.** The hull is opaque in the Exterior and Cargo presets and translucent only in the
  Arrangement preset. The deck and bulkhead layers no longer make the hull translucent by themselves.
- **Camera:** the default view is the aft-starboard quarter, 25° above the horizon, framed to the
  bounding box with 10 % margin. Reset view returns there. The fixed views stay.
- **View Presets:**

  | Preset | Hull | Deck | Tanks | Hold space | Ballast and voids | Deck Items | Propulsors | Weather shield | Sea |
  |---|---|---|---|---|---|---|---|---|---|
  | Exterior (default) | opaque, two tones | on | domes show through the deck | off | off | on | on | on | on |
  | Cargo | half-cut | lifted off | on | off | wing and double bottom on | on | off | off | off |
  | Arrangement | translucent | on | on | on | on | on | on | off | off |

  The layer toggles stay; a preset only sets them. Changing a toggle shows the preset as "Custom".
- **Cuts:** a **half-model** toggle removes the port half (a clipping plane at y = 0 in model axes); a
  **deck** toggle hides the deck plane and the weather shield. Back faces exposed by a cut draw in the flat
  `--model-cut` colour, which reads as a cut face at no extra geometry. The x section plane of spec 0008
  stays and works with both.
- **Wheelhouse:** a three-way switch, Lowered / Raised / Both. "Both" draws the raised one as a ghost
  (40 % opacity). A label gives the lift needed, the lift allowed and the light air draft.
- **Pick:** a click on a tank, a Deck Item or a propulsor shows its source values: tank OD, length, head,
  volume and centre (from the core); unit label and size; wheelhouse floor and top, lift needed and
  allowed; propulsor type, D, axis height, power. Values come from the core, not from the mesh.
- **Colours:** new tokens in `app/src/app.css` beside the `--space-*` family, with Light and Dark values
  (the Dark blocks already exist), each at least 3:1 against `--paper` (WCAG 1.4.11, non-text contrast),
  checked by `scripts/contrast.mjs`: `--model-hull`, `--model-hull-wet`, `--model-deck`, `--model-sea`,
  `--model-deck-item`, `--model-wheelhouse`, `--model-propulsor`, `--model-edge`, `--model-cut`. Tanks keep
  the cargo colour; the hold space a lighter tint of it.
- Text through the Text Library; every new string has a Message Id.
- The Spaces table beside the canvas lists the tanks, the Deck Items and the propulsors, so the fall-back
  without WebGL still shows them.

### Exports

- glTF gains one node per tank, Deck Item and propulsor, named as in the pick panel, with the Light theme
  colours as materials, under the mirrored root. The raised and lowered wheelhouse are two nodes. The
  Khronos validator still passes.
- The STL/OBJ panel mesh stays the hull only (it is for solvers).

### Performance budget

| Measure | Budget | Checked by |
|---|---|---|
| Triangles, all layers | ≤ 1.5 × the count measured on each fixture when the ticket lands, pinned per fixture | a core test over every fixture (fails CI) |
| Triangles, any Design | ≤ 200,000 | the same test on the largest sweep case (fails CI) |
| Draw calls with every layer on | reported | the web smoke test logs `renderer.info.render.calls` |
| Core mesh build, native and WebAssembly | ≤ 50 ms and ≤ 200 ms | `web_parity.mjs` timing, reported in the PR |
| View open to first frame | ≤ 1.5 s desktop, ≤ 2.5 s web | e2e timing, reported |
| Frame rate while orbiting | ≥ 50 fps at 1920 × 1080 on a 2020 integrated GPU | manual check in the PR, machine named |

Only the triangle rows fail CI. Timing on shared runners is noise.

### File ownership and hand-over

- `gas_arrangement.rs`, `tanks.rs`, `barge_capacity.rs`: owned today by the gas arrangement lane (#675).
  The Tank Layout ticket does not wait for #675 to close; it messages that lane and takes the files when
  #675's PR has merged or before it opens, by agreement on the issue.
- `ga.rs`: the Tank Layout ticket changes `Hull::tank_layout` to read the core layout, with drawing byte
  parity on every fixture. The stern drawings lane (#555) owns the stern parts of `ga.rs`.
- `machinery.rs`, `simulate.rs`, `run.rs`: the Propulsor Layout ticket waits for #643 to merge (it adds
  the `powering.*` diameter and axis keys, and edits `simulate.rs` and `run.rs`). The ticket edits
  `simulate.rs` too: the `measured.vessel` capture supplies the model inputs.
- `mesh.rs`, `vessel_model.rs`, `compartments.rs` (`SpaceShape` only), `Model3D.svelte`, `model3d.ts`,
  `export.rs`, the new `--model-*` tokens in `app.css`: this spec's lane.

### Rules (spec 0004)

No new Rule. The view reads results; it checks nothing.

## Testing Decisions

Each geometric test compares the mesh with an oracle that does not share its construction.

- **Sources:** for every part of every fixture, rebuild its bounding box from its `PartSource` values in
  the results JSON (the tank's OD and length, the unit's footprint and top, the wheelhouse floor and top,
  the propulsor's D and axis) and compare with the mesh's box to 1 mm. A group with no source fails.
- **Tank Layout moves no number.** Every fixture's results hash is unchanged after `barge_capacity`,
  `loading_conditions`, `gas_stability` and `weights` call `tank_layout`. Every fixture drawing keeps its
  bytes. Also: the mean of the tank axes' x equals the cargo zone centre the cargo LCG used, and the axis z
  equals the KCG the stability used.
- **Tank meshes:** the count equals `n_tanks_total`; each tank mesh volume is within 0.5 % of the closed
  form at the **outer** diameter (cylinder plus two heads of the head type), computed in the test; no tank
  mesh intersects another tank, a wing ballast Space or the hull mesh (a mesh–mesh test, not a clearance
  identity); a tank Space is unchanged by `clip`.
- **Deck Items:** each unit's base equals the deck height at its footprint's highest point, and its top
  equals `container_top_above_keel_m`; the raised wheelhouse floor equals `wheelhouse_floor_raised_m`; the
  lowered top is the point the light air draft uses.
- **Propulsors:** each propulsor mesh's centroid equals its mass item's LCG and KCG (the mass items call
  the same functions; the test proves the picture and the lightship agree); axis z equals
  `powering.shaft_axis_height_m`; under a tunnel stern each disc centre lies on its tunnel's axis; the count
  equals `propulsor_count`. One azimuth gas fixture and one tunnel-stern shaft fixture are pinned.
- **Handedness:** the projection check in "Axes and handedness", in the web smoke test; a glTF read-back
  test: the glTF root maps model (x, y, z) to glTF (x, z, y), a swap with determinant −1 that makes the
  axes right-handed and y-up; a port node then has negative glTF z, and the test asserts it.
- **Closed meshes:** every new mesh passes the spec 0008 edge test.
- **Parity:** native and WebAssembly mesh hashes agree on every fixture (existing check, with the new
  groups).
- **Budget:** the triangle test above.
- **Viewer e2e:** open the gas fixture and the dry bulk fixture; select each preset; turn on the half cut
  and the deck cut; switch the wheelhouse three ways; pick a tank, the wheelhouse and a thruster.
  Screenshots in Light and Dark go into the PR as evidence. They are **not** a pixel gate (GPUs differ).
- **The 3D view never changes the Design.** It shows the same Design the numbers describe. No existing
  result value moves: cargo, draught, stability, masses and every check keep their values on every
  fixture. Meshes and `drawing.*` are outside the results hash. The one results change is the added
  rows of the tank Spaces in the Spaces table (D1); the design-impact table of that ticket must list
  only added rows, and the ticket fails if any existing row changes.
- `ovcd-core` stays `no_std`; `bash tools/gates.sh` passes, WebAssembly build included.

## Out of Scope

- Hatch coamings, hatch covers, saddles, pipe racks, masts, railings, mooring gear, windows: no data in the
  model. Each becomes a data ticket first if wanted.
- The structure layer (spec 0008 phase 3).
- The stability effect of a raised wheelhouse (ES-TRIN 7.12; see #675).
- Photo-real rendering: textures, environment maps, shadows, ambient occlusion. Revisit only if the
  phase 1 picture still does not read.
- Editing in 3D (ADR 0018).

## Phases

**Phase 1: a vessel you can read (the MVP).**
- Track 0 (no new data, lands first): the mirror fix, opaque by default, one geometry per group, shading,
  lights, edges, sea plane and two-tone hull, the default camera, the new tokens.
- Track A: Tank Layout; tank Spaces, domes, weather shield; Deck Items and the wheelhouse three ways;
  presets, cuts, pick; glTF nodes; the triangle budget.
- Track B (parallel, does not block A): Propulsor Layout and propulsor meshes.

**Phase 2: more of the Design.** The sightline ray and Blind Distance in 3D (from the `wheelhouse`
result); a Formation in 3D (the tug and the barges, spec 0007 wave 2); crude oil tank details the model
holds.

**Phase 3: with the structure layer.** Frames and longitudinals when spec 0008 phase 3 lands.

## Decisions taken and open questions

Taken with the maintainer, 2026-10-30:

1. **D1 (b):** the Type C tanks join the results Spaces table in phase 1, with volume and centroid. The
   maintainer's condition: the 3D view only shows the same Design. So the added rows are the only
   results change; no existing value moves (see Testing).
2. **D2 (a):** parts with no published proportion (nozzle length, azimuth leg and pod, rudder span and
   chord, bow thruster tunnel, lift column plan size) are sized from D, the bow thruster power or the
   wheelhouse size, each with an `estimate` row in the basis.
3. **D3 (a):** ADR 0021 is accepted: tank-shaped Spaces and the Outfit layer.
4. **D4 (a):** the 3D view opens on the Exterior preset.

Open (for the build, not blocking):

- Where `thruster_offsets_m` holds fewer units than `propulsor_count`, which one is right (#691
  files the finding).
- The `estimate` ratios of D2: the ticket proposes each with its basis; the maintainer may change them.

## Build tickets

Parent issue #681. Each ticket has native blocked-by links; a builder takes the next ticket whose blockers
are all closed (`docs/agents/build-process.md`). Tracks 0, A and B can run in parallel lanes.

**Phase 1, track 0 (the picture; no new data)**
1. #682 3D view: mirror into right-handed axes (view and glTF), with the projection test. A bug fix.
2. #683 3D view: one geometry per group, opaque by default, shading by part type, lights, edges, default camera
3. #684 3D view: sea plane and two-tone hull at the shown waterline; new `--model-*` tokens and the contrast check (blocked by #683)

**Phase 1, track A (tanks, deck, presets)**
4. #685 ADR 0021 `SpaceShape`; one core Tank Layout for capacity, loading conditions, stability, weights and the GA drawer; no number and no drawing byte moves (agree the order with #675 first)
5. #686 Tank meshes, heads, domes and weather shield, with `PartSource` and the source test (blocked by #685)
6. #687 Tank Spaces in the results Spaces table; a design-impact table with only added rows (blocked by #685)
7. #688 Deck Items: units, wheelhouse lowered and raised, lift column; deck height under the footprint; the y-convention conversion (blocked by #685)
8. #689 View Presets, half-model and deck cuts with the cut face, wheelhouse switch, pick panel for the new parts (blocked by #683, #686, #688)
9. #690 glTF nodes for the new parts; the triangle budget test (blocked by #682, #686, #688)

**Phase 1, track B (propulsors)**
10. #691 Propulsor Layout: named position functions in `machinery.rs`, `mass_items` calls them, the results' `drawing.propulsors` (blocked by #355, whose PR #643 adds the diameter and axis keys)
11. #692 Propulsor meshes and pick (blocked by #691, #683, #686)
12. #555 stern drawings read `drawing.propulsors` (stern drawings lane; blocked by #691)
