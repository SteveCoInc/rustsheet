"""T-3953: F right tail on bratio's bfrac branch at a tiny kernel argument (for the fit task T-3954).

Rows: d1 log-uniform in [80, 20000] (one row in five a non-integer with two decimals, which Excel truncates), d2
log-uniform in [1, 200] (integers), X chosen so that x0 = d2/(d1*X + d2) is log-uniform in [1E-280, 1E-6] (X written as
a 12-significant-digit literal). A row is kept only where the engine's dispatch on F's kernel arguments is bfrac
(`bratio::dispatch`, mirrored in `branch` below) and the 50-digit tail is a normal double above 1E-300.
Per row r, four cells:
  A: =F.DIST.RT(X,d1,d2)          B: =FDIST(X,d1,d2)
  C: =BETA.DIST(d2/(d1t*X+d2),d2/2,d1t/2,TRUE)   (v formed in the sheet; d1t = TRUNC(d1))
  D: =BETA.DIST(v15,d2/2,d1t/2,TRUE)             (v15: the engine's x0 = 1/(1+d1t*X/d2) at 15 significant digits,
                                                  the most a formula literal carries)
Hold-out (decision 0188 R4.1), fixed here before any scoring: a row is hold-out when the first byte of
sha256("t3953/<d1 text>/<d2>/<X text>") is below 77 (77/256 = 30%). Each cell's hypothesis carries /tune/ or /hold/.
Usage (repo root): uv run --with mpmath python docs/research/t3953/mksweep.py [--write]"""

from __future__ import annotations

import hashlib
import json
import math
import random
import sys
from pathlib import Path

import mpmath

mpmath.mp.dps = 60
CASE = Path("eval/suites/probes-numeric/cases/t3953-f-tail-bfrac-tiny.case.json")
SEED = 3953
ROWS = 1200
HOLD_BYTE = 77


def branch(d1: float, d2: float, x_lit: float) -> tuple[str, float, float, float]:
    """bratio's branch for F's upper tail as the engine forms it (fisher.rs `f_beta_arguments`, then
    bratio/mod.rs `dispatch` for min(a, b) > 1), with (a0, b0, x0)."""
    a, b = d1 / 2.0, d2 / 2.0
    scaled = d1 * x_lit
    x = 1.0 / (1.0 + d2 / scaled)
    y = 1.0 / (1.0 + scaled / d2)
    if min(a, b) <= 1.0 or x == 0.0 or y == 0.0:
        return "other", a, b, x
    lam = (a + b) * y - b if a > b else a - (a + b) * x
    a0, b0, x0 = (b, a, y) if lam < 0 else (a, b, x)
    lam = abs(lam)
    if b0 < 40.0:
        return "not-bfrac(b0<40)", a0, b0, x0
    if (a0 > b0 and (b0 <= 100.0 or lam > b0 * 0.03)) or (a0 <= b0 and (a0 <= 100.0 or lam > a0 * 0.03)):
        return "bfrac", a0, b0, x0
    return "basym", a0, b0, x0


def lit(v: float, digits: int) -> str:
    """A formula literal of v at `digits` significant digits, Excel's exponent spelling."""
    s = f"{v:.{digits - 1}e}"
    mant, exp = s.split("e")
    mant = mant.rstrip("0").rstrip(".")
    return f"{mant}E{int(exp)}" if int(exp) else mant


def tail(d1t: int, d2: int, x_lit: mpmath.mpf) -> mpmath.mpf:
    """P(F > X) at 50+ digits: I_v(d2/2, d1/2), v = d2/(d1 X + d2)."""
    v = mpmath.mpf(d2) / (d1t * x_lit + d2)
    return mpmath.betainc(mpmath.mpf(d2) / 2, mpmath.mpf(d1t) / 2, 0, v, regularized=True)


def main() -> None:
    rng = random.Random(SEED)
    cells: dict[str, str] = {}
    hyp: dict[str, str] = {}
    seen: set[tuple[str, int, str]] = set()
    counts = {"tune": 0, "hold": 0, "rejected": 0}
    row = 0
    while row < ROWS:
        d1 = math.exp(rng.uniform(math.log(80), math.log(20000)))
        d1_text = f"{d1:.2f}" if rng.random() < 0.2 else str(round(d1))
        d1t = math.trunc(float(d1_text))
        d2 = math.trunc(math.exp(rng.uniform(0, math.log(200))))
        x0_target = 10 ** rng.uniform(-280, -6)
        x_text = lit(d2 * (1 / x0_target - 1) / d1t, 12)
        x_val = float(x_text)
        br, a0, b0, x0 = branch(float(d1t), float(d2), x_val)
        if br != "bfrac" or (d1_text, d2, x_text) in seen:
            counts["rejected"] += 1
            continue
        t = tail(d1t, d2, mpmath.mpf(x_val))
        if not t > mpmath.mpf("1e-300"):
            counts["rejected"] += 1
            continue
        seen.add((d1_text, d2, x_text))
        key = f"t3953/{d1_text}/{d2}/{x_text}"
        split = "hold" if hashlib.sha256(key.encode()).digest()[0] < HOLD_BYTE else "tune"
        counts[split] += 1
        row += 1
        v15 = lit(x0, 15)
        base = f"t3953/{split}/d1={d1_text}/d2={d2}/X={x_text}/a0={a0:g}/b0={b0:g}/x0={x0:.6g}"
        forms = {
            "A": (f"=F.DIST.RT({x_text},{d1_text},{d2})", "fdistrt"),
            "B": (f"=FDIST({x_text},{d1_text},{d2})", "fdist"),
            "C": (f"=BETA.DIST({d2}/({d1t}*{x_text}+{d2}),{d2}/2,{d1t}/2,TRUE)", "beta-sheet-v"),
            "D": (f"=BETA.DIST({v15},{d2}/2,{d1t}/2,TRUE)", "beta-literal-v"),
        }
        for col, (formula, kind) in forms.items():
            cells[f"{col}{row}"] = formula
            hyp[f"Calc!{col}{row}"] = f"{base}/{kind}"
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            f"T-3953: F right tail on bratio's bfrac branch at a tiny kernel argument, {ROWS} rows (seed {SEED}, "
            "docs/research/t3953/mksweep.py): d1 log-uniform in [80, 20000] (a fifth non-integer), d2 log-uniform in "
            "[1, 200], x0 = d2/(d1*X+d2) log-uniform in [1E-280, 1E-6], kept where the engine dispatches bfrac and the "
            "tail is above 1E-300. Columns: A F.DIST.RT, B FDIST, C BETA.DIST(v formed in the sheet, d2/2, TRUNC(d1)/2), "
            "D BETA.DIST(the engine's x0 at 15 digits, ...). Hold-out fixed before scoring (decision 0188 R4.1): rows "
            f"with sha256('t3953/<d1>/<d2>/<X>')[0] < {HOLD_BYTE}, {counts['hold']} rows, marked /hold/ in each "
            "hypothesis; never used for fitting."
        ),
        "tags": ["M9", "probe", "statistics", "T-3953", "T-3954"],
        "expected": {},
        "hypotheses": hyp,
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    if "--write" in sys.argv:
        CASE.write_text(json.dumps(case, indent=1) + "\n")
    print(counts, len(cells))


if __name__ == "__main__":
    main()
