#!/usr/bin/env python3
"""Run the evaluation corpus through the engine and report (issue #184, phase 0).

    python3 evaluation/run.py            # run, and write evaluation/results.md
    python3 evaluation/run.py --check    # CI: run; fail on a false pass or a stale results.md;
                                         # one warning annotation per false fail and per coverage gap

Every case (`evaluation/cases/*.yaml`) and every generated boundary mutant (`evaluation/mutants.yaml`)
is run through the engine's `box_condition`, and the engine's verdict on each of the six criteria is
compared with the expected one.

  * false fail: the engine fails a criterion the case expects to pass (a known-good criterion).
  * false pass: the engine passes a criterion the case expects to fail (a known-bad criterion).
  * coverage: a criterion is covered when at least one case expects it to pass and one expects it to
    fail, both outside the engine's grid error.

A criterion marked `grid: true` (the expected value is within the engine's grid error of its limit)
is run and listed, but never counts as a false fail or a false pass.

Exit status. A false pass fails the run (1). A false fail and a coverage gap are warnings, as the
Calibration Bands are (issue #119). With --check a results.md that is not what this run would write
also fails it. The engine is not tuned here: a disagreement is reported, and what to do with it is a
separate, deliberate change.
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path

import yaml

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from engine import box_condition  # noqa: E402

CASES = HERE / "cases"
MUTANTS = HERE / "mutants.yaml"
RESULTS = HERE / "results.md"

#: criterion key -> the name the engine gives it
ENGINE_NAMES = {
    "area_0_30": "Area 0–30°",
    "area_0_40": "Area 0–40°",
    "area_30_40": "Area 30–40°",
    "gz_30_plus": "GZ at ≥ 30°",
    "angle_max_gz": "Angle of max GZ",
    "gm": "Initial GM (fluid)",
}
LABELS = {
    "area_0_30": "Area 0-30 deg (2.2.1)",
    "area_0_40": "Area 0-40 deg (2.2.1)",
    "area_30_40": "Area 30-40 deg (2.2.1)",
    "gz_30_plus": "GZ at 30 deg or more (2.2.2)",
    "angle_max_gz": "Angle of max GZ (2.2.3)",
    "gm": "Initial GM (2.2.4)",
}


def load_cases(cases_dir: Path = CASES, mutants_file: Path | None = MUTANTS) -> list[dict]:
    cases = [yaml.safe_load(p.read_text(encoding="utf-8")) for p in sorted(cases_dir.glob("*.yaml"))]
    if mutants_file is not None and mutants_file.exists():
        cases += yaml.safe_load(mutants_file.read_text(encoding="utf-8"))["mutants"]
    return cases


def run_case(case: dict) -> dict:
    """Engine verdict per criterion for one case, next to the expected one."""
    i = case["inputs"]
    c = box_condition(i["beam_m"], i["depth_m"], i["draught_m"], i["kg_m"],
                      free_surface_m=i.get("free_surface_m", 0.0))
    by_name = {x.name: x for x in c.criteria}
    rows = {}
    for key, engine_name in ENGINE_NAMES.items():
        crit = by_name[engine_name]
        exp = case["expected"][key]
        rows[key] = {
            "expected": exp["verdict"], "engine": "pass" if crit.ok else "fail",
            "grid": bool(exp.get("grid", False)),
            "expected_value": exp["value"], "engine_value": crit.value, "limit": exp["limit"],
        }
    return rows


def evaluate(cases: list[dict]) -> dict:
    per = {k: {"pass_cases": 0, "fail_cases": 0, "false_fails": [], "false_passes": [],
               "grid_disagreements": [], "compared": 0} for k in ENGINE_NAMES}
    for case in cases:
        for key, r in run_case(case).items():
            p = per[key]
            if r["grid"]:
                if r["expected"] != r["engine"]:
                    p["grid_disagreements"].append(case["id"])
                continue
            p["compared"] += 1
            p["pass_cases" if r["expected"] == "pass" else "fail_cases"] += 1
            if r["expected"] == "pass" and r["engine"] == "fail":
                p["false_fails"].append(case["id"])
            if r["expected"] == "fail" and r["engine"] == "pass":
                p["false_passes"].append(case["id"])
    return {"cases": cases, "per": per}


def totals(result: dict) -> dict:
    per = result["per"]
    good = sum(p["pass_cases"] for p in per.values())
    bad = sum(p["fail_cases"] for p in per.values())
    ff = sum(len(p["false_fails"]) for p in per.values())
    fp = sum(len(p["false_passes"]) for p in per.values())
    gaps = [k for k, p in per.items() if p["pass_cases"] == 0 or p["fail_cases"] == 0]
    return {"known_good": good, "known_bad": bad, "false_fails": ff, "false_passes": fp,
            "coverage_gaps": gaps}


def pct(n: int, d: int) -> str:
    return "n/a" if d == 0 else f"{100.0 * n / d:.1f} %"


def report(result: dict) -> str:
    cases, per, t = result["cases"], result["per"], totals(result)
    base = [c for c in cases if "mutant" not in c]
    mutants = [c for c in cases if "mutant" in c]
    external = [c for c in base if c["provenance"].get("inputs_basis") == "published"]
    skipped = []
    if MUTANTS.exists():
        skipped = yaml.safe_load(MUTANTS.read_text(encoding="utf-8")).get("skipped", [])
    out = [
        "# Evaluation results",
        "",
        "Generated by `python3 evaluation/run.py`. Do not edit by hand. Phase 0 of the evaluation harness "
        "([#184](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/184)): "
        "the IMO Intact Stability Code 2008, Part A 2.2, for box sections with a known KG. "
        "The format and the sources are in [README.md](README.md).",
        "",
        "## Summary",
        "",
        f"- Cases: {len(base)} base cases and {len(mutants)} generated boundary mutants "
        f"({len(skipped)} possible mutants skipped because no KG on the hull crosses the limit).",
        f"- Base cases with an external published source for the inputs: **{len(external)}** "
        f"({', '.join('`' + c['id'] + '`' for c in external) or 'none'}; their verdicts are still derived by the "
        f"oracle, the sources give no IS Code 2.2 verdict). Derived entirely by the oracle: "
        f"{len(base) - len(external)}.",
        f"- Criterion verdicts compared (outside the grid error): {t['known_good'] + t['known_bad']} "
        f"({t['known_good']} expected to pass, {t['known_bad']} expected to fail).",
        f"- **False fails** (engine fails a known-good criterion): {t['false_fails']} "
        f"({pct(t['false_fails'], t['known_good'])}). A warning.",
        f"- **False passes** (engine passes a known-bad criterion): {t['false_passes']} "
        f"({pct(t['false_passes'], t['known_bad'])}). Fails the build.",
        f"- Criteria with no passing or no failing case: "
        f"{', '.join(LABELS[k] for k in t['coverage_gaps']) or 'none'}. A warning.",
        "",
        "## Per criterion",
        "",
        "| Criterion | Expected pass | Expected fail | False fails | False passes | Coverage | Inside grid error, engine differs |",
        "|---|---:|---:|---:|---:|---|---|",
    ]
    for key, p in per.items():
        cov = "yes" if p["pass_cases"] and p["fail_cases"] else "**gap**"
        grid = ", ".join(p["grid_disagreements"]) or "none"
        out.append(f"| {LABELS[key]} | {p['pass_cases']} | {p['fail_cases']} | {len(p['false_fails'])} | "
                   f"{len(p['false_passes'])} | {cov} | {grid} |")
    fails = [(k, i) for k, p in per.items() for i in p["false_fails"]]
    passes = [(k, i) for k, p in per.items() for i in p["false_passes"]]
    out += ["", "## Disagreements", ""]
    if not fails and not passes:
        out.append("None.")
    for k, i in fails:
        out.append(f"- false fail: `{i}`, {LABELS[k]}")
    for k, i in passes:
        out.append(f"- false pass: `{i}`, {LABELS[k]}")
    out += ["", "## Base cases", "", "| Case | Inputs basis | Verdict basis | Source |", "|---|---|---|---|"]
    for c in base:
        pv = c["provenance"]
        src = f"{pv['edition']}, {pv['locator']}"
        if "inputs_source" in pv:
            src = f"inputs: {pv['inputs_source']['locator']} of the paper cited in README.md; limits: {src}"
        out.append(f"| `{c['id']}` | {'published' if pv.get('inputs_basis') == 'published' else 'chosen'} | "
                   f"{pv['verdict_basis']} | {src} |")
    if skipped:
        out += ["", "## Mutants not generated", "",
                "A limit that no physical input crosses has no mutant (the KG would have to be above the deck).", ""]
        for s in skipped:
            out.append(f"- `{s['parent']}`, {LABELS[s['criterion']]}, {s['side']}, {s['magnitude']} step: {s['reason']}")
    return "\n".join(out) + "\n"


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--cases", type=Path, default=CASES)
    ap.add_argument("--mutants", type=Path, default=MUTANTS)
    ap.add_argument("--results", type=Path, default=RESULTS)
    args = ap.parse_args(argv[1:])
    result = evaluate(load_cases(args.cases, args.mutants))
    t = totals(result)
    text = report(result)
    status = 0
    for k, p in result["per"].items():
        for i in p["false_passes"]:
            print(f"::error title=Evaluation false pass::{i}: the engine passes {LABELS[k]}, which the case expects to fail")
            status = 1
        for i in p["false_fails"]:
            print(f"::warning title=Evaluation false fail::{i}: the engine fails {LABELS[k]}, which the case expects to pass")
    for k in t["coverage_gaps"]:
        print(f"::warning title=Evaluation coverage gap::{LABELS[k]} has no passing or no failing case")
    if args.check:
        if not args.results.exists() or args.results.read_text(encoding="utf-8") != text:
            print(f"::error title=Evaluation results stale::{args.results} is not what this run writes; "
                  "run python3 evaluation/run.py and commit it")
            status = 1
    else:
        args.results.write_text(text, encoding="utf-8")
        print(f"wrote {args.results}")
    print(f"evaluation: {t['known_good']} known-good and {t['known_bad']} known-bad criterion verdicts; "
          f"{t['false_fails']} false fails, {t['false_passes']} false passes")
    return status


if __name__ == "__main__":
    sys.exit(main(sys.argv))
