"""Coverage audit of the F family's Excel evidence (T-4857).

Bins every oracle-backed cell of F.DIST (cdf and pdf), F.DIST.RT, FDIST, F.INV, F.INV.RT, FINV,
F.TEST and FTEST in eval/suites/*/cases and eval/accuracy/tables into the equivalence classes of
`fclasses.py` (argument kinds per position; numeric classes of each argument; the kernel branch,
density regime and quantile path the point takes) and their pairs, and writes the counts as JSON.

    cd tools && uv run python ../docs/research/t4857/audit.py --out ../docs/research/t4857/coverage.json

`--exclude SUBSTR` leaves out cases whose id contains SUBSTR (the "before" view of this task's own
probes: `--exclude t4857`). `report.py` turns two such files into the before/after tables.
xlsx suites (ironcalc-calc's F_DIST.xlsx and F_TEST.xlsx, calipers, corpus-bench, poi) are not read:
tools never parse xlsx values (CLAUDE.md), and the corpora are not checked out here.
"""

from __future__ import annotations

import argparse
import json
import math
from collections import Counter, defaultdict
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

import cells as C
import fclasses as F

TAIL = ("F.DIST", "F.DIST.RT", "FDIST")
PDF = ("F.DIST/pdf",)
INV = ("F.INV", "F.INV.RT", "FINV")
TEST = ("F.TEST", "FTEST")
GROUPS = {"tail": TAIL, "pdf": PDF, "inv": INV}
PAIRS = {
    "tail": (
        ("df1", "df2"),
        ("xmag", "df1"),
        ("xmag", "df2"),
        ("xpos", "df1"),
        ("xpos", "df2"),
        ("kernel", "xpos"),
    ),
    "pdf": (
        ("df1", "df2"),
        ("xmag", "df1"),
        ("xmag", "df2"),
        ("xpos", "df1"),
        ("xpos", "df2"),
        ("density", "xpos"),
    ),
    "inv": (("df1", "df2"), ("p", "df1"), ("p", "df2"), ("path", "p")),
}


def _position(point: tuple[float, float, float]) -> str:
    x, d1, d2 = point
    cdf, sf, _how = F.tails(x, d1, d2)
    return F.x_position(cdf, sf)


def numeric_classes(function: str, numbers: list[float], positions: dict) -> dict[str, str]:
    """The numeric classes of one call whose arguments are all numbers."""
    out: dict[str, str] = {}
    if function in INV:
        p, r1, r2 = numbers[:3]
        out["p"], out["df1"], out["df2"] = F.p_class(p), F.df_class(r1), F.df_class(r2)
        d1, d2 = math.trunc(r1), math.trunc(r2)
        if 0 <= p <= 1 and 1 <= d1 <= F.F_DF_MAX and 1 <= d2 <= F.F_DF_MAX:
            out["path"] = F.quantile_path(p, d1, d2, function != "F.INV")
        return out
    x, r1, r2 = numbers[:3]
    out["xmag"], out["df1"], out["df2"] = F.x_magnitude(x), F.df_class(r1), F.df_class(r2)
    d1, d2 = math.trunc(r1), math.trunc(r2)
    valid = 1 <= d1 <= F.F_DF_MAX and 1 <= d2 <= F.F_DF_MAX
    if valid and x >= F.DBL_MIN and math.isfinite(d1 * x):
        out["xpos"] = positions.get((x, d1, d2), "unknown")
        out["centre"] = F.centre_class(x, d1, d2)
        if function in PDF:
            out["density"] = F.density_class(d1, d2, x)
        else:
            out["kernel"] = F.kernel_class(d1, d2, x)
    elif valid and function not in PDF:
        out["kernel"] = F.kernel_class(d1, d2, x) if x > 0 else "x<=0"
    return out


def audit(exclude: list[str], pending: tuple[str, ...] = ()) -> dict:
    cells = [c for c in C.read_cells(pending=pending) if not any(e in c.case for e in exclude)]
    points = set()
    for cell in cells:
        for call in cell.calls:
            if call.numbers and call.function not in TEST and len(call.numbers) >= 3:
                x, r1, r2 = call.numbers[:3]
                d1, d2 = math.trunc(r1), math.trunc(r2)
                if (
                    call.function not in INV
                    and 1 <= d1 <= F.F_DF_MAX
                    and 1 <= d2 <= F.F_DF_MAX
                    and x >= F.DBL_MIN
                    and math.isfinite(d1 * x)
                ):
                    points.add((x, float(d1), float(d2)))
    ordered = sorted(points)
    with ProcessPoolExecutor() as pool:
        positions = dict(zip(ordered, pool.map(_position, ordered, chunksize=256), strict=True))
    kinds: Counter = Counter()
    single: dict[str, Counter] = defaultdict(Counter)
    pairs: dict[str, Counter] = defaultdict(Counter)
    per_function: Counter = Counter()
    for cell in cells:
        for call in cell.calls:
            per_function[call.function] += 1
            for i, kind in enumerate(call.kinds):
                kinds[f"{call.function}|{i}|{kind}"] += 1
            group = next((g for g, fs in GROUPS.items() if call.function in fs), None)
            if group is None or not call.numbers or len(call.numbers) < 3:
                continue
            classes = numeric_classes(call.function, call.numbers, positions)
            for dim, cls in classes.items():
                single[f"{group}|{dim}"][cls] += 1
                single[f"{call.function}|{dim}"][cls] += 1
            for left, right in PAIRS[group]:
                if left in classes and right in classes:
                    pairs[f"{group}|{left}x{right}"][f"{classes[left]}|{classes[right]}"] += 1
    return {
        "cells": len(cells),
        "per_function": dict(per_function),
        "kinds": dict(kinds),
        "single": {k: dict(v) for k, v in single.items()},
        "pairs": {k: dict(v) for k, v in pairs.items()},
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", required=True)
    parser.add_argument("--exclude", action="append", default=[])
    parser.add_argument("--pending", action="append", default=[], help="count this case's cells before its fill")
    args = parser.parse_args()
    result = audit(args.exclude, tuple(args.pending))
    Path(args.out).write_text(json.dumps(result, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    print(f"{result['cells']} cells -> {args.out}")


if __name__ == "__main__":
    main()
