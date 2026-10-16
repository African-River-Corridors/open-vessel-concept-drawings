"""Shared test setup.

The calibration tests read the `calibration/` folder next to `tests/`. From an installed wheel the
module cannot find it from its own path, so point it there (issue #217). Set before any test module
imports `vessel_designer.calibration`; subprocesses inherit it.
"""
import os
from pathlib import Path

os.environ.setdefault("OVCD_CALIBRATION_DIR", str(Path(__file__).resolve().parents[1] / "calibration"))
