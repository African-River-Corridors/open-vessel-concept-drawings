"""The public API against the original pure-Python package (issue #143).

Every public function is called with whole numbers as `int`, with numpy integers and floats, with
an edited or rebuilt result object, and with one inputs object reused in a loop. The calls are the
scenarios of `tests/api_scenarios.py`. The expected answers are in `tests/data/api_ce96e66.json`,
printed by the frozen Python at commit ce96e66 (`tests/data/make_api_ce96e66.py` says how).

Floats are compared with the tolerance of spec 0001: relative and absolute 1e-9. It is the one the
parity harness uses, and it is not loosened here. A whole number equals the same float. Text, flags,
list lengths, keys and error messages must be identical.
"""
import json
import math
import sys
from pathlib import Path

import numpy as np
import pytest

import api_scenarios as S

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from design_impact import UNHASHED_KEYS  # noqa: E402

DATA = Path(__file__).parent / "data"
EXPECTED = json.loads((DATA / "api_ce96e66.json").read_text(encoding="utf-8"))["scenarios"]
API = json.loads((DATA / "public_api.json").read_text(encoding="utf-8"))

# Scenarios whose answer is allowed to differ from 0.1.0, with the reason. Each is listed as a
# known change in CHANGELOG.md.
KNOWN_DIFFERENCES: dict = {
    # Crude oil (spec 0006) is a seventh Cargo, so the list of Cargos and the "Valid types" text of
    # an unknown-cargo error name it. The six Cargos of 0.1.0 are unchanged.
    "cargo.get_cargo": "crude oil is a new Cargo: the unknown-cargo error lists it",
    "simulate.cargo_options": "crude oil is a new Cargo: the cargo list holds it",
    # Issue #432: the convoy beam check says it is an identity for the built-in catalogue.
    "vessel_standards": "issue #432: the convoy beam check message names the identity",
    "vessel_standards.custom_catalogue": "issue #432: the convoy beam check message names the identity",
    # Issue #414: the stern rake's LCB was mirrored (integrated from x = 0 forward); 0.1.0 gave the
    # mirror value. Only a hull with a stern rake moves; no Design has one.
    "hull.methods": "issue #414: the stern rake LCB is no longer mirrored",
    # Issue #175: no 28 m tank-length cap, so every cell has two tanks along (the old count stepped
    # up with the hull length), and a cell is infeasible when its tank is longer than 7 diameters.
    # The cells' tank length and slenderness are now the engine's own figures. `n_tanks_for_length`
    # and `DesignGrid.max_tank_length_m` are gone. The 12 m diameter cap of 0.1.0 is now 16 m.
    "design_grid": "issue #175: no tank-length cap; the grid has two tanks along and a 16 m "
                   "tank diameter cap",
    # Issue #215: the inland type-G floors (`side_clearance_type_g_floor`, `double_bottom_type_g_floor`)
    # are removed, so a gas result lists two checks fewer, and a search that used to reject an
    # arrangement for them can pick another one. Each scenario below holds a gas check list or fit.
    "barge_capacity.compute.gas": "issue #215: the inland type-G floors are removed; issue #280: the aft end-bulkhead space check (3.5.3) replaces the cofferdam check (3.6.13)",
    "barge_capacity.compute.gas_counts": "issue #215: the inland type-G floors are removed; issue #280: the aft end-bulkhead space check (3.5.3) replaces the cofferdam check (3.6.13)",
    "barge_capacity.compute.nested": "issue #215: the inland type-G floors are removed; issue #280: the aft end-bulkhead space check (3.5.3) replaces the cofferdam check (3.6.13)",
    "ga_adapter.to_ga": "issue #215: the inland type-G floors are removed; issue #280: the aft end-bulkhead space check (3.5.3) replaces the cofferdam check (3.6.13)",
    "gas_arrangement.fit": "issue #215: the inland type-G floors are removed; issue #280: the aft end-bulkhead space check (3.5.3) replaces the cofferdam check (3.6.13)",
    "gas_arrangement.fit.exact": "issue #215: the inland type-G floors are removed; issue #280: the aft end-bulkhead space check (3.5.3) replaces the cofferdam check (3.6.13)",
    "gas_separation.compute": "issue #215: the inland type-G floors are removed; issue #280: the aft end-bulkhead space check (3.5.3) replaces the cofferdam check (3.6.13)",
    "simulate.gas": "issue #215: the inland type-G floors are removed; issue #280: the aft end-bulkhead space check (3.5.3) replaces the cofferdam check (3.6.13)",
    "simulate.gas_igc_channel": "issue #215: the inland type-G floors are removed; issue #280: the aft end-bulkhead space check (3.5.3) replaces the cofferdam check (3.6.13)",
    "simulate.gas_optimise_depth": "issue #215: the inland type-G floors are removed; issue #280: the aft end-bulkhead space check (3.5.3) replaces the cofferdam check (3.6.13)",
    "edited.to_ga_with_other_draft": "issue #215: the inland type-G floors are removed; issue #280: the aft end-bulkhead space check (3.5.3) replaces the cofferdam check (3.6.13)",
    "edited.to_ga_with_rebuilt_result": "issue #215: the inland type-G floors are removed; issue #280: the aft end-bulkhead space check (3.5.3) replaces the cofferdam check (3.6.13)",
    "reuse.is_gas_and_to_ga_over_a_sweep": "issue #215: the inland type-G floors are removed; issue #280: the aft end-bulkhead space check (3.5.3) replaces the cofferdam check (3.6.13)",
    # Issue #237: one gas rule set, `IGC_2G`, the per-tank d(Vc) of IGC 2.4.1.1 (the stepped
    # `distance_d_vc` and the `IGC_2G_EXACT` name are gone; the dome now counts in Vc). A gas design
    # has less side clearance, so its tanks, cargo, weights and stability move. The scenarios below
    # hold a gas result or a rule call. The scenarios of #215 above moved again for the same reason.
    # Issue #264: a viable length needs an arrangement that passes every check, stability included.
    "gas_arrangement.smallest_viable_loa": "issue #264: a viable length needs an arrangement that "
                                            "passes every check",
    "gas_stability.assess": "issue #237: the per-tank d(Vc) moves the gas designs",
    "edited.assess_with_rebuilt_result": "issue #237: the per-tank d(Vc) moves the gas designs",
    "edited.assess_with_edited_basis": "issue #237: the per-tank d(Vc) moves the gas designs",
    "reuse.assess_over_a_sweep": "issue #237: the per-tank d(Vc) moves the gas designs",
    "gas_arrangement.required_side_clearance": "issue #237: the one rule is the per-tank d(Vc); "
                                               "IGC_2G_EXACT is not a rule set",
    "protective_location": "issue #237: `distance_d_vc` and `VesselType.IGC_2G_EXACT` are removed; "
                           "`check_protective_location` reads one tank's volume",
    # Issue #94: the class list is `catalogue/vessel_classes.yaml`, the one source. The ids are the
    # catalogue's (the 12 old ids still work for `get_standard`), `standards_for` returns every class
    # (Cargo is not a class property), the figures are the catalogue's (CEMT Va draught 2.5-2.8 m,
    # the NL2011 class is RWS-M12 with 135 m and 5601 t and no bridge clearance), CN_II_motor has no
    # moulded depth, and `VesselStandard` has four more fields and four Optional ones.
    "vessel_standards": "issue #94: the class list is the Vessel Class catalogue",
    # Issue #277: the wheelhouse sightline is an error check (the maintainer decision on #264), so
    # an aft arrangement or a dry-bulk result lists it with severity `error`, not `warning`.
    "barge_capacity.compute.bulk": "issue #277: the wheelhouse sightline is an error check",
    "barge_capacity.compute.bulk_holds": "issue #277: the wheelhouse sightline is an error check",
    "ga_adapter.to_ga.bulk": "issue #277: the wheelhouse sightline is an error check",
    "aft_arrangement.compute": "issue #277: the wheelhouse sightline is an error check",
    # Issue #418: every check that can remove a `fit` candidate is listed, and the two full-volume
    # checks (`not overloaded`, `freeboard ok`) are listed as warnings, so a `simulate` result has
    # more checks. The gas scenarios above already differ. Issues #396, #397, #398: the midship
    # section is B x T in the resistance (it was B x T x Cb), the Schijf limiting speed is the first
    # critical speed of the source (it was sqrt(2/3)(1 - n)^(1/3)), one 0.95 margin on it, the
    # open-water blockage is 0, and the electric installed power follows the highest battery power.
    "simulate.box": "issue #418: every check is listed; the full-volume ones as warnings; issue "
                    "#396: the open-water blockage is 0",
    "simulate.hopper_channel": "issue #418: every check is listed; the full-volume ones as warnings; "
                               "issues #396, #397: one midship section B x T and the Schijf first "
                               "critical speed",
    "resistance": "issues #396, #397: one midship section B x T and the Schijf first critical speed",
    # Issue #526: one g (9.80665 m/s^2) and one knot (1852/3600 m/s) in the resistance, squat and
    # wave modules. 0.1.0 used 9.81 and 0.5144 in the resistance, and 0.514444 in squat and waves.
    "squat": "issue #526: the knot is 1852/3600 m/s, not 0.514444",
    "waves.compute": "issue #526: the knot is 1852/3600 m/s, not 0.514444",
    "electric_energy.compute": "issues #396, #397, #398: the Schijf limit, its 0.95 margin and the "
                               "installed power on the highest battery power",
    # Issue #413: the legacy free surface takes the fill as a volume ratio (the liquid height is
    # the circular segment's, as the gas stability has it), not as a height ratio. Its moment at
    # 0.95 fill is about 12 times larger, so the legacy GM_fluid falls.
    # Issue #410: the hull steel is weighed to the moulded depth D, not to the wing plate, and the
    # covering above D (wing plate, side plating up to it, weather shield) is in the lightship. So
    # `moulded_depth`, `moulded_volume`, `steel_top_down`, piping, outfitting and the lightship move,
    # and the legacy stability and trim that read them.
    "tanks.free_surface_moment_cylindrical": "issue #413: the free surface takes the fill as a volume ratio",
    "hydrostatics.calculate_stability": "issue #410: the hull steel to D and the covering above D in the lightship; "
                                        "issue #413: the free surface takes the fill as a volume ratio",
    "edited.stability_with_raked_hull": "issue #410: the hull steel to D and the covering above D in the lightship; "
                                        "issue #413: the free surface takes the fill as a volume ratio",
    "edited.stability_with_heavier_weights": "issue #410: the hull steel to D and the covering above D in the lightship; "
                                             "issue #413: the free surface takes the fill as a volume ratio",
    "edited.stability_with_other_tanks": "issue #410: the hull steel to D and the covering above D in the lightship; "
                                         "issue #413: the free surface takes the fill as a volume ratio",
    "reuse.barge_inputs_in_a_loop": "issue #410: the hull steel to D and the covering above D in the lightship; "
                                    "issue #413: the free surface takes the fill as a volume ratio",
    "weights.calculate_weights": "issue #410: the hull steel to D and the covering above D in the lightship",
    "hydrostatics.calculate_trim": "issue #410: the hull steel to D and the covering above D in the lightship",
    "edited.trim_with_raked_hull": "issue #410: the hull steel to D and the covering above D in the lightship",
    "edited.weights_with_doubled_tank_shells": "issue #410: the hull steel to D and the covering above D in the lightship",
    # Issue #448: the hydrostatics come from the hull's Sections Table (the raked bow, the Stern
    # Form), not from Cb x L x B x T; the block coefficient is a result, not an input. The draughts
    # and so the derated tonnage move. The gas, capacity and stability scenarios above move again.
    "hydrostatics.find_equilibrium_draft": "issue #448: the draught comes from the hull's sections; "
                                           "Cb does not scale the volume",
    "journey_derating.compute": "issue #448: the draught comes from the hull's sections",
    "edited.journey_with_other_capacity": "issue #448: the draught comes from the hull's sections",
    "edited.journey_with_rebuilt_result": "issue #448: the draught comes from the hull's sections",
    "reuse.journey_over_a_sweep": "issue #448: the draught comes from the hull's sections",
}

# Keys added to inputs and results after 0.1.0, with the issue that added them. They are dropped
# from the actual answer before it is compared, and ONLY where 0.1.0 has no such key, so every key
# and value of 0.1.0 is still compared. Each is listed as an addition in CHANGELOG.md.
ADDED_KEYS: dict = {
    # Issue #180: gas-fuelled stern (Weichai gas gensets, accommodation beside the wheelhouse).
    **{k: "issue #180" for k in (
        "propulsion", "electrical_load_kwe", "n_gensets", "genset_model", "genset_kwe",
        "genset_length_m", "genset_width_m", "genset_height_m", "genset_engine_mass_t",
        "genset_clear_m", "engine_room_end_clear_m", "gvu_room_length_m",
        "container_layout", "container_units", "engine_room_x0_m", "engine_room_x1_m",
        "gvu_room_x0_m", "gvu_room_x1_m", "genset_engine_model", "installed_kwe",
        "genset_offsets_m", "genset_x0_m", "below_deck_length_m", "deck_length_m")},
}


def without_added(actual, expected):
    """`actual` with the ADDED_KEYS that 0.1.0 does not have removed, all the way down."""
    if isinstance(actual, dict) and isinstance(expected, dict):
        return {k: without_added(v, expected.get(k)) for k, v in actual.items()
                if k in expected or k not in ADDED_KEYS}
    if isinstance(actual, list) and isinstance(expected, list):
        return [without_added(a, e) for a, e in zip(actual, expected)] + actual[len(expected):]
    return actual


def same(actual, expected, path="result"):
    """None when equal, else a message that says where they differ."""
    if isinstance(expected, bool) or isinstance(actual, bool):
        return None if actual is expected else f"{path}: {actual!r} != {expected!r}"
    if isinstance(expected, (int, float)) and isinstance(actual, (int, float)):
        ok = actual == expected or math.isclose(actual, expected, rel_tol=1e-9, abs_tol=1e-9)
        return None if ok else f"{path}: {actual!r} != {expected!r}"
    if isinstance(expected, dict) and isinstance(actual, dict):
        if list(expected) != [k for k in actual if k not in UNHASHED_KEYS]:  # not results (#192, #231)
            return f"{path}: keys {list(actual)} != {list(expected)}"
        for key in expected:
            found = same(actual[key], expected[key], f"{path}.{key}")
            if found:
                return found
        return None
    if isinstance(expected, list) and isinstance(actual, list):
        if len(expected) != len(actual):
            return f"{path}: length {len(actual)} != {len(expected)}"
        for i, (a, e) in enumerate(zip(actual, expected)):
            found = same(a, e, f"{path}[{i}]")
            if found:
                return found
        return None
    return None if actual == expected and type(actual) is type(expected) else f"{path}: {actual!r} != {expected!r}"


CASES = [(name, mode) for name in S.SCENARIOS for mode in S.MODES]


@pytest.mark.parametrize("name,mode", CASES, ids=[f"{n}[{m}]" for n, m in CASES])
def test_the_api_gives_the_answers_of_0_1_0(name, mode):
    """Label: snapshot"""
    expected = EXPECTED[name].get(mode, EXPECTED[name]["float"])
    actual = S.run("ovd", mode, name)
    if name in KNOWN_DIFFERENCES:
        pytest.skip(KNOWN_DIFFERENCES[name])
    actual = without_added(actual, expected)
    assert same(actual, expected, "result") is None, same(actual, expected, "result")


def test_the_fixture_and_the_scenarios_list_the_same_cases():
    """Label: property"""
    assert sorted(EXPECTED) == sorted(S.SCENARIOS)
    assert len(EXPECTED) >= 60


def test_no_scenario_raised_in_0_1_0():
    # A scenario that errors in the original is a broken scenario, not an expectation.
    """Label: property"""
    raised = [(n, m) for n, modes in EXPECTED.items() for m, r in modes.items() if "raised" in r]
    assert raised == []


def test_no_scenario_is_vacuous():
    """Label: property"""
    empty = [n for n, modes in EXPECTED.items() if modes["float"]["ok"] in (None, [], {})]
    assert empty == []


def test_every_public_function_is_called_by_a_scenario():
    """Label: property"""
    needed = {"simulate." + n for n, v in API["ovd.simulate"].items() if v["kind"] == "function"}
    for module, names in API.items():
        if not module.startswith("ovd.core."):
            continue
        short = module.removeprefix("ovd.")
        needed |= {f"{short}.{n}" for n, v in names.items() if v["kind"] == "function"}
        # classes with callable methods of their own (not just an `ok` property)
        needed |= {f"{short}.{n}" for n, v in names.items()
                   if v["kind"] == "class" and [m for m in v["methods"] if m not in ("ok",)]
                   and n in ("HullGeometry", "ResistanceCalculator", "VesselStandard", "DesignGrid")}
    covered = {c for fn in S.SCENARIOS.values() for c in fn.covers}
    assert sorted(needed - covered) == []
    assert len(needed) > 50, "scanning too little"


def test_the_modes_really_hand_over_the_numbers_they_name():
    """Label: property"""
    ints = S.Ctx("ovd", "int")
    assert type(ints.n(90.0)) is int and type(ints.n(0.5)) is float and type(ints.k(2)) is int
    np_ints = S.Ctx("ovd", "np.int64")
    assert type(np_ints.n(90.0)) is np.int64 and type(np_ints.n(0.5)) is np.float64
    assert type(np_ints.k(2)) is np.int64
    np_floats = S.Ctx("ovd", "np.float64")
    assert type(np_floats.n(90.0)) is np.float64 and type(np_floats.n(0.5)) is np.float64
    plain = S.Ctx("ovd", "float")
    assert type(plain.n(90.0)) is float
