"""Check the pytest shards of one CI run (issue #716).

    python tools/check_pytest_shards.py DIR

DIR holds one `ids-<k>.txt` per shard, as tools/pytest_shard.py writes them. The check fails unless:
all N shards are there, they agree on the total collected count, no test is in two shards, and the
shards together hold exactly that total. So a test cannot be dropped or run twice.
"""
import re
import sys
from pathlib import Path


def check(directory):
    files = sorted(Path(directory).glob("ids-*.txt"))
    if not files:
        return "no shard files found"
    seen, totals, parts, n = {}, set(), set(), None
    for f in files:
        lines = f.read_text(encoding="utf-8").splitlines()
        m = re.fullmatch(r"shard (\d+)/(\d+) total (\d+)", lines[0] if lines else "")
        if not m:
            return f"{f.name}: bad header"
        k, n_f, total = (int(x) for x in m.groups())
        if n is not None and n_f != n:
            return f"{f.name}: shard count {n_f}, others say {n}"
        n = n_f
        parts.add(k)
        totals.add(total)
        for nodeid in lines[1:]:
            if nodeid in seen:
                return f"{nodeid} is in shard {seen[nodeid]} and shard {k}"
            seen[nodeid] = k
        print(f"shard {k}/{n}: {len(lines) - 1} tests")
    if parts != set(range(1, n + 1)):
        return f"shards present {sorted(parts)}, expected 1..{n}"
    if len(totals) != 1:
        return f"shards disagree on the collected total: {sorted(totals)}"
    total = totals.pop()
    if len(seen) != total:
        return f"shards hold {len(seen)} tests, {total} were collected"
    print(f"ok: {len(seen)} tests, each in exactly one of {n} shards")
    return None


if __name__ == "__main__":
    problem = check(sys.argv[1])
    if problem:
        sys.exit(f"pytest shards: FAIL: {problem}")
