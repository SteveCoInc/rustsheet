"""T-4497: the a = 1 cut sweep of F.TEST and the df1 = 2 / df2 = 2 F quantile search.

Writes eval/suites/probes-numeric/cases/t4497-f-a1-cut-sweep.case.json (round 1, Calc!A:C) and,
with --dense LO HI, a second case t4497-f-a1-cut-dense.case.json of 60 points log-uniform in
[LO, HI] (round 2, Calc!A:C again), plus docs/research/t4497/points.tsv: one row per cell with the
crossing x, the shape b, the 50-digit math value and the hold-out flag (decision 0188 R4.1).

Unswapped a = 1 tail x of each call (fisher.rs `unoriented_shape_one_tail`):
- F.TEST({-1,0,1}, big): the small sample's variance is 1 and df 2 (a = 1); the big sample is
  +-c pairs (and a 0 when its count is odd), df 2b, so f = var(big) and x = 2/(2b f + 2).
- F.INV(p,2,df2): the lower tail I_u(1, df2/2) is searched (p < 0.5), u = 1 - (1-p)^(2/df2).
- F.INV.RT(p,df1,2): the upper tail I_v(1, df1/2) is searched (p < 0.5), v = 1 - (1-p)^(2/df1).

Usage (tools/): uv run python ../docs/research/t4497/mkcase.py [--dense LO HI]
"""

from __future__ import annotations

import hashlib
import json
import math
import random
import sys
from pathlib import Path

import mpmath

mpmath.mp.dps = 60
ROOT = Path(__file__).resolve().parents[3]
CASES = ROOT / "eval" / "suites" / "probes-numeric" / "cases"
TSV = Path(__file__).resolve().parent / "points.tsv"
X_LO, X_HI = 4e-4, 0.2
HOLDOUT_PERCENT = 30


def lit(v: float) -> str:
    """A 15-significant-digit literal, so Excel's parser keeps it as written."""
    return f"{v:.15g}"


def holdout(case: str, cell: str) -> bool:
    """Fixed before any scoring: ~30% of cells by a hash of case and cell (decision 0188 R4.1)."""
    h = int(hashlib.sha256(f"{case}!{cell}".encode()).hexdigest()[:8], 16)
    return h % 100 < HOLDOUT_PERCENT


def two_pass_variance(values: list[float]) -> float:
    mean = 0.0
    for v in values:
        mean += v
    mean /= len(values)
    total = 0.0
    for v in values:
        d = v - mean
        total += d * d
    return total / (len(values) - 1)


def ftest(rng: random.Random, x: float) -> tuple[str, float, float, mpmath.mpf]:
    top = min(50.0, 1.0 / x - 1.0)
    halves = [k / 2 for k in range(3, int(2 * top) + 1)]
    b = rng.choice(halves)
    n = int(2 * b + 1)
    m = n // 2
    f = (1.0 / x - 1.0) / b
    c = float(lit(math.sqrt(f * (n - 1) / (2 * m))))
    big = [c if i % 2 else -c for i in range(2 * m)] + ([0.0] if n % 2 else [])
    formula = "=F.TEST({-1,0,1},{" + ",".join(lit(v) if v else "0" for v in big) + "})"
    fd = two_pass_variance(big) / 1.0
    xd = 2.0 / (2 * b * fd + 2.0)
    cm = mpmath.mpf(lit(c))
    fm = 2 * m * cm**2 / (n - 1)
    xm = 2 / (2 * b * fm + 2)
    u = 1 - (1 - xm) ** b
    p = 2 * min(u, 1 - u)
    return formula, xd, b, p


def quantile(rng: random.Random, x: float, right: bool) -> tuple[str, float, float, mpmath.mpf]:
    # p < 0.5 keeps the a = 1 tail the searched one: x <= 1 - 0.5^(1/b).
    dfs = [d for d in range(1, 51) if d != 2 and x < 1 - 0.5 ** (2 / d) - 1e-9]
    d = rng.choice(dfs)
    b = d / 2
    p = float(lit(float(1 - (1 - mpmath.mpf(x)) ** b)))
    pm = mpmath.mpf(lit(p))
    xm = 1 - (1 - pm) ** (1 / mpmath.mpf(b))
    if right:
        formula = f"=F.INV.RT({lit(p)},{d},2)"
        q = 2 * (1 - xm) / (d * xm)
    else:
        formula = f"=F.INV({lit(p)},2,{d})"
        q = d * xm / (2 * (1 - xm))
    return formula, float(xm), b, q


def build(name: str, seed: int, count: int, lo: float, hi: float, note: str) -> list[list[str]]:
    rng = random.Random(seed)
    cells: dict[str, str] = {}
    rows = []
    for col, kind in (("A", "F.TEST"), ("B", "F.INV"), ("C", "F.INV.RT")):
        for i in range(1, count + 1):
            x = math.exp(rng.uniform(math.log(lo), math.log(hi)))
            if kind == "F.TEST":
                formula, xd, b, mv = ftest(rng, x)
            else:
                formula, xd, b, mv = quantile(rng, x, kind == "F.INV.RT")
            cell = f"{col}{i}"
            cells[cell] = formula
            rows.append(
                [name, cell, kind, repr(xd), repr(b), mpmath.nstr(mv, 50), "1" if holdout(name, cell) else "0"]
            )
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            f"T-4497: {note} The a = 1 tail x of each call, its shape b, 50-digit math values and the "
            "hold-out flag are in docs/research/t4497/points.tsv (docs/research/t4497/mkcase.py)."
        ),
        "tags": ["M9", "probe", "statistics", "T-4497"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    (CASES / f"{name}.case.json").write_text(json.dumps(case, indent=1) + "\n", encoding="utf-8")
    return rows


def main() -> None:
    rows = build(
        "t4497-f-a1-cut-sweep",
        4497,
        160,
        X_LO,
        X_HI,
        "F.TEST (A), F.INV(p,2,df2) (B) and F.INV.RT(p,df1,2) (C) whose unswapped a = 1 tail has x "
        "log-uniform over 4E-4..0.2 (b 0.5..50), to place A1_EXPM1_MAX_X.",
    )
    if len(sys.argv) == 4 and sys.argv[1] == "--dense":
        lo, hi = float(sys.argv[2]), float(sys.argv[3])
        rows += build(
            "t4497-f-a1-cut-dense",
            44970,
            60,
            lo,
            hi,
            f"round 2, the same three calls with x log-uniform over {lit(lo)}..{lit(hi)} around the cut "
            "round 1 suggests.",
        )
    with TSV.open("w", encoding="utf-8") as out:
        out.write("case\tcell\tfunction\tx\tb\tmath\tholdout\n")
        for row in rows:
            out.write("\t".join(row) + "\n")


if __name__ == "__main__":
    main()
