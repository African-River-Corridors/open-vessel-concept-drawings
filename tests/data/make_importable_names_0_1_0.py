"""Write tests/data/importable_names_0_1_0.json: every name a script could import from each public
module of the frozen pure-Python package at commit ce96e66 (what `from ovd.core.x import
NAME` found), with its kind.

    git worktree add /tmp/oracle ce96e66
    python tests/data/make_importable_names_0_1_0.py /tmp/oracle/parity/frozen_python \
        tests/data/importable_names_0_1_0.json

The directory given holds the package `vessel_designer_frozen`. Nobody edits the JSON by hand.

Also `field_types`: the type of every field of every dataclass, as text.

Kinds: "package" is a name of the package (a class, function or module of ovd, or a
constant); "incidental" is a name that came from the standard library, typing or numpy (`math`,
`dataclass`, `Optional`, `np`, ...). The value of a plain constant (a number, text, tuple) is kept.
"""
import dataclasses
import importlib
import json
import pkgutil
import sys
import types
import typing
from pathlib import Path

PLAIN = (bool, int, float, str, tuple)


def kind_of(value) -> str:
    if isinstance(value, types.ModuleType):
        return "package" if value.__name__.startswith("ovd") else "incidental"
    origin = getattr(value, "__module__", None)
    if isinstance(origin, str):
        return "package" if origin.startswith("ovd") else "incidental"
    return "package"


def field_types(mod) -> dict:
    """{class: {field: type as text}} of the dataclasses the module defines. The frozen package name
    is written as `ovd`, so the text compares with the package under test."""
    out = {}
    for name, value in sorted(vars(mod).items()):
        if isinstance(value, type) and dataclasses.is_dataclass(value) and value.__module__ == mod.__name__:
            hints = typing.get_type_hints(value)
            out[name] = {f.name: str(hints[f.name]).replace("vessel_designer_frozen", "ovd")
                         for f in dataclasses.fields(value)}
    return out


def main(frozen_root: str, out: str) -> None:
    sys.path.insert(0, frozen_root)
    import vessel_designer_frozen as pkg

    modules = {}
    fields = {}
    for info in pkgutil.walk_packages(pkg.__path__, "vessel_designer_frozen."):
        short = info.name.split(".", 1)[1]
        if short.split(".")[-1].startswith("_"):
            continue
        mod = importlib.import_module(info.name)
        names = {}
        for name, value in sorted(vars(mod).items()):
            if name.startswith("_"):
                continue
            entry = {"kind": kind_of(value)}
            if isinstance(value, PLAIN) and entry["kind"] == "package":
                entry["value"] = list(value) if isinstance(value, tuple) else value
            names[name] = entry
        modules[short] = names
        if field_types(mod):
            fields[short] = field_types(mod)
    Path(out).write_text(json.dumps(
        {"_about": "Names importable from each public module at commit ce96e66, made by "
                   "tests/data/make_importable_names_0_1_0.py. Do not edit by hand.",
         "modules": modules, "field_types": fields}, indent=1) + "\n", encoding="utf-8")
    print(f"{len(modules)} modules, {sum(len(n) for n in modules.values())} names -> {out}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
