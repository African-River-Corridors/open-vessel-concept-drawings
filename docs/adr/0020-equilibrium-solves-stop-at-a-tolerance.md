# The equilibrium solves stop at a tolerance

Status: **accepted**, 2026-10-23. Amends spec 0008 ("Equilibrium"). Pull request #602 (issue #567).

## Context

Spec 0008 solves the free-trim equilibrium with two Illinois secant solves: an inner solve for the
height `c` that gives the target volume, and an outer solve for the trim θ that puts B on the vertical
through G. As agreed on 2026-10-21, each solve took a fixed number of steps (16 inner, 12 outer) with
**no early stop**. The reason was parity: native code and WebAssembly must give the same bits, and a
stop that depends on the path could make the two stop at different steps.

The fixed count was slow. Most solves reach their answer in a few steps and then repeat it. The default
ShipD Design took about 20 s in the IS Code conditions (issue #567): about 5 conditions of 61 heels,
each a trim solve made of volume solves.

## Decision

- Each solve keeps its step cap (16 inner, 12 outer) and **stops at the first step whose residual is
  within a tolerance**:
  - inner: the volume is within `VOLUME_TOLERANCE` = 1e−13 of the target volume (relative);
  - outer: the lever B − G is within `TRIM_TOLERANCE_M` = 1e−11 m.
- The stop compares only numbers that the solve computes itself, with the same operations on every
  target. So native code and WebAssembly stop at the same step and give the same bits. This keeps the
  parity reason of the original rule.
- The heel 0 equilibrium of a condition is solved once, and its GZ curve starts from it.
- The constants live in `crates/ovcd-core/src/sections.rs`. A change to either is a design change:
  it moves the results hashes and goes through the design-impact gate.

## Evidence

Measured on #602, against main before the change:

- **Same bits, native and WebAssembly.** The WebAssembly parity job passed on the branch.
- **Results move only in the last bits.** On the ShipD barge and the gas fixture, every number agrees
  to better than 1e−11 relative. Values near zero (for example a GZ at heel 0) differ by about 1e−16
  absolute. The design-impact tool found no design change.
- **About 2 times faster.** Volume cuts (calls of `integrate`; wall time moved with machine load, so
  counts are the fair measure):

  | Design | Before | After | Ratio |
  |---|---|---|---|
  | ShipD pushed barge, 80 × 14 × 4.5 m, IS Code conditions | 47,459 | 23,287 | 2.0 |
  | Gas fixture `gas_igc_2g_self_propelled` | 179,099 | 80,599 | 2.2 |

- **Every results hash moved, once.** The last bits move, so all stored hashes (fixtures, the v1 and
  v2 legacy files, the website example) were stamped again in #602. Two pinned values were re-pinned,
  with the old value in a comment.
- The test against an 80 × 60 bisection to 1e−12 still passes.

## Considered options

- **Keep the fixed step count (no early stop).** Rejected: it doubles the work for no change in any
  result above 1e−11, and the parity it protects holds with the stop too.
- **A looser tolerance** (for example 1e−9). Rejected: it would move results by more than the last
  bits, and the gain is small, because the secant method converges fast near the answer.
- **Bisection or Newton.** Already rejected in spec 0008: bisection is too slow for the capacity
  search, and Newton overshot at heavy trim.

## Consequences

- A Design runs about 2 times faster in the stability conditions. Every results hash changed once, in
  #602; later releases compare against the new hashes.
- The parity rule is now "the stop reads only numbers the solve computes", not "no early stop". A new
  solver that stops early must follow the same rule, and the WebAssembly parity job checks it.
