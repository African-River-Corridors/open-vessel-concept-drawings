"""Unified barge design & volume-bound cargo capacity.

Single source of truth for turning a scenario (LOA, beam, moulded depth, cargo type)
plus design assumptions into a barge design and its three headline outputs:

    - cargo_capacity_t      volume-bound tonnage (tanks/hold full at fill ratio)
    - full_load_draft_m     equilibrium draft carrying that cargo
    - air_draft_m           LADEN air draft: tallest fixed point above keel, minus the
                            full-load draft
    - air_draft_light_m     LIGHT air draft: the same top, minus the LIGHTSHIP draft.
                            This is the GOVERNING case for bridge clearance -- an empty
                            barge floats high, so it is always the larger of the two.
                            Gate on this, never on air_draft_m. Added after a
                            review found every air-draft figure in the model was the
                            favourable laden case.

Handles cryogenic gas (IGC/IGF Type-C tanks) and bauxite (trapezoidal hopper hold).

NOTE — capacity here is VOLUME-BOUND only. The weight/draft limit (channel depth − UKC)
is NOT applied in this module; it is applied downstream at the scenario / channel-design
stage, which re-derives the achievable per-journey tonnage from the actual channel.

Worked examples: see tests/core/test_barge_capacity.py and core/design_grid.py.

Pure Python.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import List, Optional

from .aft_arrangement import (AftArrangementInputs, AftArrangementResult)
from .checks import Check, all_ok
from .gas_separation import (GasSeparationInputs, GasSeparationResult)
from .models import BargeInputs
from .. import _bridge
from .models import TankResult, WeightBreakdown
from .._bridge import call as _call


@dataclass
class BargeCapacityInputs:
    """Inputs for the unified barge capacity calculation."""

    # --- Scenario fundamentals ---
    loa: float                          # length overall (m)
    beam: float                         # moulded beam (m)
    depth: float                        # moulded depth (m) — scenario input
    cargo_type: str = "LNG_ethane"      # key into cargo.py (gas or "bauxite")

    # --- Common assumptions ---
    block_coefficient: float = 0.85
    water_density: float = 1.0          # t/m^3 (1.0 fresh, 1.025 salt)
    required_freeboard: float = 0.3     # m, minimum
    # Bridge/air-draft clearance limit (m). When set, `air_draft_within_limit` checks the
    # LIGHT condition against it -- the empty return leg, not the laden one.
    air_draft_limit_m: Optional[float] = None
    # Ballast the empty leg can carry (t). Reduces the light air draft; 0 = unballasted.
    ballast_capacity_t: float = 0.0

    # --- Arrangement (the stern block and the ends of the cargo area) ---
    # DERIVED, not a fraction of LOA. `aft` describes the equipment that has to fit aft
    # (thrusters, battery, accommodation, wheelhouse); `separation` the gas-code distance
    # between the cargo area and that block. Leave both None for the standard vessel.
    aft: Optional[AftArrangementInputs] = None
    separation: Optional[GasSeparationInputs] = None
    # SELF-PROPELLED (default) carries the stern block -- thrusters, battery,
    # accommodation, wheelhouse. A PUSHED (dumb) barge does not: the crew and propulsion
    # sit on the tug, so the cargo hull keeps that length. Set False for a pushed unit.
    #
    # This is the whole architecture question in one flag, and it is worth 19-20 m of hull.
    # Do NOT set it False for CN II: it is a motor-vessel class, so the class's 90 m
    # already includes machinery and accommodation, and a pusher would deduct that
    # space twice.
    self_propelled: bool = True
    # Aft void of a PUSHED barge (m): stern rake plus the coupling/steering void. ESTIMATE.
    pushed_stern_void_m: float = 3.0
    # Bow bottom rake (m). None -> 1.6 x moulded depth. ESTIMATE: no rake is surveyed or
    # specified for this vessel; it is carried as an input so it can be replaced, not
    # buried.
    bow_rake_length_m: Optional[float] = None
    # Dry-bulk end-bulkhead allowance at each end of the hold (m). The gas branch takes
    # this from `gas_separation` instead.
    bulk_end_bulkhead_m: float = 0.50

    # --- Gas branch (IGC/IGF) ---
    vessel_type: str = "IGC_2G"
    # Optional side-clearance override (m per side). None -> IGC protective-location
    # formula (Cs = d(Vc) for 2G). For the CONSERVATIVE 12%-of-beam simplification,
    # pass 0.12*beam here; the code formula gives ~1.0 m and ~20% more cargo.
    side_clearance_override_m: Optional[float] = None
    # None = AUTO: the fewest tanks across that keeps each tank's OD within
    # MAX_TANK_OD_M. Pass an int to force an arrangement.
    n_tanks_wide: Optional[int] = None
    n_tanks_long: int = 2
    tank_wall_thickness: Optional[float] = 25.0  # mm; None = auto pressure-vessel formula
    fill_ratio: float = 0.95
    dome_height: float = 2.0
    tank_support_height: float = 0.5

    # --- Bulk branch ---
    # Hold type sets the lightship method. "sloped_hopper": the plating build-up below
    # (a sloped-hopper bauxite hold). "box" / "container": a calibrated k x L.B.D for the whole
    # lightship -- calibration pass 1 found the build-up 31-111 % heavy on real
    # box-hold barges. k values come from calibration/factors.yaml via the caller.
    hold_type: str = "sloped_hopper"     # "sloped_hopper" | "box" | "container"
    lightship_k_t_per_m3: Optional[float] = None   # required for "box"/"container"
    wing_void_width: float = 1.0        # fixed wing-void width per side (m)
    double_bottom_height_bulk: float = 1.2
    hopper_slope_h_per_v: float = 1.0   # lower-knuckle slope (~45 deg)
    coaming_height: float = 1.0         # hold extends above deck inside coaming (m)
    n_holds: int = 3
    hold_fill_ratio: float = 0.95
    bow_void_fraction: float = 0.10
    stern_void_fraction: float = 0.15
    # bulk steel build-up (plating thicknesses in mm)
    inner_bottom_t_mm: float = 14.0
    hopper_plate_t_mm: float = 14.0
    bulkhead_t_mm: float = 11.0
    coaming_t_mm: float = 10.0
    steel_density: float = 7.85
    bulk_structure_factor: float = 1.40
    bulk_steel_rate_per_cbm: float = 0.10
    bulk_piping_factor: float = 0.04
    bulk_outfitting_factor: float = 0.035


@dataclass
class BargeCapacityResult:
    """Volume-bound capacity and the resulting full-load hydrostatics."""

    cargo_type: str
    phase: str                          # "liquid_gas" | "dry_bulk"
    cargo_density_t_per_m3: float

    # Headline outputs
    cargo_capacity_t: float             # volume-bound tonnage
    full_load_draft_m: float
    air_draft_m: float

    # Supporting
    cargo_volume_m3: float
    lightship_t: float
    displacement_t: float
    deadweight_t: float
    freeboard_m: float
    air_draft_top_above_keel_m: float
    overloaded: bool                    # True if volume load can't float within moulded depth

    # Gas design detail (None for bulk)
    tank_outer_diameter_m: Optional[float] = None
    n_tanks_wide: Optional[int] = None        # resolved arrangement (auto or forced)
    max_tank_od_m: Optional[float] = None     # the OD cap that resolved it
    n_tanks_total: Optional[int] = None
    side_clearance_m: Optional[float] = None
    bottom_clearance_m: Optional[float] = None

    # Bulk design detail (None for gas)
    hold_height_m: Optional[float] = None
    hold_top_width_m: Optional[float] = None
    hold_bottom_width_m: Optional[float] = None
    hold_section_area_m2: Optional[float] = None

    # Intact stability (bulk hopper only; gas uses hydrostatics.calculate_stability)
    gm_m: Optional[float] = None        # transverse metacentric height GM = KB + BM - KG

    # --- Arrangement (derived, not assumed) ---
    bow_void_length_m: Optional[float] = None    # forward non-cargo length (rake + end bhd)
    stern_void_length_m: Optional[float] = None  # aft non-cargo length (block + separation)
    cargo_zone_length_m: Optional[float] = None  # LOA - the two voids
    bow_rake_length_m: Optional[float] = None
    aft_arrangement: Optional[AftArrangementResult] = None
    separation: Optional[GasSeparationResult] = None

    # --- Cargo volume build-up (so the 95% is visible, not implied) ---
    # Gas: the INNER volume of the tanks, after insulation and steel shell are taken off
    # the outer diameter. Bulk: the geometric hold volume. `cargo_volume_m3` is this
    # figure multiplied by the fill ratio.
    containment_inner_volume_m3: Optional[float] = None
    fill_ratio_applied: Optional[float] = None
    insulation_thickness_mm: Optional[float] = None
    tank_wall_thickness_mm: Optional[float] = None
    tank_inner_diameter_m: Optional[float] = None

    # --- Light / ballast load conditions (bridge clearance governs here) ---
    light_draft_m: Optional[float] = None        # lightship-only equilibrium draft
    air_draft_light_m: Optional[float] = None    # GOVERNING air draft (empty leg)
    ballast_draft_m: Optional[float] = None      # draft with ballast_capacity_t aboard
    air_draft_ballast_m: Optional[float] = None  # air draft on the ballasted empty leg
    # Wheelhouse RAISED. The lifting wheelhouse is lowered to pass a bridge, so it is not
    # part of the transit air draft -- but it is the real height under way, so it is
    # reported rather than dropped.
    air_draft_navigating_light_m: Optional[float] = None
    navigating_top_above_keel_m: Optional[float] = None

    # Gas only: {"inputs": BargeInputs, "tanks": TankResult, "weights": WeightBreakdown} of the
    # converged arrangement, for gas_stability. None for bulk.
    gas_basis: Optional[dict] = None

    checks: List[Check] = field(default_factory=list)
    warnings: List[str] = field(default_factory=list)

    @property
    def ok(self) -> bool:
        return all_ok(self.checks)


# Maximum gas tank outer diameter (m). DESIGN RULE: cap tank OD at
# 12 m and go to two (or more) tanks across above that, rather than letting a single tank
# grow with the beam. This closes an old `n_tanks_wide` TODO — a
# single tank abreast reached ~16 m OD at 20 m beam and ~30 m at 32.4 m, which is outside
# any built cryogenic barge tank and made every wide-beam gas number meaningless.
MAX_TANK_OD_M = 12.0


def tanks_wide_for_beam(beam_m: float, side_clearance_m: float,
                        tank_gap_transverse_m: float = 0.0,
                        max_od_m: float = MAX_TANK_OD_M) -> int:
    """Fewest tanks across the beam that keeps each tank's OD within ``max_od_m``.

    Inverts the arrangement geometry in ``tanks.calculate_tank_arrangement``:
        OD = (beam - 2*cs - (n-1)*gap) / n   <=   max_od
      => n >= (beam - 2*cs + gap) / (max_od + gap)
    Always at least 1, so a narrow hull is unaffected.
    """
    return _call("barge_capacity", "tanks_wide_for_beam", dict(
        beam_m=beam_m, side_clearance_m=side_clearance_m,
        tank_gap_transverse_m=tank_gap_transverse_m, max_od_m=max_od_m), int)


def compute(inp: BargeCapacityInputs) -> BargeCapacityResult:
    """Run the unified barge capacity chain (volume-bound)."""
    data = _bridge.raw("barge_capacity", "compute", _bridge.encode_args({"inputs": inp}))
    if "error" in data:
        _bridge._raise(data["error"])
    return _result_from_json(data["ok"])


def _result_from_json(data: dict) -> BargeCapacityResult:
    """The result object for the core's JSON answer. The result is plain data: functions that take
    it (``journey_derating``, ``gas_stability.assess``, ``ga_adapter``) use the fields it has."""
    result = _bridge.decode(BargeCapacityResult, data)
    basis = result.gas_basis
    if basis is not None:
        result.gas_basis = {"inputs": _bridge.decode(BargeInputs, basis["inputs"]),
                            "tanks": _bridge.decode(TankResult, basis["tanks"]),
                            "weights": _bridge.decode(WeightBreakdown, basis["weights"])}
    return result
