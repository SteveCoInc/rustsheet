from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import fast_gates  # noqa: E402 (import after sys.path setup)

BASE = ["fmt-check", "hygiene", "typos"]


class Targets(unittest.TestCase):
    def test_docs_skills_scripts_and_ci_run_only_the_base_gates(self) -> None:
        paths = [
            "docs/tasks/M0/T-1.md",
            ".claude/skills/x/SKILL.md",
            "scripts/x.py",
            ".github/workflows/ci.yml",
            "CLAUDE.md",
        ]
        self.assertEqual(fast_gates.targets(paths), BASE)

    def test_rust_code_adds_clippy_and_the_eval_gates(self) -> None:
        self.assertEqual(
            fast_gates.targets(["crates/rustsheet-engine/src/lib.rs"]), [*BASE, "clippy", "eval-gates"]
        )

    def test_eval_cases_baselines_and_ratchets_add_the_eval_gates(self) -> None:
        for path in (
            "eval/suites/probes/x.case.json",
            "eval/baseline/probes.txt",
            "quality/evidence-floor.json",
            "Cargo.lock",
        ):
            with self.subTest(path=path):
                self.assertIn("eval-gates", fast_gates.targets([path]))

    def test_docs_that_code_reads_count_as_code(self) -> None:
        # ci_changes.CODE_READ_PATHS: the eval harness reads docs/excel-semantics/.
        self.assertIn("eval-gates", fast_gates.targets(["docs/excel-semantics/x.md"]))

    def test_tools_add_py_check_and_bindings_add_py_bindings(self) -> None:
        self.assertEqual(fast_gates.targets(["tools/src/x.py"]), [*BASE, "py-check"])
        self.assertEqual(
            fast_gates.targets(["bindings/python/src/lib.rs"]), [*BASE, "clippy", "eval-gates", "py-bindings"]
        )

    def test_an_unknown_path_or_the_makefile_runs_every_fast_target(self) -> None:
        for path in ("Makefile", "newdir/x", "weird.toml"):
            with self.subTest(path=path):
                self.assertEqual(fast_gates.targets([path]), fast_gates.ALL)

    def test_no_paths_is_doubt_and_runs_every_fast_target(self) -> None:
        self.assertEqual(fast_gates.targets([]), fast_gates.ALL)

    def test_the_full_test_suite_is_never_a_fast_target(self) -> None:
        # CI runs `make test` in three shards; the worker runs the tests of the code it changed.
        self.assertNotIn("test", fast_gates.ALL)
        self.assertEqual(sorted(set(fast_gates.ALL)), sorted(fast_gates.ALL))


def _git(cwd: Path, *args: str) -> None:
    subprocess.run(["git", *args], cwd=cwd, check=True, capture_output=True)


class ChangedPaths(unittest.TestCase):
    def test_lists_commits_staged_unstaged_and_untracked_files_since_the_merge_base(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            _git(repo, "init", "-q", "-b", "main")
            _git(repo, "config", "user.email", "t@example.com")
            _git(repo, "config", "user.name", "t")
            (repo / "a.txt").write_text("a\n")
            (repo / "b.txt").write_text("b\n")
            _git(repo, "add", ".")
            _git(repo, "commit", "-qm", "base")
            _git(repo, "checkout", "-qb", "work")
            (repo / "committed.txt").write_text("c\n")
            _git(repo, "add", "committed.txt")
            _git(repo, "commit", "-qm", "work")
            (repo / "a.txt").write_text("changed\n")  # unstaged
            (repo / "b.txt").write_text("staged\n")
            _git(repo, "add", "b.txt")
            (repo / "new.txt").write_text("n\n")  # untracked
            paths = fast_gates.changed_paths("main", repo)
            self.assertEqual(sorted(paths or []), ["a.txt", "b.txt", "committed.txt", "new.txt"])

    def test_no_merge_base_is_none(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            _git(repo, "init", "-q", "-b", "main")
            self.assertIsNone(fast_gates.changed_paths("main", repo))

    def test_an_option_shaped_base_is_none(self) -> None:
        self.assertIsNone(fast_gates.changed_paths("--output=x", HERE))


if __name__ == "__main__":
    unittest.main()
