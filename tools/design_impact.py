#!/usr/bin/env python3
"""Design-impact check: do designs move on this branch? (spec 0001, ticket 28)

A fixed set of Designs (the calibration vessels plus a small sweep) is run through `ovcd` twice,
once built from the base (the merge base with `origin/main`, or the parent commit when this is
`main` itself) and once built from this tree: `ovcd fill`, then `ovcd run --json`. Nothing is
stored, so nothing goes stale. If every result is identical the report says "no design change".
If anything moved, the report holds a before/after table.

    python3 tools/design_impact.py report [--publish --pr N]   report only, never fails on a change
    python3 tools/design_impact.py gate                        report, and fail when a change is
                                                               missing from CHANGELOG.md
    python3 tools/design_impact.py changelog [--key NAME]      write the table into CHANGELOG.md
    python3 tools/design_impact.py publish-artifact DIR --head-sha SHA --head-repo OWNER/NAME
                                                               publish a report built by a pull request
                                                               workflow (used for fork pull requests)

The report goes to target/design-impact/report.md (and summary.json), is printed, and is added to
the GitHub step summary when there is one. `--publish` puts it into the pull request body between
two markers and adds or removes the `design-change` label (needs GH_TOKEN and the `gh` CLI).

A design change needs no sign-off. The changelog entry exists so it can be diagnosed later.
Standard library plus PyYAML (a dependency of the package).
"""
from __future__ import annotations

import argparse
import datetime
import json
import math
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "target" / "design-impact"
CHANGELOG = ROOT / "CHANGELOG.md"
START, END = "<!-- design-impact:start -->", "<!-- design-impact:end -->"
LABEL = "design-change"
MAX_ROWS = 40
CHANGELOG_HEAD = (
    "# Changelog\n\n"
    "## Design changes\n\n"
    "Each entry is a before/after table written by `python3 tools/design_impact.py changelog` when a\n"
    "branch moved the results of the fixed design set. No sign-off is needed; the entries are kept so\n"
    "a change can be traced later. Newest first.\n"
)
SECTION = "## Design changes"
ENTRIES_AT = "\n### "


# --- the fixed case set -------------------------------------------------------------------------

def _inputs(cargo, containment, formation, loa, beam, depth, **more):
    d = {"cargo": cargo, "hull_form": "raked_box", "formation": {"kind": formation},
         "length_overall_m": loa, "beam_m": beam, "moulded_depth_m": depth,
         "cargo_containment": containment}
    d.update(more)
    return d


GAS_CARGOES = ("LNG", "LNG_ethane", "LPG_propane", "LPG_butane", "LPG_mix")
# The CN note gives no depth. Draught plus this many metres is a stand-in that keeps the set stable.
CN_DEPTH_ABOVE_DRAUGHT_M = 1.5
DEPTH_NOTE = (f"The CN-derived cases take their length, beam, draught and speed from a published draft note. "
              f"The note gives no moulded depth, so each uses the draught plus {CN_DEPTH_ABOVE_DRAUGHT_M:g} m "
              f"as a stand-in. They are sweep points, not calibration comparisons.")

# One case for each optional input, on a hull where the input moves a result. Each entry:
# (case name, cargo, containment, formation, loa, beam, depth, extra inputs).
_CHANNEL = {"channel_depth_m": 5.0, "channel_bottom_width_m": 60.0}
_DEEP_CHANNEL = {"channel_depth_m": 3.0, "channel_bottom_width_m": 30.0}      # shallow and narrow
OPTIONAL_INPUT_CASES = (
    ("block_coefficient", "bauxite", "sloped_hopper_hold", "self_propelled", 90.0, 14.0, 4.5,
     {"block_coefficient": 0.7}),
    ("min_freeboard_m", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5, {"min_freeboard_m": 0.9}),
    ("lightship_factor_t_per_m3", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5,
     {"lightship_factor_t_per_m3": 0.2}),
    ("propulsive_efficiency", "bauxite", "box_hold", "self_propelled", 90.0, 14.0, 4.5,
     {"propulsive_efficiency": 0.55, "max_speed_kn": 9.0}),
    ("air_draft_limit_m", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5, {"air_draft_limit_m": 6.0}),
    ("channel_current_kn", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5,
     {**_DEEP_CHANNEL, "channel_current_kn": 2.0}),
    ("channel_bank_slope", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5,
     {**_DEEP_CHANNEL, "channel_bank_slope": 2.0}),
    ("channel_under_keel_clearance_m", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5,
     {**_DEEP_CHANNEL, "channel_under_keel_clearance_m": 0.8}),
    ("fill_ratio", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0, {"fill_ratio": 0.85}),
    ("adn_tank_cap_m3", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"ruleset": "ADN_G", "adn_tank_cap_m3": 250.0}),
)


def cases() -> dict[str, dict]:
    """Case name -> typed inputs of one Option. Everything else comes from the Defaults Profile,
    so a change to the profile moves these designs too.

    The set reaches every cargo, every optional input, dry bulk with a channel, gas designs that
    pass every check and gas designs that carry nothing. `tests/test_design_impact_cases.py` checks
    that it keeps doing so when an input or a cargo is added.
    """
    out: dict[str, dict] = {}
    cal = yaml.safe_load((ROOT / "calibration" / "cases.yaml").read_text(encoding="utf-8"))
    for c in cal["dry_barge"]:
        if c["hold_type"] != "box":
            continue                              # the Design format has no container hold yet
        extra = {"draught_limit_m": c["draught_m"]} if c.get("draught_m") else {}
        out[f"calibration {c['id']}"] = _inputs(
            "bauxite", "box_hold", "pushed_barge", c["loa_m"], c["beam_m"], c["depth_m"], **extra)
    motor = yaml.safe_load((ROOT / "calibration" / "cn_yangtze_gb38030_draft_2018.yaml").read_text(
        encoding="utf-8"))
    for cargo in ("bulk", "liquid"):
        point = motor["design_points"][cargo]
        for i, (loa, boa, draught, *_rest) in enumerate(point["rows"][:3]):
            knots = round(point["speed_kmh"] / 1.852, 1)
            depth = round(draught + CN_DEPTH_ABOVE_DRAUGHT_M, 2)
            if cargo == "bulk":
                inp = _inputs("bauxite", "sloped_hopper_hold", "self_propelled", loa, boa,
                              depth, draught_limit_m=draught, max_speed_kn=knots)
            else:
                inp = _inputs("LPG_propane", "type_c_tanks", "self_propelled", loa, boa,
                              depth, draught_limit_m=draught, max_speed_kn=knots)
            out[f"sweep CN-derived {cargo} {i + 1} ({loa:g}x{boa:g})"] = inp
    # A sweep over vessel kinds, sizes, formations and a channel.
    for loa, beam in ((60.0, 10.0), (90.0, 14.0)):
        tag = f"{loa:g}x{beam:g}"
        out[f"sweep box {tag}"] = _inputs("bauxite", "box_hold", "pushed_barge", loa, beam, 4.5)
        out[f"sweep box self-propelled {tag}"] = _inputs(
            "bauxite", "box_hold", "self_propelled", loa, beam, 4.5)
        out[f"sweep hopper {tag}"] = _inputs(
            "bauxite", "sloped_hopper_hold", "self_propelled", loa, beam, 4.5)
        out[f"sweep hopper pushed {tag}"] = _inputs(
            "bauxite", "sloped_hopper_hold", "pushed_barge", loa, beam, 4.5)
        out[f"sweep box channel {tag}"] = _inputs(
            "bauxite", "box_hold", "pushed_barge", loa, beam, 4.5, **_CHANNEL)
        out[f"sweep hopper channel {tag}"] = _inputs(
            "bauxite", "sloped_hopper_hold", "self_propelled", loa, beam, 4.5, max_speed_kn=9.0,
            channel_current_kn=1.5, **_CHANNEL)
        for ruleset in ("IGC_2G", "IGC_2G_EXACT", "ADN_G"):
            out[f"sweep gas {ruleset} {tag}"] = _inputs(
                "LPG_propane", "type_c_tanks", "self_propelled", loa, beam, 5.5, ruleset=ruleset)
        out[f"sweep gas channel {tag}"] = _inputs(
            "LNG", "type_c_tanks", "pushed_barge", loa, beam, 6.0, draught_limit_m=3.5,
            channel_depth_m=5.0, channel_bottom_width_m=60.0, optimise_depth=True)
    # Every gas cargo (the sweep above uses only propane and LNG).
    for cargo in GAS_CARGOES:
        out[f"sweep gas cargo {cargo}"] = _inputs(
            cargo, "type_c_tanks", "self_propelled", 90.0, 14.0, 5.5, max_speed_kn=9.0)
        out[f"sweep gas cargo {cargo} pushed"] = _inputs(
            cargo, "type_c_tanks", "pushed_barge", 90.0, 14.0, 6.0)
    # Gas designs that pass every check, so a change to a check's wording or limit shows.
    out["sweep gas passes all checks IGC_2G 110x11.4"] = _inputs(
        "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0, ruleset="IGC_2G")
    out["sweep gas passes all checks ADN_G 80x14"] = _inputs(
        "LPG_propane", "type_c_tanks", "self_propelled", 80.0, 14.0, 4.5, ruleset="ADN_G")
    # Designs that carry nothing: the messages are results too.
    out["sweep gas carries nothing (hull too short)"] = _inputs(
        "LPG_propane", "type_c_tanks", "self_propelled", 12.0, 4.0, 2.0)
    out["sweep gas carries nothing (draught limit)"] = _inputs(
        "LPG_propane", "type_c_tanks", "self_propelled", 90.0, 14.0, 5.5, draught_limit_m=0.5)
    out["sweep box lightship over the draught limit"] = _inputs(
        "bauxite", "box_hold", "pushed_barge", 60.0, 10.0, 4.5, draught_limit_m=0.3)
    # One case for each optional input.
    for name, cargo, containment, formation, loa, beam, depth, extra in OPTIONAL_INPUT_CASES:
        out[f"sweep optional {name}"] = _inputs(cargo, containment, formation, loa, beam, depth, **extra)
    return out


# --- running ovcd -------------------------------------------------------------------------------

def run(cmd, **kw) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, capture_output=True, text=True, **kw)


def run_case(binary: Path, name: str, typed: dict, tmp: Path) -> dict:
    """One case through one binary: {"results": {...}} or {"error": "..."}."""
    raw, filled = tmp / "in.vdesign", tmp / "filled.vdesign"
    filled.unlink(missing_ok=True)
    raw.write_text(json.dumps({"format": "vdesign", "format_version": 1, "engine_version": "0.0.0",
                               "options": [{"name": name, "inputs": typed}]}), encoding="utf-8")
    fill = run([str(binary), "fill", str(raw), str(filled)])
    if fill.returncode != 0:
        return {"error": "fill: " + " | ".join(fill.stderr.strip().splitlines()[-3:])}
    res = run([str(binary), "run", "--json", str(filled)])
    try:
        option = json.loads(res.stdout)["options"][0]
    except (ValueError, KeyError, IndexError):
        return {"error": "run: " + " | ".join((res.stderr or res.stdout).strip().splitlines()[-3:])}
    if option.get("results") is None:
        return {"error": option.get("message", "no results")}
    return {"results": option["results"]}


def run_all(binary: Path, case_set: dict[str, dict]) -> dict[str, dict]:
    with tempfile.TemporaryDirectory() as t:
        return {name: run_case(binary, name, typed, Path(t)) for name, typed in case_set.items()}


# --- comparing ----------------------------------------------------------------------------------

def leaves(value, path="") -> dict[str, object]:
    """Every scalar of a JSON value, by path."""
    if isinstance(value, dict):
        out: dict[str, object] = {}
        for k, v in value.items():
            out.update(leaves(v, f"{path}.{k}" if path else k))
        return out
    if isinstance(value, list):
        out = {}
        for i, v in enumerate(value):
            out.update(leaves(v, f"{path}[{i}]"))
        return out
    return {path: value}


def show(v) -> str:
    if v is None:
        return "none"
    if isinstance(v, float):
        return f"{v:.12g}"          # more than the 1e-9 tolerance needs, so a 1e-6 move shows
    return str(v)


def change(a, b) -> str:
    if is_number(a) and is_number(b) and isinstance(a, float) != isinstance(b, float) and a == b:
        return f"{type(a).__name__} to {type(b).__name__}"
    if isinstance(a, (int, float)) and isinstance(b, (int, float)) \
            and not isinstance(a, bool) and not isinstance(b, bool):
        if a == 0:
            return f"{b - a:+.6g}"
        return f"{(b - a) / abs(a) * 100:+.3g} %"
    return "changed"


def is_number(v) -> bool:
    return isinstance(v, (int, float)) and not isinstance(v, bool)


def same(a, b) -> bool:
    """Equal, with numbers compared at the project tolerance (relative and absolute 1e-9).
    A change of type, int to float or back, is a change: it changes the results hash."""
    if isinstance(a, bool) or isinstance(b, bool):
        return a == b and type(a) is type(b)
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        if isinstance(a, float) != isinstance(b, float):
            return False
        return math.isclose(a, b, rel_tol=1e-9, abs_tol=1e-9)
    return a == b


def size(row: dict) -> float:
    """How far a row moved, for ordering: relative for numbers, infinite for anything else."""
    a, b = row["before"], row["after"]
    if isinstance(a, (int, float)) and isinstance(b, (int, float)) \
            and not isinstance(a, bool) and not isinstance(b, bool):
        return abs(b - a) / abs(a) if a else abs(b - a)
    return math.inf


def compare(before: dict[str, dict], after: dict[str, dict]) -> list[dict]:
    """Rows of what moved, biggest first: case, quantity, before, after, change. A number that
    moved by no more than the tolerance (1e-9) has not moved."""
    rows = []
    for name in before:
        a, b = before[name], after.get(name, {"error": "case missing"})
        if "error" in a or "error" in b:
            if a != b:
                rows.append({"case": name, "quantity": "result",
                             "before": a.get("error", "computed"), "after": b.get("error", "computed"),
                             "change": "changed"})
            continue
        la, lb = leaves(a["results"]), leaves(b["results"])
        for path in sorted(set(la) | set(lb)):
            va, vb = la.get(path, "absent"), lb.get(path, "absent")
            if not same(va, vb):
                rows.append({"case": name, "quantity": path, "before": va, "after": vb,
                             "change": change(va, vb)})
    return sorted(rows, key=lambda r: (-size(r), r["case"], r["quantity"]))


def table(rows: list[dict]) -> str:
    shown = rows[:MAX_ROWS]
    lines = ["| Case | Quantity | Before | After | Change |", "|---|---|---|---|---|"]
    for r in shown:
        cell = lambda s: str(s).replace("|", "\\|").replace("\n", " ")
        lines.append(f"| {cell(r['case'])} | `{cell(r['quantity'])}` | {cell(show(r['before']))} "
                     f"| {cell(show(r['after']))} | {cell(r['change'])} |")
    if len(rows) > len(shown):
        lines.append(f"| ... | {len(rows) - len(shown)} more changed quantities | | | |")
    return "\n".join(lines)


def report_text(total: int, rows: list[dict], base: str, head: str) -> tuple[str, str]:
    """(the report, the bare table or "")."""
    moved = sorted({r["case"] for r in rows})
    if not rows:
        return (f"## Design impact\n\nno design change: {total} case designs give identical results "
                f"on `{base}` and on `{head}`.\n\n{DEPTH_NOTE}\n"), ""
    t = table(rows)
    return (f"## Design impact\n\n**design change**: {len(moved)} of {total} case designs moved "
            f"({len(rows)} quantities) between `{base}` and `{head}`. Labelled `{LABEL}`; no "
            f"sign-off is needed.\n\n{t}\n\n{DEPTH_NOTE}\n"), t


# --- finding and building the two binaries ------------------------------------------------------

def git(*args, check=True) -> str:
    r = run(["git", *args], cwd=ROOT)
    if check and r.returncode != 0:
        raise SystemExit(f"design_impact: git {' '.join(args)} failed: {r.stderr.strip()}")
    return r.stdout.strip()


def base_commit() -> tuple[str, bool]:
    """(the commit to compare with, whether this tree is ahead of main). Fails if there is none."""
    head = git("rev-parse", "HEAD")
    for ref in ("origin/main", "main"):
        if run(["git", "rev-parse", "--verify", "-q", ref], cwd=ROOT).returncode == 0:
            mb = git("merge-base", "HEAD", ref)
            if mb != head:
                return mb, True
            break
    parent = run(["git", "rev-parse", "--verify", "-q", "HEAD~1"], cwd=ROOT)
    if parent.returncode != 0:
        raise SystemExit("design_impact: no base commit to compare with (shallow clone? "
                         "CI must fetch the full history)")
    return parent.stdout.strip(), False


def build(tree: Path, target: Path, fresh: bool = False) -> Path:
    """Build `ovcd` in `tree` and return the binary that cargo says it built.

    The path comes from cargo's own JSON messages, never from a guess about the target directory.
    `fresh` starts from an empty target directory, and then the binary must be newer than the start
    of the build. The check fails when cargo reports no `ovcd` binary, or the file is missing or
    is not under `target`: a leftover binary from another tree must never be measured.
    """
    if fresh:
        shutil.rmtree(target, ignore_errors=True)
    started = time.time()
    env = {**os.environ, "CARGO_TARGET_DIR": str(target)}
    r = subprocess.run(["cargo", "build", "-q", "-p", "ovcd-cli", "--message-format=json-render-diagnostics"],
                       cwd=tree, env=env, stdout=subprocess.PIPE, text=True)
    if r.returncode != 0:
        raise SystemExit(f"design_impact: cargo build failed in {tree}")
    exe = None
    for line in r.stdout.splitlines():
        try:
            msg = json.loads(line)
        except ValueError:
            continue
        if (isinstance(msg, dict) and msg.get("reason") == "compiler-artifact" and msg.get("executable")
                and msg.get("target", {}).get("name") == "ovcd" and "bin" in msg["target"].get("kind", [])):
            exe = Path(msg["executable"])
    if exe is None:
        raise SystemExit(f"design_impact: cargo built in {tree} but reported no `ovcd` binary")
    if not exe.is_file():
        raise SystemExit(f"design_impact: cargo reported {exe} but the file is missing")
    if not exe.resolve().is_relative_to(target.resolve()):
        raise SystemExit(f"design_impact: the `ovcd` binary {exe} is not under {target}")
    if fresh and exe.stat().st_mtime < started - 2:
        raise SystemExit(f"design_impact: {exe} is older than this build (stale binary)")
    return exe


def measure() -> tuple[str, str, int, list[dict], bool]:
    """Run both binaries. (base label, head label, case count, moved rows, ahead of main)."""
    base_sha, ahead = base_commit()
    case_set = cases()
    work = Path(tempfile.mkdtemp(prefix="design-impact-"))
    try:
        base_tree = work / "base"
        git("worktree", "add", "-q", "--detach", str(base_tree), base_sha)
        try:
            base_bin = build(base_tree, ROOT / "target" / "design-impact-base", fresh=True)
            head_bin = build(ROOT, ROOT / "target")
            before, after = run_all(base_bin, case_set), run_all(head_bin, case_set)
            # Fail closed: a check that computed nothing on either side proves nothing.
            for label, side in (("base", before), ("this tree", after)):
                if not any("results" in v for v in side.values()):
                    raise SystemExit(f"design_impact: no case designs computed on the {label} binary")
        finally:
            subprocess.run(["git", "worktree", "remove", "--force", str(base_tree)], cwd=ROOT,
                           capture_output=True)
    finally:
        shutil.rmtree(work, ignore_errors=True)
    base_label = ("main " if ahead else "the parent commit ") + base_sha[:7]
    head_label = "this branch " + git("rev-parse", "--short", "HEAD")
    return base_label, head_label, len(case_set), compare(before, after), ahead


# --- outputs ------------------------------------------------------------------------------------

def write_outputs(text: str, rows: list[dict]) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "report.md").write_text(text, encoding="utf-8")
    (OUT / "summary.json").write_text(json.dumps(
        {"moved": bool(rows), "quantities": len(rows), "cases": sorted({r["case"] for r in rows})},
        indent=1), encoding="utf-8")
    print(text)
    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a", encoding="utf-8") as f:
            f.write(text + "\n")


def publish(text: str, moved: bool, pr: int) -> None:
    body = json.loads(subprocess.run(["gh", "pr", "view", str(pr), "--json", "body"], cwd=ROOT,
                                     capture_output=True, text=True, check=True).stdout)["body"] or ""
    block = f"{START}\n{text.rstrip()}\n{END}"
    pattern = re.compile(re.escape(START) + r".*?" + re.escape(END), re.S)
    new = pattern.sub(lambda _m: block, body) if pattern.search(body) else (body.rstrip() + "\n\n" + block + "\n")
    OUT.mkdir(parents=True, exist_ok=True)
    path = OUT / "body.md"
    path.write_text(new, encoding="utf-8")
    subprocess.run(["gh", "pr", "edit", str(pr), "--body-file", str(path)], cwd=ROOT, check=True)
    if moved:
        subprocess.run(["gh", "label", "create", LABEL, "--color", "fbca04", "--description",
                        "Moves design results; tracked, no sign-off"], cwd=ROOT, capture_output=True)
        subprocess.run(["gh", "pr", "edit", str(pr), "--add-label", LABEL], cwd=ROOT, check=True)
    else:
        subprocess.run(["gh", "pr", "edit", str(pr), "--remove-label", LABEL], cwd=ROOT,
                       capture_output=True)


# --- publishing a report built by an untrusted run (fork pull requests) --------------------------
# The `Design impact` workflow runs the pull request's own code with a read-only token and keeps
# report.md, summary.json and pr.txt as an artifact. A second workflow (`workflow_run`, with a write
# token, running THIS file from the default branch) calls `publish-artifact`. The artifact is data
# from an untrusted run: it is size-limited, checked for shape and made harmless before use, and it
# must belong to the pull request that the run was started for.

MAX_REPORT_CHARS = 60_000                    # a pull request body is limited to 65,536
ARTIFACT_FILES = ("report.md", "summary.json", "pr.txt")


def load_artifact(directory: Path) -> tuple[str, bool, int]:
    """(report text, whether designs moved, pull request number) from an untrusted artifact.
    Raises SystemExit on anything that is not exactly the expected shape."""
    def bad(why: str):
        raise SystemExit(f"design_impact: artifact rejected: {why}")

    for name in ARTIFACT_FILES:
        f = directory / name
        if f.is_symlink() or not f.is_file():
            bad(f"{name} is missing or is not a plain file")
        if f.stat().st_size > 4 * MAX_REPORT_CHARS:
            bad(f"{name} is too large")
    raw_pr = (directory / "pr.txt").read_text(encoding="utf-8").strip()
    if not re.fullmatch(r"[1-9][0-9]{0,8}", raw_pr):
        bad("pr.txt is not a pull request number")
    try:
        summary = json.loads((directory / "summary.json").read_text(encoding="utf-8"))
    except ValueError:
        bad("summary.json is not JSON")
    if not isinstance(summary, dict) or not isinstance(summary.get("moved"), bool):
        bad("summary.json has no boolean `moved`")
    text = (directory / "report.md").read_text(encoding="utf-8")
    if len(text) > MAX_REPORT_CHARS:
        bad("report.md is too long")
    if not text.startswith("## Design impact\n"):
        bad("report.md does not start with the Design impact heading")
    return harmless(text), summary["moved"], int(raw_pr)


def harmless(text: str) -> str:
    """The report is written by the pull request's code, so it may not carry markers that would
    break out of its block, control characters, or an @mention that notifies someone."""
    text = text.replace(START, "").replace(END, "")
    text = "".join(c for c in text if c in "\n\t" or ord(c) >= 32)
    return re.sub(r"@(?=[A-Za-z0-9_])", "@\u200b", text)


def publish_artifact(directory: Path, head_sha: str, head_repo: str) -> None:
    """Publish the report in `directory` to the pull request it names, if that pull request's head is
    the commit (and repository) that the workflow run was made for. Needs GH_TOKEN."""
    text, moved, pr = load_artifact(directory)
    repo = os.environ.get("GITHUB_REPOSITORY") or ""
    if not re.fullmatch(r"[\w.-]+/[\w.-]+", repo):
        raise SystemExit("design_impact: GITHUB_REPOSITORY is not set")
    r = subprocess.run(["gh", "api", f"repos/{repo}/pulls/{pr}", "--jq",
                        '[.state, .head.sha, .head.repo.full_name // ""] | @tsv'],
                       cwd=ROOT, capture_output=True, text=True)
    if r.returncode != 0:
        raise SystemExit(f"design_impact: cannot read pull request {pr}: {r.stderr.strip()}")
    state, sha, repo_of_head = (r.stdout.strip().split("\t") + ["", "", ""])[:3]
    if state != "open":
        print(f"design_impact: pull request {pr} is {state}; nothing to publish")
        return
    if sha != head_sha or repo_of_head != head_repo:
        print(f"design_impact: pull request {pr} has moved on (head {sha[:7]}, run was for {head_sha[:7]}); "
              "a newer run will publish")
        return
    publish(text, moved, pr)


# --- the changelog ------------------------------------------------------------------------------

def changelog_has(tbl: str) -> bool:
    return CHANGELOG.exists() and tbl in CHANGELOG.read_text(encoding="utf-8")


def write_changelog(tbl: str, key: str, base: str, head: str) -> None:
    """Add (or replace, by `key`) one entry under "Design changes", newest first.

    Entries go after the `## Design changes` heading, whatever `### ` headings come before it.
    Anything after the section (the next `## ` heading) is kept as it was.
    """
    text = CHANGELOG.read_text(encoding="utf-8") if CHANGELOG.exists() else CHANGELOG_HEAD
    heading = re.search(rf"^{re.escape(SECTION)}[ \t]*$", text, re.M)
    if heading is None:                                   # no section yet: start it at the end
        intro_text = CHANGELOG_HEAD.split(SECTION, 1)[1]
        text = text.rstrip("\n") + f"\n\n{SECTION}{intro_text}"
        heading = re.search(rf"^{re.escape(SECTION)}[ \t]*$", text, re.M)
    before, section = text[:heading.end()], text[heading.end():]
    nxt = re.search(r"\n## ", section)
    tail = section[nxt.start():] if nxt else ""
    section = section[:nxt.start()] if nxt else section
    intro, _, rest = section.partition(ENTRIES_AT)
    entries = [e for e in (ENTRIES_AT + rest).split(ENTRIES_AT) if e.strip()] if rest else []
    entries = [e for e in entries if not e.startswith(f"{key} ")]
    today = datetime.date.today().isoformat()
    entry = f"{key} ({today})\n\nBetween `{base}` and `{head}`.\n\n{tbl}\n"
    body = intro.rstrip("\n") + "\n" + "".join(ENTRIES_AT + e.rstrip("\n") + "\n" for e in [entry] + entries)
    CHANGELOG.write_text(before + body + ("\n" + tail.lstrip("\n") if tail else ""), encoding="utf-8")


def branch_key() -> str:
    name = git("rev-parse", "--abbrev-ref", "HEAD", check=False)
    return name if name and name != "HEAD" else "branch " + git("rev-parse", "--short", "HEAD")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("mode", choices=["report", "gate", "changelog", "publish-artifact"])
    ap.add_argument("artifact_dir", nargs="?", type=Path, help="publish-artifact: the downloaded artifact")
    ap.add_argument("--head-sha", help="publish-artifact: the commit the workflow run was made for")
    ap.add_argument("--head-repo", help="publish-artifact: the repository that commit is in")
    ap.add_argument("--publish", action="store_true", help="report: put the report in the PR body and set the label")
    ap.add_argument("--pr", type=int, help="the pull request number for --publish")
    ap.add_argument("--key", help="changelog entry name (default: the branch)")
    args = ap.parse_args()
    if args.mode == "publish-artifact":
        if not (args.artifact_dir and args.head_sha and args.head_repo):
            raise SystemExit("design_impact: publish-artifact needs DIR, --head-sha and --head-repo")
        publish_artifact(args.artifact_dir, args.head_sha, args.head_repo)
        return 0
    base, head, total, rows, ahead = measure()
    text, tbl = report_text(total, rows, base, head)
    write_outputs(text, rows)
    if args.publish:
        if not args.pr:
            raise SystemExit("design_impact: --publish needs --pr")
        publish(text, bool(rows), args.pr)
    if args.mode == "changelog":
        if not rows:
            print("design_impact: no design change; nothing to write to CHANGELOG.md")
            return 0
        write_changelog(tbl, args.key or branch_key(), base, head)
        print(f"design_impact: wrote CHANGELOG.md ({args.key or branch_key()})")
        return 0
    if args.mode == "gate" and rows and ahead and not changelog_has(tbl):
        print("design_impact: this branch moves designs but CHANGELOG.md does not hold the table above.\n"
              "  Run: python3 tools/design_impact.py changelog   and commit CHANGELOG.md", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
