# Tank-shaped Spaces and an Outfit layer in the Vessel Model

Status: **accepted**, 2026-10-30, with spec 0009. Amends ADR 0018 (it keeps every other part of it).

## Context

ADR 0018 gives the Vessel Model three layers (hull, arrangement, structure) and one compartment model: the
`Space` of `compartments.rs`, a prism clipped by the hull, with "no second Space type". Spec 0009 draws
the Type C cargo tanks, the deck units, the wheelhouse and the propulsors. A Type C tank is a cylinder
with heads; it is not a prism and it never touches the hull. The deck units, the wheelhouse and the
propulsors are not compartments, and spec 0008 kept every superstructure out of the hydrostatics. Their
masses are already in the lightship (spec 0007), but no layer holds their shape.

## Decision

- **A tank is a Space.** `Space` gains a shape: `SpaceShape::Prism { section }` (today's) or
  `SpaceShape::Tank { axis_x0, axis_x1, y, z, outer_diameter, head }`. There is still one Space type.
  `clip` passes a tank through unchanged. A tank's volume and centroid come from the tank rule values,
  not from its mesh. The tank Spaces are in the results Spaces table.
- **A fourth layer, Outfit,** holds the parts that are neither hull, compartment nor structure: the Deck
  Items (accommodation and battery units, the wheelhouse lowered and raised, its lift column) and the
  propulsors (propellers, nozzles, azimuth legs and pods, rudders, the bow thruster). Outfit is **not in
  the hydrostatics**. It adds geometry only: its masses stay where spec 0007 put them.
- **One place for each position.** The positions of the tanks and of the propulsors come from one core
  function each (`tanks::tank_layout`, the position functions of `machinery.rs`). The capacity, the
  loading conditions, the mass items, the drawings and the 3D view all call them.
- **The view never changes the Design.** Adding Outfit or tank shapes moves no existing result value.
- Everything else in ADR 0018 holds: parameters are stored, meshes are derived, the core builds them,
  the mesh hash is outside the results hash.

## Why

- A tank that is drawn, picked and holds cargo is a compartment. IGC damage stability (#663) will need it
  as one. A second compartment type would split every analysis that walks the Spaces.
- The deck items and propulsors are not compartments. Forcing them into `Space` would put them in the
  hydrostatics, against spec 0008.
- One position function per part means the picture, the drawings and the lightship cannot disagree.

## Consequences

- `compartments.rs` gains `SpaceShape`; existing Spaces are `Prism` and keep their values.
- `VesselModel` gains `outfit`. `mesh.rs` gains tank, Deck Item and propulsor meshes, each with a typed
  source.
- The results Spaces table gains one row per tank: a design-impact table with added rows only.
- The glossary gains Outfit, Deck Item, Tank Layout and Propulsor Layout (spec 0009).
