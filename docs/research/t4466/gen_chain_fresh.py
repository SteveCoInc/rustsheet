"""Build probes/t4466-chain-fresh, the fresh sweep drawn after the chain count model was frozen (2026-11-08,
decision 0188 R4.2; the sweep that found it, t4466-count-fresh, cannot confirm it): python3 gen_chain_fresh.py
CASE_PATH. Writes chain_fresh_predictions.json (chain, register, rounded) before the fill.

600 cells drawn with random.Random(44663) from gen_fresh.py's distribution (a day, a unit and a whole second just
below that unit's carry, the tie k + 0.95, k + 0.995 or k + 0.9995 under that unit's format), with the double nearest
the tie or 1 or 2 ulps either side.
"""

import json
import math
import os
import random
import sys

from count_models import register, rounded

HERE = os.path.dirname(os.path.abspath(__file__))


def chain(x, u, s):
    """The frozen model: the half-step serial times 60, 60 and 24, as many factors as the unit needs, truncated."""
    y = x + 0.5 / (86400 * s)
    if u == 3600:
        return math.floor(y * 24)
    if u == 60:
        return math.floor(y * 60 * 24)
    return math.floor(y * 60 * 60 * 24)


DAYS = [0, 0, 1, 2, 3, 7, 45, 200, 365, 1461, 5000, 20000, 44000, 45000, 46000, 80000, 500000, 2900000]
FORMATS = {
    ".95": ["[s].0", "[m]:ss.0", "[h]:mm:ss.0"],
    ".995": ["[s].00", "[m]:ss.00", "[h]:mm:ss.00"],
    ".9995": ["[s].000", "[m]:ss.000", "[h]:mm:ss.000"],
}
UNIT = {"s": 1, "m": 60, "h": 3600}
rng = random.Random(44663)
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
    f = FORMATS[tail][{"s": 0, "m": 1, "h": 2}[unit]]
    secs = f"{k}{tail}"
    base = (d + float(secs) / 86400) if d else float(secs) / 86400
    e = round(math.log2(math.ulp(base)))
    off = rng.choice([0, 0, -1, 1, -2, 2])
    expr = (f"{d}+{secs}/86400" if d else f"{secs}/86400") + ("" if off == 0 else f"{off:+d}*2^{e}")
    cell = f"A{i}"
    cells[cell] = f'=TEXT({expr},"{f}")'
    x = base + off * 2.0**e if off else base
    s = 10 ** len(f.split(".")[1])
    u = UNIT[f[1]]
    preds[cell] = {"chain": chain(x, u, s), "register": register(x, u, s), "rounded": rounded(x, u, s)}
case = {
    "format": "rustsheet-case/v1",
    "description": "T-4333: fresh sweep of elapsed counts beside a fraction at decimal ties, drawn after the chain "
    "count model was frozen (docs/research/t4466/gen_chain_fresh.py; predictions in chain_fresh_predictions.json).",
    "tags": ["M10", "text", "probe"],
    "sheets": [{"name": "Sheet1", "cells": cells}],
    "expected": {},
}
with open(sys.argv[1], "w") as fh:
    fh.write(json.dumps(case, indent=2) + "\n")
with open(os.path.join(HERE, "chain_fresh_predictions.json"), "w") as fh:
    json.dump(preds, fh, indent=0, sort_keys=True)
print(len(cells), "cells;", sum(p["register"] != p["chain"] for p in preds.values()), "where chain and register differ")
