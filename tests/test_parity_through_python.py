"""Every parity case, run through the PYTHON package, gives the Reference Result (ticket 27).

The parity harness proved the Rust core against the frozen Python. This test proves the other half
of the switch: the shipped `vessel_designer` package, now a thin layer over the Rust core, answers
every parity case with the same numbers, ints and floats, errors and messages as the frozen Python
did. It uses the parity driver's own adapters, pointed at `vessel_designer` instead of the frozen
copy. It goes with the Reference Results when the port is cleaned up (ticket 29).
"""
import importlib.util
import json
import math
from pathlib import Path

import pytest

PARITY = Path(__file__).resolve().parents[1] / "parity"


def _driver():
    spec = importlib.util.spec_from_file_location("parity_driver_for_python", PARITY / "driver.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module.FROZEN = "vessel_designer"       # the shipped package, not vessel_designer_frozen
    return module


def _same(a, b):
    """Equal as the parity comparison reads it: types exact, floats within 1e-9."""
    if isinstance(a, bool) or isinstance(b, bool):
        return a == b and type(a) is type(b)
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        if type(a) is not type(b):
            return False
        return math.isclose(a, b, rel_tol=1e-9, abs_tol=1e-9)
    if isinstance(a, dict):
        return isinstance(b, dict) and list(a) == list(b) and all(_same(a[k], b[k]) for k in a)
    if isinstance(a, list):
        return isinstance(b, list) and len(a) == len(b) and all(_same(x, y) for x, y in zip(a, b))
    return a == b


def _cases():
    out = []
    for path in sorted((PARITY / "cases").glob("*.json")):
        cases = json.loads(path.read_text(encoding="utf-8"))
        refs = json.loads((PARITY / "reference" / path.name).read_text(encoding="utf-8"))["results"]
        for case, ref in zip(cases["cases"], refs):
            out.append((cases["entry_point"], case, ref["output"]))
    return out


def test_every_parity_case_matches_through_the_python_package():
    driver = _driver()
    cases = _cases()
    assert len(cases) > 1000, "scanning too few cases"
    wrong = []
    for entry_point, case, expected in cases:
        got = driver.run_case(entry_point, case["input"])
        if not _same(expected, got):
            wrong.append(f"{entry_point} {case['id']}")
    assert not wrong, f"{len(wrong)} case(s) differ: {wrong[:10]}"


def test_the_driver_really_points_at_the_shipped_package():
    driver = _driver()
    mod = driver._module("core.cargo")
    assert mod.__name__ == "vessel_designer.core.cargo"


def test_the_comparison_can_fail():
    assert not _same({"ok": 1.0}, {"ok": 1.0000001})
    assert not _same({"ok": 1}, {"ok": 1.0})
    assert _same({"ok": [1.0, "x"]}, {"ok": [1.0 + 1e-12, "x"]})
