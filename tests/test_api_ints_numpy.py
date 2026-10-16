"""Whole numbers and numpy numbers where the API takes floats (issue #127).

0.1.0 (the pure-Python package, commit ce96e66) accepted ints, numpy ints and numpy floats for any
float field, and a whole float or a numpy int for an int field. Every expected value below was
printed by the frozen Python at ce96e66. The comparison is exact (`==`; `90 == 90.0` holds).
"""
import numpy as np
import pytest

from vessel_designer.core import barge_capacity as B
from vessel_designer.core import hull as H
from vessel_designer.core import hydrostatics as HY
from vessel_designer.core import journey_derating as J

# ce96e66: HullGeometry(90, 15, 7, 0, 7, 0, 7), every method at draft 3 (area methods take none)
HULL_AT_DRAFT_3 = {
    "displaced_volume": 4050.0, "waterplane_area": 1350.0, "waterplane_length": 90.0,
    "waterplane_inertia_transverse": 25312.5, "waterplane_inertia_longitudinal": 911250.0,
    "KB": 1.4999999999999998, "LCB": 45.0, "BM_transverse": 6.25, "BM_longitudinal": 225.0,
}
HULL_AREAS = {
    "bottom_area": 1350, "side_area": 1260.0, "deck_area": 1350, "bow_rake_area": 0.0,
    "stern_rake_area": 0.0, "total_hull_surface_area": 3960.0,
}
INT_KINDS = {"int": int, "np.int64": np.int64, "np.float64": np.float64, "np.int32": np.int32}


@pytest.mark.parametrize("kind", INT_KINDS)
@pytest.mark.parametrize("method", HULL_AT_DRAFT_3)
def test_hull_methods_take_ints_and_numpy_numbers(kind, method):
    """Label: snapshot"""
    make = INT_KINDS[kind]
    hull = H.HullGeometry(*(make(v) for v in (90, 15, 7, 0, 7, 0, 7)))
    assert getattr(hull, method)(make(3)) == HULL_AT_DRAFT_3[method]


@pytest.mark.parametrize("kind", INT_KINDS)
@pytest.mark.parametrize("method", HULL_AREAS)
def test_hull_area_methods_take_ints_and_numpy_numbers(kind, method):
    """Label: snapshot"""
    make = INT_KINDS[kind]
    hull = H.HullGeometry(*(make(v) for v in (90, 15, 7, 0, 7, 0, 7)))
    assert getattr(hull, method)() == HULL_AREAS[method]
    assert hull.parallel_length == 90


def test_the_ticket_example():
    # ce96e66: 4050.0
    """Label: snapshot"""
    assert H.HullGeometry(90, 15, 7, 0, 7, 0, 7).displaced_volume(3.0) == 4050.0


@pytest.mark.parametrize("kind", INT_KINDS)
def test_journey_derating_takes_ints_and_numpy_numbers(kind):
    """Label: snapshot"""
    make = INT_KINDS[kind]
    cap = B.compute(B.BargeCapacityInputs(loa=100.0, beam=16.0, depth=6.0, cargo_type="bauxite"))
    d = J.compute(J.JourneyDeratingInputs(capacity=cap, loa=make(100), beam=make(16), depth=make(6),
                                          design_depth_m=make(5), ukc_m=make(1)))
    # ce96e66: achievable 3889.5246000318816 t, draft-bound, floated at 4
    assert (d.cargo_capacity_t, d.achievable_tonnage_t, d.binding_limit,
            d.draft_at_achievable_m) == (4710.765684, 3889.5246000318816, "draft", 4)


def test_equilibrium_draft_takes_numpy_numbers():
    """Label: snapshot"""
    hull = H.HullGeometry(90, 15, 7, 0, 7, 0, 7)
    # ce96e66: 5.92591667175293 (block coefficient np.float32(0.5), exactly 0.5)
    assert HY.find_equilibrium_draft(hull, np.int64(4000), 1, 7, np.float32(0.5)) == 5.92591667175293


@pytest.mark.parametrize("kind", INT_KINDS)
def test_capacity_takes_ints_and_numpy_numbers(kind):
    """Label: snapshot"""
    make = INT_KINDS[kind]
    r = B.compute(B.BargeCapacityInputs(loa=make(100), beam=make(16), depth=make(6),
                                        cargo_type="bauxite"))
    assert r.cargo_capacity_t == 4710.765684                         # ce96e66


@pytest.mark.parametrize("n_holds", [4, 4.0, np.int64(4), np.float64(4.0)])
def test_capacity_takes_a_whole_number_of_holds_in_any_form(n_holds):
    """Label: snapshot"""
    r = B.compute(B.BargeCapacityInputs(loa=100.0, beam=16.0, depth=6.0, cargo_type="bauxite",
                                        n_holds=n_holds))
    assert r.cargo_capacity_t == 4710.765684                          # ce96e66


@pytest.mark.parametrize("wide,long", [(2, 2), (np.int64(2), 2.0), (2.0, np.int64(2))])
def test_capacity_takes_whole_tank_counts_in_any_form(wide, long):
    """Label: snapshot"""
    r = B.compute(B.BargeCapacityInputs(loa=100, beam=16, depth=6, cargo_type="LNG",
                                        n_tanks_wide=wide, n_tanks_long=long))
    # Re-pinned (issue #237: the per-tank d(Vc) of IGC 2.4.1.1, dome in Vc); was 1347.0976687711673
    # (ce96e66) under the stepped IGC_2G.
    assert (r.cargo_capacity_t, r.n_tanks_wide) == (1607.3557980997896, 2)


def test_a_float32_input_computes_to_float32_accuracy():
    # 0.1.0 did the sums in float32 here, so its answer (4710.766) is not the float64 one. Only
    # the type of the answer differs; the value agrees to float32 accuracy.
    """Label: snapshot"""
    r = B.compute(B.BargeCapacityInputs(loa=np.float32(100), beam=16.0, depth=6.0,
                                        cargo_type="bauxite"))
    assert abs(r.cargo_capacity_t - 4710.766) < 1e-3
