"""Best gas-tank arrangement for a hull: search, don't trust the single AUTO answer.

Rebuilt natively from an earlier project wrapper, fixing three defects (numbered 1, 3, 4
in the original defect list) at the engine boundary. Only IGC rules apply to the gas tanks:
``IGC_2G`` and ``IGC_2G_EXACT`` (the ``ADN_G`` ruleset was removed in issue #175):

1. ``barge_capacity``'s clearance loop has two self-consistent roots at some beams and its
   seed can land on the worse one (CN II at 14.8 m: AUTO 2 abreast ~1,156 t against
   1 abreast 2,058 t). So every tanks-abreast count is forced and converged on its own.
3. The engine fills the beam with the largest tank allowed. Under a draught cap more tank is
   more steel, and steel displaces cargo, so smaller-tank variants (extra side clearance,
   never below the code minimum) are also tried. A tank over the OD cap is re-tried ON the cap
   with the clearance widened -- and REJECTED if that clearance is below the rule for its volume.
4. Two tanks lie along the hull. Every tank must keep to the outer diameter cap (``max_tank_od_m``,
   16 m by default) and to length over diameter at most 7; ``barge_capacity`` checks both, and a
   hull whose tanks are too long for that has no feasible arrangement (``feasible=False``).

Moulded depth is not set by the waterway, so it is swept and the depth that carries the most
per journey at the draught cap is kept -- among the candidates that PASS their error-severity
checks. A candidate that carries more but is overloaded or has no freeboard never wins. If no
candidate passes, the best-tonnage one is returned with ``feasible=False`` and its failed checks
listed (the module's convention: report through checks, don't raise). Every setting is an explicit argument -- no module
globals (the wrapper's ``use_settings()`` was a stop-gap).

Pure calculation, no I/O.
"""
from __future__ import annotations

import inspect
from dataclasses import dataclass, field
from typing import List, Sequence

from .barge_capacity import DEFAULT_MAX_TANK_OD_M, BargeCapacityInputs, _result_from_json
from .. import _bridge
from .._bridge import call as _call

# 0.1.0 modules exposed these names (they imported them), and user scripts import them from here.
# Nothing in this module uses them: the calculation runs in the Rust core.
from .barge_capacity import compute as capacity  # noqa: F401
from .checks import all_ok  # noqa: F401
from .journey_derating import JourneyDeratingInputs, compute as derate  # noqa: F401
from .protective_location import (  # noqa: F401
    compute_required_clearances, d_vc_exact, distance_d_vc)

RULESETS = ("IGC_2G", "IGC_2G_EXACT")
DEFAULT_DEPTHS_M = tuple(round(3.5 + 0.25 * i, 2) for i in range(15))     # 3.5 .. 7.0 m
DEFAULT_TANKS_WIDE = (1, 2, 3)
DEFAULT_EXTRA_CLEARANCE_M = (0.5, 1.0, 1.5, 2.0)


@dataclass
class ArrangementFit:
    loa_m: float
    beam_m: float
    depth_m: float
    cargo: str
    ruleset: str
    fill_ratio: float
    self_propelled: bool
    t_max_m: float
    payload_t: float              # per journey, after the draught cap
    volume_capacity_t: float
    binding: str                  # "volume" | "draft" | "no cargo area"
    draft_at_payload_m: float
    cargo_zone_m: float
    stern_void_m: float
    bow_void_m: float
    tank_od_m: float
    n_tanks: int
    n_tanks_wide: int
    n_tanks_long: int
    lightship_t: float
    air_draft_light_m: float
    side_clearance_m: float = 0.0
    tank_inner_diameter_m: float = 0.0
    containment_inner_volume_m3: float = 0.0
    per_tank_volume_m3: float = 0.0
    cargo_volume_m3: float = 0.0
    density_t_m3: float = 0.0
    failed_checks: List[str] = field(default_factory=list)
    feasible: bool = True         # False: no candidate passed its error checks; see failed_checks
    candidates_tried: int = 0
    # The engine's own objects, for the GA generator. Not serialised.
    engine_inputs: object = field(default=None, repr=False, compare=False)
    engine_result: object = field(default=None, repr=False, compare=False)


def required_side_clearance(ruleset: str, beam: float, total_m3: float, per_tank_m3: float,
                            insulation_mm: float) -> float:
    """The code minimum for the engine's clearance datum (outer face of the insulation)."""
    return _call("gas_arrangement", "required_side_clearance", dict(
        ruleset=ruleset, beam=beam, total_m3=total_m3, per_tank_m3=per_tank_m3,
        insulation_mm=insulation_mm), float)


def fit(loa: float, beam: float, cargo: str, t_max: float, *, ruleset: str = "IGC_2G",
        fill_ratio: float = 0.95, self_propelled: bool = True,
        depths: Sequence[float] = DEFAULT_DEPTHS_M, tanks_wide: Sequence[int] = DEFAULT_TANKS_WIDE,
        extra_clearance_m: Sequence[float] = DEFAULT_EXTRA_CLEARANCE_M,
        block_coefficient: float = 0.85,
        max_tank_od_m: float = DEFAULT_MAX_TANK_OD_M) -> ArrangementFit:
    """Most cargo per journey at draught cap ``t_max`` over depth x arrangement.

    Only candidates whose error-severity checks all pass can win. If none passes, the
    best-tonnage candidate comes back with ``feasible=False``; check it before using the payload.
    """
    data = _bridge.raw("gas_arrangement", "fit", _bridge.encode_args(dict(
        loa=loa, beam=beam, cargo=cargo, t_max=t_max, ruleset=ruleset, fill_ratio=fill_ratio,
        self_propelled=self_propelled, depths=list(depths), tanks_wide=list(tanks_wide),
        extra_clearance_m=list(extra_clearance_m),
        block_coefficient=block_coefficient, max_tank_od_m=max_tank_od_m)))
    if "error" in data:
        _bridge._raise(data["error"])
    ok = data["ok"]
    f = _bridge.decode(ArrangementFit, ok)
    f.engine_inputs = _bridge.decode(BargeCapacityInputs, ok["engine_inputs"])
    f.engine_result = _result_from_json(ok["engine_result"])
    return f


_FIT_KEYWORDS = frozenset(inspect.signature(fit).parameters) - {"loa", "beam", "cargo", "t_max"}


def smallest_viable_loa(beam: float, cargo: str, t_max: float, *, lo: float = 20.0,
                        hi: float = 120.0, **kw) -> float:
    """Shortest LOA (to 0.1 m) at which the hull has any cargo area at all."""
    for name in kw:
        # What 0.1.0 said when it passed the keywords on to `fit`.
        if name in ("loa", "beam", "cargo", "t_max"):
            raise TypeError(f"fit() got multiple values for argument '{name}'")
        if name not in _FIT_KEYWORDS:
            raise TypeError(f"fit() got an unexpected keyword argument '{name}'")
    args = dict(beam=beam, cargo=cargo, t_max=t_max, lo=lo, hi=hi, **kw)
    return _call("gas_arrangement", "smallest_viable_loa", args, float)
