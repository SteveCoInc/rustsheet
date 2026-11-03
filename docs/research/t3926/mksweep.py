"""T-3926: F.DIST density where Y = density*x is subnormal (for the fit task T-2647).

Per df pair, 30 x values whose exact Y/2^-1074 is log-uniform in [1, 2^52]: 15 powers of two (`2^-k`, where Y/x is
exact in double) and 15 general literals of at most 10 significant digits. The tune case and the hold-out case use
different seeds and share no (df pair, x).
Usage (repo root): uv run --with mpmath python docs/research/t3926/mksweep.py [--write]"""

from __future__ import annotations

import json
import math
import random
import sys
from pathlib import Path

import mpmath

mpmath.mp.dps = 60
DIR = Path("eval/suites/probes-numeric/cases")
PAIRS = [(3, 5), (5, 3), (3, 1000), (4, 1000), (5, 130), (10, 1000), (3, 3), (6, 20)]
TINY = mpmath.mpf(2) ** -1074
SEEDS = {"": 3926, "-hold": 39260}


def y_of(x: mpmath.mpf, d1: int, d2: int) -> mpmath.mpf:
    """Y = density * x, exact."""
    a, b = mpmath.mpf(d1) / 2, mpmath.mpf(d2) / 2
    return (mpmath.mpf(d1) / d2) ** a * x**a * (1 + mpmath.mpf(d1) * x / d2) ** (-(a + b)) / mpmath.beta(a, b)


def solve_x(target: mpmath.mpf, d1: int, d2: int) -> mpmath.mpf:
    """x with Y(x) = target (Y rises with x near 0)."""
    lo, hi = mpmath.mpf(-1000), mpmath.mpf(0)  # ln x; Y(e^hi) >> any subnormal
    for _ in range(300):
        mid = (lo + hi) / 2
        if y_of(mpmath.exp(mid), d1, d2) < target:
            lo = mid
        else:
            hi = mid
    return mpmath.exp(lo)


def build(suffix: str, used: set) -> tuple[dict, dict, dict]:
    rng = random.Random(SEEDS[suffix])
    cells, hyp, ref = {}, {}, {}
    row = 1
    for d1, d2 in PAIRS:
        pts: list[tuple[str, mpmath.mpf]] = []
        for kind in ("pow2", "gen"):
            while sum(1 for k, _, _ in pts if k == kind) < 15:
                q = mpmath.mpf(2) ** (rng.uniform(0, 52))
                x = solve_x(q * TINY, d1, d2)
                if kind == "pow2":
                    k = round(-float(mpmath.log(x, 2)))
                    text, xv = f"2^-{k}", mpmath.mpf(2) ** -k
                else:
                    text = f"{float(x):.9g}".replace("e-0", "E-").replace("e-", "E-")
                    xv = mpmath.mpf(float(text.replace("E", "e")))
                yq = y_of(xv, d1, d2) / TINY
                if not (1 <= yq <= 2**52) or (d1, d2, text) in used:
                    continue
                used.add((d1, d2, text))
                pts.append((kind, text, yq))
                cell = f"A{row}"
                cells[cell] = f"=F.DIST({text},{d1},{d2},FALSE)"
                hyp[f"Calc!{cell}"] = f"t3926/d1={d1}/d2={d2}/x={text}/{kind}/Y={mpmath.nstr(yq, 8)}x2^-1074"
                ref[f"Calc!{cell}"] = mpmath.nstr(y_of(xv, d1, d2) / xv, 50, min_fixed=1, max_fixed=0)
                row += 1
    return cells, hyp, ref


def main() -> None:
    used: set = set()
    for suffix, label in (("", "tune"), ("-hold", "hold-out, never used for fitting")):
        cells, hyp, ref = build(suffix, used)
        case = {
            "format": "rustsheet-case/v1",
            "description": (
                f"T-3926 ({label}): F.DIST(x,d1,d2,FALSE) where Y = density*x is subnormal, df pairs {PAIRS}, per pair 30 x "
                "with exact Y/2^-1074 log-uniform in [1, 2^52], half powers of two (2^-k), half general literals of at "
                f"most 10 significant digits; seed {SEEDS[suffix]} (docs/research/t3926/mksweep.py), {len(cells)} cells."
                + (" Never used for fitting." if suffix else "")
            ),
            "tags": ["M9", "probe", "statistics", "T-3926", "T-2647"],
            "expected": {},
            "hypotheses": hyp,
            "sheets": [{"name": "Calc", "cells": cells}],
        }
        if "--write" in sys.argv:
            (DIR / f"t3926-f-density-subnormal-y{suffix}.case.json").write_text(json.dumps(case, indent=1) + "\n")
        print(suffix or "tune", len(cells))


if __name__ == "__main__":
    main()
