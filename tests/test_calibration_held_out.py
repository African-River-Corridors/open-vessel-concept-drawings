"""Held-out calibration cases (issue #3): run and banded, never used to fit a factor.

The 62 cases added for issue #3 are all `held_out`. These tests check that the fit ignores them, that
every one is transcribed so that its Cb check matches the research note, that every one cites a source
that is listed, and that the report shows them out of sample.
"""
import copy

import pytest
import yaml

from vessel_designer import calibration
from vessel_designer.calibration import (BANDS, CAL, assess, factors_yaml, fit_factors, _is_held_out,
                                         load_cases, report, run_barge, warnings_for)

NOTE_SUGGESTS = {"fit", "hold out", "cross-check", "exclude"}
NOT_RUNNABLE_HULLS = {"tank", "gas", "container (motor)"}

# Cb check column of the research note (2026-10-16), printed to 3 dp: (lightship + DWT) / (rho L B T),
# or DWT / (rho L B T) when there is no lightship (cargo capacity for the cases that publish that).
CB_CHECK = {
    "cbc-1951-deck-195x35": 0.949,
    "cbc-2762-deck-276x55": 0.886,
    "cbc-4508-deck-200x50": 0.858,
    "cbc-cordie-deck-260x72": 0.987,
    "cbc-julius-deck-300x100": 0.872,
    "cbc-oceanus-deck-400x105": 0.907,
    "cbc-761-deck-180x54": 1.093,
    "lm3004-deck-288ft": 0.935,
    "vl-deck-365ft": 0.829,
    "vl-deck-345ft": 0.805,
    "vl-deck-300ft": 0.857,
    "vl-deck-250ft": 0.893,
    "jml5019-pontoon": 0.776,
    "touax-193-194-2021": 0.802,
    "touax-seine-2020": 0.810,
    "taf-seine-2008": 0.785,
    "taf-seine-2000": 0.810,
    "taf-seine-1997": 0.812,
    "taf-seine-1996": 0.776,
    "taf-seine-1993": 0.787,
    "taf-seine-1981": 0.819,
    "taf-seine-1973": 0.840,
    "taf-rhin-2019-covered": 0.825,
    "taf-rhin-2008-covered": 0.793,
    "taf-rhin-2000": 0.808,
    "taf-rhin-2001": 0.805,
    "taf-rhin-1980-covered": 0.796,
    "taf-danube-2008": 0.744,
    "taf-parana-2013": 0.784,
    "taf-mississippi-2011": 1.051,
    "meb-dcb23-ptich": 0.727,
    "meb-dcb24-pola": 0.770,
    "meb-rdb12-samarskaya": 0.712,
    "meb-rdb20-belmax9": 0.776,
    "meb-rob20-belmax": 0.806,
    "meb-rob07-mh4001": 0.758,
    "meb-rob05-paloil": 0.755,
    "meb-rob17-nerida": 0.652,
    "meb-1635tm-fng": 0.789,
    "meb-1635t-udp-slt": 0.754,
    "kirby-10071-200x35": 0.763,
    "kirby-10201-200x35": 0.730,
    "kirby-10083-200x35": 0.745,
    "kirby-10074-195x35": 0.741,
    "cgbm-102-200x35": 0.757,
    "mer-13-200x35": 0.756,
    "kirby-301-250x50": 0.765,
    "kirby-407-263x54": 0.751,
    "hfl-406-297x54": 0.735,
    "kirby-27773-297x54": 0.757,
    "kirby-32300b-297x54": 0.799,
    "ctco-358-297x54": 0.720,
    "htco-3064-320x50": 0.724,
    "cms-1601-222x54": 0.624,
    "crowley-180-2-tank": 0.729,
}

# Cases whose dimensions the sources print in feet. The note computed its check on the exact metre
# conversions, the cases hold them to 2 dp, so these reproduce the printed check to 0.0015, not 0.0005.
FEET_SOURCED = {
    "cbc-2762-deck-276x55",
    "cbc-cordie-deck-260x72",
    "cbc-julius-deck-300x100",
    "cbc-oceanus-deck-400x105",
    "cbc-761-deck-180x54",
    "kirby-10071-200x35",
    "kirby-10201-200x35",
    "kirby-10083-200x35",
    "kirby-301-250x50",
    "kirby-407-263x54",
    "hfl-406-297x54",
    "kirby-27773-297x54",
    "kirby-32300b-297x54",
    "ctco-358-297x54",
}


def _cases():
    return load_cases()


def _held_out(cases):
    return [c for c in cases["dry_barge"] + cases.get("not_yet_runnable_cases", []) if _is_held_out(c)]


def test_the_fit_ignores_held_out_cases():
    """Label: property
    """
    cases = _cases()["dry_barge"]
    assert any(_is_held_out(c) for c in cases) and any(not _is_held_out(c) for c in cases)
    in_sample = [c for c in cases if not _is_held_out(c)]
    assert fit_factors(in_sample) == fit_factors(cases)
    assert factors_yaml(fit_factors(cases)) == (CAL / "factors.yaml").read_text()


def test_the_fit_filter_is_live():
    """Label: property
    """
    cases = _cases()["dry_barge"]
    base = fit_factors(cases)
    # A held-out box case with a DWT, made in-sample, moves the Cb fit.
    moved = copy.deepcopy(cases)
    next(c for c in moved if c["id"] == "touax-seine-2020").pop("held_out")
    assert fit_factors(moved) != base
    assert fit_factors(moved)["block_coefficient_box_barge_on_loa"]["n"] == \
        base["block_coefficient_box_barge_on_loa"]["n"] + 1
    # A held-out case with a lightship, made in-sample, moves the k fit (here, a deck k appears).
    moved = copy.deepcopy(cases)
    next(c for c in moved if c["id"] == "lm3004-deck-288ft").pop("held_out")
    assert "deck" in fit_factors(moved)["lightship_k_t_per_m3"]
    assert "deck" not in base["lightship_k_t_per_m3"]


def test_no_held_out_case_is_in_the_committed_factors():
    """Label: property
    """
    text = (CAL / "factors.yaml").read_text()
    for c in _held_out(_cases()):
        assert c["id"] not in text


def test_every_new_case_is_held_out_and_marked_with_what_the_note_suggested():
    """Label: property
    """
    cases = _cases()
    in_sample = [c for c in cases["dry_barge"] if not _is_held_out(c)]
    assert len(in_sample) == 12
    held = _held_out(cases)
    assert len(held) == 62
    assert len({c["id"] for c in held}) == 62
    for c in held:
        assert isinstance(c["held_out"], str) and c["held_out"].strip(), c["id"]
        assert c["note_suggests"] in NOTE_SUGGESTS, c["id"]
        assert c["published"].strip(), c["id"]
        assert c["confidence"] in ("high", "medium", "low"), c["id"]


def test_hull_types_and_what_runs():
    """Label: property
    """
    cases = _cases()
    held_run = [c for c in cases["dry_barge"] if _is_held_out(c)]
    not_run = cases["not_yet_runnable_cases"]
    count = lambda cs, h: sum(1 for c in cs if c["hold_type"] == h)
    assert (count(held_run, "deck"), count(held_run, "box")) == (18, 21)
    assert (count(not_run, "tank"), count(not_run, "gas"), count(not_run, "container (motor)")) == (21, 1, 1)
    for c in held_run:
        assert c["hold_type"] in ("deck", "box"), c["id"]
        assert not c.get("not_yet_runnable"), c["id"]
        if c["hold_type"] == "deck":
            assert c["run_as"] == "box", c["id"]
        else:
            assert "run_as" not in c, c["id"]
    for c in not_run:
        assert c["hold_type"] in NOT_RUNNABLE_HULLS and c["not_yet_runnable"].strip(), c["id"]
        assert _is_held_out(c)


def test_a_missing_figure_is_absent_not_filled():
    """Label: property
    """
    for c in _held_out(_cases()):
        for key, value in c["observed"].items():
            assert value is not None, (c["id"], key)
        # A cargo capacity is never recorded as a DWT.
        if "cargo_capacity_t" in c["observed"]:
            assert "dwt_t" not in c["observed"], c["id"]


def test_excluded_and_cross_check_cases_are_marked_with_a_reason():
    """Label: property
    """
    by_id = {c["id"]: c for c in _held_out(_cases())}
    assert {i for i, c in by_id.items() if c.get("exclude_from_fit_cb")} == \
        {"cbc-761-deck-180x54", "taf-mississippi-2011"}
    assert {i for i, c in by_id.items() if c.get("cross_check_only")} == \
        {"cbc-758-spud-180x54", "jml5019-pontoon", "crowley-180-2-tank"}
    assert {i for i, c in by_id.items() if c["note_suggests"] == "exclude"} == \
        {"cbc-761-deck-180x54", "taf-mississippi-2011"}
    assert {i for i, c in by_id.items() if c["note_suggests"] == "cross-check"} == \
        {"cbc-758-spud-180x54", "jml5019-pontoon", "crowley-180-2-tank"}
    for c in by_id.values():
        for key in ("exclude_from_fit_cb", "cross_check_only"):
            if key in c:
                assert str(c[key]).strip(), c["id"]


def test_every_new_case_cites_a_listed_web_source():
    """Label: property
    """
    web = yaml.safe_load((CAL / "sources.yaml").read_text())["web_sources"]
    ids = [w["id"] for w in web]
    assert len(ids) == len(set(ids))
    for w in web:
        assert w["url"].startswith("https://"), w["id"]
        assert w["accessed"] == "2026-10-16", w["id"]
        assert w["title"].strip(), w["id"]
    for c in _held_out(_cases()):
        sid = c["source"].split(" ")[0]
        assert sid in ids, (c["id"], sid)
        assert c["source"].startswith(f"{sid} — "), c["id"]
        assert c["source"].endswith("accessed 2026-10-16"), c["id"]
    # The stored-file records are untouched.
    assert len(yaml.safe_load((CAL / "sources.yaml").read_text())["sources"]) == 8


def test_each_cb_check_recomputed_from_the_cases_matches_the_research_note():
    """Label: sourced - research note 2026-10-16, Cb check column
    """
    cases = {c["id"]: c for c in _held_out(_cases())}
    for cid, printed in CB_CHECK.items():
        c, o = cases[cid], cases[cid]["observed"]
        mass = o.get("lightship_t", 0.0) + o.get("dwt_t", o.get("cargo_capacity_t"))
        rho = c.get("water_density_t_m3", 1.0)
        got = mass / (rho * c["loa_m"] * c["beam_m"] * c["draught_m"])
        tol = 0.0015 if cid in FEET_SOURCED else 0.0005
        assert got == pytest.approx(printed, abs=tol), (cid, got, printed)
    # Every case that has a draught and a DWT has a check in the note.
    for cid, c in cases.items():
        if c.get("draught_m") and "dwt_t" in c["observed"]:
            assert cid in CB_CHECK, cid
    assert set(CB_CHECK) <= set(cases)


def test_density_is_one_of_the_two_the_note_used():
    """Label: property
    """
    for c in _held_out(_cases()):
        assert c.get("water_density_t_m3", 1.0) in (1.0, 1.025), c["id"]


def test_the_runner_honours_the_water_density_and_runs_deck_as_box():
    """Label: property
    """
    cases = _cases()
    f = fit_factors(cases["dry_barge"])
    deck = next(c for c in cases["dry_barge"] if c["id"] == "lm3004-deck-288ft")
    fresh = copy.deepcopy(deck)
    fresh.pop("water_density_t_m3")
    a, b = run_barge(deck, f), run_barge(fresh, f)
    cb = f["block_coefficient_box_barge_on_loa"]["value"]
    lbt = deck["loa_m"] * deck["beam_m"] * deck["draught_m"]
    assert a.dwt_cal - b.dwt_cal == pytest.approx(cb * lbt * 0.025)
    assert a.ls_cal == b.ls_cal                      # density does not touch the lightship
    k = f["lightship_k_t_per_m3"]["box"]["value"]
    assert a.ls_cal == pytest.approx(k * deck["loa_m"] * deck["beam_m"] * deck["depth_m"])
    # A case with no density keeps density 1.000: the in-sample results do not move.
    old = next(c for c in cases["dry_barge"] if c["id"] == "amaggi-box-2000")
    assert "water_density_t_m3" not in old
    r = run_barge(old, f)
    assert r.dwt_cal == pytest.approx(cb * old["loa_m"] * old["beam_m"] * old["draught_m"] - r.ls_cal)


def test_held_out_cases_are_banded_except_cross_checks_and_cases_that_cannot_run():
    """Label: property
    """
    cases = _cases()
    checks = assess(cases)
    banded = {b.case for b in checks}
    cross = {c["id"] for c in cases["dry_barge"] if c.get("cross_check_only")}
    assert not banded & cross
    assert not banded & {c["id"] for c in cases["not_yet_runnable_cases"]}
    held_ids = {c["id"] for c in _held_out(cases)}
    for b in checks:
        assert bool(b.held_out) == (b.case in held_ids), b.case
    held = [b for b in checks if b.held_out]
    assert held and {b.figure for b in held} == {"lightship_t", "dwt_t"}
    for b in held:
        assert b.band_pct == BANDS["bands"][b.figure]["pct"][b.confidence]


def test_a_held_out_band_miss_is_a_warning_that_says_held_out():
    """Label: property
    """
    checks = assess()
    held_miss = [b for b in checks if b.held_out and not b.inside]
    in_miss = [b for b in checks if not b.held_out and not b.inside]
    assert held_miss and in_miss
    lines = warnings_for(checks)
    assert len(lines) == len(held_miss) + len(in_miss)
    for b in held_miss:
        mine = next(w for w in lines if f"{b.case}:" in w)
        assert "held-out" in mine and "in-sample" not in mine
    for b in in_miss:
        mine = next(w for w in lines if f"{b.case}:" in w)
        assert "in-sample" in mine and "held-out" not in mine


def test_the_report_has_an_out_of_sample_section_with_every_case_and_a_summary():
    """Label: property
    """
    cases = _cases()
    text = report()
    assert "## 4. Held-out cases (out of sample)" in text
    section = text.split("## 4. Held-out cases (out of sample)")[1]
    for c in _held_out(cases):
        assert f"| {c['id']} |" in section, c["id"]
    for c in cases["not_yet_runnable_cases"]:
        row = next(line for line in section.splitlines() if line.startswith(f"| {c['id']} |"))
        assert c["not_yet_runnable"] in row
    assert "### Summary: out of sample against in-sample" in section
    assert "warnings only (#119)" in section
    for group in ("box", "deck (run as box)"):
        assert f"| out of sample | {group} |" in section
    # Sections 1 to 3 hold in-sample cases only.
    before = text.split("## 4. Held-out cases")[0]
    for c in _held_out(cases):
        assert f"| {c['id']} |" not in before


def test_the_in_sample_sections_do_not_change_when_held_out_cases_are_removed():
    """Label: property
    """
    cases = _cases()
    all_text = report().split("## 4. Held-out cases")[0]
    kept = copy.deepcopy(cases)
    kept["dry_barge"] = [c for c in kept["dry_barge"] if not _is_held_out(c)]
    kept["not_yet_runnable_cases"] = []
    orig = calibration.load_cases
    calibration.load_cases = lambda: kept
    try:
        in_only = report().split("## 4. Held-out cases")[0]
    finally:
        calibration.load_cases = orig
    strip = lambda t: [ln for ln in t.splitlines() if "held-out calibrated results" not in ln]
    assert strip(in_only) == strip(all_text)


def test_summary_statistics_are_computed_from_the_checks():
    """Label: property
    """
    checks = [b for b in assess() if b.held_out and not b.excluded]
    st = calibration._summary_stats([b for b in checks if b.group == "box" and b.figure == "dwt_t"])
    errs = [b.err_pct for b in checks if b.group == "box" and b.figure == "dwt_t"]
    assert st["n"] == len(errs) == 20
    assert st["mean"] == pytest.approx(sum(errs) / len(errs))
    assert st["mean_abs"] == pytest.approx(sum(abs(e) for e in errs) / len(errs))
    assert (st["min"], st["max"]) == (min(errs), max(errs))
    assert st["inside"] == sum(1 for b in checks if b.group == "box" and b.figure == "dwt_t" and b.inside)
    assert calibration._summary_stats([]) is None
