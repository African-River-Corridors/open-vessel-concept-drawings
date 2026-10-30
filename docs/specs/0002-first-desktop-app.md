# Spec 0002: First desktop app

Status: **approved by the maintainer, 2026-10-14.**

## Problem Statement

The Rust core is complete (spec 0001), but the only way to use it is the `ovd` command-line tool.
To test a design, the maintainer writes JSON by hand in a text editor, runs a command, and reads
text output. There is no picture of the hull, no chart of speed and power, and no easy way to see
how a change to one input moves the results. Comparing two Options means reading two blocks of
text.

The maintainer wants to test the engine the way a barge designer will use it: change an input and
see the hull, the curves and the results change at once. This first build is for the maintainer.
It is not a release.

The General Arrangement (GA) views cannot be drawn yet. ADR 0009 says GA drawings in the app wait
for the Drawing Format Standard, and the standard does not exist. The app must not invent its own
drawing style in the meantime.

## Solution

A Tauri desktop app (ADR 0006) with three panels:

- **Left panel: inputs and settings.** A picker for the active Option. Every input of the Option,
  grouped, with its source (typed or default), its unit, and its hard and soft ranges. A Waterway
  Limits group. The Defaults Profile in use, with a way to load another one.
- **Main panel: views.** All views stacked in one scrolling panel. A dropdown at the top right
  jumps to a view. The views are: side elevation, plan, midship section, speed-power curves, and
  stability (gas Cargo only). Each view can zoom and pan.
- **Bottom panel: outputs.** The main results of the active Option, its checks and warnings, and a
  compare table with every Option of the Design side by side.

Results update live: every change to an input re-evaluates the Design. A run takes milliseconds.

The app holds no calculation and no validation logic. It gets everything, including the list of
inputs and their ranges, from one Rust interface on top of `ovd-format`. That interface gives the
same results as `ovd run --json` for the same file.

**Sequence.** The Drawing Format Standard is written first, as its own spec in the drawing tool's
repo, and covers a full sheet. This spec is built in parallel with it. The app shell, inputs,
outputs, compare table, speed-power curves and stability come first. The three hull views appear
as empty, labelled slots until the Rust implementation of the standard lands. Drawing them is the
last ticket of this spec, and the standard blocks it.

## User Stories

### Files
1. As the maintainer, I want to start a new Design from the app, so that I can test a design without writing JSON.
2. As the maintainer, I want the new Design to have one Option with the same starter inputs as `ovd new`, so that the app and the command-line tool agree.
3. As the maintainer, I want to open an existing `.vdesign` file, so that I can test designs I made with the command-line tool.
4. As the maintainer, I want to open a hand-written file that leaves out inputs, and have them filled in from the Defaults Profile and marked "default", as `ovd fill` does, so that I can open any valid file.
5. As the maintainer, I want to save the Design to a `.vdesign` file, so that I can keep it and run it with the command-line tool.
6. As the maintainer, I want a saved file to pass `ovd validate` and give the same results with `ovd run`, so that I can trust the file.
7. As the maintainer, I want each Option's results hash saved with the file, so that a later engine change shows up when I open it.
8. As the maintainer, I want a warning when I open a file whose results changed since it was saved, so that I know the engine moved.
9. As the maintainer, I want a warning before I close or open another file with unsaved changes, so that I don't lose work.
10. As the maintainer, I want the window title to show the file name, the active Option and whether there are unsaved changes, so that I know what I am looking at.
11. As the maintainer, I want to see the engine version in the app, so that I can match results to a version.

### Options
12. As the maintainer, I want to pick the active Option from a list, so that the panels show one Option at a time.
13. As the maintainer, I want to add an Option, so that I can try a variant.
14. As the maintainer, I want to duplicate the active Option, so that I can change one input and compare.
15. As the maintainer, I want to rename an Option, so that the names mean something.
16. As the maintainer, I want to delete an Option, with a confirm step, so that I can tidy a Design.
17. As the maintainer, I want the app to refuse to delete the last Option, so that a Design always has one.

### Inputs (left panel)
18. As the maintainer, I want the six required inputs (Cargo, Hull Form, Formation, length overall, beam, moulded depth) at the top, so that the main choices are first.
19. As the maintainer, I want choice inputs (Cargo, Hull Form, Cargo Containment, ruleset) as dropdowns that list only valid values, so that I cannot type a wrong key.
20. As the maintainer, I want Cargo shown by its display name, with the key stored in the file, so that the list is readable.
21. As the maintainer, I want to set the Formation kind, and its abreast and in-line counts for a convoy, so that I can describe the Formation.
22. As the maintainer, I want number inputs with their unit shown next to the field, so that I know what to type.
23. As the maintainer, I want only the inputs that apply to the active Cargo Containment, so that I don't see tank inputs on a box hold.
24. As the maintainer, I want each input to show whether it is typed or default, so that I know what I chose and what the profile chose.
25. As the maintainer, I want an input that I change to become "typed", so that the file records my choice.
26. As the maintainer, I want to reset an input to its default, so that I can undo a choice.
27. As the maintainer, I want to see an input's default value, hard range, soft range and basis on hover, so that I know where the numbers come from.
28. As the maintainer, I want a value outside the hard range refused, with the reason shown under the field, so that I cannot run an impossible design.
29. As the maintainer, I want a value outside the soft range accepted with a warning under the field, so that I can explore but know it is unusual.
30. As the maintainer, I want cross-checks between inputs (for example freeboard against depth) shown under the inputs they involve, so that I can see which inputs conflict.
31. As the maintainer, I want nullable inputs (such as a draught limit) to have a clear "none" state, so that I can switch a limit off.
32. As the maintainer, I want the inputs in collapsible groups, so that the panel stays short.

### Waterway Limits and settings (left panel)
33. As the maintainer, I want the channel depth, bottom width, under-keel clearance, bank slope and current in one Waterway Limits group, so that the waterway is in one place.
34. As the maintainer, I want the draught limit and the air-draft limit in the same group, so that all limits are together.
35. As the maintainer, I want to see the name of the Defaults Profile in use, so that I know which defaults apply.
36. As the maintainer, I want to load an edited Defaults Profile from a file, so that I can test other defaults and ranges.
37. As the maintainer, I want the app to refuse a profile that loosens a hard range, with the reason shown, so that the profile rules hold.
38. As the maintainer, I want to go back to the built-in profile, so that I can undo a profile change.

### Live results
39. As the maintainer, I want outputs and views to update as I change an input, without pressing a button, so that I see cause and effect at once.
40. As the maintainer, I want the app to wait for a short pause in my typing before it evaluates, so that it does not run on half-typed numbers.
41. As the maintainer, I want the last good results kept on screen and marked "out of date" while an input has a hard error, so that the panels don't go blank.
42. As the maintainer, I want an Option that cannot be computed to show the engine's message in the outputs, so that I know why.
43. As the maintainer, I want a pushed convoy to show "convoy results not yet available" and no numbers, as the command-line tool does, so that I am not shown wrong results.

### Views (main panel)
44. As the maintainer, I want all views in one scrolling panel, so that I can read them in order.
45. As the maintainer, I want a dropdown at the top right of the main panel that jumps to a view, so that I can go straight to it.
46. As the maintainer, I want the dropdown to show which view I am looking at as I scroll, so that I know where I am.
47. As the maintainer, I want to zoom and pan each view, so that I can look at detail.
48. As the maintainer, I want to reset a view's zoom, so that I can get back to the whole view.
49. As the maintainer, I want the side elevation, plan and midship section slots to say they wait for the Drawing Format Standard, so that I know why they are empty.
50. As the maintainer, I want the side elevation, plan and midship section drawn to the Drawing Format Standard once it exists, with the waterline, draught and freeboard marked, so that the hull matches the results.
51. As the maintainer, I want a speed-power chart for each channel environment, with power and fuel against speed, so that I can see the cost of speed.
52. As the maintainer, I want the limiting speed marked on each speed-power chart, so that I can see the channel limit.
53. As the maintainer, I want squat-limited points marked on the speed-power chart, so that I can see where squat governs.
54. As the maintainer, I want to read exact values when I hover on a chart, so that I can take numbers off it.
55. As the maintainer, I want a stability view for a gas Cargo, with the criteria set, the governing condition and a table of conditions, so that I can see why it passes or fails.
56. As the maintainer, I want the stability view to say "not computed for this Cargo" for dry bulk, so that its absence is explained.

### Outputs (bottom panel)
57. As the maintainer, I want the cargo tonnes and what governs them (hold volume or a draught limit), so that I know what limits the design.
58. As the maintainer, I want lightship, displacement and deadweight, so that I can check the weights.
59. As the maintainer, I want draught, maximum draught, light draught and freeboard, so that I can check the vessel against the waterway.
60. As the maintainer, I want the loaded and light air draft, so that I can check bridges.
61. As the maintainer, I want cargo volume and the depth to draught ratio (h/T), so that I can check the hull.
62. As the maintainer, I want the tank arrangement for a gas Cargo (tanks across and along, outer diameter, side clearance, ruleset, fill ratio, whether the depth was optimised), so that I can check the tanks.
63. As the maintainer, I want every check with pass or fail and its message, failures first, so that I see problems first.
64. As the maintainer, I want every warning listed, so that I don't miss one.
65. As the maintainer, I want a compare table with every Option of the Design side by side, so that I can compare variants.
66. As the maintainer, I want the compare table to mark where an Option differs from the active Option, so that differences stand out.
67. As the maintainer, I want to click an Option's column in the compare table to make it active, so that I can move between Options quickly.
68. As the maintainer, I want to drag the border between the main and bottom panels, and between the left and main panels, so that I can give space where I need it.
69. As the maintainer, I want numbers shown with sensible rounding and full precision on hover, so that the panel is readable but exact values are there.

### Agents and the command-line tool
70. As an agent, I want the same machine-readable input schema the app uses from a command (`ovd schema`), so that I can build valid Designs without reading Rust.
71. As a developer, I want all app logic behind one Rust interface that needs no Tauri and no screen, so that it is tested like the command-line tool.

## Implementation Decisions

### Modules
- **Design session (new module in `ovd-format`).** The one seam between the app and the engine.
  It has no Tauri dependency and no file, clock or screen access. It takes and returns strings and
  values; the app shell does all file reading and writing. Its interface:
  - *input schema*: for a Defaults Profile, every input with its kind, unit, valid choices, display
    names, whether it is required, which Cargo Containments it applies to, its group, its default,
    hard and soft ranges, and basis. Built from the existing input table and the profile, so there
    is one source.
  - *new design*: the same starter Design as `ovd new`.
  - *open*: parse a `.vdesign` text, fill left-out inputs as `ovd fill` does, and report results-hash
    status per Option.
  - *edit*: set an input (marks it typed), reset an input to its default, set the Formation, and add,
    duplicate, rename and delete Options. Each edit returns the new Design state.
  - *evaluate*: validate and run every Option. Per Option it returns per-input hard errors and soft
    warnings, cross-check messages, and either the results (the same JSON object `ovd run --json`
    gives) or the engine's message.
  - *save*: the `.vdesign` text with each computed Option's results hash.
  - *load profile*: parse a Defaults Profile, refusing one that loosens a hard range.
- **`ovd schema` command (CLI).** Prints the input schema as JSON. Same function as the app uses.
- **App shell (new Tauri crate in the workspace).** Thin. Tauri commands that forward to the Design
  session, plus native open and save dialogs, the unsaved-changes guard and the window title. No
  calculation, validation or default logic.
- **App front end (HTML and TypeScript).** Renders the input schema as a form, the evaluation as
  outputs, and the views. Holds only display state: active Option, panel sizes, scroll position,
  zoom. Built with Svelte and TypeScript on Vite: a small, compiled, reactive framework suits a
  form that re-renders on every evaluation. Charts use uPlot: small and fast for line charts.
- **Hull views (last ticket, blocked by the Drawing Format Standard).** Drawn by the Rust drawing
  crate that ADR 0009 calls for, from the core's GA geometry, returned to the front end as SVG.
  The front end only places and zooms the SVG.

### Behaviour
- **Live evaluation.** The front end sends each edit to the Design session after a pause of about
  150 ms in typing, then evaluates the whole Design. The whole Design is evaluated each time, so the
  compare table is always current.
- **Hard error.** The edit is kept in the Design, so the field shows what was typed. The Option's
  results are not recomputed; the last good results stay on screen marked "out of date".
- **Inputs that do not apply** to the active Cargo Containment are hidden, not deleted. The file
  keeps them as `ovd fill` would.
- **Groups.** Required inputs; Hull and containment; Gas tanks (type C tanks only); Waterway Limits
  (channel depth, bottom width, under-keel clearance, bank slope, current, draught limit, air-draft
  limit); Speed and power. The group of each input comes from the input schema, not from the front
  end.
- **Results hash.** Recomputed on save. On open, a changed hash shows a warning banner with the
  Options affected (ADR 0007).
- **Units.** SI only, as stored (speeds are in knots, as stored; ADR 0007). One display rounding rule per quantity type, in the front end.
- **Platform.** A macOS development build for the maintainer, unsigned. The Windows build compiles
  in CI but has no installer.

## Testing Decisions

- **A good test** checks behaviour a user or an agent can see: a Design in, results and messages
  out. It does not check how the session stores its state.
- **The Design session** is tested in Rust, like the command-line tool's tests:
  - For every example Design, *evaluate* gives exactly the results `ovd run --json` gives.
  - *new design* equals `ovd new`; *open* of a hand-written file equals `ovd fill`.
  - A saved Design passes `ovd validate` and round-trips through *open* unchanged.
  - Hard-range, soft-range and cross-check cases give the same messages as `ovd validate`.
  - Each edit operation, including deleting the last Option (refused) and loading a profile that
    loosens a hard range (refused).
  - The input schema lists every input in the input table, with ranges equal to the profile's.
- **`ovd schema`** has one command test: its output equals the session's input schema.
- **The design-impact check** (spec 0001, ticket 28) covers results; this spec changes no results.
- **The app** gets a few end-to-end smoke tests on the built app, run in CI on Linux through
  Tauri's WebDriver support: open a file, edit an input and see an output change, enter a value
  outside the hard range and see the error, switch Option, save and re-open. No per-component tests.
- **Prior art:** the `ovd-cli` tests (`new` output passes `validate`, `fill` refuses a file with a
  hard error) and the `ovd-format` validation tests.

## Out of Scope

- The Drawing Format Standard itself. It is its own spec, in the drawing tool's repo, and covers a
  full sheet: paper space, title block, revision register, scale bar, layers, DXF and PDF.
- GA drawing logic of any kind in this spec before the standard's Rust implementation exists.
- DXF and PDF export, and the PDF report.
- US units and translations (ADR 0008).
- A tank arrangement view (the arrangement is in the outputs table).
- Known engineering findings shown in the app, and Calibration Bands.
- Convoy results (the engine does not compute them yet).
- Signed installers, auto-update, first-run notice, feedback links and the web version.
- Any change to results.

## Further Notes

- **Suggested tickets, in order:** (1) Design session and input schema, with `ovd schema`;
  (2) app shell with open, save and the unsaved-changes guard; (3) left panel; (4) bottom panel and
  compare table; (5) main panel with the scroll and jump dropdown, empty hull view slots, and
  speed-power curves; (6) stability view; (7) smoke tests in CI; (8) hull views, blocked by the
  Rust implementation of the Drawing Format Standard.
- Tickets 1 to 7 need nothing from the standard.
- The first user is the maintainer. Wording can be plain and technical. Polish for other users comes
  with the first public installer (roadmap item 10).
