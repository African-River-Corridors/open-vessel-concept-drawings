"""The fast gate fails closed: a missing tool fails the job before any step runs.

Spec 0001: one fail-closed gates script; a missing tool or a skipped step fails the job. Risk
stopped: CI goes green because a check silently did not run.
"""
import os
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
GATES = ROOT / "tools" / "gates.sh"

# Every external tool the script calls. Each must be in the tool check, or a missing one can make a
# step run inert (a missing `mktemp` or `chmod` leaves the no-Python shims unmade, and cargo test
# then runs with the real Python on PATH and still reports ok).
GATES_TOOLS = ["dirname", "git", "mktemp", "chmod", "cat", "env"]


def _path_with_only(tmp_path: Path, tools: list[str], stubs: tuple[str, ...] = ()) -> str:
    """A PATH that holds only the named tools (symlinked from the real PATH), plus do-nothing
    stubs that only need to exist for the tool check."""
    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    for tool in tools:
        found = shutil.which(tool)
        assert found, f"test needs {tool} on PATH"
        (bin_dir / tool).symlink_to(found)
    for stub in stubs:
        (bin_dir / stub).write_text("#!/bin/sh\nexit 0\n")
        (bin_dir / stub).chmod(0o755)
    return str(bin_dir)


def _gates(env_path: str, **extra) -> subprocess.CompletedProcess:
    env = {"PATH": env_path, "HOME": os.environ.get("HOME", "/tmp"), **extra}
    bash = shutil.which("bash")
    return subprocess.run([bash, str(GATES)], cwd=ROOT, env=env, capture_output=True, text=True)


def test_gates_fail_when_cargo_is_missing(tmp_path):
    path = _path_with_only(tmp_path, GATES_TOOLS)
    result = _gates(path, OVCD_PYTHON=sys.executable)
    assert result.returncode != 0
    assert "required tool 'cargo' is not on PATH" in result.stderr
    assert "gates: ok:" not in result.stdout


def test_gates_fail_when_python_is_missing(tmp_path):
    path = _path_with_only(tmp_path, GATES_TOOLS, stubs=("cargo", "rustc"))
    result = _gates(path, OVCD_PYTHON="no-such-python")
    assert result.returncode != 0
    assert "required tool 'no-such-python' is not on PATH" in result.stderr
    assert "gates: ok:" not in result.stdout


def test_gates_fail_when_cargo_test_fails(tmp_path):
    """A failing `cargo test` must fail its own step, not pass it and leave a later step to catch
    it. Regression: `step` runs the function inside `||`, where bash ignores `set -e`."""
    libdir = tmp_path / "wasm-libdir"
    libdir.mkdir()
    path = _path_with_only(tmp_path, GATES_TOOLS)
    bin_dir = Path(path)
    # cargo passes every call except `cargo test`; rustc reports an installed wasm target.
    (bin_dir / "cargo").write_text(
        '#!/bin/sh\nif [ "$1" = test ]; then echo "test result: FAILED" >&2; exit 101; fi\nexit 0\n'
    )
    (bin_dir / "rustc").write_text(f'#!/bin/sh\necho "{libdir}"\n')
    for stub in ("cargo", "rustc"):
        (bin_dir / stub).chmod(0o755)
    result = _gates(path, OVCD_PYTHON=sys.executable)
    assert result.returncode != 0
    assert "gates: FAIL: cargo test (no Python)" in result.stderr
    assert "gates: ok: cargo test (no Python)" not in result.stdout
    assert "gates: ok: clippy -D warnings" in result.stdout


def test_gates_fail_when_cargo_nextest_is_missing(tmp_path):
    """The gate runs the Rust tests with cargo-nextest (issue #481): without it the gate fails
    before any step runs, not with a later step that never ran the tests."""
    libdir = tmp_path / "wasm-libdir"
    libdir.mkdir()
    path = _path_with_only(tmp_path, GATES_TOOLS)
    bin_dir = Path(path)
    (bin_dir / "cargo").write_text(
        '#!/bin/sh\nif [ "$1" = nextest ]; then echo "no such command: nextest" >&2; exit 101; fi\nexit 0\n'
    )
    (bin_dir / "rustc").write_text(f'#!/bin/sh\necho "{libdir}"\n')
    for stub in ("cargo", "rustc"):
        (bin_dir / stub).chmod(0o755)
    result = _gates(path, OVCD_PYTHON=sys.executable)
    assert result.returncode != 0
    assert "cargo-nextest is not installed" in result.stderr
    assert "gates: ok:" not in result.stdout


@pytest.mark.parametrize("missing", GATES_TOOLS)
def test_gates_fail_when_any_tool_they_call_is_missing(tmp_path, missing):
    libdir = tmp_path / "wasm-libdir"
    libdir.mkdir()
    path = _path_with_only(tmp_path, [t for t in GATES_TOOLS if t != missing])
    bin_dir = Path(path)
    (bin_dir / "cargo").write_text("#!/bin/sh\nexit 0\n")
    (bin_dir / "rustc").write_text(f'#!/bin/sh\necho "{libdir}"\n')
    # A Python that passes the pytest import check but fails any real step, so an inert run
    # cannot recurse into this test suite.
    (bin_dir / "py-stub").write_text(
        f'#!/bin/sh\nif [ "$1" = "-c" ]; then exec "{sys.executable}" "$@"; fi\nexit 1\n'
    )
    for stub in ("cargo", "rustc", "py-stub"):
        (bin_dir / stub).chmod(0o755)
    result = _gates(path, OVCD_PYTHON=str(bin_dir / "py-stub"))
    assert result.returncode != 0
    assert f"required tool '{missing}' is not on PATH" in result.stderr
    assert "gates: ok:" not in result.stdout


# ---------- the lint steps fail on real code (ticket 135) ----------
# These run the real gates.sh on a copy of the repository with one bad line added. They stop at the
# lint step, so they need only the compile time of clippy. They need a real cargo, so they do not use
# the stub PATH of the tests above.

BANNED_CALL = """
#[allow(dead_code)]
fn banned_probe(x: f64) -> f64 {
    x.powi(2) + x.mul_add(2.0, 1.0) + x.exp()
}
"""
CLIPPY_WARNING = """
#[allow(dead_code)]
fn clippy_probe() -> i32 {
    let x = 1;
    x + 0
}
"""


@pytest.fixture(scope="module")
def repo_copy(tmp_path_factory):
    """The tracked and new files of this checkout, copied. The bad line goes into the copy only."""
    out = subprocess.run(["git", "ls-files", "-co", "--exclude-standard", "-z"], cwd=ROOT,
                         capture_output=True, text=True, check=True).stdout
    dest = tmp_path_factory.mktemp("gates-copy")
    for name in filter(None, out.split("\0")):
        src = ROOT / name
        if src.is_file():
            (dest / name).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src, dest / name)
    return dest


def _gates_on_copy(copy: Path, rel: str, addition: str) -> subprocess.CompletedProcess:
    assert shutil.which("cargo"), "this test needs a real cargo"
    target = copy / rel
    original = target.read_text(encoding="utf-8")
    target.write_text(original + addition, encoding="utf-8")
    try:
        # The CI job that runs this test sets GATES_STEPS and GATES_PARTITION for itself (issue #561):
        # the gate on the copy must run in full.
        env = {k: v for k, v in os.environ.items() if k not in ("GATES_STEPS", "GATES_PARTITION")}
        env.update({"OVCD_PYTHON": sys.executable,
                    "CARGO_TARGET_DIR": os.environ.get("CARGO_TARGET_DIR", str(ROOT / "target"))})
        return subprocess.run([shutil.which("bash"), str(copy / "tools" / "gates.sh")], cwd=copy, env=env,
                              capture_output=True, text=True, timeout=1500)
    finally:
        target.write_text(original, encoding="utf-8")


def test_a_banned_maths_call_in_simulate_fails_the_gate(repo_copy):
    r = _gates_on_copy(repo_copy, "crates/ovcd-format/src/simulate.rs", BANNED_CALL)
    assert r.returncode != 0
    assert "gates: ok: cargo fmt --check" in r.stdout            # the line is well formatted ...
    assert "gates: FAIL: clippy -D warnings" in r.stderr         # ... and stopped by the lint
    out = r.stdout + r.stderr
    for banned in ("f64::powi", "f64::mul_add", "f64::exp"):
        assert f"use of a disallowed method `{banned}`" in out, banned
    assert "gates: ok: cargo test" not in r.stdout


def test_a_clippy_warning_in_ovcd_py_fails_the_gate(repo_copy):
    r = _gates_on_copy(repo_copy, "crates/ovcd-py/src/lib.rs", CLIPPY_WARNING)
    assert r.returncode != 0
    assert "gates: ok: clippy -D warnings" in r.stdout           # the default members are clean ...
    assert "gates: FAIL: clippy ovcd-py -D warnings" in r.stderr  # ... the bindings are not
    assert "gates: ok: cargo test" not in r.stdout


def test_the_gate_lints_ovcd_py_and_counts_the_step():
    text = GATES.read_text(encoding="utf-8")
    assert 'step lint "clippy ovcd-py -D warnings"' in text and "-p ovcd-py" in text
    steps = [line for line in text.splitlines() if line.startswith("step ")]
    assert f"EXPECTED_STEPS={len(steps)}" in text


# ---------- step groups and partitions (issue #561) ----------

def _stub_gates(tmp_path, **env):
    """gates.sh with a do-nothing cargo, rustc and Python: every step passes without running anything."""
    libdir = tmp_path / "wasm-libdir"
    libdir.mkdir()
    path = _path_with_only(tmp_path, GATES_TOOLS)
    bin_dir = Path(path)
    (bin_dir / "cargo").write_text("#!/bin/sh\nexit 0\n")
    (bin_dir / "rustc").write_text(f'#!/bin/sh\necho "{libdir}"\n')
    (bin_dir / "py-stub").write_text("#!/bin/sh\nexit 0\n")
    for stub in ("cargo", "rustc", "py-stub"):
        (bin_dir / stub).chmod(0o755)
    return _gates(path, OVCD_PYTHON=str(bin_dir / "py-stub"), **env)


def test_a_group_name_that_is_not_a_group_fails_before_any_step(tmp_path):
    """Label: property"""
    r = _stub_gates(tmp_path, GATES_STEPS="lint,tests")
    assert r.returncode != 0
    assert "GATES_STEPS names 'tests'" in r.stderr
    assert "gates: ok:" not in r.stdout


def test_a_partition_that_is_not_k_of_n_fails(tmp_path):
    """Label: property"""
    r = _stub_gates(tmp_path, GATES_PARTITION="two")
    assert r.returncode != 0
    assert "GATES_PARTITION must look like 2/4" in r.stderr


def test_a_named_group_runs_all_its_steps_and_no_others(tmp_path):
    """Label: property"""
    r = _stub_gates(tmp_path, GATES_STEPS="lint")
    assert r.returncode == 0, r.stderr
    assert r.stdout.count("gates: ok:") == 3
    assert "gates: ok: pytest" not in r.stdout
    assert "the 3 steps of lint passed" in r.stdout


def test_the_whole_gate_still_counts_every_step(tmp_path):
    """Label: property"""
    r = _stub_gates(tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.stdout.count("gates: ok:") == 8
    assert "all 8 steps passed" in r.stdout
