"""Where the calibration pass finds its files (issue #217).

From an installed wheel the module's own path is under site-packages, which has no `calibration/`
folder. So importing the module reads no file, `OVCD_CALIBRATION_DIR` names the folder, and the bands
are read from the folder `CAL` names when they are first used. `tests/conftest.py` points the variable
at the `calibration/` next to `tests/`, so these tests run the same from a checkout and from the wheel
check's scratch copy.
"""
import os
import subprocess
import sys
from pathlib import Path

from vessel_designer import calibration


def test_importing_the_module_reads_no_file_and_takes_the_folder_from_the_environment(tmp_path):
    """Label: property
    """
    folder = tmp_path / "no-such-folder"
    env = {**os.environ, "OVCD_CALIBRATION_DIR": str(folder)}
    out = subprocess.run([sys.executable, "-c", "import vessel_designer.calibration as c; print(c.CAL)"],
                         capture_output=True, text=True, env=env, cwd=tmp_path)
    assert out.returncode == 0, out.stderr
    assert out.stdout.strip() == str(folder)


def test_the_bands_are_read_from_the_folder_cal_names_when_used(tmp_path, monkeypatch):
    """Label: property
    """
    (tmp_path / "bands.yaml").write_text("bands: {lightship_t: {label: Test}}\n")
    monkeypatch.setattr(calibration, "CAL", tmp_path)
    assert dict(calibration.BANDS) == {"bands": {"lightship_t": {"label": "Test"}}}
    monkeypatch.undo()
    assert set(calibration.BANDS["bands"]) == {"lightship_t", "dwt_t"}


def test_the_tests_use_the_calibration_folder_next_to_them():
    """Label: property
    """
    assert calibration.CAL == Path(__file__).resolve().parents[1] / "calibration"
