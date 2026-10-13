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
