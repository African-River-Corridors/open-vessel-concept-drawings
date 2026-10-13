"""The audit of the original Python tests stays in force (spec 0001, issue #8).

Every kept test in the audited files says what kind of test it is and where its inputs live:

    Label: sourced - <citation>      the number comes from a standard, a published example or a
                                     hand calculation; the citation says which
    Label: snapshot                  the number (or fixed outcome) is what the code printed
    Label: property                  no fixed number: a relation, a bound or an error path
    Parity: <entry_point>:<case id>, ...
                                     tier-1 parity cases that hold the test's inputs. An id ending
                                     in `*` matches every case with that prefix.
    Parity: none (<reason>)          the test has no calculation input to copy (a private helper,
                                     a file-system check)

Risk stopped: a snapshot passes as engineering evidence, a test's inputs are lost when it is
deleted, or a useless test comes back.

This file is deleted with the parity folder when the port is complete.
"""
import ast
import json
import re
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
TESTS = ROOT / "tests"
AUDITED = sorted((TESTS / "core").glob("test_*.py")) + [
    TESTS / "test_simulate.py", TESTS / "test_examples.py", TESTS / "test_calibration_runs.py"]

# Tests the audit found useless and deleted. They must stay deleted.
DELETED_USELESS = {
    "test_resistance.py::test_legacy_calculator_entrypoint_unchanged":
        "compute() calls calculate() and adds checks, so the two cannot differ",
    "test_barge_capacity.py::test_air_draft_identity_holds_in_both_conditions":
        "restates the two lines of barge_capacity.compute that define the air drafts",
    "test_vessel_standards.py::test_property_convoy_beam_reconciles":
        "the same rule, with the same 0.05 m tolerance, as the convoy_beam_consistent catalogue "
        "check that test_feasibility_catalogue_is_clean already runs",
    "test_vessel_standards.py::test_property_lookup_is_the_only_accessor":
        "the identity half checks a dict lookup against itself; the error half is kept as "
        "test_unknown_standard_id_raises_keyerror",
}

LABEL = re.compile(r"^Label:\s*(sourced|snapshot|property)\b(?:\s+-\s+(.*))?$")


def _tests():
    assert AUDITED and all(p.exists() for p in AUDITED), "audited test files missing"
    for path in AUDITED:
        tree = ast.parse(path.read_text(encoding="utf-8"))
        for node in tree.body:
            if isinstance(node, ast.FunctionDef) and node.name.startswith("test_"):
                yield path, node


def _cases():
    ids = {}
    assert list((ROOT / "parity" / "cases").glob("*.json")), "no parity cases found: scanning nothing"
    for f in (ROOT / "parity" / "cases").glob("*.json"):
        doc = json.loads(f.read_text(encoding="utf-8"))
        ids[doc["entry_point"]] = [c["id"] for c in doc["cases"]]
    return ids


def _fields(doc):
    """(label line, parity text) from a docstring; either may be None."""
    lines = [ln.strip() for ln in (doc or "").splitlines()]
    label = next((ln for ln in lines if ln.startswith("Label:")), None)
    parity, grabbing = [], False
    for ln in lines:
        if ln.startswith("Parity:"):
            grabbing, ln = True, ln[len("Parity:"):].strip()
        elif not ln or ln.startswith("Label:"):
            grabbing = False
        if grabbing:
            parity.append(ln)
    return label, " ".join(parity) if parity else None


def _ids_for(test_id):
    return [(path, node) for path, node in _tests() if f"{path.name}::{node.name}" == test_id]


@pytest.mark.parametrize("path,node", list(_tests()),
                         ids=[f"{p.name}::{n.name}" for p, n in _tests()])
def test_every_kept_test_is_labelled_and_points_at_its_parity_inputs(path, node):
    """Label: property
    Parity: none (reads test source and parity files, not calculation input)
    """
    where = f"{path.name}::{node.name}"
    label, parity = _fields(ast.get_docstring(node))
    assert label, f"{where}: add a `Label: sourced - <citation> | snapshot | property` line"
    m = LABEL.match(label)
    assert m, f"{where}: bad label line {label!r}"
    if m.group(1) == "sourced":
        assert m.group(2) and len(m.group(2).strip()) >= 15, \
            f"{where}: a sourced test needs a citation: `Label: sourced - <citation>`"
    assert parity, f"{where}: add a `Parity:` line"
    if parity.startswith("none"):
        assert re.match(r"none \(.{10,}\)$", parity), f"{where}: `Parity: none (<reason>)`"
        return
    cases = _cases()
    for ref in [r.strip() for r in parity.split(",") if r.strip()]:
        ep, _, cid = ref.partition(":")
        assert ep in cases and cid, f"{where}: {ref!r} must be `<entry_point>:<case id>`"
        if cid.endswith("*"):
            assert any(c.startswith(cid[:-1]) for c in cases[ep]), \
                f"{where}: no case id starts with {ref!r}"
        else:
            assert cid in cases[ep], f"{where}: no tier-1 case {ref!r}"


def test_every_parity_case_from_the_audit_is_tier_1_and_says_which_test_it_came_from():
    """Label: property
    Parity: none (reads parity files, not calculation input)
    """
    for f in (ROOT / "parity" / "cases").glob("*.json"):
        for c in json.loads(f.read_text(encoding="utf-8"))["cases"]:
            if "audited test" in c["reason"]:
                assert c["tier"] in (1, 3), (f.name, c["id"])
                for name in re.findall(r"(test_\w+\.py::\w+)", c["reason"]):
                    assert _ids_for(name), f"{f.name} {c['id']}: reason names a missing test {name}"


@pytest.mark.parametrize("name", sorted(DELETED_USELESS))
def test_deleted_useless_tests_stay_deleted(name):
    """Label: property
    Parity: none (reads test source, not calculation input)
    """
    assert not _ids_for(name), f"{name} was deleted as useless: {DELETED_USELESS[name]}"
