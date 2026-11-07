"""The milestone phase order and the current milestone, for `scripts/tasks.py` (decision 0022 as
amended by T-3824's decision).

The phase order is the table under `## Phase order` in docs/milestones.md (decision 0027), read
row by row; the current milestone is the `(M<n> is current)` of docs/status.md's `## Snapshot`
heading, which the steward rewrites every run. Stdlib only, like tasks.py.
"""

from __future__ import annotations

import re
from pathlib import Path

PHASE_HEADING = "## Phase order"
SNAPSHOT = re.compile(r"^## Snapshot\b.*\((M\d+) is current\)", re.M)
MILESTONE = re.compile(r"\bM\d+\b")


def phase_order(milestones_md: str) -> list[str]:
    """The milestones in the order the phase table lists them (M0 first), each once."""
    order: list[str] = []
    _, _, rest = milestones_md.partition(f"\n{PHASE_HEADING}")
    for line in rest.splitlines()[1:]:
        if line.startswith("## "):
            break
        cells = line.strip().strip("|").split("|") if line.startswith("|") else []
        if len(cells) >= 2:
            order += [m for m in MILESTONE.findall(cells[1]) if m not in order]
    return order


def current_milestone(status_md: str) -> str | None:
    """The snapshot heading's current milestone; None without one (the bypass is then off)."""
    m = SNAPSHOT.search(status_md)
    return m.group(1) if m else None


def read(root: Path) -> tuple[str | None, list[str]]:
    """(current milestone, phase order) of the tree at `root`; (None, []) when a file is missing."""
    try:
        status = (root / "docs" / "status.md").read_text()
        milestones = (root / "docs" / "milestones.md").read_text()
    except OSError:
        return None, []
    return current_milestone(status), phase_order(milestones)


def passes_later_work(
    task: dict[str, object], flying: list[dict[str, object]], current: str | None, order: list[str]
) -> bool:
    """Whether `task` may pass a lane whose in-flight tasks are `flying`: it is of the current
    milestone and every one of them is of a later milestone in the phase order. A task not on
    main (`{}`) or of an unknown milestone is never later, and M0 comes first, so neither is passed."""
    if current is None or task.get("milestone") != current or current not in order:
        return False
    rank = order.index(current)
    return all(f.get("milestone") in order and order.index(str(f.get("milestone"))) > rank for f in flying)
