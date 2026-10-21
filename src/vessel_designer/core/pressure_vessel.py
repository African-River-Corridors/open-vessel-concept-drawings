"""ASME Section VIII / IGC Code pressure vessel wall thickness calculation.

Calculates required wall thickness for IMO Type C horizontal cylindrical
tanks based on design pressure, tank diameter, and material properties.

Formulas from ASME Boiler and Pressure Vessel Code, Section VIII, Division 1,
paragraph UG-27, as referenced by the IGC Code Chapter 4 for Type C tanks.
"""

from dataclasses import dataclass
from typing import Dict, Tuple
from .._bridge import call as _call


@dataclass
class TankMaterialProperties:
    """Mechanical and physical properties of a tank material."""

    name: str                       # Display name
    allowable_stress: float         # f, allowable stress (MPa)
    density: float                  # Material density (t/m^3)
    min_thickness: float            # IGC Code minimum thickness after forming (mm)
    corrosion_allowance: float      # Corrosion allowance (mm)
    description: str = ""


TANK_MATERIAL_DATABASE: Dict[str, TankMaterialProperties] = {
    "304L": TankMaterialProperties(
        name="304L Stainless Steel",
        allowable_stress=113.0,
        density=7.93,
        min_thickness=3.0,
        corrosion_allowance=0.0,
        description="Austenitic stainless, LNG/LPG service to -196°C",
    ),
    "316L": TankMaterialProperties(
        name="316L Stainless Steel",
        allowable_stress=113.0,
        density=7.98,
        min_thickness=3.0,
        corrosion_allowance=0.0,
        description="Austenitic stainless, enhanced corrosion resistance",
    ),
    "9Ni": TankMaterialProperties(
        name="9% Nickel Steel",
        allowable_stress=230.0,
        density=7.85,
        min_thickness=5.0,
        corrosion_allowance=0.5,
        description="ASTM A553 Type I, LNG service to -196°C",
    ),
}


def get_tank_material(material_key: str) -> TankMaterialProperties:
    """Look up tank material properties by key."""
    return _call("pressure_vessel", "get_tank_material", {"material_key": material_key}, TankMaterialProperties)


def cylinder_wall_thickness(
    design_pressure_mpa: float,
    inner_radius_mm: float,
    allowable_stress_mpa: float,
    weld_efficiency: float = 0.95,
) -> float:
    """ASME UG-27 circumferential stress formula for cylindrical shells.

    t = P * R / (f * E - 0.6 * P)

    Args:
        design_pressure_mpa: Internal design pressure P (MPa)
        inner_radius_mm: Inner radius R (mm)
        allowable_stress_mpa: Maximum allowable stress f (MPa)
        weld_efficiency: Weld joint efficiency E (0.85-1.0)

    Returns:
        Required wall thickness (mm).
    """
    return _call("pressure_vessel", "cylinder_wall_thickness", dict(
        design_pressure_mpa=design_pressure_mpa, inner_radius_mm=inner_radius_mm,
        allowable_stress_mpa=allowable_stress_mpa, weld_efficiency=weld_efficiency), float)


def hemisphere_head_thickness(
    design_pressure_mpa: float,
    inner_radius_mm: float,
    allowable_stress_mpa: float,
    weld_efficiency: float = 0.95,
) -> float:
    """Hemispherical head thickness.

    Not UG-27, which is the cylindrical shell paragraph; the ASME head paragraph is not in the
    rule-source library, so none is cited (issue #433).

    t = P * R / (2 * f * E - 0.2 * P)

    Args:
        design_pressure_mpa: Internal design pressure P (MPa)
        inner_radius_mm: Inner spherical radius R (mm)
        allowable_stress_mpa: Maximum allowable stress f (MPa)
        weld_efficiency: Weld joint efficiency E (0.85-1.0)

    Returns:
        Required wall thickness (mm).
    """
    return _call("pressure_vessel", "hemisphere_head_thickness", dict(
        design_pressure_mpa=design_pressure_mpa, inner_radius_mm=inner_radius_mm,
        allowable_stress_mpa=allowable_stress_mpa, weld_efficiency=weld_efficiency), float)


def ellipsoidal_head_thickness(
    design_pressure_mpa: float,
    inner_diameter_mm: float,
    allowable_stress_mpa: float,
    weld_efficiency: float = 0.95,
) -> float:
    """2:1 ellipsoidal head thickness.

    Not UG-27, which is the cylindrical shell paragraph; the ASME head paragraph is not in the
    rule-source library, so none is cited (issue #433).

    t = P * D / (2 * f * E - 0.2 * P)

    Args:
        design_pressure_mpa: Internal design pressure P (MPa)
        inner_diameter_mm: Inner diameter D at head skirt (mm)
        allowable_stress_mpa: Maximum allowable stress f (MPa)
        weld_efficiency: Weld joint efficiency E (0.85-1.0)

    Returns:
        Required wall thickness (mm).
    """
    return _call("pressure_vessel", "ellipsoidal_head_thickness", dict(
        design_pressure_mpa=design_pressure_mpa, inner_diameter_mm=inner_diameter_mm,
        allowable_stress_mpa=allowable_stress_mpa, weld_efficiency=weld_efficiency), float)


def calculate_wall_thicknesses(
    outer_diameter_m: float,
    design_pressure_mpa: float,
    material_key: str,
    head_type: str,
    weld_efficiency: float = 0.95,
) -> Tuple[float, float]:
    """Calculate required wall thicknesses for cylinder and heads.

    Uses the outer diameter as a starting point. Since ASME formulas use
    inner radius, iterates once to refine (thin-wall convergence is fast).

    Adds corrosion allowance, rounds up to nearest 0.5mm, and clamps to
    IGC Code minimum thickness.

    Args:
        outer_diameter_m: Tank outer diameter (m)
        design_pressure_mpa: Internal design pressure (MPa)
        material_key: Key into TANK_MATERIAL_DATABASE
        head_type: "hemisphere" or "ellipsoidal_2to1"
        weld_efficiency: Weld joint efficiency (0.85-1.0)

    Returns:
        (cylinder_thickness_mm, head_thickness_mm)
    """
    return _call("pressure_vessel", "calculate_wall_thicknesses", dict(
        outer_diameter_m=outer_diameter_m, design_pressure_mpa=design_pressure_mpa,
        material_key=material_key, head_type=head_type, weld_efficiency=weld_efficiency),
        Tuple[float, float])
