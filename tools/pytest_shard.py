"""Run one shard of the pytest suite (issue #716).

A pytest plugin. It is off unless PYTEST_SHARD is set to `k/N` (1 <= k <= N). CI loads it without
changing tools/gates.sh:

    PYTHONPATH=tools PYTEST_ADDOPTS="-p pytest_shard" PYTEST_SHARD=2/4 GATES_STEPS=pytest bash tools/gates.sh

A test belongs to shard `crc32(node id) % N + 1`. The rule depends on the node id alone, so every
xdist worker (and every run) picks the same tests, and each test lands in exactly one shard. The
deselected tests are reported as such, never as passed.

PYTEST_SHARD_IDS, when set, names a file that receives the total collected count and the selected
node ids. tools/check_pytest_shards.py reads the files of all shards and proves that the shards are
disjoint and together hold every collected test.
"""
import os
import zlib

import pytest


def _spec():
    raw = os.environ.get("PYTEST_SHARD", "")
    if not raw:
        return None
    try:
        k, n = (int(x) for x in raw.split("/"))
    except ValueError:
        raise SystemExit(f"PYTEST_SHARD must look like 2/4, not {raw!r}")
    if not 1 <= k <= n:
        raise SystemExit(f"PYTEST_SHARD must have 1 <= k <= N, not {raw!r}")
    return k, n


def shard_of(nodeid, n):
    """The shard number (1..n) of a node id. Stable across runs, machines and Python versions."""
    return zlib.crc32(nodeid.encode("utf-8")) % n + 1


# trylast: after `-m` and `-k` have deselected, so the shards divide the tests that would run.
@pytest.hookimpl(trylast=True)
def pytest_collection_modifyitems(config, items):
    spec = _spec()
    if spec is None:
        return
    k, n = spec
    keep = [i for i in items if shard_of(i.nodeid, n) == k]
    drop = [i for i in items if shard_of(i.nodeid, n) != k]
    total = len(items)
    items[:] = keep
    if drop:
        config.hook.pytest_deselected(items=drop)
    out = os.environ.get("PYTEST_SHARD_IDS")
    if out:
        tmp = f"{out}.{os.getpid()}.tmp"
        with open(tmp, "w", encoding="utf-8") as f:
            f.write(f"shard {k}/{n} total {total}\n")
            f.writelines(f"{i.nodeid}\n" for i in keep)
        os.replace(tmp, out)


def pytest_report_header(config):
    spec = _spec()
    return f"pytest shard {spec[0]} of {spec[1]}" if spec else None
