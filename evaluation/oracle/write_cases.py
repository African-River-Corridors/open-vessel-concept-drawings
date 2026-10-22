"""Write the base case files of the evaluation corpus from the oracle (issue #184).

    python3 evaluation/oracle/write_cases.py          # rewrites evaluation/cases/box_*.yaml
    python3 evaluation/oracle/write_cases.py --check  # fails if a case file differs from the oracle

The expected values of a derived case are the oracle's, to 10 significant figures; they are never
typed by hand. The mutant generator (phase 0, second pull request) reads these files.
"""
from __future__ import annotations

import sys
from pathlib import Path

import yaml

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import box_oracle as oracle  # noqa: E402

CASES_DIR = HERE.parent / "cases"

RULES = {
    "area_0_30": ("IS Code 2008 Part A 2.2.1", "area under the GZ curve, 0 to 30 degrees", "m rad"),
    "area_0_40": ("IS Code 2008 Part A 2.2.1", "area under the GZ curve, 0 to 40 degrees", "m rad"),
    "area_30_40": ("IS Code 2008 Part A 2.2.1", "area under the GZ curve, 30 to 40 degrees", "m rad"),
    "gz_30_plus": ("IS Code 2008 Part A 2.2.2", "GZ at an angle of heel of 30 degrees or more", "m"),
    "angle_max_gz": ("IS Code 2008 Part A 2.2.3", "angle of maximum GZ", "deg"),
    "gm": ("IS Code 2008 Part A 2.2.4", "initial metacentric height", "m"),
}

#: The oracle's criterion -> the id of the Rule of the built-in set `is2008-a` that states it. A case's
#: `expected` entries are keyed by these ids (issue #278); the oracle keeps its own short names.
RULE_IDS = {
    "area_0_30": "is2008/A.2.2.1/area-0-30",
    "area_0_40": "is2008/A.2.2.1/area-0-40",
    "area_30_40": "is2008/A.2.2.1/area-30-40",
    "gz_30_plus": "is2008/A.2.2.2/gz-30",
    "angle_max_gz": "is2008/A.2.2.3/angle-max-gz",
    "gm": "is2008/A.2.2.4/gm0",
}

SOURCE_RULES = (
    "IMO resolution MSC.267(85), annex, International Code on Intact Stability, 2008, Part A 2.2.1 "
    "to 2.2.4 (printed pages 11 and 12): the limits."
)

PAPER = {
    "citation": "Marlantes, K. E.; Kim, S.; Hurt, L. A. Implementation of the IMO Second Generation "
                "Intact Stability Guidelines. J. Mar. Sci. Eng. 2022, 10(1), 41, published 30 December "
                "2021, https://doi.org/10.3390/jmse10010041 (authors as printed on the paper)",
    "locator": "section 5.1, Tables 3 and 4",
    "licence": "CC BY 4.0, https://creativecommons.org/licenses/by/4.0/ (attribution: the authors above)",
    "read": "read in a browser by the maintainer's build coordinator, 2026-10-16; a scripted download "
            "returns HTTP 403",
    "as_printed": {
        "section": "5.1, a simple uniform rectangular cross section",
        "table_3_principal_dimensions": {"L_m": 100.0, "B_m": 20.0, "D_m": 10.0, "d_full_m": 5.0,
                                          "CB": 1.0, "CM": 1.0},
        "table_4_loading_condition": {"d_aft_m": 5.0, "d_fore_m": 5.0, "KG_m": 7.0, "GM_m": 2.167,
                                       "volumetric_displacement_m3": 10000.0,
                                       "downflooding_angle_phi_f_deg": 40.0},
        "not_used": "Table 5 is the second generation dead-ship level 1 check, not IS Code 2.2",
    },
    "cited_values": {"gm_m": 2.167},
    "downflooding_deg": 40.0,
}

FT = 0.3048  # metres per foot, exactly


def metres(feet: float) -> float:
    return float(f"{feet * FT:.12g}")


LCU_SOURCE = {
    "citation": "US Naval Academy, Naval Architecture and Ocean Engineering Department, EN400 Principles "
                "of Ship Performance, Course Notes, Version: Summer 2020, "
                "https://www.usna.edu/NAOE/_files/documents/Courses/EN400/EN400_Course_Notes_Summer_2020.pdf",
    "locator": "Chapter 4 (Stability), end-of-chapter problem 33, printed page 4-51 (PDF page 190 of 484)",
    "licence": "not stated; course notes of a US federal academy, likely a US Government work "
               "(17 U.S.C. 105), not verified. Numbers only: the PDF is not committed",
    "read": "downloaded and read with pdftotext by the builder, 2026-10-16 (the problem statement, "
            "the table of draft, KM and TPI, and the salt water conventions of the notes)",
    "as_printed": {
        "problem": "33: A Navy LCU can be modeled as a rectangular, box-shaped barge",
        "length_ft": 135.0, "beam_ft": 35.0, "depth_ft": 7.0,
        "deck": "located 7 ft above keel; gunwales extend 4 ft above the deck",
        "table_draft_ft_km_ft_tpi_lt_per_in": [[4, 27.5, 11.25], [5, 22.9, 11.25], [6, 20.0, 11.25]],
        "empty": "floats at a draft of 4 ft, KG = 1.9 ft",
        "loaded": "maximum capacity with 3 M-60 tanks; each weighs 60 LT, centre of gravity 4 ft "
                  "above the ground (i.e., 4 feet above the deck)",
        "not_used": "parts (d) to (g): water on deck, free surface and sketches",
    },
    "downflooding_deg": None,
}

LCU_EMPTY = dict(LCU_SOURCE)
LCU_EMPTY["cited_values"] = {
    "km_ft_at_draft_4_ft": 27.5,
    "gm_ft": 25.6,
    "note": "printed KM 27.5 ft at 4 ft; a box gives 2 + 35^2/48 = 27.52 ft, so GM = 27.52 - 1.9 = 25.6 ft",
}

#: The loaded condition, worked from the printed numbers only (all in feet and long tons).
_DELTA0 = 135.0 * 35.0 * 4.0 / 35.0          # 540 LT: the box at 4 ft, 35 ft3 per LT (64 lb/ft3)
_LOAD = 3 * 60.0                             # 180 LT
_T_LOADED_FT = 4.0 + _LOAD / 11.25 / 12.0    # 4 + 16 in = 5.3333 ft
_KG_LOADED_FT = (_DELTA0 * 1.9 + _LOAD * (7.0 + 4.0)) / (_DELTA0 + _LOAD)  # 4.175 ft

LCU_LOADED = dict(LCU_SOURCE)
LCU_LOADED["cited_values"] = {
    "km_ft_at_draft_5_ft": 22.9,
    "km_ft_at_draft_6_ft": 20.0,
    "note": "the loaded draft 5.333 ft lies between the printed drafts 5 ft (KM 22.9) and 6 ft (KM 20.0); "
            "a box gives KM = 5.333/2 + 35^2/(12 x 5.333) = 21.81 ft, inside that bracket (linear "
            "interpolation of the printed table gives 21.93 ft: the curve is convex, so the box value is "
            "a little lower)",
}
LCU_LOADED["worked_from_printed"] = {
    "salt_water_convention": "the notes derive TPI = Awp/420 from 64 lb/ft3 (1.99 lb s2/ft4 x 32.17 ft/s2) "
                             "and 1 LT = 2240 lb, so 35 ft3 per LT; Awp = 135 x 35 = 4725 ft2 gives "
                             "4725/420 = 11.25 LT/in, the printed TPI",
    "displacement_empty_lt": "135 x 35 x 4 ft3 / 35 ft3 per LT = 540 LT",
    "load_lt": "3 x 60 = 180 LT",
    "draft_ft": "4 + 180 LT / 11.25 LT per in = 4 + 16 in = 5.3333 ft (check: 720 LT x 35 / 4725 ft2 = 5.3333 ft)",
    "kg_ft": "KG = (540 x 1.9 + 180 x 11) / 720 = 4.175 ft (the tanks' centre of gravity is 7 + 4 = 11 ft above the keel)",
}

#: A crude oil tank barge, worked from closed forms only (issue #332, spec 0006): a lightship, and rectangular
#: cargo tanks filled alike to part of their height. The draught is the displacement over L x B x rho, the KG
#: the mean of the lightship and the cargo, and the free surface correction rho_cargo x n x l x b^3 / 12 over
#: the displacement, for a slack tank (a tank at 98 % or more is nominally full: IS Code 2008 Part B 3.1.2).
#: Nothing here is typed as a result: the draught, KG and free surface follow from the particulars.
LIQUID_HULL = {"L": 76.5, "B": 11.4, "D": 3.6, "density": 1.0}
LIQUID_LIGHTSHIP = {"mass_t": 330.0, "kg_m": 1.8}
LIQUID_TANK = {"length_m": 22.0, "inner_breadth_m": 9.4, "bottom_m": 0.7, "height_m": 2.9, "n": 3,
               "cargo_density_t_m3": 0.87}


def liquid_particulars(centreline: bool, fill: float) -> dict:
    t = LIQUID_TANK
    n = t["n"] * (2 if centreline else 1)
    breadth = t["inner_breadth_m"] / (2 if centreline else 1)
    volume = n * t["length_m"] * breadth * t["height_m"] * fill
    cargo = volume * t["cargo_density_t_m3"]
    kg_cargo = t["bottom_m"] + t["height_m"] * fill / 2.0
    mass = LIQUID_LIGHTSHIP["mass_t"] + cargo
    h = LIQUID_HULL
    kg = (LIQUID_LIGHTSHIP["mass_t"] * LIQUID_LIGHTSHIP["kg_m"] + cargo * kg_cargo) / mass
    draught = mass / (h["L"] * h["B"] * h["density"])
    slack = fill < 0.98
    fs = (t["cargo_density_t_m3"] * n * t["length_m"] * breadth ** 3 / 12.0 / mass) if slack else 0.0
    return {"L": h["L"], "B": h["B"], "D": h["D"], "T": draught, "KG": kg, "FS": fs, "density": h["density"],
            "liquid": {"tanks": n, "tank_breadth_m": breadth, "fill": fill, "cargo_t": cargo,
                       "displacement_t": mass, "kg_cargo_m": kg_cargo}}


BASE_CASES = [
    {
        "id": "box_a",
        "title": "Box barge A, large margins",
        "L": 100.0, "B": 20.0, "D": 8.0, "T": 4.0, "KG": 6.0,
        "note": "Wide and low: every criterion passes with a large margin.",
    },
    {
        "id": "box_b",
        "title": "Box barge B, small margins",
        "L": 100.0, "B": 14.0, "D": 8.0, "T": 5.0, "KG": 5.5,
        "note": "GM 0.267 m, areas within 0.025 m rad of their limits, angle of maximum GZ about 31.6 degrees.",
    },
    {
        "id": "lcu_empty",
        "title": "Navy LCU modelled as a box barge, empty (USNA EN400 problem 33)",
        "L": metres(135.0), "B": metres(35.0), "D": metres(7.0), "T": metres(4.0), "KG": metres(1.9),
        "note": "Feet converted to metres with 0.3048. The deck is the depth, 7 ft: the gunwales that "
                "extend 4 ft above the deck are not watertight hull and are not modelled. No downflooding "
                "angle is given, so none is set. The notes give no IS Code 2.2 verdict: the verdicts are "
                "derived by the oracle, and are recorded as the oracle gives them.",
        "published": LCU_EMPTY,
        "density": 1.025,
    },
    {
        "id": "lcu_loaded",
        "title": "Navy LCU modelled as a box barge, loaded with 3 M-60 tanks (USNA EN400 problem 33)",
        "L": metres(135.0), "B": metres(35.0), "D": metres(7.0), "T": metres(_T_LOADED_FT),
        "KG": metres(_KG_LOADED_FT),
        "note": "Draft and KG are worked from the printed TPI, weights and centres (see worked_from_printed); "
                "feet converted to metres with 0.3048. The deck is the depth, 7 ft: the gunwales are not "
                "modelled. No downflooding angle is given, so none is set. The verdicts are derived by the "
                "oracle and recorded as it gives them; the inputs are never adjusted to make a verdict pass.",
        "published": LCU_LOADED,
        "density": 1.025,
    },
    {
        "id": "liquid_slack",
        "title": "Crude oil tank barge, cargo tanks half full, no centreline bulkhead",
        **liquid_particulars(False, 0.5),
        "note": "Slack tanks 9.4 m wide: a large free surface correction. The particulars follow from the "
                "closed forms in `write_cases.py` (liquid_particulars); the verdicts are derived by the oracle.",
    },
    {
        "id": "liquid_slack_centreline",
        "title": "Crude oil tank barge, cargo tanks half full, centreline bulkhead",
        **liquid_particulars(True, 0.5),
        "note": "The same hull and cargo as liquid_slack with a centreline bulkhead: the free surface moment "
                "is a quarter. The verdicts are derived by the oracle.",
    },
    {
        "id": "liquid_full",
        "title": "Crude oil tank barge, cargo tanks nominally full",
        **liquid_particulars(True, 0.98),
        "note": "Tanks at 98 %: nominally full, no free surface correction (IS Code 2008 Part B 3.1.2). The "
                "verdicts are derived by the oracle.",
    },
    {
        "id": "generic_barge",
        "title": "Generic barge of Marlantes, Kim and Hurt (2022), section 5.1",
        "L": 100.0, "B": 20.0, "D": 10.0, "T": 5.0, "KG": 7.0,
        "note": "A simple uniform rectangular cross section. The paper gives no IS Code 2.2 verdict: the "
                "verdicts here are derived by the oracle. The published GM (2.167 m) is a cited check.",
        "published": PAPER,
    },
]


def sig(x: float, n: int = 10) -> float:
    return float(f"{x:.{n}g}")


def first_difference(a, b, rel: float = 1e-6, abs_: float = 1e-5, path: str = "") -> str | None:
    """The path and the two values of the first place where `a` and `b` differ as data, or None. Text
    and structure must match exactly; numbers to a relative 1e-6 or an absolute 1e-5. The angle of maximum GZ is
    found by a search on a flat curve, so its last digits depend on the platform's maths library
    (about 1e-6 degree): a generated file is compared with this, not byte for byte."""
    if isinstance(a, dict) and isinstance(b, dict):
        if list(a) != list(b):
            return f"{path}: keys {list(a)[:4]} vs {list(b)[:4]}"
        for k in a:
            d = first_difference(a[k], b[k], rel, abs_, f"{path}/{k}")
            if d:
                return d
        return None
    if isinstance(a, list) and isinstance(b, list):
        if len(a) != len(b):
            return f"{path}: length {len(a)} vs {len(b)}"
        for i, (x, y) in enumerate(zip(a, b)):
            d = first_difference(x, y, rel, abs_, f"{path}[{i}]")
            if d:
                return d
        return None
    if isinstance(a, bool) or isinstance(b, bool):
        return None if a is b else f"{path}: {a!r} vs {b!r}"
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        return None if abs(a - b) <= max(abs_, rel * max(abs(a), abs(b))) else f"{path}: {a!r} vs {b!r}"
    return None if a == b else f"{path}: {a!r} vs {b!r}"


def same(a, b, rel: float = 1e-6, abs_: float = 1e-5) -> bool:
    return first_difference(a, b, rel, abs_) is None


def case_document(c: dict) -> dict:
    r = oracle.assess(c["B"], c["D"], c["T"], c["KG"], c.get("FS", 0.0))
    expected = {}
    for name in oracle.CRITERIA:
        rule, what, unit = RULES[name]
        e = r[name]
        expected[RULE_IDS[name]] = {
            "rule": rule, "what": what, "unit": unit,
            "verdict": "pass" if e["ok"] else "fail",
            "value": sig(e["value"]), "limit": e["limit"], "margin": sig(e["margin"]),
        }
    published = c.get("published")
    prov = {
        "verdict_basis": "derived",
        "inputs_basis": "chosen for this corpus (not from a publication)",
        "source": "evaluation/oracle/box_oracle.py (exact polygon GZ, Simpson areas) and "
                  + SOURCE_RULES,
        "edition": "2008 IS Code, MSC.267(85)",
        "locator": "Part A 2.2.1 to 2.2.4, printed pages 11 and 12",
        "licence": "inputs and oracle: this repository (see LICENSE); the limits are quoted from the IMO text",
        "confidence": "high for the verdicts (the oracle is checked against closed forms); the case is not an external published example",
        "note": c["note"],
    }
    if c.get("liquid"):
        prov["inputs_basis"] = ("worked from closed forms for a crude oil tank barge (the particulars of "
                                "`liquid`); not from a publication")
        prov["liquid"] = {k: sig(v) for k, v in c["liquid"].items()}
        prov["liquid"].update({"lightship_t": LIQUID_LIGHTSHIP["mass_t"], "lightship_kg_m": LIQUID_LIGHTSHIP["kg_m"],
                               **{f"tank_{k}": v for k, v in LIQUID_TANK.items()}})
    if published:
        prov["inputs_basis"] = "published"
        prov["inputs_source"] = {k: published[k] for k in ("citation", "locator", "licence", "read", "as_printed")}
        prov["cited_values"] = published["cited_values"]
        if "worked_from_printed" in published:
            prov["worked_from_printed"] = published["worked_from_printed"]
        prov["licence"] = ("inputs: " + published["licence"] + "; oracle: this repository (see LICENSE); "
                           "the limits are quoted from the IMO text")
        prov["confidence"] = ("high for the inputs (read from the source as printed); high for the verdicts "
                              "(derived by the oracle, which is checked against closed forms). The source gives no "
                              "IS Code 2.2 verdict, so `verdict_basis` is derived. The cited values are checks "
                              "that the oracle and the engine reproduce")
    return {
        "id": c["id"],
        "title": c["title"],
        "kind": "box",
        "inputs": {
            "form": "particulars",
            "vdesign": None,
            "length_m": c["L"], "beam_m": c["B"], "depth_m": c["D"], "draught_m": c["T"],
            "kg_m": c["KG"], "water_density_t_m3": c.get("density", 1.0),
            "free_surface_m": sig(c.get("FS", 0.0)),
            **({"downflooding_angle_deg": published["downflooding_deg"]} if published and published.get("downflooding_deg") else {}),
        },
        "expected": expected,
        "provenance": prov,
    }


def render(doc: dict) -> str:
    return "# Generated by evaluation/oracle/write_cases.py. Do not edit by hand.\n" + yaml.safe_dump(
        doc, sort_keys=False, allow_unicode=True, width=100)


def main(argv: list[str]) -> int:
    check = "--check" in argv
    bad = 0
    for c in BASE_CASES:
        path = CASES_DIR / f"{c['id']}.yaml"
        text = render(case_document(c))
        if check:
            diff = "missing" if not path.exists() else first_difference(
                yaml.safe_load(path.read_text(encoding="utf-8")), yaml.safe_load(text))
            if diff:
                print(f"write_cases: {path} differs from the oracle at {diff}", file=sys.stderr)
                bad += 1
        else:
            path.write_text(text, encoding="utf-8")
            print(f"wrote {path}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
