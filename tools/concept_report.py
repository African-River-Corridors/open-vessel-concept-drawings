#!/usr/bin/env python3
"""A concept design report from an `ovd run --json` Option, as an Open Report Designer (ORD) document (issue #727).

Every concept report of a gas barge has the same parts. This script builds each of them from the run, so
that no figure is typed:

  particulars(o, ref)       main particulars, with a reference Option beside them when given
  lightship(o)              lightship by item (the items add to the lightship; checked)
  steel(o, status)          the hull steel figures (the panel model since #722, the top-down rate) and the gap
  hydrostatics(o)           a hydrostatic table by draught, with Cb
  stability(o)              the loading conditions table; gz_chart(o, path) draws the GZ curves
  powering(o)               installed power, speeds and the gas balance; POWERING_METHOD names the method
  compliance(o)             the compliance matrix: one row per governing or warning Rule, its worst outcome
  rule_paragraphs(o, notes) a short paragraph on each governing Rule, built from its trace lines
  not_evaluated(o)          the Rules the run does not evaluate
  provenance(...)           engine version, commit and results hashes
  ga_figures(...)           the GA sheets of the Option, drawn by `ovd draw`, as cropped PNGs
  build(run, option, ...)   the whole document: the parts above in five sections and an appendix
  render(ord, doc, stem)    `ord check` and `ord render` to PDF, HTML and Word

A project's own narrative and trade studies stay outside this repo: import this module, call the parts,
and add sections around them.

    python3 tools/concept_report.py run.json --option "A basis" [--ref "B profile"] -o report.json
    python3 tools/concept_report.py run.json --option "A basis" -o report.json --render out/ [--ord ord]

`run.json` is the output of `ovd run --json <file.vdesign>`. An Option is one element of its "options"
list ({"name", "results_hash", "hash_status", "results": {...}}). The script reads only the run; it does
not call the engine, except `ga_figures`, which runs `ovd draw`.
"""
import argparse
import json
import pathlib
import re
import subprocess
import sys

NOTICE = "Concept and preliminary design only. Not verified. Never for approval."
OUTCOME_ORDER = ["fail", "warn", "not evaluated", "note", "pass", "not applicable"]
RULE_SET_NAMES = {"igc": "IGC Code", "is2008": "IS Code 2008", "es-trin": "ES-TRIN 2025/1", "icll": "ICLL",
                  "solas": "SOLAS", "lr-inland": "LR inland", "abs-barges": "ABS barges", "rheinschpv": "RheinSchPV",
                  "ovd": "OVD"}
POWERING_METHOD = (
    "Calm-water resistance is the ITTC-1957 friction line times (1 + Cr/Cf). Cr/Cf is a bluff-barge fit that depends "
    "on the Froude number only: an uncalibrated OVD estimate, not a published method. So the method sees a change of "
    "hull form only through the wetted surface and the waterline length. It is not Holtrop–Mennen; that regression "
    "covers Cp 0.73–0.85, L/B 5.1–7.1 and B/T 2.4–3.2 for full forms (Ortigosa et al., 2009).")


# --- ORD blocks ---

def n(x, dp=0):
    return f"{x:,.{dp}f}"


def table(tid, caption, columns, rows, notes=None, aligns=None, widths=None):
    cols = [{"title": c} for c in columns]
    for i, a in enumerate(aligns or []):
        if a:
            cols[i]["align"] = a
    for i, w in enumerate(widths or []):
        if w:
            cols[i]["width"] = w
    t = {"type": "table", "id": tid, "caption": caption, "columns": cols, "rows": rows}
    if notes:
        t["notes"] = notes
    return t


def prose(md):
    return {"type": "prose", "markdown": md}


def figure(fid, caption, alt, width=1.0):
    return {"type": "figure", "id": fid, "caption": caption, "asset": fid, "alt": alt, "width": width}


def section(sid, title, body):
    return {"type": "section", "id": sid, "title": title, "body": body}


# --- the parts ---

def R(o):
    return o["results"]


def cond(o, name):
    return next(c for c in R(o)["stability"]["conditions"] if c["name"] == name)


def _particular_rows(o):
    r, res, g, h = R(o), R(o)["results"], R(o)["geometry"], R(o)["hydrostatics"]
    d = r["dims"]
    return [
        ["Length overall", f"{n(d['loa_m'], 1)} m", "Input"],
        ["Beam, moulded", f"{n(d['beam_m'], 1)} m", "Input"],
        ["Depth, moulded", f"{n(d['depth_m'], 2)} m", "Input"],
        ["Draught, laden (98 % fill)", f"{n(res['draught_m'], 2)} m", "Hydrostatics"],
        ["Draught, light", f"{n(res['light_draught_m'], 2)} m", "Hydrostatics"],
        ["Block coefficient Cb, laden", f"{n(h['cb'], 3)}", f"On Lwl {n(h['l_wl_m'], 1)} m (ES-TRIN 1.01)"],
        ["Cargo tanks", f"{g['n_tanks_total']} × IMO Type C, {n(g['tank_outer_diameter_m'], 2)} m OD", "Fit search, IGC 2.4"],
        ["Cargo at 98 % fill", f"{n(res['cargo_volume_m3'])} m³ / {n(res['cargo_t'])} t", "Tank volume × fill × density"],
        ["Lightship", f"{n(res['lightship_t'])} t", "Build-up, uncalibrated"],
        ["Displacement, laden", f"{n(res['displacement_t'])} t", "Hydrostatics"],
        ["Freeboard, laden", f"{n(res['freeboard_m'], 2)} m", "Depth less draught"],
        ["Air draught, laden / light", f"{n(res['air_draft_loaded_m'], 2)} / {n(res['air_draft_light_m'], 2)} m", "Top of the weather shield"],
        ["Installed power", f"{n(r['powering']['installed_power_kw'])} kW", "Sized to the design speed × margin"],
        ["Service speed, deep water", f"{n(r['powering']['environments'][0]['service_speed_kn'], 1)} kn", "Resistance at installed power"],
    ]


def particulars(o, ref=None, extra_rows=()):
    """Main particulars of Option o. With `ref`, a second value column gives the reference Option."""
    rows = _particular_rows(o)
    if ref is None:
        rows += [list(x) for x in extra_rows]
        return table("tbl-particulars", f"Main particulars, {o['name']}", ["Item", "Value", "Basis"], rows, aligns=[None, "right", None])
    rref = _particular_rows(ref)
    rows = [[a[0], a[1], b[1], a[2]] for a, b in zip(rows, rref)] + [list(x) for x in extra_rows]
    return table("tbl-particulars", f"Main particulars, {o['name']} (reference: {ref['name']})",
                 ["Item", o["name"], f"Reference: {ref['name']}", "Basis"], rows, aligns=[None, "right", "right", None],
                 widths=[0.30, 0.22, 0.22, 0.26])


def steel(o, status=None):
    """The hull steel figures. `status` labels them (for example a review). Since issue #722 the panel
    model (structural members with class minimum scantlings) drives the lightship and the cost; the
    top-down rate is printed for comparison only. A results file from before has no `panel_t`."""
    st = R(o)["cost_masses"]["steel"]
    td, bu = st["top_down_t"], st["bottom_up_t"]
    if "panel_t" in st:
        pn = st["panel_t"]
        rows = [["Panel model: members with class minimum scantlings", n(pn), "Drives the lightship and the cost"],
                ["Top-down: rate × L × B × D", n(td), "Comparison only"],
                ["Gap, top-down over panel", f"{n(td - pn)} ({td / pn - 1:+.0%})", "Reported"]]
    else:
        rows = [["Top-down: rate × L × B × D", n(td), "Drives the lightship"],
                ["Bottom-up: plating build-up", n(bu), "Prices the cost only"],
                ["Gap, top-down over bottom-up", f"{n(td - bu)} (+{td / bu - 1:.0%})", "Not resolved"]]
    cap = f"Hull steel: the figures OVD prints, {o['name']}" + (f", {status}" if status else "")
    return table("tbl-steel", cap, ["Figure", "Hull steel (t)", "Use"], rows, aligns=[None, "right", None],
                 notes=f"Both figures are {status}." if status else None)


def lightship_items(o):
    """The lightship by item. The items add to the lightship: the covering above D is the remainder, checked."""
    r = R(o)
    m, st = r["cost_masses"]["masses"], r["cost_masses"]["steel"]
    items = [
        ("Hull steel to the moulded depth", st.get("panel_t", st["top_down_t"]),
         "Panel model: members with class minimum scantlings" if "panel_t" in st else "Top-down: rate × L × B × D"),
        ("Cargo tanks, shells and supports", m["tank_shell_t"] + m["tank_supports_t"], "Tank shell × structure factor"),
        ("Tank insulation", m["insulation_t"], "Area × thickness × density"),
        ("Piping and machinery", m["piping_machinery_t"], "Factor on hull steel and tanks"),
        ("Outfitting", m["outfitting_t"], "Factor on hull steel and tanks"),
        ("Stern block: propulsors, gensets, deckhouse", r["stern_block_mass"]["total_t"], "Items, each placed"),
    ]
    rest = r["results"]["lightship_t"] - sum(v for _, v, _ in items)
    assert 0 <= rest < 0.1 * r["results"]["lightship_t"], f"covering {rest}"
    items.insert(5, ("Covering above the moulded depth: shield, domes", rest, "Plate areas above D"))
    return items


def lightship(o):
    items = lightship_items(o)
    total = R(o)["results"]["lightship_t"]
    rows = [[a, n(v), f"{v / total:.0%}", c] for a, v, c in items]
    rows.append(["Lightship", n(total), "100%", "Uncalibrated for a gas hull"])
    st = R(o)["cost_masses"]["steel"]
    return table("tbl-lightship", f"Lightship by item, {o['name']}", ["Item", "Mass (t)", "Share", "Basis"], rows,
                 notes=((f"The hull steel is the panel model ({n(st['panel_t'])} t): structural members with class minimum "
                         "scantlings, which drive the lightship and the cost. The top-down rate "
                         f"({n(st['top_down_t'])} t) is printed for comparison only. " if "panel_t" in st else
                         f"The hull steel has two figures in OVD. The lightship uses the top-down one ({n(st['top_down_t'])} t). "
                         f"The bottom-up plating build-up ({n(st['bottom_up_t'])} t) prices the cost only. ")
                        + "The covering above the moulded depth is weighed once, as its own items."),
                 aligns=[None, "right", "right", None])


def hydrostatics(o, draughts=None):
    ht = R(o)["hydrostatic_table"]
    if draughts:
        ht = [r for r in ht if any(abs(r["draught_m"] - d) < 1e-6 for d in draughts)]
    rows = [[n(r["draught_m"], 2), n(r["displacement_t"]), n(r["tpc_t"], 1), n(r["km_t_m"], 2), n(r["l_wl_m"], 1), n(r["cb"], 3)] for r in ht]
    return table("hydrostatic-table", f"Hydrostatics by draught, {o['name']}",
                 ["Draught (m)", "Displacement (t)", "TPC (t/cm)", "KMt (m)", "Lwl (m)", "Cb"], rows, aligns=["right"] * 6)


def _is_code_note(o):
    rows = [r for r in R(o)["rule_trace"]["rows"] if r["rule"].startswith("is2008/")]
    crit = [r for r in rows if r["rule"].startswith("is2008/A.2.2")]
    bad = sorted({r["rule"] for r in crit if r["outcome"] in ("fail", "warn")})
    ne = sorted({r["rule"] for r in rows if r["outcome"] == "not evaluated"})
    s = "Criteria of IS Code 2008 Part A 2.2: " + ("all pass." if crit and not bad else f"not all pass ({', '.join(bad) or 'none run'}).")
    if ne:
        s += " Not evaluated: " + ", ".join(ne) + "."
    return s


def stability(o):
    rows = []
    for c in R(o)["stability"]["conditions"]:
        rows.append([c["name"], n(c["displacement_t"]), n(c["draught_m"], 2), n(c["kg_m"], 2), n(c["gm_fluid_m"], 2),
                     n(c["max_gz_m"], 2), f"{c['max_gz_deg']:.0f}°", f"{c['deck_edge_deg']:.1f}°"])
    return table("tbl-stability", f"Intact stability, the loading conditions, {o['name']}",
                 ["Condition", "Displ. (t)", "T (m)", "KG (m)", "GM fluid (m)", "Max GZ (m)", "At", "Deck edge"], rows,
                 notes=_is_code_note(o), aligns=[None] + ["right"] * 7)


def gz_chart(o, path):
    """Draw the GZ curves of every loading condition to `path` (PNG). Needs matplotlib."""
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    fig, ax = plt.subplots(figsize=(6.3, 2.7))
    for c in R(o)["stability"]["conditions"]:
        ax.plot([p[0] for p in c["gz"]], [p[1] for p in c["gz"]], lw=1.4, label=c["name"])
    ax.axhline(0.2, lw=0.7, ls=":", color="grey")
    ax.set_xlabel("Heel (°)")
    ax.set_ylabel("GZ (m)")
    ax.set_title(f"Righting lever curves, {o['name']}", loc="left", fontsize=10)
    ax.legend(frameon=False, fontsize=7)
    fig.tight_layout()
    fig.savefig(path, dpi=200)
    plt.close(fig)


def powering(o):
    p = R(o)["powering"]
    e = p["environments"][0]
    b = R(o)["boil_off"]
    prop = R(o)["drawing"]["propulsors"]
    rows = [
        ["Design speed", f"{n(p['design_speed_kn'], 1)} kn", "Input"],
        ["Installed power", f"{n(p['installed_power_kw'])} kW", f"Power at the design speed × {p['margin']}"],
        ["Service speed, deep water", f"{n(e['service_speed_kn'], 2)} kn", "Speed at the installed power ÷ margin"],
        ["Power at the service speed", f"{n(e['power_at_service_kw'])} kW", "Resistance ÷ propulsive efficiency"],
        ["Propulsors", f"{len(prop['propulsors'])} × {n(prop['diameter_m'], 1)} m", prop["propulsor_type"].replace("_", ", ")],
    ]
    if e.get("fuel_gas_kg_per_h") is not None:
        rows.append(["Gas use at the service speed", f"{n(e['fuel_gas_kg_per_h'])} kg/h", "Genset heat rate (see the run's fuel_gas basis)"])
    if b.get("evaluated", True) and b.get("boil_off_kg_per_h") is not None:
        rows.append(["Boil-off", f"{n(b['boil_off_kg_per_h'])} kg/h", "Tank heat leak (IGC Code 7.2)"])
    return table("tbl-powering", f"Powering and gas balance, {o['name']}", ["Item", "Value", "Basis"], rows, aligns=[None, "right", None])


# --- rules ---

def _rule_label(r, limit=58):
    s = r["title"].split(";")[0].split(", for ")[0].split(" (")[0]
    if len(s) <= limit:
        return s
    return s[:limit].rsplit(" ", 1)[0].rstrip(",") + " …"


def set_and_clause(rule_id):
    """("IGC Code", "2.4.1.2") from "igc/2.4.1.2/side-clearance"; OVD's own checks have no published clause."""
    head, clause = rule_id.split("/")[0], rule_id.split("/")[1]
    name = RULE_SET_NAMES.get(head, head.upper())
    if head == "ovd":
        clause = f"own check ({clause})"
    return name, clause


def _fmt(x, unit):
    u = "" if unit in ("", "-", None) else f" {unit}"
    return "—" if x is None or isinstance(x, bool) else f"{x:.4g}{u}"


def _outcome(r):
    if r["outcome"] == "pass" and r.get("margin") is not None and abs(r["margin"]) < 1e-6:
        return "pass, at limit"
    return r["outcome"]


def governing(o, include=("pass", "warn", "fail")):
    """One trace line per Rule id that governs, warns or fails: the worst outcome over its objects (ties: the smallest margin)."""
    by, count = {}, {}
    for r in R(o)["rule_trace"]["rows"]:
        if r["outcome"] not in include or (r["outcome"] == "pass" and not r.get("governing")):
            continue
        k = r["rule"]
        count[k] = count.get(k, 0) + 1
        key = (OUTCOME_ORDER.index(r["outcome"]), r["margin"] if r.get("margin") is not None else 0)
        if k not in by or key < by[k][0]:
            by[k] = (key, r)
    out = [dict(r, n_objects=count[k]) for k, (_, r) in by.items()]
    return sorted(out, key=lambda r: (OUTCOME_ORDER.index(r["outcome"]), r["rule"]))


def compliance(o):
    """The compliance matrix: rule, clause, requirement, value, limit, outcome, verified or not."""
    rows = []
    for r in governing(o):
        name, clause = set_and_clause(r["rule"])
        rows.append([name, clause, _rule_label(r), _fmt(r["value"], r["unit"]), _fmt(r["limit"], r["unit"]), _outcome(r),
                     "yes" if r["rule_status"] == "verified" else "no"])
    return table("tbl-compliance", f"Compliance matrix, {o['name']}: the governing Rules and every warning",
                 ["Rule", "Clause", "Requirement", "Value", "Limit", "Outcome", "Verified"], rows,
                 notes=("One row per Rule: the worst outcome over the tanks and conditions it applies to. "
                        "\"Verified\" says whether the Rule's wording has been checked against the published clause; "
                        "a Rule not yet verified still runs. The full trace (all Rules, all objects) is in the run's output."),
                 aligns=[None, None, None, "right", "right", None, None], widths=[0.12, 0.12, 0.34, 0.11, 0.11, 0.11, 0.09])


def rule_paragraphs(o, notes=None):
    """A short paragraph on each governing Rule, from its trace lines, plus an optional note per rule id.

    Returns markdown, one paragraph per Rule of `governing(o)`, in its order. Every fact comes from the trace:
    the requirement, blocking or warning, the worst value against the limit and the number of objects."""
    notes = notes or {}
    paras = []
    for r in governing(o):
        name, clause = set_and_clause(r["rule"])
        verb = {"ge": "at least", "le": "at most", "gt": "more than", "lt": "less than", "eq": "equal to"}.get(r["compare"], r["compare"])
        objs = f"worst of {r['n_objects']} {r['object_type']}s" if r["n_objects"] > 1 else r["object_type"]
        own = r["origin"] == "method-validity" or not r.get("citation")
        role = "; it sizes the design" if r.get("sizes") else ""
        bits = [f"**{name} {clause}.** {r['title'].split(';')[0]}.",
                f"{'Blocking' if r['consequence'] == 'block' else 'A warning only'}{' (OVD guide, no published clause)' if own else ''}{role}.",
                f"{_fmt(r['value'], r['unit'])} against {verb} {_fmt(r['limit'], r['unit'])} ({objs}): {_outcome(r)}."]
        if r["rule"] in notes:
            bits.append(notes[r["rule"]])
        paras.append(" ".join(bits))
    return "\n\n".join(paras)


def not_evaluated(o):
    seen = {}
    for r in R(o)["rule_trace"]["rows"]:
        if r["outcome"] == "not evaluated" and r["rule"] not in seen:
            seen[r["rule"]] = r
    return [f"{k}: {_rule_label(r, 90)}" for k, r in sorted(seen.items())]


def rule_counts(o):
    from collections import Counter
    return Counter(r["outcome"] for r in R(o)["rule_trace"]["rows"])


# --- drawings and provenance ---

def slug(option):
    """The file stem `ovd draw` gives an Option: lower case, every run of other characters a hyphen."""
    return re.sub(r"[^a-z0-9]+", "-", option.lower()).strip("-")


def ga_figures(ovd, vdesign, option, out_dir, crop=True):
    """Draw the Option's GA with `ovd draw` and render each sheet to PNG (needs rsvg-convert; cropping needs Pillow)."""
    out_dir.mkdir(parents=True, exist_ok=True)
    subprocess.run([str(ovd), "draw", "--option", option, "--out", str(out_dir), str(vdesign)], capture_output=True, check=True)
    pngs = {}
    for view in ("side-elevation", "plan", "midship-section"):
        svg = out_dir / f"{slug(option)}-{view}.svg"
        png = out_dir / f"{slug(option)}-{view}.png"
        subprocess.run(["rsvg-convert", "-w", "2400", "-b", "white", str(svg), "-o", str(png)], check=True)
        if crop:
            _crop_views(png)
        pngs[view] = png
    return pngs


def _crop_views(png):
    """Crop a GA sheet to its drawing: inside the frame, above the notes block."""
    from PIL import Image, ImageOps
    im = Image.open(png).convert("L")
    w, h = im.size
    box = (int(0.055 * w), int(0.045 * h), int(0.97 * w), int(0.64 * h))
    inner = im.crop(box)
    bb = ImageOps.invert(inner).point(lambda v: 255 if v > 60 else 0).getbbox()
    if bb:
        pad = 30
        bb = (max(bb[0] - pad, 0), max(bb[1] - pad, 0), min(bb[2] + pad, inner.width), min(bb[3] + pad, inner.height))
        Image.open(png).convert("RGB").crop((box[0] + bb[0], box[1] + bb[1], box[0] + bb[2], box[1] + bb[3])).save(png)


def provenance(engine_version, commit, options):
    rows = [[o["name"], (o.get("results_hash") or "—")[:12], (o.get("hash_status") or "—").replace("_", " ")] for o in options]
    note = f"Open Vessel Designer {engine_version}" + (f", commit {commit[:12]}." if commit else ".")
    return table("tbl-provenance", "The run behind every figure", ["Option", "Results hash", "Hash check"], rows, notes=note)


# --- the document ---

def build(run, option, ref=None, number=None, revision="A", date="1970-01-01", title=None, commit=None, assets=None):
    """The concept report of one Option of `run` (an `ovd run --json` output) as an ORD document.

    `ref` names a reference Option shown beside it. `assets` maps asset ids to PNG paths: "gz" adds the GZ
    chart, "ga-side" and "ga-mid" add the GA views. With no assets the document has tables and text only."""
    o = next(x for x in run["options"] if x["name"] == option)
    r = next(x for x in run["options"] if x["name"] == ref) if ref else None
    assets = assets or {}
    d, res = R(o)["dims"], R(o)["results"]
    counts = rule_counts(o)
    ne = not_evaluated(o)

    design = [prose(f"Option \"{o['name']}\": {n(d['loa_m'], 1)} m × {n(d['beam_m'], 1)} m × {n(d['depth_m'], 2)} m, "
                    f"{n(res['cargo_volume_m3'])} m³ of cargo at a laden draught of {n(res['draught_m'], 2)} m. "
                    f"The cargo is {res['governs']}-bound."),
              particulars(o, r)]
    for fid, cap in (("ga-side", "Side elevation"), ("ga-mid", "Midship section")):
        if fid in assets:
            design.append(figure(fid, f"{cap}, {o['name']} (drawn by OVD)", f"{cap} drawing of the Option"))
    stab = [hydrostatics(o), stability(o)]
    if "gz" in assets:
        stab.append(figure("gz", "Righting lever (GZ) curves", "GZ curves of the loading conditions"))
    body = [
        section("design", "The design at a glance", design),
        section("lightship", "Lightship and hull steel", [lightship(o), steel(o)]),
        section("stability", "Hydrostatics and stability", stab),
        section("powering", "Powering and gas balance", [prose(f"**How OVD works out the power.** {POWERING_METHOD}"), powering(o)]),
        section("rules", "How the rules were applied", [
            prose(f"{counts['pass']} trace lines pass, {counts['warn']} warn, {counts.get('fail', 0)} fail and "
                  f"{counts['not evaluated']} are not evaluated. The matrix shows each governing Rule and every warning once."),
            compliance(o),
            prose("**The governing Rules, one by one.**\n\n" + rule_paragraphs(o)),
            prose("**Not evaluated in this run:**\n\n" + ("\n".join(f"- `{x}`" for x in ne) or "- none")),
        ]),
    ]
    control = {"revision": revision, "date": date, "status": "draft"}
    if number:
        control["number"] = number
    doc = {
        "ord": "0.2", "class": "technical-note",
        "title": title or f"Concept design report: {o['name']}",
        "language": "en-GB", "contents": True, "notice": NOTICE, "control": control,
        "body": body,
        "appendices": [{"id": "provenance", "title": "Run provenance",
                        "body": [provenance(run.get("engine_version", "—"), commit, [o] + ([r] if r else []))]}],
    }
    if assets:
        doc["assets"] = {k: {"media_type": "image/png", "path": str(p)} for k, p in assets.items()}
    return doc


def render(ord_bin, doc, out_stem):
    """Write `<stem dir>/report.json`, check it with `ord check`, and render PDF, HTML and Word beside `out_stem`."""
    out_stem = pathlib.Path(out_stem)
    src = out_stem.parent / "report.json"
    src.write_text(json.dumps(doc, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    subprocess.run([str(ord_bin), "check", str(src)], check=True)
    tmp = out_stem.parent / ".render"
    tmp.mkdir(exist_ok=True)
    subprocess.run([str(ord_bin), "render", str(src), "--format", "all", "-o", str(tmp)], check=True)
    written = []
    for ext in ("pdf", "html", "docx"):
        p = tmp / f"report.{ext}"
        if p.exists():
            dest = pathlib.Path(f"{out_stem}.{ext}")
            dest.write_bytes(p.read_bytes())
            written.append(dest)
    return written


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("run", type=pathlib.Path, help="the output of `ovd run --json <file.vdesign>`")
    ap.add_argument("--option", required=True)
    ap.add_argument("--ref", help="a reference Option shown beside the main one")
    ap.add_argument("--number")
    ap.add_argument("--revision", default="A")
    ap.add_argument("--date", default="1970-01-01")
    ap.add_argument("--title")
    ap.add_argument("--commit", help="the engine commit the run was made at")
    ap.add_argument("-o", "--out", type=pathlib.Path, required=True, help="the ORD document to write")
    ap.add_argument("--render", type=pathlib.Path, help="also render PDF, HTML and Word into this folder")
    ap.add_argument("--ord", default="ord", help="the ord binary (for --render)")
    a = ap.parse_args(argv)
    run = json.loads(a.run.read_text(encoding="utf-8"))
    doc = build(run, a.option, a.ref, a.number, a.revision, a.date, a.title, a.commit)
    a.out.write_text(json.dumps(doc, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    if a.render:
        a.render.mkdir(parents=True, exist_ok=True)
        for p in render(a.ord, doc, a.render / slug(a.option)):
            print(p)
    return 0


if __name__ == "__main__":
    sys.exit(main())
