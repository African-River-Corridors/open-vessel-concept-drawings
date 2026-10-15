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


def test_the_pull_request_gate_does_not_run_it():
    for name in ("ci.yml",):
        assert "check_fixture_hashes" not in (ROOT / ".github" / "workflows" / name).read_text(encoding="utf-8")
    assert "check_fixture_hashes" not in (ROOT / "tools" / "gates.sh").read_text(encoding="utf-8")
