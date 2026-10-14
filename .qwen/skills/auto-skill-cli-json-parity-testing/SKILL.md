---
name: cli-json-parity-testing
description: Test that a CLI's JSON output equals a library API's output — compare parsed stdout at the CLI's shape, not exit status, and deduplicate messages the API attaches to two places
source: auto-skill
extracted_at: '2026-10-14T15:45:26.132Z'
---

# Comparing CLI JSON output with an API's output in tests

When a test asserts a CLI command prints exactly what a library API returns (e.g. session
`evaluate()` vs `ovcd run --json`), three findings from a failing pass matter:

## 1. Compare the printed output, not the exit status

A CLI may exit non-zero while still printing valid JSON — e.g. `ovcd run --json` exits non-zero
when a result is not computable (a pushed convoy: `results: null, message: ...`) and still prints
the full JSON document. `assert!(out.status.success())` fails even though the outputs match. Parse
stdout and compare values; keep the exit status out of the assertion. On parse failure, panic with
the captured stdout and stderr so the message shows what was actually printed:

```rust
let out = ovcd(&["run", "--json", path]);
let run: Value = serde_json::from_str(&stdout(&out))
    .unwrap_or_else(|e| panic!("run --json is JSON: {e}{}{}", stdout(&out), stderr(&out)));
```

## 2. Messages attached to two places appear twice

A cross-check can sit on both of its inputs in a structured per-input API output (errors on input A
and on input B), while a flat report lists the sentence once. Deduplicate before comparing or the
comparison fails with a spurious duplicate.

## 3. Project the API output to the CLI's shape first

Build the expected CLI JSON from the richer API result in a small helper (drop fields the CLI
omits; turn `results: null` into `{name, results: null, message}`; otherwise
`{name, results_hash, hash_status, results}`). Comparing at the CLI's shape keeps the assertion
honest and the diff readable.
