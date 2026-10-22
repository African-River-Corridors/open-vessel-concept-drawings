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
    # stepped IGC_2G. Re-pinned (issue #277: the wheelhouse sightline blocks the search); was 2535.
    # Re-pinned (issue #410: the covering above D is in the lightship; the one-wide arrangement
    # fails the light-condition angle of max GZ); was 1763.
    # Re-pinned (issue #498: the angle of max GZ alone in the light condition is a warning); was 1406.
    assert abs(simulate(CN_II)["results"]["cargo_t"] - 1763) < 1.0


def test_gas_optimised_depth_matches_snapshot():
    """Label: snapshot
    """
    o = simulate(dict(CN_II, cargo="LNG", draught_limit_m=3.0, ruleset="IGC_2G",
                      fill_ratio=None, optimise_depth=True))
    # Re-pinned (issue #277: the wheelhouse sightline blocks the search); was 1738. Re-pinned
    # (issue #410: the covering above D is in the lightship); was 1691.
    # Re-pinned (issue #498: the angle of max GZ alone in the light condition is a warning); was 1204.
    assert abs(o["results"]["cargo_t"] - 1691) < 1.0
    assert o["arrangement"]["fill_ratio"] == 0.98          # ruleset-driven default


def test_the_tank_diameter_cap_is_a_simulate_input_16_m_by_default():
    """Label: property
    """
    default = {k: v for k, v in CN_II.items() if k != "max_tank_od_m"}
    # Issue #237: the per-tank rule asks for less side clearance, so on a 26 m beam the search
    # prefers two tanks abreast and the cap does not bind. At the CN II beam, 14.8 m, one tank
    # fills the beam. Since issue #277 the wheelhouse sightline blocks, so the tank is lower than
    # either cap (10.7 m); an 8 m cap still sets the diameter. Since issue #410 (the covering above
    # D is in the lightship) the 14.8 m beam takes two tanks abreast, so the case used 12 m, where
    # one tank still filled the beam. Issue #448: on the hull's sections the search takes two
    # tanks abreast there too; a 10 m beam and a 9.5 m depth keep one tank over 8 m.
    wide = dict(default, beam_m=10.0, loa_m=90, depth_m=9.5, cargo="LNG", draught_limit_m=4.4)
    assert simulate(dict(wide, max_tank_od_m=8.0))["geometry"]["tank_outer_diameter_m"] <= 8.0 + 1e-6
    od = simulate(wide)["geometry"]["tank_outer_diameter_m"]
    assert 8.0 < od <= 16.0 + 1e-6


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
    # Issue #498: the Light condition (no ballast) fails the angle of max GZ alone: the stability
    # block, as in 0.1.0, says the criteria are not all met, but the check of that condition is a
    # warning, and the arrangement is feasible.
    failing = {c["name"]: [k["name"] for k in c["criteria"] if not k["ok"]] for c in o["stability"]["conditions"]}
    assert failing == {"Laden": [], "Half laden": [], "Light": ["Angle of max GZ"]}
    assert o["arrangement"]["feasible"]
    assert sum(c["name"].startswith("Stability") for c in o["checks"]) == 3


def test_tender_gas_barge_shows_failed_stability_checks():
    """Label: property
    """
    # 60 x 20 x 3.5 m, self-propelled: no arrangement passes the laden and half laden conditions
    # (issue #264: with a beam of 7 m, pushed, the search finds a stable smaller arrangement; issue
    # #498: at 60 x 10 x 6 m only the light condition's angle of max GZ fails, a warning now).
    o = simulate(dict(CN_II, loa_m=60, beam_m=20.0, depth_m=3.5, draught_limit_m=3.2,
                      ruleset="IGC_2G", fill_ratio=0.98, self_propelled=True))
    assert not o["stability"]["ok"]
    assert any(c["name"].startswith("Stability") and not c["ok"] for c in o["checks"])
    assert not o["arrangement"]["feasible"]
    feasible = next(c for c in o["checks"] if c["name"] == "Feasible arrangement")
    assert not feasible["ok"] and "No arrangement passes every check" in feasible["message"]
    assert "Stability \u2014 laden" in feasible["message"]


def test_box_barge_unchanged():
    """Label: snapshot
    """
    o = simulate(dict(vessel="box_barge", cargo="bauxite", loa_m=76.5, beam_m=11.4, depth_m=3.5,
                      cb=0.928, lightship_k_t_per_m3=0.1093, freeboard_min_m=0.3))
    # Re-pinned (#448); was 2200 to 2300 t: the hull's sections give the volume, so the cb input
    # no longer reduces it (2346 t).
    assert 2300 < o["results"]["cargo_t"] < 2400


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
