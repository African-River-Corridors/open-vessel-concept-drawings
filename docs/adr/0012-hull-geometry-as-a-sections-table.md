# The master hull geometry is a table of sections (offsets)

Status: **accepted**, 2026-10-21, with ADR 0018 and spec 0008 (phase 1 builds it).

Today every hull is a raked box, described by a few numbers. Damage stability, longitudinal strength,
construction drawings and CFD all need a real hull shape. All four depend on how that shape is stored, so
the choice must be made once, before any of them is built.

## Decision

- The **master** hull geometry is a **table of sections**: offsets (half-breadths at given heights) at a set
  of stations along the length, with the deck and keel lines.
- **Smooth surfaces (NURBS) and meshes are derived** from the sections table by adapters, for drawing,
  import from and export to other tools, and CFD. They are never the master.
- Hull Forms (raked box, V-bottom, chine hulls and others) are **generators** that write a sections table
  from their parameters.
- **The raked box stays** as a Hull Form, and its closed-form results stay as an exact test case for the
  sections-table code.

## Why

- Sections integrate exactly for hydrostatics and stability (areas, volumes, centroids) with simple,
  reviewable maths that needs no geometry library and runs in WebAssembly.
- A sections table is easy for a naval architect to read and check, and it is how lines plans and
  offsets are already exchanged.
- NURBS as the master would need a geometry kernel in the core; a mesh as the master would lose the exact
  hydrostatics.

## Consequences

- The `.vdesign` format (ADR 0007) gains a hull geometry block in a later format version; existing files
  keep working through the raked-box generator.
- Compartments, the structural model and the CFD and seakeeping adapters all read the sections table.

## Added on 2026-10-21 (spec 0008, phase 1)

The decision above holds. Spec 0008 sets how it is built:

- **Exact polygon integration.** Each section is a closed polygon. The engine cuts it at the waterplane
  (upright, heeled, trimmed, or all three) and takes the area and moments of the cut polygon exactly, by
  the shoelace formula. No strips. Along the length the hull between two stations is the ruled surface
  that joins their matching points. The engine integrates along the length by Simpson's rule on each
  interval, with a mid-section from that ruled surface, and adds a station wherever a section corner
  crosses the waterplane. A generator puts stations at every knuckle and at both ends. This gives the
  displacement, KB, BM (transverse and longitudinal), LCB, LCF, the waterplane, the wetted surface and the
  GZ curve with heel and free trim. The box results are exact at every heel. The upright raked box is
  exact too: its immersed section area is linear in x.
- **The block coefficient becomes a result.** It is the displaced volume over L_WL × B_WL × T (ES-TRIN
  2025/1, Article 1.01, 4.9), taken from the sections. It is no longer an input of the hydrostatics.
- **Generators.** Box and raked box; bilge radius, sheer and camber; the transom, pram and tunnel Stern
  Forms of ADR 0017 (they now write sections, from the same inputs); V-bottom and chine; a
  ShipD-style parametric hull, written here from the published parameter definitions; and the import of
  a typed offsets table. Each closed form that a generator replaces stays as its test.
- **Format.** The hull block joins `.vdesign` format version 3, which #347 opened and no release has
  shipped. Version 3 ships with 0.3, once. A file of version 1 or 2 gets the raked-box generator from its
  own main dimensions and rake inputs.
- **Results move once.** The switch-over to the sections table is one change with one design-impact
  table. It is not hidden behind a legacy path.
