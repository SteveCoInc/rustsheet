"""Tie sweeps of T-4323: python3 sweep.py SEED COUNT SEPARATING CASE_PATH META_PATH DESCRIPTION.

SEPARATING is how many of the first cells must be ones where the candidate models disagree
(sweep 2: seed 43232, 160 cells, 120 separating; sweep 3: seed 43233, 120, 0; hold-out: 43234, 100, 0).
"""
import json
import math
import random
import sys

from eps import SN, model
from frozen_fmt import fmt

seed, count, separating = (int(a) for a in sys.argv[1:4])
case_path, meta_path, description = sys.argv[4:7]
random.seed(seed)
FMTS = [("[h]:mm.0", 10), ("h:mm:ss.0", 10), ("[s].0", 10), ("s.00", 100), ("[h]:mm:ss.000", 1000),
        ("[m]:ss.00", 100), ("mm:ss.0", 10), ("h:mm:ss.000", 1000)]
DAYS = [0, 0, 1, 2, 3, 7, 30, 365, 1000, 20000, 45000, 60000, 2958000] if separating else \
       [0, 1, 2, 3, 5, 7, 30, 365, 1000, 20000, 45000, 60000, 2958000]
cells, meta, r = {}, {}, 1
while r <= count:
    f, s = random.choice(FMTS)
    d = random.choice(DAYS)
    sec = 10 ** random.uniform(-3, math.log10(86399))
    sec = float("%.7g" % ((math.floor(sec * s) + 0.5) / s))
    x = (d + sec / 86400) if d else sec / 86400
    outs = {fmt(f, s, model(x, s, sn)) for sn in SN.values()} | {fmt(f, s, model(x, s, lambda v: v))}
    if r <= separating and len(outs) == 1:
        continue
    dpart = f"{d}+" if d else ""
    cells[f"A{r}"] = f'=TEXT({dpart}{sec:.7g}/86400,"{f}")'
    meta[f"A{r}"] = (f, s, d, float(f"{sec:.7g}"))
    r += 1
case = {"format": "rustsheet-case/v1", "description": description, "tags": ["M10", "text", "probe"],
        "sheets": [{"name": "Sheet1", "cells": cells}], "expected": {}}
open(case_path, "w").write(json.dumps(case, indent=2) + "\n")
json.dump(meta, open(meta_path, "w"))
