"""One inputs object reused across several `compute` calls (issue #125).

A capacity result must not change when the caller edits the inputs object afterwards. Expected
values are printed by the original Python at commit ce96e66 (the frozen package under
`parity/frozen_python` in that commit).
"""
from vessel_designer.core import barge_capacity as B
from vessel_designer.core import ga_adapter, gas_stability, journey_derating as J


def test_journey_derating_after_the_inputs_object_was_reused():
    # ce96e66: capacity, achievable tonnage per loa
    """Label: snapshot"""
    expected = {60.0: (1999.8456839999994, 1999.8456839999994),
                80.0: (3355.305684, 3355.305684),
                100.0: (4710.765684, 4710.765684)}  # Re-pinned (#448); was 4569.524600031882
    inp = B.BargeCapacityInputs(loa=60.0, beam=16.0, depth=6.0, cargo_type="bauxite")
    results = []
    for loa in expected:
        inp.loa = loa
        results.append((loa, B.compute(inp)))
    for loa, res in results:
        d = J.compute(J.JourneyDeratingInputs(capacity=res, loa=loa, beam=16.0, depth=6.0,
                                              design_depth_m=5.0, ukc_m=0.5))
        assert (d.cargo_capacity_t, d.achievable_tonnage_t) == expected[loa]


def test_gas_stability_assess_after_the_inputs_object_was_reused():
    # ce96e66: GM fluid of the Laden, Half laden and Light conditions, payload 1500 t
    """Label: snapshot"""
    # Re-pinned (issue #237); were (4.496070691510841, 6.706438965623047, 11.201087654316286) and
    # (3.929578290990053, 6.178009228238304, 12.235044383044745) at ce96e66.
    # Re-pinned (#448: the hull's sections); were (4.447528112519484, 6.499494396532422,
    # 10.72355412577273) and (3.9244500960933397, 6.352237203882345, 11.782312192191762).
    # Re-pinned (issue #280: the end space defaults to the type G rule); were (5.010972757840445,
    # 7.1794700173060155, 11.60946081779351) and (4.368132024606091, 6.823567215928239,
    # 12.28163398784297).
    # Re-pinned again (issue #381: the opening setback is measured from the cargo area).
    # Re-pinned (#567: the equilibrium solves stop at a tolerance); were 5.008782640783497 (90 m,
    # Laden) and 6.817748082762562 (70 m, Half laden). The values do not depend on the order or the
    # number of earlier calls.
    expected = {90.0: (5.008782640783499, 7.174444292789616, 11.599547988722012),
                70.0: (4.366034263765853, 6.8177480827625585, 12.267290419878801)}
    # The 12 m tank diameter cap of 0.1.0 (it is an input, 16 m by default, since issue #175).
    inp = B.BargeCapacityInputs(loa=90.0, beam=14.8, depth=6.5, cargo_type="LPG_propane",
                                max_tank_od_m=12.0)
    results = []
    for loa in expected:
        inp.loa = loa
        results.append((loa, B.compute(inp)))
    for loa, res in results:
        a = gas_stability.assess(res, 1500.0)
        assert tuple(c.gm_fluid_m for c in a.conditions) == expected[loa]


def test_is_gas_after_the_inputs_object_was_reused():
    # ce96e66: is_gas reads the result's phase
    """Label: snapshot"""
    inp = B.BargeCapacityInputs(loa=90.0, beam=14.8, depth=6.5, cargo_type="LPG_propane")
    gas = B.compute(inp)
    inp.cargo_type = "bauxite"
    bulk = B.compute(inp)
    inp.cargo_type = "LPG_propane"
    assert ga_adapter.is_gas(gas) is True
    assert ga_adapter.is_gas(bulk) is False


def test_gas_stability_does_not_depend_on_earlier_calls():
    """The equilibrium solves start from the same state whatever ran before (issue #567): the same
    inputs give the same bits in any order and after any number of earlier calls.

    Label: property"""
    def run(order):
        inp = B.BargeCapacityInputs(loa=90.0, beam=14.8, depth=6.5, cargo_type="LPG_propane",
                                    max_tank_od_m=12.0)
        res = {}
        for loa in order:
            inp.loa = loa
            res[loa] = B.compute(inp)
        return {loa: tuple(c.gm_fluid_m for c in gas_stability.assess(res[loa], 1500.0).conditions)
                for loa in sorted(order)}
    assert run([90.0, 70.0]) == run([70.0, 90.0]) == run([90.0, 70.0, 90.0, 70.0])
