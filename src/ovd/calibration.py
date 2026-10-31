"""Calibration pass: run Reference Vessel inputs through the core and compare with published figures.

    python -m ovd.calibration           # writes calibration/results.md and factors.yaml
    python -m ovd.calibration --check   # CI: one warning annotation per case outside its band

The core is not tuned here. This module MEASURES the error; corrections are a separate,
deliberate step recorded as a calibrated factor with its evidence.

Calibration Bands (`calibration/bands.yaml`, issue #119): each observed figure of a case has a band,
set per figure and per confidence of the case. A case outside its band is a WARNING only: the
report marks it and `--check` prints a warning annotation. Nothing here fails on a band miss; a
crash (a missing file, a bad case) still fails. The errors of the first 12 cases are in-sample: the
fitted factors came from the same cases. A case with a `held_out` marker (issue #3) is run and banded
but never used in the fit, so its error is out of sample; `report()` shows those in section 4.

The runner calls the core through the `ovd.core` modules, which run the Rust core
(`ovd._native`) like every other Python caller. It takes about a second.
"""
from __future__ import annotations

import math
import os
import sys
from collections.abc import Mapping
from dataclasses import dataclass, field
from pathlib import Path

import yaml

from ovd.core import pressure_vessel as _pv
from ovd.core.barge_capacity import BargeCapacityInputs, compute as capacity
from ovd.core.models import BargeInputs
from ovd.core.tanks import calculate_tank_arrangement
from ovd.core.resistance import (EnvironmentInputs, ResistanceInputs, TugInputs,
                                             compute as resistance)

ROOT = Path(__file__).resolve().parents[2]


def calibration_dir(root: Path, cwd: Path, env: Mapping[str, str]) -> Path:
    """The calibration folder: `OVD_CALIBRATION_DIR` when set, else `calibration/` beside the package's
    repo root (a checkout or an editable install), else `calibration/` in the working directory.

    After `pip install .` the root is in site-packages, which has no `calibration/` (issue #664), so the
    release check run from the checkout finds the folder, and `CHANGELOG.md` beside it, in the working
    directory. When neither folder exists the root's folder is returned, so the error names it.
    """
    if env.get("OVD_CALIBRATION_DIR"):
        return Path(env["OVD_CALIBRATION_DIR"])
    for base in (root, cwd):
        if (base / "calibration").is_dir():
            return base / "calibration"
    return root / "calibration"


# The calibration folder. Importing this module reads no file, so it imports from an installed wheel
# (issue #217). A caller may also set CAL.
CAL = calibration_dir(ROOT, Path.cwd(), os.environ)
RHO_FRESH = 1.0
KMH_PER_KN = 1.852


class _Bands(Mapping):
    """`CAL / "bands.yaml"`, read when first used and again when CAL names another folder."""

    def __init__(self):
        self._path, self._data = None, None

    def _load(self) -> dict:
        if self._path != CAL / "bands.yaml":
            self._data = yaml.safe_load((CAL / "bands.yaml").read_text(encoding="utf-8"))
            self._path = CAL / "bands.yaml"
        return self._data

    def __getitem__(self, key):
        return self._load()[key]

    def __iter__(self):
        return iter(self._load())

    def __len__(self):
        return len(self._load())


BANDS = _Bands()


# --- Independent lightship estimators (published, not ours) --------------------------------

def ls_padovezi(L: float, B: float, D: float) -> float:
    """Brazilian chata steel = lightship: 0.12 t/m3 x L.B.D (Padovezi 2003, USP thesis)."""
    return 0.12 * L * B * D


def ls_cn_draft(L: float, B: float, D: float, C: float = 0.128) -> float:
    """GB 38030.1 draft note (2018) §2.2.1: C.Lpp^1.4.B^0.75.D^0.4 (bulk C=0.128). Lpp ~ LOA here."""
    return C * L ** 1.4 * B ** 0.75 * D ** 0.4


def pct(model: float | None, obs: float | None) -> float | None:
    if model is None or obs in (None, 0):
        return None
    return 100.0 * (model - obs) / obs


def lbd(c: dict) -> float:
    return c["loa_m"] * c["beam_m"] * c["depth_m"]


def _is_held_out(c: dict) -> bool:
    """A held-out case is run and banded but never used to fit a factor (issue #3)."""
    return bool(c.get("held_out"))


def _rho_of(c: dict) -> float:
    """Water density (t/m3) for the case's predicted DWT: 1.000 (fresh) unless the case says so."""
    return c.get("water_density_t_m3", RHO_FRESH)


def _run_hold_type(c: dict) -> str:
    """The hold type the core runs: `run_as` when the case has one (a deck barge runs as box)."""
    return c.get("run_as", c["hold_type"])


def fit_factors(cases: list) -> dict:
    """Fit the barge factors from the cases. The fit IS the algorithm: re-run to regenerate.

    k (t/m3, lightship / L.B.D) per hold type: mean over cases with an observed lightship.
    Cb (box barges, on LOA): mean of (observed DWT + lightship) / (L.B.T.rho) over cases with
    a draught and DWT, not excluded; lightship = observed, else the fitted k.

    Held-out cases (a truthy `held_out`) and cases with `exclude_from_fit` (a one-line reason) are dropped
    first, here, so no caller can fit on them.
    """
    cases = [c for c in cases if not _is_held_out(c) and not c.get("exclude_from_fit")]
    k = {}
    for ht in sorted({c["hold_type"] for c in cases}):
        pts = [(c["id"], c["observed"]["lightship_t"] / lbd(c), c["confidence"])
               for c in cases if c["hold_type"] == ht and "lightship_t" in c["observed"]]
        if pts:
            vals = [v for _, v, _ in pts]
            k[ht] = {"value": round(sum(vals) / len(vals), 4), "min": round(min(vals), 4),
                     "max": round(max(vals), 4), "n": len(vals),
                     "evidence": [{"case": i, "k": round(v, 4), "confidence": cf} for i, v, cf in pts]}
    cb_pts = []
    for c in cases:
        o = c["observed"]
        if c["hold_type"] != "box" or not c.get("draught_m") or "dwt_t" not in o \
                or c.get("exclude_from_fit_cb"):
            continue
        ls = o.get("lightship_t", k["box"]["value"] * lbd(c))
        cb_pts.append((c["id"], (o["dwt_t"] + ls) / (c["loa_m"] * c["beam_m"] * c["draught_m"] * _rho_of(c))))
    vals = [v for _, v in cb_pts]
    cb = {"value": round(sum(vals) / len(vals), 3), "min": round(min(vals), 3),
          "max": round(max(vals), 3), "n": len(vals),
          "evidence": [{"case": i, "cb": round(v, 3)} for i, v in cb_pts],
          "excluded": [{"case": c["id"], "why": c["exclude_from_fit_cb"]}
                       for c in cases if c.get("exclude_from_fit_cb")]}
    return {"lightship_k_t_per_m3": k, "block_coefficient_box_barge_on_loa": cb}


@dataclass
class BargeRow:
    case: dict
    ls_ported: float
    ls_cal: float
    ls_padovezi: float
    ls_cn: float
    dwt_ported: float | None = None
    dwt_cal: float | None = None
    ls_from_light_draught_cb1: float | None = None


def run_barge(c: dict, f: dict) -> BargeRow:
    L, B, D, T = c["loa_m"], c["beam_m"], c["depth_m"], c.get("draught_m")
    ported = capacity(BargeCapacityInputs(loa=L, beam=B, depth=D, cargo_type="bauxite",
                                          self_propelled=False))
    ht, rho = _run_hold_type(c), _rho_of(c)
    k = f["lightship_k_t_per_m3"][ht]["value"]
    cb = f["block_coefficient_box_barge_on_loa"]["value"]
    cal = capacity(BargeCapacityInputs(loa=L, beam=B, depth=D, cargo_type="bauxite",
                                       self_propelled=False, hold_type=ht,
                                       lightship_k_t_per_m3=k, block_coefficient=cb))
    row = BargeRow(case=c, ls_ported=ported.lightship_t, ls_cal=cal.lightship_t,
                   ls_padovezi=ls_padovezi(L, B, D), ls_cn=ls_cn_draft(L, B, D))
    if T:
        row.dwt_ported = 0.85 * L * B * T * rho - ported.lightship_t
        row.dwt_cal = cb * L * B * T * rho - cal.lightship_t
    if "light_draught_m" in c["observed"]:
        row.ls_from_light_draught_cb1 = L * B * c["observed"]["light_draught_m"] * rho
    return row


@dataclass
class MotorRow:
    cargo: str
    loa: float
    boa: float
    T: float
    disp: float
    cargo_t: float
    power_kw: float
    speed_kmh: float
    cb_implied: float
    ls_plus_consumables: float
    delivered_kw: float | None


def run_motor(cargo: str, r: list, speed_kmh: float) -> MotorRow:
    loa, boa, T, disp, cargo_t, p = r
    cb = disp / (loa * boa * T * RHO_FRESH)   # on LOA: a lower bound on the true Cb (Lwl < LOA)
    v = speed_kmh / KMH_PER_KN
    deep = EnvironmentInputs(name="deep (calibration)", water_depth=50.0,
                             channel_width_bottom=1000.0, channel_width_surface=1000.0,
                             water_density=RHO_FRESH, is_deep_water=True, apply_squat_limit=False)
    ri = ResistanceInputs(loa=loa, lwl=0.97 * loa, beam=boa, draft=T, block_coefficient=min(cb / 0.97, 0.99),
                          displacement_tonnes=disp, tug=TugInputs(engine_power_kw=1e6),
                          environments=[deep], speed_min_knots=v, speed_max_knots=v + 0.05,  # core rounds speeds to 0.1 kn
                          speed_step_knots=1.0)
    pts = resistance(ri).environments[0].speed_curve
    delivered = pts[0].delivered_power_kw if pts else None
    return MotorRow(cargo, loa, boa, T, disp, cargo_t, p, speed_kmh, cb, disp - cargo_t, delivered)


# --- Calibration Bands -----------------------------------------------------------------------

def load_cases() -> dict:
    return yaml.safe_load((CAL / "cases.yaml").read_text(encoding="utf-8"))


@dataclass
class BandCheck:
    """One calibrated result against its band."""
    case: str
    figure: str                 # "lightship_t" | "dwt_t"
    observed: float
    model: float
    err_pct: float
    band_pct: float
    confidence: str
    basis: str
    excluded: str | None = None     # why the case is left out of the Cb fit, when it is

    @property
    def inside(self) -> bool:
        return abs(self.err_pct) <= self.band_pct


@dataclass
class _Banded(BandCheck):
    """What `assess` returns: a BandCheck that also says whether the case is held out of the fit
    and which hull group it belongs to (for the summary). A subclass, so BandCheck keeps its fields."""
    held_out: str | None = None     # why the case is held out of the fit, when it is
    group: str = ""                 # "box", "container", "deck (run as box)"


def _group(c: dict) -> str:
    """The hull group of a case for the summary: its hold type, and what it runs as when different."""
    ht, run = c["hold_type"], _run_hold_type(c)
    return ht if ht == run else f"{ht} (run as {run})"


def _band_check(c: dict, figure: str, model: float) -> BandCheck:
    band = BANDS["bands"][figure]
    obs = c["observed"][figure]
    return _Banded(case=c["id"], figure=figure, observed=obs, model=model, err_pct=pct(model, obs),
                     band_pct=float(band["pct"][c["confidence"]]), confidence=c["confidence"],
                     basis=band["basis"], excluded=c.get("exclude_from_fit_cb") or c.get("exclude_from_fit"),
                     held_out=c.get("held_out") or None, group=_group(c))


def assess(cases: dict | None = None) -> list[BandCheck]:
    """Every calibrated result that has a published figure, against its band. A lightship given
    only as an upper bound (light draught, Cb = 1) has no band: it is a bound, not a figure."""
    cases = cases if cases is not None else load_cases()
    f = fit_factors(cases["dry_barge"])
    out = []
    for c in cases["dry_barge"]:
        if c.get("cross_check_only") or c.get("not_yet_runnable"):
            continue                    # a cross-check has no band; a case that cannot run is not run
        r = run_barge(c, f)
        if "lightship_t" in c["observed"]:
            out.append(_band_check(c, "lightship_t", r.ls_cal))
        if "dwt_t" in c["observed"] and r.dwt_cal is not None:
            out.append(_band_check(c, "dwt_t", r.dwt_cal))
    return out


def github_warning(message: str, title: str = "", file: str = "") -> str:
    """A GitHub Actions warning annotation. `%`, CR and LF are escaped as the runner requires."""
    esc = message.replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A")
    props = ",".join(p for p in (f"file={file}" if file else "", f"title={title}" if title else "") if p)
    return f"::warning {props}::{esc}" if props else f"::warning::{esc}"


def warnings_for(checks: list[BandCheck]) -> list[str]:
    """One annotation per result outside its band."""
    lines = []
    for b in checks:
        if b.inside:
            continue
        label = BANDS["bands"][b.figure]["label"].lower()
        note = f"; case excluded from the Cb fit: {b.excluded}" if b.excluded else ""
        lines.append(github_warning(
            f"{b.case}: calibrated {label} is {b.err_pct:+.1f} % from the published "
            f"{b.observed:g} t (model {b.model:,.0f} t), outside ±{b.band_pct:g} % "
            f"({b.confidence} confidence, {'held-out' if getattr(b, 'held_out', None) else 'in-sample'}){note}",
            title="Calibration band", file="calibration/cases.yaml"))
    return lines


# --- Tank-barge cases (issue #335) -----------------------------------------------------------------
# A tank barge: lightship = k x L x B x D + outfit (`tank_lightship.yaml`, issue #330), and the deadweight
# at the stated draught is the box-barge Cb x L x B x T x rho less that lightship. It runs every
# `hold_type: tank` record of cases.yaml and the vessels of `tank_barge_cases.yaml`. The k-fit subset
# (`tank_lightship_inputs.yaml`) is in-sample; every other case is out of sample. The pass measures; it
# does not tune.

def _load_tank_barge_cases() -> dict:
    return yaml.safe_load((CAL / "tank_barge_cases.yaml").read_text(encoding="utf-8"))


def _load_tank_barge_bands() -> dict:
    return yaml.safe_load((CAL / "tank_barge_bands.yaml").read_text(encoding="utf-8"))["bands"]


def _load_tank_lightship() -> tuple[dict, dict]:
    """(the generated `tank_lightship.yaml`, the declared `tank_lightship_inputs.yaml`)."""
    return (yaml.safe_load((CAL / "tank_lightship.yaml").read_text(encoding="utf-8")),
            yaml.safe_load((CAL / "tank_lightship_inputs.yaml").read_text(encoding="utf-8")))


def _tank_barge_records(cases: dict | None = None) -> list[dict]:
    """Every tank-barge record: the `hold_type: tank` ones of cases.yaml (in either of its lists), then
    the reference vessels of tank_barge_cases.yaml."""
    cases = cases if cases is not None else load_cases()
    found = [c for key in ("dry_barge", "not_yet_runnable_cases") for c in cases.get(key, [])
             if c.get("hold_type") == "tank"]
    return found + list(_load_tank_barge_cases()["tank_barge"])


def _tank_outfit_t(volume_m3: float, outfit: dict, heating_kw: float = 0.0) -> float:
    """The outfit (t): pumps, piping, vents and gauging over the tank volume, and the heating. `outfit`
    is the `outfit` block of tank_lightship.yaml. Equal to `liquid_bulk::lightship::outfit_t` of the core."""
    per_m3 = sum(outfit[k]["value"] for k in ("pumps_t_per_m3", "piping_t_per_m3", "vents_gauging_t_per_m3"))
    return volume_m3 * per_m3 + heating_kw * outfit["heating_t_per_kw"]["value"]


@dataclass
class _TankBargeRow:
    case: dict
    sample: str                      # "in-sample" (in the k fit) | "held out" | "not scored"
    ls_model: float | None           # None when the record has no tank volume or depth
    dwt_model: float | None          # None when the record has no draught
    banded: str | None = None        # why the case has no band, when it has none


def _run_tank_barge(c: dict, f: dict, tank_lightship: dict, inputs: dict) -> _TankBargeRow:
    """One tank-barge record through the tank-barge model. `f` is the fitted factors (the box-barge Cb)."""
    o = c["observed"]
    L, B, D, T = c["loa_m"], c["beam_m"], c.get("depth_m"), c.get("draught_m")
    k = tank_lightship["tank_lightship_k_t_per_m3"]["value"]
    cb = f["block_coefficient_box_barge_on_loa"]["value"]
    ls = None
    if D and "tank_capacity_m3" in o:
        ls = k * L * B * D + _tank_outfit_t(o["tank_capacity_m3"], tank_lightship["outfit"])
    dwt = cb * L * B * T * _rho_of(c) - ls if (T and ls is not None) else None
    if c["id"] in inputs.get("not_scored", {}):
        sample, why = "not scored", inputs["not_scored"][c["id"]]
    else:
        sample = "in-sample" if c["id"] in inputs["fit_subset"] else "held out"
        why = c.get("not_banded") or c.get("cross_check_only")
    return _TankBargeRow(case=c, sample=sample, ls_model=ls, dwt_model=dwt, banded=why)


def _tank_barge_rows(cases: dict | None = None) -> list[_TankBargeRow]:
    cases = cases if cases is not None else load_cases()
    f = fit_factors(cases["dry_barge"])
    tank_lightship, inputs = _load_tank_lightship()
    return [_run_tank_barge(c, f, tank_lightship, inputs) for c in _tank_barge_records(cases)]


def _assess_tank_barges(cases: dict | None = None) -> list[BandCheck]:
    """Every tank-barge figure that has a published value and a band: the deadweight at the stated
    draught (a published deadweight, not a cargo capacity), and a lightship when a record gives one."""
    bands = _load_tank_barge_bands()
    out = []
    for r in _tank_barge_rows(cases):
        c, o = r.case, r.case["observed"]
        if r.banded:
            continue
        for fig, model in (("lightship_t", r.ls_model), ("dwt_t", r.dwt_model)):
            if fig not in o or model is None:
                continue
            band = bands[fig]
            out.append(_Banded(case=c["id"], figure=fig, observed=o[fig], model=model,
                               err_pct=pct(model, o[fig]), band_pct=float(band["pct"][c["confidence"]]),
                               confidence=c["confidence"], basis=band["basis"],
                               held_out=c.get("held_out") if r.sample == "held out" else None,
                               group="tank barge"))
    return out


def _tank_barge_warnings_for(checks: list[BandCheck]) -> list[str]:
    """One annotation per tank-barge result outside its band."""
    bands = _load_tank_barge_bands()
    lines = []
    for b in checks:
        if b.inside:
            continue
        label = bands[b.figure]["label"].lower()
        lines.append(github_warning(
            f"{b.case}: modelled {label} is {b.err_pct:+.1f} % from the published {b.observed:g} t "
            f"(model {b.model:,.0f} t), outside ±{b.band_pct:g} % ({b.confidence} confidence, "
            f"{'held-out' if b.held_out else 'in-sample'})",
            title="Calibration band", file="calibration/tank_barge_cases.yaml"))
    return lines


def _tank_barge_section() -> list[str]:
    bands = _load_tank_barge_bands()
    tank_lightship, inputs = _load_tank_lightship()
    rows = _tank_barge_rows()
    checks = _assess_tank_barges()
    k = tank_lightship["tank_lightship_k_t_per_m3"]
    out = ["", "## 5a. Tank barges (issue #335)", "",
           "Every `hold_type: tank` record of `cases.yaml` and the reference vessels of "
           "`tank_barge_cases.yaml`, run through the tank-barge model: lightship = k × L·B·D + outfit "
           f"(k = {k['value']} t/m³, an Estimate; outfit coefficients are Estimates), and the deadweight at "
           "the stated draught is the box-barge Cb × L·B·T·ρ less that lightship. Bands are in "
           "`tank_barge_bands.yaml`: starting values for the maintainer, equal to the dry-barge bands. "
           f"**The {len(inputs['fit_subset'])} cases of the k-fit subset are in-sample** (k was fitted on "
           "them, from the same deadweights); every other case is out of sample. A **self-propelled** vessel "
           "has no band: the model has no machinery, accommodation or fuel, so its deadweight comes out too "
           "high by about that weight. A cargo capacity is not a deadweight, so it is shown against the "
           "predicted deadweight and has no band. A band miss is a warning only.", "",
           "| Case | Conf. | Sample | T (m) | Published (t) | Model lightship (t) | Model DWT (t) | err % | "
           "Band ± % | vs band | Note |", "|---|---|---|---|---|---|---|---|---|---|---|"]
    for r in rows:
        c, o = r.case, r.case["observed"]
        if "dwt_t" in o:
            pub = fmt(o["dwt_t"])
        elif "cargo_capacity_t" in o:
            pub = f"{fmt(o['cargo_capacity_t'])} (cargo)"
        else:
            pub = "–"
        compared = o.get("dwt_t", o.get("cargo_capacity_t"))
        err = pct(r.dwt_model, compared) if r.dwt_model is not None and compared else None
        cells = _band_cells(checks, c["id"], "dwt_t")
        note = r.banded or ("in the k fit" if r.sample == "in-sample" else "")
        out.append(f"| {c['id']} | {c['confidence']} | {r.sample} | {fmt(c.get('draught_m'), 2)} | {pub} | "
                   f"{fmt(r.ls_model)} | {fmt(r.dwt_model)} | {fmt(err, 1)} | {cells[0]} | {cells[1]} | {note} |")
    banded = [b for b in checks if b.figure == "dwt_t"]
    out += ["", "| Sample | Banded n | Mean err % | Mean abs err % | Range % | Inside band |", "|---|---|---|---|---|---|"]
    for label, pick in (("in-sample (k-fit subset)", lambda b: not b.held_out), ("out of sample", lambda b: bool(b.held_out))):
        st = _summary_stats([b for b in banded if pick(b)])
        if st is None:
            out.append(f"| {label} | 0 | – | – | – | – |")
        else:
            out.append(f"| {label} | {st['n']} | {st['mean']:+.1f} | {st['mean_abs']:.1f} | "
                       f"{st['min']:+.1f} to {st['max']:+.1f} | {st['inside']} of {st['n']} |")
    return out


# --- Type C tank cases (issue #292) --------------------------------------------------------------
# One cargo tank, run through the tank maths only (no hull): `calibration/tank_cases.yaml`, with its
# own bands in `tank_bands.yaml`. Every case is held out: it is never used to fit a factor. The
# pass measures; it does not tune.

_TANK_FIGURES = ("tank_volume_m3", "tank_wall_mm", "tank_steel_t")


def _load_tank_cases() -> dict:
    return yaml.safe_load((CAL / "tank_cases.yaml").read_text(encoding="utf-8"))


def _load_tank_bands() -> dict:
    return yaml.safe_load((CAL / "tank_bands.yaml").read_text(encoding="utf-8"))["bands"]


@dataclass
class _TankRow:
    """One tank case, run. Diameters and the length are in the engine's datum (see `_tank_geometry`)."""
    case: dict
    steel_od_m: float
    insulated_od_m: float
    insulated_length_m: float
    volume_m3: float
    required_mm: float          # the shell formula before the allowance and the rounding
    wall_mm: float              # the full chain: allowance, rounded up to 0.5 mm, the IGC minimum
    head_wall_mm: float
    steel_t: float | None       # tank steel at the published shell thickness, times the structure factor
    steel_formula_t: float | None  # the same with the thickness the formula gives


def _tank_geometry(c: dict) -> tuple[float, float, float]:
    """(steel outer diameter, insulated outer diameter, insulated overall length), all in m.

    The engine's tank outer diameter is the outside of the insulation. A source prints the diameter
    on one datum (`diameter_basis`) and the length on another (`length_basis`); both are brought
    to the engine's. `shell_thickness_mm` and `insulation_mm` of the case do the conversion.
    """
    wall, ins = c["observed"]["shell_thickness_mm"] / 1000.0, c.get("insulation_mm", 0.0) / 1000.0
    d, basis = c["diameter_m"], c["diameter_basis"]
    steel_od = {"steel_outer": d, "steel_inner": d + 2 * wall, "insulation_outer": d - 2 * ins}[basis]
    length, lb = c["length_m"], c["length_basis"]
    envelope = {"steel_outer_overall": length + 2 * ins, "inner_overall": length + 2 * (wall + ins),
                "insulation_outer_overall": length}[lb]
    return steel_od, steel_od + 2 * ins, envelope


def _tank_inputs(c: dict, od: float, length: float, wall_mm: float | None) -> BargeInputs:
    """One tank filling the envelope: no side clearance, no gaps, no non-tank length."""
    return BargeInputs(
        loa=length, beam=od, depth=1.0, max_draft=1.0, vertical_clearance=10.0,
        n_tanks_wide=1, n_tanks_long=1, bow_non_tank_length=0.0, stern_non_tank_length=0.0,
        side_clearance=0.0, double_bottom_height=0.0, tank_wall_thickness=wall_mm,
        tank_head_type=c["run_as_head"], tank_material=c["run_as_material"],
        design_pressure=c["design_pressure_mpa"], weld_efficiency=c.get("weld_efficiency", 0.95),
        insulation_thickness=c.get("insulation_mm", 0.0), tank_structure_factor=c["structure_factor"],
        fill_ratio=1.0)


def _run_tank(c: dict) -> _TankRow:
    steel_od, od, length = _tank_geometry(c)
    published = c["observed"]["shell_thickness_mm"]
    # Volume and steel at the published shell thickness: the geometry and the datum are what is tested.
    fixed = calculate_tank_arrangement(_tank_inputs(c, od, length, published))
    # The wall from the formula, on the steel diameter (the engine's gas path does the same).
    cyl, head = _pv.calculate_wall_thicknesses(steel_od, c["design_pressure_mpa"], c["run_as_material"],
                                               c["run_as_head"], c.get("weld_efficiency", 0.95))
    mat = _pv.get_tank_material(c["run_as_material"])
    f, e, p = mat.allowable_stress, c.get("weld_efficiency", 0.95), c["design_pressure_mpa"]
    r = steel_od * 1000.0 / 2.0
    required = _pv.cylinder_wall_thickness(p, r - _pv.cylinder_wall_thickness(p, r, f, e), f, e)
    steel_t = steel_formula_t = None
    if "tank_steel_t" in c["observed"]:
        steel_t = fixed.tank_shell_weight * c["structure_factor"]
        formula = calculate_tank_arrangement(_tank_inputs(c, od, length, cyl))
        steel_formula_t = formula.tank_shell_weight * c["structure_factor"]
    return _TankRow(case=c, steel_od_m=steel_od, insulated_od_m=od, insulated_length_m=length,
                   volume_m3=fixed.total_tank_volume, required_mm=required, wall_mm=cyl,
                   head_wall_mm=head, steel_t=steel_t, steel_formula_t=steel_formula_t)


def _tank_model(row: _TankRow, figure: str) -> float | None:
    return {"tank_volume_m3": row.volume_m3, "tank_wall_mm": row.wall_mm, "tank_steel_t": row.steel_t}[figure]


def _assess_tanks(cases: dict | None = None) -> list[BandCheck]:
    """Every tank figure that has a published value, against its band."""
    cases = cases if cases is not None else _load_tank_cases()
    bands = _load_tank_bands()
    out = []
    for c in cases["type_c_tank"]:
        row = _run_tank(c)
        for fig, key in (("tank_volume_m3", "gross_volume_m3"), ("tank_wall_mm", "shell_thickness_mm"),
                         ("tank_steel_t", "tank_steel_t")):
            if key not in c["observed"]:
                continue
            band = bands[fig]
            out.append(_Banded(case=c["id"], figure=fig, observed=c["observed"][key],
                               model=_tank_model(row, fig), err_pct=pct(_tank_model(row, fig), c["observed"][key]),
                               band_pct=float(band["pct"][c["confidence"]]), confidence=c["confidence"],
                               basis=band["basis"], held_out=c.get("held_out") or None, group="type C tank"))
    return out


def _accepted_misses() -> dict[tuple[str, str], str]:
    """(case, figure) -> the reason a tank band miss is accepted (`accepted_miss` of a case)."""
    return {(c["id"], fig): why for c in _load_tank_cases()["type_c_tank"]
            for fig, why in c.get("accepted_miss", {}).items()}


def _tank_warnings_for(checks: list[BandCheck]) -> list[str]:
    """One annotation per tank result outside its band."""
    bands = _load_tank_bands()
    lines = []
    for b in checks:
        if b.inside:
            continue
        label = bands[b.figure]["label"].lower()
        accepted = _accepted_misses().get((b.case, b.figure))
        lines.append(github_warning(
            f"{b.case}: modelled {label} is {b.err_pct:+.1f} % from the published {b.observed:g} "
            f"(model {b.model:,.1f}), outside ±{b.band_pct:g} % ({b.confidence} confidence, held-out)"
            + (f"; accepted: {accepted}" if accepted else ""),
            title="Calibration band", file="calibration/tank_cases.yaml"))
    return lines


def _tank_section() -> list[str]:
    cases = _load_tank_cases()
    bands = _load_tank_bands()
    rows = [_run_tank(c) for c in cases["type_c_tank"]]
    checks = _assess_tanks(cases)
    out = ["", "## 5. Type C tank cases (held out)", "",
           "One cargo tank, run through the tank maths only (`tank_cases.yaml`, bands in `tank_bands.yaml`). "
           "Every case is held out: it is never used to fit a factor. The engine's tank diameter is the "
           "outside of the insulation, so each source's diameter and length are first brought to that "
           "datum (`diameter_basis`, `length_basis`). Volume and steel are run at the published shell "
           "thickness; the wall is the pressure-vessel formula at the steel diameter, with the "
           "material the case says it is run as. **Steel weight of the first case is in-sample for the "
           "structure factor** of fully pressurised LPG in the Defaults Profile, which was worked out from "
           "that one published weight. A miss is a warning only.", "",
           "| Case | Conf. | Figure | Published | Model | err % | Band ± % | vs band |", "|---|---|---|---|---|---|---|---|"]
    for row in rows:
        c = row.case
        for b in (x for x in checks if x.case == c["id"]):
            unit = "m³" if b.figure == "tank_volume_m3" else ("mm" if b.figure == "tank_wall_mm" else "t")
            note = " (in-sample)" if b.figure == "tank_steel_t" and c.get("steel_in_sample") else ""
            if b.figure in c.get("accepted_miss", {}):
                note += f" (accepted: {c['accepted_miss'][b.figure]})"
            out.append(f"| {c['id']} | {c['confidence']} | {bands[b.figure]['label']} ({unit}) | "
                       f"{fmt(b.observed, 1)} | {fmt(b.model, 1)} | {fmt(b.err_pct, 1)} | ±{b.band_pct:g} | "
                       f"{'inside' if b.inside else '**OUTSIDE** ⚠'}{note} |")
    out += ["", "| Case | Steel outer Ø (m) | Insulated outer Ø (m) | Insulated length (m) | Formula before allowance (mm) | "
            "Head wall (mm) | Steel at formula thickness (t) |", "|---|---|---|---|---|---|---|"]
    for row in rows:
        out.append(f"| {row.case['id']} | {row.steel_od_m:.3f} | {row.insulated_od_m:.3f} | "
                   f"{row.insulated_length_m:.3f} | {row.required_mm:.2f} | {row.head_wall_mm:g} | "
                   f"{fmt(row.steel_formula_t, 1)} |")
    out += ["", "### Tank records that cannot run yet", "", "Listed with their published figures and the reason. "
            "They are never run.", "", "| Case | Published | Why not run |", "|---|---|---|"]
    for c in cases.get("not_yet_runnable_tank_cases", []):
        out.append(f"| {c['id']} | {c['published']} | {c['not_yet_runnable']} |")
    records = cases.get("tank_weight_records", [])
    if records:
        out += ["", "### Tank weight per m³ (published records, issue #317)", "",
                "Published empty weights of marine LNG Type C tanks over their gross volume. They are the check on "
                "the tank structure factor before it is tuned. A record runs only when its diameter and length "
                "have a stated basis; none has one yet, so none is run.", "",
                "| Record | Conf. | Gross volume (m³) | Published weight (t) | t per m³ | What the weight includes | "
                "Why not run |", "|---|---|---|---|---|---|---|"]
        for c in records:
            out.append(f"| {c['id']} | {c['confidence']} | {fmt(c['gross_volume_m3'], 0)} | "
                       f"{fmt(c['tank_weight_t'], 0)} | {_tank_weight_per_m3(c):.3f} | {c['weight_includes']} | "
                       f"{c['not_yet_runnable']} |")
        lo, hi = _tank_weight_range()
        out += ["", f"Published range: {lo:.3f} to {hi:.3f} t per m³ of tank."]
    return out


def _tank_weight_per_m3(record: dict) -> float:
    """The published tank weight over the published gross volume (t per m³), arithmetic on two printed
    figures (issue #317)."""
    return record["tank_weight_t"] / record["gross_volume_m3"]


def _tank_weight_range() -> tuple[float, float]:
    """The lowest and highest published tank weight per m³ of the tank weight records."""
    values = [_tank_weight_per_m3(c) for c in _load_tank_cases().get("tank_weight_records", [])]
    return min(values), max(values)


# --- Propulsion cases (issue #351, spec 0007) -----------------------------------------------------
# Tugs and pushers with a published bollard pull, run through the Propulsor model only (no hull):
# `calibration/propulsion_cases.yaml`, banded in `bands.yaml` (`bollard_pull_t`). The in-sample cases
# of a type fit its bollard pull factor kappa_b (the mean of published over model at kappa_b = 1); the
# held-out cases are run and banded, never fitted.

# The transmission of spec 0007 "Power and transmission": the diesel-mechanical gearbox for engine
# power, then the shaft line or the azimuth drive.
_GEARBOX = {"engine": 0.97, "motor": 1.0}
_SHAFT = {"shaft": 0.99, "azimuth": 0.95}
# kappa_b of the types with no runnable case: the open-propeller value, 0.9 kN per (D P)^(2/3) of the
# Wartsila estimate with k2 = 1 over the disc limit at 1,025 kg/m3 (derived, spec 0007), an estimate.
_KAPPA_B_OPEN = 0.77
_PROPULSOR_TYPES = ("azimuth_open", "azimuth_ducted", "shaft_open", "shaft_nozzle", "voith_schneider",
                    "pump_jet")


def _load_propulsion_cases() -> dict:
    return yaml.safe_load((CAL / "propulsion_cases.yaml").read_text(encoding="utf-8"))


def _delivered_kw(c: dict) -> float:
    """P_D of one unit: the published power through the spec 0007 transmission."""
    return c["power_per_unit_kw"] * _GEARBOX[c["power_basis"]] * _SHAFT[c["drive"]]


def _bollard_pull(c: dict, kappa_b: float) -> float:
    """The engine's bollard pull of the case (t), at the factor `kappa_b`."""
    from ovd import _bridge
    out = _bridge.raw("propulsor", "bollard_pull", {
        "inputs": {"type": c["run_as_type"], "count": int(c["count"]), "diameter_m": float(c["diameter_m"]),
                   "kappa_b": float(kappa_b), "rho_kg_m3": float(c["water_density_kg_m3"])},
        "pd_kw": float(_delivered_kw(c))})
    if "error" in out:
        raise RuntimeError(out["error"])
    return float(out["ok"])


def _fit_bollard_pull_factor(cases: list) -> dict:
    """kappa_b per Propulsor type: the mean of published over model at kappa_b = 1 over the in-sample
    cases of the type; a type with none takes the open-propeller estimate."""
    out = {}
    for t in _PROPULSOR_TYPES:
        fit = [c for c in cases if c["run_as_type"] == t and not _is_held_out(c)]
        if not fit:
            out[t] = {"value": _KAPPA_B_OPEN, "n": 0,
                      "basis": "no in-sample case: the open-propeller value (Wartsila, k2 = 1), an estimate"}
            continue
        ratios = [(c["id"], c["observed"]["bollard_pull_t"] / _bollard_pull(c, 1.0)) for c in fit]
        k = sum(r for _, r in ratios) / len(ratios)
        out[t] = {"value": round(k, 3), "min": round(min(r for _, r in ratios), 3),
                  "max": round(max(r for _, r in ratios), 3), "n": len(ratios),
                  "evidence": [{"case": i, "kappa_b": round(r, 3)} for i, r in ratios]}
    return out


def _assess_propulsion(cases: dict | None = None) -> list[BandCheck]:
    cases = cases if cases is not None else _load_propulsion_cases()
    factors = _fit_bollard_pull_factor(cases["propulsion_case"])
    band = BANDS["bands"]["bollard_pull_t"]
    out = []
    for c in cases["propulsion_case"]:
        obs = c["observed"]["bollard_pull_t"]
        model = _bollard_pull(c, factors[c["run_as_type"]]["value"])
        out.append(_Banded(case=c["id"], figure="bollard_pull_t", observed=obs, model=model,
                           err_pct=pct(model, obs), band_pct=float(band["pct"][c["confidence"]]),
                           confidence=c["confidence"], basis=band["basis"],
                           held_out=c.get("held_out") or None, group=c["run_as_type"]))
    return out


def _propulsion_warnings_for(checks: list[BandCheck]) -> list[str]:
    """One annotation per bollard pull outside its band, with the reason when the miss is accepted."""
    accepted = {(c["id"], fig): why for c in _load_propulsion_cases()["propulsion_case"]
                for fig, why in c.get("accepted_miss", {}).items()}
    lines = []
    for b in checks:
        if b.inside:
            continue
        why = accepted.get((b.case, b.figure))
        lines.append(github_warning(
            f"{b.case}: modelled bollard pull is {b.err_pct:+.1f} % from the published {b.observed:g} t "
            f"(model {b.model:,.1f} t), outside ±{b.band_pct:g} % ({b.confidence} confidence, "
            f"{'held-out' if b.held_out else 'in-sample'})" + (f"; accepted: {why}" if why else ""),
            title="Calibration band", file="calibration/propulsion_cases.yaml"))
    return lines


def _propulsion_section() -> list[str]:
    cases = _load_propulsion_cases()
    factors = _fit_bollard_pull_factor(cases["propulsion_case"])
    checks = {b.case: b for b in _assess_propulsion(cases)}
    out = ["", "## 6. Bollard pull (propulsion cases, spec 0007)", "",
           "Tugs and pushers with a published bollard pull, run through the Propulsor model only "
           "(`propulsion_cases.yaml`, band `bollard_pull_t` in `bands.yaml`): kappa_b x (2 rho A)^(1/3) "
           "P_D^(2/3) per unit, P_D the published power through the spec 0007 transmission (gearbox 0.97 "
           "for engine power; shaft line 0.99, azimuth drive 0.95). The in-sample cases of a type fit its "
           "kappa_b; **their errors are in-sample**. Held-out cases were never used in the fit. A miss is "
           "a warning only.", "", "### Fitted bollard pull factors", ""]
    for t, f in factors.items():
        if f["n"]:
            out.append(f"- **{t}:** kappa_b {f['value']} (range {f['min']}–{f['max']}, n = {f['n']})")
        else:
            out.append(f"- **{t}:** kappa_b {f['value']} ({f['basis']})")
    out += ["", "| Case | Type | Sample | Conf. | n × D (m) | P_D per unit (kW) | Published (t) | Model (t) | "
            "err % | t per 100 kW published | Band ± % | vs band |", "|---|---|---|---|---|---|---|---|---|---|---|---|"]
    for c in cases["propulsion_case"]:
        b = checks[c["id"]]
        note = ""
        if "bollard_pull_t" in c.get("accepted_miss", {}):
            note = f" (accepted: {c['accepted_miss']['bollard_pull_t']})"
        per_100 = 100.0 * b.observed / (c["count"] * c["power_per_unit_kw"])
        out.append(f"| {c['id']} | {c['run_as_type']} | {'held out' if b.held_out else 'in-sample'} | "
                   f"{c['confidence']} | {c['count']} × {c['diameter_m']:g} | {fmt(_delivered_kw(c))} | "
                   f"{fmt(b.observed, 1)} | {fmt(b.model, 1)} | {fmt(b.err_pct, 1)} | {per_100:.2f} | "
                   f"±{b.band_pct:g} | {'inside' if b.inside else '**OUTSIDE** ⚠'}{note} |")
    out += ["", "### Propulsion records that cannot run yet", "", "Listed with their published figures and the "
            "reason. They are never run.", "", "| Case | Published | Why not run |", "|---|---|---|"]
    for c in cases.get("not_yet_runnable_propulsion_cases", []):
        out.append(f"| {c['id']} | {c['published']} | {c['not_yet_runnable']} |")
    return out


# Installed power (issue #353): inland ships that print both the installed power and a design
# speed, run through the whole engine. The in-sample cases of a type fit its efficiency factor kappa;
# the Installed Power is close to inverse in kappa, so the fit is the kappa at which the in-sample
# model meets the published power, found by a few steps of kappa x model / published.
_KAPPA_START = 0.8


def _installed_power(c: dict, kappa: float) -> float:
    """The Installed Power the engine sizes for the case (kW), at the efficiency factor `kappa`."""
    from ovd.simulate import simulate
    out = simulate({
        "vessel": "hopper_barge", "cargo": "bauxite", "self_propelled": True,
        "loa_m": float(c["loa_m"]), "beam_m": float(c["beam_m"]), "depth_m": float(c["depth_m"]),
        "cb": None, "draught_limit_m": float(c["draught_m"]), "stern_form": "tunnel",
        "propulsor_type": c["run_as_type"], "propulsor_count": int(c["count"]),
        "propeller_diameter_m": float(c["diameter_m"]), "power_source": c["power_source"],
        "design_speed_kn": float(c["design_speed_kn"]), "installed_power_margin": 1.2,
        "propulsor_efficiency_factor": float(kappa), "speed_max_kn": 14.0,
    })
    return float(out["powering"]["installed_power_kw"])


def _fit_efficiency_factor(cases: list) -> dict:
    """kappa per Propulsor type from its in-sample installed power cases; a type with none keeps the
    starting estimate."""
    out = {}
    for t in _PROPULSOR_TYPES:
        fit = [c for c in cases if c["run_as_type"] == t and not _is_held_out(c)]
        if not fit:
            out[t] = {"value": _KAPPA_START, "n": 0,
                      "basis": "no in-sample case: the starting estimate"}
            continue
        k = _KAPPA_START
        for _ in range(4):
            ratio = sum(_installed_power(c, k) / c["observed"]["installed_power_kw"] for c in fit) / len(fit)
            k *= ratio
        out[t] = {"value": round(k, 3), "n": len(fit), "evidence": [{"case": c["id"]} for c in fit]}
    return out


def _assess_installed_power(cases: dict | None = None) -> list[BandCheck]:
    cases = cases if cases is not None else _load_propulsion_cases()
    factors = _fit_efficiency_factor(cases["installed_power_case"])
    band = BANDS["bands"]["installed_power_kw"]
    out = []
    for c in cases["installed_power_case"]:
        obs = c["observed"]["installed_power_kw"]
        model = _installed_power(c, factors[c["run_as_type"]]["value"])
        out.append(_Banded(case=c["id"], figure="installed_power_kw", observed=obs, model=model,
                           err_pct=pct(model, obs), band_pct=float(band["pct"][c["confidence"]]),
                           confidence=c["confidence"], basis=band["basis"],
                           held_out=c.get("held_out") or None, group=c["run_as_type"]))
    return out


def _installed_power_section() -> list[str]:
    cases = _load_propulsion_cases()
    factors = _fit_efficiency_factor(cases["installed_power_case"])
    checks = {b.case: b for b in _assess_installed_power(cases)}
    out = ["", "## 7. Installed power (spec 0007)", "",
           "Inland ships that print both the installed power and a design speed, run through the whole "
           "engine: the hull at its printed size and maximum draught, its propulsors, and the Installed "
           "Power sized for the design speed in deep water with the 1.20 margin. The in-sample cases of a "
           "type fit its efficiency factor kappa; **their errors are in-sample**. A miss is a warning only.",
           "", "### Fitted efficiency factors", ""]
    for t, f in factors.items():
        out.append(f"- **{t}:** kappa {f['value']}" + (f" (n = {f['n']})" if f["n"] else f" ({f['basis']})"))
    out += ["", "| Case | Type | Sample | Conf. | L x B x T (m) | Speed (kn) | Published (kW) | Model (kW) | err % | "
            "Band ± % | vs band |", "|---|---|---|---|---|---|---|---|---|---|---|"]
    for c in cases["installed_power_case"]:
        b = checks[c["id"]]
        out.append(f"| {c['id']} | {c['run_as_type']} | {'held out' if b.held_out else 'in-sample'} | "
                   f"{c['confidence']} | {c['loa_m']:g} x {c['beam_m']:g} x {c['draught_m']:g} | "
                   f"{c['design_speed_kn']:g} | {fmt(b.observed)} | {fmt(b.model)} | {fmt(b.err_pct, 1)} | "
                   f"±{b.band_pct:g} | {'inside' if b.inside else '**OUTSIDE** ⚠'} |")
    out += ["", "### Installed power records that cannot run yet", "", "| Case | Published | Why not run |",
            "|---|---|---|"]
    for c in cases.get("not_yet_runnable_installed_power_cases", []):
        out.append(f"| {c['id']} | {c['published']} | {c['not_yet_runnable']} |")
    return out


def _installed_power_warnings_for(checks: list[BandCheck]) -> list[str]:
    return [github_warning(
        f"{b.case}: modelled installed power is {b.err_pct:+.1f} % from the published {b.observed:g} kW "
        f"(model {b.model:,.0f} kW), outside ±{b.band_pct:g} % ({b.confidence} confidence, "
        f"{'held-out' if b.held_out else 'in-sample'})",
        title="Calibration band", file="calibration/propulsion_cases.yaml") for b in checks if not b.inside]


def _all_factors(dry_barge: list) -> dict:
    """Every fitted factor of factors.yaml: the barge factors, then the bollard pull factors (#351)."""
    factors = fit_factors(dry_barge)
    propulsion = _load_propulsion_cases()
    factors["bollard_pull_factor"] = _fit_bollard_pull_factor(propulsion["propulsion_case"])
    factors["propulsor_efficiency_factor"] = _fit_efficiency_factor(propulsion["installed_power_case"])
    return factors


def main_check() -> int:
    """Print one warning annotation per case outside its band. Exit 0 whatever the result: a band
    miss is a warning. An exception (a missing file, a bad case) is a crash and fails."""
    checks = assess()
    for line in warnings_for(checks):
        print(line)
    tank_checks = _assess_tanks()
    for line in _tank_warnings_for(tank_checks):
        print(line)
    tank_barge_checks = _assess_tank_barges()
    for line in _tank_barge_warnings_for(tank_barge_checks):
        print(line)
    propulsion_checks = _assess_propulsion()
    for line in _propulsion_warnings_for(propulsion_checks):
        print(line)
    power_checks = _assess_installed_power()
    for line in _installed_power_warnings_for(power_checks):
        print(line)
    every = checks + tank_checks + tank_barge_checks + propulsion_checks + power_checks
    outside = sum(1 for b in every if not b.inside)
    print(f"calibration: {len(every)} results against their bands, {outside} outside (warning only)")
    return 0


def _changelog_section(text: str, version: str) -> str:
    """The text of the `## <version>` section of CHANGELOG.md ("" when there is none)."""
    import re
    m = re.search(rf"(?m)^## {re.escape(version)}\b[^\n]*\n", text)
    if m is None:
        return ""
    nxt = re.search(r"(?m)^## ", text[m.end():])
    return text[m.end(): m.end() + nxt.start()] if nxt else text[m.end():]


def _release_problems(changelog_text: str, version: str) -> list[str]:
    """Calibration cases outside their band that block the release `version`: one line each (issue #431).

    A case outside its band blocks a release unless the CHANGELOG entry of that release explains it (the
    entry names the case id), or the case file already records why: `accepted_miss` (tank and propulsion cases) or
    `exclude_from_fit_cb` (dry barge cases left out of the Cb fit with a reason). A release with no CHANGELOG section for its version has every miss
    unexplained.
    """
    section = _changelog_section(changelog_text, version)
    accepted = dict(_accepted_misses())
    for c in _load_propulsion_cases()["propulsion_case"]:
        accepted.update({(c["id"], fig): why for fig, why in c.get("accepted_miss", {}).items()})
    every = assess() + _assess_tanks() + _assess_propulsion() + _assess_installed_power()
    out = []
    for b in every:
        if b.inside or (b.case, b.figure) in accepted or b.excluded:
            continue
        if b.case in section:
            continue
        out.append(f"{b.case} ({b.figure}): {b.err_pct:+.1f} % against a band of ±{b.band_pct:g} %")
    return out


def _main_release(version: str, changelog: Path | None = None) -> int:
    """Fail (exit 1) when a calibration case outside its band is not explained for the release `version`."""
    path = Path(changelog) if changelog else CAL.parent / "CHANGELOG.md"
    missing = [str(p) for p in (CAL, path) if not p.exists()]
    if missing:
        print(f"::error title=Calibration release check cannot run::not found: {', '.join(missing)}. Run it "
              f"from the repo checkout, or set OVD_CALIBRATION_DIR (and --changelog)")
        return 2
    bad = _release_problems(path.read_text(encoding="utf-8"), version)
    for line in bad:
        print(f"::error file=CHANGELOG.md,title=Calibration band blocks the release::{line}: name the case in "
              f"the CHANGELOG entry of {version}, or bring it inside its band")
    print(f"calibration: release {version}: {len(bad)} case(s) outside their band without an explanation")
    return 1 if bad else 0


def _band_cells(checks: list[BandCheck], case: str, figure: str) -> tuple[str, str]:
    for b in checks:
        if b.case == case and b.figure == figure:
            return (f"±{b.band_pct:g} ({b.confidence})",
                    "inside" if b.inside else "**OUTSIDE** ⚠")
    return "–", "–"


def fmt(x, nd=0):
    return "–" if x is None else f"{x:,.{nd}f}"


def factors_yaml(f: dict) -> str:
    return ("# GENERATED by python -m ovd.calibration from calibration/cases.yaml and\n"
            "# calibration/propulsion_cases.yaml.\n"
            "# Do not edit: add or correct cases, then re-run.\n"
            + yaml.safe_dump(f, sort_keys=False, width=110))


# --- Held-out cases (issue #3): section 4 of the report ----------------------------------------

def _hull_label(c: dict) -> str:
    return f"{c['hold_type']} / {c['run_as']}" if "run_as" in c else c["hold_type"]


def _status(c: dict, b: BandCheck | None) -> tuple[str, str]:
    """Band cell and verdict cell of a held-out row."""
    if c.get("cross_check_only"):
        return "–", "cross-check, no band"
    if b is None:
        return "–", "no band (not a DWT)"
    tag = " (excluded)" if c.get("exclude_from_fit_cb") or c.get("exclude_from_fit") else ""
    return (f"±{b.band_pct:g} ({b.confidence})",
            ("inside" if b.inside else "**OUTSIDE** ⚠") + tag)


def _held_out_rows(rows: list[BargeRow], checks: list[BandCheck]) -> list[str]:
    """One table row per case and published figure. Cargo capacity (not a DWT) is shown against the
    predicted DWT but has no band. A case with no such figure gets one row with what it publishes."""
    out = []
    for r in rows:
        c, o = r.case, r.case["observed"]
        figs = []
        if "lightship_t" in o:
            figs.append(("lightship", "lightship_t", o["lightship_t"], r.ls_cal))
        if "dwt_t" in o:
            figs.append(("DWT", "dwt_t", o["dwt_t"], r.dwt_cal))
        if "cargo_capacity_t" in o and r.dwt_cal is not None:
            figs.append(("cargo capacity (not DWT)", None, o["cargo_capacity_t"], r.dwt_cal))
        head = f"| {c['id']} | {_hull_label(c)} | {c['confidence']} |"
        if not figs:
            what = (f"light draught {o['light_draught_m']:g} m only" if "light_draught_m" in o
                    else "no lightship or DWT")
            out.append(f"{head} {what} | – | – | – | – | {_status(c, None)[1].replace('no band (not a DWT)', 'nothing to compare')} |")
        for label, fig, obs, model in figs:
            b = next((x for x in checks if x.case == c["id"] and x.figure == fig), None) if fig else None
            band, verdict = _status(c, b)
            out.append(f"{head} {label} | {fmt(obs)} | {fmt(model)} | {fmt(pct(model, obs), 1)} | "
                       f"{band} | {verdict} |")
    return out


def _figures_text(c: dict) -> str:
    o, parts = c["observed"], []
    for key, label, unit in (("lightship_t", "lightship", "t"), ("dwt_t", "DWT", "t"),
                             ("cargo_capacity_t", "cargo capacity", "t"),
                             ("tank_capacity_m3", "tanks", "m³"), ("lng_capacity_m3", "LNG", "m³"),
                             ("light_draught_m", "light draught", "m")):
        if key in o:
            parts.append(f"{label} {o[key]:,g} {unit}")
    return "; ".join(parts)


def _not_yet_runnable_rows(cases: list[dict]) -> list[str]:
    out = []
    for c in cases:
        dims = "×".join(f"{c[k]:g}" for k in ("loa_m", "beam_m", "depth_m") if k in c)
        t = f", T {c['draught_m']:g}" if c.get("draught_m") else ""
        why = c["not_yet_runnable"]
        if c.get("cross_check_only"):
            why += f". Cross-check only: {c['cross_check_only']}"
        out.append(f"| {c['id']} | {c['hold_type']} | {c['confidence']} | {dims}{t} | "
                   f"{_figures_text(c)} | {why} |")
    return out


def _summary_stats(checks: list[BandCheck]) -> dict | None:
    """n, mean error, mean absolute error, range and count inside the band, or None when empty."""
    if not checks:
        return None
    e = [b.err_pct for b in checks]
    return {"n": len(e), "mean": sum(e) / len(e), "mean_abs": sum(abs(x) for x in e) / len(e),
            "min": min(e), "max": max(e), "inside": sum(1 for b in checks if b.inside)}


def _summary_rows(checks: list[BandCheck]) -> list[str]:
    """For each figure: the in-sample results, then each held-out hull group. Cases left out of the
    Cb fit for an impossible figure are not counted, in either sample."""
    counted = [b for b in checks if not b.excluded]
    out = []
    for fig in ("lightship_t", "dwt_t"):
        label = BANDS["bands"][fig]["label"]
        groups = [("in-sample", "all in-sample cases", [b for b in counted
                                                         if b.figure == fig and not b.held_out])]
        for g in sorted({b.group for b in counted if b.held_out and b.figure == fig}):
            groups.append(("out of sample", g, [b for b in counted
                                                if b.figure == fig and b.held_out and b.group == g]))
        for sample, g, bs in groups:
            st = _summary_stats(bs)
            if st:
                out.append(f"| {label} | {sample} | {g} | {st['n']} | {st['mean']:+.1f} | "
                           f"{st['mean_abs']:.1f} | {st['min']:+.1f} to {st['max']:+.1f} | "
                           f"{st['inside']} of {st['n']} |")
    return out


def _held_out_section(held_cases: list[dict], held_rows: list[BargeRow], not_run: list[dict],
                     all_checks: list[BandCheck]) -> list[str]:
    held_checks = [b for b in all_checks if b.held_out]
    out = ["", "## 4. Held-out cases (out of sample)", "",
           "These cases were never used to fit k or Cb (issue #3). The calibrated model meets each one "
           "for the first time here, so these errors are out of sample. A deck barge has hold type "
           "`deck` and is run as a box barge (`run_as: box`): box k and box Cb, with no deck factor. "
           "The test is whether the box model carries over to deck barges. Predicted DWT uses the "
           "water density on the case: 1.025 for sea-going or saltwater figures, 1.000 otherwise. "
           "Cargo capacity is not DWT, so it is shown against the predicted DWT but has no band. "
           "Cases marked *excluded* are run and shown; their published figures cannot all describe "
           "one condition. Cases marked *cross-check* have no band.", "",
           "| Case | Hull (hold type / run as) | Conf. | Figure | Observed (t) | Calibrated (t) | err % | "
           "Band ± % | Calibrated vs band |",
           "|---|---|---|---|---|---|---|---|---|"]
    out += _held_out_rows(held_rows, held_checks)
    out += ["", "### Held-out cases the dry-barge path cannot run yet", "",
            "Listed with their published figures and the reason. They are never run.", "",
            "| Case | Hull | Conf. | LOA×B×D (m), T | Published figures | Why not run |",
            "|---|---|---|---|---|---|"]
    out += _not_yet_runnable_rows(not_run)
    out += ["", "### Summary: out of sample against in-sample", "",
            "Error is (calibrated − published) ÷ published. Cross-check cases and cases excluded for an "
            "impossible figure are left out of every row. Band misses are warnings only (#119): they "
            "never fail the build.", "",
            "| Figure | Sample | Group | n | Mean err % | Mean abs err % | Range % | Inside band |",
            "|---|---|---|---|---|---|---|---|"]
    out += _summary_rows(all_checks)
    return out



def report() -> str:
    """The results text. Pure: it reads the calibration inputs and writes nothing."""
    cases = load_cases()
    f = fit_factors(cases["dry_barge"])
    in_cases = [c for c in cases["dry_barge"] if not _is_held_out(c)]
    held_cases = [c for c in cases["dry_barge"] if _is_held_out(c)]
    rows = [run_barge(c, f) for c in in_cases]
    held_rows = [run_barge(c, f) for c in held_cases]
    all_checks = assess(cases)
    checks = [b for b in all_checks if not b.held_out]
    held_checks = [b for b in all_checks if b.held_out]
    kb, cbf = f["lightship_k_t_per_m3"], f["block_coefficient_box_barge_on_loa"]
    out = ["<!-- GENERATED by python -m ovd.calibration — do not edit -->",
           "# Calibration results", "",
           "*Ported* = core as ported from the prior engine (sloped-hopper build-up, Cb 0.85). "
           "*Calibrated* = hold-type k × L·B·D and the fitted Cb, from `factors.yaml`.", "",
           "## Fitted factors", ""]
    for ht, v in kb.items():
        out.append(f"- **k, {ht} hold:** {v['value']} t/m³ (range {v['min']}–{v['max']}, n = {v['n']})")
    out += [f"- **Cb, box barge on LOA:** {cbf['value']} (range {cbf['min']}–{cbf['max']}, n = {cbf['n']}; "
            f"{len(cbf['excluded'])} excluded — see factors.yaml)", "",
            "## Calibration Bands", "",
            "Each observed figure has a band, set per figure and per confidence of the case "
            "(`bands.yaml`): lightship ±15 / 20 / 30 % and deadweight ±7.5 / 10 / 15 % at high / "
            "medium / low confidence. The band applies to the *Calibrated* result. "
            "**The errors in sections 1 and 2 are in-sample:** the fitted factors (k per hold type, "
            "box-barge Cb) were fitted to the same cases, and the container factor rests on one case, "
            "so a result inside its band is not a validation. **The errors in section 4 are out of "
            "sample:** those cases were never used in a fit. A case outside its band is marked "
            "**OUTSIDE** and raises a CI warning annotation: **warning only, it never fails the "
            "build.** A lightship given only as an upper bound has no band. Section 3 compares the "
            "core with a design note, not a published figure, and has no band.", "",
            f"{sum(1 for b in checks if b.inside)} of {len(checks)} in-sample calibrated results are "
            f"inside their band.",
            f"{sum(1 for b in held_checks if b.inside and not b.excluded)} of "
            f"{sum(1 for b in held_checks if not b.excluded)} held-out calibrated results are inside "
            f"their band (section 4; cases excluded for an impossible figure are not counted).", "",
            "## 1. Push barges — lightship", "",
            "| Case | Hold | Conf. | Observed (t) | Ported | err % | Calibrated | err % | Padovezi 0.12 | err % | CN draft | err % | Band ± % | Calibrated vs band |",
            "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|"]
    for r in rows:
        c, o = r.case, r.case["observed"]
        obs = o.get("lightship_t")
        obs_txt = fmt(obs) if obs else (f"≤{fmt(r.ls_from_light_draught_cb1)} (light draught, Cb=1)"
                                        if r.ls_from_light_draught_cb1 else "–")
        out.append(f"| {c['id']} | {c['hold_type']} | {c['confidence']} | {obs_txt} | "
                   f"{fmt(r.ls_ported)} | {fmt(pct(r.ls_ported, obs), 1)} | {fmt(r.ls_cal)} | "
                   f"{fmt(pct(r.ls_cal, obs), 1)} | {fmt(r.ls_padovezi)} | {fmt(pct(r.ls_padovezi, obs), 1)} | "
                   f"{fmt(r.ls_cn)} | {fmt(pct(r.ls_cn, obs), 1)} | "
                   f"{' | '.join(_band_cells(checks, c['id'], 'lightship_t'))} |")
    out += ["", "Note: where a case is in the k fit, its calibrated error is in-sample.", "",
            "## 2. Push barges — deadweight at the stated draught", "",
            "| Case | T (m) | Observed DWT (t) | Ported | err % | Calibrated | err % | In Cb fit | Band ± % | Calibrated vs band |",
            "|---|---|---|---|---|---|---|---|---|---|"]
    fit_ids = {e["case"] for e in cbf["evidence"]}
    for r in rows:
        c, o = r.case, r.case["observed"]
        if not c.get("draught_m"):
            continue
        infit = "yes" if c["id"] in fit_ids else ("excluded" if c.get("exclude_from_fit_cb") or c.get("exclude_from_fit") else "no")
        out.append(f"| {c['id']} | {c['draught_m']} | {fmt(o.get('dwt_t'))} | {fmt(r.dwt_ported)} | "
                   f"{fmt(pct(r.dwt_ported, o.get('dwt_t')), 1)} | {fmt(r.dwt_cal)} | "
                   f"{fmt(pct(r.dwt_cal, o.get('dwt_t')), 1)} | {infit} | "
                   f"{' | '.join(_band_cells(checks, c['id'], 'dwt_t'))} |")

    src = yaml.safe_load((CAL / "cn_yangtze_gb38030_draft_2018.yaml").read_text(encoding="utf-8"))
    mrows = []
    for cargo in ("bulk", "liquid"):
        dp = src["design_points"][cargo]
        mrows += [run_motor(cargo, r, dp["speed_kmh"]) for r in dp["rows"]]
    out += ["", "## 3. Yangtze motor vessels — GB 38030 draft design points (2018)", "",
            "Design-study points, not built ships. Cb implied on LOA (a lower bound). "
            "Delivered power = core `resistance`, deep water, Lwl = 0.97·LOA (the app uses Lwl = LOA). "
            "Ratio = installed main power ÷ delivered power.", "",
            "| Cargo | LOA×B×T | Δ (t) | Cargo (t) | Δ − cargo (t) | Implied Cb | Speed (km/h) | "
            "Installed (kW) | Delivered (kW) | Ratio |",
            "|---|---|---|---|---|---|---|---|---|---|"]
    for m in mrows:
        ratio = m.power_kw / m.delivered_kw if m.delivered_kw else None
        out.append(f"| {m.cargo} | {m.loa:g}×{m.boa:g}×{m.T:g} | {fmt(m.disp)} | {fmt(m.cargo_t)} | "
                   f"{fmt(m.ls_plus_consumables)} | {m.cb_implied:.3f} | {m.speed_kmh:g} | "
                   f"{fmt(m.power_kw)} | {fmt(m.delivered_kw)} | {fmt(ratio, 2)} |")
    out += _held_out_section(held_cases, held_rows, cases.get("not_yet_runnable_cases", []), all_checks)
    out += _tank_section()
    out += _tank_barge_section()
    out += _propulsion_section()
    out += _installed_power_section()
    return "\n".join(out) + "\n"


def main(out_dir: Path = CAL) -> None:
    """Write results.md and factors.yaml into ``out_dir`` (the repo's calibration/ by default)."""
    out_dir = Path(out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    cases = load_cases()
    (out_dir / "factors.yaml").write_text(factors_yaml(_all_factors(cases["dry_barge"])), encoding="utf-8")
    (out_dir / "results.md").write_text(report(), encoding="utf-8")
    print((out_dir / "results.md").read_text(encoding="utf-8"))


if __name__ == "__main__":
    if sys.argv[1:] == ["--check"]:
        sys.exit(main_check())
    if len(sys.argv) in (3, 5) and sys.argv[1] == "--release-check":
        sys.exit(_main_release(sys.argv[2], Path(sys.argv[4]) if len(sys.argv) == 5 and sys.argv[3] == "--changelog" else None))
    if sys.argv[1:]:
        sys.exit("usage: python -m ovd.calibration [--check | --release-check VERSION [--changelog PATH]]")
    main()
