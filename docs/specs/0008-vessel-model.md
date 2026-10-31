# Spec 0008: Vessel Model, phases 1 and 2: hull sections, hydrostatics, 3D view and exports

Status: **agreed with the maintainer, 2026-10-21.** Decision records: ADR 0012 (accepted with this spec)
and ADR 0018. Amended 2026-10-23 by ADR 0020 (the equilibrium solves stop at a tolerance). Parent issue: see "Build tickets".

## Problem Statement

OVD floats a box. The main path is a rectangular box `L × B × D` whose displacement is `Cb × L × B × T`
with one fixed block coefficient. The legacy path is a raked box in closed form. Stability heels a
prismatic box section. The Stern Forms of ADR 0017 correct the box in closed form. The drawn hull and
the hydrostatic hull are not the same shape: the drawn bow rake is not in the hydrostatics. The
engineering basis review (pages 01 and 03) named the box hydrostatics and the box GZ as a key weakness.

ADR 0012 chose a sections table as the master hull geometry, but nothing builds it. There is no 3D view,
no mesh and no way to give the hull to another tool.

## Solution

**Phase 1: hull geometry and hydrostatics.** A sections table becomes the hull of every Option. Hull
Forms are generators that write it: box, raked box, bilge radius, sheer and camber, the three Stern
Forms, V-bottom and chine, a ShipD-style parametric hull, and a typed offsets table. The hydrostatics
and the GZ curve come from the sections by exact polygon integration, with heel and free trim. The
block coefficient becomes a result. The hull inputs join `.vdesign` format version 3. Results move once,
with one design-impact table. **0.3 ships at the end of phase 1.**

**Phase 2: the Vessel Model, the 3D viewer and the exports.** The core assembles the Vessel Model (hull,
decks, bulkheads, Spaces) and builds meshes from it. The desktop app and the web app draw them with
three.js. The user turns layers on and off, moves a section plane, sees the waterline of each loading
condition and clicks a Space to see its name, volume and centroid. OVD exports glTF, a watertight STL
or OBJ panel mesh and the offsets table as CSV.

## User Stories

### Designer

1. As a designer, I want the hull I see in the drawings to be the hull the hydrostatics use, so that the
   draught and the stability belong to the shape I chose.
2. As a designer, I want to pick a Hull Form (box, raked box, V-bottom, chine, ShipD-style or my own
   offsets) and set its few parameters, so that I can compare hull shapes in one Design.
3. As a designer, I want a bilge radius, sheer and camber on a box hull, so that the volumes and the
   deck edge are those of a built barge.
4. As a designer, I want the block coefficient to follow from the hull, so that I do not guess a number
   that the shape already fixes.
5. As a designer, I want displacement, KB, BM, KM, LCB, LCF, TPC, MCT and the coefficients at every
   draught, so that I can read a hydrostatic table as a naval architect expects.
6. As a designer, I want the GZ curve of my real hull, with the trim it takes as it heels, so that the
   intact stability criteria read a true curve, not a box.
7. As a designer, I want to type or import an offsets table, so that I can check an existing hull.
8. As a designer, I want to turn the hull in 3D, so that I can see the shape, the decks, the bulkheads
   and the Spaces together.
9. As a designer, I want to hide and show the hull, decks, bulkheads, each Space type and the structure,
   so that I can see inside.
10. As a designer, I want to move a section plane along the hull, so that I can see the section at any
    station.
11. As a designer, I want to see the waterline of each loading condition, so that I can see the draught
    and trim of each.
12. As a designer, I want to click a Space and see its name, volume and centroid, so that I can check the
    arrangement.
13. As a designer, I want to export the hull and the arrangement to glTF, so that I can show it in other
    viewers.

### Reviewer or expert

14. As a reviewer, I want the hydrostatics to be exact for a box and a raked box, with the closed forms
    as tests, so that I can trust the integration.
15. As a reviewer, I want OVD's hydrostatics checked against a published dataset of parametric hulls,
    so that I can see how far they agree on shapes that are not boxes.
16. As a reviewer, I want an offsets table as CSV, so that I can check the hull in my own tools.
17. As a hydrodynamicist, I want a watertight panel mesh, so that I can run Capytaine or another solver
    on the same hull.

### Maintainer

18. As the maintainer, I want one geometry for every analysis, so that a fix in one place fixes all.
19. As the maintainer, I want the mesh in the native-against-WebAssembly parity check, outside the
    results hash, so that a tessellation change is seen but is not an engineering change.
20. As the maintainer, I want one format bump for the hull, the stern and the rule sets, so that users
    migrate once.

## Implementation Decisions

### Terms

New glossary terms (in `GLOSSARY.md`, added by the first phase 1 ticket):

- **Sections Table:** the master hull geometry of ADR 0012: closed section polygons at stations along
  the length. A Hull Form writes it; the Offsets Hull Form is one typed in.
- **Station:** a position x along the hull where the Sections Table holds a section.
- **Generator:** the code of a Hull Form that writes a Sections Table from the Hull Form's parameters.
- **Vessel Model:** the hull, arrangement and structure layers of a Hull (ADR 0018), from which every
  analysis and every view reads its geometry.
- **Waterplane:** the plane of the water surface relative to the hull, set by the draught, the heel and
  the trim.

The glossary entry **Hull Form** changes: "the parameters that set it, such as block coefficient and
rake lengths" becomes "the parameters that set it, such as rake lengths, bilge radius or deadrise". The
block coefficient is now a result.

### Axes and units

The core axes do not change: x forward from the aft end of the hull, y to starboard from the centre line,
z up from the base line (the keel at the lowest point of the moulded hull). Metres, square metres, cubic
metres, tonnes. Heel φ is positive to starboard (starboard goes down). Trim θ is positive by the bow (bow
goes down), as #349 has it.

### Modules (`ovd-core`, all `no_std`, `libm` only)

Each module has a small interface and plain structs in and out. None parses JSON.

| Module | Phase | Interface | Holds |
|---|---|---|---|
| `sections` (new) | 1 | `SectionsTable`, `Waterplane`, `immersed(&SectionsTable, &Waterplane) -> Immersed`, `waterplane_props(..)`, `upright(..)`, `equilibrium(..)`, `gz_curve(..)` | the data type, the cut, the integrator, the equilibrium solver |
| `hull` (changed) | 1 | `hull_form::*` generators, each `fn sections(&self) -> SectionsTable`; `HullFormInputs` and `fn generate(&HullFormInputs) -> Result<SectionsTable, HullError>` | the Hull Forms; `RakedBox` keeps its closed form as a test |
| `stern_form` (changed) | 1 | `geometry` as today; new `fn apply(&SternGeometry, &mut SectionsTable)` | writes the stern run into the sections; the closed forms stay as tests |
| `shipd` (new) | 1 | `ShipdParameters`, `fn check(&ShipdParameters) -> Vec<ConstraintFailure>`, `fn sections(&ShipdParameters, stations: u32) -> SectionsTable` | the ShipD-style parametric hull |
| `offsets` (new) | 1 | `fn from_rows(&[OffsetRow]) -> Result<SectionsTable, OffsetsError>`, `fn to_rows(&SectionsTable) -> Vec<OffsetRow>` | import and export of the typed offsets table |
| `hydrostatics` (changed) | 1 | keeps its entry points; they take `&SectionsTable` in place of `&HullGeometry` | the draught solver, stability and trim on the sections |
| `box_stability` (changed) | 1 | `box_condition_at` keeps its signature for its callers and calls `sections::gz_curve` | the IS Code criteria and areas as today |
| `vessel_model` (new) | 2 | `fn assemble(&VesselModelInputs) -> VesselModel` | hull, decks, bulkheads, Spaces of one Option |
| `compartments` (changed) | 2 | new `fn clip(&Space, &SectionsTable) -> ClippedSpace` | a Space cut by the hull; `Space` itself does not change |
| `mesh` (new) | 2 | `fn hull_mesh(..)`, `fn plane_mesh(..)`, `fn space_mesh(..)`, `fn waterline_mesh(..)`, `fn hash(&Mesh) -> [u8; 32]` | vertex and index arrays, layer tags |

### The Sections Table (phase 1)

```rust
pub struct Section {
    pub x: f64,                    // station, m from the aft end
    pub points: Vec<(f64, f64)>,   // half-section (y >= 0, z): keel at y = 0 first, deck at centre line last
}
pub struct SectionsTable {
    pub sections: Vec<Section>,    // x ascending; two sections may share an x (a knuckle or a step)
    pub depth_at_side: Vec<f64>,   // deck edge height per section (sheer); for the freeboard
}
```

- A section is the **starboard half** from the keel point on the centre line `(0, z_keel)`, round the
  bottom, the bilge or chine and the side, to the deck edge, and across the deck (with camber) to the
  centre line `(0, z_deck)`. The full section is this half and its mirror. So every full section is a
  **closed, simple polygon**. The deck closes it: the hull is watertight to the deck. No superstructure.
- **Same point count on every section.** Point `i` of every section lies on one longitudinal line (the
  keel, a chine, the deck edge, and so on). A generator writes them so. The offsets import checks it or
  resamples (see "Offsets table").
- An end section may have zero area (all points on the centre line, as at a stem). Two sections at the
  same x mark a step, for example a transom: the aft one may be a point and the next one the transom.
- The table is **derived**. It is not stored in `.vdesign`, except when the Hull Form is `offsets`, where
  the table is the input.

### Exact polygon integration (phase 1)

- **The waterplane** is the plane `p · k = c`, where `k` is the earth vertical in hull axes:
  `k = (−sin θ, −sin φ cos θ, cos φ cos θ)`, and `c` is its height. A point is immersed when `p · k < c`.
  Upright and level, `k = (0, 0, 1)` and `c` is the draught.
- **The cut of one section.** In the plane of a section at station x, the waterplane is a line. The
  engine clips the full section polygon to the immersed side of it (Sutherland–Hodgman against one half
  plane; the result is correct for a non-convex section). The shoelace formula gives the immersed area
  `A(x)` and its first moments in y and z exactly. The same clip gives the waterline chord (its ends and
  its length) and the immersed girth (the length of the immersed outline), for the wetted surface.
  This replaces the 100-strip KB and the box section of `box_gz`.
- **Along the length.** Between two stations the hull is the ruled surface joining point `i` of one
  section to point `i` of the next. The engine integrates `A`, `x A`, the moments in y and z, the
  waterline chord moments and the girth by **Simpson's rule on each interval**, with the mid-section taken
  from the ruled surface (point `i` at the mean of the two). Before that it **splits each interval** where
  any point line crosses the waterplane (the crossing is where `p · k − c` changes sign; it is linear
  along the point line, so it has one exact root). A generator puts stations at every knuckle and at both
  ends.
- **Accuracy.** Exact (to rounding) for a prismatic interval at any heel and trim, and wherever the
  immersed area is at most cubic in x, which covers the upright box and raked box, and the raked box at
  trim. Elsewhere the error falls with the station spacing; each generator test states its tolerance and
  shows the convergence.
- **Results of one cut** (`Immersed`): volume ∇, centre of buoyancy (LCB, TCB, KB), waterplane area,
  its centroid (LCF, TCF), its second moments about axes through its centroid (`I_T`, `I_L`), the wetted
  surface, the waterline length `L_WL` and breadth `B_WL`, and the midship section area at `L_WL / 2`.
- **Upright hydrostatics** (`upright`, at one draught, level): ∇, Δ = ρ∇, KB, BM_T = I_T / ∇,
  BM_L = I_L / ∇, KM_T, KM_L, LCB, LCF, A_WP, the wetted surface, TPC = ρ A_WP / 100,
  MCT 1 cm = Δ BM_L / (100 L_WL) (with KG = KB, the usual first estimate; the trim solver uses the
  real KG), and the coefficients: **Cb = ∇ / (L_WL B_WL T)** (ES-TRIN 2025/1, Article 1.01, 4.9), Cm =
  A_M / (B_WL T), Cp = Cb / Cm, Cwp = A_WP / (L_WL B_WL).
- **Equilibrium** (`equilibrium`, given mass Δ, centre of gravity G = (LCG, TCG, KG) and heel φ):
  the engine solves for the height `c` and the trim θ such that ρ∇ = Δ and the centre of buoyancy B lies
  on the vertical through G in the longitudinal plane (free trim). The inner solve finds `c` for the
  volume by the Illinois secant method in a bracket from the lowest to the highest point of the hull
  along k (the volume is 0 and the whole hull there); the outer solve finds θ for the longitudinal
  moment by the same method between ±10° (an Option that needs more fails with a message). Each solve
  takes at most a fixed number of steps (16 inner, 12 outer) and stops at the first step whose residual
  is within a tolerance (inner: 1e−13 of the target volume; outer: 1e−11 m of B − G). The stop compares
  numbers the solve computes, so native and WebAssembly stop at the same step and give the same bits.
  The heel 0 equilibrium of a condition is solved once and its GZ curve starts from it. (Amendment,
  2026-10-23, ADR 0020: this tolerance stop replaces the first rule, "fixed step counts and no early
  stop". Results moved by about 1e−16, and every results hash moved once.) A test
  checks the result against an 80 × 60 bisection to 1e−12. (Maintainer decision, 2026-10-21: bisection
  took about 13 s per 61-angle GZ curve on a 41-station hull, too slow for the capacity search; the
  secant solver takes about 1 s and agrees to 2e−15. Newton overshot at heavy trim.)
- **GZ** (`gz_curve`, angles in degrees, `box_stability::ANGLES_DEG` as today): at each heel, the
  equilibrium above, and GZ = the horizontal distance from G to the vertical through B, measured in the
  transverse direction, positive when righting. The free surface stays as today's correction to GM and
  to GZ (`GZ − FSM / Δ × sin φ`), with the fills of #426.
- **Deck edge and down-flooding.** The deck closes the section, so GZ past deck immersion is the true GZ
  of the closed hull. Down-flooding is still not modelled (#282, the README and the app say so).
- **Draught and trim.** The main path solves the equilibrium at φ = 0 with the lightship and cargo items
  of #349, so every Option gets draught aft, forward and at mid-length from the sections. An Option with
  no LCG data uses LCG = LCB (level), as today.

### Hull Forms (phase 1)

Each generator takes the main dimensions (L overall, B, D) and its own parameters, and writes a Sections
Table. Each has its exact or closed-form test.

| Hull Form (`hull_form`) | Parameters | Stations | Test |
|---|---|---|---|
| `box` | none | 0, L | ∇ = L B T, KB = T/2, BM = B²/12T, GZ of `box_gz` at every angle: to 1e−12 relative |
| `raked_box` | `bow_rake_length_m`, `bow_rake_rise_m` (and the Stern Form for the stern) | 0, the rake knuckles, L | `RakedBox` closed forms of `hull.rs` for ∇, A_WP, BM; LCB against hand prism-and-wedge values (triage item 29: the `hull.rs` stern-rake LCB is mirrored, so it is not the reference); to 1e−12 relative |
| bilge radius | `bilge_radius_m` (box, raked box, chine) | as the base form | the arc is written as `n` = 16 chords per quarter circle; ∇ against the closed form of the polygon to 1e−12, and against the true arc within the chord error |
| sheer | `sheer_aft_m`, `sheer_fwd_m` (parabolic, zero at mid-length) | as the base form, plus the ends | deck-edge height and the freeboard at side against the parabola; ∇ below the deck at side unchanged |
| camber | `camber_m` (parabolic across the deck) | as the base form | deck area and the volume above the deck edge against the parabola, within the chord error |
| Stern Forms (`stern_form`) | the inputs of spec 0007: transom, pram, tunnel | the run: 2 (transom), 17 (pram), 17 (tunnel) | transom: exact against `SternGeometry::removed`; pram: against `removed` within the chord error of 16 intervals; tunnel: see below |
| `v_bottom` | `deadrise_deg`, `keel_half_width_m` | as the raked box | ∇ and KB of the prism against the trapezoid and triangle closed form, exact |
| `chine` | `chine_half_breadth_m`, `chine_height_m`, `deadrise_deg` | as the raked box | as `v_bottom`, exact for the prism |
| `shipd` | the ShipD parameters (see "ShipD-style hull") | 101 by default | the validity constraints; the benchmark against the dataset |
| `offsets` | the typed table | as typed | round trip: export, import, the same table |

- **Tunnel stern.** The generator writes the shape the drawings show (`SternGeometry::profile` and
  `tunnel_sections`): a tunnel of radius D whose crown falls linearly from `tunnel_crown_height_m` at the
  transom to the base at the end of the run. The closed form of #350 assumes instead that the immersed
  tunnel area falls linearly. The two differ when the crown is near the waterline. The test compares the
  sections with an independent strip integral of the same shape to 1e−4 relative at 400 strips, and with
  the closed form at the transom section to 1e−12. The difference to the closed form is part of the
  design-impact table.
- **The Notch** of an ATB barge (spec 0007, #362) is a stern parameter of the generator (#465), with no
  closed form in `stern_form.rs`. A half-section may then start and end off the centre line (a slot), so
  its full section is two polygons; the Notch's forward end is a step. #362 is blocked by #465.
- **Bow.** Every form but `shipd` and `offsets` has the raked-box bow: a straight rake from the base at
  `L − bow_rake_length_m` to `bow_rake_rise_m` at the bow. Today's main-path inputs give these values, so
  an old file's drawn bow rake now enters the hydrostatics.

### Inputs (`.vdesign`, format version 3: the hull block)

Version 3 is open (#347) and no release has shipped it. The hull block joins it under the rule of ADR
0007 and #274. **0.3 ships version 3 once**, with the stern inputs, `service_area` and `rule_sets`
(#274) and the hull block. One bump in all.

The hull block is a new input group, **Hull Form**, in each Option:

| Input | Values | Applies when | New Option default | File of version 1 or 2 |
|---|---|---|---|---|
| `hull_form` | `box`, `raked_box`, `v_bottom`, `chine`, `shipd`, `offsets` | always | `raked_box` | `raked_box` |
| `bow_rake_length_m`, `bow_rake_rise_m` | m | not `shipd`, `offsets` | from today's bow rake over depth rule | from the file's own bow rake inputs |
| `bilge_radius_m` | m, 0 to B/2 | `box`, `raked_box`, `chine` | 0 (estimate) | 0 |
| `sheer_aft_m`, `sheer_fwd_m` | m | not `offsets` | 0 | 0 |
| `camber_m` | m | not `offsets` | 0 | 0 |
| `deadrise_deg`, `keel_half_width_m` | degrees, m | `v_bottom`, `chine` | profile values (estimate) | not held |
| `chine_half_breadth_m`, `chine_height_m` | m | `chine` | profile values (estimate) | not held |
| `shipd_parameters` | an object, one entry per ShipD parameter | `shipd` | the dataset's first hull (estimate) | not held |
| `hull_offsets` | a table: rows of `station_x_m`, `y_m`, `z_m` | `offsets` | none: typed or imported | not held |

- **`block_coefficient` leaves the hydrostatics.** The input is removed from version 3. A file of
  version 1 or 2 that stores one is read; the value is dropped with one warning per Option ("The block
  coefficient now comes from the hull shape"). The block coefficient is a result (Cb above). Issue #427
  (the 0.928 default) is superseded for the hydrostatics by this change; it stays open only if the
  maintainer keeps a Cb input elsewhere.
- **The Stern Form inputs of spec 0007 do not move.** They stay in their group and the generator reads
  them. No second input set (ADR 0017).
- `ovd schema` lists every new input with its group, unit, range, applies-to and default status.
  `shipd_parameters` and `hull_offsets` are the first object-valued inputs; the schema gives their shape.
- The app's inputs panel gets the Hull Form group: a dropdown and the parameters of the chosen form.
  `shipd_parameters` shows as a collapsed table. `hull_offsets` shows its station count and an Import
  button.

### Offsets table (phase 1)

- **CSV**, one row per point, UTF-8, comma-separated, header `station_x_m,y_m,z_m`, the points of each
  station in order from the keel to the deck centre line, stations in ascending x. A blank line between
  stations is allowed. Metres. The same format is the phase 2 export.
- **Import:** `ovd hull import <file.csv> --option <name>` and an Import button in the app write the
  table into `hull_offsets` and set `hull_form` to `offsets`. Checks, each with a message: at least two
  stations; x ascending; each half-section starts and ends on the centre line; y ≥ 0; no self-crossing;
  the same point count on every station. A table whose stations differ in count is **resampled**: each
  station is re-cut to the largest count by arc length, with its corners kept, and a warning says so.
- The main dimensions L, B and D of an `offsets` Option become read-only results of the table (L = x
  range, B = twice the largest y, D = the smallest deck edge height).

### ShipD-style hull (phase 1)

**Sources, read for this spec.** Bagazinski, N. J., and Ahmed, F., "Ship-D: Ship Hull Dataset for Design
Optimization Using Machine Learning", Proc. ASME IDETC/CIE 2023, Vol. 3A, paper V03AT03A028
(arXiv:2305.08279, v2 read). The dataset: Bagazinski, Noah; Ahmed, Faez, 2024, "Ship-D",
https://doi.org/10.7910/DVN/MMGAUS, Harvard Dataverse, V1, licence CC0 1.0 (from the Dataverse record).

- **What the sources give.** The paper sets out a hull of **45 parameters** (7 principal dimensions, 4
  midship section, 20 bow and stern taper, 14 bulb) and **49 algebraic validity constraints** (the hull
  is watertight and does not cross itself), and describes the surface in words: a cubic waterline at each
  height between the stem and the full-breadth line, bulbs merged by fourth-order polynomials. The paper
  body has no equations and no parameter table. The parameter table with ranges and scaling is the
  dataset's `HullParameterizationOverview.pdf`; the constraint list is
  `HullParameterization_ConstraintsOverview.pdf`; the construction is in the dataset's
  `HullParameterization.py`. The GitHub repository of the authors has no licence.
- **What OVD does.** It writes its own generator in Rust, `shipd.rs`, from those **definitions**. It copies
  no code, from either place. It is a Sections Table generator, not a waterline-stack mesher: each station
  is cut through the parametric surface at fixed heights.
- **Parameters.** `shipd_parameters` holds the 45 terms under the dataset's header names, in the
  dataset's non-dimensional form. LOA, `Bd` and `Dd` are not stored there: they come from
  `length_overall_m`, `beam_m` and `moulded_depth_m`. The dataset's x runs aft from the stem; OVD's
  runs forward from the aft end, so x = L − x_ShipD.
- **Surface, in short** (the generator ticket holds the full definitions):
  - the midship section: a keel arc (radius `Rk`, which may be negative), a bottom line at deadrise `Beta`,
    a side line from the chine to the deck edge (flare or tumblehome), and a chine arc of radius `Rc`
    tangent to both;
  - the bow: a parabolic stem rake above `BK_z`, a parabolic keel rise below it, a full-breadth line and a
    drift angle that vary with height, and at each height the cubic waterline from the stem (slope from
    the drift angle) to the midship half-breadth (zero slope);
  - the stern: a straight transom line above `SK_z`, a parabolic stern rise below it, a taper-start line,
    a transom section built as the midship section with its own terms, and at each height an ellipse or a
    parabola (`bit_EP_S`, `bit_EP_T`) from the midbody to the transom;
  - the bulbs: ellipsoidal noses, blended by a quartic fillet.
- **Stations and heights.** 101 stations by default, plus the knuckles (start and end of the midbody,
  `BK`, `SK`, the transom). Each section has its points at the same set of heights: 40 even levels from
  the base to the deck, plus `BK_z`, `SK_z`, the chine heights and the bulb heights. A point is on the
  centre line where the waterline at that height has not yet begun.
- **Constraints.** `shipd::check` gives each of the 49 as a value `g`, satisfied when `g ≤ 0`, with its
  index and name. An Option whose parameters fail any of them gets no hull and a message naming each one.
  Where the dataset documents disagree with each other (a base of `Kappa_SB` or `Bbb`, a scale applied
  twice in one bow check, "> 1" written for "> 0"), the generator ticket records the reading it takes and
  why.
- **Two tickets.** The hull without bulbs first (`bit_BB` = `bit_SB` = 0); the bulbs second. Inland and
  coastal hulls rarely have bulbs.
- **Bulbs as built (#453).** Each bulb is an ellipsoid with its widest part at the height `Hbb` of the
  design draught (`Hsb` of its own height `HSBOA` times the design draught, for the stern bulb), where its
  half-width is `Bbb` (`Bsb`) of the midship half-breadth. At each height below the design draught the
  waterline runs along the bulb to the half-width the hull's waterline reaches at the bulb's widest part,
  with a quartic fillet between: it matches value and slope at both ends and has no curvature at the bulb
  end. The dataset's code writes that curvature row with `6 x^2` where the second derivative has `6 x`; OVD
  writes the definition, and the benchmark report lists the hulls where the two differ. The stern bulb
  starts `Kappa_SB` of `Ls` along the stern taper (the code and its comment say `Ls`; the overview PDF says
  `Lb`). The constraint rows 36 to 48 follow the constraints PDF; the dataset's code has the sign of row 40
  the other way round, and OVD follows the PDF. A bulb needs a zero deadrise or a positive keel radius. The
  Set 1 sample passes the pass rule of #452 for most of its hulls; each hull outside is in the report.

**Benchmark against the dataset.**

- **Sample.** A fixed-seed sample of 100 hulls from each of the dataset's Constrained Randomized Sets 2
  (no bulbs) and 3 (zero deadrise, positive keel radius) for the first ticket, and 100 from Set 1 (with
  bulbs) for the second. Only hulls that pass the 49 constraints. The sampled rows (input vectors and
  the reference values) are committed under `calibration/shipd/`, a few hundred kilobytes, with a
  `SOURCE.md` that gives the citation, the CC0 licence, the files and the seed.
- **Quantities.** At the dataset's ten draughts (T as a fraction of the depth): volume, waterplane area,
  LCB, LCF, the waterplane second moments `Ixx` and `Iyy`, and the waterline length. All are divided by
  powers of LOA (LOA = 10 in every hull of the dataset), so OVD runs each hull at L = 10 m.
- **Not compared.** The dataset's vertical centre of buoyancy is not usable: its rows are identical to
  the LCB rows in the sets checked. Its wetted surface leaves out the surface slope, so it runs low on
  sloped hulls. The wave resistance is outside phase 1. OVD reports its own KB, wetted surface and
  coefficients beside the comparison, unchecked.
- **Pass.** For at least 95 % of the sampled hulls: volume, waterplane area and waterline length within
  1 %; LCB and LCF within 0.5 % of LOA; `Ixx` and `Iyy` within 2 %. Every hull outside is listed in the
  report with its numbers. The dataset integrates waterline strips by the trapezoid rule, so part of any
  difference is theirs; the report says so. The tolerances are not loosened to make a hull pass: a
  failing hull is a finding.
  (Amendment, 2026-10-23, #648.) The gate measures OVD's shape the way the dataset measured its own
  (101 waterlines, 1000 points each, the trapezoid rule, interpolation between waterlines:
  `shipd::dataset_measures`), so it compares the shape, not the integration rule; OVD's own
  integration stays in the report for information. Every sampled hull must pass. The one exception is
  a hull listed in `calibration/shipd/artefacts.csv` with a cause where the dataset's code departs
  from the definitions (`stern_ellipse`, `bulb_crossing`); a test shows that it misses by the
  definitions and that reading that one place as the code does brings it within the limits. Any other
  hull outside fails the gate.
- **Output.** `calibration/shipd/report.md`, regenerated by `cargo run -p xtask -- shipd-benchmark`, and a
  test that runs a 10-hull subset in the fast gate.
- **Credit.** `NOTICE` thanks the authors and cites the paper and the dataset (added with this spec).

### Design impact of phase 1

One switch-over ticket moves every result at once and writes **one design-impact table** in
`CHANGELOG.md` (`python3 tools/design_impact.py changelog`). What moves, and why:

- **Displacement at a draught:** from `0.85 × L × B × T` (or the stored Cb) to the volume of the raked
  box with its own bow rake and Stern Form. For a box barge with today's bow rake this is about Cb 0.93 to
  0.97 on L overall: draughts fall, weight-bound cargo rises (basis page 01, HF-A26; #427).
- **KB, BM, LCB, LCF** from the real hull. The legacy path's double use of Cb on the raked-box volume
  (triage item 7) and the mirrored stern-rake LCB (triage item 29) go.
- **GZ** from the sections with free trim, not the box section: at large heel the bow rake and the
  stern run lose waterplane, so GZ falls a little; trim by the stern at heel is now seen.
- **Consumers of Cb:** resistance (`resistance.rs`), squat (`squat.rs`), journey de-rating
  (`journey_derating.rs`), electric energy (`electric_energy.rs`), gas stability's draught solve
  (`gas_stability.rs`) and the capacity search (`barge_capacity.rs`, `gas_arrangement.rs`) read Cb, the
  midship area and L_WL from the hull results, not from an input. The midship area becomes `A_M` of the
  sections (closes #396 and #425 for the hydrostatic hull).
- **Tunnel stern:** the difference above.
- New result keys: `hydrostatics` (the upright values at the equilibrium) and `hydrostatic_table` (the
  upright values at 10 draughts from 0.1 D to D). Both are in the results hash.
- The evaluation harness must keep 0 false passes. The calibration report is re-run and its bands are
  re-read in the PR body; a case that leaves its band is listed, not hidden.

### Phase 2: the Vessel Model

```rust
pub struct VesselModel {
    pub hull: SectionsTable,
    pub decks: Vec<Deck>,           // name, z at side, x_aft..x_fwd; a plane clipped by the hull
    pub bulkheads: Vec<Bulkhead>,   // name, x, watertight: bool; a plane clipped by the hull
    pub spaces: Vec<ClippedSpace>,  // compartments::Space and its hull-clipped volume, centroid
    pub frames: Vec<f64>,           // frame stations; empty until phase 3
}
```

- **No new inputs.** The core assembles the model from the Option's inputs through the modules that
  already place things: the hold or tank layout, the double bottom, `compartments::ballast_tanks`, the
  machinery space of #352, the end bulkheads. Every layer is data in `.vdesign` because its parameters
  are.
- **One compartment model.** `compartments::Space` (#426) does not change. The crude oil Spaces of #330
  use it. Phase 2 adds `compartments::clip(&Space, &SectionsTable)`: at each station the engine
  intersects the hull's full section with the Space's section, a convex polygon (a Space with a concave
  section is split into convex parts by its builder), by Sutherland–Hodgman, and integrates as above. A
  Space wholly inside the hull keeps its prism volume exactly. A Space that the hull cuts (a peak tank in
  a rake) gets its true volume. The phase 2 ticket that puts the clipped volumes into the results has its
  own design-impact table.
- **`SpaceKind` grows by tickets.** Phase 2 adds the kinds the arrangement already places, so every
  Space in the viewer has a type: `cargo_hold`, `cargo_tank`, `void`, `cofferdam`, `machinery`,
  `peak_void`. The crude oil kinds come with #330.

### Phase 2: meshes and the parity hash

```rust
pub struct Mesh {
    pub positions: Vec<f32>,   // x, y, z per vertex, hull axes, metres
    pub indices: Vec<u32>,     // three per triangle, counter-clockwise seen from outside
    pub groups: Vec<MeshGroup>,// layer, id (a Space name, a deck name), index range
}
```

- The core computes in f64 and casts to f32 once, at the end. Normals are left to three.js.
- **Hull:** for each pair of neighbouring sections and each pair of neighbouring points, two triangles of
  the ruled quad; the full section (both sides); end caps by ear clipping. Coincident vertices are welded
  and zero-area triangles dropped. The result is closed and consistently oriented.
- **Decks and bulkheads:** the plane clipped to the hull section at that x or z, triangulated.
- **Spaces:** each clipped Space as a closed mesh, one group per Space.
- **Waterline:** for each loading condition, the waterplane cut polygon (from `equilibrium`) as a flat
  mesh.
- **Mesh hash:** SHA-256 of the f32 position bytes, the u32 index bytes (little-endian) and the group
  table, per mesh, in a fixed order. `tools/web_parity.mjs` compares it between native (`ovd mesh
  --hash`) and the WebAssembly session. It is **not** in the results hash (ADR 0018). A mismatch fails
  the parity check as a results-hash mismatch does.

### Phase 2: the 3D viewer (app and web app, shared Svelte front end)

- **three.js** (MIT), pinned to an exact version in `app/package.json`, imported only by the new view
  component. The component holds no geometry: it asks the core for meshes through the platform layer
  (`app/src/lib/platform/`: Tauri commands on desktop, the WebAssembly session on the web) and draws them.
- A new main-panel view, **3D**, beside the drawing views.
- **Navigation:** orbit, zoom and pan (three.js `OrbitControls`), a reset-view button, and fixed views
  (profile, plan, body plan).
- **Layer toggles:** hull, decks, bulkheads, Spaces by type (one toggle per `SpaceKind` present) and
  structure. The structure toggle is shown but disabled, with the note "from phase 3", until the
  structure layer exists. The hull is translucent while any inner layer is on.
- **Section plane:** a slider moves a clipping plane along x; the core gives the section outline at that
  x (`section_at(option, x)`) and the view draws it in the plane.
- **Waterline:** a loading-condition dropdown shows the waterline of that condition, with its draughts and
  trim in a label.
- **Colours** by Space type from the app theme tokens (`docs/design/style-guide.md`), light and dark,
  checked by `scripts/contrast.mjs` against the background.
- **Pick:** a click on a Space shows its name, kind, volume (m³) and centroid (LCG, TCG, VCG, m) in a
  small panel. The values come from the core, not from the mesh.
- **No editing in 3D.** Inputs stay in the inputs panel. A changed input rebuilds the meshes, as it
  rebuilds the drawings.
- Text through the Text Library (spec 0005); every new string has a Message Id.
- **Accessibility and fall-back:** keyboard orbit (arrow keys) and a text table of the Spaces beside the
  canvas. Without WebGL the view says so and shows the table.

### Phase 2: exports

| Export | Format | Holds | Command |
|---|---|---|---|
| View | glTF 2.0 binary (`.glb`) | one node per layer and per Space, named; colours as materials | `ovd export --format gltf`, app menu |
| Panel mesh | binary STL, or OBJ | the closed hull to the deck, outward normals, watertight; option `--below <condition>` clips it at that waterline and closes the lid | `ovd export --format stl\|obj` |
| Offsets | CSV | the Sections Table in the import format | `ovd export --format offsets-csv` |

- The writers live in `ovd-format` (`export.rs`, new) and are called from the CLI, the shell and the web
  crate, so every surface writes the same bytes.
- **Watertight test:** every edge is shared by exactly two triangles with opposite direction; the mesh
  volume by the divergence theorem matches the sections volume within the ruled-quad error (stated in the
  test); the glTF passes the Khronos glTF validator in CI.
- **Capytaine:** a Python test, skipped when Capytaine is not installed, loads the STL with
  `--below <condition>` and checks that Capytaine reports the hull as closed and its displaced volume
  within 1 % of OVD's.
- **Panel mesh and offsets (#461):** the STL and OBJ are in hull axes (x forward from the aft end, y to
  starboard, z up from the base line), metres, welded as f32 (vertices a nanometre apart are one point in a
  file). With `--below <condition>` the hull is cut at that condition's waterline (the trimmed plane of its
  aft and forward draughts) and the cut is closed by a lid, counter-clockwise seen from above; the cut points
  are shared by the two triangles of an edge, so the mesh stays closed. The offsets CSV is the Sections Table
  in the import format of #451, one row for each point, a blank line between stations.
- STEP and IFC come later, through an adapter (phase 6 or later).

### File ownership and hand-over

A new **geometry builder lane** builds this spec. It owns, from the hand-over:

- `crates/ovd-core/src/hull.rs`, `hydrostatics.rs`, `stern_form.rs`;
- the new `sections.rs`, `shipd.rs`, `offsets.rs`, `vessel_model.rs`, `mesh.rs`;
- `crates/ovd-format/src/export.rs` and the hull block in `vdesign.rs` (format version 3 stays with the
  tank and stern lane: the geometry lane adds its group under their rule);
- the 3D view component in `app/src/lib/views/`.

Hand-over rules:

- **Tank and stern lane** (owns format version 3; built #350 `stern_form.rs`): agreed 2026-10-21, the
  hand-over follows the merge of #353 and #414 (the mirrored stern-rake LCB). Its next tickets (#354,
  #357, #356) do not edit these files. After it, a stern ticket that needs the hull (the Notch of #362, the stern drawings of
  #356) asks the geometry lane, or the two lanes agree a ticket order on the issue.
- **Basis-fixes lane** (#426: Spaces, ballast tanks, loading conditions): `compartments.rs` stays with it
  until #426 closes. The seam is `box_stability::box_condition_at`: its signature holds, and the
  switch-over ticket makes it call `sections::gz_curve`. The hydrostatics switch-over is blocked by the
  #426 part that changes the default loading conditions, so the two design-impact tables do not mix.
- **Rules lane** (#274, #275): the hull block and #274 share version 3; whoever lands second rebases. The
  hull block does not wait for #274, and #274 does not wait for it.

### Rules (spec 0004)

No new Rule in phases 1 and 2. The IS Code criteria read the new GZ curve through `box_condition_at`.
Every Measure that reads the draught, Cb or GZ moves with the switch-over and is in its table.

## Testing Decisions

- **Through the interfaces:** `sections` is tested by its functions, not by its internals; each generator
  by `generate`; the switch-over by `ovd run` on the fixtures.
- **Closed forms are the oracles:** the box (all angles), the raked box (upright, trimmed), the prism
  V-bottom and chine, and the Stern Forms, as in the Hull Forms table. Labels: `sourced - closed form,
  <form>`.
- **Property tests:** volume is monotonic in draught; ∇ and the centroid do not change under a split of a
  prismatic interval; a symmetric hull has TCB = 0 upright and GZ(−φ) = −GZ(φ); GZ(0) = 0; at small heel,
  GZ / sin φ tends to GM; a free-trim equilibrium of a fore-and-aft symmetric hull with LCG at mid-length
  has θ = 0.
- **Parity:** the results hash and the mesh hash agree native and WebAssembly on every fixture.
- **Benchmark:** the ShipD sample (below), reported in the PR and kept as a test with its tolerance.
- **Old files:** every v1 and v2 fixture opens, with one warning per Option for the dropped Cb and the
  new hull inputs; the switch-over ticket re-pins its snapshots by running the code (build process,
  "Fixed-number tests").
- **Viewer:** an e2e test opens a fixture, shows the 3D view, toggles each layer, moves the section plane
  and clicks a Space; screenshots light and dark.
- `ovd-core` stays `no_std`; `bash tools/gates.sh` passes, WebAssembly build included.

## Out of Scope

- The structure layer (phase 3), rule scantlings (phase 4), longitudinal strength (phase 5), the
  companion adapters (phase 6), manoeuvring (phase 7): see "Later specs".
- Damage stability (a later spec on the Vessel Model; the clipped Spaces are its input).
- Down-flooding points and openings (#282).
- NURBS surfaces, smoothing and fairing. Superstructures and deckhouses in the hydrostatics.
- Editing geometry in 3D.
- An operational twin with live data (ADR 0018).
- STEP and IFC.

## Later specs (phases 3 to 7)

Each later phase gets its own spec. In short:

> **Phases 3 and 4, and the structural export of phase 6, are spec 0013** (plate-level hull steel). It
> exports to OCX (Open Class 3D Exchange) instead of STEP or IFC. Phase 5 stays a separate spec.

- **Phase 3: structure layout.** Frame spacing, longitudinals, stiffener spacing and plate zones as
  parameters tied to the frame stations, in the Vessel Model and the viewer (the structure toggle).
- **Phase 4: rule scantlings.** Through the rules engine, starting `unverified`: Bureau Veritas NR217
  (Rules for the Classification of Inland Navigation Vessels, June 2025) for inland, and the ABS Rules for
  Building and Classing Steel Barges (January 2024) for coastal. It gives a bottom-up steel weight and the
  section modulus.
- **Phase 5: longitudinal strength.** The still-water bending moment and shear force per loading
  condition from the Vessel Model's weight and buoyancy along the length; the **rule** wave bending
  moment (NR217 Part B, Chapter 3, Section 3; the ABS barge rules); and the check against the section
  modulus of phase 4.
- **Phase 6: companion adapters.** Capytaine, with its direct wave bending moment as a comparison only,
  never the check; an export for finite-element tools; STEP and IFC through an adapter.
- **Phase 7: recorded, not designed.** Concept manoeuvring indicators, for example the ES-TRIN 2025/1
  Chapter 5 manoeuvrability requirements as Rules, and turning and stopping estimates; and a coefficient
  export for ship-handling simulators.

## Decisions taken and open questions

Taken with the maintainer, 2026-10-21:

1. One engineering Vessel Model is the single source of truth for geometry; rendering is one view of it.
2. ADR 0011 holds: OVD owns the model and the concept-level analyses; companion projects do detailed
   FE, CFD, seakeeping and ship-handling simulation, fed by open exports.
3. A design twin now; an operational twin is a future direction only.
4. Parametric layers: the sections table is the master surface; an arrangement layer of planes and boxes
   clipped by the hull, with one compartment model; a structure layer of parameters. Meshes are derived.
5. The Rust core builds the meshes; three.js draws them in the shared Svelte front end; the mesh hash is
   in the parity check, outside the results hash.
6. Hydrostatics move to the sections table in phase 1 by exact polygon integration, with heel and free
   trim. The box and raked box become generators with exact tests. Results move, with one design-impact
   table.
7. The hull block joins format version 3 before 0.3 ships: one bump in all.
8. Phase 1 hulls: box, raked box, bilge radius, sheer, camber, the three Stern Forms, V-bottom, chine,
   a ShipD-style generator written from the paper, and offsets import.
9. Phase 2 exports: glTF, STL or OBJ, offsets CSV. STEP and IFC later.
10. The viewer as above; no editing in 3D.
11. The phases 1 to 7 above; 0.3 ships at the end of phase 1.
12. The block coefficient becomes a result, Cb = ∇ / (L_WL B_WL T); the input is dropped from version 3
    with a warning; old files take the raked box from their own rake inputs.

Open (for the build, not blocking):

- Default values (estimates) for the deadrise, keel width, chine and bilge radius of a new Option: the
  first generator tickets propose them with a basis; the maintainer may change them in the profile.
- Whether #427 stays open for any Cb input outside the hydrostatics (none is known).
- The station counts above are starting values; a generator ticket may raise them if its convergence
  test asks for it.

## Build tickets

Parent issue #442. Each ticket has native blocked-by links; a builder takes the next ticket whose
blockers are all closed (`docs/agents/build-process.md`).

**Phase 1: hull geometry and hydrostatics**
1. #443 Sections Table type and exact polygon integrator, exact on the box
2. #444 equilibrium with free trim and the GZ curve on the sections (blocked by #443)
3. #445 box and raked box generators write sections (blocked by #443)
4. #446 format version 3 hull block (inputs, schema, legacy values) (blocked by #445)
5. #447 the Stern Forms (transom, pram, tunnel) write sections (blocked by #445, #353)
6. #448 hydrostatics and GZ switch to the sections table; Cb becomes a result (blocked by #444, #446, #447, #426)
7. #449 bilge radius, sheer and camber (blocked by #448)
8. #450 V-bottom and chine hull forms (blocked by #448, #449)
9. #451 import a typed offsets table (blocked by #448)
10. #452 ShipD-style parametric hull (no bulbs) and its benchmark (blocked by #448)
11. #453 ShipD-style bulbous bow and stern, and their benchmark (blocked by #452)
12. #465 the ATB Notch as a stern parameter of the sections (blocked by #447, #448, #357); #362 waits for it

**Phase 2: Vessel Model, 3D viewer, exports**
12. #454 assemble the Vessel Model (hull, decks, bulkheads, Spaces) (blocked by #448, #426)
13. #455 Spaces clipped by the hull, in the results (blocked by #454)
14. #456 meshes from the core, and the mesh hash in the parity check (blocked by #454)
15. #457 mesh and section calls in the WebAssembly session and the desktop shell (blocked by #456)
16. #458 3D view with three.js: orbit, layers, colours by Space type, pick (blocked by #457)
17. #459 movable section plane and the waterline of each loading condition (blocked by #458)
18. #460 export to glTF (blocked by #456)
19. #461 watertight STL or OBJ panel mesh and offsets CSV export (blocked by #456, #451)

**Before 0.3 ships (maintainer):** phase 1 closed, #274 and #275 closed, format version 3 frozen, the version bump.
