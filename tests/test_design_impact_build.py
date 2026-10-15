"""The design-impact check must measure the binary that cargo just built (ticket 130).

A stand-in `cargo` on PATH prints cargo's JSON messages, so these tests need no compiler.
"""
import json
import os
import stat
import sys
import textwrap
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import design_impact as di  # noqa: E402


def stub_cargo(tmp_path, monkeypatch, *, exe_rel="debug/ovcd", write_exe=True, report=True,
               old_mtime=False, code=0):
    """Put a `cargo` first on PATH. It writes `exe_rel` under CARGO_TARGET_DIR and reports it."""
    bindir = tmp_path / "bin"
    bindir.mkdir(exist_ok=True)
    script = bindir / "cargo"
    script.write_text(textwrap.dedent(f'''\
        #!{sys.executable}
        import json, os, sys
        target = os.environ["CARGO_TARGET_DIR"]
        exe = os.path.join(target, {exe_rel!r})
        if {write_exe!r}:
            os.makedirs(os.path.dirname(exe), exist_ok=True)
            open(exe, "w").write("x")
            if {old_mtime!r}:
                os.utime(exe, (1_000_000_000, 1_000_000_000))
        print("not json at all")
        print(json.dumps({{"reason": "compiler-artifact", "executable": None, "target": {{"name": "serde", "kind": ["lib"]}}}}))
        if {report!r}:
            print(json.dumps({{"reason": "compiler-artifact", "executable": exe,
                              "target": {{"name": "ovcd", "kind": ["bin"]}}}}))
        sys.exit({code})
    '''))
    script.chmod(script.stat().st_mode | stat.S_IEXEC)
    monkeypatch.setenv("PATH", f"{bindir}{os.pathsep}{os.environ['PATH']}")


def test_build_returns_the_path_cargo_reports(tmp_path, monkeypatch):
    stub_cargo(tmp_path, monkeypatch, exe_rel="x86_64-unknown/debug/ovcd")
    target = tmp_path / "tgt"
    exe = di.build(tmp_path, target)
    assert exe == target / "x86_64-unknown" / "debug" / "ovcd"      # not the guessed debug/ovcd


def test_a_missing_binary_fails_the_gate(tmp_path, monkeypatch):
    stub_cargo(tmp_path, monkeypatch, write_exe=False)
    with pytest.raises(SystemExit, match="missing"):
        di.build(tmp_path, tmp_path / "tgt")


def test_a_build_that_reports_no_binary_fails_the_gate(tmp_path, monkeypatch):
    stub_cargo(tmp_path, monkeypatch, report=False)
    with pytest.raises(SystemExit, match="no `ovcd` binary"):
        di.build(tmp_path, tmp_path / "tgt")


def test_a_failed_build_fails_the_gate(tmp_path, monkeypatch):
    stub_cargo(tmp_path, monkeypatch, code=101)
    with pytest.raises(SystemExit, match="cargo build failed"):
        di.build(tmp_path, tmp_path / "tgt")


def test_a_binary_outside_the_target_directory_fails_the_gate(tmp_path, monkeypatch):
    stub_cargo(tmp_path, monkeypatch, exe_rel="../elsewhere/ovcd")
    with pytest.raises(SystemExit, match="not under"):
        di.build(tmp_path, tmp_path / "tgt")


def test_a_stale_binary_fails_a_fresh_build(tmp_path, monkeypatch):
    stub_cargo(tmp_path, monkeypatch, old_mtime=True)
    with pytest.raises(SystemExit, match="stale"):
        di.build(tmp_path, tmp_path / "tgt", fresh=True)


def test_a_fresh_build_starts_from_an_empty_target_directory(tmp_path, monkeypatch):
    stub_cargo(tmp_path, monkeypatch)
    target = tmp_path / "tgt"
    (target / "debug").mkdir(parents=True)
    leftover = target / "debug" / "leftover"
    leftover.write_text("old")
    di.build(tmp_path, target, fresh=True)
    assert not leftover.exists()


def test_the_base_build_is_fresh():
    text = (ROOT / "tools" / "design_impact.py").read_text(encoding="utf-8")
    assert 'design-impact-base", fresh=True)' in text
