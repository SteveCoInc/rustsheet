"""T-4434: write probes-numeric/t4434-huge-shape-outside-large-shape-beta, F.DIST.RT, F.DIST and BETA.DIST where the
incomplete beta has one huge shape outside large_shape_beta's bounds.

Run from the repository root with the tools venv: tools/.venv/bin/python -I docs/research/t4434/gen_probe.py

Seed 4434 (random.Random). The points are fixed by the seed alone, before any value is scored. Layout, rows 1:300:
  A  F.DIST.RT(x, df1, df2), region 1: df1 an integer log-uniform in [3, 79], df2 an integer log-uniform in
     [1E4, 2E6] (a0 = df2/2 below large_shape_beta's MIN_SHAPE 1E6), x solved so the upper tail is log-uniform in
     [1E-200, 0.5].
  B  F.DIST.RT(x, df1, df2), region 2: df1 an integer log-uniform in [80, 400] (b0 = df1/2 at or above
     MAX_SECOND_SHAPE 40), df2 an integer log-uniform in [1E5, 1E10], x = r * df2/(df2 - 2), r log-uniform in [0.7, 3].
  C  F.DIST(x, df1, df2, TRUE) at row's B point, rows 1:150.
  D  BETA.DIST(x, a, b, TRUE), rows 1:150: a log-uniform in [20, 200] and b log-uniform in [5E4, 5E9], both to 6
     significant digits, x = r * a/b with r uniform in [0.7, 1.5].

The degrees of freedom are integers, so Excel's truncation of them is moot. Every x is rounded to 12 significant
digits: Excel reads a formula literal at 15 significant digits (numbers.md), so a literal of at most 15 digits is
read as exactly the double Python parses from it. The literal is `repr` of that double, which round-trips (and so
does its 17-digit form).
"""

from __future__ import annotations

import json
import math
import random
import sys
from pathlib import Path

import mpmath as mp

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE))
import mathref  # noqa: E402

STEM = "t4434-huge-shape-outside-large-shape-beta"
SEED = 4434
ROWS = 300
HALF = 150


def round_sig(v: float, digits: int) -> float:
    return float(f"{v:.{digits - 1}e}")


def lit(v: float) -> str:
    """The shortest literal that round-trips to v, at most 15 significant digits."""
    s = repr(v)
    if s.endswith(".0"):
        s = s[:-2]
    s = s.replace("e+", "E").replace("e-", "E-")
    assert float(s) == v and float(f"{float(s):.17g}") == v
    return s


def log_uniform(rng: random.Random, lo: float, hi: float) -> float:
    return math.exp(rng.uniform(math.log(lo), math.log(hi)))


def _log_tail(x: float, d1: int, d2: int) -> float:
    """log10 of F.DIST.RT(x, d1, d2) at 240 digits, enough for tails down to 1E-220 (the search only)."""
    m = mp.mpf(d1) * mp.mpf(x)
    with mp.workdps(260):
        q = mathref._at(mp.mpf(d1) / 2, mp.mpf(d2) / 2, m, m + d2, True, 240)
        return float(mp.log10(q)) if q > mp.mpf(10) ** -225 else -1000.0


def solve_x(target: float, d1: int, d2: int) -> float:
    """x where log10 F.DIST.RT(x, d1, d2) = target, by bisection on log x (the tail falls with x)."""
    lo, hi = math.log(0.05), math.log(5000.0)
    for _ in range(45):
        mid = (lo + hi) / 2
        if _log_tail(math.exp(mid), d1, d2) > target:
            lo = mid
        else:
            hi = mid
    return round_sig(math.exp((lo + hi) / 2), 12)


def points() -> dict[str, list[tuple]]:
    rng = random.Random(SEED)
    region1 = []
    for _ in range(ROWS):
        d1 = round(log_uniform(rng, 3, 79))
        d2 = round(log_uniform(rng, 1e4, 2e6))
        target = rng.uniform(math.log10(1e-200), math.log10(0.5))
        region1.append((d1, d2, target))
    region2 = []
    for _ in range(ROWS):
        d1 = round(log_uniform(rng, 80, 400))
        d2 = round(log_uniform(rng, 1e5, 1e10))
        r = log_uniform(rng, 0.7, 3.0)
        region2.append((round_sig(r * d2 / (d2 - 2), 12), d1, d2))
    beta = []
    for _ in range(HALF):
        a = round_sig(log_uniform(rng, 20, 200), 6)
        b = round_sig(log_uniform(rng, 5e4, 5e9), 6)
        beta.append((round_sig(rng.uniform(0.7, 1.5) * a / b, 12), a, b))
    # The tail searches come last so the draws above never depend on them.
    region1 = [(solve_x(t, d1, d2), d1, d2) for d1, d2, t in region1]
    return {"A": region1, "B": region2, "D": beta}


def formulas(pts: dict[str, list[tuple]]) -> dict[str, str]:
    cells = {}
    for row, (x, d1, d2) in enumerate(pts["A"], start=1):
        cells[f"A{row}"] = f"=F.DIST.RT({lit(x)},{d1},{d2})"
    for row, (x, d1, d2) in enumerate(pts["B"], start=1):
        cells[f"B{row}"] = f"=F.DIST.RT({lit(x)},{d1},{d2})"
        if row <= HALF:
            cells[f"C{row}"] = f"=F.DIST({lit(x)},{d1},{d2},TRUE)"
    for row, (x, a, b) in enumerate(pts["D"], start=1):
        cells[f"D{row}"] = f"=BETA.DIST({lit(x)},{lit(a)},{lit(b)},TRUE)"
    return cells


def main() -> None:
    pts = points()
    cells = formulas(pts)
    order = sorted(cells, key=lambda ref: (int(ref[1:]), ref[0]))
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4434: the incomplete beta at one huge shape outside large_shape_beta's bounds, written by "
            f"docs/research/t4434/gen_probe.py with seed {SEED}. A1:A300 F.DIST.RT region 1 (df1 3..79, df2 1E4..2E6, "
            "tail log-uniform in 1E-200..0.5); B1:B300 F.DIST.RT region 2 (df1 80..400, df2 1E5..1E10, x 0.7 to 3 "
            "times the mean); C1:C150 F.DIST(..,TRUE) at B's points; D1:D150 BETA.DIST(..,TRUE) at a 20..200, "
            "b 5E4..5E9, x 0.7 to 1.5 times a/b. Evidence for T-4435 (decision 0267 item 5 exception, decision 0334)."
        ),
        "tags": ["M9", "probe", "statistics", "T-4434"],
        "expected": {},
        "check": [f"Calc!{ref}" for ref in order],
        "sheets": [{"name": "Calc", "cells": {ref: cells[ref] for ref in order}}],
    }
    out = ROOT / "eval/suites/probes-numeric/cases" / f"{STEM}.case.json"
    out.write_text(json.dumps(case, indent=2) + "\n")
    print(f"wrote {out.relative_to(ROOT)}: {len(cells)} cells")


if __name__ == "__main__":
    main()
