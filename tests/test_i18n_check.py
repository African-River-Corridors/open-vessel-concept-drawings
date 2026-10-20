"""The Text Library check (spec 0005): the real library is whole, and each kind of problem is caught."""
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import i18n_check  # noqa: E402


def test_the_text_library_is_whole_and_the_check_is_fast():
    """Label: property"""
    start = time.perf_counter()
    assert i18n_check.check() == []
    assert time.perf_counter() - start < 2.0


def tree(tmp_path: Path, english: str, code: str, **translations: str) -> Path:
    (tmp_path / "locales" / "en").mkdir(parents=True)
    (tmp_path / "locales" / "en" / "app.ftl").write_text(english, encoding="utf-8")
    for tag, text in translations.items():
        (tmp_path / "locales" / tag).mkdir()
        (tmp_path / "locales" / tag / "app.ftl").write_text(text, encoding="utf-8")
    (tmp_path / "app" / "src").mkdir(parents=True)
    (tmp_path / "app" / "src" / "a.ts").write_text(code, encoding="utf-8")
    (tmp_path / "crates").mkdir()
    return tmp_path


EN = "app-hello = Hello { $name }\napp-bye = Bye\n"
CODE = 't("app-hello", { name }); t("app-bye");'


def test_a_clean_tree_passes(tmp_path):
    """Label: property"""
    root = tree(tmp_path, EN, CODE, es="# status: draft\napp-hello = Hola { $name }\n")
    assert i18n_check.check(root) == []


def test_missing_and_unused_keys(tmp_path):
    """Label: property"""
    root = tree(tmp_path, EN, 't("app-hello", { name }); t("app-gone");')
    problems = i18n_check.check(root)
    assert any(p.startswith("missing: app-gone") for p in problems)
    assert any(p.startswith("unused: app-bye") for p in problems)


def test_a_built_prefix_counts_as_used(tmp_path):
    """Label: property"""
    root = tree(tmp_path, EN + "app-item-a = A\n", CODE + " t(`app-item-${id}`);")
    assert i18n_check.check(root) == []


def test_placeholder_mismatch_orphan_and_missing_status(tmp_path):
    """Label: property"""
    es = "# status: draft\napp-hello = Hola { $nombre }\napp-bye = Adiós\n# status: draft\napp-extra = Más\n"
    problems = i18n_check.check(tree(tmp_path, EN, CODE, es=es))
    assert any("placeholders of app-hello" in p for p in problems)
    assert any("app-bye has no status line" in p for p in problems)
    assert any("orphan: app-extra" in p for p in problems)


def test_a_review_lapses_when_the_english_changes_and_fix_sets_it_to_draft(tmp_path):
    """Label: property"""
    good = i18n_check.english_hash("app-bye = Bye")
    es = f"# status: reviewed @someone {good}\napp-bye = Adiós\n# status: reviewed @someone 00000000\napp-hello = Hola {{ $name }}\n"
    root = tree(tmp_path, EN, CODE, es=es)
    problems = i18n_check.check(root)
    assert len(problems) == 1 and "lapsed review" in problems[0] and "app-hello" in problems[0]
    assert i18n_check.check(root, fix=True) == []
    text = (root / "locales" / "es" / "app.ftl").read_text(encoding="utf-8")
    assert f"# status: reviewed @someone {good}\napp-bye" in text
    assert "# status: draft\napp-hello" in text


def test_a_multiline_message_keeps_its_placeholders(tmp_path):
    """Label: property"""
    en = "app-n = { $n ->\n        [one] one { $what }\n       *[other] { $n } { $what }\n    }\n"
    es = "# status: draft\napp-n = { $n ->\n       *[other] { $n } { $what }\n    }\n"
    assert i18n_check.check(tree(tmp_path, en, 't("app-n")', es=es)) == []


def test_a_stale_engine_file_is_caught(tmp_path):
    """Label: property"""
    root = tree(tmp_path, EN, CODE)
    (root / "crates" / "a.rs").write_text('let t = text!("check-low", "too low: {x} m", x = Arg::py(x));\n', encoding="utf-8")
    problems = i18n_check.check(root)
    assert any("engine.ftl is not what the code gives" in p for p in problems)


def test_a_stale_input_label_is_caught(tmp_path):
    """Label: property"""
    root = tree(tmp_path, EN, CODE)
    src = root / "crates" / "ovcd-format" / "src"
    src.mkdir(parents=True)
    (src / "vdesign.rs").write_text('pub const INPUTS: &[InputSpec] = &[\n    spec("beam_m", "Beam", InputKind::Number, true, Group::Required),\n];\n', encoding="utf-8")
    (root / "locales" / "en" / "inputs.ftl").write_text("input-beam-m = Breadth\ninput-old = Old\n", encoding="utf-8")
    problems = i18n_check.check(root)
    assert any("input-beam-m is 'Breadth', the code says 'Beam'" in p for p in problems)
    assert any("input-old is not an input" in p for p in problems)
