"""T-4435: the eval cells where ours moves away from Excel, with how far Excel itself is from the math in units of
max(1, |ln value|) * 2^-53 (the double floor a kernel can reach). A cell whose Excel is within FLOOR units of the
math is "near the math": moving away from it costs a match the owner prefers to keep.

    python3 docs/research/t4435/lost_excel.py MAIN_REPORT.json BRANCH_REPORT.json
"""

from __future__ import annotations

import json
import math
import sys

FLOOR = 4.0
ORDER = ["pass", "ulp1", "ulp4", "digits15", "residue13", "r3_math", "visible_numeric", "visible_other"]


def cells(path: str) -> dict[tuple[str, str], dict]:
    out = {}
    for case in json.load(open(path))["cases"]:
        for c in case.get("cells") or []:
            out[(case["id"], f"{c['sheet']}!{c['cell']}")] = {**c, "formula": c.get("formula", "")}
    return out


def num(v: dict | None) -> float | None:
    return v.get("v") if isinstance(v, dict) and v.get("t") == "n" else None


def units(a: float | None, m: float | None) -> float:
    if a is None or m is None or m == 0:
        return math.nan
    return abs(a - m) / abs(m) / (max(1.0, abs(math.log(abs(m)))) * 2.0**-53)


def main() -> None:
    base, branch = cells(sys.argv[1]), cells(sys.argv[2])
    rows = []
    for key in sorted(base.keys() | branch.keys()):
        b, a = base.get(key), branch.get(key)
        ref = a or b
        excel, m = num(ref["expected"]), num(ref.get("math"))
        before = num(b["actual"]) if b else excel
        after = num(a["actual"]) if a else excel
        if before is None or after is None or excel is None or before == after:
            continue
        if abs(after - excel) <= abs(before - excel):
            continue
        cb = b.get("closeness", "pass") if b else "pass"
        ca = a.get("closeness", "pass") if a else "pass"
        rows.append((key, ref["formula"], cb, ca, units(excel, m), units(before, m), units(after, m)))
    near = [r for r in rows if r[4] <= FLOOR]
    print(
        f"{len(rows)} cells move away from Excel; on {len(near)} Excel is within {FLOOR:g} units of the math"
    )
    print("| cell | formula | class | Excel units | ours units before -> after |")
    print("|---|---|---|---:|---:|")
    for (case, cell), f, cb, ca, ue, ub, ua in sorted(rows, key=lambda r: (r[4] > FLOOR, r[0])):
        tag = " (near the math)" if ue <= FLOOR else ""
        print(
            f"| {case.split('/')[-1]}!{cell}{tag} | `{f}` | {cb} -> {ca} | {ue:.2g} | {ub:.2g} -> {ua:.2g} |"
        )


if __name__ == "__main__":
    main()
