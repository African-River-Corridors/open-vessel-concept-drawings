"""The public API of `vessel_designer` is the one the pure-Python package had (ticket 27).

`tests/data/public_api.json` was taken from the pure-Python package just before the switch to the
Rust core: every module, and in each module the public classes (dataclass fields, frozen flag,
public methods, enum members), functions (parameter names, kinds and defaults) and constants.
After the switch the same imports, names, signatures and result fields must exist, and nothing
public may be added or lost without changing this file on purpose.
"""
import ast
import dataclasses
import enum
import importlib
import inspect
import json
import pkgutil
from pathlib import Path

import vessel_designer


def sig(f):
    out = []
    for p in inspect.signature(f).parameters.values():
        out.append([p.name, p.kind.name, None if p.default is inspect.Parameter.empty else ('<path>' if 'Path(' in repr(p.default) else repr(p.default))])
    return out

def assigned(mod):
    out = set()
    for n in ast.parse(inspect.getsource(mod)).body:
        for t in (n.targets if isinstance(n, ast.Assign) else [n.target] if isinstance(n, ast.AnnAssign) else []):
            if isinstance(t, ast.Name): out.add(t.id)
    return out

def describe(mod):
    names = {}
    mine = assigned(mod)
    for n, v in sorted(vars(mod).items()):
        if n.startswith('_'): continue
        if inspect.ismodule(v): continue
        if inspect.isclass(v):
            if v.__module__ != mod.__name__: continue
            d = {"kind": "class"}
            if issubclass(v, enum.Enum):
                d["members"] = [m.name for m in v]
            if dataclasses.is_dataclass(v):
                d["fields"] = [f.name for f in dataclasses.fields(v)]
                d["frozen"] = v.__dataclass_params__.frozen
            d["methods"] = sorted(m for m, x in vars(v).items()
                                  if not m.startswith('_') and (callable(x) or isinstance(x, (property, classmethod, staticmethod))))
            names[n] = d
        elif inspect.isfunction(v):
            if v.__module__ != mod.__name__: continue
            names[n] = {"kind": "function", "params": sig(v)}
        elif n in mine:
            names[n] = {"kind": "constant"}
    return names



DATA = Path(__file__).parent / "data" / "public_api.json"


def current() -> dict:
    snap = {}
    for m in pkgutil.walk_packages(vessel_designer.__path__, "vessel_designer."):
        if m.name.split(".")[-1].startswith("_"):
            continue
        snap[m.name] = describe(importlib.import_module(m.name))
    return snap


def test_the_snapshot_covers_the_whole_package():
    expected = json.loads(DATA.read_text(encoding="utf-8"))
    assert len(expected) >= 20 and "vessel_designer.core.barge_capacity" in expected
    assert sum(len(v) for v in expected.values()) > 150


def test_public_modules_names_signatures_and_result_fields_are_unchanged():
    expected = json.loads(DATA.read_text(encoding="utf-8"))
    actual = json.loads(json.dumps(current()))
    assert sorted(actual) == sorted(expected), "modules added or lost"
    problems = []
    for module, names in expected.items():
        got = actual[module]
        for name in sorted(set(names) | set(got)):
            if names.get(name) != got.get(name):
                problems.append(f"{module}.{name}: expected {names.get(name)}, got {got.get(name)}")
    assert not problems, "\n".join(problems)
