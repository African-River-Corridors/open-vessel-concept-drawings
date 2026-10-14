"""IMO Type C horizontal cylindrical tank geometry and arrangement.

Models horizontal pressure vessels with hemispherical or 2:1 ellipsoidal
dished heads, arranged in rows (abreast) and columns (along the length)
on a barge deck.
"""


from .models import BargeInputs, TankResult
from .._bridge import call as _call


def calculate_tank_arrangement(inputs: BargeInputs) -> TankResult:
    """Calculate tank dimensions, volumes, and weights for the given barge inputs.

    Tank sizing logic:
    1. Available beam = beam - 2 * side_clearance
    2. Total width for tanks = available_beam - (n_wide - 1) * gap_transverse
    3. Outer diameter = total_width / n_wide
    4. Available length = LOA - non_tank_length
    5. Total length for tanks = available_length - (n_long - 1) * gap_longitudinal
    6. Length per tank = total_length / n_long
    7. Cylinder length = tank_length - 2 * head_length
    """
    return _call("tanks", "calculate_tank_arrangement", {"inputs": inputs}, TankResult)


def free_surface_moment_cylindrical(
    inner_radius: float,
    cylinder_length: float,
    fill_ratio: float,
    cargo_density: float,
) -> float:
    """Transverse free surface moment for a horizontal cylindrical tank.

    For a horizontal cylinder partially filled, the free surface is a
    rectangle of width = chord length at fill level, length = cylinder length.

    The free surface inertia about the tank centerline:
    I_fs = (1/12) * L * chord^3

    The free surface correction to GM:
    GG' = rho_cargo * I_fs / displacement

    This function returns rho_cargo * I_fs (the numerator), so the caller
    divides by displacement.

    Args:
        inner_radius: Tank inner radius (m)
        cylinder_length: Length of cylindrical section (m)
        fill_ratio: Fill level as fraction (0-1)
        cargo_density: Cargo density (t/m^3)

    Returns:
        Free surface moment = cargo_density * I_fs (t*m)
    """
    return _call("tanks", "free_surface_moment_cylindrical", dict(
        inner_radius=inner_radius, cylinder_length=cylinder_length, fill_ratio=fill_ratio,
        cargo_density=cargo_density), float)
