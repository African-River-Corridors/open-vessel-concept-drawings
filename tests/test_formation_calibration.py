"""The Formation calibration cases (issue #359, part 2): the case file is well formed, and the Defaults
Profile check is in the tool (`--check`). The fit itself needs the `ovcd` binary:
`python3 tools/formation_calibration.py <ovcd> --check` (see the tool)."""

from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
CAL = ROOT / "calibration"


def _cases():
    return yaml.safe_load((CAL / "formation_cases.yaml").read_text(encoding="utf-8"))


def test_every_case_names_its_source_hash_and_each_assumed_figure():
    """Label: property"""
    for c in _cases()["formation_case"]:
        assert len(c["sha256"]) == 64 and len(c["barge_sha256"]) == 64
        assert c["source_file"] and c["published"] and c["observed"]["installed_power_kw"] > 0
        assert c["speed_basis"].split(":")[0] in {"published", "assumed", "derived"}
        assert c["barge_basis"].split(":")[0] in {"published", "derived"}
        assert ("dwt_t_per_barge" in c) != ("draught_m" in c)


def test_a_case_that_cannot_run_says_why():
    """Label: property"""
    for c in _cases()["not_yet_runnable_formation_cases"]:
        assert c["published"] and c["not_yet_runnable"]


def test_at_least_one_case_is_held_out_and_one_is_fitted():
    """Label: property"""
    cases = _cases()["formation_case"]
    assert any(c.get("held_out") for c in cases) and any(not c.get("held_out") for c in cases)
