"""Public-API scenarios, written once and run on two packages (issue #143).

A scenario is a function `f(c)` that calls the public API the way a user script does and returns
what came back. `c` is a `Ctx`: it imports the package under test (`c.mod("core.hull")`) and turns
every number written in a scenario into the form the mode asks for:

- `float`       the number as written (the control)
- `int`         a whole number as a Python `int` (a user types `90`, not `90.0`)
- `np.int64`    a whole number as a numpy integer, any other number as a numpy float
- `np.float64`  every number as a numpy float

The same scenarios run on the frozen pure-Python package at commit ce96e66 (see
`tests/data/make_api_ce96e66.py`, which writes `tests/data/api_ce96e66.json`) and on the package
under test (`tests/test_api_oracle.py`). Besides calls with plain numbers, the scenarios hand the API
the things 0.1.0 let a user do: pass an edited or rebuilt object, and reuse one inputs object in a
loop.

`covers` names the public functions a scenario calls, so a test can check that none is missed.
"""
from __future__ import annotations

import dataclasses
import enum
import importlib
import math

import numpy as np

MODES = ("float", "int", "np.int64", "np.float64")
SCENARIOS: dict = {}


def scenario(name, covers=()):
    def register(fn):
        assert name not in SCENARIOS, name
        fn.covers = tuple(covers)
        SCENARIOS[name] = fn
        return fn
    return register


class Ctx:
    def __init__(self, pkg: str, mode: str):
        assert mode in MODES, mode
        self.pkg, self.mode = pkg, mode

    def mod(self, name: str):
        return importlib.import_module(f"{self.pkg}.{name}")

    def n(self, x):
        """A number, in the form of the mode."""
        if isinstance(x, (list, tuple)):
            return [self.n(v) for v in x]
        if self.mode == "float":
            return x
        whole = float(x).is_integer()
        if self.mode == "int":
            return int(x) if whole else x
        if self.mode == "np.int64":
            return np.int64(x) if whole else np.float64(x)
        return np.float64(x)

    def k(self, i):
        """A count (an int field): an int, or a numpy integer in the numpy modes."""
        return np.int64(i) if self.mode.startswith("np.") else int(i)


# --- results -> plain data -----------------------------------------------------------------

def norm(v):
    if dataclasses.is_dataclass(v) and not isinstance(v, type):
        return {f.name: norm(getattr(v, f.name)) for f in dataclasses.fields(v)}
    if isinstance(v, enum.Enum):
        return v.value
    if isinstance(v, np.generic):
        return norm(v.item())
    if isinstance(v, (list, tuple)):
        return [norm(x) for x in v]
    if isinstance(v, dict):
        return {str(k): norm(x) for k, x in v.items()}
    if isinstance(v, float):
        if math.isnan(v):
            return "float:nan"
        if math.isinf(v):
            return "float:inf" if v > 0 else "float:-inf"
    return v


def run(pkg: str, mode: str, name: str):
    """`{"ok": result}` or `{"raised": [class name, message]}`."""
    try:
        return {"ok": norm(SCENARIOS[name](Ctx(pkg, mode)))}
    except Exception as e:      # noqa: BLE001 -- the exception is the answer
        return {"raised": [type(e).__name__, str(e)]}


def value(attr):
    """A property's value, or a method's result: 0.1.0 had both kinds on one class."""
    return attr() if callable(attr) else attr


# --- shared builders -----------------------------------------------------------------------

def barge_inputs(c, **over):
    """The CN II style gas barge of the review, with every number in the mode's form."""
    m = c.mod("core.models")
    kw = dict(loa=c.n(90.0), beam=c.n(14.8), depth=c.n(6.5), max_draft=c.n(4.4),
              vertical_clearance=c.n(10.0))
    kw.update(over)
    return m.BargeInputs(**kw)


def hull_of(c, bi, **over):
    h = c.mod("core.hull")
    kw = dict(loa=bi.loa, beam=bi.beam, depth=bi.depth, bow_rake_length=bi.bow_rake_length,
              bow_rake_rise=bi.bow_rake_rise, stern_rake_length=bi.stern_rake_length,
              stern_rake_rise=bi.stern_rake_rise)
    kw.update(over)
    return h.HullGeometry(**kw)


def chain(c, **over):
    bi = barge_inputs(c, **over)
    hull = hull_of(c, bi)
    tanks = c.mod("core.tanks").calculate_tank_arrangement(bi)
    weights = c.mod("core.weights").calculate_weights(bi, hull, tanks)
    return bi, hull, tanks, weights


def gas_capacity(c, **over):
    b = c.mod("core.barge_capacity")
    kw = dict(loa=c.n(90.0), beam=c.n(14.8), depth=c.n(6.5), cargo_type="LPG_propane")
    kw.update(over)
    inp = b.BargeCapacityInputs(**kw)
    return inp, b.compute(inp)


def bulk_capacity(c, **over):
    b = c.mod("core.barge_capacity")
    kw = dict(loa=c.n(100.0), beam=c.n(16.0), depth=c.n(6.0), cargo_type="bauxite")
    kw.update(over)
    inp = b.BargeCapacityInputs(**kw)
    return inp, b.compute(inp)


CN_II = dict(vessel="gas_barge", cargo="LPG_propane", cb=0.85, freeboard_min_m=0.3,
             self_propelled=True, ruleset="IGC_2G", fill_ratio=0.95)


# --- hull, tanks, weights, hydrostatics ----------------------------------------------------

@scenario("hull.methods", covers=["core.hull.HullGeometry"])
def _(c):
    h = c.mod("core.hull").HullGeometry(c.n(90), c.n(15), c.n(7), c.n(10), c.n(3), c.n(5), c.n(2))
    d = c.n(3)
    return dict(
        parallel_length=h.parallel_length, displaced_volume=h.displaced_volume(d),
        waterplane_area=h.waterplane_area(d), waterplane_length=h.waterplane_length(d),
        it=h.waterplane_inertia_transverse(d), il=h.waterplane_inertia_longitudinal(d),
        KB=h.KB(d), LCB=h.LCB(d), BM_t=h.BM_transverse(d), BM_l=h.BM_longitudinal(d),
        bottom=h.bottom_area(), side=h.side_area(), deck=h.deck_area(),
        bow_rake=h.bow_rake_area(), stern_rake=h.stern_rake_area(),
        total=h.total_hull_surface_area())


@scenario("hull.box_methods", covers=["core.hull.HullGeometry"])
def _(c):
    h = c.mod("core.hull").HullGeometry(c.n(90), c.n(15), c.n(7), c.n(0), c.n(7), c.n(0), c.n(7))
    d = c.n(3)
    return [h.displaced_volume(d), h.KB(d), h.bottom_area(), h.parallel_length]


@scenario("models.BargeInputs", covers=["core.models.BargeInputs"])
def _(c):
    return barge_inputs(c, n_tanks_wide=c.k(2), side_clearance=c.n(1))


@scenario("tanks.calculate_tank_arrangement", covers=["core.tanks.calculate_tank_arrangement"])
def _(c):
    return c.mod("core.tanks").calculate_tank_arrangement(barge_inputs(c))


@scenario("tanks.calculate_tank_arrangement.two_wide",
          covers=["core.tanks.calculate_tank_arrangement"])
def _(c):
    bi = barge_inputs(c, beam=c.n(20), n_tanks_wide=c.k(2), n_tanks_long=c.k(3),
                      tank_wall_thickness=None)
    return c.mod("core.tanks").calculate_tank_arrangement(bi)


@scenario("tanks.free_surface_moment_cylindrical",
          covers=["core.tanks.free_surface_moment_cylindrical"])
def _(c):
    f = c.mod("core.tanks").free_surface_moment_cylindrical
    return [f(c.n(5), c.n(20), c.n(0.5), c.n(0.5)), f(c.n(4), c.n(10), c.n(0.25), c.n(1))]


@scenario("weights.calculate_weights", covers=["core.weights.calculate_weights"])
def _(c):
    return chain(c)[3]


@scenario("hydrostatics.find_equilibrium_draft",
          covers=["core.hydrostatics.find_equilibrium_draft"])
def _(c):
    hy = c.mod("core.hydrostatics")
    hull = c.mod("core.hull").HullGeometry(c.n(90), c.n(15), c.n(7), c.n(0), c.n(7), c.n(0), c.n(7))
    return [hy.find_equilibrium_draft(hull, c.n(4000), c.n(1), c.n(7), c.n(0.5)),
            hy.find_equilibrium_draft(hull, c.n(2500), c.n(1), c.n(7)),
            hy.find_equilibrium_draft(hull, c.n(2500), c.n(1.025), c.n(7), c.n(0.85), c.n(0.0001))]


@scenario("hydrostatics.calculate_stability", covers=["core.hydrostatics.calculate_stability"])
def _(c):
    bi, hull, tanks, weights = chain(c)
    return c.mod("core.hydrostatics").calculate_stability(
        hull, bi, tanks, weights, c.n(2000), c.n(0.58), c.n(3.5))


@scenario("hydrostatics.calculate_trim", covers=["core.hydrostatics.calculate_trim"])
def _(c):
    bi, hull, tanks, weights = chain(c)
    return c.mod("core.hydrostatics").calculate_trim(hull, bi, tanks, weights, c.n(2000), c.n(3.5))


# --- capacity, journey, gas stability, GA adapter, arrangement search ----------------------

@scenario("barge_capacity.compute.gas", covers=["core.barge_capacity.compute"])
def _(c):
    return gas_capacity(c)[1]


@scenario("barge_capacity.compute.gas_counts", covers=["core.barge_capacity.compute"])
def _(c):
    return gas_capacity(c, n_tanks_wide=c.k(2), n_tanks_long=c.k(2), cargo_type="LNG")[1]


@scenario("barge_capacity.compute.bulk", covers=["core.barge_capacity.compute"])
def _(c):
    return bulk_capacity(c)[1]


@scenario("barge_capacity.compute.bulk_holds", covers=["core.barge_capacity.compute"])
def _(c):
    return bulk_capacity(c, n_holds=c.k(4), ballast_capacity_t=c.n(100), air_draft_limit_m=c.n(8))[1]


@scenario("barge_capacity.compute.nested",
          covers=["core.barge_capacity.compute", "core.aft_arrangement.compute"])
def _(c):
    aft = c.mod("core.aft_arrangement").AftArrangementInputs(beam=c.n(14.8), depth=c.n(6.5))
    sep = c.mod("core.gas_separation").GasSeparationInputs(cofferdam_m=c.n(1), opening_setback_m=c.n(2))
    return gas_capacity(c, aft=aft, separation=sep)[1]


@scenario("barge_capacity.tanks_wide_for_beam", covers=["core.barge_capacity.tanks_wide_for_beam"])
def _(c):
    f = c.mod("core.barge_capacity").tanks_wide_for_beam
    return [f(c.n(20), c.n(1)), f(c.n(30), c.n(1), c.n(1), c.n(12)), f(c.n(14.8), c.n(1))]


def derate_of(c, capacity, **over):
    j = c.mod("core.journey_derating")
    kw = dict(capacity=capacity, loa=c.n(100), beam=c.n(16), depth=c.n(6), design_depth_m=c.n(5),
              ukc_m=c.n(1))
    kw.update(over)
    return j.compute(j.JourneyDeratingInputs(**kw))


@scenario("journey_derating.compute", covers=["core.journey_derating.compute"])
def _(c):
    cap = bulk_capacity(c)[1]
    return [derate_of(c, cap), derate_of(c, cap, design_depth_m=c.n(9), ukc_m=c.n(0)),
            derate_of(c, cap, water_density=c.n(1.025), block_coefficient=c.n(0.9),
                      utilisation_floor=c.n(1))]


@scenario("gas_stability.box_gz", covers=["core.gas_stability.box_gz"])
def _(c):
    f = c.mod("core.gas_stability").box_gz
    return [f(c.n(14.8), c.n(6.5), c.n(3), c.n(4), c.n(10)), f(c.n(14.8), c.n(6.5), c.n(3), c.n(4), c.n(0)),
            f(c.n(14.8), c.n(6.5), c.n(3.5), c.n(4.5), c.n(45))]


@scenario("gas_stability.segment_fill", covers=["core.gas_stability.segment_fill"])
def _(c):
    f = c.mod("core.gas_stability").segment_fill
    return [f(c.n(4.0), c.n(0.5)), f(c.n(4.5), c.n(0.25)), f(c.n(4), c.n(1)), f(c.n(4.0), c.n(0))]


@scenario("gas_stability.criteria_set_for", covers=["core.gas_stability.criteria_set_for"])
def _(c):
    f = c.mod("core.gas_stability").criteria_set_for
    return [f("ADN_G"), f("IGC_2G")]


@scenario("gas_stability.assess", covers=["core.gas_stability.assess"])
def _(c):
    g = c.mod("core.gas_stability")
    res = gas_capacity(c)[1]
    return [g.assess(res, c.n(1500)), g.assess(res, c.n(1200), block_coefficient=c.n(0.9),
                                               water_density=c.n(1),
                                               criteria_set="ADN_9_3_1_14_2")]


@scenario("ga_adapter.is_gas", covers=["core.ga_adapter.is_gas"])
def _(c):
    a = c.mod("core.ga_adapter")
    return [a.is_gas(gas_capacity(c)[1]), a.is_gas(bulk_capacity(c)[1])]


@scenario("ga_adapter.to_ga", covers=["core.ga_adapter.to_ga"])
def _(c):
    a = c.mod("core.ga_adapter")
    inp, res = gas_capacity(c)
    return [a.to_ga(inp, res), a.to_ga(inp, res, c.n(3), False), a.to_ga(inp, res, laden_draft_m=c.n(3.5))]


@scenario("ga_adapter.to_ga.bulk", covers=["core.ga_adapter.to_ga"])
def _(c):
    inp, res = bulk_capacity(c)
    return c.mod("core.ga_adapter").to_ga(inp, res)


@scenario("gas_arrangement.fit", covers=["core.gas_arrangement.fit"])
def _(c):
    ga = c.mod("core.gas_arrangement")
    return ga.fit(c.n(90), c.n(14.8), "LPG_propane", c.n(4.4), ruleset="IGC_2G", fill_ratio=c.n(0.95),
                  depths=[c.n(6.5)], tanks_wide=[c.k(1), c.k(2)], extra_clearance_m=[c.n(1), c.n(2)])


@scenario("gas_arrangement.fit.adn", covers=["core.gas_arrangement.fit"])
def _(c):
    ga = c.mod("core.gas_arrangement")
    return ga.fit(c.n(80), c.n(12), "LNG", c.n(3), ruleset="ADN_G", fill_ratio=c.n(0.95),
                  self_propelled=False, depths=[c.n(5), c.n(6)], tanks_wide=[c.k(1)],
                  extra_clearance_m=[c.n(1)], adn_tank_cap_m3=c.n(1000), block_coefficient=c.n(0.9),
                  max_tank_od_m=c.n(12))


@scenario("gas_arrangement.required_side_clearance",
          covers=["core.gas_arrangement.required_side_clearance"])
def _(c):
    f = c.mod("core.gas_arrangement").required_side_clearance
    return [f("IGC_2G", c.n(14.8), c.n(2000), c.n(1000), c.n(350)),
            f("ADN_G", c.n(12), c.n(900), c.n(450), c.n(300))]


@scenario("gas_arrangement.smallest_viable_loa",
          covers=["core.gas_arrangement.smallest_viable_loa"])
def _(c):
    f = c.mod("core.gas_arrangement").smallest_viable_loa
    return f(c.n(14.8), "LPG_propane", c.n(4.4), lo=c.n(20), hi=c.n(120), ruleset="IGC_2G",
             depths=[c.n(6.5)], tanks_wide=[c.k(1)], extra_clearance_m=[c.n(1)])


# --- smaller modules -----------------------------------------------------------------------

@scenario("cargo.get_cargo", covers=["core.cargo.get_cargo"])
def _(c):
    g = c.mod("core.cargo").get_cargo
    return [g(k) for k in ("LNG", "LPG_propane", "bauxite")]


@scenario("pressure_vessel", covers=[
    "core.pressure_vessel.get_tank_material", "core.pressure_vessel.cylinder_wall_thickness",
    "core.pressure_vessel.hemisphere_head_thickness", "core.pressure_vessel.ellipsoidal_head_thickness",
    "core.pressure_vessel.calculate_wall_thicknesses"])
def _(c):
    p = c.mod("core.pressure_vessel")
    return dict(
        material=p.get_tank_material("9Ni"),
        cyl=p.cylinder_wall_thickness(c.n(1), c.n(5000), c.n(200), c.n(1)),
        cyl_default_weld=p.cylinder_wall_thickness(c.n(0.5), c.n(4000), c.n(180)),
        hemi=p.hemisphere_head_thickness(c.n(1), c.n(5000), c.n(200), c.n(1)),
        ell=p.ellipsoidal_head_thickness(c.n(1), c.n(10000), c.n(200), c.n(1)),
        walls=p.calculate_wall_thicknesses(c.n(11), c.n(1), "9Ni", "hemisphere"),
        walls_ell=p.calculate_wall_thicknesses(c.n(8), c.n(0.5), "304L", "ellipsoidal_2to1", c.n(0.9)))


@scenario("protective_location", covers=[
    "core.protective_location.distance_d_vc", "core.protective_location.d_vc_exact",
    "core.protective_location.required_side_protection",
    "core.protective_location.required_bottom_protection",
    "core.protective_location.compute_required_clearances",
    "core.protective_location.check_protective_location"])
def _(c):
    p = c.mod("core.protective_location")
    vt = p.VesselType
    return dict(
        d=[p.distance_d_vc(c.n(v)) for v in (500, 3000, 8000)],
        exact=[p.d_vc_exact(c.n(v)) for v in (500, 2000, 10000, 40000)],
        side=[p.required_side_protection(t, c.n(15), c.n(1)) for t in vt],
        bottom=[p.required_bottom_protection(c.n(15), c.n(1)), p.required_bottom_protection(c.n(15), c.n(1), vt.ADN_G)],
        clearances=[p.compute_required_clearances(t.value, c.n(14.8), c.n(1)) for t in vt],
        check=p.check_protective_location("IGC_2G", c.n(14.8), c.n(2000), c.n(2), c.n(1), c.n(0.5)))


@scenario("resistance", covers=[
    "core.resistance.compute", "core.resistance.default_environments",
    "core.resistance.ResistanceCalculator"])
def _(c):
    r = c.mod("core.resistance")
    env = r.EnvironmentInputs(name="River", water_depth=c.n(6), channel_width_bottom=c.n(40),
                              channel_width_surface=c.n(60), current_speed_knots=c.n(1),
                              water_density=c.n(1))
    inp = r.ResistanceInputs(loa=c.n(90), lwl=c.n(88), beam=c.n(15), draft=c.n(3),
                             tug=r.TugInputs(engine_power_kw=c.n(1000)), environments=[env],
                             speed_min_knots=c.n(2), speed_max_knots=c.n(6), speed_step_knots=c.n(1))
    return dict(compute=r.compute(inp), calc=r.ResistanceCalculator(inp).calculate(),
                defaults=r.default_environments())


@scenario("squat", covers=["core.squat.compute", "core.squat.max_speed_for_ukc"])
def _(c):
    s = c.mod("core.squat")
    inp = s.SquatInputs(speed_through_water_kn=c.n(5), draft_m=c.n(3), beam_m=c.n(15), lpp_m=c.n(90),
                        water_depth_m=c.n(5), channel_area_m2=c.n(200))
    return [s.compute(inp),
            s.max_speed_for_ukc(c.n(3), c.n(15), c.n(90), c.n(5), c.n(200)),
            s.max_speed_for_ukc(c.n(3), c.n(15), c.n(90), c.n(5), c.n(200), c.n(0.9), c.n(0.2),
                                c.n(15), c.n(0.05))]


@scenario("waves.compute", covers=["core.waves.compute"])
def _(c):
    w = c.mod("core.waves")
    return [w.compute(w.WaveResistanceInputs(c.n(1), c.n(15), c.n(90))),
            w.compute(w.WaveResistanceInputs(c.n(2), c.n(15), c.n(90), c.n(20), c.n(30), c.n(5),
                                             c.n(1), c.n(6)))]


@scenario("electric_energy.compute", covers=["core.electric_energy.compute"])
def _(c):
    e = c.mod("core.electric_energy")
    return [e.compute(e.EnergyInputs()),
            e.compute(e.EnergyInputs(beam=c.n(12), loa=c.n(80), lwl=c.n(78), draft=c.n(3),
                                     channel_depth_m=c.n(4), one_way_km=c.n(100),
                                     trips_per_year=c.n(300), speed_max_knots=c.n(6)))]


@scenario("aft_arrangement.compute", covers=["core.aft_arrangement.compute"])
def _(c):
    a = c.mod("core.aft_arrangement")
    return [a.compute(a.AftArrangementInputs(beam=c.n(14.8), depth=c.n(6.5))),
            a.compute(a.AftArrangementInputs(beam=c.n(8), depth=c.n(5), n_thrusters=c.k(2),
                                             n_accom_units=c.k(1), container_gap_m=c.n(1),
                                             sightline_obstruction_m=c.n(6)))]


@scenario("gas_separation.compute", covers=["core.gas_separation.compute"])
def _(c):
    g = c.mod("core.gas_separation")
    return [g.compute(g.GasSeparationInputs()),
            g.compute(g.GasSeparationInputs(cofferdam_m=c.n(1), opening_setback_m=c.n(0),
                                            fwd_end_bulkhead_m=c.n(1), side_clearance_m=c.n(1),
                                            double_bottom_m=c.n(1)))]


@scenario("barge_cost", covers=[
    "core.barge_cost.compute", "core.barge_cost.load_rates", "core.barge_cost.parse_flat_yaml"])
def _(c):
    b = c.mod("core.barge_cost")
    rates = dict(hull_steel_rate=c.n(2500), tank_shell_rate=c.n(10000), tank_supports_rate=c.n(5000),
                 insulation_rate=c.n(8000), piping_rate=c.n(6000), outfitting_rate=c.n(7000),
                 cargo_system_lump=c.n(500000), paint_anodes_misc=c.n(200000),
                 battery_rate_per_kwh=c.n(400), propulsion_rate_per_kw=c.n(900),
                 wheelhouse_accom_lump=c.n(300000), tug_outfitting_lump=c.n(250000),
                 yard_margin_fraction=c.n(0.15), delivery_lump=c.n(50000))
    inp = b.BargeCostInputs(hull_steel_t=c.n(250), battery_kwh=c.n(600), propulsion_kw=c.n(230))
    return dict(
        default=b.compute(b.BargeCostInputs()),
        mapping=b.compute(inp, rates),
        object=b.compute(inp, b.BargeCostRates(**rates)),
        loaded=b.load_rates(rates),
        example=b.load_rates(),
        yaml=b.parse_flat_yaml("a: 1\n# note\nb: two\n"))


@scenario("vessel_standards", covers=[
    "core.vessel_standards.get_standard", "core.vessel_standards.standards_for",
    "core.vessel_standards.validate_catalogue", "core.vessel_standards.VesselStandard"])
def _(c):
    v = c.mod("core.vessel_standards")
    std = v.get_standard("CN_II_motor")
    return dict(
        standard=std, loa=value(std.loa_nominal_m), dwt=value(std.dwt_nominal_t),
        draught=value(std.draught_nominal_m), rmin=value(std.r_min_m),
        gas=[s.standard_id for s in v.standards_for("liquid_gas")],
        checks=v.validate_catalogue(),
        extra=v.validate_catalogue(dict(v.VESSEL_STANDARDS, X=odd_standard(c, v))))


def odd_standard(c, v):
    return v.VesselStandard("X", "CEMT", "X", "motor", (c.n(90), c.n(80)), c.n(11),
                            (c.n(2), c.n(3)), (c.n(1000), c.n(2000)))


@scenario("vessel_standards.custom_catalogue", covers=["core.vessel_standards.validate_catalogue"])
def _(c):
    # 0.1.0 checked any catalogue as given, even one without the built-in standards.
    v = c.mod("core.vessel_standards")
    only = {"X": odd_standard(c, v)}
    return [v.validate_catalogue(only), v.validate_catalogue({}),
            v.validate_catalogue({"Y": odd_standard(c, v)})]


@scenario("design_grid", covers=["core.design_grid.n_tanks_for_length", "core.design_grid.sweep",
                                 "core.design_grid.DesignGrid"])
def _(c):
    d = c.mod("core.design_grid")
    grid = d.sweep([c.n(60), c.n(90)], [c.n(10), c.n(14)], c.n(6), "LPG_propane")
    return dict(n=[d.n_tanks_for_length(c.n(60)), d.n_tanks_for_length(c.n(90), c.n(30))],
                grid=grid, curve=grid.curve(c.n(90)),
                default=d.sweep([c.n(70)], [c.n(12)]))


@scenario("checks.all_ok", covers=["core.checks.all_ok"])
def _(c):
    k = c.mod("core.checks")
    return [k.all_ok([k.Check("a", True, "m"), k.Check("b", False, "m", "warning")]),
            k.all_ok([k.Check("a", False, "m")])]


@scenario("simulate.cargo_options", covers=["simulate.cargo_options"])
def _(c):
    return c.mod("simulate").cargo_options()


def sim(c, **over):
    p = dict(CN_II, loa_m=c.n(90), beam_m=c.n(14.8), depth_m=c.n(6.5), draught_limit_m=c.n(4.4))
    p.update(over)
    return c.mod("simulate").simulate(p)


@scenario("simulate.gas", covers=["simulate.simulate"])
def _(c):
    return sim(c)


@scenario("simulate.gas_adn_channel", covers=["simulate.simulate"])
def _(c):
    return sim(c, ruleset="ADN_G", fill_ratio=None, draught_limit_m=None,
               channel=dict(depth_m=c.n(5), ukc_m=c.n(0.5), bottom_width_m=c.n(40)))


@scenario("simulate.gas_optimise_depth", covers=["simulate.simulate"])
def _(c):
    return sim(c, cargo="LNG", ruleset="IGC_2G_EXACT", fill_ratio=None, draught_limit_m=c.n(3),
               optimise_depth=True)


@scenario("simulate.box", covers=["simulate.simulate"])
def _(c):
    return c.mod("simulate").simulate(dict(
        vessel="box_barge", cargo="bauxite", loa_m=c.n(76.5), beam_m=c.n(11.4), depth_m=c.n(3.5),
        cb=c.n(0.928), lightship_k_t_per_m3=c.n(0.1093), freeboard_min_m=c.n(0.3)))


@scenario("simulate.hopper_channel", covers=["simulate.simulate"])
def _(c):
    return c.mod("simulate").simulate(dict(
        vessel="hopper_barge", cargo="bauxite", loa_m=c.n(80), beam_m=c.n(12), depth_m=c.n(4),
        cb=c.n(0.9), channel=dict(depth_m=c.n(5), ukc_m=c.n(1), bottom_width_m=c.n(40)),
        air_draft_limit_m=c.n(6), speed_max_kn=c.n(6)))


@scenario("simulate.wrong_vessel_for_cargo", covers=["simulate.simulate"])
def _(c):
    return c.mod("simulate").simulate(dict(
        vessel="box_barge", cargo="LNG", loa_m=c.n(80), beam_m=c.n(12), depth_m=c.n(4), cb=c.n(0.9)))


# --- an edited or rebuilt object is used as it is (issue #126) ------------------------------

@scenario("edited.stability_with_raked_hull", covers=["core.hydrostatics.calculate_stability"])
def _(c):
    bi, hull, tanks, weights = chain(c)
    raked = hull_of(c, bi, bow_rake_length=c.n(10), bow_rake_rise=c.n(3))
    return c.mod("core.hydrostatics").calculate_stability(
        raked, bi, tanks, weights, c.n(2000), c.n(0.58), c.n(3.5))


@scenario("edited.trim_with_raked_hull", covers=["core.hydrostatics.calculate_trim"])
def _(c):
    bi, hull, tanks, weights = chain(c)
    raked = hull_of(c, bi, bow_rake_length=c.n(10), bow_rake_rise=c.n(3))
    return c.mod("core.hydrostatics").calculate_trim(raked, bi, tanks, weights, c.n(2000), c.n(3.5))


@scenario("edited.stability_with_heavier_weights", covers=["core.hydrostatics.calculate_stability"])
def _(c):
    bi, hull, tanks, weights = chain(c)
    heavy = dataclasses.replace(weights, hull_steel_total=weights.hull_steel_total + 500,
                                lightship=weights.lightship + 500)
    return c.mod("core.hydrostatics").calculate_stability(
        hull, bi, tanks, heavy, c.n(2000), c.n(0.58), c.n(3.5))


@scenario("edited.stability_with_other_tanks", covers=["core.hydrostatics.calculate_stability"])
def _(c):
    bi, hull, tanks, weights = chain(c)
    other = dataclasses.replace(tanks, outer_diameter=tanks.outer_diameter - 1, cylinder_length=tanks.cylinder_length + 2)
    return c.mod("core.hydrostatics").calculate_stability(
        hull, bi, other, weights, c.n(2000), c.n(0.58), c.n(3.5))


@scenario("edited.weights_with_doubled_tank_shells", covers=["core.weights.calculate_weights"])
def _(c):
    bi, hull, tanks, weights = chain(c)
    doubled = dataclasses.replace(tanks, tank_shell_weight=tanks.tank_shell_weight * 2)
    return c.mod("core.weights").calculate_weights(bi, hull, doubled)


@scenario("edited.to_ga_with_other_draft", covers=["core.ga_adapter.to_ga"])
def _(c):
    inp, res = gas_capacity(c)
    res.full_load_draft_m = c.n(3.0)
    return c.mod("core.ga_adapter").to_ga(inp, res)


@scenario("edited.to_ga_with_rebuilt_result", covers=["core.ga_adapter.to_ga"])
def _(c):
    inp, res = gas_capacity(c)
    return c.mod("core.ga_adapter").to_ga(inp, dataclasses.replace(res, cargo_capacity_t=c.n(1000)))


@scenario("edited.journey_with_other_capacity", covers=["core.journey_derating.compute"])
def _(c):
    cap = bulk_capacity(c)[1]
    cap.cargo_capacity_t = c.n(3000)
    return [derate_of(c, cap), derate_of(c, cap, design_depth_m=c.n(5.5), ukc_m=c.n(0.5))]


@scenario("edited.journey_with_rebuilt_result", covers=["core.journey_derating.compute"])
def _(c):
    cap = dataclasses.replace(bulk_capacity(c)[1])
    return derate_of(c, cap, design_depth_m=c.n(5.5), ukc_m=c.n(0.5))


@scenario("edited.assess_with_rebuilt_result", covers=["core.gas_stability.assess"])
def _(c):
    res = dataclasses.replace(gas_capacity(c)[1])
    return c.mod("core.gas_stability").assess(res, c.n(1500))


@scenario("edited.assess_with_edited_basis", covers=["core.gas_stability.assess"])
def _(c):
    res = gas_capacity(c)[1]
    res.gas_basis["weights"].steel_top_down += c.n(100.0)
    res.cargo_density_t_per_m3 = c.n(0.45)
    return c.mod("core.gas_stability").assess(res, c.n(1500))


@scenario("edited.is_gas_with_rebuilt_result", covers=["core.ga_adapter.is_gas"])
def _(c):
    a = c.mod("core.ga_adapter")
    gas = dataclasses.replace(gas_capacity(c)[1])
    bulk = dataclasses.replace(bulk_capacity(c)[1])
    return [a.is_gas(gas), a.is_gas(bulk)]


@scenario("edited.is_gas_with_hand_built_result", covers=["core.ga_adapter.is_gas"])
def _(c):
    b = c.mod("core.barge_capacity")
    res = dataclasses.replace(bulk_capacity(c)[1], phase="liquid_gas")
    return [c.mod("core.ga_adapter").is_gas(res), isinstance(res, b.BargeCapacityResult)]


# --- one inputs object reused in a loop (issue #125) ----------------------------------------

@scenario("reuse.journey_over_a_sweep", covers=["core.journey_derating.compute", "core.barge_capacity.compute"])
def _(c):
    b = c.mod("core.barge_capacity")
    inp = b.BargeCapacityInputs(loa=c.n(60), beam=c.n(16), depth=c.n(6), cargo_type="bauxite")
    results = []
    for loa in (60, 80, 100):
        inp.loa = c.n(loa)
        results.append((loa, b.compute(inp)))
    return [derate_of(c, res, loa=c.n(loa), design_depth_m=c.n(5), ukc_m=c.n(0.5)) for loa, res in results]


@scenario("reuse.assess_over_a_sweep", covers=["core.gas_stability.assess"])
def _(c):
    b = c.mod("core.barge_capacity")
    g = c.mod("core.gas_stability")
    inp = b.BargeCapacityInputs(loa=c.n(90), beam=c.n(14.8), depth=c.n(6.5), cargo_type="LPG_propane")
    results = []
    for loa in (90, 70):
        inp.loa = c.n(loa)
        results.append(b.compute(inp))
    return [g.assess(res, c.n(1500)) for res in results]


@scenario("reuse.is_gas_and_to_ga_over_a_sweep", covers=["core.ga_adapter.is_gas", "core.ga_adapter.to_ga"])
def _(c):
    b = c.mod("core.barge_capacity")
    a = c.mod("core.ga_adapter")
    inp = b.BargeCapacityInputs(loa=c.n(90), beam=c.n(14.8), depth=c.n(6.5), cargo_type="LPG_propane")
    gas = b.compute(inp)
    gas_inp = dataclasses.replace(inp)
    inp.cargo_type = "bauxite"
    bulk = b.compute(inp)
    inp.loa = c.n(50)
    return [a.is_gas(gas), a.is_gas(bulk), a.to_ga(gas_inp, gas)]


@scenario("reuse.barge_inputs_in_a_loop",
          covers=["core.tanks.calculate_tank_arrangement", "core.weights.calculate_weights",
                  "core.hydrostatics.calculate_stability"])
def _(c):
    bi = barge_inputs(c)
    out = []
    for loa in (90, 80):
        bi.loa = c.n(loa)
        hull = hull_of(c, bi)
        tanks = c.mod("core.tanks").calculate_tank_arrangement(bi)
        weights = c.mod("core.weights").calculate_weights(bi, hull, tanks)
        out.append(c.mod("core.hydrostatics").calculate_stability(
            hull, bi, tanks, weights, c.n(2000), c.n(0.58), c.n(3.5)))
    return out
