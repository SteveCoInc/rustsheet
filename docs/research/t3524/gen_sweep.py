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

A second case (seed 35241), t3524-cumprinc-tiny-rate-sweep.case.json, is the same draw at rates log-uniform in
[1E-12, 1E-4], below the first sweep's (an Excel-fidelity review found the first sweep left them out). At such
rates a lead of 40 needs up to 4E13 periods; nper stays an integer and Excel's own bounds apply.
"""

from __future__ import annotations

import json
import math
import random
from pathlib import Path

SEED = 3524
TINY_SEED = 35241
CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases"
OUT = CASES / "t3524-cumprinc-lead-fresh-sweep.case.json"
TINY_OUT = CASES / "t3524-cumprinc-tiny-rate-sweep.case.json"


def sig(v: float, digits: int) -> str:
    return f"{v:.{digits}g}".replace("e", "E")


def log_uniform(rng: random.Random, lo: float, hi: float) -> float:
    return math.exp(rng.uniform(math.log(lo), math.log(hi)))


def row(rng: random.Random, long: bool, period_start: int, rates: tuple[float, float] = (1e-4, 2.0)) -> str:
    rate_text = sig(log_uniform(rng, *rates), 15)
    log_growth = math.log1p(float(rate_text))
    lead = log_uniform(rng, 1e-3, 40.0)
    tail = max(1, round(lead / log_growth))
    count = round(log_uniform(rng, 33, 3000)) if long else rng.randint(1, 32)
    start = 1 if rng.random() < 0.2 else round(log_uniform(rng, 1, 1000))
    end = start + count - 1
    nper = end + tail - 1
    pv = sig(log_uniform(rng, 1.0, 1e7), 10)
    return f"{rate_text},{nper},{pv},{start},{end},{period_start}"


def write(path: Path, seed: int, rates: tuple[float, float], description: str) -> None:
    rng = random.Random(seed)
    cells: dict[str, str] = {}
    blocks = [(False, 0), (False, 1), (True, 0), (True, 1)]
    for block, (long, period_start) in enumerate(blocks):
        for i in range(1, 126):
            r = block * 125 + i
            args = row(rng, long, period_start, rates)
            cells[f"A{r}"] = f"=CUMPRINC({args})"
            cells[f"B{r}"] = f"=CUMIPMT({args})"
    case = {
        "format": "rustsheet-case/v1",
        "description": description,
        "tags": ["M9", "financial", "probe", "T-3524"],
        "expected": {},
        "sheets": [{"name": "S", "cells": cells}],
    }
    path.write_text(json.dumps(case, indent=2) + "\n")


def main() -> None:
    """Both cases (deterministic: rewriting the first leaves it unchanged)."""
    blocks = (
        "rows 1-125 short type 0, 126-250 short type 1, 251-375 long type 0, 376-500 long type 1 "
        "(docs/research/t3524/gen_sweep.py, seed {seed})"
    )
    write(
        OUT,
        SEED,
        (1e-4, 2.0),
        "T-3524 fresh sweep (decision 0188 R4.2): CUMPRINC (A) and CUMIPMT (B) with the leading exponent "
        "(nper-end+1)*ln(1+rate) log-uniform in [1E-3, 40]; " + blocks.format(seed=SEED),
    )
    write(
        TINY_OUT,
        TINY_SEED,
        (1e-12, 1e-4),
        "T-3524 tiny-rate fresh sweep (decision 0188 R4.2, the rates below the first sweep's): CUMPRINC (A) and "
        "CUMIPMT (B), rate log-uniform in [1E-12, 1E-4], the leading exponent log-uniform in [1E-3, 40] where nper "
        "allows (nper - end + 1 = max(1, round(lead/L))); " + blocks.format(seed=TINY_SEED),
    )


if __name__ == "__main__":
    main()
