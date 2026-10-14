# Test strategy: parity during the port, then the design-impact check, Calibration Bands, a fast PR gate and a release gate

Status: the parity part is done. Removed in ticket 29: the parity harness, the Reference Results, the parity case inputs, the frozen Python copy, the hidden `ovcd eval` command and the `xtask` regenerate task were deleted. The last commit that still has them is `ce96e66cf0016df3bb36bed1cdc0ae998372564e` (`git show ce96e66cf0016df3bb36bed1cdc0ae998372564e:parity/README.md`; the frozen Python runs from a checkout of that commit with Python 3.12 and numpy).

A test is good if it checks behaviour through a public seam and stops a named risk. Ranked risks:
the port gives different numbers; a later change moves results by accident; the engineering is wrong;
the app breaks on odd inputs; install or build fails on one system; confidential data leaks.

## Decisions

- **During the port:** parity. `eval(entry_point, json)` in `ovcd-format` runs every case against the
  Reference Results from the frozen Python. This is the first seam and the contract.
- **After the port:** the Reference Results are deleted. A design-impact check runs fixed case inputs
  through `ovcd` on `main` and on the branch. It says whether designs moved, with a before/after
  table. There are no stored answers, so nothing goes stale. A design change needs no sign-off. It is
  tracked.
- **Calibration Bands:** a later spec adds a check that blocks a pull request when a Reference
  Vessel leaves its band.
- **Fast gate:** every pull request, Linux, under 5 minutes, one fail-closed script. Required check
  names stay `test` and `release-guard`.
- **Release gate:** per release only, plus a manual trigger. Wheels and binaries for Linux, macOS and
  Windows. No nightly runs.
- Existing Python tests are audited and labelled sourced, snapshot or useless. Useless ones are
  deleted.
- A blocking check that has caught no real problem in 3 months, or fails at random, is removed or
  moved to the release gate.

## Consequences

- Nobody edits Reference Results by hand or loosens the tolerance.
- CI stays fast, and the release gate carries the slow cross-platform work.
