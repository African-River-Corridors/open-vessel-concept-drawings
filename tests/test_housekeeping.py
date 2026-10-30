"""Housekeeping contract (ticket 6): glossary, licence holder, repo name, ADRs, roadmap, no web simulator."""
from pathlib import Path
import tomllib

ROOT = Path(__file__).resolve().parents[1]
HOLDER = "African River Corridors Limited"
REPO = "open-vessel-designer"


def read(rel: str) -> str:
    return (ROOT / rel).read_text(encoding="utf-8")


def test_glossary_defines_the_terms():
    assert not (ROOT / "CONTEXT.md").exists()
    text = read("GLOSSARY.md")
    for term in ("Cargo Containment", "Hull Form", "Design", "Option", "Defaults Profile", "Calibration Band"):
        assert f"**{term}**:" in text, term


def test_agent_files_and_spec_present():
    for rel in ("AGENTS.md", "docs/agents/build-process.md", "docs/specs/0001-parity-harness-and-rust-core.md"):
        assert (ROOT / rel).is_file(), rel


def test_licence_holder():
    assert HOLDER in read("NOTICE")
    assert HOLDER in read("README.md")
    authors = tomllib.loads(read("pyproject.toml"))["project"]["authors"]
    assert [a["name"] for a in authors] == [HOLDER]
    assert "Mobile Clipboard" not in read("NOTICE") + read("README.md") + read("pyproject.toml")


def test_repo_name_in_urls():
    for rel in ("README.md", "CONTRIBUTING.md", "ROADMAP.md", "pyproject.toml"):
        text = read(rel)
        assert "African-River-Corridors/ovd" not in text, rel
    assert f"African-River-Corridors/{REPO}" in read("pyproject.toml")
    assert f"African-River-Corridors/{REPO}" in read("ROADMAP.md")


def test_adrs_0005_to_0010():
    names = {p.name[:4]: p for p in (ROOT / "docs/adr").glob("*.md")}
    for n in ("0005", "0006", "0007", "0008", "0009", "0010"):
        assert n in names, n
    assert "Supersedes ADR 0003" in names["0009"].read_text(encoding="utf-8")
    assert "Superseded by ADR 0009" in names["0003"].read_text(encoding="utf-8")


def test_roadmap_has_ten_step_plan():
    text = read("ROADMAP.md")
    for n in range(1, 11):
        assert f"\n{n}. " in text, n
    for key in ("Calibration Bands", "moulded-depth", "Formations", "Tauri", "WebAssembly", "installer"):
        assert key in text, key


def test_web_simulator_removed():
    assert not (ROOT / "web").exists()
    assert not (ROOT / "tools/build_web.py").exists()
    readme = read("README.md").lower()
    assert "web/" not in readme and "pyodide" not in readme and "build_web" not in readme


def test_embargo_list_stays():
    assert (ROOT / "tools/embargo.py").is_file()
    assert (ROOT / "tools/release_guard.py").is_file()
