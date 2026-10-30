"""Electric-barge energy & service speed from hull resistance in a confined channel.

Couples ``resistance.compute`` — ITTC-1957 friction + residuary + Schijf/Lackenby
confined-water increase, which already depends on **beam, hull shape (Cb, draft)** and
**channel width** (through the blockage ratio n = Am/Ac and the Schijf limiting speed) —
to the **electric drivetrain**:

    shaft (delivered) power  P_D(V)      from resistance  (P_D = R·V / propulsive_coeff)
    battery power            P_bat(V)  = P_D(V) / drivetrain_efficiency
    energy per km            e(V)      = P_bat(V) / V_kmh          [kWh/km]

It returns a speed sweep (resistance, shaft & battery power, energy/km) for each
environment, the **Schijf limiting speed**, a chosen **service speed** (a target cruise
capped safely below the limit), and the energy **per trip / per year** — i.e. the model
for the energy consumption of the electric barge and the speed it can hold.

The calculation runs in the Rust core (`ovd._native`). No I/O.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import List, Optional

from .checks import Check, all_ok
from .._bridge import call as _call
from . import resistance as R  # noqa: F401  (0.1.0 exposed this alias)

KN_TO_KMH = 1.852


@dataclass
class EnergyInputs:
    # --- barge (beam, shape) ---
    beam: float = 12.0
    loa: float = 80.0
    lwl: float = 78.0
    draft: float = 3.0
    block_coefficient: float = 0.85

    # --- confined design channel (width, depth) ---
    channel_depth_m: float = 4.0
    channel_bottom_width_m: float = 34.0
    channel_top_width_m: float = 58.0
    current_knots: float = 0.5
    fresh_water: bool = True
    also_open_sea: bool = True         # add an open-sea env for comparison

    # --- electric drivetrain ---
    propulsive_coefficient: float = 0.45   # P_D = R·V / PC (hull + propeller)
    drivetrain_efficiency: float = 0.92    # battery → shaft (motor + inverter + line)

    # --- service speed & route ---
    target_service_speed_knots: float = 6.0
    service_speed_safety_fraction: float = 0.9   # cap service speed at this × Schijf limit
    power_margin: float = 1.20             # installed power = P_bat@service × margin
    one_way_km: float = 102.0              # example one-way river leg (km)
    trips_per_year: float = 300.0        # example annual round trips

    # --- speed sweep for the curves ---
    speed_min_knots: float = 2.0
    speed_max_knots: float = 10.0
    speed_step_knots: float = 0.25


@dataclass
class EnergyPoint:
    speed_knots: float
    speed_kmh: float
    froude_depth: float
    resistance_kn: float
    shaft_power_kw: float       # delivered P_D
    battery_power_kw: float     # P_D / drivetrain_efficiency
    energy_kwh_per_km: float


@dataclass
class EnergyEnvironment:
    name: str
    limiting_speed_knots: Optional[float]
    service_speed_knots: float
    blockage_ratio: float
    curve: List[EnergyPoint]
    # at service speed (interpolated on the curve):
    resistance_kn: float
    shaft_power_kw: float
    battery_power_kw: float
    energy_kwh_per_km: float
    energy_per_trip_kwh: float      # one-way over one_way_km
    energy_per_year_mwh: float      # round trips × trips_per_year


@dataclass
class EnergyResult:
    inputs: EnergyInputs
    installed_power_kw: float
    environments: List[EnergyEnvironment]
    checks: List[Check] = field(default_factory=list)

    @property
    def ok(self) -> bool:
        return all_ok(self.checks)

    @property
    def binding(self) -> Optional[EnergyEnvironment]:
        """Most-confined environment (lowest service speed) — sizes the barge."""
        return min(self.environments, key=lambda e: e.service_speed_knots) if self.environments else None


def compute(inp: EnergyInputs) -> EnergyResult:
    # Build the environments and run the (validated) resistance model. Installed engine
    # power is set huge so resistance doesn't cap the sweep — we choose service speed here.
    return _call("electric_energy", "compute", {"inputs": inp}, EnergyResult)
