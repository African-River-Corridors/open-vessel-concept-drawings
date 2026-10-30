# Renamed Open Vessel Designer (OVD), with no compatibility

Status: **accepted**, 2026-10-30, with spec 0010.

## Context

The tool had three names: the product "Open Vessel Concept Design" (OVCD), the repository
`open-vessel-concept-drawings`, and the Python package `vessel-designer`. The sister projects use one
pattern, `open-<thing>-designer` with three initials (OMD, ORD, OWD). Nothing was published on PyPI or
crates.io, and 0.3.0 is the first release for outside reviewers.

## Decision

- **The name is Open Vessel Designer (OVD)**, and every code name matches it: crates `ovd-*`, the command
  `ovd`, the Python distribution `open-vessel-designer` with `import ovd`, the app OVD, environment variables
  `OVD_*`, storage keys `ovd.*`, the website path `/ovd`.
- **No compatibility period.** Old names stop working on the rename commit: no command alias, no old
  environment variables, no move of old app settings, no redirect from the old website path.
- **Old design files do not open.** The project Rule Set is renamed `ovd` and starts at 3.0.0, after the
  old 2.1.0, with the same Rules. The loader accepts only `ovd`; a file that stores the old id fails with
  one error that names the rename. There is no read-time mapping. The repository's own files are
  rewritten.
- **The released list keeps no entry for the old id.** "A released version is kept for ever" holds within
  one id; the old id is retired with its versions.
- **`.vdesign` stays.** It names the format, not the product; the header stays `"format": "vdesign"`.
- **History keeps the old name:** earlier ADRs, `changes/`, earlier CHANGELOG entries and spec 0010. A gate
  (`tools/old_name_gate.py`) holds every other file to the new name.

## Consequences

- Anyone holding a 0.2.x design must make it again in OVD. The release notes say so plainly.
- The Rule Set id and hash change in every stored file, but no result and no results hash moves.
- Projects that depend on this repository by git URL or tag keep working through GitHub's redirect;
  old tags are never deleted.
