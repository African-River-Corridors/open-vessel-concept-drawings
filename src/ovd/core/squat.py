"""Squat — how much deeper a moving vessel sits than a stationary one.

WHY THIS MODULE EXISTS. `resistance.py` computed a service speed from resistance alone and
returned 9.2 knots for the laden CN II motor in an example 4.80 m channel. At that speed the
vessel squats about 1.7 m into an under-keel clearance of 0.40 m. It would be aground long
before it was underpowered. Squat, not power, is what limits speed in a shallow canalised
river, and the engine had no model of it at all.

The effect is strongly non-linear — roughly V^2 — so it does not merely trim the answer, it
halves it. Between a 4.80 m channel (UKC 0.40 m) and a 5.28 m one (UKC 0.88 m) the laden
service speed goes from under 4 knots to about 6.5.

TWO METHODS, BOTH EMPIRICAL, AND THE DIFFERENCE IS REPORTED
-----------------------------------------------------------
There is no closed-form squat. Both of these are fits to model and full-scale data, and they
disagree by tens of percent, which is itself worth seeing on a sheet.

**Barrass II** (Barrass, 1979 and later; the form quoted in his *Ship Squat* work)::

    S_max = C_b * S^0.81 * V_k^2.08 / 20        [m]

with ``S`` the blockage ratio (midship area / channel section area) and ``V_k`` the speed
THROUGH WATER in knots. Barrass gives this for confined and restricted channels and it is the
one usually reached for in inland work. It is dimensionally inhomogeneous — the constants
carry the units — which is normal for this class of formula and a reason to treat it as an
estimate, not a calculation.

**ICORELS / PIANC** for the bow of a slender ship::

    S_max = 2.4 * (Vol / L_pp^2) * F_nh^2 / sqrt(1 - F_nh^2)

with ``F_nh = V / sqrt(g*h)`` the depth Froude number. This is the form PIANC's channel-design
guidance uses. It blows up as ``F_nh -> 1`` — correctly, because that is the critical speed —
so it is only meaningful below about 0.7 and the result says so.

WHAT THIS DOES NOT DO
---------------------
* No trim: it returns MAXIMUM squat, not bow-down or stern-down separately. For a full-bodied
  barge (C_b 0.85) maximum squat is at the bow; for a fine hull it can be at the stern.
* No passing-vessel or bank-suction effect, both of which add squat in a narrow channel.
* No wave-induced or wind-induced motion. In a river that is right; at sea it is not, and this
  module should not be used for the sea leg.
* No dynamic heel from turning, which in a bend adds draught on the outboard bilge.

Typed dataclasses and functions; the maths runs in the Rust core (`ovd._native`). No I/O.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import List, Optional

from .checks import Check, all_ok
from .._bridge import call as _call

GRAVITY = 9.80665

#: Under-keel clearance we insist remains BELOW the squatted keel. Not a standard's number:
#: a working margin for survey error, siltation between dredging campaigns, and the fact that
#: squat itself is an estimate. PIANC's own net-UKC allowances for a river are of this order.
DEFAULT_SAFETY_MARGIN_M = 0.15

#: Above this depth Froude number the ICORELS form is no longer meaningful and the flow is
#: approaching critical. Reported as a failed check rather than silently extrapolated.
FNH_VALID_MAX = 0.70


@dataclass(frozen=True)
class SquatInputs:
    """One vessel, one channel section, one speed."""

    speed_through_water_kn: float
    draft_m: float
    beam_m: float
    lpp_m: float
    block_coefficient: float = 0.85
    water_depth_m: float = 0.0            # bed to still-water surface
    channel_area_m2: float = 0.0          # wetted section of the CHANNEL; 0 = open water
    displacement_m3: float = 0.0          # 0 -> estimated from Cb x L x B x T
    safety_margin_m: float = DEFAULT_SAFETY_MARGIN_M

    def __post_init__(self):
        if self.speed_through_water_kn < 0:
            raise ValueError("speed_through_water_kn must be >= 0")
        for name in ("draft_m", "beam_m", "lpp_m"):
            if getattr(self, name) <= 0:
                raise ValueError(f"{name} must be > 0")
        if not 0.3 <= self.block_coefficient <= 1.0:
            raise ValueError("block_coefficient outside 0.3-1.0")


@dataclass
class SquatResult:
    method: str
    squat_m: float                  # the adopted (larger) estimate
    squat_barrass_m: float
    squat_icorels_m: Optional[float]
    blockage_ratio: float
    froude_depth: float
    static_ukc_m: float             # water depth - static draft
    dynamic_ukc_m: float            # what is left under the keel at speed
    grounds: bool
    checks: List[Check] = field(default_factory=list)
    warnings: List[str] = field(default_factory=list)

    @property
    def ok(self) -> bool:
        return all_ok(self.checks)


def compute(inp: SquatInputs) -> SquatResult:
    """Maximum squat by two methods; the LARGER is adopted."""
    return _call("squat", "compute", {"inputs": inp}, SquatResult)


def max_speed_for_ukc(draft_m: float, beam_m: float, lpp_m: float, water_depth_m: float,
                      channel_area_m2: float, block_coefficient: float = 0.85,
                      safety_margin_m: float = DEFAULT_SAFETY_MARGIN_M,
                      v_hi_kn: float = 20.0, tol_kn: float = 0.01) -> float:
    """The greatest speed through water that keeps ``safety_margin_m`` under the keel.

    Bisection on :func:`compute`, so it uses whichever method is governing rather than
    inverting one formula analytically. Returns 0.0 if the vessel cannot float clear at rest.
    """
    return _call("squat", "max_speed_for_ukc", dict(
        draft_m=draft_m, beam_m=beam_m, lpp_m=lpp_m, water_depth_m=water_depth_m,
        channel_area_m2=channel_area_m2, block_coefficient=block_coefficient,
        safety_margin_m=safety_margin_m, v_hi_kn=v_hi_kn, tol_kn=tol_kn), float)
