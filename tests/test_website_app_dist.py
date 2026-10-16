"""Website build: --app-dist puts the web app where the ARC proxy needs it (issue #247).

Label: property

The page goes to app.html (served at /ovcd/app) and its files to app-files/. A folder named app/
would make GitHub Pages redirect /app to /app/ on its own host, which breaks the proxy.
No mkdocs, Node or Rust: it calls the copy step on a fake dist folder.
"""
import importlib.util
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]


@pytest.fixture
def build(tmp_path, monkeypatch):
    spec = importlib.util.spec_from_file_location("website_build", ROOT / "website" / "build.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    site = tmp_path / "_site"
    site.mkdir()
    (site / "app.html").write_text("coming soon")
    monkeypatch.setattr(mod, "SITE", site)
    return mod, site


def fake_dist(root: Path) -> Path:
    dist = root / "dist-web"
    (dist / "assets").mkdir(parents=True)
    (dist / "index.html").write_text('<script src="/ovcd/app-files/assets/x.js"></script>')
    (dist / "assets" / "x.js").write_text("js")
    (dist / "assets" / "core_bg.wasm").write_bytes(b"\0asm")
    return dist


def test_app_dist_lands_at_app_html_and_app_files(build, tmp_path):
    """Label: property"""
    mod, site = build
    mod.add_app(fake_dist(tmp_path))
    assert "app-files/assets/x.js" in (site / "app.html").read_text()
    assert (site / "app-files" / "assets" / "x.js").read_text() == "js"
    assert (site / "app-files" / "assets" / "core_bg.wasm").read_bytes() == b"\0asm"
    assert not (site / "app-files" / "index.html").exists()
    assert not (site / "app").exists()


def test_missing_index_fails_clearly(build, tmp_path, capsys):
    """Label: property"""
    mod, site = build
    (tmp_path / "empty").mkdir()
    msg = mod.main(["--app-dist", str(tmp_path / "empty")])
    assert isinstance(msg, str) and "index.html" in msg
    assert (site / "app.html").read_text() == "coming soon"
