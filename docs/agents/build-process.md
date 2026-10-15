# Build process (spec 0001)

How a spec that has been split into tickets is built, reviewed and merged. Agreed by the maintainer on
2026-10-13. Since 2026-10-14 it runs in **single-agent mode**: one builder works the tickets one at a
time, and adversarial review happens after the build, not on every ticket.

## The loop

The tickets are a task graph. Native GitHub "blocked by" links are the edges. The builder takes the next
ticket whose blockers are all closed, and no other. For each ticket:

1. **Read.** Read the ticket and its comments (`gh issue view <n> --json body,comments`). Later comments
   may revise the acceptance criteria.
2. **Build.** Work in its own git worktree under `.claude/worktrees/t<n>`, on a branch
   `ticket/<n>-<slug>` cut from an up-to-date `origin/main`. Test first. One PR to `main` closes the
   ticket. Every acceptance criterion ticked in the PR body links to its evidence. Evidence must be
   reachable from the PR: a test name or file in the diff, a command and its output pasted in the
   body, or a CI job. "Kept with the build notes" is not enough; a reviewer must not have to ask the
   builder for it.
3. **Local gates.** `cargo fmt --check`, clippy with warnings as errors, `cargo test --workspace`, the
   WebAssembly build, the Python tests, `python3 tools/release_guard.py --history`, and the embargo check
   of the PR body (see `issue-tracker.md`). `bash tools/gates.sh` runs the fast gate in one go.
4. **CI.** The `test` and `release-guard` jobs must be green on the head commit. The design-impact check
   is part of `test`: it reports whether designs moved, and fails only when a change is missing from
   `CHANGELOG.md` (run `python3 tools/design_impact.py changelog`).
5. **Merge.** Squash-merge with the repo-local address, so the commit on `main` carries it and not
   the account's personal address: `gh pr merge <pr> --squash --delete-branch --author-email <repo-local
   address>`. The repo-local address is the one in this clone's `git config user.email` (do not write
   it in a ticket, a PR or a doc). GitHub accepts `--author-email` only for an address verified on the merging account (account
   settings, Emails). Until the maintainer verifies it there, GitHub refuses the merge with "Invalid email
   address": merge without `--author-email` and note it in the build notes. A local "main is already used by worktree" message is harmless.
   Check that the PR is merged, the issue is closed, `origin/main` holds the squash commit, and
   `git log -1 --format=%ae origin/main` shows the repo-local address (once it is verified). Then write the **build notes**
   (below) and remove the worktree.

   **Build notes** are one closing comment on the ticket: what was built, the commands run and their
   result, anything not done, and the PR link. Check the comment with the embargo one-liner in
   `issue-tracker.md` first. The comment is the home of the notes; the PR body stays the home of the
   evidence for each ticked criterion. Large outputs (a calibration run, say) go in the PR as a file
   or a pasted block, not only in the comment.
6. **Mutation run, after the merge.** A background mutation run on the merged change starts and the
   builder moves on without waiting. Its findings are reviewed later; the builder does not fix them.

There is no mutation test and no per-ticket reviewer before a merge.

## After the build: adversarial review

Separate review sessions read what was merged:

1. A **break-it review** of everything merged since the last review: try inputs where the Rust differs
   from the original behaviour, acceptance criteria claimed without evidence, gates that cannot fail, and
   disclosure leaks. Verify each finding before filing it.
2. A **two-axis review** of the whole spec: repo standards, and the spec itself.
3. Each verified finding, and each mutation finding that changes a public output, becomes a fix-forward
   issue (a sub-issue of the spec's issue, labelled `ready-for-agent`). A builder fixes it under the
   same gates.

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
