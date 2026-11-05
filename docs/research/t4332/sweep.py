"""T-4332's fresh sweep, generated after chain.py was frozen: python3 sweep.py SEED COUNT SEPARATING CASE_PATH META_PATH.

Rounding ties of the shown fraction of a second (sec = (n + 0.5)/s, log-uniform from 1E-3 s), 3 of 4 at a serial below
1 and the rest past whole days. The first SEPARATING cells are ones where chain.py's model and T-4323's register print
different text. The fresh sweep is seed 43320, 120 cells, 60 separating.
"""
import json
import math
import random
import sys

from chain import as_fields, chain_units, fmt, register_units

seed, count, separating = (int(a) for a in sys.argv[1:4])
case_path, meta_path = sys.argv[4:6]
random.seed(seed)
FMTS = [("[h]:mm.0", 10), ("h:mm:ss.0", 10), ("[s].0", 10), ("s.00", 100), ("[h]:mm:ss.000", 1000),
        ("[m]:ss.00", 100), ("mm:ss.0", 10), ("h:mm:ss.000", 1000)]
DAYS = [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 7, 45000]
cells, meta, r = {}, {}, 1
while r <= count:
    f, s = random.choice(FMTS)
    d = random.choice(DAYS)
    sec = 10 ** random.uniform(-3, math.log10(86399))
    sec = float("%.7g" % ((math.floor(sec * s) + 0.5) / s))
    x = (d + sec / 86400) if d else sec / 86400
    outs = {fmt(f, s, as_fields(m)(x, s)) for m in (chain_units, register_units)}
    if r <= separating and len(outs) == 1:
        continue
    dpart = f"{d}+" if d else ""
    cells[f"A{r}"] = f'=TEXT({dpart}{sec:.7g}/86400,"{f}")'
    meta[f"A{r}"] = (f, s, d, float(f"{sec:.7g}"))
    r += 1
description = ("T-4332 fresh sweep (generated after the field-by-field model of docs/research/t4332/chain.py was "
               "frozen): rounding ties of the shown fraction of a second, mostly at serials below 1, under elapsed "
               "and clock formats with .0, .00 and .000; the first 60 cells separate that model from T-4323's register.")
case = {"format": "rustsheet-case/v1", "description": description, "tags": ["M10", "text", "probe"],
        "sheets": [{"name": "Sheet1", "cells": cells}], "expected": {}}
open(case_path, "w").write(json.dumps(case, indent=2) + "\n")
json.dump(meta, open(meta_path, "w"))
