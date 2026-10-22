"""Cargo properties for LNG and LPG variants, bauxite and crude oil."""

from dataclasses import dataclass
from typing import Dict, Generic, Optional, TypeVar
from .._bridge import call as _call


@dataclass
class CargoProperties:
    """Physical properties of a liquid gas cargo."""

    name: str
    density: float          # kg/m^3 (liquid density for gas; stowage density for dry bulk)
    temperature: float      # Storage temperature (deg C)
    pressure: float         # Storage pressure (barg)
    description: str = ""
    phase: str = "liquid_gas"  # "liquid_gas" (cryogenic tanks), "dry_bulk" (hopper hold) or "liquid_bulk" (cargo tanks)
    status: str = "released"   # "released", or "development" (work in development; spec 0006 section 4)


CARGO_DATABASE: Dict[str, CargoProperties] = {
    "LNG": CargoProperties(
        name="LNG",
        density=450.0,
        temperature=-162.0,
        pressure=0.25,
        description="Liquefied Natural Gas (methane-rich)",
    ),
    "LNG_ethane": CargoProperties(
        name="LNG / Ethane",
        density=500.0,
        temperature=-162.0,
        pressure=0.25,
        description="LNG–ethane blend at cryogenic temperature",
    ),
    "LPG_propane": CargoProperties(
        name="LPG (Propane)",
        # 581 kg/m3 = saturated liquid propane at its normal boiling point, -42.1 C,
        # near-atmospheric. This is the FULLY REFRIGERATED case, the default basis for
        # cryogenic propane.
        #
        # It was 500.0, and an earlier model carried 493.0, BOTH labelled -42 C. Neither is a
        # refrigerated density: 489-493 kg/m3 is propane at about +25 C, i.e. the
        # pressurised-ambient case. The catalogue had taken the tank design from one storage
        # regime and the density from the other, and every gas tonnage the model produced was
        # 16.2% light as a result.
        #
        # The regime is not just a number: fully refrigerated means -42 C at near-atmospheric
        # pressure, with insulation and reliquefaction, against thick-walled vessels at ~8 barg
        # for ambient storage. Confirm the cargo spec with the shipper.
        # Sources: NIST WebBook; Air Liquide Gas Encyclopedia; Engineering ToolBox.
        density=581.0,
        temperature=-42.1,
        pressure=0.0,
        description="Fully refrigerated liquefied propane (-42.1 C, near-atmospheric)",
    ),
    "LPG_butane": CargoProperties(
        name="LPG (Butane)",
        density=600.0,
        temperature=-1.0,
        pressure=0.0,
        description="Cryogenic liquefied butane",
    ),
    "LPG_mix": CargoProperties(
        name="LPG (60/40 Propane/Butane)",
        # 588.4 = harmonic (volume-additive) blend of 60% propane at 581 and 40% butane at
        # 600 (LPG_butane above), by mass: 1/rho = 0.6/581 + 0.4/600 = 1/588.45.
        # Was 530.0, which was the same ambient-temperature error as LPG_propane carried.
        # LPG_butane (600) was already right. Confirm the actual cargo and composition with
        # the shipper.
        density=588.4,
        temperature=-42.1,
        pressure=0.0,
        description="Fully refrigerated LPG mix (60% propane / 40% butane by mass)",
    ),
    "bauxite": CargoProperties(
        name="Bauxite",
        density=1500.0,
        temperature=20.0,
        pressure=0.0,
        description="Dry bulk bauxite ore (stowage density, ~0.67 m3/t)",
        phase="dry_bulk",
    ),
    "crude_oil": CargoProperties(
        name="Crude oil",
        # The default Crude Grade: density at 15 C. An estimate (the middle of the SG 0.77 to 0.97
        # range of published assays). The grade data, with a status and a basis for every value,
        # is CRUDE_GRADES below.
        density=870.0,
        temperature=15.0,
        pressure=0.0,
        description="Crude oil, default grade (density at 15 C; see liquid_bulk::crude_grade)",
        phase="liquid_bulk",
        status="development",
    ),
    "diesel_en590": CargoProperties(
        name="Diesel fuel (EN 590)",
        # Density at 20 C: the middle of the relative density range 0.82 to 0.85 that ADN 2025
        # Table C gives for UN 1202 diesel fuel to EN 590. The grade data is CRUDE_GRADES below.
        density=835.0,
        temperature=20.0,
        pressure=0.0,
        description="Diesel fuel to EN 590, UN 1202 (density at 20 C, middle of ADN Table C 0.82 to 0.85; see liquid_bulk::crude_grade)",
        phase="liquid_bulk",
        status="development",
    ),
}


_T = TypeVar("T")


@dataclass
class Graded(Generic[_T]):
    """A value with its status and its basis. `status` is "sourced" (a document that was read) or
    "estimate" (a working value; the basis says why)."""

    value: _T
    status: str
    basis: str


@dataclass
class CrudeGrade:
    """The properties of one crude oil that drive a design (spec 0006 section 3.1). The flags are
    "unknown", "no" or "yes"; `aquatic` is "unknown", "none", "n2" or "n1". An unknown flag gives
    ADN vessel type C (decision D2 of spec 0006)."""

    name: str
    density_kg_m3: Graded[float]               # at density_reference_c
    density_reference_c: float
    design_density_kg_m3: Graded[float]        # for structure and tank test head
    sour: Graded[bool]                         # UN 3494: always ADN type C
    cmr: Graded[str]
    benzene_over_10_percent: Graded[str]
    aquatic: Graded[str]
    flash_point_below_23c: Graded[str]
    vapour_pressure_50c_at_least_1kpa: Graded[str]
    pour_point_c: Optional[Graded[float]]      # None: not known, no heating assumed
    carriage_temperature_c: Optional[Graded[float]]   # None: ambient
    vapour_pressure_class: Graded[str]         # "low" or "volatile"


_UNKNOWN = "Not known: unknown gives type C (spec 0006 decision D2)"

# Kept equal to the Rust data (`ovcd-core/src/liquid_bulk/crude_grade.rs`) by tests/test_crude_grade.py.
CRUDE_GRADES: Dict[str, CrudeGrade] = {
    "crude_oil": CrudeGrade(
        name="Crude oil (default grade)",
        density_kg_m3=Graded(
            870.0, "estimate",
            "Middle of the SG 0.77 to 0.97 range of crude assays (03-design-practice.md 1.1); spec 0006 section 3.1"),
        density_reference_c=15.0,
        design_density_kg_m3=Graded(
            1025.0, "estimate",
            "Designer practice 1.015 to 1.025 t/m3 for tank scantlings (03-design-practice.md 1.1); spec 0006 section 3.1"),
        sour=Graded(
            False, "estimate",
            "Not sour until the user says so; UN 3494 is always ADN type C (ADN 2025 3.2.3.2 Table C, column 6)"),
        cmr=Graded("unknown", "estimate", _UNKNOWN),
        benzene_over_10_percent=Graded("unknown", "estimate", _UNKNOWN),
        aquatic=Graded("unknown", "estimate", _UNKNOWN),
        flash_point_below_23c=Graded(
            "yes", "estimate",
            "Crude oil is Class 3, group F1 in ADN 2025 Table C (UN 1267); flash point below 23 deg C is the usual case"),
        vapour_pressure_50c_at_least_1kpa=Graded(
            "yes", "estimate",
            "Published crude assays give vapour pressure far above 1 kPa at 50 deg C (03-design-practice.md 1.3: Bakken 115 kPa maximum)"),
        pour_point_c=None,
        carriage_temperature_c=None,
        vapour_pressure_class=Graded("low", "estimate", "spec 0006 section 3.1"),
    ),
    "diesel_en590": CrudeGrade(
        name="Diesel fuel (EN 590)",
        density_kg_m3=Graded(
            835.0, "estimate",
            "Middle of the relative density range 0.82 to 0.85 at 20 deg C that ADN 2025 Table C gives for UN 1202 diesel fuel to EN 590"),
        density_reference_c=20.0,
        design_density_kg_m3=Graded(
            850.0, "estimate",
            "The top of the ADN 2025 Table C relative density range (0.85) for UN 1202 diesel fuel to EN 590; a tank is not designed for a denser cargo than the one the table names"),
        sour=Graded(
            False, "sourced",
            "ADN 2025 3.2.3.2 Table C, UN 1202 EN 590 row: not UN 3494, and type N in column 6"),
        cmr=Graded(
            "no", "sourced",
            "ADN 2025 3.2.3.2 Table C, UN 1202 EN 590 row, column 5: hazard labels 3+N2+F, no CMR"),
        benzene_over_10_percent=Graded(
            "no", "sourced",
            "ADN 2025 3.2.3.2 Table C, UN 1202 EN 590 row, column 6: type N; more than 10 % benzene would give type C (3.2.3.3 flowchart)"),
        aquatic=Graded(
            "n2", "sourced",
            "ADN 2025 3.2.3.2 Table C, UN 1202 EN 590 row, column 5: hazard labels 3+N2+F"),
        flash_point_below_23c=Graded(
            "no", "estimate",
            "The flash point of EN 590 diesel was not read here; Table C gives this row open cargo tanks (column 7: 4), which the 3.2.3.3 flowchart allows only for a flash point of 23 deg C or more"),
        vapour_pressure_50c_at_least_1kpa=Graded(
            "no", "estimate",
            "The vapour pressure of diesel was not read here. The flowchart reads this flag only for aquatic toxicity N1, and this grade is N2, so it does not change the type"),
        pour_point_c=None,
        carriage_temperature_c=None,
        vapour_pressure_class=Graded(
            "low", "estimate",
            "ADN 2025 Table C gives the row open cargo tanks (column 7: 4), so the vapour pressure is low"),
    ),
}


def get_cargo(cargo_type: str) -> CargoProperties:
    """Look up cargo properties by type key."""
    if not isinstance(cargo_type, str):
        cargo_type in CARGO_DATABASE        # an unhashable key raises TypeError, as in 0.1.0
        valid = ", ".join(CARGO_DATABASE.keys())
        raise ValueError(f"Unknown cargo type '{cargo_type}'. Valid types: {valid}")
    return _call("cargo", "get_cargo", {"cargo_type": cargo_type}, CargoProperties)
