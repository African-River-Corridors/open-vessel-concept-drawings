#!/usr/bin/env python3
"""Decide whether a run of the App build workflow publishes a release, and which (issue #173).

    python3 tools/release_plan.py      # in the workflow; writes its answer to $GITHUB_OUTPUT

A release is made in two cases only:
  - the version number is bumped: a push to `main` that changes `version` in pyproject.toml publishes
    `v<version>` (unless that tag exists already). A `v*` tag pushed by hand does the same.
  - a test-release tag is pushed: `test-<anything>` publishes a pre-release named after the tag.
Any other push publishes nothing and builds nothing. A manual start builds and publishes nothing.

Answer (one `key=value` line each): `build` (true/false), `publish` (true/false), `tag` (the release
tag), `kind` (`numbered` or `test`), `on_main` (true when the commit must be on main), `reason`.
Standard library only. Reads GITHUB_EVENT_NAME, GITHUB_REF, GITHUB_SHA and BEFORE_SHA (the commit
`main` pointed at before the push).
"""
from __future__ import annotations

import os
import re
import subprocess
import sys
import tomllib

TEST_TAG = re.compile(r"^test-[A-Za-z0-9._-]+$")
NO_COMMIT = re.compile(r"^0+$")


def git(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(["git", *args], capture_output=True, text=True)


def version_at(commit: str) -> str | None:
    """The package version in pyproject.toml at `commit`, or None if it cannot be read."""
    r = git("show", f"{commit}:pyproject.toml")
    if r.returncode != 0:
        return None
    try:
        return tomllib.loads(r.stdout)["project"]["version"]
    except (tomllib.TOMLDecodeError, KeyError):
        return None


def tag_exists(tag: str) -> bool:
    return bool(git("tag", "--list", tag).stdout.strip())


def plan(event: str, ref: str, sha: str, before: str, version_at=version_at, tag_exists=tag_exists) -> dict:
    nothing = {"build": False, "publish": False, "tag": "", "kind": "", "on_main": False}
    if event == "workflow_dispatch":
        return {**nothing, "build": True, "reason": "manual start: build the installers, publish nothing"}
    if event != "push":
        return {**nothing, "reason": f"event {event!r} never releases"}
    if ref.startswith("refs/tags/"):
        tag = ref.removeprefix("refs/tags/")
        if tag.startswith("v"):
            return {"build": True, "publish": True, "tag": tag, "kind": "numbered", "on_main": True,
                    "reason": f"version tag {tag} pushed"}
        if TEST_TAG.match(tag):
            return {"build": True, "publish": True, "tag": tag, "kind": "test", "on_main": False,
                    "reason": f"test-release tag {tag} pushed"}
        return {**nothing, "reason": f"tag {tag!r} is neither v* nor test-*"}
    if ref != "refs/heads/main":
        return {**nothing, "reason": f"a push to {ref} never releases"}
    now = version_at(sha)
    if now is None:
        raise SystemExit(f"release_plan: cannot read the version in pyproject.toml at {sha}")
    was = None if NO_COMMIT.match(before or "0") else version_at(before)
    if was == now:
        return {**nothing, "reason": f"version {now} unchanged: no release"}
    tag = f"v{now}"
    if tag_exists(tag):
        return {**nothing, "reason": f"version is {now}, but tag {tag} exists already: no release"}
    return {"build": True, "publish": True, "tag": tag, "kind": "numbered", "on_main": True,
            "reason": f"version bumped {was} -> {now}: release {tag}"}


def main() -> int:
    env = os.environ
    p = plan(env.get("GITHUB_EVENT_NAME", ""), env.get("GITHUB_REF", ""), env.get("GITHUB_SHA", ""),
             env.get("BEFORE_SHA", ""))
    lines = [f"{k}={str(v).lower() if isinstance(v, bool) else v}" for k, v in p.items()]
    print("\n".join(lines))
    out = env.get("GITHUB_OUTPUT")
    if out:
        with open(out, "a", encoding="utf-8") as f:
            f.write("\n".join(lines) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
