"""The release gate checks the fixture results hashes on every system (ticket 144).

The script runs against a stand-in `ovcd`, so these tests need no compiler. They show that a hash
mismatch, a missing result, a failing binary or an empty fixture folder fails it, and that only the
release gate (not the pull request gate) runs it.
"""
import json
import stat
import subprocess
import sys
import tempfile
import textwrap
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "tools" / "check_fixture_hashes.py"
sys.path.insert(0, str(ROOT / "tools"))
import check_fixture_hashes as chk  # noqa: E402


def fake_ovcd(tmp_path: Path, status="matches", results=True, code=0, text=None) -> Path:
    out = text if text is not None else json.dumps({"options": [
        {"name": "Option 1", "hash_status": status, "results_hash": "ab" * 32,
         "results": {"x": 1} if results else None}]})
    script = tmp_path / "ovcd"
    script.write_text(textwrap.dedent(f'''\
        #!{sys.executable}
        import sys
        print({out!r})
        sys.exit({code})
    '''))
    script.chmod(script.stat().st_mode | stat.S_IEXEC)
    return script


def fixtures(tmp_path: Path, names) -> Path:
    d = Path(tempfile.mkdtemp(dir=tmp_path))
    for n in names:
        (d / n).write_text("{}")
    return d


SIX = [f"f{i}.vdesign" for i in range(6)]


def run(tmp_path, ovcd, names=SIX):
    return subprocess.run([sys.executable, str(SCRIPT), str(ovcd), str(fixtures(tmp_path, names))],
                          capture_output=True, text=True)


def test_matching_hashes_pass(tmp_path):
    r = run(tmp_path, fake_ovcd(tmp_path))
    assert r.returncode == 0, r.stderr
    assert "every fixture hash matches" in r.stdout


def test_a_hash_mismatch_fails(tmp_path):
    r = run(tmp_path, fake_ovcd(tmp_path, status="differs"))
    assert r.returncode == 1 and "'differs'" in r.stderr


def test_a_missing_stored_hash_fails(tmp_path):
    assert run(tmp_path, fake_ovcd(tmp_path, status="not_stored")).returncode == 1


def test_an_option_without_results_fails_unless_it_is_the_convoy(tmp_path):
    r = run(tmp_path, fake_ovcd(tmp_path, results=False))
    assert r.returncode == 1 and "no results" in r.stderr
    r = run(tmp_path, fake_ovcd(tmp_path, results=False, code=1), names=["pushed_convoy.vdesign"])
    assert r.returncode == 1 and "only 0 Options matched" in r.stderr     # allowed, but nothing matched
    assert "no results yet (allowed)" in r.stdout and "no results\n" not in r.stderr


def test_a_failing_or_silent_binary_fails(tmp_path):
    assert run(tmp_path, fake_ovcd(tmp_path, code=1)).returncode == 1
    assert run(tmp_path, fake_ovcd(tmp_path, text="not json")).returncode == 1


def test_too_few_matches_or_no_fixtures_fail(tmp_path):
    r = run(tmp_path, fake_ovcd(tmp_path), names=SIX[:3])
    assert r.returncode == 1 and "only 3 Options matched" in r.stderr
    assert run(tmp_path, fake_ovcd(tmp_path), names=[]).returncode == 1


def test_a_missing_binary_fails(tmp_path):
    r = subprocess.run([sys.executable, str(SCRIPT), str(tmp_path / "no-such-ovcd")],
                       capture_output=True, text=True)
    assert r.returncode == 1 and "no ovcd binary" in r.stderr


def test_the_real_fixtures_are_what_the_script_expects():
    names = {p.name for p in chk.FIXTURES.glob("*.vdesign")}
    assert chk.NO_RESULTS_YET <= names
    assert len(names - chk.NO_RESULTS_YET) >= 5


# ---------- WebAssembly, under wasmtime (ticket 168) ----------

def fake_wasmtime(tmp_path: Path) -> Path:
    """A stand-in wasmtime: `wasmtime run --dir D module args...` runs `module args...`."""
    script = tmp_path / "wasmtime"
    script.write_text(textwrap.dedent(f'''\
        #!{sys.executable}
        import subprocess, sys
        a = sys.argv[1:]
        assert a[:2] == ["run", "--dir"], a
        sys.exit(subprocess.run([sys.executable, a[3]] + a[4:]).returncode)
    '''))
    script.chmod(script.stat().st_mode | stat.S_IEXEC)
    return script


def run_wasm(tmp_path, module, names=SIX, wasmtime=None):
    wt = wasmtime or fake_wasmtime(tmp_path)
    return subprocess.run([sys.executable, str(SCRIPT), "--wasmtime", str(wt), str(module),
                           str(fixtures(tmp_path, names))], capture_output=True, text=True)


def test_the_wasm_module_is_run_under_wasmtime_and_matching_hashes_pass(tmp_path):
    r = run_wasm(tmp_path, fake_ovcd(tmp_path))
    assert r.returncode == 0, r.stderr
    assert "under" in r.stdout and "every fixture hash matches" in r.stdout


def test_a_wasm_hash_mismatch_fails(tmp_path):
    r = run_wasm(tmp_path, fake_ovcd(tmp_path, status="differs"))
    assert r.returncode == 1 and "'differs'" in r.stderr


def test_a_missing_wasmtime_or_module_fails(tmp_path):
    r = run_wasm(tmp_path, fake_ovcd(tmp_path), wasmtime=tmp_path / "no-such-wasmtime")
    assert r.returncode == 1 and "no wasmtime" in r.stderr
    r = run_wasm(tmp_path, tmp_path / "no-such.wasm")
    assert r.returncode == 1 and "no ovcd binary" in r.stderr


# ---------- where it runs ----------

def workflow(name):
    return yaml.safe_load((ROOT / ".github" / "workflows" / name).read_text(encoding="utf-8"))


def test_the_release_gate_runs_it_on_windows_and_both_macs():
    job = workflow("release.yml")["jobs"]["binaries"]
    assert set(job["strategy"]["matrix"]["os"]) == {"macos-15-intel", "macos-latest", "windows-latest"}
    runs = [s.get("run", "") for s in job["steps"]]
    build = next(i for i, r in enumerate(runs) if "cargo build --release" in r)
    check = next(i for i, r in enumerate(runs) if "tools/check_fixture_hashes.py target/release/ovcd" in r)
    assert check > build
    assert not any("--no-fail" in r or "|| true" in r for r in runs)


def test_the_pull_request_gate_runs_it_natively_through_gates_sh():
    """a wrong stored hash shipped on main, because only the release gate ran the check. The
    fast gate runs it on Linux, in the `impact` group, so it runs on every pull request and merge group.
    The cross-platform runs stay in release.yml."""
    gates = (ROOT / "tools" / "gates.sh").read_text(encoding="utf-8")
    assert 'step impact "fixture results hashes" fixture_hashes' in gates
    assert "tools/check_fixture_hashes.py" in gates
    ci = (ROOT / ".github" / "workflows" / "ci.yml").read_text(encoding="utf-8")
    assert "check_fixture_hashes" not in ci            # it is in gates.sh, not a second place


def test_the_release_gate_runs_it_on_webassembly_with_a_pinned_wasmtime():
    wf = workflow("release.yml")
    job = wf["jobs"]["wasm-hashes"]
    assert job["needs"] == "gate"
    text = "\n".join(str(s.get("run", "")) + str(s.get("with", "")) for s in job["steps"])
    runs = [s.get("run", "") for s in job["steps"]]
    assert "--target wasm32-wasip1" in text
    # An exact wasmtime version, locked.
    assert "cargo install wasmtime-cli --version 49.0.3 --locked" in text
    build = next(i for i, r in enumerate(runs) if "cargo build --release" in r and "wasm32-wasip1" in r)
    check = next(i for i, r in enumerate(runs) if "check_fixture_hashes.py --wasmtime" in r
                 and "ovcd.wasm" in r)
    assert check > build
    assert not any("--no-fail" in r or "|| true" in r for r in runs)
    assert "continue-on-error" not in str(job)


def test_the_default_run_checks_the_website_examples_too(tmp_path):
    # A stand-in that gives a stale hash only for a website example: the default run must fail on it.
    script = tmp_path / "ovcd"
    script.write_text(textwrap.dedent(f'''\
        #!{sys.executable}
        import json, sys
        status = "differs" if "examples" in sys.argv[-1] else "matches"
        print(json.dumps({{"options": [{{"name": "Option 1", "hash_status": status,
                                         "results_hash": "ab" * 32, "results": {{"x": 1}}}}]}}))
    '''))
    script.chmod(script.stat().st_mode | stat.S_IEXEC)
    r = subprocess.run([sys.executable, str(SCRIPT), str(script)], capture_output=True, text=True)
    assert r.returncode == 1, r.stdout
    assert any(p.name in r.stderr for p in chk.EXAMPLES.glob("*.vdesign")), r.stderr
    assert "'differs'" in r.stderr


def test_an_empty_examples_folder_fails(tmp_path):
    empty = Path(tempfile.mkdtemp(dir=tmp_path))
    problems = chk.check(fake_ovcd(tmp_path), fixtures(tmp_path, SIX), also=(empty,))
    assert problems == [f"no .vdesign designs in {empty}"]
