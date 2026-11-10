"""T-4435: every Gate A point whose engine value moves between two `rustsheet-eval accuracy` runs
(eval/runs/accuracy.json), with its class and its distance to Excel and to the math before and after.

    python3 docs/research/t4435/gate_a_moves.py MAIN_ACCURACY.json BRANCH_ACCURACY.json
"""

from __future__ import annotations

import json
import sys


def points(path: str) -> dict[str, dict]:
    return {p["id"]: p for p in json.load(open(path))["check"]["points"]}


def num(v: dict | None) -> float | None:
    return v.get("v") if isinstance(v, dict) and v.get("t") == "n" else None


def rel(a: float | None, b: float | None) -> float:
    if a is None or b is None:
        return float("nan")
    if b == 0:
        return 0.0 if a == 0 else float("inf")
    return abs(a - b) / abs(b)


def main() -> None:
    before, after = points(sys.argv[1]), points(sys.argv[2])
    moved = 0
    for pid in sorted(before.keys() & after.keys()):
        b, a = before[pid], after[pid]
        if b["engine"] == a["engine"] and b["class"] == a["class"] and b["standing"] == a["standing"]:
            continue
        moved += 1
        ex, mt = num(a.get("excel")), num(a.get("math"))
        eb, ea = num(b["engine"]), num(a["engine"])
        print(
            f"{pid}\t{a['formula']}\t{b['class']}/{b['standing']} -> {a['class']}/{a['standing']}\t"
            f"Excel {rel(eb, ex):.1e} -> {rel(ea, ex):.1e}\tmath {rel(eb, mt):.1e} -> {rel(ea, mt):.1e}\t"
            f"Excel vs math {rel(ex, mt):.1e}"
        )
    print(f"{moved} of {len(after)} points move ({len(before.keys() ^ after.keys())} in one run only)")


if __name__ == "__main__":
    main()
