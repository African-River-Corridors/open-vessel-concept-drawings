"""The web simulator's gas path goes through the arrangement search and hits the benchmarks."""
from vessel_designer.simulate import simulate

# `max_tank_od_m` is the 12 m tank diameter cap of 0.1.0: it is an input, 16 m by default, since
# issue #175. The snapshots below were printed by 0.1.0, so they set it.
CN_II = dict(vessel="gas_barge", cargo="LPG_propane", loa_m=90, beam_m=14.8, depth_m=6.5, cb=0.85,
             freeboard_min_m=0.3, draught_limit_m=4.4, self_propelled=True, ruleset="IGC_2G",
             fill_ratio=0.95, max_tank_od_m=12.0)


def test_gas_uses_the_search_not_auto():
    """Label: snapshot
    """
    # Re-pinned (issue #237: the per-tank d(Vc) of IGC 2.4.1.1, dome in Vc); was 2058 under the
    # stepped IGC_2G.
    assert abs(simulate(CN_II)["results"]["cargo_t"] - 2535) < 1.0


def test_gas_optimised_depth_matches_snapshot():
    """Label: snapshot
    """
    o = simulate(dict(CN_II, cargo="LNG", draught_limit_m=3.0, ruleset="IGC_2G",
                      fill_ratio=None, optimise_depth=True))
    assert abs(o["results"]["cargo_t"] - 1738) < 1.0
    assert o["arrangement"]["fill_ratio"] == 0.98          # ruleset-driven default


def test_the_tank_diameter_cap_is_a_simulate_input_16_m_by_default():
    """Label: property
    """
    default = {k: v for k, v in CN_II.items() if k != "max_tank_od_m"}
    # Issue #237: the per-tank rule asks for less side clearance, so on a 26 m beam the search
    # prefers two tanks abreast and the cap does not bind. At the CN II beam, 14.8 m, one tank
    # fills the beam and the cap sets its diameter (13.7 m at the default cap).
    wide = dict(default, beam_m=14.8, loa_m=90, depth_m=6.5, cargo="LNG", draught_limit_m=4.4)
    assert simulate(dict(wide, max_tank_od_m=12.0))["geometry"]["tank_outer_diameter_m"] <= 12.0 + 1e-6
    od = simulate(wide)["geometry"]["tank_outer_diameter_m"]
    assert 12.0 < od <= 16.0 + 1e-6


def test_a_tank_longer_than_seven_diameters_is_a_failed_check():
    """Label: property
    """
    o = simulate(dict(CN_II, loa_m=200, beam_m=11.0, depth_m=5.0, cargo="LNG", draught_limit_m=4.0))
    failed = [c for c in o["checks"] if c["name"] == "tank length over diameter"]
    assert failed and not failed[0]["ok"] and "<= 7" in failed[0]["message"]
    assert not o["arrangement"]["feasible"]


def test_gas_stability_is_assessed_for_three_conditions():
    """Label: property
    """
    o = simulate(CN_II)
    assert [c["name"] for c in o["stability"]["conditions"]] == ["Laden", "Half laden", "Light"]
    assert o["stability"]["ok"]
    assert sum(c["name"].startswith("Stability") for c in o["checks"]) == 3


def test_tender_gas_barge_shows_failed_stability_checks():
    """Label: property
    """
    o = simulate(dict(CN_II, loa_m=60, beam_m=7.0, depth_m=6.0, draught_limit_m=None,
                      ruleset="IGC_2G", fill_ratio=0.98, self_propelled=False))
    assert not o["stability"]["ok"]
    assert any(c["name"].startswith("Stability") and not c["ok"] for c in o["checks"])


def test_box_barge_unchanged():
    """Label: snapshot
    """
    o = simulate(dict(vessel="box_barge", cargo="bauxite", loa_m=76.5, beam_m=11.4, depth_m=3.5,
                      cb=0.928, lightship_k_t_per_m3=0.1093, freeboard_min_m=0.3))
    assert 2200 < o["results"]["cargo_t"] < 2300


def test_gas_infeasible_fit_is_shown_as_a_failed_check():
    """Issue #6: when fit() finds no passing arrangement, the page says so, not just the tonnage.

    A 150 x 8 m hull carries cargo but no tank is within 7 diameters of length (the Rust core runs
    the search, so the test uses this hull instead of patching the search). The 60 x 7 m tender hull
    used here before failed only the inland type-G side floor, which issue #215 removed: it is
    feasible now.

    Label: property
    """
    tender = dict(vessel="gas_barge", cargo="LPG_propane", loa_m=60, beam_m=7.0, depth_m=6.0,
                  cb=0.85, freeboard_min_m=0.3, self_propelled=False, ruleset="IGC_2G",
                  fill_ratio=0.98)
    assert simulate(tender)["arrangement"]["feasible"] is True
    o = simulate(dict(tender, loa_m=150, beam_m=8.0))
    assert o["results"]["cargo_t"] > 0
    assert o["arrangement"]["feasible"] is False
    assert any(c["name"] == "Feasible arrangement" and not c["ok"] for c in o["checks"])
    o = simulate(CN_II)
    assert o["arrangement"]["feasible"] is True
    assert not any(c["name"] == "Feasible arrangement" for c in o["checks"])
