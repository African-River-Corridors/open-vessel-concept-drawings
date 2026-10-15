"""The design-impact gate, end to end, on a small fixture repository (ticket 145).

The fixture is a real git repository that holds a copy of tools/design_impact.py and the two
calibration files. A stand-in `cargo` (a shell script on PATH) "builds" a stand-in `ovcd` whose result
comes from `factor.txt` in the tree it was built from, so a branch can move results or break the build
without a compiler. The tests run `python tools/design_impact.py gate` in the fixture and look at the
exit code and what it says.
"""
import os
import shutil
import subprocess
import sys
import textwrap
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

FAKE_CARGO = textwrap.dedent('''\
    #!/bin/sh
    # stand-in for `cargo build -p ovcd-cli`, run in the tree to build
    if [ -e BREAK_BUILD ]; then echo "error: could not compile" >&2; exit 101; fi
    mkdir -p "$CARGO_TARGET_DIR/debug"
    exe="$CARGO_TARGET_DIR/debug/ovcd"
    factor=$(cat factor.txt)
    cat > "$exe" <<EOS
    #!/bin/sh
    case "\\$1" in
      fill) if [ "$factor" = dead ]; then echo "error: refused" >&2; exit 1; fi; cp "\\$2" "\\$3" ;;
      run) echo '{"options":[{"name":"x","results":{"results":{"cargo_t":$factor,"governs":"hold"},"checks":[]}}]}' ;;
    esac
    EOS
    chmod +x "$exe"
    printf '{"reason":"compiler-artifact","executable":"%s","target":{"name":"ovcd","kind":["bin"]}}\\n' "$exe"
''')


class Fixture:
    def __init__(self, tmp: Path):
        self.root = tmp / "repo"
        (self.root / "tools").mkdir(parents=True)
        (self.root / "calibration").mkdir()
        shutil.copy(ROOT / "tools" / "design_impact.py", self.root / "tools")
        for name in ("cases.yaml", "cn_yangtze_gb38030_draft_2018.yaml"):
            shutil.copy(ROOT / "calibration" / name, self.root / "calibration")
        (self.root / "factor.txt").write_text("10\n")
        (self.root / "CHANGELOG.md").write_text("# Changelog\n\n## Unreleased\n")
        (self.root / ".gitignore").write_text("target/\n")
        self.bin = tmp / "bin"
        self.bin.mkdir()
        (self.bin / "cargo").write_text(FAKE_CARGO)
        (self.bin / "cargo").chmod(0o755)
        self.env = {**os.environ, "PATH": f"{self.bin}{os.pathsep}{os.environ['PATH']}",
                    "GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@example.com",
                    "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@example.com"}
        self.env.pop("GITHUB_STEP_SUMMARY", None)
        self.git("init", "-q", "-b", "main")
        self.commit("base")

    def git(self, *args):
        subprocess.run(["git", *args], cwd=self.root, env=self.env, check=True, capture_output=True)

    def commit(self, message):
        self.git("add", "-A")
        self.git("commit", "-q", "-m", message)

    def write(self, rel, text):
        (self.root / rel).write_text(text)

    def run(self, mode="gate"):
        return subprocess.run([sys.executable, "tools/design_impact.py", mode], cwd=self.root,
                              env=self.env, capture_output=True, text=True)


def branch(tmp_path):
    fx = Fixture(tmp_path)
    fx.git("checkout", "-q", "-b", "feature")
    return fx


def test_a_branch_that_moves_nothing_passes(tmp_path):
    fx = branch(tmp_path)
    fx.write("README.md", "x\n")
    fx.commit("docs")
    r = fx.run()
    assert r.returncode == 0, r.stderr
    assert "no design change" in r.stdout


def test_a_moving_branch_without_the_changelog_table_fails(tmp_path):
    fx = branch(tmp_path)
    fx.write("factor.txt", "11\n")
    fx.commit("move results")
    r = fx.run()
    assert r.returncode == 1
    assert "design change" in r.stdout
    assert "CHANGELOG.md does not hold the table" in r.stderr


def test_the_same_branch_passes_once_the_changelog_holds_the_table(tmp_path):
    fx = branch(tmp_path)
    fx.write("factor.txt", "11\n")
    fx.commit("move results")
    assert fx.run().returncode == 1
    w = fx.run("changelog")
    assert w.returncode == 0 and "wrote CHANGELOG.md" in w.stdout
    fx.commit("changelog")
    r = fx.run()
    assert r.returncode == 0, r.stderr
    assert "### feature" in (fx.root / "CHANGELOG.md").read_text()


def test_a_table_for_an_older_move_does_not_cover_a_new_one(tmp_path):
    fx = branch(tmp_path)
    fx.write("factor.txt", "11\n")
    fx.commit("move")
    fx.run("changelog")
    fx.commit("changelog")
    fx.write("factor.txt", "12\n")
    fx.commit("move again")
    assert fx.run().returncode == 1


def test_a_failed_base_build_fails_the_gate(tmp_path):
    fx = Fixture(tmp_path)
    fx.write("BREAK_BUILD", "")
    fx.commit("a broken main")
    fx.git("checkout", "-q", "-b", "feature")
    (fx.root / "BREAK_BUILD").unlink()                      # the branch builds, the base does not
    fx.write("factor.txt", "10\n")
    fx.write("README.md", "fix\n")
    fx.commit("fix the build")
    r = fx.run()
    assert r.returncode != 0
    assert "cargo build failed" in r.stderr


def test_a_failed_branch_build_fails_the_gate(tmp_path):
    fx = branch(tmp_path)
    fx.write("BREAK_BUILD", "")
    fx.commit("break the build")
    r = fx.run()
    assert r.returncode != 0 and "cargo build failed" in r.stderr


def test_a_branch_whose_designs_all_stop_computing_fails_the_gate(tmp_path):
    fx = branch(tmp_path)
    fx.write("factor.txt", "dead\n")
    fx.commit("every design is refused")
    r = fx.run()
    assert r.returncode != 0
    assert "no case designs computed on the this tree binary" in r.stderr


def test_a_branch_with_fewer_cases_than_the_base_fails_the_gate(tmp_path):
    fx = branch(tmp_path)
    cases = (fx.root / "calibration" / "cases.yaml").read_text()
    assert "dry_barge:" in cases
    # drop the calibration vessels: the branch's own case set shrinks, results do not move
    import yaml
    data = yaml.safe_load(cases)
    data["dry_barge"] = data["dry_barge"][:2]
    fx.write("calibration/cases.yaml", yaml.safe_dump(data))
    fx.commit("remove cases")
    r = fx.run()
    assert r.returncode == 1
    assert "but the base commit runs" in r.stderr


def test_an_empty_case_file_does_not_pass(tmp_path):
    fx = branch(tmp_path)
    fx.write("calibration/cases.yaml", "")
    fx.commit("empty")
    assert fx.run().returncode != 0


def test_a_move_on_main_itself_is_reported_but_does_not_fail(tmp_path):
    fx = Fixture(tmp_path)                                   # on main: the base is the parent commit
    fx.write("factor.txt", "11\n")
    fx.commit("move on main")
    r = fx.run()
    assert r.returncode == 0, r.stderr
    assert "design change" in r.stdout


def test_a_repository_with_no_base_commit_fails_the_gate(tmp_path):
    fx = Fixture(tmp_path)                                   # one commit, on main: nothing to compare with
    r = fx.run()
    assert r.returncode != 0
    assert "no base commit" in r.stderr
