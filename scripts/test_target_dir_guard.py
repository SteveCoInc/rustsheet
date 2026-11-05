"""Tests for scripts/target_dir_guard.py (T-3832)."""

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import target_dir_guard as guard  # noqa: E402

SCRIPT = Path(__file__).resolve().parent / "target_dir_guard.py"


class Outside(unittest.TestCase):
    def setUp(self) -> None:
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.base = Path(tmp.name)
        self.root = self.base / "wt"
        self.root.mkdir()

    def test_own_target_is_inside(self) -> None:
        self.assertFalse(guard.outside(self.root / "target", self.root))

    def test_missing_target_inside_is_inside(self) -> None:
        self.assertFalse(guard.outside(self.root / "nested" / "target", self.root))

    def test_shared_dir_is_outside(self) -> None:
        self.assertTrue(guard.outside(self.base / "shared-target", self.root))

    def test_sibling_with_common_prefix_is_outside(self) -> None:
        self.assertTrue(guard.outside(self.base / "wt-other" / "target", self.root))

    def test_symlink_leaving_the_worktree_is_outside(self) -> None:
        shared = self.base / "shared"
        shared.mkdir()
        (self.root / "target").symlink_to(shared)
        self.assertTrue(guard.outside(self.root / "target", self.root))


class TargetDirFallback(unittest.TestCase):
    """Without a Cargo workspace, `cargo metadata` fails and the environment decides."""

    def setUp(self) -> None:
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        self.env = {"PATH": os.environ.get("PATH", "")}

    def test_default_is_root_target(self) -> None:
        self.assertEqual(guard.target_dir(self.root, self.env), self.root / "target")

    def test_env_var_wins(self) -> None:
        env = {**self.env, "CARGO_TARGET_DIR": "/elsewhere/t"}
        self.assertEqual(guard.target_dir(self.root, env), Path("/elsewhere/t"))

    def test_relative_env_var_is_under_root(self) -> None:
        env = {**self.env, "CARGO_BUILD_TARGET_DIR": "out"}
        self.assertEqual(guard.target_dir(self.root, env), self.root / "out")


class Repository(unittest.TestCase):
    """The guard run on this checkout: Cargo's own answer, through the environment."""

    ROOT = Path(__file__).resolve().parent.parent

    def run_guard(self, **extra: str) -> subprocess.CompletedProcess[str]:
        env = {k: v for k, v in os.environ.items() if k not in ("CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR")}
        env.update(extra)
        return subprocess.run(
            [sys.executable, str(SCRIPT), str(self.ROOT)],
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )

    def test_shared_target_dir_is_refused(self) -> None:
        with tempfile.TemporaryDirectory() as shared:
            result = self.run_guard(CARGO_TARGET_DIR=shared)
        self.assertEqual(result.returncode, 1)
        self.assertIn("outside this worktree", result.stderr)

    def test_target_dir_inside_is_accepted(self) -> None:
        result = self.run_guard(CARGO_TARGET_DIR=str(self.ROOT / "target"))
        self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == "__main__":
    unittest.main()
