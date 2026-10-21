"""Write `locales/en/engine.ftl` and `drawing.ftl` from the engine's `text!` calls (spec 0005).

    python3 tools/i18n_extract.py           # write the file
    python3 tools/i18n_extract.py --check   # exit 1 if the file is not what the code gives

The Rust source is the one home of the engine's English: each `text!("id", "English {arg} …", …)`
call and each `Text::fixed("id", "English" | CONSTANT)` gives one Fluent message, with `{arg}` written
as `{ $arg }`. So the English in the Text Library cannot drift from the English the engine writes.
`tools/i18n_check.py` runs the check form.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
HEADER = """\
# The engine's messages: checks, warnings, stability notes and names (spec 0005, ADR 0015).
# Written by tools/i18n_extract.py from the Rust source. Do not edit: change the English in the
# `text!` call and run the tool again. A number argument arrives as text, already rounded.
"""

# `label!` is the drawing crates' form of `text!`: the same id and English, the English only used.
CALL = re.compile(r'\b(?:text|label)!\(\s*"([a-z0-9-]+)"\s*,\s*')
FIXED = re.compile(r'\bText::fixed\(\s*"([a-z0-9-]+)"\s*,\s*')
CONST = re.compile(r'\bconst\s+([A-Z_][A-Z0-9_]*)\s*:\s*&(?:\'static\s+)?str\s*=\s*')
NAME = re.compile(r"\{([a-z_][a-z0-9_]*)\}")
# A check's name is a key in the results, so it stays as the engine writes it. The app names a check
# in another language by the derived id `check-name-<slug>`; its English here is the name as read.
CHECK_NAME = re.compile(r'\bCheck::(?:new|with_severity)\(\s*')


def slug(text: str) -> str:
    """A Message Id piece from a name, as the app's `slug` makes it."""
    return re.sub(r"^-+|-+$", "", re.sub(r"[^a-z0-9]+", "-", text.lower()))


class ParseError(Exception):
    pass


def string_literal(text: str, at: int) -> tuple[str, int]:
    """The value of the Rust string literal at `at` (plain or raw), and the index after it."""
    raw = re.match(r'r(#*)"', text[at:])
    if raw:
        close = '"' + raw.group(1)
        start = at + raw.end()
        end = text.index(close, start)
        return text[start:end], end + len(close)
    if text[at] != '"':
        raise ParseError(f"expected a string literal at {text[at:at + 40]!r}")
    out: list[str] = []
    i = at + 1
    while True:
        c = text[i]
        if c == '"':
            return "".join(out), i + 1
        if c == "\\":
            n = text[i + 1]
            if n == "\n":  # a line continuation: the newline and the next line's indent are dropped
                i += 2
                while text[i] in " \t\n":
                    i += 1
                continue
            simple = {"n": "\n", "t": "\t", "\\": "\\", '"': '"', "'": "'", "0": "\0"}
            if n in simple:
                out.append(simple[n])
                i += 2
                continue
            m = re.match(r"u\{([0-9a-fA-F]+)\}", text[i + 1:])
            if m:
                out.append(chr(int(m.group(1), 16)))
                i += 1 + m.end()
                continue
            raise ParseError(f"unsupported escape \\{n}")
        out.append(c)
        i += 1


def fluent_value(english: str, file: str) -> tuple[str, list[str]]:
    """The Fluent pattern for an English format string: `{a}` becomes `{ $a }`, `{{` and `}}` braces."""
    names: list[str] = []
    out: list[str] = []
    i = 0
    while i < len(english):
        c = english[i]
        if english.startswith("{{", i) or english.startswith("}}", i):
            out.append('{ "' + c + '" }')
            i += 2
            continue
        if c == "{":
            m = NAME.match(english, i)
            if not m:
                raise ParseError(f"{file}: only plain named arguments are allowed, not {english[i:i + 20]!r}")
            names.append(m.group(1))
            out.append("{ $" + m.group(1) + " }")
            i = m.end()
            continue
        if c == "}":
            raise ParseError(f"{file}: a lone }} in {english!r}")
        if c == "\n":
            out.append('{ "\\u000A" }')
        elif c == "\\":
            out.append('{ "\\\\" }')
        else:
            out.append(c)
        i += 1
    value = "".join(out)
    # Fluent trims the space around a value: keep any that the English has as a placeable.
    lead = len(value) - len(value.lstrip(" "))
    trail = len(value) - len(value.rstrip(" "))
    if lead:
        value = '{ "' + " " * lead + '" }' + value.lstrip(" ")
    if trail:
        value = value.rstrip(" ") + '{ "' + " " * trail + '" }'
    return value, names


def fixed_value(english: str, file: str) -> str:
    """A fixed text has no arguments: every brace in it is literal."""
    return fluent_value(english.replace("{", "{{").replace("}", "}}"), file)[0]


def extract(root: Path = ROOT) -> tuple[list[tuple[str, str, str]], list[str]]:
    """The (id, Fluent value, file) of every engine message, in source order, and any problems."""
    found: dict[str, tuple[str, str]] = {}
    order: list[tuple[str, str, str]] = []
    problems: list[str] = []
    for path in sorted((root / "crates").rglob("*.rs")):
        if "target" in path.parts or path.name == "text.rs":
            continue
        text = path.read_text(encoding="utf-8")
        if "text!(" not in text and "label!(" not in text and "Text::fixed(" not in text and "Check::" not in text:
            continue
        rel = str(path.relative_to(root))
        consts: dict[str, str] = {}
        for m in CONST.finditer(text):
            try:
                consts[m.group(1)] = string_literal(text, m.end())[0]
            except (ParseError, ValueError, IndexError):
                pass
        items: list[tuple[int, str, str]] = []
        for m in CALL.finditer(text):
            try:
                english, after = string_literal(text, m.end())
                value, names = fluent_value(english, rel)
                given = re.findall(r",\s*([a-z_][a-z0-9_]*)\s*=", text[after:matching_paren(text, text.index("(", m.start()))])
                if sorted(set(names)) != sorted(set(given)):
                    problems.append(f"{rel}: {m.group(1)}: the English names {sorted(set(names))} but the call gives {sorted(set(given))}")
                items.append((m.start(), m.group(1), value))
            except (ParseError, ValueError, IndexError) as e:
                problems.append(f"{rel}: {m.group(1)}: {e}")
        for m in FIXED.finditer(text):
            rest = text[m.end():]
            ident = re.match(r"([A-Z_][A-Z0-9_]*)\s*\)", rest)
            try:
                if ident:
                    if ident.group(1) not in consts:
                        raise ParseError(f"constant {ident.group(1)} is not defined in this file")
                    english = consts[ident.group(1)]
                else:
                    english = string_literal(text, m.end())[0]
                items.append((m.start(), m.group(1), fixed_value(english, rel)))
            except (ParseError, ValueError, IndexError) as e:
                problems.append(f"{rel}: {m.group(1)}: {e}")
        for m in CHECK_NAME.finditer(text) if "tests" not in path.parts else ():
            if text[m.end()] != '"':
                continue  # a name built at run time is not a fixed name
            name = string_literal(text, m.end())[0]
            items.append((m.start(), f"check-name-{slug(name)}", fixed_value(name.replace("_", " "), rel)))
        for _, ident, value in sorted(items):
            if ident in found:
                if found[ident][0] != value:
                    problems.append(f"{rel}: {ident} is also used in {found[ident][1]} with other English")
                continue
            found[ident] = (value, rel)
            order.append((ident, value, rel))
    return order, problems


def matching_paren(text: str, open_at: int) -> int:
    """The index of the `)` that closes the `(` at `open_at`, skipping string literals."""
    depth = 0
    i = open_at
    while i < len(text):
        c = text[i]
        if c == '"' or (c == "r" and re.match(r'r#*"', text[i:]) and not text[i - 1].isalnum()):
            i = string_literal(text, i)[1]
            continue
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    raise ParseError("unclosed call")


# Each file of the Text Library the engine writes, by Message Id prefix; the first match wins.
FILES = [("draw-", "drawing.ftl", "drawing labels and notes (English only for now: the Drawing Format\n# Standard allows ASCII only)"), ("", "engine.ftl", "checks, warnings, stability notes and names")]


def file_of(ident: str) -> str:
    return next(name for prefix, name, _ in FILES if ident.startswith(prefix))


def render(order: list[tuple[str, str, str]], name: str = "engine.ftl") -> str:
    what = next(w for _, n, w in FILES if n == name)
    lines = [HEADER.replace("checks, warnings, stability notes and names", what)]
    last = None
    for ident, value, rel in order:
        if file_of(ident) != name:
            continue
        if rel != last:
            lines.append(f"\n## {rel}\n")
            last = rel
        lines.append(f"{ident} = {value}")
    return "\n".join(lines) + "\n"


def outputs(order: list[tuple[str, str, str]]) -> dict[str, str]:
    """Each file the engine's messages go to, with its text; a file with no message is left out."""
    used = {file_of(i) for i, _, _ in order}
    return {name: render(order, name) for _, name, _ in FILES if name in used}


def problems(root: Path = ROOT) -> list[str]:
    """What the check form reports: extraction problems, and a file that is not up to date."""
    order, found = extract(root)
    folder = root / "locales" / "en"
    want = outputs(order)
    for _, name, _ in FILES:
        path = folder / name
        have = path.read_text(encoding="utf-8") if path.exists() else None
        if have != want.get(name) and not (name not in want and have is None):
            found.append(f"locales/en/{name} is not what the code gives: run python3 tools/i18n_extract.py")
    return found


def main(argv: list[str]) -> int:
    if "--check" in argv:
        found = problems()
        for p in found:
            print(p)
        return 1 if found else 0
    order, found = extract()
    for p in found:
        print(p)
    if found:
        return 1
    for name, text in outputs(order).items():
        (ROOT / "locales" / "en" / name).write_text(text, encoding="utf-8")
    print(f"i18n extract: {len(order)} message(s) in locales/en")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
