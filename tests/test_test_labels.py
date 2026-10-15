"""Fixed-number tests carry a label: `sourced` or `snapshot` (spec 0001, user story 30; issue #141).

A fixed-number test pins a number. The label says where the number came from, so that a reader
knows what a failure means and what a change needs:

    Label: sourced - <citation>   from a standard, a published example or a hand calculation; the
                                  citation says which. A change needs the source, never a re-pin.
    Label: snapshot               what the code printed (for the port: the original Python, frozen
                                  at ce96e66). After an agreed engineering change it is re-pinned
                                  as `docs/agents/build-process.md` says.
    Label: property               no fixed number: a relation, a bound or an error path.

Python. The label is a line in the test's docstring. The rule covers every test function in
`tests/`, except the files in `INFRASTRUCTURE` (file names or globs): tests of the tooling (packaging, the gate, the
release guard, housekeeping, the design-impact tool), which pin no engineering number. A new test
file is checked until someone adds it to that list, on purpose, in a reviewed change.

Rust. The label is a `Label:` line in the file's leading `//!` comment. The rule covers every file in
`crates/*/tests/` that holds a long float literal (8 or more decimals: a number printed by the
original code), except files in `RUST_UNLABELLED`. That list is the files that already existed when
the rule began; it may only shrink. A new Rust test file with a long literal must be labelled.

Risk stopped: a snapshot passes as engineering evidence, or a new fixed number goes in with no word
on where it came from.
"""
import ast
import fnmatch
import re
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
TESTS = ROOT / "tests"

# Tests of the tooling. They pin no engineering number.
INFRASTRUCTURE = {
    "test_design_impact*.py",
    "test_workflows*.py",
    "test_gates_fail_closed.py",
    "test_housekeeping.py",
    "test_native.py",
    "test_packaging.py",
    "test_public_api.py",
    "test_pystr_tables.py",
    "test_release_guard.py",
    "test_test_labels.py",
    "test_vdesign_edge_sweep.py",
}

# Rust test files that held a long float literal before the rule began and have no `Label:` line.
# Add a `//! Label: ...` line to a file and delete it from here. Never add a file.
RUST_UNLABELLED = {
    "crates/ovcd-cli/tests/design_commands.rs",
    "crates/ovcd-cli/tests/run_command.rs",
    "crates/ovcd-core/tests/aft_arrangement.rs",
    "crates/ovcd-core/tests/barge_capacity.rs",
    "crates/ovcd-core/tests/design_grid.rs",
    "crates/ovcd-core/tests/electric_energy.rs",
    "crates/ovcd-core/tests/gas_arrangement.rs",
    "crates/ovcd-core/tests/gas_stability.rs",
    "crates/ovcd-core/tests/hull.rs",
    "crates/ovcd-core/tests/hydrostatics.rs",
    "crates/ovcd-core/tests/journey_derating.rs",
    "crates/ovcd-core/tests/pressure_vessel.rs",
    "crates/ovcd-core/tests/protective_location.rs",
    "crates/ovcd-core/tests/pyfmt.rs",
    "crates/ovcd-core/tests/resistance.rs",
    "crates/ovcd-core/tests/squat.rs",
    "crates/ovcd-core/tests/tanks.rs",
    "crates/ovcd-core/tests/waves.rs",
    "crates/ovcd-core/tests/weights.rs",
    "crates/ovcd-format/tests/eval_refusals.rs",
    "crates/ovcd-format/tests/gas_stability.rs",
}

LABEL = re.compile(r"^Label:\s*(sourced|snapshot|property)\b(?:\s+-\s+(.*))?$")
LONG_FLOAT = re.compile(r"\b\d+\.\d{8,}")
MIN_CITATION = 15


def label_problem(label_line):
    """Why this `Label:` line is not acceptable, or None."""
    if not label_line:
        return "add a `Label: sourced - <citation> | snapshot | property` line"
    m = LABEL.match(label_line.strip())
    if not m:
        return f"bad label line {label_line!r}"
    if m.group(1) == "sourced" and not (m.group(2) and len(m.group(2).strip()) >= MIN_CITATION):
        return "a sourced test needs a citation: `Label: sourced - <citation>`"
    return None


def docstring_label(doc):
    for line in (doc or "").splitlines():
        if line.strip().startswith("Label:"):
            return line.strip()
    return None


def is_infrastructure(name):
    return any(fnmatch.fnmatch(name, pattern) for pattern in INFRASTRUCTURE)


def python_tests():
    files = sorted(TESTS.rglob("test_*.py"))
    assert files, "no test files found: scanning nothing"
    for path in files:
        if is_infrastructure(path.name):
            continue
        tree = ast.parse(path.read_text(encoding="utf-8"))
        for node in ast.walk(tree):
            if isinstance(node, ast.FunctionDef) and node.name.startswith("test_"):
                yield path, node


def rust_header_label(text):
    """The `Label:` line in the leading `//!` block of a Rust file, or None."""
    for line in text.splitlines():
        s = line.strip()
        if not s.startswith("//!"):
            if s:
                break
            continue
        body = s[3:].strip()
        if body.startswith("Label:"):
            return body
    return None


def rust_files_needing_a_label():
    files = sorted(ROOT.glob("crates/*/tests/*.rs"))
    assert files, "no Rust test files found: scanning nothing"
    for path in files:
        if LONG_FLOAT.search(path.read_text(encoding="utf-8")):
            yield path


PYTHON = list(python_tests())


@pytest.mark.parametrize(
    "path,node", PYTHON, ids=[f"{p.name}::{n.name}" for p, n in PYTHON]
)
def test_every_python_fixed_number_test_is_labelled(path, node):
    """Label: property"""
    problem = label_problem(docstring_label(ast.get_docstring(node)))
    assert problem is None, f"{path.relative_to(ROOT)}::{node.name}: {problem}"


def test_the_infrastructure_list_names_files_that_exist():
    """Label: property"""
    have = {p.name for p in TESTS.rglob("test_*.py")}
    dead = [pat for pat in INFRASTRUCTURE if not fnmatch.filter(have, pat)]
    assert not dead, f"patterns that match no test file: {dead}"


RUST = list(rust_files_needing_a_label())


@pytest.mark.parametrize("path", RUST, ids=[str(p.relative_to(ROOT)) for p in RUST])
def test_every_rust_file_with_fixed_numbers_is_labelled(path):
    """Label: property"""
    rel = str(path.relative_to(ROOT))
    if rel in RUST_UNLABELLED:
        pytest.skip("grandfathered: label it and delete it from RUST_UNLABELLED")
    problem = label_problem(rust_header_label(path.read_text(encoding="utf-8")))
    assert problem is None, f"{rel}: put a `//! Label: ...` line in the file header; {problem}"


def test_the_rust_baseline_only_shrinks():
    """Label: property"""
    needing = {str(p.relative_to(ROOT)) for p in RUST}
    gone = sorted(RUST_UNLABELLED - needing)
    assert not gone, f"no longer need a label (or are gone); delete from RUST_UNLABELLED: {gone}"
    labelled = sorted(
        f
        for f in RUST_UNLABELLED
        if f in needing and rust_header_label((ROOT / f).read_text(encoding="utf-8"))
    )
    assert not labelled, f"already labelled; delete from RUST_UNLABELLED: {labelled}"


# The check can fail: these cases show it.


@pytest.mark.parametrize(
    "line,fragment",
    [
        (None, "add a `Label:"),
        ("Label: guess", "bad label line"),
        ("Label: sourced", "needs a citation"),
        ("Label: sourced - table", "needs a citation"),
        ("Label: sourced - PIANC WG 141 Table 2.1, CEMT class IV", None),
        ("Label: snapshot", None),
        ("Label: property", None),
    ],
)
def test_the_label_rule_accepts_and_refuses_what_it_should(line, fragment):
    """Label: property"""
    problem = label_problem(line)
    if fragment is None:
        assert problem is None
    else:
        assert problem and fragment in problem


def test_the_python_scan_finds_an_unlabelled_test():
    """Label: property"""
    tree = ast.parse('def test_x():\n    """No label here."""\n    assert 1.2345678901 == 1.2345678901\n')
    doc = ast.get_docstring(tree.body[0])
    assert label_problem(docstring_label(doc))


def test_the_rust_scan_reads_only_the_header():
    """Label: property"""
    assert rust_header_label("//! Seam.\n//! Label: snapshot\n\nuse x;\n") == "Label: snapshot"
    assert rust_header_label("//! Seam.\n\nuse x;\n// Label: snapshot\n") is None
    assert LONG_FLOAT.search("let x = 1.23456789012;")
    assert not LONG_FLOAT.search("let x = 1.5; let y = 0.1234567;")
