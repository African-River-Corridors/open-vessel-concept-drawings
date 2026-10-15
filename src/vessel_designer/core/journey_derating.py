"""Journey / channel de-rating — where volume-bound meets the channel draft cap.

``barge_capacity`` reports VOLUME-BOUND capacity and the full-load draft, deliberately
ignoring the channel. This module applies the downstream weight/draft limit
(``T_max = design_depth - UKC``) and returns the achievable per-journey tonnage plus the
binding limit ("volume" if it floats fully laden, else "draft").

The weight branch inverts the *same* box-hull hydrostatics ``barge_capacity`` uses
(``HullGeometry`` + block coefficient + water density), so the two modules cannot diverge.

Worked examples: see tests/core/test_journey_derating.py.

Pure Python.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import List

from .barge_capacity import BargeCapacityResult
from .checks import Check, all_ok
from .. import _bridge
from .._bridge import call as _call


@dataclass
class JourneyDeratingInputs:
    """A volume-bound barge design + a channel, to be de-rated to a per-journey load."""

    capacity: BargeCapacityResult     # from barge_capacity.compute()

    # Box-hull geometry — MUST match the hull barge_capacity built (zero rakes), so the
    # weight branch and the full-load draft are consistent by construction.
    loa: float
    beam: float
    depth: float
    block_coefficient: float = 0.85
    water_density: float = 1.0        # t/m^3 (1.0 fresh, 1.025 salt)

    # Channel draft cap, kept as depth + UKC so the bookkeeping is explicit (never pass a
    # pre-reduced T_max — that is the silent double-subtract trap).
    design_depth_m: float = 0.0       # channel design depth below min water level
    ukc_m: float = 0.0                # under-keel clearance already in design_depth

    utilisation_floor: float = 0.5    # advisory: below this, the hull is over-built for the channel

    def __post_init__(self):
        if self.loa <= 0 or self.beam <= 0 or self.depth <= 0:
            raise ValueError("loa, beam, depth must be positive")
        if self.design_depth_m < 0 or self.ukc_m < 0:
            raise ValueError("design_depth_m and ukc_m must be non-negative")


@dataclass
class JourneyDeratingResult:
    cargo_capacity_t: float            # echo: volume-bound capacity
    t_max_m: float                     # channel draft cap used (design_depth - UKC)
    full_load_draft_m: float           # echo from barge_capacity
    achievable_tonnage_t: float        # HEADLINE: per-journey payload after de-rating
    binding_limit: str                 # "volume" | "draft"
    volume_utilisation: float          # achievable / capacity (<= 1.0)
    draft_at_achievable_m: float       # draft actually floated
    shed_tonnage_t: float              # capacity - achievable (0 if volume-bound)
    checks: List[Check] = field(default_factory=list)
    warnings: List[str] = field(default_factory=list)

    @property
    def ok(self) -> bool:
        return all_ok(self.checks)


def compute(inp: JourneyDeratingInputs) -> JourneyDeratingResult:
    """Apply the channel draft cap to a volume-bound barge design."""
    cap = getattr(inp.capacity, "_inputs", None)
    if cap is None:
        raise TypeError("journey_derating needs a capacity result returned by barge_capacity.compute "
                        "or gas_arrangement.fit: the Rust core recomputes it from its inputs")
    args = _bridge.encode_fields(inp, exclude=("capacity",))
    return _call("journey_derating", "compute", {"capacity_inputs": cap, "inputs": args},
                 JourneyDeratingResult)
