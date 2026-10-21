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
#   2. clippy, warnings as errors (includes the banned-maths rule: crates/ovcd-core/clippy.toml and
#      crates/ovcd-format/clippy.toml, which hold the same list)
#   3. clippy on ovcd-py, warnings as errors (not a default member, so step 2 skips it; it needs
#      Python only to configure PyO3, and clippy does not link, so the step needs no libpython)
#   4. cargo test, with Python made unusable, to prove the Rust loop needs no Python: cargo-nextest
#      with the profile GATES_NEXTEST_PROFILE (default `pr`; `nightly` runs every test, see
#      .config/nextest.toml), then the doctests, which nextest does not run (issue #481)
#   4a. the Rule Set gate: released versions unchanged, version classes, rule ids, provenance
#       (`ovcd rules --provenance`; it prints the unverified blocking Rules of each set)
#   5. the WebAssembly build of ovcd-core and ovcd-web (ovcd-web builds the pinned otd crate too)
#   6. the design-impact check (before/after on main; fails only if a change is missing from
#      CHANGELOG.md)
#   7. pytest against the shipped package, on every core (pytest-xdist). The `pr` profile leaves out
#      the tests marked `slow`; `nightly` runs them too. docs/agents/slow-tests.md lists every test
#      the `pr` profile leaves out.
#
# OVCD_PYTHON picks the Python for steps 3, 6 and 7 (default python3).
# GATES_NEXTEST_PROFILE picks `pr` (default) or `nightly`.
set -euo pipefail

fail() { echo "gates: FAIL: $*" >&2; exit 1; }

need() {
  command -v "$1" >/dev/null 2>&1 || fail "required tool '$1' is not on PATH"
}

# --- Tool check: every external tool the script calls, before anything runs. --------------------
# A missing tool must fail here, not make a step run inert: without `dirname` the `cd` below lands
# in `/`; without `mktemp` or `chmod` the no-Python shims are never made. Add a `need` line for any
# new tool (tests/test_gates_fail_closed.py lists them too).
need dirname
cd "$(dirname "$0")/.."
PY="${OVCD_PYTHON:-python3}"
WASM_TARGET="wasm32-unknown-unknown"
PROFILE="${GATES_NEXTEST_PROFILE:-pr}"
EXPECTED_STEPS=8
steps_run=0

need git
need mktemp
need chmod
need cat
need env
need cargo
need rustc
need "$PY"
cargo fmt --version >/dev/null 2>&1 || fail "rustfmt is not installed (rustup component add rustfmt)"
cargo clippy --version >/dev/null 2>&1 || fail "clippy is not installed (rustup component add clippy)"
cargo nextest --version >/dev/null 2>&1 || fail "cargo-nextest is not installed (cargo install cargo-nextest --locked)"
case "$PROFILE" in
  pr) PYTEST_SELECT=(-m "not slow") ;;
  nightly) PYTEST_SELECT=() ;;
  *) fail "GATES_NEXTEST_PROFILE must be pr or nightly, not '$PROFILE'" ;;
esac
[ -d "$(rustc --print target-libdir --target "$WASM_TARGET" 2>/dev/null)" ] \
  || fail "the $WASM_TARGET target is not installed (rustup target add $WASM_TARGET)"
"$PY" -c "import pytest" 2>/dev/null || fail "pytest is not importable by $PY (pip install -e '.[dev]')"
"$PY" -c "import xdist" 2>/dev/null || fail "pytest-xdist is not importable by $PY (pip install -e '.[dev]')"

# Note: "$@" runs inside `||`, so bash ignores `set -e` in it. A step that is a shell function
# must return non-zero on every failure itself (see cargo_test_without_python).
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
  # Explicit `|| return 1`: `step` calls this function inside `||`, where bash ignores `set -e`,
  # so without it a failing cargo test would fall through and the step would report ok.
  PATH="$shims:$PATH" OVCD_PYTHON="$shims/python3" \
    cargo nextest run --profile "$PROFILE" --all-features --no-tests=fail || return 1
  PATH="$shims:$PATH" OVCD_PYTHON="$shims/python3" cargo test --doc --all-features || return 1
  if [ -e "$marker" ]; then
    echo "gates: cargo test called Python:" >&2
    cat "$marker" >&2
    return 1
  fi
}

step "cargo fmt --check" cargo fmt --all --check
step "clippy -D warnings" cargo clippy --all-targets --all-features -- -D warnings
step "clippy ovcd-py -D warnings" env PYO3_PYTHON="$PY" cargo clippy -p ovcd-py --all-targets --all-features -- -D warnings
step "cargo test (no Python)" cargo_test_without_python
step "Rule Set gate" cargo run -q -p ovcd-cli -- rules --provenance
step "WebAssembly build of ovcd-core and ovcd-web" cargo build -p ovcd-core -p ovcd-web --all-features --target "$WASM_TARGET"
step "design impact" "$PY" tools/design_impact.py gate
step "pytest" "$PY" -m pytest -q -n auto ${PYTEST_SELECT[@]+"${PYTEST_SELECT[@]}"}

[ "$steps_run" -eq "$EXPECTED_STEPS" ] || fail "only $steps_run of $EXPECTED_STEPS steps ran"
echo "gates: all $EXPECTED_STEPS steps passed in $SECONDS s"
