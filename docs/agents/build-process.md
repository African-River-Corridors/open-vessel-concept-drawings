# Build process (spec 0001)

How a spec that has been split into tickets is built, reviewed and merged. Agreed by the maintainer on
2026-10-13. Since 2026-10-14 it runs in **single-agent mode**: one builder works the tickets one at a
time, and adversarial review happens after the build, not on every ticket. Since 2026-10-20 several builder **lanes** run in parallel. Each lane builds its own tickets one at a
time, owns the files its brief names, and messages the owning lane before it touches another lane's file.

## The loop

The tickets are a task graph. Native GitHub "blocked by" links are the edges. The builder takes the next
ticket whose blockers are all closed, and no other. For each ticket:

1. **Read.** Read the ticket and its comments (`gh issue view <n> --json body,comments`). Later comments
   may revise the acceptance criteria.
2. **Build.** Work in its own git worktree under `.claude/worktrees/t<n>`, on a branch
   `ticket/<n>-<slug>` cut from an up-to-date `origin/main`. Set one shared build folder for all the
   worktrees of your lane, `CARGO_TARGET_DIR=$HOME/Dev/_cargo-target/ovcd-<lane>`, so a new worktree
   reuses the compiled dependencies and a merged worktree can be removed without losing the build. Use
   one folder **per lane**, not one for all lanes: cargo locks its build folder, so lanes that share one
   wait for each other. Do not use sccache: its cache keys hold the absolute source path, so it gets no
   hits across worktrees. Test first. One PR to `main` closes the
   ticket. Every acceptance criterion ticked in the PR body links to its evidence. Evidence must be
   reachable from the PR: a test name or file in the diff, a command and its output pasted in the
   body, or a CI job. "Kept with the build notes" is not enough; a reviewer must not have to ask the
   builder for it.
3. **Local checks.** CI is the full gate, so before a push run only what the change can break:
   `cargo fmt --all --check`; clippy with warnings as errors on the crates touched; the tests of the crates
   or Python modules touched; `python3 tools/release_guard.py` and `--history`; and the embargo check of
   the PR body (see `issue-tracker.md`). Run the full fast gate (`bash tools/gates.sh`) only for a change
   across many crates, or to reproduce a red CI run.
4. **Auto-merge, and do not wait.** As soon as the PR is open, turn on auto-merge:
   `gh pr merge <pr> --squash --auto --delete-branch --author-email <repo-local address>` (see step 5 for
   the address). Then start the next ticket's reading and tests. Do not watch CI. Branch protection
   requires `test` and `release-guard` green on the head commit, so auto-merge cannot land red code. The
   design-impact check is part of `test`: it reports whether designs moved, and fails only when a change
   is missing from the changelog (run `python3 tools/design_impact.py changelog`, which writes your fragment;
   see "Changelog fragments" below). Branch protection is
   not strict, so a PR needs a rebase only for a real conflict, not each time `main` moves. If CI goes
   red, auto-merge waits: fix it when you see it.
5. **Merge.** Auto-merge (step 4) squash-merges with the repo-local address, so the commit on `main` carries it and not
   the account's personal address: `gh pr merge <pr> --squash --delete-branch --author-email <repo-local
   address>`. The repo-local address is the one in this clone's `git config user.email` (do not write
   it in a ticket, a PR or a doc). GitHub accepts `--author-email` only for an address verified on the merging account (account
   settings, Emails). Until the maintainer verifies it there, GitHub refuses the merge with "Invalid email
   address": merge without `--author-email` and note it in the build notes. A local "main is already used by worktree" message is harmless.
   When the merge lands, check that the PR is merged, the issue is closed, `origin/main` holds the squash commit, and
   `git log -1 --format=%ae origin/main` shows the repo-local address (once it is verified). Then write the **build notes**
   (below) and remove the worktree.

   **Build notes** are one closing comment on the ticket: what was built, the commands run and their
   result, anything not done, and the PR link. Check the comment with the embargo one-liner in
   `issue-tracker.md` first. The comment is the home of the notes; the PR body stays the home of the
   evidence for each ticked criterion. Large outputs (a calibration run, say) go in the PR as a file
   or a pasted block, not only in the comment.

Mutation testing is **not part of the build**: no mutation run before or after a merge, and no
per-ticket reviewer before a merge. See "Hardening pass" below.

## Which CI jobs run where

Only `test` and `release-guard` are required. Keep the others from making a pull request wait (issue #565).

| Workflow, job | On a pull request | Elsewhere |
|---|---|---|
| `ci.yml`: `test` (the `gate-*` jobs), `release-guard`, `drawing-crosscheck`, `app-e2e` | always | push to main, tags |
| `web-parity.yml`: `web-parity` (a release gate) | never | only from `release.yml` (every release), or by hand |
| `app-checks.yml`: `web-app` | only when `app/**`, `crates/ovcd-web/**`, `tools/build_web.sh` or `tools/web_parity.mjs` change | push to main on the same paths; every day |
| `app-checks.yml`: `app-linux`, `app-windows` | never | every day (07:17 UTC); on request (`workflow_dispatch`) |
| `codeql.yml` | never | push to main; every Monday |
| `website.yml` | only when `website/**`, `docs/**` or `*.md` change | push to main |
| `nightly.yml` | when the slow-test list or the gate script changes | every day |

A failed daily run of `app-checks.yml` opens an issue (or comments on the open one).
WebAssembly parity is a release gate: `release.yml` calls `web-parity.yml`, so a release cannot ship with a
mismatch. It left the pull request path because it never failed on a mismatch and it cost a web build on most
pull requests. Start it by hand (`gh workflow run web-parity.yml`) for a change to `crates/ovcd-web/**`.
## Changelog fragments

A pull request never edits `CHANGELOG.md`: that file made every two open pull requests conflict (issue #560).
Each pull request adds one file, `changes/<branch>.md` (the format is in `changes/README.md`):

- Your own bullets for `## Unreleased` go under a `<!-- unreleased -->` line.
- When designs move, `python3 tools/design_impact.py changelog` writes the before/after table into the same
  file, under a `<!-- design-impact -->` line, and keeps your bullets. Commit the file.
- **The gate matches rows, not numbers** (issue #625). A fragment passes when its table names the (case, quantity) rows
  the branch moves; the numbers may be from an earlier base, so a merge on main does not force a regenerate. It fails
  when a moved row is missing, the table names a row that does not move, or there is no table. A table lists at most
  40 rows and counts the rest, so with more than 40 moved rows the check is: every listed row moves, and listed plus
  counted rows equal the moved rows. Regenerate when you change what the branch moves. The merge-group check is unchanged.
- **New branches use fragments.** During the move, the design-impact gate also accepts the table in
  `CHANGELOG.md` (`changelog --legacy` writes it there), so an open branch need not move. A follow-up turns
  that off once the open pull requests have merged.
- A version-bump pull request runs `python3 tools/changelog_collect.py`: it moves every fragment into
  `CHANGELOG.md` and deletes the files. The release notes still come from `CHANGELOG.md`.
- `python3 tools/changelog_collect.py --check` tests that every fragment is well formed.

### The merge queue

GitHub's merge queue merges in batches. `ci.yml` also runs on `merge_group`, so `test` and `release-guard` run
on the commit of a merge group. In a merge group the design-impact check cannot compare one table with the
combined change of several pull requests. Under `merge_group` it checks that the group adds at least one
well-formed fragment (or changes `CHANGELOG.md`) when designs move. The pull request run stays the strict check:
it must hold the table of that pull request.

- A pull request that moves no hash can batch with others.
- A pull request that moves hashes can queue too, because its own pull request run already checked its table.

## After the build: adversarial review

Separate review sessions read what was merged:

1. A **break-it review** of everything merged since the last review: try inputs where the Rust differs
   from the original behaviour, acceptance criteria claimed without evidence, gates that cannot fail, and
   disclosure leaks. Verify each finding before filing it.
2. A **two-axis review** of the whole spec: repo standards, and the spec itself.
3. Each verified finding, and each mutation finding that changes a public output, becomes a fix-forward
   issue (a sub-issue of the spec's issue, labelled `ready-for-agent`). A builder fixes it under the
   same gates.

## Hardening pass (mutation testing)

Mutation testing checks whether the tests would notice a change to the code. It does not check whether
an answer is right; the oracles do that (the design-impact check, Calibration Bands, the evaluation
harness). It pays off on code whose behaviour is settled, and it slows iteration on code that still
changes every day. So it is a separate activity, never a gate on a merge:

- **When:** once per release, before the version bump, or weekly, whichever comes first.
- **What:** only **stable** modules: files unchanged on `main` for about two weeks, or listed as stable by
  the maintainer. Code still under active change is skipped.
- **How:** one run per module on a detached worktree at a fixed `main` commit, with capped concurrency.
  Never stop another session's processes.
- **Output:** one batch of issues. A surviving mutant that shows a **wrong answer** is a `bug`,
  `ready-for-agent`. A test gap is an `enhancement` under the test-gap epic. An equivalent mutant is
  recorded with its reason in the batch's summary comment.
- Acceptance criteria of the form "`cargo mutants` no longer reports …" are checked by the next
  hardening pass, not by the builder.
- **Exception:** mutation of **rule data** (moving a limit or coefficient to check that some evaluation
  case notices) is part of the evaluation harness, not of this pass.

## Releases

Installers are built and published only on purpose (issue #173; `tools/release_plan.py` decides):

0.3 is a developer pre-release: it is for developers and testers, not for expert review. The first release shared with expert reviewers is 0.4. During development the app makes 0.3.x patch releases only, and any minor bump needs the maintainer.

- **A calibration case outside its band** blocks a release unless that release's `CHANGELOG.md` entry names it
  and says why (`calibration/README.md`, "When a band blocks a release"). The `calibration-bands` job of
  `release.yml` checks it; run `python -m vessel_designer.calibration --release-check <version>` before the bump.
- **A version bump.** A merge to `main` that changes `version` in `pyproject.toml` publishes the
  pre-release `v<version>`, tagged at that commit. Bump the version in every place the version test
  names, run `python3 tools/changelog_collect.py` (it moves the fragments into `CHANGELOG.md`), and add a
  `## <version>` section to `CHANGELOG.md` (its text is the release notes; the release fails without it). A `v*` tag pushed by hand does the same.
- **PyPI** (issue #1). `publish-pypi.yml` runs on a `v*` tag pushed by hand (a tag made with `GITHUB_TOKEN` starts no workflow).
  It runs the release gate, builds the sdist, then the `publish` job waits for the maintainer to approve the `pypi`
  environment and uploads the gate's wheels with trusted publishing. There is no token.
- **A test-release tag.** `git tag test-<name> <commit> && git push origin test-<name>` publishes a
  pre-release named after the tag, with the `Unreleased` notes. It is for trying a build; the commit need
  not be on `main`. Delete old test releases by hand when they are no longer needed.
- **Rule Set versions.** A version bump releases the Rule Set versions it ships. In the bump PR, add
  each built-in set version that is not yet in `crates/ovcd-format/rules/RELEASED.json` (`ovcd rules
  --released-entries` prints the entries with their content hashes). From then on the file of a
  listed version never changes: a change is a new version, patch, minor or major by its content hash
  (spec 0004, "Version numbers"). The fast gate (`ovcd rules --provenance`) fails on a changed
  released file, a version number below its class, a rule id that changes its Measure or direction
  or comes back after it was dropped, and a regulation or class Rule with no paragraph. The release
  workflow fails a `v*` release whose built-in set versions are not all listed.
- Any other merge builds and publishes nothing. Both kinds need `test` and `release-guard` green on the
  commit, and the release gate (wheels, binaries, pytest on each system) runs before anything is
  published.

## Stuck tickets

If one problem is still open after two honest attempts, the builder labels the ticket `ready-for-human`,
comments why in plain English (checked with the embargo one-liner), and moves to the next ticket whose
blockers are closed. Tickets that depend on it wait.

## What the maintainer decides

Only flagged tickets. The maintainer delegated the range-table approval (`.vdesign` ticket) and the
switch to the Rust core to the agents. Problems found after a merge are fixed forward.

## Rules every agent follows

- One PR per ticket, straight to `main`. Never push to `main` directly.
- Never loosen a test tolerance to make a number pass. Never hand-edit an expected value: a value is
  printed by the original code or worked out and cited.
- This repo is public. Check every PR body and comment with the embargo one-liner in `issue-tracker.md`,
  and run the release guard with `--history` before every push. Commit with a repo-local address, not a
  personal one, and squash-merge with `--author-email` (step 5), because GitHub stamps the merging
  account's address on a squash merge otherwise. New files need `git add -N` before the guard sees
  them.
- Several worktrees share one clone. Stage files by name, never with `git add -A` in a shared tree, and
  never touch another worktree.
- Never stop a process by name. Stop only a process you started, by its process id.
- Long builds and test runs go to the background with their output in a file that is polled, so a timeout
  cannot lose the result.
- The original Python and its Reference Results are in the history, not in the tree: the last commit that
  has them is named in `docs/specs/0001-parity-harness-and-rust-core.md`.

## Fixed-number tests: labels, and re-pinning a snapshot

A test that pins a number carries a label (`tests/test_test_labels.py` enforces it):

- `Label: sourced - <citation>`: the number comes from a standard, a published example or a hand
  calculation. It changes only when its source does. Name every changed sourced test in the PR body
  (the template has the line "Sourced tests changed").
- `Label: snapshot`: the number is what code printed. For the port it is what the original Python
  printed (frozen at `ce96e66`). It shows that the Rust core still agrees, not that the number is
  right.
- `Label: property`: no fixed number.

In Python the label is a line in the test's docstring. In Rust it is a `//! Label:` line in the file
header; the check covers any file with a long float literal, except the files listed as unlabelled in
`tests/test_test_labels.py`. That list may only shrink: label a file, then delete it from the list.

**Re-pinning a snapshot after an agreed engineering change.** Never hand-edit the value and never
loosen a tolerance. Change the code. Run the code, print the new value, and paste that value into
the test. The before/after table that `tools/design_impact.py` writes into your changelog fragment (run
`python3 tools/design_impact.py changelog`) is the record that the change moved the numbers. Say in
the PR body which snapshots moved and why. A sourced test is not re-pinned this way: it needs a new
source.
