#!/usr/bin/env python3
"""Calibrate the pushed Formation allowance against published pushers (issue #359, part 2).

Each case of `calibration/formation_cases.yaml` is built as a pushed-convoy Design, run with
`ovcd run --json`, and its modelled pusher power (`formation.tug.installed_power_kw`) is compared
with the published installed power. The in-sample cases fit `formation_resistance_allowance`, the
ratio of the Formation's resistance to that of its Equivalent Hull, so that the modelled powers sit
on the published ones (least squares on the log of the ratio). A held-out case is run and banded,
never fitted. Bands: `calibration/bands.yaml`, `installed_power_kw`. A band miss is a warning, never
a failure.

    python3 tools/formation_calibration.py <path to ovcd>            print the report
    python3 tools/formation_calibration.py <path to ovcd> --write    write calibration/formation_results.md
                                                                     and calibration/formation_factors.yaml
    python3 tools/formation_calibration.py <path to ovcd> --check    fail if the written files are stale

The fit is an Estimate: the sources print no convoy speed for most pushers, so every case says which
figures are published and which are assumed.
"""
import copy
import json
import math
import subprocess
import sys
import tempfile
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
CAL = ROOT / "calibration"
BASE = ROOT / "crates/ovcd-format/tests/fixtures/pushed_convoy.vdesign"
BASE_KEYS = ("tug_included", "tug_design_speed_kn")


def typed(inputs: dict, key: str, value) -> None:
    inputs[key] = {"value": value, "source": "typed"}


def run_case(ovcd: Path, base: dict, c: dict, draught: float, allowance: float) -> dict:
    """The `formation` and `convoy` blocks of one case at a draught and an allowance."""
    d = copy.deepcopy(base)
    i = d["options"][0]["inputs"]
    b = c["barge"]
    typed(i, "formation", {"kind": "pushed_convoy", "abreast": b["abreast"], "in_line": b["in_line"]})
    typed(i, "length_overall_m", b["length_m"])
    typed(i, "beam_m", b["beam_m"])
    typed(i, "moulded_depth_m", b["depth_m"])
    typed(i, "tug_included", True)
    typed(i, "tug_design_speed_kn", c["speed_kn"])
    typed(i, "draught_limit_m", draught)
    typed(i, "formation_resistance_allowance", allowance)
    with tempfile.TemporaryDirectory() as tmp:
        f = Path(tmp) / "case.vdesign"
        f.write_text(json.dumps(d))
        r = subprocess.run([str(ovcd), "run", "--json", str(f)], capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(f"ovcd failed on {c['id']}: {r.stderr[-400:]}")
    res = json.loads(r.stdout)["options"][0]["results"]
    return {"formation": res["formation"], "convoy": res["convoy"]}


def draught_of(ovcd: Path, base: dict, c: dict) -> float:
    """The loaded draught: given, or the one at which one barge carries `dwt_t_per_barge`."""
    if "draught_m" in c:
        return float(c["draught_m"])
    lo, hi = 0.5, c["barge"]["depth_m"] - 0.3
    for _ in range(40):
        mid = (lo + hi) / 2
        dwt = run_case(ovcd, base, c, mid, 1.0)["convoy"]["barge"]["deadweight_t"]
        lo, hi = (mid, hi) if dwt < c["dwt_t_per_barge"] else (lo, mid)
        if hi - lo < 0.002:
            break
    return round((lo + hi) / 2, 3)


def model_power(ovcd, base, c, draught, allowance) -> float:
    return run_case(ovcd, base, c, draught, allowance)["formation"]["tug"]["installed_power_kw"]


def fit_allowance(ovcd, base, cases, draughts) -> float:
    """The allowance that minimises the squared log error of the in-sample powers (golden section)."""
    def cost(log_a):
        a = math.exp(log_a)
        return sum(math.log(model_power(ovcd, base, c, draughts[c["id"]], a)
                            / c["observed"]["installed_power_kw"]) ** 2 for c in cases)
    lo, hi, g = math.log(0.5), math.log(4.0), (math.sqrt(5) - 1) / 2
    x1, x2 = hi - g * (hi - lo), lo + g * (hi - lo)
    f1, f2 = cost(x1), cost(x2)
    for _ in range(30):
        if f1 < f2:
            hi, x2, f2 = x2, x1, f1
            x1 = hi - g * (hi - lo)
            f1 = cost(x1)
        else:
            lo, x1, f1 = x1, x2, f2
            x2 = lo + g * (hi - lo)
            f2 = cost(x2)
    return round(math.exp((lo + hi) / 2), 3)


def build(ovcd: Path) -> tuple[str, str]:
    data = yaml.safe_load((CAL / "formation_cases.yaml").read_text(encoding="utf-8"))
    band = yaml.safe_load((CAL / "bands.yaml").read_text(encoding="utf-8"))["bands"]["installed_power_kw"]
    base = json.loads(BASE.read_text(encoding="utf-8"))
    cases = data["formation_case"]
    draughts = {c["id"]: draught_of(ovcd, base, c) for c in cases}
    fit = [c for c in cases if not c.get("held_out")]
    allowance = fit_allowance(ovcd, base, fit, draughts)
    rows = []
    for c in cases:
        r = run_case(ovcd, base, c, draughts[c["id"]], allowance)
        r0 = run_case(ovcd, base, c, draughts[c["id"]], 1.0)
        obs = c["observed"]["installed_power_kw"]
        model = r["formation"]["tug"]["installed_power_kw"]
        err = 100.0 * (model / obs - 1.0)
        lim = float(band["pct"][c["confidence"]])
        rows.append({"c": c, "draught": draughts[c["id"]], "obs": obs, "model": model,
                     "model1": r0["formation"]["tug"]["installed_power_kw"], "err": err, "band": lim,
                     "inside": abs(err) <= lim, "converged": r["formation"]["sizing"]["converged"],
                     "pusher_length": r["formation"]["tug"]["length_m"]})
    md = ["# Formation calibration (issue #359, part 2)", "",
          "GENERATED by `python3 tools/formation_calibration.py <ovcd> --write` from "
          "`calibration/formation_cases.yaml`. Do not edit.", "",
          "A published pusher with a published convoy is run as a pushed convoy, and the installed power of the "
          "pusher the Formation loop sizes is compared with the published installed power (band "
          "`installed_power_kw`). The in-sample cases fit the Formation resistance allowance; **their errors are "
          "in-sample**. A held-out case is never fitted. A miss is a warning only.", "",
          f"**Fitted `formation_resistance_allowance`: {allowance}** (an Estimate; n = {len(fit)}). "
          "The sources print no convoy speed for most pushers, so every case lists its assumed figures below.", "",
          "| Case | Sample | Conf. | Barge draught (m) | Speed (kn) | Published (kW) | Model at 1.00 (kW) | "
          f"Model at {allowance} (kW) | err % | Band ± % | vs band |", "|---|---|---|---|---|---|---|---|---|---|---|"]
    for r in rows:
        c = r["c"]
        md.append(f"| {c['id']} | {'held out' if c.get('held_out') else 'in-sample'} | {c['confidence']} | "
                  f"{r['draught']:g} | {c['speed_kn']:g} | {r['obs']:,.0f} | {r['model1']:,.0f} | {r['model']:,.0f} | "
                  f"{r['err']:+.1f} | ±{r['band']:g} | {'inside' if r['inside'] else '**OUTSIDE**'} |")
    md += ["", "## How each case is run", ""]
    for r in rows:
        c = r["c"]
        md += [f"- **{c['id']}**: {c['barge_basis']}. " + (c["dwt_basis"] + f"; the draught that gives it is {r['draught']:g} m. "
               if "dwt_basis" in c else c["draught_basis"] + ". ") + c["speed_basis"] + ".",
               f"  The sized pusher is {r['pusher_length']:.0f} m long at the fitted allowance"
               + ("" if r["converged"] else "; **the sizing loop did not converge**") + "."]
    md += ["", "## Records that cannot run yet", "", "| Case | Published | Why not run |", "|---|---|---|"]
    for c in data.get("not_yet_runnable_formation_cases", []):
        md.append(f"| {c['id']} | {c['published']} | {c['not_yet_runnable']} |")
    md.append("")
    ya = {"formation_resistance_allowance": {
        "pushed_convoy": {"value": allowance, "n": len(fit),
                          "evidence": [{"case": c["id"]} for c in fit],
                          "basis": "fitted to the in-sample pushers of formation_cases.yaml; an Estimate (the sources print no convoy speed)"}}}
    head = ("# GENERATED by python3 tools/formation_calibration.py from calibration/formation_cases.yaml.\n"
            "# Do not edit: add or correct cases, then re-run.\n")
    return "\n".join(md), head + yaml.safe_dump(ya, sort_keys=False)


def main(argv: list[str]) -> int:
    args = [a for a in argv if not a.startswith("--")]
    if len(args) != 1:
        print(__doc__)
        return 2
    report, factors = build(Path(args[0]))
    out = {"formation_results.md": report, "formation_factors.yaml": factors}
    if "--write" in argv:
        for name, text in out.items():
            (CAL / name).write_text(text, encoding="utf-8")
        print("formation_calibration: wrote", ", ".join(out))
    elif "--check" in argv:
        stale = [n for n, t in out.items() if (CAL / n).read_text(encoding="utf-8") != t]
        if stale:
            print("formation_calibration: stale:", ", ".join(stale))
            return 1
        print("formation_calibration: up to date")
    else:
        print(report)
    for line in report.splitlines():
        if "**OUTSIDE**" in line:
            print(f"::warning title=Calibration band::{line.split('|')[1].strip()}: pusher power outside its band")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
