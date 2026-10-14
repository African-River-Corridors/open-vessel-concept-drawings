# Spec 0003: The drawing crate, and the hull views

Status: **approved by the maintainer, 2026-10-14.**

## Problem Statement

The app's main panel has three empty slots: side elevation, plan and midship section. ADR 0009 says
the app draws them with a Rust implementation of the Drawing Format Standard, so that the app needs
no Python and its drawings match the ones agents make with the Python tool.

The standard now exists, in the public `technical_drawings_for_agents` repository (its spec 0001).
It defines a drawing document (JSON), its normalised content, and a conformance suite of 194 cases.
The Python tool passes every case. There is no Rust implementation yet, and nothing turns a Design
into a drawing document.

## Solution

1. A new crate, **`ovcd-draw`**, implements the Drawing Format Standard in Rust: drawing document →
   normalised content → SVG, PDF and DXF. It passes the whole conformance suite.
2. A **GA view builder** turns an Option's results into drawing documents for the side elevation, the
   plan and the midship section.
3. The **app** shows those views in its main panel (ticket #79 of spec 0002).

The calculation core is not touched, and no result changes.

## User Stories

1. As the maintainer, I want the side elevation, plan and midship section of the active Option in the app, so that I can see the hull I am designing.
2. As the maintainer, I want the views to update live as I change an input, so that I can see the effect of a change.
3. As the maintainer, I want the waterline at the design draught, the freeboard and the main dimensions marked on the views, so that the drawing matches the outputs panel.
4. As the maintainer, I want the hold or the tanks drawn as the model has them (box hold, sloped hopper, or type C tanks with their arrangement), so that the views show the real Cargo Containment.
5. As the maintainer, I want each view drawn at a standard scale that fits its frame, so that it reads as a drawing.
6. As the maintainer, I want the app's drawings to follow the same standard as the agents' drawings, so that both look the same.
7. As an agent, I want `ovcd draw <file.vdesign>` to write the views as SVG, PDF and DXF, so that I can make GA views without the app.
8. As an agent, I want `ovcd draw --document` to print the drawing documents themselves, so that I can build them with the Python tool too.
9. As a developer, I want the Rust implementation checked against the standard's own conformance suite in CI, so that it cannot drift from the Python implementation.
10. As a developer, I want the suite copied in at a pinned version of the standard, with a check that the copy is unchanged, so that a suite update is a deliberate change.
11. As a developer, I want the drawing crate to have no file, clock or network access in its core, so that it can run in the web version later.
12. As a CAD user, I want the DXF to open with the full sheet in paper space and the views at scale, so that a drafter can take it further.

## Implementation Decisions

### The drawing crate `ovcd-draw`
- A workspace member and a default member. It depends on `serde_json` and pure-Rust libraries only;
  it does not depend on `ovcd-core` or `ovcd-format`.
- Its core takes strings and values in and out: a drawing document (plus the data files the standard
  names: line styles, font width table, hatch patterns, symbol libraries, title block templates) in;
  normalised content or the list of errors out. Then SVG, PDF and DXF writers that take normalised
  content. File reading lives only in the CLI and the app.
- It follows the standard's clauses exactly: rounding, order, origins, the house scale list, the
  line group by sheet size, hatch marks, text width from the font width table, dimensions computed
  from geometry, symbols, title block templates and the revision register.
- PDF embeds the standard's font (Barlow Semi Condensed, OFL 1.1), shipped with its licence.
- DXF has model space in metres and a paper-space layout with the sheet and viewports at scale.
- Libraries: prefer maintained pure-Rust crates for PDF writing and font handling; write DXF directly
  if no maintained crate gives paper-space layouts and viewports. Record the choice and why.
- Every check the standard calls an error is an error here, with the same clause number.

### The conformance suite in this repository
- A copy of the standard's `docs/standard/` folder (standard text, schema, data, conformance suite)
  at a pinned commit of the standard's repository, with the commit recorded and a hash of the copy.
- A test runs every case of `suite.json` and every unit check through the crate, as the standard's
  runner contract says. A second test fails if the copy differs from its recorded hash.
- Updating the copy is a separate, deliberate change.

### The GA view builder
- A module in `ovcd-format` (it needs results and the Design session; it needs nothing from the
  drawing crate except the document format, which is JSON). For one Option it returns three drawing
  documents: side elevation, plan, midship section. Each document is one A3 landscape sheet with one
  viewport, scale chosen by the standard's fit search (preferred scales plus the house list), no
  title block, status `concept`.
- What each view shows (all from results and the GA geometry the core already gives; no new
  calculation):
  - **Side elevation:** hull outline with bow and stern rake, deck line, waterline at the design
    draught, light waterline, the hold or tanks in dashed hidden lines, dimensions LOA and depth,
    labels for draught and freeboard.
  - **Plan:** hull outline, hold opening or tank outlines with their arrangement, centre line,
    dimensions LOA and beam.
  - **Midship section:** section at amidships with the hold shape (box, sloped hopper) or the tank
    circle(s) with side and bottom clearance, waterline, dimensions beam, depth, draught, freeboard.
- Layers from the standard's layer table; hatches only where the standard has a pattern for the
  material (none needed for a first build).
- A convoy or an Option that cannot be computed gives no views, with the same message as the outputs
  panel.

### The CLI
- `ovcd draw <file.vdesign> --out <dir> [--option <name>]` writes, per Option and view, the drawing
  document, the normalised content, SVG, PDF and DXF.
- `ovcd draw --document` prints the drawing documents only.

### The app
- The main panel's three slots show the SVG of each view, made live from the Design session's
  evaluation. The front end only places and zooms the SVG.

## Testing Decisions

- **The seam is the conformance suite:** the crate passes every case and unit check. That is the main
  test, and it is the standard's own definition of a correct implementation.
- **Cross-implementation check:** for each fixture Design, the drawing documents from `ovcd draw
  --document` are run through the Python tool's `normalise` and through `ovcd-draw`; the normalised
  content must be equal. This runs in CI where Python is available; the plain Rust tests need no
  Python.
- **GA view builder tests:** for each fixture Design, every drawing document validates against the
  standard's schema; the dimension values on the drawing equal the results (LOA, beam, depth,
  draught, freeboard); dry bulk shows its own containment, gas shows its tank arrangement.
- **Output tests:** SVG, PDF and DXF are made from normalised content; the PDF embeds the font; the
  DXF read back with a second reader (not the writer) gives the same entities.
- **App:** one smoke test (spec 0002's suite) sees the three views drawn.
- **Prior art:** the parity and design-impact checks of spec 0001; the Python tool's conformance
  command.

## Out of Scope

- Any change to the standard itself. A problem found in the standard is reported in its repository.
- A GA sheet with a title block, several views on one sheet, or PDF export from the app (later).
- Hatched materials, tug drawings, deck equipment, and construction detail.
- The standard's pending ISO and CAD checks.

## Further Notes

- **Tickets, in order:** (1) crate skeleton, vendored suite with hash check, sheet, viewports and core
  entities, the runner; (2) text and font, hatch marks, dimensions and leaders, symbols, title block
  and register: all 194 cases pass; (3) SVG and PDF writers; (4) DXF writer with paper space;
  (5) GA view builder and `ovcd draw`; (6) ticket #79: the views in the app.
- Tickets 3 and 4 can run in either order after 2. Ticket 5 needs 1 and 2; ticket 6 needs 3 and 5.
