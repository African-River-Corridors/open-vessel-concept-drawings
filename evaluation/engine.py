"""The one place the evaluation harness calls the engine (issue #184).

`box_condition` runs the Rust core's `gas_stability.box_condition` through the same bridge every
Python function of the package uses. It is not part of the package's public API: the harness is
the only caller.
"""
from __future__ import annotations

from vessel_designer._bridge import call
from vessel_designer.core.gas_stability import Condition


def box_condition(beam: float, depth: float, draught: float, kg: float, *,
                  free_surface_m: float = 0.0, criteria_set: str = "IS_CODE_2008") -> Condition:
    """The criteria of a box section for a known KG: the GZ curve on the 1 degree grid, the areas, the
    angle of maximum GZ and the criteria, as `assess` works them out for one loading condition."""
    return call("gas_stability", "box_condition", dict(
        beam=beam, depth=depth, draught=draught, kg=kg, free_surface_m=free_surface_m,
        criteria_set=criteria_set), Condition)
