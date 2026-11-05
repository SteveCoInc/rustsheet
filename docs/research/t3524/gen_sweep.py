"""T-3524: a fresh sweep of CUMPRINC and CUMIPMT over the domain of the geometric-principal selectors
(functions/financial/cumulative.rs `GEOMETRIC_MIN_LEAD`, `LONG_GEOMETRIC_MIN_LEAD`), decision 0188 R4.2.
Deterministic (seed 3524). Writes eval/suites/probes-numeric/cases/t3524-cumprinc-lead-fresh-sweep.case.json.

The selectors pick the geometric sum by the leading exponent `lead = (nper-end+1)·L`, `L = ln(1+rate)`, so the sweep
draws the lead, not the periods: 500 rows in four blocks of 125 (rows 1-125 short type 0, 126-250 short type 1,
251-375 long type 0, 376-500 long type 1; short is 1..32 periods, end - start < 32, long 33..3000 periods
log-uniform). Each row draws the rate log-uniform in [1E-4, 2] (15 significant digits), the lead log-uniform in
[1E-3, 40] (both cuts, 0.3 and 3.6, and both fit brackets, 0.095..0.398 and 2.9..4.3, inside), so
`nper - end + 1 = max(1, round(lead/L))`, a start of 1 one time in five and otherwise log-uniform in [1, 1000], and
pv log-uniform in [1, 1E7] (10 significant digits). A: CUMPRINC, B: CUMIPMT, the same arguments.
"""

from __future__ import annotations

import json
import math
import random
from pathlib import Path

SEED = 3524
OUT = (
    Path(__file__).resolve().parents[3]
    / "eval/suites/probes-numeric/cases/t3524-cumprinc-lead-fresh-sweep.case.json"
)


def sig(v: float, digits: int) -> str:
    return f"{v:.{digits}g}".replace("e", "E")


def log_uniform(rng: random.Random, lo: float, hi: float) -> float:
    return math.exp(rng.uniform(math.log(lo), math.log(hi)))


def row(rng: random.Random, long: bool, period_start: int) -> str:
    rate_text = sig(log_uniform(rng, 1e-4, 2.0), 15)
    log_growth = math.log1p(float(rate_text))
    lead = log_uniform(rng, 1e-3, 40.0)
    tail = max(1, round(lead / log_growth))
    count = round(log_uniform(rng, 33, 3000)) if long else rng.randint(1, 32)
    start = 1 if rng.random() < 0.2 else round(log_uniform(rng, 1, 1000))
    end = start + count - 1
    nper = end + tail - 1
    pv = sig(log_uniform(rng, 1.0, 1e7), 10)
    return f"{rate_text},{nper},{pv},{start},{end},{period_start}"


def main() -> None:
    rng = random.Random(SEED)
    cells: dict[str, str] = {}
    blocks = [(False, 0), (False, 1), (True, 0), (True, 1)]
    for block, (long, period_start) in enumerate(blocks):
        for i in range(1, 126):
            r = block * 125 + i
            args = row(rng, long, period_start)
            cells[f"A{r}"] = f"=CUMPRINC({args})"
            cells[f"B{r}"] = f"=CUMIPMT({args})"
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-3524 fresh sweep (decision 0188 R4.2): CUMPRINC (A) and CUMIPMT (B) with the leading exponent "
            "(nper-end+1)*ln(1+rate) log-uniform in [1E-3, 40]; rows 1-125 short type 0, 126-250 short type 1, "
            "251-375 long type 0, 376-500 long type 1 (docs/research/t3524/gen_sweep.py, seed 3524)"
        ),
        "tags": ["M9", "financial", "probe", "T-3524"],
        "expected": {},
        "sheets": [{"name": "S", "cells": cells}],
    }
    OUT.write_text(json.dumps(case, indent=2) + "\n")


if __name__ == "__main__":
    main()
