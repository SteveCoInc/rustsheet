"""T-4435: every eval cell whose value moves between two `make eval` reports, and whether it moves toward Excel.

    python3 docs/research/t4435/moved_cells.py MAIN_REPORT.json BRANCH_REPORT.json

A report lists every cell that is not bit-exact; a cell missing from a failing case's list, or from a passing
case, was Excel's value. Prints per case the moved cells (closeness class before -> after, relative distance to
Excel before -> after, and to the math sidecar where there is one), then the counts.
"""

from __future__ import annotations

import json
import math
import sys
from collections import Counter

ORDER = ["pass", "ulp1", "ulp4", "digits15", "residue13", "r3_math", "visible_numeric", "visible_other"]


def cells(path: str) -> dict[tuple[str, str], dict]:
    report = json.load(open(path))
    out = {}
    for case in report["cases"]:
        for c in case.get("cells") or []:
            out[(case["id"], f"{c['sheet']}!{c['cell']}")] = c
    return out


def value(c: dict | None, other: dict | None) -> float | None:
    """The cell's actual number, or Excel's where the cell is bit-exact (absent)."""
    src = c if c is not None else other
    if src is None:
        return None
    side = c["actual"] if c is not None else src["expected"]
    return side.get("v") if side.get("t") == "n" else None


def rel(a: float | None, b: float | None) -> float:
    if a is None or b is None:
        return math.nan
    if b == 0:
        return 0.0 if a == 0 else math.inf
    return abs(a - b) / abs(b)


def closeness(c: dict | None) -> str:
    return "pass" if c is None else str(c.get("closeness") or c.get("verdict"))


def main() -> None:
    main_cells, branch_cells = cells(sys.argv[1]), cells(sys.argv[2])
    moved = []
    for key in sorted(set(main_cells) | set(branch_cells)):
        m, b = main_cells.get(key), branch_cells.get(key)
        vm, vb = value(m, b), value(b, m)
        if vm == vb and closeness(m) == closeness(b):
            continue
        ref = (m or b)["expected"]
        excel = ref.get("v") if ref.get("t") == "n" else None
        math_v = ((m or b).get("math") or {}).get("v")
        moved.append(
            (
                key,
                (m or b)["formula"],
                closeness(m),
                closeness(b),
                rel(vm, excel),
                rel(vb, excel),
                rel(vm, math_v),
                rel(vb, math_v),
            )
        )
    tally: Counter[str] = Counter()
    for key, f, cm, cb, em, eb, mm, mb in moved:
        rank = lambda s: ORDER.index(s) if s in ORDER else len(ORDER)  # noqa: E731
        kind = "better" if rank(cb) < rank(cm) else "worse" if rank(cb) > rank(cm) else "same class"
        toward = "toward Excel" if eb < em else "away from Excel" if eb > em else "same distance"
        tally[f"{kind}, {toward}"] += 1
        print(f"{key[0]}!{key[1]}\t{f}\t{cm} -> {cb}\tExcel {em:.1e} -> {eb:.1e}\tmath {mm:.1e} -> {mb:.1e}")
    print(f"\n{len(moved)} cells moved")
    for k, n in sorted(tally.items()):
        print(f"  {k}: {n}")


if __name__ == "__main__":
    main()
