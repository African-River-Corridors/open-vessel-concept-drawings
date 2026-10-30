"""The one place the evaluation harness calls the engine (issues #184 and #278).

`liquid_trace` runs `gas_stability.liquid_trace` (issue #338): the Rule Trace of a liquid bulk layout under
the selected Rule Sets. `box_condition` runs the Rust core's `gas_stability.box_condition` through the same bridge every
Python function of the package uses. `box_trace` runs `gas_stability.box_trace`: the Rule Trace of
the same box loading condition under the selected Rule Sets. Neither is part of the package's
public API: the harness is the only caller.
"""
from __future__ import annotations

from ovd._bridge import _raise, call, raw
from ovd.core.gas_stability import Condition

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


def stern_trace(inputs: dict, measured: dict, *, rule_sets: tuple[str, ...] = (),
                select: tuple[str, ...] = ()) -> dict:
    """The Rule Trace of a self-propelled Option (`kind: stern`, issue #355): `inputs` are its typed
    inputs, `measured` the numbers a run reads from its results (the Blind Distance in each condition,
    the speed in deep water in km/h, the Design Speed in kn, and the three propeller ratios). `rule_sets`
    and `select` are as for `box_trace`. The trace is the JSON a run writes."""
    out = raw("gas_stability", "stern_trace", dict(
        inputs=dict(inputs), measured={k: float(v) for k, v in measured.items()},
        rule_sets=list(rule_sets), select=list(select)))
    if "error" in out:
        _raise(out["error"])
    return out["ok"]


#: The limits the harness reads a propeller case against, open then ducted: the shaft immersion over the
#: diameter (Beveridge 1971) and the blade load in kW per m2 of disc (Veth). They are the harness's own copy,
#: so a change of the engine's number shows as a verdict that differs.
IMMERSION = (0.76, 0.71)
BLADE_LOAD = (400.0, 500.0)


def stern_case_trace(i: dict, *, rule_sets: tuple[str, ...] = (), select: tuple[str, ...] = ()) -> dict:
    """The Rule Trace of a `kind: stern` case: the typed inputs of a self-propelled craft, and the measured values
    a run reads from its results. The propeller numbers of the case (diameter, the largest the stern allows,
    the lightest aft draught, the blade load) are turned into the ratios the Measures give."""
    ducted = bool(i["propeller_ducted"])
    k = 1 if ducted else 0
    d = float(i["propeller_diameter_m"])
    inputs = {
        "cargo": "LPG_propane", "hull_form": "raked_box", "formation": {"kind": "self_propelled"},
        "length_overall_m": float(i["length_overall_m"]), "beam_m": float(i["beam_m"]),
        "moulded_depth_m": float(i["depth_m"]), "bow_thruster": i["bow_thruster"],
        "propulsor_count": int(i["propulsor_count"]),
        "propulsor_type": "shaft_nozzle" if ducted else "shaft_open", "stern_form": "transom",
    }
    measured = {
        "blind_distance_light_m": i["blind_distance_light_m"], "blind_distance_laden_m": i["blind_distance_laden_m"],
        "speed_deep_water_kmh": i["speed_deep_water_kmh"], "design_speed_kn": i["design_speed_kn"],
        "diameter_over_max": d / float(i["propeller_d_max_m"]),
        "immersion_over_limit": (float(i["aft_draught_light_m"]) - d / 2.0) / d / IMMERSION[k],
        "blade_load_over_limit": float(i["blade_load_kw_m2"]) / BLADE_LOAD[k],
    }
    return stern_trace(inputs, measured, rule_sets=rule_sets, select=select)
