"""The bridge from the Python API to the Rust core (`vessel_designer._native`).

Every calculation of the package runs in Rust. A public function here builds the case the Rust
`eval` seam reads (`{"fn": ..., "args": {...}}`), calls it, and turns the JSON result back into the
package's own dataclasses. Import paths, names and result types stay as they were; the arithmetic
does not live in Python any more.

Conventions:

- a whole number given for a float field is sent as a float, as the dataclass declares;
- NaN and infinity travel as the strings "float:nan", "float:inf" and "float:-inf";
- an error the core reports comes back as the Python exception of the same class and message.
"""
from __future__ import annotations

import ast
import dataclasses
import enum
import json
import math
import typing
from functools import lru_cache

from . import _native

_TAGS = {"float:nan": math.nan, "float:inf": math.inf, "float:-inf": -math.inf}


@lru_cache(maxsize=None)
def _hints(cls):
    return typing.get_type_hints(cls)


def _is_float_hint(hint) -> bool:
    if hint is float:
        return True
    if typing.get_origin(hint) is typing.Union:
        return any(a is float for a in typing.get_args(hint))
    return False


def encode(value, hint=None):
    """Plain JSON data for a value: dataclasses become objects, enums their value."""
    if dataclasses.is_dataclass(value) and not isinstance(value, type):
        hints = _hints(type(value))
        return {f.name: encode(getattr(value, f.name), hints.get(f.name))
                for f in dataclasses.fields(value)}
    if isinstance(value, enum.Enum):
        return value.value
    if isinstance(value, bool):
        return value
    if isinstance(value, int):
        return float(value) if _is_float_hint(hint) else value
    if isinstance(value, float):
        if math.isnan(value):
            return "float:nan"
        if math.isinf(value):
            return "float:inf" if value > 0 else "float:-inf"
        return value
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
    return {k: (float(v) if type(v) is int and k not in int_ok else encode(v))
            for k, v in args.items()}


def call(entry_point: str, fn: str, args: dict, result=None, int_ok=()):
    """Run one function of the core. `args` may hold dataclasses; `result` is the type hint of
    the answer (a dataclass, `list[...]`, a float, ...). Raises the core's Python exception."""
    out = raw(entry_point, fn, encode_args(args, int_ok))
    if "error" in out:
        _raise(out["error"])
    return decode(result, out["ok"])
