"""The release check finds `calibration/` and `CHANGELOG.md` after `pip install .` (issue #664).

A non-editable install puts the module in site-packages, so its repo root (`parents[2]`) is
`lib/python3.x`, with no `calibration/` and no `CHANGELOG.md`. The release gate runs the check from
the checkout, so the module falls back to the working directory. When the files are truly missing the
check still fails, and says what it could not find.
"""
import os
import shutil
import subprocess
import sys
from pathlib import Path

from ovd import calibration

CHECKOUT_CAL = Path(__file__).resolve().parents[1] / "calibration"


def _installed_layout(tmp_path: Path) -> tuple[Path, Path]:
    """A copy of the module in a fake site-packages, as `pip install .` lays it out. Returns the
    site-packages folder and the module's repo root (`parents[2]`), which holds no calibration files."""
    site = tmp_path / "venv" / "lib" / "python3.12" / "site-packages"
    pkg = site / "ovd_installed_copy"
    pkg.mkdir(parents=True)
    (pkg / "__init__.py").write_text("", encoding="utf-8")
    shutil.copy(calibration.__file__, pkg / "calibration.py")
    return site, pkg.resolve().parents[1]


def _release_check(site: Path, cwd: Path) -> subprocess.CompletedProcess:
    env = {k: v for k, v in os.environ.items() if k != "OVD_CALIBRATION_DIR"}
    env["PYTHONPATH"] = os.pathsep.join([str(site), env.get("PYTHONPATH", "")])
    return subprocess.run([sys.executable, "-m", "ovd_installed_copy.calibration", "--release-check", "9.9.9"],
                          capture_output=True, text=True, env=env, cwd=cwd)


def test_the_folder_comes_from_the_working_directory_when_the_install_root_has_none(tmp_path):
    """Label: property
    """
    root = tmp_path / "lib" / "python3.12"
    root.mkdir(parents=True)
    checkout = tmp_path / "checkout"
    (checkout / "calibration").mkdir(parents=True)
    assert calibration._calibration_dir(root, checkout, {}) == checkout / "calibration"
    # A checkout or editable install keeps its own folder, and the variable still wins.
    (root / "calibration").mkdir()
    assert calibration._calibration_dir(root, checkout, {}) == root / "calibration"
    assert calibration._calibration_dir(root, checkout, {"OVD_CALIBRATION_DIR": "/x"}) == Path("/x")
    # Neither folder: the root's folder, so the error names the path the install looked for.
    assert calibration._calibration_dir(tmp_path / "a", tmp_path / "b", {}) == tmp_path / "a" / "calibration"


def test_the_release_check_runs_from_the_checkout_after_a_non_editable_install(tmp_path):
    """Label: property
    """
    site, root = _installed_layout(tmp_path)
    assert not (root / "calibration").exists() and not (root / "CHANGELOG.md").exists()
    checkout = tmp_path / "checkout"
    shutil.copytree(CHECKOUT_CAL, checkout / "calibration")
    (checkout / "CHANGELOG.md").write_text("# Changelog\n\n## 9.9.9\n\n- Test entry.\n", encoding="utf-8")
    out = _release_check(site, checkout)
    assert "FileNotFoundError" not in out.stderr, out.stderr
    assert "calibration: release 9.9.9:" in out.stdout, out.stdout + out.stderr
    assert out.returncode in (0, 1)                      # 1 = a band miss left unexplained: the check ran


def test_the_release_check_fails_when_the_files_are_truly_missing(tmp_path):
    """Label: property
    """
    site, _ = _installed_layout(tmp_path)
    empty = tmp_path / "empty"
    empty.mkdir()
    out = _release_check(site, empty)
    assert out.returncode == 2, out.stdout + out.stderr
    assert "Calibration release check cannot run" in out.stdout and "calibration" in out.stdout
    assert "calibration: release" not in out.stdout
