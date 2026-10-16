# The master hull geometry is a table of sections (offsets)

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
