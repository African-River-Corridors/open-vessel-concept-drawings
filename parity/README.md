# Parity harness (spec 0001)

The Rust core is a port of the Python core with identical results. This folder is the proof. It is
deleted, with the frozen Python and the hidden `ovcd eval` command, when the port is complete.

| Path | What it holds | Who writes it |
|---|---|---|
| `cases/<entry_point>.json` | Case inputs, one file per parity entry point | People (tiers 1 and 3); `sweep.py` (tier 2) |
| `reference/<entry_point>.json` | Reference Results: the frozen Python's output for every case | `cargo xtask regen` only |
| `tolerance.json` | Float tolerance: relative and absolute 1e-9 | Never loosened |
| `ported.json` | Entry points the Rust dispatch has ported; each must be fully green | Port tickets add to it |
| `frozen_python/vessel_designer_frozen/` | Frozen copy of the pure-Python code, under its own import name | Nobody: it does not change |
| `driver.py` | Runs the cases through the frozen Python | Guarded: a change needs a reason in `generator.json` |
| `sweep.py` | Generates the tier-2 cases: a seeded sweep, knife-edge points dropped | Guarded: a change needs a reason in `generator.json` |
| `generator.json` | `{"reason": ...}`: why the generator (`driver.py`, `sweep.py`, `frozen_python/`) last changed | Whoever changes the generator |

## Case and result shapes

A case: `{"id": "get_cargo/LNG", "tier": 1, "reason": "...", "input": {"fn": "get_cargo", "args": {...}}}`.
Tier 1 is a named case, tier 2 a seeded sweep point, tier 3 an error case. Every case carries the
reason it exists or last changed. A removed case moves to the file's `retired` list, with a reason.
A tier-1 or tier-3 case that sits on a knife edge also carries `knife_edge`: what flips, and why Rust
reaches the same side (see "What identical means").

A Reference Result is `{"ok": <value>}`, or `{"error": {"type": "ValueError", "message": "..."}}`
when Python raises (`ValueError`, or `KeyError` for an unknown lookup key; `type` is the class
name). Floats are shortest round-trip text. NaN and infinity are the strings `"float:nan"`,
`"float:inf"` and `"float:-inf"`.

## The three tiers (issue #9)

Every entry point in spec 0001 has cases in all three tiers, and every public function it lists
has a tier-1 case and (unless its input space is a fixed list) tier-2 cases.
`tests/test_parity_cases.py` checks this, and `crates/ovcd-format/tests/parity.rs` checks the
tiers per entry point.

- **Tier 1, named.** Inputs of the audited tests, the quickstart run (`examples/quickstart.py`)
  and the calibration cases. Each `reason` says which.
- **Tier 2, seeded sweep.** `sweep.py` draws points from the valid input space of each function,
  with one `random.Random` per function seeded from `SEED`. A point is kept only if the frozen
  Python returns a normal result and the point is not on a knife edge: moving every float input
  by a relative 1e-12 (speed-grid inputs excepted) must not change any discrete output, and must
  not move any float output outside the parity tolerance. Nobody edits tier-2 cases by hand;
  `cargo xtask regen` rewrites them, and the test fails if the committed files differ from the
  generator by one byte. To change tier 2, change `sweep.py`, and say why in its `REVISIONS`
  entry for that function: the note goes into every changed point's `reason`, which the oracle
  guard needs. `tests/test_parity_cases.py` also lists inputs a port could hardcode at their
  defaults (`MUST_VARY`); the sweep must move each one.
- **Tier 3, errors.** A `ValueError`, a `KeyError` from a lookup, or a `{"error": ...}` result. The
  reason starts `tier 3: error path`. Where Python has no error path (hull geometry, for
  example), tier 3 holds degenerate inputs instead, with a reason that starts
  `tier 3: degenerate input`.

### Adapter shapes worth knowing

The driver's adapter for each entry point is the contract the Rust `eval` must copy:

- `hull` `quantities`: one `HullGeometry` (`args.hull`), every public quantity at `args.draft`,
  then the plating areas, in a fixed key order.
- `tanks`, `weights`, `hydrostatics` (`calculate_stability`, `calculate_trim`): `args.inputs` is
  plain `BargeInputs` data. The hull carries the inputs' own rakes; the tank arrangement and the
  weights come from the frozen functions, in that order.
- `protective_location` `required_*_protection`: `vessel_type` is the `VesselType` value as text
  (or null); an unknown text raises the enum's own `ValueError`.
- `barge_cost` `compute`: `rates` builds `BargeCostRates` field by field; `rates_mapping` passes a
  plain mapping through `BargeCostRates.from_mapping`, the path a user's rates file takes.

Tier-1 cases that come from the audit of the Python tests (issue #8) name their test in `reason`.
The test's docstring lists the case ids back (a `Parity:` line), and `tests/test_test_audit.py`
checks both directions.

## What "identical" means

Floats match within `tolerance.json` (Python's `math.isclose` rule). Everything else matches
exactly: integers (an integer never matches a float), booleans, text including messages, list order,
object keys. A knife-edge case is nudged or replaced, with the reason recorded on the case. The
tolerance is never loosened. A tier-1 or tier-3 case may stay on a knife edge (an audited test that
checks the flip itself, or an input exactly on a rule's bound) only with a `knife_edge` text; the
test fails if the sweep's own detector flags a tier-1 or tier-3 case that has none.

## Tasks

```
cargo test                    # parity seam: every case of every ported entry point, in-process
cargo xtask regen             # rewrite tier 2 (sweep.py), then reference/, from the frozen Python (needs Python + numpy: `pip install -e ".[parity]"`)
cargo xtask oracle-guard      # what the fast gate runs; see below
cargo xtask mutants           # cargo-mutants on the ovcd-core code this branch changed
```

Reference Results are generated on Linux, as CI does. macOS libm differs from Linux in the last
digit for some floats, so `regen` on a Mac changes `electric_energy`, `gas_stability`, `resistance`,
`simulate` and `waves`, and the oracle guard then fails on CI. Regenerate in a Linux container
(amd64, Python 3.12, numpy and pyyaml installed) and commit that output:

```
docker run --rm --platform linux/amd64 -v "$PWD":/w -w /w python:3.12-slim \
  sh -c "pip install -q numpy pyyaml && python3 parity/sweep.py && python3 parity/driver.py parity/reference"
```

The tier-2 cases themselves come out the same on a Mac and on Linux; only the Reference Results
differ. The guard fails on a Mac for the same reason; CI is the judge.

`cargo test` needs no Python. The `ovcd-cli` test also runs a few cases through `ovcd eval`.

## Oracle guard

The fast gate fails a pull request that changes `cases/`, `reference/`, `tolerance.json`,
`ported.json` or the generator (`driver.py`, `sweep.py`, any file under `frozen_python/`), unless:

- re-running the frozen Python reproduces every committed Reference Result byte for byte (so the
  change came from `cargo xtask regen`, not a hand edit);
- every new case, and every case whose input or Reference Result changed, carries a new `reason`;
- every removed case is in `retired` with a reason;
- the tolerance is not loosened (and never above 1e-9), never negative, always has a non-empty
  `reason`, and a tightened tolerance has a new reason;
- no entry point leaves `ported.json`;
- `cases/` and `reference/` hold only top-level `.json` files (the driver never reads a nested
  folder, so a nested file would be an unchecked oracle);
- no number in a guarded file is an integer outside the 64-bit range (it would read as a float,
  and two different big integers would compare equal);
- a changed, added or removed generator file comes with a new, non-empty `reason` in
  `generator.json`. The guard then re-runs the changed generator, which must still reproduce
  every committed Reference Result.

The guard re-runs the PR's own generator, not the base branch's, and guards the generator
instead. Running the base generator would block every legitimate generator change: a tier-2
change is made by changing `sweep.py`, and its new cases can only be reproduced by the new
`sweep.py`. So a generator change is allowed, but never silently: it fails without a recorded
reason, and the reason and the diff are then in front of the reviewers.

## Mutation test

`cargo xtask mutants` runs `cargo-mutants` on the `ovcd-core` lines this branch changed against
`origin/main`, with every workspace test (so the parity test does the killing), and writes a short
killed / survived / timeout report to `target/mutants-report.md`. Link the report from the PR body.
A surviving mutant that changes a public output is closed by a new parity case, regenerated, with
the reason "kills mutant ...".
