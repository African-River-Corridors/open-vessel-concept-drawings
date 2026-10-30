"""Protective location checks per IGF Code and IGC Code for Type C tanks."""

from dataclasses import dataclass
from enum import Enum
from .._bridge import call as _call


class VesselType(Enum):
    """Vessel/gas code classification controlling protective location rules."""
    IGF = "IGF"
    IGC_1G = "IGC_1G"
    IGC_2G = "IGC_2G"
    IGC_3G = "IGC_3G"
    # (The ADN tank vessel type G member was removed: the engine applies IGC rules only to gas tanks; issue #175.)
    # (IGC_2G_EXACT was removed in issue #237: IGC_2G is the per-tank rule of 2.4.1.1, measured plate to plate per 2.4.2.)


@dataclass
class ProtectiveLocationResult:
    """Result of protective location clearance checks."""
    vessel_type: str
    d_vc: float              # Volume-based minimum distance (m)
    total_tank_volume: float  # Vc used for d(Vc) lookup (m³)
    cs_required: float       # Required side clearance (m)
    cs_actual: float         # Actual side clearance (m)
    cs_ok: bool
    cb_required: float       # Required bottom clearance (m)
    cb_actual: float         # Actual bottom clearance (m)
    cb_ok: bool


def d_vc_exact(vc_individual_m3: float) -> float:
    """IGC 2.4.1.1 / IGF 5.3.3.4.2 d(Vc): Vc = gross volume of ONE tank (m3), domes and appendages included.

    Distance plate to plate, from the moulded line to the moulded line, excluding insulation
    (IGC 2.4.2): the clear gap from the outer face of the insulation is d less the insulation.
    """
    return _call("protective_location", "d_vc_exact", {"vc_individual_m3": vc_individual_m3}, float)


def required_side_protection(vessel_type: VesselType, beam: float, d_vc: float) -> float:
    """Required side clearance Cs from tank to hull side.

    Args:
        vessel_type: Code classification.
        beam: Moulded beam (m).
        d_vc: Volume-based minimum distance (m).

    Returns:
        Required Cs in metres.
    """
    if not isinstance(vessel_type, VesselType):
        # 0.1.0 compared against the enum members, so plain text (even "IGC_2G") was unknown.
        raise ValueError(f"Unknown vessel type: {vessel_type}")
    return _call("protective_location", "required_side_protection", {"vessel_type": vessel_type, "beam": beam, "d_vc": d_vc}, float)


def required_bottom_protection(beam: float, d_vc: float, vessel_type: VesselType = None) -> float:
    """Required bottom clearance Cb from tank bottom to keel.

    Same formula for all vessel types.

    Args:
        beam: Moulded beam (m).
        d_vc: Volume-based minimum distance (m).

    Returns:
        Required Cb in metres.
    """
    if not isinstance(vessel_type, VesselType):
        vessel_type = None
    return _call("protective_location", "required_bottom_protection", {"beam": beam, "d_vc": d_vc, "vessel_type": vessel_type}, float)


@dataclass
class RequiredClearances:
    """Required side and bottom clearances from protective location rules."""
    cs_required: float   # Required side clearance Cs (m)
    cb_required: float   # Required bottom clearance Cb (m)
    d_vc: float          # Volume-based minimum distance (m)


def compute_required_clearances(vessel_type_str: str, beam: float, d_vc: float) -> RequiredClearances:
    """Compute the required Cs and Cb for given vessel type, beam, and d(Vc).

    Args:
        vessel_type_str: One of "IGF", "IGC_1G", "IGC_2G", "IGC_3G".
        beam: Moulded beam (m).
        d_vc: Volume-based minimum distance d(Vc) in metres.

    Returns:
        RequiredClearances with cs_required, cb_required, and d_vc.
    """
    return _call("protective_location", "compute_required_clearances", {"vessel_type_str": vessel_type_str, "beam": beam, "d_vc": d_vc}, RequiredClearances)


def check_protective_location(
    vessel_type_str: str,
    beam: float,
    total_tank_volume: float,
    side_clearance: float,
    double_bottom_height: float,
    tank_support_height: float,
) -> ProtectiveLocationResult:
    """Run the full protective location check.

    Args:
        vessel_type_str: One of "IGF", "IGC_1G", "IGC_2G", "IGC_3G".
        beam: Moulded beam (m).
        total_tank_volume: Vc of ONE tank (m³), its gross volume including dome (IGC 2.4.1.1).
            The name is kept from before issue #237, when the total of all tanks was read.
        side_clearance: Actual distance from tank to hull side (m).
        double_bottom_height: Double bottom height (m).
        tank_support_height: Tank saddle/support height above inner bottom (m).

    Returns:
        ProtectiveLocationResult with required vs actual clearances and pass/fail.
    """
    return _call("protective_location", "check_protective_location", dict(
        vessel_type_str=vessel_type_str, beam=beam, total_tank_volume=total_tank_volume,
        side_clearance=side_clearance, double_bottom_height=double_bottom_height,
        tank_support_height=tank_support_height), ProtectiveLocationResult)
