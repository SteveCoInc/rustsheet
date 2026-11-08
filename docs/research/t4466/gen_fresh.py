"""Build probes/t4466-count-fresh, the fresh sweep drawn after count_models.py's `max` was frozen (2026-11-08,
decision 0188 R4.2): python3 gen_fresh.py CASE_PATH. Also writes fresh_predictions.json (register and max, before the
fill).

600 cells drawn with random.Random(44661): a day from a log-spread list, a unit (`[s]` three times in five, `[m]`,
`[h]`) with a whole second k just below that unit's carry (any second, the last second of a minute, the last second
of an hour), the tie k + 0.95, k + 0.995 or k + 0.9995 under that unit's format with its own digits, and the double
nearest the tie (two times in three) or one ulp either side.
"""

import json
import math
import os
import random
import sys

from count_models import MODELS

HERE = os.path.dirname(os.path.abspath(__file__))
DAYS = [0, 0, 1, 2, 4, 9, 31, 100, 366, 1000, 9999, 30000, 45001, 50000, 65535, 70000, 400000, 2958000]
FORMATS = {
    ".95": ["[s].0", "[m]:ss.0", "[h]:mm:ss.0"],
    ".995": ["[s].00", "[m]:ss.00", "[h]:mm:ss.00"],
    ".9995": ["[s].000", "[m]:ss.000", "[h]:mm:ss.000"],
}
UNIT = {"s": 1, "m": 60, "h": 3600}
rng = random.Random(44661)
cells, preds = {}, {}
for i in range(1, 601):
    d = rng.choice(DAYS)
    unit = rng.choice("sssmh")
    if unit == "s":
        k = rng.randrange(0, 86399)
    elif unit == "m":
        k = rng.randrange(0, 1440) * 60 + 59
    else:
        k = rng.randrange(0, 24) * 3600 + 3599
    if d == 0 and k == 86399:
        k = 86398
    tail = rng.choice(list(FORMATS))
    f = {"s": 0, "m": 1, "h": 2}[unit]
    f = FORMATS[tail][f]
    secs = f"{k}{tail}"
    base = (d + float(secs) / 86400) if d else float(secs) / 86400
    e = round(math.log2(math.ulp(base)))
    off = rng.choice([0, 0, 0, 0, -1, 1])
    expr = (f"{d}+{secs}/86400" if d else f"{secs}/86400") + ("" if off == 0 else f"{off:+d}*2^{e}")
    cell = f"A{i}"
    cells[cell] = f'=TEXT({expr},"{f}")'
    x = base + off * 2.0**e if off else base
    s = 10 ** len(f.split(".")[1])
    u = UNIT[f[1]]
    preds[cell] = {name: fn(x, u, s) for name, fn in MODELS.items()}
case = {
    "format": "rustsheet-case/v1",
    "description": "T-4333: fresh sweep of elapsed counts beside a fraction at decimal ties, drawn after the count "
    "model was frozen (docs/research/t4466/gen_fresh.py).",
    "tags": ["M10", "text", "probe"],
    "sheets": [{"name": "Sheet1", "cells": cells}],
    "expected": {},
}
with open(sys.argv[1], "w") as fh:
    fh.write(json.dumps(case, indent=2) + "\n")
with open(os.path.join(HERE, "fresh_predictions.json"), "w") as fh:
    json.dump(preds, fh, indent=0, sort_keys=True)
print(len(cells), "cells;", sum(p["register"] != p["max"] for p in preds.values()), "where the models differ")
