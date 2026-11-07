"""T-3403 fresh sweep (decision 0188 R4.2), generated after the ungated Newton quantile was frozen
(commit 3fa09266): GAMMA.INV and CHISQ.INV at shapes 1 to 20 and lower-tail p log-uniform on
1E-300..1E-10, and CHISQ.INV.RT at 1-1E-k (k = 10 to 16), the region T-3311's gates excluded
and kept alike. Literals have at most 6 significant digits.

    python3 docs/research/t3403/fresh_sweep.py > eval/suites/probes-numeric/cases/t3403-fresh-sweep.case.json
"""

import json
import random

SEED = 20261109


def sig(v: float, digits: int = 3) -> str:
    return f"{float(f'{v:.{digits - 1}e}'):.{digits}g}"


def main() -> None:
    rng = random.Random(SEED)
    cells: list[tuple[str, str]] = []
    for _ in range(100):
        a = sig(rng.uniform(1.0, 20.0), 4)
        p = sig(10 ** rng.uniform(-300, -10))
        b = rng.choice(["1", "2", "0.5"])
        cells.append((f"=GAMMA.INV({p},{a},{b})", f"gi/a={a}/p={p}/b={b}"))
    for _ in range(60):
        df = rng.randint(3, 39)
        p = sig(10 ** rng.uniform(-300, -10))
        cells.append((f"=CHISQ.INV({p},{df})", f"ci/df={df}/p={p}"))
    for _ in range(20):
        df = rng.randint(3, 39)
        k = rng.randint(10, 16)
        cells.append((f"=CHISQ.INV.RT(1-1E-{k},{df})", f"cir/df={df}/k={k}"))
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-3403 fresh sweep after the ungated Newton quantile was frozen (decision 0188 R4.2):"
            " GAMMA.INV and CHISQ.INV at shapes 1 to 20 and p 1E-300..1E-10, CHISQ.INV.RT at 1-1E-k"
            " (docs/research/t3403/fresh_sweep.py, seed 20261109)."
        ),
        "tags": ["M9", "probe", "statistics", "T-3403", "gamma", "chisq", "quantile", "sweep"],
        "expected": {},
        "hypotheses": {f"Calc!A{i + 1}": h for i, (_, h) in enumerate(cells)},
        "sheets": [{"name": "Calc", "cells": {f"A{i + 1}": f for i, (f, _) in enumerate(cells)}}],
    }
    print(json.dumps(case, indent=1))


if __name__ == "__main__":
    main()
