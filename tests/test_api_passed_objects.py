"""Functions use the object the caller passes (issue #126).

`weights`, `hydrostatics`, `ga_adapter`, `journey_derating` and `gas_stability.assess` take a hull,
tank result, weight breakdown or capacity result. 0.1.0 used the object it was given, even when the
caller had built it by hand or edited it. Every expected value below was printed by the frozen
Python at commit ce96e66. Floats are compared with the tolerance of spec 0001 (relative and absolute
1e-9), the one the parity harness uses; it is not loosened here.
"""
import dataclasses

from pytest import approx

from vessel_designer.core import barge_capacity as B
from vessel_designer.core import ga_adapter, gas_stability, journey_derating as J
from vessel_designer.core import hull as H
from vessel_designer.core import hydrostatics as HY
from vessel_designer.core import models as M
from vessel_designer.core import tanks as T
from vessel_designer.core import weights as W

BI = M.BargeInputs(loa=90.0, beam=14.8, depth=6.5, max_draft=4.4, vertical_clearance=10.0)
HULL = H.HullGeometry(BI.loa, BI.beam, BI.depth, BI.bow_rake_length, BI.bow_rake_rise,
                      BI.stern_rake_length, BI.stern_rake_rise)
RAKED = H.HullGeometry(BI.loa, BI.beam, BI.depth, 10.0, 3.0, BI.stern_rake_length,
                       BI.stern_rake_rise)
TANKS = T.calculate_tank_arrangement(BI)
WEIGHTS = W.calculate_weights(BI, HULL, TANKS)
# The 12 m tank diameter cap of 0.1.0: the cap is now an input, 16 m by default (issue #175), and the
# numbers below were printed by 0.1.0.
GAS = B.BargeCapacityInputs(loa=90.0, beam=14.8, depth=6.5, cargo_type="LPG_propane", max_tank_od_m=12.0)
BULK = B.BargeCapacityInputs(loa=100.0, beam=16.0, depth=6.0, cargo_type="bauxite")


def close(x):
    return approx(x, rel=1e-9, abs=1e-9)


def derate(capacity):
    return J.compute(J.JourneyDeratingInputs(capacity=capacity, loa=100.0, beam=16.0, depth=6.0,
                                             design_depth_m=5.5, ukc_m=0.5))


def gm_fluid(result, payload=1500.0):
    return tuple(c.gm_fluid_m for c in gas_stability.assess(result, payload).conditions)


def test_stability_uses_the_hull_it_is_given():
    """Label: snapshot"""
    s = HY.calculate_stability(RAKED, BI, TANKS, WEIGHTS, 2000.0, 0.58, 3.5)
    # Re-pinned (issue #413: the free surface takes the fill as a volume ratio; issue #410: hull
    # steel to D, the covering above D in the lightship); was 0.05969187822861491 at ce96e66.
    assert s.GM_fluid == close(-0.7757945091720747)


def test_trim_uses_the_hull_it_is_given():
    """Label: snapshot"""
    t = HY.calculate_trim(RAKED, BI, TANKS, WEIGHTS, 2000.0, 3.5)
    # Re-pinned (issue #410); was 1.3653183822264685 at ce96e66.
    assert t.trim == close(1.4585011885805368)


def test_stability_uses_the_weights_it_is_given():
    """Label: snapshot"""
    heavier = dataclasses.replace(WEIGHTS, hull_steel_total=WEIGHTS.hull_steel_total + 500,
                                  lightship=WEIGHTS.lightship + 500)
    # Re-pinned (issue #410); was 6.59527868136034 at ce96e66.
    assert HY.calculate_stability(HULL, BI, TANKS, heavier, 2000.0, 0.58, 3.5).KG == close(6.6034575199425944)


def test_weights_use_the_tanks_they_are_given():
    """Label: snapshot"""
    doubled = dataclasses.replace(TANKS, tank_shell_weight=TANKS.tank_shell_weight * 2)
    # Re-pinned (issue #410: the hull steel is weighed to D, not to the wing plate); was
    # 2814.5959265880624 at ce96e66.
    assert W.calculate_weights(BI, HULL, doubled).lightship == close(2502.322865499698)


def test_to_ga_uses_the_result_it_is_given():
    """Label: property"""
    res = B.compute(GAS)
    res.full_load_draft_m = 3.0
    gi, gr = ga_adapter.to_ga(GAS, res)
    assert (gi.max_draft, gr.draft) == close((3.0, 3.0))                   # ce96e66


def test_journey_derating_uses_the_capacity_it_is_given():
    """Label: snapshot"""
    cap = B.compute(BULK)
    cap.cargo_capacity_t = 3000.0
    d = derate(cap)
    assert (d.cargo_capacity_t, d.achievable_tonnage_t) == close((3000.0, 3000.0))   # ce96e66
    assert d.binding_limit == "volume"


def test_a_result_rebuilt_with_replace_still_works():
    # ce96e66: a rebuilt result carries all its fields, so every function accepts it
    """Label: snapshot"""
    d = derate(dataclasses.replace(B.compute(BULK)))
    assert (d.cargo_capacity_t, d.achievable_tonnage_t) == close((4710.765684, 4710.765684))
    gas = dataclasses.replace(B.compute(GAS))
    # Re-pinned (issue #237); were (4.496070691510841, 6.706438965623047, 11.201087654316286) at ce96e66.
    assert gm_fluid(gas) == close((4.447528112519484, 6.499494396532422, 10.72355412577273))
    assert ga_adapter.is_gas(gas) is True


def test_assess_uses_the_gas_basis_and_density_it_is_given():
    """Label: snapshot"""
    res = B.compute(GAS)
    res.gas_basis["weights"].steel_top_down += 100.0
    # Re-pinned (issue #237); were (4.329311723992825, 6.3607094944568106, 10.35664347102438) and
    # (4.329311723992825, 6.416895514461225, 10.35664347102438) at ce96e66.
    assert gm_fluid(res) == close((4.28684718522825, 6.172666462858952, 9.940701176338345))
    res.cargo_density_t_per_m3 = 0.45
    assert gm_fluid(res) == close((4.28684718522825, 6.245181641069995, 9.940701176338345))
