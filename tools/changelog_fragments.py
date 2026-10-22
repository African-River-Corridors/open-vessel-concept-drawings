#!/usr/bin/env python3
"""Changelog fragments (issue #560): one `changes/<branch>.md` file per pull request.

    python3 tools/changelog_collect.py            move every fragment into CHANGELOG.md, delete the files
    python3 tools/changelog_collect.py --check    only test that every fragment is well formed

The format is in `changes/README.md`. This module holds the parts the design-impact check shares: the
file name of a branch, the well-formed test, and writing and reading a fragment. Standard library only.
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CHANGES = ROOT / "changes"
CHANGELOG = ROOT / "CHANGELOG.md"
UNRELEASED = "<!-- unreleased -->"
IMPACT = "<!-- design-impact -->"
README = "README.md"
SECTION_UNRELEASED = "## Unreleased"
SECTION_DESIGN = "## Design changes"


def fragment_name(key: str) -> str:
    """The file name for a branch or entry key: `ticket/449-x` is `ticket-449-x.md`."""
    slug = re.sub(r"[^A-Za-z0-9._-]+", "-", key).strip("-.")
    return (slug or "change") + ".md"


def fragment_path(key: str, folder: Path = CHANGES) -> Path:
    return folder / fragment_name(key)


def split(text: str) -> tuple[str, str]:
    """(the unreleased part, the design-impact part) of a fragment, each stripped."""
    unreleased, impact = "", ""
    if IMPACT in text:
        head, impact = text.split(IMPACT, 1)
    else:
        head = text
    if UNRELEASED in head:
        unreleased = head.split(UNRELEASED, 1)[1]
    return unreleased.strip("\n"), impact.strip("\n")


def join(unreleased: str, impact: str) -> str:
    parts = []
    if unreleased.strip():
        parts.append(f"{UNRELEASED}\n{unreleased.strip(chr(10))}\n")
    if impact.strip():
        parts.append(f"{IMPACT}\n{impact.strip(chr(10))}\n")
    return "\n".join(parts)


def problems(text: str) -> list[str]:
    """Why a fragment is not well formed; an empty list when it is."""
    out: list[str] = []
    if text.count(UNRELEASED) > 1 or text.count(IMPACT) > 1:
        out.append("a marker line appears more than once")
    pre = text.split(UNRELEASED if UNRELEASED in text else IMPACT, 1)[0]
    if pre.strip():
        out.append("text before the first marker line")
    unreleased, impact = split(text)
    if not unreleased and not impact:
        out.append(f"no `{UNRELEASED}` part and no `{IMPACT}` part")
    if UNRELEASED in text and not unreleased:
        out.append("the unreleased part is empty")
    if unreleased:
        for line in unreleased.splitlines():
            if line.strip() and not (line.startswith("- ") or line.startswith("  ")):
                out.append(f"an unreleased line is not a bullet: {line[:60]!r}")
                break
    if IMPACT in text and not impact:
        out.append("the design-impact part is empty")
    if impact:
        if not impact.lstrip().startswith("### "):
            out.append("the design-impact part does not start with a `### <key> (<date>)` heading")
        if "| Case | Quantity |" not in impact:
            out.append("the design-impact part has no table (`| Case | Quantity | ...`)")
    return out


fragment_problems = problems


def fragments(folder: Path = CHANGES) -> list[Path]:
    return sorted(p for p in folder.glob("*.md") if p.name != README) if folder.is_dir() else []


def write_impact(key: str, entry: str, folder: Path = CHANGES) -> Path:
    """Put the design-impact entry in the fragment of `key`, keeping its unreleased part."""
    path = fragment_path(key, folder)
    unreleased = split(path.read_text(encoding="utf-8"))[0] if path.exists() else ""
    folder.mkdir(parents=True, exist_ok=True)
    path.write_text(join(unreleased, entry), encoding="utf-8")
    return path


def fragment_holds(tbl: str, folder: Path = CHANGES) -> bool:
    """Whether some fragment holds the design-impact table `tbl`."""
    return any(tbl in p.read_text(encoding="utf-8") for p in fragments(folder))


def collect_text(changelog: str, texts: list[str]) -> str:
    """`changelog` with the fragments in `texts` added. Bullets go to the top of `## Unreleased`, entries to
    the top of `## Design changes`, in the order of `texts`. The rest of the text is not touched."""
    bullets = [u for u, _ in map(split, texts) if u]
    entries = [i for _, i in map(split, texts) if i]
    out = changelog
    if bullets:
        m = re.search(rf"^{re.escape(SECTION_UNRELEASED)}[ \t]*\n\n?", out, re.M)
        if m is None:
            raise SystemExit(f"changelog_collect: CHANGELOG.md has no `{SECTION_UNRELEASED}` section")
        out = out[:m.end()] + "".join(b.rstrip("\n") + "\n" for b in bullets) + out[m.end():]
    if entries:
        m = re.search(rf"^{re.escape(SECTION_DESIGN)}[ \t]*$", out, re.M)
        if m is None:
            raise SystemExit(f"changelog_collect: CHANGELOG.md has no `{SECTION_DESIGN}` section")
        nxt = out.find("\n### ", m.end())
        at = nxt if nxt != -1 else len(out)
        block = "".join("\n" + e.rstrip("\n") + "\n" for e in entries)
        out = out[:at].rstrip("\n") + "\n" + block + out[at:] if nxt == -1 else out[:at] + block + out[at:]
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--check", action="store_true", help="only test that every fragment is well formed")
    ap.add_argument("--changes", type=Path, default=CHANGES)
    ap.add_argument("--changelog", type=Path, default=CHANGELOG)
    args = ap.parse_args()
    files = fragments(args.changes)
    bad = {p.name: problems(p.read_text(encoding="utf-8")) for p in files}
    bad = {n: v for n, v in bad.items() if v}
    for name, why in bad.items():
        print(f"changelog_collect: {name}: " + "; ".join(why), file=sys.stderr)
    if bad:
        return 1
    if args.check:
        print(f"changelog_collect: {len(files)} fragment(s), all well formed")
        return 0
    if not files:
        print("changelog_collect: no fragments")
        return 0
    args.changelog.write_text(
        collect_text(args.changelog.read_text(encoding="utf-8"), [p.read_text(encoding="utf-8") for p in files]),
        encoding="utf-8")
    for p in files:
        p.unlink()
    print(f"changelog_collect: moved {len(files)} fragment(s) into {args.changelog.name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
