"""Longitudinal separation between the gas cargo area and the rest of the vessel.

WHAT WAS MISSING. ``protective_location.py`` encodes the IGC/IGF PROTECTIVE LOCATION rules,
which are TRANSVERSE (Cs, tank to shell) and VERTICAL (Cb, tank to bottom). Nothing in the
model separated the cargo tanks from the accommodation, the wheelhouse or the machinery
ALONG THE LENGTH: the aft tank head sat directly against the stern block. That is the rule
this module supplies, and it costs cargo length.

THE RULES, and which one governs (checked against the classification
rules for inland-waterways ships carrying dangerous liquids in bulk, Part 4 Ch. 4 Sec. 3,
July 2022 — the inland implementation of ADN Part 9, and consistent with IGC Code 3.1):

  * 3.3.9   Accommodation spaces and the wheelhouse shall be located OUTSIDE the cargo
            area, forward of the fore or abaft the aft vertical plane bounding the part of
            the cargo area below deck.
  * 3.5.3   (type G — liquefied gas) Hold spaces shall be separated from accommodation and
            service spaces outside the cargo area below deck by bulkheads with A-60 fire
            protection insulation. NOTE: for type G this is an insulated BULKHEAD rule, not
            a dimensioned cofferdam. The 0,60 m cofferdam is the type C / type N rule
            (3.6.13, 3.7.4) and IGC 3.1.2 offers the same either/or.
  * 3.3.7   Engine-room entrances not less than 2,00 m from the cargo area.
  * 3.3.11  Entrances and openable windows of superstructures and accommodation spaces
            not less than 2,00 m from the cargo area.
  * 3.5.3 / 3.6.3  Space between the tanks and the end bulkheads not less than 0,50 m,
            reducible to 0,20 m where the tank is a pressure tank.

NOT CHECKED: the inland type G floors of 3.5.2 (side clearance 0,80 m, double bottom 0,60 m) were
removed in issue #215. Gas tanks are sized under IGC only; the side clearance is the IGC d(Vc).

WHAT GOVERNS AFT. The A-60 bulkhead carries no width, so on its own it would take no
length. The binding number is the 2,00 m setback of any openable entrance or window
(3.3.11) and of the engine-room entrance (3.3.7) from the cargo area. A deckhouse whose
front bulkhead is flush with the cargo area cannot have a door or a window in it. So the
DESIGN RULE ADOPTED HERE is a 2,00 m aft separation, built as a 0,60 m cofferdam plus
1,40 m of service space, which satisfies the bulkhead rule, the cofferdam convention and
the opening setback at once. Override it if a designer places the openings elsewhere.

OPEN: the clause numbers above are the classification-society implementation. Confirm them
against the ADN text itself before relying on them for a design.

Typed dataclasses and one function; the calculation runs in the Rust core (`vessel_designer._native`). No I/O.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import List

from .checks import Check, all_ok
from .._bridge import call as _call

# --- Rule constants, each with its clause. Do not retype these anywhere else. ---
COFFERDAM_MIN_M = 0.60          # 3.6.13 / 3.7.4 (type C/N convention; IGC 3.1.2 either/or)
OPENING_SETBACK_M = 2.00        # 3.3.7 + 3.3.11 — entrances/windows from the cargo area
END_BULKHEAD_SPACE_M = 0.50     # 3.5.3, tanks with plane ends
END_BULKHEAD_SPACE_PRESSURE_M = 0.20    # 3.5.3, any other tank end (Type C domed heads); #281


@dataclass
class GasSeparationInputs:
    """Inputs for the longitudinal cargo-area separation."""

    # Issue #280 (maintainer decision, 2026-10-22): the end space defaults to the type G rule of 3.5.3
    # for the tank end shape, 0,20 m for the domed heads of a Type C tank (0,50 m only for plane
    # ends), not the 0,60 m type C cofferdam (0.1.0 used 0,60 m aft and 0,50 m forward).
    cofferdam_m: float = END_BULKHEAD_SPACE_PRESSURE_M
    opening_setback_m: float = OPENING_SETBACK_M
    fwd_end_bulkhead_m: float = END_BULKHEAD_SPACE_PRESSURE_M
    # Kept so older callers still build. No check reads it: the space is keyed to the tank end
    # shape (issue #281), and every Type C tank has domed heads.
    is_pressure_tank: bool = True
    # Kept so older callers still build. No check reads them (the inland type-G floors went in
    # issue #215).
    side_clearance_m: float = 0.0
    double_bottom_m: float = 0.0

    def __post_init__(self):
        if self.cofferdam_m < 0 or self.opening_setback_m < 0 or self.fwd_end_bulkhead_m < 0:
            raise ValueError("separations must be non-negative")


@dataclass
class GasSeparationResult:
    """The length the gas code takes off the cargo area at each end."""

    aft_separation_m: float         # cargo area aft boundary -> front of the stern block
    fwd_separation_m: float         # cargo area fwd boundary -> forward end bulkhead
    cofferdam_m: float
    service_space_m: float          # the remainder of the aft separation, aft of the cofferdam
    total_length_taken_m: float

    checks: List[Check] = field(default_factory=list)
    warnings: List[str] = field(default_factory=list)

    @property
    def ok(self) -> bool:
        return all_ok(self.checks)


def compute(inp: GasSeparationInputs) -> GasSeparationResult:
    """Resolve the aft and forward separations, with the rules as checks."""
    return _call("gas_separation", "compute", {"inputs": inp}, GasSeparationResult)
