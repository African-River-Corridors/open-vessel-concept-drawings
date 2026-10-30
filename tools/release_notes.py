#!/usr/bin/env python3
"""Fit the release notes into a GitHub release body, which holds at most 125,000 characters.

    python3 tools/release_notes.py notes.md --link <url of the full CHANGELOG.md>

The App build workflow writes `notes.md` from the CHANGELOG.md section of the version, then runs this.
Notes that fit are left as they are. Longer notes are cut at the end of the last whole line that fits,
and a last paragraph gives the link to the full list. The section opens with its summary, so the cut
keeps the summary and drops the oldest entries. The 0.3.0 section (139,695 characters, every collected
fragment) made `gh release create` fail with HTTP 422 "body is too long".

Standard library only.
"""
import argparse
import sys
from pathlib import Path

GITHUB_LIMIT = 125_000
LIMIT = 120_000  # a margin under GITHUB_LIMIT


def cap(text: str, link: str, limit: int = LIMIT) -> str:
    """`text` if it fits in `limit` characters; else its whole lines that fit, then the link."""
    if len(text) <= limit:
        return text
    tail = f"\n\nThe list is cut here to fit a GitHub release. Every change of this release is in [CHANGELOG.md]({link}).\n"
    room = limit - len(tail)
    if room <= 0:
        raise ValueError(f"the limit {limit} leaves no room for the link")
    cut = text.rfind("\n", 0, room + 1)
    kept = text[:cut] if cut > 0 else ""
    return kept.rstrip("\n") + tail


def main(argv=None) -> int:
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("notes", type=Path)
    p.add_argument("--link", required=True, help="the full CHANGELOG.md, at the released commit")
    a = p.parse_args(argv)
    text = a.notes.read_text(encoding="utf-8")
    out = cap(text, a.link)
    a.notes.write_text(out, encoding="utf-8")
    if out != text:
        print(f"release_notes: cut from {len(text):,} to {len(out):,} characters", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
