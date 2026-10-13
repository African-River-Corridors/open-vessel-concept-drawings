# Parity harness (spec 0001)

The Rust core is a port of the Python core with identical results. This folder is the proof. It is
deleted, with the frozen Python and the hidden `ovcd eval` command, when the port is complete.

| Path | What it holds | Who writes it |
|---|---|---|
| `cases/<entry_point>.json` | Case inputs, one file per parity entry point | People (later: a seeded sweep script) |
| `reference/<entry_point>.json` | Reference Results: the frozen Python's output for every case | `cargo xtask regen` only |
| `tolerance.json` | Float tolerance: relative and absolute 1e-9 | Never loosened |
| `ported.json` | Entry points the Rust dispatch has ported; each must be fully green | Port tickets add to it |
| `frozen_python/vessel_designer_frozen/` | Frozen copy of the pure-Python code, under its own import name | Nobody: it does not change |
| `driver.py` | Runs the cases through the frozen Python | |

## Case and result shapes

A case: `{"id": "get_cargo/LNG", "tier": 1, "reason": "...", "input": {"fn": "get_cargo", "args": {...}}}`.
Tier 1 is a named case, tier 2 a seeded sweep point, tier 3 an error case. Every case carries the
reason it exists or last changed. A removed case moves to the file's `retired` list, with a reason.

A Reference Result is `{"ok": <value>}`, or `{"error": {"type": "ValueError", "message": "..."}}`
when Python raises (`ValueError`, or `KeyError` for an unknown lookup key; `type` is the class
name). Floats are shortest round-trip text. NaN and infinity are the strings `"float:nan"`,
`"float:inf"` and `"float:-inf"`.

Tier-1 cases that come from the audit of the Python tests (issue #8) name their test in `reason`.
The test's docstring lists the case ids back (a `Parity:` line), and `tests/test_test_audit.py`
checks both directions.

## What "identical" means

Floats match within `tolerance.json` (Python's `math.isclose` rule). Everything else matches
exactly: integers (an integer never matches a float), booleans, text including messages, list order,
object keys. A knife-edge case is nudged or replaced, with the reason recorded on the case. The
tolerance is never loosened.

## Tasks

```
cargo test                    # parity seam: every case of every ported entry point, in-process
cargo xtask regen             # rewrite reference/ from the frozen Python (needs Python + numpy: `pip install -e ".[parity]"`)
cargo xtask oracle-guard      # what the fast gate runs; see below
cargo xtask mutants           # cargo-mutants on the ovcd-core code this branch changed
```

`cargo test` needs no Python. The `ovcd-cli` test also runs a few cases through `ovcd eval`.

## Oracle guard

The fast gate fails a pull request that changes `cases/`, `reference/`, `tolerance.json` or
`ported.json`, unless:

- re-running the frozen Python reproduces every committed Reference Result byte for byte (so the
  change came from `cargo xtask regen`, not a hand edit);
- every new case, and every case whose input or Reference Result changed, carries a new `reason`;
- every removed case is in `retired` with a reason;
- the tolerance is not loosened (and never above 1e-9), and a tightened tolerance has a new reason;
- no entry point leaves `ported.json`.

## Mutation test

`cargo xtask mutants` runs `cargo-mutants` on the `ovcd-core` lines this branch changed against
`origin/main`, with every workspace test (so the parity test does the killing), and writes a short
killed / survived / timeout report to `target/mutants-report.md`. Link the report from the PR body.
A surviving mutant that changes a public output is closed by a new parity case, regenerated, with
the reason "kills mutant ...".
