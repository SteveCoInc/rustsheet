"""Build probes/t4466-chain-fresh, the fresh sweep drawn after the chain count model was frozen (2026-11-08,
decision 0188 R4.2; the sweep that found it, t4466-count-fresh, cannot confirm it): python3 gen_chain_fresh.py
CASE_PATH. Writes chain_fresh_predictions.json (chain, register, rounded) before the fill.

600 cells drawn with random.Random(44663) from gen_fresh.py's distribution (a day, a unit and a whole second just
below that unit's carry, the tie k + 0.95, k + 0.995 or k + 0.9995 under that unit's format), with the double nearest
the tie or 1 or 2 ulps either side.
"""

import math
import random
import sys

from count_models import register, rounded
from caseio import UNIT, draw_tie, tie_expr, write_case


def chain(x, u, s):
    """The frozen model: the half-step serial times 60, 60 and 24, as many factors as the unit needs, truncated."""
    y = x + 0.5 / (86400 * s)
    if u == 3600:
        return math.floor(y * 24)
    if u == 60:
        return math.floor(y * 60 * 24)
    return math.floor(y * 60 * 60 * 24)


DAYS = [0, 0, 1, 2, 3, 7, 45, 200, 365, 1461, 5000, 20000, 44000, 45000, 46000, 80000, 500000, 2900000]
rng = random.Random(44663)
cells, preds = {}, {}
for i in range(1, 601):
    d, secs, f = draw_tie(rng, DAYS)
    off = rng.choice([0, 0, -1, 1, -2, 2])
    expr, x = tie_expr(d, secs, off)
    cell = f"A{i}"
    cells[cell] = f'=TEXT({expr},"{f}")'
    s = 10 ** len(f.split(".")[1])
    u = UNIT[f[1]]
    preds[cell] = {"chain": chain(x, u, s), "register": register(x, u, s), "rounded": rounded(x, u, s)}
write_case(
    sys.argv[1],
    "T-4333: fresh sweep of elapsed counts beside a fraction at decimal ties, drawn after the chain "
    "count model was frozen (docs/research/t4466/gen_chain_fresh.py; predictions in chain_fresh_predictions.json).",
    cells,
    preds,
    "chain_fresh_predictions.json",
)
print(len(cells), "cells;", sum(p["register"] != p["chain"] for p in preds.values()), "where chain and register differ")
