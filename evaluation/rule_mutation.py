"""Mutation of rule data (issue #279; spec 0004, "How the evaluation harness uses it", T5.6).

For each numeric limit and coefficient of each evaluated Rule of a selected set, write a mutated copy
of the set with the value moved by one step to each side, load it at run time, and run the corpus. A
value that moves without any case changing its verdict on that Rule is a weak corpus: no case sits
close enough to the limit to notice it. Warnings only: a weak corpus fails nothing.

A mutated copy keeps the family, so its rule ids are the built-in ones; its set id is
`<id>-mutant-<n>`, so it loads next to the built-in set and is selected alone.
"""
from __future__ import annotations

import copy
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RULES = ROOT / "crates" / "ovcd-format" / "rules"

#: One step per unit for a constant limit, above the engine's grid error (the big steps of the
#: boundary mutants in README.md: areas 0.002 m rad, GZ 0.01 m, angle 2 degrees; GM, also in
#: metres, takes the GZ step, which is above its own step of 0.005 m).
STEPS = {"m.rad": 0.002, "m": 0.01, "deg": 2.0}
#: The step of a formula coefficient, of a table value, and of a constant in a unit not in `STEPS`:
#: this share of the value (or this much, for a zero).
RELATIVE_STEP = 0.01
#: The Rule kinds whose limit is a number to move.
LIMIT_KINDS = ("limit", "table", "formula")


def set_text(key: str) -> str:
    """The text of a built-in set, by its `id@version` key."""
    set_id, version = key.split("@")
    return (RULES / set_id / f"{version}.json").read_text(encoding="utf-8")


def step_of(unit: str | None, value: float) -> float:
    """A constant limit's step comes from its unit; any other number moves by `RELATIVE_STEP`."""
    if unit in STEPS:
        return STEPS[unit]
    return abs(value) * RELATIVE_STEP or RELATIVE_STEP


def _number(v) -> bool:
    return isinstance(v, (int, float)) and not isinstance(v, bool)


def _tables_named(limit) -> list[str]:
    """The ids of the tables a limit reads: `table`, or `{"table": ...}` inside an operand."""
    names = []
    if isinstance(limit, dict):
        if isinstance(limit.get("table"), str):
            names.append(limit["table"])
        for v in limit.values():
            if isinstance(v, dict):
                names += _tables_named(v)
    return names


def _places(doc: dict) -> list[dict]:
    """Every number a mutant can move, per Rule: `rule` (its id), `where` (readable), `path` (in
    the set's JSON), `value` and `unit` (the unit of a constant limit; None for the rest). The
    Rules that are not evaluated, notes and checks have none. A table's `when` bounds are where a
    row applies, not a limit, and do not move."""
    out = []
    table_index = {t["id"]: i for i, t in enumerate(doc.get("tables", []))}
    for i, r in enumerate(doc["rules"]):
        if not r.get("evaluated", True) or r["kind"] not in LIMIT_KINDS:
            continue
        limit = r.get("limit") or {}
        for k, v in limit.items():
            if not _number(v):
                continue
            constant = k == "value"
            out.append({"rule": r["id"], "where": "limit" if constant else f"coefficient {k}",
                        "unit": limit.get("unit") if constant else None, "value": v,
                        "path": ("rules", i, "limit", k)})
        for name in dict.fromkeys(_tables_named(limit)):
            ti = table_index.get(name)
            if ti is None:
                continue
            for j, row in enumerate(doc["tables"][ti].get("rows", [])):
                for k, v in row.items():
                    if k != "when" and _number(v):
                        out.append({"rule": r["id"], "where": f"table {name} row {j + 1} {k}",
                                    "unit": None, "value": v, "path": ("tables", ti, "rows", j, k)})
    return out


def _put(doc: dict, path: tuple, value: float) -> None:
    node = doc
    for p in path[:-1]:
        node = node[p]
    node[path[-1]] = value


def mutants(key: str, text: str | None = None) -> list[dict]:
    """The mutated copies of the set `key` (`id@version`): one per number and side. Each has the
    Rule, where the number is, its value, the moved value, the side (`below` or `above`), the step,
    the copy's text and the key to select it by."""
    doc = json.loads(text if text is not None else set_text(key))
    out = []
    for place in _places(doc):
        step = step_of(place["unit"], place["value"])
        for side, sign in (("below", -1.0), ("above", 1.0)):
            m = copy.deepcopy(doc)
            moved = float(f"{place['value'] + sign * step:.12g}")
            _put(m, place["path"], moved)
            m["id"] = f"{doc['id']}-mutant-{len(out) + 1}"
            out.append({"rule": place["rule"], "where": place["where"], "value": place["value"],
                        "moved": moved, "side": side, "step": step, "unit": place["unit"],
                        "text": json.dumps(m), "key": f"{m['id']}@{m['version']}"})
    return out


def verdicts(trace: dict, rule_id: str) -> list[str]:
    """The outcome of each row of one Rule in a trace, in trace order."""
    return [r["outcome"] for r in trace["rows"] if r["rule"] == rule_id]


def run(cases: list[dict], select: list[str], trace_of, rule_sets: tuple[str, ...] = ()) -> list[dict]:
    """Run every mutant of every selected set over `cases`. `trace_of(case, rule_sets, select)` gives
    a case's Rule Trace; `rule_sets` are set texts loaded at run time (a selected set not among them
    is a built-in one). Each result is a mutant (without its text) and `noticed_by`, the cases whose
    verdict on the mutated Rule changed."""
    loaded = {f"{d['id']}@{d['version']}": t for t in rule_sets for d in [json.loads(t)]}
    baseline = {c["id"]: trace_of(c, rule_sets, tuple(select)) for c in cases}
    out = []
    for key in select:
        for m in mutants(key, loaded.get(key)):
            chosen = tuple(m["key"] if k == key else k for k in select)
            noticed = [c["id"] for c in cases
                       if verdicts(trace_of(c, rule_sets + (m["text"],), chosen), m["rule"])
                       != verdicts(baseline[c["id"]], m["rule"])]
            out.append({**{k: v for k, v in m.items() if k != "text"}, "set": key, "noticed_by": noticed})
    return out
