# Changelog fragments

One file per pull request: `changes/<branch>.md`. A pull request adds its own file and never edits
`CHANGELOG.md`, so two open pull requests never conflict on it (issue #560).

A fragment has up to two parts, each opened by a marker line:

```
<!-- unreleased -->
- Change (issue #123): one bullet, as in the `## Unreleased` list of CHANGELOG.md.

<!-- design-impact -->
### ticket/123-slug (2026-10-22)

Between `main abc1234` and `this branch def5678`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
```

- The `unreleased` part is hand-written: bullets that start with `- `.
- The `design-impact` part is written by `python3 tools/design_impact.py changelog` when the branch
  moves designs. Run it, commit the file, and add your bullets above the marker.
- A version-bump pull request runs `python3 tools/changelog_collect.py`. It moves every fragment into
  `CHANGELOG.md` (bullets to the top of `## Unreleased`, tables to the top of `## Design changes`) and
  deletes the fragment files. This file stays.

`python3 tools/changelog_collect.py --check` tests that every fragment is well formed.
