# Spec 0010: Rename to Open Vessel Designer (OVD)

Status: **draft. The name and the choices below are agreed with the maintainer, 2026-10-30.** The name
decision sits in the maintainer's own records; this spec holds what the builder needs. An ADR comes with the
rename PR. This spec changes no code.

## Problem statement

The tool has three names:

- the product is "Open Vessel Concept Design" (OVCD), and five places still say "Open Vessel Concept
  Drawings";
- the repository is `open-vessel-concept-drawings`;
- the Python package is `vessel-designer` (import `vessel_designer`).

People cannot tell which is which. The sister projects use one pattern: `open-model-designer` (OMD),
`open-report-designer` (ORD), `open-waterway-designer` (OWD). This project does not fit it.

## Solution

Rename the product to **Open Vessel Designer (OVD)**, and make every code name match it.

- **No compatibility period.** Old names stop working on the rename commit: no `ovcd` command alias, no old
  environment variables, no move of old app settings. Nothing is published on PyPI or crates.io yet, and
  0.3.0 is the first release for outside reviewers, so this is the cheapest moment for a clean break.
- **Old design files do not open.** The loader accepts only the new Rule Set id `ovd`. A file saved by 0.2.x
  fails with a clear error that names the rename. Every `.vdesign` file in this repository is rewritten in the
  rename PR.
- **The file extension stays `.vdesign`.** It names the format, not the product. The file header stays
  `"format": "vdesign"`.
- **No numbers change.** The rename PR changes names only.

## Identifiers, old → new

### Names people see
| Thing | Old | New |
|---|---|---|
| Product name | Open Vessel Concept Design (OVCD) | Open Vessel Designer (OVD) |
| App name (`productName`, window title) | `OVCD` | `OVD` |
| File-type label | "OVCD Design" | "OVD Design" |
| Docs site name | "OVCD Docs" | "OVD Docs" |
| Lockup letters | OVCD | OVD (the body-plan mark does not change) |

### Repository and web
| Thing | Old | New |
|---|---|---|
| GitHub repository | `open-vessel-concept-drawings` | `open-vessel-designer` (renamed in place; GitHub redirects git and web URLs) |
| `repository` in `Cargo.toml`, URLs in `pyproject.toml` | old URL | new URL |
| Schema `$id` in `rules/rule-set.schema.json` | old URL | new URL |
| `mkdocs.yml` `site_url`, `repo_url`, `repo_name` | `/ovcd/docs/`, old repository | `/ovd/docs/`, new repository |
| Website path | `africanrivercorridors.com/ovcd` | `africanrivercorridors.com/ovd`. The old path is not redirected. |
| GitHub Pages | `…github.io/open-vessel-concept-drawings/` | `…github.io/open-vessel-designer/` |

### Code
| Thing | Old | New |
|---|---|---|
| Crates and folders | `ovcd-core`, `ovcd-format`, `ovcd-cli`, `ovcd-py`, `ovcd-shell`, `ovcd-web`, `ovcd-app` | `ovd-core`, `ovd-format`, `ovd-cli`, `ovd-py`, `ovd-shell`, `ovd-web`, `ovd-app` |
| `arc-rules` | `arc-rules` | unchanged (a neutral name; other projects use it) |
| OTD alias in `Cargo.toml` | `ovcd-draw` | `ovd-draw`, or use `otd` directly (say which in the PR) |
| Command | `ovcd`, `ovcd.exe`, `ovcd.wasm` | `ovd`, `ovd.exe`, `ovd.wasm` |
| Python distribution | `vessel-designer` | `open-vessel-designer` |
| Python import | `vessel_designer`, `vessel_designer._native` | `ovd`, `ovd._native` |
| Front-end package | `ovcd-app` | `ovd-app` |
| Environment variables | `OVCD_*` (`OVCD_PYTHON`, `OVCD_CLI`, `OVCD_CALIBRATION_DIR`, `OVCD_WRITE_TEXT`, `OVCD_WEB__*`, `OVCD_TEST_*`, `OVCD_BIN`, `OVCD_APP_BINARY`, `OVCD_BASE`, `OVCD_LANG`, `OVCD_WEB_SHOTS` and the rest) | `OVD_*`, same suffixes. Old names are not read. |
| Browser storage keys | `ovcd.language`, `ovcd.lastDesign`, `ovcd.noticeSeen`, `ovcd.panels`, `ovcd.theme` | `ovd.*`. Old keys are not read. |

### App and data
| Thing | Old | New |
|---|---|---|
| App identifier | `org.africanrivercorridors.ovcd` | `org.africanrivercorridors.ovd` (a new settings folder; old settings are not moved) |
| File-type identifier | `org.africanrivercorridors.ovcd.vdesign` | `org.africanrivercorridors.ovd.vdesign` |
| File extension and header | `.vdesign`, `"format": "vdesign"` | unchanged |
| Project Rule Set | id `ovcd` (1.0.0, 2.0.0, 2.1.0), folder `rules/ovcd/` | id `ovd` at **3.0.0**, folder `rules/ovd/`; `RELEASED.json` and the built-in Defaults Profile updated |
| Load check | requires a Rule Set entry `ovcd` | requires `ovd`. A file with `ovcd` fails with an error that says the file is from OVCD 0.2.x and does not open in OVD. |
| `.vdesign` files in this repository | 21 files (examples and fixtures) | rewritten, with new hashes |

### CI, release and docs
| Thing | Old | New |
|---|---|---|
| Workflows | `ovcd` in `app-build`, `app-checks`, `ci`, `lean-proofs`, `release` (artefacts `ovcd-<os>`), `web-parity`, `website` | `ovd` throughout |
| PyPI trusted publishing | not set up (never uploaded) | the maintainer adds a pending publisher for `open-vessel-designer` after the repository rename |
| `.release-guard-allow` | old paths | new paths |
| Fixture and design-impact hashes | old ids | regenerated |
| README, AGENTS.md, CLAUDE.md, GLOSSARY.md, docs, specs, ROADMAP, NOTICE, TRADEMARKS.md, CONTRIBUTING, website | OVCD | OVD. ADRs, `changes/` and earlier CHANGELOG entries keep their history. |
| Tags and releases | `v0.2.1` to `v0.2.4` | unchanged. The first OVD release is 0.3.0. |

## Order

1. **Clear the queue.** The open PRs land or close first. The 0.3.0 preparation PR stays a draft and is
   rebased after step 4.
2. **Freeze.** An issue says that no other work merges until the rename PR merges.
3. **This spec** is approved.
4. **The rename PR**, by one builder, in this order inside the PR. Each step leaves the build green.
   1. Crate folders and names, `Cargo.toml`. `cargo build` and `cargo test` pass.
   2. The Python package `ovd` and `pyproject.toml`. `pytest` passes.
   3. The Rule Set `ovd` 3.0.0, the load check, `RELEASED.json`, the profile. Rewrite the 21 `.vdesign`
      files and regenerate fixture hashes.
   4. The app: `tauri.conf.json`, `package.json`, storage keys, environment variables. App checks pass.
   5. CI workflows, artefact names, the release-guard allowlist.
   6. Docs, README, GLOSSARY, AGENTS.md, the website, URLs. One new ADR records the rename.
   7. A grep gate in CI: no `ovcd` or `OVCD` outside the CHANGELOG history, `changes/`, ADR history and the
      error message that names the rename.
5. **The maintainer renames the repository** and adds the PyPI pending publisher.
6. **Links in the sister repositories and the website proxy** move to the new name and `/ovd`, in the same
   sitting as step 5, so the site is down for as short a time as possible.
7. **0.3.0.** The CHANGELOG entry reads:

   > OVCD is now Open Vessel Designer (OVD). The repository, crates, the `ovd` command, the Python package
   > (`open-vessel-designer`, `import ovd`) and the app use the new name. The old names no longer work.
   > Design files keep the `.vdesign` extension, but files saved by 0.2.x do not open in 0.3.0.

## Acceptance

- `cargo test`, `pytest`, the app checks and the web parity check pass.
- **No result changes.** For every example and fixture, the results before and after the rename are equal,
  ignoring only the Rule Set id and hash fields. The PR shows this check.
- A `.vdesign` file from 0.2.4 fails to load with the error above, and a test proves it.
- The grep gate passes.
- `tools/release_guard.py` passes on the tree and on history.

## Risks

| Risk | Guard |
|---|---|
| Users with 0.2.x design files cannot open them in 0.3.0 | The error names the cause; the CHANGELOG says so plainly. |
| Every hash changes at once, so a real numeric change could hide | The before/after results check in Acceptance. |
| The old website path stops working, with no redirect | All links this project owns move in step 6. |
| GitHub Pages does not follow a repository rename | Steps 5 and 6 happen together. |
| Projects that depend on this repository by git URL or tag | GitHub redirects the old URL; old tags are never deleted. Each project moves to OVD on its own schedule. |
| Another party takes `ovd` or `open-vessel-designer` on PyPI or crates.io | Both are free on 2026-10-30. Upload 0.3.0 soon after the rename. |
| Merge conflicts across 500+ files | The freeze, and one builder. |

## Out of scope

- Any change to results, rules or the file format beyond the Rule Set id.
- The brand restyle. Only the lockup letters and icon file names change.
- Publishing a release. The maintainer does that.
