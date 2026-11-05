#!/usr/bin/env python3
"""Refuse to build or gate from a Cargo target directory outside this worktree (T-3832).

Concurrent worktrees that share one target directory run each other's binaries: Cargo fingerprints
by package, not by source tree, and the Makefile runs `target/release/rustsheet-eval` by path. In
the 2026-11-02 cleanup a shared `CARGO_TARGET_DIR` made one worktree's FORECAST.ETS return
`#N/IMPL!` from another worktree's engine, and once recorded a baseline with the wrong engine. So
`make build` (and with it every eval gate) and `make check-fast` stop when the target directory,
symlinks resolved, is not inside the worktree's own root.

The target directory is the one Cargo itself would use (`cargo metadata`, which honours
`CARGO_TARGET_DIR`, `CARGO_BUILD_TARGET_DIR` and any `.cargo/config.toml`). When Cargo cannot be
asked, the environment variables decide, else `<root>/target`. A `target` symlink that leaves the
worktree is refused too.

  scripts/target_dir_guard.py [ROOT]   # exit 0 when the target directory is inside ROOT (default: cwd)
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from collections.abc import Mapping
from pathlib import Path


def target_dir(root: Path, env: Mapping[str, str]) -> Path:
    """The target directory Cargo uses for the workspace at `root`, before resolving symlinks."""
    try:
        out = subprocess.run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1", "--offline"],
            cwd=root,
            env=dict(env),
            capture_output=True,
            text=True,
            check=True,
            timeout=60,
        ).stdout
        found = json.loads(out).get("target_directory")
        if isinstance(found, str) and found:
            return Path(found)
    except (OSError, subprocess.SubprocessError, ValueError):
        pass
    for name in ("CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR"):
        value = env.get(name)
        if value:
            path = Path(value)
            return path if path.is_absolute() else root / path
    return root / "target"


def outside(target: Path, root: Path) -> bool:
    """Whether `target`, with symlinks resolved, lies outside `root`."""
    return not target.resolve().is_relative_to(root.resolve())


def main() -> int:
    root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path.cwd()
    target = target_dir(root, os.environ)
    if not outside(target, root):
        return 0
    print(
        f"target-dir guard: Cargo's target directory {target} (resolved: {target.resolve()}) is "
        f"outside this worktree {root.resolve()}.\n"
        "Each worktree needs its own target directory: a shared one runs binaries built from "
        "another worktree's sources (T-3832, docs/process/agent-workflow.md). Unset "
        "CARGO_TARGET_DIR / CARGO_BUILD_TARGET_DIR, or point it inside this worktree.",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
