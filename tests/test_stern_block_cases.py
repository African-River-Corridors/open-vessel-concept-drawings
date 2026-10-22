"""The stern-block cases (issue #365): built ships whose stern block and aft peak are measured from the
maker's general arrangement. The list is well formed, and each figure says how it was read. The run
itself needs the `ovcd` binary: `python3 tools/stern_block_calibration.py <ovcd> --check`."""

from pathlib import Path

import yaml

CAL = Path(__file__).resolve().parent.parent / "calibration"


def _cases():
    return yaml.safe_load((CAL / "propulsion_cases.yaml").read_text(encoding="utf-8"))["stern_block_case"]


def test_at_least_five_fitted_ships_each_with_a_source_hash_and_a_method():
    """Label: property"""
    cases = _cases()
    assert sum(c["fit"] for c in cases) >= 5
    for c in cases:
        assert len(c["sha256"]) == 64 and c["source_file"] and c["url"].startswith("http")
        assert "scale" in c["method"] and c["tol_m"] > 0
        assert c["observed"]["stern_block_m"] > 0
        assert c["run_as_type"] in {"shaft_nozzle", "azimuth_ducted"}


def test_a_ship_kept_out_of_the_fit_says_why():
    """Label: property"""
    for c in _cases():
        assert c["fit"] or c["excluded"]


def test_the_stern_block_band_exists_for_every_confidence():
    """Label: property"""
    bands = yaml.safe_load((CAL / "bands.yaml").read_text(encoding="utf-8"))["bands"]["stern_block_m"]
    assert set(bands["pct"]) == {"high", "medium", "low"}
    assert {c["confidence"] for c in _cases()} <= set(bands["pct"])
