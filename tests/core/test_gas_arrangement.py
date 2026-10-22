"""Benchmarks for the gas arrangement search, reproduced natively from an earlier project wrapper.

The river-side code depends on these numbers, so they are pinned here.
"""
import pytest

from vessel_designer.core.barge_capacity import BargeCapacityInputs, compute as capacity
from vessel_designer.core.gas_arrangement import fit, required_side_clearance, smallest_viable_loa


def test_cn_ii_benchmark_is_reproduced():
    """Example CN II gas figure: 90 x 14.8 x 6.5 m, propane, T 4.4 m, IGC 2G, fill 0.95.

    Re-pinned in issue #237 (the per-tank d(Vc) of IGC 2.4.1.1, datum of 2.4.2, dome in Vc) to
    2,943.44 t; it was 2,058 t under the stepped reading. Re-pinned again in issue #264 (the search
    keeps only arrangements that pass stability) to 2,832.66 t, and in issue #277 (the wheelhouse
    sightline blocks the search) to 2,021.25 t. Re-pinned again in issue #410 (the covering above D
    is in the lightship at its own heights): the one-wide arrangement fails the light-condition
    angle of max GZ, so the search takes two tanks wide, 1,405.85 t as the code prints it now. See
    the before/after table in CHANGELOG.md.

    Label: snapshot
    """
    f = fit(90.0, 14.8, "LPG_propane", 4.4, depths=[6.5], ruleset="IGC_2G", fill_ratio=0.95)
    # Re-pinned again (issue #498: the angle of max GZ alone in the light condition with no ballast is
    # a warning, so the one-wide arrangement is feasible again): 2,021.25 t, one tank wide.
    assert abs(f.payload_t - 2021.25) < 1.0
    assert f.n_tanks_wide == 1


def test_search_beats_the_auto_seed_on_cn_ii():
    """Defect 1: the engine's AUTO arrangement lands on the worse fixed point at 14.8 m beam.

    Label: property
    """
    # The old 12 m tank diameter cap: under the 16 m default one tank across is the AUTO answer.
    auto = capacity(BargeCapacityInputs(loa=90, beam=14.8, depth=6.5, cargo_type="LPG_propane",
                                        max_tank_od_m=12.0))
    best = fit(90.0, 14.8, "LPG_propane", 4.4, depths=[6.5], max_tank_od_m=12.0)
    # Issue #277: the wheelhouse sightline now blocks, so the search answer is lower (1,763 t); it
    # still beats the AUTO seed (1,406 t). The margin was 1.5 times. Issue #410: the one-wide
    # arrangement that beat the seed now fails the light-condition angle of max GZ, so the search
    # lands on the seed's own two-wide answer. It must never do worse than the seed.
    assert best.payload_t >= auto.cargo_capacity_t - 1e-9


@pytest.mark.parametrize("cap", [12.0, 16.0])
@pytest.mark.parametrize("L,B", [(60, 14.8), (90, 18.0), (100, 20.0)])
def test_capped_tank_respects_side_clearance(L, B, cap):
    """A tank put on the diameter cap keeps at least the code's side clearance for its volume.

    Label: property
    """
    f = fit(float(L), B, "LNG", 3.0, max_tank_od_m=cap)
    need = required_side_clearance("IGC_2G", B, f.containment_inner_volume_m3,
                                   f.per_tank_volume_m3, 350.0)
    assert f.tank_od_m <= cap + 1e-6
    assert f.side_clearance_m + 1e-6 >= need


def test_gas_barge_needs_a_hull_longer_than_its_stern_block():
    """Label: property
    """
    short = fit(22.0, 12.0, "LNG", 3.0)
    assert short.payload_t == 0 and short.binding == "no cargo area"
    viable = smallest_viable_loa(12.0, "LNG", 3.0)
    assert viable > short.stern_void_m
    assert fit(viable + 5, 12.0, "LNG", 3.0).payload_t > 0


def test_payload_never_exceeds_the_draught_cap():
    """Label: property
    """
    f = fit(90.0, 12.9, "LNG", 3.0)
    assert f.draft_at_payload_m <= 3.0 + 1e-6


# Regression snapshot: IGC_2G (before issue #237 named IGC_2G_EXACT; the dome in Vc moves these by
# less than 1 t, within the tolerance below), fill 0.98, T 3.0 m, self-propelled, LNG.
# 120 x 18 was re-pinned from 2,922 t (2 x 8.0 m tanks) to 2,813 t (1 x 11.0 m) by the
# pass-first fix (issue #6): the old design had 0.48 m side clearance, below an inland 0.80 m
# floor. Issue #215 removed that floor, so it is 2,922 t (2 x 8.0 m) again, the value of 0.1.0.
# These are the cases of the 12 m tank diameter cap, which was fixed until issue #175 made it
# an input (16 m by default), so each sets `max_tank_od_m=12.0`. The 16 m cases are below.
# (90, 12.9) re-pinned in issue #264 (stability gates the search): printed 1497.6289987038715; was 1505.
# Issue #277 (the wheelhouse sightline blocks; the wing tank width follows the IGC side clearance)
# re-pinned the others to the printed values 1691.1903238505604, 1875.1180321250488 and
# 2957.2540744793505; they were 1738, 2033 and 2922.
# Issue #410 (the covering above D is in the lightship; the one-wide arrangements fail the
# light-condition angle of max GZ) re-pinned two to the printed values 1129.9948047917674 and
# 1204.2783981854238; they were 1498 and 1691.
# Issue #448 (the hull's sections, not Cb) re-pinned two to the printed values 858.7067575154662 and
# 1889.6665938938233; they were 1130 and 1875.
# Issue #498 (the angle of max GZ alone in the light condition is a warning) re-pinned the first two
# to the printed values 1667.1735160692485 and 1691.1903238505604; they were 859 and 1204.
SNAPSHOT = [((90, 12.9), 1667), ((90, 14.8), 1691), ((90, 18.0), 1890), ((120, 18.0), 2957)]


@pytest.mark.parametrize("dims,expected", SNAPSHOT)
def test_regression_snapshot(dims, expected):
    """Label: snapshot
    """
    L, B = dims
    f = fit(float(L), B, "LNG", 3.0, ruleset="IGC_2G", fill_ratio=0.98, max_tank_od_m=12.0)
    assert abs(f.payload_t - expected) < 1.0, f.payload_t


def test_snapshot_120x18_is_two_8m_tanks_abreast():
    """Issue #6 moved this to one 11 m tank because the 2 x 8 m design broke an inland type-G side
    floor (0.48 m clearance). Issue #215 removed the floor: the search picks 2 x 8 m again.
    Issue #277 moved the diameter from 8.017 m to the printed 8.01586 m.

    Label: snapshot
    """
    f = fit(120.0, 18.0, "LNG", 3.0, ruleset="IGC_2G", fill_ratio=0.98, max_tank_od_m=12.0)
    assert f.feasible
    assert f.n_tanks_wide == 2 and abs(f.tank_od_m - 8.016) < 0.001


# The same hulls under the default 16 m cap. Values printed by this code when the cap moved
# (issue #175), not by the frozen Python; the before/after table is in CHANGELOG.md. The 120 x 18
# value moved again with issue #215 (the inland type-G side floor went); the printed value is pinned.
# Issue #237 (the dome now counts in Vc) moved both again, by under 0.1 t: re-pinned to the printed
# values; they were 1755.1158212773869 and 2921.7916111312247.
# (90, 14.8) re-pinned in issue #264 (stability gates the search): printed 1692.4687453268486; was
# 1755.2051149886904.
# Issue #277 (the wheelhouse sightline blocks; the wing tank width follows the IGC side clearance)
# re-pinned both to the printed values; they were 1692.4687453268486 and 2921.887266646023.
# Issue #410 (the covering above D is in the lightship) re-pinned (90, 14.8) to the printed value;
# it was 1614.9482417833697.
# Issue #498 re-pinned (90, 14.8) to the printed value 1614.9482417833697.
SNAPSHOT_16 = [((90, 14.8), 1614.9482417833697), ((120, 18.0), 2957.2540744793505)]


@pytest.mark.parametrize("dims,expected", SNAPSHOT_16)
def test_regression_snapshot_with_the_default_16_m_cap(dims, expected):
    """Label: snapshot
    """
    L, B = dims
    f = fit(float(L), B, "LNG", 3.0, ruleset="IGC_2G", fill_ratio=0.98)
    assert f.engine_inputs.max_tank_od_m == 16.0
    assert abs(f.payload_t - expected) < 1e-9, f.payload_t


def test_cn_ii_free_depth_picks_a_passing_hull():
    """Issue #6: CN II at a 4.4 m draught limit, depth free. The top-tonnage candidate was a
    3.5 m-deep hull at 3.50 m draught -- zero freeboard, overloaded. It must not win.

    Label: property
    """
    f = fit(90.0, 14.8, "LPG_propane", 4.4, ruleset="IGC_2G", fill_ratio=0.95)
    assert f.feasible
    assert not any(m.startswith(("not_overloaded", "freeboard_ok")) for m in f.failed_checks)
    assert f.depth_m - f.draft_at_payload_m >= 0.3 - 1e-6       # the engine's minimum freeboard
    assert f.draft_at_payload_m <= 4.4 + 1e-6
    assert not (f.depth_m == 3.5 and abs(f.draft_at_payload_m - 3.5) < 1e-3)


def test_the_design_that_failed_only_the_type_g_floor_now_passes():
    """Issue #215: the 0.48 m-clearance 2-abreast design failed only the inland type-G side
    floor, which is gone. Its feasibility holds and no check carries a type-G name.

    Label: property
    """
    f = fit(120.0, 18.0, "LNG", 3.0, ruleset="IGC_2G", fill_ratio=0.98,
            depths=[5.75], tanks_wide=(2,), extra_clearance_m=())
    assert f.feasible
    assert f.failed_checks == []


def test_nothing_fits_is_infeasible():
    """Label: property
    """
    assert not fit(22.0, 12.0, "LNG", 3.0).feasible


def test_adn_g_is_gone_and_only_igc_2g_is_left():
    """Issue #175: the engine applies IGC rules only to gas tanks. Issue #237: one rule set, IGC_2G
    (the per-tank d(Vc) of IGC 2.4.1.1); IGC_2G_EXACT is not offered.

    Label: property
    """
    from vessel_designer.core import gas_arrangement
    assert gas_arrangement.RULESETS == ("IGC_2G",)
    with pytest.raises(ValueError, match=r"ruleset must be one of \('IGC_2G',\)"):
        fit(110.0, 11.4, "LNG", 3.0, ruleset="ADN_G")
    with pytest.raises(ValueError, match=r"ruleset must be one of \('IGC_2G',\)"):
        fit(110.0, 11.4, "LNG", 3.0, ruleset="IGC_2G_EXACT")
    with pytest.raises(TypeError):
        fit(110.0, 11.4, "LNG", 3.0, adn_tank_cap_m3=380.0)


def test_the_tank_diameter_cap_is_an_argument_16_m_by_default():
    """Label: property
    """
    import inspect
    assert inspect.signature(fit).parameters["max_tank_od_m"].default == 16.0
    for ruleset in ("IGC_2G",):
        wide = {cap: fit(110.0, 26.0, "LNG", 4.0, depths=[5.0], tanks_wide=(1,), ruleset=ruleset,
                         max_tank_od_m=cap)
                for cap in (12.0, 16.0)}
        assert all(f.tank_od_m <= cap + 1e-6 for cap, f in wide.items())
        assert wide[16.0].tank_od_m > 12.0 >= wide[12.0].tank_od_m - 1e-6
        assert wide[12.0].engine_inputs.max_tank_od_m == 12.0


def test_no_feasible_fit_has_a_tank_longer_than_seven_diameters():
    """A fit with L/D > 7 is not accepted, under the rule set. The sweep must refuse some hulls
    for it, or it proves nothing.

    Label: property
    """
    accepted = refused = 0
    for ruleset in ("IGC_2G",):
        # Two lengths more than before issue #237, which had two rule sets in this sweep and so
        # twice the cases; the thresholds below are unchanged. Issue #448: on the hull's sections
        # more hulls fail the angle of maximum GZ (#498), so the sweep adds an 80 m length and a
        # 14 m beam to keep more than 5 accepted.
        for loa in (60.0, 80.0, 100.0, 150.0, 180.0, 210.0, 240.0):
            for beam in (8.0, 12.0, 14.0, 18.0, 26.0):
                f = fit(loa, beam, "LNG", 4.0, depths=[5.0], ruleset=ruleset)
                tank = f.engine_result.gas_basis["tanks"]
                slenderness = tank.overall_tank_length / tank.outer_diameter
                if f.feasible and f.payload_t > 0:
                    accepted += 1
                    assert slenderness <= 7.0, (ruleset, loa, beam, slenderness)
                elif slenderness > 7.0:
                    refused += 1
                    assert any(m.startswith("tank_length_over_diameter") for m in f.failed_checks)
    assert accepted > 5 and refused > 2


def test_settings_are_arguments_not_globals():
    """Label: property
    """
    a = fit(90.0, 14.8, "LNG", 3.0, ruleset="IGC_2G", fill_ratio=0.98)
    b = fit(90.0, 14.8, "LNG", 3.0, ruleset="IGC_2G", fill_ratio=0.95)
    c = fit(90.0, 14.8, "LNG", 3.0, ruleset="IGC_2G", fill_ratio=0.98)
    assert a.payload_t == c.payload_t != b.payload_t
