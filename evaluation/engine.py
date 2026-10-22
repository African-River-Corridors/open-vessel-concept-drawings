"""The one place the evaluation harness calls the engine (issues #184 and #278).

`liquid_trace` runs `gas_stability.liquid_trace` (issue #338): the Rule Trace of a liquid bulk layout under
the selected Rule Sets. `box_condition` runs the Rust core's `gas_stability.box_condition` through the same bridge every
Python function of the package uses. `box_trace` runs `gas_stability.box_trace`: the Rule Trace of
the same box loading condition under the selected Rule Sets. Neither is part of the package's
public API: the harness is the only caller.
"""
from __future__ import annotations

from vessel_designer._bridge import _raise, call, raw
from vessel_designer.core.gas_stability import Condition

#: The sets the harness selects when a run names none.
DEFAULT_SELECT = ("is2008-a@1.0.0",)


def box_condition(beam: float, depth: float, draught: float, kg: float, *,
                  free_surface_m: float = 0.0, criteria_set: str = "IS_CODE_2008") -> Condition:
    """The criteria of a box section for a known KG: the GZ curve on the 1 degree grid, the areas, the
    angle of maximum GZ and the criteria, as `assess` works them out for one loading condition."""
    return call("gas_stability", "box_condition", dict(
        beam=beam, depth=depth, draught=draught, kg=kg, free_surface_m=free_surface_m,
        criteria_set=criteria_set), Condition)


def box_trace(beam: float, depth: float, draught: float, kg: float, *, free_surface_m: float = 0.0,
              rule_sets: tuple[str, ...] = (), select: tuple[str, ...] = DEFAULT_SELECT) -> dict:
    """The Rule Trace of a box loading condition (named "Box"): `rule_sets` are Rule Set texts loaded
    at run time next to the built-in ones, `select` the `id@version` keys in selection order. The
    trace is the JSON a run writes: `rule_sets`, `rows` (one per Rule and object) and `verdict`."""
    out = raw("gas_stability", "box_trace", dict(
        beam=float(beam), depth=float(depth), draught=float(draught), kg=float(kg),
        free_surface_m=float(free_surface_m), rule_sets=list(rule_sets), select=list(select)))
    if "error" in out:
        _raise(out["error"])
    return out["ok"]


def liquid_trace(inputs: dict, *, rule_sets: tuple[str, ...] = (), select: tuple[str, ...] = ()) -> dict:
    """The Rule Trace of a liquid bulk layout case (`kind: tank`): a hull, a cargo zone and a tank layout,
    the deadweight, the Service Area and the ADN Tank Vessel Type. `rule_sets` and `select` are as for
    `box_trace`. The trace is the JSON a run writes."""
    keys = ("loa_m", "beam_m", "depth_m", "zone_aft_m", "zone_fwd_m", "double_side_m", "double_bottom_m",
            "cofferdam_m", "slop_share", "deadweight_t", "pump_room_m")
    args = {k: float(inputs[k]) for k in keys if k in inputs}
    args.update(tanks_in_line=float(inputs["tanks_in_line"]),
                centreline_bulkhead=bool(inputs["centreline_bulkhead"]),
                service_area=inputs["service_area"], adn_vessel_type=inputs["adn_vessel_type"],
                rule_sets=list(rule_sets), select=list(select))
    out = raw("gas_stability", "liquid_trace", args)
    if "error" in out:
        _raise(out["error"])
    return out["ok"]
