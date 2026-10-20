"""The Rule Set JSON Schema and the Rust loader agree (spec 0004, issue #268).

Label: property
"""
from __future__ import annotations

import copy
import json
from pathlib import Path

import pytest

jsonschema = pytest.importorskip("jsonschema")

ROOT = Path(__file__).resolve().parents[1]
RULES = ROOT / "crates" / "ovcd-format" / "rules"
SCHEMA = json.loads((RULES / "rule-set.schema.json").read_text())


def example() -> dict:
    return json.loads((RULES / "examples" / "example-1.0.0.json").read_text())


def errors(doc: dict) -> list[str]:
    validator = jsonschema.Draft202012Validator(SCHEMA)
    return [e.message for e in validator.iter_errors(doc)]


def test_the_schema_is_a_valid_schema():
    """Label: property"""
    jsonschema.Draft202012Validator.check_schema(SCHEMA)


def test_the_example_set_validates():
    """Label: property"""
    assert errors(example()) == []


def test_every_rule_set_file_in_the_repo_validates():
    """Label: property"""
    files = [p for p in RULES.rglob("*.json") if p.name != "rule-set.schema.json"]
    assert files, "no Rule Set files found"
    for path in files:
        assert errors(json.loads(path.read_text())) == [], path


def _bad(change) -> dict:
    doc = copy.deepcopy(example())
    change(doc)
    return doc


@pytest.mark.parametrize(
    "name, change",
    [
        ("missing family", lambda d: d.pop("family")),
        ("unknown field", lambda d: d["rules"][0].update(override=True)),
        ("malformed rule id", lambda d: d["rules"][0].update(id="example/1.1")),
        ("person named", lambda d: d["rules"][0]["codified"].update(by="A. Person")),
        ("limit on a check", lambda d: d["rules"][4].update(compare="ge")),
        ("no gap when not evaluated", lambda d: d["rules"][4].pop("gap")),
        ("note consequence on a limit", lambda d: d["rules"][0].update(consequence="note")),
        ("regulation with no locator", lambda d: d["rules"][0]["source"].pop("locator")),
        ("negative tolerance", lambda d: d["rules"][0].update(tolerance=-1)),
        ("bad version", lambda d: d.update(version="1.0")),
        ("bad finding", lambda d: d["rules"][5].update(finding="282")),
    ],
)
def test_the_schema_refuses_what_the_loader_refuses(name, change):
    """Label: property"""
    # The Rust test crates/ovcd-format/tests/rule_set.rs refuses each of these in the loader.
    assert errors(_bad(change)), name
