# Build process (spec 0001)

How agents build a spec that has been split into tickets. Agreed by the maintainer on 2026-10-13.

## The loop

The tickets are a task graph. Native GitHub "blocked by" links are the edges. A coordinator starts every
ticket whose blockers are all closed. Each ticket then goes through these steps:

1. **Build.** One implementer works in its own git worktree on a branch `ticket/<n>-<slug>`, test first.
   It opens one PR to `main` that closes the ticket.
2. **Machine gates.** The fast gate (`test`) and `release-guard` must be green. From the harness ticket
   on, this includes the oracle guard.
3. **Three reviews, in parallel:**
   - a two-axis review: repo standards, and the ticket and spec;
   - two cold reviews by models that did not write the code. Each tries to **break** the PR. It must name
     a failing input or a missed acceptance criterion, or say "none found". One cold reviewer runs the
     mutation test on the changed core files.
4. **Fix.** At most 2 fix rounds.
5. **Diagnose, if 2 rounds fail.** A diagnose agent forms theories about why the ticket fails, picks a
   fix, and checks the wider effect on other tickets (both ways: does this ticket cause problems
   elsewhere, or is another ticket the cause?). Then it either revises the ticket and starts one more
   build, or flags the maintainer. It flags only where domain judgement (naval architecture, barge
   design) is needed. Code judgement stays with the agents.
6. **Merge.** Squash-merge on green checks and clean reviews. The ticket closes.

A flagged ticket gets the `ready-for-human` label and a comment. Tickets that depend on it wait.

## What the maintainer decides

Only flagged tickets. The maintainer delegated the range-table approval (`.vdesign` ticket) and the
switch to the Rust core to the agents. Problems found after a merge are fixed forward.

## Rules every agent follows

- One PR per ticket, straight to `main`. The shipped package stays pure Python until the switch, so a
  partly ported core never reaches a user.
- Never edit Reference Results by hand. Never loosen the tolerance. A changed parity case needs a recorded reason.
- Every acceptance criterion ticked in a PR body links to its evidence.
- This repo is public. Check every PR body and comment with the embargo one-liner in `issue-tracker.md`,
  and run the release guard with `--history` before every push.
- Implementers use Sonnet for module ports. The harness, the Reference Results, the file format, the
  switch and the design-impact check use Opus.
