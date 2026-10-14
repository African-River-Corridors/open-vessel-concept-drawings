---
name: worktree-verification-gate
description: Run the OVCD verification gate (fmt, clippy, test, wasm, release guard, fixture validate) from a git worktree under macOS Seatbelt, with workarounds for blocked parent-repo .git writes
source: auto-skill
extracted_at: '2026-10-14T15:45:26.132Z'
---

# Verification gate in a Seatbelt-sandboxed git worktree

When running the ticket verification gate from a `wt-*` worktree of this Rust workspace on macOS
with Qwen Code's Seatbelt sandbox, writes outside the worktree directory are denied — and a
worktree's git metadata (`index.lock`, `worktrees/`) lives in the **parent** repo's `.git`, which
is outside the sandbox.

## The gate (run all in the worktree root)

```bash
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"   # shells are non-interactive; cargo is not on PATH
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build -p ovcd-core --target wasm32-unknown-unknown
python3 tools/release_guard.py
```

- After writing new Rust files, run `cargo fmt --all` before the `--check` — hand-written code
  drifts (import order, call wrapping) and the check fails.
- Every example `.vdesign` fixture must pass `cargo run -q -p ovcd-cli -- validate <file>`.

## Seatbelt blocks git writes into the parent repo's `.git`

- `git add -N <new files>` → `fatal: ... index.lock ... Operation not permitted`.
- `python3 tools/design_impact.py report` → `fatal: could not create directory of
  '.../.git/worktrees/base'` (it runs `git worktree add`).

### release guard without `git add -N`

`release_guard.py` scans `git ls-files`, which cannot see untracked files, so new files are missed.
Scan them with the guard's own scanner (read-only; no index write):

```python
import sys; sys.path.insert(0, "tools")
import release_guard as rg
from pathlib import Path

root = Path(".").resolve()
allow = rg.load_allow(root)
hits, ok = [], []
for rel in new_file_paths:                      # repo-relative strings
    text = (root / rel).read_text(encoding="utf-8", errors="replace")
    rg._scan_text(rel, rel, enumerate(text.splitlines(), 1), allow, None, hits, ok)
print(f"release guard (new files): {len(hits)} hit(s), {len(ok)} allowlisted")
```

Pass `hashes=None`: `embargo.hashes` does not exist (the constant is `embargo.TERM_HASHES`) and
`find_lines(None)` defaults to it.

### design impact without `git worktree add`

Verify "no design change" directly: for a frozen crate such as `crates/ovcd-core`,
`git diff HEAD --stat -- <crate>` and `git status --short -- <crate>` must both be empty. Report
the tool as Seatbelt-blocked and tell the user they must allow writes under the parent repo's
`.git` to run it.

## design_impact.py usage

Requires a mode argument: `python3 tools/design_impact.py report` (also `gate`, `changelog`).
Running it bare prints usage and exits 2.
