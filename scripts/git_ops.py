"""The git plumbing the repository's tools share, as one definition (decision 0252).

`git` runs a git command in a checkout, `merge_tree` computes a merge without touching the work
tree, and `push_lease` is the compare-and-swap push the id registry relies on (decision 0215).
They lived in the merge lock's script until the merge train and the lock were deleted.
Standard library only.
"""

from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REMOTE = "origin"
BASE_BRANCH = "main"


class GitError(Exception):
    """A git command failed for a reason other than a lost race."""


def git(*args: str, root: Path = ROOT, check: bool = True) -> subprocess.CompletedProcess[str]:
    """`git -C root args`, text output captured; GitError when it fails and `check` is set."""
    result = subprocess.run(
        ["git", "-C", str(root), *args],
        stdin=subprocess.DEVNULL,
        capture_output=True,
        text=True,
        check=False,
    )
    if check and result.returncode != 0:
        raise GitError(f"git {' '.join(args)}: {result.stderr.strip()}")
    return result


def merge_tree(root: Path, a: str, b: str) -> tuple[str, list[str]] | None:
    """(tree, conflicted paths) of the merge of commits `a` and `b` (`git merge-tree
    --write-tree`, conflicts written with markers); None when it cannot be computed. `make
    catch-up` and `make renumber` share it."""
    merged = git("merge-tree", "--write-tree", "--name-only", a, b, root=root, check=False)
    if merged.returncode not in (0, 1):
        return None
    lines = merged.stdout.split("\n")
    # After the tree: the conflicted paths, a blank line, then informational messages.
    rest = lines[1:]
    return lines[0].strip(), rest[: rest.index("")] if "" in rest else rest


def push_lease(expected: str, new: str, root: Path, ref: str) -> bool:
    """Sets `ref` on origin to the commit `new` only if origin still has `expected` there ("" for
    absent). False when another writer moved it first: of two racing writers exactly one wins."""
    lease = f"--force-with-lease={ref}:{expected}"
    result = git("push", "-q", lease, REMOTE, f"{new}:{ref}", root=root, check=False)
    return result.returncode == 0
