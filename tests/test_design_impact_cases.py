"""What the design-impact case set reaches, and how changes are compared (ticket 131)."""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import design_impact as di  # noqa: E402


def used(key: str) -> set:
    return {v[key] for v in di.cases().values() if key in v}


def test_every_cargo_in_the_database_is_in_the_case_set():
    src = (ROOT / "crates/ovcd-core/src/cargo.rs").read_text(encoding="utf-8")
    keys = set(re.findall(r'^        "([A-Za-z_]+)",$', src, re.M))
    assert {"LNG", "LNG_ethane", "LPG_propane", "LPG_butane", "LPG_mix", "bauxite"} <= keys
    assert keys <= used("cargo"), keys - used("cargo")


def test_every_input_of_an_option_is_in_the_case_set():
    src = (ROOT / "crates/ovcd-format/src/vdesign.rs").read_text(encoding="utf-8")
    inputs = set(re.findall(r'(?:spec|only)\(\s*"([a-z0-9_]+)"', src))
    assert len(inputs) >= 20
    in_cases = set().union(*(v.keys() for v in di.cases().values()))
    assert inputs <= in_cases, inputs - in_cases


def test_the_optional_inputs_the_review_found_missing_are_covered():
    names = {"channel_current_kn", "air_draft_limit_m", "channel_bank_slope", "channel_under_keel_clearance_m",
             "fill_ratio", "max_tank_od_m", "block_coefficient", "min_freeboard_m",
             "propulsive_efficiency", "lightship_factor_t_per_m3"}
    covered = {n[0] for n in di.OPTIONAL_INPUT_CASES}
    assert names <= covered
    for name in names:
        case = di.cases()[f"sweep optional {name}"]
        assert name in case


def test_the_case_set_has_the_shapes_the_review_found_missing():
    cs = di.cases()
    assert any(v["cargo"] == "bauxite" and "channel_depth_m" in v for v in cs.values())          # dry, channel
    assert any(v["cargo_containment"] == "sloped_hopper_hold" and v["formation"]["kind"] == "pushed_barge"
               for v in cs.values())                                                              # pushed hopper
    assert any(v["cargo_containment"] == "box_hold" and v["formation"]["kind"] == "self_propelled"
               for v in cs.values())                                                              # self-propelled box
    assert any(k.startswith("sweep gas passes all checks") for k in cs)
    assert any(k.startswith("sweep gas carries nothing") for k in cs)


def test_the_cn_derived_cases_are_not_called_calibration():
    names = list(di.cases())
    assert not any(n.startswith("calibration CN") for n in names)
    assert sum(n.startswith("sweep CN-derived") for n in names) == 6


def test_the_depth_stand_in_is_stated_in_both_reports():
    for rows in ([], [{"case": "a", "quantity": "q", "before": 1.0, "after": 2.0, "change": "+100 %"}]):
        text, _ = di.report_text(3, rows, "main abc", "this branch def")
        assert "stand-in" in text and f"{di.CN_DEPTH_ABOVE_DRAUGHT_M:g} m" in text


def test_the_set_is_large_enough():
    cs = di.cases()
    assert len(cs) >= 60


# ---------- comparing ----------

def test_a_change_between_int_and_float_is_a_change():
    assert not di.same(90, 90.0) and not di.same(90.0, 90)
    assert di.same(90, 90) and di.same(90.0, 90.0 + 1e-12)
    rows = di.compare({"a": {"results": {"n": 90}}}, {"a": {"results": {"n": 90.0}}})
    assert [(r["quantity"], r["change"]) for r in rows] == [("n", "int to float")]


def test_a_1e_6_move_is_visible_in_the_table():
    rows = di.compare({"a": {"results": {"x": 1.13599}}}, {"a": {"results": {"x": 1.13599 * (1 + 1e-6)}}})
    assert len(rows) == 1
    before, after = di.show(rows[0]["before"]), di.show(rows[0]["after"])
    assert before != after, (before, after)


def test_a_base_that_refuses_a_new_input_is_run_without_it(tmp_path):
    """The base binary of a design-impact run may predate an input (issue #4). `run_case` leaves out
    an input the binary refuses by name, so the case still runs on it."""
    fake = tmp_path / "ovcd"
    fake.write_text(
        '#!/bin/sh\n'
        'if [ "$1" = fill ]; then\n'
        '  if [ "$2" = --as-new ]; then shift; fi\n'
        '  if grep -q n_holds "$2"; then echo \'Option x: unknown input `n_holds`\' >&2; exit 1; fi\n'
        '  cp "$2" "$3"; exit 0\n'
        'fi\n'
        'echo \'{"options":[{"results":{"seen":"ok"}}]}\'\n',
        encoding="utf-8")
    fake.chmod(0o755)
    out = di.run_case(fake, "case", {"cargo": "bauxite", "n_holds": 3}, tmp_path)
    assert out == {"results": {"seen": "ok"}}


def test_a_base_from_before_the_as_new_flag_is_filled_without_it(tmp_path):
    """`fill --as-new` (issue #292) gives a file made now the Defaults Profile value of an input added
    after the first files. A base that does not know the flag refuses it by name; `fill_file` then
    fills the way that base always did, and a head that knows it is given the flag.

    Label: property
    """
    log = tmp_path / "calls.txt"
    fake = tmp_path / "ovcd"
    fake.write_text(
        '#!/bin/sh\n'
        f'echo "$@" >> {log}\n'
        'if [ "$1" = fill ]; then\n'
        '  if [ "$2" = --as-new ] && [ -e "$(dirname "$0")/old" ]; then\n'
        '    echo "error: unknown option `--as-new`" >&2; exit 1; fi\n'
        '  if [ "$2" = --as-new ]; then shift; fi\n'
        '  cp "$2" "$3"; exit 0\n'
        'fi\n'
        'echo \'{"options":[{"results":{"seen":"ok"}}]}\'\n',
        encoding="utf-8")
    fake.chmod(0o755)
    case = {"cargo": "bauxite"}
    assert di.run_case(fake, "case", case, tmp_path) == {"results": {"seen": "ok"}}
    assert "fill --as-new" in log.read_text(encoding="utf-8")
    log.unlink()
    (tmp_path / "old").write_text("")
    assert di.run_case(fake, "case", case, tmp_path) == {"results": {"seen": "ok"}}
    calls = log.read_text(encoding="utf-8").splitlines()
    assert any(c.startswith("fill --as-new") for c in calls) and any(c.startswith("fill ") and "--as-new" not in c for c in calls)
