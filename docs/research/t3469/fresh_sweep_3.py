"""T-3469 R4.2 fresh sweep after the direct-deviance form was frozen (decision 0188 R4.2), filled once.

Written after excel_tail.rs's direct-deviance exponent was frozen (a9735e107; unchanged since). Restricted
to the form's own domain: shape a log-uniform on [1.5, 11] rounded to a multiple of 1/2, x log-uniform on
(30, 600] (160 cells); plus cells near its edges: x log-uniform on [27, 34] (the x = 30 cut, 15 cells),
x on [500, 700] (the x = 600 cut, 10 cells), both at shapes 1.5 to 11, and shapes 10.5 to 12.5 at x 30 to
600 (the shape-11 cut, 15 cells). Integer shapes alternate POISSON(a - 1, x, TRUE) and CHISQ.DIST.RT(2x, 2a);
half-integer shapes are CHISQ.DIST.RT. Each x has 6 significant digits (2x is then exact in 15). Seeded:
re-running writes the same case, eval/suites/probes-numeric/cases/t3469-fresh-sweep-3.case.json.

Run (repo root): tools/.venv/bin/python -I docs/research/t3469/fresh_sweep_3.py
"""

import math
import random
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "docs/research/t3269"))
from fresh_sweep import write_case  # noqa: E402

CASE = ROOT / "eval/suites/probes-numeric/cases/t3469-fresh-sweep-3.case.json"
rng = random.Random(34693)


def log_uniform(lo: float, hi: float) -> float:
    return math.exp(rng.uniform(math.log(lo), math.log(hi)))


def shape(lo: float, hi: float) -> float:
    return min(max(round(2.0 * log_uniform(lo, hi)) / 2.0, lo), hi)


plan = (
    [(1.5, 11.0, 30.0, 600.0)] * 160
    + [(1.5, 11.0, 27.0, 34.0)] * 15
    + [(1.5, 11.0, 500.0, 700.0)] * 10
    + [(10.5, 12.5, 30.0, 600.0)] * 15
)
cells, hypotheses = {}, {}
for n, (a_lo, a_hi, x_lo, x_hi) in enumerate(plan, 1):
    a = shape(a_lo, a_hi)
    x = float(f"{log_uniform(x_lo, x_hi):.6g}")
    if a.is_integer() and n % 2:
        formula, label = f"=POISSON({int(a) - 1},{x:g},TRUE)", f"fresh3/poisson/a={a:g}/x={x:g}"
    else:
        formula, label = f"=CHISQ.DIST.RT({2 * x:g},{int(2 * a)})", f"fresh3/chisq/a={a:g}/x={x:g}"
    cells[f"A{n}"] = formula
    hypotheses[f"Calc!A{n}"] = label
write_case(
    CASE,
    "T-3469 fresh log-uniform sweep after the direct-deviance far-tail form was frozen (decision 0188 R4.2): "
    "Q(a, x) by POISSON(a-1,x,TRUE) or CHISQ.DIST.RT(2x,2a), a log-uniform on 1.5..11 (multiples of 1/2), "
    "x log-uniform on 30..600, plus 15 cells at x 27..34, 10 at x 500..700 and 15 at shapes 10.5..12.5 "
    "(200 cells; docs/research/t3469/fresh_sweep_3.py).",
    cells,
    hypotheses,
    task="T-3469",
)
print(len(cells))
