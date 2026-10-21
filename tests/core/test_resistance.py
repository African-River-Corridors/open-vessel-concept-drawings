"""Resistance & propulsion power — golden, property and feasibility checks.

The free compute() wraps the unchanged ResistanceCalculator and attaches validity
checks. Golden values freeze the confined-shallow (binding) environment at the default
barge.
"""
from vessel_designer.core.resistance import (
    ResistanceCalculator,
    ResistanceInputs,
    EnvironmentInputs,
    TugInputs,
    default_environments,
    compute,
)


def _shallow(r):
    return next(e for e in r.environments if e.name == "Confined shallow river")


# ---------- golden: confined-shallow (binding) environment, default barge ----------

def test_resistance_golden_confined_shallow():
    """RE-BASELINED when squat entered the model. This is a CORRECTION, not a
    retune: service speed was 9.0 kn and 2,218.9 kW, computed from resistance alone. At 9 kn
    this vessel squats far into a 0.50 m under-keel clearance, so that speed was never
    available. It is now 2.5 kn and 66.2 kW.

    The default case is also, on inspection, physically extreme — see the validity test
    below — so treat these numbers as a regression anchor, not as a design answer.

    Label: snapshot
    """
    r = compute(ResistanceInputs())
    assert abs(r.wetted_surface_area - 4590.0) < 1.0
    a = _shallow(r)
    # Re-pinned (issues #396, #397): the Schijf first critical speed on B x T; was 10.828.
    assert abs(a.limiting_speed_knots - 3.799) < 0.01
    assert a.service_speed_knots == 2.5
    assert a.service_speed_governed_by == "squat"
    # Re-pinned (issues #396, #397, the blockage term on B x T): 66.6, 16.65, 15.37 and 6.148; were
    # 66.2, 16.54, 15.27 and 6.109.
    assert abs(a.power_at_service_kw - 66.6) < 1.0         # delivered power -> battery feed
    assert abs(a.resistance_at_service_kn - 16.65) < 0.1
    assert abs(a.fuel_at_service_lph - 15.37) < 0.5
    assert abs(a.fuel_at_service_lpnm - 6.148) < 0.05


def test_feasibility_the_default_case_is_outside_barrass_validity():
    """The default pairs a 130 x 24 m hull at 6.0 m draft with a 40 m wide, 6.5 m deep
    channel: blockage 0.44 and 0.50 m of water under the keel. Barrass is fitted below about
    0.35. The old model returned 9 knots here without comment; the least this one can do is
    say the combination is not physical.

    Label: property
    """
    from vessel_designer.core.squat import SquatInputs, compute as squat
    i = ResistanceInputs()
    area = (40.0 + 60.0) / 2 * 6.5
    sr = squat(SquatInputs(speed_through_water_kn=4.0, draft_m=i.draft, beam_m=i.beam,
                           lpp_m=i.lwl, block_coefficient=i.block_coefficient,
                           water_depth_m=6.5, channel_area_m2=area))
    assert sr.blockage_ratio > 0.4, sr.blockage_ratio
    assert not [c for c in sr.checks if c.name == "blockage_within_barrass_range"][0].ok


def test_binding_environment_is_confined_shallow_and_feeds_battery():
    # The single number that leaves the module for battery sizing.
    """Label: snapshot
    """
    r = compute(ResistanceInputs())
    assert r.binding_environment.name == "Confined shallow river"   # lowest service speed
    assert abs(r.binding_power_at_service_kw - 66.2) < 1.0
    assert r.binding_power_at_service_kw == _shallow(r).power_at_service_kw


def test_checks_pass_on_default_design():
    """Label: property
    """
    r = compute(ResistanceInputs())
    assert r.ok                                            # no error-severity failures
    names = {c.name for c in r.checks}
    assert "coefficients_are_first_order" in names
    advisory = next(c for c in r.checks if c.name == "coefficients_are_first_order")
    assert advisory.severity == "warning" and advisory.ok
    # the legacy entry point stays check-free: checks are the additive change in compute()
    assert ResistanceCalculator(ResistanceInputs()).calculate().checks == []


# ---------- property: the physics that must always hold ----------

def test_resistance_monotone_in_speed():
    """Label: property
    """
    r = compute(ResistanceInputs())
    a = _shallow(r)
    totals = [p.resistance_total_kn for p in a.speed_curve if p.speed_through_water_knots > 0]
    assert all(b >= x - 1e-9 for x, b in zip(totals, totals[1:]))


def _shallow_factor(env, knots=8.0):
    """The shallow-water resistance factor of the first speed point of one environment."""
    r = compute(ResistanceInputs(environments=[env], speed_min_knots=knots,
                                 speed_max_knots=knots, speed_step_knots=0.5))
    return r.environments[0].speed_curve[0].resistance_shallow_factor


def test_shallow_factor_at_least_one_and_grows_as_depth_falls():
    # Compare the shallow factor at a fixed speed for two depths.
    """Label: property
    """
    deep = EnvironmentInputs(name="d", water_depth=20.0, channel_width_bottom=200.0,
                             channel_width_surface=200.0, water_density=1.0)
    shallow = EnvironmentInputs(name="s", water_depth=8.0, channel_width_bottom=200.0,
                                channel_width_surface=200.0, water_density=1.0)
    f_deep, f_shallow = _shallow_factor(deep), _shallow_factor(shallow)
    assert f_deep >= 1.0 and f_shallow >= 1.0
    assert f_shallow > f_deep


def test_deep_water_factor_is_one():
    """Label: property
    """
    env = EnvironmentInputs(name="open", is_deep_water=True)
    assert _shallow_factor(env, knots=10.0) == 1.0


def test_service_speed_falls_as_channel_gets_shallower():
    """Label: property
    """
    def svc(depth):
        env = EnvironmentInputs(name="x", water_depth=depth, channel_width_bottom=40.0,
                                channel_width_surface=60.0, current_speed_knots=1.0,
                                water_density=1.0)
        r = compute(ResistanceInputs(environments=[env]))
        return r.environments[0].service_speed_knots
    assert svc(10.0) >= svc(7.0) >= svc(5.0)


# ---------- feasibility: checks fire when they should ----------

def test_service_speed_check_passes_within_schijf():
    # Default river env: sweep capped at 95% of Schijf, so the check holds.
    """Label: property
    """
    r = compute(ResistanceInputs())
    svc_checks = [c for c in r.checks if c.name.startswith("service_speed_below_limiting")]
    assert svc_checks and all(c.ok for c in svc_checks)


def test_blockage_in_range_fires_for_overtight_channel():
    # A channel barely wider than the barge -> n -> 1 (or am >= ac) -> check fails.
    # midship area = 24*6*0.85 = 122.4 m^2; this channel gives ac ~ 110 m^2 -> n > 1.
    """Label: property
    """
    tight = EnvironmentInputs(name="tight", water_depth=6.5, channel_width_bottom=16.0,
                              channel_width_surface=18.0, current_speed_knots=0.0,
                              water_density=1.0)
    r = compute(ResistanceInputs(beam=24.0, draft=6.0, environments=[tight]))
    block_checks = [c for c in r.checks if c.name.startswith("blockage_in_range")]
    assert block_checks
    assert not all(c.ok for c in block_checks)             # over-tight -> fires
    assert not r.ok


def test_power_within_installed_fires_when_underpowered():
    # If even the slowest swept speed needs more than the installed power, no speed is
    # power_available; the service point falls back to the slowest, whose delivered power
    # exceeds the engine -> power_within_installed fires. ~5 kW installed forces this
    # (slowest-speed delivered power is ~40 kW).
    """Label: property
    """
    weak = TugInputs(engine_power_kw=5.0)
    r = compute(ResistanceInputs(tug=weak))
    pwr_checks = [c for c in r.checks if c.name.startswith("power_within_installed")]
    assert pwr_checks
    assert not all(c.ok for c in pwr_checks)
    assert not r.ok


def test_power_within_installed_passes_when_low_speed_feasible():
    # A modest engine still finds a feasible slow service speed -> check passes.
    """Label: property
    """
    r = compute(ResistanceInputs(tug=TugInputs(engine_power_kw=50.0)))
    pwr_checks = [c for c in r.checks if c.name.startswith("power_within_installed")]
    assert pwr_checks and all(c.ok for c in pwr_checks)
