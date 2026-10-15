"""Intact stability of a gas barge: GM, the GZ curve and the code criteria (defect 8).

Tall IMO Type C tanks on a shallow hull are exactly where intact stability bites, and the
gas branch had no check at all. This module takes a converged gas arrangement
(``BargeCapacityResult.gas_basis``) and assesses three loading conditions:

  * laden    -- the payload actually carried (after any draught cap), so the tanks may be
                partly filled and their free surface is the real one;
  * half     -- half that payload: the worst free surface for a horizontal cylinder;
  * light    -- lightship, tanks empty: the highest KG relative to displacement.

Method (concept stage, stated on every result):
  * Hull is a prismatic box section (no rakes), the same idealisation the capacity chain uses.
    Draught at a displacement follows Cb x L x B x T x rho, as in ``barge_capacity``.
  * GZ is computed EXACTLY for the heeled box section (polygon clipping), so it stays valid
    past deck-edge immersion and bilge emergence. No downflooding openings are modelled.
  * Cargo free surface: liquid height is solved from the fill VOLUME fraction (circular
    segment), not taken as a height ratio. GZ_fluid = GZ_solid - FSC x sin(phi).
  * KG from the weight split: hull steel at 0.5 D, tanks + insulation + cargo at their own
    heights, piping at deck, outfitting at 0.75 D. These are first-order positions.
  * Not included: the self-propelled stern block's mass (it is not in the gas lightship either),
    wind heel, damage stability. Each is stated as a limitation.

Criteria sets:
  * ``IS_CODE_2008`` -- IMO Intact Stability Code 2008, Part A 2.2 (the general criteria IGC
    ch. 2 relies on). Seagoing; an inland administration may accept less.
  * ``ADN_9_3_1_14_2`` -- ADN 2025 9.3.1.14.2, applies when a cargo tank is wider than 0.70 B:
    GM >= 0.10 m, GZ >= 0.10 m and area >= 0.024 m.rad up to 27 deg. Values as transcribed from
    a secondary research note; confirm against the ADN text before release.

Pure calculation, no I/O.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import List, Optional, Tuple
from .._bridge import call as _call

ANGLES_DEG = tuple(range(0, 61))


# --- geometry helpers -----------------------------------------------------------------------


def box_gz(beam: float, depth: float, draught: float, kg: float, phi_deg: float) -> float:
    """Righting lever (m) of a box section heeled phi, at constant displaced area B x T."""
    return _call("gas_stability", "box_gz", dict(beam=beam, depth=depth, draught=draught, kg=kg, phi_deg=phi_deg), float)


def segment_fill(radius: float, volume_fraction: float):
    """Liquid height h (from tank bottom) and centroid height for a horizontal cylinder
    filled to `volume_fraction` of its cross-section area."""
    return _call("gas_stability", "segment_fill", {"radius": radius, "volume_fraction": volume_fraction}, Tuple[float, float],
             int_ok=("radius",))


# --- results ----------------------------------------------------------------------------

@dataclass
class Criterion:
    name: str
    value: float
    limit: float
    ok: bool
    source: str


@dataclass
class Condition:
    name: str
    cargo_t: float
    fill_fraction: float
    displacement_t: float
    draught_m: float
    kg_m: float
    gm_solid_m: float
    free_surface_m: float
    gm_fluid_m: float
    gz: List[Tuple[int, float]]                          # [(deg, GZ_fluid m)]
    max_gz_m: float
    max_gz_deg: float
    area_0_27: float
    area_0_30: float
    area_0_40: float
    area_30_40: float
    deck_edge_deg: Optional[float]
    criteria: List[Criterion] = field(default_factory=list)

    @property
    def ok(self) -> bool:
        return all(c.ok for c in self.criteria)


@dataclass
class StabilityAssessment:
    criteria_set: str
    conditions: List[Condition]
    limitations: List[str]

    @property
    def ok(self) -> bool:
        return all(c.ok for c in self.conditions)

    @property
    def governing(self) -> Condition:
        return min(self.conditions, key=lambda c: c.gm_fluid_m)


def criteria_set_for(ruleset: str) -> str:
    return _call("gas_stability", "criteria_set_for", {"ruleset": ruleset}, str)


def assess(result, payload_t: float, *, block_coefficient: float = 0.85,
           water_density: float = 1.0, criteria_set: str = "IS_CODE_2008") -> StabilityAssessment:
    """Assess a converged gas arrangement (a ``BargeCapacityResult`` with ``gas_basis``)."""
    return _call("gas_stability", "assess", dict(
        result=result, payload_t=payload_t, block_coefficient=block_coefficient,
        water_density=water_density, criteria_set=criteria_set), StabilityAssessment,
    int_ok=("payload_t",))
