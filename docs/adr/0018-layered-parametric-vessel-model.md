# One layered, parametric Vessel Model; meshes are derived; three.js draws them

Status: **accepted**, 2026-10-21, with spec 0008.

## Context

OVCD holds its geometry in several places. The hydrostatics use a box or a raked box in closed form. The
stability uses a prismatic box section. The Stern Forms (ADR 0017) correct the box in closed form. The
ballast Spaces (#426) are prisms. The drawings build their own outlines. No 3D view, mesh or WebGL code
exists. The engineering basis review named the box hydrostatics and the box GZ as a key weakness.
Damage stability, longitudinal strength, scantlings and the companion solvers all need one shape of the
vessel that they agree on.

## Decision

- **One engineering Vessel Model** is the single source of truth for the geometry of a Hull. Every
  analysis reads it. A 3D view is one view of it, as a drawing is.
- **Three parametric layers, all data in `.vdesign`:**
  1. **Hull.** The sections table of ADR 0012 is the master surface. Hull Forms are generators that
     write it.
  2. **Arrangement.** Decks and bulkheads are planes. Spaces are boxes or prisms. The hull clips them
     all. There is **one compartment model**: the `Space` type of `compartments.rs` (#426, the ballast
     tanks), which the crude oil Spaces (#330, ADR 0016) also use. No second Space type.
  3. **Structure.** Frame spacing, longitudinals, stiffener spacing and plate zones, as parameters tied
     to the frame stations.
- **Meshes are always derived.** A mesh is never stored in `.vdesign` and never read back as input.
- **The core builds the meshes.** `ovcd-core` (`no_std`, `libm`) turns the layers into vertex and index
  arrays. The desktop app and the web app draw them with **three.js**, in the shared Svelte front end. The
  front end holds no geometry; it only draws what the core gives.
- **A mesh hash in the parity check.** The native-against-WebAssembly parity check gains a hash of each
  mesh. It is **kept outside the results hash**, as the Rule Trace is. A mesh change does not move the
  results hash, and a results change does not hide a mesh difference.
- **Design twin now; operational twin later.** The Vessel Model and its analyses stay in step with the
  design: change an input and every layer, result and view follows. A twin that takes live data from a
  vessel in service is a future direction only. Nothing in the format or the core is built for it now.
- **Scope (ADR 0011 holds).** OVCD owns the model and the concept-level analyses: hydrostatics, intact
  and damage stability, rule scantlings and longitudinal strength. Detailed finite elements, CFD,
  seakeeping and ship-handling simulation stay with companion projects. OVCD feeds them through open
  exports.
- **Exports.** glTF for viewing. A watertight panel mesh (STL or OBJ) for Capytaine and other solvers.
  The offsets table as CSV. STEP and IFC come later, through an adapter.
- **No editing in 3D.** Inputs stay in the inputs panel. The viewer only shows and queries.

## Why

- One model means the drawn hull, the hydrostatic hull and the compartments cannot disagree.
- Parameters as the stored data keep `.vdesign` small, readable and diffable, and every number has an
  input behind it. A stored mesh would be large, opaque and could drift from its inputs.
- Meshes from the core give the same geometry on desktop, web, CLI and Python, and the parity check
  covers them. three.js is a widely used WebGL library, it is MIT-licensed, and it runs in both the
  Tauri web view and the browser.
- A separate mesh hash keeps the results hash about results, so a change in tessellation alone is
  visible but does not read as an engineering change.
- Open exports let established solvers do the heavy physics, as the roadmap says, without OVCD
  writing new solvers.

## Consequences

- ADR 0012 is accepted. Spec 0008 builds the sections table and moves the hydrostatics and GZ to it in
  phase 1. Results move once, with one design-impact table.
- The `block_coefficient` input leaves the hydrostatics: the block coefficient becomes a result of the
  hull shape.
- `compartments.rs` gains hull clipping, not a new type. `SpaceKind` grows by tickets.
- The app gains a 3D view and a three.js dependency. The WebAssembly API gains mesh calls.
- `tools/web_parity.mjs` compares mesh hashes beside the results hashes.
- A new geometry builder lane owns `hull.rs`, `hydrostatics.rs` and `stern_form.rs` (spec 0008, "File
  ownership").
- Later specs build the structure layer, rule scantlings, longitudinal strength and the companion
  adapters (spec 0008, "Later specs").
