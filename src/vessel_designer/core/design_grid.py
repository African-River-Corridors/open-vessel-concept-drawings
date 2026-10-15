"""Barge design-space sweep: gas cargo vs beam, for a family of barge lengths.

Pre-computes volume-bound gas cargo (Type-C tanks) across a grid of barge length
x beam, so a study can draw the cargo-against-beam curves (one line per LOA)
without any live calculation.

Design rule for the tank count (the one free choice as length grows):
  n_tanks_long = max(1, ceil(available_length / max_tank_length))
where available_length = (1 - bow_pct - stern_pct) * barge_length. The count
depends on LENGTH ONLY (not beam), so every fixed-LOA curve is smooth — you only
step up a tank when the hull gets long enough to need one. max_tank_length is a
fabrication/transport/support limit (ESTIMATE, 28 m; an example 60 m barge
carries 2 tanks at ~21.5 m each, comfortably under).

Cargo is sized on the BARGE length (the tank-bearing hull); total ATB LOA = barge length + tug (16 m for the E-Pusher).

The sweep runs in the Rust core (`vessel_designer._native`), on the same calculation as barge_capacity.compute. No I/O.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import List, Optional

from .._bridge import call as _call

MAX_TANK_LENGTH_M = 28.0       # overall Type-C tank length cap — ESTIMATE
SIDE_CLEARANCE_M = 1.0         # standard IGC-2G protective-location spacing per side
BOW_NON_TANK_PCT = 0.10        # mirror models.py defaults
STERN_NON_TANK_PCT = 0.15
TUG_LOA_M = 16.0               # Kotug E-Pusher pattern — for the total-LOA note


def n_tanks_for_length(barge_length_m: float,
                       max_tank_length_m: float = MAX_TANK_LENGTH_M) -> int:
    """Tanks in a line for a given barge length (length-only, so curves stay smooth)."""
    return _call("design_grid", "n_tanks_for_length", {"barge_length_m": barge_length_m, "max_tank_length_m": max_tank_length_m}, int)


@dataclass
class GridCell:
    barge_length_m: float
    beam_m: float
    n_tanks_long: int
    feasible: bool
    cargo_t: Optional[float] = None            # volume-bound (1.0 m standard side clearance)
    tank_outer_diameter_m: Optional[float] = None
    tank_length_m: Optional[float] = None
    slenderness: Optional[float] = None         # overall tank length / OD
    full_load_draft_m: Optional[float] = None
    air_draft_m: Optional[float] = None
    reason: str = ""                            # why infeasible, if so


@dataclass
class DesignGrid:
    barge_lengths_m: List[float]
    beams_m: List[float]
    depth_m: float
    cargo_type: str
    clearance_basis: str
    max_tank_length_m: float
    cells: List[GridCell] = field(default_factory=list)

    def curve(self, barge_length_m: float) -> List[GridCell]:
        """Feasible cells for one barge length, ordered by beam (one plotted line)."""
        return [c for c in self.cells
                if c.barge_length_m == barge_length_m and c.feasible]


def sweep(barge_lengths_m: List[float], beams_m: List[float],
          depth_m: float = 6.0, cargo_type: str = "LPG_propane") -> DesignGrid:
    return _call("design_grid", "sweep", dict(
        barge_lengths_m=list(barge_lengths_m), beams_m=list(beams_m), depth_m=depth_m,
        cargo_type=cargo_type), DesignGrid, int_ok=("depth_m",))
