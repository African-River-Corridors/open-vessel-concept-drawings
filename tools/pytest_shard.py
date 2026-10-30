"""Run one shard of the pytest suite (issue #716).

A pytest plugin. It is off unless PYTEST_SHARD is set to `k/N` (1 <= k <= N). CI loads it without
changing tools/gates.sh:

    PYTHONPATH=tools PYTEST_ADDOPTS="-p pytest_shard" PYTEST_SHARD=2/4 GATES_STEPS=pytest bash tools/gates.sh

A test belongs to shard `crc32(node id) % N + 1`, except the few slow tests in PINNED, which are
placed by hand so that no shard waits on two of them. Either way the shard depends on the node id
alone, so every xdist worker (and every run) picks the same tests, and each test lands in exactly
one shard. A pinned test that is renamed or removed falls back to the hash: it still runs once. The
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


# The slowest tests of the `pr` profile (pytest --durations, 2026-10-30), placed for 4 shards. Shard 1
# also runs the evaluation harness and the Calibration Bands in CI, so it gets the lighter ones.
PINNED = {
    4: {
        "tests/test_stern_oracle.py::test_the_case_files_are_what_the_oracle_writes": 4,
        "tests/test_evaluation_runner.py::test_the_real_corpus_has_no_false_pass_and_its_results_are_current": 2,
        "tests/test_evaluation_runner.py::test_a_limit_no_case_comes_near_is_a_weak_corpus_warning": 3,
        "tests/test_evaluation_runner.py::test_a_stale_results_file_fails_the_check": 3,
        "tests/test_evaluation_runner.py::test_a_coverage_gap_is_only_a_warning": 2,
        "tests/test_evaluation_runner.py::test_a_deliberately_wrong_expected_verdict_turns_the_runner_red": 1,
        "tests/test_evaluation_runner.py::test_a_false_fail_is_only_a_warning": 1,
    },
}


def shard_of(nodeid, n):
    """The shard number (1..n) of a node id. Stable across runs, machines and Python versions."""
    pinned = PINNED.get(n, {}).get(nodeid)
    if pinned is not None:
        return pinned
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
