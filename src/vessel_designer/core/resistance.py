"""Hull resistance, service speed, and fuel efficiency for ATB barge.

Covers three operating environments:
- Open sea (deep water): ITTC 1957 frictional + residuary for bluff bodies
- Confined shallow river (6.5 m deep): Schijf limiting speed, Lackenby shallow water corrections
- Confined creek (8.0 m deep): same confined-water methods, different channel geometry

References:
- ITTC 1957 friction line
- Schijf (1949) limiting speed in confined waterways
- Lackenby (1963) shallow water resistance increase
- Guldhammer-Harvald residuary resistance charts
"""

from dataclasses import dataclass, field
from typing import List, Optional

from .checks import Check, all_ok
from .._bridge import call as _call


# --- Physical constants ---
GRAVITY = 9.81  # m/s^2
KNOTS_TO_MS = 0.5144  # 1 knot = 0.5144 m/s
KW_PER_KN_MS = 1.0  # 1 kN × 1 m/s = 1 kW


@dataclass
class TugInputs:
    """Tug propulsion characteristics."""
    engine_power_kw: float = 2400.0          # Total installed power (kW)
    propulsive_efficiency: float = 0.45       # Overall propulsive coefficient
    sfc_g_per_kwh: float = 195.0             # Specific fuel consumption (g/kWh)
    fuel_density_kg_per_litre: float = 0.845  # Marine diesel density


@dataclass
class EnvironmentInputs:
    """Channel/waterway parameters for one operating environment."""
    name: str = "Open Sea"
    water_depth: float = 50.0                 # m
    channel_width_bottom: float = 1000.0      # Bottom width of trapezoidal channel (m)
    channel_width_surface: float = 1000.0     # Surface width (m)
    bank_slope: float = 3.0                   # Horizontal : vertical
    current_speed_knots: float = 0.0          # Opposing current (knots)
    water_density: float = 1.025              # t/m^3
    is_deep_water: bool = False
    #: Significant wave height. Zero in a river. Non-zero brings waves.compute into the
    #: resistance, replacing the "sea margin the analyst picked" that this engine used to
    #: force on its reader.
    significant_wave_height_m: float = 0.0
    wave_heading_deg_off_bow: float = 0.0
    peak_wave_period_s: float = 0.0
    #: Under-keel clearance to keep BELOW the squatted keel. Squat is what actually limits
    #: speed in a shallow canalised river, and this engine had no model of it: it returned
    #: 9.2 kn for a vessel that squats 1.7 m into 0.40 m of clearance.
    squat_safety_margin_m: float = 0.15
    apply_squat_limit: bool = True               # Skip shallow/confined corrections


def default_environments() -> List[EnvironmentInputs]:
    """Factory for the three standard operating environments."""
    return [
        EnvironmentInputs(
            name="Open Sea",
            water_depth=50.0,
            channel_width_bottom=1000.0,
            channel_width_surface=1000.0,
            current_speed_knots=0.0,
            water_density=1.025,
            is_deep_water=True,
        ),
        EnvironmentInputs(
            name="Confined shallow river",
            water_depth=6.5,
            channel_width_bottom=40.0,
            channel_width_surface=60.0,
            bank_slope=3.0,
            current_speed_knots=1.0,
            water_density=1.0,
        ),
        EnvironmentInputs(
            name="Confined creek",
            water_depth=8.0,
            channel_width_bottom=60.0,
            channel_width_surface=100.0,
            bank_slope=3.0,
            current_speed_knots=0.5,
            water_density=1.0,
        ),
    ]


@dataclass
class ResistanceInputs:
    """All inputs for the resistance calculation."""
    # Vessel parameters
    loa: float = 130.0
    lwl: float = 125.0                        # Waterline length (m)
    beam: float = 24.0
    draft: float = 6.0
    block_coefficient: float = 0.85
    wetted_surface_area: float = 0.0          # m^2; 0 = auto-calculate
    displacement_tonnes: float = 0.0          # tonnes; 0 = auto from Cb

    # Tug
    tug: TugInputs = field(default_factory=TugInputs)

    # Environments
    environments: List[EnvironmentInputs] = field(default_factory=default_environments)

    # Speed range for curves
    speed_min_knots: float = 2.0
    speed_max_knots: float = 12.0
    speed_step_knots: float = 0.5


# --- Results ---

@dataclass
class SpeedPoint:
    """Resistance and power at one speed in one environment."""
    speed_knots: float                    # Speed over ground
    speed_through_water_knots: float      # = SOG + current
    froude_number: float                  # V / sqrt(g * L)
    froude_depth: float                   # V / sqrt(g * h)
    resistance_frictional_kn: float
    resistance_residuary_kn: float
    resistance_shallow_factor: float      # Multiplier from shallow water
    resistance_total_kn: float
    effective_power_kw: float             # R_T * V
    delivered_power_kw: float             # P_E / eta
    fuel_litres_per_hour: float
    fuel_litres_per_nm: float
    power_available: bool                 # delivered_power <= engine power
    resistance_waves_kn: float = 0.0      # STAwave-1 added resistance (waves.py)
    squat_m: float = 0.0                  # maximum squat at this speed (squat.py)
    dynamic_ukc_m: float = float("inf")   # clearance left under the keel at this speed
    squat_limited: bool = False           # True where squat, not power, forbids this speed


@dataclass
class EnvironmentResult:
    """Resistance results for one operating environment."""
    name: str
    limiting_speed_knots: Optional[float]   # Schijf critical speed (None for deep water)
    service_speed_knots: float              # Max speed where power available
    blockage_ratio: float                   # Am / Ac
    speed_curve: List[SpeedPoint]
    # At service speed:
    resistance_at_service_kn: float
    power_at_service_kw: float
    fuel_at_service_lph: float
    fuel_at_service_lpnm: float
    squat_limited_speed_knots: Optional[float] = None   # fastest speed keeping the margin
    service_speed_governed_by: str = "power"            # power | squat | channel


@dataclass
class ResistanceResults:
    """Complete resistance calculation output.

    The single number that leaves this module to size the electric-barge battery is
    ``binding_power_at_service_kw`` — the delivered power at service speed in the
    *binding* (shallowest, lowest service-speed) environment.
    """
    inputs: ResistanceInputs
    wetted_surface_area: float
    environments: List[EnvironmentResult]
    checks: List[Check] = field(default_factory=list)
    warnings: List[str] = field(default_factory=list)

    @property
    def ok(self) -> bool:
        """True if no error-severity check has failed (warnings don't gate)."""
        return all_ok(self.checks)

    @property
    def binding_environment(self) -> Optional[EnvironmentResult]:
        """The environment that sizes propulsion: the one with the lowest service
        speed (the shallowest/most-confined leg the barge must still make headway in).
        This is the river leg in practice. Returns None if there are no environments.
        """
        if not self.environments:
            return None
        return min(self.environments, key=lambda e: e.service_speed_knots)

    @property
    def binding_power_at_service_kw(self) -> float:
        """Delivered power at service speed in the binding environment.

        THIS is the propulsion power that feeds ``electric_barge`` battery sizing,
        replacing its hard-coded 600 kW. Read this off the result.
        """
        env = self.binding_environment
        return env.power_at_service_kw if env is not None else 0.0


# --- Calculator ---

class ResistanceCalculator:
    """Hull resistance, power, and fuel consumption calculator (runs in the Rust core)."""

    def __init__(self, inputs: ResistanceInputs):
        self.inputs = inputs

    def calculate(self) -> ResistanceResults:
        """The resistance results; ``checks`` stays empty, as ``compute`` adds them afterwards."""
        result = compute(self.inputs)
        result.checks = []
        return result


# --- Standardised module contract: checks + free compute() ---


def compute(inputs: ResistanceInputs) -> ResistanceResults:
    """Pure free-function entrypoint to the standardised module contract.

    Wraps ``ResistanceCalculator(inputs).calculate()`` (the resistance maths is
    unchanged) and attaches the validity ``checks``. Use ``result.binding_power_at_service_kw``
    as the propulsion-power feed to electric-barge battery sizing.
    """
    return _call("resistance", "compute", {"inputs": inputs}, ResistanceResults)
