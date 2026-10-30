"""The Text Library check (spec 0005): keeps the Fluent files whole, fast, with the standard library only.

    python3 tools/i18n_check.py          # report; exit 1 on any problem
    python3 tools/i18n_check.py --fix    # also set lapsed reviews back to draft
    python3 tools/i18n_check.py --hashes es   # print the "reviewed" status line of each English message

It checks:
  - every id the code uses is in English (missing keys);
  - every English id is used by the code (unused keys), except the derived ids (`input-`, `choice-`,
    `group-`, and any prefix the code builds with a template string), which a Rust test checks;
  - every translated id is in English (orphans), with the same placeholders (placeholder mismatch);
  - every translated message has a status line, `# status: draft` or
    `# status: reviewed @handle <hash>`, and a reviewed one still matches the English it was
    reviewed against (the hash is the start of the SHA-256 of the English message source).

Drawing messages (`drawing.ftl`) are English only for now (the Drawing Format Standard allows ASCII
only), so a translation may not have them.
"""
from __future__ import annotations

import hashlib
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = "en"
ENGLISH_ONLY = {"drawing.ftl"}
DERIVED = ("input-", "choice-", "group-")
CODE = [("app/src", (".ts", ".svelte")), ("app/index.html", ()), ("crates", (".rs",))]
# In Rust only the id of a `text!` call counts: other quoted kebab strings there are not Message Ids.
RUST_USED = re.compile(r'\b(?:text!|label!|Text::fixed)\(\s*"([a-z0-9-]+)"')

MESSAGE = re.compile(r"^([a-zA-Z][a-zA-Z0-9_-]*)\s*=(.*)$")
PLACEHOLDER = re.compile(r"\{\s*\$([a-zA-Z][a-zA-Z0-9_-]*)")
STATUS = re.compile(r"^# status: (?:(draft)|reviewed @([A-Za-z0-9-]+) ([0-9a-f]{8}))\s*$")
# A Message Id in code: a quoted literal of an id with a known prefix, or the head of a template string.
USED = re.compile(r"""["'`]((?:app|check|warn|stab|name|msg|draw|err)-[a-z0-9-]+)["'`]""")
BUILT = re.compile(r"`((?:app|check|warn|stab|name|msg|draw|err|input|choice|group)-[a-z0-9-]*)\$\{")


@dataclass
class Message:
    id: str
    source: str
    placeholders: set[str]
    status: str | None = None
    line: int = 0


@dataclass
class File:
    path: Path
    messages: dict[str, Message] = field(default_factory=dict)
    problems: list[str] = field(default_factory=list)


def parse(path: Path) -> File:
    """Read the messages of one Fluent file: id, source lines, placeholders, and the status comment above."""
    out = File(path)
    lines = path.read_text(encoding="utf-8").splitlines()
    comment: str | None = None
    current: Message | None = None
    for n, line in enumerate(lines, 1):
        m = MESSAGE.match(line)
        if m:
            current = Message(m.group(1), line.rstrip(), set(), comment, n)
            if current.id in out.messages:
                out.problems.append(f"{path.relative_to(ROOT).as_posix()}:{n}: duplicate id {current.id}")
            out.messages[current.id] = current
            comment = None
        elif line.startswith((" ", "\t")) and current and line.strip():
            current.source += "\n" + line.rstrip()
        else:
            current = None
            if line.startswith("# status:"):
                comment = line.rstrip()
            elif line.strip() and not line.startswith("#"):
                out.problems.append(f"{path.relative_to(ROOT).as_posix()}:{n}: not a message or a comment: {line[:60]}")
    for msg in out.messages.values():
        msg.placeholders = set(PLACEHOLDER.findall(msg.source.split("=", 1)[1]))
    return out


def english_hash(source: str) -> str:
    return hashlib.sha256(source.encode("utf-8")).hexdigest()[:8]


def code_ids(root: Path) -> tuple[set[str], set[str]]:
    """The ids the code names, and the prefixes it builds ids from."""
    used: set[str] = set()
    built: set[str] = set()
    for rel, suffixes in CODE:
        base = root / rel
        files = [base] if base.is_file() else [p for p in base.rglob("*") if p.suffix in suffixes and "node_modules" not in p.parts and "target" not in p.parts]
        for p in files:
            text = p.read_text(encoding="utf-8", errors="replace")
            if p.suffix == ".rs":
                if p.name != "text.rs":  # the macro's own example
                    used.update(RUST_USED.findall(text))
                continue
            used.update(USED.findall(text))
            used.update(re.findall(r'data-i18n(?:-aria)?="([a-z0-9-]+)"', text))
            built.update(BUILT.findall(text))
    if root == ROOT:  # the pinned otd crate's `label!` calls are code too (tools/otd.py)
        import otd

        for p in (otd.root() / "src").rglob("*.rs"):
            used.update(RUST_USED.findall(p.read_text(encoding="utf-8")))
    return used, built


def check(root: Path = ROOT, fix: bool = False) -> list[str]:
    locales = root / "locales"
    problems: list[str] = []
    english: dict[str, Message] = {}
    english_files: dict[str, File] = {}
    for p in sorted((locales / SOURCE).glob("*.ftl")):
        f = parse(p)
        problems += f.problems
        english_files[p.name] = f
        for k, v in f.messages.items():
            if k in english:
                problems.append(f"{p.relative_to(root).as_posix()}:{v.line}: id {k} is also in another English file")
            english[k] = v

    import i18n_extract  # the engine's English comes from the Rust source; engine.ftl must match it

    problems += i18n_extract.problems(root)
    problems += check_input_labels(root, english)
    used, built = code_ids(root)
    for k in sorted(used - english.keys()):
        problems.append(f"missing: {k} is used in the code but not in locales/{SOURCE}")
    skip = DERIVED + tuple(built)
    for k in sorted(english.keys() - used):
        if not k.startswith(skip):
            problems.append(f"unused: {k} is in locales/{SOURCE} but the code does not use it")

    for folder in sorted(d for d in locales.iterdir() if d.is_dir() and d.name != SOURCE):
        for p in sorted(folder.glob("*.ftl")):
            rel = p.relative_to(root).as_posix()
            if p.name in ENGLISH_ONLY:
                problems.append(f"{rel}: drawing messages are English only for now")
                continue
            f = parse(p)
            problems += f.problems
            lapsed: list[Message] = []
            for k, msg in f.messages.items():
                en = english.get(k)
                if en is None:
                    problems.append(f"{rel}:{msg.line}: orphan: {k} is not in locales/{SOURCE}")
                    continue
                if msg.placeholders != en.placeholders:
                    problems.append(f"{rel}:{msg.line}: placeholders of {k}: {sorted(msg.placeholders)}, English has {sorted(en.placeholders)}")
                s = STATUS.match(msg.status or "")
                if not s:
                    problems.append(f"{rel}:{msg.line}: {k} has no status line (# status: draft)")
                elif s.group(3) and s.group(3) != english_hash(en.source):
                    lapsed.append(msg)
            if fix and lapsed:
                text = p.read_text(encoding="utf-8").splitlines(keepends=True)
                for msg in lapsed:
                    text[msg.line - 2] = "# status: draft\n"
                p.write_text("".join(text), encoding="utf-8")
            elif lapsed:
                problems += [f"{rel}:{m.line}: lapsed review: the English of {m.id} changed (run with --fix)" for m in lapsed]
    languages = sorted(d.name for d in locales.iterdir() if d.is_dir() and d.name != SOURCE)
    problems += check_terms(root, languages)
    return problems


# The input table: each `spec(…)`-style constructor call gives an input's key and its English label.
INPUT_SPEC = re.compile(r'\b(?:spec|only|only_in|pushed|self_propelled|lpg|tug|atb)\(\s*"([a-z0-9_]+)",\s*"((?:[^"\\]|\\.)*)"')


def check_input_labels(root: Path, english: dict[str, "Message"]) -> list[str]:
    """`inputs.ftl` holds each input of the input table with its label: a stale file fails here, fast.
    The Rust test `crates/ovd-format/tests/i18n_inputs.rs` checks the rest (choices, groups, notes)."""
    path = root / "crates/ovd-format/src/vdesign.rs"
    if not path.exists():
        return []
    src = path.read_text(encoding="utf-8")
    start = src.find("pub const INPUTS")
    if start < 0:
        return ["crates/ovd-format/src/vdesign.rs: no input table (`pub const INPUTS`) found"]
    table = {f"input-{k.replace('_', '-')}": label for k, label in INPUT_SPEC.findall(src[start:src.index("];", start)])}
    rerun = "run OVD_WRITE_TEXT=1 cargo test -p ovd-format --test i18n_inputs"
    out = []
    for k, label in table.items():
        value = english[k].source.split("=", 1)[1].strip() if k in english else None
        if value is None:
            out.append(f"locales/en/inputs.ftl is stale: {k} is not in it ({rerun})")
        elif value != label:
            out.append(f"locales/en/inputs.ftl is stale: {k} is {value!r}, the code says {label!r} ({rerun})")
    for k in english:
        if k.startswith("input-") and not k.endswith("-default-by") and k not in table:
            out.append(f"locales/en/inputs.ftl is stale: {k} is not an input ({rerun})")
    return out


TERM_STATUSES = {"draft", "reviewed", "needs-native"}


def check_terms(root: Path, languages: list[str]) -> list[str]:
    """The Term List's shape: each term has its English and, per language, [term, status, note]."""
    path = root / "locales" / "terms.toml"
    if not path.exists():
        return []
    import tomllib

    try:
        terms = tomllib.loads(path.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as e:
        return [f"locales/terms.toml: {e}"]
    out: list[str] = []
    for key, entry in terms.items():
        if not isinstance(entry, dict) or not isinstance(entry.get("en"), str):
            out.append(f"locales/terms.toml: [{key}] needs an `en` term")
            continue
        for tag in languages:
            v = entry.get(tag)
            if not (isinstance(v, list) and len(v) == 3 and all(isinstance(x, str) for x in v) and v[0] and v[1] in TERM_STATUSES):
                out.append(f"locales/terms.toml: [{key}] {tag} must be [term, status, note], status one of {sorted(TERM_STATUSES)}")
    return out


def hashes(tag: str, root: Path = ROOT) -> list[str]:
    """For a reviewer: the status line that marks each English message reviewed, by id."""
    out = []
    for p in sorted((root / "locales" / SOURCE).glob("*.ftl")):
        if p.name in ENGLISH_ONLY:
            continue
        for k, msg in parse(p).messages.items():
            out.append(f"{k}: # status: reviewed @your-handle {english_hash(msg.source)}")
    return out


def main(argv: list[str]) -> int:
    if "--hashes" in argv:
        print("\n".join(hashes(argv[argv.index("--hashes") + 1] if len(argv) > argv.index("--hashes") + 1 else "")))
        return 0
    problems = check(fix="--fix" in argv)
    for p in problems:
        print(p)
    print(f"i18n check: {len(problems)} problem(s)")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
