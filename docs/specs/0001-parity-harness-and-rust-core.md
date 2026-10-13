# Spec 0001: Parity harness and Rust core port

Status: **approved by the maintainer, 2026-10-13**.

## Problem Statement

Open Vessel Concept Drawings (the Python package `vessel-designer`) is a Python library. People who
design inland barges, but who do not write code, cannot use it. The plan is a free, offline,
installable desktop app with a compiled Rust core. Later, the same core runs in a web page.

Rewriting the calculation core in another language is risky. A naval architect cannot easily see a
port error in tonnage, draught or stability. If we also fix known engineering errors during the port,
we cannot tell a port error from a deliberate change. Python users must also keep a working
`pip install vessel-designer` with the same public API.

The project also needs a file that holds a design, a command-line tool, and a test and CI setup. That
setup must catch real mistakes without slowing work down. In the past, CI has slowed and blocked work
without catching much.

## Solution

Port the calculation core to Rust **with identical results first**. Fix nothing during the port.

1. **Ticket 1 is the parity harness.** The current Python code runs a fixed set of cases and writes
   Reference Results. The Rust code must match them. Every later ticket is done only when its part
   of the harness is green.
2. Port the core module by module, in dependency order, each module against its Reference Results.
3. Add a `.vdesign` file format, a defaults and validation layer, and a command-line tool `ovcd`.
4. Switch the Python package to call the Rust core. The public Python API stays the same.
5. When the port is complete, delete the Reference Results and the frozen Python copy. From then
   on, a design-impact check on every pull request shows whether vessel designs moved.

This spec changes no results. It is a port only. The known engineering findings are still under
review, and nobody has agreed yet that they need fixing. General improvements come after the port.

## User Stories

### Barge designer (no coding knowledge)
1. As a barge designer, I want to give only the Cargo, Hull Form, length, beam, Formation and depth,
   so that I can get a first design without knowing every engineering input.
2. As a barge designer, I want every other input to take a stated default, so that I can see what the
   tool assumed.
3. As a barge designer, I want to see which inputs came from a default and which I typed, so that I
   know which numbers to check.
4. As a barge designer, I want a clear message when an input is out of range, so that I do not get a
   confident number from a wrong input.
5. As a barge designer, I want a warning, not a refusal, when my input is outside the standard vessel
   classes but still physically possible, so that I can explore unusual designs.
6. As a barge designer, I want to save several Options in one Design file, so that I can compare them
   later.
7. As a barge designer, I want a Design file to give the same results when I open it again, so that I
   can trust what I saved.
8. As a barge designer, I want to be told when a newer engine version gives different results for my
   saved file, so that I know the numbers changed and why.
9. As a barge designer, I want a Design file to be readable on Windows and macOS with identical
   results, so that I can share it with a colleague on another computer.
10. As a barge designer, I want a clear "not yet supported" message when I describe a pushed convoy,
    so that I know the tool will not silently give wrong convoy results.
11. As a barge designer, I want a clear message that depth is required for now, so that I know a
    default rule is coming but is not there yet.

### Engineer using the command line
12. As an engineer, I want `ovcd new` to write a starter Design, so that I do not have to write the
    file by hand.
13. As an engineer, I want `ovcd validate` to check a Design file without running it, so that I can
    find input errors fast.
14. As an engineer, I want `ovcd run` to print a short summary, so that I can read the main results at
    a glance.
15. As an engineer, I want `ovcd run --json` to print the full results, so that I can feed them into my
    own tools.
16. As an engineer, I want all files and output in SI units, so that there is no unit confusion. (US
    units arrive later with the app's unit setting.)

### Python library user
17. As a Python user, I want `pip install vessel-designer` to keep working, so that my scripts do not
    break.
18. As a Python user, I want the same import paths, function names and result fields as today, so that
    I do not have to change my code.
19. As a Python user, I want the same numbers as today (within a stated tolerance), so that my earlier
    studies stay valid.
20. As a Python user on Windows, macOS or Linux, I want a ready-built package for my system, so that I
    do not need a Rust compiler to install it.
21. As a Python user, I want the calibration tool to keep working on top of the new core, so that I
    can still compare the model with real barges.

### Maintainer
22. As the maintainer, I want the port to prove itself against the Python results, so that I do not
    have to read Rust code to trust it.
23. As the maintainer, I want discrete outputs (tank count, governing limit, check pass or fail, check
    message text) to match exactly, so that the same design decisions come out.
24. As the maintainer, I want every float to match within a fixed tolerance that nobody may loosen to
    make a case pass, so that the parity bar means something.
25. As the maintainer, I want the parity cases to include the named cases from the current tests, a
    seeded sweep across the input space, and the error cases, so that the port is tested broadly.
26. As the maintainer, I want the Reference Results deleted when the port is complete, so that stale
    numbers never block deliberate logic changes later.
27. As the maintainer, I want every pull request to say whether vessel designs moved, so that I can
    tell logic changes from presentation changes.
28. As the maintainer, I want design changes tracked in one place with before/after tables, so that I
    can diagnose later when and why results changed.
29. As the maintainer, I want useless tests deleted, so that CI only checks things that matter.
30. As the maintainer, I want each kept fixed-number test labelled "sourced" or "snapshot", so that I
    know which numbers come from real engineering and which are only what the code printed.
31. As the maintainer, I want fast checks (under 5 minutes) on every pull request, and slow checks
    only on a release, so that CI does not slow down development.
32. As the maintainer, I want a check that blocks pull requests without catching real problems to be
    removed, so that CI stays useful.
33. As the maintainer, I want no publishing to PyPI in this spec, and a release workflow that cannot
    publish unless the tests and the release guard pass, so that nothing broken goes out.
34. As the maintainer, I want the domain glossary and the architecture decisions written down in the
    repo, so that agents and contributors use the same words and do not reopen settled choices.
35. As the maintainer, I want the copyright holder named correctly, so that the licence notice is
    legally right.
36. As the maintainer, I want the roadmap to show the post-port plan, so that contributors can see
    what comes next.

### Agent implementer and reviewer
37. As an implementer agent, I want each ticket to have its own green bar (its module's Reference
    Results), so that I can work in a separate worktree in parallel with other tickets.
38. As an implementer agent, I want `cargo test` to run without Python installed, so that the Rust
    loop is fast.
39. As an implementer agent, I want a lint rule that blocks platform-dependent maths functions in the
    core, so that I cannot add a cross-platform difference by accident.
40. As a cold reviewer, I want the Rust modules to mirror the Python files one to one, so that I can
    review the port file by file.

### Future app and web page
41. As the future desktop app, I want one entry point that reads a Design and returns results, so
    that the app does not duplicate any logic.
42. As the future web page, I want the core to build for WebAssembly with no file or system access,
    so that the same engine runs in a browser.

## Implementation Decisions

### Workspace and crates
- One Cargo workspace at the repo root, edition 2024, with a pinned toolchain. Workspace crates:
  - **`ovcd-core`**: pure calculation. `no_std` plus `alloc`. Its only dependencies are `libm` and an
    optional `serde`. No file, clock, environment or randomness access. It must build for
    `wasm32-unknown-unknown`, and CI checks this. Its modules mirror the Python core files one to
    one. Restructuring waits until after the port. A `hull_form` module holds the one Hull Form that
    exists today, the raked box. V-bottom and others come later.
  - **`ovcd-format`**: the `.vdesign` format (read, write, schema, versions, migration), the
    Defaults Profile, validation, the results hash, the port of the one-call `simulate`
    orchestration, and an `eval(entry_point, json) -> json` dispatch over every parity entry point.
  - **`ovcd-cli`**: the `ovcd` binary. A thin layer over `ovcd-format`.
  - **`ovcd-py`**: PyO3 bindings, abi3 for Python 3.12 and later, built by maturin into the
    `vessel-designer` wheel. It is not in the workspace's default members, so `cargo build` and
    `cargo test` need no Python. The core never depends on PyO3.
  - **`xtask`**: developer tasks. It regenerates the Reference Results from the frozen Python and
    runs the gates script.
- Crates for later, not this spec: a WebAssembly wrapper, a report crate (PDF, translations), a
  drawing crate, and a CFD export crate. Units are display-only and belong in the app UI, so there is
  no units crate.

### Python package
- The project file at the repo root switches to the maturin build backend. The distribution name is
  written `vessel-designer`.
- The shipped Python package is a thin layer. It keeps today's public import paths, names and
  dataclass-style result types, and it calls the Rust core.
- During the port, the current pure-Python code is moved (keeping its git history) to a frozen,
  test-only reference location under a different import name, so a test can never import the wrong
  one by accident. Only `xtask` runs it.
- The shipped package stays pure Python until every entry point is at parity. Then one switch pull
  request moves it to the Rust core and sets the version to 0.2.0.
- numpy is dropped as a dependency. The core never calls it.
- The calibration tool stays Python, on top of the Rust-backed package. It is tooling, not app logic.
- `calculator` is dropped. Nothing calls it, it has no tests, it prints to the console, and
  `barge_capacity` replaces it.

### Port scope and entry points
- Everything on the `simulate` path is ported, plus `design_grid`, `ga_adapter`, `barge_cost`,
  `vessel_standards`, `electric_energy` and all of `hydrostatics`.
- Built-in data (the cargo database, the vessel standards and the example cost rates) is compiled into
  the core. The binding layer reads a user's rates file and passes the text to the core, because the
  core does no I/O.
- **Rule: every public function of the Python package gets Reference Results.** Private helpers do
  not. The entry points are:
  - `cargo`: cargo lookup and the cargo list
  - `protective_location`: `distance_d_vc`, `d_vc_exact`, `required_side_protection`,
    `required_bottom_protection`, `compute_required_clearances`, `check_protective_location`
  - `pressure_vessel`: the public wall-thickness functions
  - `hull`: the hull geometry quantities (volume, KB, LCB, BM, areas)
  - `aft_arrangement.compute`, `gas_separation.compute`
  - `tanks`: `calculate_tank_arrangement`, `free_surface_moment_cylindrical`
  - `weights.calculate_weights`
  - `hydrostatics`: `find_equilibrium_draft`, `calculate_stability`, `calculate_trim`
  - `barge_capacity`: `compute`, `tanks_wide_for_beam`
  - `journey_derating.compute`
  - `gas_arrangement`: `fit`, `smallest_viable_loa`, `required_side_clearance`
  - `gas_stability`: `box_gz`, `segment_fill`, `assess`, `criteria_set_for`
  - `squat`: `compute`, `max_speed_for_ukc`
  - `waves.compute`, `resistance.compute`, `electric_energy.compute`
  - `barge_cost`: `compute`, `parse_flat_yaml`
  - `design_grid`: `sweep`, `n_tanks_for_length`
  - `ga_adapter`: `to_ga`, `is_gas`
  - `vessel_standards`: `get_standard`, `standards_for`, `validate_catalogue`
  - `simulate`: `simulate`, `cargo_options`

### What "identical results" means
- Floats: relative tolerance 1e-9 and absolute tolerance 1e-9, in SI units.
- Exact match for everything discrete: integers, booleans, enum and ruleset choices, the governing
  limit, check pass or fail, and all text, including check messages and warnings.
- Errors: where Python raises `ValueError` or returns `{"error": ...}`, Rust must fail in the same way,
  with the same message.
- If a case sits on a knife edge (a tiny float difference flips a branch or a near-tie), the case is
  replaced or nudged, and the reason is recorded next to it. The tolerance is never loosened.
- Known traps the port must copy from Python:
  - Python 3.12 `sum()` uses compensated summation.
  - `max` and `min` return the *first* of equal items.
  - `round()` rounds half to even.
  - Bisection loops stop early on tolerance, so the result depends on the path taken.
  - Candidate choice in `gas_arrangement.fit` uses small margins.
  - The speed grid is built by repeated addition, then rounding.
  - Floor division is done on floats.
  - Python's float-to-text formatting differs from Rust's. The core needs a small helper that
    formats numbers the way Python does, with its own tests.

### Platform-identical maths
- Every transcendental function in `ovcd-core` (`sin`, `cos`, `atan`, `exp`, `log10`, `pow`, and so
  on) uses the pure-Rust `libm` crate. Python's `**` maps to `libm::pow`.
- A clippy `disallowed-methods` rule bans the standard-library versions, plus `powi` and `mul_add`,
  in the core.
- Result: bit-identical output on Windows, macOS, Linux and WebAssembly. The results hash depends on
  this.

### Parity harness (ticket 1)
- Case inputs and Reference Results live in one top-level parity folder: case inputs in one place,
  Python outputs in another.
- Three tiers of cases:
  1. named cases taken from the current tests, the quickstart example and the calibration cases;
  2. a seeded sweep across the valid input space of each entry point, committed as data (no random
     generator in Rust);
  3. error cases.
- Keep all committed parity files under about 5 MB.
- Floats are stored as shortest round-trip text. NaN and infinity are stored as tagged strings.
- `xtask` regenerates the Reference Results by running the frozen Python. It needs no maturin build.
  The frozen Python cannot change during the port, so CI does not re-run it.
- The main parity test calls `eval` in-process for every case. A second, small test runs the `ovcd`
  binary's hidden `eval` command on a few cases, so the binary is proven too.
- Ticket 1 delivers the harness, the case set, the Reference Results for **every** entry point, and
  an empty Rust dispatch. All cases fail at first. Each later port ticket turns its own entry points
  green.

### The `.vdesign` file
- Versioned JSON. Always SI. Unit switching is only for display and input, in the app.
- A Design holds a named list of one or more Options, the engine version, and a results hash for each
  Option.
- An Option holds the typed `simulate` inputs, with glossary names (for example `cargo_containment`,
  not `vessel`), plus `hull_form` and `formation`.
- **Required inputs:** Cargo, Hull Form, length, beam, Formation and moulded depth.
  - Depth is marked "default from rule". There is no rule yet, so for now it is required, and the
    error message says a default rule is planned.
  - Formation can be one self-propelled Hull, one pushed barge, or a pushed convoy of barges abreast
    and in line. This spec computes only the first two. A pushed barge is computed without its pusher,
    as today. A convoy is valid in the file, but `ovcd run` reports "convoy results not yet
    available".
- Every other input comes from the **Defaults Profile**. A built-in profile ships with the tool, and a
  user may copy and edit it. It holds the defaults (Cb 0.85, minimum freeboard 0.3 m, ruleset
  IGC_2G_EXACT, fill ratio by ruleset, lightship factor by Cargo Containment, propulsive efficiency
  0.45, maximum speed 10 kn, no channel, and so on) and the validation ranges.
- Each range is **hard** (refuse the input) or **soft** (accept it with a warning). The first ranges
  are proposed from two sources: physical limits (no NaN, no negative lengths, Cb in (0, 1], and so
  on) as hard ranges, and the catalogue's vessel class envelopes, with headroom, as soft ranges. The
  maintainer approves the proposed table in the ticket's pull request.
- The file stores every input **as used** and marks which ones came from a default. A later change to
  the Defaults Profile never changes the results of a saved file.
- Validation lives only in `ovcd-format`. The core copies Python exactly and adds no checks.
- **Results hash:** SHA-256 of the canonical JSON of each Option's results. If a file opens on an
  engine version that gives a different hash, the tool warns that results have changed since the
  file was saved with engine version X.
- One version number covers every `ovcd-*` crate and the Python package.

### CLI (`ovcd`)
- `ovcd new`: writes a starter Design with one Option and the defaults filled in and marked.
- `ovcd validate <file>`: checks the file and lists hard errors and soft warnings.
- `ovcd run <file>`: a short summary by default. `--json` gives the full results.
- `ovcd eval <entry-point>`: hidden. JSON on stdin, JSON on stdout. For parity only. It is removed
  with the Reference Results.
- Output is SI only in this spec.

### End of port
- "Port complete" means every entry point is at parity **and** the switch pull request has merged.
- The last tickets then, in order:
  1. **Design-impact check.** On every pull request, CI runs a fixed set of case inputs (the
     calibration vessels plus a small sweep) through `ovcd` twice: once on `main` and once on the
     branch. If nothing moved, the PR body says "no design change". If anything moved, the PR gets the
     `design-change` label and a before/after table, and the same table goes into a "Design changes"
     section of the changelog. There are no stored answers, so nothing goes stale. A design change
     needs no sign-off. It is tracked so we can diagnose later.
  2. **Delete** the Reference Results, the parity case inputs, the frozen Python copy and the hidden
     `eval` command, in one pull request.

### Tests already in the repo
- Early ticket, right after the parity harness: audit every Python test.
  - Label each one **sourced** (the number comes from a standard, a published example or a hand
    calculation, with a citation), **snapshot** (the number is what the code printed) or
    **useless**.
  - Copy the inputs of a test into the tier-1 parity cases before deleting it.
  - Delete the useless tests in the same pull request, and list them in the PR body.
- The calibration test stops writing into the repo. It writes to a temporary folder.
- After the switch, the kept Python tests run against the Rust-backed package.
- The old Python "pure core" import test is replaced by the core's `no_std` build and the WebAssembly
  build check.

### CI
- **Fast gate, every pull request, Linux, target under 5 minutes.** One fail-closed gates script: a
  missing tool or a skipped step fails the job. It runs:
  - `cargo fmt --check`
  - clippy with warnings as errors (this includes the banned-maths rule)
  - `cargo test`, which includes parity during the port
  - pytest against the package
  - the WebAssembly build check
  - after the port, the design-impact check
- The required check names stay `test` and `release-guard`, so branch protection does not change.
- **Release gate, per release only, plus a manual trigger.** No nightly runs. It builds wheels for
  Linux, macOS (Intel and Apple chip) and Windows, runs pytest on each, and builds the `ovcd` binary
  for Windows and macOS.
- The release workflow cannot build or publish unless `test` and `release-guard` pass. Nothing is
  published to PyPI in this spec.
- Rule for later retros: a blocking check that has caught no real problem in 3 months, or that fails
  at random, is removed or moved to the release gate.
- What the maintainer sees: the PR body lists which checks ran, whether designs moved (and by how
  much), and whether any **sourced** test changed. The maintainer does not read test output.

### Housekeeping (first pull request)
- Rename the domain glossary file to `GLOSSARY.md`. Add the new terms: Cargo Containment, Hull Form,
  Design, Option, Defaults Profile, Calibration Band.
- Change the copyright holder in the licence notice, the README and the project file to "Mobile
  Clipboard Limited (trading as African River Corridors)" until the company's rename is registered.
- Update the repo URLs in the README, CONTRIBUTING, ROADMAP and the project file to the new name
  `open-vessel-concept-drawings`. The old URL redirects, but the docs should be right.
- Add architecture decision records 0005–0010 in `docs/adr/`:
  - 0005 Rust core with PyO3 bindings and a Rust CLI; port with identical results before fixing
  - 0006 Tauri desktop app with an HTML/TypeScript UI; the same core in WebAssembly for the web later
  - 0007 the `.vdesign` file format: SI only, Options, Defaults Profile, stored inputs as used,
    results hash
  - 0008 translations with Fluent: English, Simplified Chinese, Latin American Spanish
  - 0009 one Drawing Format Standard with two implementations: Python for agents, Rust for the app.
    It supersedes ADR 0003
  - 0010 test strategy: parity during the port, then the design-impact check, Calibration Bands,
    a fast PR gate and a release gate
  - ADR 0002 and ADR 0004 stay as they are.
- Update `ROADMAP.md` with the post-port plan (see Further Notes).
- **Remove the static web simulator** (the Pyodide page, its build tool and its generated data), and
  remove it from the README. It cannot load the compiled package after the switch. A new web
  version on the WebAssembly core is on the roadmap. The shared embargo list stays, because the
  release guard uses it.

## Testing Decisions

- **A good test checks behaviour through a public seam, never internals.** It must stop a named risk.
  Ranked risks:
  1. the port gives different numbers;
  2. a later change moves results by accident;
  3. the engineering is wrong;
  4. the app breaks on odd inputs;
  5. install or build fails on one system;
  6. confidential data leaks.
- **Seams** (in order of importance):
  1. `eval(entry_point, json)` in `ovcd-format`: the parity contract for every entry point.
  2. The `ovcd` binary: a few end-to-end cases for `eval`, `run`, `validate` and `new`.
  3. The Python public API: the audited pytest suite, run against the Rust-backed package.
  4. A `.vdesign` file in, results out: defaults, hard and soft ranges, the convoy and depth
     messages, the results hash, and stability across file versions.
  5. The design-impact check: the same inputs through `ovcd` on `main` and on the branch.
- Private Rust helpers get only the tests their implementer finds useful. Seam 1 is the contract.
- **Prior art in the repo:** the current pytest suite (fixed-number cases such as the quickstart
  tonnage, regression snapshots for `gas_arrangement`, worked figures for `aft_arrangement` and
  `resistance`), the release guard's own tests, and the calibration runner, which compares the model
  with real barges.
- A port ticket is done only when its entry points are green on seam 1, clippy is clean, and the
  WebAssembly build passes.
- Reference Results are generated on CI Linux from the frozen Python and committed. They are never
  edited by hand.

## Out of Scope

- Any change to results, including the known engineering findings. They are under review, and no fix
  is agreed yet.
- Calibration Bands and the calibration check that blocks a PR when a Reference Vessel leaves its
  band. This is the first spec after the port.
- The default moulded-depth rule by Cargo.
- Formation results: convoy totals (overall length and beam, total cargo, total cost) and convoy
  resistance and power.
- Any Hull Form other than the raked box.
- The Tauri desktop app, the PDF report, translations, US units, GA drawings, DXF, construction
  drawings and plate lists, and CFD.
- Extracting the Drawing Format Standard (a separate spec in the drawing tool's repo).
- The WebAssembly wrapper and a new web version.
- Publishing to PyPI, code signing, auto-update.
- Updating the dependency URL in the sibling channel-design repo. That happens later, in that repo.

## Further Notes

### Post-port roadmap (to go into `ROADMAP.md`)
1. Calibration Bands and the calibration check (high priority).
2. Default moulded-depth rule by Cargo (high priority; fewer inputs for non-experts).
3. Review of the known engineering findings and general improvements to the model. Nothing is agreed
   yet. Each agreed change comes one at a time, with a before/after table.
4. Formations: convoy totals, then convoy resistance and power.
5. More Hull Forms (V-bottom and others), with configurable parameters.
6. The Tauri desktop app (in parallel with the CLI), US units, translations, the PDF report.
7. The Drawing Format Standard, then GA drawings and DXF in the app.
8. Later: construction drawings and plate lists; CFD runs.
9. A web version on the WebAssembly core.
10. The first public installer, once the known engineering findings are reviewed and each is fixed
    or shown in the app as a known limitation.

### Process notes
- Each ticket runs in its own worktree. The parity files are committed, so ignored fixture files
  cannot quietly turn tests off in a worktree.
- Review: a two-axis code review, then two cold reviews by models that did not write the code. At
  most 2 fix rounds, then the maintainer decides.
- Suggested port order (leaves first): checks, cargo, protective_location, pressure_vessel, hull,
  aft_arrangement, gas_separation, models, tanks, weights, hydrostatics, barge_capacity,
  journey_derating, gas_arrangement, gas_stability, squat, waves, resistance, simulate. After those:
  electric_energy, barge_cost, design_grid, ga_adapter, vessel_standards. Modules with no shared
  dependencies can run in parallel.
