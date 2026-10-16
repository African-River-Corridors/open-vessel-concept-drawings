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
    # Was 150. Issue #175 removed 20 names (see REMOVED_ON_PURPOSE), so the floor moved with them.
    assert sum(len(v) for v in expected.values()) > 130


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


# --- names importable in 0.1.0 (issue #143) --------------------------------------------------
#
# The snapshot above lists what each module defines. A script also imports what a module only
# imported itself (`from vessel_designer.simulate import RULESETS`). `importable_names_0_1_0.json`
# holds every name each public module of the 0.1.0 package had (made by
# `tests/data/make_importable_names_0_1_0.py` from commit ce96e66). Every name of the package must
# still import. Names that came from the standard library, typing or numpy are not kept when the
# module no longer needs them; CHANGELOG.md lists them.

NAMES = json.loads((Path(__file__).parent / "data" / "importable_names_0_1_0.json").read_text(encoding="utf-8"))["modules"]
NOT_KEPT = {"math", "np", "replace", "asdict", "dataclass", "field", "Optional", "Tuple"}
ROOT = Path(__file__).resolve().parents[1]

# Names of 0.1.0 that issue #175 removed on purpose, with the reason. The maintainer's decision: the
# engine applies IGC rules only to gas tanks (the ADN_G ruleset and everything that served it is
# gone), a tank's outer diameter cap is the input `max_tank_od_m` (16 m by default) instead of a
# 12 m constant, and there is no 28 m cap on tank length. CHANGELOG.md lists them.
REMOVED_ON_PURPOSE = {
    "core.design_grid": {"MAX_TANK_LENGTH_M", "n_tanks_for_length"},
    "core.gas_separation": {"SIDE_MIN_TYPE_G_M", "DOUBLE_BOTTOM_MIN_TYPE_G_M"},   # issue #215
    # Issue #237: the stepped d(Vc) (total volume; 0.8, 1.0, 2.0 m) is removed; `d_vc_exact` is the one
    # rule (IGC 2.4.1.1, per tank). The enum member `IGC_2G_EXACT` went with it (a file that names it
    # still opens, as IGC_2G).
    "core.barge_capacity": {"MAX_TANK_OD_M", "distance_d_vc"},                 # now DEFAULT_MAX_TANK_OD_M = 16 m
    "core.gas_arrangement": {"ADN_G_MAX_L_OVER_D", "ADN_G_TANK_CAP_9_3_4_M3", "ADN_G_WIDE_TANK_FRACTION",
                             "ADN_LONGITUDINAL_GAP_M", "MAX_TANK_OD_M", "distance_d_vc"},
    "core.protective_location": {"ADN_G_BOTTOM_M", "ADN_G_DOUBLE_BOTTOM_M", "ADN_G_DOUBLE_HULL_M",
                                 "ADN_G_INSPECTION_GAP_M", "ADN_G_MAX_L_OVER_D", "ADN_G_SADDLE_M",
                                 "ADN_G_SIDE_M", "ADN_G_TANK_CAP_9_3_4_M3", "ADN_G_TANK_CAP_M3",
                                 "ADN_G_WIDE_TANK_FRACTION", "distance_d_vc"},
}
# Constants that stay but whose value changed on purpose: the rulesets lost `ADN_G`.
CHANGED_ON_PURPOSE = {"core.gas_arrangement": {"RULESETS"}, "simulate": {"RULESETS"}}
# Dataclass fields that went on purpose: the grid no longer has a tank-length cap.
REMOVED_FIELDS = {("core.design_grid", "DesignGrid", "max_tank_length_m")}
# Dataclass fields whose type changed on purpose. Issue #180: `None` gives the propulsion's own
# default battery count (1 battery-electric, 0 gas-fuelled); a given int is used as before.
CHANGED_FIELD_TYPES = {("core.aft_arrangement", "AftArrangementInputs", "n_battery_units"):
                       "typing.Optional[int]"}
# Dataclass fields whose type changed on purpose (issue #94): the Vessel Class catalogue does not give
# every figure for every class, so a length, beam, draught or deadweight can be None.
OPTIONAL_NOW = {("core.vessel_standards", "VesselStandard", f) for f in ("loa_m", "beam_m", "draught_m", "dwt_t")}


def test_the_name_list_covers_every_public_module():
    assert sorted(NAMES) == sorted(m.removeprefix("vessel_designer.") for m in json.loads(DATA.read_text(encoding="utf-8")) if m != "vessel_designer.calibration")
    assert sum(len(v) for v in NAMES.values()) > 300


def test_every_package_name_importable_in_0_1_0_still_imports():
    lost = []
    for module, names in NAMES.items():
        mod = importlib.import_module(f"vessel_designer.{module}")
        for name, entry in names.items():
            if entry["kind"] != "package":
                continue
            if not hasattr(mod, name):
                if name not in REMOVED_ON_PURPOSE.get(module, ()):
                    lost.append(f"{module}.{name}")
            elif "value" in entry and entry["value"] != json.loads(json.dumps(getattr(mod, name))):
                if name not in CHANGED_ON_PURPOSE.get(module, ()):
                    lost.append(f"{module}.{name} has another value")
    assert lost == []
    # The list of removals is not a way to hide a loss: each name on it really is gone.
    for module, gone in REMOVED_ON_PURPOSE.items():
        mod = importlib.import_module(f"vessel_designer.{module}")
        assert not [n for n in gone if hasattr(mod, n)]


def test_the_names_that_are_not_kept_are_only_standard_library_and_numpy_helpers():
    missing = set()
    for module, names in NAMES.items():
        mod = importlib.import_module(f"vessel_designer.{module}")
        missing |= {n for n, e in names.items() if e["kind"] == "incidental" and not hasattr(mod, n)}
    assert missing <= NOT_KEPT, sorted(missing - NOT_KEPT)
    changelog = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
    assert all(f"`{n}`" in changelog for n in NOT_KEPT)


def test_every_name_removed_on_purpose_is_in_the_changelog():
    changelog = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
    names = {n for gone in REMOVED_ON_PURPOSE.values() for n in gone}
    names |= {f for _, _, f in REMOVED_FIELDS}
    assert len(names) == 18
    assert [n for n in sorted(names) if f"`{n}`" not in changelog] == []


def test_a_restored_name_is_the_object_of_its_defining_module():
    from vessel_designer import simulate
    from vessel_designer.core import barge_capacity, design_grid, electric_energy, gas_arrangement
    from vessel_designer.core import hull, hydrostatics, journey_derating, protective_location, resistance, tanks
    assert simulate.BargeCapacityInputs is barge_capacity.BargeCapacityInputs
    assert simulate.gas_fit is gas_arrangement.fit and simulate.capacity is barge_capacity.compute
    assert barge_capacity.HullGeometry is hull.HullGeometry is journey_derating.HullGeometry
    assert gas_arrangement.derate is journey_derating.compute
    assert design_grid.BargeCapacityInputs is barge_capacity.BargeCapacityInputs
    assert hydrostatics.free_surface_moment_cylindrical is tanks.free_surface_moment_cylindrical
    assert electric_energy.R is resistance


def test_field_types_are_the_types_of_0_1_0():
    # The snapshot above compares field names. This compares their types, as text.
    import typing
    expected = json.loads((Path(__file__).parent / "data" / "importable_names_0_1_0.json").read_text(encoding="utf-8"))["field_types"]
    assert sum(len(c) for c in expected.values()) > 30, "scanning too little"
    wrong = []
    for module, classes in expected.items():
        mod = importlib.import_module(f"vessel_designer.{module}")
        for cls_name, fields in classes.items():
            hints = typing.get_type_hints(getattr(mod, cls_name))
            for name, text in fields.items():
                if (module, cls_name, name) in REMOVED_FIELDS:
                    assert name not in hints
                    continue
                if (module, cls_name, name) in OPTIONAL_NOW:
                    assert str(hints[name]).startswith("typing.Optional["), name
                    inner = "float" if name == "beam_m" else "typing.Tuple[float, float]"
                    assert inner in str(hints[name]), name
                    continue
                # Known change (CHANGELOG.md): a bare `list` of 0.1.0 is `List[...]` now, because the
                # bridge reads the item type to build the result objects.
                refined = text == "<class 'list'>" and str(hints[name]).startswith("typing.List[")
                changed = CHANGED_FIELD_TYPES.get((module, cls_name, name)) == str(hints[name])
                if str(hints[name]) != text and not refined and not changed:
                    wrong.append(f"{module}.{cls_name}.{name}: {hints[name]} != {text}")
    assert wrong == []
