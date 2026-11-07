"""Coverage tables from `audit.py`'s JSON (T-4857): per class and per pair, before and after.

    python3 docs/research/t4857/report.py BEFORE.json AFTER.json > docs/research/t4857/COVERAGE.md

A class or pair is *empty* with 0 Excel cells and *thin* with 1 or 2. Pairs are counted over the
classes that can carry a number (the error classes df < 1, df > 1E10, x < 0 and p outside [0, 1]
pair with nothing).
"""

from __future__ import annotations

import json
import sys

import fclasses as F
from audit import PAIRS

THIN = 3
DIMS = {
    "df1": F.DF_CLASSES,
    "df2": F.DF_CLASSES,
    "xmag": F.X_MAG_CLASSES,
    "xpos": F.X_POS_CLASSES,
    "p": F.P_CLASSES,
}
ERROR_CLASSES = {"lt1", "gt1e10", "neg", "gt1", "nan", "unknown"}
GROUP_DIMS = {
    "tail": ("xmag", "xpos", "df1", "df2", "kernel", "centre"),
    "pdf": ("xmag", "xpos", "df1", "df2", "density", "centre"),
    "inv": ("p", "df1", "df2", "path"),
}
PAIR_DIMS = PAIRS  # audit.py's pair tables, one definition
KINDS = (
    "number-literal",
    "number-arith",
    "ref-number",
    "numeric-text",
    "text",
    "empty-text",
    "bool-literal",
    "ref-numeric-text",
    "ref-text",
    "ref-empty-text",
    "ref-bool",
    "ref-empty",
    "ref-error",
    "error-literal",
    "error-call",
    "array-1",
    "array-multi",
    "range-1",
    "range-multi",
    "omitted",
    "formula",
)
ARITY = {
    "F.DIST": 4,
    "F.DIST.RT": 3,
    "FDIST": 3,
    "F.INV": 3,
    "F.INV.RT": 3,
    "FINV": 3,
    "F.TEST": 2,
    "FTEST": 2,
}


def classes_of(dim: str, *docs: dict, group: str) -> list[str]:
    if dim in DIMS:
        # A subnormal p or x reads as 0 in Excel (numbers.md "Subnormal results become 0"), so it pairs with nothing.
        if dim == "xmag":
            return [c for c in DIMS[dim] if c not in ("neg", "-0", "0", "subnormal", "dblmax")]
        return [c for c in DIMS[dim] if c not in ERROR_CLASSES and c != "subnormal"]
    seen: set[str] = set()
    for d in docs:
        seen |= set(d["single"].get(f"{group}|{dim}", {}))
    return sorted(seen - ERROR_CLASSES)


def pair_counts(doc: dict, group: str, left: str, right: str) -> dict[str, int]:
    return doc["pairs"].get(f"{group}|{left}x{right}", {})


def main() -> None:
    before, after = (json.load(open(p, encoding="utf-8")) for p in sys.argv[1:3])
    out = ["# F family coverage: Excel cells per class and pair, before and after T-4857", ""]
    out.append(f"Cells read: {before['cells']} before, {after['cells']} after (`audit.py`).")
    out.append("")
    out.append(
        "Only json cases are read (eval/suites and eval/accuracy/tables): the xlsx suites (ironcalc-calc's F_DIST.xlsx "
        "and F_TEST.xlsx, calipers, corpus-bench, poi) are excluded, so every empty or thin count below is an upper "
        "bound on the gap."
    )
    out.append("")
    out.append("## Summary")
    out.append("")
    out.append(
        "| Group | Table | Classes or pairs | Empty before | Thin (<3) before | Empty after | Thin after |"
    )
    out.append("|---|---|---|---|---|---|---|")
    details = []
    for group in GROUP_DIMS:
        for dim in GROUP_DIMS[group]:
            cls = classes_of(dim, before, after, group=group)
            b = before["single"].get(f"{group}|{dim}", {})
            a = after["single"].get(f"{group}|{dim}", {})
            row = [f"{group}", f"{dim}", str(len(cls))]
            for doc in (b, a):
                row.append(str(sum(1 for c in cls if doc.get(c, 0) == 0)))
                row.append(str(sum(1 for c in cls if 0 < doc.get(c, 0) < THIN)))
            out.append("| " + " | ".join(row) + " |")
            details.append((f"{group} / {dim}", [(c, b.get(c, 0), a.get(c, 0)) for c in cls]))
        for left, right in PAIR_DIMS[group]:
            lc, rc = (
                classes_of(left, before, after, group=group),
                classes_of(right, before, after, group=group),
            )
            b, a = pair_counts(before, group, left, right), pair_counts(after, group, left, right)
            keys = [f"{x}|{y}" for x in lc for y in rc]
            observed = {k for k in keys if a.get(k, 0) or b.get(k, 0)}
            row = [group, f"{left} x {right}", f"{len(keys)} ({len(observed)} reached)"]
            for doc in (b, a):
                row.append(str(sum(1 for k in keys if doc.get(k, 0) == 0)))
                row.append(str(sum(1 for k in keys if 0 < doc.get(k, 0) < THIN)))
            out.append("| " + " | ".join(row) + " |")
            details.append((f"{group} / {left} x {right}", [(k, b.get(k, 0), a.get(k, 0)) for k in keys]))
    out.append("")
    out.append(
        "A pair is *reached* when either view has a cell in it. Pairs no view reaches are mostly impossible"
    )
    out.append(
        "(a tail band no double x can reach at that df, ANALYSIS.md section 4) or outside this task's budget."
    )
    out.append("")
    out.append("## Argument kinds (cells per function, argument position and kind; before -> after)")
    out.append("")
    out.append("| Function | Arg | " + " | ".join(KINDS) + " |")
    out.append("|---|---|" + "---|" * len(KINDS))
    for fn, arity in ARITY.items():
        for i in range(arity):
            cells = []
            for kind in KINDS:
                b = before["kinds"].get(f"{fn}|{i}|{kind}", 0)
                a = after["kinds"].get(f"{fn}|{i}|{kind}", 0)
                cells.append(f"{b}" if a == b else f"{b}->{a}")
            out.append(f"| {fn} | {i + 1} | " + " | ".join(cells) + " |")
    out.append("")
    out.append("## Per-class and per-pair counts (before -> after; empty and thin rows only)")
    for title, rows in details:
        thin = [(k, b, a) for k, b, a in rows if b < THIN]
        if not thin:
            continue
        out.append("")
        out.append(f"### {title}")
        out.append("")
        out.append(", ".join(f"`{k}` {b}->{a}" for k, b, a in thin))
    print("\n".join(out))


if __name__ == "__main__":
    main()
