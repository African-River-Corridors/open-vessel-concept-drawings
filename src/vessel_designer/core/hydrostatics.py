"""Hydrostatic calculations: displacement, draft, stability, and trim."""


from .hull import HullGeometry
from .models import (
    BargeInputs,
    StabilityResult,
    TankResult,
    TrimResult,
    WeightBreakdown,
)
from .._bridge import call as _call


def find_equilibrium_draft(
    hull: HullGeometry,
    total_weight: float,
    water_density: float,
    max_draft: float,
    block_coefficient: float = 1.0,
    tolerance: float = 0.001,
) -> float:
    """Find the draft at which displacement equals total weight.

    Uses bisection method since displaced_volume is monotonically increasing
    with draft.

    The block coefficient Cb accounts for the difference between the
    moulded hull volume and the actual displacement volume:
        Effective displacement = Cb × hull_volume × water_density

    Args:
        hull: Hull geometry object
        total_weight: Total loaded weight (tonnes)
        water_density: Water density (t/m^3)
        max_draft: Upper bound for search (m)
        block_coefficient: Cb, ratio of actual to moulded displacement (0-1)
        tolerance: Convergence tolerance (m)

    Returns:
        Equilibrium draft (m)
    """
    return _call("hydrostatics", "find_equilibrium_draft", dict(
        hull=hull, total_weight=total_weight, water_density=water_density, max_draft=max_draft,
        block_coefficient=block_coefficient, tolerance=tolerance), float)


def calculate_stability(
    hull: HullGeometry,
    inputs: BargeInputs,
    tanks: TankResult,
    weights: WeightBreakdown,
    cargo_mass: float,
    cargo_density_t: float,
    draft: float,
) -> StabilityResult:
    """Calculate intact stability (GM) at the equilibrium draft.

    Args:
        hull: Hull geometry
        inputs: Barge inputs
        tanks: Tank results
        weights: Weight breakdown
        cargo_mass: Cargo mass (tonnes)
        cargo_density_t: Cargo density (t/m^3)
        draft: Equilibrium draft (m)

    Returns:
        StabilityResult with KB, BM, KG, GM values.
    """
    return _call("hydrostatics", "calculate_stability", dict(
        hull=hull, inputs=inputs, tanks=tanks, weights=weights, cargo_mass=cargo_mass, cargo_density_t=cargo_density_t, draft=draft),
        StabilityResult)


def calculate_trim(
    hull: HullGeometry,
    inputs: BargeInputs,
    tanks: TankResult,
    weights: WeightBreakdown,
    cargo_mass: float,
    draft: float,
) -> TrimResult:
    """Calculate longitudinal trim.

    Convention: x measured from stern (AP). x=0 at stern, x=LOA at bow.
    Positive trim = bow deeper than stern.

    Args:
        hull: Hull geometry
        inputs: Barge inputs
        tanks: Tank results
        weights: Weight breakdown
        cargo_mass: Cargo mass (tonnes)
        draft: Equilibrium draft (m)

    Returns:
        TrimResult with LCG, LCB, trim, and drafts at bow/stern.
    """
    return _call("hydrostatics", "calculate_trim", dict(
        hull=hull, inputs=inputs, weights=weights, cargo_mass=cargo_mass, draft=draft), TrimResult)
