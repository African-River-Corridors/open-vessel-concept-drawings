"""The list of results keys that the results hash leaves out is the same in Rust and in Python.

Label: property. No fixed number: the two lists must be equal.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import design_impact as di  # noqa: E402


def test_python_and_rust_list_the_same_unhashed_keys():
    """Label: property"""
    text = (ROOT / "crates" / "ovd-format" / "src" / "run.rs").read_text(encoding="utf-8")
    found = re.search(r"pub const UNHASHED_KEYS: \[&str; \d+\] = \[([^\]]*)\];", text)
    assert found, "UNHASHED_KEYS not found in run.rs"
    rust = re.findall(r'"([^"]+)"', found.group(1))
    assert rust == list(di.UNHASHED_KEYS)


def test_design_impact_ignores_an_unhashed_key():
    """Label: property"""
    base = {"cargo_t": 1.0}
    before = {"case": {"results": {**base, "stability_marks": {"criteria_angles_deg": [30, 40]}}}}
    after = {"case": {"results": {**base, "stability_marks": {"criteria_angles_deg": [31, 41]}, "drawing": {"x": 1}}}}
    assert di.compare(before, after) == []
    moved = {"case": {"results": {"cargo_t": 2.0}}}
    assert di.compare(before, moved) != []
