"""The tests the PR gate leaves out are the ones docs/agents/slow-tests.md lists (issue #481).

Label: property - three lists of one set: the `pr` filter of .config/nextest.toml, the pytest tests
marked `slow`, and the table of docs/agents/slow-tests.md.
"""
import ast
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
NEXTEST = ROOT / ".config" / "nextest.toml"
LIST = ROOT / "docs" / "agents" / "slow-tests.md"
GATES = ROOT / "tools" / "gates.sh"

# The tests a PR gate must never leave out: the gates themselves.
GATE_TESTS = ("false_pass", "fixture_hash", "results_hash", "web_parity", "release_guard", "embargo",
              "design_impact")


def _listed() -> set[str]:
    rows = re.findall(r"^\| `([^`]+)` \|", LIST.read_text(encoding="utf-8"), flags=re.M)
    return set(rows)


def _nextest_left_out() -> set[str]:
    text = NEXTEST.read_text(encoding="utf-8")
    pr = text.split("[profile.pr]", 1)[1].split("[profile.", 1)[0]
    pairs = re.findall(r"binary_id\(([^)]+)\) & test\(=([^)]+)\)", pr)
    return {f"{binary} {test}" for binary, test in pairs}


def _pytest_slow() -> set[str]:
    out = set()
    for path in sorted((ROOT / "tests").glob("test_*.py")):
        tree = ast.parse(path.read_text(encoding="utf-8"))
        for node in tree.body:
            if isinstance(node, ast.FunctionDef):
                for dec in node.decorator_list:
                    if ast.unparse(dec) == "pytest.mark.slow":
                        out.add(f"tests/{path.name}::{node.name}")
    return out


def test_the_list_names_exactly_the_tests_the_pr_gate_leaves_out():
    """Label: property - the table, the nextest `pr` filter and the `slow` marks name one set."""
    nextest, slow = _nextest_left_out(), _pytest_slow()
    assert nextest and slow
    assert _listed() == nextest | slow


def test_no_gate_test_is_left_out():
    """Label: property - no test of a gate is on the slow list."""
    for name in _listed():
        assert not any(g in name for g in GATE_TESTS), name


def test_the_gate_runs_the_pr_profile_by_default_and_every_test_nightly():
    """Label: property - `pr` is the default profile and `nightly` selects every test."""
    text = GATES.read_text(encoding="utf-8")
    assert 'PROFILE="${GATES_NEXTEST_PROFILE:-pr}"' in text
    assert 'pr) PYTEST_SELECT=(-m "not slow") ;;' in text
    assert "nightly) PYTEST_SELECT=() ;;" in text
    nightly = NEXTEST.read_text(encoding="utf-8").split("[profile.nightly]", 1)[1]
    assert 'default-filter = "all()"' in nightly
