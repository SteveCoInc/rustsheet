"""T-4349: the fresh sweep drawn after the short-range selector was frozen (decision 0188 R4.2).

The frozen selector (dump.py, select.py; chosen on the train side of the old cells only) takes a short range's
principal as the geometric sum where it has at most 4 periods, the rate is at least 1E-3 and the leading exponent
`lead = (nper-end+1)·ln(1+rate)` is below 0.095, as well as from a lead of 0.3 up (today's cut). It changes the
engine only there, so the sweep draws only there: 400 rows, rows 1-200 type 0 and 201-400 type 1. Each row draws the
rate log-uniform in [1E-3, 0.0997] (15 significant digits; `ln(1+rate)` below 0.095, so a lead of one period is in
range), the lead log-uniform in [ln(1+rate), 0.095], `nper - end + 1 = max(1, round(lead/L))`, 1 to 4 periods, a start
of 1 one time in five and otherwise log-uniform in [1, 1000], and pv log-uniform in [1, 1E7] (10 significant digits).
A: CUMPRINC, B: CUMIPMT, the same arguments. Deterministic (seed 4349). Writes
eval/suites/probes-numeric/cases/t4349-cumprinc-short-selector-fresh-sweep.case.json.
"""

from __future__ import annotations

import json
import math
import random
from pathlib import Path

SEED = 4349
OUT = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases/t4349-cumprinc-short-selector-fresh-sweep.case.json"
MAX_LEAD = 0.095


def sig(v: float, digits: int) -> str:
    return f"{v:.{digits}g}".replace("e", "E")


def log_uniform(rng: random.Random, lo: float, hi: float) -> float:
    return math.exp(rng.uniform(math.log(lo), math.log(hi)))


def row(rng: random.Random, period_start: int) -> str:
    rate_text = sig(log_uniform(rng, 1e-3, 0.0997), 15)
    log_growth = math.log1p(float(rate_text))
    lead = log_uniform(rng, log_growth, MAX_LEAD)
    tail = max(1, round(lead / log_growth))
    count = rng.randint(1, 4)
    start = 1 if rng.random() < 0.2 else round(log_uniform(rng, 1, 1000))
    end = start + count - 1
    nper = end + tail - 1
    pv = sig(log_uniform(rng, 1.0, 1e7), 10)
    return f"{rate_text},{nper},{pv},{start},{end},{period_start}"


def main() -> None:
    rng = random.Random(SEED)
    cells: dict[str, str] = {}
    for block, period_start in enumerate((0, 1)):
        for i in range(1, 201):
            r = block * 200 + i
            args = row(rng, period_start)
            cells[f"A{r}"] = f"=CUMPRINC({args})"
            cells[f"B{r}"] = f"=CUMIPMT({args})"
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4349 fresh sweep (decision 0188 R4.2), drawn after the short-range selector was frozen: CUMPRINC (A) "
            "and CUMIPMT (B) over 1 to 4 periods, rate log-uniform in [1E-3, 0.0997], the leading exponent "
            "(nper-end+1)*ln(1+rate) log-uniform in [ln(1+rate), 0.095]; rows 1-200 type 0, 201-400 type 1 "
            "(docs/research/t4349/gen_sweep.py, seed 4349)"
        ),
        "tags": ["M9", "financial", "probe", "T-4349"],
        "expected": {},
        "sheets": [{"name": "S", "cells": cells}],
    }
    OUT.write_text(json.dumps(case, indent=2) + "\n")


if __name__ == "__main__":
    main()
