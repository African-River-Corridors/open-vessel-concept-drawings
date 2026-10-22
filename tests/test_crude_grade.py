"""The Crude Grade data and the Cargo database agree between Python and the Rust core (issue #329).

Label: property
"""
import dataclasses

from vessel_designer import _bridge
from vessel_designer.core import cargo


def _rust(fn):
    out = _bridge.raw("cargo", fn, {})
    assert "ok" in out, out
    return out["ok"]


def test_the_python_cargo_database_equals_the_rust_one_in_order():
    """Label: property"""
    rust = _rust("cargo_database")
    assert [r["key"] for r in rust] == list(cargo.CARGO_DATABASE)
    for row in rust:
        py = dataclasses.asdict(cargo.CARGO_DATABASE[row["key"]])
        assert py == row["cargo"], row["key"]


def test_the_python_crude_grades_equal_the_rust_ones():
    """Label: property"""
    rust = _rust("crude_grades")
    assert [r["key"] for r in rust] == list(cargo.CRUDE_GRADES)
    for row in rust:
        py = dataclasses.asdict(cargo.CRUDE_GRADES[row["key"]])
        assert py == row["grade"], row["key"]


def test_a_liquid_bulk_cargo_is_in_development_and_every_other_cargo_is_released():
    """Label: property"""
    for key, props in cargo.CARGO_DATABASE.items():
        assert props.status == ("development" if props.phase == "liquid_bulk" else "released"), key


def test_every_grade_value_has_a_status_and_a_basis():
    """Label: property"""
    for grade in cargo.CRUDE_GRADES.values():
        for f in dataclasses.fields(grade):
            v = getattr(grade, f.name)
            if isinstance(v, cargo.Graded):
                assert v.status in ("sourced", "estimate"), f.name
                assert len(v.basis) > 10, f.name


def test_a_grade_belongs_to_a_liquid_bulk_cargo():
    """Label: property"""
    for key in cargo.CRUDE_GRADES:
        assert cargo.CARGO_DATABASE[key].phase == "liquid_bulk"
