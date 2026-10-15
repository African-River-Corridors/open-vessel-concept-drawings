"""The bridge from the Python API to the Rust core (`vessel_designer._native`).

Every calculation of the package runs in Rust. A public function here builds the case the Rust
`eval` seam reads (`{"fn": ..., "args": {...}}`), calls it, and turns the JSON result back into the
package's own dataclasses. Import paths, names and result types stay as they were; the arithmetic
does not live in Python any more.

Conventions:

- a whole number given for a float field is sent as a float, as the dataclass declares, and a
  whole float given for an int field is sent as an int;
- numpy scalars and arrays are turned into plain Python numbers and lists first;
- NaN and infinity travel as the strings "float:nan", "float:inf" and "float:-inf";
- an error the core reports comes back as the Python exception of the same class and message.
"""
from __future__ import annotations

import ast
import dataclasses
import enum
import json
import math
import numbers
import typing
from functools import lru_cache

from . import _native

_TAGS = {"float:nan": math.nan, "float:inf": math.inf, "float:-inf": -math.inf}


@lru_cache(maxsize=None)
def _hints(cls):
    return typing.get_type_hints(cls)


def _hint_members(hint):
    """The types a hint allows: `Optional[float]` gives (float, NoneType)."""
    if typing.get_origin(hint) is typing.Union:
        return typing.get_args(hint)
    return (hint,)


def _is_float_hint(hint) -> bool:
    return float in _hint_members(hint)


def _is_int_hint(hint) -> bool:
    members = _hint_members(hint)
    return int in members and float not in members


def _plain(value):
    """A Python number, bool or list for a numpy scalar or array (and for any other
    `numbers.Real`); every other value comes back as it is."""
    if isinstance(value, (bool, int, float, str, list, tuple, dict)) or value is None:
        return value
    if type(value).__module__ == "numpy":
        if getattr(value, "shape", ()) == ():
            return value.item()
        return value.tolist()
    if isinstance(value, numbers.Integral):
        return int(value)
    if isinstance(value, numbers.Real):
        return float(value)
    return value


def _number(value, hint):
    """A number cast to the type its hint declares: a whole number given for a float is a float,
    a whole float or a bool given for an int is an int (as 0.1.0 accepted them)."""
    if isinstance(value, float):
        if math.isnan(value):
            return "float:nan"
        if math.isinf(value):
            return "float:inf" if value > 0 else "float:-inf"
        if _is_int_hint(hint) and value.is_integer():
            return int(value)
        return value
    if _is_float_hint(hint):
        return float(value)
    return int(value) if isinstance(value, bool) and _is_int_hint(hint) else value


def encode(value, hint=None):
    """Plain JSON data for a value: dataclasses become objects, enums their value. Numbers are
    cast by the type hint; numpy scalars and arrays become plain Python values."""
    value = _plain(value)
    if dataclasses.is_dataclass(value) and not isinstance(value, type):
        hints = _hints(type(value))
        return {f.name: encode(getattr(value, f.name), hints.get(f.name))
                for f in dataclasses.fields(value)}
    if isinstance(value, enum.Enum):
        return value.value
    if isinstance(value, (bool, int, float)):
        return _number(value, hint)
    if isinstance(value, (list, tuple)):
        item = None
        args = typing.get_args(hint) if hint is not None else ()
        if args and typing.get_origin(hint) in (list, tuple, typing.Sequence) and \
                (len(args) == 1 or (len(args) == 2 and args[1] is Ellipsis)):
            item = args[0]
        return [encode(v, item) for v in value]
    if isinstance(value, dict):
        return {str(k): encode(v) for k, v in value.items()}
    return value


def encode_fields(obj, exclude=()) -> dict:
    """`encode(obj)` without the named fields."""
    hints = _hints(type(obj))
    return {f.name: encode(getattr(obj, f.name), hints.get(f.name))
            for f in dataclasses.fields(obj) if f.name not in exclude}


def decode(hint, data):
    """The Python value for JSON `data`, using the type hint `hint`."""
    if data is None:
        return None
    if isinstance(data, str) and data in _TAGS:
        return _TAGS[data]
    if hint is None or hint is typing.Any or hint is object or hint in (list, dict):
        if isinstance(data, dict):
            return {k: decode(None, v) for k, v in data.items()}
        if isinstance(data, list):
            return [decode(None, v) for v in data]
        return data
    origin = typing.get_origin(hint)
    if origin is typing.Union:
        inner = [a for a in typing.get_args(hint) if a is not type(None)]
        return decode(inner[0], data) if len(inner) == 1 else data
    if origin in (list, typing.List):
        (item,) = typing.get_args(hint) or (None,)
        return [decode(item, v) for v in data]
    if origin in (tuple, typing.Tuple):
        args = typing.get_args(hint)
        if len(args) == 2 and args[1] is Ellipsis:
            return tuple(decode(args[0], v) for v in data)
        return tuple(decode(a, v) for a, v in zip(args, data))
    if origin in (dict, typing.Dict):
        args = typing.get_args(hint)
        return {k: decode(args[1] if len(args) == 2 else None, v) for k, v in data.items()}
    if isinstance(hint, type) and issubclass(hint, enum.Enum):
        return hint(data)
    if dataclasses.is_dataclass(hint) and isinstance(data, dict):
        hints = _hints(hint)
        return hint(**{k: decode(hints.get(k), v) for k, v in data.items()})
    return data


_RAISED = {"ValueError": ValueError, "ZeroDivisionError": ZeroDivisionError,
           "OverflowError": OverflowError, "KeyError": KeyError, "TypeError": TypeError}


def _raise(error: dict):
    cls = _RAISED.get(error["type"], RuntimeError)
    message = error["message"]
    if cls is KeyError and message[:1] in ("'", '"'):
        message = ast.literal_eval(message)   # str(KeyError(x)) is repr(x); give back x
    raise cls(message)


def raw(entry_point: str, fn: str, args: dict):
    """The core's JSON answer (already parsed): {"ok": ...} or {"error": ...}."""
    case = json.dumps({"fn": fn, "args": args}, allow_nan=False)
    return json.loads(_native.eval(entry_point, case))


def encode_args(args: dict, int_ok=()) -> dict:
    """The JSON for a call's arguments. A whole number given for a float argument goes as a float
    (the core reads floats strictly); `int_ok` names the arguments where Python keeps an int."""
    out = {}
    for k, v in args.items():
        v = _plain(v)
        out[k] = (float(v) if type(v) is int and k not in int_ok
                  else encode(v))
    return out


def call(entry_point: str, fn: str, args: dict, result=None, int_ok=()):
    """Run one function of the core. `args` may hold dataclasses; `result` is the type hint of
    the answer (a dataclass, `list[...]`, a float, ...). Raises the core's Python exception."""
    out = raw(entry_point, fn, encode_args(args, int_ok))
    if "error" in out:
        _raise(out["error"])
    return decode(result, out["ok"])
