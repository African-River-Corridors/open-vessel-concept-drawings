"""The tank-barge path of the calibration (issue #335, spec 0006 section 7).

Every `hold_type: tank` record of `cases.yaml` and the reference vessels of `tank_barge_cases.yaml` run
through the tank-barge model (lightship = k x L x B x D + outfit, deadweight from the box-barge Cb), with
bands in `tank_barge_bands.yaml`. The k-fit subset is in-sample, every other case is out of sample, and a
band miss is a warning, never a failure.
"""
import re

import pytest
import yaml

from vessel_designer.calibration import (CAL, _assess_tank_barges, _load_tank_barge_bands,
                                         _load_tank_barge_cases, _load_tank_lightship, _tank_barge_rows,
                                         _tank_barge_warnings_for, fit_factors, load_cases, pct, report,
                                         _run_tank_barge, _tank_barge_records, _tank_outfit_t)

CONFIDENCES = ("high", "medium", "low")
NEW_SOURCES = {"M15", "M16", "M17", "M18", "M19", "M20", "M21", "M22", "M23", "M24", "M25", "R1"}


def _web():
    return {w["id"]: w for w in yaml.safe_load((CAL / "sources.yaml").read_text(encoding="utf-8"))["web_sources"]}


def test_every_tank_record_of_cases_yaml_runs_in_the_tank_barge_path():
    """Label: property
    """
    cases = load_cases()
    tank_ids = {c["id"] for key in ("dry_barge", "not_yet_runnable_cases") for c in cases[key]
                if c.get("hold_type") == "tank"}
    assert len(tank_ids) == 41
    rows = {r.case["id"]: r for r in _tank_barge_rows()}
    assert tank_ids <= set(rows)
    # A record with a depth, a draught and a tank volume has a model lightship and deadweight.
    for cid in tank_ids:
        c = rows[cid].case
        complete = c.get("depth_m") and c.get("draught_m") and "tank_capacity_m3" in c["observed"]
        assert (rows[cid].dwt_model is not None) == bool(complete), cid
    # The reason the dry-barge path gave is no longer "no tank model": the record is run elsewhere.
    for key in ("not_yet_runnable_cases",):
        for c in cases[key]:
            if c.get("hold_type") == "tank":
                assert "tank-barge path" in c["not_yet_runnable"], c["id"]


def test_the_reference_vessels_are_held_out_cited_and_not_in_cases_yaml_twice():
    """Label: property
    """
    web = _web()
    new = _load_tank_barge_cases()["tank_barge"]
    assert len(new) == 12
    old_ids = {c["id"] for c in load_cases()["not_yet_runnable_cases"] + load_cases()["dry_barge"]}
    assert not {c["id"] for c in new} & old_ids
    assert len({c["id"] for c in new}) == len(new)
    cited = set()
    for c in new:
        assert c["hold_type"] == "tank" and c["confidence"] in CONFIDENCES, c["id"]
        assert str(c["held_out"]).strip() and c["published"].strip(), c["id"]
        sid = c["source"].split(" ")[0]
        cited.add(sid)
        assert sid in web, (c["id"], sid)
        assert c["source"].startswith(f"{sid} — ") and re.search(r"accessed 2026-\d\d-\d\d$", c["source"]), c["id"]
        # A figure the source does not give is absent, and a cargo capacity is never a deadweight.
        o = c["observed"]
        assert all(v is not None for v in o.values()), c["id"]
        assert not ("cargo_capacity_t" in o and "dwt_t" in o), c["id"]
        assert "tank_capacity_m3" in o and ("dwt_t" in o or "cargo_capacity_t" in o), c["id"]
        # A case with no band says why.
        if c.get("self_propelled") or "cargo_capacity_t" in o:
            assert str(c["not_banded"]).strip(), c["id"]
    assert cited == NEW_SOURCES
    for sid in NEW_SOURCES:
        s = web[sid]
        assert re.fullmatch(r"[0-9a-f]{64}", s["sha256"]) and s["retrieved"] == "2026-10-22", sid


def test_the_model_is_k_lbd_plus_outfit_and_the_deadweight_follows_from_the_box_cb():
    """Label: property
    """
    cases = load_cases()
    f = fit_factors(cases["dry_barge"])
    tank_lightship, inputs = _load_tank_lightship()
    k = tank_lightship["tank_lightship_k_t_per_m3"]["value"]
    cb = f["block_coefficient_box_barge_on_loa"]["value"]
    # The outfit coefficients are those the inputs file declares, summed by hand here.
    per_m3 = sum(inputs["outfit"][n]["value"] for n in ("pumps_t_per_m3", "piping_t_per_m3", "vents_gauging_t_per_m3"))
    for c in _load_tank_barge_cases()["tank_barge"]:
        row = _run_tank_barge(c, f, tank_lightship, inputs)
        want_ls = k * c["loa_m"] * c["beam_m"] * c["depth_m"] + c["observed"]["tank_capacity_m3"] * per_m3
        assert row.ls_model == pytest.approx(want_ls, rel=1e-12), c["id"]
        rho = c.get("water_density_t_m3", 1.0)
        assert row.dwt_model == pytest.approx(cb * c["loa_m"] * c["beam_m"] * c["draught_m"] * rho - want_ls, rel=1e-12)
    assert _tank_outfit_t(1000.0, tank_lightship["outfit"], 100.0) == pytest.approx(
        1000.0 * per_m3 + 100.0 * inputs["outfit"]["heating_t_per_kw"]["value"])


def test_the_run_agrees_with_the_fit_tool_on_every_scored_record():
    """The fit tool (`tools/fit_tank_lightship.py`) writes each record's lightship model error against
    its implied lightship. The run's lightship must give the same error.

    Label: property
    """
    tank_lightship, _ = _load_tank_lightship()
    rows = {r.case["id"]: r for r in _tank_barge_rows()}
    cb = fit_factors(load_cases()["dry_barge"])["block_coefficient_box_barge_on_loa"]["value"]
    listed = tank_lightship["fit_cases"] + tank_lightship["held_out_cases"]
    assert len(listed) >= 35
    for e in listed:
        r = rows[e["case"]]
        c = r.case
        implied = cb * c["loa_m"] * c["beam_m"] * c["draught_m"] * c.get("water_density_t_m3", 1.0) - c["observed"]["dwt_t"]
        # The tool rounds the implied lightship to 0.1 t before it works out the error.
        assert round(implied, 1) == pytest.approx(e["lightship_implied_t"], abs=1e-9), e["case"]
        # The tool uses the mean k unrounded and the file has it to 4 places: the errors agree to 0.15 point.
        assert pct(r.ls_model, round(implied, 1)) == pytest.approx(e["model_error_pct"], abs=0.15), e["case"]


def test_the_k_fit_subset_is_in_sample_and_every_other_case_is_not():
    """Label: property
    """
    _, inputs = _load_tank_lightship()
    rows = _tank_barge_rows()
    in_sample = {r.case["id"] for r in rows if r.sample == "in-sample"}
    assert in_sample == set(inputs["fit_subset"])
    for r in rows:
        if r.case["id"] in inputs["not_scored"]:
            assert r.sample == "not scored" and r.banded
        elif r.case["id"] not in in_sample:
            assert r.sample == "held out", r.case["id"]
    checks = _assess_tank_barges()
    # A check of an in-sample case carries no held-out marker; every other has one.
    for b in checks:
        assert bool(b.held_out) == (b.case not in in_sample), b.case
    # k was fitted on the subset from the same deadweights, so those errors are small; the others are not.
    mean_abs = lambda xs: sum(abs(b.err_pct) for b in xs) / len(xs)
    ins = [b for b in checks if b.case in in_sample]
    outs = [b for b in checks if b.case not in in_sample]
    assert len(ins) == len(inputs["fit_subset"])
    assert mean_abs(ins) < mean_abs(outs)


def test_a_self_propelled_vessel_a_cargo_capacity_and_a_cross_check_have_no_band():
    """Label: property
    """
    banded = {b.case for b in _assess_tank_barges()}
    for r in _tank_barge_rows():
        c = r.case
        o = c["observed"]
        if c.get("self_propelled") or "dwt_t" not in o or c.get("cross_check_only") or r.sample == "not scored":
            assert c["id"] not in banded, c["id"]
        elif r.dwt_model is not None:
            assert c["id"] in banded, c["id"]
    # Of the new vessels only the one non-self-propelled river barge with a deadweight is banded.
    new = {c["id"] for c in _load_tank_barge_cases()["tank_barge"]}
    assert banded & new == {"meb-rob21"}


def test_a_band_miss_is_a_warning_and_a_doctored_figure_is_caught():
    """The check must be able to fail: a published deadweight changed by 30 % gives a miss, and an
    annotation, and the crash path is not taken.

    Label: property
    """
    cases = load_cases()
    for c in cases["not_yet_runnable_cases"]:
        if c["id"] == "meb-rob07-mh4001":
            before = _assess_tank_barges(cases)
            c["observed"]["dwt_t"] = c["observed"]["dwt_t"] * 1.3
    after = _assess_tank_barges(cases)
    pick = lambda cs: next(b for b in cs if b.case == "meb-rob07-mh4001")
    assert pick(before).inside and not pick(after).inside
    lines = _tank_barge_warnings_for(after)
    assert any("meb-rob07-mh4001" in ln and ln.startswith("::warning ") for ln in lines)
    assert not any("meb-rob07-mh4001" in ln for ln in _tank_barge_warnings_for(before))


def test_the_tank_barge_bands_have_both_figures_for_every_confidence_with_a_basis():
    """Label: property
    """
    bands = _load_tank_barge_bands()
    assert set(bands) == {"lightship_t", "dwt_t"}
    for name, band in bands.items():
        assert set(band["pct"]) == set(CONFIDENCES), name
        assert band["pct"]["high"] < band["pct"]["medium"] < band["pct"]["low"], name
        assert band["label"].strip() and "proposed" in band["basis"], name


def test_the_report_has_a_tank_barge_section_that_labels_the_samples():
    """Label: property
    """
    text = report()
    assert "## 5a. Tank barges (issue #335)" in text
    section = text.split("## 5a. Tank barges (issue #335)")[1].split("\n## 6.")[0]
    assert "in-sample" in section and "out of sample" in section
    assert "in the k fit" in section
    for c in _tank_barge_records():
        assert f"| {c['id']} |" in section, c["id"]
    assert "self-propelled" in section
