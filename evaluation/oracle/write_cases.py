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
}

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
    r = oracle.assess(c["B"], c["D"], c["T"], c["KG"], 0.0)
    expected = {}
    for name in oracle.CRITERIA:
        rule, what, unit = RULES[name]
        e = r[name]
        expected[name] = {
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
    if published:
        prov["inputs_basis"] = "published"
        prov["inputs_source"] = {k: published[k] for k in ("citation", "locator", "licence", "read", "as_printed")}
        prov["cited_values"] = published["cited_values"]
        prov["licence"] = ("inputs: " + published["licence"] + "; oracle: this repository (see LICENSE); "
                           "the limits are quoted from the IMO text")
        prov["confidence"] = ("high for the inputs (read from Tables 3 and 4 as printed); high for the verdicts "
                              "(derived by the oracle, which is checked against closed forms). The paper gives no "
                              "IS Code 2.2 verdict, so `verdict_basis` is derived. The published GM is a cited "
                              "check that the oracle and the engine reproduce")
    return {
        "id": c["id"],
        "title": c["title"],
        "kind": "box",
        "inputs": {
            "form": "particulars",
            "vdesign": None,
            "length_m": c["L"], "beam_m": c["B"], "depth_m": c["D"], "draught_m": c["T"],
            "kg_m": c["KG"], "water_density_t_m3": 1.0, "free_surface_m": 0.0,
            **({"downflooding_angle_deg": 40.0} if published else {}),
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
