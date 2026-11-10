"""Stale owners of the pivot feature catalog (T-4952, decision 0274 item 6).

eval/catalog/pivot-features.tsv's hand-filled `owner` column names the open M16 task that owns a
row. The M16 exit row (crates/rustsheet-eval/src/milestone_exit/pivot.rs) counts a row with an owner
as owned without reading the task files, which code may not read (decision 0066). So, as for a
`TODO(T-nnnn)`, `scripts/tasks.py validate` (`make hygiene`, always run) fails while an owner is not an
open M16 task: the PR that closes a task re-points or clears its rows.
"""

from __future__ import annotations

from pathlib import Path

CATALOG = "eval/catalog/pivot-features.tsv"
MILESTONE = "M16"


def catalog(root: Path) -> str:
    """The catalog's text under `root`; empty when the tree has none."""
    path = root / CATALOG
    return path.read_text() if path.is_file() else ""


def stale(tasks: list[dict[str, object]], text: str, open_statuses: tuple[str, ...]) -> list[str]:
    """Every row of catalog `text` whose `owner` is not a task of MILESTONE with a status in
    `open_statuses`, as `path:line: ...`; nothing when the text has no `owner` column."""
    lines = text.splitlines()
    header = lines[0].split("\t") if lines else []
    if "owner" not in header:
        return []
    col = header.index("owner")
    by_id = {str(t.get("id")): t for t in tasks}
    errors = []
    for n, line in enumerate(lines[1:], 2):
        cells = line.split("\t")
        owner = cells[col] if col < len(cells) else ""
        if not owner:
            continue
        task = by_id.get(owner)
        if task is None:
            state = "does not exist"
        elif task.get("status") not in open_statuses:
            state = f"is {task.get('status')}"
        elif task.get("milestone") != MILESTONE:
            state = f"is {task.get('milestone')}'s"
        else:
            continue
        errors.append(
            f"{CATALOG}:{n}: {cells[0]} owner {owner} names a task that {state}; re-point it to the "
            f"open {MILESTONE} task that owns the row, or clear it"
        )
    return errors
