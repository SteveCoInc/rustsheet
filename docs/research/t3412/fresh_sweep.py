"""T-3412 fresh sweeps (decision 0188 R4.2), generated after the forms were frozen (commit 5d8b1b79).

Lower region below shape 20: GAMMA.DIST and CHISQ.DIST cdf at x/scale up to 1 (the leading-factor
form), CHISQ.DIST.RT and the POISSON cdf below x = a + 1 (the (0.5 - P) + 0.5 complement), plus
edge cells just past each cut (x/scale in (1, 1.5], shapes in (19, 20) and (1, 1.2)). Every literal
has at most 6 significant digits, so Excel's 15-digit cut of a literal does not move it.

    python3 docs/research/t3412/fresh_sweep.py > eval/suites/probes-numeric/cases/t3412-fresh-sweep.case.json
    python3 docs/research/t3412/fresh_sweep.py 2 > eval/suites/probes-numeric/cases/t3412-fresh-sweep-2.case.json

Sweep 2 (seed 20261108), after the form was widened to every shape above 1 and frozen (commit
56bf7cc5): GAMMA.DIST and CHISQ.DIST cdf at shapes 20 to 170, x/scale up
to 1, plus 20 cells just past x/scale = 1 there.
"""

import json
import random
import sys

SEED = 20261107


def sig(v: float, digits: int = 6) -> str:
    return f"{float(f'{v:.{digits - 1}e}'):.{digits}g}"


def sweep_2() -> list[tuple[str, str]]:
    rng = random.Random(20261108)
    cells: list[tuple[str, str]] = []
    for _ in range(60):
        a = sig(10 ** rng.uniform(1.30103, 2.2304), 4)
        b = rng.choice(["1", "2", "0.25"])
        y = 10 ** rng.uniform(-3, 0)
        cells.append((f"=GAMMA.DIST({sig(y * float(b))},{a},{b},TRUE)", f"gd/a={a}/b={b}"))
    for _ in range(30):
        df = rng.randint(40, 340)
        cells.append((f"=CHISQ.DIST({sig(2 * 10 ** rng.uniform(-2, 0))},{df},TRUE)", f"cd/df={df}"))
    for _ in range(20):
        a = sig(10 ** rng.uniform(1.30103, 2.2304), 4)
        cells.append((f"=GAMMA.DIST({sig(rng.uniform(1.0, 1.5))},{a},1,TRUE)", f"edge/x>1/a={a}"))
    return cells


def main() -> None:
    if sys.argv[1:] == ["2"]:
        emit(sweep_2(), "T-3412 second fresh sweep after the leading-factor form was widened to every shape above 1"
             " and frozen (decision 0188 R4.2): GAMMA.DIST and CHISQ.DIST cdf at shapes 20 to 170, x/scale up to 1,"
             " and 20 cells past x/scale = 1 (docs/research/t3412/fresh_sweep.py 2, seed 20261108).")
        return
    rng = random.Random(SEED)
    cells: list[tuple[str, str]] = []
    for _ in range(90):
        a = sig(rng.uniform(1.0, 20.0), 4)
        b = rng.choice(["1", "2", "0.5", "3.7"])
        y = 10 ** rng.uniform(-12, 0)
        cells.append((f"=GAMMA.DIST({sig(y * float(b))},{a},{b},TRUE)", f"gd/a={a}/b={b}"))
    for _ in range(50):
        df = rng.randint(3, 39)
        cells.append((f"=CHISQ.DIST({sig(2 * 10 ** rng.uniform(-8, 0))},{df},TRUE)", f"cd/df={df}"))
    for i in range(60):
        df = rng.randint(3, 39)
        x = 2 * 10 ** rng.uniform(-4, 0) if i % 2 == 0 else rng.uniform(2.0, df + 2.0)
        cells.append((f"=CHISQ.DIST.RT({sig(x)},{df})", f"cr/df={df}"))
    for i in range(50):
        k = rng.randint(1, 18)
        mean = 10 ** rng.uniform(-3, 0) if i % 2 == 0 else rng.uniform(1.0, k + 2.0)
        cells.append((f"=POISSON({k},{sig(mean)},TRUE)", f"po/k={k}"))
    for _ in range(10):
        a = sig(rng.uniform(1.0, 20.0), 4)
        cells.append((f"=GAMMA.DIST({sig(rng.uniform(1.0, 1.5))},{a},1,TRUE)", f"edge/x>1/a={a}"))
    for _ in range(10):
        a = sig(rng.uniform(19.0, 20.0), 4)
        cells.append((f"=GAMMA.DIST({sig(10 ** rng.uniform(-6, 0))},{a},1,TRUE)", f"edge/a<20/a={a}"))
    for _ in range(10):
        a = sig(rng.uniform(1.0, 1.2), 4)
        cells.append((f"=GAMMA.DIST({sig(10 ** rng.uniform(-6, 0))},{a},1,TRUE)", f"edge/a>1/a={a}"))
    emit(cells, (
        "T-3412 fresh sweep after the lower-region forms were frozen (decision 0188 R4.2): GAMMA.DIST and"
        " CHISQ.DIST cdf at x/scale up to 1 and shape 1 to 20, CHISQ.DIST.RT and the POISSON cdf below"
        " x = a + 1, and edge cells past each cut (docs/research/t3412/fresh_sweep.py, seed 20261107)."
    ))


def emit(cells: list[tuple[str, str]], description: str) -> None:
    case = {
        "format": "rustsheet-case/v1",
        "description": description,
        "tags": ["M9", "probe", "statistics", "T-3412", "gamma", "chisq", "poisson", "sweep"],
        "expected": {},
        "hypotheses": {f"Calc!A{i + 1}": h for i, (_, h) in enumerate(cells)},
        "sheets": [{"name": "Calc", "cells": {f"A{i + 1}": f for i, (f, _) in enumerate(cells)}}],
    }
    print(json.dumps(case, indent=1))


if __name__ == "__main__":
    main()
