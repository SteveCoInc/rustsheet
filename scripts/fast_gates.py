#!/usr/bin/env python3
"""The make targets of `make check-fast`: the local pre-push gate (decision 0248).

CI runs every gate of `make check` on each PR head and again on the merge group (decision 0247),
so a worker runs only the gates its change can break, plus the tests of the code it changed:

- always `fmt-check`, `hygiene` (the ratchets, task and decision checks and the scripts' tests)
  and `typos`, which are fast and catch most docs and process mistakes;
- `clippy` and `eval-gates` when Rust, the eval data, the ratchet files, or a document that code
  reads (`ci_changes.CODE_READ_PATHS`) changed;
- `py-check` when `tools/` changed, `py-bindings` when `bindings/` changed;
- every target above for a path it does not know (the Makefile, a new top-level file) and when
  the changed paths cannot be listed: doubt runs more, as in `ci_changes`.

The full Rust test suite (`make test`) is never one: CI runs it in three shards.

  scripts/fast_gates.py [--base origin/main]   # prints the targets, space-separated
"""

from __future__ import annotations

import sys
from pathlib import Path

import ci_changes

BASE = ["fmt-check", "hygiene", "typos"]
ALL = [*BASE, "clippy", "eval-gates", "py-check", "py-bindings"]

# Paths whose change can break clippy or the eval gates. An entry ending in `/` is a directory.
RUST_OR_EVAL = (
    "crates/",
    "bindings/",
    "eval/",
    "fuzz/",
    "quality/",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "clippy.toml",
    "rustfmt.toml",
    "deny.toml",
)
# Paths that only the base gates check: docs and skills (outside CODE_READ_PATHS), the scripts and
# CI files (hygiene runs the scripts' tests and the CI-workflow lint), top-level text.
BASE_ONLY = (
    "docs/",
    ".claude/",
    "scripts/",
    ".github/",
    "_typos.toml",
    ".gitignore",
    ".gitattributes",
    ".editorconfig",
    "LICENSE",
    "NOTICE",
)


def _under(path: str, prefixes: tuple[str, ...]) -> bool:
    return any(path.startswith(p) if p.endswith("/") else path == p for p in prefixes)


def targets(paths: list[str]) -> list[str]:
    """The fast targets for these changed paths, in `ALL`'s order; no paths is doubt."""
    if not paths:
        return list(ALL)
    wanted = set(BASE)
    for path in paths:
        if ci_changes.is_code_read(path) or _under(path, RUST_OR_EVAL):
            wanted |= {"clippy", "eval-gates"}
        if path.startswith("tools/") or path.endswith(".whl"):
            wanted.add("py-check")
        if path.startswith("bindings/"):
            wanted.add("py-bindings")
        known = _under(path, RUST_OR_EVAL + BASE_ONLY) or path.startswith("tools/") or path.endswith(".whl")
        if not known and not ("/" not in path and path.endswith(".md")):
            return list(ALL)
    return [t for t in ALL if t in wanted]


def changed_paths(base: str, cwd: Path | None = None) -> list[str] | None:
    """Paths changed since the merge base of `base` and HEAD: commits, the index, the working tree
    and untracked files. None when they cannot be listed (no single merge base, a git failure)."""
    if not ci_changes.REVISION.fullmatch(base):
        return None
    bases = ci_changes.git_bytes(cwd, "merge-base", "--all", base, "HEAD")
    if bases is None or len(bases.split()) != 1:
        return None
    merge_base = bases.decode("ascii", "replace").strip()
    # No second revision: the diff runs from the merge base to the working tree, index included.
    tracked = ci_changes.git_bytes(cwd, "diff", "-z", "--no-renames", "--name-only", merge_base, "--")
    untracked = ci_changes.git_bytes(cwd, "ls-files", "-z", "--others", "--exclude-standard")
    if tracked is None or untracked is None:
        return None
    try:
        names = (tracked + untracked).decode("utf-8").split("\0")
    except UnicodeDecodeError:
        return None
    return sorted({n for n in names if n})


def main() -> int:
    args = sys.argv[1:]
    base = args[1] if len(args) == 2 and args[0] == "--base" else "origin/main"
    paths = changed_paths(base)
    chosen = targets(paths or [])
    print(f"check-fast: {len(paths) if paths is not None else 'unknown'} changed paths", file=sys.stderr)
    print(" ".join(chosen))
    return 0


if __name__ == "__main__":
    sys.exit(main())
