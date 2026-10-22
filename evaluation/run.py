#!/usr/bin/env python3
"""Run the evaluation corpus through the engine and report (issues #184 and #278).

    python3 evaluation/run.py            # run, and write evaluation/results.md
    python3 evaluation/run.py --check    # CI: run; fail on a false pass or a stale results.md;
                                         # one warning annotation per false fail and per coverage gap

Every case (`evaluation/cases/*.yaml`) and every generated boundary mutant (`evaluation/mutants.yaml`)
is run through the engine's Rule Trace (`box_trace`), and the trace row of each Rule the case expects a
verdict for is compared with that verdict, keyed by rule id. The first row whose outcome differs
names the Rule.

  * false fail: the engine fails a Rule the case expects to pass (a known-good verdict).
  * false pass: the engine passes a blocking Rule the case expects to fail (a known-bad verdict).
  * coverage: an evaluated blocking Rule is covered when at least one case expects it to pass and one
    expects it to fail, both outside the engine's grid error.

Each disagreement goes in one triage bucket (spec 0004, "How the evaluation harness uses it"):

  * model-gap: the Rule is `not evaluated` (or has no row), or the case needs an input the engine lacks;
  * known-limitation: the Rule links a finding;
  * code-bug: the independent oracle and the engine disagree;
  * rule-suspect: the oracle and the engine agree with each other, but not with a published verdict;
  * needs-expert: the rest.

A `not evaluated` row against an expected `fail` is a model gap, never a pass. A Rule marked
`grid: true` (the expected value is within the engine's grid error of its limit) is run and listed,
but never counts as a false fail, a false pass or a disagreement.

Exit status. A false pass on a blocking Rule fails the run (1). A false fail and a coverage gap are
warnings, as the Calibration Bands are (issue #119). With --check a results.md that is not what this
run would write also fails it. The engine is not tuned here: a disagreement is reported, and what to
do with it is a separate, deliberate change.
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path

import yaml

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import rule_mutation  # noqa: E402
from engine import DEFAULT_SELECT, box_trace, liquid_trace, stern_case_trace  # noqa: E402

CASES = HERE / "cases"
MUTANTS = HERE / "mutants.yaml"
TANK_MUTANTS = HERE / "tank_mutants.yaml"
STERN_MUTANTS = HERE / "stern_mutants.yaml"
RESULTS = HERE / "results.md"

#: The sets a liquid bulk (`kind: tank`) case selects (issue #338); `normam-202` is optional, never in a default
#: stack, so no base case selects it.
TANK_SELECT = ("adn-tank@1.0.0", "marpol-i@1.0.0")
#: The sets a stern (`kind: stern`) case selects (issue #355): the Blind Distance, speed, redundancy and propeller
#: Rules of a self-propelled craft.
STERN_SELECT = ("es-trin-2025@1.0.0", "solas-v22@1.0.0", "rheinschpv@1.0.0", "ovcd@2.1.0")
#: The case inputs of a stern case; a case that gives any other is a model gap.
STERN_INPUTS = {"length_overall_m", "beam_m", "depth_m", "bow_thruster", "propulsor_count",
                "blind_distance_light_m", "blind_distance_laden_m", "speed_deep_water_kmh", "design_speed_kn",
                "propeller_diameter_m", "propeller_d_max_m", "propeller_ducted", "aft_draught_light_m",
                "blade_load_kw_m2"}
BUCKETS = ("model-gap", "known-limitation", "code-bug", "rule-suspect", "needs-expert")
#: The trace inputs a box case gives; a case that needs any other input is a model gap.
BOX_INPUTS = {"form", "vdesign", "length_m", "beam_m", "depth_m", "draught_m", "kg_m",
              "water_density_t_m3", "free_surface_m", "downflooding_angle_deg"}
#: The trace inputs a liquid bulk case gives.
TANK_INPUTS = {"loa_m", "beam_m", "depth_m", "zone_aft_m", "zone_fwd_m", "double_side_m", "double_bottom_m",
               "tanks_in_line", "centreline_bulkhead", "cofferdam_m", "pump_room_m", "slop_share",
               "deadweight_t", "service_area", "adn_vessel_type", "rule_sets"}


def kind_of(case: dict) -> str:
    return case.get("kind", "box")


def load_cases(cases_dir: Path = CASES, mutants_file: Path | None = MUTANTS,
               tank_mutants_file: Path | None = TANK_MUTANTS,
               stern_mutants_file: Path | None = STERN_MUTANTS) -> list[dict]:
    cases = [yaml.safe_load(p.read_text(encoding="utf-8")) for p in sorted(cases_dir.glob("*.yaml"))]
    for f in (mutants_file, tank_mutants_file, stern_mutants_file):
        if f is not None and f.exists():
            cases += yaml.safe_load(f.read_text(encoding="utf-8"))["mutants"]
    return cases


def trace_of(case: dict, rule_sets: tuple[str, ...] = (), select: tuple[str, ...] = DEFAULT_SELECT) -> dict:
    i = case["inputs"]
    if kind_of(case) == "tank":
        return liquid_trace(i, rule_sets=rule_sets, select=select)
    if kind_of(case) == "stern":
        return stern_case_trace(i, rule_sets=rule_sets, select=select)
    return box_trace(i["beam_m"], i["depth_m"], i["draught_m"], i["kg_m"],
                     free_surface_m=i.get("free_surface_m", 0.0), rule_sets=rule_sets, select=select)


def engine_verdict(row: dict | None) -> str:
    """`pass`, `fail`, `not evaluated` or `not applicable`: a failed `warn` or `note` Rule counts as
    `fail`, and a Rule with no row as `not evaluated`."""
    if row is None:
        return "not evaluated"
    return {"warn": "fail", "note": "pass"}.get(row["outcome"], row["outcome"])


def triage(case: dict, expected: dict, row: dict | None) -> str:
    """The bucket of one disagreement between `expected` (a case's entry for one Rule) and the
    engine's `row` for it."""
    missing = sorted(set(case["inputs"]) - {"tank": TANK_INPUTS, "stern": STERN_INPUTS}.get(kind_of(case), BOX_INPUTS))
    if row is None or row["outcome"] in ("not evaluated", "not applicable") or missing:
        return "model-gap"
    if row.get("finding"):
        return "known-limitation"
    engine = engine_verdict(row)
    published = case["provenance"].get("verdict_basis") == "published"
    # For a derived case the expected verdict IS the oracle's; a published one may also carry it.
    oracle = expected.get("oracle", None if published else expected["verdict"])
    if oracle is not None and oracle != engine:
        return "code-bug"
    if published and oracle is not None and oracle == engine:
        return "rule-suspect"
    return "needs-expert"


def run_case(case: dict, rule_sets: tuple[str, ...] = (), select: tuple[str, ...] = DEFAULT_SELECT) -> dict:
    """The engine's row, next to the expected verdict, for each Rule the case expects one for."""
    trace = trace_of(case, rule_sets, select)
    # A Rule on a tank has a row per tank: the failing row stands for the Rule, else the first.
    by_rule: dict[str, dict] = {}
    for r in trace["rows"]:
        if not r["rule"]:
            continue
        held = by_rule.get(r["rule"])
        if held is None or (held["outcome"] not in ("fail", "warn") and r["outcome"] in ("fail", "warn")):
            by_rule[r["rule"]] = r
    out = {}
    for rule_id, exp in case["expected"].items():
        row = by_rule.get(rule_id)
        out[rule_id] = {
            "expected": exp["verdict"], "engine": engine_verdict(row),
            "grid": bool(exp.get("grid", False)),
            "expected_value": exp["value"], "engine_value": row["value"] if row else None,
            "limit": exp["limit"], "row": row,
            "blocking": bool(row) and row["consequence"] == "block",
        }
    return {"trace": trace, "rows": out}


def evaluate(cases: list[dict], rule_sets: tuple[str, ...] = (),
             select: tuple[str, ...] = DEFAULT_SELECT) -> dict:
    per: dict[str, dict] = {}
    rules: dict[str, dict] = {}
    disagreements = []
    for case in cases:
        # A liquid bulk case selects the liquid Rule Sets, a box case the one the caller names.
        kind = kind_of(case)
        ran = run_case(case, rule_sets, TANK_SELECT if kind == "tank" else STERN_SELECT if kind == "stern" else select)
        for row in ran["trace"]["rows"]:
            if row["rule"] and row["rule"] not in rules:
                rules[row["rule"]] = {"set": row["rule_sets"][0], "title": row["title"],
                                      "consequence": row["consequence"], "finding": row["finding"],
                                      "evaluated": row["outcome"] != "not evaluated"}
        for rule_id, r in ran["rows"].items():
            p = per.setdefault(rule_id, {"pass_cases": 0, "fail_cases": 0, "false_fails": [],
                                         "false_passes": [], "grid_disagreements": [], "compared": 0,
                                         "buckets": {b: 0 for b in BUCKETS}})
            if r["grid"]:
                if r["expected"] != r["engine"]:
                    p["grid_disagreements"].append(case["id"])
                continue
            p["compared"] += 1
            p["pass_cases" if r["expected"] == "pass" else "fail_cases"] += 1
            if r["expected"] == r["engine"]:
                continue
            bucket = triage(case, case["expected"][rule_id], r["row"])
            p["buckets"][bucket] += 1
            disagreements.append({"case": case["id"], "rule": rule_id, "expected": r["expected"],
                                  "engine": r["engine"], "bucket": bucket})
            if r["expected"] == "pass" and r["engine"] == "fail":
                p["false_fails"].append(case["id"])
            if r["expected"] == "fail" and r["engine"] == "pass" and r["blocking"]:
                p["false_passes"].append(case["id"])
    return {"cases": cases, "per": per, "rules": rules, "disagreements": disagreements,
            "select": list(select), "select_tank": list(TANK_SELECT), "select_stern": list(STERN_SELECT)}


def mutate_rules(result: dict) -> list[dict]:
    """Run every rule-data mutant of the selected sets over the corpus (issue #279)."""
    box = [c for c in result["cases"] if kind_of(c) == "box"]
    tank = [c for c in result["cases"] if kind_of(c) == "tank"]
    stern = [c for c in result["cases"] if kind_of(c) == "stern"]
    return (rule_mutation.run(box, result["select"], trace_of)
            + rule_mutation.run(tank, result["select_tank"], trace_of)
            + rule_mutation.run(stern, result["select_stern"], trace_of, expected_only=True, near_only=True))


def weak(mutation: list[dict]) -> list[dict]:
    """The rule-data mutants no case noticed: the weak-corpus findings."""
    return [m for m in mutation if not m["noticed_by"]]


def coverage_gaps(result: dict) -> list[str]:
    """The evaluated blocking Rules with no passing or no failing case."""
    gaps = []
    for rule_id, info in result["rules"].items():
        if info["consequence"] != "block" or not info["evaluated"]:
            continue
        p = result["per"].get(rule_id)
        if p is None or p["pass_cases"] == 0 or p["fail_cases"] == 0:
            gaps.append(rule_id)
    return gaps


def totals(result: dict) -> dict:
    per = result["per"]
    good = sum(p["pass_cases"] for p in per.values())
    bad = sum(p["fail_cases"] for p in per.values())
    ff = sum(len(p["false_fails"]) for p in per.values())
    fp = sum(len(p["false_passes"]) for p in per.values())
    buckets = {b: sum(1 for d in result["disagreements"] if d["bucket"] == b) for b in BUCKETS}
    return {"known_good": good, "known_bad": bad, "false_fails": ff, "false_passes": fp,
            "coverage_gaps": coverage_gaps(result), "buckets": buckets}


def pct(n: int, d: int) -> str:
    return "n/a" if d == 0 else f"{100.0 * n / d:.1f} %"


def label(result: dict, rule_id: str) -> str:
    info = result["rules"].get(rule_id)
    return f"`{rule_id}` ({info['title']})" if info else f"`{rule_id}`"


def report(result: dict) -> str:
    cases, per, t = result["cases"], result["per"], totals(result)
    base = [c for c in cases if "mutant" not in c]
    mutants = [c for c in cases if "mutant" in c]
    tank_base = [c for c in base if kind_of(c) == "tank"]
    tank_mutants = [c for c in mutants if kind_of(c) == "tank"]
    stern_base = [c for c in base if kind_of(c) == "stern"]
    stern_mutants = [c for c in mutants if kind_of(c) == "stern"]
    external = [c for c in base if c["provenance"].get("inputs_basis") == "published"]
    skipped = []
    if MUTANTS.exists():
        skipped = yaml.safe_load(MUTANTS.read_text(encoding="utf-8")).get("skipped", [])
    tank_skipped = []
    if TANK_MUTANTS.exists():
        tank_skipped = yaml.safe_load(TANK_MUTANTS.read_text(encoding="utf-8")).get("skipped", [])
    stern_skipped = []
    if STERN_MUTANTS.exists():
        stern_skipped = yaml.safe_load(STERN_MUTANTS.read_text(encoding="utf-8")).get("skipped", [])
    out = [
        "# Evaluation results",
        "",
        "Generated by `python3 evaluation/run.py`. Do not edit by hand. The evaluation harness "
        "([#184](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/184), keyed by "
        "rule id since [#278](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/278)): "
        "the IMO Intact Stability Code 2008, Part A 2.2, for box sections with a known KG, read from the "
        "engine's Rule Trace. The format and the sources are in [README.md](README.md).",
        "",
        "## Summary",
        "",
        f"- Rule Sets selected: {', '.join('`' + s + '`' for s in result['select'])} for the box cases, "
        f"{', '.join('`' + s + '`' for s in result['select_tank'])} for the liquid bulk cases, "
        f"{', '.join('`' + s + '`' for s in result['select_stern'])} for the stern cases.",
        f"- Cases: {len(base)} base cases ({len(tank_base)} liquid bulk, {len(stern_base)} stern) and {len(mutants)} "
        f"generated boundary mutants ({len(tank_mutants)} liquid bulk, {len(stern_mutants)} stern; {len(skipped)} "
        f"possible box mutants skipped because no KG on the hull crosses the limit, {len(tank_skipped)} liquid bulk "
        f"mutants skipped because no layout reaches the limit, {len(stern_skipped)} stern mutants skipped).",
        f"- Base cases with an external published source for the inputs: **{len(external)}** "
        f"({', '.join('`' + c['id'] + '`' for c in external) or 'none'}; their verdicts are still derived by the "
        f"oracle, the sources give no IS Code 2.2 verdict). Derived entirely by the oracle: "
        f"{len(base) - len(external)}.",
        f"- Rule verdicts compared (outside the grid error): {t['known_good'] + t['known_bad']} "
        f"({t['known_good']} expected to pass, {t['known_bad']} expected to fail).",
        f"- **False fails** (engine fails a known-good verdict): {t['false_fails']} "
        f"({pct(t['false_fails'], t['known_good'])}). A warning.",
        f"- **False passes** (engine passes a blocking Rule a case expects to fail): {t['false_passes']} "
        f"({pct(t['false_passes'], t['known_bad'])}). Fails the build.",
        f"- Evaluated blocking Rules with no passing or no failing case: "
        f"{', '.join('`' + k + '`' for k in t['coverage_gaps']) or 'none'}. A warning.",
        "",
        "## Triage",
        "",
        "Each disagreement outside the grid error, in one bucket.",
        "",
        "| Bucket | Disagreements |",
        "|---|---:|",
    ]
    for b in BUCKETS:
        out.append(f"| {b} | {t['buckets'][b]} |")
    out += ["", "## Coverage by rule id", ""]
    sets = sorted({info["set"] for info in result["rules"].values()})
    for s in sets:
        out += [f"### {s}", "",
                "| Rule | Consequence | Expected pass | Expected fail | False fails | False passes | Coverage "
                "| Inside grid error, engine differs | Buckets |",
                "|---|---|---:|---:|---:|---:|---|---|---|"]
        for rule_id, info in result["rules"].items():
            if info["set"] != s:
                continue
            p = per.get(rule_id)
            if not info["evaluated"]:
                cov = "not evaluated"
            elif info["consequence"] != "block":
                cov = "-"
            else:
                cov = "yes" if p and p["pass_cases"] and p["fail_cases"] else "**gap**"
            title = info["title"] + (f" (finding {info['finding']})" if info["finding"] else "")
            if p is None:
                out.append(f"| `{rule_id}` {title} | {info['consequence']} | 0 | 0 | 0 | 0 | {cov} | none | none |")
                continue
            grid = ", ".join(p["grid_disagreements"]) or "none"
            buckets = ", ".join(f"{b} {n}" for b, n in p["buckets"].items() if n) or "none"
            out.append(f"| `{rule_id}` {title} | {info['consequence']} | {p['pass_cases']} | {p['fail_cases']} "
                       f"| {len(p['false_fails'])} | {len(p['false_passes'])} | {cov} | {grid} | {buckets} |")
        out.append("")
    mutation = result.get("mutation", [])
    out += ["## Rule mutation", "",
            "Each numeric limit and coefficient of each evaluated Rule, moved one step to each side in a "
            "mutated copy of its set, loaded at run time "
            "([#279](https://github.com/African-River-Corridors/open-vessel-concept-drawings/issues/279)). "
            "A mutant no case notices (no case changes its verdict on that Rule) is a **weak corpus** "
            "finding: a warning. The steps are in [README.md](README.md).", "",
            f"- Mutants run: {len(mutation)}. Not noticed by any case: {len(weak(mutation))}.", "",
            "| Rule | Number | Value | Mutant | Cases that notice it |",
            "|---|---|---:|---|---:|"]
    for m in mutation:
        noticed = str(len(m["noticed_by"])) if m["noticed_by"] else "**none: weak corpus**"
        out.append(f"| `{m['rule']}` | {m['where']} | {m['value']} | {m['side']}: {m['moved']} | {noticed} |")
    out += ["", "## Disagreements", ""]
    if not result["disagreements"]:
        out.append("None.")
    for d in result["disagreements"]:
        out.append(f"- {d['bucket']}: `{d['case']}`, `{d['rule']}`: expected {d['expected']}, engine {d['engine']}")
    out += ["", "## Base cases", "", "| Case | Inputs basis | Verdict basis | Source |", "|---|---|---|---|"]
    for c in base:
        pv = c["provenance"]
        src = f"{pv['edition']}, {pv['locator']}"
        if "inputs_source" in pv:
            src = f"inputs: {pv['inputs_source']['locator']} of the paper cited in README.md; limits: {src}"
        out.append(f"| `{c['id']}` | {'published' if pv.get('inputs_basis') == 'published' else 'chosen'} | "
                   f"{pv['verdict_basis']} | {src} |")
    if skipped or tank_skipped or stern_skipped:
        out += ["", "## Mutants not generated", "",
                "A limit that no physical input crosses has no mutant (the KG would have to be above the deck; "
                "a tank limit that no cargo zone inside the hull reaches).", ""]
        for s in skipped + tank_skipped + stern_skipped:
            out.append(f"- `{s['parent']}`, `{s['criterion']}`, {s['side']}, {s['magnitude']} step: {s['reason']}")
    return "\n".join(out) + "\n"


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--cases", type=Path, default=CASES)
    ap.add_argument("--mutants", type=Path, default=MUTANTS)
    ap.add_argument("--tank-mutants", type=Path, default=TANK_MUTANTS)
    ap.add_argument("--stern-mutants", type=Path, default=STERN_MUTANTS)
    ap.add_argument("--results", type=Path, default=RESULTS)
    args = ap.parse_args(argv[1:])
    result = evaluate(load_cases(args.cases, args.mutants, args.tank_mutants, args.stern_mutants))
    result["mutation"] = mutate_rules(result)
    t = totals(result)
    text = report(result)
    status = 0
    for rule_id, p in result["per"].items():
        for i in p["false_passes"]:
            print(f"::error title=Evaluation false pass::{i}: the engine passes {label(result, rule_id)}, "
                  "which the case expects to fail")
            status = 1
        for i in p["false_fails"]:
            print(f"::warning title=Evaluation false fail::{i}: the engine fails {label(result, rule_id)}, "
                  "which the case expects to pass")
    for k in t["coverage_gaps"]:
        print(f"::warning title=Evaluation coverage gap::{label(result, k)} has no passing or no failing case")
    for m in weak(result["mutation"]):
        print(f"::warning title=Evaluation weak corpus::no case notices {label(result, m['rule'])} "
              f"with its {m['where']} moved from {m['value']} to {m['moved']}")
    if args.check:
        if not args.results.exists() or args.results.read_text(encoding="utf-8") != text:
            print(f"::error title=Evaluation results stale::{args.results} is not what this run writes; "
                  "run python3 evaluation/run.py and commit it")
            status = 1
    else:
        args.results.write_text(text, encoding="utf-8")
        print(f"wrote {args.results}")
    b = t["buckets"]
    print(f"evaluation: {t['known_good']} known-good and {t['known_bad']} known-bad Rule verdicts; "
          f"{t['false_fails']} false fails, {t['false_passes']} false passes; triage: "
          + ", ".join(f"{k} {b[k]}" for k in BUCKETS)
          + f"; rule mutation: {len(result['mutation'])} mutants, {len(weak(result['mutation']))} not noticed")
    return status


if __name__ == "__main__":
    sys.exit(main(sys.argv))
