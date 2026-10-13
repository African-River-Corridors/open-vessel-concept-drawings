"""Run the parity cases through the frozen Python and write the Reference Results.

    python3 parity/driver.py <out-dir>

Only `cargo xtask regen` (and the oracle guard's reproduction check) should run this. It reads every
`parity/cases/<entry_point>.json`, calls the frozen Python (`vessel_designer_frozen`, never the
shipped package) for each case, and writes `<out-dir>/<entry_point>.json`.

Output rules (spec 0001):
- floats are written as shortest round-trip text (Python's repr);
- NaN and infinity are written as the tagged strings "float:nan", "float:inf" and "float:-inf";
- a raised ValueError, ZeroDivisionError or OverflowError is recorded as
  {"error": {"type": "<class name>", "message": ...}}; a KeyError the
  frozen Python raises on purpose is recorded the same way, but only where an adapter wraps the frozen
  call in `frozen_key_error` (a KeyError from the driver itself, such as a mistyped case, fails regen);
- a normal return is recorded as {"ok": <value>}.
Standard library only (plus whatever the frozen module itself imports).
"""
from __future__ import annotations

import dataclasses
import enum
import importlib
import json
import math
import sys
import typing
from pathlib import Path

PARITY = Path(__file__).resolve().parent
sys.path.insert(0, str(PARITY / "frozen_python"))

FROZEN = "vessel_designer_frozen"


def tag_float(x: float):
    if math.isnan(x):
        return "float:nan"
    if math.isinf(x):
        return "float:inf" if x > 0 else "float:-inf"
    return x


def to_json(value):
    """Convert a Python result into plain JSON values, keeping dict and field order."""
    if isinstance(value, bool) or value is None or isinstance(value, (int, str)):
        return value
    if isinstance(value, float):
        return tag_float(value)
    if isinstance(value, enum.Enum):
        return to_json(value.value)
    if dataclasses.is_dataclass(value) and not isinstance(value, type):
        return {f.name: to_json(getattr(value, f.name)) for f in dataclasses.fields(value)}
    if isinstance(value, dict):
        return {str(k): to_json(v) for k, v in value.items()}
    if isinstance(value, (list, tuple)):
        return [to_json(v) for v in value]
    raise TypeError(f"no JSON form for {type(value).__name__}")


class FrozenKeyError(Exception):
    """A KeyError the frozen Python raised on purpose, to be recorded as a Reference Result."""

    def __init__(self, exc: KeyError):
        super().__init__(str(exc))
        self.exc = exc


class frozen_key_error:
    """Context manager: wrap ONLY the frozen call, so driver KeyErrors still fail regen."""

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb):
        if exc_type is not None and issubclass(exc_type, KeyError):
            raise FrozenKeyError(exc) from exc
        return False


def _module(name: str):
    return importlib.import_module(f"{FROZEN}.{name}")


def build(cls, data):
    """Build a (nested) dataclass from plain JSON data, using its type hints."""
    if data is None or not isinstance(data, dict):
        return data
    hints = typing.get_type_hints(cls)
    kwargs = {}
    for key, value in data.items():
        hint = hints[key]
        kwargs[key] = _coerce(hint, value)
    return cls(**kwargs)


def _coerce(hint, value):
    if value is None:
        return None
    origin = typing.get_origin(hint)
    if origin is typing.Union:
        inner = [a for a in typing.get_args(hint) if a is not type(None)]
        return _coerce(inner[0], value) if len(inner) == 1 else value
    if origin in (list, typing.List):
        (item,) = typing.get_args(hint)
        return [_coerce(item, v) for v in value]
    if dataclasses.is_dataclass(hint) and isinstance(value, dict):
        return build(hint, value)
    if hint is float and isinstance(value, int) and not isinstance(value, bool):
        # JSON `12` and `12.0` are the same number to the Rust side, which always holds an f64.
        # Cast here so the frozen Python also holds a float and echoes `12.0`, not `12`.
        return float(value)
    if hint is bool and not isinstance(value, bool):
        # Rust rejects a non-boolean here as BadInput; Python would accept a truthy value.
        raise TypeError(f"a bool field was given {value!r}; write true or false")
    return value


# One adapter per entry point: (function name, args) -> Python result.
def cargo(fn: str, args: dict):
    mod = _module("core.cargo")
    if fn == "get_cargo":
        return mod.get_cargo(**args)
    if fn == "cargo_database":
        return [{"key": k, "cargo": v} for k, v in mod.CARGO_DATABASE.items()]
    raise KeyError(f"cargo has no parity function {fn!r}")


def protective_location(fn: str, args: dict):
    mod = _module("core.protective_location")
    if fn in ("distance_d_vc", "d_vc_exact", "compute_required_clearances",
              "check_protective_location"):
        return getattr(mod, fn)(**args)
    if fn in ("required_side_protection", "required_bottom_protection"):
        # vessel_type is the VesselType value (a string); null = None. An unknown string raises
        # the enum's own ValueError, as compute_required_clearances does.
        args = dict(args)
        vt = args.pop("vessel_type", None)
        return getattr(mod, fn)(vessel_type=None if vt is None else mod.VesselType(vt), **args)
    raise KeyError(f"protective_location has no parity function {fn!r}")


def pressure_vessel(fn: str, args: dict):
    mod = _module("core.pressure_vessel")
    if fn in ("get_tank_material", "cylinder_wall_thickness", "hemisphere_head_thickness",
              "ellipsoidal_head_thickness", "calculate_wall_thicknesses"):
        return getattr(mod, fn)(**args)
    raise KeyError(f"pressure_vessel has no parity function {fn!r}")


HULL_QUANTITIES = ("displaced_volume", "waterplane_area", "waterplane_length",
                   "waterplane_inertia_transverse", "waterplane_inertia_longitudinal",
                   "KB", "LCB", "BM_transverse", "BM_longitudinal")
HULL_AREAS = ("bottom_area", "side_area", "deck_area", "bow_rake_area", "stern_rake_area",
              "total_hull_surface_area")


def hull(fn: str, args: dict):
    mod = _module("core.hull")
    if fn == "quantities":
        # Every public quantity of one HullGeometry: the draft-dependent ones at args.draft,
        # then the plating areas. Key order is fixed and part of the Reference Result.
        h = mod.HullGeometry(**args["hull"])
        out = {"parallel_length": h.parallel_length}
        out.update({q: getattr(h, q)(args["draft"]) for q in HULL_QUANTITIES})
        out.update({q: getattr(h, q)() for q in HULL_AREAS})
        return out
    raise KeyError(f"hull has no parity function {fn!r}")


def _barge_chain(inputs: dict):
    """(BargeInputs, HullGeometry, TankResult, WeightBreakdown) from plain BargeInputs data.

    The hull carries the inputs' own rakes; tanks and weights come from the frozen functions.
    Used by the tanks, weights and hydrostatics entry points.
    """
    models, hull_mod = _module("core.models"), _module("core.hull")
    bi = build(models.BargeInputs, inputs)
    h = hull_mod.HullGeometry(bi.loa, bi.beam, bi.depth, bi.bow_rake_length, bi.bow_rake_rise,
                              bi.stern_rake_length, bi.stern_rake_rise)
    t = _module("core.tanks").calculate_tank_arrangement(bi)
    w = _module("core.weights").calculate_weights(bi, h, t)
    return bi, h, t, w


def tanks(fn: str, args: dict):
    mod = _module("core.tanks")
    if fn == "calculate_tank_arrangement":
        return mod.calculate_tank_arrangement(build(_module("core.models").BargeInputs,
                                                    args["inputs"]))
    if fn == "free_surface_moment_cylindrical":
        return mod.free_surface_moment_cylindrical(**args)
    raise KeyError(f"tanks has no parity function {fn!r}")


def weights(fn: str, args: dict):
    if fn == "calculate_weights":
        return _barge_chain(args["inputs"])[3]
    raise KeyError(f"weights has no parity function {fn!r}")


def hydrostatics(fn: str, args: dict):
    mod = _module("core.hydrostatics")
    if fn == "find_equilibrium_draft":
        args = dict(args)
        h = _module("core.hull").HullGeometry(**args.pop("hull"))
        return mod.find_equilibrium_draft(h, **args)
    if fn in ("calculate_stability", "calculate_trim"):
        args = dict(args)
        bi, h, t, w = _barge_chain(args.pop("inputs"))
        return getattr(mod, fn)(h, bi, t, w, **args)
    raise KeyError(f"hydrostatics has no parity function {fn!r}")


def aft_arrangement(fn: str, args: dict):
    mod = _module("core.aft_arrangement")
    if fn == "compute":
        return mod.compute(build(mod.AftArrangementInputs, args["inputs"]))
    raise KeyError(f"aft_arrangement has no parity function {fn!r}")


def gas_separation(fn: str, args: dict):
    mod = _module("core.gas_separation")
    if fn == "compute":
        return mod.compute(build(mod.GasSeparationInputs, args["inputs"]))
    raise KeyError(f"gas_separation has no parity function {fn!r}")


def barge_capacity(fn: str, args: dict):
    mod = _module("core.barge_capacity")
    if fn == "compute":
        return mod.compute(build(mod.BargeCapacityInputs, args["inputs"]))
    if fn == "tanks_wide_for_beam":
        return mod.tanks_wide_for_beam(**args)
    raise KeyError(f"barge_capacity has no parity function {fn!r}")


def journey_derating(fn: str, args: dict):
    mod = _module("core.journey_derating")
    if fn == "compute":
        cap = _module("core.barge_capacity")
        capacity = cap.compute(build(cap.BargeCapacityInputs, args["capacity_inputs"]))
        return mod.compute(mod.JourneyDeratingInputs(capacity=capacity, **args["inputs"]))
    raise KeyError(f"journey_derating has no parity function {fn!r}")


def gas_arrangement(fn: str, args: dict):
    mod = _module("core.gas_arrangement")
    if fn in ("fit", "smallest_viable_loa", "required_side_clearance"):
        return getattr(mod, fn)(**args)
    raise KeyError(f"gas_arrangement has no parity function {fn!r}")


def gas_stability(fn: str, args: dict):
    mod = _module("core.gas_stability")
    if fn in ("box_gz", "segment_fill", "criteria_set_for"):
        return getattr(mod, fn)(**args)
    if fn == "assess":
        # The result to assess comes from a gas fit ("fit") or a capacity run ("capacity").
        args = dict(args)
        if "fit" in args:
            f = _module("core.gas_arrangement").fit(**args.pop("fit"))
            result, payload = f.engine_result, args.pop("payload_t", f.payload_t)
        else:
            cap = _module("core.barge_capacity")
            result = cap.compute(build(cap.BargeCapacityInputs, args.pop("capacity")))
            payload = args.pop("payload_t")
        return mod.assess(result, payload, **args)
    raise KeyError(f"gas_stability has no parity function {fn!r}")


def squat(fn: str, args: dict):
    mod = _module("core.squat")
    if fn == "compute":
        return mod.compute(build(mod.SquatInputs, args["inputs"]))
    if fn == "max_speed_for_ukc":
        return mod.max_speed_for_ukc(**args)
    raise KeyError(f"squat has no parity function {fn!r}")


def waves(fn: str, args: dict):
    mod = _module("core.waves")
    if fn == "compute":
        return mod.compute(build(mod.WaveResistanceInputs, args["inputs"]))
    raise KeyError(f"waves has no parity function {fn!r}")


def resistance(fn: str, args: dict):
    mod = _module("core.resistance")
    if fn == "compute":
        return mod.compute(build(mod.ResistanceInputs, args["inputs"]))
    raise KeyError(f"resistance has no parity function {fn!r}")


def electric_energy(fn: str, args: dict):
    mod = _module("core.electric_energy")
    if fn == "compute":
        return mod.compute(build(mod.EnergyInputs, args["inputs"]))
    raise KeyError(f"electric_energy has no parity function {fn!r}")


def barge_cost(fn: str, args: dict):
    mod = _module("core.barge_cost")
    if fn == "compute":
        # rates: absent or null = the built-in example rates; an object = explicit rates.
        # rates_mapping: a plain key -> value mapping, checked by BargeCostRates.from_mapping
        # (the path a user's rates file takes, with its unknown / missing key errors).
        if "rates_mapping" in args:
            rates = dict(args["rates_mapping"])
        else:
            rates = args.get("rates")
            rates = build(mod.BargeCostRates, rates) if rates is not None else None
        return mod.compute(build(mod.BargeCostInputs, args["inputs"]), rates)
    if fn == "parse_flat_yaml":
        return mod.parse_flat_yaml(**args)
    raise KeyError(f"barge_cost has no parity function {fn!r}")


def design_grid(fn: str, args: dict):
    mod = _module("core.design_grid")
    if fn in ("sweep", "n_tanks_for_length"):
        return getattr(mod, fn)(**args)
    raise KeyError(f"design_grid has no parity function {fn!r}")


def ga_adapter(fn: str, args: dict):
    mod = _module("core.ga_adapter")
    cap = _module("core.barge_capacity")
    inp = build(cap.BargeCapacityInputs, args["inputs"])
    res = cap.compute(inp)
    if fn == "to_ga":
        return mod.to_ga(inp, res, **args.get("kwargs", {}))
    if fn == "is_gas":
        return mod.is_gas(res)
    raise KeyError(f"ga_adapter has no parity function {fn!r}")


def vessel_standards(fn: str, args: dict):
    mod = _module("core.vessel_standards")
    if fn in ("get_standard", "standards_for"):
        with frozen_key_error():
            return getattr(mod, fn)(**args)
    if fn == "validate_catalogue":
        # extra: standards added to the built-in catalogue (id -> fields); null = the built-in one.
        extra = args.get("extra")
        if extra is None:
            return mod.validate_catalogue()
        catalogue = dict(mod.VESSEL_STANDARDS)
        for key, fields in extra.items():
            fields = {k: tuple(v) if isinstance(v, list) else v for k, v in fields.items()}
            catalogue[key] = mod.VesselStandard(**fields)
        return mod.validate_catalogue(catalogue)
    raise KeyError(f"vessel_standards has no parity function {fn!r}")


def simulate(fn: str, args: dict):
    mod = importlib.import_module(f"{FROZEN}.simulate")
    if fn == "simulate":
        return mod.simulate(args["params"])
    if fn == "cargo_options":
        return mod.cargo_options()
    raise KeyError(f"simulate has no parity function {fn!r}")


ADAPTERS = {
    "cargo": cargo,
    "protective_location": protective_location,
    "pressure_vessel": pressure_vessel,
    "hull": hull,
    "aft_arrangement": aft_arrangement,
    "gas_separation": gas_separation,
    "tanks": tanks,
    "weights": weights,
    "hydrostatics": hydrostatics,
    "barge_capacity": barge_capacity,
    "journey_derating": journey_derating,
    "gas_arrangement": gas_arrangement,
    "gas_stability": gas_stability,
    "squat": squat,
    "waves": waves,
    "resistance": resistance,
    "electric_energy": electric_energy,
    "barge_cost": barge_cost,
    "design_grid": design_grid,
    "ga_adapter": ga_adapter,
    "vessel_standards": vessel_standards,
    "simulate": simulate,
}


def run_case(entry_point: str, case_input: dict) -> dict:
    adapter = ADAPTERS[entry_point]
    try:
        result = adapter(case_input["fn"], case_input.get("args", {}))
    except FrozenKeyError as wrapped:
        exc = wrapped.exc
        return {"error": {"type": type(exc).__name__, "message": str(exc)}}
    except (ValueError, ZeroDivisionError, OverflowError) as exc:
        return {"error": {"type": type(exc).__name__, "message": str(exc)}}
    return {"ok": to_json(result)}


def dumps(doc) -> str:
    return json.dumps(doc, indent=2, ensure_ascii=False, allow_nan=False) + "\n"


def main(out_dir: str) -> int:
    out = Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)
    case_files = sorted((PARITY / "cases").glob("*.json"))
    for stale in out.glob("*.json"):
        if stale.name not in {p.name for p in case_files}:
            stale.unlink()
    for path in case_files:
        cases = json.loads(path.read_text(encoding="utf-8"))
        entry_point = cases["entry_point"]
        if entry_point not in ADAPTERS:
            raise SystemExit(f"{path.name}: no driver adapter for entry point {entry_point!r}")
        results = [{"id": c["id"], "output": run_case(entry_point, c["input"])}
                   for c in cases["cases"]]
        doc = {"entry_point": entry_point,
               "generated_by": "cargo xtask regen (frozen Python); never edit by hand",
               "results": results}
        (out / path.name).write_text(dumps(doc), encoding="utf-8", newline="\n")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    sys.exit(main(sys.argv[1]))
