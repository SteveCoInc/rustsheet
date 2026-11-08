"""Build probes/t4466-count-fresh, the fresh sweep drawn after count_models.py's `max` was frozen (2026-11-08,
decision 0188 R4.2): python3 gen_fresh.py CASE_PATH. Also writes fresh_predictions.json (register and max, before the
fill).

600 cells drawn with random.Random(44661): a day from a log-spread list, a unit (`[s]` three times in five, `[m]`,
`[h]`) with a whole second k just below that unit's carry (any second, the last second of a minute, the last second
of an hour), the tie k + 0.95, k + 0.995 or k + 0.9995 under that unit's format with its own digits, and the double
nearest the tie (two times in three) or one ulp either side.
"""

import random
import sys

from count_models import MODELS
from caseio import UNIT, draw_tie, tie_expr, write_case

DAYS = [0, 0, 1, 2, 4, 9, 31, 100, 366, 1000, 9999, 30000, 45001, 50000, 65535, 70000, 400000, 2958000]
rng = random.Random(44661)
cells, preds = {}, {}
for i in range(1, 601):
    d, secs, f = draw_tie(rng, DAYS)
    off = rng.choice([0, 0, 0, 0, -1, 1])
    expr, x = tie_expr(d, secs, off)
    cell = f"A{i}"
    cells[cell] = f'=TEXT({expr},"{f}")'
    s = 10 ** len(f.split(".")[1])
    u = UNIT[f[1]]
    preds[cell] = {name: fn(x, u, s) for name, fn in MODELS.items()}
write_case(
    sys.argv[1],
    "T-4333: fresh sweep of elapsed counts beside a fraction at decimal ties, drawn after the count "
    "model was frozen (docs/research/t4466/gen_fresh.py).",
    cells,
    preds,
    "fresh_predictions.json",
)
print(len(cells), "cells;", sum(p["register"] != p["max"] for p in preds.values()), "where the models differ")
