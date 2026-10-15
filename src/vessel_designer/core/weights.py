"""Component weight breakdown for ATB barge."""


from .hull import HullGeometry
from .models import BargeInputs, TankResult, WeightBreakdown
from .._bridge import call as _call


def calculate_weights(
    inputs: BargeInputs,
    hull: HullGeometry,
    tanks: TankResult,
) -> WeightBreakdown:
    """Calculate component weight breakdown.

    Hull steel estimated from plating areas and thicknesses, plus a
    structure factor for internal framing (floors, frames, longitudinals,
    bulkheads).

    Args:
        inputs: Barge design inputs
        hull: Hull geometry object
        tanks: Tank arrangement results

    Returns:
        WeightBreakdown with all component weights in tonnes.
    """
    return _call("weights", "calculate_weights", {"inputs": inputs, "hull": hull, "tanks": tanks},
                 WeightBreakdown)
