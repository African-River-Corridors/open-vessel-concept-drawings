"""Barge design-space sweep: gas cargo vs beam, for a family of barge lengths.

Pre-computes volume-bound gas cargo (Type-C tanks) across a grid of barge length
x beam, so a study can draw the cargo-against-beam curves (one line per LOA)
without any live calculation.

Every cell has two tanks along the hull, as ``gas_arrangement.fit`` has. There is no cap on tank
length (issue #175 removed the 28 m cap): a tank is limited only by its outer diameter cap and by
length over diameter at most 7. A hull too long for two tanks is an infeasible cell, and its
``reason`` says so. ``tank_length_m`` and ``slenderness`` are the engine's own figures.

Cargo is sized on the BARGE length (the tank-bearing hull); total ATB LOA = barge length + tug (16 m for the E-Pusher).

The sweep runs in the Rust core (`vessel_designer._native`), on the same calculation as barge_capacity.compute. No I/O.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import List, Optional

from .._bridge import call as _call
from .barge_capacity import BargeCapacityInputs  # noqa: F401  (0.1.0 exposed it here)

SIDE_CLEARANCE_M = 1.0         # kept for the 0.1.0 API; the grid takes IGC 2.4.1.2 per tank (#283, #277)
BOW_NON_TANK_PCT = 0.10        # mirror models.py defaults
STERN_NON_TANK_PCT = 0.15
TUG_LOA_M = 16.0               # Kotug E-Pusher pattern — for the total-LOA note


@dataclass
class GridCell:
    barge_length_m: float
    beam_m: float
    n_tanks_long: int
    feasible: bool
    cargo_t: Optional[float] = None            # volume-bound (1.0 m default side clearance)
    tank_outer_diameter_m: Optional[float] = None
    tank_length_m: Optional[float] = None
    slenderness: Optional[float] = None         # overall tank length / inner shell diameter
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
