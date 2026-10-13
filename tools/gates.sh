#!/usr/bin/env bash
# Fast gate (spec 0001): one fail-closed script, run by the `test` CI job and by `cargo xtask gates`.
#
#   bash tools/gates.sh
#
# Fail-closed means: a missing tool fails the job before any step runs, any failing step stops the
# job, and the job fails unless every step ran. There is no skip switch.
#
# Steps:
#   1. cargo fmt --check
#   2. clippy, warnings as errors (includes the banned-maths rule in crates/ovcd-core/clippy.toml)
#   3. cargo test, with Python made unusable, to prove the Rust loop needs no Python
#   4. the WebAssembly build of ovcd-core
#   5. the oracle guard (parity oracle changes only via `cargo xtask regen`, with reasons)
#   6. pytest against the shipped package
#
# OVCD_PYTHON picks the Python for steps 5 and 6 (default python3).
set -euo pipefail

cd "$(dirname "$0")/.."
PY="${OVCD_PYTHON:-python3}"
WASM_TARGET="wasm32-unknown-unknown"
EXPECTED_STEPS=6
steps_run=0

fail() { echo "gates: FAIL: $*" >&2; exit 1; }

need() {
  command -v "$1" >/dev/null 2>&1 || fail "required tool '$1' is not on PATH"
}

# --- Tool check: every tool the steps need, before anything runs. -------------------------------
need git
need cargo
need rustc
need "$PY"
cargo fmt --version >/dev/null 2>&1 || fail "rustfmt is not installed (rustup component add rustfmt)"
cargo clippy --version >/dev/null 2>&1 || fail "clippy is not installed (rustup component add clippy)"
[ -d "$(rustc --print target-libdir --target "$WASM_TARGET" 2>/dev/null)" ] \
  || fail "the $WASM_TARGET target is not installed (rustup target add $WASM_TARGET)"
"$PY" -c "import pytest" 2>/dev/null || fail "pytest is not importable by $PY (pip install -e '.[dev]')"

step() {
  local name="$1"; shift
  local start=$SECONDS
  echo "::group::gates: $name"
  "$@" || { echo "::endgroup::"; fail "$name"; }
  echo "::endgroup::"
  steps_run=$((steps_run + 1))
  echo "gates: ok: $name ($((SECONDS - start)) s)"
}

# cargo test with every python on PATH replaced by a shim that records the call and fails.
cargo_test_without_python() {
  local shims marker py
  shims="$(mktemp -d)"
  marker="$shims/python-was-called"
  for py in python python3 "python3.12" "python3.13" "python3.14"; do
    printf '#!/bin/sh\necho "$0 $*" >> "%s"\necho "gates: cargo test tried to run Python" >&2\nexit 97\n' \
      "$marker" > "$shims/$py"
    chmod +x "$shims/$py"
  done
  PATH="$shims:$PATH" OVCD_PYTHON="$shims/python3" cargo test --all-features
  if [ -e "$marker" ]; then
    echo "gates: cargo test called Python:" >&2
    cat "$marker" >&2
    return 1
  fi
}

step "cargo fmt --check" cargo fmt --all --check
step "clippy -D warnings" cargo clippy --all-targets --all-features -- -D warnings
step "cargo test (no Python)" cargo_test_without_python
step "WebAssembly build of ovcd-core" cargo build -p ovcd-core --all-features --target "$WASM_TARGET"
step "oracle guard" env OVCD_PYTHON="$PY" cargo xtask oracle-guard
step "pytest" "$PY" -m pytest -q

[ "$steps_run" -eq "$EXPECTED_STEPS" ] || fail "only $steps_run of $EXPECTED_STEPS steps ran"
echo "gates: all $EXPECTED_STEPS steps passed in $SECONDS s"
